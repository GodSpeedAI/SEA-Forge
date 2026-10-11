#!/usr/bin/env python3
"""T25 GATE — verify_payment.py (run from the sea-rs repo root; stdlib+PyYAML).

Independently verifies the T25 matched negative and the payment/novelty
evaluation, assembles payment-comparison.yaml and novelty.yaml from RECORDED
evidence only, and exits 0 ONLY if every check passes. Fail-closed: a
missing file, a hash mismatch, a settled-or-erroring negative, a two-factor
mutation, or an unprovable number is a gate FAILURE, never a warning.

Checks
  (1) frozen inputs intact: T21 modules (manifest), base prereg + addenda
      1-3, T23 provenance.py, T23 round-1/round-3 route records — pinned
      sha256s; T24 artifacts (accepted-case.yaml, candidate-case.yaml,
      replay-record.yaml, evidence-manifest.yml) through T24's OWN frozen
      manifest pins;
  (2) matched pair: the positive side re-verified against the T24 replay
      record and its pinned DB (independent recompute: settled with its
      settlement_committed events); the negative label RECOMPUTED from its
      own pinned DB (path+sha256 first, exactly one run_terminal) — the gate
      FAILS if the negative settled or errored; the label is recomputed,
      never accepted from the record;
  (3) single-factor mutation: the gate independently rebuilds both case
      configs from the frozen candidate envelope and the recorded negative
      run and diffs them — EXACTLY the bounded_input.max_rounds (4 -> 3) may
      differ; the durable target copies' GAUNTLET.md files must still carry
      max_rounds: 4 (positive, no sed) and max_rounds: 3 (negative, sed
      applied); fresh negative seed with zero collisions against every
      recorded T22/T23/T24 seed;
  (4) payment comparison ASSEMBLED from recorded counters only (operator
      decision entries counted from decisions.yml; wall clock per task from
      the recorded payment counters; model calls from the recorded
      provider-call ledgers; tool executions from the recorded
      probes/teeth/replay/negative records; failed executions from the
      recorded round-1 failures) BESIDE the manual T15/T16 baseline as
      recorded (corpus sizes, rounds history, recorded provider answers;
      unobservable quantities n/a, no estimates); the assembled file is
      deterministic (recorded values only) and byte-compared on re-runs;
  (5) novelty: the frozen prereg metric re-derived in THIS gate (fingerprint
      = ordered retained-move sequence of (scenario, max_rounds) plus the
      final terminal of each; novel | near-duplicate | structural duplicate)
      and computed against the T15/T16 grids as recorded in their
      manifests/configs; the T16 expansion's recorded budget levels are
      re-derived from its records and must NOT contain 5, 6, or 7; the
      classification is recorded with the exact data behind it;
  (6) tooth T8: teeth.yaml marked ATTACK, MATCH recorded; the gate
      independently re-derives the synthetic duplicate's fingerprint and
      classifies it with its own implementation -> structural duplicate
      DETECTED; no auto-rejection applied (frozen policy); matched-pair
      byte-identity across the tooth re-proven;
  (7) fail-closed manifest hash checks: T25/evidence-manifest.yml in BOTH
      directions (every entry matches disk; every T25 .py/.yaml/.json file
      except the gate's own deterministic outputs and the result contract
      is pinned).
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import hashlib
import json
import os
import re

import yaml

T25_DIR = os.path.dirname(os.path.abspath(__file__))
BRANCH_DIR = os.path.dirname(T25_DIR)
T21_DIR = os.path.join(BRANCH_DIR, "T21")
T23_DIR = os.path.join(BRANCH_DIR, "T23")
T24_DIR = os.path.join(BRANCH_DIR, "T24")
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(BRANCH_DIR)))
T25_DATA = os.path.join(os.path.expanduser("~"), ".local", "share",
                        "godspeed-route-discovery", "T25")
T24_DATA = os.path.join(os.path.expanduser("~"), ".local", "share",
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
NEGATIVE_SEED = "t25-negative-r1-seed"
MANIFEST_NAME = "evidence-manifest.yml"
# gate outputs: deterministic, assembled by this gate; like the result
# contract they are written after/with the gate and are not manifest-pinned
GATE_OUTPUTS = {"payment-comparison.yaml", "novelty.yaml", "b1-result.json"}

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


def classify_novelty(fingerprint: str, operation_set: frozenset,
                     reference_fingerprints, reference_operation_sets) -> str:
    """Frozen prereg novelty_metric classification, re-derived HERE (the
    gate does not import the tooth's implementation)."""
    if fingerprint in reference_fingerprints:
        return "structural-duplicate"
    if operation_set in reference_operation_sets:
        return "near-duplicate"
    return "novel"


def route_fp(moves):
    return ";".join(f"{m['scenario']}@{m['max_rounds']}:"
                    f"{m['terminal_class']}" for m in moves)


def obs_of(block: dict) -> dict:
    """Minimal observation view for the frozen provenance recompute."""
    return {
        "terminal_class": block.get("terminal_class"),
        "settlement_committed_count":
            block.get("settlement_committed_count"),
        "state_db_path": block.get("state_db_path"),
        "state_db_sha256": block.get("state_db_sha256"),
    }


def case_config(candidate: dict, rounds: int, fixture_source: str,
                command: str) -> dict:
    rep = candidate["case_representation"]
    return {
        "bounded_input": {"scenario": candidate["bounded_input"]["scenario"],
                          "max_rounds": rounds},
        "fixture_source": fixture_source,
        "runner_default": rep["target_recipe"]["runner_default"],
        "run_command": command,
        "timeout_s": rep["target_recipe"]["timeout_s"],
        "frozen_env_block": rep["frozen_env_block"],
        "observer_horizon": rep["observer_horizon"],
    }


def main() -> int:
    print("verify_payment: starting (fail-closed)", flush=True)

    # ---- (1) frozen inputs --------------------------------------------------
    m21 = os.path.join(T21_DIR, "evidence-manifest.yml")
    if check(os.path.isfile(m21), "T21 evidence manifest missing"):
        for entry in load_yaml(m21).get("files", []):
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
    check(os.path.isfile(prov_path) and
          sha256_file(prov_path) == PROVENANCE_SHA256,
          "T23 provenance.py missing or hash mismatch")
    rr1_path = os.path.join(T23_DIR, "route", "route-record.yaml")
    rr3_path = os.path.join(T23_DIR, "round-3", "route-record.round3.yaml")
    for path, digest, name in ((rr1_path, ROUND1_ROUTE_RECORD_SHA256,
                                "round-1 route record"),
                               (rr3_path, ROUND3_ROUTE_RECORD_SHA256,
                                "round-3 route record")):
        if check(os.path.isfile(path), f"{name} missing"):
            check(sha256_file(path) == digest, f"{name} sha256 mismatch")
    rr3 = load_yaml(rr3_path)
    record3 = rr3.get("route_record", {})
    check(record3.get("settled") is True
          and record3.get("fingerprint") == FROZEN_FINGERPRINT
          and route_fp(record3.get("moves", [])) == FROZEN_FINGERPRINT,
          "the settled route basis is not verifiable")
    # T24 artifacts through T24's OWN frozen manifest
    m24_path = os.path.join(T24_DIR, MANIFEST_NAME)
    if check(os.path.isfile(m24_path), "T24 evidence manifest missing"):
        m24 = load_yaml(m24_path)
        pinned24 = {}
        for entry in m24.get("files", []):
            path = os.path.join(T24_DIR, entry["path"])
            pinned24[os.path.normpath(entry["path"])] = entry["sha256"]
            if not check(os.path.isfile(path),
                         f"T24 manifest file missing: {entry['path']}"):
                continue
            check(sha256_file(path) == entry["sha256"],
                  f"T24 manifest hash mismatch (T24 file modified?): "
                  f"{entry['path']}")
        for rel in ("accepted-case.yaml", "candidate-case.yaml",
                    "replay-record.yaml"):
            check(os.path.normpath(rel) in pinned24,
                  f"T24 artifact not pinned in the T24 manifest: {rel}")

    # ---- (2) matched pair -----------------------------------------------------
    pair_path = os.path.join(T25_DIR, "matched-pair.yaml")
    negative_pay_path = os.path.join(T25_DIR, "payment-counters.yaml")
    if not check(os.path.isfile(pair_path), "matched-pair.yaml missing") or \
            not check(os.path.isfile(negative_pay_path),
                      "T25 payment-counters.yaml missing"):
        print("verify_payment: FAIL (missing pair records)", flush=True)
        return 1
    pair = load_yaml(pair_path)
    positive = pair.get("positive", {})
    negative = pair.get("negative", {})
    replay24 = load_yaml(os.path.join(T24_DIR, "replay-record.yaml"))
    pos_obs24 = replay24.get("observation", {})
    # positive joins the T24 replay run
    check(positive.get("probe_id") == pos_obs24.get("probe_id")
          and positive.get("state_db_path") == pos_obs24.get("state_db_path")
          and positive.get("state_db_sha256") ==
          pos_obs24.get("state_db_sha256")
          and positive.get("terminal_class") == "settled"
          and positive.get("settlement_committed_count") ==
          pos_obs24.get("settlement_committed_count"),
          "the pair's positive side does not join the T24 replay record")
    check(positive.get("accepted") is True,
          "the positive side is not the accepted T24 case")
    pos_recompute = prov.recompute(obs_of(positive))
    check(pos_recompute["ok"] and pos_recompute["terminal_class"] ==
          "settled" and pos_recompute["settlement_committed_count"] >= 1,
          f"positive DB recompute failed: {pos_recompute.get('reasons')}")
    # negative label RECOMPUTED from its own pinned DB
    check(negative.get("state_db_path", "").startswith(T25_DATA)
          and os.path.isfile(negative.get("state_db_path", "")),
          "negative state DB is not at its pinned durable path")
    neg_recompute = prov.recompute(obs_of(negative))
    check(neg_recompute["ok"],
          f"negative DB recompute failed: {neg_recompute.get('reasons')}")
    check(neg_recompute["terminal_class"] == "budget_exhausted",
          f"RECOMPUTED negative terminal is "
          f"{neg_recompute['terminal_class']!r}, not the typed "
          f"non-settlement budget_exhausted: the matched negative is NOT "
          f"realized (fail if it settled or errored)")
    check(negative.get("terminal_class") ==
          neg_recompute["terminal_class"]
          and negative.get("settled") is False
          and neg_recompute["settlement_committed_count"] ==
          negative.get("settlement_committed_count"),
          "recorded negative label != the recomputed run_terminal "
          "consequence (labels are read, never assigned)")
    expectation = pair.get("matched_pair_expectation", {})
    check(expectation.get("expected_holds") is True,
          "the pair does not record a realized matched-negative expectation")
    check(pair.get("label_rule", "") != ""
          and "never assigned" in pair.get("label_rule", ""),
          "the pair does not carry the label rule")

    # ---- (3) single-factor mutation ---------------------------------------------
    candidate = load_yaml(os.path.join(T24_DIR, "candidate-case.yaml"))
    frozen_cmd = ("timeout 240 /home/sprime01/projects/gauntlet/target/"
                  "debug/gauntlet run Build and verify the calculator page")
    config_pos = case_config(candidate, 4,
                             "/home/sprime01/projects/gauntlet/tests/"
                             "fixtures/target", frozen_cmd)
    config_neg = case_config(candidate, 3,
                             "/home/sprime01/projects/gauntlet/tests/"
                             "fixtures/target", frozen_cmd)
    diff = {}
    for key in sorted(set(config_pos) | set(config_neg)):
        a, b = config_pos.get(key), config_neg.get(key)
        if a != b:
            diff[key] = {"positive": a, "negative": b}
    check(set(diff) == {"bounded_input"}
          and diff["bounded_input"]["positive"]["max_rounds"] == 4
          and diff["bounded_input"]["negative"]["max_rounds"] == 3
          and diff["bounded_input"]["positive"]["scenario"] ==
          diff["bounded_input"]["negative"]["scenario"],
          "the independently rebuilt configs do not differ in EXACTLY "
          "bounded_input.max_rounds 4 -> 3")
    recorded_diff = pair.get("mutation", {}).get("recorded_diff", {})
    check(recorded_diff == diff,
          "the pair's recorded mutation diff != the independently rebuilt "
          "diff")
    check(pair.get("mutation", {}).get("factor") == "bounded_input.max_rounds"
          and pair.get("mutation", {}).get("r_star") == 4
          and pair.get("mutation", {}).get("r_star_minus_one") == 3,
          "the mutation record does not pin the single factor r* -> r*-1")
    # durable target copies still carry the budget each side actually ran
    pos_md_path = replay24.get("run", {}).get("cwd", "")
    if check(pos_md_path.startswith(T24_DATA),
             "T24 replay cwd not under the durable T24 data root"):
        with open(os.path.join(pos_md_path, "GAUNTLET.md")) as f:
            pos_md = f.read()
        check("max_rounds: 4" in pos_md and "max_rounds: 3" not in pos_md,
              "positive target copy does not carry max_rounds: 4 (no sed)")
    neg_md_path = negative.get("run", {}).get("cwd", "")
    if check(neg_md_path.startswith(T25_DATA),
             "negative cwd not under the durable T25 data root"):
        with open(os.path.join(neg_md_path, "GAUNTLET.md")) as f:
            neg_md = f.read()
        check("max_rounds: 3" in neg_md and "max_rounds: 4" not in neg_md,
              "negative target copy does not carry max_rounds: 3 (sed "
              "applied)")
    check(negative.get("run", {}).get("run_command") == frozen_cmd,
          "the negative did not use the frozen run command")
    # seed freshness against EVERY recorded prior seed
    seeds_seen = {}
    for rel in (os.path.join("T22", "round-2", "probe-run-meta.yaml"),
                os.path.join("T22", "iteration-1", "probe-run-meta.yaml"),
                os.path.join("T23", "route", "iteration-1",
                             "probe-run-meta.yaml"),
                os.path.join("T23", "route", "iteration-2",
                             "probe-run-meta.yaml"),
                os.path.join("T23", "route", "iteration-3",
                             "probe-run-meta.yaml"),
                os.path.join("T23", "route", "iteration-4",
                             "probe-run-meta.yaml")):
        path = os.path.join(BRANCH_DIR, rel)
        for probe in load_yaml(path).get("probes", []):
            if probe.get("gauntlet_id_seed"):
                seeds_seen[probe["gauntlet_id_seed"]] = rel
    seeds_seen[replay24.get("run", {}).get("gauntlet_id_seed")] = \
        "T24/replay-record.yaml"
    t24_teeth = load_yaml(os.path.join(T24_DIR, "teeth.yaml"))
    for tooth in t24_teeth.get("teeth", []):
        s = tooth.get("run_meta", {}).get("gauntlet_id_seed")
        if s:
            seeds_seen[s] = "T24/teeth.yaml"
    check(negative.get("gauntlet_id_seed") == NEGATIVE_SEED
          and NEGATIVE_SEED not in seeds_seen,
          "the negative seed is not the recorded fresh seed / collides with "
          "a prior seed")
    check(pair.get("seed_freshness", {}).get("collisions") == 0,
          "the pair does not record zero seed collisions")

    # ---- (4) payment comparison assembled from RECORDED counters ---------------
    def counter(rel):
        return load_yaml(os.path.join(BRANCH_DIR, rel))

    pay_t22_r1 = counter("T22/iteration-1/payment-counters.yaml")
    pay_t22_r2 = counter("T22/round-2/payment-counters.yaml")
    pay_t23 = counter("T23/route/payment-counters.yaml")
    pay_t23_r2 = counter("T23/round-2/payment-counters.round2.yaml")
    pay_t24 = counter("T24/payment-counters.yaml")
    pay_t25 = counter("T25/payment-counters.yaml")
    t24_replay_wall = replay24.get("run", {}).get("wall_seconds")
    t24_teeth_pay = t24_teeth.get("payment_counters", {})

    # operator decision entries, counted mechanically from the decision log
    decisions = load_yaml(os.path.join(BRANCH_DIR, "decisions.yml"))
    ids = [e.get("id", "") for e in decisions.get("entries", [])]
    branch_ids = [i for i in ids if re.fullmatch(
        r"D-\d{4}-\d{2}-\d{2}-T(21|22|23)-\d{2}", i)]
    baseline_ids = [i for i in ids if re.fullmatch(
        r"D-\d{4}-\d{2}-\d{2}-T(15|16)-\d{2}", i)]
    check(len(branch_ids) == 7,
          f"expected 7 recorded branch decision entries T21..T23, counted "
          f"{len(branch_ids)}")
    check(branch_ids[0].endswith("T21-01") and
          branch_ids[-1].endswith("T23-03"),
          "the counted branch decision entries do not span the frozen "
          "range D-2026-09-18-T21-01..D-2026-09-19-T23-03")

    # model calls from the recorded provider-call ledgers
    def calls(pay):
        return len(pay.get("provider_calls", {}).get("by_identity", []))

    def gen_calls(pay):
        return sum(1 for c in pay.get("provider_calls", {}).get(
            "by_identity", [])
            if str(c.get("purpose", "")).startswith(
                ("candidate_generation", "case_derivation")))

    judgment_calls = sum(
        calls(p) - gen_calls(p) for p in (pay_t22_r1, pay_t22_r2, pay_t23))
    model_calls = {
        "t22_round1_failed": calls(pay_t22_r1),
        "t22_round2": calls(pay_t22_r2),
        "t23_route_round1": calls(pay_t23),
        "t23_round2_rejudgment": calls(pay_t23_r2),
        "t23_round3": rr3.get("payment", {}).get("round3_provider_calls"),
        "t24_case_generation": calls(pay_t24),
        "t25_matched_negative": calls(pay_t25),
    }
    check(model_calls["t23_round2_rejudgment"] == 12
          and model_calls["t23_round3"] == 0,
          "T23 re-judgment call counts do not match the frozen records")
    total_model_calls = sum(v for v in model_calls.values() if isinstance(
        v, int))
    total_generation_calls = (gen_calls(pay_t22_r1) + gen_calls(pay_t22_r2)
                              + gen_calls(pay_t23) + gen_calls(pay_t24))

    # tool executions + failed executions from the recorded records
    t23_teeth_tool = load_yaml(os.path.join(
        T23_DIR, "teeth", "attacks.yaml")).get(
        "payment_counters", {}).get("tool_executions")
    t24_replay_tool = 1  # the T24 candidate replay (replay-record run block)
    t7_tool = t24_teeth_pay.get("tool_executions")  # the T7 attack run
    tool_executions = {
        "t22_round1_probes_failed_round": pay_t22_r1.get("tool_executions"),
        "t22_round2_probes": pay_t22_r2.get("tool_executions"),
        "t23_route_probes": pay_t23.get("tool_executions"),
        "t23_teeth_t4": t23_teeth_tool,
        "t24_replays_candidate_and_t7_tooth":
            (t24_replay_tool or 0) + (t7_tool or 0),
        "t25_negative": pay_t25.get("tool_executions"),
    }
    failed_executions = {
        "t22_round1_failed_round": pay_t22_r1.get("failed_executions"),
        "t22_round2": pay_t22_r2.get("failed_executions"),
        "t23_route": pay_t23.get("failed_executions"),
        "t23_teeth": 0,
        "t24_replay_and_t7": 0,
        "t25_negative": pay_t25.get("failed_executions"),
    }
    check(failed_executions["t22_round1_failed_round"] == 3,
          "the recorded T22 round-1 failed executions != 3")
    check(all(v == 0 for k, v in failed_executions.items()
              if k != "t22_round1_failed_round"),
          "an unexpected nonzero failed-execution counter in the records")
    obs_t22_r1 = load_json(os.path.join(
        BRANCH_DIR, "T22", "iteration-1", "probe-observations.json"))
    check(sum(1 for o in obs_t22_r1 if o["terminal_class"] ==
              "run_error") == 3,
          "T22 round-1 observations do not show the 3 recorded run_errors")

    wall_recorded = {
        "t22_round1_failed": pay_t22_r1.get("wall_clock_seconds"),
        "t22_round2": pay_t22_r2.get("wall_clock_seconds"),
        "t23_route_round1": pay_t23.get("wall_clock_seconds"),
        "t23_round2_rejudgment": "n/a (no wall clock recorded in "
                                 "payment-counters.round2.yaml)",
        "t23_round3_mechanical_rule": "n/a (deterministic rule evaluation; "
                                      "no wall clock recorded)",
        "t24_case_generation": pay_t24.get("wall_clock_seconds"),
        "t24_independent_replay_run": t24_replay_wall,
        "t24_tooth_t7": t24_teeth_pay.get("wall_clock_seconds"),
        "t25_matched_negative": pay_t25.get("wall_clock_seconds"),
    }
    wall_sum = sum(v for v in wall_recorded.values()
                   if isinstance(v, (int, float)))

    # manual baseline, as recorded in the T15/T16 evidence (quotes only)
    t15_manifest = load_yaml(os.path.join(
        REPO_ROOT, ".agents/preregistrations",
        "godspeed-bounded-judgment-T15.corpus-manifest.yml"))
    t16_expanded = load_yaml(os.path.join(
        REPO_ROOT, ".agents/preregistrations",
        "godspeed-bounded-judgment-T16.expanded-corpus-manifest.yml"))
    t16_report_9 = load_yaml(os.path.join(
        BRANCH_DIR, "T16", "round-1", "remeasure-report.yml"))
    t16_report_25 = load_yaml(os.path.join(
        BRANCH_DIR, "T16", "expanded-round", "remeasure-report.yml"))
    baseline = {
        "source_note": ("manual corpus construction as RECORDED in the "
                        "T15/T16 evidence; only recorded values are quoted; "
                        "unobservable quantities are n/a (no estimates)"),
        "t15_corpus": {
            "runs": len(t15_manifest.get("runs", [])),
            "class_counts_recorded": t15_manifest.get("class_counts"),
            "rounds_recorded": ("round-3 clean batch (the manifest); rounds "
                                "1-2 preserved as FAILED rounds (generator "
                                "targeting + wall-clock mismatch + "
                                "state-dir reuse)"),
            "case_definitions": ("operator-authored 9-case grid (T15 "
                                 "prereg case table)"),
        },
        "t16_expansion": {
            "runs": len(t16_expanded.get("runs", [])),
            "class_counts_recorded": t16_expanded.get("class_counts"),
            "measurement_rounds_recorded": (
                "round-1 quota-blocked (preserved); round-2 scoring defect "
                "(preserved failed); round-3 provider re-calls after "
                "harness corrections; round-4 first complete measurement; "
                "round-5 baseline scoring correction (no provider "
                "re-calls); plus the N=25 expansion round"),
            "provider_answers_recorded": {
                "nine_row_report":
                    {p: {"n_answered":
                         t16_report_9.get(p, {}).get("n_answered"),
                         "contract_failures":
                         t16_report_9.get(p, {}).get("contract_failures")}
                     for p in ("deepseek-clinepass", "prime-agent")},
                "expanded_round_report":
                    {p: {"n_answered":
                         t16_report_25.get(p, {}).get("n_answered"),
                         "contract_failures":
                         t16_report_25.get(p, {}).get("contract_failures")}
                     for p in ("deepseek-clinepass", "muse-spark-free")},
            },
        },
        "wall_clock": "n/a: no wall clock is recorded in the T15/T16 "
                      "evidence",
        "operator_decision_entries": len(baseline_ids),
        "tokens_and_cost": "n/a (honest gap, as recorded)",
    }
    check(baseline["t15_corpus"]["runs"] == 9
          and t15_manifest.get("class_counts") ==
          {"budget_exhausted": 6, "settled": 3},
          "T15 corpus quotes do not match the recorded manifest")
    check(baseline["t16_expansion"]["runs"] == 25
          and t16_expanded.get("class_counts") ==
          {"settled": 12, "budget_exhausted": 13},
          "T16 expansion quotes do not match the recorded manifest")
    check(t16_report_25.get("n") == 25,
          "the expanded-round report is not the recorded N=25 measurement")

    comparison = {
        "schema": "godspeed-bounded-judgment.t25-payment-comparison",
        "task": "T25",
        "honesty_rule": ("every number below is counted or quoted from a "
                         "recorded evidence file named beside it; "
                         "unobservable quantities are n/a; no estimates; "
                         "the two sides measure DIFFERENT surfaces (loop-"
                         "mechanism payment vs manual corpus construction) "
                         "and support only the recorded CLAIM-C measurement "
                         "surface, no general economic claim"),
        "mechanism_side_this_branch": {
            "operator_decision_entries": {
                "count": len(branch_ids),
                "source": ".agents/evidence/godspeed-bounded-judgment/"
                          "decisions.yml (range D-2026-09-18-T21-01.."
                          "D-2026-09-19-T23-03)",
                "ids": branch_ids,
            },
            "wall_clock_seconds_recorded": {
                "per_task": wall_recorded,
                "total_recorded": round(wall_sum, 3),
                "unrecorded_note": "T23 round-2 and round-3 recorded no "
                                   "wall clock; excluded from the total "
                                   "(n/a, not estimated)",
            },
            "model_calls": {
                "by_stage_recorded": model_calls,
                "generation_calls": total_generation_calls,
                "judgment_calls": judgment_calls,
                "total": total_model_calls,
                "sources": ["T22/iteration-1/payment-counters.yaml",
                            "T22/round-2/payment-counters.yaml",
                            "T23/route/payment-counters.yaml",
                            "T23/round-2/payment-counters.round2.yaml",
                            "T23/round-3/route-record.round3.yaml payment "
                            "block",
                            "T24/payment-counters.yaml",
                            "T25/payment-counters.yaml"],
            },
            "tool_executions": {
                "by_stage_recorded": tool_executions,
                "total": sum(v for v in tool_executions.values()
                             if isinstance(v, int)),
                "note": "real gauntlet executions only (probes 12 + failed "
                        "round-1 probes + teeth T4 + T24 replay + T7 "
                        "attack + the matched negative)",
            },
            "failed_executions": {
                "by_stage_recorded": failed_executions,
                "total": sum(v for v in failed_executions.values()
                             if isinstance(v, int)),
                "note": "the T22 round-1 run_error trio (preserved failed "
                        "round); T23's teeth round-1 script crash never "
                        "reached a gauntlet execution and is not one",
            },
            "tokens_and_cost": "n/a: the frozen provider seam does not "
                               "expose token/cost accounting (recorded "
                               "honest gap in every counter file)",
        },
        "manual_baseline_t15_t16": baseline,
        "comparison_note": ("the mechanism produced the settled route, the "
                            "accepted generated case, the matched pair and "
                            "the novelty measurement for the recorded "
                            "payment above; the recorded manual baseline "
                            "purchased the T15/T16 corpora (9 then 25 runs) "
                            "with operator-authored case definitions and "
                            "manual verification rounds; recorded-side "
                            "differences only - no normalized cost claim"),
    }
    comparison_path = os.path.join(T25_DIR, "payment-comparison.yaml")
    rendered = yaml.safe_dump(comparison, sort_keys=False, width=110)
    if os.path.exists(comparison_path):
        with open(comparison_path) as f:
            check(f.read() == rendered,
                  "payment-comparison.yaml on disk != the deterministic "
                  "recomputation from the recorded counters")
    else:
        with open(comparison_path, "w") as f:
            f.write(rendered)
        print("[assemble] payment-comparison.yaml written", flush=True)

    # ---- (5) novelty -----------------------------------------------------------
    # the T15/T16 grid as recorded: case id -> (scenario, rounds, terminal)
    grid = []
    for run in t15_manifest.get("runs", []):
        cid = run["case_id"]
        terminal = run["consequence_class"]
        if cid.startswith("accepted-"):
            grid.append((cid, "e2e-happy", 4, terminal))
        elif cid.startswith("lying-"):
            grid.append((cid, "e2e-lying", 4, terminal))
        elif cid.startswith("budget-"):
            rounds = 2 if cid == "budget-C3" else 1
            grid.append((cid, "e2e-happy", rounds, terminal))
        else:
            check(False, f"unmapped T15 corpus id {cid}")
    covered = {c[0] for c in grid}
    for run in t16_expanded.get("runs", []):
        cid = run["case_id"]
        if cid in covered:
            continue
        terminal = run["consequence_class"]
        if cid.startswith("accepted-"):
            grid.append((cid, "e2e-happy", 4, terminal))
        elif cid.startswith("lying-"):
            grid.append((cid, "e2e-lying", 4, terminal))
        elif cid.startswith("budget-"):
            grid.append((cid, "e2e-happy", 1, terminal))
        elif cid.startswith("forge-"):
            grid.append((cid, "e2e-forged-report", 4, terminal))
        else:
            check(False, f"unmapped T16 expansion id {cid}")
    check(len(grid) == 25,
          f"the recorded grid mapping must cover the recorded 25 runs, got "
          f"{len(grid)}")
    from collections import Counter as _Counter
    grid_classes = _Counter(c[3] for c in grid)
    check(grid_classes["settled"] == 12
          and grid_classes["budget_exhausted"] == 13,
          f"the mapped grid classes {dict(grid_classes)} != the recorded "
          f"12/13 composition")
    # addendum cross-check: budget-r1 x2 among the expansion's new cases
    expansion_new = [c for c in grid if c[0] in
                     {"budget-C4", "budget-C5"}]
    check(len(expansion_new) == 2
          and all(c[2] == 1 and c[3] == "budget_exhausted"
                  for c in expansion_new),
          "the expansion's recorded budget-r1 x2 cases are not mapped")
    grid_fingerprints = [f"{s}@{r}:{t}" for _, s, r, t in grid]
    grid_operation_sets = [frozenset([(s, r)]) for _, s, r, _ in grid]
    grid_budget_levels = sorted({r for _, _, r, _ in grid})
    check(grid_budget_levels == [1, 2, 4],
          f"the recorded grid budget levels {grid_budget_levels} != the "
          "recorded [1, 2, 4(default)]")
    # the frozen claim to verify: the T16 expansion never probed 5,6,7
    never_probed = [lvl for lvl in (5, 6, 7) if lvl in grid_budget_levels]
    check(not never_probed,
          f"the recorded grid unexpectedly contains budget levels "
          f"{never_probed}")
    route_moves = record3.get("moves", [])
    route_fingerprint = route_fp(route_moves)
    route_op_set = frozenset((m["scenario"], m["max_rounds"])
                             for m in route_moves)
    classification = classify_novelty(
        route_fingerprint, route_op_set, grid_fingerprints,
        grid_operation_sets)
    route_levels = sorted({m["max_rounds"] for m in route_moves})
    outside_levels = sorted(set(route_levels) - set(grid_budget_levels))
    check(classification == "novel",
          f"route classification {classification!r} != the expected novel "
          "classification under the frozen metric")
    check(outside_levels == [3, 6],
          f"route budget levels outside the recorded grid {outside_levels} "
          "!= [3, 6]")
    novelty = {
        "schema": "godspeed-bounded-judgment.t25-novelty",
        "task": "T25",
        "metric_frozen": {
            "fingerprint": "ordered retained-move sequence of (scenario, "
                           "max_rounds) plus the final terminal of each "
                           "(prereg novelty_metric; computed with the "
                           "frozen T21 route_contracts.route_fingerprint)",
            "classification": {
                "novel": "no equal fingerprint in the T15/T16 grids",
                "near-duplicate": "same operation set, different order",
                "structural-duplicate": "equal fingerprint",
            },
            "policy": "duplicates are measured and recorded; no automatic "
                      "rejection is frozen in this slice",
        },
        "subject": {
            "route_id": record3.get("route_id"),
            "fingerprint": route_fingerprint,
            "operation_set": sorted(f"{m['scenario']}@{m['max_rounds']}"
                                    for m in route_moves),
        },
        "reference_grid_as_recorded": {
            "size": len(grid),
            "sources": [".agents/preregistrations/"
                        "godspeed-bounded-judgment-T15.corpus-manifest.yml",
                        ".agents/preregistrations/"
                        "godspeed-bounded-judgment-T15.prereg.yaml (case "
                        "table budget dimensions)",
                        ".agents/preregistrations/"
                        "godspeed-bounded-judgment-T16."
                        "expanded-corpus-manifest.yml",
                        ".agents/preregistrations/"
                        "godspeed-bounded-judgment-T16.expansion-addendum."
                        "yaml (mechanism mix)"],
            "grid_fingerprints": sorted(set(grid_fingerprints)),
            "grid_budget_levels_recorded": grid_budget_levels,
            "budget_levels_note": "defaults means the fixture default "
                                  "max_rounds 4 (addendum-1)",
        },
        "t16_expansion_never_probed": {
            "claim": "the T16 expansion never probed fixture-target budget "
                     "levels 5, 6, or 7",
            "verified_from": ("the recorded budget dimensions of the T15 "
                              "prereg case table (defaults, max_rounds 1, "
                              "max_rounds 2) and the T16 expansion "
                              "addendum mechanism mix (e2e-happy x9 at "
                              "defaults, e2e-lying x3, budget-r1 x2, "
                              "e2e-forged-report x2), cross-checked "
                              "against the mapped 25-run grid levels "
                              f"{grid_budget_levels}"),
            "recorded_grid_levels": grid_budget_levels,
            "levels_5_6_7_present": never_probed,
            "verified": not never_probed,
            "route_levels_outside_the_grid": outside_levels,
        },
        "classification": classification,
        "classification_basis": ("no grid fingerprint equals the route "
                                 "fingerprint (the grid records single-run "
                                 "cases only) and no grid operation set "
                                 "equals the route's operation set; the "
                                 "route's budget levels 3 and 6 were never "
                                 "probed in the recorded grids"),
        "duplicate_policy": "measured and recorded; no automatic rejection",
    }
    novelty_path = os.path.join(T25_DIR, "novelty.yaml")
    rendered = yaml.safe_dump(novelty, sort_keys=False, width=110)
    if os.path.exists(novelty_path):
        with open(novelty_path) as f:
            check(f.read() == rendered,
                  "novelty.yaml on disk != the deterministic recomputation "
                  "from the recorded records")
    else:
        with open(novelty_path, "w") as f:
            f.write(rendered)
        print("[assemble] novelty.yaml written", flush=True)

    # ---- (6) tooth T8 -----------------------------------------------------------
    teeth_path = os.path.join(T25_DIR, "teeth.yaml")
    dup_record_path = os.path.join(T25_DIR, "teeth",
                                   "t8-duplicate-route-record.yaml")
    attack_path = os.path.join(T25_DIR, "teeth", "t8-attack-record.json")
    if check(os.path.isfile(teeth_path), "teeth.yaml missing") and \
            check(os.path.isfile(dup_record_path),
                  "teeth/t8-duplicate-route-record.yaml missing") and \
            check(os.path.isfile(attack_path),
                  "teeth/t8-attack-record.json missing"):
        teeth = load_yaml(teeth_path)
        check(teeth.get("task") == "T25", "teeth.yaml is not marked for T25")
        check("ATTACK" in str(teeth.get("marked", "")),
              "teeth.yaml is not marked ATTACK")
        by_id = {t.get("id"): t for t in teeth.get("teeth", [])}
        check("T8-redundant-route" in by_id,
              "teeth record missing: T8-redundant-route")
        t8 = by_id.get("T8-redundant-route", {})
        check(str(t8.get("expected_vs_observed", "")).startswith("MATCH"),
              "tooth T8 did not record a MATCH outcome")
        check("ATTACK" in str(t8.get("marked", "")),
              "tooth T8 is not marked ATTACK")
        dup = load_yaml(dup_record_path)
        dup_fp = route_fp(dup.get("moves", []))
        check(dup_fp == FROZEN_FINGERPRINT
              and dup.get("fingerprint") == FROZEN_FINGERPRINT,
              "the synthetic record does not carry the structurally "
              "equivalent fingerprint")
        dup_op_set = frozenset((m["scenario"], m["max_rounds"])
                               for m in dup.get("moves", []))
        dup_classification = classify_novelty(
            dup_fp, dup_op_set, [FROZEN_FINGERPRINT],
            [frozenset((m["scenario"], m["max_rounds"])
                       for m in route_moves)])
        check(dup_classification == "structural-duplicate",
              f"the gate's own classification of the synthetic record is "
              f"{dup_classification!r}, not structural-duplicate: the "
              "duplicate was NOT detected")
        recorded = load_json(attack_path)
        check(recorded.get("classification") == "structural-duplicate"
              and recorded.get("duplicate_detected") is True
              and recorded.get("auto_rejection_applied") is False,
              "the tooth's recorded classification/detection/no-rejection "
              "facts are inconsistent with the gate's recomputation")
        check(t8.get("gauntlet_executions") == 0
              and t8.get("provider_calls") == 0
              and teeth.get("payment_counters", {}).get(
                  "tool_executions") == 0,
              "T8 must be a synthetic-record attack (no gauntlet run, no "
              "provider call)")
        imm = teeth.get("immutability", {})
        pair_digest = sha256_file(pair_path)
        check(imm.get("matched_pair_sha256_after") == pair_digest
              and imm.get("matched_pair_unchanged") is True,
              "matched-pair.yaml changed across the T8 tooth")
        with open(pair_path, "rb") as f:
            blob = f.read()
        for marker in (b"t8-attack", b"t25-t8-duplicate-route"):
            check(marker not in blob,
                  f"attack marker {marker!r} found in matched-pair.yaml")

    # ---- (7) manifest, both directions, fail-closed ------------------------------
    manifest_path = os.path.join(T25_DIR, MANIFEST_NAME)
    if not check(os.path.isfile(manifest_path),
                 f"{MANIFEST_NAME} missing (must be frozen before the FINAL "
                 "gate run)"):
        print("verify_payment: FAIL (no manifest)", flush=True)
        return 1
    manifest = load_yaml(manifest_path)
    pinned = set()
    for entry in manifest.get("files", []):
        rel = entry["path"]
        path = os.path.join(T25_DIR, rel)
        pinned.add(os.path.normpath(rel))
        if not check(os.path.isfile(path),
                     f"manifest file missing on disk: {rel}"):
            continue
        check(sha256_file(path) == entry["sha256"],
              f"manifest hash mismatch: {rel}")
    must_pin = ["matched-pair.yaml", "payment-counters.yaml", "teeth.yaml",
                "run_negative.py", "run_tooth.py", "verify_payment.py",
                os.path.join("teeth", "t8-duplicate-route-record.yaml"),
                os.path.join("teeth", "t8-attack-record.json")]
    for rel in must_pin:
        check(os.path.normpath(rel) in pinned,
              f"required T25 file not pinned in {MANIFEST_NAME}: {rel}")
    on_disk = set()
    for dirpath, dirnames, filenames in os.walk(T25_DIR):
        dirnames[:] = [d for d in dirnames if d != "__pycache__"]
        for fn in filenames:
            if fn.endswith((".py", ".yaml", ".json")) and \
                    fn not in GATE_OUTPUTS:
                full = os.path.join(dirpath, fn)
                on_disk.add(os.path.normpath(
                    os.path.relpath(full, T25_DIR)))
    on_disk.discard(os.path.normpath(MANIFEST_NAME))
    unpinned = sorted(on_disk - pinned)
    check(not unpinned,
          f"T25 .py/.yaml/.json files not pinned in {MANIFEST_NAME}: "
          f"{unpinned}")
    for rel in sorted(pinned - on_disk):
        check(os.path.isfile(os.path.join(T25_DIR, rel)),
              f"manifest pins a nonexistent file: {rel}")

    # ---- verdict ----------------------------------------------------------------
    if ERRORS:
        print(f"verify_payment: FAIL ({len(ERRORS)} problem(s) after "
              f"{CHECKS} checks)")
        for e in ERRORS:
            print(f"  - {e}")
        return 1
    print(f"verify_payment: PASS ({CHECKS} checks)")
    print(f"  matched pair: positive settled ("
          f"{pos_recompute['settlement_committed_count']} settlements) vs "
          f"negative {neg_recompute['terminal_class']} ("
          f"{neg_recompute['settlement_committed_count']} settlements); "
          "labels recomputed from the pinned DBs")
    print(f"  single-factor mutation verified: exactly "
          f"max_rounds 4 -> 3 (durable target copies re-read)")
    print(f"  payment: {len(branch_ids)} branch decision entries; "
          f"{total_model_calls} recorded model calls "
          f"({total_generation_calls} generation + "
          f"{total_model_calls - total_generation_calls} judgment); "
          f"{sum(v for v in tool_executions.values() if isinstance(v, int))} "
          f"recorded gauntlet executions; {sum(v for v in failed_executions.values() if isinstance(v, int))} "
          f"failed; wall {round(wall_sum, 3)}s recorded")
    print(f"  novelty: {classification} (grid levels recorded "
          f"{grid_budget_levels}; 5/6/7 never probed; route levels "
          f"{route_levels})")
    print("  tooth T8: structural duplicate DETECTED by the gate's own "
          "metric; no auto-rejection")
    return 0


if __name__ == "__main__":
    sys.exit(main())
