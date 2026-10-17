#!/usr/bin/env python3
"""Gitleaks allowlist canary suite (2026-10-17 continuation exception).

Builds a throwaway git repo with one commit per canary at the exact paths the
exception names, then scans it with: the committed config (commit pin active),
a variant whose 2026-10-17 tuples are re-pinned to the canary commits (so the
path/rule/value conjuncts are tested in isolation), the previous line-regex
config re-pinned the same way, and the default ruleset with no allowlist.
All canary values are freshly generated synthetic strings.
"""
import json, os, re, secrets, string, subprocess, sys, pathlib, shutil
repo_cfg, out = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
old_cfg = pathlib.Path(sys.argv[3])
work = out / "repo"; shutil.rmtree(work, ignore_errors=True); work.mkdir(parents=True)
def git(*a): return subprocess.run(["git", "-C", str(work), *a], check=True, capture_output=True, text=True).stdout.strip()
git("init", "-q"); git("config", "user.email", "canary@invalid"); git("config", "user.name", "canary")
RES = ".agents/evidence/security-v1/continuation-20261017/all/results.json"
LOG = ".agents/evidence/security-v1/continuation-20261017/all/rust-A.log"
TRC = ".agents/evidence/security-v2/realitytrace-sec12-20261017/bundle/sec12-trace.json"
ALLOWED_RES = "dc81f0712a185286079d08854f23a70bf81fea8b31ad9dd765272502861cf22d"
ALLOWED_RES_ONLY = "845d0079ee6df06022173ff6766f45cf7ecfa2d9de8fd2b78249c429747a8e13"  # allowed in results/log, not trace
def hexs(n): return secrets.token_hex(n // 2)
def alnum(n): return "".join(secrets.choice(string.ascii_letters + string.digits) for _ in range(n))
pem_body = "\n".join(alnum(64) for _ in range(6))
canaries = [
 ("P1", "positive control: exact allowed value, allowed path", RES, f'  "idempotency_key": "{ALLOWED_RES}",'),
 ("C1", "allowed path, field shape, NEW 64-hex value", RES, f'  "idempotency_key": "{hexs(64)}",'),
 ("C2", "allowed path, credential-like api_key", RES, f'  "api_key": "{alnum(32)}",'),
 ("C3", "allowed value moved to another allowed tuple's path", TRC, f'  "idempotency_key": "{ALLOWED_RES_ONLY}",'),
 ("C4", "allowed value in an unlisted path", ".agents/evidence/security-v2/other-20261017/x.json", f'  "idempotency_key": "{ALLOWED_RES}",'),
 ("C5", "allowed value altered: uppercase", RES, f'  "idempotency_key": "{ALLOWED_RES.upper()}",'),
 ("C6", "allowed value altered: extended to 66 hex", RES, f'  "idempotency_key": "{ALLOWED_RES}ab",'),
 ("C7", "allowed value altered: different field name", RES, f'  "access_key": "{ALLOWED_RES}",'),
 ("C8", "credential on the same line as an allowed digest", RES, f'  {{"idempotency_key": "{ALLOWED_RES}", "client_secret": "{alnum(40)}"}}'),
 ("C9", "AWS access key id in allowed path", LOG, f'aws_access_key_id = AKIA{"".join(secrets.choice(string.ascii_uppercase+"234567") for _ in range(16))}'),
 ("C10", "GitHub PAT in allowed path", TRC, f'  "token": "ghp_{alnum(36)}",'),
 ("C11", "PEM private key in allowed path", LOG, f"-----BEGIN RSA PRIVATE KEY-----\n{pem_body}\n-----END RSA PRIVATE KEY-----"),
 ("C12", "Slack bot token in allowed path", RES, f'  "slack": "xoxb-{secrets.randbelow(10**12):012d}-{secrets.randbelow(10**12):012d}-{alnum(24)}",'),
]
(work / "README").write_text("canary\n"); git("add", "-A"); git("commit", "-qm", "base")
sha = {}
for cid, desc, path, line in canaries:
    f = work / path; f.parent.mkdir(parents=True, exist_ok=True)
    f.write_text((f.read_text() if f.exists() else "") + line + "\n")
    git("add", "-A"); git("commit", "-qm", cid); sha[cid] = git("rev-parse", "HEAD")
pins = "[" + ", ".join(f'"{s}"' for s in sha.values()) + "]"
def repin(text, marker):
    i = text.index(marker); head, tail = text[:i], text[i:]
    return head + re.sub(r'commits = \[[^\]]*\]', "commits = " + pins, tail)
real = repo_cfg.read_text()
cfgs = {
 "committed": real,
 "committed-repinned": repin(real, "# Security continuation evidence (2026-10-17)"),
 "previous-line-regex-repinned": repin(old_cfg.read_text(), "# Security continuation evidence (2026-10-17)"),
 "default-no-allowlist": 'title = "default"\n[extend]\nuseDefault = true\n',
}
by_sha = {v: k for k, v in sha.items()}
report = {"canaries": [{"id": c, "description": d, "path": p} for c, d, p, _ in canaries], "configs": {}}
for name, text in cfgs.items():
    cp = out / f"{name}.toml"; cp.write_text(text); rp = out / f"{name}.findings.json"
    r = subprocess.run(["gitleaks", "detect", "--no-banner", "--redact", "--source", str(work), "--config", str(cp),
                        "--report-format", "json", "--report-path", str(rp)], capture_output=True, text=True)
    found = {}
    for x in json.loads(rp.read_text() or "[]"):
        found.setdefault(by_sha.get(x["Commit"], "?"), []).append(x["RuleID"])
    report["configs"][name] = {"exit": r.returncode, "detected": {c: sorted(set(found.get(c, []))) for c in sha}}
    rp.unlink()
(out / "canary-results.json").write_text(json.dumps(report, indent=1))
hdr = "canary".ljust(5) + "".join(n[:28].ljust(30) for n in cfgs)
print(hdr)
for c in sha:
    print(c.ljust(5) + "".join((",".join(report["configs"][n]["detected"][c]) or "-- not detected")[:28].ljust(30) for n in cfgs))
print({n: v["exit"] for n, v in report["configs"].items()})
