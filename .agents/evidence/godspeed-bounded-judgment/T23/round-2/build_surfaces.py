#!/usr/bin/env python3
"""T23 round-2 surface builder (addendum-2 corrected judgment surface).

Re-judgment input construction ONLY: reads the round-1 records and the
preserved per-probe state databases / evidence artifact stores READ-ONLY and
renders one corrected judgment surface per probe observation of route
t23-route-001. No probe is re-run; no round-1 file is modified.

Corrected surface per addendum-2 minimum_sufficient_input: typed terminal;
settlement_committed count and unit-level progression (which unit_ids
settled); the unit_dispatch retry pattern (same-unit repeats without
settlement); the content of the produced evidence artifacts fetched by their
sha256 pins from the run's own artifact dir (GAUNTLET_ARTIFACT_DIR under the
recorded probe data root); and timing. Never the route goal, never round-1
labels, never any narrative.

The round-1 state-database sha256 pin is verified FIRST for every probe; a
mismatched probe is skipped and the mismatch recorded (it would then receive
no provider call and fall through to the declared insufficiency label).

This script is frozen before its first execution and never overwritten.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import hashlib
import json
import os

T23_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, T23_DIR)

import provenance as prov  # noqa: E402  (frozen round-1 module, unmodified)

ROUND2_DIR = os.path.dirname(os.path.abspath(__file__))
ROUTE_DIR = os.path.join(T23_DIR, "route")
SURFACES_DIR = os.path.join(ROUND2_DIR, "surfaces")

ADDENDUM2_SHA256 = (
    "087887c27cc50c6c989fcf7bb214e77aca62ea74d7e32a868ef0f7dd211e4c5a")

SURFACE_SCHEMA = "godspeed-bounded-judgment.t23-corrected-judgment-surface"


def load_json(path: str):
    with open(path) as f:
        return json.load(f)


def load_yaml(path: str):
    import yaml
    with open(path) as f:
        return yaml.safe_load(f)


def read_events(db_path: str) -> list[dict]:
    """Read-only event-log read (kind column = event kind, payload column =
    payload JSON, at column = integer event clock)."""
    import sqlite3
    conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    try:
        rows = conn.execute(
            "select kind, payload, at from event_log order by offset"
        ).fetchall()
    finally:
        conn.close()
    events = []
    for kind, payload, at in rows:
        try:
            body = json.loads(payload)
        except json.JSONDecodeError:
            body = {}
        events.append({"kind": kind, "body": body, "at": at})
    return events


def artifact_path(artifact_dir: str, sha256: str) -> str | None:
    """Content-addressed evidence object under the run's own artifact dir
    (GAUNTLET_ARTIFACT_DIR): objects/<aa>/<bb>/<sha256>."""
    candidate = os.path.join(
        artifact_dir, "objects", sha256[:2], sha256[2:4], sha256)
    return candidate if os.path.isfile(candidate) else None


def fetch_artifact(artifact_dir: str, sha256: str) -> dict:
    """Fetch one artifact by its sha256 pin, verify the bytes hash to the
    pin, and return locator/kind/content. Fails closed on any mismatch."""
    path = artifact_path(artifact_dir, sha256)
    if path is None:
        raise SystemExit(
            f"artifact {sha256} not found under {artifact_dir} (pin broken)")
    digest = prov.sha256_file(path)
    if digest != sha256:
        raise SystemExit(
            f"artifact bytes at {path} hash to {digest}, pinned {sha256}")
    kind_path = path + ".kind"
    kind = ""
    if os.path.isfile(kind_path):
        with open(kind_path) as f:
            kind = f.read().strip()
    with open(path, encoding="utf-8", errors="replace") as f:
        content = f.read()
    return {"sha256": sha256, "kind": kind, "content": content,
            "artifact_path": path}


def build_surface(obs: dict, run_meta_entry: dict | None) -> dict:
    """Build one corrected judgment surface record (surface + provenance)."""
    probe_id = obs["probe_id"]
    db_path = obs["state_db_path"]
    artifact_dir = os.path.join(os.path.dirname(os.path.dirname(db_path)),
                                "evidence")

    # 1. verify the round-1 state-database pin FIRST (skip+record on mismatch)
    recheck = prov.recompute(obs)
    if not recheck["ok"]:
        return {"schema": SURFACE_SCHEMA, "probe_id": probe_id,
                "correction_round": 2, "addendum2_sha256": ADDENDUM2_SHA256,
                "status": "skipped_db_pin_mismatch",
                "reasons": recheck["reasons"], "surface": None,
                "provenance": {"state_db_path": db_path,
                               "state_db_sha256_actual":
                                   recheck["db_sha256"]}}

    events = read_events(db_path)

    # 2. typed terminal + settlement count + unit-level progression
    terminal_states = [e["body"].get("state") for e in events
                       if e["kind"] == "run_terminal"]
    settlements = [e["body"] for e in events
                   if e["kind"] == "settlement_committed"]
    progression = [
        {"order": i + 1, "unit_id": s.get("unit_id"),
         "verification_ref": s.get("verification"),
         "evidence_ref": s.get("evidence")}
        for i, s in enumerate(settlements)]

    # 3. unit_dispatch retry pattern (same-unit repeats without settlement)
    dispatch_seq = [e["body"].get("unit_id") for e in events
                    if e["kind"] == "unit_dispatch"]
    per_unit = []
    for unit in dict.fromkeys(dispatch_seq):  # order-preserving distinct
        per_unit.append({
            "unit_id": unit,
            "dispatches": dispatch_seq.count(unit),
            "settlements": sum(1 for s in settlements
                               if s.get("unit_id") == unit)})

    # 4. produced evidence artifacts by sha256 pin from the run's artifact dir
    evidence_refs = []
    for e in events:
        if e["kind"] == "evidence_produced":
            ref = e["body"].get("evidence", "")
            if ref.startswith("sha256:"):
                evidence_refs.append(ref[7:])
        elif e["kind"] == "settlement_committed":
            ref = e["body"].get("evidence", "")
            if ref.startswith("sha256:"):
                evidence_refs.append(ref[7:])
    artifacts = [fetch_artifact(artifact_dir, sha)
                 for sha in dict.fromkeys(evidence_refs)]

    # 5. timing (preserved round-1 observation timestamps + sidecar wall time
    #    + the run's own event-clock/checkpoint shape)
    checkpoints = [e["body"].get("iteration_no") for e in events
                   if e["kind"] == "checkpoint_written"]
    timing = {
        "started_at_utc": obs.get("started_at_utc"),
        "finished_at_utc": obs.get("finished_at_utc"),
        "wall_seconds": (run_meta_entry or {}).get("wall_seconds"),
        "checkpoint_iterations": checkpoints,
        "event_clock_first": events[0]["at"] if events else None,
        "event_clock_last": events[-1]["at"] if events else None,
    }

    surface = {
        "probe_id": probe_id,
        "candidate_id": obs.get("candidate_id"),
        "correlation_id": obs.get("correlation_id"),
        "operation": obs.get("operation"),
        "bounded_input": obs.get("bounded_input"),
        "mechanism": obs.get("mechanism"),
        "typed_terminal": {"terminal_class": obs.get("terminal_class"),
                           "run_terminal_states": terminal_states},
        "settlement_committed_count": len(settlements),
        "settlement_progression": progression,
        "unit_dispatch_pattern": {"dispatch_sequence": dispatch_seq,
                                  "per_unit": per_unit},
        "evidence_artifacts": [
            {"locator": None, "sha256": a["sha256"], "kind": a["kind"],
             "content": a["content"]} for a in artifacts],
        "timing": timing,
        "state_db_sha256": obs.get("state_db_sha256"),
    }
    # locator for each artifact, from its evidence_produced event (the pin is
    # the content digest; the locator is the run-relative produced path)
    locators = {}
    for e in events:
        if e["kind"] == "evidence_produced":
            ref = e["body"].get("evidence", "")
            if ref.startswith("sha256:"):
                locators.setdefault(ref[7:], e["body"].get("locator"))
    for a, art in zip(surface["evidence_artifacts"], artifacts):
        a["locator"] = locators.get(a["sha256"])

    return {"schema": SURFACE_SCHEMA, "probe_id": probe_id,
            "correction_round": 2, "addendum2_sha256": ADDENDUM2_SHA256,
            "status": "ok", "surface": surface,
            "provenance": {
                "state_db_path": db_path,
                "state_db_sha256_pinned": obs.get("state_db_sha256"),
                "state_db_sha256_actual": recheck["db_sha256"],
                "db_pin_verified": True,
                "artifact_dir": artifact_dir,
                "artifact_paths": {a["sha256"]: a["artifact_path"]
                                   for a in artifacts},
                "recompute_reasons": recheck["reasons"]}}


def main() -> int:
    os.makedirs(SURFACES_DIR, exist_ok=True)
    records = []
    run_meta_by_probe: dict[str, dict] = {}
    for iteration in (1, 2, 3, 4):
        idir = os.path.join(ROUTE_DIR, f"iteration-{iteration}")
        observations = load_json(os.path.join(idir, "probe-observations.json"))
        meta = load_yaml(os.path.join(idir, "probe-run-meta.yaml"))
        for m in meta.get("probes", []):
            run_meta_by_probe[m["probe_id"]] = m
        for obs in observations:
            print(f"[surface] {obs['probe_id']} building ...", flush=True)
            record = build_surface(obs, run_meta_by_probe.get(obs["probe_id"]))
            if record["status"] != "ok":
                print(f"[surface] {obs['probe_id']} SKIPPED: "
                      f"{record['reasons']}", flush=True)
            records.append(record)
            out = os.path.join(SURFACES_DIR, f"{obs['probe_id']}.json")
            with open(out, "w") as f:
                json.dump(record, f, indent=1, sort_keys=True)
                f.write("\n")

    with open(os.path.join(SURFACES_DIR, "corrected-surfaces.json"),
              "w") as f:
        json.dump({"schema": SURFACE_SCHEMA, "correction_round": 2,
                   "addendum2_sha256": ADDENDUM2_SHA256,
                   "route_id": "t23-route-001",
                   "surfaces": [r["probe_id"] for r in records],
                   "records": records}, f, indent=1, sort_keys=True)
        f.write("\n")

    skipped = [r["probe_id"] for r in records if r["status"] != "ok"]
    print(f"[surface] built {len(records) - len(skipped)} surfaces, "
          f"skipped {len(skipped)}: {skipped}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
