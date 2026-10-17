# Runs INSIDE the isolated workload (python3 -I). Prints one JSON line per check.
# Records outcome and errno only; never prints file contents of sensitive files.
import json, os, socket, hashlib, errno, sys, glob
out = []
def rec(name, fn):
    try:
        r = fn()
        out.append({"check": name, "ok": True, "detail": str(r)[:300]})
    except OSError as e:
        out.append({"check": name, "ok": False, "errno": errno.errorcode.get(e.errno, e.errno), "detail": str(e)[:160]})
    except Exception as e:
        out.append({"check": name, "ok": False, "exc": type(e).__name__, "detail": str(e)[:160]})

def write_probe(path):
    with open(path, "w") as f: f.write("x")
    os.unlink(path); return "wrote"
def read_hash(path):
    with open(path, "rb") as f: data = f.read(1 << 20)
    return f"size={len(data)} sha256={hashlib.sha256(data).hexdigest()[:12]}"
def list_dir(path):
    return f"entries={len(os.listdir(path))}"
def stat_mode(path):
    st = os.stat(path); return f"uid={st.st_uid} gid={st.st_gid} mode={oct(st.st_mode & 0o7777)}"
H = "/home/sprime01"
rec("identity", lambda: f"uid={os.getuid()} gid={os.getgid()} groups={os.getgroups()}")
rec("write_etc_passwd", lambda: write_probe("/etc/adv_probe"))
rec("write_home", lambda: write_probe(H + "/adv_probe_write"))
rec("write_tmp", lambda: write_probe("/tmp/adv_probe_write"))
rec("write_devshm", lambda: write_probe("/dev/shm/adv_probe_write"))
rec("write_var_tmp", lambda: write_probe("/var/tmp/adv_probe_write"))
rec("write_workspace", lambda: write_probe(os.path.join(os.getcwd(), "adv_probe_ws")))
rec("write_artifacts", lambda: write_probe(os.environ.get("ADV_ART", "/nonexistent") + "/x") if False else "skipped")
rec("write_dev_null", lambda: open("/dev/null", "w").write("x") and "ok")
rec("stat_gitconfig", lambda: stat_mode(H + "/.gitconfig"))
rec("read_gitconfig_hash", lambda: read_hash(H + "/.gitconfig"))
rec("list_ssh_dir", lambda: list_dir(H + "/.ssh"))
rec("read_bash_history", lambda: read_hash(H + "/.bash_history"))
rec("read_npmrc", lambda: read_hash(H + "/.npmrc"))
rec("read_claude_json", lambda: read_hash(H + "/.claude.json"))
rec("list_run_user_1000", lambda: list_dir("/run/user/1000"))
rec("list_mnt_c", lambda: list_dir("/mnt/c"))
rec("list_docker_sock", lambda: stat_mode("/var/run/docker.sock"))
rec("list_pg_socket_dir", lambda: stat_mode("/var/run/postgresql"))
rec("proc_pids_visible", lambda: sorted(p for p in os.listdir("/proc") if p.isdigit()))
rec("proc_mountinfo_lines", lambda: len(open("/proc/self/mountinfo").read().splitlines()))
rec("proc_1_cmdline", lambda: open("/proc/1/cmdline").read().replace("\0"," ")[:120])
rec("proc_sys_io_uring_disabled", lambda: open("/proc/sys/kernel/io_uring_disabled").read().strip())
rec("nonewprivs", lambda: [l for l in open("/proc/self/status") if l.startswith(("NoNewPrivs","Seccomp:","CapEff","CapBnd"))])
def abstract(name):
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.settimeout(2)
    s.connect("\0" + name); return "connected"
rec("abstract_x11", lambda: abstract("/tmp/.X11-unix/X0"))
rec("abstract_dbus_probe", lambda: abstract("/tmp/dbus-probe-none"))
def tcp(host, port):
    s = socket.create_connection((host, port), timeout=2); s.close(); return "connected"
rec("tcp_loopback_5432", lambda: tcp("127.0.0.1", 5432))
rec("tcp_external_443", lambda: tcp("1.1.1.1", 443))
def udp(host, port):
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM); s.connect((host, port)); s.send(b"adv"); s.close(); return "sent"
rec("udp_external_53", lambda: udp("1.1.1.1", 53))
rec("net_ifaces", lambda: [i for _, i in socket.if_nameindex()])
print(json.dumps(out))
