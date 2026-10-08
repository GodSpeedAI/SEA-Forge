#!/usr/bin/env python3
"""T21 route-discovery contract: typed record schemas, validation, and
lossless YAML/JSON (de)serialization.

Implements exactly the contract frozen in
.agents/preregistrations/godspeed-bounded-judgment-T21.prereg.yaml (sha256
recorded below). Validation refuses: free-form shell plans (any
shell_plan/command field anywhere in a record, or an operation other than
run_case), scenarios outside the three existing fixture scenarios,
max_rounds outside 1..8, labels outside the frozen T15/spec answer domain,
and records with a missing correlation identity.

Every record is inert typed data and carries no execution path; the T22
loop executes only candidates whose admission classification is admitted.
"""
from __future__ import annotations

import datetime
import json
import re

import yaml

SCHEMA = "godspeed-bounded-judgment.t21-route-contracts"
PREREG_SHA256 = "1fb55d9c56d87d94adc4850bc5ca713db3e600297fbdc997accfab54c98ea8b5"

SCENARIOS = ("e2e-happy", "e2e-lying", "e2e-forged-report")
MIN_ROUNDS = 1
MAX_ROUNDS_LIMIT = 8
ANSWER_DOMAIN = (
    "settled_accepted",
    "verification_failed_nonsettlement",
    "budget_exhausted",
    "execution_failed",
    "insufficient_evidence",
)
EXPECTED_ROLES = ("establish_baseline", "establish_discriminator", "boundary_probe")
TERMINAL_CLASSES = ("settled", "budget_exhausted", "other-nonsettlement", "run_error")
ADMISSION_CLASSIFICATIONS = (
    "admitted",
    "denied",
    "unreachable",
    "unaffordable",
    "invalid",
    "redundant",
)
STOP_CONDITIONS = (
    "route_settled",
    "no_eligible_candidate",
    "probe_budget_exhausted",
    "iteration_limit_reached",
    "route_wall_clock_exceeded",
    "unrecoverable_execution_failure",
)
MECHANISM = "t15-frozen-case-execution"
SELECTION_POLICY_VERSION = "selection-policy-v1"
MAX_ROUTE_DEPTH_RETAINED_MOVES = 5
MAX_PROBES_PER_ROUTE = 12
FORBIDDEN_FIELDS = ("shell_plan", "command")

_SHA256_RE = re.compile(r"^[0-9a-f]{64}$")


def _nonempty_str(v):
    return isinstance(v, str) and v != ""


def _any_str(v):
    return isinstance(v, str)


def _is_bool(v):
    return isinstance(v, bool)


def _int_min(lo):
    def check(v):
        return type(v) is int and v >= lo
    return check


def _int_in(lo, hi):
    def check(v):
        return type(v) is int and lo <= v <= hi
    return check


def _lit(*allowed):
    def check(v):
        return v in allowed
    return check


def _timestamp(v):
    if not isinstance(v, str):
        return False
    try:
        datetime.datetime.fromisoformat(v)
    except ValueError:
        return False
    return True


def _hex_sha256(v):
    return isinstance(v, str) and bool(_SHA256_RE.match(v))


def _str_list(v):
    return isinstance(v, list) and all(_nonempty_str(item) for item in v)


def _nonempty_str_list(v):
    return isinstance(v, list) and bool(v) and all(_nonempty_str(item) for item in v)


def _bounded_input(v):
    return (
        isinstance(v, dict)
        and set(v) == {"scenario", "max_rounds"}
        and v["scenario"] in SCENARIOS
        and _int_in(MIN_ROUNDS, MAX_ROUNDS_LIMIT)(v["max_rounds"])
    )


def _generator(v):
    return (
        isinstance(v, dict)
        and set(v) == {"identity", "version"}
        and _nonempty_str(v["identity"])
        and _nonempty_str(v["version"])
    )


def _move_entry(v):
    return (
        isinstance(v, dict)
        and set(v) == {"candidate_id", "correlation_id", "scenario", "max_rounds",
                       "expected_role", "judgment_label"}
        and _nonempty_str(v["candidate_id"])
        and _nonempty_str(v["correlation_id"])
        and v["scenario"] in SCENARIOS
        and _int_in(MIN_ROUNDS, MAX_ROUNDS_LIMIT)(v["max_rounds"])
        and v["expected_role"] in EXPECTED_ROLES
        and (v["judgment_label"] is None or v["judgment_label"] in ANSWER_DOMAIN)
    )


def _probed_entry(v):
    return (
        isinstance(v, dict)
        and set(v) == {"candidate_id", "correlation_id", "scenario", "max_rounds",
                       "terminal_class", "judgment_label"}
        and _nonempty_str(v["candidate_id"])
        and _nonempty_str(v["correlation_id"])
        and v["scenario"] in SCENARIOS
        and _int_in(MIN_ROUNDS, MAX_ROUNDS_LIMIT)(v["max_rounds"])
        and v["terminal_class"] in TERMINAL_CLASSES
        and (v["judgment_label"] is None or v["judgment_label"] in ANSWER_DOMAIN)
    )


def _denied_entry(v):
    return (
        isinstance(v, dict)
        and set(v) == {"scenario", "max_rounds", "reason"}
        and v["scenario"] in SCENARIOS
        and _int_in(MIN_ROUNDS, MAX_ROUNDS_LIMIT)(v["max_rounds"])
        and _nonempty_str(v["reason"])
    )


def _ineligible_entry(v):
    return (
        isinstance(v, dict)
        and set(v) == {"candidate_id", "reason"}
        and _nonempty_str(v["candidate_id"])
        and _nonempty_str(v["reason"])
    )


def _terminal_move_entry(v):
    return (
        isinstance(v, dict)
        and set(v) == {"scenario", "max_rounds", "terminal_class"}
        and v["scenario"] in SCENARIOS
        and _int_in(MIN_ROUNDS, MAX_ROUNDS_LIMIT)(v["max_rounds"])
        and v["terminal_class"] in TERMINAL_CLASSES
    )


def _replay(v):
    return (
        isinstance(v, dict)
        and set(v) == {"probe_id", "state_db_path", "state_db_sha256",
                       "terminal_class", "settled", "mechanism"}
        and _nonempty_str(v["probe_id"])
        and _nonempty_str(v["state_db_path"])
        and _hex_sha256(v["state_db_sha256"])
        and v["terminal_class"] in TERMINAL_CLASSES
        and isinstance(v["settled"], bool)
        and v["mechanism"] == MECHANISM
    )


SPEC = {
    "candidate_move": (
        ("candidate_id", _nonempty_str),
        ("correlation_id", _nonempty_str),
        ("operation", _lit("run_case")),
        ("bounded_input", _bounded_input),
        ("expected_role", _lit(*EXPECTED_ROLES)),
        ("prerequisite_refs", _str_list),
        ("generator", _generator),
        ("rationale", _any_str),
    ),
    "admission_decision": (
        ("candidate_id", _nonempty_str),
        ("correlation_id", _nonempty_str),
        ("classification", _lit(*ADMISSION_CLASSIFICATIONS)),
        ("reason", _nonempty_str),
        ("policy_version", _nonempty_str),
        ("decided_at_utc", _timestamp),
    ),
    "probe_observation": (
        ("probe_id", _nonempty_str),
        ("candidate_id", _nonempty_str),
        ("correlation_id", _nonempty_str),
        ("operation", _lit("run_case")),
        ("bounded_input", _bounded_input),
        ("terminal_class", _lit(*TERMINAL_CLASSES)),
        ("settlement_committed_count", _int_min(0)),
        ("state_db_path", _nonempty_str),
        ("state_db_sha256", _hex_sha256),
        ("mechanism", _lit(MECHANISM)),
        ("started_at_utc", _timestamp),
        ("finished_at_utc", _timestamp),
    ),
    "judgment_result": (
        ("judgment_id", _nonempty_str),
        ("candidate_id", _nonempty_str),
        ("correlation_id", _nonempty_str),
        ("label", _lit(*ANSWER_DOMAIN)),
        ("observation_refs", _nonempty_str_list),
        ("judge_identity", _nonempty_str),
        ("judged_at_utc", _timestamp),
    ),
    "selection_record": (
        ("correlation_id", _nonempty_str),
        ("iteration", _int_min(0)),
        ("policy_version", _lit(SELECTION_POLICY_VERSION)),
        ("retained_candidate_id", _any_str),
        ("eligible_candidate_ids", _str_list),
        ("ineligible", lambda v: isinstance(v, list)
            and all(_ineligible_entry(x) for x in v)),
        ("decided_at_utc", _timestamp),
    ),
    "route_state": (
        ("route_id", _nonempty_str),
        ("created_at_utc", _timestamp),
        ("probes_remaining", _int_in(0, MAX_PROBES_PER_ROUTE)),
        ("retained_moves", lambda v: isinstance(v, list)
            and len(v) <= MAX_ROUTE_DEPTH_RETAINED_MOVES
            and all(_move_entry(x) for x in v)),
        ("probed", lambda v: isinstance(v, list)
            and len(v) <= MAX_PROBES_PER_ROUTE
            and all(_probed_entry(x) for x in v)),
        ("denied_keys", lambda v: isinstance(v, list)
            and all(_denied_entry(x) for x in v)),
    ),
    "route_record": (
        ("route_id", _nonempty_str),
        ("correlation_id", _nonempty_str),
        ("settled", _is_bool),
        ("stop_condition", _lit(*STOP_CONDITIONS)),
        ("moves", lambda v: isinstance(v, list)
            and all(_terminal_move_entry(x) for x in v)),
        ("fingerprint", _any_str),
        ("correlation_ids", _nonempty_str_list),
        ("policy_version", _lit(SELECTION_POLICY_VERSION)),
        ("prereg_sha256", _hex_sha256),
        ("opened_at_utc", _timestamp),
        ("closed_at_utc", _timestamp),
    ),
    "generated_case_record": (
        ("case_id", _nonempty_str),
        ("correlation_id", _nonempty_str),
        ("generator", _generator),
        ("source_route_id", _nonempty_str),
        ("source_route_fingerprint", _any_str),
        ("bounded_input", _bounded_input),
        ("replay", _replay),
        ("accepted", _is_bool),
        ("invalid_reason", _any_str),
        ("created_at_utc", _timestamp),
    ),
}


def _cross_probe(r):
    if r["terminal_class"] == "settled" and r["settlement_committed_count"] < 1:
        return "settled terminal requires at least one settlement_committed event"
    return None


def _cross_selection(r):
    if r["retained_candidate_id"] != "" and r["retained_candidate_id"] not in r["eligible_candidate_ids"]:
        return "retained_candidate_id must be empty or one of eligible_candidate_ids"
    return None


def _cross_route_record(r):
    if r["settled"] != (r["stop_condition"] == "route_settled"):
        return "settled must be true exactly when stop_condition is route_settled"
    if r["fingerprint"] != route_fingerprint(r["moves"]):
        return "fingerprint mismatch against the recorded move sequence"
    return None


def _cross_generated(r):
    replay = r["replay"]
    if replay["settled"] != (replay["terminal_class"] == "settled"):
        return "replay.settled must match the replay terminal_class"
    if r["accepted"] and not replay["settled"]:
        return "accepted requires the independent replay terminal to be settled"
    if r["accepted"] and r["invalid_reason"] != "":
        return "an accepted case must not carry invalid_reason"
    if not r["accepted"] and r["invalid_reason"] == "":
        return "a rejected case must record invalid_reason with its preserved replay evidence"
    return None


CROSS = {
    "probe_observation": _cross_probe,
    "selection_record": _cross_selection,
    "route_record": _cross_route_record,
    "generated_case_record": _cross_generated,
}


def _find_forbidden(value):
    if isinstance(value, dict):
        for key, sub in value.items():
            if key in FORBIDDEN_FIELDS:
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


def validate(record_type, data):
    """Return None when data is a valid record of record_type, else a named
    refusal reason."""
    spec = SPEC.get(record_type)
    if spec is None:
        return f"unknown record type {record_type!r}"
    if not isinstance(data, dict):
        return "record must be a mapping"
    forbidden = _find_forbidden(data)
    if forbidden is not None:
        return f"forbidden field {forbidden!r}: free-form shell plans are invalid by schema"
    missing = [key for key, _ in spec if key not in data]
    if missing:
        if "correlation_id" in missing:
            return "missing correlation identity: correlation_id is absent"
        return "missing field(s): " + ", ".join(missing)
    unknown = sorted(set(data) - {key for key, _ in spec})
    if unknown:
        return "unknown field(s): " + ", ".join(unknown)
    if "operation" in data and data["operation"] != "run_case":
        return f"operation {data['operation']!r} is outside the frozen catalog: only run_case"
    if "correlation_id" in data and not _nonempty_str(data["correlation_id"]):
        return "missing correlation identity: correlation_id must be a nonempty string"
    bounded = data.get("bounded_input")
    if isinstance(bounded, dict):
        if "scenario" in bounded and bounded["scenario"] not in SCENARIOS:
            return f"scenario {bounded['scenario']!r} is outside the frozen scenario domain"
        if "max_rounds" in bounded and not _int_in(MIN_ROUNDS, MAX_ROUNDS_LIMIT)(bounded["max_rounds"]):
            return f"max_rounds {bounded['max_rounds']!r} is outside 1..{MAX_ROUNDS_LIMIT}"
    if "label" in data and data["label"] not in ANSWER_DOMAIN:
        return f"label {data['label']!r} is outside the frozen answer domain"
    for key, checker in spec:
        if not checker(data[key]):
            return f"field {key!r} failed the {record_type} contract"
    cross = CROSS.get(record_type)
    if cross is not None:
        reason = cross(data)
        if reason is not None:
            return reason
    return None


def to_json(record):
    return json.dumps(record, sort_keys=True, separators=(",", ":"))


def from_json(text):
    return json.loads(text)


def to_yaml(record):
    return yaml.safe_dump(record, sort_keys=True, default_flow_style=False)


def from_yaml(text):
    return yaml.safe_load(text)


def utcnow():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def route_fingerprint(moves):
    """Novelty-metric fingerprint: the ordered retained-move sequence of
    (scenario, max_rounds) plus the final terminal class of each move."""
    return ";".join(
        "{scenario}@{max_rounds}:{terminal_class}".format(**move) for move in moves
    )
