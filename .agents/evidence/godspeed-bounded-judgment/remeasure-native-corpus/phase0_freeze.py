#!/usr/bin/env python3
"""Phase-0 second freeze stage for the native-corpus remeasurement.

Prereg: .agents/preregistrations/godspeed-bounded-judgment-native-corpus-remeasure.prereg.yaml
(frozen, sha256 8142c0a52537f9b600d4853afd4871d5ec04dc29acffba58fe67c7f7616d1f6e,
decision D-2026-09-19-RM-01).

Materializes corpus-manifest.yaml from the frozen branch records, re-hashes
every pinned state database (fail-closed: any mismatch STOPS the measurement),
derives mechanical truth by the frozen addendum-3 rule from the pinned DBs and
artifact objects (never from the records' claims), cross-checks the prereg
truth classes and frozen class counts, computes the manifest sha256, and
appends decision D-2026-09-19-RM-02 to the append-only decision log BEFORE any
provider call. Reads every frozen source read-only; the only writes are the
manifest file and the appended decision-log entry.

Frozen before its first execution; never overwritten. Corrections use a
round-2 filename.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import datetime
import hashlib
import json
import os
import re
import sqlite3

import yaml

BASE = "/home/sprime01/projects/sea-rs/.agents/evidence/godspeed-bounded-judgment"
OUT_DIR = os.path.join(BASE, "remeasure-native-corpus")
REPO = "/home/sprime01/projects/sea-rs"

PREREG_PATH = os.path.join(
    REPO, ".agents/preregistrations/"
    "godspeed-bounded-judgment-native-corpus-remeasure.prereg.yaml")
PREREG_SHA256 = (
    "8142c0a52537f9b600d4853afd4871d5ec04dc29acffba58fe67c7f7616d1f6e")
DECISIONS_PATH = os.path.join(BASE, "decisions.yml")
MANIFEST_PATH = os.path.join(OUT_DIR, "corpus-manifest.yaml")
DECISION_ID = "D-2026-09-19-RM-02"

T23_ROUTE = os.path.join(BASE, "T23", "route")
T24_DIR = os.path.join(BASE, "T24")
T25_DIR = os.path.join(BASE, "T25")

MECHANISM = "t15-frozen-case-execution"  # frozen T21 route_contracts.MECHANISM


def stop(msg: str):
    print(f"[phase0] STOP: {msg}", flush=True)
    sys.exit(2)


def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def load_yaml(path: str):
    with open(path) as f:
        return yaml.safe_load(f)


def load_json(path: str):
    with open(path) as f:
        return json.load(f)


def jbody(payload) -> dict:
    try:
        d = json.loads(payload)
    except json.JSONDecodeError:
        return {}
    return d if isinstance(d, dict) else {}


def parse_probe_spec(probe: str):
    """'e2e-happy@4 (route baseline)' -> ('e2e-happy', 4)."""
    head = probe.split("(", 1)[0].strip()
    m = re.fullmatch(r"([A-Za-z0-9_-]+)@(\d+)", head)
    if not m:
        stop(f"unparseable probe spec {probe!r}")
    return m.group(1), int(m.group(2))


def read_db_facts(db_path: str):
    """Read-only DB read: terminal states, settlement count, event list."""
    if not os.path.isfile(db_path):
        stop(f"no state database at pinned path: {db_path}")
    conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    try:
        ev = [(k, jbody(p)) for k, p in conn.execute(
            "select kind, payload from event_log order by offset")]
    finally:
        conn.close()
    states = [b.get("state") for k, b in ev if k == "run_terminal"]
    settlements = sum(1 for k, _ in ev if k == "settlement_committed")
    return ev, states, settlements


def failing_artifact(db_path: str, ev) -> str:
    """Addendum-3 clause (3): at least one evidence_produced artifact of the
    run, hash-verified at its pinned sha256, has content recording a failing
    verification (failures > 0). Returns the failing artifact sha256 or ''."""
    refs = []
    for k, b in ev:
        if k == "evidence_produced":
            r = b.get("evidence", "")
            if r.startswith("sha256:"):
                r = r[7:]
                if r not in refs:
                    refs.append(r)
    art_dir = os.path.join(
        os.path.dirname(os.path.dirname(db_path)), "evidence")
    for ref in refs:
        obj = os.path.join(art_dir, "objects", ref[:2], ref[2:4], ref)
        if not os.path.isfile(obj):
            stop(f"artifact object missing for pin {ref} under {art_dir}")
        if sha256_file(obj) != ref:
            stop(f"artifact bytes at {obj} do not hash to pin {ref}")
        try:
            with open(obj, encoding="utf-8", errors="replace") as f:
                content = json.load(f)
        except json.JSONDecodeError:
            continue
        failures = content.get("failures") if isinstance(content, dict) else None
        if (isinstance(failures, (int, float))
                and not isinstance(failures, bool) and failures > 0):
            return ref
    return ""


def classify_terminal(raw_state) -> str:
    """Frozen T22/T23 typed-terminal mapping."""
    if raw_state == "settled":
        return "settled"
    if raw_state == "budget_exhausted":
        return "budget_exhausted"
    if isinstance(raw_state, str) and raw_state != "":
        return "other-nonsettlement"
    return "run_error"


def derive_truth(terminal_state, settlements, failing_ref):
    """Frozen mechanical rule (prereg truth_derivation + addendum-3):
    settled terminal -> settled_accepted; bounded-non-settlement terminal
    AND 0 settlements AND >=1 sha256-verified failing artifact ->
    verification_failed_nonsettlement; any other bounded non-settlement ->
    budget_exhausted; run error -> execution_failed (none expected)."""
    terminal = classify_terminal(terminal_state)
    c1 = terminal not in ("settled", "run_error")
    c2 = settlements == 0
    c3 = bool(failing_ref)
    if terminal == "settled":
        return "settled_accepted", {"c1": c1, "c2": c2, "c3": c3}
    if terminal == "run_error":
        return "execution_failed", {"c1": c1, "c2": c2, "c3": c3}
    if c1 and c2 and c3:
        return "verification_failed_nonsettlement", {"c1": c1, "c2": c2,
                                                     "c3": c3}
    return "budget_exhausted", {"c1": c1, "c2": c2, "c3": c3}


def record_pin(path: str) -> dict:
    if not os.path.isfile(path):
        stop(f"frozen source record missing: {path}")
    return {"path": os.path.relpath(path, REPO), "sha256": sha256_file(path)}


def resolve_row(row, prereg_sha):
    """Resolve one prereg corpus row against the frozen branch records."""
    rid, source, probe_spec, prereg_truth = (
        row["id"], row["source"], row["probe"], row["truth_class"])
    scenario, max_rounds = parse_probe_spec(probe_spec)
    want = {"scenario": scenario, "max_rounds": max_rounds}
    obs = None
    run_meta = None
    source_records = []
    identity_source = "frozen probe-observations.json entry"

    if source.startswith("t23-iter"):
        it = source.split("t23-iter", 1)[1]
        idir = os.path.join(T23_ROUTE, f"iteration-{it}")
        observations = load_json(os.path.join(idir, "probe-observations.json"))
        source_records.append(record_pin(
            os.path.join(idir, "probe-observations.json")))
        meta = load_yaml(os.path.join(idir, "probe-run-meta.yaml"))
        source_records.append(record_pin(os.path.join(idir,
                                                      "probe-run-meta.yaml")))
        matches = [o for o in observations
                   if o.get("bounded_input") == want]
        if len(matches) != 1:
            stop(f"{rid}: expected exactly 1 observation for {want} in "
                 f"iteration-{it}, found {len(matches)}")
        obs = matches[0]
        metas = [m for m in meta.get("probes", [])
                 if m.get("probe_id") == obs["probe_id"]]
        if len(metas) != 1:
            stop(f"{rid}: expected exactly 1 run-meta entry for "
                 f"{obs['probe_id']}")
        run_meta = metas[0]

    elif source == "t24-replay":
        rr_path = os.path.join(T24_DIR, "replay-record.yaml")
        source_records.append(record_pin(rr_path))
        ac_path = os.path.join(T24_DIR, "accepted-case.yaml")
        source_records.append(record_pin(ac_path))
        rr = load_yaml(rr_path)
        obs = rr["observation"]
        if obs.get("bounded_input") != want:
            stop(f"{rid}: replay observation bounded_input != prereg probe "
                 f"spec {want}")
        acc = load_yaml(ac_path)
        if (acc["replay"]["state_db_path"] != obs["state_db_path"]
                or acc["replay"]["state_db_sha256"] != obs["state_db_sha256"]):
            stop(f"{rid}: accepted-case replay pin disagrees with "
                 f"replay-record observation")
        run_meta = {"wall_seconds": rr["run"]["wall_seconds"]}
        identity_source = "T24/replay-record.yaml observation block"

    elif source == "t24-tooth-t7":
        tt_path = os.path.join(T24_DIR, "teeth.yaml")
        source_records.append(record_pin(tt_path))
        teeth = load_yaml(tt_path)
        t7 = [t for t in teeth["teeth"]
              if t["id"] == "T7-invalid-inverse-case"]
        if len(t7) != 1:
            stop(f"{rid}: T7 tooth entry not found in teeth.yaml")
        obs = t7[0]["attack_observation"]
        if obs.get("bounded_input") != want:
            stop(f"{rid}: T7 attack_observation bounded_input != prereg "
                 f"probe spec {want}")
        run_meta = {"wall_seconds": t7[0]["run_meta"]["wall_seconds"]}
        identity_source = "T24/teeth.yaml attack_observation (provenance " \
                          "labeled: tooth run, included per prereg nat-14 " \
                          "provenance_note)"

    elif source == "t25-negative":
        mp_path = os.path.join(T25_DIR, "matched-pair.yaml")
        source_records.append(record_pin(mp_path))
        mp = load_yaml(mp_path)
        neg = mp["negative"]
        if neg.get("bounded_input") != want:
            stop(f"{rid}: matched-pair negative bounded_input != prereg "
                 f"probe spec {want}")
        # identity fields pinned from the frozen T25/run_negative.py
        # observation construction (matched-pair.yaml omits them)
        obs = {
            "probe_id": neg["probe_id"],
            "candidate_id": "t25-negative-r1",
            "correlation_id": "t25-negative-r1",
            "operation": "run_case",
            "bounded_input": neg["bounded_input"],
            "terminal_class": neg["terminal_class"],
            "settlement_committed_count": neg["settlement_committed_count"],
            "state_db_path": neg["state_db_path"],
            "state_db_sha256": neg["state_db_sha256"],
            "mechanism": MECHANISM,
            "started_at_utc": neg["run"]["started_at_utc"],
            "finished_at_utc": neg["run"]["finished_at_utc"],
        }
        run_meta = {"wall_seconds": neg["run"]["wall_seconds"]}
        identity_source = ("T25/matched-pair.yaml negative block; identity "
                           "fields (candidate_id/correlation_id/operation/"
                           "mechanism) pinned from the frozen T25/"
                           "run_negative.py observation construction")
    else:
        stop(f"{rid}: unknown corpus source {source!r}")

    db_path = obs["state_db_path"]
    pin = obs["state_db_sha256"]
    # fail-closed DB re-hash
    actual = sha256_file(db_path)
    if actual != pin:
        stop(f"{rid}: PIN MISMATCH {db_path}: pinned {pin} != actual {actual}")

    ev, states, settlements = read_db_facts(db_path)
    if len(states) != 1:
        stop(f"{rid}: expected exactly one run_terminal event, found "
             f"{len(states)}")
    failing_ref = failing_artifact(db_path, ev)
    truth, clauses = derive_truth(states[0], settlements, failing_ref)
    if truth != prereg_truth:
        stop(f"{rid}: TRUTH-DERIVATION AMBIGUITY: derived {truth!r} != prereg "
             f"truth_class {prereg_truth!r}")

    return {
        "id": rid,
        "source": source,
        "source_records": source_records,
        "probe": {"scenario": scenario, "max_rounds": max_rounds},
        "probe_id": obs["probe_id"],
        "identity": {"candidate_id": obs.get("candidate_id"),
                     "correlation_id": obs.get("correlation_id"),
                     "operation": obs.get("operation"),
                     "mechanism": obs.get("mechanism")},
        "identity_source": identity_source,
        "state_db_path": db_path,
        "db_sha256": pin,
        "truth_class": truth,
        "truth_derivation": {
            "terminal_state": states[0],
            "settlement_committed_count": settlements,
            "failing_artifact_sha256": failing_ref or None,
            "rule_clauses": clauses,
        },
        "actual_settled_units": settlements,
        "wall_seconds": (run_meta or {}).get("wall_seconds"),
    }


def main() -> int:
    os.makedirs(OUT_DIR, exist_ok=True)
    if os.environ.get("PYTHONDONTWRITEBYTECODE") != "1":
        stop("export PYTHONDONTWRITEBYTECODE=1 before running")

    # 1. prereg freeze check
    digest = sha256_file(PREREG_PATH)
    if digest != PREREG_SHA256:
        stop(f"prereg sha256 mismatch: {digest}")
    prereg = load_yaml(PREREG_PATH)
    rows_in = prereg["corpus"]["rows"]
    counts_frozen = prereg["corpus"]["class_counts_frozen"]

    # idempotence: if the decision entry already exists, verify the manifest
    # hash recorded there matches the manifest on disk and exit
    if os.path.isfile(DECISIONS_PATH):
        with open(DECISIONS_PATH) as f:
            dtext = f.read()
        if DECISION_ID in dtext and os.path.isfile(MANIFEST_PATH):
            m = re.search(r"corpus_manifest_sha256: \"([0-9a-f]{64})\"",
                          dtext[dtext.index(DECISION_ID):])
            if m and m.group(1) == sha256_file(MANIFEST_PATH):
                print(f"[phase0] {DECISION_ID} already recorded; manifest "
                      f"hash matches; nothing to do", flush=True)
                return 0
            stop(f"{DECISION_ID} already present but manifest hash differs")

    print(f"[phase0] resolving {len(rows_in)} rows from the frozen branch "
          f"records ...", flush=True)
    rows = [resolve_row(r, PREREG_SHA256) for r in rows_in]

    # cross-check frozen class counts
    counts = {}
    for r in rows:
        counts[r["truth_class"]] = counts.get(r["truth_class"], 0) + 1
    if counts != dict(counts_frozen):
        stop(f"derived class counts {counts} != frozen "
             f"{dict(counts_frozen)}")

    manifest = {
        "schema": "godspeed-bounded-judgment.remeasure-native-corpus-manifest",
        "created_utc": datetime.datetime.now(
            datetime.timezone.utc).isoformat(),
        "prereg": {"path": os.path.relpath(PREREG_PATH, REPO),
                   "sha256": PREREG_SHA256,
                   "decision_parent": "D-2026-09-19-RM-01"},
        "truth_rule": (
            "frozen mechanical rule (prereg truth_derivation + addendum-3): "
            "settled terminal -> settled_accepted; bounded-non-settlement "
            "terminal AND 0 settlements AND >=1 sha256-verified failing "
            "artifact -> verification_failed_nonsettlement; any other "
            "bounded non-settlement -> budget_exhausted; run error -> "
            "execution_failed (none expected); truth is derived from the "
            "pinned DBs and artifact objects, never hand-assigned"),
        "verification": {"db_pins_verified": len(rows),
                         "db_pin_mismatches": 0,
                         "truth_mismatches_vs_prereg": 0},
        "class_counts": counts,
        "rows": rows,
    }
    with open(MANIFEST_PATH, "w") as f:
        yaml.safe_dump(manifest, f, sort_keys=False, width=110,
                       allow_unicode=False)
    manifest_sha = sha256_file(MANIFEST_PATH)
    print(f"[phase0] manifest written: {MANIFEST_PATH}")
    print(f"[phase0] manifest sha256: {manifest_sha}")

    # 2. decision-log append (BEFORE any provider call)
    entry = (
        f"\n  - id: {DECISION_ID}\n"
        f"    date: \"2026-09-19\"\n"
        f"    task: remeasure-native-corpus\n"
        f"    kind: corpus-manifest-frozen\n"
        f"    decided: >-\n"
        f"      Second freeze stage of the remeasurement (parent\n"
        f"      D-2026-09-19-RM-01): the 15-row native corpus manifest was\n"
        f"      materialized from the pinned branch records BEFORE the first\n"
        f"      scored provider call. Every row's state database re-hashed to\n"
        f"      its pin (15/15 verified, fail-closed); mechanical truth\n"
        f"      derived by the frozen addendum-3 rule from the pinned DBs and\n"
        f"      hash-verified artifact objects reproduced the prereg truth\n"
        f"      classes exactly (6 settled_accepted / 5 budget_exhausted /\n"
        f"      4 verification_failed_nonsettlement; frozen class counts\n"
        f"      hold). Manifest sha256 {manifest_sha}; row count 15. No\n"
        f"      provider call has been made at this decision's timestamp.\n"
        f"    evidence: \".agents/evidence/godspeed-bounded-judgment/"
        f"remeasure-native-corpus/corpus-manifest.yaml\"\n"
        f"    corpus_manifest_sha256: \"{manifest_sha}\"\n"
        f"    corpus_row_count: 15\n"
        f"    prereg_sha256: \"{PREREG_SHA256}\"\n")
    with open(DECISIONS_PATH, "a") as f:
        f.write(entry)
    print(f"[phase0] decision entry {DECISION_ID} appended to "
          f"{DECISIONS_PATH}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
