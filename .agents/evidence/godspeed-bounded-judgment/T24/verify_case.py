#!/usr/bin/env python3
"""T24 GATE — verify_case.py (run from the sea-rs repo root; stdlib+PyYAML).

Independently verifies the T24 settlement-first inverse case generation
evidence and exits 0 ONLY if every check passes. Fail-closed throughout: a
missing file, a hash mismatch, an ordering violation, or an unprovable
terminal is a gate FAILURE, never a warning.

Checks
  (1) frozen inputs intact: T21 modules (via the T21 manifest), base prereg +
      addenda 1-3, the T23 provenance validator, the T23 round-1 and round-3
      route records — all by their pinned sha256s;
  (2) the SETTLED route basis: the round-3 record carries goal_met /
      settled / route_settled, its fingerprint recomputes from its moves and
      equals the frozen route fingerprint, r*=4 with r*-1=3, and the round-3
      record's own pin of the round-1 record matches;
  (3) the candidate case record: envelope fields (generated_case_id
      t24-case-001, source_route_id/fingerprint joined to the settled route,
      bounded_input inside the frozen catalog, case representation with
      target recipe / observer horizon / expected route reference, replay
      fields in the frozen EMPTY state), generator identity from the frozen
      provider ladder, provider-raw prompt + raw output present;
  (4) freeze BEFORE replay: the freeze pin's sha256 equals the candidate
      file's CURRENT hash, the replay record re-pins the same sha256, the
      mtimes order candidate <= freeze <= replay record, and the recorded
      frozen_at_utc <= replay_started_at_utc;
  (5) the replay run: FRESH seed (zero collisions against every GAUNTLET_ID_SEED
      recorded by T22/T23 and by this task), frozen run command, fresh target
      copy, and INDEPENDENT recomputation of the terminal from the pinned
      state database through the frozen T23 provenance validator (imported
      unchanged) — path + sha256 verified, exactly one run_terminal,
      settlement_committed count joined;
  (6) acceptance <=> replay settled: exactly one of accepted-case.yaml /
      rejected-case.yaml exists, it schema-validates through the FROZEN T21
      generated_case_record contract (exact field set, cross rules), its
      accepted flag equals the recomputed replay settlement, the typed
      fields join the frozen candidate envelope, and an accepted case
      carries >= 1 settlement_committed event with an empty invalid_reason;
  (7) tooth T7: teeth.yaml is marked ATTACK, the T7 record shows MATCH, the
      attack observation is independently recomputed from its pinned DB (it
      must NOT be a settlement), the acceptance path REJECTED it (accepted
      record with a nonempty invalid_reason, schema-valid), the attack is
      NOT part of the developmental set (rejected record only under teeth/),
      and the developmental records are byte-identical across the tooth
      (immutability hashes match the current files; attack ids absent from
      the developmental records);
  (8) fail-closed manifest hash checks: T24/evidence-manifest.yml in BOTH
      directions (every entry matches disk; every T24 .py/.yaml file plus
      the freeze pin, attack records, and provider-raw files are pinned).

This verifier is independent of the builder scripts: it imports only the
frozen T21 route_contracts and the frozen T23 provenance validator and
re-derives every consequence itself.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import datetime
import hashlib
import json
import os

import yaml

T24_DIR = os.path.dirname(os.path.abspath(__file__))
BRANCH_DIR = os.path.dirname(T24_DIR)
T21_DIR = os.path.join(BRANCH_DIR, "T21")
T23_DIR = os.path.join(BRANCH_DIR, "T23")
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(BRANCH_DIR)))
DATA_ROOT = os.path.join(os.path.expanduser("~"), ".local", "share",
                         "godspeed-route-discovery", "T24")

sys.path.insert(0, T21_DIR)
sys.path.insert(0, T23_DIR)

import route_contracts as rc  # noqa: E402  (frozen T21 module, unchanged)
import provenance as prov     # noqa: E402  (frozen T23 validator, unchanged)

PREREG_SHA256 = "1fb55d9c56d87d94adc4850bc5ca713db3e600297fbdc997accfab54c98ea8b5"
ADDENDUM1_SHA256 = "c472d1b63248ff5895433df16c545c4e145c8171e9d475a8f4b0732fa1c0cea8"
ADDENDUM2_SHA256 = "087887c27cc50c6c989fcf7bb214e77aca62ea74d7e32a868ef0f7dd211e4c5a"
ADDENDUM3_SHA256 = "1c13889ae0c2a9621b7f0b803b327ee6dd76a16983c9ed72b00ff309f59c6ddb"
PROVENANCE_SHA256 = "0baa7f63e94052562de2ac7037c15752265386f6bb3279d8fe5dbd67b4cce9ce"
ROUND1_ROUTE_RECORD_SHA256 = (
    "9345c43e6787ef32fbe1a7749c90872de7b5be4b46f73af9708516f4f8ded67e")
ROUND3_ROUTE_RECORD_SHA256 = (
    "c6b9ee9cff7c8f2047dd29e2eef5576f9b113d72d5f8618c6849ec618bb27097")
FROZEN_FINGERPRINT = ("e2e-happy@3:budget_exhausted;e2e-happy@6:settled;"
                      "e2e-happy@2:budget_exhausted")
FROZEN_RUN_COMMAND = ("timeout 240 /home/sprime01/projects/gauntlet/target/"
                      "debug/gauntlet run Build and verify the calculator "
                      "page")
LADDER_IDENTITIES = ("deepseek-clinepass", "muse-spark-free",
                     "deterministic-fallback-v1")
MANIFEST_NAME = "evidence-manifest.yml"
CASE_ID = "t24-case-001"
SOURCE_ROUTE_ID = "t23-route-001"
REPLAY_SEED = "t24-case-001-replay-r1-seed"
T7_SEED = "t24-t7-attack-seed"
T7_PROBE_ID = "probe-t24-t7-attack"
TEETH_DIR = os.path.join(T24_DIR, "teeth")
RESULT_CONTRACT_NAME = "b1-result.json"  # written AFTER the gate; never pinned

PRIOR_SEED_SOURCES = [
    os.path.join(BRANCH_DIR, "T22", "round-2", "probe-run-meta.yaml"),
    os.path.join(BRANCH_DIR, "T22", "iteration-1", "probe-run-meta.yaml"),
    os.path.join(T23_DIR, "route", "iteration-1", "probe-run-meta.yaml"),
    os.path.join(T23_DIR, "route", "iteration-2", "probe-run-meta.yaml"),
    os.path.join(T23_DIR, "route", "iteration-3", "probe-run-meta.yaml"),
    os.path.join(T23_DIR, "route", "iteration-4", "probe-run-meta.yaml"),
]

ERRORS: list[str] = []
CHECKS = 0


def check(condition: bool, msg: str) -> bool:
    global CHECKS
    CHECKS += 1
    if not condition:
        ERRORS.append(msg)
    return bool(condition)


def load_yaml(path: str):
    with open(path) as f:
        return yaml.safe_load(f)


def load_json(path: str):
    with open(path) as f:
        return json.load(f)


def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def parse_ts(text: str):
    return datetime.datetime.fromisoformat(text)


def main() -> int:
    print("verify_case: starting (fail-closed)", flush=True)

    # ---- (1) frozen inputs --------------------------------------------------
    manifest21_path = os.path.join(T21_DIR, "evidence-manifest.yml")
    if check(os.path.isfile(manifest21_path), "T21 evidence manifest missing"):
        for entry in load_yaml(manifest21_path).get("files", []):
            path = os.path.join(T21_DIR, entry["path"])
            if check(os.path.isfile(path),
                     f"T21 frozen file missing: {entry['path']}"):
                check(sha256_file(path) == entry["sha256"],
                      f"T21 frozen file hash mismatch: {entry['path']}")
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
        if check(os.path.isfile(path), f"frozen input missing: {name}"):
            check(sha256_file(path) == expected,
                  f"frozen input hash mismatch: {name}")
    prov_path = os.path.join(T23_DIR, "provenance.py")
    check(os.path.isfile(prov_path), "T23 provenance.py missing")
    if os.path.isfile(prov_path):
        check(sha256_file(prov_path) == PROVENANCE_SHA256,
              "T23 provenance.py does not match its pinned sha256")
    rr1_path = os.path.join(T23_DIR, "route", "route-record.yaml")
    rr3_path = os.path.join(T23_DIR, "round-3", "route-record.round3.yaml")
    for path, digest, name in ((rr1_path, ROUND1_ROUTE_RECORD_SHA256,
                                "round-1 route record"),
                               (rr3_path, ROUND3_ROUTE_RECORD_SHA256,
                                "round-3 route record")):
        if check(os.path.isfile(path), f"{name} missing: {path}"):
            check(sha256_file(path) == digest, f"{name} sha256 mismatch")

    # ---- (2) settled route basis ---------------------------------------------
    check(prov.sha256_file(rr1_path) ==
          load_yaml(rr3_path).get("basis", {}).get(
              "round1_route_record_sha256"),
          "round-3 record's round1 pin != the actual round-1 record hash")
    rr3 = load_yaml(rr3_path)
    goal = rr3.get("goal_evaluation", {})
    check(goal.get("goal_met") is True and goal.get("baseline_met") is True
          and goal.get("boundary_met") is True
          and goal.get("discriminator_met") is True,
          "round-3 route record does not carry a fully met goal")
    check(goal.get("r_star") == 4, "round-3 r_star != 4")
    record3 = rr3.get("route_record", {})
    check(record3.get("settled") is True
          and record3.get("stop_condition") == "route_settled",
          "round-3 route record is not settled")
    check(record3.get("route_id") == SOURCE_ROUTE_ID,
          "round-3 route record names a different route")
    recomputed_fp = rc.route_fingerprint(record3.get("moves", []))
    check(recomputed_fp == record3.get("fingerprint") ==
          FROZEN_FINGERPRINT,
          "round-3 fingerprint does not recompute / differs from the frozen "
          "route fingerprint")
    baseline_rows = [r for r in rr3.get("mechanical_labels_table", [])
                     if r.get("scenario") == "e2e-happy"
                     and r.get("max_rounds") == 4 and r.get("terminal") ==
                     "settled" and r.get("settlements", 0) >= 1]
    check(bool(baseline_rows),
          "route ledger lacks the settled e2e-happy@4 baseline basis")

    # ---- (3) candidate case record --------------------------------------------
    candidate_path = os.path.join(T24_DIR, "candidate-case.yaml")
    freeze_path = os.path.join(T24_DIR, "candidate-freeze.json")
    replay_path = os.path.join(T24_DIR, "replay-record.yaml")
    accepted_path = os.path.join(T24_DIR, "accepted-case.yaml")
    rejected_path = os.path.join(T24_DIR, "rejected-case.yaml")
    for path, name in ((candidate_path, "candidate-case.yaml"),
                       (freeze_path, "candidate-freeze.json"),
                       (replay_path, "replay-record.yaml")):
        if not check(os.path.isfile(path), f"{name} missing"):
            print("verify_case: FAIL (missing core record)", flush=True)
            return 1
    candidate = load_yaml(candidate_path)
    check(candidate.get("schema") ==
          "godspeed-bounded-judgment.t24-candidate-case",
          "candidate-case.yaml carries an unexpected schema")
    check(candidate.get("generated_case_id") == CASE_ID
          and candidate.get("correlation_id") == CASE_ID,
          "candidate case id is not t24-case-001")
    check(candidate.get("source_route_id") == SOURCE_ROUTE_ID
          and candidate.get("source_route_fingerprint") ==
          FROZEN_FINGERPRINT,
          "candidate does not join the settled route t23-route-001")
    check(candidate.get("source_route_record_sha256") ==
          ROUND3_ROUTE_RECORD_SHA256,
          "candidate does not pin the settled round-3 route record sha256")
    bi = candidate.get("bounded_input", {})
    check(rc.validate("candidate_move", dict(
        candidate_id=CASE_ID, correlation_id=CASE_ID, operation="run_case",
        bounded_input=bi, expected_role="establish_baseline",
        prerequisite_refs=[], generator={"identity": "g", "version": "v"},
        rationale="")) is None,
        "candidate bounded_input fails the frozen catalog contract")
    gen = candidate.get("generator", {})
    check(gen.get("identity") in LADDER_IDENTITIES,
          f"generator identity {gen.get('identity')!r} is outside the "
          "frozen provider ladder")
    if gen.get("identity") == "deterministic-fallback-v1":
        check(any(a.get("outcome", "").startswith(("provider_error",
                  "provider_timeout", "schema_failure"))
                  for a in candidate.get("generation_attempts", [])),
              "deterministic fallback without recorded provider failures")
    else:
        check(any(a.get("outcome") == "ok"
                  for a in candidate.get("generation_attempts", [])),
              "no successful recorded generation attempt for the winning "
              "generator identity")
    for a in candidate.get("generation_attempts", []):
        for key in ("prompt_path", "raw_output_path"):
            p = a.get(key)
            if p:
                check(os.path.isfile(p),
                      f"generation attempt {key} missing: {p}")
    rep = candidate.get("case_representation", {})
    check(rep.get("scenario") == bi.get("scenario")
          and rep.get("max_rounds") == bi.get("max_rounds"),
          "case_representation scenario/max_rounds != bounded_input")
    recipe = rep.get("target_recipe", {})
    check(recipe.get("fixture_source") ==
          "/home/sprime01/projects/gauntlet/tests/fixtures/target"
          and "mock-agent" in recipe.get("runner_default", "")
          and "timeout 240" in recipe.get("run_command", "")
          and recipe.get("timeout_s") == 240,
          "target recipe is not the frozen addendum-1 fixture recipe")
    env_block = rep.get("frozen_env_block", {})
    for key, value in (("GAUNTLET_CLOCK", "deterministic"),
                       ("GAUNTLET_STALL_TIMEOUT_MS", "60000"),
                       ("GAUNTLET_RETRY_BASE_MS", "100"),
                       ("GAUNTLET_RETRY_MAX_MS", "2000"),
                       ("GAUNTLET_MAX_ATTEMPTS_PER_UNIT", "2")):
        check(env_block.get(key) == value,
              f"frozen env block {key} != {value!r}")
    check(env_block.get("GAUNTLET_MOCK_SCENARIO") == bi.get("scenario"),
          "frozen env block GAUNTLET_MOCK_SCENARIO != the case scenario")
    check(isinstance(rep.get("observer_horizon"), str)
          and "run_terminal" in rep.get("observer_horizon", "")
          and "settlement_committed" in rep.get("observer_horizon", ""),
          "observer horizon does not state the frozen observation surface")
    ref = rep.get("expected_route_reference", {})
    check(ref.get("source_route_id") == SOURCE_ROUTE_ID
          and ref.get("fingerprint") == FROZEN_FINGERPRINT
          and ref.get("retained_moves") == record3.get("moves", []),
          "expected route reference does not mirror the settled route")
    frozen_replay = rep.get("replay", {})
    check(frozen_replay.get("replayed") is False
          and frozen_replay.get("probe_id") == ""
          and frozen_replay.get("state_db_path") == ""
          and frozen_replay.get("state_db_sha256") == ""
          and frozen_replay.get("terminal_class") == ""
          and frozen_replay.get("settled") is None
          and frozen_replay.get("gauntlet_id_seed") == "",
          "candidate replay fields are not in the frozen EMPTY state")
    check(candidate.get("replay_status") == "pending",
          "candidate replay_status is not the frozen pending state")

    # ---- (4) freeze BEFORE replay ----------------------------------------------
    freeze = load_json(freeze_path)
    candidate_digest = sha256_file(candidate_path)
    check(freeze.get("candidate_sha256") == candidate_digest,
          "freeze pin sha256 != current candidate-case.yaml hash")
    check(freeze.get("replay_record_exists_at_freeze") is False,
          "freeze pin does not attest it preceded any replay record")
    replay = load_yaml(replay_path)
    ordering = replay.get("freeze_ordering", {})
    check(ordering.get("candidate_seen_sha256") == candidate_digest,
          "replay record does not re-pin the frozen candidate sha256")
    check(ordering.get("freeze_pin_sha256") == candidate_digest,
          "replay-recorded freeze pin != current candidate hash")
    candidate_mtime = os.stat(candidate_path).st_mtime
    freeze_mtime = os.stat(freeze_path).st_mtime
    replay_mtime = os.stat(replay_path).st_mtime
    check(candidate_mtime <= freeze_mtime <= replay_mtime,
          "mtime ordering violated: expected candidate <= freeze <= replay "
          "record")
    try:
        check(parse_ts(freeze.get("frozen_at_utc")) <=
              parse_ts(ordering.get("replay_started_at_utc", "")),
              "recorded frozen_at_utc is not <= replay_started_at_utc")
    except (TypeError, ValueError):
        check(False, "unparseable freeze/replay timestamps")

    # ---- (5) the replay run -----------------------------------------------------
    independence = replay.get("independence", {})
    run = replay.get("run", {})
    check(independence.get("fresh_target_copy") is True
          and independence.get("fixture_source") ==
          "/home/sprime01/projects/gauntlet/tests/fixtures/target",
          "replay independence block does not attest a fresh fixture copy")
    check(run.get("run_command") == FROZEN_RUN_COMMAND,
          "replay did not use the frozen run command")
    check(run.get("cwd", "").startswith(DATA_ROOT),
          "replay cwd is not under the durable T24 data root")
    seeds_seen = {}
    for path in PRIOR_SEED_SOURCES:
        if not check(os.path.isfile(path),
                     f"prior seed sidecar missing: {path}"):
            continue
        for probe in load_yaml(path).get("probes", []):
            seed = probe.get("gauntlet_id_seed")
            if seed:
                seeds_seen[seed] = probe.get("probe_id")
    check(run.get("gauntlet_id_seed") == REPLAY_SEED,
          "replay seed is not the recorded fresh seed")
    check(REPLAY_SEED not in seeds_seen and T7_SEED not in seeds_seen,
          "T24 seeds collide with recorded T22/T23 seeds")
    check(run.get("max_rounds_sed_applied") is False,
          "the accepted case (max_rounds 4 = fixture default) must not need "
          "a budget sed")
    obs = replay.get("observation", {})
    check(rc.validate("probe_observation", obs) is None,
          f"replay observation fails the frozen schema: "
          f"{rc.validate('probe_observation', obs)}")
    check(obs.get("state_db_path", "").startswith(DATA_ROOT)
          and os.path.isfile(obs.get("state_db_path", "")),
          "replay state DB is not at its pinned durable path")
    check(obs.get("bounded_input") == dict(bi),
          "replay observation bounded_input != candidate bounded_input")
    # INDEPENDENT recomputation through the frozen validator (never trusted
    # from the record): path+sha256 verified, terminal recomputed, settlements
    # joined.
    recompute = prov.recompute(obs)
    check(recompute["ok"],
          f"independent provenance recompute failed: {recompute['reasons']}")
    check(recompute["terminal_class"] == obs.get("terminal_class")
          and recompute["settlement_committed_count"] ==
          obs.get("settlement_committed_count"),
          "recomputed terminal/settlements != the replay observation")
    recorded_recompute = replay.get("independent_recompute", {})
    check(recorded_recompute.get("terminal_class") ==
          recompute["terminal_class"]
          and recorded_recompute.get("settlement_committed_count") ==
          recompute["settlement_committed_count"]
          and recorded_recompute.get("db_sha256") == recompute["db_sha256"],
          "recorded independent_recompute != fresh recomputation")

    # ---- (6) acceptance <=> replay settled ---------------------------------------
    accepted_exists = os.path.isfile(accepted_path)
    rejected_exists = os.path.isfile(rejected_path)
    check(accepted_exists != rejected_exists,
          "exactly one of accepted-case.yaml / rejected-case.yaml must "
          "exist")
    check(replay.get("acceptance", {}).get("accepted") ==
          (recompute["terminal_class"] == "settled"),
          "replay-recorded acceptance != the recomputed replay settlement")
    if accepted_exists:
        final_path = accepted_path
    elif rejected_exists:
        final_path = rejected_path
    else:
        print("verify_case: FAIL (no acceptance decision record)",
              flush=True)
        return 1
    case_record = load_yaml(final_path)
    err = rc.validate("generated_case_record", case_record)
    check(err is None,
          f"{os.path.basename(final_path)} fails the frozen T21 "
          f"generated_case_record contract: {err}")
    check(case_record.get("case_id") == CASE_ID
          and case_record.get("source_route_id") == SOURCE_ROUTE_ID
          and case_record.get("source_route_fingerprint") ==
          FROZEN_FINGERPRINT
          and case_record.get("bounded_input") == dict(bi)
          and case_record.get("generator") == gen,
          "typed case record fields do not join the frozen candidate "
          "envelope")
    case_replay = case_record.get("replay", {})
    check(case_replay.get("probe_id") == obs.get("probe_id")
          and case_replay.get("state_db_path") == obs.get("state_db_path")
          and case_replay.get("state_db_sha256") ==
          obs.get("state_db_sha256")
          and case_replay.get("terminal_class") == obs.get("terminal_class")
          and case_replay.get("mechanism") == rc.MECHANISM,
          "typed record replay block does not join the replay evidence")
    check(case_record.get("accepted") ==
          (recompute["terminal_class"] == "settled"),
          "acceptance flag != the recomputed replay settlement (acceptance "
          "is the replay's real consequence, never an assertion)")
    if case_record.get("accepted"):
        check(case_replay.get("settled") is True
              and recompute["terminal_class"] == "settled"
              and recompute["settlement_committed_count"] >= 1
              and case_record.get("invalid_reason") == "",
              "an accepted case must carry a genuinely settled replay with "
              ">= 1 settlement_committed event and no invalid_reason")
    else:
        check(case_record.get("invalid_reason") != "",
              "a rejected case must record invalid_reason with preserved "
              "replay evidence")

    # ---- (7) tooth T7 --------------------------------------------------------------
    teeth_path = os.path.join(T24_DIR, "teeth.yaml")
    attack_json_path = os.path.join(TEETH_DIR, "t7-attack-record.json")
    rejected_attack_path = os.path.join(TEETH_DIR,
                                        "t7-attack-rejected-case.yaml")
    if check(os.path.isfile(teeth_path), "teeth.yaml missing") and \
            check(os.path.isfile(attack_json_path),
                  "teeth/t7-attack-record.json missing") and \
            check(os.path.isfile(rejected_attack_path),
                  "teeth/t7-attack-rejected-case.yaml missing"):
        teeth = load_yaml(teeth_path)
        check(teeth.get("task") == "T24", "teeth.yaml is not marked for T24")
        check("ATTACK" in str(teeth.get("marked", "")),
              "teeth.yaml is not marked ATTACK")
        by_id = {t.get("id"): t for t in teeth.get("teeth", [])}
        check("T7-invalid-inverse-case" in by_id,
              "teeth record missing: T7-invalid-inverse-case")
        t7 = by_id.get("T7-invalid-inverse-case", {})
        check(str(t7.get("expected_vs_observed", "")).startswith("MATCH"),
              "tooth T7 did not record a MATCH outcome")
        check("ATTACK" in str(t7.get("marked", "")),
              "tooth T7 is not marked ATTACK")
        attack_obs = t7.get("attack_observation", {})
        check(rc.validate("probe_observation", attack_obs) is None,
              "T7 attack observation fails the frozen probe_observation "
              "schema")
        attack_recompute = prov.recompute(attack_obs)
        check(attack_recompute["ok"],
              f"T7 attack observation recompute failed: "
              f"{attack_recompute['reasons']}")
        check(attack_recompute["terminal_class"] != "settled",
              "T7: the attack replay genuinely settled; the tooth cannot "
              "record a MATCH")
        check(attack_obs.get("state_db_path", "").startswith(DATA_ROOT),
              "T7 attack DB is not under the durable T24 data root")
        run_meta = t7.get("run_meta", {})
        check(run_meta.get("gauntlet_id_seed") == T7_SEED
              and run_meta.get("run_command") == FROZEN_RUN_COMMAND,
              "T7 run meta lacks the fresh seed / frozen command")
        attack_case = t7.get("attack_case", {})
        check(attack_case.get("bounded_input") == dict(
            scenario=bi.get("scenario"), max_rounds=2),
            "T7 attack case is not the accepted case with max_rounds 2")
        check("ATTACK" in str(attack_case.get("marked", "")),
              "T7 attack case is not marked ATTACK")
        rejection = t7.get("acceptance_rejection", {})
        check(rejection.get("accepted") is False
              and rejection.get("not_added_to_developmental_set") is True
              and str(rejection.get("invalid_reason", "")) != "",
              "T7 acceptance-path rejection is not recorded")
        rejected_attack = load_yaml(rejected_attack_path)
        err = rc.validate("generated_case_record", rejected_attack)
        check(err is None,
              f"T7 rejected record fails the frozen schema: {err}")
        check(rejected_attack.get("accepted") is False
              and rejected_attack.get("invalid_reason") != ""
              and rejected_attack.get("replay", {}).get("settled") is False,
              "T7 rejected record does not carry the honest rejection")
        check(os.path.normpath(rejection.get("rejected_record_path", "")) ==
              os.path.relpath(rejected_attack_path, REPO_ROOT),
              "T7 rejection path mismatch")
        # the developmental set is byte-identical across the tooth
        imm = teeth.get("immutability", {})
        if accepted_exists:
            check(imm.get("accepted_case_sha256_after") ==
                  sha256_file(accepted_path),
                  "accepted-case.yaml changed after the T7 attack")
        check(imm.get("candidate_case_sha256_after") ==
              sha256_file(candidate_path),
              "candidate-case.yaml changed after the T7 attack")
        check(imm.get("developmental_set_unchanged") is True,
              "T7 immutability block does not attest an unchanged "
              "developmental set")
        blob = b""
        for p in (accepted_path, candidate_path):
            if os.path.isfile(p):
                with open(p, "rb") as f:
                    blob += f.read()
        for marker in ("t7-attack", "t24-case-t7-attack", T7_PROBE_ID):
            check(marker.encode() not in blob,
                  f"attack marker {marker!r} found in a developmental "
                  "record")
        tooth_pay = teeth.get("payment_counters", {})
        check(tooth_pay.get("tool_executions") == 1
              and tooth_pay.get("provider_calls") == 0,
              "T7 tooth payment counters are inconsistent")

    # ---- (8) manifest, both directions, fail-closed ----------------------------
    manifest_path = os.path.join(T24_DIR, MANIFEST_NAME)
    if not check(os.path.isfile(manifest_path),
                 f"{MANIFEST_NAME} missing (must be frozen before the FINAL "
                 "gate run)"):
        print("verify_case: FAIL (no manifest)", flush=True)
        return 1
    manifest = load_yaml(manifest_path)
    pinned = set()
    for entry in manifest.get("files", []):
        rel = entry["path"]
        path = os.path.join(T24_DIR, rel)
        pinned.add(os.path.normpath(rel))
        if not check(os.path.isfile(path),
                     f"manifest file missing on disk: {rel}"):
            continue
        check(sha256_file(path) == entry["sha256"],
              f"manifest hash mismatch: {rel}")
    must_pin = ["candidate-case.yaml", "candidate-freeze.json",
                "replay-record.yaml", "teeth.yaml",
                os.path.join("teeth", "t7-attack-record.json"),
                os.path.join("teeth", "t7-attack-rejected-case.yaml"),
                "generate_case.py", "replay_case.py", "run_tooth.py",
                "verify_case.py"]
    if accepted_exists:
        must_pin.append("accepted-case.yaml")
    if rejected_exists:
        must_pin.append("rejected-case.yaml")
    for rel in must_pin:
        check(os.path.normpath(rel) in pinned,
              f"required T24 file not pinned in {MANIFEST_NAME}: {rel}")
    on_disk = set()
    for dirpath, dirnames, filenames in os.walk(T24_DIR):
        dirnames[:] = [d for d in dirnames if d != "__pycache__"]
        for fn in filenames:
            full = os.path.join(dirpath, fn)
            rel = os.path.normpath(os.path.relpath(full, T24_DIR))
            if fn.endswith((".py", ".yaml", ".json", ".txt")) and \
                    fn != RESULT_CONTRACT_NAME:
                on_disk.add(rel)
    on_disk.discard(os.path.normpath(MANIFEST_NAME))
    unpinned = sorted(on_disk - pinned)
    check(not unpinned,
          f"T24 .py/.yaml/.json/.txt files not pinned in {MANIFEST_NAME}: "
          f"{unpinned}")
    stale = sorted(pinned - on_disk -
                   {os.path.normpath(r) for r in must_pin})
    for rel in stale:
        check(os.path.isfile(os.path.join(T24_DIR, rel)),
              f"manifest pins a nonexistent file: {rel}")

    # ---- verdict ------------------------------------------------------------
    if ERRORS:
        print(f"verify_case: FAIL ({len(ERRORS)} problem(s) after "
              f"{CHECKS} checks)")
        for e in ERRORS:
            print(f"  - {e}")
        return 1
    print(f"verify_case: PASS ({CHECKS} checks)")
    if case_record.get("accepted"):
        print(f"  case {CASE_ID}: ACCEPTED (replay settled with "
              f"{recompute['settlement_committed_count']} "
              f"settlement_committed events; db pinned "
              f"{obs.get('state_db_sha256', '')[:12]}..)")
    else:
        print(f"  case {CASE_ID}: REJECTED (replay terminal "
              f"{recompute['terminal_class']}; evidence preserved)")
    print(f"  generator: {gen.get('identity')} ({gen.get('version')}); "
          f"frozen candidate sha256 {candidate_digest[:12]}.. verified "
          "before replay (mtimes + pin + replay re-pin)")
    print(f"  source route: {SOURCE_ROUTE_ID} settled, fingerprint "
          f"{FROZEN_FINGERPRINT!r}")
    print(f"  tooth T7: MATCH (attack replay non-settlement, acceptance "
          f"path rejected, developmental set byte-identical)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
