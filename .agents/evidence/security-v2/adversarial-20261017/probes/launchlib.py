# Hand-reproduced isolated-workload launch, mirroring crates/sea-forge-sandbox/src/workload.rs
# isolated_command() + sea-forge-workload-exec. Run with python3 -I.
import fcntl, os, struct, subprocess, sys

UNSHARE = "/usr/bin/unshare"
HELPER = "/home/sprime01/projects/sea-rs/target/debug/sea-forge-workload-exec"
UID_BASE, GID_BASE = 100000, 100000
INNER = 1000
ACL_USER_OBJ, ACL_USER, ACL_GROUP_OBJ, ACL_MASK, ACL_OTHER = 1, 2, 4, 0x10, 0x20
UNDEF = 0xFFFFFFFF

def encode_acl(users):
    entries = [(ACL_USER_OBJ, 7, UNDEF)] + [(ACL_USER, 7, u) for u in sorted(set(users))]
    entries += [(ACL_GROUP_OBJ, 0, UNDEF), (ACL_MASK, 7, UNDEF), (ACL_OTHER, 0, UNDEF)]
    out = struct.pack("<I", 2)
    for tag, perm, ident in entries:
        out += struct.pack("<HHI", tag, perm, ident)
    return out

def grant(root, host_uid, launcher_uid):
    """Mirror grant_exclusive_access: access ACL for identity; default ACL for dirs."""
    access = encode_acl([host_uid])
    default = encode_acl([host_uid, launcher_uid])
    stack = [root]
    while stack:
        p = stack.pop()
        st = os.lstat(p)
        os.setxattr(p, "system.posix_acl_access", access)
        if os.path.isdir(p) and not os.path.islink(p):
            os.setxattr(p, "system.posix_acl_default", default)
            stack += [os.path.join(p, e) for e in os.listdir(p)]

def acquire_slot(state, slot):
    d = os.path.join(state, "workload-slots")
    os.makedirs(d, exist_ok=True)
    fd = os.open(os.path.join(d, f"{slot}.lock"), os.O_CREAT | os.O_RDWR, 0o600)
    fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)  # raises if held
    return fd

def launch_cmd(host_uid, host_gid, helper, ws, art, argv, net_denied=True, ports=()):
    cmd = [UNSHARE, "--user",
           f"--map-users={INNER}:{host_uid}:1", f"--map-groups={INNER}:{host_gid}:1",
           f"--setuid={INNER}", f"--setgid={INNER}",
           "--pid", "--fork", "--kill-child", "--mount", "--mount-proc", "--ipc"]
    if net_denied:
        cmd.append("--net")
    cmd += ["--", helper, "--workspace", ws, "--artifacts", art]
    for p in ports:
        cmd += ["--tcp-port", str(p)]
    cmd += ["--"] + list(argv)
    return cmd

def run(argv, ws, art, slot=1, timeout=30, net_denied=True, ports=(), stdin=None):
    host_uid = UID_BASE + slot
    host_gid = GID_BASE + slot
    cmd = launch_cmd(host_uid, host_gid, HELPER, ws, art, argv, net_denied, ports)
    return subprocess.run(cmd, cwd=ws, env={"PATH": "/usr/bin:/bin"}, capture_output=True,
                          text=True, timeout=timeout, input=stdin)

if __name__ == "__main__":
    print(" ".join(launch_cmd(100001, 100001, HELPER, "/WS", "/ART", ["/bin/true"])))
