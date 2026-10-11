#!/usr/bin/env python3
"""T21 contract gate.

Proves the frozen native route-discovery contract against itself WITHOUT
executing anything: schema round-trips (valid records survive YAML/JSON
round-trips losslessly; each frozen refusal class is refused with a named
reason), admission classification vectors, frozen selection-policy unit
vectors, a provider-capability source scan of admission.py and
selection_policy.py, operations-catalog integrity, and — when
evidence-manifest.yml exists — manifest sha256 verification that fails
closed on any mismatch. No route loop, no probe, no provider call, no
gauntlet run. Run from the sea-rs repository root; exit 0 only if every
check passes.
"""
from __future__ import annotations

import copy
import hashlib
import os
import sys

import yaml

T21_DIR = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, T21_DIR)

import route_contracts as rc          # noqa: E402
import admission as adm               # noqa: E402
import selection_policy as sp         # noqa: E402

TS0 = "2026-09-18T00:00:00+00:00"
PREREG_REL = ".agents/preregistrations/godspeed-bounded-judgment-T21.prereg.yaml"
REPO_ROOT = os.path.normpath(os.path.join(T21_DIR, "..", "..", "..", ".."))

RESULTS = []


def check(name, ok, detail=""):
    RESULTS.append(bool(ok))
    line = ("PASS" if ok else "FAIL") + ": " + name
    if detail:
        line += " :: " + str(detail)
    print(line)


def sha256_file(path):
    with open(path, "rb") as fh:
        return hashlib.sha256(fh.read()).hexdigest()


def expect_refuse(name, rtype, data, needle):
    err = rc.validate(rtype, data)
    ok = err is not None and needle in err
    check(f"schema/refuses/{name}", ok, f"reason={err!r}")


def sample_records():
    moves = [
        {"scenario": "e2e-happy", "max_rounds": 8, "terminal_class": "settled"},
        {"scenario": "e2e-lying", "max_rounds": 8, "terminal_class": "other-nonsettlement"},
        {"scenario": "e2e-happy", "max_rounds": 3, "terminal_class": "budget_exhausted"},
    ]
    return {
        "candidate_move": {
            "candidate_id": "cand-1", "correlation_id": "corr-1",
            "operation": "run_case",
            "bounded_input": {"scenario": "e2e-happy", "max_rounds": 5},
            "expected_role": "boundary_probe", "prerequisite_refs": ["cand-0"],
            "generator": {"identity": "deterministic-fallback",
                          "version": "t21-deterministic-v1"},
            "rationale": "midpoint probe of the uncertain interval",
        },
        "admission_decision": {
            "candidate_id": "cand-1", "correlation_id": "corr-1",
            "classification": "admitted",
            "reason": "cataloged, unprobed, reachable, and affordable",
            "policy_version": adm.ADMISSION_POLICY_VERSION,
            "decided_at_utc": TS0,
        },
        "probe_observation": {
            "probe_id": "probe-1", "candidate_id": "cand-1",
            "correlation_id": "corr-1", "operation": "run_case",
            "bounded_input": {"scenario": "e2e-happy", "max_rounds": 5},
            "terminal_class": "settled", "settlement_committed_count": 1,
            "state_db_path": ("/home/sprime01/.local/share/godspeed-route-discovery/"
                              "T21/runs/probe-1/state/state.db"),
            "state_db_sha256": "a" * 64,
            "mechanism": rc.MECHANISM,
            "started_at_utc": TS0, "finished_at_utc": TS0,
        },
        "judgment_result": {
            "judgment_id": "judg-1", "candidate_id": "cand-1",
            "correlation_id": "corr-1", "label": "settled_accepted",
            "observation_refs": ["probe-1"],
            "judge_identity": "t21-bounded-judgment-v1 (observation records only)",
            "judged_at_utc": TS0,
        },
        "selection_record": {
            "correlation_id": "corr-sel-1", "iteration": 1,
            "policy_version": sp.POLICY_VERSION,
            "retained_candidate_id": "cand-1",
            "eligible_candidate_ids": ["cand-1"],
            "ineligible": [{"candidate_id": "cand-2",
                            "reason": "admission classification denied is ineligible"}],
            "decided_at_utc": TS0,
        },
        "route_state": {
            "route_id": "route-1", "created_at_utc": TS0, "probes_remaining": 10,
            "retained_moves": [{
                "candidate_id": "cand-1", "correlation_id": "corr-1",
                "scenario": "e2e-happy", "max_rounds": 8,
                "expected_role": "establish_baseline",
                "judgment_label": "settled_accepted"}],
            "probed": [{
                "candidate_id": "cand-1", "correlation_id": "corr-1",
                "scenario": "e2e-happy", "max_rounds": 8,
                "terminal_class": "settled",
                "judgment_label": "settled_accepted"}],
            "denied_keys": [{"scenario": "e2e-forged-report", "max_rounds": 2,
                             "reason": "authority attack recorded on this route"}],
        },
        "route_record": {
            "route_id": "route-1", "correlation_id": "corr-route-1",
            "settled": True, "stop_condition": "route_settled",
            "moves": moves,
            "fingerprint": rc.route_fingerprint(moves),
            "correlation_ids": ["corr-1", "corr-2", "corr-3"],
            "policy_version": sp.POLICY_VERSION,
            "prereg_sha256": rc.PREREG_SHA256,
            "opened_at_utc": TS0, "closed_at_utc": TS0,
        },
        "generated_case_record": {
            "case_id": "gen-case-1", "correlation_id": "corr-gen-1",
            "generator": {"identity": "t24-inverse-generator", "version": "t24-v1"},
            "source_route_id": "route-1",
            "source_route_fingerprint": rc.route_fingerprint(moves),
            "bounded_input": {"scenario": "e2e-happy", "max_rounds": 3},
            "replay": {
                "probe_id": "replay-1",
                "state_db_path": ("/home/sprime01/.local/share/godspeed-route-discovery/"
                                  "T21/runs/replay-1/state/state.db"),
                "state_db_sha256": "b" * 64,
                "terminal_class": "settled", "settled": True,
                "mechanism": rc.MECHANISM},
            "accepted": True, "invalid_reason": "", "created_at_utc": TS0,
        },
    }


def check_schemas():
    samples = sample_records()
    for rtype, sample in samples.items():
        err = rc.validate(rtype, sample)
        check(f"schema/valid/{rtype}", err is None, err)
        via_yaml = rc.from_yaml(rc.to_yaml(sample))
        check(f"schema/yaml-roundtrip/{rtype}",
              via_yaml == sample and rc.validate(rtype, via_yaml) is None)
        via_json = rc.from_json(rc.to_json(sample))
        check(f"schema/json-roundtrip/{rtype}",
              via_json == sample and rc.validate(rtype, via_json) is None)

    cm = samples["candidate_move"]
    jr = samples["judgment_result"]
    po = samples["probe_observation"]
    ad = samples["admission_decision"]
    sr = samples["selection_record"]
    rr = samples["route_record"]
    gc = samples["generated_case_record"]
    rs = samples["route_state"]

    def mutated(sample, **changes):
        out = copy.deepcopy(sample)
        out.update(changes)
        return out

    with_shell = copy.deepcopy(cm)
    with_shell["shell_plan"] = "rm -rf target && gauntlet run"
    expect_refuse("shell_plan-field", "candidate_move", with_shell, "forbidden")

    with_command = copy.deepcopy(cm)
    with_command["bounded_input"]["command"] = "sed -i s/x/y/"
    expect_refuse("nested-command-field", "candidate_move", with_command, "forbidden")

    expect_refuse("non-catalog-operation", "candidate_move",
                  mutated(cm, operation="run_shell"), "operation")
    expect_refuse("scenario-outside-domain", "candidate_move",
                  mutated(cm, bounded_input={"scenario": "e2e-contradictory",
                                             "max_rounds": 5}),
                  "scenario")
    expect_refuse("max_rounds-below-1", "candidate_move",
                  mutated(cm, bounded_input={"scenario": "e2e-happy", "max_rounds": 0}),
                  "max_rounds")
    expect_refuse("max_rounds-above-8", "candidate_move",
                  mutated(cm, bounded_input={"scenario": "e2e-happy", "max_rounds": 9}),
                  "max_rounds")
    no_corr = copy.deepcopy(cm)
    del no_corr["correlation_id"]
    expect_refuse("missing-correlation-identity", "candidate_move", no_corr,
                  "correlation")
    expect_refuse("label-outside-answer-domain", "judgment_result",
                  mutated(jr, label="probably_settled"), "label")
    expect_refuse("hand-authored-mechanism", "probe_observation",
                  mutated(po, mechanism="hand-authored"), "mechanism")
    expect_refuse("unknown-admission-classification", "admission_decision",
                  mutated(ad, classification="execute-anyway"), "classification")
    expect_refuse("retained-not-eligible", "selection_record",
                  mutated(sr, retained_candidate_id="cand-99"), "retained")
    expect_refuse("route-fingerprint-mismatch", "route_record",
                  mutated(rr, fingerprint="fingerprint-of-a-different-route"),
                  "fingerprint")
    forged = copy.deepcopy(gc)
    forged["replay"]["settled"] = False
    forged["replay"]["terminal_class"] = "budget_exhausted"
    expect_refuse("fabricated-acceptance", "generated_case_record", forged, "accepted")
    six_moves = copy.deepcopy(rs)
    six_moves["retained_moves"] = [dict(rs["retained_moves"][0],
                                        candidate_id=f"cand-{i}") for i in range(6)]
    expect_refuse("route-depth-over-bound", "route_state", six_moves,
                  "retained_moves")


def candidate(cid="cand-1", scenario="e2e-happy", rounds=5, role="boundary_probe",
              prereqs=(), **extra):
    rec = {
        "candidate_id": cid,
        "correlation_id": "corr-" + cid,
        "operation": "run_case",
        "bounded_input": {"scenario": scenario, "max_rounds": rounds},
        "expected_role": role,
        "prerequisite_refs": list(prereqs),
        "generator": {"identity": "deterministic-fallback",
                      "version": "t21-deterministic-v1"},
        "rationale": "gate vector",
    }
    rec.update(extra)
    return rec


def fresh_state(probes_remaining=12):
    return {
        "route_id": "route-gate",
        "created_at_utc": TS0,
        "probes_remaining": probes_remaining,
        "retained_moves": [],
        "probed": [],
        "denied_keys": [],
    }


def check_admission(catalog):
    def vector(name, expected, candidate_rec, state, payment, cat=None):
        decision = adm.classify(candidate_rec, state,
                                catalog if cat is None else cat, payment)
        ok = (decision["classification"] == expected
              and rc.validate("admission_decision", decision) is None)
        check(f"admission/vector/{name}", ok,
              f"classification={decision['classification']!r} "
              f"reason={decision['reason']!r}")

    vector("admitted", "admitted", candidate(), fresh_state(), 12)
    vector("invalid-scenario-outside-catalog", "invalid",
           candidate(scenario="e2e-contradictory"), fresh_state(), 12)
    vector("unaffordable-zero-budget", "unaffordable", candidate(),
           fresh_state(), 0)

    probed_state = fresh_state()
    probed_state["probed"] = [{
        "candidate_id": "c-prev", "correlation_id": "corr-c-prev",
        "scenario": "e2e-happy", "max_rounds": 5,
        "terminal_class": "settled", "judgment_label": "settled_accepted"}]
    vector("redundant-already-probed", "redundant", candidate(rounds=5),
           probed_state, 12)

    vector("denied-non-catalog-operation", "denied",
           candidate(operation="run_shell"), fresh_state(), 12)
    vector("denied-non-mock-runner", "denied",
           candidate(runner="prime-agent"), fresh_state(), 12)

    reduced = copy.deepcopy(catalog)
    del reduced["derivation_provenance"]["scenarios"]["e2e-lying"]
    vector("unreachable-unknown-scenario-reference", "unreachable",
           candidate("c-lying", scenario="e2e-lying",
                     role="establish_discriminator"),
           fresh_state(), 12, cat=reduced)


def observation_for(c, terminal):
    return {
        "probe_id": "probe-" + c["candidate_id"],
        "candidate_id": c["candidate_id"],
        "correlation_id": c["correlation_id"],
        "operation": "run_case",
        "bounded_input": dict(c["bounded_input"]),
        "terminal_class": terminal,
        "settlement_committed_count": 1 if terminal == "settled" else 0,
        "state_db_path": ("/home/sprime01/.local/share/godspeed-route-discovery/"
                          "T21/gate/" + c["candidate_id"] + "/state/state.db"),
        "state_db_sha256": "a" * 64,
        "mechanism": rc.MECHANISM,
        "started_at_utc": TS0, "finished_at_utc": TS0,
    }


def judgment_for(c, label):
    return {
        "judgment_id": "judg-" + c["candidate_id"],
        "candidate_id": c["candidate_id"],
        "correlation_id": c["correlation_id"],
        "label": label,
        "observation_refs": ["probe-" + c["candidate_id"]],
        "judge_identity": "t21-gate-vector",
        "judged_at_utc": TS0,
    }


def prior_probe(cid, scenario, rounds, terminal, label):
    return {
        "candidate_id": cid, "correlation_id": "corr-" + cid,
        "scenario": scenario, "max_rounds": rounds,
        "terminal_class": terminal, "judgment_label": label,
    }


def selection_case(name, candidates, terminals, labels, state, payment, catalog):
    admissions = [adm.classify(c, state, catalog, payment) for c in candidates]
    observations = [observation_for(c, terminals[c["candidate_id"]])
                    for c in candidates]
    judgments = [judgment_for(c, labels[c["candidate_id"]]) for c in candidates]
    return sp.select(candidates, admissions, observations, judgments, state,
                     payment, correlation_id="corr-sel-" + name, iteration=1)


def check_selection(catalog):
    # v1: a baseline-completing candidate beats boundary probes; exactly one
    # move is retained.
    v1 = [candidate("c-base", rounds=8, role="establish_baseline"),
          candidate("c-x1", rounds=5),
          candidate("c-x2", rounds=6)]
    rec = selection_case(
        "v1", v1,
        {"c-base": "settled", "c-x1": "budget_exhausted", "c-x2": "budget_exhausted"},
        {"c-base": "settled_accepted", "c-x1": "budget_exhausted",
         "c-x2": "budget_exhausted"},
        fresh_state(), 12, catalog)
    check("selection/v1-baseline-beats-boundary",
          rc.validate("selection_record", rec) is None
          and rec["retained_candidate_id"] == "c-base"
          and rec["eligible_candidate_ids"] == ["c-base", "c-x1", "c-x2"],
          str(rec["eligible_candidate_ids"]))
    check("selection/v1-exactly-one-retained",
          rec["retained_candidate_id"] != ""
          and rec["eligible_candidate_ids"].count(rec["retained_candidate_id"]) == 1)

    # v2: midpoint max_rounds wins among boundary probes; ties break to the
    # smaller rounds, then lexicographic candidate_id.
    state2 = fresh_state()
    state2["probed"] = [
        prior_probe("c-prev-lo", "e2e-happy", 2, "budget_exhausted", "budget_exhausted"),
        prior_probe("c-prev-hi", "e2e-happy", 8, "settled", "settled_accepted")]
    v2 = [candidate("c-b4", rounds=4), candidate("c-b5", rounds=5),
          candidate("c-b6", rounds=6)]
    rec = selection_case(
        "v2", v2,
        {c["candidate_id"]: "budget_exhausted" for c in v2},
        {c["candidate_id"]: "budget_exhausted" for c in v2},
        state2, 12, catalog)
    check("selection/v2-midpoint-wins",
          rec["retained_candidate_id"] == "c-b5"
          and rec["eligible_candidate_ids"] == ["c-b5", "c-b4", "c-b6"],
          str(rec["eligible_candidate_ids"]))

    v2b = [candidate("c-b4", rounds=4), candidate("c-b6", rounds=6)]
    rec = selection_case(
        "v2b", v2b,
        {"c-b4": "budget_exhausted", "c-b6": "budget_exhausted"},
        {"c-b4": "budget_exhausted", "c-b6": "budget_exhausted"},
        state2, 12, catalog)
    check("selection/v2b-tie-breaks-to-smaller-rounds",
          rec["retained_candidate_id"] == "c-b4",
          str(rec["eligible_candidate_ids"]))

    v2c = [candidate("c-b5b", rounds=5), candidate("c-b5a", rounds=5)]
    rec = selection_case(
        "v2c", v2c,
        {"c-b5b": "budget_exhausted", "c-b5a": "budget_exhausted"},
        {"c-b5b": "budget_exhausted", "c-b5a": "budget_exhausted"},
        state2, 12, catalog)
    check("selection/v2c-tie-breaks-to-lexicographic-id",
          rec["retained_candidate_id"] == "c-b5a",
          str(rec["eligible_candidate_ids"]))

    # v3: when r* is found (8 settles) but r*-1 (7) is unprobed, the r*-1
    # candidate has boundary priority over the midpoint (6) candidate.
    state3 = fresh_state()
    state3["probed"] = [
        prior_probe("c-p5", "e2e-happy", 5, "budget_exhausted", "budget_exhausted"),
        prior_probe("c-p8", "e2e-happy", 8, "settled", "settled_accepted")]
    v3 = [candidate("c-m6", rounds=6), candidate("c-r7", rounds=7)]
    rec = selection_case(
        "v3", v3,
        {"c-m6": "budget_exhausted", "c-r7": "settled"},
        {"c-m6": "budget_exhausted", "c-r7": "settled_accepted"},
        state3, 12, catalog)
    check("selection/v3-rstar-minus-one-priority",
          rec["retained_candidate_id"] == "c-r7",
          str(rec["eligible_candidate_ids"]))

    # v4: an ineligible-but-judgment-preferred candidate is NOT retained.
    state4 = fresh_state()
    state4["denied_keys"] = [{"scenario": "e2e-forged-report", "max_rounds": 4,
                              "reason": "gate vector: prior authority denial "
                                        "on this route"}]
    v4 = [candidate("c-good", rounds=5),
          candidate("c-bad", scenario="e2e-forged-report", rounds=4),
          candidate("c-prereq", rounds=6, prereqs=["c-missing"])]
    rec = selection_case(
        "v4", v4,
        {"c-good": "budget_exhausted", "c-bad": "settled", "c-prereq": "settled"},
        {"c-good": "budget_exhausted", "c-bad": "settled_accepted",
         "c-prereq": "settled_accepted"},
        state4, 12, catalog)
    ineligible_ids = {e["candidate_id"] for e in rec["ineligible"]}
    check("selection/v4-ineligible-preferred-not-retained",
          rec["retained_candidate_id"] == "c-good"
          and "c-bad" in ineligible_ids and "c-prereq" in ineligible_ids,
          f"retained={rec['retained_candidate_id']} ineligible={sorted(ineligible_ids)}")

    # v5: no remaining payment -> nothing is eligible and nothing is retained.
    v5 = [candidate("c-p1", rounds=5), candidate("c-p2", rounds=6)]
    rec = selection_case(
        "v5", v5,
        {"c-p1": "settled", "c-p2": "settled"},
        {"c-p1": "settled_accepted", "c-p2": "settled_accepted"},
        fresh_state(), 0, catalog)
    check("selection/v5-no-payment-no-retention",
          rc.validate("selection_record", rec) is None
          and rec["retained_candidate_id"] == ""
          and rec["eligible_candidate_ids"] == [],
          str(rec["ineligible"]))


PROVIDER_TOKENS = ("subprocess", "os.system", "popen", "requests", "urllib",
                   "socket", "opencode", "http.client", "httpx",
                   "telnetlib", "ftplib")


def check_source_scan():
    for fname in ("admission.py", "selection_policy.py"):
        with open(os.path.join(T21_DIR, fname), encoding="utf-8") as fh:
            src = fh.read()
        hits = [tok for tok in PROVIDER_TOKENS if tok in src]
        check(f"source-scan/{fname}-no-provider-capability", not hits,
              f"forbidden tokens present: {hits}" if hits
              else "no provider/subprocess capability in source")


def check_catalog(catalog):
    ops = catalog.get("operations")
    check("catalog/twenty-four-operations",
          isinstance(ops, list) and len(ops) == 24,
          f"found {len(ops) if isinstance(ops, list) else 'non-list'}")
    keys = {(o.get("scenario"), o.get("max_rounds")) for o in ops}
    expected = {(s, r) for s in rc.SCENARIOS for r in range(1, 9)}
    check("catalog/full-3x8-coverage", keys == expected,
          f"missing={sorted(expected - keys)} extra={sorted(keys - expected)}")
    check("catalog/unique-op-ids", len({o.get("op_id") for o in ops}) == 24)
    check("catalog/operation-shape-and-provenance",
          all(o.get("operation") == "run_case"
              and isinstance(o.get("provenance"), dict)
              and o["provenance"].get("scenario_fixture")
              and o["provenance"].get("max_rounds_source") for o in ops))
    rule = catalog.get("neighborhood_rule")
    check("catalog/neighborhood-rule-present",
          isinstance(rule, str) and rule.strip() != "")

    provenance = catalog.get("derivation_provenance", {})
    scenario_prov = provenance.get("scenarios", {})
    fixture_paths = [scenario_prov.get(s, {}).get("fixture_path") for s in rc.SCENARIOS]
    check("catalog/scenario-provenance-pinned", all(fixture_paths), str(fixture_paths))

    def exists_under_root(rel):
        return bool(rel) and os.path.exists(os.path.join(REPO_ROOT, rel))

    check("catalog/fixture-files-exist",
          all(exists_under_root(p) for p in fixture_paths))
    lever = provenance.get("max_rounds", {}).get("lever_source")
    check("catalog/budget-lever-source-exists", exists_under_root(lever), str(lever))
    mechanism = catalog.get("mechanism_provenance", {}).get("case_execution_mechanism")
    check("catalog/mechanism-reference-exists", exists_under_root(mechanism),
          str(mechanism))


def check_prereg_and_manifest():
    actual = sha256_file(os.path.join(REPO_ROOT, PREREG_REL))
    check("prereg/frozen-sha256-matches-disk", actual == rc.PREREG_SHA256, actual)

    manifest_path = os.path.join(T21_DIR, "evidence-manifest.yml")
    if not os.path.exists(manifest_path):
        check("manifest/hash-check", True,
              "skipped: evidence-manifest.yml not yet written (pre-freeze run)")
        return
    with open(manifest_path, encoding="utf-8") as fh:
        manifest = yaml.safe_load(fh)
    entries = manifest.get("files", [])
    mismatches = []
    for entry in entries:
        path = os.path.join(T21_DIR, entry["path"])
        digest = sha256_file(path)
        if digest != entry.get("sha256"):
            mismatches.append(f"{entry['path']}: manifest={entry.get('sha256')} "
                              f"disk={digest}")
    check("manifest/hash-check", bool(entries) and not mismatches,
          "; ".join(mismatches) if mismatches
          else f"{len(entries)} files verified against on-disk sha256")
    check("manifest/prereg-crossref",
          manifest.get("prereg", {}).get("sha256") == rc.PREREG_SHA256)


def main():
    with open(os.path.join(T21_DIR, "operations_catalog.yaml"), encoding="utf-8") as fh:
        catalog = yaml.safe_load(fh)
    check_schemas()
    check_admission(catalog)
    check_selection(catalog)
    check_source_scan()
    check_catalog(catalog)
    check_prereg_and_manifest()
    failed = RESULTS.count(False)
    print(f"verify_contract: {'PASS' if failed == 0 else 'FAIL'} "
          f"({len(RESULTS) - failed}/{len(RESULTS)} checks passed)")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
