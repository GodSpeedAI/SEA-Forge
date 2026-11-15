#!/usr/bin/env sh
# WP-20: the S2 contracts cannot drift from the declared FDE SOP model.
#   - contracts/fde/field-case.v1.json lists exactly the SOP 4.2 groups and fields
#     (x-sop-group, x-sop-label, x-sop-dagger), against the frozen SOP text, whose
#     sha256 must match docs/sop/README.md and the schema's sop.sha256;
#   - the translation map has the rows those contracts rest on (4.2 stays `unknown`
#     with limitation S7: the model does not declare the fields, the SOP text is the
#     source; 4.4, 7.5d, 8.1a, 5.1, 7.2a);
#   - gate names/states, disposition kinds/levels, episode kinds, epistemic results,
#     payment currencies, reliability dimensions and aim dimensions used by
#     case-record-append.v2, assignment.v1, continuation.v1 and the v1 payloads equal
#     the model's enums and instances;
#   - continuation.v1 has one property per SOP 10.1 item, with the SOP text;
#   - case-record-append.v2 field_case.revised path enum equals the field-case paths.
# Exits non-zero on any violation.
#
# Usage: scripts/check-fde-field-case-schema.sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
command -v python3 >/dev/null 2>&1 || { echo 'fail: python3 is required' >&2; exit 2; }

python3 - "$root" <<'PY'
import glob, hashlib, json, re, sys

root = sys.argv[1]
sop_path = f"{root}/docs/sop/GS-FDE-SOP-001-v0.4.1.md"
sop = open(sop_path).read()
readme = open(f"{root}/docs/sop/README.md").read()
tmap = open(f"{root}/.sea/interaction/fde-sop/source-translation-map.md").read()
sea = "\n".join(open(p).read() for p in sorted(glob.glob(f"{root}/.sea/interaction/fde-sop/*.sea")))
C = f"{root}/contracts/fde"
load = lambda n: json.load(open(f"{C}/{n}"))
errors = []
def err(m): errors.append(m)

# --- SOP hash
sha = hashlib.sha256(sop.encode()).hexdigest()
if f"`{sha}`" not in readme: err("SOP sha256 differs from docs/sop/README.md")
fc = load("field-case.v1.json")
if fc["properties"]["sop"]["properties"]["sha256"]["const"] != sha: err("field-case.v1 sop.sha256 differs from the SOP")

# --- 4.2 fields
sec = sop.split("4.2 Minimum case content")[1].split("4.3 A reopened")[0]
want = []
for line in sec.splitlines():
    if not line.startswith("|") or line.startswith("|---") or line.startswith("| Group"): continue
    _, g, f, _ = [c.strip() for c in line.split("|")]
    fields = []
    for part in f.split(";"):
        part = part.strip().rstrip(".")
        label = re.sub(r"\s*\(.*$", "", part.replace("†", "").replace("`", "")).strip()
        fields.append((label, "†" in part))
    want.append((g.replace("†", "").strip(), fields))
groups = fc["properties"]["fields"]["properties"]
have = [(gs["x-sop-group"], [(p["x-sop-label"], p["x-sop-dagger"]) for p in gs["properties"].values()]) for gs in groups.values()]
if want != have:
    for (wg, wf), (hg, hf) in zip(want, have):
        if wg != hg: err(f"field-case.v1 group {hg!r} differs from SOP 4.2 group {wg!r}")
        if wf != hf: err(f"field-case.v1 fields of {hg!r} differ from SOP 4.2: only in SOP {sorted(set(wf)-set(hf))}, only in schema {sorted(set(hf)-set(wf))}")
    if len(want) != len(have): err(f"SOP 4.2 has {len(want)} groups, field-case.v1 has {len(have)}")
paths = []
for gk, gs in groups.items():
    if sorted(gs["required"]) != sorted(gs["properties"]): err(f"group {gk}: required differs from properties")
    if gs.get("additionalProperties") is not False: err(f"group {gk} is not closed")
    for fk in gs["properties"]: paths.append(f"{gk}.{fk}")
if len(set(paths)) != len(paths): err("field paths are not unique")
if sorted(fc["properties"]["fields"]["required"]) != sorted(groups): err("fields.required differs from groups")

# --- translation map rows
def row(clause):
    m = re.search(rf"^\| {re.escape(clause)} \|(.*)$", tmap, re.M)
    return [c.strip() for c in m.group(1).split("|")] if m else None
r = row("4.2")
if not r: err("translation map has no row 4.2")
elif r[2] != "unknown" or "S7" not in r[3]: err("translation map row 4.2 is no longer unknown/S7: the model now declares the fields, so field-case.v1 must be generated from the model instead of the SOP text")
for c in ("4.4", "5.1", "7.2a", "7.5d", "8.1a", "8.1b", "7.6b", "10.1"):
    if not row(c): err(f"translation map has no row {c}")

# --- model enums
def sea_enum(name):
    m = re.search(rf'enum "?{name}"?\s*\{{(.*?)\}}', sea, re.S)
    return re.findall(r'=\s*"([^"]+)"', m.group(1)) if m else None
def inst(prefix, key):
    return re.findall(rf'instance {prefix}\w* of "[A-Za-z]+" \{{\s*{key}: "([^"]+)"', sea)
v2 = load("case-record-append.v2.json")["$defs"]
v1 = load("case-record-append.v1.json")["$defs"]
fr = load("frozen-reference.v1.json")
asg = load("assignment.v1.json")["$defs"]
cont = load("continuation.v1.json")["$defs"]
def eq(a, b, what):
    if a is None or sorted(a) != sorted(b): err(f"{what}: contract {sorted(b)} differs from model {a and sorted(a)}")
eq(sea_enum("EvidenceGateName"), v2["GateAssessedPayload"]["properties"]["gate"]["enum"], "gate names (gate.assessed)")
eq(sea_enum("EvidenceGateName"), [g["properties"]["gate"]["enum"] for g in [cont["Continuation"]["properties"]["gate_evidence"]["items"]]][0], "gate names (continuation)")
eq(sea_enum("GateState"), v2["GateAssessedPayload"]["properties"]["state"]["enum"], "gate states")
eq(sea_enum("GateState"), cont["Continuation"]["properties"]["gate_evidence"]["items"]["properties"]["state"]["enum"], "gate states (continuation)")
eq(sea_enum("DispositionKind"), v2["DispositionRecordedPayload"]["properties"]["disposition"]["enum"], "disposition kinds")
eq(sea_enum("DispositionLevel"), v2["DispositionRecordedPayload"]["properties"]["level"]["enum"], "disposition levels")
eq(sea_enum("CaseLifecycle"), v2["Response"]["properties"]["lifecycle"]["enum"], "case lifecycle")
eq(sea_enum("CaseLifecycle"), cont["Continuation"]["properties"]["case"]["properties"]["lifecycle"]["enum"], "case lifecycle (continuation)")
eq(sea_enum("EpisodeKind"), fr["$defs"]["body"]["properties"]["kind"]["enum"], "episode kinds")
eq(sea_enum("EpisodeKind"), cont["Continuation"]["properties"]["episodes"]["items"]["properties"]["kind"]["enum"], "episode kinds (continuation)")
eq(list(re.findall(r"[SFUC]", re.search(r'pattern "EpistemicResult" matches "\^\[([A-Z]+)\]\$"', sea).group(1))), v1["EpisodeSettledPayload"]["properties"]["result"]["enum"], "epistemic results")
eq(inst("currency_", "currency_id"), [e for e in asg["Budget"]["items"]["properties"]["currency"]["enum"]], "payment currencies (assignment)")
eq(inst("currency_", "currency_id"), cont["Continuation"]["properties"]["work"]["properties"]["remaining_budgets"]["items"]["properties"]["currency"]["enum"], "payment currencies (continuation)")
eq(inst("reliability_", "dimension_id"), list(v1["EpisodeSettledPayload"]["properties"]["reliability"]["properties"]), "reliability dimensions")
eq(inst("aim_", "dimension_id"), cont["Continuation"]["properties"]["next"]["properties"]["dimension"]["enum"], "aim dimensions (blocker)")

# --- reserved operation params and the registry's read-only paths
res = load("reserved-operation.v1.json")["$defs"]
eq(sea_enum("DispositionKind"), res["CanonicalParamsDisposition"]["properties"]["disposition"]["enum"], "disposition kinds (reserved-operation)")
eq(sea_enum("EvidenceGateName"), res["CanonicalParamsGate"]["properties"]["gate"]["enum"], "gate names (reserved-operation)")
kinds = load("case-record-kinds.v1.json")
rp = kinds["properties"]["reserved_paths_after_freeze"]["items"]["enum"]

# --- field_case.revised path enum
pe = v2["FieldChange"]["properties"]["path"]["enum"]
if sorted(pe) != sorted(paths): err("case-record-append.v2 FieldChange.path enum differs from field-case.v1 paths")
for r_ in rp:
    if r_ not in paths: err(f"reserved path {r_} is not a field-case.v1 path")

# --- SOP 10.1 items
s101 = sop.split("10.1 Case continuation, derived from source-owned records (A:134-138): ")[1].split("\n10.2")[0]
segs = [s.strip() for s in " ".join(s101.split()).rstrip(".").split(";")]
props = cont["Continuation"]["properties"]
items = {p["x-sop-item"]: p["x-sop-segment"] for k, p in props.items() if "x-sop-item" in p}
if sorted(items) != list(range(1, len(segs) + 1)): err(f"continuation.v1 items {sorted(items)} differ from the {len(segs)} items of SOP 10.1")
for i, s in enumerate(segs, 1):
    if items.get(i) != s: err(f"continuation.v1 item {i} segment differs from SOP 10.1: {s!r}")
extra = [k for k, p in props.items() if "x-sop-item" not in p and k not in ("computed_from", "availability")]
if extra: err(f"continuation.v1 properties with no SOP 10.1 item: {extra}")

if errors:
    print("\n".join("FIELD-CASE SCHEMA DRIFT: " + e for e in errors)); sys.exit(1)
print(f"ok: field-case.v1 matches SOP 4.2 ({len(paths)} fields, {len(groups)} groups); model enums, SOP 10.1 items ({len(segs)}) and the field_case.revised paths agree")
PY
