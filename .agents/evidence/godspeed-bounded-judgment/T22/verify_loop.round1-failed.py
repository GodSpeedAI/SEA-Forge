#!/usr/bin/env python3
"""T22 gate: verify the loop evidence end-to-end (fail-closed).

Runs from the sea-rs repository root:
    python3 .agents/evidence/godspeed-bounded-judgment/T22/verify_loop.py

Checks (exit 0 only if ALL pass):
  (a) every iteration-1 record schema-validates through the frozen T21
      route_contracts (candidates, admissions, probe observations, judgments,
      selection record, route_state, route_record);
  (b) correlation joins are complete and bijective: each probe_observation
      joins exactly one candidate / admission / judgment; every candidate has
      exactly one admission; every admitted candidate has exactly one probe;
      the selection record joins the route record; route_state.probed and
      route_record.moves mirror the observations; probe state-database paths
      are structurally bound to their own candidate_id (the T5 correlation-
      swap detector) and the pinned state-database digests match the durable
      files on disk;
  (c) no record claims route settlement: route_record.settled is false, the
      stop condition is not route_settled, and the route goal is NOT met at
      depth 1 (a probe terminal `settled` is a probe consequence only);
  (d) authority teeth T1-authority-bypass, T2-model-override and
      T5-correlation-swap are present with expected rejections (MATCH);
      no attack candidate left a probe target dir or entered the route;
  (e) manifest hash checks fail closed for the T21 imports (frozen modules +
      prereg) and for T22's own evidence-manifest.yml (both directions:
      every entry matches disk, every T22 .py/.yaml on disk is pinned);
  (f) payment counters are present and consistent with the observations.

With --records-dir PATH --records-only, runs only the records-level checks
(a),(b),(c) against an alternate records directory (used by the T5 attack).
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True  # no __pycache__ debris in evidence dirs

import hashlib
import json
import os

import yaml

T22_DIR = os.path.dirname(os.path.abspath(__file__))
BRANCH_DIR = os.path.dirname(T22_DIR)
T21_DIR = os.path.join(BRANCH_DIR, "T21")
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(BRANCH_DIR)))

sys.path.insert(0, T21_DIR)

import route_contracts as rc  # noqa: E402  (frozen T21 module, imported unchanged)

PREREG_SHA256 = "1fb55d9c56d87d94adc4850bc5ca713db3e600297fbdc997accfab54c98ea8b5"
DATA_ROOT = os.path.join(
    os.path.expanduser("~"), ".local", "share", "godspeed-route-discovery", "T22"
)
# The gate verifies the corrected ROUND-2 advance; the failed round-1 records
# remain present under iteration-1/ and can be checked with --records-dir.
DEFAULT_RECORDS_DIR = os.path.join(T22_DIR, "round-2")
DB_NAME = "gauntlet-state.db"
EMPTY_SHA256 = hashlib.sha256(b"").hexdigest()
POLICY_VERSION = "selection-policy-v1"

TYPED_RECORD_FILES = [
    ("candidates.json", "candidate_move"),
    ("admission-decisions.json", "admission_decision"),
    ("probe-observations.json", "probe_observation"),
    ("judgment-results.json", "judgment_result"),
    ("selection-record.json", "selection_record"),
]
REQUIRED_TEETH = ("T1-authority-bypass", "T2-model-override", "T5-correlation-swap")


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def load_json(path):
    with open(path) as f:
        return json.load(f)


def load_yaml(path):
    with open(path) as f:
        return yaml.safe_load(f)


def load_records(records_dir):
    data = {"records_dir": records_dir}
    for name, _rtype in TYPED_RECORD_FILES:
        data[name] = load_json(os.path.join(records_dir, name))
    route_doc = load_yaml(os.path.join(records_dir, "route-record.yaml"))
    data["route_state"] = route_doc["route_state"]
    data["route_record"] = route_doc["route_record"]
    # the durable probe-data root this round's records pin (round-2 uses a
    # round-namespaced root; round-1 used the task root)
    data["probe_data_root"] = route_doc.get("probe_data_root", DATA_ROOT)
    return data


# ---------------------------------------------------------------------------
# (a) schemas
# ---------------------------------------------------------------------------

def check_schemas(rec):
    errors = []
    for name, rtype in TYPED_RECORD_FILES:
        for i, record in enumerate(rec[name]):
            err = rc.validate(rtype, record)
            if err is not None:
                errors.append(f"schema {name}[{i}]: {err}")
    err = rc.validate("route_state", rec["route_state"])
    if err is not None:
        errors.append(f"schema route_state: {err}")
    err = rc.validate("route_record", rec["route_record"])
    if err is not None:
        errors.append(f"schema route_record: {err}")
    return errors


# ---------------------------------------------------------------------------
# (b) correlation joins (complete and bijective) + evidence pins
# ---------------------------------------------------------------------------

def _exactly_one(items, key, value):
    return sum(1 for x in items if x[key] == value)


def check_joins(rec):
    errors = []
    candidates = rec["candidates.json"]
    admissions = rec["admission-decisions.json"]
    observations = rec["probe-observations.json"]
    judgments = rec["judgment-results.json"]
    selection = rec["selection-record.json"]
    route_state = rec["route_state"]
    route_record = rec["route_record"]

    corr_ids = {c["correlation_id"] for c in candidates}
    if len(corr_ids) != 1:
        errors.append(f"candidates do not share one correlation id: {sorted(corr_ids)}")
    corr = corr_ids.pop() if corr_ids else ""

    # candidate <-> admission: bijective, one correlation id
    cand_ids = [c["candidate_id"] for c in candidates]
    if len(set(cand_ids)) != len(cand_ids):
        errors.append("duplicate candidate_id among candidates")
    for c in candidates:
        n = _exactly_one(admissions, "candidate_id", c["candidate_id"])
        if n != 1:
            errors.append(f"candidate {c['candidate_id']}: {n} admission decisions "
                          "(expected exactly 1)")
    for a in admissions:
        if _exactly_one(candidates, "candidate_id", a["candidate_id"]) != 1:
            errors.append(f"admission for unknown candidate {a['candidate_id']}")
        if a["correlation_id"] != corr:
            errors.append(f"admission {a['candidate_id']}: correlation id "
                          f"{a['correlation_id']!r} != {corr!r}")

    admitted = {c["candidate_id"] for c, a in
                zip(candidates, admissions)  # zip is safe: bijective checked above
                if a["classification"] == "admitted"}
    for a in admissions:
        if a["classification"] == "admitted" and a["candidate_id"] not in cand_ids:
            errors.append(f"admitted candidate {a['candidate_id']} not in candidates")

    # admitted candidate <-> probe observation: bijective
    for cid in sorted(admitted):
        n = _exactly_one(observations, "candidate_id", cid)
        if n != 1:
            errors.append(f"admitted candidate {cid}: {n} probe observations "
                          "(expected exactly 1)")
    for o in observations:
        cid = o["candidate_id"]
        if cid not in admitted:
            errors.append(f"probe observation for non-admitted candidate {cid}")
        c = next((c for c in candidates if c["candidate_id"] == cid), None)
        if c is not None:
            if o["correlation_id"] != c["correlation_id"]:
                errors.append(f"probe {o['probe_id']}: correlation id does not join "
                              "its candidate")
            if o["bounded_input"] != c["bounded_input"]:
                errors.append(f"probe {o['probe_id']}: bounded_input does not join "
                              "its candidate")
        # structural binding of evidence to the candidate (T5 detector) + digest pin
        expected_path = os.path.join(rec["probe_data_root"], "runs", cid, "state",
                                     DB_NAME)
        if o["state_db_path"] != expected_path:
            errors.append(
                f"probe {o['probe_id']} (candidate {cid}): state_db_path "
                f"{o['state_db_path']!r} is not the frozen-mechanism state database "
                f"of its own candidate ({expected_path!r})")
        if os.path.isfile(o["state_db_path"]):
            digest = sha256_file(o["state_db_path"])
            if digest != o["state_db_sha256"]:
                errors.append(f"probe {o['probe_id']}: state db digest mismatch "
                              f"(disk {digest} != pinned {o['state_db_sha256']})")
        elif o["state_db_sha256"] != EMPTY_SHA256:
            errors.append(f"probe {o['probe_id']}: pinned state db is missing on disk")
        if o["terminal_class"] == "settled" and o["settlement_committed_count"] < 1:
            errors.append(f"probe {o['probe_id']}: settled terminal without a "
                          "settlement_committed event")

    # probe observation <-> judgment: bijective
    for o in observations:
        n = _exactly_one(judgments, "candidate_id", o["candidate_id"])
        if n != 1:
            errors.append(f"probe {o['probe_id']} (candidate {o['candidate_id']}): "
                          f"{n} judgment results (expected exactly 1)")
    for j in judgments:
        o = next((o for o in observations if o["candidate_id"] == j["candidate_id"]),
                 None)
        if o is None:
            errors.append(f"judgment {j['judgment_id']} joins no probe observation")
            continue
        if j["observation_refs"] != [o["probe_id"]]:
            errors.append(f"judgment {j['judgment_id']}: observation_refs "
                          f"{j['observation_refs']!r} != [probe {o['probe_id']!r}]")
        if j["correlation_id"] != o["correlation_id"]:
            errors.append(f"judgment {j['judgment_id']}: correlation id does not "
                          "join its probe observation")

    # selection joins the route record
    if selection["correlation_id"] != route_record["correlation_id"]:
        errors.append("selection correlation id does not join the route record")
    if selection["correlation_id"] not in route_record["correlation_ids"]:
        errors.append("selection correlation id absent from route_record.correlation_ids")
    if route_record["policy_version"] != POLICY_VERSION or \
            selection["policy_version"] != POLICY_VERSION:
        errors.append("policy version mismatch against the frozen selection-policy-v1")
    retained = selection["retained_candidate_id"]
    retained_moves = route_state["retained_moves"]
    if retained == "":
        if retained_moves:
            errors.append("route_state retains moves although the selection "
                          "retained nothing")
    else:
        if retained not in selection["eligible_candidate_ids"]:
            errors.append("retained candidate is not among the eligible ids")
        if len(retained_moves) != 1 or retained_moves[0]["candidate_id"] != retained:
            errors.append("route_state.retained_moves does not mirror the "
                          "selection's single retained candidate")
        else:
            m = retained_moves[0]
            c = next((c for c in candidates if c["candidate_id"] == retained), None)
            o = next((o for o in observations if o["candidate_id"] == retained), None)
            j = next((j for judgments_x in [judgments]
                      for j in judgments_x if j["candidate_id"] == retained), None)
            if c is not None and (m["scenario"] != c["bounded_input"]["scenario"]
                                  or m["max_rounds"] != c["bounded_input"]["max_rounds"]
                                  or m["expected_role"] != c["expected_role"]
                                  or m["correlation_id"] != c["correlation_id"]):
                errors.append("retained move does not join its candidate record")
            if o is not None and len(route_record["moves"]) == 1:
                mv = route_record["moves"][0]
                if (mv["scenario"] != o["bounded_input"]["scenario"]
                        or mv["max_rounds"] != o["bounded_input"]["max_rounds"]
                        or mv["terminal_class"] != o["terminal_class"]):
                    errors.append("route_record.moves does not join the retained "
                                  "candidate's probe observation")
            if j is not None and m["judgment_label"] != j["label"]:
                errors.append("retained move judgment_label does not join its "
                              "judgment result")

    # route_state.probed mirrors the observations bijectively
    if len(route_state["probed"]) != len(observations):
        errors.append("route_state.probed size != probe observations size")
    for entry in route_state["probed"]:
        o = next((o for o in observations
                  if o["candidate_id"] == entry["candidate_id"]), None)
        if o is None:
            errors.append(f"route_state.probed entry {entry['candidate_id']} joins "
                          "no probe observation")
            continue
        if (entry["scenario"] != o["bounded_input"]["scenario"]
                or entry["max_rounds"] != o["bounded_input"]["max_rounds"]
                or entry["terminal_class"] != o["terminal_class"]
                or entry["correlation_id"] != o["correlation_id"]):
            errors.append(f"route_state.probed entry {entry['candidate_id']} does "
                          "not join its probe observation")
        j = next((j for j in judgments if j["candidate_id"] == entry["candidate_id"]),
                 None)
        if j is not None and entry["judgment_label"] != j["label"]:
            errors.append(f"route_state.probed entry {entry['candidate_id']} "
                          "judgment_label does not join its judgment result")

    # budget accounting
    if route_state["probes_remaining"] != rc.MAX_PROBES_PER_ROUTE - len(observations):
        errors.append("probes_remaining does not equal the frozen budget minus the "
                      "probes actually run")

    # no attack candidate anywhere in the route evidence
    for kind, records in (("candidate", candidates), ("admission", admissions),
                          ("probe observation", observations),
                          ("judgment", judgments)):
        for r in records:
            cid = r.get("candidate_id", "")
            if cid.startswith("attack-"):
                errors.append(f"attack candidate {cid} present among {kind} records")
    for entry in route_state["retained_moves"] + route_state["probed"]:
        if entry["candidate_id"].startswith("attack-"):
            errors.append(f"attack candidate {entry['candidate_id']} present in "
                          "route_state")
    return errors


# ---------------------------------------------------------------------------
# (c) no route settlement at depth 1
# ---------------------------------------------------------------------------

def check_no_settlement(rec):
    errors = []
    route_state = rec["route_state"]
    route_record = rec["route_record"]
    if route_record["settled"] is not False:
        errors.append("route_record claims settlement")
    if route_record["stop_condition"] == "route_settled":
        errors.append("route_record stop condition claims route_settled")
    if len(route_state["retained_moves"]) > 1:
        errors.append("route depth exceeds 1 retained move for this task-scoped run")

    # independently recompute the frozen route goal from this route's own probes
    probed = route_state["probed"]
    baseline_met = any(p["scenario"] == "e2e-happy" and p["max_rounds"] == 8
                       and p["judgment_label"] == "settled_accepted" for p in probed)
    discriminator_met = any(p["scenario"] == "e2e-lying"
                            and p["judgment_label"] == "verification_failed_nonsettlement"
                            for p in probed)
    settled = {p["max_rounds"] for p in probed
               if p["scenario"] == "e2e-happy"
               and p["judgment_label"] == "settled_accepted"}
    exhausted = {p["max_rounds"] for p in probed
                 if p["scenario"] == "e2e-happy"
                 and p["judgment_label"] == "budget_exhausted"}
    boundary_met = bool(settled) and (min(settled) - 1) in exhausted
    if baseline_met and discriminator_met and boundary_met:
        errors.append("route goal IS met at depth 1; the task-scoped single advance "
                      "must not be recorded as route settlement")
    # a probe terminal `settled` is a probe consequence, never route settlement:
    # even when some probe settled, the route record must stay non-settled here.
    any_probe_settled = any(p["terminal_class"] == "settled"
                            for p in probed)
    if any_probe_settled and route_record["settled"]:
        errors.append("a probe consequence was converted into route settlement")
    return errors


# ---------------------------------------------------------------------------
# (d) teeth outcomes
# ---------------------------------------------------------------------------

def check_teeth():
    errors = []
    path = os.path.join(T22_DIR, "teeth", "attacks.yaml")
    if not os.path.isfile(path):
        return [f"teeth/attacks.yaml missing ({path})"]
    doc = load_yaml(path)
    teeth = doc.get("teeth", []) if isinstance(doc, dict) else []
    by_id = {t.get("id"): t for t in teeth if isinstance(t, dict)}
    for tid in REQUIRED_TEETH:
        t = by_id.get(tid)
        if t is None:
            errors.append(f"tooth {tid} missing from teeth/attacks.yaml")
            continue
        for field in ("construction", "observed", "expected_vs_observed"):
            if not str(t.get(field, "")).strip():
                errors.append(f"tooth {tid}: field {field!r} missing/empty")
        if "MATCH" not in str(t.get("expected_vs_observed", "")):
            errors.append(f"tooth {tid}: expected_vs_observed does not record a "
                          "MATCH between expected rejection and observed outcome")
    # no attack candidate left a probe target dir behind (any round's data root)
    leaked = []
    for root, dirs, _files in os.walk(DATA_ROOT):
        if os.path.basename(root) == "targets":
            leaked.extend(d for d in os.listdir(root) if d.startswith("attack-"))
    if leaked:
        errors.append(f"attack candidates left probe target dirs: {sorted(leaked)}")
    return errors


# ---------------------------------------------------------------------------
# (e) manifest hash checks (fail-closed, both directions)
# ---------------------------------------------------------------------------

def check_manifests():
    errors = []
    t21_manifest_path = os.path.join(T21_DIR, "evidence-manifest.yml")
    if not os.path.isfile(t21_manifest_path):
        return [f"T21 evidence manifest missing: {t21_manifest_path}"]
    t21 = load_yaml(t21_manifest_path)
    for entry in t21.get("files", []):
        path = os.path.join(T21_DIR, entry["path"])
        if not os.path.isfile(path):
            errors.append(f"T21 frozen file missing: {entry['path']}")
            continue
        if sha256_file(path) != entry["sha256"]:
            errors.append(f"T21 frozen file hash mismatch: {entry['path']}")
    prereg_path = os.path.join(REPO_ROOT, ".agents/preregistrations",
                               "godspeed-bounded-judgment-T21.prereg.yaml")
    if sha256_file(prereg_path) != PREREG_SHA256:
        errors.append("frozen T21 prereg hash mismatch")

    t22_manifest_path = os.path.join(T22_DIR, "evidence-manifest.yml")
    if not os.path.isfile(t22_manifest_path):
        return errors + [f"T22 evidence manifest missing: {t22_manifest_path}"]
    t22 = load_yaml(t22_manifest_path)
    pinned = set()
    for entry in t22.get("files", []):
        rel = entry["path"]
        path = os.path.join(T22_DIR, rel)
        pinned.add(os.path.normpath(path))
        if not os.path.isfile(path):
            errors.append(f"T22 manifest entry missing on disk: {rel}")
            continue
        if sha256_file(path) != entry["sha256"]:
            errors.append(f"T22 manifest hash mismatch: {rel}")
    on_disk = set()
    for root, dirs, files in os.walk(T22_DIR):
        dirs[:] = [d for d in dirs if d != "__pycache__"]
        for name in files:
            if name.endswith((".py", ".yaml")):
                on_disk.add(os.path.normpath(os.path.join(root, name)))
    on_disk.discard(os.path.normpath(t22_manifest_path))
    unpinned = sorted(on_disk - pinned)
    if unpinned:
        errors.append("T22 manifest does not pin every .py/.yaml on disk: "
                      + ", ".join(os.path.relpath(p, T22_DIR) for p in unpinned))
    return errors


# ---------------------------------------------------------------------------
# (f) payment counters
# ---------------------------------------------------------------------------

def check_payment(rec):
    errors = []
    path = os.path.join(rec["records_dir"], "payment-counters.yaml")
    if not os.path.isfile(path):
        return [f"payment-counters.yaml missing ({path})"]
    payment = load_yaml(path)
    for key in ("wall_clock_seconds", "provider_calls", "probes_run",
                "tool_executions", "failed_executions", "tokens_and_cost"):
        if key not in payment:
            errors.append(f"payment counters missing key {key!r}")
    if "wall_clock_seconds" in payment and not (
            isinstance(payment["wall_clock_seconds"], (int, float))
            and payment["wall_clock_seconds"] >= 0):
        errors.append("wall_clock_seconds is not a nonnegative number")
    calls = payment.get("provider_calls", {})
    if isinstance(calls, dict):
        listed = calls.get("by_identity", [])
        if calls.get("count") != len(listed):
            errors.append("provider_calls.count does not match the recorded call list")
        for i, c in enumerate(listed):
            for field in ("provider", "model", "outcome", "purpose"):
                if not str(c.get(field, "")).strip():
                    errors.append(f"provider call {i + 1}: field {field!r} missing")
    else:
        errors.append("provider_calls is not a mapping with a call list")
    n_obs = len(rec["probe-observations.json"])
    if payment.get("probes_run") != n_obs:
        errors.append(f"probes_run {payment.get('probes_run')!r} != "
                      f"{n_obs} recorded probe observations")
    failed = sum(1 for o in rec["probe-observations.json"]
                 if o["terminal_class"] == "run_error")
    if payment.get("failed_executions") != failed:
        errors.append(f"failed_executions {payment.get('failed_executions')!r} != "
                      f"{failed} run_error observations")
    if payment.get("tool_executions") != n_obs:
        errors.append("tool_executions does not match the recorded probe count")
    return errors


# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------

def check_records_dir(records_dir):
    rec = load_records(records_dir)
    errors = []
    errors += check_schemas(rec)
    errors += check_joins(rec)
    errors += check_no_settlement(rec)
    return rec, errors


def main(argv):
    records_only = "--records-only" in argv
    records_dir = DEFAULT_RECORDS_DIR
    if "--records-dir" in argv:
        records_dir = os.path.abspath(argv[argv.index("--records-dir") + 1])

    rec, errors = check_records_dir(records_dir)

    if not records_only:
        errors += check_teeth()
        errors += check_manifests()
        errors += check_payment(rec)

    label = (f"records[{records_dir}]"
             if records_only else "T22 loop evidence")
    if errors:
        print(f"verify_loop: FAIL ({label})")
        for e in errors:
            print(f"  - {e}")
        return 1
    n_probes = len(rec["probe-observations.json"])
    print(f"verify_loop: PASS ({label}) - schemas, bijective correlation joins, "
          f"evidence pins, no-settlement-at-depth-1"
          + ("" if records_only else
             f", teeth T1/T2/T5 rejected-as-expected, manifest hash checks "
             f"(T21 imports + T22 own), payment counters; probes={n_probes}"))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
