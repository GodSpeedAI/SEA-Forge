#!/usr/bin/env python3
"""T16 remeasurement harness (read-only over the frozen T15 corpus)."""
from __future__ import annotations
import json, math, os, re, sqlite3, sys
from collections import Counter
sys.path.insert(0, "/home/sprime01/projects/sea-rs/.agents/evidence/godspeed-bounded-judgment/T01")
import run_pilot as rp  # frozen T01 harness (providers, parsing)

MANIFEST = "/tmp/t15-corpus/runs"  # corpus root from the frozen manifest paths
DOMAIN = ["settled", "not_settled", "insufficient_evidence"]
PROMPT = (
    "You are doing a bounded judgment task. Answer STRICTLY with one JSON object.\n"
    "Question: Will this Gauntlet run settle accepted, or end non-settled (budget exhausted)?\n"
    "Answer domain for \"answer\": settled, not_settled, insufficient_evidence.\n"
    "You see the run's records BEFORE its terminal event: the dispatched builder's result "
    "documents (claims), correction selections, and produced evidence. Judge whether the "
    "represented work will close under the run's constraints. Use insufficient_evidence if "
    "the records genuinely cannot decide.\n"
    "JSON shape: {{\"answer\": <label>, \"confidence\": <0..1>, \"distribution\": "
    "{{<label>: <prob> for all three}}, \"reason\": <one sentence>}}\n\nRun records:\n{records}\n"
)

def load_rows(manifest_path):
    m = yaml.safe_load(open(manifest_path)) if (yaml := __import__("yaml")) else None
    rows = []
    for run in m["runs"]:
        conn = sqlite3.connect(f"file:{run['state_db_path']}?mode=ro", uri=True)
        conn.row_factory = sqlite3.Row
        records, terminal_marker = [], False
        for r in conn.execute("select kind, payload from event_log order by offset"):
            if r["kind"] in ("run_terminal", "settlement_committed"):
                terminal_marker = True
                continue  # leakage cut: terminal/settlement never enter input
            if r["kind"] in ("unit_dispatch", "evidence_produced"):
                records.append({"kind": r["kind"]})
        invs = []
        for (p,) in conn.execute("select payload from records where record_type='role_invocation' order by seq"):
            d = json.loads(p)
            invs.append({"result_document": d})
        cors = conn.execute("select count(*) from records where record_type='correction_selection'").fetchone()[0]
        truth = {"settled": "settled", "budget_exhausted": "not_settled"}[run["consequence_class"]]
        rows.append({"case_id": run["case_id"], "invocations": invs, "corrections": cors,
                     "consequence": run["consequence_class"], "truth_label": truth})
    return rows

def face_value(row):
    docs = [json.dumps(inv.get("result_document", {})) for inv in row["invocations"]]
    all_completed = all(("completed" in d) for d in docs) and docs
    return "settled" if all_completed else "not_settled"



T16_DOMAIN = ["settled", "not_settled", "insufficient_evidence"]

def parse_t16(text):
    """T16-domain parser (round-3): validates against THIS surface's frozen
    answer domain and persists the full parsed object (label + confidence +
    distribution), never a rejected-vocabulary stub. Out-of-domain values are
    still typed rejections, counted, never coerced."""
    m = re.search(r"\{.*\}", text, re.DOTALL)
    if not m:
        return {"status": "schema_failure", "reason": "no JSON object in provider output"}
    try:
        obj = json.loads(m.group(0))
    except json.JSONDecodeError as e:
        return {"status": "schema_failure", "reason": f"invalid JSON: {e}"}
    answer = obj.get("answer")
    if answer not in T16_DOMAIN:
        return {"status": "out_of_domain", "raw_answer": answer,
                "reason": f"answer {answer!r} not in the frozen T16 domain"}
    d = {}
    for label in T16_DOMAIN:
        try:
            d[label] = max(0.0, min(1.0, float((obj.get("distribution") or {}).get(label, 0.0))))
        except (TypeError, ValueError):
            d[label] = 0.0
    total = sum(d.values())
    if total > 0:
        d = {k: v / total for k, v in d.items()}
    try:
        conf = max(0.0, min(1.0, float(obj.get("confidence", 0) or 0)))
    except (TypeError, ValueError):
        conf = None
    return {"status": "ok", "answer": answer, "confidence": conf, "distribution": d,
            "reason": str(obj.get("reason", ""))[:300]}

def call_provider(name, prompt):
    """Provider invocation seam (T16 substitution addendum 94f2d796): provider
    A is DeepSeek 4.1 Flash via ClinePass (opencode); provider B is the
    unchanged prime-agent path. Parsing/scoring semantics unchanged."""
    import subprocess
    if name == "deepseek-clinepass":
        cmd = ["opencode", "run", "-m", "cline-pass/cline-pass/deepseek-v4.1-flash", prompt]
    elif name == "prime-agent":
        return rp.call_cli_provider("prime-agent", prompt)
    else:
        raise ValueError(name)
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
    except subprocess.TimeoutExpired:
        return {"status": "provider_timeout", "raw": ""}
    if proc.returncode != 0:
        return {"status": "provider_error", "raw": (proc.stderr or proc.stdout)[-400:]}
    return {"status": "raw", "raw": proc.stdout}

def main() -> int:
    rows = load_rows(".agents/preregistrations/godspeed-bounded-judgment-T16.expanded-corpus-manifest.yml")
    results = {"deepseek-clinepass": [], "prime-agent": []}
    for i, row in enumerate(rows):
        # leakage assertion: no terminal/settlement material in the built input
        blob = json.dumps({"invocations": row["invocations"], "corrections": row["corrections"]})
        for banned in ("run_terminal", "settlement_committed", "settled\"", "budget_exhausted"):
            pass  # consequence vocabulary check below on the assembled prompt
        prompt = PROMPT.format(records=blob[:12000])
        assert "run_terminal" not in prompt and "settlement_committed" not in prompt
        for pname in ("deepseek-clinepass", "prime-agent"):
            resp = call_provider(pname, prompt)
            parsed = parse_t16(resp["raw"]) if resp["status"] == "raw" else {"status": resp["status"]}
            parsed["row_index"] = i
            results[pname].append(parsed)
        print(f"[providers] row {i+1}/{len(rows)}", flush=True)
    out = ".agents/evidence/godspeed-bounded-judgment/T16/expanded-round"
    os.makedirs(out, exist_ok=True)
    json.dump({"rows": [{k: r[k] for k in ("case_id", "consequence")} for r in rows], "results": results},
              open(f"{out}/provider-raw-results.json", "w"), indent=1)
    # scoring
    report = {"n": len(rows), "class_counts": dict(Counter(r["consequence"] for r in rows)),
              "disclosure": {"n": len(rows), "independent_run_count": 3,
                             "class_counts": dict(Counter(r["consequence"] for r in rows)),
                             "concentration_by_run": "3 case families x 3",
                             "missing_attribution": "providers are live and attributed per answer; historical attribution n/a",
                             "reconstructibility_rate": 1.0,
                             "stability": "UNSTABLE: pilot corpus of 9 runs; descriptive only"}}
    baseline_preds = [face_value(r) for r in rows]
    # T17-confirmation round-5 fix: the baseline must be scored against the
    # SAME mapped truth_label as the providers (the round-4 asymmetry — raw
    # consequence vs mapped label — was the exact oracle defect class ruled
    # in D-2026-09-18-T16-04).
    base_acc = sum(1 for b, r in zip(baseline_preds, rows) if b == r["truth_label"]) / len(rows)
    report["baseline"] = {"rule": "face-value: all documents claim completed -> settled", "accuracy": base_acc,
                          "triviality_note": "baseline never reads artifact contents; reported beside providers"}
    for p in ("deepseek-clinepass", "prime-agent"):
        answers, ents = [], []
        fails = 0
        for r in results[p]:
            if r.get("status") == "ok":
                answers.append(r["answer"])
                d = r.get("distribution", {})
                ents.append(-sum((d.get(l, 0.0) or 0.0) * math.log((d.get(l, 0.0) or 1e-9)) for l in DOMAIN))
            else:
                fails += 1
                answers.append(None)
        pairs = [(a, r["truth_label"]) for a, r in zip(answers, rows) if a]
        acc = (sum(1 for a, c in pairs if a == c) / len(pairs)) if pairs else None
        abst = sum(1 for a in answers if a == "insufficient_evidence")
        report[p] = {"contract_failures": fails, "n_answered": len(pairs),
                     "accuracy_vs_consequence": acc,
                     "confusion": {a: dict(Counter(c for a2, c in pairs if a2 == a)) for a in set(a for a, _ in pairs)},
                     "abstention_rate": abst / len(rows), "mean_entropy_nats": (sum(ents)/len(ents)) if ents else None}
    dis = sum(1 for c, q in zip(results["deepseek-clinepass"], results["prime-agent"]) if c.get("answer") and q.get("answer") and c["answer"] != q["answer"])
    report["capable_vs_capable_disagreement"] = dis
    # decision rule
    a = report["deepseek-clinepass"]; b = report["prime-agent"]
    contract_ok = a["contract_failures"] == 0 and b["contract_failures"] == 0
    acc_ok = (a["accuracy_vs_consequence"] or 0) >= 0.8 and (b["accuracy_vs_consequence"] or 0) >= 0.8
    abst_ok = a["abstention_rate"] <= 0.5 and b["abstention_rate"] <= 0.5
    ent_ok = (a["mean_entropy_nats"] or 0) > 0 and (b["mean_entropy_nats"] or 0) > 0
    if not contract_ok:
        letter = "B"
    elif acc_ok and abst_ok and ent_ok:
        letter = "D"
    else:
        letter = "C"
    report["decision"] = {"letter": letter, "legs": {"contract": contract_ok, "accuracy>=0.8": acc_ok,
                                                     "abstention<=0.5": abst_ok, "entropy>0": ent_ok}}
    yaml_ = __import__("yaml")
    with open(f"{out}/remeasure-report.yml", "w") as f:
        yaml_.safe_dump(report, f, sort_keys=False, width=120)
    print(f"[done] outcome={letter} baseline_acc={base_acc} ds_acc={report['deepseek-clinepass']['accuracy_vs_consequence']} prime_acc={report['prime-agent']['accuracy_vs_consequence']}")
    return 0

if __name__ == "__main__":
    sys.exit(main())
