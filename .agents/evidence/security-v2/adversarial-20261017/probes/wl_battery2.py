# Runs INSIDE the isolated workload (python3 -I). One JSON line per check; no secret content printed.
import json, os, socket, errno, ctypes, subprocess, glob
out = []
def rec(name, fn):
    try:
        out.append({"check": name, "ok": True, "detail": str(fn())[:300]})
    except OSError as e:
        out.append({"check": name, "ok": False, "errno": errno.errorcode.get(e.errno, e.errno), "detail": str(e)[:160]})
    except Exception as e:
        out.append({"check": name, "ok": False, "exc": type(e).__name__, "detail": str(e)[:160]})
def sh(cmd):
    p = subprocess.run(["/bin/sh", "-c", cmd], capture_output=True, text=True, timeout=20)
    return f"rc={p.returncode} out={p.stdout.strip()[:120]} err={p.stderr.strip()[:120]}"
P = os.environ.get("ADV_PROBE_ROOT", "/home/sprime01/.adv-probe-20261017")
OTHER_WS = P + "/ws2"; OTHER_ART = P + "/art2"
rec("cross_read_other_ws", lambda: open(OTHER_WS + "/secret_b.txt").read()[:0] or "readable")
rec("cross_list_other_ws", lambda: len(os.listdir(OTHER_WS)))
rec("cross_list_other_art", lambda: len(os.listdir(OTHER_ART)))
rec("cross_write_other_ws", lambda: open(OTHER_WS + "/planted", "w").write("x") and "wrote")
rec("state_dir_list", lambda: os.listdir(P + "/state"))
rec("slot_locks_list", lambda: os.listdir(P + "/state/workload-slots"))
rec("state_slot_lock_read", lambda: open(P + "/state/workload-slots/2.lock", "rb").read(1) and "readable")
rec("nested_userns_create", lambda: sh("unshare -U true; echo \"nested_rc=$?\""))
rec("nested_userns_mount_tmpfs_home", lambda: sh("unshare -Urm sh -c 'mount -t tmpfs none /home/sprime01 && echo mounted' ; echo rc=$?"))
rec("nested_netns_unshare", lambda: sh("unshare -rn sh -c 'ip link 2>/dev/null | head -2; cat /proc/net/route | wc -l'"))
rec("setuid_sudo", lambda: sh("command -v sudo && sudo -n id; echo rc=$?"))
rec("setuid_newuidmap_exec", lambda: sh("/usr/bin/newuidmap 2>&1 | head -2; echo rc=$?"))
rec("proc_sys_write_kernel", lambda: open("/proc/sys/kernel/domainname", "w").write("x") and "wrote")
rec("proc_sys_read_userns", lambda: open("/proc/sys/user/max_user_namespaces").read().strip())
libc = ctypes.CDLL(None, use_errno=True)
def io_uring_setup():
    # SYS_io_uring_setup = 425 on x86_64; struct io_uring_params zeroed (120 bytes)
    params = ctypes.create_string_buffer(120)
    fd = libc.syscall(425, 4, params)
    if fd < 0: raise OSError(ctypes.get_errno(), os.strerror(ctypes.get_errno()))
    os.close(fd); return "io_uring ring created"
rec("io_uring_setup", io_uring_setup)
def userfaultfd():
    fd = libc.syscall(323, 0)  # SYS_userfaultfd on x86_64
    if fd < 0: raise OSError(ctypes.get_errno(), os.strerror(ctypes.get_errno()))
    os.close(fd); return "userfaultfd created"
rec("userfaultfd", userfaultfd)
def ptrace_parent():
    PTRACE_ATTACH = 16
    ppid = os.getppid()
    r = libc.ptrace(PTRACE_ATTACH, ppid, 0, 0)
    if r != 0: raise OSError(ctypes.get_errno(), "ptrace attach ppid: " + os.strerror(ctypes.get_errno()))
    return "attached parent"
rec("ptrace_parent_attach", ptrace_parent)
def open_fds():
    res = []
    for fd in sorted(os.listdir("/proc/self/fd"), key=int):
        try: res.append(f"{fd}->{os.readlink('/proc/self/fd/'+fd)[:60]}")
        except OSError: pass
    return res
rec("inherited_fds", open_fds)
rec("mount_bind_attempt", lambda: sh("mount --bind /tmp /home/sprime01 2>&1; echo rc=$?"))
rec("etc_shadow_read", lambda: open("/etc/shadow").read()[:0] or "readable")
rec("root_owned_ro_read_etc_sudoers", lambda: sh("test -r /etc/sudoers && echo readable || echo unreadable"))
rec("write_proc_self_landlock_fd", lambda: os.stat("/proc/self/exe").st_mode and "stat ok")
print(json.dumps(out))
