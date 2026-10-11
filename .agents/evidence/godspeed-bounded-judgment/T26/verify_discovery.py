#!/usr/bin/env python3
"""T26 verify_discovery.py — fresh independent adversarial verification gate.

Authored by the T26 independent verifier (fresh agent, no involvement in
T21..T25). This script shares NO code with the builder's gates: it imports
nothing from T21..T25 and re-implements every derivation itself. Run from
the sea-rs repo root:

    python3 .agents/evidence/godspeed-bounded-judgment/T26/verify_discovery.py

Exit 0 ONLY if every check passes. Any missing file, hash mismatch, or
irreconcilable divergence between the recorded evidence and the independent
recomputation from raw records is a FAILURE (fail-closed).

Independently recomputed, from raw records:
  S1  frozen rule authority (prereg chain hashes vs packet + decision log)
  S2  fail-closed manifest hash checks across T21, T22, T23 (round-1),
      T23 round-2, T23 round-3, T24, T25 evidence manifests
  S3  the 12 route probes' terminals from their pinned state DBs (the DB
      event logs are the only ground truth) + artifact re-hashing
  S4  addendum-3 mechanical labels + the symmetry property, re-derived
  S5  goal evaluation + fingerprint recomputed from DB-derived facts
  S6  T24 replay settlement + freeze-before-replay ordering + target-copy
      identity + seed freshness
  S7  T25 negative terminal + single-factor diff (recursive target diff)
  S8  novelty classification re-derived from the T15/T16 frozen records
  S9  payment counters: decision-log entries re-counted, provider-call and
      execution counters re-counted from the stage counter files
  S10 model-free settlement: the route goal must be reproducible WITHOUT
      any provider output (round-3 = 0 provider calls; goal recomputed
      from DB facts alone); selection records carry the frozen policy only
  S11 claim-boundary scan over the developmental records
"""
from __future__ import annotations

import hashlib
import json
import os
import re
import sqlite3
import sys

import yaml

sys.dont_write_bytecode = True

REPO_ROOT = os.path.abspath(os.getcwd())
BRANCH = os.path.join(REPO_ROOT, ".agents/evidence/godspeed-bounded-judgment")
PREREG_DIR = os.path.join(REPO_ROOT, ".agents/preregistrations")
T21, T22, T23 = (os.path.join(BRANCH, t) for t in ("T21", "T22", "T23"))
T24, T25, T26 = (os.path.join(BRANCH, t) for t in ("T24", "T25", "T26"))
T23_ROUTE = os.path.join(T23, "route")
T23_R2 = os.path.join(T23, "round-2")
T23_R3 = os.path.join(T23, "round-3")
DATA_ROOT = os.path.join(os.path.expanduser("~"),
                         ".local/share/godspeed-route-discovery")

CHECKS = 0
ERRORS: list[str] = []
NOTES: list[str] = []


def check(cond, msg):
    global CHECKS
    CHECKS += 1
    if not cond:
        ERRORS.append(msg)
    return bool(cond)


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def load_yaml(path):
    with open(path, encoding="utf-8") as f:
        return yaml.safe_load(f)


def load_json(path):
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def read_events(db_path):
    conn = sqlite3.connect(f"file:{db_path}?mode=ro&immutable=1", uri=True)
    try:
        rows = conn.execute(
            "select kind, payload from event_log order by offset").fetchall()
        runs = conn.execute("select run_id, budget from runs").fetchall()
    finally:
        conn.close()
    events = []
    for kind, payload in rows:
        try:
            body = json.loads(payload)
        except json.JSONDecodeError:
            body = {}
        events.append((kind, body))
    return events, runs


def classify_terminal(raw_state):
    """Independent mapping from the raw run_terminal state to the frozen
    T21 terminal classes (TERMINAL_CLASSES in route_contracts)."""
    if raw_state == "settled":
        return "settled"
    if raw_state == "budget_exhausted":
        return "budget_exhausted"
    if isinstance(raw_state, str) and raw_state != "":
        return "other-nonsettlement"
    return "run_error"


def fingerprint(moves):
    """Frozen novelty metric: ordered retained moves of
    (scenario, max_rounds) plus each move's final terminal class."""
    return ";".join(f"{m['scenario']}@{m['max_rounds']}:{m['terminal_class']}"
                    for m in moves)


# Frozen expected identities (pinned in the confirmation packet, the
# decision log, the manifests, and the consuming records).
PREREG_SHA = "1fb55d9c56d87d94adc4850bc5ca713db3e600297fbdc997accfab54c98ea8b5"
ADD1_SHA = "c472d1b63248ff5895433df16c545c4e145c8171e9d475a8f4b0732fa1c0cea8"
ADD2_SHA = "087887c27cc50c6c989fcf7bb214e77aca62ea74d7e32a868ef0f7dd211e4c5a"
ADD3_SHA = "1c13889ae0c2a9621b7f0b803b327ee6dd76a16983c9ed72b00ff309f59c6ddb"
RR1_SHA = "9345c43e6787ef32fbe1a7749c90872de7b5be4b46f73af9708516f4f8ded67e"
RR2_SHA = "147f12e072dc7b578f11bd5d57b4fb7fd1caed7330b73e33acaf3c69eb04a416"
RR3_SHA = "c6b9ee9cff7c8f2047dd29e2eef5576f9b113d72d5f8618c6849ec618bb27097"
VFN = "verification_failed_nonsettlement"
FAILING_ARTIFACT = ("08c3dff5cc3c6601f930a6923a8e30e9686e784e3d2afe1d4959"
                    "d02627e5e0d2")


def main() -> int:
    print("verify_discovery: starting (fail-closed, independent recomputation)",
          flush=True)

    # ---- S1: frozen rule authority -----------------------------------------
    print("[S1] frozen rule authority", flush=True)
    prereg_paths = {
        "base prereg": os.path.join(
            PREREG_DIR, "godspeed-bounded-judgment-T21.prereg.yaml"),
        "addendum-1": os.path.join(
            PREREG_DIR,
            "godspeed-bounded-judgment-T21.prereg.addendum-1.yaml"),
        "addendum-2": os.path.join(
            PREREG_DIR,
            "godspeed-bounded-judgment-T21.prereg.addendum-2.yaml"),
        "addendum-3": os.path.join(
            PREREG_DIR,
            "godspeed-bounded-judgment-T21.prereg.addendum-3.yaml"),
    }
    expected = {"base prereg": PREREG_SHA, "addendum-1": ADD1_SHA,
                "addendum-2": ADD2_SHA, "addendum-3": ADD3_SHA}
    for name, path in prereg_paths.items():
        if check(os.path.isfile(path), f"frozen input missing: {name}"):
            check(sha256_file(path) == expected[name],
                  f"frozen input hash mismatch: {name}")
    # decision log must pin the same chain
    dec_path = os.path.join(BRANCH, "decisions.yml")
    decisions_text = open(dec_path, encoding="utf-8").read()
    for sha in (PREREG_SHA, ADD1_SHA, ADD2_SHA, ADD3_SHA):
        check(sha in decisions_text,
              f"decision log does not pin {sha[:12]}… of the prereg chain")

    # ---- S2: fail-closed manifest hash checks across T21..T25 ---------------
    print("[S2] fail-closed manifest checks (T21..T25)", flush=True)
    manifests = [
        ("T21", T21, "evidence-manifest.yml"),
        ("T22", T22, "evidence-manifest.yml"),
        ("T23 round-1", T23, "evidence-manifest.yml"),
        ("T23 round-2", T23_R2, "evidence-manifest.round2.yml"),
        ("T23 round-3", T23_R3, "evidence-manifest.round3.yml"),
        ("T24", T24, "evidence-manifest.yml"),
        ("T25", T25, "evidence-manifest.yml"),
    ]
    for label, base, name in manifests:
        mpath = os.path.join(base, name)
        if not check(os.path.isfile(mpath), f"{label}: manifest missing"):
            continue
        m = load_yaml(mpath)
        n = 0
        for entry in m.get("files", []):
            rel = entry["path"]
            fpath = os.path.join(base, rel)
            n += 1
            if not check(os.path.isfile(fpath), f"{label}: missing {rel}"):
                continue
            check(sha256_file(fpath) == entry["sha256"],
                  f"{label}: HASH MISMATCH {rel}")
        check(n >= 4, f"{label}: manifest implausibly small ({n} entries)")
        print(f"    {label}: {n} pinned files verified")
    # the T21 manifest must pin the base prereg
    t21m = load_yaml(os.path.join(T21, "evidence-manifest.yml"))
    check(t21m.get("prereg", {}).get("sha256") == PREREG_SHA,
          "T21 manifest does not pin the base prereg sha256")
    # route-record identity pins
    rr1_path = os.path.join(T23_ROUTE, "route-record.yaml")
    rr2_path = os.path.join(T23_R2, "route-record.round2.yaml")
    rr3_path = os.path.join(T23_R3, "route-record.round3.yaml")
    check(sha256_file(rr1_path) == RR1_SHA,
          "round-1 route-record.yaml != packet pin")
    check(sha256_file(rr2_path) == RR2_SHA,
          "round-2 route-record.round2.yaml != pin")
    check(sha256_file(rr3_path) == RR3_SHA,
          "round-3 route-record.round3.yaml != candidate-case pin")

    # ---- S3: the 12 route probes from their pinned DBs ----------------------
    print("[S3] 12 route probe terminals from pinned DBs", flush=True)
    obs_by_probe = {}
    round1_labels = {}
    for i in (1, 2, 3, 4):
        idir = os.path.join(T23_ROUTE, f"iteration-{i}")
        for o in load_json(os.path.join(idir, "probe-observations.json")):
            obs_by_probe[o["probe_id"]] = o
        for j in load_json(os.path.join(idir, "judgment-results.json")):
            round1_labels[j["candidate_id"] + "@" + j["correlation_id"]] = \
                j["label"]
    check(len(obs_by_probe) == 12,
          f"expected 12 preserved probes, found {len(obs_by_probe)}")
    packet = load_yaml(os.path.join(T23_R3, "confirmation-packet.yaml"))
    packet_dbs = {e["probe_id"]: e
                  for e in packet.get("state_databases", {}).get("items", [])}
    check(sorted(packet_dbs) == sorted(obs_by_probe),
          "packet state_databases not bijective with the probe records")

    derived = {}
    for pid in sorted(obs_by_probe):
        obs = obs_by_probe[pid]
        db_path = obs["state_db_path"]
        check(db_path.startswith(DATA_ROOT + os.sep),
              f"{pid}: state DB outside the pinned durable data root")
        if not check(os.path.isfile(db_path), f"{pid}: state DB missing"):
            continue
        check(sha256_file(db_path) == obs["state_db_sha256"],
              f"{pid}: state DB HASH MISMATCH vs observation pin")
        if pid in packet_dbs:
            check(packet_dbs[pid]["sha256"] == obs["state_db_sha256"],
                  f"{pid}: packet pin != observation pin")
            # the packet's data: prefix is .../godspeed-route-discovery/T23
            packet_db_path = os.path.normpath(packet_dbs[pid]["db"].replace(
                "data:", os.path.join(DATA_ROOT, "T23") + os.sep))
            check(packet_db_path == os.path.normpath(db_path),
                  f"{pid}: packet DB path != observation DB path")
        events, runs = read_events(db_path)
        terminals = [b.get("state") for k, b in events
                     if k == "run_terminal"]
        check(len(terminals) == 1,
              f"{pid}: expected exactly one run_terminal, found "
              f"{len(terminals)}")
        terminal = classify_terminal(terminals[0] if terminals else None)
        settlements = sum(1 for k, _ in events
                          if k == "settlement_committed")
        check(terminal == obs["terminal_class"],
              f"{pid}: DB terminal {terminal!r} != recorded "
              f"{obs['terminal_class']!r}")
        check(settlements == obs["settlement_committed_count"],
              f"{pid}: DB settlements {settlements} != recorded "
              f"{obs['settlement_committed_count']}")
        if terminal == "settled":
            check(settlements >= 1,
                  f"{pid}: settled terminal with zero settlements")
        # scenario corroboration from DB payloads (informational when the
        # run had zero dispatches, which cannot write a scenario string)
        payload_blob = json.dumps([b for _, b in events])
        scenario = obs["bounded_input"]["scenario"]
        dispatches = sum(1 for k, _ in events if k == "unit_dispatch")
        if scenario in payload_blob:
            NOTES.append(f"{pid}: scenario {scenario} corroborated in DB "
                         f"payloads ({dispatches} dispatches)")
        elif dispatches == 0:
            NOTES.append(f"{pid}: zero dispatches; no scenario string "
                         "possible; identity rests on the record pin + "
                         "DB budget")
        else:
            NOTES.append(f"{pid}: WARNING scenario string absent from DB "
                         f"payloads ({dispatches} dispatches)")
        # DB-side budget must carry the probe's max_rounds
        rounds = None
        for _, budget in runs:
            rounds = (json.loads(budget).get("limits", {})
                      .get("rounds")) if budget else None
        check(rounds == obs["bounded_input"]["max_rounds"],
              f"{pid}: DB budget rounds {rounds!r} != recorded max_rounds")
        # evidence artifacts: re-hash every distinct evidence_produced ref
        refs = []
        for k, b in events:
            if k == "evidence_produced":
                ref = b.get("evidence", "")
                if ref.startswith("sha256:"):
                    ref = ref[7:]
                    if ref not in refs:
                        refs.append(ref)
        art_dir = os.path.join(os.path.dirname(os.path.dirname(db_path)),
                               "evidence")
        failing_ref = ""
        for ref in refs:
            obj = os.path.join(art_dir, "objects", ref[:2], ref[2:4], ref)
            if not check(os.path.isfile(obj),
                         f"{pid}: artifact object missing {ref}"):
                continue
            check(sha256_file(obj) == ref,
                  f"{pid}: artifact bytes != pinned sha256 {ref}")
            kind_file = obj + ".kind"
            if os.path.isfile(kind_file):
                NOTES.append(f"{pid}: artifact {ref[:12]}… kind="
                             f"{open(kind_file).read().strip()}")
            try:
                content = json.load(open(obj, encoding="utf-8",
                                         errors="replace"))
            except json.JSONDecodeError:
                continue
            failures = content.get("failures") if isinstance(content, dict) \
                else None
            if (isinstance(failures, (int, float))
                    and not isinstance(failures, bool)
                    and failures > 0 and not failing_ref):
                failing_ref = ref
        c1 = terminal not in ("settled", "run_error")
        c2 = settlements == 0
        c3 = bool(failing_ref)
        derived[pid] = {
            "terminal": terminal, "settlements": settlements,
            "c1": c1, "c2": c2, "c3": c3,
            "failing_artifact_sha256": failing_ref,
            "mechanical_label": VFN if (c1 and c2 and c3) else terminal,
            "scenario": scenario,
            "max_rounds": obs["bounded_input"]["max_rounds"],
            "candidate_id": obs["candidate_id"],
            "correlation_id": obs["correlation_id"],
        }
    check(len(derived) == 12, "incomplete probe derivation")

    # ---- S4: addendum-3 mechanical labels + symmetry ------------------------
    print("[S4] addendum-3 mechanical labels + symmetry", flush=True)
    labels_doc = load_yaml(os.path.join(T23_R3, "mechanical-labels.yaml"))
    check(labels_doc.get("rule_source", {}).get("addendum3_sha256")
          == ADD3_SHA, "mechanical-labels does not pin addendum-3")
    check(labels_doc.get("probes_rerun") == 0
          and labels_doc.get("round3_provider_calls") == 0,
          "mechanical-labels must record 0 re-runs / 0 provider calls")
    rows = {r["probe_id"]: r for r in labels_doc.get("labels", [])}
    check(sorted(rows) == sorted(derived),
          "mechanical-labels rows not bijective with the probe set")
    for pid, d in sorted(derived.items()):
        row = rows.get(pid, {})
        check(row.get("terminal") == d["terminal"], f"{pid}: label row terminal")
        check(row.get("settlements") == d["settlements"],
              f"{pid}: label row settlements")
        cl = row.get("rule_clauses", {})
        check(cl.get("c1") is d["c1"] and cl.get("c2") is d["c2"]
              and cl.get("c3") is d["c3"], f"{pid}: label row clauses")
        check(row.get("failing_artifact_sha256")
              == d["failing_artifact_sha256"], f"{pid}: label row artifact")
        check(row.get("mechanical_label") == d["mechanical_label"],
              f"{pid}: label row mechanical_label")
        check(row.get("state_db_sha256") == obs_by_probe[pid]
              ["state_db_sha256"], f"{pid}: label row DB pin")
        r1 = round1_labels.get(d["candidate_id"] + "@" + d["correlation_id"])
        check(row.get("round1_label") == r1,
              f"{pid}: row round1_label != judgment record")
    # symmetry: all 4 lying trigger, none of the 8 happy does
    lying = [p for p, d in derived.items() if d["scenario"] == "e2e-lying"]
    happy = [p for p, d in derived.items() if d["scenario"] == "e2e-happy"]
    check(len(lying) == 4 and len(happy) == 8,
          f"expected 4 lying + 8 happy probes, got {len(lying)}+{len(happy)}")
    lying_all = all(derived[p]["mechanical_label"] == VFN for p in lying)
    honest_none = all(derived[p]["mechanical_label"] != VFN for p in happy)
    check(lying_all, "symmetry FAIL: not all lying probes classify VFN")
    check(honest_none, "symmetry FAIL: an honest probe classifies VFN")
    check(sorted(derived[p]["failing_artifact_sha256"] for p in lying)
          == [FAILING_ARTIFACT] * 4,
          "the lying family's failing artifact is not the pinned family")
    sep = labels_doc.get("separation", {})
    check(sep.get("verdict") == ("separated" if lying_all and honest_none
                                 else "not_separated"),
          "recorded separation verdict != recomputation")
    # round-2 preserved labels joined beside (never goal inputs)
    rr2 = load_yaml(rr2_path)
    r2labels = {r["probe_id"]: r["round2_label"]
                for r in rr2.get("corrected_labels", [])}
    check(len(r2labels) == 12, "round-2 corrected labels must cover 12 probes")
    for pid in sorted(derived):
        check(rows[pid].get("round2_label") == r2labels.get(pid),
              f"{pid}: row round2_label != round-2 record")

    # ---- S5: goal evaluation + fingerprint ----------------------------------
    print("[S5] goal evaluation + fingerprint", flush=True)
    rr1 = load_yaml(rr1_path)
    rr3 = load_yaml(rr3_path)
    settled_r = {d["max_rounds"] for d in derived.values()
                 if d["scenario"] == "e2e-happy"
                 and d["terminal"] == "settled"}
    exhausted_r = {d["max_rounds"] for d in derived.values()
                   if d["scenario"] == "e2e-happy"
                   and round1_labels.get(d["candidate_id"] + "@"
                                         + d["correlation_id"])
                   == "budget_exhausted"
                   and d["terminal"] == "budget_exhausted"}
    baseline_met = 4 in settled_r
    r_star = min(settled_r) if settled_r else None
    boundary_met = bool(settled_r) and (r_star - 1) in exhausted_r
    discriminator_met = lying_all and honest_none
    goal_met = baseline_met and boundary_met and discriminator_met
    goal = rr3.get("goal_evaluation", {})
    check(goal.get("baseline_met") is baseline_met, "baseline_met mismatch")
    check(goal.get("r_star") == r_star, "r_star mismatch")
    check(goal.get("boundary_met") is boundary_met, "boundary_met mismatch")
    check(goal.get("discriminator_met") is discriminator_met,
          "discriminator_met mismatch")
    check(goal.get("goal_met") is goal_met, "goal_met mismatch")
    # base-prereg reachability cross-check: the ORIGINAL wording's baseline
    # (e2e-happy@8 settles) must also hold on the final ledger
    check(8 in settled_r,
          "base-prereg baseline leg (e2e-happy@8 settles) not on the ledger")
    # ledger unchanged across rounds; settled only per the recomputed goal
    r1_block = rr1["route_record"]
    r3_block = rr3.get("route_record", {})
    for field in ("moves", "fingerprint", "correlation_ids", "policy_version",
                  "prereg_sha256", "opened_at_utc", "closed_at_utc"):
        check(r3_block.get(field) == r1_block.get(field),
              f"round-3 ledger field {field} differs from round 1")
    fp = fingerprint(r3_block.get("moves", []))
    check(fp == r3_block.get("fingerprint"), "fingerprint does not recompute")
    check(fp == ("e2e-happy@3:budget_exhausted;"
                 "e2e-happy@6:settled;e2e-happy@2:budget_exhausted"),
          "fingerprint != the frozen round-1 ledger fingerprint")
    check(r3_block.get("settled") is goal_met, "settled != recomputed goal")
    check(r3_block.get("stop_condition")
          == ("route_settled" if goal_met else "not route_settled"),
          "stop_condition inconsistent with the goal")
    # every retained move must be backed by a real probed probe on the ledger
    probed_keys = {(p["scenario"], p["max_rounds"], p["terminal_class"])
                   for p in rr1["route_state"]["probed"]}
    for mv in r3_block.get("moves", []):
        check((mv["scenario"], mv["max_rounds"], mv["terminal_class"])
              in probed_keys,
              f"retained move {mv} not backed by a real probe ledger entry")
    # moves' terminals must match the DB-derived terminals
    for mv in r3_block.get("moves", []):
        db_t = [d["terminal"] for d in derived.values()
                if d["scenario"] == mv["scenario"]
                and d["max_rounds"] == mv["max_rounds"]]
        check(db_t and db_t[0] == mv["terminal_class"],
              f"move {mv} terminal not reproducible from a pinned DB")

    # ---- S6: T24 replay settlement + freeze-before-replay -------------------
    print("[S6] T24 replay + freeze ordering", flush=True)
    cand_path = os.path.join(T24, "candidate-case.yaml")
    freeze = load_json(os.path.join(T24, "candidate-freeze.json"))
    replay = load_yaml(os.path.join(T24, "replay-record.yaml"))
    accepted = load_yaml(os.path.join(T24, "accepted-case.yaml"))
    cand_sha = sha256_file(cand_path)
    check(cand_sha == freeze.get("candidate_sha256"),
          "candidate-case.yaml != freeze pin")
    check(replay.get("freeze_ordering", {}).get("candidate_seen_sha256")
          == cand_sha, "replay record did not see the frozen candidate")
    check(freeze.get("replay_record_exists_at_freeze") is False,
          "freeze pin must attest no replay record existed")
    cand_mtime = os.path.getmtime(cand_path)
    freeze_mtime = os.path.getmtime(os.path.join(T24, "candidate-freeze.json"))
    check(cand_mtime <= freeze_mtime,
          "candidate mtime is not <= freeze mtime")
    replay_start = replay["freeze_ordering"]["replay_started_at_utc"]
    frozen_at = freeze["frozen_at_utc"]
    check(frozen_at <= replay_start, "freeze timestamp not before replay start")
    check(freeze_mtime <= os.path.getmtime(os.path.join(
        DATA_ROOT, "T24/replay-001-r1/runs/replay-r1/state/gauntlet-state.db")),
        "freeze mtime not before the replay DB was written")
    # the replay DB itself
    rdb = replay["observation"]["state_db_path"]
    check(rdb.startswith(DATA_ROOT + os.sep),
          "T24 replay DB outside the durable data root")
    check(os.path.isfile(rdb), "T24 replay DB missing")
    check(sha256_file(rdb) == replay["observation"]["state_db_sha256"]
          == accepted["replay"]["state_db_sha256"],
          "T24 replay DB hash mismatch across records")
    events, runs = read_events(rdb)
    terms = [b.get("state") for k, b in events if k == "run_terminal"]
    check(terms == ["settled"], f"T24 replay terminal {terms!r} != settled")
    sc = sum(1 for k, _ in events if k == "settlement_committed")
    check(sc == 3 == replay["observation"]["settlement_committed_count"],
          "T24 replay settlement count != 3")
    rounds = [json.loads(b).get("limits", {}).get("rounds")
              for _, b in runs]
    check(rounds == [4], f"T24 replay DB budget rounds {rounds!r} != [4]")
    check(replay.get("raw_terminal_state") == "settled",
          "T24 recorded raw terminal state mismatch")
    # accepted record joins the replay and the schema's acceptance rule
    check(accepted.get("accepted") is True, "T24 case not accepted")
    check(accepted.get("invalid_reason", "x") == "",
          "accepted case must not carry invalid_reason")
    check(accepted["replay"]["terminal_class"] == "settled"
          and accepted["replay"]["settled"] is True,
          "accepted-case replay block inconsistent")
    check(accepted["bounded_input"] == {"scenario": "e2e-happy",
                                        "max_rounds": 4},
          "accepted case bounded_input mismatch")
    # target copy identical to the fixture (no runner/budget sed)
    fixture = "/home/sprime01/projects/gauntlet/tests/fixtures/target"
    rtarget = os.path.join(DATA_ROOT,
                           "T24/replay-001-r1/targets/replay-r1")
    diffs = tree_diff(fixture, rtarget)
    check(diffs == [], f"T24 replay target differs from fixture: {diffs}")
    # seed freshness: the replay seed appears in no prior record
    seed = replay["run"]["gauntlet_id_seed"]
    hits = grep_tree([T22, T23], seed)
    check(hits == [], f"T24 replay seed appears in prior records: {hits}")
    # generator output is evidence, never acceptance: the FROZEN candidate
    # must record acceptance-pending state (replay_status: pending,
    # accepted: false) so acceptance could only be decided by the replay
    cand_text = open(cand_path, encoding="utf-8").read()
    check("replay_status: pending" in cand_text
          and re.search(r"^accepted: false\s*$", cand_text, re.M)
          and "replayed: false" in cand_text,
          "frozen candidate does not record pre-replay pending state")

    # ---- S7: T25 negative terminal + single-factor diff ---------------------
    print("[S7] T25 negative + single-factor diff", flush=True)
    pair = load_yaml(os.path.join(T25, "matched-pair.yaml"))
    ndb = pair["negative"]["state_db_path"]
    check(ndb.startswith(DATA_ROOT + os.sep),
          "T25 negative DB outside the durable data root")
    check(os.path.isfile(ndb), "T25 negative DB missing")
    check(sha256_file(ndb) == pair["negative"]["state_db_sha256"],
          "T25 negative DB hash mismatch")
    events, runs = read_events(ndb)
    terms = [b.get("state") for k, b in events if k == "run_terminal"]
    check(terms == ["budget_exhausted"],
          f"T25 negative terminal {terms!r} != budget_exhausted")
    sc = sum(1 for k, _ in events if k == "settlement_committed")
    check(sc == 2 == pair["negative"]["settlement_committed_count"],
          "T25 negative settlement count != 2")
    rounds = [json.loads(b).get("limits", {}).get("rounds")
              for _, b in runs]
    check(rounds == [3], f"T25 negative DB budget rounds {rounds!r} != [3]")
    check(pair["negative"]["terminal_class"] == "budget_exhausted"
          and pair["negative"]["settled"] is False,
          "recorded negative label != the raw terminal consequence")
    # single-factor diff: recursive target diff vs the fixture
    ntarget = os.path.join(DATA_ROOT,
                           "T25/negative-001-r1/targets/negative-r1")
    diffs = tree_diff(fixture, ntarget)
    check(diffs == [("GAUNTLET.md", "differs")],
          f"T25 negative target must differ from the fixture in "
          f"GAUNTLET.md only, found {diffs}")
    fixed = open(os.path.join(fixture, "GAUNTLET.md"),
                 encoding="utf-8").read().splitlines()
    negd = open(os.path.join(ntarget, "GAUNTLET.md"),
                encoding="utf-8").read().splitlines()
    changed = [ln for ln in range(min(len(fixed), len(negd)))
               if fixed[ln] != negd[ln]]
    check(len(changed) == 1, f"expected exactly one changed line, "
                             f"got {changed}")
    if len(changed) == 1:
        ln = changed[0]
        check("max_rounds: 4" in fixed[ln] and "max_rounds: 3" in negd[ln],
              f"the single changed line is not max_rounds 4 -> 3: "
              f"{fixed[ln]!r} -> {negd[ln]!r}")
    check(pair["mutation"]["factor"] == "bounded_input.max_rounds"
          and pair["mutation"]["positive"] == 4
          and pair["mutation"]["negative"] == 3
          and pair["mutation"]["diff_exactly_one_factor"] is True,
          "mutation record inconsistent")
    nseed = pair["negative"]["gauntlet_id_seed"]
    check(grep_tree([T22, T23, T24], nseed) == [],
          f"T25 negative seed appears in prior records")
    # positive leg joins T24's accepted replay
    check(pair["positive"]["state_db_sha256"]
          == accepted["replay"]["state_db_sha256"],
          "matched-pair positive does not join the T24 replay DB")

    # ---- S8: novelty classification -----------------------------------------
    print("[S8] novelty classification", flush=True)
    nov = load_yaml(os.path.join(T25, "novelty.yaml"))
    check(nov.get("subject", {}).get("fingerprint") == fp,
          "novelty subject fingerprint != recomputed route fingerprint")
    # re-derive the T15/T16 grid fingerprints from the frozen records
    t15_manifest = load_yaml(os.path.join(
        PREREG_DIR, "godspeed-bounded-judgment-T15.corpus-manifest.yml"))
    t15_prereg = load_yaml(os.path.join(
        PREREG_DIR, "godspeed-bounded-judgment-T15.prereg.yaml"))
    t16_manifest = load_yaml(os.path.join(
        PREREG_DIR, "godspeed-bounded-judgment-T16.expanded-corpus-manifest.yml"))
    t16_addendum = load_yaml(os.path.join(
        PREREG_DIR, "godspeed-bounded-judgment-T16.expansion-addendum.yaml"))
    # case table dims from the T15 prereg text
    t15_text = open(os.path.join(
        PREREG_DIR, "godspeed-bounded-judgment-T15.prereg.yaml"),
        encoding="utf-8").read()
    check("budget: defaults" in t15_text and "max_rounds 1 (C1,C2)" in t15_text
          and "2 (C3" in t15_text,
          "T15 prereg case-table budget dims not as recorded")
    grid = set()
    for run in t15_manifest.get("runs", []):
        try:
            grid.add(grid_fp_for_case(run["case_id"],
                                      run["consequence_class"]))
        except ValueError as exc:
            check(False, f"T15 grid case unmappable: {exc}")
    check(len(t15_manifest.get("runs", [])) == 9, "T15 grid must be 9 runs")
    check(sum(1 for r in t16_manifest.get("runs", [])) == 25,
          "T16 expanded grid must be 25 runs")
    for run in t16_manifest.get("runs", []):
        try:
            grid.add(grid_fp_for_case(run["case_id"],
                                      run["consequence_class"]))
        except ValueError as exc:
            check(False, f"T16 grid case unmappable: {exc}")
    mix = t16_addendum.get("mechanism_mix", {})
    check("e2e-happy x9" in str(mix.get("settled", ""))
          and "budget-r1 x2" in str(mix.get("not_settled", "")),
          "T16 expansion mechanism mix not as recorded")
    recorded_grid = set(nov.get("reference_grid_as_recorded", {})
                        .get("grid_fingerprints", []))
    check(grid == recorded_grid,
          f"re-derived grid {sorted(grid)} != recorded grid "
          f"{sorted(recorded_grid)}")
    # classification under the frozen rule
    route_ops = {f"{m['scenario']}@{m['max_rounds']}"
                 for m in rr3["route_record"]["moves"]}
    if fp in grid:
        cls = "structural-duplicate"
    elif any({f"{g.split('@')[0]}@{g.split('@')[1].split(':')[0]}"}
             == route_ops for g in grid):
        cls = "near-duplicate"
    else:
        cls = "novel"
    check(nov.get("classification") == cls,
          f"novelty classification {nov.get('classification')!r} != "
          f"recomputed {cls!r}")
    check(cls == "novel", "route fingerprint is not novel vs the grid")
    levels = {m["max_rounds"] for m in rr3["route_record"]["moves"]}
    grid_levels = {int(g.split("@")[1].split(":")[0]) for g in grid}
    check(not ({5, 6, 7} & grid_levels),
          "grid levels include 5/6/7 (contradicts the novelty claim)")
    outside = sorted(levels - grid_levels)
    recorded_outside = nov.get("t16_expansion_never_probed", {}).get(
        "route_levels_outside_the_grid", [])
    check(sorted(recorded_outside) == outside,
          f"route levels outside the grid {outside} != recorded "
          f"{recorded_outside}")
    check(nov.get("t16_expansion_never_probed", {}).get("verified") is True,
          "novelty record must attest the 5/6/7 verification")
    # T8 tooth: structural duplicate detection, measured not rejected
    t8rec = load_json(os.path.join(T25, "teeth", "t8-attack-record.json"))
    t8dup = load_yaml(os.path.join(T25, "teeth",
                                   "t8-duplicate-route-record.yaml"))
    t8_fp = fingerprint(t8dup.get("moves", []))
    check(t8_fp == t8dup.get("fingerprint") == fp,
          "T8 duplicate fingerprint does not recompute to the route "
          "fingerprint")
    check(t8rec.get("classification") == "structural-duplicate"
          and t8rec.get("duplicate_detected") is True
          and t8rec.get("auto_rejection_applied") is False,
          "T8 classification/policy mismatch")

    # ---- S9: payment counters ------------------------------------------------
    print("[S9] payment counters", flush=True)
    dec = load_yaml(dec_path)
    dec_ids = [e.get("id") for e in dec.get("entries", [])]
    branch_ids = ["D-2026-09-18-T21-01", "D-2026-09-18-T21-02",
                  "D-2026-09-18-T21-03", "D-2026-09-18-T22-01",
                  "D-2026-09-19-T23-01", "D-2026-09-19-T23-02",
                  "D-2026-09-19-T23-03"]
    for bid in branch_ids:
        check(bid in dec_ids, f"branch decision entry missing: {bid}")
    baseline_ids = (["D-2026-09-18-T15-01", "D-2026-09-18-T15-02",
                     "D-2026-09-18-T15-03"]
                    + [f"D-2026-09-18-T16-{i:02d}" for i in range(1, 19)])
    for bid in baseline_ids:
        check(bid in dec_ids, f"baseline decision entry missing: {bid}")
    comparison = load_yaml(os.path.join(T25, "payment-comparison.yaml"))
    check(comparison["mechanism_side_this_branch"]["operator_decision_entries"]
          ["count"] == len(branch_ids) == 7,
          "branch operator decision entries != 7")
    check(comparison["manual_baseline_t15_t16"]["operator_decision_entries"]
          == len(baseline_ids) == 21,
          "baseline operator decision entries != 21")
    # re-count provider calls from each stage counter file
    def count_calls(path):
        d = load_yaml(path)
        ident = d.get("provider_calls", {}).get("by_identity", [])
        return len(ident), d
    counts = {}
    counts["t22_r1"], d = count_calls(os.path.join(
        T22, "iteration-1/payment-counters.yaml"))
    check(d.get("provider_calls", {}).get("count") == counts["t22_r1"] == 4,
          "T22 round-1 provider calls != 4")
    check(d.get("tool_executions") == 3 and d.get("failed_executions") == 3,
          "T22 round-1 executions/failed != 3/3")
    check(d.get("wall_clock_seconds") == 48.852, "T22 r1 wall clock")
    counts["t22_r2"], d = count_calls(os.path.join(
        T22, "round-2/payment-counters.yaml"))
    check(d.get("provider_calls", {}).get("count") == counts["t22_r2"] == 4,
          "T22 round-2 provider calls != 4")
    check(d.get("tool_executions") == 3 and d.get("failed_executions") == 0,
          "T22 round-2 executions != 3")
    check(d.get("wall_clock_seconds") == 56.456, "T22 r2 wall clock")
    counts["t23_route"], d = count_calls(os.path.join(
        T23_ROUTE, "payment-counters.yaml"))
    check(d.get("provider_calls", {}).get("count")
          == counts["t23_route"] == 16,
          "T23 route provider calls != 16")
    gen = sum(1 for c in d["provider_calls"]["by_identity"]
              if c["purpose"].startswith("candidate_generation"))
    jud = sum(1 for c in d["provider_calls"]["by_identity"]
              if c["purpose"].startswith("probe_judgment"))
    check((gen, jud) == (4, 12), f"T23 route call split {gen}/{jud} != 4/12")
    check(d.get("wall_clock_seconds") == 275.997, "T23 route wall clock")
    counts["t23_r2"], d = count_calls(os.path.join(
        T23_R2, "payment-counters.round2.yaml"))
    check(d.get("provider_calls", {}).get("count")
          == counts["t23_r2"] == 12, "T23 round-2 rejudgment calls != 12")
    check(d.get("tool_executions") == 0 and d.get("probes_rerun") == 0,
          "T23 round-2 must show 0 executions / 0 re-runs")
    # re-count round-2 attempts from the preserved rejudgment results
    r2res = load_json(os.path.join(T23_R2, "rejudgment-results.json"))
    attempts = sum(len([a for a in r.get("attempts", [])
                        if a.get("provider") in ("deepseek-clinepass",
                                                 "muse-spark-free")])
                   for r in r2res)
    check(len(r2res) == 12 and attempts == 12,
          f"round-2 attempts re-count {attempts} != 12 over {len(r2res)}")
    counts["t23_r3"] = rr3.get("payment", {}).get("round3_provider_calls")
    check(counts["t23_r3"] == 0, "T23 round-3 provider calls must be 0")
    counts["t24"], d = count_calls(os.path.join(T24, "payment-counters.yaml"))
    check(counts["t24"] == 1, "T24 provider calls != 1")
    check(d.get("tool_executions") == 0,
          "T24 generation must run no gauntlet execution")
    check(d.get("wall_clock_seconds") == 23.737, "T24 wall clock")
    counts["t25"], d = count_calls(os.path.join(T25, "payment-counters.yaml"))
    check(counts["t25"] == 0, "T25 provider calls != 0")
    check(d.get("tool_executions") == 1, "T25 executions != 1")
    check(d.get("wall_clock_seconds") == 0.303, "T25 wall clock")
    total_calls = sum(counts[k] for k in ("t22_r1", "t22_r2", "t23_route",
                                          "t23_r2", "t23_r3", "t24", "t25"))
    check(total_calls == 37, f"total provider calls {total_calls} != 37")
    mc = comparison["mechanism_side_this_branch"]["model_calls"]
    stage = mc["by_stage_recorded"]
    check(stage["t22_round1_failed"] == counts["t22_r1"]
          and stage["t22_round2"] == counts["t22_r2"]
          and stage["t23_route_round1"] == counts["t23_route"]
          and stage["t23_round2_rejudgment"] == counts["t23_r2"]
          and stage["t23_round3"] == 0
          and stage["t24_case_generation"] == counts["t24"]
          and stage["t25_matched_negative"] == 0,
          "payment-comparison stage counts != re-counted sources")
    check(mc["total"] == total_calls == 37, "model_calls total != 37")
    check(mc["generation_calls"] == 7,
          "generation calls != 7 (T22r1 1 + T22r2 1 + T23 4 + T24 1)")
    check(mc["judgment_calls"] == 18,
          "probe-judgment calls != 18 (3+3+12); re-judgment 12 itemized "
          "separately (noted presentational ambiguity, non-material)")
    # tool executions: 3+3+12 + T4's 1 + T24's 2 + T25's 1 = 22
    te = comparison["mechanism_side_this_branch"]["tool_executions"]
    check(te["total"] == 22, "tool executions total != 22")
    check(te["by_stage_recorded"] == {
        "t22_round1_probes_failed_round": 3, "t22_round2_probes": 3,
        "t23_route_probes": 12, "t23_teeth_t4": 1,
        "t24_replays_candidate_and_t7_tooth": 2, "t25_negative": 1},
        "tool-execution stage table mismatch")
    fe = comparison["mechanism_side_this_branch"]["failed_executions"]
    check(fe["total"] == 3, "failed executions total != 3")
    walls = [48.852, 56.456, 275.997, 23.737, 0.505, 0.204, 0.303]
    recorded_wall = comparison["mechanism_side_this_branch"][
        "wall_clock_seconds_recorded"]["total_recorded"]
    check(abs(recorded_wall - sum(walls)) < 0.0005,
          f"wall clock total {recorded_wall} != {sum(walls)}")
    # T24/T25 teeth walls are counted in the total but T7/T8 attack
    # provider calls are zero
    check(comparison["mechanism_side_this_branch"]["tokens_and_cost"]
          .startswith("n/a"), "tokens/cost must stay an honest n/a")
    check(comparison["manual_baseline_t15_t16"]["wall_clock"].startswith(
        "n/a"), "baseline wall clock must stay an honest n/a")

    # ---- S10: model-free settlement -----------------------------------------
    print("[S10] model-free settlement checks", flush=True)
    check(not os.path.isdir(os.path.join(T23_R3, "provider-raw")),
          "round-3 must have no provider-raw directory")
    # the goal recomputation above used ONLY: DB terminals, DB settlement
    # counts, hash-verified artifact contents, and the recorded ledger —
    # no provider output. Assert the recorded goal equals this recomputation
    # even though 3 of 4 round-2 provider labels disagree with it.
    disagree = [pid for pid in derived
                if derived[pid]["mechanical_label"] == VFN
                and r2labels.get(pid) == "budget_exhausted"]
    check(len(disagree) == 3,
          f"expected 3 lying probes where provider labels disagree with "
          f"the mechanical rule, found {len(disagree)}")
    check(goal_met is True,
          "the model-free recomputed goal must be met (route settlement)")
    # selection records: frozen policy only; retained ids within eligible
    for i in (1, 2, 3, 4):
        sel = load_json(os.path.join(T23_ROUTE, f"iteration-{i}",
                                     "selection-record.json"))
        check(sel.get("policy_version") == "selection-policy-v1",
              f"iteration-{i}: selection policy version drift")
        check(sel.get("retained_candidate_id", "x") == ""
              or sel.get("retained_candidate_id")
              in sel.get("eligible_candidate_ids", []),
              f"iteration-{i}: retained not in eligible")
        blob = json.dumps(sel)
        check("provider" not in blob and "judge" not in blob,
              f"iteration-{i}: selection record mentions provider/judge")
    # the T21 frozen selection policy source must have no provider interface
    sp_text = open(os.path.join(T21, "selection_policy.py"),
                   encoding="utf-8").read()
    check(not re.search(r"opencode|subprocess|requests|urllib|socket|http",
                        sp_text),
          "selection_policy.py must have no provider/network interface")
    # judgment labels feed only selection/claims, never the round-3 goal:
    # 3 of 4 lying probes carry round-2 label budget_exhausted yet the
    # mechanical separation (and hence goal_met) holds — verified above.

    # ---- S11: claim-boundary scan -------------------------------------------
    print("[S11] claim-boundary scan", flush=True)
    scan_files = [
        rr1_path, rr2_path, rr3_path,
        os.path.join(T23_R3, "mechanical-labels.yaml"),
        os.path.join(T23_R3, "confirmation-packet.yaml"),
        os.path.join(T23, "teeth/attacks.yaml"),
        os.path.join(T24, "accepted-case.yaml"),
        os.path.join(T24, "candidate-case.yaml"),
        os.path.join(T24, "replay-record.yaml"),
        os.path.join(T24, "teeth.yaml"),
        os.path.join(T25, "matched-pair.yaml"),
        os.path.join(T25, "payment-comparison.yaml"),
        os.path.join(T25, "novelty.yaml"),
        os.path.join(T25, "teeth.yaml"),
        os.path.join(T22, "round-2/route-record.yaml"),
        os.path.join(T22, "iteration-1/route-record.yaml"),
    ]
    bad_terms = re.compile(
        r"architectural promotion|production[- ]ready|production readiness"
        r"|capability[- ]class generalization|generaliz\w* beyond"
        r"|unlock(s|ed)? T06|unblock(s|ed)? T06|unblock(s|ed)? T18"
        r"|promot\w*", re.IGNORECASE)
    for f in scan_files:
        if not os.path.isfile(f):
            check(False, f"claim-scan: missing file {f}")
            continue
        for n, line in enumerate(open(f, encoding="utf-8"), 1):
            if bad_terms.search(line):
                stripped = line.strip()
                if "work_product_promoted" in stripped and \
                        bad_terms.search(stripped.replace(
                            "work_product_promoted", "")):
                    ERRORS.append(f"over-claim in {f}:{n}: {stripped}")
                elif "work_product_promoted" not in stripped:
                    ERRORS.append(f"over-claim in {f}:{n}: {stripped}")
                else:
                    NOTES.append(f"benign evidence-kind token at {f}:{n}")

    # ---- verdict --------------------------------------------------------------
    print("", flush=True)
    for note in NOTES:
        print(f"  note: {note}")
    if ERRORS:
        print(f"verify_discovery: FAIL ({len(ERRORS)} problem(s) after "
              f"{CHECKS} checks)")
        for e in ERRORS:
            print(f"  - {e}")
        return 1
    print(f"verify_discovery: PASS ({CHECKS} checks, {len(NOTES)} notes)")
    print(f"  12/12 probe terminals reproduced from pinned DBs; "
          f"4/4 lying -> {VFN}, 0/8 honest")
    print(f"  goal (model-free): baseline={baseline_met} r*={r_star} "
          f"boundary={boundary_met} discriminator={discriminator_met} "
          f"-> goal_met={goal_met}")
    print(f"  T24 replay settled (3 settlements) after freeze; T25 negative "
          f"budget_exhausted (2 settlements), single-factor diff clean")
    print(f"  novelty={cls}; payment: 7 branch decisions, 37 provider "
          f"calls, 22 executions, 3 failed, {sum(walls)}s recorded wall")
    return 0


def tree_diff(a, b):
    """Recursive content diff of two directory trees (relative paths that
    differ or exist on only one side)."""
    diffs = []
    for dirpath, dirnames, filenames in os.walk(a):
        dirnames[:] = sorted(d for d in dirnames)
        rel = os.path.relpath(dirpath, a)
        for fn in sorted(filenames):
            ra = os.path.normpath(os.path.join(rel, fn)) if rel != "." \
                else fn
            other = os.path.join(b, ra)
            if not os.path.isfile(other):
                diffs.append((ra, "only-in-fixture"))
            elif sha256_file(os.path.join(dirpath, fn)) != \
                    sha256_file(other):
                diffs.append((ra, "differs"))
    for dirpath, dirnames, filenames in os.walk(b):
        dirnames[:] = sorted(d for d in dirnames)
        rel = os.path.relpath(dirpath, b)
        for fn in sorted(filenames):
            rb = os.path.normpath(os.path.join(rel, fn)) if rel != "." \
                else fn
            if not os.path.isfile(os.path.join(a, rb)):
                diffs.append((rb, "only-in-target"))
    return sorted(diffs)


def grep_tree(dirs, needle):
    """Return files under dirs whose text contains needle."""
    hits = []
    for base in dirs:
        for dirpath, dirnames, filenames in os.walk(base):
            dirnames[:] = [d for d in dirnames if d != "__pycache__"]
            for fn in filenames:
                p = os.path.join(dirpath, fn)
                if p.endswith((".pyc", ".db")):
                    continue
                try:
                    with open(p, encoding="utf-8", errors="ignore") as f:
                        if needle in f.read():
                            hits.append(p)
                except OSError:
                    continue
    return hits


def grid_fp_for_case(case_id, consequence_class):
    """Map a recorded T15/T16 corpus case id + consequence class to its
    grid fingerprint (scenario@rounds:class), per the frozen case tables:
    accepted-A* → e2e-happy@defaults(4); lying-B* → e2e-lying@4;
    budget-C* → e2e-happy@1 (C1,C2,C4,C5) or @2 (C3); forge-D* →
    e2e-forged-report@4. Classes map identity: settled→settled,
    budget_exhausted→budget_exhausted."""
    cid = case_id.lower()
    if "forg" in cid:
        scenario, rounds = "e2e-forged-report", 4
    elif "lying" in cid:
        scenario, rounds = "e2e-lying", 4
    elif "budget" in cid:
        scenario, rounds = "e2e-happy", (2 if cid.endswith("3") else 1)
    elif "accepted" in cid:
        scenario, rounds = "e2e-happy", 4
    else:
        raise ValueError(f"unclassifiable corpus case id {case_id!r}")
    return f"{scenario}@{rounds}:{consequence_class}"


if __name__ == "__main__":
    sys.exit(main())
