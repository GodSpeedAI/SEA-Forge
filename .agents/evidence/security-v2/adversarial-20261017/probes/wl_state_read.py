# Runs INSIDE the workload. Reads trusted-server state files; prints only path, size, mode-class and a sha256 prefix.
import os, json, hashlib
R = os.environ.get("ADV_ROOT", "/home/sprime01/projects/sea-rs/.sea-forge")
out = []
for dp, dn, fn in os.walk(R):
    for f in fn:
        p = os.path.join(dp, f)
        try:
            with open(p, "rb") as h: d = h.read(1 << 22)
            out.append({"path": p.replace(R, ".sea-forge"), "size": len(d), "sha256_12": hashlib.sha256(d).hexdigest()[:12]})
        except OSError as e:
            out.append({"path": p.replace(R, ".sea-forge"), "err": e.errno})
print(json.dumps({"readable": sum(1 for o in out if "err" not in o), "total": len(out), "sample": out[:20]}))
