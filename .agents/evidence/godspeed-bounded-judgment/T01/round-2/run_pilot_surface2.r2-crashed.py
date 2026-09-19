#!/usr/bin/env python3
"""T01 round-2 — preregistered SECOND-SURFACE evaluation (outcome C consequence).

Surface (frozen in the round-1 preregistration, evaluated now because outcome C
occurred): multiclass chain-deficiency classification.

    question: "Which verification-chain deficiency, if any, does this record's
               chain exhibit?"
    domain:   [no_chain_deficiency, missing_evidence_object, unresolvable_reference,
               instrument_mismatch, insufficient_provenance]

Deterministic baseline (frozen in the round-2 addendum): the MECHANICAL chain
classifier defined below — a rule over the reconstructed chain facts only, no
model involvement. Providers receive the chain facts (never surface-1 answers,
never the mechanical classification, never any recorded outcome).

This harness reuses the frozen round-1 reconstruction functions by import (the
round-1 harness file is unmodified; imports do not edit frozen artifacts).
"""

from __future__ import annotations

import json
import math
import os
import re
import subprocess
import sys
from collections import Counter

import yaml

HERE = os.path.dirname(os.path.abspath(__file__))
T01 = os.path.abspath(os.path.join(HERE, ".."))
sys.path.insert(0, T01)
import run_pilot as rp  # noqa: E402  (frozen round-1 harness, imported not modified)

MANIFEST = os.path.join(T01, "../../preregistrations/godspeed-bounded-judgment-T01.corpus-manifest.yml")
ADDENDUM = os.path.join(HERE, "prereg-round2-addendum.yml")
ROUND_DIR = HERE

DOMAIN2 = ["no_chain_deficiency", "missing_evidence_object", "unresolvable_reference",
           "instrument_mismatch", "insufficient_provenance"]

PROMPT2 = (
    "You are doing a bounded judgment task. Answer STRICTLY with one JSON object "
    "and nothing else.\n"
    "Question: Which verification-chain deficiency, if any, does this record's "
    "chain exhibit?\n"
    "Answer domain (choose exactly one value for \"answer\"): no_chain_deficiency, "
    "missing_evidence_object, unresolvable_reference, instrument_mismatch, "
    "insufficient_provenance.\n"
    "Meanings: missing_evidence_object = the chain cites evidence that is absent; "
    "unresolvable_reference = a reference in the chain cannot be resolved to its "
    "target; instrument_mismatch = the instrument that produced the verification "
    "does not match what the requirement demands; insufficient_provenance = a chain "
    "hop lacks enough provenance to attribute it; no_chain_deficiency = none of the "
    "above applies to the facts shown.\n"
    "Required JSON shape: {{\"answer\": <one domain label>, \"confidence\": <0..1>, "
    "\"distribution\": {{<label>: <probability> for all five labels}}, \"reason\": "
    "<one sentence>}}. Distribution values must sum to approximately 1.\n\n"
    "Chain facts (JSON):\n{chain}\n"
)


def mechanical_classify(row):
    """Frozen deterministic baseline: rule over reconstructed chain facts only."""
    ch = row.get("chain", {})
    cons = ch.get("consequence", {}) or {}
    if not cons.get("evidence_object_resolvable", False):
        return "missing_evidence_object"
    if isinstance(cons.get("evidence_production"), dict) and "gap" in cons["evidence_production"]:
        return "unresolvable_reference"
    if isinstance(cons.get("observation_binding"), dict) and "gap" in cons["observation_binding"]:
        return "unresolvable_reference"
    if isinstance(ch.get("settlement"), dict) and "gap" in ch.get("settlement", {}):
        return "insufficient_provenance"
    if isinstance(ch.get("action"), dict) and "gap" in ch.get("action", {}):
        return "insufficient_provenance"
    if str(ch.get("temporal_integrity", "")).startswith("VIOLATED"):
        return "insufficient_provenance"
    inst = row.get("instrument") or {}
    req = row.get("requirement") or {}
    if isinstance(inst, dict) and isinstance(req, dict):
        if inst.get("deterministic") is not True:
            return "instrument_mismatch"
    return "no_chain_deficiency"


def build_input2(row):
    ch = row.get("chain", {})
    return {
        "chain_hops_present": {
            "state_before": "gap" not in (ch.get("state_before") or {}),
            "action": "gap" not in (ch.get("action") or {}),
            "consequence_observation_binding": "gap" not in ((ch.get("consequence") or {}).get("observation_binding") or {}),
            "consequence_evidence_production": "gap" not in ((ch.get("consequence") or {}).get("evidence_production") or {}),
            "evidence_object_resolvable": bool((ch.get("consequence") or {}).get("evidence_object_resolvable")),
            "settlement": "gap" not in (ch.get("settlement") or {}),
        },
        "temporal_integrity": str((ch.get("temporal_integrity") or "")),
        "instrument": {"name": (row.get("instrument") or {}).get("name"),
                       "version": (row.get("instrument") or {}).get("version"),
                       "declared_deterministic": (row.get("instrument") or {}).get("deterministic")},
        "requirement_kind": (row.get("requirement") or {}).get("kind"),
        "evidence_production_locator": ((ch.get("consequence") or {}).get("evidence_production") or {}).get("locator"),
    }


def assert_no_leakage2(inp):
    blob = json.dumps(inp)
    for banned in ("deterministic_outcome", "mechanical_class", "\"answer\"", "surface-1", "satisfied"):
        if banned in blob:
            raise RuntimeError(f"label-leakage: input contains {banned}")


def main() -> int:
    # freeze verification: the addendum must exist and reference the manifest hash
    addendum_text = open(ADDENDUM).read()
    manifest_hash = rp.sha256_file(MANIFEST)
    if manifest_hash not in addendum_text:
        rp.fail("round-2 addendum does not reference the corpus manifest hash")
    manifest, entries = rp.load_and_verify_manifest(MANIFEST)
    seen, bad = rp.verify_object_hashes(entries)
    if bad:
        rp.fail(f"evidence drift since F0: {bad[:3]}")
    before = rp.snapshot_state(entries)
    print(f"[verify] manifest + {seen} evidence files OK", flush=True)

    rows, _gaps = rp.reconstruct(entries, ROUND_DIR)
    mech = [mechanical_classify(r) for r in rows]
    baseline_counts = Counter(mech)
    print(f"[baseline] mechanical classes: {dict(baseline_counts)}", flush=True)

    results = {"codex": [], "prime-agent": []}
    for i, row in enumerate(rows):
        inp = build_input2(row)
        assert_no_leakage2(inp)
        prompt = PROMPT2.format(chain=json.dumps(inp, indent=1))
        for pname in ("codex", "prime-agent"):
            resp = rp.call_cli_provider(pname, prompt)
            if resp["status"] == "raw":
                parsed = rp.parse_provider_answer2(resp["raw"]) if hasattr(rp, "parse_provider_answer2") else parse2(resp["raw"])
                parsed["raw_tail"] = resp["raw"][-300:]
            else:
                parsed = {"status": resp["status"], "reason": "CLI failure"}
            parsed["row_index"] = i
            results[pname].append(parsed)
        print(f"[providers] row {i + 1}/{len(rows)} done", flush=True)
    with open(os.path.join(ROUND_DIR, "provider-raw-results-surface2.json"), "w") as f:
        json.dump({"domain": DOMAIN2, "mechanical_baseline": mech, "results": results}, f, indent=1)

    # scoring
    pair = {p: [(r.get("answer"), m) for r, m in zip(results[p], mech) if r.get("status") == "ok"] for p in results}
    report_rows = []
    for p in results:
        ok_pairs = pair[p]
        acc = (sum(1 for a, m in ok_pairs if a == m) / len(ok_pairs)) if ok_pairs else None
        conf_mat = {a: {m: 0 for m in DOMAIN2} for a in DOMAIN2}
        ents = []
        for r in results[p]:
            if r.get("status") == "ok":
                conf_mat[r["answer"]][mech[r["row_index"]]] += 1
                d = r.get("distribution") or {}
                ents.append(-sum((d.get(l, 0.0) or 0.0) * math.log((d.get(l, 0.0) or 1e-9)) for l in DOMAIN2))
        dis = sum(1 for a, b in zip(results["codex"], results["prime-agent"])
                  if a.get("answer") and b.get("answer") and a["answer"] != b["answer"])
        report_rows.append({
            "provider": p,
            "n_ok": len(ok_pairs),
            "contract_failures": len(results[p]) - len(ok_pairs),
            "accuracy_vs_mechanical_baseline": acc,
            "confusion_rows_provider_cols_baseline": conf_mat,
            "mean_entropy_bits": (sum(ents) / len(ents)) if ents else None,
            "cross_capable_disagreement_rows": dis if p == "codex" else None,
        })
    n_classes = len(baseline_counts)
    degenerate = n_classes < 2
    authorization = {
        "rule": ("authorization requires ALL of: 0 contract failures for both capable providers; "
                 "mechanical baseline has >=2 distinct classes on this corpus; each capable provider "
                 "scores accuracy >= 0.8 vs the mechanical baseline with mean entropy > 0"),
        "contract_leg": all(r["contract_failures"] == 0 for r in report_rows),
        "baseline_nondegenerate_leg": not degenerate,
        "baseline_class_counts": dict(baseline_counts),
        "agreement_leg": all((r["accuracy_vs_mechanical_baseline"] or 0.0) >= 0.8 and (r["mean_entropy_bits"] or 0.0) > 0
                             for r in report_rows),
    }
    authorization["authorized"] = all(authorization[k] for k in
                                      ("contract_leg", "baseline_nondegenerate_leg", "agreement_leg"))
    report = {
        "schema": "godspeed-bounded-judgment.t01-round2-surface2",
        "disclosure": {
            "n": len(rows), "independent_run_count": 6,
            "class_counts": dict(baseline_counts),
            "concentration_by_run": "see round-1 concentration-report.yml (same rows)",
            "missing_attribution": "17/17 rows unattributed",
            "reconstructibility_rate": 1.0,
            "stability": "UNSTABLE: pilot corpus of order 10^1; descriptive only",
        },
        "surface2": {
            "question": "Which verification-chain deficiency, if any, does this record's chain exhibit?",
            "domain": DOMAIN2,
            "deterministic_baseline": "mechanical_classify() over reconstructed chain facts (rule frozen in prereg-round2-addendum.yml)",
        },
        "providers": report_rows,
        "authorization": authorization,
        "source_integrity": {"drifted": rp.check_source_unchanged(entries, before)},
    }
    with open(os.path.join(ROUND_DIR, "surface2-report.yml"), "w") as f:
        yaml.safe_dump(report, f, sort_keys=False, width=120)
    print(f"[done] authorized={authorization['authorized']} baseline={dict(baseline_counts)}")
    return 0


def parse2(text):
    m = re.search(r"\{.*\}", text, re.DOTALL)
    if not m:
        return {"status": "schema_failure"}
    try:
        obj = json.loads(m.group(0))
    except json.JSONDecodeError:
        return {"status": "schema_failure"}
    if obj.get("answer") not in DOMAIN2:
        return {"status": "out_of_domain", "raw_answer": obj.get("answer")}
    d = {l: max(0.0, min(1.0, float((obj.get("distribution") or {}).get(l, 0.0)))) for l in DOMAIN2}
    t = sum(d.values())
    return {"status": "ok", "answer": obj.get("answer"),
            "confidence": max(0.0, min(1.0, float(obj.get("confidence", 0) or 0))),
            "distribution": ({k: v / t for k, v in d.items()} if t else d),
            "reason": str(obj.get("reason", ""))[:300]}


if __name__ == "__main__":
    sys.exit(main())
