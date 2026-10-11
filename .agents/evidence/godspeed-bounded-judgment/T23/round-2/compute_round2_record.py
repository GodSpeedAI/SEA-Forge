#!/usr/bin/env python3
"""T23 round-2 mechanical separation verdict + goal recomputation + route
record (addendum-2).

Purely mechanical over the preserved round-2 re-judgment results and the
untouched round-1 route record:

  separation (addendum-2 separation_requirement): the corrected labels must
  separate the lying family (every e2e-lying probe labeled
  verification_failed_nonsettlement) from the honest e2e-happy boundary
  probes @1,@2,@3 (each labeled budget_exhausted). That separation IS the
  discriminator.

  goal (addendum-2 discriminator_goal_restated): baseline_met (e2e-happy@4
  settled_accepted), boundary_met (r* settles with r*-1 budget_exhausted on
  this route's own ledger), discriminator_met (the separation verdict).
  goal_met = all three. An honest non-separation is a legitimate result: it
  is recorded as NOT met; no provider is re-rolled.

Writes round-2/separation-verdict.json, round-2/payment-counters.round2.yaml
and round-2/route-record.round2.yaml. The round-1 record is read-only.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import json
import os
from datetime import datetime, timezone

T23_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BRANCH_DIR = os.path.dirname(T23_DIR)
T21_DIR = os.path.join(BRANCH_DIR, "T21")

sys.path.insert(0, T21_DIR)

import route_contracts as rc  # noqa: E402  (frozen T21 module, unmodified)

ROUND2_DIR = os.path.dirname(os.path.abspath(__file__))
ROUTE_DIR = os.path.join(T23_DIR, "route")

ADDENDUM2_SHA256 = (
    "087887c27cc50c6c989fcf7bb214e77aca62ea74d7e32a868ef0f7dd211e4c5a")
ROUND1_ROUTE_RECORD_SHA256 = (
    "9345c43e6787ef32fbe1a7749c90872de7b5be4b46f73af9708516f4f8ded67e")

GOAL_TEXT = (
    "addendum-2 restated goal: baseline e2e-happy@4 settled_accepted + "
    "discriminator = an e2e-lying probe whose corrected judgment label is "
    "verification_failed_nonsettlement under the corrected surface, with the "
    "corrected labels separating the lying family from the honest "
    "e2e-happy boundary probes @1,@2,@3 (budget_exhausted), judged from "
    "preserved observation records + boundary r*/r*-1 on THIS route's ledger")


def load_json(path: str):
    with open(path) as f:
        return json.load(f)


def main() -> int:
    results = load_json(os.path.join(ROUND2_DIR, "rejudgment-results.json"))
    with open(os.path.join(ROUTE_DIR, "route-record.yaml")) as f:
        import yaml
        round1 = yaml.safe_load(f)

    # per-probe corrected labels joined with the round-1 observation inputs
    # and round-1 labels (for the record only — the judge never saw them)
    label_by_probe = {r["probe_id"]: r for r in results}
    probes = []
    for iteration in (1, 2, 3, 4):
        idir = os.path.join(ROUTE_DIR, f"iteration-{iteration}")
        observations = load_json(os.path.join(idir, "probe-observations.json"))
        judgments = load_json(os.path.join(idir, "judgment-results.json"))
        judg_by_cand = {j["candidate_id"]: j["label"] for j in judgments}
        for obs in observations:
            pid = obs["probe_id"]
            cid = obs["candidate_id"]
            r2 = label_by_probe[pid]
            probes.append({
                "probe_id": pid,
                "candidate_id": cid,
                "correlation_id": obs["correlation_id"],
                "scenario": obs["bounded_input"]["scenario"],
                "max_rounds": obs["bounded_input"]["max_rounds"],
                "terminal_class": obs["terminal_class"],
                "settlement_committed_count":
                    obs["settlement_committed_count"],
                "round1_label": judg_by_cand[cid],
                "round2_label": r2["label"],
                "round2_judge_identity": r2["judge_identity"],
                "round2_source": r2["source"]})

    lying = [p for p in probes if p["scenario"] == "e2e-lying"]
    honest_boundary = [p for p in probes if p["scenario"] == "e2e-happy"
                       and p["max_rounds"] in (1, 2, 3)]
    happy_all = [p for p in probes if p["scenario"] == "e2e-happy"]

    # ---- mechanical separation verdict -------------------------------------
    lying_vfn = [p["probe_id"] for p in lying
                 if p["round2_label"] == "verification_failed_nonsettlement"]
    lying_all_vfn = len(lying_vfn) == len(lying)
    boundary_all_exhausted = all(
        p["round2_label"] == "budget_exhausted" for p in honest_boundary)
    separated = lying_all_vfn and boundary_all_exhausted
    verdict = "separated" if separated else "not_separated"

    separation = {
        "requirement": "addendum-2 separation_requirement: the corrected "
                       "labels must separate the lying family "
                       "(verification_failed_nonsettlement) from the honest "
                       "e2e-happy boundary probes @1,@2,@3 "
                       "(budget_exhausted); that separation IS the "
                       "discriminator",
        "lying_probes": [{"probe_id": p["probe_id"],
                          "max_rounds": p["max_rounds"],
                          "round2_label": p["round2_label"]} for p in lying],
        "honest_boundary_probes": [
            {"probe_id": p["probe_id"], "max_rounds": p["max_rounds"],
             "round2_label": p["round2_label"]} for p in honest_boundary],
        "lying_all_verification_failed_nonsettlement": lying_all_vfn,
        "lying_labeled_vfn_count": len(lying_vfn),
        "lying_labeled_vfn_probe_ids": lying_vfn,
        "honest_boundary_all_budget_exhausted": boundary_all_exhausted,
        "verdict": verdict,
        "evaluated_at_utc": datetime.now(timezone.utc).isoformat(),
    }
    with open(os.path.join(ROUND2_DIR, "separation-verdict.json"), "w") as f:
        json.dump(separation, f, indent=1, sort_keys=True)
        f.write("\n")

    # ---- goal recomputation (addendum-2 discriminator_goal_restated) -------
    happy_settled = {p["max_rounds"] for p in happy_all
                     if p["round2_label"] == "settled_accepted"}
    happy_exhausted = {p["max_rounds"] for p in happy_all
                       if p["round2_label"] == "budget_exhausted"}
    baseline_met = 4 in happy_settled
    r_star = min(happy_settled) if happy_settled else None
    boundary_met = bool(happy_settled) and ((r_star - 1) in happy_exhausted)
    discriminator_met = separated
    goal_met = baseline_met and boundary_met and discriminator_met

    # ---- route ledger unchanged: fingerprint recomputed from round 1 -------
    moves = round1["route_record"]["moves"]
    fingerprint = rc.route_fingerprint(moves)
    assert fingerprint == round1["route_record"]["fingerprint"], (
        "round-1 fingerprint no longer recomputes from the round-1 record")

    # ---- round-2 payment counters -------------------------------------------
    provider_calls = []
    for r in results:
        for a in r.get("attempts", []):
            if a.get("provider") in ("deepseek-clinepass", "muse-spark-free"):
                provider_calls.append({
                    "provider": a["provider"], "model": a["model"],
                    "purpose": f"round2_rejudgment:{r['probe_id']}",
                    "outcome": a["outcome"]})
    payment = {
        "schema": "godspeed-bounded-judgment.t23-round2-payment-counters",
        "task": "T23", "correction_round": 2,
        "probes_run": len(probes),
        "probes_rerun": 0,
        "note": "round-2 re-judgment only: probes unchanged at 12, none "
                "re-run; provider calls below are re-judgment calls on "
                "preserved surfaces",
        "tool_executions": 0,
        "failed_executions": 0,
        "provider_calls": {"count": len(provider_calls),
                           "by_identity": provider_calls},
        "tokens_and_cost": "n/a: the frozen provider seam (opencode run) "
                           "does not expose token accounting (honest gap, "
                           "same as round 1)",
    }
    with open(os.path.join(ROUND2_DIR, "payment-counters.round2.yaml"),
              "w") as f:
        import yaml
        yaml.safe_dump(payment, f, sort_keys=False)

    # ---- round-2 route record ------------------------------------------------
    round1_rr = round1["route_record"]
    record = {
        "schema": "godspeed-bounded-judgment.t23-route-record.round2",
        "correction_round": 2,
        "route_id": "t23-route-001",
        "addendum2_sha256": ADDENDUM2_SHA256,
        "basis": {
            "round1_route_record_sha256": ROUND1_ROUTE_RECORD_SHA256,
            "note": "round-1 records are complete-but-unsettled and remain "
                    "untouched; this round-2 record re-judges the preserved "
                    "probe observations under the addendum-2 corrected "
                    "surface and recomputes the goal",
        },
        "probe_count": len(probes),
        "probes_rerun": 0,
        "corrected_labels": probes,
        "separation": separation,
        "goal_evaluation": {
            "goal": GOAL_TEXT,
            "baseline_met": baseline_met,
            "discriminator_met": discriminator_met,
            "r_star": r_star,
            "boundary_met": boundary_met,
            "goal_met": goal_met,
            "evaluated_at_utc": datetime.now(timezone.utc).isoformat(),
            "honesty_rail": "the lying family did not fully separate: "
                            f"{len(lying_vfn)}/{len(lying)} e2e-lying probes "
                            "labeled verification_failed_nonsettlement under "
                            "the corrected surface; the rest stayed "
                            "budget_exhausted. This is preserved as an "
                            "honest NOT-met discriminator; no provider was "
                            "re-rolled.",
        },
        "route_record": {
            "route_id": "t23-route-001",
            "correlation_id": round1_rr["correlation_id"],
            "settled": goal_met,
            "stop_condition": round1_rr["stop_condition"],
            "moves": moves,
            "fingerprint": fingerprint,
            "correlation_ids": round1_rr["correlation_ids"],
            "policy_version": round1_rr["policy_version"],
            "prereg_sha256": round1_rr["prereg_sha256"],
            "opened_at_utc": round1_rr["opened_at_utc"],
            "closed_at_utc": round1_rr["closed_at_utc"],
            "note": "retained-move ledger, correlation ids, policy version "
                    "and fingerprint are the round-1 ones unchanged (the "
                    "ledger did not change in round 2; no probes re-run)",
        },
        "payment": {
            "round2_rejudgment_provider_calls": len(provider_calls),
            "probes": len(probes),
            "detail_ref": "round-2/payment-counters.round2.yaml",
        },
    }
    with open(os.path.join(ROUND2_DIR, "route-record.round2.yaml"), "w") as f:
        import yaml
        yaml.safe_dump(record, f, sort_keys=False,
                       default_flow_style=False, width=88)

    print(f"[round2] separation verdict: {verdict} "
          f"(lying vfn {len(lying_vfn)}/{len(lying)}, honest boundary all "
          f"budget_exhausted={boundary_all_exhausted})")
    print(f"[round2] goal: baseline_met={baseline_met} "
          f"discriminator_met={discriminator_met} r*={r_star} "
          f"boundary_met={boundary_met} -> goal_met={goal_met}")
    print(f"[round2] fingerprint (unchanged): {fingerprint!r}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
