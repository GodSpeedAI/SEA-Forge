#!/usr/bin/env python3
"""T23 GATE — verify_route.py (run from the sea-rs repo root; stdlib+PyYAML).

Independently verifies the T23 route-discovery evidence and exits 0 ONLY if
every check passes. Fail-closed throughout: a missing file, a hash mismatch,
a failed join, or an unprovable terminal is a gate FAILURE, never a warning.

Checks
  (a) every record schema-validates through the FROZEN T21 route_contracts
      (candidates, admissions, probe observations, judgments, selections,
      route states, route record);
  (b) bijective correlation joins across candidate/admission/probe/
      judgment/selection/route, candidates drawn only from the bounded
      neighborhood, and judgment observation_refs bound to the candidate's
      own probe;
  (c) INDEPENDENT recomputation of every route probe's terminal from its
      pinned state database (path + sha256 verified first, then the event
      log read read-only with inline sqlite code — the recorded
      terminal_class and settlement_committed_count must match the
      database; settled claims require >= 1 settlement_committed event;
      run_error claims require the absence of a typed terminal);
  (d) goal evaluation recomputed from the records under the CORRECTED
      addendum-1 goal (baseline e2e-happy@4 settled_accepted +
      discriminator e2e-lying verification_failed_nonsettlement + boundary
      r* settling with r*-1 budget_exhausted on THIS route's ledger); the
      route record may claim settled only when the recomputed goal holds;
  (e) route fingerprint recomputed and matching;
  (f) teeth T3/T4/T6 recorded with expected (MATCH) outcomes — T3 variants
      all provenance-refused, T4's genuine typed failure provable from its
      own database, T6 payment=0 retaining nothing / payment=1 selecting
      exactly one affordable alternative — and the route ledger untouched
      by any attack id;
  (g) fail-closed manifest hash checks: the frozen T21 modules + prereg +
      addendum-1, and T23's own evidence-manifest.yml in BOTH directions
      (every entry matches disk; every T23 .py/.yaml is pinned);
  (h) frozen bounds respected (K=3 candidates per iteration, retained depth
      <= 5, probes <= 12, iterations <= 6, unique per-probe id seeds, frozen
      probe command) and payment counters present and consistent.

This verifier is independent of the builder scripts: it imports only the
frozen T21 route_contracts and re-derives every consequence itself.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import hashlib
import json
import os
import re
import sqlite3

import yaml

T23_DIR = os.path.dirname(os.path.abspath(__file__))
BRANCH_DIR = os.path.dirname(T23_DIR)
T21_DIR = os.path.join(BRANCH_DIR, "T21")
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(BRANCH_DIR)))

sys.path.insert(0, T21_DIR)

import route_contracts as rc  # noqa: E402  (frozen T21 module, imported unchanged)

PREREG_SHA256 = "1fb55d9c56d87d94adc4850bc5ca713db3e600297fbdc997accfab54c98ea8b5"
ADDENDUM_SHA256 = "c472d1b63248ff5895433df16c545c4e145c8171e9d475a8f4b0732fa1c0cea8"

ROUTE_DIR = os.path.join(T23_DIR, "route")
TEETH_DIR = os.path.join(T23_DIR, "teeth")
K = 3
MAX_ITERATIONS = 6
MAX_PROBES = 12
MAX_RETAINED = 5
ZERO_SHA = hashlib.sha256(b"").hexdigest()
FROZEN_COMMAND = ("timeout 240 /home/sprime01/projects/gauntlet/target/debug/"
                  "gauntlet run Build and verify the calculator page")

ERRORS: list[str] = []
CHECKS = 0


def check(condition: bool, msg: str) -> bool:
    global CHECKS
    CHECKS += 1
    if not condition:
        ERRORS.append(msg)
    return bool(condition)


def load_json(path: str):
    with open(path) as f:
        return json.load(f)


def load_yaml(path: str):
    with open(path) as f:
        return yaml.safe_load(f)


def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


# --- independent database recomputation (no builder code) -------------------

def classify_terminal(raw_state) -> str:
    if raw_state == "settled":
        return "settled"
    if raw_state == "budget_exhausted":
        return "budget_exhausted"
    if isinstance(raw_state, str) and raw_state != "":
        return "other-nonsettlement"
    return "run_error"


def db_facts(db_path: str):
    """Read-only read of a state database's event log; returns
    (terminal_states, settlement_committed_count)."""
    conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    try:
        rows = conn.execute(
            "select payload from event_log where kind='run_terminal'"
        ).fetchall()
        count = conn.execute(
            "select count(*) from event_log where kind='settlement_committed'"
        ).fetchone()[0]
    finally:
        conn.close()
    states = []
    for (payload,) in rows:
        try:
            states.append(json.loads(payload).get("state"))
        except json.JSONDecodeError:
            states.append(None)
    return states, count


def recompute_observation(obs: dict) -> None:
    """Independently recompute one probe observation's consequence."""
    pid = obs["probe_id"]
    db_path = obs["state_db_path"]
    if obs["terminal_class"] == "run_error":
        if not os.path.isfile(db_path):
            check(obs["state_db_sha256"] == ZERO_SHA,
                  f"{pid}: run_error without a database must pin the "
                  "zero-byte digest")
            return
        states, count = db_facts(db_path)
        check(len(states) == 0,
              f"{pid}: run_error claim but the database carries a terminal")
        check(obs["settlement_committed_count"] == count,
              f"{pid}: settlement_committed_count != database ({count})")
        return
    if not os.path.isfile(db_path):
        check(False, f"{pid}: no state database at the pinned path {db_path}")
        return
    digest = sha256_file(db_path)
    check(digest == obs["state_db_sha256"],
          f"{pid}: state database digest mismatch (pinned "
          f"{obs['state_db_sha256'][:12]}.. != actual {digest[:12]}..)")
    states, count = db_facts(db_path)
    check(len(states) == 1,
          f"{pid}: expected exactly one run_terminal event, found "
          f"{len(states)}")
    terminal = classify_terminal(states[0] if states else None)
    check(terminal == obs["terminal_class"],
          f"{pid}: recorded terminal_class {obs['terminal_class']!r} != "
          f"database terminal {terminal!r}")
    check(count == obs["settlement_committed_count"],
          f"{pid}: recorded settlement_committed_count "
          f"{obs['settlement_committed_count']} != database {count}")
    if terminal == "settled":
        check(count >= 1,
              f"{pid}: settled terminal with no settlement_committed event")


# --- manifest verification ---------------------------------------------------

def verify_t21_manifest() -> None:
    manifest = load_yaml(os.path.join(T21_DIR, "evidence-manifest.yml"))
    for entry in manifest.get("files", []):
        path = os.path.join(T21_DIR, entry["path"])
        if not check(os.path.isfile(path),
                     f"T21 frozen file missing: {entry['path']}"):
            continue
        check(sha256_file(path) == entry["sha256"],
              f"T21 frozen file hash mismatch: {entry['path']}")
    for name, expected in (("godspeed-bounded-judgment-T21.prereg.yaml",
                            PREREG_SHA256),
                           ("godspeed-bounded-judgment-T21.prereg.addendum-1.yaml",
                            ADDENDUM_SHA256)):
        path = os.path.join(REPO_ROOT, ".agents/preregistrations", name)
        check(os.path.isfile(path), f"frozen input missing: {name}")
        if os.path.isfile(path):
            check(sha256_file(path) == expected,
                  f"frozen input hash mismatch: {name}")


def verify_t23_manifest() -> None:
    manifest_path = os.path.join(T23_DIR, "evidence-manifest.yml")
    if not check(os.path.isfile(manifest_path),
                 "T23 evidence-manifest.yml missing"):
        return
    manifest = load_yaml(manifest_path)
    entries = manifest.get("files", [])
    check(len(entries) > 0, "T23 evidence-manifest.yml pins no files")
    pinned = set()
    for entry in entries:
        rel = entry["path"]
        path = os.path.join(T23_DIR, rel)
        pinned.add(os.path.normpath(rel))
        if not check(os.path.isfile(path),
                     f"T23 manifest file missing on disk: {rel}"):
            continue
        check(sha256_file(path) == entry["sha256"],
              f"T23 manifest hash mismatch: {rel}")
    on_disk = set()
    for dirpath, dirnames, filenames in os.walk(T23_DIR):
        dirnames[:] = [d for d in dirnames if d != "__pycache__"]
        for fn in filenames:
            if fn.endswith((".py", ".yaml")):
                full = os.path.join(dirpath, fn)
                on_disk.add(os.path.normpath(os.path.relpath(full, T23_DIR)))
    # the manifest cannot pin itself (T22 convention)
    on_disk.discard(os.path.normpath("evidence-manifest.yml"))
    unpinned = sorted(on_disk - pinned)
    check(not unpinned,
          f"T23 .py/.yaml files not pinned in the manifest: {unpinned}")


# --- route record verification -----------------------------------------------

def main() -> int:
    print("verify_route: starting (fail-closed)", flush=True)

    # (g) manifests first: frozen inputs + this task's pinned identities
    verify_t21_manifest()
    verify_t23_manifest()

    # ---- load the route record -------------------------------------------
    rr_path = os.path.join(ROUTE_DIR, "route-record.yaml")
    if not check(os.path.isfile(rr_path), "route-record.yaml missing"):
        print("verify_route: FAIL (no route record)"); return 1
    route_doc = load_yaml(rr_path)
    route_state = route_doc.get("route_state", {})
    route_record = route_doc.get("route_record", {})
    check(route_doc.get("schema") == "godspeed-bounded-judgment.t23-route-record",
          "route-record.yaml carries an unexpected schema")
    check(rc.validate("route_state", route_state) is None,
          f"route_state fails the frozen schema: "
          f"{rc.validate('route_state', route_state)}")
    check(rc.validate("route_record", route_record) is None,
          f"route_record fails the frozen schema: "
          f"{rc.validate('route_record', route_record)}")
    payment = load_yaml(os.path.join(ROUTE_DIR, "payment-counters.yaml"))
    check(route_doc.get("correction_round") == 1,
          "route record must carry correction_round 1")

    corr_ids = route_record.get("correlation_ids", [])
    iteration_dirs = sorted(
        d for d in os.listdir(ROUTE_DIR)
        if re.fullmatch(r"iteration-\d+", d))
    check(len(corr_ids) == len(iteration_dirs),
          f"correlation_ids ({len(corr_ids)}) != iteration dirs "
          f"({len(iteration_dirs)})")

    all_observations, all_judgments = [], []
    retained_moves = []
    seeds_seen = {}

    for idx, corr in enumerate(sorted(corr_ids), start=1):
        idir = os.path.join(ROUTE_DIR, f"iteration-{idx}")
        if not check(os.path.isdir(idir), f"missing iteration dir {idir}"):
            continue
        candidates = load_json(os.path.join(idir, "candidates.json"))
        admissions = load_json(os.path.join(idir, "admission-decisions.json"))
        observations = load_json(os.path.join(idir, "probe-observations.json"))
        surfaces = load_json(os.path.join(idir, "judgment-surfaces.json"))
        judgments = load_json(os.path.join(idir, "judgment-results.json"))
        selection = load_json(os.path.join(idir, "selection-record.json"))
        neighborhood = load_yaml(os.path.join(idir, "neighborhood.yaml"))
        generation = load_yaml(os.path.join(idir, "generation-record.yaml"))
        run_meta = load_yaml(os.path.join(idir, "probe-run-meta.yaml"))
        state_after = load_yaml(os.path.join(idir, "route-state-after.yaml"))
        judg_record = load_yaml(os.path.join(idir, "judgment-record.yaml"))

        # (a) schema validation through the frozen T21 contracts
        for c in candidates:
            check(rc.validate("candidate_move", c) is None,
                  f"{corr}: candidate {c.get('candidate_id')!r} fails the "
                  f"frozen schema: {rc.validate('candidate_move', c)}")
        for a in admissions:
            check(rc.validate("admission_decision", a) is None,
                  f"{corr}: admission_decision fails the frozen schema")
        for o in observations:
            check(rc.validate("probe_observation", o) is None,
                  f"{corr}: probe_observation {o.get('probe_id')!r} fails the "
                  f"frozen schema: {rc.validate('probe_observation', o)}")
        for s in surfaces:
            check(set(s) <= {"probe_id", "candidate_id", "correlation_id",
                             "operation", "bounded_input", "terminal_class",
                             "settlement_committed_count", "state_db_sha256",
                             "mechanism", "started_at_utc", "finished_at_utc"},
                  f"{corr}: judgment surface carries unexpected fields")
        for j in judgments:
            check(rc.validate("judgment_result", j) is None,
                  f"{corr}: judgment_result fails the frozen schema")
        check(rc.validate("selection_record", selection) is None,
              f"{corr}: selection_record fails the frozen schema")
        check(rc.validate("route_state", state_after["route_state"]) is None,
              f"{corr}: route-state-after fails the frozen route_state schema")

        # (b) bijective correlation joins
        cids = [c["candidate_id"] for c in candidates]
        check(len(set(cids)) == len(cids),
              f"{corr}: duplicate candidate_id in candidates")
        check(len(candidates) == K,
              f"{corr}: expected exactly K={K} candidates, got "
              f"{len(candidates)}")
        check({c["candidate_id"] for c in candidates} ==
              {a["candidate_id"] for a in admissions},
              f"{corr}: candidates and admissions are not bijective")
        check({c["candidate_id"] for c in candidates} ==
              {o["candidate_id"] for o in observations},
              f"{corr}: candidates and probe observations are not bijective "
              "(all admitted candidates must be probed)")
        check({c["candidate_id"] for c in candidates} ==
              {j["candidate_id"] for j in judgments},
              f"{corr}: candidates and judgments are not bijective")
        for group in (candidates, admissions, observations, judgments):
            for rec in group:
                check(rec["correlation_id"] == corr,
                      f"{corr}: record {rec.get('candidate_id')} carries "
                      f"foreign correlation_id {rec['correlation_id']!r}")
        check(selection["correlation_id"] == corr,
              f"{corr}: selection_record carries a foreign correlation_id")
        obs_by_probe = {o["probe_id"]: o for o in observations}
        check(len(obs_by_probe) == len(observations),
              f"{corr}: duplicate probe_id in observations")
        for j in judgments:
            refs = j["observation_refs"]
            check(len(refs) == 1 and refs[0] in obs_by_probe,
                  f"{corr}: judgment {j['judgment_id']} does not reference "
                  "exactly one probe of this iteration")
            if refs and refs[0] in obs_by_probe:
                check(obs_by_probe[refs[0]]["candidate_id"] == j["candidate_id"],
                      f"{corr}: judgment {j['judgment_id']} references another "
                      "candidate's probe (correlation swap)")
        # the judgment surfaces mirror the observations they judge
        surface_by_probe = {s["probe_id"]: s for s in surfaces}
        for o in observations:
            s = surface_by_probe.get(o["probe_id"])
            check(s is not None, f"{corr}: no judgment surface for {o['probe_id']}")
            if s is not None:
                check(
                    s["terminal_class"] == o["terminal_class"]
                    and s["settlement_committed_count"] ==
                        o["settlement_committed_count"]
                    and s["state_db_sha256"] == o["state_db_sha256"]
                    and s["bounded_input"] == o["bounded_input"],
                    f"{corr}: judgment surface {o['probe_id']} does not "
                    "mirror its observation record")
        # candidates drawn only from the bounded neighborhood
        neighborhood_keys = {
            (op["scenario"], op["max_rounds"]) for op in neighborhood["operations"]}
        check(neighborhood["correlation_id"] == corr,
              f"{corr}: neighborhood carries a foreign correlation_id")
        check(neighborhood["neighborhood_size"] == len(neighborhood["operations"]),
              f"{corr}: neighborhood_size mismatch")
        for c in candidates:
            key = (c["bounded_input"]["scenario"], c["bounded_input"]["max_rounds"])
            check(key in neighborhood_keys,
                  f"{corr}: candidate {c['candidate_id']} proposes "
                  f"{key}, which is outside the bounded neighborhood")
        # the retained candidate is eligible and the recorded joins hold
        for rec in selection["ineligible"]:
            check(rec["candidate_id"] in cids,
                  f"{corr}: selection ineligible names an unknown candidate")
        eligible = selection["eligible_candidate_ids"]
        check(set(eligible) <= set(cids),
              f"{corr}: selection eligibility names unknown candidates")
        check(sorted(eligible + [i["candidate_id"]
                                 for i in selection["ineligible"]]) ==
              sorted(cids),
              f"{corr}: eligible+ineligible != candidates (not bijective)")
        if selection["retained_candidate_id"] != "":
            check(selection["retained_candidate_id"] in eligible,
                  f"{corr}: retained candidate is not eligible")
        # generation raw outputs preserved verbatim
        for attempt in generation.get("attempts", []):
            for key in ("raw_output_path", "prompt_path"):
                p = attempt.get(key)
                if p:
                    check(os.path.isfile(p),
                          f"{corr}: generation attempt {key} missing: {p}")
        # judgment record paths preserved
        for pj in judg_record.get("probe_judgments", []):
            for attempt in pj.get("attempts", []):
                p = attempt.get("raw_output_path")
                if p:
                    check(os.path.isfile(p),
                          f"{corr}: judgment raw output missing: {p}")

        # (h) probe hygiene: unique seeds, frozen command, durable cwd
        metas = {m["probe_id"]: m for m in run_meta["probes"]}
        check(len(metas) == len(observations),
              f"{corr}: probe-run-meta does not cover every probe")
        for o in observations:
            m = metas.get(o["probe_id"])
            check(m is not None,
                  f"{corr}: no run-meta sidecar entry for {o['probe_id']}")
            if m is None:
                continue
            seed = m.get("gauntlet_id_seed")
            check(isinstance(seed, str) and seed != "",
                  f"{o['probe_id']}: missing GAUNTLET_ID_SEED in the sidecar")
            if seed:
                check(seed not in seeds_seen,
                      f"{o['probe_id']}: GAUNTLET_ID_SEED reused from "
                      f"{seeds_seen.get(seed)}")
                seeds_seen[seed] = o["probe_id"]
            check(m.get("command") == FROZEN_COMMAND,
                  f"{o['probe_id']}: probe command is not the frozen command")
            check(isinstance(m.get("exit_code"), int)
                  and isinstance(m.get("wall_seconds"), (int, float)),
                  f"{o['probe_id']}: run-meta lacks exit code / wall seconds")
            expected_cwd = os.path.join(
                route_doc.get("probe_data_root", ""), "targets",
                f"i{idx}-{o['candidate_id']}")
            check(m.get("cwd") == expected_cwd,
                  f"{o['probe_id']}: probe cwd {m.get('cwd')!r} is not the "
                  "per-candidate durable target dir")

        # (c) independent terminal recomputation from the pinned databases
        for o in observations:
            recompute_observation(o)
        all_observations.extend(observations)
        all_judgments.extend(judgments)

        # route-state-after mirrors this iteration's absorption
        after = state_after["route_state"]
        check(len(after["probed"]) == idx * K,
              f"{corr}: route-state-after probed count "
              f"{len(after['probed'])} != {idx * K}")
        check(after["probes_remaining"] == MAX_PROBES - idx * K,
              f"{corr}: probes_remaining arithmetic broken")
        retained = selection["retained_candidate_id"]
        if retained != "":
            move = next(m for m in after["retained_moves"]
                        if m["correlation_id"] == corr)
            check(move["candidate_id"] == retained,
                  f"{corr}: route-state-after retained move does not match "
                  "the selection record")
            cand = next(c for c in candidates if c["candidate_id"] == retained)
            check((move["scenario"], move["max_rounds"]) ==
                  (cand["bounded_input"]["scenario"],
                   cand["bounded_input"]["max_rounds"]),
                  f"{corr}: retained move input does not match its candidate")
            retained_moves.append(move)
        check(len(state_after["goal_evaluation"]) >= 5,
              f"{corr}: route-state-after lacks the goal evaluation")

    # ---- (h) frozen bounds ------------------------------------------------
    check(len(iteration_dirs) <= MAX_ITERATIONS,
          f"iterations {len(iteration_dirs)} exceed the frozen bound "
          f"{MAX_ITERATIONS}")
    check(len(all_observations) <= MAX_PROBES,
          f"probes {len(all_observations)} exceed the frozen bound "
          f"{MAX_PROBES}")
    check(len(retained_moves) <= MAX_RETAINED,
          f"retained depth {len(retained_moves)} exceeds the frozen bound "
          f"{MAX_RETAINED}")
    check(len(route_state["retained_moves"]) == len(retained_moves),
          "final route_state retained depth differs from the iteration joins")
    check(len(route_state["probed"]) == len(all_observations),
          "final route_state probed count != total observations")
    check(route_state["probes_remaining"] ==
          MAX_PROBES - len(all_observations),
          "final probes_remaining arithmetic broken")

    # (b) route-wide joins: probed entries mirror observations bijectively
    obs_key = {(o["correlation_id"], o["candidate_id"]): o
               for o in all_observations}
    check(len(obs_key) == len(all_observations),
          "duplicate (correlation, candidate) probe keys")
    probed_key = {(p["correlation_id"], p["candidate_id"]): p
                  for p in route_state["probed"]}
    check(set(obs_key) == set(probed_key),
          "route_state.probed is not bijective with the probe observations")
    for key, o in obs_key.items():
        p = probed_key.get(key)
        if p is None:
            continue
        check((p["scenario"], p["max_rounds"], p["terminal_class"]) ==
              (o["bounded_input"]["scenario"], o["bounded_input"]["max_rounds"],
               o["terminal_class"]),
              f"probed entry {key} does not mirror its observation")
        j = next(j for j in all_judgments
                 if (j["correlation_id"], j["candidate_id"]) == key)
        check(p["judgment_label"] == j["label"],
              f"probed entry {key} judgment_label != judgment result")

    # retained moves mirror the selections across iterations, in order
    check(len(route_state["retained_moves"]) == len(retained_moves),
          "retained_moves mismatch between final state and iteration joins")
    for m_final, m_iter in zip(route_state["retained_moves"], retained_moves):
        check(m_final == m_iter,
              f"retained move order/content mismatch at "
              f"{m_final.get('correlation_id')}")

    # route_record.moves mirror the retained moves (with terminals)
    check(len(route_record["moves"]) == len(route_state["retained_moves"]),
          "route_record.moves length != retained moves")
    for move, m in zip(route_record["moves"], route_state["retained_moves"]):
        check((move["scenario"], move["max_rounds"]) ==
              (m["scenario"], m["max_rounds"]),
              "route_record.move input does not match its retained move")
        o = obs_key[(m["correlation_id"], m["candidate_id"])]
        check(move["terminal_class"] == o["terminal_class"],
              "route_record.move terminal_class != the probe's recorded "
              "terminal")

    # ---- (d) goal evaluation recomputed from the records ------------------
    happy_settled = {p["max_rounds"] for p in route_state["probed"]
                     if p["scenario"] == "e2e-happy"
                     and p["judgment_label"] == "settled_accepted"}
    happy_exhausted = {p["max_rounds"] for p in route_state["probed"]
                       if p["scenario"] == "e2e-happy"
                       and p["judgment_label"] == "budget_exhausted"}
    baseline_met = 4 in happy_settled
    discriminator_met = any(p["scenario"] == "e2e-lying"
                            and p["judgment_label"] ==
                            "verification_failed_nonsettlement"
                            for p in route_state["probed"])
    r_star = min(happy_settled) if happy_settled else None
    boundary_met = bool(happy_settled) and ((min(happy_settled) - 1)
                                            in happy_exhausted)
    goal_met = baseline_met and discriminator_met and boundary_met
    recorded_goal = route_doc.get("goal_evaluation", {})
    check(recorded_goal.get("baseline_met") == baseline_met,
          "recorded baseline_met != recomputed")
    check(recorded_goal.get("discriminator_met") == discriminator_met,
          "recorded discriminator_met != recomputed")
    check(recorded_goal.get("r_star") == r_star,
          "recorded r_star != recomputed")
    check(recorded_goal.get("boundary_met") == boundary_met,
          "recorded boundary_met != recomputed")
    check(recorded_goal.get("goal_met") == goal_met,
          "recorded goal_met != recomputed")
    # a settle claim is ONLY allowed when the recomputed goal holds
    check(route_record["settled"] == goal_met,
          "route_record.settled does not match the recomputed goal: a settle "
          "claim requires baseline + discriminator + boundary on this "
          "route's own ledger")
    if not goal_met:
        check(route_record["stop_condition"] != "route_settled",
              "unsettled goal must not record route_settled")
    goal_text = route_doc.get("goal_definition", {}).get("goal", "")
    check("e2e-happy@4" in goal_text
          and "verification_failed_nonsettlement" in goal_text
          and "r*-1" in goal_text,
          "goal_definition does not state the CORRECTED addendum-1 goal")

    # ---- (e) fingerprint recomputed ----------------------------------------
    recomputed_fp = rc.route_fingerprint(route_record["moves"])
    check(recomputed_fp == route_record["fingerprint"],
          f"fingerprint mismatch: recorded "
          f"{route_record['fingerprint']!r} != recomputed {recomputed_fp!r}")

    # ---- (f) teeth ---------------------------------------------------------
    attacks_path = os.path.join(TEETH_DIR, "attacks.yaml")
    if check(os.path.isfile(attacks_path), "teeth/attacks.yaml missing"):
        attacks = load_yaml(attacks_path)
        check(attacks.get("task") == "T23",
              "teeth/attacks.yaml is not marked for T23")
        by_id = {t.get("id"): t for t in attacks.get("teeth", [])}
        for tid in ("T3-fabricated-success", "T4-execution-failure",
                    "T6-budget-exhaustion"):
            check(tid in by_id, f"teeth record missing: {tid}")
            t = by_id.get(tid, {})
            check(str(t.get("expected_vs_observed", "")).startswith("MATCH"),
                  f"tooth {tid} did not record a MATCH outcome")
            check("ATTACK" in str(t.get("marked", "")),
                  f"tooth {tid} is not marked ATTACK")
        imm = attacks.get("route_immutability", {})
        check(imm.get("route_ledger_unchanged") is True
              and imm.get("attack_ids_absent_from_route") is True,
              "teeth did not prove the route ledger untouched")
        check(imm.get("route_fingerprint_before")
              == imm.get("route_fingerprint_after"),
              "route fingerprint changed during the teeth run")
        t3 = load_json(os.path.join(TEETH_DIR, "t3-attack-records.json"))
        check(all(v.get("provenance_refused") is True
                  for v in t3.get("variants", {}).values()),
              "T3: a fabricated-success variant was NOT refused by the "
              "provenance validator")
        t4 = load_json(os.path.join(TEETH_DIR, "t4-attack-records.json"))
        t4_obs = t4.get("attack_observation", {})
        if check(bool(t4_obs), "T4: no attack observation recorded"):
            recompute_observation(t4_obs)  # the genuine failure must be real
            check(t4_obs.get("terminal_class") != "settled",
                  "T4: the execution-failure tooth cannot claim a settled "
                  "terminal")
        t6 = load_json(os.path.join(TEETH_DIR, "t6-attack-records.json"))
        sel_a = t6.get("variant_a_selection_record", {})
        sel_b = t6.get("variant_b_selection_record", {})
        check(sel_a.get("retained_candidate_id") == ""
              and len(sel_a.get("ineligible", [])) > 0
              and all("payment" in i.get("reason", "")
                      for i in sel_a.get("ineligible", [])),
              "T6 variant A: the policy must retain nothing under payment 0 "
              "with payment reasons only")
        check(sel_b.get("retained_candidate_id") != ""
              and sel_b.get("retained_candidate_id")
              in sel_b.get("eligible_candidate_ids", []),
              "T6 variant B: the policy must select one affordable "
              "alternative under payment 1")

    # route hygiene: no attack ids in the route ledger
    attack_markers = (b"t3-attack", b"t4-attack", b"t6-attack", b"t23-t3-",
                      b"t23-t4-", b"t23-t6-")
    for dirpath, dirnames, filenames in os.walk(ROUTE_DIR):
        for fn in filenames:
            with open(os.path.join(dirpath, fn), "rb") as f:
                blob = f.read()
            for marker in attack_markers:
                check(marker not in blob,
                      f"attack marker {marker!r} found in route ledger file "
                      f"{os.path.join(dirpath, fn)}")

    # ---- (h) payment counters present and consistent -----------------------
    check(payment.get("task") == "T23", "payment counters are not T23's")
    check(payment.get("probes_run") == len(all_observations),
          "payment counters probes_run != recorded observations")
    check(payment.get("tool_executions") == payment.get("probes_run"),
          "payment counters tool_executions != probes_run")
    check(payment.get("failed_executions") ==
          sum(1 for o in all_observations
              if o["terminal_class"] == "run_error"),
          "payment counters failed_executions mismatch")
    check(isinstance(payment.get("wall_clock_seconds"), (int, float)),
          "payment counters lack wall_clock_seconds")
    provider_calls = payment.get("provider_calls", {})
    check(isinstance(provider_calls, dict)
          and isinstance(provider_calls.get("count"), int)
          and len(provider_calls.get("by_identity", [])) ==
          provider_calls.get("count"),
          "payment counters lack a consistent provider-call ledger")
    check(isinstance(payment.get("tokens_and_cost"), str)
          and payment.get("tokens_and_cost") != "",
          "payment counters must record the tokens/cost state (honest gap "
          "allowed, silence not)")

    # ---- verdict ------------------------------------------------------------
    if ERRORS:
        print(f"verify_route: FAIL ({len(ERRORS)} problem(s) after "
              f"{CHECKS} checks)")
        for e in ERRORS:
            print(f"  - {e}")
        return 1
    print(f"verify_route: PASS ({CHECKS} checks)")
    print(f"  route {route_record['route_id']}: settled="
          f"{route_record['settled']} stop={route_record['stop_condition']}")
    print(f"  goal: baseline_met={baseline_met} "
          f"discriminator_met={discriminator_met} r*={r_star} "
          f"boundary_met={boundary_met} -> goal_met={goal_met}")
    print(f"  probes={len(all_observations)} iterations={len(iteration_dirs)} "
          f"retained={len(route_state['retained_moves'])} "
          f"fingerprint={route_record['fingerprint']!r}")
    print(f"  teeth T3/T4/T6 recorded with MATCH outcomes; route ledger "
          f"provenance intact")
    return 0


if __name__ == "__main__":
    sys.exit(main())
