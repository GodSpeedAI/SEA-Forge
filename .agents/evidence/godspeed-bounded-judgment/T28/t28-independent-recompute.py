#!/usr/bin/env python3
"""T28 independent recomputation of the T27 paired result.

Standalone by design: imports NOTHING from the T27 harness. Truth is
re-derived from the corpus state DBs directly (not from any manifest), the
selection rule is replayed from the repeatability per-case partition, all
statistics are recomputed from the raw persisted observations, and the
condition-A prompt bytes are reconstructed from the frozen prereg template
text alone and checked against the stored prompt hashes.
"""
import hashlib, json, os, re, sqlite3, sys, yaml
from collections import Counter
from math import comb

REPO = "/home/sprime01/projects/sea-rs"
T27 = os.path.join(REPO, ".agents/evidence/godspeed-bounded-judgment/T27")
CORPUS = os.path.expanduser("~/.local/share/godspeed-t16-corpus/runs")
PREREG_PATH = os.path.join(REPO, ".agents/preregistrations/godspeed-bounded-judgment-T27.prereg.yaml")
ADDENDUM_PATH = os.path.join(REPO, ".agents/preregistrations/godspeed-bounded-judgment-T27.prereg.addendum-1.yaml")
OBS = os.path.join(T27, "observations.jsonl")
PERCASE = os.path.join(REPO, ".agents/evidence/godspeed-bounded-judgment/repeatability/per-case.json")

ABSTAIN = "insufficient_evidence"
DOMAIN = {"settled", "not_settled", ABSTAIN}
problems = []

# ---------- 1. corpus identity from the frozen prereg pins -----------------
# NOTE (recorded defect): the frozen prereg file fails strict YAML parsing
# (relation_rules block scalars indented at their keys' level). The file is
# audited AS FROZEN (bytes never rewritten); its sha256 is re-verified here
# and the pins are extracted tolerantly.
PREREG_BYTES = open(PREREG_PATH, "rb").read()
prereg_sha = hashlib.sha256(PREREG_BYTES).hexdigest()
if prereg_sha != "41af1921285bdbea33684d06b226fc8add83a59ed71378dbed464ab5b4b74343":
    problems.append("prereg bytes do not match the frozen sha256")
try:
    yaml.safe_load(PREREG_BYTES)
    prereg_yaml_valid = True
except yaml.YAMLError as e:
    prereg_yaml_valid = False
    yaml_err = str(e).splitlines()[0]
pins_block = re.search(r"selected_dbs_sha256:\n((?:    \S+:\s+[0-9a-f]{64}\n)+)",
                       PREREG_BYTES.decode()).group(1)
pins = {}
for line in pins_block.strip().splitlines():
    cid, h = line.split(":")
    pins[cid.strip()] = h.strip()
db_hash_ok = 0
for cid, want in pins.items():
    p = os.path.join(CORPUS, cid, "state", "gauntlet-state.db")
    h = hashlib.sha256(open(p, "rb").read()).hexdigest()
    if h != want:
        problems.append(f"corpus: {cid} hash mismatch")
    db_hash_ok += (h == want)

# ---------- 2. truth re-derived from DBs (independent of any manifest) ------
truth, terminal_state = {}, {}
for cid in pins:
    db = os.path.join(CORPUS, cid, "state", "gauntlet-state.db")
    conn = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
    row = conn.execute("select payload from event_log where kind='run_terminal'").fetchone()
    st = json.loads(row[0])["state"]
    terminal_state[cid] = st
    truth[cid] = {"settled": "settled", "budget_exhausted": "not_settled"}[st]
    conn.close()

# cross-check against the frozen T16 manifest class labels (must agree)
m16 = yaml.safe_load(open(os.path.join(
    REPO, ".agents/preregistrations/godspeed-bounded-judgment-T16.expanded-corpus-manifest.yml")))
manifest_class = {r["case_id"]: r["consequence_class"] for r in m16["runs"]}
for cid in pins:
    if manifest_class[cid] != terminal_state[cid]:
        problems.append(f"truth: {cid} DB terminal {terminal_state[cid]} != manifest {manifest_class[cid]}")

# ---------- 3. selection rule replayed from repeatability partition --------
pc = json.load(open(PERCASE))
part = {"stable_wrong": [], "unstable": [], "stable_correct": [], "other": []}
for cid, d in pc.items():
    answers = [a for a in d["answers"] if a]
    if d["all_agree"] and answers:
        part["stable_correct" if d["aggregate_correct"] else "stable_wrong"].append(cid)
    elif not d["all_agree"]:
        part["unstable"].append(cid)
    else:
        part["other"].append(cid)
expected_frontier = sorted(part["stable_wrong"] + part["unstable"])
fams = sorted({c.split("-")[0] for c in part["stable_correct"]})
controls = []
for fam in fams:
    controls.append(min(c for c in part["stable_correct"] if c.split("-")[0] == fam))
rest = sorted(c for c in part["stable_correct"] if c not in controls)
controls = sorted(controls + rest[: 4 - len(controls)])
expected_selected = sorted(expected_frontier + controls)
if expected_selected != sorted(pins):
    problems.append(f"selection rule replay mismatch: {set(expected_selected) ^ set(pins)}")

# ---------- 4. raw observations ---------------------------------------------
recs = [json.loads(l) for l in open(OBS)]
if len(recs) != 140:
    problems.append(f"observations: {len(recs)} != 140")
keys = [r["key"] for r in recs]
if len(set(keys)) != len(keys):
    problems.append("observations: duplicate keys")
cells = {}
for r in recs:
    cells.setdefault((r["case_id"], r["condition"], r["rep"]), []).append(r)
for k, v in cells.items():
    if len(v) != 1:
        problems.append(f"observations: cell {k} has {len(v)} records")
for cid in pins:
    for cond in "AB":
        reps = {rep for (c, cd, rep) in cells if c == cid and cd == cond}
        if reps != set(range(5)):
            problems.append(f"observations: {cid}/{cond} reps {sorted(reps)}")
status_counts = Counter(r["status"] for r in recs)

# ---------- 5. independent case-level readouts (unique modal of 5) ----------
def readout(cid, cond):
    ans = []
    for rep in range(5):
        r = cells[(cid, cond, rep)][0]
        ans.append(r.get("answer") if r.get("status") == "ok" else None)
    valid = [a for a in ans if a]
    if len(valid) < 4:
        return ABSTAIN, valid
    c = Counter(valid)
    top = c.most_common(2)
    if len(top) > 1 and top[0][1] == top[1][1]:
        return ABSTAIN, valid
    return top[0][0], valid

table = []
for cid in sorted(pins):
    a_ro, a_calls = readout(cid, "A")
    b_ro, b_calls = readout(cid, "B")
    a_ok, b_ok = a_ro == truth[cid], b_ro == truth[cid]
    a_dec, b_dec = a_ro != ABSTAIN, b_ro != ABSTAIN
    if a_ro == b_ro:
        tr = "unchanged-correct" if b_ok else "unchanged-wrong"
    elif not a_ok and b_ok:
        tr = "correction(decided)" if a_dec and b_dec else "abstention-gain"
    elif a_ok and not b_ok:
        tr = "regression(decided)" if a_dec and b_dec else "abstention-loss"
    else:
        tr = "wrong->wrong flip"
    cat = ("stable-wrong" if cid in part["stable_wrong"] else
           "unstable-frontier" if cid in part["unstable"] else "control")
    table.append({"case": cid, "cat": cat, "truth": truth[cid], "A": a_ro, "B": b_ro,
                  "A_calls": a_calls, "B_calls": b_calls, "a_ok": a_ok, "b_ok": b_ok,
                  "transition": tr})

frontier = {t["case"] for t in table if t["cat"] != "control"}
controls_set = {t["case"] for t in table if t["cat"] == "control"}
b = sum(1 for t in table if not t["a_ok"] and t["b_ok"])
c = sum(1 for t in table if t["a_ok"] and not t["b_ok"])
b_f = sum(1 for t in table if t["case"] in frontier and not t["a_ok"] and t["b_ok"])
c_f = sum(1 for t in table if t["case"] in frontier and t["a_ok"] and not t["b_ok"])
c_ctrl = sum(1 for t in table if t["case"] in controls_set and t["a_ok"] and not t["b_ok"])
accA = sum(t["a_ok"] for t in table) / len(table)
accB = sum(t["b_ok"] for t in table) / len(table)
target = "budget-C3"
trow = next(t for t in table if t["case"] == target)
T = 1 if (not trow["a_ok"] and trow["b_ok"]) else 0

# ---------- 6. McNemar exact, own implementation ----------------------------
def p_two(bb, cc):
    tot = bb + cc
    return 1.0 if tot == 0 else min(1.0, 2 * sum(comb(tot, k) for k in range(min(bb, cc) + 1)) / 2 ** tot)
def p_one_lo(bb, cc):  # P(X <= min) under Bin(bb+cc, .5)
    tot = bb + cc
    return 1.0 if tot == 0 else sum(comb(tot, k) for k in range(min(bb, cc) + 1)) / 2 ** tot
p2, p_imp, p_harm = p_two(b, c), p_one_lo(b, c), p_one_lo(c, b)
net, fnet = b - c, b_f - c_f
practical = net >= 3 and fnet >= 2

# ---------- 7. frozen C-rule traversal (base prereg + addendum-1) -----------
degraded = sum(1 for t in table for cond in ("A_calls", "B_calls") if len(t[cond]) < 4)
validity = (db_hash_ok == len(pins)) and degraded == 0 and not [p for p in problems if p.startswith("observations")]
if not validity:
    trav = "C5(validity)"
elif c_ctrl >= 2 or (c > b and p_harm < 0.05):
    trav = "C4"
elif p2 < 0.05 and net >= 3 and fnet >= 2 and c_ctrl == 0:
    trav = "C1"
elif T == 1 and fnet >= 2 and c_ctrl == 0:
    trav = "C2"
elif T == 0 and b_f == 0 and c_f == 0 and c_ctrl == 0:
    trav = "C3"
else:
    trav = "C5(boundary)"

# ---------- 8. abstention shift ---------------------------------------------
abst = {cond: sum(1 for r in recs if r["condition"] == cond and r.get("answer") == ABSTAIN)
        for cond in "AB"}
abst_dec = {cond: sum(1 for r in recs if r["condition"] == cond and r.get("answer") in ("settled", "not_settled"))
            for cond in "AB"}

# ---------- 9. condition-A prompt-byte reproduction from prereg text --------
tmpl = (
    "You are doing a bounded judgment task. Answer STRICTLY with one JSON object.\n"
    "Question: Will this run settle accepted, or end non-settled (budget exhausted)?\n"
    "Answer domain: settled, not_settled, insufficient_evidence.\n"
    'JSON shape: {"answer": <label>, "confidence": <0..1>, "distribution": '
    '{<label>: <prob> for all three}, "reason": <one sentence>}\n\n'
)
a_match = a_total = 0
for cid in sorted(pins):
    db = os.path.join(CORPUS, cid, "state", "gauntlet-state.db")
    conn = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
    invs = [json.loads(p) for (p,) in conn.execute(
        "select payload from records where record_type='role_invocation' order by seq")]
    k = conn.execute("select count(*) from records where record_type='correction_selection'").fetchone()[0]
    conn.close()
    blob = json.dumps({"invocations": [{"result_document": i} for i in invs], "corrections": k},
                      default=str)[:12000]
    prompt = tmpl + blob
    ph = hashlib.sha256(prompt.encode()).hexdigest()[:12]
    for rep in range(5):
        r = cells[(cid, "A", rep)][0]
        key = hashlib.sha256(f"{prereg_sha}|{cid}|A|{rep}|{ph}".encode()).hexdigest()[:16]
        a_total += 1
        a_match += (r["prompt_hash"] == ph and r["key"] == key)

# ---------- 10. B consistency: 5 reps share one prompt hash ------------------
b_consistent = all(len({cells[(cid, "B", rep)][0]["prompt_hash"] for rep in range(5)}) == 1
                   for cid in pins)

out = {
    "prereg_sha256_verified": prereg_sha[:12] + "...",
    "prereg_strict_yaml_valid": prereg_yaml_valid,
    "prereg_yaml_defect": None if prereg_yaml_valid else yaml_err,
    "corpus_hash_ok": f"{db_hash_ok}/{len(pins)}",
    "truth_source": "run_terminal events re-derived from DBs; cross-checked vs T16 manifest (0 disagreements)" if not problems else problems,
    "selection_rule_replay": "MATCH" if expected_selected == sorted(pins) else "MISMATCH",
    "partition": {k: sorted(v) for k, v in part.items()},
    "status_counts": dict(status_counts),
    "table": [{k: t[k] for k in ("case", "cat", "truth", "A", "B", "transition")} for t in table],
    "accA": round(accA, 4), "accB": round(accB, 4),
    "b": b, "c": c, "b_f": b_f, "c_f": c_f, "c_ctrl": c_ctrl,
    "unchanged_correct": sum(1 for t in table if t["a_ok"] and t["b_ok"]),
    "unchanged_wrong": sum(1 for t in table if not t["a_ok"] and not t["b_ok"]),
    "target_T": T,
    "p_two": p2, "p_imp": p_imp, "p_harm": p_harm,
    "net": net, "frontier_net": fnet, "practical": practical,
    "c_traversal": trav,
    "abstentions": {k: abst[k] for k in abst}, "decided": abst_dec,
    "A_prompt_bytes_reproduced": f"{a_match}/{a_total}",
    "B_prompt_hash_consistent_across_reps": b_consistent,
}
json.dump(out, open(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                 "t28-independent-recompute.json"), "w"), indent=1)
print(json.dumps({k: v for k, v in out.items() if k != "table"}, indent=1))
print("\nPAIRED TABLE")
for t in table:
    print(f"  {t['case']:12s} {t['cat']:18s} truth={t['truth']:13s} A={t['A']:22s} B={t['B']:22s} {t['transition']}")
print("\nPROBLEMS:", problems if problems else "none")
sys.exit(1 if problems else 0)
