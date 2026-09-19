#!/usr/bin/env python3
"""T25 tooth T8 (redundant-route) — ATTACK artifact, never developmental data.

ATTACK: regenerate a structurally equivalent route record — the SAME
retained-move sequence and fingerprint as the settled route t23-route-001 —
and present it as if it were a fresh route. Run it through the SAME frozen
novelty metric the gate uses (prereg novelty_metric: fingerprint = ordered
retained-move sequence of (scenario, max_rounds) plus the final terminal of
each; classification: novel | near-duplicate (same operation set, different
order) | structural duplicate (equal fingerprint)).

EXPECTED: the novelty computation DETECTS the duplicate — the synthetic
record classifies as a STRUCTURAL DUPLICATE of t23-route-001 — and the
duplicate is measured and recorded WITHOUT automatic rejection (frozen
policy: "duplicates are measured and recorded; no automatic rejection is
frozen in this slice").

No gauntlet execution and no provider call: the regenerated record is
synthetic by construction and honestly marked ATTACK.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import os

import yaml

T25_DIR = os.path.dirname(os.path.abspath(__file__))
BRANCH_DIR = os.path.dirname(T25_DIR)
T21_DIR = os.path.join(BRANCH_DIR, "T21")
T23_DIR = os.path.join(BRANCH_DIR, "T23")
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(BRANCH_DIR)))
TEETH_DIR = os.path.join(T25_DIR, "teeth")

sys.path.insert(0, T21_DIR)

import route_contracts as rc  # noqa: E402  (frozen T21 module: the metric)

ROUND3_RECORD = os.path.join(T23_DIR, "round-3", "route-record.round3.yaml")
SOURCE_ROUTE_ID = "t23-route-001"


def classify_novelty(fingerprint: str, operation_set: frozenset,
                     reference_fingerprints: list[str],
                     reference_operation_sets: list[frozenset]) -> str:
    """Frozen prereg novelty_metric classification (the same function shape
    the gate re-derives independently):
      structural duplicate: equal fingerprint in the reference set
      near-duplicate: same operation set, different order
      novel: otherwise"""
    if fingerprint in reference_fingerprints:
        return "structural-duplicate"
    if operation_set in reference_operation_sets:
        return "near-duplicate"
    return "novel"


def fail(msg: str):
    print(f"T25 tooth T8: FAIL - {msg}", flush=True)
    sys.exit(1)


def main() -> int:
    os.makedirs(TEETH_DIR, exist_ok=True)
    for name in ("t8-duplicate-route-record.yaml",
                 "t8-attack-record.json"):
        if os.path.exists(os.path.join(TEETH_DIR, name)):
            fail(f"append-only violation: {name} already exists")
    if os.path.exists(os.path.join(T25_DIR, "teeth.yaml")):
        fail("append-only violation: teeth.yaml already exists")
    pair_path = os.path.join(T25_DIR, "matched-pair.yaml")
    if not os.path.isfile(pair_path):
        fail("matched-pair.yaml missing; the tooth must not precede the "
             "matched pair")
    import hashlib
    h = hashlib.sha256()
    with open(pair_path, "rb") as f:
        h.update(f.read())
    pair_digest_before = h.hexdigest()

    with open(ROUND3_RECORD) as f:
        rr3 = yaml.safe_load(f)
    route = rr3["route_record"]
    if route.get("route_id") != SOURCE_ROUTE_ID or \
            route.get("settled") is not True:
        fail("the settled route record is not what the tooth expects")
    moves = route["moves"]
    fingerprint = rc.route_fingerprint(moves)
    if fingerprint != route["fingerprint"]:
        fail("the settled route fingerprint does not recompute")

    # the REGENERATED (synthetic) route record: structurally equivalent by
    # construction — same retained-move sequence, same terminals
    duplicate_record = {
        "schema": "godspeed-bounded-judgment.t25-t8-duplicate-route-record",
        "marked": ("ATTACK - synthetic regenerated route record, never part "
                   "of any route ledger or developmental set"),
        "route_id": "t25-t8-duplicate-route",
        "correlation_id": "t25-t8-attack",
        "construction": ("regenerated retained-move sequence copied from the "
                         "settled route t23-route-001: structurally "
                         "equivalent by construction (same moves in the same "
                         "order with the same terminals)"),
        "settled": True,
        "stop_condition": "route_settled",
        "moves": [dict(m) for m in moves],
        "fingerprint": fingerprint,
        "policy_version": "selection-policy-v1",
        "note": "a fabricated 'fresh' route with no own probes; used ONLY "
                "to show the frozen novelty metric detects it",
    }

    # the SAME frozen metric, applied to the synthetic record against the
    # real settled route as reference
    op_set = frozenset((m["scenario"], m["max_rounds"])
                       for m in duplicate_record["moves"])
    reference_fingerprints = [route["fingerprint"]]
    reference_operation_sets = [
        frozenset((m["scenario"], m["max_rounds"]) for m in moves)]
    classification = classify_novelty(
        duplicate_record["fingerprint"], op_set,
        reference_fingerprints, reference_operation_sets)

    # and, as the tooth's control, the REAL route against the T15/T16 grid
    # fingerprint set is computed by the gate (novelty.yaml); here only the
    # duplicate-detection half of the tooth is executed.
    if classification != "structural-duplicate":
        fail(f"the regenerated record classified {classification!r}; the "
             "duplicate was NOT detected")

    import json
    attack_record = {
        "id": "T8-redundant-route",
        "marked": "ATTACK - synthetic record, never part of any route "
                  "ledger or developmental set",
        "construction": ("regenerated a structurally equivalent route record "
                         "(same retained-move sequence and terminals as "
                         "t23-route-001, fingerprint "
                         f"{fingerprint}) and ran the SAME frozen novelty "
                         "metric (route_contracts.route_fingerprint + the "
                         "prereg classification rule) over it against the "
                         "settled route as reference"),
        "expected": ("the novelty computation DETECTS the duplicate: "
                     "classification structural duplicate for the synthetic "
                     "record; duplication measurable and recorded, with NO "
                     "automatic rejection (frozen policy)"),
        "observed": (f"classification = {classification}: the equal "
                     "fingerprint was detected by direct comparison; the "
                     "duplicate is recorded here and rejected by nothing"),
        "expected_vs_observed": ("MATCH - structural duplicate detected and "
                                 "recorded; no auto-rejection applied "
                                 "(measured, per the frozen novelty policy)"),
        "classification": classification,
        "classification_rule": {
            "structural-duplicate": "equal fingerprint in the reference set",
            "near-duplicate": "same operation set, different order",
            "novel": "otherwise",
            "metric_source": "prereg novelty_metric.fingerprint via frozen "
                             "T21 route_contracts.route_fingerprint",
        },
        "reference": {
            "route_id": SOURCE_ROUTE_ID,
            "fingerprint": fingerprint,
            "operation_set": sorted(f"{m['scenario']}@{m['max_rounds']}"
                                    for m in moves),
        },
        "duplicate_detected": True,
        "auto_rejection_applied": False,
        "no_auto_rejection_note": ("frozen policy: duplicates are measured "
                                   "and recorded; no automatic rejection is "
                                   "frozen in this slice"),
        "gauntlet_executions": 0,
        "provider_calls": 0,
    }
    with open(os.path.join(TEETH_DIR, "t8-attack-record.json"), "w") as f:
        json.dump(attack_record, f, indent=2, sort_keys=True)
        f.write("\n")
    with open(os.path.join(TEETH_DIR, "t8-duplicate-route-record.yaml"),
              "w") as f:
        yaml.safe_dump(duplicate_record, f, sort_keys=False, width=110)

    h2 = hashlib.sha256()
    with open(pair_path, "rb") as f:
        h2.update(f.read())

    teeth = {
        "schema": "godspeed-bounded-judgment.t25-teeth",
        "task": "T25",
        "marked": ("ATTACK - every record in this directory is an attack "
                   "artifact and never part of the developmental records"),
        "matched_pair_read": os.path.relpath(pair_path, REPO_ROOT),
        "teeth": [attack_record],
        "immutability": {
            "matched_pair_sha256_before": pair_digest_before,
            "matched_pair_sha256_after": h2.hexdigest(),
            "matched_pair_unchanged": pair_digest_before == h2.hexdigest(),
            "attack_ids_absent_from_developmental_records": True,
            "attack_ids": ["t8-attack", "t25-t8-duplicate-route",
                           "t25-t8-attack"],
        },
        "payment_counters": {
            "wall_clock_seconds": 0,
            "provider_calls": 0,
            "tool_executions": 0,
            "failed_executions": 0,
            "note": ("T8 is a synthetic-record attack: no gauntlet "
                     "execution and no provider call"),
        },
    }
    with open(os.path.join(T25_DIR, "teeth.yaml"), "w") as f:
        yaml.safe_dump(teeth, f, sort_keys=False, width=110)

    print(f"[tooth] T8: regenerated record fingerprint {fingerprint!r} -> "
          f"classification {classification} (detected, no auto-rejection)",
          flush=True)
    print(f"[tooth] matched-pair.yaml unchanged: "
          f"{pair_digest_before == h2.hexdigest()}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
