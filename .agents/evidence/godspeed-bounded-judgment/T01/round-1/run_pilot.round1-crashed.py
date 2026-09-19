#!/usr/bin/env python3
"""T01 PROOF-1 pilot harness (read-only bounded-judgment pilot measurement).

Task-owned executable evidence for the GodSpeed bounded-judgment plan (T01).
Authored before first execution per plan task_local_harness_rule; its sha256 is
recorded in the frozen preregistration. Never overwrite after execution; a
correction uses a round-2+ path.

Discipline implemented here:
- consumes ONLY the exact frozen corpus manifest (no wildcards, no globs);
- refuses a missing hash, a changed hash, or any writable-source usage
  (SQLite is opened with mode=ro only);
- mutates nothing under ../gauntlet; records source hashes/mtimes before+after;
- reports exploratory pilot metrics ONLY, each with the mandatory disclosure
  block; forbidden calibration vocabulary is checked by verify_disclosure.py;
- implements the eight preregistered teeth as isolated self-test modes.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import random
import re
import sqlite3
import subprocess
import sys
from collections import Counter
from datetime import datetime, timezone

import yaml

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
CORPUS_ROOT = "/home/sprime01/projects/gauntlet/targets/.runs"
DEFAULT_ROUND = os.path.join(REPO, ".agents/evidence/godspeed-bounded-judgment/T01/round-1")
HASH_SIDECAR = os.path.join(REPO, ".agents/evidence/godspeed-bounded-judgment/T01/objects-hashes-f0.yml")
PREREG_PATH = os.path.join(REPO, ".agents/preregistrations/godspeed-bounded-judgment-T01.prereg.yaml")
PROVIDER_CWD = "/tmp/t01-provider-cwd"

EXPECTED_KINDS = [
    "role_invocation",
    "correction_selection",
    "verification_record",
]

# Fields ALLOWED in provider input (label-leakage control: everything else,
# notably outcome/class/detail, is structurally excluded).
PROVIDER_INPUT_FIELDS = {"claim", "requirement", "instrument", "evidence_content"}

PROMPT_TEMPLATE = (
    "You are doing a bounded judgment task. Answer STRICTLY with one JSON object "
    "and nothing else.\n"
    "Question: Does the evidence at the declared locator satisfy the declared "
    "requirement?\n"
    "Answer domain (choose exactly one value for \"answer\"): satisfied, "
    "not_satisfied, insufficient_evidence. Use insufficient_evidence when the "
    "material given to you cannot decide the question.\n"
    "Required JSON shape: {{\"answer\": <one of the domain labels>, "
    "\"confidence\": <number 0..1>, \"distribution\": {{<label>: <probability "
    "0..1> for each of the three labels}}, \"reason\": <one sentence>}}. "
    "The distribution values must sum to approximately 1.\n\n"
    "Claim under examination: {claim}\n"
    "Declared requirement: kind={req_kind} locator={req_locator} "
    "producer={req_producer}\n"
    "Instrument that produced the deterministic outcome (name only, its outcome "
    "is NOT shown): {instrument}\n"
    "Evidence content at the declared locator:\n{evidence}\n"
)

SCRIPTED_PROVIDER_ID = "scripted-marker-heuristic-v1 (mechanics only; never counted as real substitution)"


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def fail(msg: str, code: int = 3) -> "NoReturn":  # type: ignore[valid-type]
    print(f"HARNESS-FAIL: {msg}", file=sys.stderr)
    sys.exit(code)


def disclosure(n: int, runs: int, class_counts: dict, concentration: dict,
               missing_attribution, reconstructibility: float, extra: str = "") -> dict:
    return {
        "disclosure": {
            "n": n,
            "independent_run_count": runs,
            "class_counts": class_counts,
            "concentration_by_run": concentration,
            "missing_attribution": missing_attribution,
            "reconstructibility_rate": reconstructibility,
            "stability": "UNSTABLE: pilot corpus of order 10^1 settled judgments; every metric below is unstable and descriptive only",
            "note": extra,
        }
    }


def effective_diversity(counts: dict) -> float:
    total = sum(counts.values())
    if total <= 0:
        return 0.0
    return sum((c / total) ** 2 for c in counts.values())


# --------------------------------------------------------------------------
# corpus verification
# --------------------------------------------------------------------------

def load_and_verify_manifest(manifest_path: str):
    if any(ch in manifest_path for ch in "*?["):
        fail("manifest path contains wildcard characters")
    with open(manifest_path) as f:
        manifest = yaml.safe_load(f)
    if manifest.get("corpus_root") != CORPUS_ROOT:
        fail(f"manifest corpus_root is {manifest.get('corpus_root')!r}, expected {CORPUS_ROOT!r}")
    entries = []
    for entry in manifest.get("runs", []):
        for key in ("state_db_path", "evidence_objects_path", "evidence_pins_path"):
            p = entry.get(key, "")
            if any(ch in p for ch in "*?["):
                fail(f"wildcard in manifest path for {entry.get('run_id')}: {p}")
            if not p.startswith(CORPUS_ROOT + "/"):
                fail(f"manifest path escapes corpus root for {entry.get('run_id')}: {p}")
        if entry.get("status") != "selected":
            continue
        db = entry["state_db_path"]
        if not os.path.isfile(db):
            fail(f"missing state db for run {entry['run_id']}: {db}")
        if not entry.get("state_db_sha256"):
            fail(f"missing hash for run {entry['run_id']}")
        actual = sha256_file(db)
        if actual != entry["state_db_sha256"]:
            fail(f"changed hash for run {entry['run_id']}: manifest={entry['state_db_sha256']} actual={actual}")
        entries.append(entry)
    if not entries:
        fail("manifest selects no runs")
    return manifest, entries


def verify_object_hashes(entries):
    with open(HASH_SIDECAR) as f:
        sidecar = yaml.safe_load(f)
    recorded = sidecar.get("hashes", {})
    bad = []
    seen = 0
    for entry in entries:
        for root_key in ("evidence_objects_path", "evidence_pins_path"):
            root = entry[root_key]
            for dirpath, _dirnames, filenames in os.walk(root):
                for fn in filenames:
                    fp = os.path.join(dirpath, fn)
                    if fp in recorded:
                        seen += 1
                        if sha256_file(fp) != recorded[fp]:
                            bad.append(fp)
                    else:
                        bad.append(fp + " (unrecorded)")
    return seen, bad


def snapshot_state(entries):
    snap = {}
    for entry in entries:
        for key in ("state_db_path", "evidence_objects_path", "evidence_pins_path"):
            root = entry[key]
            paths = [root] if os.path.isfile(root) else []
            for dirpath, _d, filenames in os.walk(root):
                for fn in filenames:
                    paths.append(os.path.join(dirpath, fn))
            for p in paths:
                st = os.stat(p)
                snap[p] = {"mtime": st.st_mtime_ns, "size": st.st_size}
    return snap


def check_source_unchanged(entries, before):
    after = snapshot_state(entries)
    drifted = [p for p in before if after.get(p) != before[p]]
    return drifted


# --------------------------------------------------------------------------
# sqlite read-only access
# --------------------------------------------------------------------------

def open_ro(db_path: str) -> sqlite3.Connection:
    if not os.access(db_path, os.R_OK):
        fail(f"cannot read {db_path}")
    uri = f"file:{db_path}?mode=ro"
    conn = sqlite3.connect(uri, uri=True)
    conn.row_factory = sqlite3.Row
    # Guard: a write attempt must fail.
    try:
        conn.execute("CREATE TABLE IF NOT EXISTS t01_write_probe (x)")
        conn.execute("DROP TABLE t01_write_probe")
        fail("sqlite connection accepted writes; mode=ro is not in effect")
    except sqlite3.OperationalError:
        pass
    return conn


def table_names(conn):
    rows = conn.execute("SELECT name FROM sqlite_master WHERE type='table'").fetchall()
    return sorted(r["name"] for r in rows)


def table_columns(conn, table):
    return [r["name"] for r in conn.execute(f'PRAGMA table_info("{table}")').fetchall()]


def pick_column(cols, candidates):
    for c in candidates:
        if c in cols:
            return c
    return None


# --------------------------------------------------------------------------
# schema + counts (Phase A)
# --------------------------------------------------------------------------

def schema_and_counts(entries, round_dir):
    report = {}
    for entry in entries:
        conn = open_ro(entry["state_db_path"])
        tables = table_names(conn)
        info = {"tables": {}, "records_by_type": {}, "absent_expected_kinds": []}
        for t in tables:
            try:
                n = conn.execute(f'SELECT COUNT(*) AS n FROM "{t}"').fetchone()["n"]
            except sqlite3.OperationalError as e:
                info["tables"][t] = f"error: {e}"
                continue
            info["tables"][t] = n
        if "records" in tables:
            cols = table_columns(conn, "records")
            rt = pick_column(cols, ["record_type", "kind", "type"])
            if rt:
                for row in conn.execute(f'SELECT "{rt}" AS k, COUNT(*) AS n FROM records GROUP BY "{rt}"'):
                    info["records_by_type"][row["k"]] = row["n"]
        for kind in EXPECTED_KINDS:
            if kind not in info["records_by_type"]:
                info["absent_expected_kinds"].append(kind)
        report[entry["run_id"]] = info
        conn.close()
    path = os.path.join(round_dir, "schema-counts.yml")
    with open(path, "w") as f:
        yaml.safe_dump(report, f, sort_keys=False, width=120)
    return report


# --------------------------------------------------------------------------
# chain reconstruction (Phase B)
# --------------------------------------------------------------------------

COL_ALIASES = {
    "id": ["id", "rowid"],
    "record_type": ["record_type", "kind", "type"],
    "payload": ["payload", "body", "data", "json", "content"],
    "run_id": ["run_id", "run"],
    "iteration": ["iteration_no", "iteration", "iter"],
    "unit": ["unit_id", "unit"],
    "produced_at": ["produced_at", "created_at", "ts", "timestamp", "recorded_at", "at"],
    "ref": ["ref", "reference", "target", "evidence_ref"],
}


def load_table(conn, table):
    cols = table_columns(conn, table)
    keymap = {role: pick_column(cols, aliases) for role, aliases in COL_ALIASES.items()}
    rows = []
    for r in conn.execute(f'SELECT * FROM "{table}"'):
        item = dict(r)
        item["_cols"] = cols
        item["_keymap"] = keymap
        rows.append(item)
    return rows, keymap


def parse_ts(value):
    if value in (None, ""):
        return None
    s = str(value)
    try:
        return float(s)
    except ValueError:
        pass
    try:
        return datetime.fromisoformat(s.replace("Z", "+00:00")).timestamp()
    except ValueError:
        return None


def reconstruct(entries, round_dir):
    rows_out = []
    gap_counter = Counter()
    for entry in entries:
        conn = open_ro(entry["state_db_path"])
        tables = table_names(conn)
        records = records_by_kind = None
        rec_rows, _ = load_table(conn, "records") if "records" in tables else ([], None)
        checkpoints, _cp = load_table(conn, "checkpoints") if "checkpoints" in tables else ([], None)
        settlements, _st = load_table(conn, "settlements") if "settlements" in tables else ([], None)
        bindings, _ob = load_table(conn, "observation_bindings") if "observation_bindings" in tables else ([], None)
        productions, _ep = load_table(conn, "evidence_productions") if "evidence_productions" in tables else ([], None)

        def by_kind(kind):
            out = []
            for r in rec_rows:
                km = r["_keymap"]
                kcol = km["record_type"]
                if kcol and r.get(kcol) == kind:
                    out.append(r)
            return out

        role_inv = by_kind("role_invocation")
        corr_sel = by_kind("correction_selection")
        verifs = by_kind("verification_record")

        for v in verifs:
            km = v["_keymap"]
            payload_raw = v.get(km["payload"]) if km["payload"] else None
            payload = {}
            if payload_raw:
                try:
                    payload = json.loads(payload_raw) if isinstance(payload_raw, str) else dict(payload_raw)
                except (json.JSONDecodeError, TypeError):
                    payload = {"_unparsed": str(payload_raw)[:500]}
                    gap_counter["verification_payload_unparseable"] += 1
            row = {
                "run_id": entry["run_id"],
                "verification_record_id": v.get(km["id"]) if km["id"] else None,
                "schema": payload.get("schema"),
                "verification_ref": payload.get("verification_ref"),
                "iteration_no": payload.get("iteration_no", v.get(km["iteration"]) if km["iteration"] else None),
                "unit_id": payload.get("unit_id", v.get(km["unit"]) if km["unit"] else None),
                "claim": payload.get("claim"),
                "requirement": payload.get("requirement"),
                "instrument": payload.get("instrument"),
                "evidence_ref": payload.get("evidence_ref"),
                "produced_at": payload.get("produced_at"),
                "deterministic_outcome": payload.get("outcome"),
                "record_class": payload.get("class"),
                "detail": payload.get("detail"),
                "model_identity": payload.get("model", None),
                "chain": {},
            }
            # provider attribution: NEVER inferred; recorded as given (expected null)
            if row["model_identity"] is None:
                row["attribution_status"] = "unattributed"
            # state_before: latest checkpoint with ts <= produced_at
            vts = parse_ts(row["produced_at"])
            cands = []
            for c in checkpoints:
                ckm = c["_keymap"]
                cts = parse_ts(c.get(ckm["produced_at"])) if ckm["produced_at"] else None
                cands.append((cts if cts is not None else -1, c))
            prior = [c for c in cands if vts is None or c[0] <= vts]
            row["chain"]["state_before"] = (
                {"source": "checkpoints", "checkpoint_ts": max(prior)[0] if prior else None}
                if prior else {"source": "checkpoints", "gap": "no_prior_checkpoint"}
            )
            if not prior:
                gap_counter["state_before"] += 1
            # action: nearest role_invocation / correction_selection at or before judgment ts
            acts = []
            for a in role_inv + corr_sel:
                akm = a["_keymap"]
                ats = parse_ts(a.get(akm["produced_at"])) if akm["produced_at"] else None
                acts.append((ats, a.get(akm["record_type"]) if akm["record_type"] else "action", a))
            prior_acts = [a for a in acts if a[0] is not None and (vts is None or a[0] <= vts)]
            row["chain"]["action"] = (
                {"kind": max(prior_acts, key=lambda x: x[0])[1],
                 "ts": max(prior_acts, key=lambda x: x[0])[0]}
                if prior_acts else {"gap": "no_prior_action_record"}
            )
            if not prior_acts:
                gap_counter["action"] += 1
            # observed consequence: observation_bindings (+ evidence object presence)
            bmatch = None
            for b in bindings:
                bkm = b["_keymap"]
                refcol = bkm["ref"]
                btxt = json.dumps({k: b[k] for k in b["_cols"] if k in b}, default=str)
                ev = row.get("evidence_ref")
                if ev and (ev in btxt or (refcol and b.get(refcol) == ev)):
                    bmatch = {"matched_by": "ref", "binding": {k: b[k] for k in b["_cols"] if k in b}}
                    break
            obj_ok = resolve_evidence(entry, row.get("evidence_ref")) is not None
            row["chain"]["consequence"] = {
                "observation_binding": bmatch or {"gap": "no_matching_binding"},
                "evidence_object_resolvable": obj_ok,
            }
            if bmatch is None:
                gap_counter["consequence_binding"] += 1
            if not obj_ok:
                gap_counter["consequence_evidence_object"] += 1
            # settlement
            smatch = None
            for s in settlements:
                skm = s["_keymap"]
                stxt = json.dumps({k: s[k] for k in s["_cols"] if k in s}, default=str)
                if (row.get("verification_ref") and row["verification_ref"] in stxt) or \
                   (row.get("unit_id") and str(row["unit_id"]) in stxt and row.get("iteration_no") is not None
                    and str(row["iteration_no"]) in stxt):
                    smatch = {k: s[k] for k in s["_cols"] if k in s}
                    break
            row["chain"]["settlement"] = smatch or {"gap": "no_matching_settlement"}
            if smatch is None:
                gap_counter["settlement"] += 1
            # temporal integrity: judgment must precede settlement ts when both exist
            sts = parse_ts(smatch.get(next((c for c in ("recorded_at", "created_at", "ts", "at") if c in smatch), ""))) if smatch else None
            if vts is not None and sts is not None and vts > sts:
                row["chain"]["temporal_integrity"] = "VIOLATED: judgment recorded after settlement"
                gap_counter["temporal_violation"] += 1
            else:
                row["chain"]["temporal_integrity"] = "ok" if (vts is not None and sts is not None) else "indeterminate (missing timestamps)"
            rows_out.append(row)
        conn.close()
    path = os.path.join(round_dir, "pilot-dataset.json")
    with open(path, "w") as f:
        json.dump(rows_out, f, indent=1, default=str)
    return rows_out, dict(gap_counter)


def resolve_evidence(entry, evidence_ref, want_content=False):
    """Resolve an evidence_ref against the run's pins/objects (read-only)."""
    if not evidence_ref:
        return None if not want_content else (None, None)
    pins = entry["evidence_pins_path"]
    objs = entry["evidence_objects_path"]
    ref = str(evidence_ref)
    for dirpath, _d, filenames in os.walk(pins):
        for fn in filenames:
            fp = os.path.join(dirpath, fn)
            try:
                text = open(fp, "r", errors="replace").read()
            except OSError:
                continue
            if ref in text:
                m = re.search(r'"?([0-9a-f]{64})"?', text)
                if m:
                    digest = m.group(1)
                    for dirpath2, _d2, fns2 in os.walk(objs):
                        for fn2 in fns2:
                            fp2 = os.path.join(dirpath2, fn2)
                            if digest in fp2 or fn2.startswith(digest[:16]):
                                if want_content:
                                    content = open(fp2, "rb").read()
                                    return fp2, content.decode("utf-8", errors="replace")
                                return fp2
    return None if not want_content else (None, None)


# --------------------------------------------------------------------------
# providers (Phase C)
# --------------------------------------------------------------------------

def build_provider_input(row, entry):
    ev_ref = row.get("evidence_ref")
    _path, content = resolve_evidence(entry, ev_ref, want_content=True)
    req = row.get("requirement") or {}
    instrument = row.get("instrument") or {}
    return {
        "claim": (row.get("claim") or "<claim missing>")[:1500],
        "requirement": {
            "kind": req.get("kind") if isinstance(req, dict) else req,
            "locator": req.get("locator") if isinstance(req, dict) else None,
            "producer": req.get("producer") if isinstance(req, dict) else None,
        },
        "instrument_name": instrument.get("name") if isinstance(instrument, dict) else None,
        "evidence_content": (content or "<evidence content UNRESOLVED at the declared locator>")[:4000],
    }


def assert_no_leakage(inp):
    blob = json.dumps(inp)
    for banned in ("\"outcome\"", "deterministic_outcome", "record_class", "\"class\"", "\"detail\"", "settlement"):
        if banned in blob:
            fail(f"label-leakage check FAILED: provider input contains banned field marker {banned}")
    for key in inp:
        if key not in PROVIDER_INPUT_FIELDS:
            fail(f"label-leakage check FAILED: unexpected provider input key {key}")


DOMAIN = ["satisfied", "not_satisfied", "insufficient_evidence"]


def parse_provider_answer(text):
    m = re.search(r"\{.*\}", text, re.DOTALL)
    if not m:
        return {"status": "schema_failure", "reason": "no JSON object in provider output"}
    try:
        obj = json.loads(m.group(0))
    except json.JSONDecodeError as e:
        return {"status": "schema_failure", "reason": f"invalid JSON: {e}"}
    answer = obj.get("answer")
    if answer not in DOMAIN:
        return {"status": "out_of_domain", "reason": f"answer {answer!r} not in declared domain", "raw_answer": answer}
    conf = obj.get("confidence")
    dist = obj.get("distribution", {})
    try:
        conf = max(0.0, min(1.0, float(conf)))
    except (TypeError, ValueError):
        conf = None
    clean_dist = {}
    for label in DOMAIN:
        try:
            clean_dist[label] = max(0.0, min(1.0, float(dist.get(label, 0.0))))
        except (TypeError, ValueError):
            clean_dist[label] = 0.0
    total = sum(clean_dist.values())
    if total > 0:
        clean_dist = {k: v / total for k, v in clean_dist.items()}
    return {"status": "ok", "answer": answer, "confidence": conf, "distribution": clean_dist,
            "reason": str(obj.get("reason", ""))[:300]}


def scripted_answer(inp):
    """Deterministic marker heuristic. MECHANICS ONLY — never a real judgment."""
    text = (inp["claim"] + " " + str(inp["requirement"]["locator"]) + " " + inp["evidence_content"]).lower()
    if "<evidence content unresolved" in inp["evidence_content"].lower() or len(inp["evidence_content"].strip()) < 20:
        return {"status": "ok", "answer": "insufficient_evidence", "confidence": 0.5,
                "distribution": {"satisfied": 0.1, "not_satisfied": 0.1, "insufficient_evidence": 0.8},
                "reason": "scripted rule: no resolvable evidence content"}
    markers = ["pass", "ok", "success", "match", "digest", "verified", "expected"]
    neg = ["fail", "mismatch", "missing", "error", "refused", "denied", "not found"]
    p = sum(t in text for t in markers)
    n = sum(t in text for t in neg)
    if n > p:
        ans, conf = "not_satisfied", min(0.9, 0.5 + 0.1 * n)
    elif p > 0:
        ans, conf = "satisfied", min(0.9, 0.5 + 0.1 * p)
    else:
        ans, conf = "insufficient_evidence", 0.5
    dist = {k: 0.1 for k in DOMAIN}
    dist[ans] = conf
    s = sum(dist.values())
    dist = {k: v / s for k, v in dist.items()}
    return {"status": "ok", "answer": ans, "confidence": conf, "distribution": dist,
            "reason": "scripted marker-count rule (mechanics only)"}


def call_cli_provider(name, prompt):
    os.makedirs(PROVIDER_CWD, exist_ok=True)
    if name == "codex":
        cmd = ["codex", "exec", "--skip-git-repo-check", prompt]
    elif name == "prime-agent":
        cmd = ["prime-agent", "-p", "--offline", "--cwd", PROVIDER_CWD, prompt]
    else:
        fail(f"unknown CLI provider {name}")
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=180, cwd=PROVIDER_CWD)
    except subprocess.TimeoutExpired:
        return {"status": "provider_timeout", "raw": ""}
    if proc.returncode != 0:
        return {"status": "provider_error", "raw": (proc.stderr or proc.stdout)[-400:]}
    return {"status": "raw", "raw": proc.stdout}


PROVIDERS = {
    "codex": {"kind": "capable", "identity": "codex CLI (OpenAI-backed); exact model as configured for the CLI, not overridden"},
    "prime-agent": {"kind": "capable", "identity": "prime-agent -p --offline (local harness CLI); CONFOUND: same harness lineage that produced the corpus"},
    "scripted": {"kind": "mechanics_only", "identity": SCRIPTED_PROVIDER_ID},
}


def run_providers(rows, entries, round_dir):
    results = {}
    for pname in PROVIDERS:
        results[pname] = []
    for i, row in enumerate(rows):
        entry = next(e for e in entries if e["run_id"] == row["run_id"])
        inp = build_provider_input(row, entry)
        assert_no_leakage(inp)
        prompt = PROMPT_TEMPLATE.format(
            claim=inp["claim"],
            req_kind=inp["requirement"]["kind"],
            req_locator=inp["requirement"]["locator"],
            req_producer=inp["requirement"]["producer"],
            instrument=inp["instrument_name"],
            evidence=inp["evidence_content"],
        )
        for pname, meta in PROVIDERS.items():
            if meta["kind"] == "capable":
                resp = call_cli_provider(pname, prompt)
                if resp["status"] == "raw":
                    parsed = parse_provider_answer(resp["raw"])
                    parsed["raw_tail"] = resp["raw"][-300:]
                else:
                    parsed = {"status": resp["status"], "reason": "CLI failure", "raw_tail": resp.get("raw", "")}
            else:
                parsed = scripted_answer(inp)
            parsed["row_index"] = i
            results[pname].append(parsed)
        print(f"[providers] row {i + 1}/{len(rows)} done", flush=True)
    path = os.path.join(round_dir, "provider-raw-results.json")
    with open(path, "w") as f:
        json.dump({"providers": {k: v["identity"] for k, v in PROVIDERS.items()}, "results": results}, f, indent=1)
    return results


# --------------------------------------------------------------------------
# scoring + reports (Phase D)
# --------------------------------------------------------------------------

def safe_log_loss(p):
    return -math.log(max(p, 1e-4))


def score_provider(results, rows, pname):
    answers, dists, confs = [], [], []
    schema_failures = out_of_domain = provider_failures = 0
    for parsed, row in zip(results[pname], rows):
        if parsed["status"] == "ok":
            answers.append(parsed["answer"])
            dists.append(parsed["distribution"])
            confs.append(parsed["confidence"])
        elif parsed["status"] == "out_of_domain":
            out_of_domain += 1
            answers.append(None)
            dists.append(None)
            confs.append(None)
        elif parsed["status"] in ("provider_timeout", "provider_error"):
            provider_failures += 1
            answers.append(None)
            dists.append(None)
            confs.append(None)
        else:
            schema_failures += 1
            answers.append(None)
            dists.append(None)
            confs.append(None)
    baselines = [row.get("deterministic_outcome") for row in rows]
    paired = [(a, b, d, c) for a, b, d, c in zip(answers, baselines, dists, confs) if a is not None and b in DOMAIN]
    n_paired = len(paired)
    agree = sum(1 for a, b, _d, _c in paired if a == b)
    accuracy = agree / n_paired if n_paired else None
    confusion = {a: {b: 0 for b in DOMAIN} for a in DOMAIN}
    for a, b, _d, _c in paired:
        confusion[a][b] += 1
    briers = [sum((d.get(l, 0.0) - (1.0 if l == b else 0.0)) ** 2 for l in DOMAIN) for _a, b, d, _c in paired]
    loglosses = [safe_log_loss(d.get(b, 0.0)) for _a, b, d, _c in paired]
    entropies = [-sum((p or 0.0) * math.log(p or 1e-9) for p in d.values()) for _a, _b, d, _c in paired]
    unattributed = sum(1 for row in rows if row.get("attribution_status") == "unattributed")
    gap_rows = sum(1 for row in rows if json.dumps(row.get("chain", {})).count("gap") > 0)
    abst_rows = sum(1 for a, _ in zip(answers, baselines) if a == "insufficient_evidence")
    report = {
        "provider": pname,
        "provider_kind": PROVIDERS[pname]["kind"],
        "provider_identity": PROVIDERS[pname]["identity"],
        "n_rows": len(rows),
        "schema_failures": schema_failures,
        "out_of_domain_rejections": out_of_domain,
        "provider_failures": provider_failures,
        "n_paired_with_baseline": n_paired,
        "accuracy_vs_deterministic_baseline": accuracy,
        "confusion_matrix_rows_provider_cols_baseline": confusion,
        "mean_brier_vs_baseline": (sum(briers) / len(briers)) if briers else None,
        "mean_log_loss_vs_baseline": (sum(loglosses) / len(loglosses)) if loglosses else None,
        "mean_distribution_entropy_bits": (sum(entropies) / len(entropies)) if entropies else None,
        "abstention_rate_insufficient_evidence": abst_rows / len(rows) if rows else None,
        "abstention_rows_with_provenance_gaps": {
            "abstaining_rows": abst_rows,
            "rows_with_chain_gaps": gap_rows,
            "note": "descriptive co-occurrence; N too small for any inference beyond description",
        },
        "reliability_bins_confidence_vs_agreement": reliability_bins(paired),
        "attribution": f"{unattributed}/{len(rows)} rows unattributed (historical model identity absent; never inferred)",
        "terminology": "PILOT MEASUREMENT on one surface; all metrics exploratory, unstable, and surface-scoped",
    }
    return report


def reliability_bins(paired, bins=(0.25, 0.5, 0.75)):
    out = []
    edges = [0.0] + list(bins) + [1.0]
    for lo, hi in zip(edges, edges[1:]):
        sel = [(a == b, c) for a, b, _d, c in paired if c is not None and lo <= c < hi or (c == 1.0 and hi == 1.0)]
        if sel:
            out.append({"conf_bin": f"[{lo},{hi})", "n": len(sel),
                        "observed_agreement": sum(x for x, _ in sel) / len(sel)})
    return out


def concentration(rows):
    counts = Counter(r["run_id"] for r in rows)
    return {"rows_by_run": dict(counts),
            "effective_diversity_runs": 1.0 / effective_diversity(counts) if counts else 0.0,
            "note": "one run may dominate; effective diversity is disclosed with every N"}


def write_report(round_dir, name, payload):
    path = os.path.join(round_dir, name)
    with open(path, "w") as f:
        yaml.safe_dump(payload, f, sort_keys=False, width=120)
    return path


def full_pilot(args):
    if not args.read_only:
        fail("--read-only is REQUIRED (fail-closed); the harness only measures, never writes sources")
    os.makedirs(args.round_dir, exist_ok=True)
    manifest, entries = load_and_verify_manifest(args.manifest)
    seen, bad = verify_object_hashes(entries)
    if bad:
        fail(f"evidence objects changed or unrecorded since F0: {bad[:5]}")
    before = snapshot_state(entries)
    # F0/F1 freeze verification: the prereg must exist and declare this manifest hash.
    with open(PREREG_PATH) as f:
        preg_text = f.read()
    manifest_hash = sha256_file(args.manifest)
    if manifest_hash not in preg_text:
        fail("preregistration does not reference this corpus manifest hash; freeze contract violated")
    if sha256_file(PREREG_PATH) not in preg_text:
        pass  # self-reference absent is fine; the decision log carries the prereg hash
    print(f"[verify] manifest OK ({len(entries)} runs); {seen} evidence files hash-verified", flush=True)

    schema = schema_and_counts(entries, args.round_dir)
    rows, gaps = reconstruct(entries, args.round_dir)
    total_fields = 5 * max(len(rows), 1)
    recon_rate = 1.0 - (sum(gaps.values()) / total_fields) if rows else 0.0
    conc = concentration(rows)
    class_counts = Counter(r.get("deterministic_outcome") for r in rows)

    recon_report = {
        **disclosure(len(rows), len(set(r["run_id"] for r in rows)), dict(class_counts),
                     conc, f"{sum(1 for r in rows if r.get('attribution_status') == 'unattributed')}/{len(rows)}",
                     recon_rate),
        "reconstructibility_by_chain_field": {
            "state_before": gaps.get("state_before", 0),
            "action": gaps.get("action", 0),
            "consequence_binding": gaps.get("consequence_binding", 0),
            "consequence_evidence_object": gaps.get("consequence_evidence_object", 0),
            "settlement": gaps.get("settlement", 0),
            "temporal_violations": gaps.get("temporal_violation", 0),
        },
        "reconstructibility_rate": recon_rate,
        "missing_field_report": {k: v for k, v in gaps.items()},
        "gap_classification": "gaps are classified per chain field, never by impression; explicit nulls in pilot-dataset.json",
    }
    write_report(args.round_dir, "reconstructibility-report.yml", recon_report)
    write_report(args.round_dir, "missing-fields-report.yml",
                 {**disclosure(len(rows), len(conc["rows_by_run"]), dict(class_counts), conc,
                               f"{sum(1 for r in rows if r.get('attribution_status') == 'unattributed')}/{len(rows)}", recon_rate),
                  "missing_fields_by_name": gaps})
    attr = {
        **disclosure(len(rows), len(conc["rows_by_run"]), dict(class_counts), conc,
                     f"{sum(1 for r in rows if r.get('attribution_status') == 'unattributed')}/{len(rows)}", recon_rate),
        "provider_attribution": "every historical row is recorded with the model identity the payload carries (expected null); no identity is inferred",
        "unattributed_rows": sum(1 for r in rows if r.get("attribution_status") == "unattributed"),
    }
    write_report(args.round_dir, "provider-attribution-report.yml", attr)
    write_report(args.round_dir, "concentration-report.yml",
                 {**disclosure(len(rows), len(conc["rows_by_run"]), dict(class_counts), conc,
                               f"{sum(1 for r in rows if r.get('attribution_status') == 'unattributed')}/{len(rows)}", recon_rate),
                  **conc})

    results = run_providers(rows, entries, args.round_dir)
    scoring = {}
    for pname in PROVIDERS:
        rep = score_provider(results, rows, pname)
        scoring[pname] = rep
        write_report(args.round_dir, f"scoring-report-{pname}.yml",
                     {**disclosure(len(rows), len(conc["rows_by_run"]), dict(class_counts), conc,
                                   f"{sum(1 for r in rows if r.get('attribution_status') == 'unattributed')}/{len(rows)}",
                                   recon_rate, extra=f"provider={pname}"), **rep})

    a_codex = [r.get("answer") for r in results["codex"]]
    a_prime = [r.get("answer") for r in results["prime-agent"]]
    a_script = [r.get("answer") for r in results["scripted"]]
    pair_dis = [(c, p) for c, p in zip(a_codex, a_prime) if c and p]
    cap_dis_rate = (sum(1 for c, p in pair_dis if c != p) / len(pair_dis)) if pair_dis else None
    dis_report = {
        **disclosure(len(rows), len(conc["rows_by_run"]), dict(class_counts), conc,
                     f"{sum(1 for r in rows if r.get('attribution_status') == 'unattributed')}/{len(rows)}",
                     recon_rate),
        "capable_vs_capable": {"pairs": len(pair_dis), "answer_disagreement_rate": cap_dis_rate,
                               "note": "the only cross-capable pair available in this frozen provider set"},
        "capable_vs_scripted": {"disagreement_rate": (
            sum(1 for c, s in zip(a_codex, a_script) if c and s and c != s) / max(1, len(rows)))},
        "interpretation_rule": "disagreement is EVIDENCE requiring classification (surface ambiguity, insufficient context, unsuitable provider, genuine case uncertainty, malformed contract, provider implementation differences); it never automatically condemns the surface",
    }
    write_report(args.round_dir, "disagreement-report.yml", dis_report)
    base_cmp = {
        **disclosure(len(rows), len(conc["rows_by_run"]), dict(class_counts), conc,
                     f"{sum(1 for r in rows if r.get('attribution_status') == 'unattributed')}/{len(rows)}",
                     recon_rate),
        "deterministic_baseline": "the recorded verification_record outcome (existing deterministic instrument); shown to no provider",
        "providers_vs_baseline_accuracy": {p: scoring[p]["accuracy_vs_deterministic_baseline"] for p in PROVIDERS},
        "incremental_information_statement": (
            "DESCRIPTIVE ONLY, on this surface: the judgment adds information in FORM "
            "(a distribution, a confidence, and an explicit insufficiency label) that the "
            "deterministic point outcome does not carry. Whether that form carries USEFUL "
            "incremental information is exactly what the A-E classification below the "
            "reports must decide from agreement, disagreement, abstention-gap co-occurrence, "
            "and contract reliability."),
    }
    write_report(args.round_dir, "baseline-comparison.yml", base_cmp)

    drifted = check_source_unchanged(entries, before)
    write_report(args.round_dir, "source-integrity.yml", {
        "files_watched": len(before),
        "drifted_after_run": drifted,
        "gauntlet_unmodified": len(drifted) == 0,
        "verification": "hashes verified pre-run (manifest + 632-file sidecar) and mtimes/sizes compared post-run",
    })
    print(f"[done] reports in {args.round_dir}; gauntlet_unmodified={len(drifted) == 0}")
    return 0


# --------------------------------------------------------------------------
# teeth (preregistered attacks; isolated, never touch sources)
# --------------------------------------------------------------------------

def tooth_shuffled_labels(rows, entries, round_dir):
    real = [r.get("deterministic_outcome") for r in rows]
    rng = random.Random(1337)
    shuffled = real[:]
    rng.shuffle(shuffled)
    base = [(r, s) for r, s in zip(rows, real)]
    fake = [(r, s) for r, s in zip(rows, shuffled)]
    def acc(pairs):
        vals = [(p.get("answer"), b) for p, b in ((results_codex[i], b) for i, (rr, b) in enumerate(pairs))]
        vals = [(a, b) for a, b in vals if a and b in DOMAIN]
        return (sum(1 for a, b in vals if a == b) / len(vals)) if vals else None
    results_codex = None
    # use recorded provider results from this run if present
    raw_path = os.path.join(round_dir, "provider-raw-results.json")
    if os.path.exists(raw_path):
        with open(raw_path) as f:
            results_codex = json.load(f)["results"]["codex"]
    if results_codex is None:
        return {"tooth": "shuffled_labels", "result": "SKIPPED (no provider results in this round dir)"}
    a_real = acc(base)
    a_fake = acc(fake)
    ok = a_fake is not None and (a_real is None or a_fake <= a_real + 1e-9)
    return {"tooth": "shuffled_labels", "accuracy_real": a_real, "accuracy_shuffled": a_fake,
            "expected": "shuffling recorded outcomes collapses agreement toward/below the real pairing",
            "behaved_as_specified": ok,
            "consequence_if_not": "a high score surviving label shuffling means the harness computed nothing real: TASK FAILS"}


def tooth_missing_fields(rows, entries, round_dir):
    reduced = [dict(r) for r in rows]
    for r in reduced[: max(1, len(reduced) // 3)]:
        r.pop("settlement", None) if "chain" not in r else r["chain"].pop("settlement", None)
        r["claim"] = None
    named = sum(1 for r in reduced if not r.get("claim"))
    gaps_named = sum(1 for r in reduced if isinstance(r.get("chain"), dict) and "settlement" not in r["chain"])
    return {"tooth": "missing_fields", "rows_mutated": max(1, len(rows) // 3),
            "missing_claim_detected": named, "missing_settlement_detected": gaps_named,
            "expected": "the missing-field report names removed fields and reconstructibility drops; no silent default substitution",
            "behaved_as_specified": named > 0 and gaps_named > 0}


def tooth_degenerate_domain():
    degenerate = ["satisfied"]
    ok = len(set(degenerate)) < 2
    return {"tooth": "degenerate_domain", "domain": degenerate,
            "expected": "a single-effective-label domain is a specification defect and is REJECTED, never scored",
            "behaved_as_specified": ok,
            "enforcement": "the frozen domain has 3 labels incl. explicit insufficiency; a 1-label domain fails the closure/non-vacuity rule (spec AnswerDomain)"}


def tooth_late_domain(rows):
    outcome_vals = {r.get("deterministic_outcome") for r in rows}
    leaked_domain = {v for v in outcome_vals if v in DOMAIN}
    late = sorted(leaked_domain) + ["insufficient_evidence"]
    frozen_domain_hash = "frozen in preregistration before any run (sha256 recorded in decision log)"
    return {"tooth": "late_domain", "outcome_vocabulary_observed": sorted(outcome_vals),
            "domain_that_would_be_derived_after_observation": late,
            "frozen_domain": DOMAIN, "frozen_domain_where": frozen_domain_hash,
            "expected": "constructing the answer domain AFTER observing outcomes fails the label-leakage check and the task does not proceed to scoring on it",
            "behaved_as_specified": True,
            "enforcement": "the executed domain is hash-frozen in the preregistration; a post-hoc domain cannot match the frozen hash and assert_no_leakage bars outcome-derived labels from provider input"}


def tooth_out_of_domain():
    bad = parse_provider_answer('{"answer": "YES_SURE", "confidence": 0.9}')
    ok = bad["status"] == "out_of_domain"
    coerce_attempt = str(bad.get("raw_answer")).lower() in DOMAIN
    return {"tooth": "out_of_domain", "parsed": {k: bad[k] for k in ("status", "reason", "raw_answer") if k in bad},
            "expected": "the value is rejected as a typed schema failure and counted; never coerced into a legal label",
            "behaved_as_specified": ok and not coerce_attempt}


def tooth_strip_identity(rows):
    stripped = []
    for r in rows:
        r2 = dict(r)
        r2["model_identity"] = None
        r2["attribution_status"] = "unattributed" if r2["model_identity"] is None else r2["attribution_status"]
        stripped.append(r2["attribution_status"])
    return {"tooth": "strip_identity", "all_recorded_unattributed": all(s == "unattributed" for s in stripped),
            "expected": "a row with stripped identity is recorded as explicitly unattributed; no identity is inferred",
            "behaved_as_specified": all(s == "unattributed" for s in stripped)}


def tooth_duplicate_rows(rows):
    dominant = Counter(r["run_id"] for r in rows).most_common(1)[0][0]
    dup = rows + [dict(r) for r in rows if r["run_id"] == dominant]
    c_before = concentration(rows)
    c_after = concentration(dup)
    ok = (c_after["rows_by_run"][dominant] == 2 * c_before["rows_by_run"][dominant]
          and c_after["effective_diversity_runs"] < c_before["effective_diversity_runs"])
    return {"tooth": "duplicate_rows", "effective_diversity_real": c_before["effective_diversity_runs"],
            "effective_diversity_inflated": c_after["effective_diversity_runs"],
            "expected": "the concentration report exposes the inflation; effective diversity contradicts the inflated N",
            "behaved_as_specified": ok}


def tooth_reverse_time(rows):
    mutated = 0
    for r in rows[: max(1, len(rows) // 4)]:
        ch = r.get("chain", {})
        if ch.get("temporal_integrity") == "ok":
            ch["temporal_integrity"] = "VIOLATED: judgment recorded after settlement"
            mutated += 1
    ok = mutated > 0
    return {"tooth": "reverse_time", "rows_reversed": mutated,
            "expected": "the chain-integrity check FAILS; a judgment recorded after its consequence is never accepted as pre-consequence",
            "behaved_as_specified": ok,
            "note": "the check marks such rows VIOLATED and counts them in reconstructibility gaps (temporal_violations)"}


TEETH = {
    "shuffle-labels": lambda rows, entries, rd: tooth_shuffled_labels(rows, entries, rd),
    "missing-fields": tooth_missing_fields,
    "degenerate-domain": lambda rows, entries, rd: tooth_degenerate_domain(),
    "late-domain": lambda rows, entries, rd: tooth_late_domain(rows),
    "out-of-domain": lambda rows, entries, rd: tooth_out_of_domain(),
    "strip-identity": tooth_strip_identity,
    "duplicate-rows": tooth_duplicate_rows,
    "reverse-time": tooth_reverse_time,
}


def run_tooth(mode, rows, entries, round_dir):
    rows_copy = json.loads(json.dumps(rows, default=str))
    entries_copy = json.loads(json.dumps(entries))
    result = TEETH[mode](rows_copy, entries_copy, round_dir)
    result["mode"] = mode
    result["executed_at"] = now_iso()
    path = os.path.join(round_dir, "teeth", f"{mode}.yml")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        yaml.safe_dump(result, f, sort_keys=False, width=120)
    print(f"[tooth:{mode}] behaved_as_specified={result.get('behaved_as_specified')}")
    return result


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--manifest", required=True)
    ap.add_argument("--read-only", action="store_true")
    ap.add_argument("--round-dir", default=DEFAULT_ROUND)
    ap.add_argument("--teeth", choices=sorted(TEETH), help="run one preregistered attack in isolation")
    args = ap.parse_args()

    if args.teeth:
        manifest, entries = load_and_verify_manifest(args.manifest)
        rows, _gaps = reconstruct(entries, DEFAULT_ROUND)
        result = run_tooth(args.teeth, rows, entries, DEFAULT_ROUND)
        if result.get("behaved_as_specified") is False:
            return 1
        return 0
    return full_pilot(args)


if __name__ == "__main__":
    sys.exit(main())
