"""Kernel check: does a USER_NOTIF filter on connect(2) actually trap it?

Same BPF shape as socket_policy.rs. Prints one verdict line; used by the
kernel-security preflight to separate kernel behavior from SEA-Forge code."""
import ctypes, socket, struct, threading

libc = ctypes.CDLL(None, use_errno=True)
libc.syscall.restype = ctypes.c_long
NOTIF, ALLOW = 0x7FC00000, 0x7FFF0000
X32 = 0x40000000
MON = [42, 44, 46, 307]


def insn(code, jt, jf, k):
    return struct.pack("<HBBI", code, jt, jf, k)


nrs = [n for m in MON for n in (m, m + X32)]
prog = [insn(0x20, 0, 0, 4), insn(0x15, 1, 0, 0xC000003E), insn(0x06, 0, 0, ALLOW)]
for n in nrs:
    prog += [insn(0x20, 0, 0, 0), insn(0x15, 0, 1, n), insn(0x06, 0, 0, NOTIF)]
prog.append(insn(0x06, 0, 0, ALLOW))
buf = ctypes.create_string_buffer(b"".join(prog))


class FProg(ctypes.Structure):
    _fields_ = [("len", ctypes.c_ushort), ("filter", ctypes.c_void_p)]


fp = FProg(len(prog), ctypes.cast(buf, ctypes.c_void_p).value)
box = {}
ready = threading.Event()
got = threading.Event()


def supervisor():
    ready.wait()
    fd = box["fd"]
    req = ctypes.create_string_buffer(80)
    r = libc.ioctl(fd, ctypes.c_ulong(0xC0502100), req)
    if r < 0:
        print("RECV error", ctypes.get_errno()); return
    id_, pid, flags, nr = struct.unpack_from("<QIIi", req, 0)
    print("RECV ok nr=%d" % nr); got.set()
    resp = ctypes.create_string_buffer(struct.pack("<QqiI", id_, 0, -13, 0))
    libc.ioctl(fd, ctypes.c_ulong(0xC0182101), resp)


def worker():
    libc.prctl(38, 1, 0, 0, 0)
    fd = libc.syscall(317, 1, 8, ctypes.byref(fp))
    print("filter install ->", fd, "errno", ctypes.get_errno())
    box["fd"] = fd
    ready.set()
    s = socket.socket(socket.AF_UNIX)
    try:
        s.connect("/nonexistent-probe/x.sock"); print("connect: ok?!")
    except OSError as e:
        print("connect errno", e.errno, "(13 = trapped+denied by broker, 2 = untrapped)")


t = threading.Thread(target=supervisor, daemon=True)
t.start()
w = threading.Thread(target=worker)
w.start()
w.join(8)
print("worker finished" if not w.is_alive() else "worker HUNG (trapped, unanswered?)", "| notif seen:", got.is_set())
