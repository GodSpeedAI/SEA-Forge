#!/usr/bin/env python3
"""T21 frozen deterministic selection policy (selection-policy-v1).

Implements selection_policy_v1_frozen exactly as preregistered in
.agents/preregistrations/godspeed-bounded-judgment-T21.prereg.yaml:
the eligibility list, ordering rules 1-3, and single retention. The policy
consumes only typed records (validated through route_contracts), never asks
any judge or model which candidate is best, and has no provider interface
and no execution capability: it reads records and returns a
selection_record. Probing all admitted candidates is the loop's
responsibility (T22); this policy only retains.

Ordering precedence reading (recorded as an interpretation in the task
result): rule 1 (completes an unmet route-goal prerequisite) outranks rule
3 (r*-1 boundary priority), which outranks the rule-2 midpoint heuristic;
without rule 3 outranking rule 2 the frozen boundary goal (both r* and
r*-1 probed on this route) could not close.
"""
from __future__ import annotations

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import route_contracts as rc

POLICY_VERSION = "selection-policy-v1"
PROBE_COST = 1

_ROLE_RANK = {"establish_baseline": 0, "establish_discriminator": 1, "boundary_probe": 2}


def _affordable(payment):
    return type(payment) is int and payment >= PROBE_COST


def _require_typed(record_type, records):
    for record in records:
        err = rc.validate(record_type, record)
        if err is not None:
            raise ValueError(
                f"selection_policy consumes typed records only; {record_type}: {err}")


def _index_by_candidate(records, what):
    out = {}
    for record in records:
        cid = record["candidate_id"]
        if cid in out:
            raise ValueError(f"duplicate {what} for candidate {cid!r}")
        out[cid] = record
    return out


def _route_knowledge(route_state):
    """Route-goal and boundary knowledge derived only from this route's own
    absorbed probe records (judgment labels); T15 rows are never cited."""
    probed = route_state["probed"]
    exhausted = {p["max_rounds"] for p in probed
                 if p["scenario"] == "e2e-happy"
                 and p["judgment_label"] == "budget_exhausted"}
    settled = {p["max_rounds"] for p in probed
               if p["scenario"] == "e2e-happy"
               and p["judgment_label"] == "settled_accepted"}
    return {
        "exhausted": exhausted,
        "settled": settled,
        "baseline_met": any(p["scenario"] == "e2e-happy" and p["max_rounds"] == 8
                            and p["judgment_label"] == "settled_accepted"
                            for p in probed),
        "discriminator_met": any(p["scenario"] == "e2e-lying"
                                 and p["judgment_label"] == "verification_failed_nonsettlement"
                                 for p in probed),
        "boundary_met": bool(settled) and (min(settled) - 1) in exhausted,
        "lo": max(exhausted) if exhausted else 0,
        "hi": min(settled) if settled else rc.MAX_ROUNDS_LIMIT + 1,
    }


def _eligible(c, admissions, observations, judgments, route_state, payment):
    """Return None when eligible, else the ineligibility reason. Check order
    follows the prereg eligibility list."""
    cid = c["candidate_id"]
    a = admissions.get(cid)
    if a is None:
        return "no admission decision"
    if a["classification"] != "admitted":
        return (f"admission classification {a['classification']!r} is ineligible "
                "(only admitted is eligible)")
    o = observations.get(cid)
    if o is None:
        return "no probe observation: a typed terminal is required"
    if o["terminal_class"] == "run_error":
        return "probe ended in a run error (execution_failed); ineligible for retention"
    j = judgments.get(cid)
    if j is None:
        return "no judgment result"
    if j["label"] == "insufficient_evidence":
        return ("judgment label insufficient_evidence disqualifies this candidate "
                f"(expected_role {c['expected_role']!r}; its consequence would be "
                "cited by the settlement criterion)")
    if not _affordable(payment):
        return (f"probe cost {PROBE_COST} unit does not fit the remaining "
                f"payment {payment!r}")
    retained_ids = {m["candidate_id"] for m in route_state["retained_moves"]}
    unmet = [ref for ref in c["prerequisite_refs"] if ref not in retained_ids]
    if unmet:
        return f"prerequisite_refs not yet satisfied on the route: {unmet}"
    return None


def _ordering_key(c, label, knowledge, payment):
    cid = c["candidate_id"]
    role = c["expected_role"]
    scenario = c["bounded_input"]["scenario"]
    rounds = c["bounded_input"]["max_rounds"]
    exhausted, settled = knowledge["exhausted"], knowledge["settled"]

    # Rule 1: completes an unmet route-goal prerequisite, in canonical order
    # (baseline, then discriminator, then boundary).
    if (role == "establish_baseline" and not knowledge["baseline_met"]
            and label == "settled_accepted" and (scenario, rounds) == ("e2e-happy", 8)):
        return (0, _ROLE_RANK[role], rounds, cid)
    if (role == "establish_discriminator" and not knowledge["discriminator_met"]
            and label == "verification_failed_nonsettlement" and scenario == "e2e-lying"):
        return (0, _ROLE_RANK[role], rounds, cid)
    if role == "boundary_probe" and not knowledge["boundary_met"]:
        if label == "settled_accepted" and (rounds - 1) in exhausted:
            return (0, _ROLE_RANK[role], rounds, cid)
        if label == "budget_exhausted" and (rounds + 1) in settled:
            return (0, _ROLE_RANK[role], rounds, cid)

    # Rule 3: when r* is found but r*-1 is unprobed, r*-1 has boundary
    # priority (outranks the rule-2 midpoint heuristic).
    if role == "boundary_probe" and settled:
        rstar = min(settled)
        if ((rstar - 1) not in exhausted and (rstar - 1) not in settled
                and rounds == rstar - 1):
            return (1, 0, rounds, cid)

    # Rule 2: among boundary probes, prefer the midpoint max_rounds of the
    # largest uncertain interval (floor((lo+hi)/2)) when it is unprobed and
    # affordable; ties break to the smaller max_rounds, then lexicographic
    # candidate_id.
    if role == "boundary_probe":
        mid = (knowledge["lo"] + knowledge["hi"]) // 2
        preferred = 0 if (rounds == mid and mid not in exhausted
                          and mid not in settled and _affordable(payment)) else 1
        return (2, preferred, rounds, cid)

    return (3, 0, rounds, cid)


def select(candidates, admissions, observations, judgments, route_state, payment,
           correlation_id, iteration):
    """Return a schema-valid selection_record for one route iteration.

    Exactly one eligible candidate is retained (empty retained_candidate_id
    when none is eligible); ALL admitted candidates are still probed by the
    loop — evidence is never discarded. The policy never asks a judge or
    model which candidate is best.
    """
    _require_typed("candidate_move", candidates)
    _require_typed("admission_decision", admissions)
    _require_typed("probe_observation", observations)
    _require_typed("judgment_result", judgments)
    err = rc.validate("route_state", route_state)
    if err is not None:
        raise ValueError(f"selection_policy consumes typed records only; route_state: {err}")
    if len({c["candidate_id"] for c in candidates}) != len(candidates):
        raise ValueError("duplicate candidate_id in candidates")
    if not (type(iteration) is int and iteration >= 0):
        raise ValueError("iteration must be a nonnegative integer")
    if not isinstance(correlation_id, str) or correlation_id == "":
        raise ValueError("correlation_id must be a nonempty string")

    adm_by = _index_by_candidate(admissions, "admission_decision")
    obs_by = _index_by_candidate(observations, "probe_observation")
    judg_by = _index_by_candidate(judgments, "judgment_result")
    knowledge = _route_knowledge(route_state)

    eligible, ineligible = [], []
    for c in candidates:
        reason = _eligible(c, adm_by, obs_by, judg_by, route_state, payment)
        if reason is None:
            eligible.append(c)
        else:
            ineligible.append({"candidate_id": c["candidate_id"], "reason": reason})

    ranked = sorted(
        eligible,
        key=lambda c: _ordering_key(c, judg_by[c["candidate_id"]]["label"],
                                    knowledge, payment))
    retained = ranked[0]["candidate_id"] if ranked else ""

    rec = {
        "correlation_id": correlation_id,
        "iteration": iteration,
        "policy_version": POLICY_VERSION,
        "retained_candidate_id": retained,
        "eligible_candidate_ids": [c["candidate_id"] for c in ranked],
        "ineligible": ineligible,
        "decided_at_utc": rc.utcnow(),
    }
    err = rc.validate("selection_record", rec)
    if err is not None:
        raise ValueError(f"selection policy produced an invalid selection_record: {err}")
    return rec
