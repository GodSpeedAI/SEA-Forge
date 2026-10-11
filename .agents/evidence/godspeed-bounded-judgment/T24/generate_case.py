#!/usr/bin/env python3
"""T24 inverse generator: settlement-first candidate case derivation from the
SETTLED route t23-route-001 (and ONLY from it).

Contract authority (read-only, hash-verified fail-closed below):
  .agents/preregistrations/godspeed-bounded-judgment-T21.prereg.yaml
    (sha256 1fb55d9c56d87d94adc4850bc5ca713db3e600297fbdc997accfab54c98ea8b5)
    + addenda 1-3 (frozen sha256s pinned below)
  .agents/evidence/godspeed-bounded-judgment/T23/round-3/route-record.round3.yaml
    (the SETTLED route record; sha256 pinned below)

Prereg inverse_generation_acceptance_criterion (frozen): a candidate
inverse-generated case is accepted only if an INDEPENDENT REPLAY (fresh
disposable target copy, identical frozen config, fresh id seed) genuinely
settles through gauntlet's own settlement path. Generator assertions are
never acceptance evidence. This script therefore only DERIVES and FREEZES the
candidate; replay_case.py decides acceptance.

Derivation (settlement-first): the settled route's own ledger establishes
that e2e-happy@4 settles with genuine settlement_committed events (baseline
basis; also the discovered minimal settling budget r*=4 with r*-1=3
budget_exhausted). That makes (scenario=e2e-happy, max_rounds=4) THE case for
which route t23-route-001 is a legitimate solution: the case replay walks the
same fixture mechanism the route settled, and the route's retained-move
sequence is the expected route reference for the case.

Provider ladder (frozen, identical to T22/T23): primary deepseek-clinepass
via `opencode run -m cline-pass/cline-pass/deepseek-v4.1-flash` presenting the
settled route's fingerprint + goal ledger and asking for the case derivation;
on provider_error/provider_timeout/schema_failure exactly ONE substitution
via muse-spark-free (`opencode run -m opencode/muse-spark-1.3-contributor-
free`); if both fail, the DECLARED deterministic fallback
(deterministic-fallback-v1, mechanics-only, never counted as a provider
result). A provider derivation is admitted only when its echoed
bounded_input matches the mechanical derivation; anything else is a
schema_failure. Provider output is evidence only: it can never widen the
case, authorize acceptance, or override the frozen mechanism.

Outputs (append-only; nothing here is ever overwritten):
  T24/candidate-case.yaml      the candidate case envelope (replay fields
                               explicitly EMPTY until replay)
  T24/candidate-freeze.json    the freeze pin: candidate sha256 + frozen_at,
                               written BEFORE any replay exists
  T24/provider-raw/*.txt       generation prompt + raw output, verbatim
  T24/payment-counters.yaml    T24 generation payment counters
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import hashlib
import json
import os
import re
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

import route_contracts as rc  # noqa: E402  (frozen T21 module, imported unchanged)

PREREG_SHA256 = "1fb55d9c56d87d94adc4850bc5ca713db3e600297fbdc997accfab54c98ea8b5"
ADDENDUM1_SHA256 = "c472d1b63248ff5895433df16c545c4e145c8171e9d475a8f4b0732fa1c0cea8"
ADDENDUM2_SHA256 = "087887c27cc50c6c989fcf7bb214e77aca62ea74d7e32a868ef0f7dd211e4c5a"
ADDENDUM3_SHA256 = "1c13889ae0c2a9621b7f0b803b327ee6dd76a16983c9ed72b00ff309f59c6ddb"
ROUND3_ROUTE_RECORD_SHA256 = (
    "c6b9ee9cff7c8f2047dd29e2eef5576f9b113d72d5f8618c6849ec618bb27097")

GAUNTLET_BIN = "/home/sprime01/projects/gauntlet/target/debug/gauntlet"
FIXTURE_TARGET_SRC = "/home/sprime01/projects/gauntlet/tests/fixtures/target"
FIXTURE_DEFAULT_MAX_ROUNDS = 4
PROBE_CMD = ("timeout 240 /home/sprime01/projects/gauntlet/target/debug/"
             "gauntlet run Build and verify the calculator page")
PROVIDER_TIMEOUT_S = 300

PRIMARY_PROVIDER = "deepseek-clinepass"
PRIMARY_MODEL = "cline-pass/cline-pass/deepseek-v4.1-flash"
SUB_PROVIDER = "muse-spark-free"
SUB_MODEL = "opencode/muse-spark-1.3-contributor-free"

CASE_ID = "t24-case-001"
SOURCE_ROUTE_ID = "t23-route-001"
ROUND3_RECORD = os.path.join(T23_DIR, "round-3", "route-record.round3.yaml")
RAW_DIR = os.path.join(T24_DIR, "provider-raw")

provider_calls: list[dict] = []


def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def utcnow() -> str:
    return rc.utcnow()


def fail(msg: str) -> None:
    print(f"T24 generate: FAIL - {msg}", flush=True)
    sys.exit(1)


# ---------------------------------------------------------------------------
# Frozen-input verification (fail-closed)
# ---------------------------------------------------------------------------

def verify_frozen_inputs() -> None:
    manifest_path = os.path.join(T21_DIR, "evidence-manifest.yml")
    if not os.path.isfile(manifest_path):
        fail(f"T21 evidence manifest missing: {manifest_path}")
    with open(manifest_path) as f:
        manifest = yaml.safe_load(f)
    for entry in manifest.get("files", []):
        path = os.path.join(T21_DIR, entry["path"])
        if not os.path.isfile(path):
            fail(f"T21 frozen file missing: {entry['path']}")
        if sha256_file(path) != entry["sha256"]:
            fail(f"T21 frozen module hash mismatch: {entry['path']}")
    pre_dir = os.path.join(REPO_ROOT, ".agents/preregistrations")
    for name, expected in (
            ("godspeed-bounded-judgment-T21.prereg.yaml", PREREG_SHA256),
            ("godspeed-bounded-judgment-T21.prereg.addendum-1.yaml",
             ADDENDUM1_SHA256),
            ("godspeed-bounded-judgment-T21.prereg.addendum-2.yaml",
             ADDENDUM2_SHA256),
            ("godspeed-bounded-judgment-T21.prereg.addendum-3.yaml",
             ADDENDUM3_SHA256)):
        path = os.path.join(pre_dir, name)
        if not os.path.isfile(path) or sha256_file(path) != expected:
            fail(f"frozen prereg input missing or hash mismatch: {name}")
    if not os.path.isfile(ROUND3_RECORD) or \
            sha256_file(ROUND3_RECORD) != ROUND3_ROUTE_RECORD_SHA256:
        fail("settled round-3 route record missing or hash mismatch")
    print("[inputs] T21 modules + prereg + addenda 1-3 + settled round-3 "
          "route record verified (fail-closed)", flush=True)


# ---------------------------------------------------------------------------
# Mechanical settlement-first derivation (the ruler for the case fields)
# ---------------------------------------------------------------------------

def derive_case_mechanically(route_doc: dict) -> dict:
    goal = route_doc.get("goal_evaluation", {})
    record = route_doc.get("route_record", {})
    if not goal.get("goal_met") or not record.get("settled"):
        fail("the source route record does not carry a settled goal; T24 "
             "requires the SETTLED route only")
    if record.get("route_id") != SOURCE_ROUTE_ID:
        fail(f"source route id {record.get('route_id')!r} != {SOURCE_ROUTE_ID!r}")
    if record.get("stop_condition") != "route_settled":
        fail("settled route must carry stop_condition route_settled")
    fingerprint = record.get("fingerprint", "")
    if fingerprint != rc.route_fingerprint(record.get("moves", [])):
        fail("route fingerprint does not recompute from its moves")
    r_star = goal.get("r_star")
    if r_star != FIXTURE_DEFAULT_MAX_ROUNDS:
        fail(f"route r* {r_star!r} != fixture default "
             f"{FIXTURE_DEFAULT_MAX_ROUNDS}; derivation basis changed")
    if not goal.get("boundary_met"):
        fail("route boundary (r*/r*-1) not met on its own ledger")
    # baseline basis: the e2e-happy@4 probe settled on this route's ledger
    ledger = route_doc.get("mechanical_labels_table", [])
    baseline = [row for row in ledger
                if row.get("scenario") == "e2e-happy"
                and row.get("max_rounds") == FIXTURE_DEFAULT_MAX_ROUNDS
                and row.get("terminal") == "settled"
                and row.get("settlements", 0) >= 1]
    if not baseline:
        fail("route ledger does not show e2e-happy@4 settled with "
             "settlement_committed events")
    return {
        "scenario": "e2e-happy",
        "max_rounds": FIXTURE_DEFAULT_MAX_ROUNDS,
        "route_fingerprint": fingerprint,
        "retained_moves": record.get("moves", []),
        "r_star": r_star,
        "r_star_minus_one": r_star - 1,
        "baseline_settlements": baseline[0].get("settlements"),
        "goal_basis": ("baseline e2e-happy@4 settled_accepted; boundary "
                       f"r*={r_star} with r*-1={r_star - 1} budget_exhausted; "
                       "discriminator separated by the frozen addendum-3 "
                       "mechanical rule"),
    }


# ---------------------------------------------------------------------------
# Provider seam (frozen T16/T22/T23 identity; output is evidence only)
# ---------------------------------------------------------------------------

def call_provider(provider: str, model: str, prompt: str) -> dict:
    cmd = ["opencode", "run", "-m", model, prompt]
    started = time.monotonic()
    started_at = utcnow()
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True,
                              timeout=PROVIDER_TIMEOUT_S, cwd=REPO_ROOT)
    except subprocess.TimeoutExpired:
        provider_calls.append({"provider": provider, "model": model,
                               "purpose": "case_derivation",
                               "outcome": "provider_timeout",
                               "started_at_utc": started_at,
                               "duration_seconds":
                                   round(time.monotonic() - started, 3)})
        return {"status": "provider_timeout", "raw": ""}
    duration = round(time.monotonic() - started, 3)
    if proc.returncode != 0:
        provider_calls.append({"provider": provider, "model": model,
                               "purpose": "case_derivation",
                               "outcome": "provider_error",
                               "started_at_utc": started_at,
                               "duration_seconds": duration})
        return {"status": "provider_error",
                "raw": (proc.stderr or proc.stdout)[-400:]}
    provider_calls.append({"provider": provider, "model": model,
                           "purpose": "case_derivation",
                           "outcome": "responded",
                           "started_at_utc": started_at,
                           "duration_seconds": duration})
    return {"status": "raw", "raw": proc.stdout}


def derivation_prompt(provider: str, model: str, basis: dict) -> str:
    moves = "; ".join(
        f"{m['scenario']}@{m['max_rounds']}:{m['terminal_class']}"
        for m in basis["retained_moves"])
    return (
        "You are the inverse-case generator in a governed, settlement-first "
        "case-generation loop. A route of bounded gauntlet probes has "
        "GENUINELY SETTLED (verified from its own durable state databases, "
        "not asserted). Derive the ONE candidate case for which this settled "
        "route should be a legitimate solution.\n\n"
        "SETTLED ROUTE (frozen record):\n"
        f"- route_id: {SOURCE_ROUTE_ID}\n"
        f"- fingerprint (ordered retained moves with final terminals): {moves}\n"
        f"- goal ledger: {basis['goal_basis']}\n"
        f"- minimal settling budget on this route's own probes: r* = "
        f"{basis['r_star']} (r*-1 = {basis['r_star_minus_one']} exhausts "
        "payment)\n"
        f"- settled baseline probe: e2e-happy@4 with "
        f"{basis['baseline_settlements']} settlement_committed events\n\n"
        "THE CASE REPRESENTATION (typed, fixed by the harness; you supply the "
        "derivation): a case is (scenario in {e2e-happy, e2e-lying, "
        "e2e-forged-report}, max_rounds in 1..8) executed through the frozen "
        "fixture mechanism (fresh tests/fixtures/target copy, mock replay "
        "runner, deterministic clock, 240s timeout). The case is accepted "
        "later ONLY if an independent replay of it genuinely settles through "
        "gauntlet's own settlement path; your assertion is never acceptance "
        "evidence.\n\n"
        "TASK: state which single case the settled route legitimately solves "
        "and justify it from the route ledger above (which probe established "
        "the settlement, why the budget suffices, what the replay should "
        "observe).\n\n"
        "OUTPUT: STRICTLY one JSON object with EXACTLY these fields:\n"
        "{\n"
        f'  "case_id": "{CASE_ID}",\n'
        '  "source_route_id": "' + SOURCE_ROUTE_ID + '",\n'
        '  "bounded_input": {"scenario": "<e2e-happy|e2e-lying|'
        'e2e-forged-report>", "max_rounds": <integer 1..8>},\n'
        '  "derivation": "<from which settled route probe this case is '
        'derived>",\n'
        '  "why_legitimate_for_route": "<one short paragraph, evidence '
        'only>"\n'
        "}\n"
        "No prose, no markdown fences, no commentary."
    )


def parse_derivation(text: str, basis: dict) -> tuple[dict | None, str]:
    m = re.search(r"\{.*\}", text, re.DOTALL)
    if not m:
        return None, "no JSON object in provider output"
    try:
        obj = json.loads(m.group(0))
    except json.JSONDecodeError as e:
        return None, f"invalid JSON: {e}"
    required = {"case_id", "source_route_id", "bounded_input",
                "derivation", "why_legitimate_for_route"}
    if not isinstance(obj, dict) or set(obj) != required:
        return None, ("derivation must be a JSON object with exactly the "
                      "fields " + ", ".join(sorted(required)))
    if obj["case_id"] != CASE_ID:
        return None, f"case_id {obj['case_id']!r} != {CASE_ID!r}"
    if obj["source_route_id"] != SOURCE_ROUTE_ID:
        return None, ("source_route_id "
                      f"{obj['source_route_id']!r} != {SOURCE_ROUTE_ID!r}")
    bi = obj["bounded_input"]
    if not isinstance(bi, dict) or set(bi) != {"scenario", "max_rounds"}:
        return None, "bounded_input must have exactly scenario and max_rounds"
    if bi["scenario"] not in rc.SCENARIOS:
        return None, (f"scenario {bi['scenario']!r} is outside the frozen "
                      "scenario domain")
    if type(bi["max_rounds"]) is not int or \
            not (rc.MIN_ROUNDS <= bi["max_rounds"] <= rc.MAX_ROUNDS_LIMIT):
        return None, (f"max_rounds {bi['max_rounds']!r} is outside "
                      f"{rc.MIN_ROUNDS}..{rc.MAX_ROUNDS_LIMIT}")
    if (bi["scenario"], bi["max_rounds"]) != \
            (basis["scenario"], basis["max_rounds"]):
        return None, ("provider derivation proposes "
                      f"({bi['scenario']}, {bi['max_rounds']}) but the "
                      "mechanical settlement-first derivation of the settled "
                      f"route is ({basis['scenario']}, "
                      f"{basis['max_rounds']}); refusing to widen or move "
                      "the case")
    return obj, ""


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> int:
    if os.path.exists(os.path.join(T24_DIR, "candidate-case.yaml")):
        fail("append-only violation: candidate-case.yaml already exists; a "
             "correction uses a round-2 script and its own record names")
    if os.path.exists(os.path.join(T24_DIR, "candidate-freeze.json")):
        fail("append-only violation: candidate-freeze.json already exists")
    if not os.path.isdir(FIXTURE_TARGET_SRC):
        fail(f"fixture target missing: {FIXTURE_TARGET_SRC}")
    if not os.access(GAUNTLET_BIN, os.X_OK):
        fail(f"gauntlet binary missing or not executable: {GAUNTLET_BIN}")
    os.makedirs(RAW_DIR, exist_ok=True)

    verify_frozen_inputs()
    started = time.monotonic()
    with open(ROUND3_RECORD) as f:
        route_doc = yaml.safe_load(f)
    basis = derive_case_mechanically(route_doc)
    print(f"[derive] mechanical settlement-first case: "
          f"({basis['scenario']}, {basis['max_rounds']}) from "
          f"{SOURCE_ROUTE_ID} ({basis['route_fingerprint']!r})", flush=True)

    # provider ladder: primary -> ONE substitution -> declared fallback
    attempts: list[dict] = []
    derivation = None
    generator_identity = None
    generator_version = None
    generator_path = None
    for tag, provider, model in (
            ("primary", PRIMARY_PROVIDER, PRIMARY_MODEL),
            ("substitution", SUB_PROVIDER, SUB_MODEL)):
        prompt = derivation_prompt(provider, model, basis)
        prompt_path = os.path.join(RAW_DIR, f"derivation-{tag}.prompt.txt")
        with open(prompt_path, "w") as f:
            f.write(prompt)
        resp = call_provider(provider, model, prompt)
        raw_path = os.path.join(RAW_DIR, f"derivation-{tag}.raw.txt")
        with open(raw_path, "w") as f:
            f.write(resp["raw"] if resp["raw"].endswith("\n")
                    or resp["raw"] == "" else resp["raw"] + "\n")
        if resp["status"] != "raw":
            attempts.append({"attempt": tag, "provider": provider,
                             "model": model, "outcome": resp["status"],
                             "prompt_path": prompt_path,
                             "raw_output_path": raw_path})
            print(f"[derive] {tag} {provider}: {resp['status']}", flush=True)
            continue
        parsed, err = parse_derivation(resp["raw"], basis)
        outcome = "ok" if parsed is not None else f"schema_failure: {err}"
        attempts.append({"attempt": tag, "provider": provider,
                         "model": model, "outcome": outcome,
                         "prompt_path": prompt_path,
                         "raw_output_path": raw_path})
        print(f"[derive] {tag} {provider}: {outcome}", flush=True)
        if parsed is not None:
            derivation = parsed
            generator_identity, generator_version = provider, model
            generator_path = ("deepseek-clinepass primary" if tag == "primary"
                              else "muse-substitution (one substitution)")
            break
    if derivation is None:
        attempts.append({
            "attempt": "deterministic-fallback", "provider":
            "deterministic-fallback-v1", "model": "n/a",
            "outcome": "ok (declared deterministic fallback; mechanics-only, "
                       "never counted as a provider result)"})
        generator_identity = "deterministic-fallback-v1"
        generator_version = "1"
        generator_path = "deterministic-fallback-v1 (both providers failed)"
        print("[derive] deterministic-fallback-v1 engaged", flush=True)

    # candidate envelope (replay fields explicitly EMPTY until replay)
    candidate = {
        "schema": "godspeed-bounded-judgment.t24-candidate-case",
        "task": "T24",
        "generated_case_id": CASE_ID,
        "correlation_id": CASE_ID,
        "source_route_id": SOURCE_ROUTE_ID,
        "source_route_fingerprint": basis["route_fingerprint"],
        "source_route_record": os.path.relpath(ROUND3_RECORD, REPO_ROOT),
        "source_route_record_sha256": ROUND3_ROUTE_RECORD_SHA256,
        "generator": {"identity": generator_identity,
                      "version": generator_version},
        "generator_path": generator_path,
        "generation_attempts": attempts,
        "derivation_basis": {
            "rule": "settlement-first: the case is derived from the settled "
                    "route's own ledger only (prereg relation: settled T23 "
                    "route -> candidate case); provider output is evidence, "
                    "never acceptance",
            "route_goal_basis": basis["goal_basis"],
            "baseline_probe": ("e2e-happy@4 settled with "
                               f"{basis['baseline_settlements']} "
                               "settlement_committed events on the route "
                               "ledger (probe-i1-cand-1)"),
            "provider_derivation_accepted": derivation is not None
                and derivation.get("derivation", ""),
            "provider_why_legitimate": derivation is not None
                and derivation.get("why_legitimate_for_route", ""),
        },
        "bounded_input": {"scenario": basis["scenario"],
                          "max_rounds": basis["max_rounds"]},
        "case_representation": {
            "target_recipe": {
                "fixture_source": FIXTURE_TARGET_SRC,
                "copy": "fresh disposable copy per run (never reused, never "
                        "written back)",
                "runner_default": "mock-agent (fixture default; NO runner "
                                  "sed, addendum-1)",
                "budget_lever": ("GAUNTLET.md budget.max_rounds stays the "
                                 f"fixture default {FIXTURE_DEFAULT_MAX_ROUNDS}"
                                 " (the case value equals the default; no "
                                 "sed)"),
                "run_command": PROBE_CMD,
                "timeout_s": 240,
            },
            "scenario": basis["scenario"],
            "max_rounds": basis["max_rounds"],
            "frozen_env_block": {
                "GAUNTLET_STATE_DIR": "<durable per-run state dir>",
                "GAUNTLET_ARTIFACT_DIR": "<durable per-run evidence dir>",
                "GAUNTLET_MOCK_SCENARIO": basis["scenario"],
                "GAUNTLET_ID_SEED": "<unique per run, recorded>",
                "GAUNTLET_CLOCK": "deterministic",
                "GAUNTLET_STALL_TIMEOUT_MS": "60000",
                "GAUNTLET_RETRY_BASE_MS": "100",
                "GAUNTLET_RETRY_MAX_MS": "2000",
                "GAUNTLET_MAX_ATTEMPTS_PER_UNIT": "2",
                "GAUNTLET_TOOL_TEST": "<target>/scripts/check.sh",
                "PYTHONDONTWRITEBYTECODE": "1",
            },
            "observer_horizon": ("the run's own event log over one 240s "
                                 "window, read read-only from the pinned "
                                 "state database: the typed run_terminal "
                                 "state, the settlement_committed events and "
                                 "their unit-level progression, the "
                                 "unit_dispatch retry pattern, and the "
                                 "evidence_produced artifacts by sha256; "
                                 "nothing outside the run's own records"),
            "expected_route_reference": {
                "source_route_id": SOURCE_ROUTE_ID,
                "legitimacy_claim": ("route t23-route-001 is a legitimate "
                                     "solution to this case: the route "
                                     "settled this exact operation "
                                     "(e2e-happy@4) with genuine "
                                     "settlement_committed events on its "
                                     "own ledger"),
                "retained_moves": basis["retained_moves"],
                "fingerprint": basis["route_fingerprint"],
            },
            "replay": {
                "replayed": False,
                "probe_id": "",
                "state_db_path": "",
                "state_db_sha256": "",
                "terminal_class": "",
                "settled": None,
                "settlement_committed_count": None,
                "gauntlet_id_seed": "",
                "note": "EMPTY until the independent replay; acceptance is "
                        "decided only by the replay's real settlement",
            },
        },
        "acceptance_rule": ("accepted ONLY if the independent replay (fresh "
                            "target copy, same frozen config, fresh id seed) "
                            "genuinely settles through gauntlet's own "
                            "settlement path; generator assertions are never "
                            "acceptance evidence"),
        "replay_status": "pending",
        "accepted": False,
        "invalid_reason": "",
        "created_at_utc": utcnow(),
    }
    candidate_path = os.path.join(T24_DIR, "candidate-case.yaml")
    with open(candidate_path, "w") as f:
        yaml.safe_dump(candidate, f, sort_keys=False, width=110)

    # FREEZE before any replay exists
    digest = sha256_file(candidate_path)
    freeze = {
        "schema": "godspeed-bounded-judgment.t24-candidate-freeze",
        "task": "T24",
        "candidate_case_path": os.path.relpath(candidate_path, REPO_ROOT),
        "candidate_sha256": digest,
        "frozen_at_utc": utcnow(),
        "frozen_before": "any replay run (no replay record exists at freeze "
                         "time; replay_case.py re-verifies this pin before "
                         "running)",
        "replay_record_exists_at_freeze":
            os.path.exists(os.path.join(T24_DIR, "replay-record.yaml")),
        "rule": "the candidate case record is frozen BEFORE replaying; the "
                "sha256 pin is the freeze identity checked by the replay and "
                "by verify_case.py",
    }
    if freeze["replay_record_exists_at_freeze"]:
        fail("a replay record already exists at freeze time; freeze must "
             "precede the replay")
    with open(os.path.join(T24_DIR, "candidate-freeze.json"), "w") as f:
        json.dump(freeze, f, indent=2, sort_keys=True)
        f.write("\n")

    payment = {
        "schema": "godspeed-bounded-judgment.t24-payment-counters",
        "task": "T24",
        "stage": "generation",
        "wall_clock_seconds": round(time.monotonic() - started, 3),
        "provider_calls": {
            "count": len(provider_calls),
            "by_identity": [
                {"seq": i + 1, "purpose": c["purpose"],
                 "provider": c["provider"], "model": c["model"],
                 "outcome": c["outcome"], "started_at_utc":
                     c["started_at_utc"],
                 "duration_seconds": c["duration_seconds"]}
                for i, c in enumerate(provider_calls)],
        },
        "generator_path": generator_path,
        "tool_executions": 0,
        "note": "generation makes NO gauntlet executions; the replay and the "
                "T7 tooth executions are counted in their own records",
        "tokens_and_cost": "n/a: the frozen provider seam (opencode run) does "
                           "not expose token/cost accounting; honest gap per "
                           "prereg payment_counters_frozen.tokens_and_cost",
        "operator_decisions": 0,
    }
    with open(os.path.join(T24_DIR, "payment-counters.yaml"), "w") as f:
        yaml.safe_dump(payment, f, sort_keys=False, width=110)

    print(f"[freeze] candidate-case.yaml frozen sha256={digest}", flush=True)
    print(f"[done] generator_path={generator_path}; case="
          f"({basis['scenario']}, {basis['max_rounds']})", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
