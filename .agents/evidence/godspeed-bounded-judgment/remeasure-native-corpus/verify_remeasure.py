#!/usr/bin/env python3
"""Verification gate for the native-corpus remeasurement (run from the
sea-rs repo root; stdlib + PyYAML only).

Independently re-derives, from the frozen sources and preserved raw outputs:

  R1  every pinned state database re-hashes to its corpus-manifest pin
      (fail-closed);
  R2  mechanical truth re-derived from each DB by the frozen addendum-3 rule
      equals the manifest and results truth classes; frozen class counts hold;
  R3  every preserved raw output re-parses (independent parser, T16 parse
      semantics) and reproduces the recorded labels/parsed objects;
  R4  arm1 leakage assertions hold on the preserved prompt files AND on the
      prompt re-derived from the DB (T16 PROMPT hash + records blob);
  R5  arm2/arm2b prompts re-derive from the pinned round-2 template prefix
      and the preserved surfaces; template hashes match the results file;
  R6  surfaces re-build (frozen T23 builder) byte-identically to the
      preserved surface files;
  R7  accuracies, Clopper-Pearson CIs, majority aggregation, stability,
      McNemar, comparison-7 error list and arm2b MAE/hit-rate recompute from
      the results rows and equal the recorded scoring sections;
  R8  payment counters recompute from the per-call records;
  R9  manifest/decision-log ordering: RM-02 records the exact manifest
      sha256, the manifest predates the first preserved provider call
      (mtime + recorded started_at_utc), the decision-log append postdates
      the manifest;
  R10 fail-closed self-manifest hash checks: every file in
      evidence-manifest.yml hashes to its recorded sha256.

Exit 0 only if every check passes. The gate never writes (except nothing);
it is frozen before execution and never overwritten (corrections use a
round-2 filename).
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import datetime
import glob
import hashlib
import json
import math
import os
import re
import sqlite3

import yaml

REPO = os.getcwd()
BASE = os.path.join(
    REPO, ".agents/evidence/godspeed-bounded-judgment")
OUT_DIR = os.path.join(BASE, "remeasure-native-corpus")
T16_DIR = os.path.join(BASE, "T16")
R2_DIR = os.path.join(BASE, "T23", "round-2")

MANIFEST_PATH = os.path.join(OUT_DIR, "corpus-manifest.yaml")
RESULTS_PATH = os.path.join(OUT_DIR, "remeasure-results.json")
GATE_MANIFEST_PATH = os.path.join(OUT_DIR, "evidence-manifest.yml")
DECISIONS_PATH = os.path.join(BASE, "decisions.yml")
PREREG_REL = ".agents/preregistrations/godspeed-bounded-judgment-native-corpus-remeasure.prereg.yaml"
PREREG_SHA256 = (
    "8142c0a52537f9b600d4853afd4871d5ec04dc29acffba58fe67c7f7616d1f6e")
DECISION_ID = "D-2026-09-19-RM-02"

ARM1_DOMAIN = ["settled", "not_settled", "insufficient_evidence"]
ARM2_DOMAIN = ["settled_accepted", "verification_failed_nonsettlement",
               "budget_exhausted", "execution_failed",
               "insufficient_evidence"]
TRUTH_MAP_ARM1 = {"settled_accepted": "settled",
                  "budget_exhausted": "not_settled",
                  "verification_failed_nonsettlement": "not_settled"}
TEMPLATE_SPLIT = "Corrected observation record:\n"

FAILURES = []


def check(cond, msg):
    if cond:
        print(f"[gate]   ok: {msg}")
    else:
        FAILURES.append(msg)
        print(f"[gate] FAIL: {msg}", flush=True)
    return bool(cond)


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_text(t):
    return hashlib.sha256(t.encode("utf-8")).hexdigest()


def load_yaml(path):
    with open(path) as f:
        return yaml.safe_load(f)


def load_json(path):
    with open(path) as f:
        return json.load(f)


def close(a, b, tol=1e-9):
    if a is None or b is None:
        return a is None and b is None
    return abs(float(a) - float(b)) <= tol


def close_ci(a, b, tol=1e-9):
    if not isinstance(a, list) or not isinstance(b, list) or len(a) != 2 \
            or len(b) != 2:
        return False
    return close(a[0], b[0], tol) and close(a[1], b[1], tol)


# ---------------- independent stats (stdlib only) --------------------------

def _betacf(a, b, x):
    maxit, eps, fpmin = 200, 3e-16, 1e-300
    qab, qap, qam = a + b, a + 1.0, a - 1.0
    c = 1.0
    d = 1.0 - qab * x / qap
    if abs(d) < fpmin:
        d = fpmin
    d = 1.0 / d
    h = d
    for m in range(1, maxit + 1):
        m2 = 2 * m
        aa = m * (b - m) * x / ((qam + m2) * (a + m2))
        d = 1.0 + aa * d
        if abs(d) < fpmin:
            d = fpmin
        c = 1.0 + aa / c
        if abs(c) < fpmin:
            c = fpmin
        d = 1.0 / d
        h *= d * c
        aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2))
        d = 1.0 + aa * d
        if abs(d) < fpmin:
            d = fpmin
        c = 1.0 + aa / c
        if abs(c) < fpmin:
            c = fpmin
        d = 1.0 / d
        de = d * c
        h *= de
        if abs(de - 1.0) < eps:
            break
    return h


def _betainc(a, b, x):
    if x <= 0:
        return 0.0
    if x >= 1:
        return 1.0
    lb = math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b)
    front = math.exp(lb + a * math.log(x) + b * math.log(1.0 - x))
    if x < (a + 1.0) / (a + b + 2.0):
        return front * _betacf(a, b, x) / a
    return 1.0 - math.exp(lb + b * math.log(1.0 - x) + a * math.log(x)) \
        * _betacf(b, a, 1.0 - x) / b


def _betainc_inv(p, a, b):
    lo, hi = 0.0, 1.0
    for _ in range(200):
        mid = (lo + hi) / 2.0
        if _betainc(a, b, mid) < p:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2.0


def cp_ci(k, n, alpha=0.05):
    if n == 0:
        return None, None
    lo = 0.0 if k == 0 else _betainc_inv(alpha / 2.0, k, n - k + 1)
    hi = 1.0 if k == n else _betainc_inv(1.0 - alpha / 2.0, k + 1, n - k)
    return lo, hi


def mcnemar_exact(b, c):
    n = b + c
    if n == 0:
        return 1.0
    return min(1.0, 2.0 * sum(math.comb(n, k)
                              for k in range(0, min(b, c) + 1)) / 2 ** n)


# ---------------- independent parsers (T16 parse semantics) ----------------

def parse_domain(text, answer_key, domain, with_estimate=False):
    m = re.search(r"\{.*\}", text, re.DOTALL)
    if not m:
        return {"status": "schema_failure",
                "reason": "no JSON object in provider output"}
    try:
        obj = json.loads(m.group(0))
    except json.JSONDecodeError as e:
        return {"status": "schema_failure", "reason": f"invalid JSON: {e}"}
    answer = obj.get(answer_key)
    if answer not in domain:
        return {"status": "out_of_domain", "raw_answer": answer,
                "reason": f"answer {answer!r} not in the frozen domain"}
    d = {}
    for label in domain:
        try:
            d[label] = max(0.0, min(1.0, float(
                (obj.get("distribution") or {}).get(label, 0.0))))
        except (TypeError, ValueError):
            d[label] = 0.0
    total = sum(d.values())
    if total > 0:
        d = {k: v / total for k, v in d.items()}
    try:
        conf = max(0.0, min(1.0, float(obj.get("confidence", 0) or 0)))
    except (TypeError, ValueError):
        conf = None
    out = {"status": "ok", "answer": answer, "confidence": conf,
           "distribution": d, "reason": str(obj.get("reason", ""))[:300]}
    if with_estimate:
        est = obj.get("settled_units_estimate", "__absent__")
        if est == "__absent__":
            out["settled_units_estimate"] = None
            out["estimate_status"] = "absent"
        elif est is None:
            out["settled_units_estimate"] = None
            out["estimate_status"] = "null"
        elif isinstance(est, bool) or not isinstance(est, int):
            out["settled_units_estimate"] = None
            out["estimate_status"] = "invalid"
            out["estimate_raw"] = str(est)[:100]
        else:
            out["settled_units_estimate"] = est
            out["estimate_status"] = "valid"
    return out


def parsed_equal(a, b):
    """Structural equality of two parsed objects with float tolerance."""
    if not isinstance(a, dict) or not isinstance(b, dict):
        return False
    if set(a.keys()) != set(b.keys()):
        return False
    for k in a:
        va, vb = a[k], b[k]
        if isinstance(va, dict) and isinstance(vb, dict):
            if not parsed_equal(va, vb):
                return False
        elif isinstance(va, float) or isinstance(vb, float):
            if not close(va, vb, 1e-12):
                return False
        else:
            if va != vb:
                return False
    return True


# ---------------- mechanical truth (frozen addendum-3 rule) ----------------

def jbody(payload):
    try:
        d = json.loads(payload)
    except json.JSONDecodeError:
        return {}
    return d if isinstance(d, dict) else {}


def derive_truth(db_path):
    if not os.path.isfile(db_path):
        return "execution_failed", None, None, False
    conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    try:
        ev = [(k, jbody(p)) for k, p in conn.execute(
            "select kind, payload from event_log order by offset")]
    finally:
        conn.close()
    states = [b.get("state") for k, b in ev if k == "run_terminal"]
    settlements = sum(1 for k, _ in ev if k == "settlement_committed")
    raw_state = states[0] if len(states) == 1 else None
    if raw_state == "settled":
        terminal = "settled"
    elif raw_state == "budget_exhausted":
        terminal = "budget_exhausted"
    elif isinstance(raw_state, str) and raw_state != "":
        terminal = "other-nonsettlement"
    else:
        terminal = "run_error"
    refs = []
    for k, b in ev:
        if k == "evidence_produced":
            r = b.get("evidence", "")
            if r.startswith("sha256:"):
                r = r[7:]
                if r not in refs:
                    refs.append(r)
    art_dir = os.path.join(os.path.dirname(os.path.dirname(db_path)),
                           "evidence")
    failing = False
    for ref in refs:
        obj = os.path.join(art_dir, "objects", ref[:2], ref[2:4], ref)
        if not os.path.isfile(obj) or sha256_file(obj) != ref:
            continue
        try:
            with open(obj, encoding="utf-8", errors="replace") as f:
                content = json.load(f)
        except json.JSONDecodeError:
            continue
        failures = content.get("failures") \
            if isinstance(content, dict) else None
        if (isinstance(failures, (int, float))
                and not isinstance(failures, bool) and failures > 0):
            failing = True
    c1 = terminal not in ("settled", "run_error")
    c2 = settlements == 0
    c3 = failing
    if terminal == "settled":
        label = "settled_accepted"
    elif terminal == "run_error":
        label = "execution_failed"
    elif c1 and c2 and c3:
        label = "verification_failed_nonsettlement"
    else:
        label = "budget_exhausted"
    return label, terminal, settlements, True


def majority_of(labels):
    ok = [l for l in labels if l]
    tally = {}
    for l in ok:
        tally[l] = tally.get(l, 0) + 1
    if not ok:
        return "insufficient_evidence", tally
    top = max(tally.values())
    winners = [l for l, c in tally.items() if c == top]
    if len(winners) == 1 and top >= 2:
        return winners[0], tally
    return "insufficient_evidence", tally


def slice_stats(entries):
    answered = [e for e in entries if e["label"]]
    k = sum(1 for e in answered if e["label"] == e["truth"])
    n = len(answered)
    lo, hi = cp_ci(k, n)
    return {"n": len(entries), "n_answered": n, "k_correct": k,
            "accuracy": (k / n) if n else None, "ci95": [lo, hi]}


def check_slice(tag, recorded, entries):
    """Compare a recorded slice against independently recomputed stats."""
    exp = slice_stats(entries)
    ok = True
    for field in ("n", "n_answered", "k_correct"):
        if recorded.get(field) != exp[field]:
            check(False, f"{tag}: {field} recorded {recorded.get(field)} "
                         f"!= recomputed {exp[field]}")
            ok = False
    if not close(recorded.get("accuracy"), exp["accuracy"]):
        check(False, f"{tag}: accuracy {recorded.get('accuracy')} != "
                     f"recomputed {exp['accuracy']}")
        ok = False
    if not close_ci(recorded.get("ci95"), exp["ci95"]):
        check(False, f"{tag}: ci95 {recorded.get('ci95')} != recomputed "
                     f"{exp['ci95']}")
        ok = False
    if ok:
        check(True, f"{tag}: slice stats reproduce (n={exp['n']}, "
                    f"n_answered={exp['n_answered']}, k={exp['k_correct']})")
    return ok


def main() -> int:
    print("[gate] R0: file presence", flush=True)
    for p in (MANIFEST_PATH, RESULTS_PATH, DECISIONS_PATH,
              GATE_MANIFEST_PATH):
        if not check(os.path.isfile(p), f"present: {os.path.relpath(p, REPO)}"):
            print("[gate] gate cannot proceed; exit 1", flush=True)
            return 1
    manifest = load_yaml(MANIFEST_PATH)
    results = load_json(RESULTS_PATH)
    rows_m = manifest["rows"]
    rows_r = results["rows"]

    # ---------------- R-prereg: prereg identity ---------------------------
    check(sha256_file(os.path.join(REPO, PREREG_REL)) == PREREG_SHA256,
          "prereg sha256 matches the frozen digest")
    check(results["prereg"]["sha256"] == PREREG_SHA256,
          "results file pins the frozen prereg sha256")

    # ---------------- R1: DB pins -----------------------------------------
    print("[gate] R1: DB pins", flush=True)
    ok_all = True
    for row in rows_m:
        ok = os.path.isfile(row["state_db_path"]) and \
            sha256_file(row["state_db_path"]) == row["db_sha256"]
        ok_all = ok_all and ok
    check(ok_all, f"all {len(rows_m)} pinned state databases re-hash to "
                  f"their manifest pins")

    # ---------------- R2: mechanical truth --------------------------------
    print("[gate] R2: mechanical truth", flush=True)
    ok_all = True
    for row in rows_m:
        derived, terminal, settlements, _ = derive_truth(row["state_db_path"])
        ok = (derived == row["truth_class"]
              and settlements == row["actual_settled_units"])
        ok_all = ok_all and ok
        if not ok:
            print(f"[gate] FAIL: {row['id']}: derived {derived} != manifest "
                  f"{row['truth_class']}", flush=True)
    check(ok_all, "mechanical truth re-derivation matches every manifest row")
    counts = {}
    for row in rows_m:
        counts[row["truth_class"]] = counts.get(row["truth_class"], 0) + 1
    check(counts == {"settled_accepted": 6, "budget_exhausted": 5,
                     "verification_failed_nonsettlement": 4},
          f"frozen class counts hold: {counts}")
    check(len(rows_r) == 15 and
          all(r["truth_class"] == next(m["truth_class"] for m in rows_m
                                       if m["id"] == r["id"])
              for r in rows_r),
          "results truth classes equal the manifest truth classes")
    check(all(r["truth_mapped_arm1"] == TRUTH_MAP_ARM1[r["truth_class"]]
              for r in rows_r),
          "arm1 truth mapping is the frozen mapped truth on every row")

    # ---------------- R3+R4: arm1 raw re-parse + leakage -------------------
    print("[gate] R3/R4: arm1 raw re-parse + leakage assertions", flush=True)
    sys.path.insert(0, T16_DIR)
    import run_remeasure as t16  # read-only import of the frozen T16 harness
    check(results["templates"]["arm1"]["sha256"] == sha256_text(t16.PROMPT),
          "recorded arm1 template hash == the T16 PROMPT (verbatim import)")
    ok_parse = ok_leak = ok_rebuild = True
    for r in rows_r:
        a = r["arm1"]
        stem = f"arm1-{r['id']}"
        if a["provider_used"] is None:
            # total call-slot failure: no raw output exists; the recorded
            # error files and the typed failure status are the evidence
            err_ok = all(os.path.isfile(cl.get("error_path", "/nonexistent"))
                         for cl in a["calls"] if cl["status"]
                         not in ("raw",) and cl.get("error_path"))
            if not err_ok or os.path.isfile(
                    os.path.join(OUT_DIR, "arm1", "provider-raw",
                                 f"{stem}-deepseek.raw.txt")):
                check(False, f"{r['id']}: total-failure slot not recorded "
                             f"consistently")
                ok_parse = False
            continue
        if a["provider_used"] == "muse-spark-free":
            raw_path = os.path.join(OUT_DIR, "arm1", "provider-raw",
                                    f"{stem}-muse.raw.txt")
        else:
            raw_path = os.path.join(OUT_DIR, "arm1", "provider-raw",
                                    f"{stem}-deepseek.raw.txt")
        if not os.path.isfile(raw_path):
            check(False, f"{r['id']}: preserved raw missing: {raw_path}")
            ok_parse = False
            continue
        with open(raw_path) as f:
            raw = f.read()
        parsed = parse_domain(raw, "answer", ARM1_DOMAIN)
        if not parsed_equal(parsed, a["parsed"]):
            check(False, f"{r['id']}: arm1 re-parse differs from the "
                         f"recorded parsed object")
            ok_parse = False
        expect_label = parsed["answer"] if parsed["status"] == "ok" else None
        if expect_label != a["label"]:
            check(False, f"{r['id']}: arm1 recorded label {a['label']} != "
                         f"re-parsed {expect_label}")
            ok_parse = False
        if a["label"] is not None:
            if a["correct"] != (a["label"] == r["truth_mapped_arm1"]):
                check(False, f"{r['id']}: arm1 correct flag inconsistent")
                ok_parse = False
        # leakage + prompt rebuild from the DB (exact T16 assembly)
        conn = sqlite3.connect(f"file:{r['state_db_path']}?mode=ro", uri=True)
        conn.row_factory = sqlite3.Row
        invs = []
        for (p,) in conn.execute(
                "select payload from records where "
                "record_type='role_invocation' order by seq"):
            invs.append({"result_document": json.loads(p)})
        cors = conn.execute("select count(*) from records where "
                            "record_type='correction_selection'"
                            ).fetchone()[0]
        conn.close()
        blob = json.dumps({"invocations": invs, "corrections": cors})[:12000]
        prompt = t16.PROMPT.format(records=blob)
        pfile = os.path.join(OUT_DIR, "arm1", "provider-raw",
                             f"{stem}.prompt.txt")
        with open(pfile) as f:
            pdisk = f.read()
        if "run_terminal" in pdisk or "settlement_committed" in pdisk:
            check(False, f"{r['id']}: LEAKAGE in preserved arm1 prompt")
            ok_leak = False
        if pdisk != prompt:
            check(False, f"{r['id']}: preserved arm1 prompt != prompt "
                         f"re-derived from the DB")
            ok_rebuild = False
        if a["prompt_sha256"] != sha256_text(prompt):
            check(False, f"{r['id']}: arm1 prompt hash mismatch")
            ok_rebuild = False
        baseline = t16.face_value({"invocations": invs,
                                   "corrections": cors})
        if baseline != a["baseline_label"]:
            check(False, f"{r['id']}: baseline label mismatch")
            ok_rebuild = False
    check(ok_parse, "every preserved arm1 raw output re-parses to the "
                    "recorded label/parsed object")
    check(ok_leak, "arm1 leakage assertion holds on every preserved prompt "
                   "(no run_terminal/settlement_committed vocabulary)")
    check(ok_rebuild, "every arm1 prompt re-derives from the DB + T16 PROMPT "
                      "and hashes to the recorded value; face-value baseline "
                      "labels reproduce")

    # ---------------- R5+R6: arm2 template + surfaces ----------------------
    print("[gate] R5/R6: arm2 template + surfaces", flush=True)
    prompts = sorted(glob.glob(os.path.join(
        R2_DIR, "provider-raw", "r2-judgment-*-primary.prompt.txt")))
    prefixes = set()
    for pf in prompts:
        with open(pf) as f:
            txt = f.read()
        prefix, blob = txt.split(TEMPLATE_SPLIT)
        m = re.search(r"r2-judgment-(probe-[a-z0-9-]+)-primary",
                      os.path.basename(pf))
        surface = load_json(os.path.join(
            R2_DIR, "surfaces", f"{m.group(1)}.json"))["surface"]
        if json.dumps(surface, sort_keys=True, indent=1) == blob:
            prefixes.add(prefix)
    check(len(prefixes) == 1,
          "the 12 preserved round-2 prompts share one template prefix")
    prefix = next(iter(prefixes))
    check(results["templates"]["arm2"]["template_prefix_sha256"]
          == sha256_text(prefix),
          "recorded arm2 template prefix hash matches the preserved round-2 "
          "template")
    arm2b_old = ('JSON shape: {"label": "<one label from the answer '
                 'domain>"}\n')
    arm2b_new = ('JSON shape: {"label": "<one label from the answer '
                 'domain>", "settled_units_estimate": <integer or null>}\n')
    arm2b_note = ('Exploratory addendum (clearly marked exploratory, outside '
                  'the scored label domain): "settled_units_estimate" is '
                  'your estimate of the number of units that will have '
                  "committed settlement in this run's recorded consequence; "
                  'give an integer, or null if you cannot estimate.\n')
    check(results["templates"]["arm2b"]["template_prefix_sha256"]
          == sha256_text(prefix.replace(arm2b_old, arm2b_new + arm2b_note)),
          "recorded arm2b template prefix hash matches the recorded "
          "extension of the arm2 template")

    sys.path.insert(0, os.path.join(BASE, "T23"))
    sys.path.insert(0, R2_DIR)
    import build_surfaces as bs

    def resolve_obs(row):
        source = row["source"]
        want = {"scenario": row["probe"]["scenario"],
                "max_rounds": row["probe"]["max_rounds"]}
        if source.startswith("t23-iter"):
            it = source.split("t23-iter", 1)[1]
            idir = os.path.join(BASE, "T23", "route", f"iteration-{it}")
            observations = load_json(os.path.join(idir,
                                                  "probe-observations.json"))
            obs = [o for o in observations
                   if o.get("bounded_input") == want][0]
            meta = load_yaml(os.path.join(idir, "probe-run-meta.yaml"))
            run_meta = [m for m in meta.get("probes", [])
                        if m.get("probe_id") == obs["probe_id"]][0]
            return obs, run_meta
        if source == "t24-replay":
            rr = load_yaml(os.path.join(BASE, "T24", "replay-record.yaml"))
            return rr["observation"], {
                "wall_seconds": rr["run"]["wall_seconds"]}
        if source == "t24-tooth-t7":
            teeth = load_yaml(os.path.join(BASE, "T24", "teeth.yaml"))
            t7 = [t for t in teeth["teeth"]
                  if t["id"] == "T7-invalid-inverse-case"][0]
            return t7["attack_observation"], {
                "wall_seconds": t7["run_meta"]["wall_seconds"]}
        if source == "t25-negative":
            neg = load_yaml(os.path.join(BASE, "T25",
                                         "matched-pair.yaml"))["negative"]
            return {
                "probe_id": neg["probe_id"],
                "candidate_id": "t25-negative-r1",
                "correlation_id": "t25-negative-r1",
                "operation": "run_case",
                "bounded_input": neg["bounded_input"],
                "terminal_class": neg["terminal_class"],
                "settlement_committed_count":
                    neg["settlement_committed_count"],
                "state_db_path": neg["state_db_path"],
                "state_db_sha256": neg["state_db_sha256"],
                "mechanism": "t15-frozen-case-execution",
                "started_at_utc": neg["run"]["started_at_utc"],
                "finished_at_utc": neg["run"]["finished_at_utc"],
            }, {"wall_seconds": neg["run"]["wall_seconds"]}
        raise SystemExit(f"unknown source {source}")

    ok_surf = ok_prompt = ok_raw2 = ok_maj = True
    for r in rows_r:
        a2 = r["arm2"]
        row_m = next(m for m in rows_m if m["id"] == r["id"])
        obs, run_meta = resolve_obs(row_m)
        record = bs.build_surface(obs, run_meta)
        surface = record["surface"]
        blob = json.dumps(surface, sort_keys=True, indent=1)
        if record.get("status") != "ok" or \
                sha256_text(blob) != a2["surface_sha256"]:
            check(False, f"{r['id']}: surface re-build differs from the "
                         f"preserved surface")
            ok_surf = False
        with open(a2["surface_path"]) as f:
            saved = json.load(f)
        if json.dumps(saved["surface"], sort_keys=True, indent=1) != blob:
            check(False, f"{r['id']}: saved surface file != re-built surface")
            ok_surf = False
        prompt = prefix + TEMPLATE_SPLIT + blob
        with open(a2["prompt_path"]) as f:
            if f.read() != prompt:
                check(False, f"{r['id']}: arm2 prompt file != template + "
                             f"re-built surface")
                ok_prompt = False
        if a2["prompt_sha256"] != sha256_text(prompt):
            check(False, f"{r['id']}: arm2 prompt hash mismatch")
            ok_prompt = False
        # raw re-parse + majority recompute
        labels = []
        for s in a2["samples"]:
            if s["raw_path"] is None:
                # total call-slot failure: no raw output exists; the recorded
                # error files and the typed failure status are the evidence
                err_ok = all(
                    os.path.isfile(cl.get("error_path", "/nonexistent"))
                    for cl in s["calls"]
                    if not cl["status"].startswith("raw")
                    and cl.get("error_path"))
                if not err_ok:
                    check(False, f"{r['id']} sample {s['sample']}: "
                                 f"total-failure slot not recorded "
                                 f"consistently")
                    ok_raw2 = False
                labels.append(None)
                continue
            if not os.path.isfile(s["raw_path"]):
                check(False, f"{r['id']} sample {s['sample']}: raw missing")
                ok_raw2 = False
                labels.append(None)
                continue
            with open(s["raw_path"]) as f:
                parsed = parse_domain(f.read(), "label", ARM2_DOMAIN)
            if not parsed_equal(parsed, s["parsed"]):
                check(False, f"{r['id']} sample {s['sample']}: re-parse "
                             f"differs from recorded")
                ok_raw2 = False
            expect = parsed["answer"] if parsed["status"] == "ok" else None
            if expect != s["label"]:
                check(False, f"{r['id']} sample {s['sample']}: label "
                             f"mismatch")
                ok_raw2 = False
            labels.append(expect)
        maj, tally = majority_of(labels)
        if maj != a2["majority_label"] or tally != a2["majority_tally"]:
            check(False, f"{r['id']}: majority/tally mismatch")
            ok_maj = False
        if a2["majority_correct"] != (maj == r["truth_class"]):
            check(False, f"{r['id']}: majority_correct flag wrong")
            ok_maj = False
        ok_labels = [l for l in labels if l]
        modal = max(tally.values()) if tally else 0
        if not close(a2["agreement_rate"], modal / 3.0, 1e-12):
            check(False, f"{r['id']}: agreement rate mismatch")
            ok_maj = False
        if a2["all_three_agree"] != (len(ok_labels) == 3
                                     and len(set(ok_labels)) == 1):
            check(False, f"{r['id']}: all_three_agree flag wrong")
            ok_maj = False
        # arm2b raw re-parse
        ab = r["arm2b"]
        if ab["provider_used"] is None:
            continue  # total call-slot failure: recorded via typed status
        if ab["provider_used"] == "muse-spark-free":
            raw_path = os.path.join(OUT_DIR, "arm2b", "provider-raw",
                                    f"arm2b-{r['id']}-muse.raw.txt")
        else:
            raw_path = os.path.join(OUT_DIR, "arm2b", "provider-raw",
                                    f"arm2b-{r['id']}-deepseek.raw.txt")
        with open(raw_path) as f:
            parsed_b = parse_domain(f.read(), "label", ARM2_DOMAIN,
                                    with_estimate=True)
        if not parsed_equal(parsed_b, ab["parsed"]):
            check(False, f"{r['id']}: arm2b re-parse differs")
            ok_raw2 = False
        if ab["estimate_status"] == "valid":
            if ab["abs_error"] != abs(ab["estimate"]
                                      - ab["actual_settled_units"]):
                check(False, f"{r['id']}: arm2b abs_error wrong")
                ok_raw2 = False
    check(ok_surf, "every arm2 surface re-builds byte-identically (frozen "
                   "builder) and matches the saved surface file")
    check(ok_prompt, "every arm2/arm2b prompt re-derives from the pinned "
                     "template prefix + re-built surface blob")
    check(ok_raw2, "every preserved arm2/arm2b raw output re-parses to the "
                   "recorded labels/estimates")
    check(ok_maj, "majority aggregation, tallies and agreement rates "
                  "reproduce on every row")

    # ---------------- R7: scoring recompute --------------------------------
    print("[gate] R7: scoring recompute", flush=True)
    a1s = results["arm1_scoring"]
    prim = [r for r in rows_r if r["arm1"]["provider_used"]
            == "deepseek-clinepass"]
    subs = [r for r in rows_r if r["arm1"]["provider_used"]
            == "muse-spark-free"]
    check_slice("arm1 deepseek", a1s["deepseek_slice"],
                [{"label": r["arm1"]["label"],
                  "truth": r["truth_mapped_arm1"]} for r in prim])
    check_slice("arm1 substituted (never pooled)",
                a1s["substituted_slice_never_pooled"],
                [{"label": r["arm1"]["label"],
                  "truth": r["truth_mapped_arm1"]} for r in subs])
    paired = [r for r in prim if r["arm1"]["correct"] is not None]
    b = sum(1 for r in paired
            if r["arm1"]["correct"] and not r["arm1"]["baseline_correct"])
    c = sum(1 for r in paired
            if not r["arm1"]["correct"] and r["arm1"]["baseline_correct"])
    mrec = a1s["mcnemar_learned_vs_baseline"]
    check(mrec["n_paired"] == len(paired)
          and mrec["learned_right_baseline_wrong"] == b
          and mrec["learned_wrong_baseline_right"] == c
          and close(mrec["p_exact_two_sided"], mcnemar_exact(b, c), 1e-12),
          f"McNemar exact reproduces (b={b}, c={c}, "
          f"p={mcnemar_exact(b, c)})")
    base_k = sum(1 for r in prim if r["arm1"]["baseline_correct"])
    blo, bhi = cp_ci(base_k, len(prim))
    brec = a1s["baseline_face_value_deepseek_slice"]
    check(brec["k_correct"] == base_k and brec["n"] == len(prim)
          and close(brec["accuracy"], base_k / len(prim) if prim else None)
          and close_ci(brec["ci95"], [blo, bhi]),
          "face-value baseline accuracy + CI reproduce")

    a2s = results["arm2_scoring"]
    a2p = [r for r in rows_r if all(s["provider_used"] == "deepseek-clinepass"
                                    for s in r["arm2"]["samples"])]
    a2sub = [r for r in rows_r
             if not all(s["provider_used"] == "deepseek-clinepass"
                        for s in r["arm2"]["samples"])]
    check_slice("arm2 single-sample-1 deepseek",
                a2s["single_sample1_deepseek_slice"],
                [{"label": r["arm2"]["single1_label"],
                  "truth": r["truth_class"]} for r in a2p])
    check_slice("arm2 majority deepseek", a2s["majority_deepseek_slice"],
                [{"label": r["arm2"]["majority_label"],
                  "truth": r["truth_class"]} for r in a2p])
    stab = a2s["stability"]
    mean_rate = (sum(r["arm2"]["agreement_rate"] for r in a2p)
                 / len(a2p)) if a2p else None
    check(close(stab["mean_agreement_rate_deepseek_rows"], mean_rate)
          and stab["n_deepseek_rows"] == len(a2p)
          and stab["rows_all_three_agree"] == sum(
              1 for r in a2p if r["arm2"]["all_three_agree"]),
          "stability (per-row agreement) reproduces")
    mvc = a2s["majority_vs_single_material_change"]
    check(close(mvc["accuracy_single"],
                a2s["single_sample1_deepseek_slice"]["accuracy"])
          and close(mvc["accuracy_majority"],
                    a2s["majority_deepseek_slice"]["accuracy"])
          and mvc["rows_changed_label"] == [
              {"id": r["id"], "single1": r["arm2"]["single1_label"],
               "majority": r["arm2"]["majority_label"],
               "truth": r["truth_class"]}
              for r in a2p
              if r["arm2"]["single1_label"] != r["arm2"]["majority_label"]],
          "majority-vs-single comparison reproduces")
    err_expected = [
        {"id": r["id"], "truth": r["truth_class"],
         "majority_label": r["arm2"]["majority_label"],
         "majority_tally": r["arm2"]["majority_tally"],
         "raw_paths": [s["raw_path"] for s in r["arm2"]["samples"]]}
        for r in a2p if r["arm2"]["majority_label"] != r["truth_class"]]
    check(a2s["comparison7_error_list_majority"] == err_expected,
          f"comparison-7 error list reproduces ({len(err_expected)} rows)")
    lying = [r for r in rows_r if r["family"] == "adversarial"]
    check(a2s["historical_freshness_lying_rows"] == [
        {"id": r["id"], "probe_id": r["probe_id"],
         "truth": r["truth_class"],
         "fresh_majority_label": r["arm2"]["majority_label"],
         "fresh_labels": r["arm2"]["labels_ok"],
         "preserved_round2_label": r["historical_round2_label"],
         "fresh_matches_preserved": (
             r["arm2"]["majority_label"] == r["historical_round2_label"])}
        for r in lying],
        "historical freshness block reproduces (4 lying rows)")

    b2s = results["arm2b_scoring"]

    def arm2b_expected(entry_rows):
        valid = [r for r in entry_rows
                 if r["arm2b"]["estimate_status"] == "valid"]
        n_valid = len(valid)
        mae = (sum(r["arm2b"]["abs_error"] for r in valid) / n_valid) \
            if n_valid else None
        hits = sum(1 for r in valid
                   if r["arm2b"]["estimate"]
                   == r["arm2b"]["actual_settled_units"])
        lo, hi = cp_ci(hits, n_valid)
        counts = {}
        for r in entry_rows:
            st = r["arm2b"]["estimate_status"]
            counts[st] = counts.get(st, 0) + 1
        return {"n_rows": len(entry_rows), "n_valid_estimates": n_valid,
                "mae": mae, "exact_hits": hits,
                "exact_hit_rate": (hits / n_valid) if n_valid else None,
                "exact_hit_ci95": [lo, hi],
                "estimate_status_counts": counts}

    bprim = [r for r in rows_r
             if r["arm2b"]["provider_used"] == "deepseek-clinepass"]
    for tag, rec, exp_rows in (
            ("arm2b all rows", b2s["deepseek_slice_all_rows"], bprim),
            ("arm2b honest family", b2s["deepseek_slice_honest_family"],
             [r for r in bprim if r["family"] == "honest"])):
        exp = arm2b_expected(exp_rows)
        ok = all(rec.get(kk) == exp[kk] for kk in
                 ("n_rows", "n_valid_estimates", "exact_hits",
                  "estimate_status_counts"))
        ok = ok and close(rec.get("mae"), exp["mae"], 1e-12) \
            and close(rec.get("exact_hit_rate"), exp["exact_hit_rate"], 1e-12) \
            and close_ci(rec.get("exact_hit_ci95"), exp["exact_hit_ci95"])
        check(ok, f"{tag}: MAE {rec.get('mae')}, exact hits "
                  f"{rec.get('exact_hits')}/{rec.get('n_valid_estimates')} "
                  f"reproduce")

    # ---------------- R8: payment counters ---------------------------------
    print("[gate] R8: payment counters", flush=True)
    pay = results["payment"]
    for arm, key in (("arm1", "arm1"), ("arm2", "arm2"), ("arm2b", "arm2b")):
        d_calls = d_failed = d_sub = 0
        wall = 0.0
        for r in rows_r:
            if arm == "arm2":
                items = [s["calls"] for s in r["arm2"]["samples"]]
            else:
                items = [r[arm]["calls"]]
            for calls in items:
                for cl in calls:
                    if cl.get("duration_seconds"):
                        wall += cl["duration_seconds"]
                    if cl["provider"] == "deepseek-clinepass":
                        if cl["status"].startswith("raw"):
                            d_calls += 1
                        else:
                            d_failed += 1
                    elif cl["provider"] == "muse-spark-free" \
                            and cl["status"].startswith("raw"):
                        d_sub += 1
        rec = pay[key]
        check(rec["deepseek_calls"] == d_calls
              and rec["deepseek_failed"] == d_failed
              and rec["muse_substitutions"] == d_sub
              and close(pay["wall_seconds_by_arm"][arm], round(wall, 3),
                        0.01),
              f"{arm}: payment counters reproduce (deepseek {d_calls}, "
              f"failed {d_failed}, muse {d_sub}, wall {round(wall, 3)}s)")
    tot = sum(pay["total_calls_by_arm"][a] for a in
              ("arm1", "arm2", "arm2b"))
    check(pay["learned_layer_total_provider_calls"] == tot
          and pay["mechanical_layer_provider_calls"] == 0,
          f"learned-layer total calls {tot}; mechanical layer 0 calls")

    # ---------------- R9: ordering -----------------------------------------
    print("[gate] R9: manifest/decision ordering", flush=True)
    with open(DECISIONS_PATH) as f:
        dtext = f.read()
    idx = dtext.index(DECISION_ID)
    seg = dtext[idx:idx + 2000]
    m = re.search(r'corpus_manifest_sha256: "([0-9a-f]{64})"', seg)
    check(m is not None
          and m.group(1) == sha256_file(MANIFEST_PATH),
          "RM-02 records exactly the corpus manifest sha256")
    raw_mtimes = []
    started = []
    for r in rows_r:
        for armkey, calls_list in (
                ("arm1", r["arm1"]["calls"]),
                ("arm2b", r["arm2b"]["calls"])):
            for cl in calls_list:
                if cl.get("raw_path") and os.path.isfile(cl["raw_path"]):
                    raw_mtimes.append(os.path.getmtime(cl["raw_path"]))
                if cl.get("started_at_utc"):
                    started.append(cl["started_at_utc"])
        for s in r["arm2"]["samples"]:
            for cl in s["calls"]:
                if cl.get("raw_path") and os.path.isfile(cl["raw_path"]):
                    raw_mtimes.append(os.path.getmtime(cl["raw_path"]))
                if cl.get("started_at_utc"):
                    started.append(cl["started_at_utc"])
    man_mtime = os.path.getmtime(MANIFEST_PATH)
    dec_mtime = os.path.getmtime(DECISIONS_PATH)
    check(man_mtime <= min(raw_mtimes),
          "corpus manifest mtime predates every preserved provider raw file")
    check(dec_mtime >= man_mtime,
          "decision-log append (RM-02) postdates the manifest write")
    earliest = min(started)
    earliest_epoch = datetime.datetime.fromisoformat(earliest).timestamp()
    man_dt = datetime.datetime.fromtimestamp(
        man_mtime, datetime.timezone.utc).isoformat()
    check(earliest_epoch > man_mtime,
          f"first provider call ({earliest}) postdates the manifest freeze "
          f"({man_dt})")
    check("corpus-manifest-frozen" in seg
          and "corpus_row_count: 15" in seg,
          "RM-02 decision entry present with kind corpus-manifest-frozen "
          "and row count 15")

    # ---------------- R10: evidence-manifest hashes ------------------------
    print("[gate] R10: evidence-manifest hash checks", flush=True)
    evman = load_yaml(GATE_MANIFEST_PATH)
    ok_all = True
    for entry in evman.get("files", []):
        path = os.path.join(REPO, entry["path"])
        if not os.path.isfile(path):
            check(False, f"evidence-manifest file missing: {entry['path']}")
            ok_all = False
            continue
        if sha256_file(path) != entry["sha256"]:
            check(False, f"evidence-manifest hash mismatch: "
                         f"{entry['path']}")
            ok_all = False
    check(ok_all, f"every evidence-manifest entry ({len(evman.get('files', []))}) "
                  f"hashes to its recorded sha256")

    print(f"[gate] {'PASS: all checks passed' if not FAILURES else 'FAIL'}",
          flush=True)
    for f_ in FAILURES:
        print(f"[gate]   failure: {f_}", flush=True)
    return 0 if not FAILURES else 1


if __name__ == "__main__":
    sys.exit(main())
