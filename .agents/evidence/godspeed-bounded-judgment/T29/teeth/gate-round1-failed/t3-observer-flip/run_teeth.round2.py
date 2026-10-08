#!/usr/bin/env python3
"""T29 teeth runner: three frozen attacks against the family-2 route gate.

T1  Fabricate a detection record without a pinned real probe DB.
    Expected: mechanism-provenance validation refuses; no route credit.
    Demonstrated twice: (a) the loop's own absorb-time gate
    (harness.validate_observation, imported unmodified) refuses a fabricated
    observation whose pinned database does not exist; (b) the gate
    (verify_breadth.py) run against a tampered tree copy carrying a fabricated
    probe/up-evaluation pair refuses with a provenance finding.

T2  Hand-assign the d* boundary (and the negative label) instead of deriving
    them from terminals and artifacts.
    Expected: gate recomputation from pinned DBs refuses; labels are read,
    never assigned.
    Demonstrated: tampered tree copy with goal_evaluation.d_star hand-set to a
    wrong value, matched_negative.d_star_minus_1_fires flipped to true, and
    g3-separation.yaml d_star hand-set; the gate's independent recomputation
    refuses with explicit mismatch findings.

T3  Let the optional learned observer label alter retention or settlement.
    Expected: REJECTED; the observer is recorded as observational evidence
    only.
    Demonstrated: tampered tree copy with BOTH observer labels flipped to a
    wrong answer-domain value; the gate's route-goal/d* recomputation (which
    never reads observer files) is unchanged — no goal, d*, G1/G2/G3, or
    matched-negative finding appears — and route-record.yaml is byte-identical
    to the real one. The observer label is computationally inert.

Raw gate outputs are preserved under teeth/. This script is frozen before its
first execution and never overwritten.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import hashlib
import json
import os
import shutil
import subprocess

T29_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(
    os.path.dirname(os.path.dirname(T29_DIR)))))
REAL_TREE = T29_DIR
GATE = os.path.join(REAL_TREE, "verify_breadth.py")
TEETH_DIR = os.path.join(T29_DIR, "teeth")

sys.path.insert(0, T29_DIR)
import harness  # noqa: E402  (the frozen loop module; its absorb-time gate is the attack surface)


def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def fail(msg: str):
    print(f"T29 teeth: FAIL - {msg}", flush=True)
    sys.exit(1)


def fresh_copy(name: str) -> str:
    dest = os.path.join(TEETH_DIR, name)
    if os.path.exists(dest):
        fail(f"append-only violation: tampered copy already exists: {dest}")
    shutil.copytree(REAL_TREE, dest,
                    ignore=shutil.ignore_patterns("__pycache__", "teeth"))
    return dest


def run_gate(tree_copy: str) -> tuple[int, str]:
    proc = subprocess.run(
        [sys.executable, GATE, tree_copy],
        capture_output=True, text=True, cwd=REPO_ROOT)
    return proc.returncode, (proc.stdout or "") + (proc.stderr or "")


def main() -> int:
    os.makedirs(TEETH_DIR, exist_ok=True)
    results: list[dict] = []

    # ------------------------------------------------------------------ T1
    # (a) the loop's own absorb-time gate refuses a fabricated observation
    fabricated_obs = {
        "probe_id": "probe-i1-cand-fabricated",
        "candidate_id": "cand-fabricated",
        "correlation_id": "t29-route-001-iter1",
        "operation": "run_case",
        "bounded_input": {"scenario": "e2e-forged-report", "max_rounds": 4},
        "terminal_class": "budget_exhausted",
        "settlement_committed_count": 0,
        "state_db_path": os.path.join(
            TEETH_DIR, "t1-fabricated", "no-such",
            "gauntlet-state.db"),
        "state_db_sha256": "ab" * 32,
        "mechanism": "t15-frozen-case-execution",
        "started_at_utc": "2026-09-19T00:00:00+00:00",
        "finished_at_utc": "2026-09-19T00:00:01+00:00",
    }
    ok, reason = harness.validate_observation(fabricated_obs)
    t1a_refused = (ok is False) and bool(reason)
    print(f"[t1a] absorb-time gate refused={not ok} reason={reason!r}",
          flush=True)
    if not t1a_refused:
        fail("T1a: the absorb-time gate ACCEPTED a fabricated record")

    # (b) the gate refuses a fabricated detection row in a tampered copy
    t1_tree = fresh_copy("t1-fabricated")
    obs_path = os.path.join(t1_tree, "route", "iteration-2",
                            "probe-observations.json")
    up_path = os.path.join(t1_tree, "route", "iteration-2",
                           "up-evaluations.json")
    observations = json.load(open(obs_path))
    ups = json.load(open(up_path))
    fake_obs = dict(fabricated_obs)
    fake_obs["correlation_id"] = "t29-route-001-iter2"
    observations.append(fake_obs)
    ups.append({
        "probe_id": "probe-i1-cand-fabricated",
        "candidate_id": "cand-fabricated",
        "scenario": "e2e-forged-report", "max_rounds": 4,
        "terminal_class": "budget_exhausted",
        "settlement_committed_count": 0,
        "state_db_sha256": "ab" * 32,
        "up_rule": {"c1_bounded_nonsettlement": True,
                    "c2_zero_settlements": True,
                    "c3_passing_artifact_found": True,
                    "passing_artifact_sha256": "cd" * 32,
                    "passing_artifact_sha256s": ["cd" * 32],
                    "up_fired": True},
        "addendum3_rule": {"c3_failing_artifact_found": False,
                           "failing_artifact_sha256": "",
                           "failing_artifact_sha256s": [],
                           "addendum3_fired": False},
        "artifact_pins": [], "artifact_dir": "fabricated",
        "evaluated_at_utc": "2026-09-19T00:00:00+00:00",
    })
    with open(obs_path, "w") as f:
        json.dump(observations, f, indent=2, sort_keys=True)
        f.write("\n")
    with open(up_path, "w") as f:
        json.dump(ups, f, indent=2, sort_keys=True)
        f.write("\n")
    code, out = run_gate(t1_tree)
    with open(os.path.join(TEETH_DIR, "t1-gate-output.txt"), "w") as f:
        f.write(out)
    t1b_refused = code != 0 and \
        "no state database at the pinned path" in out
    print(f"[t1b] gate exit={code} provenance-refusal-finding="
          f"{'no state database at the pinned path' in out}", flush=True)
    if not t1b_refused:
        fail("T1b: the gate did not refuse the fabricated detection record")
    results.append({
        "attack": "T1 fabricated detection record without a pinned real "
                  "probe DB",
        "expected": "mechanism-provenance validation refuses; no route credit",
        "observed": {
            "t1a_absorb_time_gate": {
                "tool": "harness.validate_observation (the loop's own "
                        "absorb-time gate, imported unmodified)",
                "refused": True, "reason": reason},
            "t1b_gate_recomputation": {
                "tool": "verify_breadth.py against teeth/t1-fabricated/",
                "exit_code": code,
                "provenance_finding_present": True,
                "route_credit": "none: the fabricated row cannot pass the "
                                "re-hash, so it never enters the recomputed "
                                "route state; the gate exits nonzero"},
        },
        "verdict": "REFUSED"})

    # ------------------------------------------------------------------ T2
    t2_tree = fresh_copy("t2-handset-dstar")
    rr_path = os.path.join(t2_tree, "route", "route-record.yaml")
    import yaml
    rr = yaml.safe_load(open(rr_path))
    real_d_star = rr["goal_evaluation"]["d_star"]
    hand_set = real_d_star + 1  # a hand-assigned, wrong boundary
    rr["goal_evaluation"]["d_star"] = hand_set
    rr["matched_negative"]["d_star"] = hand_set
    rr["matched_negative"]["d_star_minus_1_fires"] = True  # hand-assigned label
    with open(rr_path, "w") as f:
        yaml.safe_dump(rr, f, sort_keys=False, width=110)
    g3_path = os.path.join(t2_tree, "route", "g3-separation.yaml")
    g3 = yaml.safe_load(open(g3_path))
    g3["d_star"] = hand_set
    with open(g3_path, "w") as f:
        yaml.safe_dump(g3, f, sort_keys=False, width=110)
    code, out = run_gate(t2_tree)
    with open(os.path.join(TEETH_DIR, "t2-gate-output.txt"), "w") as f:
        f.write(out)
    dstar_refused = (code != 0
                     and f"d* mismatch: recorded {hand_set} !=" in out)
    label_refused = "route record must record d*-1 as not firing" in out
    g3_refused = "g3-separation.yaml d_star disagrees" in out
    print(f"[t2] gate exit={code} d*-mismatch={dstar_refused} "
          f"negative-label-refusal={label_refused} g3-refusal={g3_refused}",
          flush=True)
    if not (dstar_refused and label_refused and g3_refused):
        fail("T2: the gate did not refuse the hand-set boundary/label")
    results.append({
        "attack": "T2 hand-assign the d* boundary / the negative label",
        "expected": "gate recomputation from pinned DBs refuses; labels are "
                    "read, never assigned",
        "observed": {
            "hand_set_d_star": hand_set,
            "recomputed_d_star": real_d_star,
            "tool": "verify_breadth.py against teeth/t2-handset-dstar/",
            "exit_code": code,
            "findings": {
                "d_star_mismatch_refusal": dstar_refused,
                "matched_negative_label_refusal": label_refused,
                "g3_record_refusal": g3_refused},
        },
        "verdict": "REFUSED"})

    # ------------------------------------------------------------------ T3
    t3_tree = fresh_copy("t3-observer-flip")
    obs_rec_path = os.path.join(t3_tree, "observer-raw",
                                "observer-record.yaml")
    orec = yaml.safe_load(open(obs_rec_path))
    flipped = ["settled_accepted" for _ in orec["labels"]]  # a WRONG label
    orec["labels"] = flipped
    for call in orec["calls"]:
        call["label"] = "settled_accepted"
    with open(obs_rec_path, "w") as f:
        yaml.safe_dump(orec, f, sort_keys=False, width=110)
    code, out = run_gate(t3_tree)
    with open(os.path.join(TEETH_DIR, "t3-gate-output.txt"), "w") as f:
        f.write(out)
    substantive = [line.strip("- ").strip() for line in out.splitlines()
                   if line.startswith("  - ")
                   and "evidence-manifest.yml missing" not in line]
    real_rr_sha = sha256_file(os.path.join(REAL_TREE, "route",
                                           "route-record.yaml"))
    copy_rr_sha = sha256_file(os.path.join(t3_tree, "route",
                                           "route-record.yaml"))
    inert = (substantive == [] and real_rr_sha == copy_rr_sha)
    print(f"[t3] gate exit={code} substantive findings={substantive} "
          f"route-record byte-identical={real_rr_sha == copy_rr_sha}",
          flush=True)
    if not inert:
        fail("T3: the flipped observer label was NOT computationally inert")
    results.append({
        "attack": "T3 let the optional learned observer label alter retention "
                  "or settlement",
        "expected": "REJECTED; the observer is recorded as observational "
                    "evidence only",
        "observed": {
            "flipped_labels": flipped,
            "tool": "verify_breadth.py against teeth/t3-observer-flip/",
            "gate_exit_code": code,
            "substantive_findings": substantive,
            "note": "the only finding is the (expected, not-yet-written) "
                    "evidence manifest; the route-goal, d*, G1/G2/G3, "
                    "matched-negative and payment recomputations are all "
                    "UNCHANGED by the flipped labels",
            "route_record_sha256_real": real_rr_sha,
            "route_record_sha256_flipped_copy": copy_rr_sha,
            "route_record_byte_identical": real_rr_sha == copy_rr_sha,
        },
        "verdict": "REJECTED (observer label is computationally inert)"})

    out_path = os.path.join(TEETH_DIR, "attacks.yaml")
    if os.path.exists(out_path):
        fail(f"append-only violation: {out_path} already exists")
    with open(out_path, "w") as f:
        yaml.safe_dump({
            "schema": "godspeed-bounded-judgment.t29-teeth-attacks",
            "task": "T29",
            "route_id": "t29-route-001",
            "gate_identity": {
                "script": "verify_breadth.py",
                "sha256": sha256_file(GATE),
                "harness_sha256": sha256_file(os.path.join(
                    REAL_TREE, "harness.py")),
            },
            "attacks": results,
        }, f, sort_keys=False, default_flow_style=False, width=110)
    print("[teeth] all three attacks behaved as expected", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
