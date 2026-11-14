#!/usr/bin/env python3
"""S1 claim-ledger builder (WP-19). Mechanical: results file + frozen predictions + frozen tests in, ledger out.

  scripts/fde-claim-ledger.py --results .tmp/fde-proof/s1/results.json [--cell <cell>] [--out-dir <dir>]

No verdict is written by hand. Every row is computed from `results.json` (written by scripts/fde-proof.sh) and
.agents/preregistrations/fde-s1/{tests,predictions,episodes}.yaml. A frozen test missing from the results makes
the builder fail (exit 1): a missing result is never read as a pass. A prediction mismatch is reported as a
mismatch; predictions are never rewritten. Output: claim-ledger.md and claim-ledger.json in --out-dir.
"""
import argparse, json, pathlib, re, sys, yaml

ROOT = pathlib.Path(__file__).resolve().parents[1]
PREREG = ROOT / ".agents/preregistrations/fde-s1"
STATUS_MD = pathlib.Path(__import__("os").environ.get("FDE_STATUS_MD", str(pathlib.Path.home() / "projects/sea-rs/.agents/reports/fde-convergence-2026-11-12/STATUS.md")))
CLAIM = "the Mission §1–7 path exists and runs once on program execution."  # G section 2 (RD-20), exactly
NOT_EVIDENCE = "It is not evidence about organisational behaviour."
VOCAB = ("PASS", "FAIL", "PARTIAL", "NOT_TESTED", "NOT_RUN")
AREAS = [  # G section 7 item 6 areas -> member tests (frozen S1 ids)
    ("initial semantic modelling", ["AT-01"]), ("provenance (external 'before')", ["AT-04", "NT-01", "NT-16"]),
    ("semantic exhaust", ["AT-05", "NT-02"]), ("observed-model derivation", ["AT-06", "AT-08", "NT-03", "NT-03b"]),
    ("divergence classification", ["AT-07", "NT-04", "NT-06"]), ("governed revision", ["AT-12", "NT-05", "NT-07", "NT-08"]),
    ("subsequent operational capability", ["AT-13"]), ("Horizon inspection (read adapters)", ["AT-14", "NT-09"]),
    ("reproducibility and independent 'before'", ["AT-18"]),
]
PLANNED = [("transfer", "AT-16 is an S5 fixture-based test; not in the S1 gate"), ("installation", "clean-cell install from release artifacts is WP-57; not in the S1 gate")]


def open_limits():
    """The 'STILL OPEN' list of STATUS.md '2026-11-14 ~20:30Z - review fixes landed', verbatim."""
    text = STATUS_MD.read_text()
    m = re.search(r"## 2026-11-14 ~20:30Z.*?STILL OPEN[^\n]*\n(.*?)(?:\n## |\Z)", text, re.S)
    if not m: sys.exit(f"claim-ledger: the open-limits list was not found in {STATUS_MD}")
    items = [l.strip()[2:].strip() for l in m.group(1).splitlines() if l.strip().startswith("* ") or l.strip().startswith("- ")]
    items = [i for i in items if not i.startswith("WP-20 contract inputs")]
    if len(items) < 7: sys.exit(f"claim-ledger: expected 7 open limits in {STATUS_MD}, found {len(items)}")
    return items


def area_status(member_status):
    s = set(member_status)
    if "FAIL" in s: return "FAILED"
    if s == {"PASS"}: return "PROVEN"
    if s <= {"NOT_RUN", "NOT_TESTED"}: return "NOT TESTED"
    return "PARTIAL"


def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--results", required=True); ap.add_argument("--cell"); ap.add_argument("--out-dir", required=True)
    a = ap.parse_args()
    res_doc = json.loads(pathlib.Path(a.results).read_text())
    tests = yaml.safe_load((PREREG / "tests.yaml").read_text())["tests"]
    preds = {p["id"]: p for p in yaml.safe_load((PREREG / "predictions.yaml").read_text())["predictions"]}
    by_id = {r["id"]: r for r in res_doc["results"] if not r["id"].startswith("preflight:")}
    missing = [t["id"] for t in tests if t["id"] not in by_id]
    if missing: sys.exit(f"claim-ledger: frozen tests missing from the results: {', '.join(missing)}")
    unknown = [i for i, r in by_id.items() if r["status"] not in VOCAB]
    if unknown: sys.exit(f"claim-ledger: status outside the vocabulary for {unknown}")
    cell = json.loads((pathlib.Path(a.cell) / "state.json").read_text()) if a.cell else None
    rows = []
    for t in tests:
        r = by_id[t["id"]]; p = preds[t["id"]]
        rows.append({"id": t["id"], "statement": t["statement"], "prediction": p["status"], "prediction_confidence": p.get("confidence"), "observed": r["status"],
                     "match": "match" if r["status"] == p["status"] else "MISMATCH", "reason": r.get("reason", ""), "exit_code": r["exit_code"], "duration_s": r.get("duration_s"),
                     "command": t["command"], "evidence": r.get("evidence", ""), "prediction_reason": p["reason"]})
    counts = {s: sum(1 for r in rows if r["observed"] == s) for s in VOCAB}
    expected = yaml.safe_load((PREREG / "predictions.yaml").read_text())["expected_counts"]
    areas = []
    for name, members in AREAS:
        st = area_status([by_id[m]["status"] for m in members]); worst = [m for m in members if by_id[m]["status"] != "PASS"]
        areas.append({"area": name, "status": st, "tests": members, "limitation": "; ".join(f"{m} {by_id[m]['status']}: {by_id[m].get('reason') or 'see log'}" for m in worst) or "none recorded"})
    red = [r["id"] for r in rows if r["observed"] in ("FAIL", "NOT_RUN", "NOT_TESTED")]
    partial = [r["id"] for r in rows if r["observed"] == "PARTIAL"]
    if red: claim = "NOT SUPPORTED: the S1 gate is not green (FAIL / NOT_RUN / NOT_TESTED rows: " + ", ".join(red) + ")"
    else: claim = "SUPPORTED at bounded-interaction level, once, on program execution; PARTIAL rows: " + ", ".join(partial) + "; all other behaviour classes NOT TESTED"
    out = {"claim": CLAIM + " " + NOT_EVIDENCE, "claim_status": claim, "evidence_level": "bounded interaction (once, program executors)", "counts": counts, "expected_counts": expected,
           "mismatches": [r["id"] for r in rows if r["match"] == "MISMATCH"], "tests": rows, "areas": areas, "open_limits": open_limits(),
           "repos": res_doc.get("repos"), "tools": res_doc.get("tools"), "generated_at": res_doc.get("generated_at")}
    if cell:
        eps = yaml.safe_load((PREREG / "episodes.yaml").read_text())["episodes"]
        erows = []
        for e in eps:
            st = cell["episodes"].get(e["id"], {})
            erows.append({"episode": e["id"], "predicted_result": e["predicted_result"], "settled_result": st.get("settlement", {}).get("result"),
                          "settled_by": st.get("settlement", {}).get("actor"), "executed_by": st.get("settlement", {}).get("executors"),
                          "predicted_classes": e.get("predicted_comparison_classes"), "observed_classes": [(x["class"], x["basis"], x["expectation"][:46]) for x in (st.get("analysis", {}).get("classes") or st.get("analysis", {}).get("classes_v2") or [])],
                          "executor_run": st.get("executor_run"), "verifier_run": st.get("verifier_run")})
        out["episodes"] = erows
        out["capability_delta"] = {"predicted": "GAIN", **(cell.get("capability_delta") or {})}
        out["revision"] = {k: v for k, v in (cell.get("revision") or {}).items() if k not in ("promote", "pin_after", "transitions")}
        gp = pathlib.Path(a.cell) / "gaps.jsonl"
        cell["gaps"] = [json.loads(l) for l in gp.read_text().splitlines() if l.strip()] if gp.exists() else []
        out["cell_gaps"] = cell["gaps"]
    od = pathlib.Path(a.out_dir); od.mkdir(parents=True, exist_ok=True)
    (od / "claim-ledger.json").write_text(json.dumps(out, indent=1, sort_keys=True, default=str) + "\n")
    L = ["# S1 claim ledger", "", "Built mechanically by `scripts/fde-claim-ledger.py` from `results.json` and the frozen predictions; no hand-edited verdicts.", "",
         f"**Claim (G section 2, RD-20, verbatim):** \"{CLAIM}\" {NOT_EVIDENCE}", "", f"**Claim status:** {claim}", "", f"**Evidence level:** {out['evidence_level']}. Repetition, variation, recovery and transfer are not established by this run.", "",
         f"**Counts:** observed {counts}; pre-registered {expected}. Prediction mismatches: {', '.join(out['mismatches']) or 'none'}.", "", "## Per test: prediction, observed, match", "",
         "| Test | Prediction | Observed | Match | Exit | Reason / limitation |", "|---|---|---|---|---|---|"]
    for r in rows: L.append(f"| {r['id']} | {r['prediction']} | {r['observed']} | {r['match']} | {r['exit_code']} | {(r['reason'] or '').replace('|', '/')} |")
    L += ["", "Evidence pointers: per test the log is in `evidence`; command is `scripts/fde-proof.sh --slice s1 --only <ID>`.", ""]
    for r in rows: L.append(f"- {r['id']}: `{r['command']}`, log `{r['evidence']}`, duration {r['duration_s']} s")
    L += ["", "## Per claim area", "", "| Area | Status | Tests | Limitation |", "|---|---|---|---|"]
    for x in areas: L.append(f"| {x['area']} | {x['status']} | {', '.join(x['tests'])} | {x['limitation'].replace('|', '/')} |")
    for n, why in PLANNED: L.append(f"| {n} | PLANNED | none | {why} |")
    L += ["", "## Open limits (STATUS.md 2026-11-14 ~20:30Z, verbatim)", ""] + [f"- {x}" for x in out["open_limits"]]
    L += ["", "Further S1 limits stated by the gate: approval is at `session` assurance, not A3 (pre-S3 identity); pre-S3 identity (actors are bound by a trusted-issuer test driver, assurance `session`); anchoring is a scratch-repository tag with a local clock (OD-6); `--mutate` and `--replay` of the runner are NOT_RUN; human-reserved rows are PARTIAL."]
    if cell:
        L += ["", "## Episodes (predicted vs observed)", ""]
        for e in out["episodes"]:
            L.append(f"- **{e['episode']}**: predicted result {e['predicted_result']}, settled {e['settled_result']} by `{e['settled_by']}` (executed by {e['executed_by']}).")
            L.append(f"  - observed classes: {e['observed_classes']}")
            L.append(f"  - predicted classes: {[(x['expectation'][:40], x.get('class_v1'), x.get('class_v2')) for x in (e['predicted_classes'] or [])]}")
        d = out["capability_delta"]; L += ["", f"**AT-13 capability delta:** predicted {d['predicted']}, result {d.get('result')}, classes {d.get('classes')}, falsifier triggered {d.get('falsifier_triggered')}, R-58 supported: {d.get('r58_supported')}."]
        L += ["", "## Gaps recorded by the run", ""] + [f"- [{g['tests']}] {g['gap']}" for g in (cell.get("gaps") or [])]
    (od / "claim-ledger.md").write_text("\n".join(L) + "\n")
    print(f"claim-ledger: {claim}")


if __name__ == "__main__":
    main()
