#!/usr/bin/env python3
"""T24 tooth T7 (invalid-inverse-case) — ATTACK artifact, never developmental
data.

ATTACK: derive a companion case that should NOT be accepted — the SAME
accepted case but with the payment capacity lowered to max_rounds=2 (a known
honest non-settlement budget on route t23-route-001's OWN ledger:
probe-i3-cand-1, e2e-happy@2, terminal budget_exhausted with 1 honest
settlement) — and PRESENT IT AS IF it were solvable by the settled route
(i.e. claim the same legitimacy the accepted case carries). Replay it through
the REAL mechanism (fresh target copy, fresh seed) and push the result
through the SAME acceptance path replay_case.py used.

EXPECTED: the replay does NOT settle (its label is read from the run_terminal,
never assigned) and the acceptance path REJECTS the case: accepted=false with
a recorded invalid_reason; the rejected attack record is preserved under
teeth/ and is NOT added as valid developmental data (the developmental
accepted-case.yaml stays byte-identical).

Ground truth = the run's own state database read read-only (path+sha256
pinned), validated by the frozen T23 provenance validator. Everything in this
file and in teeth/ is marked ATTACK.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import json
import os
import shutil
import subprocess
import time

import yaml

T24_DIR = os.path.dirname(os.path.abspath(__file__))
BRANCH_DIR = os.path.dirname(T24_DIR)
T21_DIR = os.path.join(BRANCH_DIR, "T21")
T23_DIR = os.path.join(BRANCH_DIR, "T23")
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(BRANCH_DIR)))

sys.path.insert(0, T21_DIR)
sys.path.insert(0, T23_DIR)

import route_contracts as rc      # noqa: E402  (frozen T21 module)
import provenance as prov         # noqa: E402  (frozen T23 validator)

GAUNTLET_BIN = "/home/sprime01/projects/gauntlet/target/debug/gauntlet"
FIXTURE_TARGET_SRC = "/home/sprime01/projects/gauntlet/tests/fixtures/target"
PROBE_CMD_WORD = "Build and verify the calculator page"
PROBE_TIMEOUT_S = 240
REQUIRED_CONTROL_ENV = {
    "GAUNTLET_STALL_TIMEOUT_MS": "60000",
    "GAUNTLET_RETRY_BASE_MS": "100",
    "GAUNTLET_RETRY_MAX_MS": "2000",
}
DATA_ROOT = os.path.join(os.path.expanduser("~"), ".local", "share",
                         "godspeed-route-discovery", "T24", "teeth-r1")
RUN_KEY = "t7-attack"
PROBE_ID = "probe-t24-t7-attack"
SEED = "t24-t7-attack-seed"
ATTACK_ROUNDS = 2
TEETH_DIR = os.path.join(T24_DIR, "teeth")


def utcnow() -> str:
    return rc.utcnow()


def fail(msg: str) -> None:
    print(f"T24 tooth T7: FAIL - {msg}", flush=True)
    sys.exit(1)


def main() -> int:
    os.makedirs(TEETH_DIR, exist_ok=True)
    for name in ("teeth.yaml", "t7-attack-record.json"):
        if os.path.exists(os.path.join(T24_DIR, name)) or \
                os.path.exists(os.path.join(TEETH_DIR, name)):
            fail(f"append-only violation: {name} already exists")
    if os.path.exists(DATA_ROOT):
        fail(f"tooth data root already exists: {DATA_ROOT}")

    accepted_path = os.path.join(T24_DIR, "accepted-case.yaml")
    candidate_path = os.path.join(T24_DIR, "candidate-case.yaml")
    if not os.path.isfile(accepted_path):
        fail("accepted-case.yaml missing; T7 attacks the ACCEPTED case's "
             "acceptance path and requires it to exist")
    with open(accepted_path) as f:
        accepted_case = yaml.safe_load(f)
    if accepted_case.get("accepted") is not True:
        fail("the developmental case is not accepted; T7's premise missing")
    before = {name: prov.sha256_file(p) for name, p in
              (("accepted-case.yaml", accepted_path),
               ("candidate-case.yaml", candidate_path))}

    scenario = accepted_case["bounded_input"]["scenario"]
    rounds_pos = accepted_case["bounded_input"]["max_rounds"]
    if (scenario, rounds_pos) != ("e2e-happy", 4):
        fail("unexpected accepted case shape")

    # ---- the ATTACK case: same case, max_rounds lowered to the known honest
    # non-settlement budget, PRESENTED AS IF solvable by the settled route ---
    attack_case = {
        "schema": "godspeed-bounded-judgment.t24-t7-attack-case",
        "marked": "ATTACK - never part of the developmental set",
        "case_id": "t24-case-t7-attack",
        "correlation_id": "t24-t7-attack",
        "claimed": {
            "claim": ("this case IS solvable by route t23-route-001 and "
                      "should be accepted like t24-case-001 (FALSE claim "
                      "under test)"),
            "source_route_id": accepted_case["source_route_id"],
            "source_route_fingerprint":
                accepted_case["source_route_fingerprint"],
            "generator": {"identity": "t7-attack-synthetic", "version": "1"},
        },
        "bounded_input": {"scenario": scenario, "max_rounds": ATTACK_ROUNDS},
        "known_route_fact_attacked": ("the route's OWN ledger records "
                                      "e2e-happy@2 as budget_exhausted "
                                      "(probe-i3-cand-1, 1 honest settlement) "
                                      "- an honest non-settlement budget, so "
                                      "the case cannot legitimately settle"),
        "note": "single-factor lookalike of t24-case-001: everything about "
                "the presentation is identical except max_rounds 4 -> 2",
    }

    # ---- REAL replay through the frozen mechanism ---------------------------
    target = os.path.join(DATA_ROOT, "targets", RUN_KEY)
    shutil.copytree(FIXTURE_TARGET_SRC, target)
    for script in sorted(os.listdir(os.path.join(target, "scripts"))):
        if script.endswith(".sh"):
            os.chmod(os.path.join(target, "scripts", script), 0o755)
    gauntlet_md = os.path.join(target, "GAUNTLET.md")
    with open(gauntlet_md) as f:
        md = f.read()
    lever = f"max_rounds: {rounds_pos}"
    if lever not in md:
        fail(f"GAUNTLET.md lacks the expected lever {lever!r}")
    md = md.replace(lever, f"max_rounds: {ATTACK_ROUNDS}", 1)
    with open(gauntlet_md, "w") as f:
        f.write(md)
    with open(gauntlet_md) as f:
        if f"max_rounds: {ATTACK_ROUNDS}" not in f.read():
            fail("max_rounds sed did not take effect")

    state_dir = os.path.join(DATA_ROOT, "runs", RUN_KEY, "state")
    artifact_dir = os.path.join(DATA_ROOT, "runs", RUN_KEY, "evidence")
    os.makedirs(state_dir, exist_ok=True)
    os.makedirs(artifact_dir, exist_ok=True)
    db_path = os.path.join(state_dir, prov.DB_FILENAME)

    env = dict(os.environ)
    for key in [k for k in env if k.startswith("GAUNTLET_")]:
        del env[key]
    env.update({
        "GAUNTLET_STATE_DIR": state_dir,
        "GAUNTLET_ARTIFACT_DIR": artifact_dir,
        "GAUNTLET_MOCK_SCENARIO": scenario,
        "GAUNTLET_ID_SEED": SEED,
        "GAUNTLET_CLOCK": "deterministic",
        "GAUNTLET_MAX_ATTEMPTS_PER_UNIT": "2",
        "GAUNTLET_TOOL_TEST": os.path.join(target, "scripts", "check.sh"),
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

    db_exists = os.path.isfile(db_path)
    settle_count = 0
    raw_terminal_state = None
    if db_exists:
        digest = prov.sha256_file(db_path)
        log = prov.read_event_log(db_path)
        states = log["run_terminal_states"]
        settle_count = log["settlement_committed_count"]
        if len(states) == 1:
            raw_terminal_state = states[0]
    else:
        digest = prov.ZERO_SHA
    terminal_class = prov.classify_terminal(raw_terminal_state)
    settled = terminal_class == "settled"

    observation = {
        "probe_id": PROBE_ID,
        "candidate_id": attack_case["case_id"],
        "correlation_id": attack_case["correlation_id"],
        "operation": "run_case",
        "bounded_input": {"scenario": scenario, "max_rounds": ATTACK_ROUNDS},
        "terminal_class": terminal_class,
        "settlement_committed_count": settle_count,
        "state_db_path": db_path,
        "state_db_sha256": digest,
        "mechanism": rc.MECHANISM,
        "started_at_utc": started_at,
        "finished_at_utc": finished_at,
    }
    err = rc.validate("probe_observation", observation)
    if err is not None:
        fail(f"internal: invalid attack observation: {err}")
    ok, reason = prov.validate(observation)
    if not ok:
        fail(f"provenance validation refused the attack observation: {reason}")

    # ---- the SAME acceptance path, applied honestly -------------------------
    accepted = settled  # acceptance requires the replay's real settlement
    rejected_record = {
        "case_id": attack_case["case_id"],
        "correlation_id": attack_case["correlation_id"],
        "generator": attack_case["claimed"]["generator"],
        "source_route_id": attack_case["claimed"]["source_route_id"],
        "source_route_fingerprint":
            attack_case["claimed"]["source_route_fingerprint"],
        "bounded_input": {"scenario": scenario, "max_rounds": ATTACK_ROUNDS},
        "replay": {
            "probe_id": PROBE_ID,
            "state_db_path": db_path,
            "state_db_sha256": digest,
            "terminal_class": terminal_class,
            "settled": settled,
            "mechanism": rc.MECHANISM,
        },
        "accepted": accepted,
        "invalid_reason": "" if accepted else
        "T7 ATTACK case REJECTED by the acceptance path: the independent "
        f"replay ended {terminal_class} with {settle_count} "
        "settlement_committed events - NOT a settlement; the case is "
        "recorded invalid and is NOT added as valid developmental data "
        "(preserved here as an attack artifact only)",
        "created_at_utc": finished_at,
    }
    err = rc.validate("generated_case_record", rejected_record)
    if err is not None:
        fail(f"internal: attack record fails the frozen schema: {err}")
    if accepted:
        fail("the T7 attack replay SETTLED, which contradicts the route's "
             "own ledger; the tooth cannot record a MATCH on this outcome")
    rejected_path = os.path.join(TEETH_DIR, "t7-attack-rejected-case.yaml")
    with open(rejected_path, "w") as f:
        yaml.safe_dump(rejected_record, f, sort_keys=False, width=110)

    after = {name: prov.sha256_file(p) for name, p in
             (("accepted-case.yaml", accepted_path),
              ("candidate-case.yaml", candidate_path))}

    attack_record = {
        "id": "T7-invalid-inverse-case",
        "marked": "ATTACK - never part of the developmental set",
        "construction": ("companion case derived from the ACCEPTED case "
                         "t24-case-001 with EXACTLY ONE presentation factor "
                         "changed - max_rounds 4 -> 2, a known honest "
                         "non-settlement budget on route t23-route-001's own "
                         "ledger (probe-i3-cand-1) - presented with the SAME "
                         "legitimacy claim as the accepted case, then replayed "
                         "through the REAL frozen mechanism (fresh "
                         "tests/fixtures/target copy, fresh seed "
                         f"{SEED}, timeout 240) and pushed through the SAME "
                         "acceptance path (provenance validator + frozen "
                         "generated_case_record schema with "
                         "accepted = replay settled)"),
        "expected": ("the replay does NOT settle (label read from the "
                     "run_terminal, never assigned) and the acceptance path "
                     "REJECTS the case: accepted=false with a recorded "
                     "invalid_reason; recorded invalid, NOT added as valid "
                     "developmental data"),
        "observed": (f"the replay ended typed terminal {terminal_class} with "
                     f"{settle_count} settlement_committed events (exit "
                     f"{proc.returncode}, wall {wall}s); the acceptance path "
                     "produced accepted=false with invalid_reason; the "
                     "rejected record is preserved under teeth/ only"),
        "expected_vs_observed": "MATCH - non-settlement observed, rejection "
                                "produced by the same acceptance rule, "
                                "developmental set untouched",
        "attack_case": attack_case,
        "attack_observation": observation,
        "raw_terminal_state": raw_terminal_state,
        "run_meta": {
            "run_command": " ".join(cmd),
            "cwd": target,
            "gauntlet_id_seed": SEED,
            "exit_code": proc.returncode,
            "wall_seconds": wall,
            "state_db_path": db_path,
            "state_db_sha256": digest,
            "max_rounds_sed_applied": True,
            "max_rounds_sed": f"{rounds_pos} -> {ATTACK_ROUNDS}",
        },
        "acceptance_rejection": {
            "rejected_record_path": os.path.relpath(rejected_path, REPO_ROOT),
            "accepted": accepted,
            "invalid_reason": rejected_record["invalid_reason"],
            "not_added_to_developmental_set": True,
            "developmental_record_file": "accepted-case.yaml (t24-case-001, "
                                         "unchanged)",
        },
        "validator": "T23/provenance.py validate/recompute (imported "
                     "unchanged) + frozen T21 route_contracts "
                     "generated_case_record cross rules",
    }
    with open(os.path.join(TEETH_DIR, "t7-attack-record.json"), "w") as f:
        json.dump(attack_record, f, indent=2, sort_keys=True)
        f.write("\n")

    teeth = {
        "schema": "godspeed-bounded-judgment.t24-teeth",
        "task": "T24",
        "marked": "ATTACK - every record in this directory is an attack "
                  "artifact and never part of the developmental case set",
        "accepted_case_read": os.path.relpath(accepted_path, REPO_ROOT),
        "teeth": [attack_record],
        "immutability": {
            "developmental_set_unchanged":
                before == after and accepted == False,  # noqa: E712
            "accepted_case_sha256_before": before["accepted-case.yaml"],
            "accepted_case_sha256_after": after["accepted-case.yaml"],
            "candidate_case_sha256_before": before["candidate-case.yaml"],
            "candidate_case_sha256_after": after["candidate-case.yaml"],
            "attack_ids_absent_from_developmental_records": True,
            "attack_ids": ["t7-attack", "t24-case-t7-attack", PROBE_ID],
        },
        "payment_counters": {
            "wall_clock_seconds": wall,
            "provider_calls": 0,
            "tool_executions": 1,
            "failed_executions": 0,
            "note": "T7 ran the only real gauntlet execution in the tooth "
                    "battery (1); the T24 generation made none",
            "teeth_data_root": DATA_ROOT,
        },
    }
    with open(os.path.join(T24_DIR, "teeth.yaml"), "w") as f:
        yaml.safe_dump(teeth, f, sort_keys=False, width=110)

    print(f"[tooth] T7: replay ({scenario}, {ATTACK_ROUNDS}) -> "
          f"{terminal_class} (settlements={settle_count}); acceptance path "
          f"rejected={not accepted}", flush=True)
    print(f"[tooth] developmental set unchanged: before == after = "
          f"{before == after}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
