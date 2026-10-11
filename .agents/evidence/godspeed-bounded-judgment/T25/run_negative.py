#!/usr/bin/env python3
"""T25 matched negative: from the ACCEPTED T24 case, mutate EXACTLY ONE
factor — payment capacity: max_rounds 4 -> 3 (the discovered r*-1) — and run
the REAL mechanism on a fresh disposable target copy with a fresh id seed.

Prereg matched_negative_mutation (frozen):
  single_factor: "payment capacity only: the accepted generated case's
    max_rounds lowered one notch (r* -> r*-1); no other field changes"
  expectation: "the negative must actually fail to settle; its label is read
    from the run_terminal consequence, never assigned"

The negative label is therefore READ from the new run's own state database
(read-only; path+sha256 pinned; validated by the frozen T23 provenance
validator). If the run unexpectedly settles, that terminal is recorded
honestly (exit 2) and verify_payment.py will fail the matched-pair
requirement — no label is ever assigned by this script.

Outputs (append-only):
  T25/matched-pair.yaml       positive (T24 replay run reference, pinned) +
                              negative (this run, pinned) + the recorded
                              single-factor mutation diff
  T25/payment-counters.yaml   the negative run's payment counters
The T24 files are read-only; nothing outside T25/ and the durable
$HOME/.local/share/godspeed-route-discovery/T25/ tree is written.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import os
import shutil
import subprocess
import time

import yaml

T25_DIR = os.path.dirname(os.path.abspath(__file__))
BRANCH_DIR = os.path.dirname(T25_DIR)
T24_DIR = os.path.join(BRANCH_DIR, "T24")
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
                         "godspeed-route-discovery", "T25", "negative-001-r1")
RUN_KEY = "negative-r1"
PROBE_ID = "probe-t25-negative-r1"
SEED = "t25-negative-r1-seed"

POSITIVE_ROUNDS = 4   # the accepted case (also the route's r*)
NEGATIVE_ROUNDS = 3   # r*-1: the single mutated factor

T24_ACCEPTED = os.path.join(T24_DIR, "accepted-case.yaml")
T24_REJECTED = os.path.join(T24_DIR, "rejected-case.yaml")
T24_CANDIDATE = os.path.join(T24_DIR, "candidate-case.yaml")
T24_REPLAY = os.path.join(T24_DIR, "replay-record.yaml")

PRIOR_SEED_SOURCES = [
    os.path.join(BRANCH_DIR, "T22", "round-2", "probe-run-meta.yaml"),
    os.path.join(BRANCH_DIR, "T22", "iteration-1", "probe-run-meta.yaml"),
    os.path.join(T23_DIR, "route", "iteration-1", "probe-run-meta.yaml"),
    os.path.join(T23_DIR, "route", "iteration-2", "probe-run-meta.yaml"),
    os.path.join(T23_DIR, "route", "iteration-3", "probe-run-meta.yaml"),
    os.path.join(T23_DIR, "route", "iteration-4", "probe-run-meta.yaml"),
    os.path.join(T24_DIR, "replay-record.yaml"),      # replay.seed
    os.path.join(T24_DIR, "teeth.yaml"),              # T7 attack seed
]


def recorded_seeds() -> dict:
    seeds = {}
    for path in PRIOR_SEED_SOURCES:
        if not os.path.isfile(path):
            fail(f"prior seed source missing: {path}")
        if path.endswith("replay-record.yaml"):
            with open(path) as _f:
                doc = yaml.safe_load(_f)
            seed = doc.get("run", {}).get("gauntlet_id_seed")
            if seed:
                seeds[seed] = "T24/replay-record.yaml"
            continue
        if path.endswith("teeth.yaml"):
            with open(path) as _f:
                doc = yaml.safe_load(_f)
            for tooth in doc.get("teeth", []):
                seed = tooth.get("run_meta", {}).get("gauntlet_id_seed")
                if seed:
                    seeds[seed] = "T24/teeth.yaml"
            continue
        with open(path) as _f:
            _doc = yaml.safe_load(_f)
        for probe in _doc.get("probes", []):
            seed = probe.get("gauntlet_id_seed")
            if seed:
                seeds[seed] = f"{os.path.relpath(path, BRANCH_DIR)}:" \
                              f"{probe.get('probe_id')}"
    return seeds


def fail(msg: str):
    print(f"T25 negative: FAIL - {msg}", flush=True)
    sys.exit(1)


def case_config(case_envelope: dict, rounds: int) -> dict:
    """The case configuration of one side of the pair (everything EXCEPT the
    identity/created fields): bounded input + the frozen mechanism
    parameters the run actually uses."""
    rep = case_envelope["case_representation"]
    return {
        "bounded_input": {"scenario": case_envelope["bounded_input"]
                          ["scenario"], "max_rounds": rounds},
        "fixture_source": FIXTURE_TARGET_SRC,
        "runner_default": rep["target_recipe"]["runner_default"],
        "run_command": ("timeout 240 /home/sprime01/projects/gauntlet/"
                        "target/debug/gauntlet run " + PROBE_CMD_WORD),
        "timeout_s": PROBE_TIMEOUT_S,
        "frozen_env_block": rep["frozen_env_block"],
        "observer_horizon": rep["observer_horizon"],
    }


def main() -> int:
    for name in ("matched-pair.yaml", "payment-counters.yaml"):
        if os.path.exists(os.path.join(T25_DIR, name)):
            fail(f"append-only violation: {name} already exists")
    if os.path.exists(DATA_ROOT):
        fail(f"negative data root already exists: {DATA_ROOT}")
    if not os.access(GAUNTLET_BIN, os.X_OK):
        fail(f"gauntlet binary missing or not executable: {GAUNTLET_BIN}")

    # ---- positive side: the T24 accepted case and its replay run ------------
    positive_record_path = T24_ACCEPTED if os.path.isfile(T24_ACCEPTED) \
        else (T24_REJECTED if os.path.isfile(T24_REJECTED) else None)
    if positive_record_path is None:
        fail("no T24 decision record (accepted-case.yaml nor "
             "rejected-case.yaml); T25 has no case to mutate")
    with open(positive_record_path) as _f:
        positive_case = yaml.safe_load(_f)
    with open(T24_CANDIDATE) as _f:
        candidate = yaml.safe_load(_f)
    with open(T24_REPLAY) as _f:
        replay = yaml.safe_load(_f)
    pos_obs = replay["observation"]
    if positive_case.get("accepted") is not True:
        print("[pair] POSITIVE IS NOT ACCEPTED: T24's replay did not "
              "settle; the negative is compared against the non-accepted "
              "candidate, honestly labeled", flush=True)
    if pos_obs["bounded_input"]["max_rounds"] != POSITIVE_ROUNDS:
        fail("the T24 replayed case is not the expected r* case")
    positive = {
        "source": "T24 accepted generated case t24-case-001 (independent "
                  "replay run, unchanged)",
        "record_file": os.path.relpath(positive_record_path, REPO_ROOT),
        "accepted": positive_case.get("accepted", False),
        "probe_id": pos_obs["probe_id"],
        "bounded_input": dict(pos_obs["bounded_input"]),
        "terminal_class": pos_obs["terminal_class"],
        "settled": pos_obs["terminal_class"] == "settled",
        "settlement_committed_count":
            pos_obs["settlement_committed_count"],
        "state_db_path": pos_obs["state_db_path"],
        "state_db_sha256": pos_obs["state_db_sha256"],
        "gauntlet_id_seed": replay["run"]["gauntlet_id_seed"],
    }

    # ---- the mutated negative config (exactly one factor) -------------------
    config_positive = case_config(candidate, POSITIVE_ROUNDS)
    config_negative = case_config(candidate, NEGATIVE_ROUNDS)
    diff = {}
    for key in sorted(set(config_positive) | set(config_negative)):
        a, b = config_positive.get(key), config_negative.get(key)
        if a != b:
            diff[key] = {"positive": a, "negative": b}
    if set(diff) != {"bounded_input"}:
        fail(f"mutation diff touches {sorted(diff)}; exactly the "
             "bounded_input (max_rounds) must differ")
    if diff["bounded_input"]["positive"]["max_rounds"] != POSITIVE_ROUNDS \
            or diff["bounded_input"]["negative"]["max_rounds"] != \
            NEGATIVE_ROUNDS or \
            diff["bounded_input"]["positive"]["scenario"] != \
            diff["bounded_input"]["negative"]["scenario"]:
        fail("the recorded diff is not exactly max_rounds 4 -> 3 on the "
             "same scenario")

    # ---- fresh seed ----------------------------------------------------------
    prior = recorded_seeds()
    if SEED in prior:
        fail(f"negative seed {SEED!r} collides with a recorded prior seed")
    print(f"[seed] {SEED} fresh against {len(prior)} recorded prior seeds",
          flush=True)

    # ---- fresh disposable target copy, budget sed 4 -> 3 ---------------------
    target = os.path.join(DATA_ROOT, "targets", RUN_KEY)
    shutil.copytree(FIXTURE_TARGET_SRC, target)
    for script in sorted(os.listdir(os.path.join(target, "scripts"))):
        if script.endswith(".sh"):
            os.chmod(os.path.join(target, "scripts", script), 0o755)
    gauntlet_md = os.path.join(target, "GAUNTLET.md")
    with open(gauntlet_md) as f:
        md = f.read()
    lever = f"max_rounds: {POSITIVE_ROUNDS}"
    if lever not in md:
        fail(f"GAUNTLET.md lacks the expected lever {lever!r}")
    md = md.replace(lever, f"max_rounds: {NEGATIVE_ROUNDS}", 1)
    with open(gauntlet_md, "w") as f:
        f.write(md)
    with open(gauntlet_md) as f:
        if f"max_rounds: {NEGATIVE_ROUNDS}" not in f.read():
            fail("max_rounds sed did not take effect")

    # ---- the REAL mechanism run ----------------------------------------------
    scenario = candidate["bounded_input"]["scenario"]
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
    started = time.monotonic()
    started_at = rc.utcnow()
    proc = subprocess.run(cmd, capture_output=True, text=True, cwd=target,
                          env=env)
    wall = round(time.monotonic() - started, 3)
    finished_at = rc.utcnow()

    # ---- ground truth: the run's own state database, read-only ---------------
    if not os.path.isfile(db_path):
        fail("the negative run produced no state database")
    digest = prov.sha256_file(db_path)
    log = prov.read_event_log(db_path)
    states = log["run_terminal_states"]
    settle_count = log["settlement_committed_count"]
    if len(states) != 1:
        fail(f"expected exactly one run_terminal event, found {len(states)}")
    raw_terminal_state = states[0]
    terminal_class = prov.classify_terminal(raw_terminal_state)
    observation = {
        "probe_id": PROBE_ID,
        "candidate_id": "t25-negative-r1",
        "correlation_id": "t25-negative-r1",
        "operation": "run_case",
        "bounded_input": {"scenario": scenario,
                          "max_rounds": NEGATIVE_ROUNDS},
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
        fail(f"internal: invalid negative observation: {err}")
    ok, reason = prov.validate(observation)
    if not ok:
        fail(f"provenance validation refused the negative observation: "
             f"{reason}")

    # the label is READ here (and re-read independently by the gate); it is
    # never assigned. `matched` records whether the preregistered
    # expectation actually held.
    settled = terminal_class == "settled"
    negative = {
        "source": "this task's single-factor mutated case run through the "
                  "real mechanism",
        "probe_id": PROBE_ID,
        "bounded_input": dict(observation["bounded_input"]),
        "terminal_class": terminal_class,
        "raw_terminal_state": raw_terminal_state,
        "settled": settled,
        "settlement_committed_count": settle_count,
        "state_db_path": db_path,
        "state_db_sha256": digest,
        "gauntlet_id_seed": SEED,
        "run": {
            "run_command": " ".join(cmd),
            "cwd": target,
            "started_at_utc": started_at,
            "finished_at_utc": finished_at,
            "exit_code": proc.returncode,
            "wall_seconds": wall,
            "max_rounds_sed_applied": True,
            "max_rounds_sed": f"{POSITIVE_ROUNDS} -> {NEGATIVE_ROUNDS}",
            "gauntlet_stdout_tail": (proc.stdout or "")[-500:],
            "gauntlet_stderr_tail": (proc.stderr or "")[-500:],
        },
    }

    pair = {
        "schema": "godspeed-bounded-judgment.t25-matched-pair",
        "task": "T25",
        "mutation": {
            "rule": "single factor: payment capacity only (prereg "
                    "matched_negative_mutation); no other field changes",
            "factor": "bounded_input.max_rounds",
            "positive": POSITIVE_ROUNDS,
            "negative": NEGATIVE_ROUNDS,
            "r_star": POSITIVE_ROUNDS,
            "r_star_minus_one": NEGATIVE_ROUNDS,
            "recorded_diff": diff,
            "diff_exactly_one_factor": True,
        },
        "label_rule": "the negative's label is READ from its own run_terminal "
                      "consequence (state database, read-only, pinned); it "
                      "is never assigned",
        "positive": positive,
        "negative": negative,
        "matched_pair_expectation": {
            "expected_negative_terminal": "a typed non-settlement "
                                          "(budget_exhausted on this "
                                          "surface)",
            "expected_holds": not settled,
            "note": "if the negative had settled, this pair would NOT be a "
                    "matched negative and verify_payment.py must fail the "
                    "requirement",
        },
        "seed_freshness": {"seed": SEED, "collisions": 0,
                           "prior_recorded_seeds_checked": len(prior)},
    }
    with open(os.path.join(T25_DIR, "matched-pair.yaml"), "w") as f:
        yaml.safe_dump(pair, f, sort_keys=False, width=110)

    payment = {
        "schema": "godspeed-bounded-judgment.t25-payment-counters",
        "task": "T25",
        "stage": "matched-negative",
        "wall_clock_seconds": wall,
        "provider_calls": {"count": 0, "by_identity": []},
        "tool_executions": 1,
        "failed_executions": 0,
        "note": "the negative is a mechanical single-factor mutation: no "
                "provider calls; 1 real gauntlet execution",
        "tokens_and_cost": "n/a: no provider calls in this stage",
        "operator_decisions": 0,
        "negative_data_root": DATA_ROOT,
    }
    with open(os.path.join(T25_DIR, "payment-counters.yaml"), "w") as f:
        yaml.safe_dump(payment, f, sort_keys=False, width=110)

    print(f"[negative] {PROBE_ID} (e2e-happy, {NEGATIVE_ROUNDS}) -> "
          f"{terminal_class} (settlements={settle_count}, "
          f"exit={proc.returncode}, wall={wall}s)", flush=True)
    if settled:
        print("[result] the negative SETTLED: the matched-pair expectation "
              "does NOT hold (recorded honestly; the gate will fail it)",
              flush=True)
        return 2
    print("[result] matched negative realized: positive settled, negative "
          "did not; labels read from the terminals", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
