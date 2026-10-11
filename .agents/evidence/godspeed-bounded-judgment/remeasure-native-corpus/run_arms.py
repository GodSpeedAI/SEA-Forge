#!/usr/bin/env python3
"""Arm runner for the native-corpus remeasurement (prereg Phases 1-3).

Prereg: .agents/preregistrations/godspeed-bounded-judgment-native-corpus-remeasure.prereg.yaml
(frozen sha256 8142c0a52537f9b600d4853afd4871d5ec04dc29acffba58fe67c7f7616d1f6e;
corpus frozen by D-2026-09-19-RM-02, manifest
remeasure-native-corpus/corpus-manifest.yaml).

Phase 1 (arm1): T16-comparable predictive judgment — the T16 loader's
pre-terminal records view, the T16 PROMPT verbatim, parse_t16 verbatim, the
T16 face-value baseline scored against the same mapped truth, McNemar exact.

Phase 2 (arm2): discrimination on the frozen T23 round-2 enriched surface
(builder imported unchanged) with the pinned round-2 prompt template (only the
records blob substituted), 3 sequential samples per row, frozen majority
aggregation (ties -> insufficient_evidence).

Phase 3 (arm2b): exploratory graded progress — the arm2 template extended
with ONE additional JSON field settled_units_estimate (integer or null),
clearly marked exploratory; scored (MAE + exact-hit) against the actual
settlement_committed unit counts from the pinned DBs.

Provider policy (all arms): deepseek-clinepass is the scored provider; on
provider_timeout/provider_error ONE muse-spark-free substitution per call,
recorded per-call; substituted rows form a separate provider-qualified slice,
never pooled; total-failure stops with preserved evidence and the arm
recorded NOT_RUN_PROVIDER. No provider call is ever re-rolled: a preserved
raw output file is reused on resume.

Read-only over every frozen source. Writes only under
remeasure-native-corpus/ (raw outputs, prompts, surfaces, results JSON).
Frozen before its first execution; never overwritten. Corrections use a
round-2 filename.
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
import time

import yaml

BASE = ("/home/sprime01/projects/sea-rs/.agents/evidence/"
        "godspeed-bounded-judgment")
T16_DIR = os.path.join(BASE, "T16")
R2_DIR = os.path.join(BASE, "T23", "round-2")
T24_DIR = os.path.join(BASE, "T24")
T25_DIR = os.path.join(BASE, "T25")
OUT_DIR = os.path.join(BASE, "remeasure-native-corpus")
REPO = "/home/sprime01/projects/sea-rs"

PREREG_PATH = os.path.join(
    REPO, ".agents/preregistrations/"
    "godspeed-bounded-judgment-native-corpus-remeasure.prereg.yaml")
PREREG_SHA256 = (
    "8142c0a52537f9b600d4853afd4871d5ec04dc29acffba58fe67c7f7616d1f6e")
MANIFEST_PATH = os.path.join(OUT_DIR, "corpus-manifest.yaml")
RESULTS_PATH = os.path.join(OUT_DIR, "remeasure-results.json")
DECISIONS_PATH = os.path.join(BASE, "decisions.yml")
R2_RESULTS = os.path.join(R2_DIR, "rejudgment-results.json")
ARM1_RAW = os.path.join(OUT_DIR, "arm1", "provider-raw")
ARM2_RAW = os.path.join(OUT_DIR, "arm2", "provider-raw")
ARM2_SURF = os.path.join(OUT_DIR, "arm2", "surfaces")
ARM2B_RAW = os.path.join(OUT_DIR, "arm2b", "provider-raw")

sys.path.insert(0, T16_DIR)
import run_remeasure as t16  # noqa: E402  (frozen T16 harness: PROMPT,
# parse_t16, call_provider, face_value — imported, never modified)
sys.path.insert(0, R2_DIR)
import build_surfaces as bs  # noqa: E402  (frozen T23 round-2 surface builder)

PRIMARY = "deepseek-clinepass"
SUB = "muse-spark-free"
MODEL = {PRIMARY: "cline-pass/cline-pass/deepseek-v4.1-flash",
         SUB: "opencode/muse-spark-1.3-contributor-free"}

ARM1_DOMAIN = ["settled", "not_settled", "insufficient_evidence"]
ARM2_DOMAIN = ["settled_accepted", "verification_failed_nonsettlement",
               "budget_exhausted", "execution_failed",
               "insufficient_evidence"]
TRUTH_MAP_ARM1 = {"settled_accepted": "settled",
                  "budget_exhausted": "not_settled",
                  "verification_failed_nonsettlement": "not_settled"}
MECHANISM = "t15-frozen-case-execution"
TEMPLATE_SPLIT = "Corrected observation record:\n"
ARM2B_JSON_OLD = 'JSON shape: {"label": "<one label from the answer domain>"}\n'
ARM2B_JSON_NEW = ('JSON shape: {"label": "<one label from the answer domain>",'
                  ' "settled_units_estimate": <integer or null>}\n')
ARM2B_NOTE = ('Exploratory addendum (clearly marked exploratory, outside the '
              'scored label domain): "settled_units_estimate" is your '
              'estimate of the number of units that will have committed '
              "settlement in this run's recorded consequence; give an "
              'integer, or null if you cannot estimate.\n')

DEVIATIONS = [
    {"id": "DV-1",
     "issue": ("prereg arm2b says the settled_units_estimate is asked 'on "
               "the honest-family rows' while the operator execution "
               "directive pins 1 sample/row = 15 calls"),
     "reading": ("arm2b runs on all 15 rows (operator directive); the "
                 "honest-family slice is reported separately beside the "
                 "full-corpus comparison-8 numbers, matching the prereg's "
                 "scoring wording"),
     "kind": "scope-reading"},
    {"id": "DV-2",
     "issue": ("prereg aggregation_frozen does not pin how failed "
               "(unparseable) samples enter the 3-sample majority"),
     "reading": ("most restrictive: a label needs >=2 of the 3 cast votes "
                 "to win; ties and rows with fewer than 2 parseable samples "
                 "resolve to insufficient_evidence (never fabricated)"),
     "kind": "most-restrictive-reading"},
    {"id": "DV-3",
     "issue": ("prereg reporting.mutable_state says CURRENT_STATUS is "
               "updated after the report; the operator directive scopes "
               "writes to the remeasure directory plus the decision-log "
               "append"),
     "reading": ("this executor does not modify "
                 ".agents/current_status.yml (orchestrator-owned); the "
                 "recommendation is recorded in the durable report and "
                 "b1-result for the orchestrator to settle"),
     "kind": "scope-reading"},
]


def stop(msg: str):
    print(f"[arms] STOP: {msg}", flush=True)
    sys.exit(2)


def utcnow() -> str:
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def sha256_text(t: str) -> str:
    return hashlib.sha256(t.encode("utf-8")).hexdigest()


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


# ---------------------------------------------------------------------------
# provider seam (frozen T16 call_provider + per-call policy wrapper)
# ---------------------------------------------------------------------------

def timed_call(provider: str, prompt: str) -> dict:
    t0 = time.monotonic()
    started = utcnow()
    resp = t16.call_provider(provider, prompt)
    resp["provider"] = provider
    resp["model"] = MODEL[provider]
    resp["started_at_utc"] = started
    resp["duration_seconds"] = round(time.monotonic() - t0, 3)
    return resp


def call_record(n: int, resp: dict) -> dict:
    return {"call": n, "provider": resp["provider"], "model": resp["model"],
            "status": resp["status"],
            "started_at_utc": resp["started_at_utc"],
            "duration_seconds": resp["duration_seconds"]}


def preserved_call(raw_dir: str, stem: str, prompt: str,
                   write_prompt: bool = True) -> dict:
    """One call slot under the frozen substitution policy.

    The raw output is preserved verbatim BEFORE parsing; an existing raw file
    is reused (never re-rolled). On provider_timeout/provider_error of the
    primary, ONE muse-spark-free substitution is attempted and recorded
    per-call. Parse failures are NOT substituted (typed rejections are
    results, per the frozen normalization)."""
    os.makedirs(raw_dir, exist_ok=True)
    prompt_path = os.path.join(raw_dir, f"{stem}.prompt.txt")
    if os.path.isfile(prompt_path):
        with open(prompt_path) as f:
            if f.read() != prompt:
                stop(f"prompt drift at {prompt_path} (template/corpus "
                     f"changed since an earlier pass)")
    elif write_prompt:
        with open(prompt_path, "w") as f:
            f.write(prompt)
    calls = []
    raw_path = os.path.join(raw_dir, f"{stem}-deepseek.raw.txt")
    if os.path.isfile(raw_path):
        with open(raw_path) as f:
            raw = f.read()
        calls.append({"call": 1, "provider": PRIMARY, "model": MODEL[PRIMARY],
                      "status": "raw (preserved; never re-rolled)",
                      "raw_path": raw_path, "started_at_utc": None,
                      "duration_seconds": None})
        return {"calls": calls, "status": "raw", "provider_used": PRIMARY,
                "raw_path": raw_path, "substituted": False, "raw_text": raw}
    resp = timed_call(PRIMARY, prompt)
    calls.append(call_record(1, resp))
    if resp["status"] != "raw":
        err_path = os.path.join(raw_dir, f"{stem}-deepseek.error.txt")
        with open(err_path, "w") as f:
            f.write(resp.get("raw", "") or "")
        calls[-1]["error_path"] = err_path
        print(f"[arms]   primary {resp['status']}: one "
              f"muse-spark-free substitution", flush=True)
        sub = timed_call(SUB, prompt)
        calls.append(call_record(2, sub))
        if sub["status"] == "raw":
            sub_path = os.path.join(raw_dir, f"{stem}-muse.raw.txt")
            with open(sub_path, "w") as f:
                f.write(sub["raw"])
            calls[-1]["raw_path"] = sub_path
            return {"calls": calls, "status": "raw", "provider_used": SUB,
                    "raw_path": sub_path, "substituted": True,
                    "raw_text": sub["raw"]}
        sub_err = os.path.join(raw_dir, f"{stem}-muse.error.txt")
        with open(sub_err, "w") as f:
            f.write(sub.get("raw", "") or "")
        calls[-1]["error_path"] = sub_err
        return {"calls": calls, "status": sub["status"],
                "provider_used": None, "raw_path": None,
                "substituted": True, "raw_text": None}
    with open(raw_path, "w") as f:
        f.write(resp["raw"])
    calls[-1]["raw_path"] = raw_path
    return {"calls": calls, "status": "raw", "provider_used": PRIMARY,
            "raw_path": raw_path, "substituted": False,
            "raw_text": resp["raw"]}


# ---------------------------------------------------------------------------
# stats (stdlib only)
# ---------------------------------------------------------------------------

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


def cp_ci(k: int, n: int, alpha: float = 0.05):
    """Clopper-Pearson exact two-sided 95% CI."""
    if n == 0:
        return None, None
    lo = 0.0 if k == 0 else _betainc_inv(alpha / 2.0, k, n - k + 1)
    hi = 1.0 if k == n else _betainc_inv(1.0 - alpha / 2.0, k + 1, n - k)
    return lo, hi


def mcnemar_exact(b: int, c: int) -> float:
    """Two-sided exact binomial McNemar (no scipy)."""
    n = b + c
    if n == 0:
        return 1.0
    return min(1.0, 2.0 * sum(math.comb(n, k)
                              for k in range(0, min(b, c) + 1)) / 2 ** n)


# ---------------------------------------------------------------------------
# arm1: exact T16 view
# ---------------------------------------------------------------------------

def t16_view(db_path: str) -> dict:
    """Exact mirror of the T16 loader's per-row pre-terminal view: the
    unit_dispatch + evidence_produced kinds, the role_invocation result
    documents from the records table, correction counts;
    run_terminal/settlement_committed events EXCLUDED (leakage cut)."""
    conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    conn.row_factory = sqlite3.Row
    kinds, terminal_marker = [], False
    for r in conn.execute(
            "select kind, payload from event_log order by offset"):
        if r["kind"] in ("run_terminal", "settlement_committed"):
            terminal_marker = True
            continue  # leakage cut
        if r["kind"] in ("unit_dispatch", "evidence_produced"):
            kinds.append(r["kind"])
    invs = []
    for (p,) in conn.execute(
            "select payload from records where record_type='role_invocation' "
            "order by seq"):
        invs.append({"result_document": json.loads(p)})
    cors = conn.execute(
        "select count(*) from records where record_type="
        "'correction_selection'").fetchone()[0]
    conn.close()
    return {"invocations": invs, "corrections": cors, "kinds_seen": kinds,
            "terminal_excluded": terminal_marker}


# ---------------------------------------------------------------------------
# arm2: frozen round-2 template pinning + surface building
# ---------------------------------------------------------------------------

def pin_round2_template():
    """Pin the T23 round-2 judgment prompt template: all 12 preserved
    provider-raw prompt files must share one template prefix whose records
    blob round-trips byte-exactly to json.dumps(surface, sort_keys=True,
    indent=1) of the preserved surface."""
    prompts = sorted(glob.glob(os.path.join(
        R2_DIR, "provider-raw", "r2-judgment-*-primary.prompt.txt")))
    if len(prompts) != 12:
        stop(f"expected 12 round-2 prompt files, found {len(prompts)}")
    prefixes = {}
    for pf in prompts:
        with open(pf) as f:
            txt = f.read()
        if txt.count(TEMPLATE_SPLIT) != 1:
            stop(f"{pf}: template split marker not unique")
        prefix, blob = txt.split(TEMPLATE_SPLIT)
        m = re.search(r"r2-judgment-(probe-[a-z0-9-]+)-primary",
                      os.path.basename(pf))
        if not m:
            stop(f"{pf}: cannot extract probe id")
        probe_id = m.group(1)
        surface = load_json(os.path.join(
            R2_DIR, "surfaces", f"{probe_id}.json"))["surface"]
        if json.dumps(surface, sort_keys=True, indent=1) != blob:
            stop(f"{probe_id}: round-2 prompt blob does not round-trip to "
                 f"the canonical surface dump")
        prefixes.setdefault(prefix, []).append(probe_id)
    if len(prefixes) != 1:
        stop(f"round-2 prompt files do not share one template "
             f"({len(prefixes)} distinct prefixes)")
    prefix = next(iter(prefixes))
    return prefix, prompts, prefixes[prefix]


def resolve_obs(row: dict):
    """Re-resolve one manifest row's frozen observation record (the same
    frozen sources phase0 pinned); build_surface's own prov.recompute
    re-verifies the DB pin fail-closed."""
    source = row["source"]
    want = {"scenario": row["probe"]["scenario"],
            "max_rounds": row["probe"]["max_rounds"]}
    if source.startswith("t23-iter"):
        it = source.split("t23-iter", 1)[1]
        idir = os.path.join(BASE, "T23", "route", f"iteration-{it}")
        observations = load_json(os.path.join(idir, "probe-observations.json"))
        matches = [o for o in observations if o.get("bounded_input") == want]
        if len(matches) != 1:
            stop(f"{row['id']}: {len(matches)} observations for {want}")
        obs = matches[0]
        meta = load_yaml(os.path.join(idir, "probe-run-meta.yaml"))
        metas = [m for m in meta.get("probes", [])
                 if m.get("probe_id") == obs["probe_id"]]
        if len(metas) != 1:
            stop(f"{row['id']}: run-meta entry not unique")
        return obs, metas[0]
    if source == "t24-replay":
        rr = load_yaml(os.path.join(T24_DIR, "replay-record.yaml"))
        return rr["observation"], {"wall_seconds": rr["run"]["wall_seconds"]}
    if source == "t24-tooth-t7":
        teeth = load_yaml(os.path.join(T24_DIR, "teeth.yaml"))
        t7 = [t for t in teeth["teeth"]
              if t["id"] == "T7-invalid-inverse-case"]
        if len(t7) != 1:
            stop("T7 tooth entry not found")
        return t7[0]["attack_observation"], {
            "wall_seconds": t7[0]["run_meta"]["wall_seconds"]}
    if source == "t25-negative":
        mp = load_yaml(os.path.join(T25_DIR, "matched-pair.yaml"))
        neg = mp["negative"]
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
        return obs, {"wall_seconds": neg["run"]["wall_seconds"]}
    stop(f"{row['id']}: unknown source {source!r}")


# ---------------------------------------------------------------------------
# parsers (T16 parse semantics verbatim, per arm domain)
# ---------------------------------------------------------------------------

def parse_arm_domain(text: str, answer_key: str, domain,
                     with_estimate=False):
    """T16 parse_t16 semantics verbatim, restated per the arm's frozen
    answer domain and answer-field name. Out-of-domain values are typed
    rejections, counted, never coerced."""
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


def majority_of(labels):
    """Frozen aggregation (DV-2 reading): majority over the 3 samples; a
    label needs >=2 of 3 cast votes; ties and rows with <2 parseable samples
    resolve to insufficient_evidence (never fabricated)."""
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


# ---------------------------------------------------------------------------
# scoring
# ---------------------------------------------------------------------------

def slice_scores(entries):
    """entries: list of {id, label (or None), truth, family, rejection_kind}.
    T16 scoring semantics: accuracy over answered pairs + CP CI; typed
    rejections counted separately, never coerced."""
    answered = [e for e in entries if e["label"]]
    k = sum(1 for e in answered if e["label"] == e["truth"])
    n = len(answered)
    lo, hi = cp_ci(k, n)
    all_k = sum(1 for e in entries if e["label"] == e["truth"])
    out = {
        "n": len(entries),
        "n_answered": n,
        "k_correct": k,
        "accuracy": (k / n) if n else None,
        "ci95": [lo, hi],
        "accuracy_all_rows_rejection_is_wrong": (all_k / len(entries))
        if entries else None,
        "contract_failures": sum(1 for e in entries if not e["label"]),
        "abstentions": sum(1 for e in entries
                           if e["label"] == "insufficient_evidence"),
        "abstention_rate": (sum(1 for e in entries
                                if e["label"] == "insufficient_evidence")
                            / len(entries)) if entries else None,
        "typed_rejections": sum(1 for e in entries if not e["label"]),
        "typed_rejection_rate": (sum(1 for e in entries if not e["label"])
                                 / len(entries)) if entries else None,
    }
    conf = {}
    for e in answered:
        conf.setdefault(e["label"], {})
        conf[e["label"]][e["truth"]] = \
            conf[e["label"]].get(e["truth"], 0) + 1
    out["confusion_pred_to_truth"] = conf
    for fam in ("honest", "adversarial"):
        sub = [e for e in entries if e["family"] == fam]
        ans = [e for e in sub if e["label"]]
        ck = sum(1 for e in ans if e["label"] == e["truth"])
        flo, fhi = cp_ci(ck, len(ans))
        out[f"family_{fam}"] = {
            "n": len(sub), "n_answered": len(ans), "k_correct": ck,
            "accuracy": (ck / len(ans)) if ans else None, "ci95": [flo, fhi]}
    out["errors"] = [
        {"id": e["id"], "truth": e["truth"], "label": e["label"]}
        for e in answered if e["label"] != e["truth"]]
    return out


# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------

def main() -> int:
    if os.environ.get("PYTHONDONTWRITEBYTECODE") != "1":
        stop("export PYTHONDONTWRITEBYTECODE=1 before running")
    if sha256_file(PREREG_PATH) != PREREG_SHA256:
        stop("prereg sha256 mismatch")
    if not os.path.isfile(MANIFEST_PATH):
        stop("corpus manifest missing (phase 0 must freeze first)")
    manifest_sha = sha256_file(MANIFEST_PATH)
    with open(DECISIONS_PATH) as f:
        dtext = f.read()
    if "D-2026-09-19-RM-02" not in dtext:
        stop("decision D-2026-09-19-RM-02 not found in the decision log "
             "(phase-0 ordering violated)")
    manifest = load_yaml(MANIFEST_PATH)
    rows = manifest["rows"]
    if len(rows) != 15:
        stop(f"manifest has {len(rows)} rows, expected 15")
    for row in rows:  # fail-closed DB re-hash at arm-runner start
        if sha256_file(row["state_db_path"]) != row["db_sha256"]:
            stop(f"{row['id']}: DB pin mismatch at start")

    historical = {r["probe_id"]: r["label"]
                  for r in load_json(R2_RESULTS)}

    results = {
        "schema": "godspeed-bounded-judgment.remeasure-native-corpus-results",
        "created_utc": utcnow(),
        "prereg": {"path": os.path.relpath(PREREG_PATH, REPO),
                   "sha256": PREREG_SHA256},
        "corpus_manifest": {
            "path": os.path.relpath(MANIFEST_PATH, REPO),
            "sha256": manifest_sha,
            "decision": "D-2026-09-19-RM-02"},
        "provider_policy": {
            "scored_provider": PRIMARY, "scored_model": MODEL[PRIMARY],
            "substitution_provider": SUB, "substitution_model": MODEL[SUB],
            "substitution_rule": (
                "ONE muse-spark-free substitution per call on "
                "provider_timeout/provider_error; recorded per-call; "
                "substituted rows are a separate provider-qualified slice, "
                "never pooled; parse failures are typed rejections, never "
                "substituted"),
            "total_failure_rule": (
                "deepseek arm recorded NOT_RUN_PROVIDER when every deepseek "
                "call of the arm fails; muse slice reported "
                "provider-qualified"),
            "no_reroll": ("a preserved raw output file is reused, never "
                          "re-rolled")},
        "templates": {},
        "arm_status": {},
        "rows": [],
        "deviations": DEVIATIONS,
    }

    def save():
        with open(RESULTS_PATH, "w") as f:
            json.dump(results, f, indent=1, sort_keys=False)
            f.write("\n")

    # ---------- arm2 template pinning (before any call) -------------------
    prefix, r2_prompt_files, r2_probes = pin_round2_template()
    prefix2b = prefix.replace(ARM2B_JSON_OLD, ARM2B_JSON_NEW + ARM2B_NOTE)
    results["templates"] = {
        "arm1": {"source": "T16/run_remeasure.py PROMPT (verbatim import)",
                 "sha256": sha256_text(t16.PROMPT),
                 "records_cap_chars": 12000,
                 "records_blob": (
                     'json.dumps({"invocations": [{result_document}...], '
                     '"corrections": N}) — exactly the T16 main() assembly; '
                     'the unit_dispatch/evidence_produced kinds are loaded '
                     'by the T16 loader but enter no prompt, as in T16')},
        "arm2": {"template_prefix_sha256": sha256_text(prefix),
                 "source_files": [os.path.relpath(p, REPO)
                                  for p in r2_prompt_files],
                 "source_probes": r2_probes,
                 "blob_rule": ("json.dumps(surface, sort_keys=True, "
                               "indent=1); only the records blob is "
                               "substituted; verified to round-trip against "
                               "all 12 preserved round-2 prompts")},
        "arm2b": {"template_prefix_sha256": sha256_text(prefix2b),
                  "extends": "arm2 template",
                  "replaced_text": ARM2B_JSON_OLD,
                  "added_text": ARM2B_JSON_NEW + ARM2B_NOTE},
    }
    save()

    # ======================================================================
    # PHASE 1 — arm1 (T16-comparable predictive, 1 sample/row)
    # ======================================================================
    print("[arm1] begin (15 rows x 1 deepseek call)", flush=True)
    arm1_calls = {"deepseek_calls": 0, "deepseek_failed": 0,
                  "muse_substitutions": 0}
    for i, row in enumerate(rows):
        rid = row["id"]
        mapped = TRUTH_MAP_ARM1[row["truth_class"]]
        view = t16_view(row["state_db_path"])
        blob = json.dumps({"invocations": view["invocations"],
                           "corrections": view["corrections"]})[:12000]
        prompt = t16.PROMPT.format(records=blob)
        # T16's own leakage assertion, verbatim
        assert "run_terminal" not in prompt and \
            "settlement_committed" not in prompt, f"{rid}: leakage"
        baseline = t16.face_value({"invocations": view["invocations"],
                                   "corrections": view["corrections"]})
        pc = preserved_call(ARM1_RAW, f"arm1-{rid}", prompt)
        if pc["provider_used"] == PRIMARY:
            arm1_calls["deepseek_calls"] += 1
        else:
            arm1_calls["deepseek_failed"] += 1
            if pc["substituted"] and pc["status"] == "raw":
                arm1_calls["muse_substitutions"] += 1
        if pc["status"] == "raw":
            parsed = t16.parse_t16(pc["raw_text"])
        else:
            parsed = {"status": pc["status"]}
        label = parsed.get("answer") if parsed.get("status") == "ok" else None
        results["rows"].append({
            "id": rid, "source": row["source"],
            "probe": row["probe"], "probe_id": row["probe_id"],
            "state_db_path": row["state_db_path"],
            "db_sha256": row["db_sha256"],
            "truth_class": row["truth_class"],
            "truth_mapped_arm1": mapped,
            "family": "adversarial"
                      if row["probe"]["scenario"] == "e2e-lying"
                      else "honest",
            "historical_round2_label": historical.get(row["probe_id"]),
            "arm1": {
                "view": {"n_result_documents": len(view["invocations"]),
                         "corrections": view["corrections"],
                         "kinds_seen": view["kinds_seen"],
                         "terminal_excluded": view["terminal_excluded"]},
                "prompt_path": os.path.join(ARM1_RAW,
                                            f"arm1-{rid}.prompt.txt"),
                "prompt_sha256": sha256_text(prompt),
                "calls": pc["calls"],
                "provider_used": pc["provider_used"],
                "substituted": pc["substituted"],
                "status": parsed.get("status", pc["status"]),
                "raw_path": pc["raw_path"],
                "parsed": parsed,
                "label": label,
                "correct": (label == mapped) if label else None,
                "baseline_label": baseline,
                "baseline_correct": baseline == mapped,
            },
        })
        save()
        print(f"[arm1] {i + 1}/15 {rid}: label={label} truth={mapped} "
              f"baseline={baseline}", flush=True)
    results["arm_status"]["arm1"] = (
        "NOT_RUN_PROVIDER"
        if arm1_calls["deepseek_calls"] == 0
        and arm1_calls["deepseek_failed"] > 0 else "run")
    save()

    # ---------------- arm1 scoring ----------------------------------------
    def arm1_entry(r):
        a = r["arm1"]
        return {"id": r["id"], "label": a["label"],
                "truth": r["truth_mapped_arm1"], "family": r["family"],
                "rejection_kind": None if a["label"] else a["status"]}

    a1_primary = [r for r in results["rows"]
                  if r["arm1"]["provider_used"] == PRIMARY]
    a1_sub = [r for r in results["rows"]
              if r["arm1"]["provider_used"] == SUB]
    arm1_scoring = {
        "slice_rule": ("deepseek slice is the scored measurement; the "
                       "substituted slice is reported beside it, never "
                       "pooled"),
        "deepseek_slice": slice_scores([arm1_entry(r) for r in a1_primary]),
        "substituted_slice_never_pooled": slice_scores(
            [arm1_entry(r) for r in a1_sub]),
    }
    paired = [r for r in a1_primary if r["arm1"]["correct"] is not None]
    b = sum(1 for r in paired
            if r["arm1"]["correct"] and not r["arm1"]["baseline_correct"])
    c = sum(1 for r in paired
            if not r["arm1"]["correct"] and r["arm1"]["baseline_correct"])
    arm1_scoring["mcnemar_learned_vs_baseline"] = {
        "comparison": "learned arm1 vs face-value baseline, paired rows",
        "n_paired": len(paired),
        "learned_right_baseline_wrong": b,
        "learned_wrong_baseline_right": c,
        "p_exact_two_sided": mcnemar_exact(b, c)}
    base_k = sum(1 for r in a1_primary if r["arm1"]["baseline_correct"])
    blo, bhi = cp_ci(base_k, len(a1_primary))
    arm1_scoring["baseline_face_value_deepseek_slice"] = {
        "rule": ("face-value: all result documents claim completed -> "
                 "settled (T16 rule verbatim, same mapped truth)"),
        "n": len(a1_primary), "k_correct": base_k,
        "accuracy": base_k / len(a1_primary) if a1_primary else None,
        "ci95": [blo, bhi],
        "triviality_note": "baseline never reads artifact contents"}
    results["arm1_scoring"] = arm1_scoring
    results["payment"] = {"arm1": dict(arm1_calls)}
    save()
    print(f"[arm1] scoring: deepseek acc="
          f"{arm1_scoring['deepseek_slice']['accuracy']} "
          f"(n_answered={arm1_scoring['deepseek_slice']['n_answered']}) "
          f"baseline acc="
          f"{arm1_scoring['baseline_face_value_deepseek_slice']['accuracy']} "
          f"mcnemar p="
          f"{arm1_scoring['mcnemar_learned_vs_baseline']['p_exact_two_sided']}",
          flush=True)

    # ======================================================================
    # PHASE 2 — arm2 (frozen enriched surface, 3 samples/row)
    # ======================================================================
    print("[arm2] begin (15 rows x 3 deepseek calls)", flush=True)
    os.makedirs(ARM2_SURF, exist_ok=True)
    arm2_calls = {"deepseek_calls": 0, "deepseek_failed": 0,
                  "muse_substitutions": 0}
    for i, row in enumerate(rows):
        rid = row["id"]
        existing = next(r for r in results["rows"] if r["id"] == rid)
        obs, run_meta = resolve_obs(row)
        record = bs.build_surface(obs, run_meta)
        if record.get("status") != "ok" or not record.get("surface"):
            stop(f"{rid}: surface build failed: {record}")
        surface = record["surface"]
        if surface["settlement_committed_count"] != \
                row["actual_settled_units"]:
            stop(f"{rid}: surface settlement count disagrees with manifest")
        blob = json.dumps(surface, sort_keys=True, indent=1)
        prompt = prefix + TEMPLATE_SPLIT + blob
        prompt_path = os.path.join(ARM2_RAW, f"arm2-{rid}.prompt.txt")
        if os.path.isfile(prompt_path):
            with open(prompt_path) as f:
                if f.read() != prompt:
                    stop(f"prompt drift at {prompt_path}")
        else:
            with open(prompt_path, "w") as f:
                f.write(prompt)
        surf_path = os.path.join(ARM2_SURF, f"{rid}.surface.json")
        with open(surf_path, "w") as f:
            json.dump({"schema": record["schema"],
                       "probe_id": record["probe_id"],
                       "correction_round": record["correction_round"],
                       "addendum2_sha256": record["addendum2_sha256"],
                       "status": record["status"],
                       "surface": surface,
                       "provenance": record["provenance"]}, f,
                      indent=1, sort_keys=True)
            f.write("\n")
        samples = []
        for s in (1, 2, 3):
            pc = preserved_call(ARM2_RAW, f"arm2-{rid}-s{s}", prompt,
                                write_prompt=False)
            if pc["provider_used"] == PRIMARY:
                arm2_calls["deepseek_calls"] += 1
            else:
                arm2_calls["deepseek_failed"] += 1
                if pc["substituted"] and pc["status"] == "raw":
                    arm2_calls["muse_substitutions"] += 1
            if pc["status"] == "raw":
                parsed = parse_arm_domain(pc["raw_text"], "label",
                                          ARM2_DOMAIN)
            else:
                parsed = {"status": pc["status"]}
            label = parsed.get("answer") if parsed.get("status") == "ok" \
                else None
            samples.append({
                "sample": s, "calls": pc["calls"],
                "provider_used": pc["provider_used"],
                "substituted": pc["substituted"],
                "status": parsed.get("status", pc["status"]),
                "raw_path": pc["raw_path"], "parsed": parsed,
                "label": label,
                "correct": (label == row["truth_class"]) if label else None})
            print(f"[arm2] {rid} sample {s}/3 (row {i + 1}/15): "
                  f"label={label} truth={row['truth_class']}", flush=True)
        labels = [s["label"] for s in samples]
        maj, tally = majority_of(labels)
        ok_labels = [l for l in labels if l]
        modal = max(tally.values()) if tally else 0
        existing["arm2"] = {
            "surface_path": surf_path,
            "surface_sha256": sha256_text(blob),
            "prompt_path": prompt_path,
            "prompt_sha256": sha256_text(prompt),
            "samples": samples,
            "labels_ok": ok_labels,
            "majority_label": maj,
            "majority_tally": tally,
            "majority_correct": (maj == row["truth_class"]),
            "single1_label": labels[0],
            "single1_correct": (labels[0] == row["truth_class"])
                               if labels[0] else None,
            "agreement_modal_of_cast": modal,
            "agreement_rate": modal / 3.0,
            "all_three_agree": (len(ok_labels) == 3
                                and len(set(ok_labels)) == 1),
        }
        save()
    results["arm_status"]["arm2"] = (
        "NOT_RUN_PROVIDER"
        if arm2_calls["deepseek_calls"] == 0
        and arm2_calls["deepseek_failed"] > 0 else "run")
    save()

    # ---------------- arm2 scoring ----------------------------------------
    def arm2_single_entry(r):
        s1 = r["arm2"]["samples"][0]
        return {"id": r["id"], "label": s1["label"],
                "truth": r["truth_class"], "family": r["family"],
                "rejection_kind": None if s1["label"] else s1["status"]}

    def arm2_majority_entry(r):
        kind = None
        if r["arm2"]["majority_label"] == "insufficient_evidence" \
                and len(r["arm2"]["labels_ok"]) < 2:
            kind = "fewer_than_2_parseable_samples"
        return {"id": r["id"], "label": r["arm2"]["majority_label"],
                "truth": r["truth_class"], "family": r["family"],
                "rejection_kind": kind}

    a2_deepseek = [r for r in results["rows"]
                   if all(s["provider_used"] == PRIMARY
                          for s in r["arm2"]["samples"])]
    a2_sub = [r for r in results["rows"]
              if not all(s["provider_used"] == PRIMARY
                         for s in r["arm2"]["samples"])]
    s_single = slice_scores([arm2_single_entry(r) for r in a2_deepseek])
    s_maj = slice_scores([arm2_majority_entry(r) for r in a2_deepseek])
    arm2_scoring = {
        "aggregation_rule": ("majority over the 3 samples; >=2 of 3 cast "
                             "votes needed; ties and rows with <2 parseable "
                             "samples -> insufficient_evidence (DV-2)"),
        "single_sample1_deepseek_slice": s_single,
        "majority_deepseek_slice": s_maj,
        "substituted_slice_never_pooled": {
            "n_rows": len(a2_sub),
            "row_ids": [r["id"] for r in a2_sub],
            "single_sample1": slice_scores(
                [arm2_single_entry(r) for r in a2_sub]) if a2_sub else None,
            "majority": slice_scores(
                [arm2_majority_entry(r) for r in a2_sub]) if a2_sub else None},
        "stability": {
            "definition": ("per-row agreement rate = modal-label count of "
                           "the 3 cast samples / 3"),
            "per_row_agreement_rate": {
                r["id"]: r["arm2"]["agreement_rate"] for r in a2_deepseek},
            "mean_agreement_rate_deepseek_rows": (
                sum(r["arm2"]["agreement_rate"] for r in a2_deepseek)
                / len(a2_deepseek)) if a2_deepseek else None,
            "rows_all_three_agree": sum(
                1 for r in a2_deepseek if r["arm2"]["all_three_agree"]),
            "n_deepseek_rows": len(a2_deepseek)},
        "majority_vs_single_material_change": {
            "accuracy_single": s_single["accuracy"],
            "accuracy_majority": s_maj["accuracy"],
            "rows_changed_label": [
                {"id": r["id"], "single1": r["arm2"]["single1_label"],
                 "majority": r["arm2"]["majority_label"],
                 "truth": r["truth_class"]}
                for r in a2_deepseek
                if r["arm2"]["single1_label"]
                != r["arm2"]["majority_label"]]},
        "comparison7_error_list_majority": [
            {"id": r["id"], "truth": r["truth_class"],
             "majority_label": r["arm2"]["majority_label"],
             "majority_tally": r["arm2"]["majority_tally"],
             "raw_paths": [s["raw_path"] for s in r["arm2"]["samples"]]}
            for r in a2_deepseek
            if r["arm2"]["majority_label"] != r["truth_class"]],
        "historical_freshness_lying_rows": [
            {"id": r["id"], "probe_id": r["probe_id"],
             "truth": r["truth_class"],
             "fresh_majority_label": r["arm2"]["majority_label"],
             "fresh_labels": r["arm2"]["labels_ok"],
             "preserved_round2_label": r["historical_round2_label"],
             "fresh_matches_preserved": (
                 r["arm2"]["majority_label"]
                 == r["historical_round2_label"])}
            for r in results["rows"] if r["family"] == "adversarial"],
    }
    results["arm2_scoring"] = arm2_scoring
    results["payment"]["arm2"] = dict(arm2_calls)
    save()
    print(f"[arm2] scoring: single acc={s_single['accuracy']} "
          f"majority acc={s_maj['accuracy']} "
          f"stability="
          f"{arm2_scoring['stability']['mean_agreement_rate_deepseek_rows']}",
          flush=True)

    # ======================================================================
    # PHASE 3 — arm2b (exploratory graded progress, 1 sample/row)
    # ======================================================================
    print("[arm2b] begin (15 rows x 1 deepseek call, exploratory)",
          flush=True)
    arm2b_calls = {"deepseek_calls": 0, "deepseek_failed": 0,
                   "muse_substitutions": 0}
    for i, row in enumerate(rows):
        rid = row["id"]
        existing = next(r for r in results["rows"] if r["id"] == rid)
        obs, run_meta = resolve_obs(row)
        record = bs.build_surface(obs, run_meta)
        if record.get("status") != "ok" or not record.get("surface"):
            stop(f"{rid}: surface build failed: {record}")
        blob = json.dumps(record["surface"], sort_keys=True, indent=1)
        prompt = prefix2b + TEMPLATE_SPLIT + blob
        pc = preserved_call(ARM2B_RAW, f"arm2b-{rid}", prompt)
        if pc["provider_used"] == PRIMARY:
            arm2b_calls["deepseek_calls"] += 1
        else:
            arm2b_calls["deepseek_failed"] += 1
            if pc["substituted"] and pc["status"] == "raw":
                arm2b_calls["muse_substitutions"] += 1
        if pc["status"] == "raw":
            parsed = parse_arm_domain(pc["raw_text"], "label", ARM2_DOMAIN,
                                      with_estimate=True)
        else:
            parsed = {"status": pc["status"]}
        label = parsed.get("answer") if parsed.get("status") == "ok" else None
        est = parsed.get("settled_units_estimate") \
            if parsed.get("status") == "ok" else None
        est_status = parsed.get("estimate_status") \
            if parsed.get("status") == "ok" else pc["status"]
        abs_err = abs(est - row["actual_settled_units"]) \
            if est_status == "valid" else None
        existing["arm2b"] = {
            "prompt_path": os.path.join(ARM2B_RAW, f"arm2b-{rid}.prompt.txt"),
            "prompt_sha256": sha256_text(prompt),
            "calls": pc["calls"],
            "provider_used": pc["provider_used"],
            "substituted": pc["substituted"],
            "status": parsed.get("status", pc["status"]),
            "raw_path": pc["raw_path"], "parsed": parsed,
            "label": label,
            "estimate": est, "estimate_status": est_status,
            "actual_settled_units": row["actual_settled_units"],
            "abs_error": abs_err,
        }
        save()
        print(f"[arm2b] {i + 1}/15 {rid}: label={label} "
              f"estimate={est}({est_status}) "
              f"actual={row['actual_settled_units']}", flush=True)
    results["arm_status"]["arm2b"] = (
        "NOT_RUN_PROVIDER"
        if arm2b_calls["deepseek_calls"] == 0
        and arm2b_calls["deepseek_failed"] > 0 else "run")
    save()

    # ---------------- arm2b scoring (exploratory) -------------------------
    def arm2b_score(entry_rows):
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

    b_rows = [r for r in results["rows"]
              if r["arm2b"]["provider_used"] == PRIMARY]
    results["arm2b_scoring"] = {
        "status": "exploratory; excluded from the primary verdicts",
        "deepseek_slice_all_rows": arm2b_score(b_rows),
        "deepseek_slice_honest_family": arm2b_score(
            [r for r in b_rows if r["family"] == "honest"]),
        "per_row": [{"id": r["id"], "family": r["family"],
                     "estimate": r["arm2b"]["estimate"],
                     "estimate_status": r["arm2b"]["estimate_status"],
                     "actual": r["arm2b"]["actual_settled_units"],
                     "abs_error": r["arm2b"]["abs_error"]}
                    for r in b_rows]}
    results["payment"]["arm2b"] = dict(arm2b_calls)

    # ---------------- payment counters ------------------------------------
    def arm_wall(arm_key):
        secs = 0.0
        for r in results["rows"]:
            if arm_key == "arm2":
                if "arm2" in r:
                    for s in r["arm2"]["samples"]:
                        for cl in s["calls"]:
                            if cl.get("duration_seconds"):
                                secs += cl["duration_seconds"]
            elif arm_key in r:
                for cl in r[arm_key]["calls"]:
                    if cl.get("duration_seconds"):
                        secs += cl["duration_seconds"]
        return round(secs, 3)

    def arm_total(c):
        return c["deepseek_calls"] + c["deepseek_failed"] \
            + c["muse_substitutions"]

    results["payment"]["wall_seconds_by_arm"] = {
        "arm1": arm_wall("arm1"), "arm2": arm_wall("arm2"),
        "arm2b": arm_wall("arm2b")}
    results["payment"]["total_calls_by_arm"] = {
        "arm1": arm_total(arm1_calls), "arm2": arm_total(arm2_calls),
        "arm2b": arm_total(arm2b_calls)}
    results["payment"]["mechanical_layer_provider_calls"] = 0
    results["payment"]["learned_layer_total_provider_calls"] = (
        arm_total(arm1_calls) + arm_total(arm2_calls)
        + arm_total(arm2b_calls))
    results["payment"]["note"] = (
        "call counts include scored deepseek calls, recorded deepseek "
        "failures and muse substitutions; the mechanical consequence layer "
        "requires zero provider calls")
    save()
    print(f"[arms] complete: total provider calls="
          f"{results['payment']['learned_layer_total_provider_calls']} "
          f"status={results['arm_status']}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
