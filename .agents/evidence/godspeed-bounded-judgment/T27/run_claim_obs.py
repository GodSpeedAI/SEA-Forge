#!/usr/bin/env python3
"""T27 claim-vs-observation paired judgment experiment (gate harness).

Frozen prereg:  .agents/preregistrations/godspeed-bounded-judgment-T27.prereg.yaml
                (sha256 41af1921285bdbea33684d06b226fc8add83a59ed71378dbed464ab5b4b74343)
Frozen addendum:.agents/preregistrations/godspeed-bounded-judgment-T27.prereg.addendum-1.yaml
                (sha256 a7fb31e295dfcca4c1fe6aba32f9345857d22187038b7c29f918b28681e07609;
                 binary-correctness McNemar: abstain scored not-correct for the primary test)

Modes:
  --teeth          run the four frozen teeth (real renderings + simulated attacks)
  --condition both verify corpus identity, run teeth, acquire all 140 calls
                   (persist-before-score), score, analyze, write the paired report
  --score          recompute scoring/analysis from persisted observations only
  --verify-only    T28 verification: corpus + store + teeth re-verification and
                   analysis recomputation; no provider calls; nonzero on any mismatch
"""
from __future__ import annotations
import argparse, hashlib, json, os, re, sqlite3, subprocess, sys, time
from collections import Counter
from math import comb

REPO = "/home/sprime01/projects/sea-rs"
OUT = os.path.join(REPO, ".agents/evidence/godspeed-bounded-judgment/T27")
PREREG = os.path.join(REPO, ".agents/preregistrations/godspeed-bounded-judgment-T27.prereg.yaml")
PREREG_SHA = "41af1921285bdbea33684d06b226fc8add83a59ed71378dbed464ab5b4b74343"
ADDENDUM_SHA = "a7fb31e295dfcca4c1fe6aba32f9345857d22187038b7c29f918b28681e07609"
CORPUS = os.path.expanduser("~/.local/share/godspeed-t16-corpus/runs")
STORE = os.path.join(OUT, "observations.jsonl")
REPORT = os.path.join(OUT, "t27-paired-report.yml")
TEETH_LOG = os.path.join(OUT, "teeth-results.yml")

DOMAIN = ["settled", "not_settled", "insufficient_evidence"]
ABSTAIN = "insufficient_evidence"
PROVIDER_CMD = ["opencode", "run", "-m", "cline-pass/cline-pass/deepseek-v4.1-flash"]
PROVIDER_ID = "deepseek-v4.1-flash-via-clinepass"
R = 5

# ---- frozen corpus partition (prereg section 1) ---------------------------
STABLE_WRONG = ["budget-C3"]
UNSTABLE = ["accepted-A4", "accepted-A10", "accepted-A11", "accepted-A12",
            "budget-C1", "budget-C2", "budget-C4", "budget-C5", "forge-D2"]
CONTROLS = ["accepted-A1", "accepted-A2", "forge-D1", "lying-B1"]
SELECTED = sorted(CONTROLS + STABLE_WRONG + UNSTABLE)
FRONTIER = sorted(STABLE_WRONG + UNSTABLE)
TARGET = "budget-C3"
DB_SHA256 = {
    "accepted-A1": "8d419e023118775f777abeacf1497521be10aef8b00e08ab022ab5325759c2d5",
    "accepted-A2": "e284bb0ed8d6f197b63bda677482828eee21053eec89205c9d4869c291dd88ba",
    "accepted-A4": "50b0b8a0795e45d17a21c77eb9efde6ee10aec857b4eaeef65b8c4f705cb5ac5",
    "accepted-A10": "3c169cfaa9cbe896e9cdf8bf72de3ae0b206563a074039b9b6ef25facf33dea7",
    "accepted-A11": "58a7461bdee8ca6fedf900aa46e086eed45c058572facb824c2eab1bb5ac3b51",
    "accepted-A12": "93c544f2375955ca5617baed8952c43f0c4ea0da0f3876815977ea60f5e5b13d",
    "budget-C1": "8706a12a4df2a957b5327665a35a6ee62d779c533a4c781ab5655472dbb5c0f0",
    "budget-C2": "1db0a710ab8b86a30bead02ffc5a974803d258342d5958ef00819914d2f62424",
    "budget-C3": "bed986f7f134a9d2f6b96c752549623c191d27f8e18883568d6e43de81402109",
    "budget-C4": "b8ac779a391feedaec7696e79a5685194d7bcae4ddb57df40b96d9c43b08dfe3",
    "budget-C5": "01dbb9cbd7ee022a9654f2f673befab1a5a6cf2e39b45cbbdbe67a307c2eac5c",
    "forge-D1": "3bd599dd69639b45136ee6b53b5ef17b4c7754305db8d49dc71ae2f4d46d1037",
    "forge-D2": "c7fbfa3e981826ebdddacb2a6b03df67804df0c6eb03b5c83daf12afda9bc4e3",
    "lying-B1": "ab6cc686ed3bf18f4e15e8d05b4e691aaffe80ff6f9432e17d6cd48768485b04",
}
FAMILY = {"accepted": "accepted", "lying": "lying", "budget": "budget", "forge": "forge"}

# ---- frozen prompt templates (prereg section 2) ----------------------------
HEADER_COMMON = (
    "You are doing a bounded judgment task. Answer STRICTLY with one JSON object.\n"
    "Question: Will this run settle accepted, or end non-settled (budget exhausted)?\n"
    "Answer domain: settled, not_settled, insufficient_evidence.\n"
    'JSON shape: {"answer": <label>, "confidence": <0..1>, "distribution": '
    '{<label>: <prob> for all three}, "reason": <one sentence>}\n'
)
HEADER_B_EXTRA = (
    "The information below is TYPED by epistemic role. [CLAIM] = what a source\n"
    "ASSERTS about the run (it may or may not be accurate). [OBSERVATION] =\n"
    "what was independently recorded. [PROVENANCE] = the source record each\n"
    "item comes from. [RELATION] = whether observed records mechanically\n"
    "support, contradict, or leave each claim unresolved. These labels\n"
    "describe the structure of the records only; judge the question on the\n"
    "records as presented.\n"
)
ROLE_LABELS = {"CLAIM", "OBSERVATION", "EXPECTED", "PROVENANCE", "RELATION"}
RELATION_VALUES = {"supports", "contradicts", "unresolved"}
BANNED_LABEL_TOKENS = ["settled", "budget", "exhausted", "pass", "fail", "correct",
                       "wrong", "true", "false", "valid", "invalid", "success",
                       "lying", "forged", "honest", "deceptive", "accept", "reject"]
TRUTH_TOKENS = ["settled", "not_settled", "budget_exhausted",
                "run_terminal", "settlement_committed"]


# ---------------------------------------------------------------- corpus ----
def verify_corpus():
    problems = []
    for cid in SELECTED:
        p = os.path.join(CORPUS, cid, "state", "gauntlet-state.db")
        if not os.path.isfile(p):
            problems.append(f"{cid}: missing DB {p}")
            continue
        h = hashlib.sha256(open(p, "rb").read()).hexdigest()
        if h != DB_SHA256[cid]:
            problems.append(f"{cid}: sha256 {h} != frozen {DB_SHA256[cid]} (S3)")
    return problems


def load_case(cid):
    db = os.path.join(CORPUS, cid, "state", "gauntlet-state.db")
    conn = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
    conn.row_factory = sqlite3.Row
    term = conn.execute("select payload from event_log where kind='run_terminal'").fetchone()
    truth = {"settled": "settled", "budget_exhausted": "not_settled"}[
        json.loads(term["payload"])["state"]]
    invocations = [json.loads(p) for (p,) in conn.execute(
        "select payload from records where record_type='role_invocation' order by seq")]
    corrections = conn.execute(
        "select count(*) from records where record_type='correction_selection'").fetchone()[0]
    conn.close()
    return {"case_id": cid, "truth": truth, "invocations": invocations,
            "corrections": corrections}


# -------------------------------------------------------------- renderers ---
def a_structure(case):
    return {"invocations": [{"result_document": inv} for inv in case["invocations"]],
            "corrections": case["corrections"]}


def render_a(case):
    blob = json.dumps(a_structure(case), default=str)[:12000]
    assert len(json.dumps(a_structure(case), default=str)) <= 12000, \
        f"{case['case_id']}: A blob exceeds the frozen 12000-char cap"
    return HEADER_COMMON + "\n" + blob


def _claim_blocks(case):
    """Per frozen renderer_spec: one CLAIM block per self-describing field
    per invocation, PROVENANCE with the full payload, OBSERVATION blocks,
    RELATION lines by frozen rules R1-R3."""
    invs = case["invocations"]
    lines = []
    claims = []  # (text, kind, idx)
    for i, inv in enumerate(invs, start=1):
        session = inv.get("session_ref")
        state = inv.get("state")
        role = inv.get("role")
        c1 = f'The session of this run is declared as "{session}".'
        c2 = f'Invocation #{i} is declared with state "{state}".'
        c3 = f'The actor of invocation #{i} declares role "{role}".'
        lines.append(f"[CLAIM | source: session_ref of role_invocation #{i}]")
        lines.append(f"  {c1}")
        lines.append(f"[CLAIM | source: state field of role_invocation #{i}]")
        lines.append(f"  {c2}")
        lines.append(f"[CLAIM | source: role field of role_invocation #{i}]")
        lines.append(f"  {c3}")
        lines.append(f"[PROVENANCE] invocation #{i}: {json.dumps(inv, default=str, sort_keys=True)}")
        claims += [(c1, "session", i), (c2, "state", i), (c3, "role", i)]
    trace = "; ".join(
        f"iteration {inv.get('iteration_no')} declared admitted at t={inv.get('admitted_at')}"
        for inv in invs)
    if trace:
        trace += "; no further invocations are recorded in this run's pre-terminal records"
    else:
        trace = "none recorded"
    lines.append("[OBSERVATION | source: role_invocation records, sequence]")
    lines.append(f"  Observed invocation trace: {trace}.")
    lines.append("[OBSERVATION | source: correction_selection records, count]")
    lines.append(f"  Observed correction selections recorded: {case['corrections']}.")
    lines.append("[RELATION]")
    if not claims:
        lines.append("  (no claims recorded)")
    for text, kind, i in claims:
        lines.append(f"  claim({text}) vs observed trace: {_relation(kind, i, invs)}")
    return "\n".join(lines)


def _relation(kind, i, invs):
    # Frozen rules R1-R3; indices are 1-based over invs.
    if kind == "state":
        if i <= len(invs):
            inv = invs[i - 1]
            it = inv.get("iteration_no")
            at = inv.get("admitted_at")
            if isinstance(it, int) and it > 0 and at is not None:
                return "supports"
            return "contradicts"
        return "unresolved"
    if kind == "role":
        # R2: the claim text is read from the same payload it claims about,
        # so the mechanical check is payload-carries-role == claim (always
        # supports when the invocation exists); kept explicit so the relation
        # is derived, never asserted.
        return "supports" if i <= len(invs) else "unresolved"
    if kind == "session":
        if invs:
            vals = {inv.get("session_ref") for inv in invs}
            return "supports" if len(vals) == 1 else "unresolved"
        return "unresolved"
    return "unresolved"


def render_b(case):
    body = _claim_blocks(case)
    return HEADER_COMMON + HEADER_B_EXTRA + "\n" + body


# -------------------------------------------------------------- teeth -------
def _body(prompt):
    return prompt.split("\n\n", 1)[1]


def _atoms_of_structure(struct):
    atoms = set()
    def walk(v):
        if isinstance(v, dict):
            for x in v.values(): walk(x)
        elif isinstance(v, list):
            for x in v: walk(x)
        elif v is not None:
            atoms.add(str(v))
    walk(struct)
    return atoms


def content_equivalence(a_blob, b_body):
    """Set equality both directions between A-blob atoms and B-body atoms.
    B atoms: leaves of PROVENANCE payloads + quoted claim values + trace
    iteration/t values + the corrections count. Returns (ok, detail)."""
    a_struct = json.loads(a_blob)
    a_atoms = _atoms_of_structure(a_struct)
    b_atoms = set()
    for m in re.finditer(r"^\[PROVENANCE\] invocation #\d+: (\{.*\})$", b_body, re.M):
        b_atoms |= _atoms_of_structure(json.loads(m.group(1)))
    for m in re.finditer(r'declared as "([^"]*)"\.', b_body):
        b_atoms.add(m.group(1))
    for m in re.finditer(r'with state "([^"]*)"\.', b_body):
        b_atoms.add(m.group(1))
    for m in re.finditer(r'declares role "([^"]*)"\.', b_body):
        b_atoms.add(m.group(1))
    for m in re.finditer(r"iteration (\d+) declared admitted at t=([^;.]+)", b_body):
        b_atoms.add(m.group(1)); b_atoms.add(m.group(2))
    for m in re.finditer(r"correction selections recorded: (\d+)\.", b_body):
        b_atoms.add(m.group(1))
    missing = a_atoms - b_atoms
    added = b_atoms - a_atoms
    return (not missing and not added), {"missing_from_B": sorted(missing)[:8],
                                         "added_in_B": sorted(added)[:8],
                                         "n_a_atoms": len(a_atoms), "n_b_atoms": len(b_atoms)}


def role_vocabulary_validate(b_body):
    problems = []
    for m in re.finditer(r"^\[([A-Z_]+)", b_body, re.M):
        label = m.group(1)
        if label not in ROLE_LABELS:
            problems.append(f"role label not in frozen vocabulary: {label}")
        low = label.lower()
        for tok in BANNED_LABEL_TOKENS:
            if tok in low:
                problems.append(f"banned token '{tok}' in role label {label}")
    for m in re.finditer(r"vs observed trace: (\w+)", b_body):
        val = m.group(1)
        if val not in RELATION_VALUES:
            problems.append(f"relation value not in frozen vocabulary: {val}")
        for tok in BANNED_LABEL_TOKENS:
            if tok in val:
                problems.append(f"banned token '{tok}' in relation value {val}")
    return problems


def horizon_check(prompt):
    body = _body(prompt)
    problems = []
    for tok in TRUTH_TOKENS:
        if tok in body:
            problems.append(f"truth/post-judgment token in body: {tok}")
    return problems


def run_teeth(cases, store_check=True):
    """Execute the four frozen teeth. Each: real renderings must PASS the
    validator AND the simulated attack must be REJECTED."""
    results = {}
    by_id = {c["case_id"]: c for c in cases}
    # T27-1 content equivalence
    t1 = {"real": {}, "attack_rejected": False}
    ok_all = True
    for c in cases:
        blob = json.dumps(a_structure(c), default=str)[:12000]
        body = _body(render_b(c))
        ok, detail = content_equivalence(blob, body)
        t1["real"][c["case_id"]] = {"pass": ok, **detail}
        ok_all &= ok
        if c["case_id"] == TARGET:
            injected = body + f"\n[PROVENANCE] invocation #99: {json.dumps({'fabricated_digest': 'sha256:deadbeef' + '0' * 55})}"
            ok_inj, dinj = content_equivalence(blob, injected)
            t1["attack_rejected"] = not ok_inj
            t1["attack_detail"] = dinj
    results["T27-1_content_equivalence"] = {
        "pass": ok_all and t1["attack_rejected"], **t1}
    # T27-2 role vocabulary
    t2 = {"real": {}, "attack_rejected": False}
    ok_all = True
    for c in cases:
        probs = role_vocabulary_validate(_body(render_b(c)))
        t2["real"][c["case_id"]] = "pass" if not probs else probs
        ok_all &= not probs
    bad_body = _body(render_b(by_id[TARGET])).replace("[CLAIM |", "[CONTRADICTED_CLAIM |", 1)
    bad_body += "\n  claim(X) vs observed trace: claim_is_false"
    probs_bad = role_vocabulary_validate(bad_body)
    t2["attack_rejected"] = bool(probs_bad)
    t2["attack_detail"] = probs_bad[:4]
    results["T27-2_role_vocabulary"] = {"pass": ok_all and t2["attack_rejected"], **t2}
    # T27-3 temporal horizon
    t3 = {"real": {}, "attack_rejected": False}
    ok_all = True
    for c in cases:
        pa, pb = render_a(c), render_b(c)
        probs = horizon_check(pa) + horizon_check(pb)
        t3["real"][c["case_id"]] = "pass" if not probs else probs
        ok_all &= not probs
    injected_b = render_b(by_id[TARGET]) + "\nrun_terminal state budget_exhausted"
    probs_bad = horizon_check(injected_b)
    t3["attack_rejected"] = bool(probs_bad)
    t3["attack_detail"] = probs_bad[:4]
    results["T27-3_temporal_horizon"] = {"pass": ok_all and t3["attack_rejected"], **t3}
    # T27-4 case identity (prompt-hash key binding; verified against store at score time)
    t4 = {"key_binding": "sha256(prereg_sha|case_id|condition|rep|prompt_hash)[:16]",
          "store_consistent": None, "attack_rejected": False}
    b_c3 = render_b(by_id[TARGET]); b_a1 = render_b(by_id["accepted-A1"])
    k_real_c3 = obs_key(TARGET, "B", 0, b_c3)
    k_swap = obs_key(TARGET, "B", 0, b_a1)  # swapped body under C3's identity
    t4["attack_rejected"] = (k_real_c3 != k_swap) and (b_c3 != b_a1)
    t4["pass"] = t4["attack_rejected"]
    results["T27-4_case_identity"] = t4
    return results


# ------------------------------------------------------------ store/calls ---
def obs_key(cid, cond, rep, prompt):
    ph = hashlib.sha256(prompt.encode()).hexdigest()[:12]
    return hashlib.sha256(f"{PREREG_SHA}|{cid}|{cond}|{rep}|{ph}".encode()).hexdigest()[:16]


def load_store():
    seen = {}
    if os.path.exists(STORE):
        for line in open(STORE):
            r = json.loads(line)
            seen[r["key"]] = r
    return seen


def persist(rec):
    with open(STORE, "a") as f:
        f.write(json.dumps(rec, default=str) + "\n")
        f.flush()
        os.fsync(f.fileno())


def parse(raw):
    m = re.search(r"\{.*\}", raw, re.DOTALL)
    if not m:
        return {"status": "schema_failure"}
    try:
        obj = json.loads(m.group(0))
    except json.JSONDecodeError:
        return {"status": "schema_failure"}
    if obj.get("answer") not in DOMAIN:
        return {"status": "out_of_domain", "raw_answer": obj.get("answer")}
    dd = {l: max(0.0, min(1.0, float((obj.get("distribution") or {}).get(l, 0.0)))) for l in DOMAIN}
    tt = sum(dd.values())
    return {"status": "ok", "answer": obj["answer"],
            "confidence": max(0.0, min(1.0, float(obj.get("confidence", 0) or 0))),
            "distribution": {k: v / tt for k, v in dd.items()} if tt else dd,
            "reason": str(obj.get("reason", ""))[:300]}


def call_provider(prompt):
    for attempt in range(3):
        try:
            proc = subprocess.run(PROVIDER_CMD + [prompt], capture_output=True,
                                  text=True, timeout=300, cwd="/tmp")
        except subprocess.TimeoutExpired:
            if attempt < 2:
                time.sleep(5); continue
            return {"status": "provider_timeout", "raw": ""}
        if proc.returncode == 0:
            return {"status": "raw", "raw": proc.stdout}
        if attempt < 2:
            time.sleep(5); continue
        return {"status": "provider_error", "raw": (proc.stderr or proc.stdout)[-400:]}
    return {"status": "provider_error", "raw": ""}


def phase_acquire(cases, prompts):
    store = load_store()
    made = reused = 0
    for c in cases:
        cid = c["case_id"]
        for rep in range(R):
            for cond in ("A", "B"):
                prompt = prompts[cid][cond]
                k = obs_key(cid, cond, rep, prompt)
                if k in store:
                    reused += 1
                    continue
                resp = call_provider(prompt)
                parsed = parse(resp["raw"]) if resp["status"] == "raw" else {"status": resp["status"]}
                parsed.update({"key": k, "case_id": cid, "condition": cond, "rep": rep,
                               "provider": PROVIDER_ID,
                               "prompt_hash": hashlib.sha256(prompt.encode()).hexdigest()[:12],
                               "ts": time.time(),
                               "raw": (resp.get("raw") or "")[:4000]})
                persist(parsed)
                made += 1
                print(f"  [acquire] {cid} rep{rep} {cond} -> {parsed.get('status')}", flush=True)
    print(f"[acquire] made={made} reused={reused} store={len(load_store())}", flush=True)
    return made, reused


# ------------------------------------------------------------- scoring ------
def case_readout(records):
    valid = [r["answer"] for r in records if r.get("status") == "ok" and r.get("answer")]
    degraded = False
    if len(valid) < 4:
        degraded = True
        return ABSTAIN, degraded, valid
    counts = Counter(valid)
    top = counts.most_common(2)
    if len(top) > 1 and top[0][1] == top[1][1]:
        return ABSTAIN, degraded, valid
    return top[0][0], degraded, valid


def analyze(cases, prompts, store):
    per_case = []
    for c in cases:
        cid = c["case_id"]
        row = {"case_id": cid, "truth": c["truth"],
               "category": ("stable-wrong" if cid in STABLE_WRONG else
                            "unstable-frontier" if cid in UNSTABLE else "control"),
               "conflict_or_control": ("conflict" if cid in FRONTIER else "control"),
               "family": FAMILY[cid.split("-")[0]]}
        for cond in ("A", "B"):
            prompt = prompts[cid][cond]
            k_exp = {obs_key(cid, cond, rep, prompt) for rep in range(R)}
            recs = [store[k] for k in k_exp if k in store]
            if len(recs) != R:
                row[f"{cond}_missing"] = R - len(recs)
            readout, degraded, valid = case_readout(recs)
            row[f"{cond}_readout"] = readout
            row[f"{cond}_degraded"] = degraded
            row[f"{cond}_per_call"] = valid
            row[f"{cond}_correct"] = readout == c["truth"]
        a, b = row["A_readout"], row["B_readout"]
        a_ok, b_ok = row["A_correct"], row["B_correct"]
        a_dec, b_dec = a != ABSTAIN, b != ABSTAIN
        if a == b:
            tr = "unchanged-correct" if b_ok else "unchanged-wrong"
        elif not a_ok and b_ok:
            tr = "correction (decided-only)" if (a_dec and b_dec) else "abstention-transition gain"
        elif a_ok and not b_ok:
            tr = "regression (decided-only)" if (a_dec and b_dec) else "abstention-transition loss"
        else:
            tr = "wrong->wrong flip"
        row["A_to_B_transition"] = tr
        per_case.append(row)
    n = len(per_case)
    b = sum(1 for r in per_case if not r["A_correct"] and r["B_correct"])
    c = sum(1 for r in per_case if r["A_correct"] and not r["B_correct"])
    fr = [r for r in per_case if r["conflict_or_control"] == "conflict"]
    ct = [r for r in per_case if r["conflict_or_control"] == "control"]
    b_f = sum(1 for r in fr if not r["A_correct"] and r["B_correct"])
    c_f = sum(1 for r in fr if r["A_correct"] and not r["B_correct"])
    c_ctrl = sum(1 for r in ct if r["A_correct"] and not r["B_correct"])
    def mcnemar_two(bb, cc):
        tot = bb + cc
        if tot == 0: return 1.0
        tail = sum(comb(tot, k) for k in range(0, min(bb, cc) + 1)) / (2 ** tot)
        return min(1.0, 2 * tail)
    def mcnemar_one_improve(bb, cc):
        tot = bb + cc
        if tot == 0: return 1.0
        return sum(comb(tot, k) for k in range(0, min(bb, cc) + 1)) / (2 ** tot)
    p_two = mcnemar_two(b, c)
    p_one_imp = mcnemar_one_improve(b, c)
    p_one_harm = mcnemar_one_improve(c, b)
    target_row = next(r for r in per_case if r["case_id"] == TARGET)
    T = 1 if (not target_row["A_correct"] and target_row["B_correct"]) else 0
    net = b - c
    frontier_net = b_f - c_f
    practical = (net >= 3) and (frontier_net >= 2)
    # frozen ordered C1-C5 (addendum-1 corrected b/c)
    degraded_cells = sum(1 for r in per_case for cond in ("A", "B") if r[f"{cond}_degraded"])
    valid = not degraded_cells and all(r.get(f"{cond}_missing", 0) == 0
                                       for r in per_case for cond in ("A", "B"))
    if not valid:
        disposition, reason = "C5", f"validity gate failed (degraded/missing cells={degraded_cells})"
    elif c_ctrl >= 2 or (c > b and p_one_harm < 0.05):
        disposition, reason = "C4", f"c_ctrl={c_ctrl}, c={c}, b={b}, p_harm={p_one_harm:.4f}"
    elif p_two < 0.05 and net >= 3 and frontier_net >= 2 and c_ctrl == 0:
        disposition, reason = "C1", f"p_two={p_two:.5f}, net={net}, frontier_net={frontier_net}, c_ctrl=0"
    elif T == 1 and frontier_net >= 2 and c_ctrl == 0:
        disposition, reason = "C2", f"target corrected, frontier_net={frontier_net}, c_ctrl=0, p_two={p_two:.5f} (not significant or practical criterion unmet for C1)"
    elif T == 0 and b_f == 0 and c_f == 0 and c_ctrl == 0:
        disposition, reason = "C3", "no frontier or control movement"
    else:
        disposition, reason = "C5", ("boundary/inconclusive: "
            f"p_two={p_two:.5f}, net={net}, frontier_net={frontier_net}, T={T}, c_ctrl={c_ctrl}")
    acc = {cond: sum(1 for r in per_case if r[f"{cond}_correct"]) / n for cond in ("A", "B")}
    return {"per_case": per_case, "b": b, "c": c, "b_f": b_f, "c_f": c_f,
            "c_ctrl": c_ctrl, "p_two": p_two, "p_one_improvement": p_one_imp,
            "p_one_harm": p_one_harm, "T": T, "net": net, "frontier_net": frontier_net,
            "practical": practical, "disposition": disposition, "reason": reason,
            "accuracy": acc, "baseline": sum(1 for r in per_case if r["truth"] == "settled") / n,
            "degraded_cells": degraded_cells}


def build_report(cases, prompts, store, teeth, note=""):
    res = analyze(cases, prompts, store)
    import yaml
    out = {
        "schema": "godspeed-bounded-judgment.t27-paired-report",
        "created_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "prereg_sha256": PREREG_SHA,
        "addendum1_sha256": ADDENDUM_SHA,
        "provider": PROVIDER_ID,
        "n": len(cases),
        "repetitions": R,
        "note": note,
        "primary_analysis": {
            "unit": "case-level unique-modal readout (abstain = not correct, per addendum-1)",
            "corrections_b": res["b"], "regressions_c": res["c"],
            "frontier_b_f": res["b_f"], "frontier_c_f": res["c_f"],
            "control_regressions_c_ctrl": res["c_ctrl"],
            "mcnemar_exact_two_sided_p": round(res["p_two"], 6),
            "mcnemar_one_sided_improvement_p": round(res["p_one_improvement"], 6),
            "mcnemar_one_sided_harm_p": round(res["p_one_harm"], 6),
            "alpha": 0.05,
            "target_case_budget_C3_corrected": bool(res["T"]),
            "net": res["net"], "frontier_net": res["frontier_net"],
            "practical_effect": res["practical"],
            "regression_guard": {"hard_line_c_ctrl_gte_2": res["c_ctrl"] >= 2,
                                 "soft_line_c_ctrl_eq_1": res["c_ctrl"] == 1,
                                 "pass_for_C1_C2": res["c_ctrl"] == 0},
            "accuracy_A": round(res["accuracy"]["A"], 4),
            "accuracy_B": round(res["accuracy"]["B"], 4),
            "baseline_constant_settled": round(res["baseline"], 4),
        },
        "disposition": {"letter": res["disposition"], "mechanical_reason": res["reason"]},
        "per_case": res["per_case"],
        "teeth": {k: ({"pass": v["pass"]} if k.startswith(("T27-1", "T27-2", "T27-3"))
                      else {"store_consistent": v.get("store_consistent"),
                            "attack_rejected": v.get("attack_rejected")})
                  for k, v in (teeth or {}).items()},
    }
    with open(REPORT, "w") as f:
        yaml.safe_dump(out, f, sort_keys=False, width=110)
    return out


def verify_store_consistency(cases, prompts, store):
    """T27-4 at score time: every stored record must key-match a recomputed
    prompt hash for its (case, condition)."""
    problems = []
    for r in store.values():
        cid, cond = r.get("case_id"), r.get("condition")
        if cid not in {c["case_id"] for c in cases} or cond not in ("A", "B"):
            problems.append(f"record {r.get('key')}: unknown case/condition")
            continue
        prompt = prompts[cid][cond]
        k = obs_key(cid, cond, r.get("rep", -1), prompt)
        if k != r["key"]:
            problems.append(f"record {r['key']}: key mismatch for {cid}/{cond}/rep{r.get('rep')}")
    return problems


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--teeth", action="store_true")
    ap.add_argument("--condition", choices=["both"])
    ap.add_argument("--score", action="store_true")
    ap.add_argument("--verify-only", action="store_true")
    args = ap.parse_args()

    problems = verify_corpus()
    if problems:
        print("[S3] CORPUS IDENTITY FAILURE:", *problems, sep="\n  ")
        return 3
    print(f"[corpus] {len(SELECTED)} DBs verified against the frozen prereg sha256s")

    cases = [load_case(cid) for cid in SELECTED]
    prompts = {c["case_id"]: {"A": render_a(c), "B": render_b(c)} for c in cases}
    # assembler horizon guard on every real prompt (S4)
    for cid, pp in prompts.items():
        pa = horizon_check(pp["A"]); pb = horizon_check(pp["B"])
        if pa or pb:
            print(f"[S4] LEAKAGE in {cid}: {pa + pb}")
            return 4

    if args.teeth or args.condition == "both" or args.verify_only:
        teeth = run_teeth(cases)
        teeth["T27-4_case_identity"]["store_consistent"] = None
        import yaml
        with open(TEETH_LOG, "w") as f:
            yaml.safe_dump(teeth, f, sort_keys=False, width=110)
        all_pass = all(teeth[k]["pass"] for k in
                       ("T27-1_content_equivalence", "T27-2_role_vocabulary",
                        "T27-3_temporal_horizon", "T27-4_case_identity")
                       if "pass" in teeth[k])
        print(f"[teeth] " + " ".join(
            f"{k.split('_')[0]}={'PASS' if v.get('pass') else 'FAIL'}" for k, v in teeth.items()))
        if not all_pass:
            print("[teeth] a frozen tooth FAILED — stopping (validity gate)")
            return 5
        if args.teeth and args.condition is None and not args.score and not args.verify_only:
            return 0

    if args.condition == "both":
        store = load_store()
        if store:
            print(f"[store] existing observations present ({len(store)}); acquiring only missing cells")
        phase_acquire(cases, prompts)
        store = load_store()
        problems = verify_store_consistency(cases, prompts, store)
        if problems:
            print("[T27-4/score-time] store key mismatches:", *problems[:10], sep="\n  ")
            return 6
        teeth = run_teeth(cases)
        teeth["T27-4_case_identity"]["store_consistent"] = not problems
        report = build_report(cases, prompts, store, teeth)
        d = report["disposition"]; p = report["primary_analysis"]
        print(f"[report] A={p['accuracy_A']} B={p['accuracy_B']} b={p['corrections_b']} "
              f"c={p['regressions_c']} p_two={p['mcnemar_exact_two_sided_p']} "
              f"c_ctrl={p['control_regressions_c_ctrl']} -> {d['letter']}")
        return 0

    if args.score or args.verify_only:
        store = load_store()
        expected = 2 * R * len(cases)
        if len(store) < expected:
            print(f"[verify] store incomplete: {len(store)}/{expected}")
            return 7
        problems = verify_store_consistency(cases, prompts, store)
        if problems:
            print("[verify] store key mismatches:", *problems[:10], sep="\n  ")
            return 6
        teeth = run_teeth(cases)
        teeth["T27-4_case_identity"]["store_consistent"] = not problems
        report = build_report(cases, prompts, store, teeth,
                              note="recomputed from persisted observations (verify-only)" if args.verify_only else "")
        d = report["disposition"]; p = report["primary_analysis"]
        print(f"[report] A={p['accuracy_A']} B={p['accuracy_B']} b={p['corrections_b']} "
              f"c={p['regressions_c']} p_two={p['mcnemar_exact_two_sided_p']} "
              f"c_ctrl={p['control_regressions_c_ctrl']} -> {d['letter']}")
        return 0

    ap.print_help()
    return 1


if __name__ == "__main__":
    sys.exit(main())
