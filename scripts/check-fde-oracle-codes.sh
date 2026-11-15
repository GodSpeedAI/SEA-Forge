#!/usr/bin/env sh
# WP-20: every oracle code that G-qualification-plan.md names for the S2 negative
# tests (NT-10, 11, 12, 13, 17, 18, 22, 23, 24, 32, 33, 34, 48) exists in exactly one
# contract error table (its home), and the mapping in contracts/fde/oracle-codes.json
# agrees with the schemas: every code of every Error enum is mapped, the enums that
# contain a code are exactly its home plus also_raised_by, and every AT row served by
# the package is mapped. Exits non-zero on any violation.
#
# Usage: scripts/check-fde-oracle-codes.sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
command -v python3 >/dev/null 2>&1 || { echo 'fail: python3 is required' >&2; exit 2; }

python3 - "$root" "${FDE_CONTRACTS_DIR:-$root/contracts/fde}" <<'PY'
import glob, json, os, re, sys
root, cdir = sys.argv[1], sys.argv[2]
g = open(f"{root}/.agents/reports/fde-convergence-2026-11-12/G-qualification-plan.md").read()
m = json.load(open(f"{cdir}/oracle-codes.json"))
errors = []
def err(x): errors.append(x)

# 1. G rows -> codes (4th column, first backticked UPPER_CASE token)
for row_id, code in m["g_rows"].items():
    line = re.search(rf"^\| {row_id} \|(.*)$", g, re.M)
    if not line: err(f"{row_id} is not in G"); continue
    cols = [c.strip() for c in line.group(1).split("|")]
    found = re.findall(r"`([A-Z][A-Z0-9_]+)`", cols[2])
    if code not in found: err(f"{row_id}: G names {found}, mapping says {code}")
# every NT row of the WP-20 list must be mapped
for row_id in ("NT-10", "NT-11", "NT-12", "NT-13", "NT-17", "NT-18", "NT-22", "NT-23", "NT-24", "NT-32", "NT-33", "NT-34", "NT-48"):
    if row_id not in m["g_rows"]: err(f"{row_id} missing from g_rows")
for row_id in ("AT-03", "AT-09", "AT-10", "AT-11", "AT-15", "AT-17"):
    if row_id not in m["at_rows"]: err(f"{row_id} missing from at_rows")

# 2. enums
enums = {}
for p in sorted(glob.glob(f"{cdir}/*.v*.json")):
    n = os.path.basename(p)
    e = json.load(open(p)).get("$defs", {}).get("Error")
    if e: enums[n] = e["properties"]["code"]["enum"]
v2 = set(enums["case-record-append.v2.json"])
if not set(enums["case-record-append.v1.json"]) <= v2: err("a v1 append code is missing from the v2 enum")
by_code = {}
for f, es in enums.items():
    if f == "case-record-append.v1.json": continue
    for c in es:
        by_code.setdefault(c, set()).add(f)
mapped = {}
for e in m["codes"]:
    if e["code"] in mapped: err(f"{e['code']} mapped twice")
    mapped[e["code"]] = e
    home = e["home"].split("#")[0]
    want = {home} | set(e["also_raised_by"])
    if by_code.get(e["code"]) != want: err(f"{e['code']}: enums containing it {sorted(by_code.get(e['code'], []))} differ from home+also_raised_by {sorted(want)}")
    if home not in by_code.get(e["code"], set()): err(f"{e['code']}: not in its home table {home}")
for c in by_code:
    if c not in mapped: err(f"{c} is in an error table but not in oracle-codes.json")
for row_id, code in m["g_rows"].items():
    e = mapped.get(code)
    if not e: err(f"{row_id}: {code} is in no error table")
    elif row_id not in e["oracle_rows"]: err(f"{code}: oracle_rows lacks {row_id}")
if errors:
    print("\n".join("ORACLE CODE: " + e for e in errors)); sys.exit(1)
print(f"ok: {len(m['g_rows'])} G oracle codes, {len(mapped)} error codes, each in exactly one home table")
PY
