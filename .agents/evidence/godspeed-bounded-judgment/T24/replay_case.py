#!/usr/bin/env python3
"""T24 independent replay validator: runs the FROZEN candidate case through
the real mechanism and decides acceptance ONLY by the replay's own settled
terminal.

Prereg inverse_generation_acceptance_criterion (frozen): accepted only if an
independent replay (fresh disposable target copy, identical frozen config,
fresh id seed) genuinely settles through gauntlet's own settlement path.
Generator assertions are never acceptance evidence; rejections are recorded
as invalid with the replay evidence preserved.

Frozen mechanism (addendum-1, identical to the T22/T23 probes): fresh copy of
/home/sprime01/projects/gauntlet/tests/fixtures/target (mock-agent runner
default; NO runner sed); the case's max_rounds (4) equals the fixture default
so the budget lever needs no sed; env = GAUNTLET_STATE_DIR /
GAUNTLET_ARTIFACT_DIR (durable per-run dirs under
$HOME/.local/share/godspeed-route-discovery/T24/), GAUNTLET_MOCK_SCENARIO,
GAUNTLET_ID_SEED (FRESH, unique against every seed recorded by T22/T23 and
recorded here), GAUNTLET_CLOCK=deterministic, GAUNTLET_STALL_TIMEOUT_MS=60000,
GAUNTLET_RETRY_BASE_MS=100, GAUNTLET_RETRY_MAX_MS=2000,
GAUNTLET_MAX_ATTEMPTS_PER_UNIT=2, GAUNTLET_TOOL_TEST=<target>/scripts/check.sh;
`timeout 240 <gauntlet> run "Build and verify the calculator page"`. Ground
truth = run_terminal + settlement_committed from the state database read
read-only, path+sha256 pinned (T23 provenance.recompute, imported unchanged,
is the absorb-time gate).

Outputs (append-only):
  T24/replay-record.yaml    the replay run evidence (seed, target, pinned DB,
                            terminal, settlements, recomputation, freeze
                            ordering facts)
  T24/accepted-case.yaml    the completed typed T21 generated_case_record
                            (accepted=true) - ONLY when the replay settled
  T24/rejected-case.yaml    the completed typed record (accepted=false,
                            invalid_reason recorded) - otherwise
The frozen candidate-case.yaml is NEVER modified.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import datetime
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
FIXTURE_DEFAULT_MAX_ROUNDS = 4
PROBE_CMD_WORD = "Build and verify the calculator page"
PROBE_TIMEOUT_S = 240
REQUIRED_CONTROL_ENV = {
    "GAUNTLET_STALL_TIMEOUT_MS": "60000",
    "GAUNTLET_RETRY_BASE_MS": "100",
    "GAUNTLET_RETRY_MAX_MS": "2000",
}
DATA_ROOT = os.path.join(os.path.expanduser("~"), ".local", "share",
                         "godspeed-route-discovery", "T24", "replay-001-r1")
RUN_KEY = "replay-r1"
PROBE_ID = f"probe-t24-case-001-{RUN_KEY}"
SEED = "t24-case-001-replay-r1-seed"

# every GAUNTLET_ID_SEED recorded by the preserved T22/T23 runs (freshness
# check: the replay seed must not collide with any of them)
PRIOR_SEED_SOURCES = [
    os.path.join(BRANCH_DIR, "T22", "round-2", "probe-run-meta.yaml"),
    os.path.join(BRANCH_DIR, "T22", "iteration-1", "probe-run-meta.yaml"),
    os.path.join(BRANCH_DIR, "T23", "route", "iteration-1",
                 "probe-run-meta.yaml"),
    os.path.join(BRANCH_DIR, "T23", "route", "iteration-2",
                 "probe-run-meta.yaml"),
    os.path.join(BRANCH_DIR, "T23", "route", "iteration-3",
                 "probe-run-meta.yaml"),
    os.path.join(BRANCH_DIR, "T23", "route", "iteration-4",
                 "probe-run-meta.yaml"),
]


def sha256_file(path: str) -> str:
    return prov.sha256_file(path)


def utcnow() -> str:
    return rc.utcnow()


def fail(msg: str) -> None:
    print(f"T24 replay: FAIL - {msg}", flush=True)
    sys.exit(1)


def recorded_seeds() -> dict:
    seeds = {}
    for path in PRIOR_SEED_SOURCES:
        if not os.path.isfile(path):
            fail(f"prior seed sidecar missing: {path}")
        with open(path) as f:
            doc = yaml.safe_load(f)
        for probe in doc.get("probes", []):
            seed = probe.get("gauntlet_id_seed")
            if seed:
                seeds[seed] = f"{os.path.relpath(path, BRANCH_DIR)}:" \
                              f"{probe.get('probe_id')}"
    return seeds


def main() -> int:
    for name in ("replay-record.yaml", "accepted-case.yaml",
                 "rejected-case.yaml"):
        if os.path.exists(os.path.join(T24_DIR, name)):
            fail(f"append-only violation: {name} already exists")
    if os.path.exists(DATA_ROOT):
        fail(f"replay data root already exists: {DATA_ROOT}")
    if not os.access(GAUNTLET_BIN, os.X_OK):
        fail(f"gauntlet binary missing or not executable: {GAUNTLET_BIN}")

    # ---- load the FROZEN candidate and re-verify the freeze pin -------------
    candidate_path = os.path.join(T24_DIR, "candidate-case.yaml")
    freeze_path = os.path.join(T24_DIR, "candidate-freeze.json")
    if not os.path.isfile(candidate_path) or not os.path.isfile(freeze_path):
        fail("candidate record or freeze pin missing")
    with open(freeze_path) as f:
        freeze = json.load(f)
    candidate_digest = sha256_file(candidate_path)
    if candidate_digest != freeze.get("candidate_sha256"):
        fail("candidate-case.yaml does not hash to the frozen pin; refusing "
             "to replay a modified candidate")
    if freeze.get("replay_record_exists_at_freeze") is not False:
        fail("freeze pin does not attest that it preceded any replay")
    with open(candidate_path) as f:
        candidate = yaml.safe_load(f)
    if candidate.get("replay_status") != "pending" or \
            candidate.get("case_representation", {}).get("replay", {}).get(
                "replayed") is not False:
        fail("candidate replay fields are not in the frozen EMPTY state")
    scenario = candidate["bounded_input"]["scenario"]
    rounds = candidate["bounded_input"]["max_rounds"]
    if (scenario, rounds) != ("e2e-happy", FIXTURE_DEFAULT_MAX_ROUNDS):
        fail(f"candidate case ({scenario}, {rounds}) is not the frozen "
             "derived case")
    replay_started_at = utcnow()
    freeze_mtime = os.stat(freeze_path).st_mtime
    candidate_mtime = os.stat(candidate_path).st_mtime

    # ---- fresh seed against every recorded T22/T23 seed ---------------------
    prior = recorded_seeds()
    if SEED in prior:
        fail(f"replay seed {SEED!r} collides with a recorded prior seed")
    print(f"[seed] {SEED} fresh against {len(prior)} recorded prior seeds",
          flush=True)

    # ---- fresh disposable target copy ---------------------------------------
    target = os.path.join(DATA_ROOT, "targets", RUN_KEY)
    shutil.copytree(FIXTURE_TARGET_SRC, target)
    for script in sorted(os.listdir(os.path.join(target, "scripts"))):
        if script.endswith(".sh"):
            os.chmod(os.path.join(target, "scripts", script), 0o755)
    gauntlet_md = os.path.join(target, "GAUNTLET.md")
    with open(gauntlet_md) as f:
        md = f.read()
    if "default: mock-agent" not in md:
        fail("fixture target does not carry the mock-agent runner default")
    if rounds != FIXTURE_DEFAULT_MAX_ROUNDS:
        lever = f"max_rounds: {FIXTURE_DEFAULT_MAX_ROUNDS}"
        if lever not in md:
            fail(f"GAUNTLET.md lacks the expected lever {lever!r}")
        md = md.replace(lever, f"max_rounds: {rounds}", 1)
        with open(gauntlet_md, "w") as f:
            f.write(md)
    with open(gauntlet_md) as f:
        if rounds != FIXTURE_DEFAULT_MAX_ROUNDS and \
                f"max_rounds: {rounds}" not in f.read():
            fail("max_rounds sed did not take effect")

    # ---- the REAL mechanism run ---------------------------------------------
    state_dir = os.path.join(DATA_ROOT, "runs", RUN_KEY, "state")
    artifact_dir = os.path.join(DATA_ROOT, "runs", RUN_KEY, "evidence")
    os.makedirs(state_dir, exist_ok=True)
    os.makedirs(artifact_dir, exist_ok=True)
    db_path = os.path.join(state_dir, prov.DB_FILENAME)
    if os.path.exists(db_path):
        fail(f"state db already exists before the run: {db_path}")

    env = dict(os.environ)
    for key in [k for k in env if k.startswith("GAUNTLET_")]:
        del env[key]  # defensive isolation: only the frozen env is admitted
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

    # ---- ground truth: the run's own state database, read-only --------------
    db_exists = os.path.isfile(db_path)
    settle_count = 0
    raw_terminal_state = None
    if db_exists:
        digest = sha256_file(db_path)
        try:
            log = prov.read_event_log(db_path)
        except Exception as exc:
            fail(f"replay state db unreadable: {exc}")
        states = log["run_terminal_states"]
        settle_count = log["settlement_committed_count"]
        if len(states) == 1:
            raw_terminal_state = states[0]
        elif len(states) > 1:
            fail("expected at most one run_terminal event, found "
                 f"{len(states)}")
    else:
        digest = prov.ZERO_SHA
    terminal_class = prov.classify_terminal(raw_terminal_state)
    settled = terminal_class == "settled"

    observation = {
        "probe_id": PROBE_ID,
        "candidate_id": candidate["generated_case_id"],
        "correlation_id": candidate["correlation_id"],
        "operation": "run_case",
        "bounded_input": {"scenario": scenario, "max_rounds": rounds},
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
        fail(f"internal: invalid replay probe_observation: {err}")
    # absorb-time mechanism-provenance gate (frozen T23 validator, unchanged)
    ok, reason = prov.validate(observation)
    if not ok:
        fail(f"provenance validation refused the replay observation: {reason}")
    independent = prov.recompute(observation)
    if not independent["ok"]:
        fail(f"independent recomputation refused the replay: "
             f"{independent['reasons']}")

    # ---- acceptance decision: the replay's real terminal, nothing else ------
    accepted = settled
    completed_record = {
        "case_id": candidate["generated_case_id"],
        "correlation_id": candidate["correlation_id"],
        "generator": candidate["generator"],
        "source_route_id": candidate["source_route_id"],
        "source_route_fingerprint": candidate["source_route_fingerprint"],
        "bounded_input": {"scenario": scenario, "max_rounds": rounds},
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
        "independent replay did NOT settle: typed terminal "
        f"{terminal_class} with {settle_count} settlement_committed events "
        "(prereg inverse_generation_acceptance_criterion); replay evidence "
        "preserved in replay-record.yaml",
        "created_at_utc": finished_at,
    }
    err = rc.validate("generated_case_record", completed_record)
    if err is not None:
        fail(f"internal: completed record fails the frozen schema: {err}")
    record_name = "accepted-case.yaml" if accepted else "rejected-case.yaml"
    with open(os.path.join(T24_DIR, record_name), "w") as f:
        yaml.safe_dump(completed_record, f, sort_keys=False, width=110)

    replay_record = {
        "schema": "godspeed-bounded-judgment.t24-replay-record",
        "task": "T24",
        "independence": {
            "fresh_target_copy": True,
            "fixture_source": FIXTURE_TARGET_SRC,
            "target_cwd": target,
            "same_frozen_config": "addendum-1 env block exactly as frozen in "
                                  "the candidate case_representation",
            "fresh_id_seed": SEED,
            "seed_freshness": {"seed": SEED, "collisions": 0,
                               "prior_recorded_seeds_checked": len(prior)},
            "note": "no T22/T23 state, target, or seed reused; the frozen "
                    "candidate was re-hashed against its freeze pin at "
                    "replay start",
        },
        "freeze_ordering": {
            "candidate_seen_sha256": candidate_digest,
            "freeze_pin_sha256": freeze.get("candidate_sha256"),
            "match": candidate_digest == freeze.get("candidate_sha256"),
            "candidate_mtime": candidate_mtime,
            "freeze_mtime": freeze_mtime,
            "replay_started_at_utc": replay_started_at,
            "frozen_at_utc": freeze.get("frozen_at_utc"),
            "ordering_rule": "candidate-freeze.json was written before this "
                             "replay started (replay_record_exists_at_freeze "
                             "= false attested in the pin; mtimes and "
                             "timestamps re-checked by verify_case.py)",
        },
        "run": {
            "probe_id": PROBE_ID,
            "run_command": " ".join(cmd),
            "cwd": target,
            "gauntlet_id_seed": SEED,
            "started_at_utc": started_at,
            "finished_at_utc": finished_at,
            "exit_code": proc.returncode,
            "wall_seconds": wall,
            "max_rounds_sed_applied": rounds != FIXTURE_DEFAULT_MAX_ROUNDS,
            "gauntlet_stdout_tail": (proc.stdout or "")[-500:],
            "gauntlet_stderr_tail": (proc.stderr or "")[-500:],
        },
        "observation": observation,
        "raw_terminal_state": raw_terminal_state,
        "independent_recompute": {
            "ok": independent["ok"],
            "terminal_class": independent["terminal_class"],
            "settlement_committed_count":
                independent["settlement_committed_count"],
            "db_sha256": independent["db_sha256"],
        },
        "acceptance": {
            "rule": "accepted ONLY if the replay genuinely settles (typed "
                    "terminal settled through gauntlet's own settlement "
                    "path); the label is read from the run_terminal, never "
                    "assigned",
            "accepted": accepted,
            "record_file": record_name,
        },
    }
    with open(os.path.join(T24_DIR, "replay-record.yaml"), "w") as f:
        yaml.safe_dump(replay_record, f, sort_keys=False, width=110)

    print(f"[replay] {PROBE_ID} ({scenario}, {rounds}) -> {terminal_class} "
          f"(settlements={settle_count}, exit={proc.returncode}, "
          f"wall={wall}s)", flush=True)
    print(f"[verdict] {record_name} written "
          f"(accepted={accepted}); db pinned {db_path} sha256={digest[:12]}..",
          flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
