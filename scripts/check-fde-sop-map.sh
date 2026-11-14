#!/usr/bin/env sh
# AT-02: every numbered clause of the frozen SOP has a row in
# .sea/interaction/fde-sop/source-translation-map.md with a status and a model
# element, or `unknown` with a reason and a named limitation. No row may lack a
# verbatim source excerpt, and every named model element must exist in the
# declared model. Exits non-zero on any violation.
#
# Usage: scripts/check-fde-sop-map.sh [--summary]
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

command -v python3 >/dev/null 2>&1 || { echo 'fail: python3 is required' >&2; exit 2; }

python3 - "$root" "$@" <<'PY'
import glob
import hashlib
import re
import sys

root = sys.argv[1]
sop_path = f"{root}/docs/sop/GS-FDE-SOP-001-v0.4.1.md"
readme_path = f"{root}/docs/sop/README.md"
map_path = f"{root}/.sea/interaction/fde-sop/source-translation-map.md"
lim_path = f"{root}/.sea/interaction/fde-sop/validation/limitations.md"
sea_glob = f"{root}/.sea/interaction/fde-sop/*.sea"

errors = []
def err(msg):
    errors.append(msg)

sop = open(sop_path, encoding="utf-8").read()
digest = hashlib.sha256(sop.encode("utf-8")).hexdigest()
readme = open(readme_path, encoding="utf-8").read()
if digest not in readme:
    err(f"docs/sop/README.md does not record the SOP sha256 {digest}")

def norm(t):
    t = t.replace("**", "").replace("`", "").replace("*", "")
    return re.sub(r"\s+", " ", t).strip()
sop_norm = norm(sop)

# ---- extract numbered clauses from the SOP ----
lines = sop.split("\n")
clauses = []   # ordered ids
section = None
started = False
for ln in lines:
    m = re.match(r"^## (\d+)\.", ln)
    if m:
        started = True
        section = int(m.group(1))
        clauses.append(f"§{section}")
        continue
    if not started:
        continue
    for m in re.finditer(r"(?:^|(?<=\s))(\d{1,2}\.\d{1,2})(?= (?:[A-Z]|\*\*))", ln):
        clauses.append(m.group(1))
    if section == 11:
        m = re.match(r"^(\d+)\. ", ln)
        if m:
            clauses.append(f"11#{m.group(1)}")
seen = set()
clauses = [c for c in clauses if not (c in seen or seen.add(c))]

# ---- known limitation ids ----
lim = open(lim_path, encoding="utf-8").read()
known_lims = set(re.findall(r"^#{2,4} (S\d+)\b", lim, flags=re.M))
known_lims |= set(re.findall(r"^#{2,4} (L\d+)\b", lim, flags=re.M))
if not known_lims:
    err("no limitation ids found in validation/limitations.md")

# ---- declared model text ----
model_text = ""
for p in sorted(glob.glob(sea_glob)):
    model_text += open(p, encoding="utf-8").read() + "\n"
def element_exists(name):
    return re.search(r"(?<![\w.-])" + re.escape(name) + r"(?![\w-])", model_text) is not None
kinds = {"operation", "policy", "role", "role_binding", "entity", "enum", "relation",
         "flow", "metric", "instance", "record", "namespace", "pattern", "resource", "field"}

# ---- parse map rows ----
rows = []
for ln in open(map_path, encoding="utf-8").read().split("\n"):
    if not ln.startswith("|"):
        continue
    cells = [c.strip() for c in ln.strip().strip("|").split("|")]
    if cells[0] in ("Clause",) or set(cells[0]) <= set("-: "):
        continue
    rows.append(cells)

id_re = re.compile(r"^(§\d+|\d+\.\d+[a-z]?|\d+\.\d+/.+|\d+/.+|11#\d)$")
status_set = {"declared", "inferred", "unknown"}
counts = {"declared": 0, "inferred": 0, "unknown": 0}
covered = set()
for cells in rows:
    cid = cells[0]
    if len(cells) != 6:
        err(f"row {cid!r}: expected 6 cells (Clause, Excerpt, Elements, Status, Limits, Note), got {len(cells)}")
        continue
    _, excerpt, elements, status, limits, note = cells
    if not id_re.match(cid):
        err(f"row {cid!r}: malformed clause id")
        continue
    # base clause
    if cid.startswith("§") or cid.startswith("11#"):
        base = cid
    elif "/" in cid:
        base, child = cid.split("/", 1)
        base = base if base != "6" and base != "8.1" else base
        if not re.search(re.escape(child), sop_norm):
            err(f"row {cid!r}: child name {child!r} does not occur in the SOP")
    else:
        base = re.sub(r"[a-z]$", "", cid)
    if "/" in cid:
        base = cid.split("/", 1)[0]
        if base == "6":
            base = "§6"
    if base not in clauses:
        err(f"row {cid!r}: base clause {base!r} is not a numbered clause of the SOP (no row without a source)")
        continue
    covered.add(base)
    if not excerpt or norm(excerpt) not in sop_norm:
        err(f"row {cid!r}: excerpt is empty or not a verbatim substring of the SOP: {excerpt!r}")
    if status not in status_set:
        err(f"row {cid!r}: status must be declared|inferred|unknown, got {status!r}")
        continue
    counts[status] += 1
    lims = [] if limits in ("", "-") else [x.strip() for x in limits.split(",")]
    for x in lims:
        if x not in known_lims:
            err(f"row {cid!r}: unknown limitation id {x!r}")
    if status == "unknown":
        if elements != "-":
            err(f"row {cid!r}: unknown rows must have element '-'")
        if len(note) < 15:
            err(f"row {cid!r}: unknown rows need a reason")
        if not lims:
            err(f"row {cid!r}: unknown rows must name a recorded limitation")
    else:
        els = [e.strip() for e in elements.split(",") if e.strip()]
        if not els or elements == "-":
            err(f"row {cid!r}: {status} rows need at least one model element")
        for e in els:
            m = re.match(r"^([a-z_]+):(.+)$", e)
            if not m or m.group(1) not in kinds:
                err(f"row {cid!r}: element {e!r} must be kind:name with kind in {sorted(kinds)}")
            elif not element_exists(m.group(2)):
                err(f"row {cid!r}: element {e!r} does not exist in .sea/interaction/fde-sop/*.sea")

for c in clauses:
    if c not in covered:
        err(f"clause {c} has no translation-map row")

total = sum(counts.values())
sec_rows = [r for r in rows if len(r) == 6 and r[0].startswith("§")]
print(f"SOP sha256: {digest}")
print(f"numbered clauses in SOP: {len(clauses)} ({sum(1 for c in clauses if c.startswith('§'))} sections, "
      f"{sum(1 for c in clauses if not c.startswith('§'))} subsections/clauses)")
print(f"map rows: {total}  declared={counts['declared']} inferred={counts['inferred']} unknown={counts['unknown']}")
if errors:
    for e in errors:
        print(f"fail: {e}", file=sys.stderr)
    print(f"VERDICT: FAIL ({len(errors)} problem(s))", file=sys.stderr)
    sys.exit(1)
print("VERDICT: PASS")
PY
