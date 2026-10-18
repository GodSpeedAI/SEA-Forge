# Runs INSIDE the workload. (1) symlink planted in workspace; (2) connect to another identity's MCP socket.
import os, json, socket, errno
out = {}
ws = os.getcwd()
try:
    os.symlink("/home/sprime01/.gitconfig", os.path.join(ws, "planted_link"))
    out["plant_symlink"] = "ok"
except OSError as e: out["plant_symlink"] = errno.errorcode.get(e.errno, e.errno)
try:
    out["read_via_link_size"] = len(open(os.path.join(ws, "planted_link"), "rb").read())
except OSError as e: out["read_via_link_size"] = errno.errorcode.get(e.errno, e.errno)
try:
    os.symlink("/home/sprime01", os.path.join(ws, ".agent-harness"))
    out["agent_harness_symlink_is_dir"] = os.path.isdir(os.path.join(ws, ".agent-harness"))
    out["agent_harness_entries"] = len(os.listdir(os.path.join(ws, ".agent-harness")))
except OSError as e: out["agent_harness"] = errno.errorcode.get(e.errno, e.errno)
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
try:
    s.connect(os.environ["ADV_OTHER_MCP"]); out["connect_other_mcp"] = "connected"
except OSError as e: out["connect_other_mcp"] = errno.errorcode.get(e.errno, e.errno)
print(json.dumps(out))
