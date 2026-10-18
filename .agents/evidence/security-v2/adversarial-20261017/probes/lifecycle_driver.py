# Lifecycle probes run from the HOST side (python3 -I). Launches the hand-reproduced isolated command.
#   mode "detach": workload double-forks + setsid a sleeper, then the episode returns normally.
#   mode "crash":  launcher parent SIGKILLs itself while the workload is still running.
import os, sys, time, signal, subprocess, json
sys.path.insert(0, "/home/sprime01/projects/sea-rs/.agents/evidence/security-v2/adversarial-20261017/probes")
import launchlib as L
mode, P = sys.argv[1], sys.argv[2]
ws, art = P + "/ws1", P + "/art1"
L.grant(ws, 100001, os.getuid()); L.grant(art, 100001, os.getuid())
lockfd = L.acquire_slot(P + "/state", 1)  # slot lock held by "launcher"
argv = ["/bin/sh", "-c", "(setsid sleep 1234 </dev/null >/dev/null 2>&1 &); (sleep 1235 &) ; echo started"]
if mode == "detach":
    cmd = L.launch_cmd(100001, 100001, L.HELPER, ws, art, argv)
    r = subprocess.run(cmd, cwd=ws, env={"PATH": "/usr/bin:/bin"}, capture_output=True, text=True, timeout=30)
    print(json.dumps({"mode": mode, "rc": r.returncode, "out": r.stdout.strip()}))
elif mode == "crash":
    argv2 = ["/bin/sh", "-c", "sleep 1236 & echo started; sleep 1237"]
    cmd = L.launch_cmd(100001, 100001, L.HELPER, ws, art, argv2)
    p = subprocess.Popen(cmd, cwd=ws, env={"PATH": "/usr/bin:/bin"}, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    time.sleep(1.5)
    print(json.dumps({"mode": mode, "launcher_pid": os.getpid(), "unshare_pid": p.pid}))
    sys.stdout.flush()
    os.kill(os.getpid(), signal.SIGKILL)
