# Host-side control (launcher context, not isolated): proves the abstract listener is live; then slot exclusivity.
import socket, os, fcntl, sys, time, subprocess
P = sys.argv[1]
a = socket.socket(socket.AF_UNIX); a.bind("\0adv-host-listener-abstract"); a.listen(4)
t = socket.socket(socket.AF_INET); t.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1); t.bind(("127.0.0.1", 38999)); t.listen(4)
c = socket.socket(socket.AF_UNIX); c.settimeout(3); c.connect("\0adv-host-listener-abstract"); print("host_control_abstract: connected (listener live)")
c.close()
probe = subprocess.run([sys.executable, "-I", os.path.join(os.path.dirname(os.path.abspath(__file__)), "wl_host_listener_probe.py")], capture_output=True, text=True, timeout=30)
print("host_control_tcp_connect:", end=" ")
try:
    s = socket.create_connection(("127.0.0.1", 38999), timeout=3); print("connected"); s.close()
except OSError as e: print("refused", e)
a.close(); t.close()
fd1 = os.open(P + "/state/workload-slots/3.lock", os.O_CREAT | os.O_RDWR, 0o600); fcntl.flock(fd1, fcntl.LOCK_EX | fcntl.LOCK_NB)
fd2 = os.open(P + "/state/workload-slots/3.lock", os.O_RDWR)
try: fcntl.flock(fd2, fcntl.LOCK_EX | fcntl.LOCK_NB); print("slot3 second acquire: SUCCEEDED (bad)")
except BlockingIOError: print("slot3 second acquire: refused while held (HELD)")
os.close(fd1); os.close(fd2)
