#!/usr/bin/env python3
"""T23 GATE, round 3 — verify_route.round3.py (run from the sea-rs repo root).

Round-3 correction gate for the FROZEN addendum-3 mechanical discriminator
rule applied to the PRESERVED route t23-route-001 ledger. Rounds 1-2 gates
are NOT modified; this gate reuses their frozen helpers by import (shared
fail-closed check pool) and adds the round-3 checks. Exits 0 ONLY if every
check passes; a missing file, a hash mismatch, or an inconsistent label or
goal is a FAILURE.

The round-3 rule (frozen, addendum-3 sha256 1c13889a…6ddb): a probe
observation is mechanically classified verification_failed_nonsettlement
if and only if ALL hold, evaluated deterministically from preserved
records: (c1) the typed terminal is a bounded non-settlement (not settled,
not a run error); (c2) the run's settlement_committed count is 0; (c3) at
least one evidence_produced artifact of the run, hash-verified at its
pinned sha256, has content recording a failing verification
(failures > 0). Learned judgment labels are recorded beside the mechanical
label and never decide the goal.

Checks
  (1) frozen inputs intact: T21 frozen modules + base prereg + addendum-1
      (round-1 helper), addendum-2 AND addendum-3 by their frozen sha256s,
      and the round-1/round-2 route records by their pinned sha256s;
  (2) preserved rounds UNMODIFIED (fail-closed): every entry of round-1's
      evidence-manifest.yml and round-2's evidence-manifest.round2.yml
      still hashes to its pinned sha256 (round-2 checked in both
      directions within round-2/);
  (3) DB pins: all 12 round-1 probe observations re-verified through the
      frozen provenance validator (import, unmodified);
  (4) mechanical labels INDEPENDENTLY re-derived from the pinned state
      databases and hash-verified artifact objects — the round-3
      mechanical-labels.yaml table is NOT trusted, only compared against
      the recomputation; each row's round-1/round-2 judgment labels are
      re-joined from the round-1 judgment records and the round-2
      corrected labels;
  (5) symmetry property: all 4 e2e-lying probes classify
      verification_failed_nonsettlement, none of the 8 e2e-happy probes
      does; the recorded separation verdict must match;
  (6) goal recomputation from the recomputed labels + the round-1 records
      (baseline e2e-happy@4 settled_accepted; boundary r*/r*-1 on this
      ledger; discriminator = the separation verdict); route
      record.round3 settled/stop_condition must match the recomputed goal;
      ledger fields (moves, fingerprint, correlation ids, policy version,
      prereg) must equal round 1 and the fingerprint must recompute;
  (7) the preserved round-2 judgment non-separation finding (1/4) is
      reported unchanged beside the mechanical result;
  (8) payment counters: probes 12, reruns 0, 12 round-2 re-judgment
      provider calls re-counted from the preserved re-judgment results,
      0 round-3 provider calls;
  (9) round-3 manifest hash checks, both directions, fail-closed (every
      entry matches disk; every round-3 .py/.yaml file is pinned), plus
      the confirmation packet's state-DB list joins the round-1 records.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import hashlib
import json
import os
import sqlite3

T23_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ROUND3_DIR = os.path.dirname(os.path.abspath(__file__))
BRANCH_DIR = os.path.dirname(T23_DIR)
T21_DIR = os.path.join(BRANCH_DIR, "T21")
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(BRANCH_DIR)))
DATA_ROOT = os.path.join(os.path.expanduser("~"),
                         ".local/share/godspeed-route-discovery/T23")

sys.path.insert(0, T21_DIR)
sys.path.insert(0, T23_DIR)

import route_contracts as rc  # noqa: E402  (frozen T21 module)
import provenance as prov    # noqa: E402  (frozen round-1 validator)

# round-1 gate module: helpers + shared fail-closed check pool, unmodified
import verify_route as vr    # noqa: E402

import yaml                  # noqa: E402

PREREG_SHA256 = vr.PREREG_SHA256
ADDENDUM1_SHA256 = vr.ADDENDUM_SHA256
ADDENDUM2_SHA256 = (
    "087887c27cc50c6c989fcf7bb214e77aca62ea74d7e32a868ef0f7dd211e4c5a")
ADDENDUM3_SHA256 = (
    "1c13889ae0c2a9621b7f0b803b327ee6dd76a16983c9ed72b00ff309f59c6ddb")
ROUND1_ROUTE_RECORD_SHA256 = (
    "9345c43e6787ef32fbe1a7749c90872de7b5be4b46f73af9708516f4f8ded67e")
ROUND2_ROUTE_RECORD_SHA256 = (
    "147f12e072dc7b578f11bd5d57b4fb7fd1caed7330b73e33acaf3c69eb04a416")
MANIFEST_NAME = "evidence-manifest.round3.yml"
ROUND2_MANIFEST_NAME = "evidence-manifest.round2.yml"

ROUTE_DIR = vr.ROUTE_DIR
VFN = "verification_failed_nonsettlement"


def read_events(db_path: str):
    """Independent read-only event-log read (no builder code)."""
    conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    try:
        rows = conn.execute(
            "select kind, payload from event_log order by offset").fetchall()
    finally:
        conn.close()
    events = []
    for kind, payload in rows:
        try:
            body = json.loads(payload)
        except json.JSONDecodeError:
            body = {}
        events.append((kind, body))
    return events


def classify_terminal(raw_state) -> str:
    """Frozen T22/T23 typed-terminal mapping (same as provenance.py)."""
    return prov.classify_terminal(raw_state)


def rederive_mechanical(obs: dict) -> dict:
    """Independently re-derive the frozen rule for one probe observation
    from its pinned state database and the content-addressed artifacts.

    Fail-closed: a pinned artifact that is missing or does not hash to its
    pin is a gate error (recorded in `errors`), not merely a false clause.
    """
    pid = obs["probe_id"]
    errors: list[str] = []
    db_path = obs["state_db_path"]
    events = read_events(db_path)
    terminals = [b.get("state") for k, b in events if k == "run_terminal"]
    if len(terminals) != 1:
        errors.append(f"{pid}: expected exactly one run_terminal event, "
                      f"found {len(terminals)}")
    terminal = classify_terminal(terminals[0] if terminals else None)
    settlements = sum(1 for k, _ in events if k == "settlement_committed")
    if settlements != obs["settlement_committed_count"]:
        errors.append(f"{pid}: settlement_committed_count mismatch against "
                      "the database")
    # clause 3: distinct evidence_produced artifact refs, hash-verified,
    # JSON content recording failures > 0
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
        if not os.path.isfile(obj):
            errors.append(f"{pid}: pinned artifact object missing for "
                          f"{ref}")
            continue
        h = hashlib.sha256()
        with open(obj, "rb") as f:
            for chunk in iter(lambda: f.read(1 << 20), b""):
                h.update(chunk)
        if h.hexdigest() != ref:
            errors.append(f"{pid}: artifact bytes do NOT hash to the "
                          f"pinned sha256 {ref}")
            continue
        try:
            with open(obj, encoding="utf-8", errors="replace") as f:
                content = json.load(f)
        except json.JSONDecodeError:
            continue  # not a structured verification report: cannot fire c3
        failures = content.get("failures") if isinstance(content, dict) \
            else None
        if isinstance(failures, (int, float)) and not isinstance(
                failures, bool) and failures > 0 and not failing_ref:
            failing_ref = ref
    c1 = terminal not in ("settled", "run_error")
    c2 = settlements == 0
    c3 = bool(failing_ref)
    label = VFN if (c1 and c2 and c3) else terminal
    return {"probe_id": pid, "terminal": terminal, "settlements": settlements,
            "c1": c1, "c2": c2, "c3": c3, "failing_artifact_sha256":
            failing_ref, "mechanical_label": label, "errors": errors}


def resolve_path(spec: str) -> str:
    """Expand a confirmation-packet path spec (repo: / data: prefixes)."""
    if spec.startswith("repo:"):
        return os.path.join(REPO_ROOT, spec[5:])
    if spec.startswith("data:"):
        return os.path.join(DATA_ROOT, spec[5:])
    return spec


def main() -> int:
    print("verify_route.round3: starting (fail-closed)", flush=True)

    # ---- (1) frozen inputs --------------------------------------------------
    vr.verify_t21_manifest()  # T21 modules + base prereg + addendum-1
    pre_dir = os.path.join(REPO_ROOT, ".agents/preregistrations")
    for name, expected in (
            ("godspeed-bounded-judgment-T21.prereg.addendum-2.yaml",
             ADDENDUM2_SHA256),
            ("godspeed-bounded-judgment-T21.prereg.addendum-3.yaml",
             ADDENDUM3_SHA256)):
        path = os.path.join(pre_dir, name)
        if vr.check(os.path.isfile(path), f"frozen input missing: {name}"):
            vr.check(vr.sha256_file(path) == expected,
                     f"frozen input hash mismatch: {name}")
    rr1_path = os.path.join(ROUTE_DIR, "route-record.yaml")
    rr2_path = os.path.join(ROUND3_DIR, os.pardir, "round-2",
                            "route-record.round2.yaml")
    rr2_path = os.path.normpath(rr2_path)
    vr.check(vr.sha256_file(rr1_path) == ROUND1_ROUTE_RECORD_SHA256,
             "round-1 route-record.yaml does not match its pinned sha256")
    vr.check(vr.sha256_file(rr2_path) == ROUND2_ROUTE_RECORD_SHA256,
             "round-2 route-record.round2.yaml does not match its pinned "
             "sha256")

    # ---- (2) round-1/round-2 files unmodified (fail-closed) ------------------
    manifest1_path = os.path.join(T23_DIR, "evidence-manifest.yml")
    if vr.check(os.path.isfile(manifest1_path),
                "round-1 evidence-manifest.yml missing"):
        manifest1 = vr.load_yaml(manifest1_path)
        n1 = 0
        for entry in manifest1.get("files", []):
            path = os.path.join(T23_DIR, entry["path"])
            n1 += 1
            if not vr.check(os.path.isfile(path),
                            f"round-1 file missing: {entry['path']}"):
                continue
            vr.check(vr.sha256_file(path) == entry["sha256"],
                     f"round-1 file MODIFIED since freeze: {entry['path']}")
        vr.check(n1 >= 40, f"round-1 manifest implausibly small ({n1})")
    manifest2_path = os.path.join(ROUND3_DIR, os.pardir, "round-2",
                                  ROUND2_MANIFEST_NAME)
    manifest2_path = os.path.normpath(manifest2_path)
    round2_dir = os.path.dirname(manifest2_path)
    if vr.check(os.path.isfile(manifest2_path),
                f"{ROUND2_MANIFEST_NAME} missing"):
        manifest2 = vr.load_yaml(manifest2_path)
        pinned2 = set()
        for entry in manifest2.get("files", []):
            rel = entry["path"]
            path = os.path.join(round2_dir, rel)
            pinned2.add(os.path.normpath(rel))
            if not vr.check(os.path.isfile(path),
                            f"round-2 file missing: {rel}"):
                continue
            vr.check(vr.sha256_file(path) == entry["sha256"],
                     f"round-2 file MODIFIED since freeze: {rel}")
        on_disk2 = set()
        for dirpath, dirnames, filenames in os.walk(round2_dir):
            dirnames[:] = [d for d in dirnames if d != "__pycache__"]
            for fn in filenames:
                full = os.path.join(dirpath, fn)
                on_disk2.add(os.path.normpath(
                    os.path.relpath(full, round2_dir)))
        on_disk2.discard(os.path.normpath(ROUND2_MANIFEST_NAME))
        unpinned2 = sorted(on_disk2 - pinned2)
        vr.check(not unpinned2,
                 f"round-2 files not pinned in {ROUND2_MANIFEST_NAME}: "
                 f"{unpinned2}")
        stale2 = sorted(pinned2 - on_disk2)
        vr.check(not stale2,
                 f"round-2 manifest pins nonexistent files: {stale2}")

    # ---- load the preserved round-1 probe set -------------------------------
    round1 = vr.load_yaml(rr1_path)
    obs_by_probe = {}
    round1_labels = {}
    for iteration in (1, 2, 3, 4):
        idir = os.path.join(ROUTE_DIR, f"iteration-{iteration}")
        for o in vr.load_json(os.path.join(idir, "probe-observations.json")):
            obs_by_probe[o["probe_id"]] = o
        for j in vr.load_json(os.path.join(idir, "judgment-results.json")):
            round1_labels[j["candidate_id"] + "@" + j["correlation_id"]] = \
                j["label"]
    vr.check(len(obs_by_probe) == 12,
             f"expected 12 preserved round-1 probes, found "
             f"{len(obs_by_probe)}")

    # ---- (3) DB pins through the frozen validator ----------------------------
    for pid in sorted(obs_by_probe):
        res = prov.recompute(obs_by_probe[pid])
        vr.check(res["ok"], f"{pid}: state-DB pin/terminal recheck failed: "
                            f"{res['reasons']}")

    # ---- (4) mechanical labels independently re-derived ----------------------
    labels_path = os.path.join(ROUND3_DIR, "mechanical-labels.yaml")
    labels_doc = vr.load_yaml(labels_path) if vr.check(
        os.path.isfile(labels_path), "mechanical-labels.yaml missing") else {}
    vr.check(labels_doc.get("schema") ==
             "godspeed-bounded-judgment.t23-round3-mechanical-labels",
             "mechanical-labels.yaml carries an unexpected schema")
    vr.check(labels_doc.get("route_id") == "t23-route-001",
             "mechanical-labels.yaml names a different route")
    vr.check(labels_doc.get("rule_source", {}).get("addendum3_sha256") ==
             ADDENDUM3_SHA256,
             "mechanical-labels.yaml does not pin the addendum-3 sha256")
    vr.check(labels_doc.get("probes_rerun") == 0
             and labels_doc.get("round3_provider_calls") == 0,
             "mechanical-labels.yaml must record 0 re-runs and 0 round-3 "
             "provider calls")
    rows = {r.get("probe_id"): r for r in labels_doc.get("labels", [])}
    vr.check(sorted(rows) == sorted(obs_by_probe),
             "mechanical-labels rows are not bijective with the preserved "
             "probe set")

    derived = {}
    for pid in sorted(obs_by_probe):
        obs = obs_by_probe[pid]
        d = rederive_mechanical(obs)
        derived[pid] = d
        for e in d["errors"]:
            vr.check(False, e)
        row = rows.get(pid)
        if not vr.check(isinstance(row, dict),
                        f"{pid}: mechanical-labels row missing"):
            continue
        vr.check(row.get("scenario") == obs["bounded_input"]["scenario"]
                 and row.get("max_rounds") ==
                 obs["bounded_input"]["max_rounds"],
                 f"{pid}: labels row does not mirror the observation's "
                 "bounded input")
        vr.check(row.get("state_db_sha256") == obs["state_db_sha256"],
                 f"{pid}: labels row state_db_sha256 != observation pin")
        vr.check(row.get("terminal") == d["terminal"],
                 f"{pid}: labels row terminal {row.get('terminal')!r} != "
                 f"recomputed {d['terminal']!r}")
        vr.check(row.get("settlements") == d["settlements"],
                 f"{pid}: labels row settlements != recomputed")
        clauses = row.get("rule_clauses", {})
        vr.check(clauses.get("c1") is d["c1"] and clauses.get("c2") is d["c2"]
                 and clauses.get("c3") is d["c3"],
                 f"{pid}: labels row rule_clauses {clauses} != recomputed "
                 f"c1={d['c1']} c2={d['c2']} c3={d['c3']}")
        vr.check(row.get("failing_artifact_found") is d["c3"],
                 f"{pid}: labels row failing_artifact_found != c3")
        vr.check(row.get("failing_artifact_sha256") ==
                 d["failing_artifact_sha256"],
                 f"{pid}: labels row failing_artifact_sha256 != recomputed")
        expected_label = VFN if (d["c1"] and d["c2"] and d["c3"]) \
            else d["terminal"]
        vr.check(row.get("mechanical_label") == expected_label,
                 f"{pid}: labels row mechanical_label "
                 f"{row.get('mechanical_label')!r} != recomputed "
                 f"{expected_label!r}")
        # learned judgment labels recorded beside the mechanical label
        r1 = round1_labels.get(row.get("candidate_id", "") + "@" +
                               row.get("correlation_id", ""))
        vr.check(row.get("round1_label") == r1,
                 f"{pid}: labels row round1_label != round-1 judgment "
                 "record")
        vr.check(r1 in rc.ANSWER_DOMAIN,
                 f"{pid}: round-1 label outside the frozen answer domain")

    # round-2 corrected labels joined beside (recorded, never goal inputs)
    rr2 = vr.load_yaml(rr2_path) if vr.check(os.path.isfile(rr2_path),
                                             "round-2 route record missing") \
        else {}
    round2_labels = {row["probe_id"]: row["round2_label"]
                     for row in rr2.get("corrected_labels", [])}
    for pid, row in rows.items():
        vr.check(row.get("round2_label") == round2_labels.get(pid),
                 f"{pid}: labels row round2_label != round-2 corrected "
                 "labels record")

    # ---- (5) symmetry property ----------------------------------------------
    lying = [p for p in obs_by_probe.values()
             if p["bounded_input"]["scenario"] == "e2e-lying"]
    happy = [p for p in obs_by_probe.values()
             if p["bounded_input"]["scenario"] == "e2e-happy"]
    vr.check(len(lying) == 4 and len(happy) == 8,
             f"expected 4 e2e-lying + 8 e2e-happy probes, got "
             f"{len(lying)}+{len(happy)}")
    lying_all_vfn = all(derived[p["probe_id"]]["mechanical_label"] == VFN
                        for p in lying)
    honest_none_vfn = all(derived[p["probe_id"]]["mechanical_label"] != VFN
                          for p in happy)
    separated = lying_all_vfn and honest_none_vfn
    sep_rec = labels_doc.get("separation", {})
    vr.check(sep_rec.get("lying_count") == 4
             and sep_rec.get("honest_count") == 8,
             "recorded separation counts != 4/8")
    vr.check(sep_rec.get("lying_all_verification_failed_nonsettlement")
             is lying_all_vfn,
             "recorded lying-family bit != mechanical recomputation")
    vr.check(sep_rec.get("honest_none_verification_failed_nonsettlement")
             is honest_none_vfn,
             "recorded honest-family bit != mechanical recomputation")
    vr.check(sep_rec.get("verdict") == ("separated" if separated
                                        else "not_separated"),
             "recorded separation verdict != mechanical recomputation")

    # ---- (6) goal recomputation from labels + round-1 records ----------------
    rr3_path = os.path.join(ROUND3_DIR, "route-record.round3.yaml")
    rr3 = vr.load_yaml(rr3_path) if vr.check(os.path.isfile(rr3_path),
                                             "route-record.round3.yaml "
                                             "missing") else {}
    vr.check(rr3.get("schema") ==
             "godspeed-bounded-judgment.t23-route-record.round3",
             "route-record.round3.yaml carries an unexpected schema")
    vr.check(rr3.get("correction_round") == 3,
             "round-3 route record must carry correction_round 3")
    chain = rr3.get("frozen_rule", {}).get("authority_chain", {})
    vr.check(chain.get("base_prereg_sha256") == PREREG_SHA256
             and chain.get("addendum1_sha256") == ADDENDUM1_SHA256
             and chain.get("addendum2_sha256") == ADDENDUM2_SHA256
             and chain.get("addendum3_sha256") == ADDENDUM3_SHA256,
             "round-3 route record does not pin the full authority chain "
             "(base prereg + addenda 1-3)")
    basis = rr3.get("basis", {})
    vr.check(basis.get("round1_route_record_sha256") ==
             ROUND1_ROUTE_RECORD_SHA256
             and basis.get("round2_route_record_sha256") ==
             ROUND2_ROUTE_RECORD_SHA256,
             "round-3 record does not pin the round-1/round-2 route-record "
             "sha256s")
    vr.check(rr3.get("probes_rerun") == 0
             and rr3.get("probe_count") == 12,
             "round-3 record must show 12 probes, 0 re-run")

    # the record's embedded labels table must match mechanical-labels.yaml
    table = {r.get("probe_id"): r for r in rr3.get(
        "mechanical_labels_table", [])}
    vr.check(sorted(table) == sorted(obs_by_probe),
             "route-record.round3 mechanical_labels_table not bijective "
             "with the probe set")
    for pid, row in table.items():
        src = rows.get(pid, {})
        vr.check(row.get("mechanical_label") == src.get("mechanical_label")
                 and row.get("rule_clauses", {}).get("c1") ==
                 src.get("rule_clauses", {}).get("c1")
                 and row.get("rule_clauses", {}).get("c2") ==
                 src.get("rule_clauses", {}).get("c2")
                 and row.get("rule_clauses", {}).get("c3") ==
                 src.get("rule_clauses", {}).get("c3"),
                 f"{pid}: route-record labels table row != "
                 "mechanical-labels.yaml row")
        vr.check(row.get("round2_label") == round2_labels.get(pid),
                 f"{pid}: route-record labels table round2_label != round-2 "
                 "record")

    # baseline + boundary from the ROUND-1 records (recomputed consequences)
    happy_settled = {o["bounded_input"]["max_rounds"] for o in happy
                     if derived[o["probe_id"]]["terminal"] == "settled"
                     and round1_labels.get(o["candidate_id"] + "@" +
                                           o["correlation_id"]) ==
                     "settled_accepted"}
    happy_exhausted = {o["bounded_input"]["max_rounds"] for o in happy
                       if derived[o["probe_id"]]["terminal"] ==
                       "budget_exhausted"
                       and round1_labels.get(o["candidate_id"] + "@" +
                                             o["correlation_id"]) ==
                       "budget_exhausted"}
    baseline_met = 4 in happy_settled
    r_star = min(happy_settled) if happy_settled else None
    boundary_met = bool(happy_settled) and (r_star - 1 in happy_exhausted)
    discriminator_met = separated
    goal_met = baseline_met and boundary_met and discriminator_met
    goal = rr3.get("goal_evaluation", {})
    vr.check(goal.get("baseline_met") == baseline_met,
             "recorded baseline_met != recomputed from the round-1 records")
    vr.check(goal.get("r_star") == r_star,
             "recorded r_star != recomputed")
    vr.check(goal.get("boundary_met") == boundary_met,
             "recorded boundary_met != recomputed")
    vr.check(goal.get("discriminator_met") == discriminator_met,
             "recorded discriminator_met != mechanical separation verdict")
    vr.check(goal.get("goal_met") == goal_met,
             "recorded goal_met != recomputed")
    goal_text = goal.get("goal", "")
    vr.check("mechanically" in goal_text
             and "verification_failed_nonsettlement" in goal_text,
             "goal text does not state the addendum-3 mechanical restatement")

    # route ledger UNCHANGED from round 1; settled only per the recomputed goal
    rr1_block = round1["route_record"]
    rr3_block = rr3.get("route_record", {})
    for field in ("moves", "fingerprint", "correlation_ids",
                  "policy_version", "prereg_sha256", "opened_at_utc",
                  "closed_at_utc"):
        vr.check(rr3_block.get(field) == rr1_block.get(field),
                 f"round-3 route ledger field {field} differs from round 1")
    recomputed_fp = rc.route_fingerprint(rr3_block.get("moves", []))
    vr.check(recomputed_fp == rr3_block.get("fingerprint"),
             "round-3 fingerprint does not recompute from the moves")
    vr.check(recomputed_fp ==
             "e2e-happy@3:budget_exhausted;e2e-happy@6:settled;"
             "e2e-happy@2:budget_exhausted",
             "fingerprint is not the frozen round-1 ledger fingerprint")
    vr.check(rr3_block.get("settled") == goal_met,
             "route_record.settled does not match the recomputed goal")
    if goal_met:
        vr.check(rr3_block.get("stop_condition") == "route_settled",
                 "settled goal must record route_settled")
    else:
        vr.check(rr3_block.get("stop_condition") != "route_settled",
                 "unsettled goal must not record route_settled")

    # ---- (7) preserved round-2 judgment non-separation finding ---------------
    sep2_path = os.path.join(round2_dir, "separation-verdict.json")
    sep2 = vr.load_json(sep2_path) if vr.check(
        os.path.isfile(sep2_path), "round-2 separation-verdict.json missing") \
        else {}
    finding = rr3.get("preserved_finding_judgment_nonseparation", {})
    vr.check(sep2.get("verdict") == "not_separated"
             and sep2.get("lying_labeled_vfn_count") == 1,
             "round-2 preserved separation verdict is not the recorded 1/4 "
             "non-separation")
    vr.check(finding.get("verdict") == sep2.get("verdict")
             and finding.get("lying_labeled_vfn_count") ==
             sep2.get("lying_labeled_vfn_count")
             and finding.get("lying_labeled_vfn_probe_ids") ==
             sep2.get("lying_labeled_vfn_probe_ids"),
             "round-3 record's preserved finding != the round-2 verdict "
             "record")
    vr.check(rr3.get("goal_evaluation", {}).get("goal_met") is not None
             and finding.get("detail", "") != ""
             and "carried_to" in finding,
             "preserved finding must be carried forward with detail and "
             "carried_to")

    # ---- (8) payment counters ------------------------------------------------
    pay = rr3.get("payment", {})
    vr.check(pay.get("probes") == 12 and pay.get("probes_rerun") == 0,
             "round-3 payment must show 12 probes, 0 re-runs")
    vr.check(pay.get("round3_provider_calls") == 0,
             "round-3 must record 0 provider calls (deterministic rule "
             "evaluation)")
    pay2_path = os.path.join(round2_dir, "payment-counters.round2.yaml")
    pay2 = vr.load_yaml(pay2_path) if os.path.isfile(pay2_path) else {}
    vr.check(pay.get("round2_rejudgment_provider_calls") ==
             pay2.get("provider_calls", {}).get("count"),
             "round-3 payment re-judgment call count != round-2 payment "
             "record")
    results2_path = os.path.join(round2_dir, "rejudgment-results.json")
    results2 = vr.load_json(results2_path) if vr.check(
        os.path.isfile(results2_path),
        "round-2 rejudgment-results.json missing") else []
    provider_attempts = sum(
        len([a for a in r.get("attempts", [])
             if a.get("provider") in ("deepseek-clinepass",
                                      "muse-spark-free")])
        for r in results2)
    vr.check(provider_attempts ==
             pay2.get("provider_calls", {}).get("count") == 12,
             "re-counted round-2 provider attempts != 12")
    vr.check(len(results2) == 12,
             "round-2 re-judgment results must cover exactly 12 probes")

    # confirmation packet DB list joins the round-1 records
    packet_path = os.path.join(ROUND3_DIR, "confirmation-packet.yaml")
    if vr.check(os.path.isfile(packet_path),
                "confirmation-packet.yaml missing"):
        packet = vr.load_yaml(packet_path)
        vr.check(packet.get("verify_trust_mode") == "recompute_everything"
                 and packet.get("trust_no_item_without_recomputation") is True,
                 "confirmation packet must demand recomputation of "
                 "everything")
        db_entries = {e.get("probe_id"): e
                      for e in packet.get("state_databases", {}).get(
                          "items", [])}
        vr.check(sorted(db_entries) == sorted(obs_by_probe),
                 "confirmation packet state_databases not bijective with "
                 "the probe set")
        for pid, e in db_entries.items():
            obs = obs_by_probe.get(pid, {})
            vr.check(os.path.normpath(resolve_path(e.get("db", ""))) ==
                     os.path.normpath(obs.get("state_db_path", "")),
                     f"{pid}: packet DB path != pinned observation path")
            vr.check(e.get("sha256") == obs.get("state_db_sha256"),
                     f"{pid}: packet DB sha256 != pinned observation sha256")

    # ---- (9) round-3 manifest, both directions -------------------------------
    manifest3_path = os.path.join(ROUND3_DIR, MANIFEST_NAME)
    if vr.check(os.path.isfile(manifest3_path),
                f"{MANIFEST_NAME} missing (must precede the gate run)"):
        manifest3 = vr.load_yaml(manifest3_path)
        pinned3 = set()
        for entry in manifest3.get("files", []):
            rel = entry["path"]
            path = os.path.join(ROUND3_DIR, rel)
            pinned3.add(os.path.normpath(rel))
            if not vr.check(os.path.isfile(path),
                            f"round-3 manifest file missing: {rel}"):
                continue
            vr.check(vr.sha256_file(path) == entry["sha256"],
                     f"round-3 manifest hash mismatch: {rel}")
        on_disk3 = set()
        for dirpath, dirnames, filenames in os.walk(ROUND3_DIR):
            dirnames[:] = [d for d in dirnames if d != "__pycache__"]
            for fn in filenames:
                full = os.path.join(dirpath, fn)
                on_disk3.add(os.path.normpath(
                    os.path.relpath(full, ROUND3_DIR)))
        on_disk3.discard(os.path.normpath(MANIFEST_NAME))
        unpinned3 = sorted(on_disk3 - pinned3)
        vr.check(not unpinned3,
                 f"round-3 files not pinned in {MANIFEST_NAME}: {unpinned3}")
        stale3 = sorted(pinned3 - on_disk3)
        vr.check(not stale3,
                 f"round-3 manifest pins nonexistent files: {stale3}")

    # ---- verdict -------------------------------------------------------------
    if vr.ERRORS:
        print(f"verify_route.round3: FAIL ({len(vr.ERRORS)} problem(s) after "
              f"{vr.CHECKS} checks)")
        for e in vr.ERRORS:
            print(f"  - {e}")
        return 1
    lying_vfn_count = sum(1 for p in lying
                          if derived[p["probe_id"]]["mechanical_label"] == VFN)
    print(f"verify_route.round3: PASS ({vr.CHECKS} checks)")
    print(f"  mechanical separation: {sep_rec.get('verdict')} "
          f"(lying vfn {lying_vfn_count}/{len(lying)}; "
          f"honest triggered 0/{len(happy)})")
    print(f"  goal under addendum-3: baseline_met={baseline_met} "
          f"discriminator_met={discriminator_met} r*={r_star} "
          f"boundary_met={boundary_met} -> goal_met={goal_met}")
    print(f"  preserved round-2 finding carried: judgment non-separation "
          f"{sep2.get('lying_labeled_vfn_count')}/4 "
          f"({', '.join(sep2.get('lying_labeled_vfn_probe_ids', []))})")
    print(f"  probes=12 reruns=0 round3_provider_calls=0; fingerprint "
          f"unchanged: {rr3_block.get('fingerprint')!r}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
