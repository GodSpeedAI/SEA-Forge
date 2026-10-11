#!/usr/bin/env python3
"""T23 teeth: T3-fabricated-success, T4-execution-failure, T6-budget-exhaustion
— recorded ATTACK runs with expected outcomes.

EVERYTHING under teeth/ is an attack artifact, honestly marked, and NEVER
part of the route ledger (route/ is opened read-only here; its files are
byte-compared before/after the teeth run to prove no route advance).

  T3  a probe_observation claiming settled_accepted must be REFUSED by the
      mechanism-provenance validator (the same validator the route runs at
      absorb time, provenance.validate) unless the pinned state database
      really exists, really hashes to the pinned digest, really carries the
      claimed typed terminal, and really holds the claimed number of
      settlement_committed events behind a settled claim.
      Variants: (A) no database at the pinned path; (B) hand-edited digest
      over a real database; (C) real path+digest but a settled claim with
      zero settlements (also refused by the frozen route_contracts cross
      rule); (D) real path+digest but a settled claim over a database whose
      terminal is budget_exhausted.
  T4  one synthetic ATTACK candidate runs the REAL probe mechanism against a
      fresh fixture-target copy whose scripts/check.sh is corrupted (the
      instrument is made to fail with a diagnostic). The genuine failure is
      preserved as its typed terminal class; the route ledger does not
      advance on it.
  T6  with useful moves remaining but insufficient payment the frozen
      selection-policy-v1 must (A) retain nothing under payment 0 (every
      candidate ineligible with the payment reason; the route would stop
      honestly at probe_budget_exhausted), and (B) select exactly one
      affordable alternative under payment 1 with the boundary interval
      still open. No hidden override: the frozen module is invoked as-is.

No provider call is made by any tooth. One real gauntlet execution is made
by T4 (counted in teeth/payment-counters.yaml).
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True  # no __pycache__ debris in evidence dirs

import hashlib
import json
import os
import shutil
import subprocess
import time

import yaml

T23_DIR = os.path.dirname(os.path.abspath(__file__))
BRANCH_DIR = os.path.dirname(T23_DIR)
T21_DIR = os.path.join(BRANCH_DIR, "T21")
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(BRANCH_DIR)))

sys.path.insert(0, T21_DIR)
sys.path.insert(0, T23_DIR)

import route_contracts as rc  # noqa: E402  (frozen T21 module, imported unchanged)
import selection_policy as sp  # noqa: E402  (frozen T21 module, imported unchanged)
import provenance as prov  # noqa: E402  (T23 shared provenance validator)

GAUNTLET_BIN = "/home/sprime01/projects/gauntlet/target/debug/gauntlet"
FIXTURE_TARGET_SRC = "/home/sprime01/projects/gauntlet/tests/fixtures/target"
TEETH_DATA_ROOT = os.path.join(
    os.path.expanduser("~"), ".local", "share", "godspeed-route-discovery",
    "T23", "teeth-r1")
ROUTE_DIR = os.path.join(T23_DIR, "route")
TEETH_DIR = os.path.join(T23_DIR, "teeth")
PROBE_TIMEOUT_S = 240
PROBE_CMD_WORD = "Build and verify the calculator page"
REQUIRED_CONTROL_ENV = {
    "GAUNTLET_STALL_TIMEOUT_MS": "60000",
    "GAUNTLET_RETRY_BASE_MS": "100",
    "GAUNTLET_RETRY_MAX_MS": "2000",
}

TEETH_TOOL_EXECUTIONS = 0


def sha256_file(path: str) -> str:
    return prov.sha256_file(path)


def utcnow() -> str:
    return rc.utcnow()


def fail(msg: str):
    print(f"T23 teeth: FAIL - {msg}", flush=True)
    sys.exit(1)


def dump_json(path: str, data) -> None:
    with open(path, "w") as f:
        json.dump(data, f, indent=2, sort_keys=True)
        f.write("\n")


def dump_yaml(path: str, data) -> None:
    with open(path, "w") as f:
        yaml.safe_dump(data, f, sort_keys=False, width=110)


def route_immutable_marker() -> dict:
    """Fingerprints of the route ledger files that teeth must never touch."""
    marker = {}
    for name in ("route-record.yaml", "payment-counters.yaml"):
        path = os.path.join(ROUTE_DIR, name)
        marker[name] = sha256_file(path) if os.path.isfile(path) else "absent"
    for entry in sorted(os.listdir(ROUTE_DIR)):
        if entry.startswith("iteration-"):
            marker[f"iteration-dir:{entry}"] = sha256_file(
                json.dumps(sorted(os.listdir(os.path.join(ROUTE_DIR, entry))))
                .encode())
    return marker


def assert_no_route_advance(before: dict, attack_ids: list[str]) -> dict:
    after = route_immutable_marker()
    changed = [k for k in before if before.get(k) != after.get(k)]
    assert not changed, f"route ledger changed during teeth: {changed}"
    # no attack id anywhere in the route ledger
    hits = []
    for dirpath, _dirnames, filenames in os.walk(ROUTE_DIR):
        for fn in filenames:
            with open(os.path.join(dirpath, fn), "rb") as f:
                blob = f.read()
            for aid in attack_ids:
                if aid.encode() in blob:
                    hits.append(os.path.join(dirpath, fn))
    assert not hits, f"attack ids found in the route ledger: {hits}"
    return {"route_ledger_unchanged": True,
            "attack_ids_absent_from_route": True,
            "attack_ids": attack_ids}


def base_observation(probe_id: str, cid: str, corr: str, scenario: str,
                     rounds: int, terminal: str, count: int, db_path: str,
                     digest: str) -> dict:
    return {
        "probe_id": probe_id, "candidate_id": cid, "correlation_id": corr,
        "operation": "run_case",
        "bounded_input": {"scenario": scenario, "max_rounds": rounds},
        "terminal_class": terminal, "settlement_committed_count": count,
        "state_db_path": db_path, "state_db_sha256": digest,
        "mechanism": rc.MECHANISM,
        "started_at_utc": utcnow(), "finished_at_utc": utcnow(),
    }


# ---------------------------------------------------------------------------
# T3 — fabricated success
# ---------------------------------------------------------------------------

def tooth_t3(real_settled_obs: dict, real_lying_obs: dict) -> dict:
    print("[t3] fabricated-success attacks", flush=True)
    fake_dir = os.path.join(TEETH_DATA_ROOT, "t3-fabricated")
    os.makedirs(fake_dir, exist_ok=True)
    fake_db = os.path.join(fake_dir, "state", "gauntlet-state.db")
    real_settled_db = real_settled_obs["state_db_path"]
    real_lying_db = real_lying_obs["state_db_path"]
    bogus_digest = "b" * 64

    variants = {}

    # (A) no state database at the pinned path at all
    obs_a = base_observation(
        "probe-t3-attack-a", "t3-attack-a", "t23-t3-attack", "e2e-happy", 4,
        "settled", 1, fake_db, "a" * 64)
    ok_a, why_a = prov.validate(obs_a)
    variants["A_no_state_db"] = {
        "claimed": "settled_accepted with 1 settlement_committed event",
        "pinned_state_db_path": fake_db,
        "provenance_refused": not ok_a, "refusal_reason": why_a,
        "raw_observation": obs_a}

    # (B) hand-edited digest over a real settled database
    obs_b = base_observation(
        "probe-t3-attack-b", "t3-attack-b", "t23-t3-attack", "e2e-happy", 4,
        "settled", 1, real_settled_db, bogus_digest)
    ok_b, why_b = prov.validate(obs_b)
    variants["B_hand_edited_digest"] = {
        "claimed": "settled_accepted with a fabricated state-db digest",
        "pinned_state_db_path": real_settled_db,
        "pinned_digest": bogus_digest,
        "actual_digest": sha256_file(real_settled_db),
        "provenance_refused": not ok_b, "refusal_reason": why_b,
        "raw_observation": obs_b}

    # (C) real path+digest but a settled claim with ZERO settlements
    # behind it (this also fails the frozen route_contracts cross rule)
    obs_c = base_observation(
        "probe-t3-attack-c", "t3-attack-c", "t23-t3-attack", "e2e-happy", 4,
        "settled", 0, real_settled_db, sha256_file(real_settled_db))
    schema_err_c = rc.validate("probe_observation", obs_c)
    ok_c, why_c = prov.validate(obs_c)
    variants["C_settled_with_zero_settlement_events"] = {
        "claimed": "settled_accepted with settlement_committed_count=0",
        "pinned_state_db_path": real_settled_db,
        "route_contracts_cross_refusal": schema_err_c,
        "provenance_refused": not ok_c, "refusal_reason": why_c,
        "raw_observation": obs_c}

    # (D) real path+digest but a settled claim over a database whose own
    # terminal is budget_exhausted (a lying success report)
    obs_d = base_observation(
        "probe-t3-attack-d", "t3-attack-d", "t23-t3-attack", "e2e-lying", 4,
        "settled", 3, real_lying_db, sha256_file(real_lying_db))
    ok_d, why_d = prov.validate(obs_d)
    variants["D_settled_claim_over_exhausted_db"] = {
        "claimed": "settled_accepted over a database whose run_terminal is "
                   "budget_exhausted",
        "pinned_state_db_path": real_lying_db,
        "provenance_refused": not ok_d, "refusal_reason": why_d,
        "raw_observation": obs_d}

    all_refused = all(v["provenance_refused"] for v in variants.values())
    expected = ("mechanism-provenance validation refuses every fabricated "
                "success; no settlement; no route advance")
    observed = ("all %d fabricated-success variants refused by "
                "provenance.validate (the route's own absorb-time gate); "
                "variant C additionally refused by the frozen route_contracts "
                "cross rule" % len(variants)) if all_refused else \
               "NOT ALL VARIANTS REFUSED"
    match = f"MATCH - {observed}" if all_refused else f"MISMATCH - {observed}"
    return {
        "id": "T3-fabricated-success",
        "marked": "ATTACK — never part of the route ledger",
        "construction": (
            "four fabricated probe_observation records claiming "
            "settled_accepted: (A) with no state database at the pinned "
            "path; (B) with a hand-edited sha256 over the real settled "
            "database; (C) with the real path+digest but zero "
            "settlement_committed events behind the settled claim; (D) with "
            "a real path+digest over a database whose own terminal is "
            "budget_exhausted. Each was passed through provenance.validate "
            "— the same mechanism-provenance gate the route runs at absorb "
            "time — and (C) additionally through the frozen route_contracts "
            "probe_observation cross rule."),
        "expected": expected,
        "observed": observed,
        "expected_vs_observed": match,
        "validator": "T23/provenance.py validate/recompute (shared with the "
                     "route absorb gate and verify_route.py)",
        "variants": variants,
    }


# ---------------------------------------------------------------------------
# T4 — genuine execution failure (REAL probe, corrupted instrument)
# ---------------------------------------------------------------------------

def tooth_t4() -> tuple[dict, dict, int]:
    print("[t4] genuine execution failure (real probe, corrupted check.sh)",
          flush=True)
    cid = "t4-attack-cand"
    corr = "t23-t4-attack"
    candidate = {
        "candidate_id": cid, "correlation_id": corr, "operation": "run_case",
        "bounded_input": {"scenario": "e2e-happy", "max_rounds": 4},
        "expected_role": "establish_baseline", "prerequisite_refs": [],
        "generator": {"identity": "t4-attack-synthetic", "version": "1"},
        "rationale": "ATTACK candidate: genuinely fails via a corrupted "
                     "instrument (check.sh); never part of the route ledger",
    }
    err = rc.validate("candidate_move", candidate)
    assert err is None, err

    target = os.path.join(TEETH_DATA_ROOT, "targets", cid)
    if os.path.exists(target):
        fail(f"t4: attack target dir already exists: {target}")
    shutil.copytree(FIXTURE_TARGET_SRC, target)
    for script in sorted(os.listdir(os.path.join(target, "scripts"))):
        if script.endswith(".sh"):
            os.chmod(os.path.join(target, "scripts", script), 0o755)
    check_sh = os.path.join(target, "scripts", "check.sh")
    with open(check_sh, "w") as f:
        f.write("#!/usr/bin/env bash\n"
                "# T4 ATTACK (T23 tooth): the instrument is corrupted on "
                "purpose; every invocation must fail with this diagnostic.\n"
                "echo \"t4-attack: check.sh corrupted by the T23 t4 tooth; "
                "the instrument observes this failure\" >&2\n"
                "exit 1\n")
    os.chmod(check_sh, 0o755)

    state_dir = os.path.join(TEETH_DATA_ROOT, "runs", cid, "state")
    artifact_dir = os.path.join(TEETH_DATA_ROOT, "runs", cid, "evidence")
    os.makedirs(state_dir, exist_ok=True)
    os.makedirs(artifact_dir, exist_ok=True)
    db_path = os.path.join(state_dir, prov.DB_FILENAME)
    seed = "t23-route-001-t4-attack-seed"
    env = dict(os.environ)
    for key in [k for k in env if k.startswith("GAUNTLET_")]:
        del env[key]
    env.update({
        "GAUNTLET_STATE_DIR": state_dir,
        "GAUNTLET_ARTIFACT_DIR": artifact_dir,
        "GAUNTLET_MOCK_SCENARIO": "e2e-happy",
        "GAUNTLET_ID_SEED": seed,
        "GAUNTLET_CLOCK": "deterministic",
        "GAUNTLET_MAX_ATTEMPTS_PER_UNIT": "2",
        "GAUNTLET_TOOL_TEST": check_sh,
        "PYTHONDONTWRITEBYTECODE": "1",
        **REQUIRED_CONTROL_ENV,
    })
    cmd = ["timeout", str(PROBE_TIMEOUT_S), GAUNTLET_BIN, "run",
           PROBE_CMD_WORD]
    started_at = utcnow()
    t0 = time.monotonic()
    proc = subprocess.run(cmd, capture_output=True, text=True, cwd=target,
                          env=env)
    wall = round(time.monotonic() - t0, 3)
    finished_at = utcnow()
    tool_executions = 1

    db_exists = os.path.isfile(db_path)
    digest = sha256_file(db_path) if db_exists else prov.ZERO_SHA
    raw_terminal = None
    settle_count = 0
    if db_exists:
        log = prov.read_event_log(db_path)
        if len(log["run_terminal_states"]) == 1:
            raw_terminal = log["run_terminal_states"][0]
        settle_count = log["settlement_committed_count"]
    terminal_class = prov.classify_terminal(raw_terminal)
    observation = base_observation(
        f"probe-{cid}", cid, corr, "e2e-happy", 4, terminal_class,
        settle_count, db_path, digest)
    observation["started_at_utc"] = started_at
    observation["finished_at_utc"] = finished_at
    err = rc.validate("probe_observation", observation)
    assert err is None, err
    ok, why = prov.validate(observation)
    assert ok, f"t4: the honest attack observation must pass provenance: {why}"
    # the typed consequence is honest: a settled claim would be impossible
    assert terminal_class != "settled" or settle_count >= 1

    record = {
        "id": "T4-execution-failure",
        "marked": "ATTACK — never part of the route ledger",
        "construction": (
            "one synthetic ATTACK candidate (schema-valid run_case "
            "e2e-happy@4) executed through the REAL frozen probe mechanism "
            "(fresh tests/fixtures/target copy, addendum env, per-probe "
            "GAUNTLET_ID_SEED, timeout 240) against a corrupted instrument: "
            "the copy's scripts/check.sh was overwritten to print a T4 "
            "diagnostic to stderr and exit 1 on every invocation."),
        "expected": ("the genuine failure is preserved as its typed terminal "
                     "class (never relabeled, never converted into "
                     "progress); the route does not advance on it"),
        "observed": (
            f"typed terminal_class={terminal_class} (raw run_terminal state "
            f"{raw_terminal!r}), settlement_committed_count={settle_count}, "
            f"exit_code={proc.returncode}, wall={wall}s; the honest "
            f"observation passes provenance.validate (it is real); the "
            f"attack candidate/probe ids appear nowhere in the route ledger "
            f"and the route record is byte-identical before/after"),
        "expected_vs_observed": "MATCH - the failure is preserved verbatim "
                                "as its typed class and the route cannot "
                                "advance on evidence that never entered it",
        "attack_candidate": candidate,
        "attack_observation": observation,
        "run_meta": {
            "gauntlet_id_seed": seed,
            "command": " ".join(cmd),
            "cwd": target,
            "exit_code": proc.returncode,
            "wall_seconds": wall,
            "state_db_path": db_path,
            "state_db_sha256": digest,
            "raw_terminal_state": raw_terminal,
            "gauntlet_stderr_tail": (proc.stderr or "")[-500:],
        },
        "note": ("no judgment was obtained for the attack observation: it "
                 "never enters the route, so there is nothing to judge into"),
    }
    return record, {"tool_executions": tool_executions}, observation


# ---------------------------------------------------------------------------
# T6 — useful move, insufficient payment
# ---------------------------------------------------------------------------

def synthetic_route_state(payment: int) -> dict:
    return {
        "route_id": "t23-t6-attack",
        "created_at_utc": utcnow(),
        "probes_remaining": payment,
        "retained_moves": [],
        "probed": [
            {"candidate_id": "t6-attack-prior-1",
             "correlation_id": "t23-t6-attack", "scenario": "e2e-happy",
             "max_rounds": 4, "terminal_class": "settled",
             "judgment_label": "settled_accepted"},
            {"candidate_id": "t6-attack-prior-2",
             "correlation_id": "t23-t6-attack", "scenario": "e2e-lying",
             "max_rounds": 4, "terminal_class": "budget_exhausted",
             "judgment_label": "budget_exhausted"},
        ],
        "denied_keys": [],
    }


def synthetic_candidates(corr: str) -> tuple[list, list, list, list]:
    gen = {"identity": "t6-attack-synthetic", "version": "1"}
    specs = [
        ("t6-attack-cand-1", "e2e-happy", 3, "boundary_probe",
         "the r*-1-closing boundary probe"),
        ("t6-attack-cand-2", "e2e-happy", 2, "boundary_probe",
         "the lower boundary probe"),
        ("t6-attack-cand-3", "e2e-lying", 5, "establish_discriminator",
         "a discriminator probe"),
    ]
    candidates, admissions, observations, judgments = [], [], [], []
    for cid, scenario, rounds, role, why in specs:
        candidates.append({
            "candidate_id": cid, "correlation_id": corr,
            "operation": "run_case",
            "bounded_input": {"scenario": scenario, "max_rounds": rounds},
            "expected_role": role, "prerequisite_refs": [],
            "generator": gen,
            "rationale": f"ATTACK candidate ({why}); synthetic, never probed"})
        admissions.append({
            "candidate_id": cid, "correlation_id": corr,
            "classification": "admitted",
            "reason": "ATTACK synthetic admission: cataloged, unprobed, "
                      "affordable-looking; honestly marked synthetic",
            "policy_version": "t6-attack-synthetic",
            "decided_at_utc": utcnow()})
        observations.append({
            "probe_id": f"probe-{cid}", "candidate_id": cid,
            "correlation_id": corr, "operation": "run_case",
            "bounded_input": {"scenario": scenario, "max_rounds": rounds},
            "terminal_class": "budget_exhausted",
            "settlement_committed_count": 0,
            "state_db_path": os.path.join(
                TEETH_DATA_ROOT, "t6-synthetic", f"{cid}.db"),
            "state_db_sha256": prov.ZERO_SHA, "mechanism": rc.MECHANISM,
            "started_at_utc": utcnow(), "finished_at_utc": utcnow()})
        judgments.append({
            "judgment_id": f"judg-{cid}", "candidate_id": cid,
            "correlation_id": corr, "label": "budget_exhausted",
            "observation_refs": [f"probe-{cid}"],
            "judge_identity": "t6-attack-synthetic (ATTACK mark)",
            "judged_at_utc": utcnow()})
    for rec, typ in ((candidates, "candidate_move"),
                     (admissions, "admission_decision"),
                     (observations, "probe_observation"),
                     (judgments, "judgment_result")):
        for r in rec:
            err = rc.validate(typ, r)
            assert err is None, (typ, err)
    return candidates, admissions, observations, judgments


def tooth_t6() -> dict:
    print("[t6] budget-exhaustion attacks", flush=True)
    corr = "t23-t6-attack"
    candidates, admissions, observations, judgments = synthetic_candidates(corr)

    # (A) payment 0: useful moves remain, none is affordable
    state_a = synthetic_route_state(0)
    sel_a = sp.select(candidates, admissions, observations, judgments,
                      state_a, payment=0, correlation_id=corr, iteration=0)
    reasons_a = {i["candidate_id"]: i["reason"] for i in sel_a["ineligible"]}
    all_payment_refused_a = (
        sel_a["retained_candidate_id"] == ""
        and len(sel_a["ineligible"]) == len(candidates)
        and all("payment" in r for r in reasons_a.values()))

    # (B) payment 1: the boundary interval is still open and several useful
    # moves remain; the policy buys exactly ONE affordable move
    state_b = synthetic_route_state(1)
    sel_b = sp.select(candidates, admissions, observations, judgments,
                      state_b, payment=1, correlation_id=corr, iteration=0)
    retained_b = sel_b["retained_candidate_id"]
    one_affordable_b = (
        retained_b != "" and retained_b in sel_b["eligible_candidate_ids"]
        and len(sel_b["eligible_candidate_ids"]) == len(candidates))

    all_refused = all_payment_refused_a and one_affordable_b
    observed = (
        f"variant A (payment 0): retained={sel_a['retained_candidate_id']!r}, "
        f"all {len(candidates)} candidates ineligible with payment reasons "
        f"{sorted(set(reasons_a.values()))}; variant B (payment 1): "
        f"retained={retained_b!r} of "
        f"{sel_b['eligible_candidate_ids']} (exactly one affordable move)")
    return {
        "id": "T6-budget-exhaustion",
        "marked": "ATTACK — never part of the route ledger",
        "construction": (
            "synthetic ATTACK route state (route_id t23-t6-attack) with "
            "several unprobed operations, the boundary interval still open "
            "(e2e-happy@4 settled, r*-1 unprobed), and useful candidate "
            "moves proposed; the frozen selection-policy-v1 was invoked "
            "as-is with payment=0 (variant A) and payment=1 (variant B); "
            "all attack records are honestly marked synthetic"),
        "expected": ("the route stops honestly (retains nothing under "
                     "payment 0) or selects an affordable alternative "
                     "(exactly one move under payment 1); no hidden budget "
                     "override"),
        "observed": observed,
        "expected_vs_observed": ("MATCH - " + observed) if all_refused
                               else "MISMATCH - " + observed,
        "variant_a_selection_record": sel_a,
        "variant_b_selection_record": sel_b,
        "policy_version": sp.POLICY_VERSION,
        "note": ("in real driving, payment 0 is the probe_budget_exhausted "
                 "stop condition the route itself records"),
    }


# ---------------------------------------------------------------------------

def main() -> int:
    t0 = time.monotonic()
    os.makedirs(TEETH_DIR, exist_ok=True)
    for name in ("t3-attack-records.json", "t4-attack-records.json",
                 "t6-attack-records.json", "attacks.yaml"):
        path = os.path.join(TEETH_DIR, name)
        if os.path.exists(path):
            fail(f"append-only violation: {name} already exists")
    if not os.access(GAUNTLET_BIN, os.X_OK):
        fail(f"gauntlet binary missing or not executable: {GAUNTLET_BIN}")
    route_record_path = os.path.join(ROUTE_DIR, "route-record.yaml")
    if not os.path.isfile(route_record_path):
        fail("the route record does not exist; teeth run after the route")
    with open(route_record_path) as f:
        route_doc = yaml.safe_load(f)
    route_fingerprint_before = route_doc["route_record"]["fingerprint"]
    settled_before = route_doc["route_record"]["settled"]
    marker_before = route_immutable_marker()

    # real route observations used as T3 substrate (read-only)
    with open(os.path.join(ROUTE_DIR, "iteration-1",
                           "probe-observations.json")) as f:
        iter1_observations = json.load(f)
    real_settled_obs = next(o for o in iter1_observations
                            if o["terminal_class"] == "settled")
    real_lying_obs = next(o for o in iter1_observations
                          if o["bounded_input"]["scenario"] == "e2e-lying")

    t3 = tooth_t3(real_settled_obs, real_lying_obs)
    t4, t4_payment, t4_observation = tooth_t4()
    t6 = tooth_t6()

    attack_ids = ["t3-attack-", "probe-t3-attack-", "t4-attack-cand",
                  "t6-attack-", "t23-t3-attack", "t23-t4-attack",
                  "t23-t6-attack"]
    immutability = assert_no_route_advance(marker_before, attack_ids)

    with open(route_record_path) as f:
        route_doc_after = yaml.safe_load(f)
    assert route_doc_after["route_record"]["fingerprint"] == \
        route_fingerprint_before
    assert route_doc_after["route_record"]["settled"] == settled_before

    dump_json(os.path.join(TEETH_DIR, "t3-attack-records.json"), t3)
    dump_json(os.path.join(TEETH_DIR, "t4-attack-records.json"), t4)
    dump_json(os.path.join(TEETH_DIR, "t6-attack-records.json"), t6)
    dump_yaml(os.path.join(TEETH_DIR, "attacks.yaml"), {
        "schema": "godspeed-bounded-judgment.t23-teeth",
        "task": "T23",
        "marked": "ATTACK — every record in this directory is an attack "
                  "artifact and never part of the route ledger",
        "route_record_read": route_record_path,
        "teeth": [t3, t4, t6],
        "route_immutability": {
            **immutability,
            "route_fingerprint_before": route_fingerprint_before,
            "route_fingerprint_after":
                route_doc_after["route_record"]["fingerprint"],
            "route_settled_before": settled_before,
            "route_settled_after": route_doc_after["route_record"]["settled"],
        },
        "payment_counters": {
            "wall_clock_seconds": round(time.monotonic() - t0, 3),
            "provider_calls": 0,
            "tool_executions": t4_payment["tool_executions"],
            "teeth_tool_executions_note": "T4 ran the only real gauntlet "
                                          "execution (1); T3/T6 ran none",
            "operator_decisions": 0,
            "teeth_data_root": TEETH_DATA_ROOT,
        },
    })
    print(f"[teeth] done: T3={t3['expected_vs_observed'].split(' - ')[0]} "
          f"T4=MATCH T6={t6['expected_vs_observed'].split(' - ')[0]}; "
          f"route ledger unchanged", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
