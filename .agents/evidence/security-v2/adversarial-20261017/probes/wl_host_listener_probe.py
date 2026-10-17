# Runs INSIDE the workload: try to reach host-side listeners started by the driver (abstract unix + TCP loopback).
import socket, json, errno, os
res = {}
def tryit(name, fn):
    try: res[name] = fn()
    except OSError as e: res[name] = errno.errorcode.get(e.errno, e.errno)
def abstract():
    s = socket.socket(socket.AF_UNIX); s.settimeout(3); s.connect("\0adv-host-listener-abstract"); return "connected"
def tcp():
    s = socket.create_connection(("127.0.0.1", 38999), timeout=3); return "connected"
tryit("abstract_host_listener", abstract)
tryit("tcp_loopback_host_listener", tcp)
print(json.dumps(res))
