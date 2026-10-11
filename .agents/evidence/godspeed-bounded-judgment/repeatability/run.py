#!/usr/bin/env python3
"""Repeatability experiment: R=5 independent DeepSeek judgments per case × 25 cases."""
import json, math, os, sqlite3, hashlib, subprocess, sys, time
from collections import Counter
from math import comb

DOMAIN = ["settled", "not_settled", "insufficient_evidence"]
CORPUS = os.path.expanduser("~/.local/share/godspeed-t16-corpus")
OUT = os.path.dirname(os.path.abspath(__file__))
STORE = os.path.join(OUT, "observations.jsonl")
PREREG_SHA = "a83b7e59c605cc7aeafbc134769f160de13855875b0c31bfac02672b84c0d18d"
R = 5
PROVIDER_CMD = ["opencode", "run", "-m", "cline-pass/cline-pass/deepseek-v4.1-flash"]
PROVIDER_ID = "deepseek-v4.1-flash-via-clinepass"

def key(cid, rep):
    return hashlib.sha256(f"{PREREG_SHA}|{cid}|rep{rep}".encode()).hexdigest()[:16]

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

def load_cases():
    cases = []
    runs_dir = os.path.join(CORPUS, "runs")
    for cid in sorted(os.listdir(runs_dir)):
        db = os.path.join(runs_dir, cid, "state", "gauntlet-state.db")
        conn = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
        term = json.loads(conn.execute("select payload from event_log where kind='run_terminal'").fetchone()[0])
        truth = {"settled": "settled", "budget_exhausted": "not_settled"}[term["state"]]
        invocations = []
        for (p,) in conn.execute("select payload from records where record_type='role_invocation' order by seq"):
            invocations.append(json.loads(p))
        corrections = conn.execute("select count(*) from records where record_type='correction_selection'").fetchone()[0]
        blob = json.dumps({"invocations": invocations, "corrections": corrections}, default=str)[:12000]
        cases.append({"case_id": cid, "truth": truth, "blob": blob})
        conn.close()
    return cases

def prompt_for(case):
    return (
        "You are doing a bounded judgment task. Answer STRICTLY with one JSON object.\n"
        "Question: Will this run settle accepted, or end non-settled (budget exhausted)?\n"
        "Answer domain: settled, not_settled, insufficient_evidence.\n"
        "JSON shape: {\"answer\": <label>, \"confidence\": <0..1>, \"distribution\": "
        "{<label>: <prob> for all three}, \"reason\": <one sentence>}\n\n"
        + case["blob"]
    )

def parse(raw):
    import re
    m = re.search(r"\{.*\}", raw, re.DOTALL)
    if not m: return {"status": "schema_failure"}
    try: obj = json.loads(m.group(0))
    except json.JSONDecodeError: return {"status": "schema_failure"}
    if obj.get("answer") not in DOMAIN:
        return {"status": "out_of_domain", "raw_answer": obj.get("answer")}
    dd = {l: max(0.0, min(1.0, float((obj.get("distribution") or {}).get(l, 0.0)))) for l in DOMAIN}
    tt = sum(dd.values())
    return {"status": "ok", "answer": obj["answer"],
            "confidence": max(0.0, min(1.0, float(obj.get("confidence", 0) or 0))),
            "distribution": {k: v/tt for k, v in dd.items()} if tt else dd,
            "reason": str(obj.get("reason", ""))[:300]}

def phase_acquire(cases):
    store = load_store()
    made = reused = 0
    for case in cases:
        cid = case["case_id"]
        prompt = prompt_for(case)
        ph = hashlib.sha256(prompt.encode()).hexdigest()[:12]
        for rep in range(R):
            k = key(cid, rep)
            if k in store:
                reused += 1
                continue
            for attempt in range(3):  # bounded retry for infrastructure only
                try:
                    proc = subprocess.run(PROVIDER_CMD + [prompt], capture_output=True,
                                         text=True, timeout=300, cwd="/tmp")
                    if proc.returncode == 0:
                        parsed = parse(proc.stdout)
                        parsed.update({"key": k, "case_id": cid, "rep": rep,
                                       "provider": PROVIDER_ID, "prompt_hash": ph,
                                       "attempt": attempt, "ts": time.time()})
                        persist(parsed)
                        made += 1
                        break
                    elif attempt < 2:
                        time.sleep(5)
                    else:
                        persist({"key": k, "case_id": cid, "rep": rep,
                                 "status": "provider_error", "attempt": attempt,
                                 "provider": PROVIDER_ID, "ts": time.time()})
                        made += 1
                except subprocess.TimeoutExpired:
                    if attempt < 2: time.sleep(5)
                    else:
                        persist({"key": k, "case_id": cid, "rep": rep,
                                 "status": "provider_timeout", "attempt": attempt,
                                 "provider": PROVIDER_ID, "ts": time.time()})
                        made += 1
            print(f"  {cid} rep{rep} done", flush=True)
    print(f"[acquire] made={made} reused={reused} total_store={len(load_store())}")

def phase_score(cases):
    store = load_store()
    truth = [c["truth"] for c in cases]
    cids = [c["case_id"] for c in cases]
    n = len(cases)
    # per-rep per-provider metrics
    per_case = {}
    for case in cases:
        cid = case["case_id"]
        answers = []
        for rep in range(R):
            k = key(cid, rep)
            entry = store.get(k, {})
            answers.append(entry.get("answer"))
        correct = sum(1 for a, t in zip(answers, truth) if a and a == t)
        abstain = sum(1 for a in answers if a == "insufficient_evidence")
        modal_counts = Counter(a for a in answers if a)
        modal = modal_counts.most_common(1)[0][0] if modal_counts else None
        modal_is_unique = (len(modal_counts) > 0 and
                          list(modal_counts.values()).count(modal_counts.most_common(1)[0][1]) == 1)
        all_agree = len(set(a for a in answers if a)) <= 1
        agg_answer = modal if (modal and modal_is_unique) else "insufficient_evidence"
        agg_correct = (agg_answer == case["truth"])
        per_case[cid] = {
            "truth": case["truth"], "answers": answers,
            "correct_count": correct, "abstain_count": abstain,
            "modal": modal, "modal_unique": modal_is_unique,
            "all_agree": all_agree,
            "aggregate_answer": agg_answer, "aggregate_correct": agg_correct,
            "stability": max(Counter(a for a in answers if a).values()) / R if any(answers) else 0,
        }
    # per-repetition accuracy
    rep_accs = []
    for rep in range(R):
        correct = sum(1 for c in cases
                      if store.get(key(c["case_id"], rep), {}).get("answer") == c["truth"])
        rep_accs.append(correct / n)
    # provider-level aggregate
    all_answers = [store.get(key(c["case_id"], rep), {}).get("answer") for c in cases for rep in range(R)]
    answered = [a for a in all_answers if a]
    overall_acc = sum(1 for a, c in zip(all_answers, [t for c in cases for _ in range(R) for t in [c["truth"]]]) if a and a == c) / len(answered) if answered else 0
    agg_correct = sum(1 for c in per_case.values() if c["aggregate_correct"])
    agg_acc = agg_correct / n
    abstain_total = sum(c["abstain_count"] for c in per_case.values())
    # classify R1-R5
    single_accs = [rep_accs]
    mean_single = sum(rep_accs) / len(rep_accs)
    stable_acc = all(abs(ra - mean_single) < 0.15 for ra in rep_accs)
    unanimous = sum(1 for c in per_case.values() if c["all_agree"]) / n
    if stable_acc and mean_single >= 0.75:
        interpretation = "R1_stable_accurate"
    elif mean_single < 0.6 and agg_acc > mean_single + 0.1:
        interpretation = "R2_noisy_aggregation_helps"
    elif not stable_acc and agg_acc <= mean_single + 0.05:
        interpretation = "R3_unstable_aggregation_weak"
    elif stable_acc and mean_single < 0.5:
        interpretation = "R4_stable_errors"
    elif base_acc >= agg_acc:
        interpretation = "R5_baseline_superior"
    else:
        interpretation = "unresolved"
    summary = {
        "n": n, "repetitions": R,
        "single_call_mean_accuracy": round(mean_single, 4),
        "per_rep_accuracy": [round(a, 4) for a in rep_accs],
        "accuracy_stable": stable_acc,
        "unanimous_fraction": round(unanimous, 4),
        "overall_answered_accuracy": round(overall_acc, 4),
        "aggregate_accuracy": round(agg_acc, 4),
        "aggregate_coverage": round(sum(1 for c in per_case.values() if c["aggregate_answer"] != "insufficient_evidence") / n, 4),
        "baseline_accuracy": round(base_acc, 4),
        "total_abstentions": abstain_total,
        "interpretation": interpretation,
    }
    return per_case, summary

def phase_report(per_case, summary, cases):
    out = os.path.join(OUT, "repeatability-report.yml")
    # baseline
    baseline_preds = ["settled"] * len(cases)
    base_acc = sum(1 for b, c in zip(baseline_preds, cases) if b == c["truth"]) / len(cases)
    summary["baseline_accuracy"] = round(base_acc, 4)
    summary["baseline_note"] = "constant settled predictor (= class prior); face-value rule degenerates on this fixture"
    stable_correct = sum(1 for c in per_case.values() if c["all_agree"] and c["answers"][0] == c["truth"])
    stable_wrong = sum(1 for c in per_case.values() if c["all_agree"] and c["answers"][0] != c["truth"])
    stable_abstain = sum(1 for c in per_case.values() if c["all_agree"] and c["answers"][0] == "insufficient_evidence")
    unstable = sum(1 for c in per_case.values() if not c["all_agree"])
    summary["case_stability_partition"] = {
        "stable_correct": stable_correct, "stable_wrong": stable_wrong,
        "stable_abstain": stable_abstain, "unstable_flipping": unstable}
    with open(out, "w") as f:
        yaml.dump(summary, f, sort_keys=False, default_flow_style=False,
                  allow_unicode=True) if False else yaml.safe_dump(summary, f, sort_keys=False, width=120)
    print(f"[report] interpretation={summary['interpretation']} "
          f"single={summary['single_call_mean_accuracy']} agg={summary['aggregate_accuracy']} "
          f"baseline={base_acc:.4f} unanimous={summary['unanimous_fraction']}")

import yaml
def main():
    cases = load_cases()
    print(f"[load] {len(cases)} cases", flush=True)
    if "--acquire" in sys.argv or len(sys.argv) == 1:
        phase_acquire(cases)
    per_case, summary = phase_score(cases)
    phase_report(per_case, summary, cases)
    return 0

if __name__ == "__main__":
    sys.exit(main())
