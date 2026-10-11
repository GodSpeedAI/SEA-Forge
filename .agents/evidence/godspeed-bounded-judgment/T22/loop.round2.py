#!/usr/bin/env python3
"""T22 loop executor ROUND 2 (correction round): ONE real route advance of
the frozen loop machinery.

ROUND-2 CORRECTIONS (round-1 is preserved under iteration-1/ as the failed
round: all three probes ended run_error). Smallest falsified layer:
probe_execution (environmental prerequisites of the reused gauntlet binary):
  (a) the binary REFUSES to run without three REQUIRED control settings
      that have no spec default (crates/gauntlet-app/src/config/resolve.rs:
      ControlStallTimeoutMs / ControlRetryBaseMs / ControlRetryMaxMs, "the
      spec tables no default, so the control loop refuses to run without
      it"); they are set to the repo-canonical values used by the justfile
      recipe and the e2e_full_loop_mock test (60000/100/2000). These are
      liveness/backoff controls, NOT budget or authority levers.
  (b) GAUNTLET_CLOCK=deterministic: the T15 corpus generator and the
      e2e_full_loop_mock test both set it; without it the mock replay's
      fixed-2023 event epoch is stale against the real clock and the stall
      detector kills every invocation (terminal "escalated", no typed
      settled/budget terminal is reachable at all). This restores exactly
      the working T15 case-execution mechanism the prereg freezes by
      reference.
  (c) probe stdout/stderr tails are now recorded in the run-meta sidecar
      (round-1 could not diagnose its run_error probes).
The fixture itself is NOT corrected: the frozen demo-calculator fixture
and the mock scenario fixtures are answer-key mismatched (see
b1-progress.md); every probe is expected to return a typed
budget_exhausted consequence. That contradiction is surfaced, not fixed.

Iteration scope (task T22): neighborhood -> K=3 candidates -> admission ->
real sequential gauntlet probes -> observations -> bounded judgment ->
selection-policy-v1 -> route record + payment counters.

Contract authority: .agents/preregistrations/godspeed-bounded-judgment-T21.prereg.yaml
(frozen, sha256 pinned below). The T21 modules are imported UNCHANGED and
sha256-verified against T21/evidence-manifest.yml before anything runs
(fail-closed).

Probe mechanism: exactly the frozen T15 case-execution mechanism (prereg
fixture_and_case_family; T15/batch-round1-failed.sh): a fresh disposable
demo-calculator copy per candidate under
$HOME/.local/share/godspeed-route-discovery/T22/targets/<candidate_id>,
`sed` runners.default to mock-agent (and max_rounds when != 8), frozen env
(GAUNTLET_STATE_DIR / GAUNTLET_ARTIFACT_DIR / GAUNTLET_TOOL_TEST /
GAUNTLET_MAX_ATTEMPTS_PER_UNIT=2 / GAUNTLET_MOCK_SCENARIO), one headless
`gauntlet run "Build and verify the calculator page"` with timeout 240.
The typed terminal from gauntlet's own event log (run_terminal state,
settlement_committed count) is the only ground truth for a probe.

Authority rules honored: provider output is evidence only; the loop executes
only candidates classified admitted by the frozen T21 admission module; a
probe terminal `settled` is a probe consequence, never route settlement.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True  # no __pycache__ debris in T21/T22 evidence dirs

import hashlib
import json
import os
import re
import shutil
import sqlite3
import subprocess
import time

import yaml

T22_DIR = os.path.dirname(os.path.abspath(__file__))
BRANCH_DIR = os.path.dirname(T22_DIR)
T21_DIR = os.path.join(BRANCH_DIR, "T21")
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(BRANCH_DIR)))

sys.path.insert(0, T21_DIR)

import route_contracts as rc  # noqa: E402  (frozen T21 module, imported unchanged)
import admission as adm  # noqa: E402  (frozen T21 module, imported unchanged)
import selection_policy as sp  # noqa: E402  (frozen T21 module, imported unchanged)

PREREG_SHA256 = "1fb55d9c56d87d94adc4850bc5ca713db3e600297fbdc997accfab54c98ea8b5"

GAUNTLET_BIN = "/home/sprime01/projects/gauntlet/target/debug/gauntlet"
DEMO_TARGET_SRC = "/home/sprime01/projects/gauntlet/examples/demo-calculator"
DATA_ROOT = os.path.join(
    os.path.expanduser("~"), ".local", "share", "godspeed-route-discovery",
    "T22", "round2"
)
PROBE_TIMEOUT_S = 240
PROVIDER_TIMEOUT_S = 300
PROBE_CMD_WORD = "Build and verify the calculator page"
# REQUIRED control settings (no spec default; binary refuses without them),
# at the repo-canonical values (gauntlet justfile + e2e_full_loop_mock test).
REQUIRED_CONTROL_ENV = {
    "GAUNTLET_STALL_TIMEOUT_MS": "60000",
    "GAUNTLET_RETRY_BASE_MS": "100",
    "GAUNTLET_RETRY_MAX_MS": "2000",
}
# The Annex E deterministic clock hook: part of the working T15 corpus
# mechanism; without it the mock replay cannot produce a typed terminal.
CLOCK_ENV = {"GAUNTLET_CLOCK": "deterministic"}

ROUTE_ID = "t22-route-001-round2"
ITERATION = 1
CORR = f"{ROUTE_ID}-iter{ITERATION}"
K = 3
PROBE_BUDGET = rc.MAX_PROBES_PER_ROUTE  # 12

PRIMARY_PROVIDER = "deepseek-clinepass"
PRIMARY_MODEL = "cline-pass/cline-pass/deepseek-v4.1-flash"
SUB_PROVIDER = "muse-spark-free"
SUB_MODEL = "opencode/muse-spark-1.3-contributor-free"

CAND_ID_RE = re.compile(r"^[a-z0-9][a-z0-9-]{0,63}$")

ROUTE_GOAL_FROZEN = [
    "baseline: an e2e-happy probe at the default budget (max_rounds 8) settles (settled_accepted)",
    "discriminator: an e2e-lying probe on this route honestly fails verification "
    "(verification_failed_nonsettlement) - the instrument discriminates on this surface",
    "boundary: the minimal settling budget r* is discovered by this route's own probes such "
    "that max_rounds=r* settles and max_rounds=r*-1 exhausts payment (budget_exhausted); "
    "r*=8 with 7 exhausting is a valid boundary",
]

# Mechanics-only mapping used ONLY when both providers fail (declared
# deterministic fallback; never counted as a provider result).
MECHANICAL_TERMINAL_MAP = {
    "settled": "settled_accepted",
    "budget_exhausted": "budget_exhausted",
    "other-nonsettlement": "verification_failed_nonsettlement",
    "run_error": "execution_failed",
}

OUT_DIR = os.path.join(T22_DIR, "round-2")
RAW_DIR = os.path.join(OUT_DIR, "provider-raw")

EXPECTED_OUTPUTS = [
    "neighborhood.yaml",
    "import-verification.yaml",
    "generation-record.yaml",
    "candidates.json",
    "admission-decisions.json",
    "probe-observations.json",
    "probe-run-meta.yaml",
    "judgment-surfaces.json",
    "judgment-results.json",
    "judgment-record.yaml",
    "selection-record.json",
    "route-record.yaml",
    "payment-counters.yaml",
]

provider_calls: list[dict] = []


def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def utcnow() -> str:
    return rc.utcnow()


def fail(msg: str) -> "NoReturn":  # type: ignore[valid-type]
    print(f"T22 loop: FAIL - {msg}", flush=True)
    sys.exit(1)


# ---------------------------------------------------------------------------
# T21 import verification (fail-closed)
# ---------------------------------------------------------------------------

def verify_t21_imports() -> dict:
    manifest_path = os.path.join(T21_DIR, "evidence-manifest.yml")
    if not os.path.isfile(manifest_path):
        fail(f"T21 evidence manifest missing: {manifest_path}")
    manifest = yaml.safe_load(open(manifest_path))
    results = {"manifest": manifest_path, "files": [], "prereg": {}}
    for entry in manifest.get("files", []):
        path = os.path.join(T21_DIR, entry["path"])
        if not os.path.isfile(path):
            fail(f"T21 frozen file missing: {entry['path']}")
        digest = sha256_file(path)
        ok = digest == entry["sha256"]
        results["files"].append({"path": entry["path"], "sha256": digest, "match": ok})
        if not ok:
            fail(f"T21 frozen module hash mismatch: {entry['path']}")
    prereg_path = os.path.join(REPO_ROOT, ".agents/preregistrations",
                               "godspeed-bounded-judgment-T21.prereg.yaml")
    digest = sha256_file(prereg_path)
    results["prereg"] = {"path": prereg_path, "sha256": digest,
                         "match": digest == PREREG_SHA256}
    if digest != PREREG_SHA256:
        fail("frozen T21 prereg hash mismatch")
    print("[imports] T21 frozen modules + prereg verified (fail-closed)", flush=True)
    return results


# ---------------------------------------------------------------------------
# Provider seam (frozen T16 identity; provider output is evidence only)
# ---------------------------------------------------------------------------

def call_provider(provider: str, model: str, prompt: str) -> dict:
    cmd = ["opencode", "run", "-m", model, prompt]
    started = time.monotonic()
    started_at = utcnow()
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True,
                              timeout=PROVIDER_TIMEOUT_S, cwd=REPO_ROOT)
    except subprocess.TimeoutExpired:
        provider_calls.append({
            "provider": provider, "model": model, "purpose": None,
            "outcome": "provider_timeout", "started_at_utc": started_at,
            "duration_seconds": round(time.monotonic() - started, 3),
        })
        return {"status": "provider_timeout", "raw": ""}
    duration = round(time.monotonic() - started, 3)
    if proc.returncode != 0:
        provider_calls.append({
            "provider": provider, "model": model, "purpose": None,
            "outcome": "provider_error", "started_at_utc": started_at,
            "duration_seconds": duration,
        })
        return {"status": "provider_error", "raw": (proc.stderr or proc.stdout)[-400:]}
    provider_calls.append({
        "provider": provider, "model": model, "purpose": None,
        "outcome": "responded", "started_at_utc": started_at,
        "duration_seconds": duration,
    })
    return {"status": "raw", "raw": proc.stdout}


def set_purpose(purpose: str) -> None:
    if provider_calls and provider_calls[-1]["purpose"] is None:
        provider_calls[-1]["purpose"] = purpose


def save_raw(name: str, text: str) -> str:
    path = os.path.join(RAW_DIR, name)
    with open(path, "w") as f:
        f.write(text if text.endswith("\n") or text == "" else text + "\n")
    return path


# ---------------------------------------------------------------------------
# Step 1: bounded neighborhood (prereg operation_surface.neighborhood_rule)
# ---------------------------------------------------------------------------

def build_neighborhood(route_state: dict, catalog: dict) -> dict:
    ops = []
    for op in catalog["operations"]:
        # (a) not yet probed on this route
        probed = any(
            (p["scenario"], p["max_rounds"]) == (op["scenario"], op["max_rounds"])
            for p in route_state["probed"]
        )
        # (b) affordable under the remaining probe budget (1 unit per probe)
        affordable = route_state["probes_remaining"] >= adm.PROBE_COST
        # (c) admission-compatible: catalog operations are run_case over the
        #     frozen mock replay runner on disposable copies (no credentials,
        #     no network) by construction of the frozen catalog
        compatible = op["operation"] == "run_case"
        # (d) not excluded by a prior denial on this route
        denied = any(
            (d["scenario"], d["max_rounds"]) == (op["scenario"], op["max_rounds"])
            for d in route_state["denied_keys"]
        )
        if not probed and affordable and compatible and not denied:
            ops.append({"op_id": op["op_id"], "operation": op["operation"],
                        "scenario": op["scenario"], "max_rounds": op["max_rounds"]})
    return {
        "schema": "godspeed-bounded-judgment.t22-neighborhood",
        "route_id": ROUTE_ID,
        "iteration": ITERATION,
        "correlation_id": CORR,
        "rule": "prereg operation_surface.neighborhood_rule (frozen)",
        "filters_applied": {
            "probed_on_route": [f"{p['scenario']}@{p['max_rounds']}"
                                for p in route_state["probed"]],
            "denied_on_route": [f"{d['scenario']}@{d['max_rounds']}"
                                for d in route_state["denied_keys"]],
            "probes_remaining": route_state["probes_remaining"],
            "probe_cost_per_operation": adm.PROBE_COST,
            "admission_compatible": "mock replay runner only; disposable copy; "
                                    "no credentials; no network (frozen catalog)",
        },
        "neighborhood_size": len(ops),
        "operations": ops,
    }


# ---------------------------------------------------------------------------
# Step 2: candidate generation (K=3; provider -> ONE substitution -> fallback)
# ---------------------------------------------------------------------------

def generation_prompt(provider: str, model: str, neighborhood: dict) -> str:
    ops_lines = "\n".join(
        f"  - {{op_id: {op['op_id']}, scenario: {op['scenario']}, max_rounds: {op['max_rounds']}}}"
        for op in neighborhood["operations"]
    )
    return (
        "You are the candidate generator in a governed route-discovery loop. "
        "Propose exactly 3 candidate moves.\n\n"
        "ROUTE GOAL (frozen):\n"
        + "\n".join(f"- {g}" for g in ROUTE_GOAL_FROZEN)
        + "\n\nBOUNDED NEIGHBORHOOD (the ONLY operations you may propose; each costs "
        f"1 probe unit; {neighborhood['filters_applied']['probes_remaining']} probe units "
        "remain; operations already probed or denied on this route are excluded):\n"
        + ops_lines
        + "\n\nTYPED CANDIDATE SCHEMA - each candidate is EXACTLY one JSON object with "
        "EXACTLY these fields (no extra fields, no shell plans, no commands):\n"
        "{\n"
        '  "candidate_id": "<unique id, lowercase letters/digits/hyphens, e.g. cand-1>",\n'
        f'  "correlation_id": "{CORR}",\n'
        '  "operation": "run_case",\n'
        '  "bounded_input": {"scenario": "<e2e-happy|e2e-lying|e2e-forged-report>", '
        '"max_rounds": <integer 1..8>},\n'
        '  "expected_role": "<establish_baseline|establish_discriminator|boundary_probe>",\n'
        '  "prerequisite_refs": [],\n'
        f'  "generator": {{"identity": "{provider}", "version": "{model}"}},\n'
        '  "rationale": "<one short sentence, evidence only>"\n'
        "}\n"
        "Constraint: all three candidates must propose DISTINCT (scenario, max_rounds) "
        "operations from the bounded neighborhood.\n\n"
        "OUTPUT: STRICTLY a JSON array of exactly 3 candidate objects. No prose, "
        "no markdown fences, no commentary."
    )


def parse_generation(text: str, provider: str, model: str) -> tuple[list | None, str]:
    m = re.search(r"\[.*\]", text, re.DOTALL)
    if not m:
        return None, "no JSON array in provider output"
    try:
        arr = json.loads(m.group(0))
    except json.JSONDecodeError as e:
        return None, f"invalid JSON array: {e}"
    if not isinstance(arr, list):
        return None, "provider output is not a JSON array"
    if len(arr) != K:
        return None, f"expected exactly {K} candidates, got {len(arr)}"
    cands, seen = [], set()
    for i, c in enumerate(arr):
        if not isinstance(c, dict):
            return None, f"candidate {i} is not a JSON object"
        err = rc.validate("candidate_move", c)
        if err is not None:
            return None, f"candidate {i} schema failure: {err}"
        # joinability conditions (recorded interpretation 3 in b1-progress.md)
        if c["correlation_id"] != CORR:
            return None, (f"candidate {i} did not echo the iteration correlation_id "
                          f"({c['correlation_id']!r} != {CORR!r})")
        if not CAND_ID_RE.match(c["candidate_id"]):
            return None, (f"candidate {i} candidate_id {c['candidate_id']!r} is not "
                          "filesystem-safe (^[a-z0-9][a-z0-9-]{0,63}$)")
        if c["candidate_id"] in seen:
            return None, f"duplicate candidate_id {c['candidate_id']!r}"
        seen.add(c["candidate_id"])
        cands.append(c)
    # stamp the actual provider identity if the provider left placeholders
    for c in cands:
        if not isinstance(c.get("generator"), dict) or not c["generator"]:
            c["generator"] = {"identity": provider, "version": model}
    return cands, ""


def fallback_candidates() -> list[dict]:
    """Declared deterministic fallback (prereg generator_fallback_rule): unmet
    baseline (e2e-happy@8), unmet discriminator (e2e-lying@8), then the midpoint
    boundary probe of the initial uncertain interval [3,7] -> e2e-happy@5."""
    gen = {"identity": "deterministic-fallback-v1", "version": "1"}
    return [
        {
            "candidate_id": "cand-1",
            "correlation_id": CORR,
            "operation": "run_case",
            "bounded_input": {"scenario": "e2e-happy", "max_rounds": 8},
            "expected_role": "establish_baseline",
            "prerequisite_refs": [],
            "generator": gen,
            "rationale": "frozen fallback rule: unmet baseline probe at the default "
                         "budget (e2e-happy, 8)",
        },
        {
            "candidate_id": "cand-2",
            "correlation_id": CORR,
            "operation": "run_case",
            "bounded_input": {"scenario": "e2e-lying", "max_rounds": 8},
            "expected_role": "establish_discriminator",
            "prerequisite_refs": [],
            "generator": gen,
            "rationale": "frozen fallback rule: unmet discriminator probe (e2e-lying, 8)",
        },
        {
            "candidate_id": "cand-3",
            "correlation_id": CORR,
            "operation": "run_case",
            "bounded_input": {"scenario": "e2e-happy", "max_rounds": 5},
            "expected_role": "boundary_probe",
            "prerequisite_refs": [],
            "generator": gen,
            "rationale": "frozen fallback rule: midpoint boundary probe of the initial "
                         "uncertain interval [3,7] (floor((3+7)/2)=5)",
        },
    ]


def generate_candidates(neighborhood: dict) -> tuple[list[dict], dict, str]:
    """Returns (candidates, generation_record, provider_path)."""
    attempts = []
    for tag, provider, model in (
        ("primary", PRIMARY_PROVIDER, PRIMARY_MODEL),
        ("substitution", SUB_PROVIDER, SUB_MODEL),
    ):
        prompt = generation_prompt(provider, model, neighborhood)
        resp = call_provider(provider, model, prompt)
        # re-attribute: the call we just made was the generation call
        provider_calls[-1]["purpose"] = "candidate_generation"
        raw_name = f"generation-{tag}.raw.txt"
        raw_path = save_raw(raw_name, resp["raw"])
        prompt_path = os.path.join(RAW_DIR, f"generation-{tag}.prompt.txt")
        with open(prompt_path, "w") as f:
            f.write(prompt)
        if resp["status"] != "raw":
            attempts.append({
                "attempt": tag, "provider": provider, "model": model,
                "outcome": resp["status"], "raw_output_path": raw_path,
                "prompt_path": prompt_path,
            })
            print(f"[generate] {tag} {provider}: {resp['status']}", flush=True)
            continue
        cands, err = parse_generation(resp["raw"], provider, model)
        outcome = "ok" if cands is not None else f"schema_failure: {err}"
        attempts.append({
            "attempt": tag, "provider": provider, "model": model,
            "outcome": outcome, "raw_output_path": raw_path,
            "prompt_path": prompt_path,
        })
        print(f"[generate] {tag} {provider}: {outcome}", flush=True)
        if cands is not None:
            return cands, {
                "schema": "godspeed-bounded-judgment.t22-generation-record",
                "route_id": ROUTE_ID, "iteration": ITERATION,
                "correlation_id": CORR,
                "provider_path": ("deepseek-clinepass" if tag == "primary"
                                  else "muse-substitution"),
                "generator_identity": provider,
                "attempts": attempts,
                "validation_rule": "route_contracts.candidate_move + correlation_id "
                                   "echo + filesystem-safe unique candidate_id",
            }, ("deepseek-clinepass" if tag == "primary" else "muse-substitution")
    # both providers failed: declared deterministic fallback (mechanics-only)
    cands = fallback_candidates()
    attempts.append({
        "attempt": "deterministic-fallback", "provider": "deterministic-fallback-v1",
        "model": "n/a", "outcome": "ok (declared deterministic fallback; "
                                   "mechanics-only, never counted as a provider result)",
    })
    print("[generate] deterministic-fallback-v1 engaged", flush=True)
    return cands, {
        "schema": "godspeed-bounded-judgment.t22-generation-record",
        "route_id": ROUTE_ID, "iteration": ITERATION, "correlation_id": CORR,
        "provider_path": "deterministic-fallback-v1",
        "generator_identity": "deterministic-fallback-v1",
        "attempts": attempts,
        "validation_rule": "frozen prereg generator_fallback_rule",
    }, "deterministic-fallback-v1"


# ---------------------------------------------------------------------------
# Step 4: real probes (frozen T15 mechanism, strictly sequential)
# ---------------------------------------------------------------------------

def prepare_target(candidate: dict) -> str:
    cid = candidate["candidate_id"]
    target = os.path.join(DATA_ROOT, "targets", cid)
    if os.path.exists(target):
        fail(f"probe target dir already exists for {cid}: {target}")
    shutil.copytree(DEMO_TARGET_SRC, target)
    for script in sorted(os.listdir(os.path.join(target, "scripts"))):
        if script.endswith(".sh"):
            os.chmod(os.path.join(target, "scripts", script), 0o755)
    gauntlet_md = os.path.join(target, "GAUNTLET.md")
    with open(gauntlet_md) as f:
        md = f.read()
    if "default: prime-agent" not in md:
        fail(f"{cid}: GAUNTLET.md does not contain the expected runner default")
    md = md.replace("default: prime-agent", "default: mock-agent")
    rounds = candidate["bounded_input"]["max_rounds"]
    if rounds != 8:
        if f"max_rounds: 8" not in md:
            fail(f"{cid}: GAUNTLET.md does not contain the expected max_rounds lever")
        md = md.replace("max_rounds: 8", f"max_rounds: {rounds}")
    with open(gauntlet_md, "w") as f:
        f.write(md)
    with open(gauntlet_md) as f:
        md2 = f.read()
    if "default: mock-agent" not in md2:
        fail(f"{cid}: runner sed did not take effect")
    if rounds != 8 and f"max_rounds: {rounds}" not in md2:
        fail(f"{cid}: max_rounds sed did not take effect")
    return target


def run_probe(candidate: dict) -> tuple[dict, dict]:
    """Run ONE real gauntlet probe; return (probe_observation, meta)."""
    cid = candidate["candidate_id"]
    scenario = candidate["bounded_input"]["scenario"]
    rounds = candidate["bounded_input"]["max_rounds"]
    probe_id = f"probe-{cid}"
    target = prepare_target(candidate)
    state_dir = os.path.join(DATA_ROOT, "runs", cid, "state")
    artifact_dir = os.path.join(DATA_ROOT, "runs", cid, "evidence")
    os.makedirs(state_dir, exist_ok=True)
    os.makedirs(artifact_dir, exist_ok=True)
    db_path = os.path.join(state_dir, "gauntlet-state.db")
    if os.path.exists(db_path):
        fail(f"{cid}: state db already exists before the run: {db_path}")

    env = dict(os.environ)
    env.update({
        "GAUNTLET_STATE_DIR": state_dir,
        "GAUNTLET_ARTIFACT_DIR": artifact_dir,
        "GAUNTLET_TOOL_TEST": os.path.join(target, "scripts", "check.sh"),
        "GAUNTLET_MAX_ATTEMPTS_PER_UNIT": "2",
        "GAUNTLET_MOCK_SCENARIO": scenario,
        "PYTHONDONTWRITEBYTECODE": "1",
        **REQUIRED_CONTROL_ENV,
        **CLOCK_ENV,
    })
    cmd = ["timeout", str(PROBE_TIMEOUT_S), GAUNTLET_BIN, "run", PROBE_CMD_WORD]
    started_at = utcnow()
    t0 = time.monotonic()
    proc = subprocess.run(cmd, capture_output=True, text=True, cwd=target, env=env)
    wall = round(time.monotonic() - t0, 3)
    finished_at = utcnow()

    timed_out = proc.returncode == 124
    raw_terminal_state = None
    settle_count = 0
    db_exists = os.path.isfile(db_path)
    if db_exists:
        digest = sha256_file(db_path)
        conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
        try:
            rows = conn.execute(
                "select payload from event_log where kind='run_terminal'").fetchall()
            settle_count = conn.execute(
                "select count(*) from event_log where kind='settlement_committed'"
            ).fetchone()[0]
        finally:
            conn.close()
        if len(rows) == 1:
            raw_terminal_state = json.loads(rows[0][0]).get("state")
    else:
        digest = sha256_bytes(b"")

    if raw_terminal_state == "settled":
        terminal_class = "settled"
    elif raw_terminal_state == "budget_exhausted":
        terminal_class = "budget_exhausted"
    elif isinstance(raw_terminal_state, str) and raw_terminal_state != "":
        terminal_class = "other-nonsettlement"
    else:
        terminal_class = "run_error"  # no typed terminal (crash/timeout/no DB)

    observation = {
        "probe_id": probe_id,
        "candidate_id": cid,
        "correlation_id": CORR,
        "operation": "run_case",
        "bounded_input": {"scenario": scenario, "max_rounds": rounds},
        "terminal_class": terminal_class,
        "settlement_committed_count": settle_count,
        "state_db_path": db_path,
        "state_db_sha256": digest,
        "mechanism": rc.MECHANISM,
        "started_at_utc": started_at,
        "finished_at_utc": finished_at,
    }
    err = rc.validate("probe_observation", observation)
    if err is not None:
        fail(f"internal: invalid probe_observation for {cid}: {err}")
    meta = {
        "probe_id": probe_id,
        "candidate_id": cid,
        "gauntlet_stdout_tail": (proc.stdout or "")[-500:],
        "gauntlet_stderr_tail": (proc.stderr or "")[-500:],
        "command": " ".join(cmd),
        "cwd": target,
        "exit_code": proc.returncode,
        "timed_out": timed_out,
        "wall_seconds": wall,
        "state_db_exists": db_exists,
        "raw_terminal_state": raw_terminal_state,
        "settlement_committed_count_from_db": settle_count,
        "note": ("state_db_sha256 is the digest of zero bytes when no state db was "
                 "produced (run_error); the path is the expected frozen-mechanism "
                 "path for this candidate"
                 if not db_exists else
                 "state_db_sha256 is the digest of the real gauntlet state database"),
    }
    print(f"[probe] {probe_id} scenario={scenario} rounds={rounds} -> "
          f"{terminal_class} (exit={proc.returncode}, wall={wall}s, "
          f"settlements={settle_count})", flush=True)
    return observation, meta


# ---------------------------------------------------------------------------
# Step 5: bounded judgment (observation records only; never generation rationale)
# ---------------------------------------------------------------------------

def judgment_prompt(surface: dict) -> str:
    return (
        "You are doing a bounded judgment task. Answer STRICTLY with one JSON object.\n"
        "Question: classify the recorded consequence of one bounded Gauntlet probe.\n"
        "Answer domain for \"label\" (choose exactly one): settled_accepted | "
        "verification_failed_nonsettlement | budget_exhausted | execution_failed | "
        "insufficient_evidence\n"
        "Judge ONLY from the observation record below (typed terminal, "
        "settlement_committed count, state-database digest, timing). Use "
        "insufficient_evidence if the observation genuinely cannot decide.\n"
        "JSON shape: {\"label\": \"<one label from the answer domain>\"}\n\n"
        "Observation record:\n"
        + json.dumps(surface, sort_keys=True, indent=1)
    )


def parse_judgment(text: str) -> tuple[str | None, str]:
    m = re.search(r"\{.*\}", text, re.DOTALL)
    if not m:
        return None, "no JSON object in provider output"
    try:
        obj = json.loads(m.group(0))
    except json.JSONDecodeError as e:
        return None, f"invalid JSON: {e}"
    if not isinstance(obj, dict) or "label" not in obj:
        return None, "no label field in provider JSON object"
    label = obj["label"]
    if label not in rc.ANSWER_DOMAIN:
        return None, f"label {label!r} outside the frozen answer domain"
    return label, ""


def judge_probe(surface: dict, probe_id: str) -> tuple[dict, dict, dict]:
    """Judge one probe. Returns (judgment_result, judgment_path_record, surfaces)."""
    cid = surface["candidate_id"]
    label = None
    path_record = {"probe_id": probe_id, "candidate_id": cid, "attempts": []}
    for tag, provider, model in (
        ("primary", PRIMARY_PROVIDER, PRIMARY_MODEL),
        ("substitution", SUB_PROVIDER, SUB_MODEL),
    ):
        prompt = judgment_prompt(surface)
        resp = call_provider(provider, model, prompt)
        provider_calls[-1]["purpose"] = f"probe_judgment:{probe_id}"
        raw_path = save_raw(f"judgment-{probe_id}-{tag}.raw.txt", resp["raw"])
        prompt_path = os.path.join(RAW_DIR, f"judgment-{probe_id}-{tag}.prompt.txt")
        with open(prompt_path, "w") as f:
            f.write(prompt)
        if resp["status"] != "raw":
            path_record["attempts"].append({
                "attempt": tag, "provider": provider, "model": model,
                "outcome": resp["status"], "raw_output_path": raw_path,
            })
            print(f"[judge] {probe_id} {tag} {provider}: {resp['status']}", flush=True)
            continue
        parsed, err = parse_judgment(resp["raw"])
        outcome = "ok" if parsed is not None else f"schema_failure: {err}"
        path_record["attempts"].append({
            "attempt": tag, "provider": provider, "model": model,
            "outcome": outcome, "raw_output_path": raw_path,
            "abstention": parsed == "insufficient_evidence" if parsed else None,
        })
        print(f"[judge] {probe_id} {tag} {provider}: {outcome} label={parsed}",
              flush=True)
        if parsed is not None:
            label = parsed
            break
    if label is None:
        # declared deterministic fallback: mechanical mapping from the typed
        # terminal; mechanics-only, never counted as a provider result
        label = MECHANICAL_TERMINAL_MAP[surface["terminal_class"]]
        path_record["attempts"].append({
            "attempt": "deterministic-fallback",
            "provider": "deterministic-terminal-mapping-v1",
            "model": "n/a",
            "outcome": f"ok (mechanics-only mechanical mapping "
                       f"{surface['terminal_class']} -> {label}; never counted as "
                       "a provider result)",
        })
        judge_identity = "deterministic-terminal-mapping-v1 (mechanics-only)"
    else:
        winning = path_record["attempts"][-1]
        judge_identity = f"{winning['provider']}:{winning['model']}"
    judgment = {
        "judgment_id": f"judg-{cid}",
        "candidate_id": cid,
        "correlation_id": CORR,
        "label": label,
        "observation_refs": [probe_id],
        "judge_identity": judge_identity,
        "judged_at_utc": utcnow(),
    }
    err = rc.validate("judgment_result", judgment)
    if err is not None:
        fail(f"internal: invalid judgment_result for {cid}: {err}")
    return judgment, path_record, surface


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def dump_json(path: str, data) -> None:
    with open(path, "w") as f:
        json.dump(data, f, indent=2, sort_keys=True)
        f.write("\n")


def dump_yaml(path: str, data) -> None:
    with open(path, "w") as f:
        yaml.safe_dump(data, f, sort_keys=False, width=110)


def main() -> int:
    t_start = time.monotonic()
    os.makedirs(RAW_DIR, exist_ok=True)
    for name in EXPECTED_OUTPUTS:
        if os.path.exists(os.path.join(OUT_DIR, name)):
            fail(f"append-only violation: {name} already exists; a correction uses "
                 "round-2 file names, never an overwrite")
    if not os.access(GAUNTLET_BIN, os.X_OK):
        fail(f"gauntlet binary missing or not executable: {GAUNTLET_BIN}")
    if not os.path.isdir(DEMO_TARGET_SRC):
        fail(f"demo-calculator fixture missing: {DEMO_TARGET_SRC}")

    import_verification = verify_t21_imports()
    catalog = yaml.safe_load(open(os.path.join(T21_DIR, "operations_catalog.yaml")))
    if len(catalog["operations"]) != 24:
        fail("operations catalog does not contain the frozen 24 operations")
    dump_yaml(os.path.join(OUT_DIR, "import-verification.yaml"),
              {"schema": "godspeed-bounded-judgment.t22-import-verification",
               "verified_at_utc": utcnow(),
               **import_verification})

    route_state = {
        "route_id": ROUTE_ID,
        "created_at_utc": utcnow(),
        "probes_remaining": PROBE_BUDGET,
        "retained_moves": [],
        "probed": [],
        "denied_keys": [],
    }
    err = rc.validate("route_state", route_state)
    if err is not None:
        fail(f"internal: invalid initial route_state: {err}")

    # 1. bounded neighborhood
    neighborhood = build_neighborhood(route_state, catalog)
    dump_yaml(os.path.join(OUT_DIR, "neighborhood.yaml"), neighborhood)
    print(f"[neighborhood] {neighborhood['neighborhood_size']} operations presented",
          flush=True)

    # 2. generate K=3 candidates
    candidates, generation_record, provider_path = generate_candidates(neighborhood)
    dump_yaml(os.path.join(OUT_DIR, "generation-record.yaml"), generation_record)
    dump_json(os.path.join(OUT_DIR, "candidates.json"), candidates)
    for c in candidates:
        err = rc.validate("candidate_move", c)
        if err is not None:
            fail(f"internal: invalid candidate {c.get('candidate_id')!r}: {err}")

    # 3. admission through the frozen T21 module (real route state)
    admissions = []
    for c in candidates:
        decision = adm.classify(c, route_state, catalog,
                                payment=route_state["probes_remaining"])
        err = rc.validate("admission_decision", decision)
        if err is not None:
            fail(f"internal: invalid admission_decision: {err}")
        admissions.append(decision)
        print(f"[admit] {c['candidate_id']} -> {decision['classification']}",
              flush=True)
    dump_json(os.path.join(OUT_DIR, "admission-decisions.json"), admissions)
    admitted = [c for c, a in zip(candidates, admissions)
                if a["classification"] == "admitted"]
    if not admitted:
        fail("no candidate was admitted; the route cannot advance")

    # 4. real probes, strictly sequential (one gauntlet process at a time)
    observations, metas = [], []
    for c in admitted:
        obs, meta = run_probe(c)
        observations.append(obs)
        metas.append(meta)
    dump_json(os.path.join(OUT_DIR, "probe-observations.json"), observations)
    dump_yaml(os.path.join(OUT_DIR, "probe-run-meta.yaml"),
              {"schema": "godspeed-bounded-judgment.t22-probe-run-meta",
               "route_id": ROUTE_ID, "iteration": ITERATION,
               "note": "typed sidecar for exit codes and wall seconds; the frozen "
                       "probe_observation schema has an exact field set and refuses "
                       "extra fields",
               "probes": metas})

    # 5. bounded judgment from observation records ONLY
    surfaces, judgments, judgment_paths = [], [], []
    for obs in observations:
        surface = {
            "probe_id": obs["probe_id"],
            "candidate_id": obs["candidate_id"],
            "correlation_id": obs["correlation_id"],
            "operation": obs["operation"],
            "bounded_input": obs["bounded_input"],
            "terminal_class": obs["terminal_class"],
            "settlement_committed_count": obs["settlement_committed_count"],
            "state_db_sha256": obs["state_db_sha256"],
            "mechanism": obs["mechanism"],
            "started_at_utc": obs["started_at_utc"],
            "finished_at_utc": obs["finished_at_utc"],
        }
        surfaces.append(surface)
        judgment, path_record, _ = judge_probe(surface, obs["probe_id"])
        judgments.append(judgment)
        judgment_paths.append(path_record)
    dump_json(os.path.join(OUT_DIR, "judgment-surfaces.json"), surfaces)
    dump_json(os.path.join(OUT_DIR, "judgment-results.json"), judgments)
    dump_yaml(os.path.join(OUT_DIR, "judgment-record.yaml"),
              {"schema": "godspeed-bounded-judgment.t22-judgment-record",
               "route_id": ROUTE_ID, "iteration": ITERATION,
               "judge_input_rule": "observation records only (typed terminal, "
                                   "settlement_committed count, artifact digest, "
                                   "timing); never the builder/provider narrative",
               "probe_judgments": judgment_paths})

    # absorb probes into the route state (probed entries carry judgment labels)
    label_by_candidate = {j["candidate_id"]: j["label"] for j in judgments}
    for obs in observations:
        route_state["probed"].append({
            "candidate_id": obs["candidate_id"],
            "correlation_id": obs["correlation_id"],
            "scenario": obs["bounded_input"]["scenario"],
            "max_rounds": obs["bounded_input"]["max_rounds"],
            "terminal_class": obs["terminal_class"],
            "judgment_label": label_by_candidate[obs["candidate_id"]],
        })
    route_state["probes_remaining"] -= len(observations)
    err = rc.validate("route_state", route_state)
    if err is not None:
        fail(f"internal: invalid route_state after absorption: {err}")

    # 6. deterministic selection through the frozen T21 policy
    selection = sp.select(candidates, admissions, observations, judgments,
                          route_state, payment=route_state["probes_remaining"],
                          correlation_id=CORR, iteration=ITERATION)
    dump_json(os.path.join(OUT_DIR, "selection-record.json"), selection)
    print(f"[select] retained={selection['retained_candidate_id']!r} "
          f"eligible={selection['eligible_candidate_ids']}", flush=True)

    retained_moves, moves = [], []
    if selection["retained_candidate_id"] != "":
        cid = selection["retained_candidate_id"]
        cand = next(c for c in candidates if c["candidate_id"] == cid)
        obs = next(o for o in observations if o["candidate_id"] == cid)
        retained_moves.append({
            "candidate_id": cid,
            "correlation_id": CORR,
            "scenario": cand["bounded_input"]["scenario"],
            "max_rounds": cand["bounded_input"]["max_rounds"],
            "expected_role": cand["expected_role"],
            "judgment_label": label_by_candidate[cid],
        })
        moves.append({
            "scenario": cand["bounded_input"]["scenario"],
            "max_rounds": cand["bounded_input"]["max_rounds"],
            "terminal_class": obs["terminal_class"],
        })
        route_state["retained_moves"] = retained_moves
        err = rc.validate("route_state", route_state)
        if err is not None:
            fail(f"internal: invalid route_state after retention: {err}")

    route_record = {
        "route_id": ROUTE_ID,
        "correlation_id": CORR,
        "settled": False,
        "stop_condition": "iteration_limit_reached",
        "moves": moves,
        "fingerprint": rc.route_fingerprint(moves),
        "correlation_ids": [CORR],
        "policy_version": sp.POLICY_VERSION,
        "prereg_sha256": PREREG_SHA256,
        "opened_at_utc": route_state["created_at_utc"],
        "closed_at_utc": utcnow(),
    }
    err = rc.validate("route_record", route_record)
    if err is not None:
        fail(f"internal: invalid route_record: {err}")
    dump_yaml(os.path.join(OUT_DIR, "route-record.yaml"), {
        "schema": "godspeed-bounded-judgment.t22-route-record",
        "correction_round": 2,
        "probe_data_root": DATA_ROOT,
        "note": "stop_condition records THIS task-scoped run's bound of exactly one "
                "advance (see deviations in b1-result.json); the route itself stays "
                "open for T23 under the frozen 6-iteration bound; settled is false "
                "because a probe terminal is a probe consequence, never route "
                "settlement",
        "route_state": route_state,
        "route_record": route_record,
    })

    # 7. payment counters
    payment = {
        "schema": "godspeed-bounded-judgment.t22-payment-counters",
        "task": "T22",
        "iteration": ITERATION,
        "correction_round": 2,
        "wall_clock_seconds": round(time.monotonic() - t_start, 3),
        "provider_calls": {
            "count": len(provider_calls),
            "by_identity": [
                {"seq": i + 1, "purpose": c["purpose"], "provider": c["provider"],
                 "model": c["model"], "outcome": c["outcome"],
                 "started_at_utc": c["started_at_utc"],
                 "duration_seconds": c["duration_seconds"]}
                for i, c in enumerate(provider_calls)
            ],
        },
        "probes_run": len(observations),
        "tool_executions": len(observations),
        "failed_executions": sum(1 for o in observations
                                 if o["terminal_class"] == "run_error"),
        "tokens_and_cost": "n/a: the frozen provider seam (opencode run) does not "
                           "expose token/cost accounting; honest gap per prereg "
                           "payment_counters_frozen.tokens_and_cost",
        "operator_decisions": 0,
        "probe_data_root": DATA_ROOT,
    }
    dump_yaml(os.path.join(OUT_DIR, "payment-counters.yaml"), payment)

    print("[done] iteration-1 records written; provider_path="
          f"{provider_path}; probes={len(observations)}; "
          f"retained={selection['retained_candidate_id']!r}; "
          f"fingerprint={route_record['fingerprint']!r}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
