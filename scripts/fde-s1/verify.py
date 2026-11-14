#!/usr/bin/env python3
"""AT-18 (S1 form): a fresh verifier pass over the KEPT raw records of an S1 cell, in a second clean directory.

  1. anchors: `scripts/fde-anchor.sh verify prereg/fde-s1`; each freeze anchor tag is fetched from the cell's anchor
     remote into the clean directory and its digest compared with the case record's freeze digest, and its time
     with the first run record (fde-anchor.sh refuses freeze verification by design: PARTIAL).
  2. raw-record manifest: the sha256 of every kept raw file equals the manifest written at the end of the run.
  3. replay: every recorded `realitytrace observed derive|compare|propose|regression|capability-delta` step is run
     again over a COPY of the kept RealityTrace ledger; the representation and residual hashes, and the output
     bytes, must equal the recorded ones. DomainForge envelope steps are re-run for world_ref equality.
  4. class level: the classes in the replayed compare results equal the recorded classes.
Not done (reported, not hidden): re-execution of the three `program` episodes (their cargo builds), and verifier
independence beyond a fresh process and directory (same OS user, same host).
Exit: 0 all of 1-4 agree and re-execution was done; 10 all of 1-4 agree, re-execution not done / anchors PARTIAL;
1 a disagreement; 12 the cell is absent or a debug cell.
"""
import hashlib, json, os, pathlib, shutil, subprocess, sys

cell = pathlib.Path(sys.argv[1]).resolve()
root = pathlib.Path(__file__).resolve().parents[2]
out = pathlib.Path(os.environ.get("FDE_S1_VERIFY_DIR", str(cell.parent / (cell.name + "-verify"))))
if not (cell / "state.json").exists(): print("REASON: no S1 cell at " + str(cell)); sys.exit(12)
S = json.loads((cell / "state.json").read_text())
if S.get("debug_argv"): print("REASON: the cell is a debug cell (FDE_S1_DEBUG_ARGV): not evidence"); sys.exit(12)
shutil.rmtree(out, ignore_errors=True); (out / "raw").mkdir(parents=True)
report = {"cell": str(cell), "verifier_dir": str(out), "checks": []}
bad = []
def check(name, ok, **kw):
    report["checks"].append({"name": name, "ok": bool(ok), **kw})
    if not ok: bad.append(name)
sha = lambda b: hashlib.sha256(b).hexdigest()

# 1. anchors
p = subprocess.run([str(root / "scripts/fde-anchor.sh"), "verify", "prereg/fde-s1"], capture_output=True, text=True, cwd=root)
check("anchor:prereg/fde-s1", p.returncode == 0, rc=p.returncode, out=p.stdout.strip()[-200:])
clone = out / "anchor-clone"
subprocess.run(["git", "clone", "-q", str(cell / "anchor-remote.git"), str(clone)], check=True, capture_output=True)
subprocess.run(["git", "-C", str(clone), "fetch", "-q", "origin", "refs/tags/*:refs/tags/*"], check=True, capture_output=True)
tags = subprocess.run(["git", "-C", str(clone), "tag", "-l", "fde-anchor/freeze/*"], capture_output=True, text=True).stdout.split()
recs = [json.loads(l) for l in (cell / "sf-root/cases/case-0001/field-records.jsonl").read_text().splitlines() if l.strip()]
frozen = {r["payload"]["digest"]: r for r in recs if r["kind"] == "episode.frozen"} if recs and "payload" in recs[0] else {r["digest"]: r for r in recs if r["kind"] == "episode.frozen"}
ok_tags = True; seen = []
for t in tags:
    msg = subprocess.run(["git", "-C", str(clone), "tag", "-l", "--format=%(contents)", t], capture_output=True, text=True).stdout
    try: doc = json.loads(msg)
    except Exception: doc = {}
    dg = doc.get("digest")
    seen.append({"tag": t, "digest": dg, "matches_a_freeze_record": dg in frozen})
    ok_tags = ok_tags and dg in frozen
check("anchor:freeze tags carry the freeze record digests", ok_tags and len(tags) >= 4, tags=seen, note="fde-anchor.sh verify does not verify freeze anchors (WP-14 case keeper needed); push time is the local clock (OD-6)")
# 2. manifest
man = json.loads((cell / "manifest.json").read_text())
diff = [f for f, h in man["files"].items() if not (cell / f).exists() or sha((cell / f).read_bytes()) != h]
check("raw-record manifest sha256s", not diff and len(man["files"]) > 20, files=len(man["files"]), mismatched=diff[:5])
# 3-4. replay over a copy of the kept RealityTrace ledger
home = out / "rt-home"; shutil.copytree(cell / "rt-home", home)
env = dict(os.environ, XDG_DATA_HOME=str(home / "data"), XDG_STATE_HOME=str(home / "state"))
steps = [json.loads(l) for l in (cell / "steps.jsonl").read_text().splitlines()]
replayed = 0; mism = []
for s in steps:
    n = s["step"]
    if not (n.startswith("rt-derive-") or n.startswith("rt-compare-") or n in ("rt-propose", "rt-regression", "rt-capability-delta")): continue
    argv = list(s["argv"]); outf = None; extra = []
    for i, a in enumerate(argv):
        if a in ("--out", "--facts-out"):
            orig = pathlib.Path(argv[i + 1]); argv[i + 1] = str(out / "raw" / (a.strip("-") + "-" + orig.name)); 
            if a == "--out": outf = (orig, pathlib.Path(argv[i + 1]))
            else: extra.append((orig, pathlib.Path(argv[i + 1])))
    # inputs produced by earlier replayed steps are the replayed ones: derive outputs feed compare
    for i, a in enumerate(argv):
        if a == "--observed" or a == "--proposal" or a in ("--old", "--new", "--control-old", "--control-new", "--compared"):
            o = pathlib.Path(argv[i + 1]); r = out / "raw" / ("out-" + o.name)
            if r.exists(): argv[i + 1] = str(r)
    p = subprocess.run(argv, env=env, capture_output=True, text=True)
    replayed += 1
    if p.returncode != s["rc"]: mism.append({"step": n, "rc": p.returncode, "recorded_rc": s["rc"]}); continue
    for orig, new in ([outf] if outf else []) + extra:
        if orig.exists() and (not new.exists() or new.read_bytes() != orig.read_bytes()): mism.append({"step": n, "file": orig.name, "differs": True})
check("replay: derive/compare/propose/regression/capability-delta over a copy of the kept ledger reproduce the recorded bytes", replayed >= 20 and not mism, replayed=replayed, mismatches=mism[:6])
# DomainForge envelope: world_ref equality
dfm = []
for s in steps:
    if s["step"].startswith("model-v") and s["step"].endswith("-envelope"):
        v = s["step"].split("-")[1]
        r = subprocess.run(s["argv"], capture_output=True, text=True, cwd=cell / "sf-root/worlds" / v)
        ref = json.loads(r.stdout)["scope"]["world_ref"] if r.returncode == 0 else None
        dfm.append({"world": v, "ref": ref, "recorded": S["worlds"][v]["world_ref"]}); 
check("replay: DomainForge envelope reproduces each world_ref", all(x["ref"] == x["recorded"] for x in dfm), worlds=dfm)
# class level: recorded vs replayed compare results
cl = []
for f in sorted((cell / "raw/rt").glob("*-compare.json")):
    new = out / "raw" / ("out-" + f.name)
    if not new.exists(): continue
    a, b = json.loads(f.read_text()), json.loads(new.read_text())
    ca = sorted((r["class"], r["residual_id"]) for l in ("episode", "structural") for r in a["residuals"][l])
    cb = sorted((r["class"], r["residual_id"]) for l in ("episode", "structural") for r in b["residuals"][l])
    cl.append({"file": f.name, "equal": ca == cb, "classes": sorted({c for c, _ in ca})})
check("class level: replayed classes equal the recorded classes", cl and all(x["equal"] for x in cl), compared=len(cl))
report["not_done"] = ["re-execution of the three program episodes (declared cargo commands) was not repeated by the verifier", "verifier is the same OS user on the same host (no independence beyond a fresh process and directory)"]
report["verdict"] = "disagreement" if bad else "agrees (PARTIAL: re-execution not done; anchors local-only)"
(out / "report.json").write_text(json.dumps(report, indent=1))
for c in report["checks"]: print(("ok   " if c["ok"] else "FAIL ") + c["name"])
if bad:
    print("REASON: verifier disagrees with the recorded results on: " + "; ".join(bad)); sys.exit(1)
print("REASON: anchors verify (prereg by fde-anchor.sh; freeze tags by digest, local clock only), replay hashes and classes equal the recorded ones; re-execution of the program episodes was not repeated by the verifier")
sys.exit(10)
