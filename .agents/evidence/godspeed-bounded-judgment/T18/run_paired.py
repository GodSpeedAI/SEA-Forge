#!/usr/bin/env python3
"""T18 Stage 1 paired representation experiment.

Three independently rerunnable phases:
  acquire  — provider call -> durable persist (idempotent, resumable)
  score    — persisted raw observations -> parsed results -> metrics
  report   — scored evidence -> T18 paired report

Condition A = T16 expanded-round persisted answers (reused, not re-run).
Condition B = same data re-presented with epistemic-role tags; fresh calls.
"""
from __future__ import annotations
import json, math, os, sqlite3, sys, hashlib, argparse, yaml
from collections import Counter
from math import comb

REPO = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "../../../.."))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "../T16"))
import run_remeasure as rm

DOMAIN = ["settled", "not_settled", "insufficient_evidence"]
MANIFEST = os.path.join(REPO, ".agents/preregistrations/godspeed-bounded-judgment-T16.expanded-corpus-manifest.yml")
COND_A = os.path.join(REPO, ".agents/evidence/godspeed-bounded-judgment/T16/expanded-round/provider-raw-results.json")
STORE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "observations.jsonl")
PREREG_SHA = "cd2ff459ef1f81f5eca13889657de68f72a4c45fe69cd8901cdb9109c2704b26"
CORPUS = os.path.expanduser("~/.local/share/godspeed-t16-corpus")

def key(case_id, condition, provider, prompt_hash):
    return hashlib.sha256(f"{PREREG_SHA}|{case_id}|{condition}|{provider}|{prompt_hash}".encode()).hexdigest()[:16]

def load_store():
    seen = {}
    if os.path.exists(STORE):
        for line in open(STORE):
            r = json.loads(line)
            seen[r["key"]] = r
    return seen

def persist(key_id, record):
    with open(STORE, "a") as f:
        f.write(json.dumps({"key": key_id, **record}, default=str) + "\n")
        f.flush()
        os.fsync(f.fileno())

def load_rows_persistent():
    rows = []
    runs_dir = os.path.join(CORPUS, "runs")
    for cid in sorted(os.listdir(runs_dir)):
        db = os.path.join(runs_dir, cid, "state", "gauntlet-state.db")
        if not os.path.isfile(db): continue
        conn = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
        conn.row_factory = sqlite3.Row
        term = conn.execute("select payload from event_log where kind='run_terminal'").fetchone()
        terminal = json.loads(term["payload"])["state"] if term else "unknown"
        invocations = []
        for (p,) in conn.execute("select payload from records where record_type='role_invocation' order by seq"):
            invocations.append({"result_document": json.loads(p)})
        corrections = conn.execute("select count(*) from records where record_type='correction_selection'").fetchone()[0]
        evidence = []
        for r in conn.execute("select payload from event_log where kind='evidence_produced' order by offset"):
            d = json.loads(r["payload"])
            evidence.append({"locator": d.get("locator",""), "digest": d.get("evidence","")[:24]})
        truth = {"settled": "settled", "budget_exhausted": "not_settled"}[terminal]
        rows.append({"case_id": cid, "truth": truth, "invocations": invocations,
                     "corrections": corrections, "evidence": evidence})
        conn.close()
    return rows

def build_prompt_a(row):
    blob = json.dumps({"invocations": row["invocations"], "corrections": row["corrections"]}, default=str)[:12000]
    return rm.PROMPT2 if hasattr(rm, 'PROMPT2') else (
        "You are doing a bounded judgment task. Answer STRICTLY with one JSON object.\n"
        "Question: Will this run settle accepted, or end non-settled (budget exhausted)?\n"
        "Answer domain: settled, not_settled, insufficient_evidence.\n"
        "JSON shape: {\"answer\": <label>, \"confidence\": <0..1>, \"distribution\": "
        "{<label>: <prob> for all three}, \"reason\": <one sentence>}\n\n" + blob)

def build_prompt_b(row):
    parts = []
    for i, inv in enumerate(row["invocations"]):
        doc = inv.get("result_document", inv)
        parts.append(f"[builder_claim {i+1}] {json.dumps(doc, default=str)[:1500]}")
    for j, e in enumerate(row.get("evidence", [])):
        parts.append(f"[observed_artifact {j+1}] locator={e['locator']} digest={e['digest']}")
    parts.append(f"[correction_count] {row['corrections']}")
    return (
        "You are doing a bounded judgment task. Answer STRICTLY with one JSON object.\n"
        "Question: Will this run settle accepted, or end non-settled (budget exhausted)?\n"
        "Answer domain for \"answer\": settled, not_settled, insufficient_evidence.\n"
        "The information below is TAGGED by epistemic role. [builder_claim] means the "
        "builder ASSERTS this about its own work — it may or may not be accurate. "
        "[observed_artifact] is what the system ACTUALLY OBSERVED. [correction_count] is "
        "how many times the system retried.\n"
        "JSON shape: {\"answer\": <label>, \"confidence\": <0..1>, \"distribution\": "
        "{<label>: <prob> for all three}, \"reason\": <one sentence>}\n\n" + "\n".join(parts))

def call_provider(name, prompt):
    import subprocess
    if name == "deepseek-clinepass":
        cmd = ["opencode", "run", "-m", "cline-pass/cline-pass/deepseek-v4.1-flash", prompt]
    elif name == "muse-spark-free":
        cmd = ["opencode", "run", "-m", "opencode/muse-spark-1.3-contributor-free", prompt]
    else: raise ValueError(name)
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=300, cwd="/tmp")
    except subprocess.TimeoutExpired:
        return {"status": "provider_timeout", "raw": ""}
    if proc.returncode != 0:
        return {"status": "provider_error", "raw": (proc.stderr or proc.stdout)[-400:]}
    return {"status": "raw", "raw": proc.stdout}

def parse_t16(text):
    import re
    m = re.search(r"\{.*\}", text, re.DOTALL)
    if not m: return {"status": "schema_failure"}
    try: obj = json.loads(m.group(0))
    except json.JSONDecodeError: return {"status": "schema_failure"}
    if obj.get("answer") not in DOMAIN:
        return {"status": "out_of_domain", "raw_answer": obj.get("answer")}
    dd = {l: max(0.0, min(1.0, float((obj.get("distribution") or {}).get(l, 0.0)))) for l in DOMAIN}
    tt = sum(dd.values())
    return {"status": "ok", "answer": obj.get("answer"),
            "confidence": max(0.0, min(1.0, float(obj.get("confidence", 0) or 0))),
            "distribution": {k: v/tt for k, v in dd.items()} if tt else dd,
            "reason": str(obj.get("reason", ""))[:300]}

def parse_answer(text):
    m = __import__("re").search(r"\{.*\}", text, __import__("re").DOTALL)
    if not m: return {"status": "schema_failure"}
    try: obj = json.loads(m.group(0))
    except json.JSONDecodeError: return {"status": "schema_failure"}
    if obj.get("answer") not in DOMAIN: return {"status": "out_of_domain", "raw_answer": obj.get("answer")}
    dd = {l: max(0.0, min(1.0, float((obj.get("distribution") or {}).get(l, 0.0)))) for l in DOMAIN}
    tt = sum(dd.values())
    return {"status": "ok", "answer": obj.get("answer"),
            "confidence": max(0.0, min(1.0, float(obj.get("confidence", 0) or 0))),
            "distribution": {k: v/tt for k, v in dd.items()} if tt else dd,
            "reason": str(obj.get("reason", ""))[:300]}

def phase_acquire(rows, providers):
    store = load_store()
    calls_made = calls_reused = 0
    for row in rows:
        for cond_name, prompt_builder in [("A", build_prompt_a), ("B", build_prompt_b)]:
            prompt = prompt_builder(row)
            ph = hashlib.sha256(prompt.encode()).hexdigest()[:12]
            for pname in providers:
                k = key(row["case_id"], cond_name, pname, ph)
                if k in store:
                    calls_reused += 1
                    continue
                resp = call_provider(pname, prompt)
                parsed = parse_t16(resp["raw"]) if resp["status"] == "raw" else {"status": resp["status"]}
                parsed["key"] = k
                parsed["case_id"] = row["case_id"]
                parsed["condition"] = cond_name
                parsed["provider"] = pname
                persist(k, parsed)
                calls_made += 1
                print(f"  [acquire] {row['case_id']} {cond_name}/{pname} -> {parsed.get('status','?')}", flush=True)
    print(f"[acquire] calls_made={calls_made} calls_reused={calls_reused}")
    return calls_made, calls_reused

def phase_score(rows, providers):
    store = load_store()
    cond_a_raw = json.load(open(COND_A))
    truth = [r["truth"] for r in rows]
    case_ids = [r["case_id"] for r in rows]
    # condition A: reuse T16 persisted answers
    cond_a_answers = {p: [r.get("answer") for r in cond_a_raw["results"][p]] for p in providers}
    # condition B: parse from store
    cond_b_answers = {p: [] for p in providers}
    for i, cid in enumerate(case_ids):
        row = rows[i]
        prompt_b = build_prompt_b(row)
        ph = hashlib.sha256(prompt_b.encode()).hexdigest()[:12]
        for pname in providers:
            k = key(cid, "B", pname, ph)
            entry = store.get(k)
            if entry and entry.get("status") == "ok" and entry.get("answer"):
                cond_b_answers[pname].append(entry["answer"])
            else:
                cond_b_answers[pname].append(None)
    # score both conditions per provider
    baseline_preds = ["settled"] * n_rows(rows)  # face-value degenerates on this fixture
    report = {"n": n_rows(rows), "providers": {}}
    for pname in providers:
        for cond, ans in [("A", cond_a_answers[pname]), ("B", cond_b_answers[pname])]:
            pairs = [(a, t) for a, t in zip(ans, truth) if a]
            acc = sum(1 for a, t in pairs if a == t) / len(pairs) if pairs else 0
            abst = sum(1 for a in ans if a == "insufficient_evidence")
            ents = []
            report["providers"].setdefault(pname, {})[cond] = {
                "accuracy": round(acc, 4), "n_answered": len(pairs),
                "abstention_rate": round(abst / len(ans), 4) if ans else 0}
        a_ans = cond_a_answers[pname]; b_ans = cond_b_answers[pname]
        corr = sum(1 for a, b, t in zip(a_ans, b_ans, truth) if a != t and b == t)
        regr = sum(1 for a, b, t in zip(a_ans, b_ans, truth) if a == t and b != t)
        report["providers"][pname]["paired_deltas"] = {
            "A_to_B_corrections": corr, "A_to_B_regressions": regr, "net": corr - regr}
    return report, cond_a_answers, cond_b_answers, truth

def phase_report(rows, report, providers):
    truth = [r["truth"] for r in rows]
    n = len(rows)
    baseline_preds = ["settled"] * n
    base_acc = sum(1 for b, t in zip(baseline_preds, truth) if b == t) / n
    report["baseline"] = {"accuracy": round(base_acc, 4),
                          "note": "constant settled predictor = class prior 12/25"}
    for pname in providers:
        a_d = report["providers"][pname]["A"]; b_d = report["providers"][pname]["B"]
        a_imp = b_d["accuracy"] - a_d["accuracy"]
        report["providers"][pname]["improvement"] = round(a_imp, 4)
    report["stage1_decision"] = {
        "H_REPRESENTATION_supported": any(
            report["providers"][p]["improvement"] >= 0.1 for p in providers),
        "authorize_stage2": any(
            report["providers"][p]["improvement"] >= 0.1 for p in providers)}
    out = os.path.dirname(os.path.abspath(__file__))
    with open(os.path.join(out, "paired-report.yml"), "w") as f:
        yaml.safe_dump(report, f, sort_keys=False, width=120)
    print(f"[report] authorize_stage2={report['stage1_decision']['authorize_stage2']}")
    for pname in providers:
        pr = report["providers"][pname]
        print(f"  {pname}: A={pr['A']['accuracy']} B={pr['B']['accuracy']} "
              f"improvement={pr['improvement']} corrections={pr['paired_deltas']['A_to_B_corrections']} "
              f"regressions={pr['paired_deltas']['A_to_B_regressions']}")
    return report

def n_rows(rows): return len(rows)

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--phase", choices=["acquire", "score", "report", "all"], default="all")
    ap.add_argument("--providers", default="deepseek-clinepass,muse-spark-free")
    args = ap.parse_args()
    providers = args.providers.split(",")
    rows = load_rows_persistent()
    print(f"[load] {len(rows)} rows loaded from persistent corpus", flush=True)
    if args.phase in ("acquire", "all"):
        phase_acquire(rows, providers)
    if args.phase in ("score", "all"):
        report, _, _, _ = phase_score(rows, providers)
    if args.phase == "report" or args.phase == "all":
        report, _, _, _ = phase_score(rows, providers)
        phase_report(rows, report, providers)
    return 0

if __name__ == "__main__":
    sys.exit(main())
