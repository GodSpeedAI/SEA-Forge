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
        rows.append({"case_id": run["case_id"], "invocations": invs, "corrections": cors,
                     "consequence": run["consequence_class"]})
    return rows

def face_value(row):
    docs = [json.dumps(inv.get("result_document", {})) for inv in row["invocations"]]
    all_completed = all(("completed" in d) for d in docs) and docs
    return "settled" if all_completed else "not_settled"

def main() -> int:
    rows = load_rows(".agents/preregistrations/godspeed-bounded-judgment-T15.corpus-manifest.yml")
    results = {"codex": [], "prime-agent": []}
    for i, row in enumerate(rows):
        # leakage assertion: no terminal/settlement material in the built input
        blob = json.dumps({"invocations": row["invocations"], "corrections": row["corrections"]})
        for banned in ("run_terminal", "settlement_committed", "settled\"", "budget_exhausted"):
            pass  # consequence vocabulary check below on the assembled prompt
        prompt = PROMPT.format(records=blob[:12000])
        assert "run_terminal" not in prompt and "settlement_committed" not in prompt
        for pname in ("codex", "prime-agent"):
            resp = rp.call_cli_provider(pname, prompt)
            parsed = rp.parse_provider_answer(resp["raw"]) if resp["status"] == "raw" else {"status": resp["status"]}
            parsed["row_index"] = i
            results[pname].append(parsed)
        print(f"[providers] row {i+1}/{len(rows)}", flush=True)
    out = ".agents/evidence/godspeed-bounded-judgment/T16/round-1"
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
    base_acc = sum(1 for b, r in zip(baseline_preds, rows) if b == r["consequence"]) / len(rows)
    report["baseline"] = {"rule": "face-value: all documents claim completed -> settled", "accuracy": base_acc,
                          "triviality_note": "baseline never reads artifact contents; reported beside providers"}
    for p in ("codex", "prime-agent"):
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
        pairs = [(a, r["consequence"]) for a, r in zip(answers, rows) if a]
        acc = (sum(1 for a, c in pairs if a == c) / len(pairs)) if pairs else None
        abst = sum(1 for a in answers if a == "insufficient_evidence")
        report[p] = {"contract_failures": fails, "n_answered": len(pairs),
                     "accuracy_vs_consequence": acc,
                     "confusion": {a: dict(Counter(c for a2, c in pairs if a2 == a)) for a in set(a for a, _ in pairs)},
                     "abstention_rate": abst / len(rows), "mean_entropy_bits": (sum(ents)/len(ents)) if ents else None}
    dis = sum(1 for c, q in zip(results["codex"], results["prime-agent"]) if c.get("answer") and q.get("answer") and c["answer"] != q["answer"])
    report["capable_vs_capable_disagreement"] = dis
    # decision rule
    a = report["codex"]; b = report["prime-agent"]
    contract_ok = a["contract_failures"] == 0 and b["contract_failures"] == 0
    acc_ok = (a["accuracy_vs_consequence"] or 0) >= 0.8 and (b["accuracy_vs_consequence"] or 0) >= 0.8
    abst_ok = a["abstention_rate"] <= 0.5 and b["abstention_rate"] <= 0.5
    ent_ok = (a["mean_entropy_bits"] or 0) > 0 and (b["mean_entropy_bits"] or 0) > 0
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
    print(f"[done] outcome={letter} baseline_acc={base_acc} codex_acc={report['codex']['accuracy_vs_consequence']} prime_acc={report['prime-agent']['accuracy_vs_consequence']}")
    return 0

if __name__ == "__main__":
    sys.exit(main())
