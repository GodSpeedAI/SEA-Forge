#!/usr/bin/env sh
# Prints JSON {repos, tools} for the proof results file: the commit (and cleanliness) of every repository
# involved and the versions/digests of the tools. Read-only.
. "$(dirname "$0")/_lib.sh"
python3 - "$FDE_ROOT" "$RT_DIR" "$COG_DIR" "$HZ_DIR" "$HOME/projects/domainforge" "$DF_DEV_BIN" "$DF_RELEASE_BIN" "$RT_BIN" <<'PY'
import json, subprocess, sys, hashlib, os
root, rt, cog, hz, df, dfdev, dfrel, rtbin = sys.argv[1:9]
def sh(*a, cwd=None):
    try: return subprocess.run(a, cwd=cwd, capture_output=True, text=True, timeout=30).stdout.strip()
    except Exception: return ""
def repo(p, ref="HEAD"):
    if not os.path.isdir(p): return {"path": p, "commit": None, "present": False}
    st = sh("git", "status", "--porcelain", "--untracked-files=no", cwd=p)
    return {"path": p, "commit": sh("git", "rev-parse", ref, cwd=p), "branch": sh("git", "rev-parse", "--abbrev-ref", "HEAD", cwd=p),
            "tracked_changes": len([l for l in st.splitlines() if "/.jolli/" not in l and "/.claude/" not in l]), "present": True}
def sha(p):
    try: return hashlib.sha256(open(p, "rb").read()).hexdigest()
    except Exception: return None
def ver(p):
    return sh(p, "--version") if os.path.isfile(p) else None
repos = {"sea-forge": repo(root), "realitytrace": repo(rt), "cognate": repo(cog), "horizon": repo(hz),
         "domainforge": {**repo(df, "origin/main"), "tag_v0.22.0": sh("git", "rev-parse", "v0.22.0^{commit}", cwd=df)}}
tools = {"rustc": sh("rustc", "--version"), "cargo": sh("cargo", "--version"), "bun": sh("bun", "--version"),
         "node": sh("node", "--version"), "python3": sh("python3", "--version"), "git": sh("git", "--version"),
         "domainforge_release": {"path": dfrel, "version": ver(dfrel), "sha256": sha(dfrel)},
         "domainforge_dev_3960326": {"path": dfdev, "version": ver(dfdev), "sha256": sha(dfdev)},
         "realitytrace_bin": {"path": rtbin, "version": ver(rtbin), "sha256": sha(rtbin)}}
print(json.dumps({"repos": repos, "tools": tools}))
PY
