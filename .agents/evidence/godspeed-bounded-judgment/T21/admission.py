#!/usr/bin/env python3
"""T21 governed admission for candidate moves.

Implements the frozen neighborhood_rule and payment rule from
.agents/preregistrations/godspeed-bounded-judgment-T21.prereg.yaml.

classify(candidate, route_state, catalog, payment) returns a schema-valid,
inert admission_decision record. Classification precedence (first match
wins): non-mapping -> invalid; authority violations (an operation other
than run_case, any shell_plan/command field, a runner other than the frozen
mock replay runner) -> denied; typed-schema failure -> invalid; a prior
denial on this route -> denied; a scenario the catalog cannot resolve to a
pinned fixture path -> unreachable; payment below one probe unit ->
unaffordable; a (scenario, max_rounds) already probed on this route ->
redundant; otherwise -> admitted.

Denied/invalid decisions carry no execution path: a decision is plain data
and the T22 loop executes only candidates classified admitted.
"""
from __future__ import annotations

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import route_contracts as rc

ADMISSION_POLICY_VERSION = "t21-neighborhood-admission-v1"
PROBE_COST = 1
FROZEN_RUNNER = "mock-agent"


def _affordable(payment):
    return type(payment) is int and payment >= PROBE_COST


def _find_forbidden(value):
    if isinstance(value, dict):
        for key, sub in value.items():
            if key in rc.FORBIDDEN_FIELDS:
                return key
            found = _find_forbidden(sub)
            if found is not None:
                return found
    elif isinstance(value, list):
        for item in value:
            found = _find_forbidden(item)
            if found is not None:
                return found
    return None


def classify(candidate, route_state, catalog, payment):
    if isinstance(candidate, dict):
        cid = candidate.get("candidate_id")
        corr = candidate.get("correlation_id")
    else:
        cid = corr = None
    identity = {
        "candidate_id": cid if isinstance(cid, str) and cid != "" else "unknown-candidate",
        "correlation_id": corr if isinstance(corr, str) and corr != "" else "unidentifiable-input",
    }

    def decision(classification, reason):
        rec = dict(identity)
        rec.update({
            "classification": classification,
            "reason": reason,
            "policy_version": ADMISSION_POLICY_VERSION,
            "decided_at_utc": rc.utcnow(),
        })
        err = rc.validate("admission_decision", rec)
        if err is not None:  # classify must never emit an invalid decision
            raise ValueError(f"internal: invalid admission_decision: {err}")
        return rec

    if not isinstance(candidate, dict):
        return decision("invalid", "candidate is not a mapping")

    op = candidate.get("operation")
    if op is not None and op != "run_case":
        return decision(
            "denied",
            f"authority: operation {op!r} is outside the frozen catalog (only run_case)")
    bad_field = _find_forbidden(candidate)
    if bad_field is not None:
        return decision(
            "denied",
            f"authority: forbidden shell-plan field {bad_field!r} in candidate")
    runner = candidate.get("runner")
    if runner is not None and runner != FROZEN_RUNNER:
        return decision(
            "denied",
            f"authority: runner {runner!r} is not the frozen mock replay runner")

    err = rc.validate("candidate_move", candidate)
    if err is not None:
        return decision("invalid", f"schema: {err}")

    bounded = candidate["bounded_input"]
    key = (bounded["scenario"], bounded["max_rounds"])

    for entry in route_state.get("denied_keys", []):
        if (entry["scenario"], entry["max_rounds"]) == key:
            return decision(
                "denied",
                f"prior denial on this route for {key[0]}@{key[1]}: {entry['reason']}")

    scenarios = catalog.get("derivation_provenance", {}).get("scenarios", {})
    provenance = scenarios.get(key[0])
    if not isinstance(provenance, dict) or not provenance.get("fixture_path"):
        return decision(
            "unreachable",
            f"catalog does not resolve a scenario fixture for {key[0]!r}")

    if not _affordable(payment):
        return decision(
            "unaffordable",
            f"probe cost {PROBE_COST} unit does not fit the remaining payment {payment!r}")

    for entry in route_state.get("probed", []):
        if (entry["scenario"], entry["max_rounds"]) == key:
            return decision(
                "redundant",
                f"{key[0]}@{key[1]} was already probed on this route; the neighborhood "
                "requires an unprobed operation")

    return decision(
        "admitted",
        f"{key[0]}@{key[1]} is cataloged, unprobed, un-denied, reachable, and affordable")
