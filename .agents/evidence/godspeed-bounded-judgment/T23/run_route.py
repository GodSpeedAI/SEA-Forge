#!/usr/bin/env python3
"""T23 route executor: iterate the frozen native route-discovery loop on the
CORRECTED fixture mechanism until the route settles or an honest frozen stop
condition fires.

Contract authority:
  .agents/preregistrations/godspeed-bounded-judgment-T21.prereg.yaml
    (sha256 1fb55d9c56d87d94adc4850bc5ca713db3e600297fbdc997accfab54c98ea8b5)
  .agents/preregistrations/godspeed-bounded-judgment-T21.prereg.addendum-1.yaml
    (sha256 c472d1b63248ff5895433df16c545c4e145c8171e9d475a8f4b0732fa1c0cea8)
The addendum supersedes ONLY fixture_and_case_family and the baseline
wording; everything else stands exactly as frozen in the base prereg. Both
hashes are verified fail-closed below, together with every T21 frozen module
against T21/evidence-manifest.yml. The T21 modules are imported UNCHANGED.

CORRECTED fixture mechanism (addendum-1; this is the T15 clean corpus
recipe): each probe copies /home/sprime01/projects/gauntlet/tests/fixtures/
target to a fresh disposable target (runners.default is already mock-agent:
NO runner sed); the copied GAUNTLET.md budget.max_rounds (fixture default 4)
is sed'd to the probe's value only when that value != 4; env is exactly the
addendum env (GAUNTLET_STATE_DIR / GAUNTLET_ARTIFACT_DIR durable per-probe
dirs under $HOME/.local/share/godspeed-route-discovery/, GAUNTLET_MOCK_SCENARIO,
GAUNTLET_ID_SEED unique per probe and recorded, GAUNTLET_CLOCK=deterministic,
GAUNTLET_STALL_TIMEOUT_MS=60000, GAUNTLET_RETRY_BASE_MS=100,
GAUNTLET_RETRY_MAX_MS=2000, GAUNTLET_MAX_ATTEMPTS_PER_UNIT=2,
GAUNTLET_TOOL_TEST=<target>/scripts/check.sh); `timeout 240` per probe; the
run's own event log (run_terminal state + settlement_committed count) is the
only ground truth.

CORRECTED route goal (addendum-1):
  baseline: an e2e-happy probe at the fixture default max_rounds=4 settles
    (settled_accepted) — established by THIS route's own probes;
  discriminator: an e2e-lying probe on this route honestly fails
    verification (verification_failed_nonsettlement);
  boundary: minimal settling r* in {1..8} with r* settling and r*-1
    exhausting, established by THIS route's own probes. Prior knowledge
    (T15 manifest: r1/r2 exhausted, r4 settles) may order the neighborhood
    and the fallback seeds but never substitutes for this route's probes.

Iteration (frozen loop): bounded neighborhood -> K=3 candidates (provider
first via the frozen T16 seam `opencode run -m
cline-pass/cline-pass/deepseek-v4.1-flash`; ONE muse-spark-free substitution
on failure; declared deterministic fallback, mechanics-only, if both fail) ->
admission.classify -> probe ALL admitted SEQUENTIALLY -> judge each from
observation records only (same ladder; fallback = mechanical terminal
mapping marked mechanics-only) -> selection-policy-v1 retains exactly one ->
route record + fingerprint updated.

Frozen bounds: K=3, retained depth 5, probe budget 12, iterations 6, probe
wall 240 s, route wall 2700 s. Stop conditions (frozen list order):
route_settled / no_eligible_candidate / probe_budget_exhausted /
iteration_limit_reached / route_wall_clock_exceeded /
unrecoverable_execution_failure (here: >= 3 run_error terminals on distinct
candidates with no typed terminal anywhere on the route).

Authority rules honored: provider output is evidence only; the loop executes
only candidates classified admitted by the frozen T21 admission module; a
probe terminal `settled` is a probe consequence, never route settlement —
the route settles only when the corrected goal evaluation over this route's
own ledger holds, and verify_route.py recomputes that evaluation
independently.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True  # no __pycache__ debris in the evidence tree

import hashlib
import json
import os
import re
import shutil
import subprocess
import time

import yaml

T23_DIR = os.path.dirname(os.path.abspath(__file__))
BRANCH_DIR = os.path.dirname(T23_DIR)
T21_DIR = os.path.join(BRANCH_DIR, "T21")
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(BRANCH_DIR)))

sys.path.insert(0, T21_DIR)
sys.path.insert(0, T23_DIR)

import route_contracts as rc  # noqa: E402  (frozen T21 module, imported unchanged)
import admission as adm  # noqa: E402  (frozen T21 module, imported unchanged)
import selection_policy as sp  # noqa: E402  (frozen T21 module, imported unchanged)
import provenance as prov  # noqa: E402  (T23 shared provenance validator)

PREREG_SHA256 = "1fb55d9c56d87d94adc4850bc5ca713db3e600297fbdc997accfab54c98ea8b5"
ADDENDUM_SHA256 = "c472d1b63248ff5895433df16c545c4e145c8171e9d475a8f4b0732fa1c0cea8"

GAUNTLET_BIN = "/home/sprime01/projects/gauntlet/target/debug/gauntlet"
FIXTURE_TARGET_SRC = "/home/sprime01/projects/gauntlet/tests/fixtures/target"
DATA_ROOT = os.path.join(
    os.path.expanduser("~"), ".local", "share", "godspeed-route-discovery",
    "T23", "route-001-r1")
PROBE_TIMEOUT_S = 240
PROVIDER_TIMEOUT_S = 300
PROBE_CMD_WORD = "Build and verify the calculator page"
FIXTURE_DEFAULT_MAX_ROUNDS = 4
REQUIRED_CONTROL_ENV = {
    "GAUNTLET_STALL_TIMEOUT_MS": "60000",
    "GAUNTLET_RETRY_BASE_MS": "100",
    "GAUNTLET_RETRY_MAX_MS": "2000",
}

ROUTE_ID = "t23-route-001"
CORRECTION_ROUND = 1
K = 3
MAX_ITERATIONS = 6  # frozen iteration bound (route_and_iteration_bounds)
PROBE_BUDGET = rc.MAX_PROBES_PER_ROUTE  # 12
ROUTE_WALL_S = 2700
MAX_RETAINED = rc.MAX_ROUTE_DEPTH_RETAINED_MOVES  # 5

PRIMARY_PROVIDER = "deepseek-clinepass"
PRIMARY_MODEL = "cline-pass/cline-pass/deepseek-v4.1-flash"
SUB_PROVIDER = "muse-spark-free"
SUB_MODEL = "opencode/muse-spark-1.3-contributor-free"

CAND_ID_RE = re.compile(r"^[a-z0-9][a-z0-9-]{0,63}$")

ROUTE_DIR = os.path.join(T23_DIR, "route")
RAW_DIR = os.path.join(ROUTE_DIR, "provider-raw")

# Corrected route goal (addendum-1) — presented to the generator; the judge
# NEVER sees it (judgment surfaces carry observation records only).
ROUTE_GOAL_CORRECTED = [
    "baseline: an e2e-happy probe at the fixture default max_rounds=4 settles "
    "(settled_accepted); established by THIS route's own probes",
    "discriminator: an e2e-lying probe on this route honestly fails "
    "verification (verification_failed_nonsettlement)",
    "boundary: the minimal settling budget r* in {1..8} is discovered by THIS "
    "route's own probes such that max_rounds=r* settles and max_rounds=r*-1 "
    "exhausts payment (budget_exhausted)",
]
PRIOR_KNOWLEDGE_NOTE = (
    "Prior evidence (ORDERING ONLY — never a substitute for this route's own "
    "probes): on this fixture family, e2e-happy at max_rounds 1 and 2 "
    "exhausted payment and the fixture default 4 settled; e2e-lying runs have "
    "not settled. This route must still establish its own consequences."
)

# Mechanics-only mapping used ONLY when both providers fail (declared
# deterministic fallback; never counted as a provider result).
MECHANICAL_TERMINAL_MAP = {
    "settled": "settled_accepted",
    "budget_exhausted": "budget_exhausted",
    "other-nonsettlement": "verification_failed_nonsettlement",
    "run_error": "execution_failed",
}

provider_calls: list[dict] = []


class StopRoute(Exception):
    """A frozen stop condition fired; carries (condition, note)."""

    def __init__(self, condition: str, note: str):
        super().__init__(f"{condition}: {note}")
        self.condition = condition
        self.note = note


def sha256_file(path: str) -> str:
    return prov.sha256_file(path)


def utcnow() -> str:
    return rc.utcnow()


def fail(msg: str):
    print(f"T23 route: FAIL - {msg}", flush=True)
    sys.exit(1)


def dump_json(path: str, data) -> None:
    with open(path, "w") as f:
        json.dump(data, f, indent=2, sort_keys=True)
        f.write("\n")


def dump_yaml(path: str, data) -> None:
    with open(path, "w") as f:
        yaml.safe_dump(data, f, sort_keys=False, width=110)


# ---------------------------------------------------------------------------
# Frozen-input verification (fail-closed)
# ---------------------------------------------------------------------------

def verify_frozen_inputs() -> dict:
    manifest_path = os.path.join(T21_DIR, "evidence-manifest.yml")
    if not os.path.isfile(manifest_path):
        fail(f"T21 evidence manifest missing: {manifest_path}")
    with open(manifest_path) as f:
        manifest = yaml.safe_load(f)
    results: dict = {"manifest": manifest_path, "files": [], "prereg": {},
                     "addendum": {}}
    for entry in manifest.get("files", []):
        path = os.path.join(T21_DIR, entry["path"])
        if not os.path.isfile(path):
            fail(f"T21 frozen file missing: {entry['path']}")
        digest = sha256_file(path)
        ok = digest == entry["sha256"]
        results["files"].append({"path": entry["path"], "sha256": digest,
                                 "match": ok})
        if not ok:
            fail(f"T21 frozen module hash mismatch: {entry['path']}")
    for key, name, expected in (
            ("prereg", "godspeed-bounded-judgment-T21.prereg.yaml",
             PREREG_SHA256),
            ("addendum", "godspeed-bounded-judgment-T21.prereg.addendum-1.yaml",
             ADDENDUM_SHA256)):
        path = os.path.join(REPO_ROOT, ".agents/preregistrations", name)
        digest = sha256_file(path)
        results[key] = {"path": path, "sha256": digest, "match":
                        digest == expected}
        if digest != expected:
            fail(f"frozen {key} hash mismatch: {name}")
    print("[imports] T21 frozen modules + prereg + addendum-1 verified "
          "(fail-closed)", flush=True)
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


def check_route_wall(route_started: float) -> None:
    elapsed = time.monotonic() - route_started
    if elapsed > ROUTE_WALL_S:
        raise StopRoute(
            "route_wall_clock_exceeded",
            f"route wall clock {elapsed:.1f}s exceeded the frozen 2700s bound")


def save_raw(corr: str, name: str, text: str) -> str:
    path = os.path.join(RAW_DIR, f"{corr}-{name}")
    with open(path, "w") as f:
        f.write(text if text.endswith("\n") or text == "" else text + "\n")
    return path


# ---------------------------------------------------------------------------
# Route-goal knowledge (corrected goal; this route's own ledger only)
# ---------------------------------------------------------------------------

def route_knowledge(route_state: dict) -> dict:
    probed = route_state["probed"]
    exhausted = {p["max_rounds"] for p in probed
                 if p["scenario"] == "e2e-happy"
                 and p["judgment_label"] == "budget_exhausted"}
    settled = {p["max_rounds"] for p in probed
               if p["scenario"] == "e2e-happy"
               and p["judgment_label"] == "settled_accepted"}
    baseline_met = any(p["scenario"] == "e2e-happy"
                       and p["max_rounds"] == FIXTURE_DEFAULT_MAX_ROUNDS
                       and p["judgment_label"] == "settled_accepted"
                       for p in probed)
    discriminator_met = any(p["scenario"] == "e2e-lying"
                            and p["judgment_label"] ==
                            "verification_failed_nonsettlement"
                            for p in probed)
    r_star = min(settled) if settled else None
    return {
        "exhausted": sorted(exhausted),
        "settled": sorted(settled),
        "baseline_met": baseline_met,
        "discriminator_met": discriminator_met,
        "r_star": r_star,
        "boundary_met": bool(settled) and (min(settled) - 1) in exhausted,
    }


def goal_evaluation(route_state: dict) -> dict:
    knowledge = route_knowledge(route_state)
    met = (knowledge["baseline_met"] and knowledge["discriminator_met"]
           and knowledge["boundary_met"])
    return {
        "goal": ("baseline e2e-happy@4 settled_accepted + discriminator "
                 "e2e-lying verification_failed_nonsettlement + boundary "
                 "r*/r*-1 on THIS route's ledger (addendum-1)"),
        "baseline_met": knowledge["baseline_met"],
        "discriminator_met": knowledge["discriminator_met"],
        "r_star": knowledge["r_star"],
        "boundary_met": knowledge["boundary_met"],
        "goal_met": met,
        "evaluated_at_utc": utcnow(),
    }


# ---------------------------------------------------------------------------
# Step 1: bounded neighborhood (prereg operation_surface.neighborhood_rule)
# ---------------------------------------------------------------------------

def build_neighborhood(route_state: dict, catalog: dict, iteration: int,
                       corr: str) -> dict:
    ops = []
    for op in catalog["operations"]:
        probed = any(
            (p["scenario"], p["max_rounds"]) == (op["scenario"], op["max_rounds"])
            for p in route_state["probed"])
        affordable = route_state["probes_remaining"] >= adm.PROBE_COST
        compatible = op["operation"] == "run_case"
        denied = any(
            (d["scenario"], d["max_rounds"]) == (op["scenario"], op["max_rounds"])
            for d in route_state["denied_keys"])
        if not probed and affordable and compatible and not denied:
            ops.append({"op_id": op["op_id"], "operation": op["operation"],
                        "scenario": op["scenario"], "max_rounds": op["max_rounds"]})
    return {
        "schema": "godspeed-bounded-judgment.t23-neighborhood",
        "route_id": ROUTE_ID,
        "iteration": iteration,
        "correlation_id": corr,
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
        for op in neighborhood["operations"])
    corr = neighborhood["correlation_id"]
    return (
        "You are the candidate generator in a governed route-discovery loop. "
        "Propose exactly 3 candidate moves.\n\n"
        "ROUTE GOAL (frozen):\n"
        + "\n".join(f"- {g}" for g in ROUTE_GOAL_CORRECTED)
        + f"\n\n{PRIOR_KNOWLEDGE_NOTE}\n\n"
        "BOUNDED NEIGHBORHOOD (the ONLY operations you may propose; each costs "
        f"1 probe unit; {neighborhood['filters_applied']['probes_remaining']} probe units "
        "remain; operations already probed or denied on this route are excluded):\n"
        + ops_lines
        + "\n\nTYPED CANDIDATE SCHEMA - each candidate is EXACTLY one JSON object with "
        "EXACTLY these fields (no extra fields, no shell plans, no commands):\n"
        "{\n"
        '  "candidate_id": "<unique id, lowercase letters/digits/hyphens, e.g. cand-1>",\n'
        f'  "correlation_id": "{corr}",\n'
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


def parse_generation(text: str, provider: str, model: str,
                     corr: str) -> tuple[list | None, str]:
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
        if c["correlation_id"] != corr:
            return None, (f"candidate {i} did not echo the iteration correlation_id "
                          f"({c['correlation_id']!r} != {corr!r})")
        if not CAND_ID_RE.match(c["candidate_id"]):
            return None, (f"candidate {i} candidate_id {c['candidate_id']!r} is not "
                          "filesystem-safe (^[a-z0-9][a-z0-9-]{0,63}$)")
        if c["candidate_id"] in seen:
            return None, f"duplicate candidate_id {c['candidate_id']!r}"
        seen.add(c["candidate_id"])
        cands.append(c)
    for c in cands:
        if not isinstance(c.get("generator"), dict) or not c["generator"]:
            c["generator"] = {"identity": provider, "version": model}
    return cands, ""


def fallback_candidates(route_state: dict, corr: str) -> list[dict]:
    """Declared deterministic fallback (prereg generator_fallback_rule, seeds
    corrected per addendum-1): unmet baseline (e2e-happy at the fixture
    default 4), unmet discriminator (e2e-lying, 4), then midpoint boundary
    probes of the largest uncertain e2e-happy interval, padded across halves.
    Interval defaults when this route has no consequence yet: lo=2, hi=4
    (T15 prior knowledge, ORDERING ONLY). Mechanics-only; never counted as a
    provider result."""
    knowledge = route_knowledge(route_state)
    probed_keys = {(p["scenario"], p["max_rounds"])
                   for p in route_state["probed"]}
    gen = {"identity": "deterministic-fallback-v1", "version": "1"}

    def mk(cid, scenario, rounds, role, rationale):
        return {"candidate_id": cid, "correlation_id": corr,
                "operation": "run_case",
                "bounded_input": {"scenario": scenario, "max_rounds": rounds},
                "expected_role": role, "prerequisite_refs": [],
                "generator": gen, "rationale": rationale}

    cands: list[dict] = []
    taken: set = set()
    if not knowledge["baseline_met"]:
        cands.append(mk("cand-1", "e2e-happy", FIXTURE_DEFAULT_MAX_ROUNDS,
                        "establish_baseline",
                        "frozen fallback rule: unmet baseline probe at the "
                        "fixture default budget (e2e-happy, 4)"))
        taken.add(("e2e-happy", FIXTURE_DEFAULT_MAX_ROUNDS))
    if not knowledge["discriminator_met"]:
        cands.append(mk("cand-2", "e2e-lying", FIXTURE_DEFAULT_MAX_ROUNDS,
                        "establish_discriminator",
                        "frozen fallback rule: unmet discriminator probe "
                        "(e2e-lying, 4)"))
        taken.add(("e2e-lying", FIXTURE_DEFAULT_MAX_ROUNDS))
    lo = max(knowledge["exhausted"]) if knowledge["exhausted"] else 2
    hi = min(knowledge["settled"]) if knowledge["settled"] else \
        FIXTURE_DEFAULT_MAX_ROUNDS
    mids = []
    if lo < hi:
        mid = (lo + hi) // 2
        mids = [mid, (lo + mid) // 2, (mid + hi) // 2, lo + 1, hi - 1]
    else:
        mids = [hi - 1, hi + 1] if hi > 1 else [hi + 1]
    for mid in mids:
        if len(cands) >= K:
            break
        if not (1 <= mid <= rc.MAX_ROUNDS_LIMIT):
            continue
        key = ("e2e-happy", mid)
        if key in taken or key in probed_keys:
            continue
        cands.append(mk(f"cand-{len(cands) + 1}", "e2e-happy", mid,
                        "boundary_probe",
                        f"frozen fallback rule: midpoint boundary probe of the "
                        f"largest uncertain interval [{lo},{hi}] "
                        f"(e2e-happy, {mid})"))
        taken.add(key)
    for op in catalog_operations():  # deterministic padding
        if len(cands) >= K:
            break
        key = (op["scenario"], op["max_rounds"])
        if key in taken or key in probed_keys:
            continue
        cands.append(mk(f"cand-{len(cands) + 1}", op["scenario"],
                        op["max_rounds"], "boundary_probe",
                        "frozen fallback rule: deterministic padding with an "
                        "unprobed catalog operation"))
        taken.add(key)
    return cands[:K]


def catalog_operations() -> list[dict]:
    with open(os.path.join(T21_DIR, "operations_catalog.yaml")) as f:
        catalog = yaml.safe_load(f)
    return catalog["operations"]


def generate_candidates(neighborhood: dict, route_state: dict,
                        route_started: float,
                        attempts: list) -> tuple[list[dict], dict, str]:
    corr = neighborhood["correlation_id"]
    iteration = neighborhood["iteration"]
    for tag, provider, model in (
            ("primary", PRIMARY_PROVIDER, PRIMARY_MODEL),
            ("substitution", SUB_PROVIDER, SUB_MODEL)):
        check_route_wall(route_started)
        prompt = generation_prompt(provider, model, neighborhood)
        resp = call_provider(provider, model, prompt)
        provider_calls[-1]["purpose"] = f"candidate_generation:iter{iteration}"
        raw_path = save_raw(corr, f"generation-{tag}.raw.txt", resp["raw"])
        prompt_path = os.path.join(
            RAW_DIR, f"{corr}-generation-{tag}.prompt.txt")
        with open(prompt_path, "w") as f:
            f.write(prompt)
        if resp["status"] != "raw":
            attempts.append({
                "attempt": tag, "provider": provider, "model": model,
                "outcome": resp["status"], "raw_output_path": raw_path,
                "prompt_path": prompt_path})
            print(f"[generate:i{iteration}] {tag} {provider}: {resp['status']}",
                  flush=True)
            continue
        cands, err = parse_generation(resp["raw"], provider, model, corr)
        outcome = "ok" if cands is not None else f"schema_failure: {err}"
        attempts.append({
            "attempt": tag, "provider": provider, "model": model,
            "outcome": outcome, "raw_output_path": raw_path,
            "prompt_path": prompt_path})
        print(f"[generate:i{iteration}] {tag} {provider}: {outcome}", flush=True)
        if cands is not None:
            return cands, {
                "schema": "godspeed-bounded-judgment.t23-generation-record",
                "route_id": ROUTE_ID, "iteration": iteration,
                "correlation_id": corr,
                "provider_path": ("deepseek-clinepass" if tag == "primary"
                                  else "muse-substitution"),
                "generator_identity": provider,
                "attempts": attempts,
                "validation_rule": "route_contracts.candidate_move + "
                                   "correlation_id echo + filesystem-safe "
                                   "unique candidate_id",
            }, ("deepseek-clinepass" if tag == "primary"
                else "muse-substitution")
    cands = fallback_candidates(route_state, corr)
    attempts.append({
        "attempt": "deterministic-fallback",
        "provider": "deterministic-fallback-v1", "model": "n/a",
        "outcome": "ok (declared deterministic fallback; mechanics-only, "
                   "never counted as a provider result)"})
    print(f"[generate:i{iteration}] deterministic-fallback-v1 engaged",
          flush=True)
    return cands, {
        "schema": "godspeed-bounded-judgment.t23-generation-record",
        "route_id": ROUTE_ID, "iteration": iteration, "correlation_id": corr,
        "provider_path": "deterministic-fallback-v1",
        "generator_identity": "deterministic-fallback-v1",
        "attempts": attempts,
        "validation_rule": "frozen prereg generator_fallback_rule (seeds "
                           "corrected per addendum-1)",
    }, "deterministic-fallback-v1"


# ---------------------------------------------------------------------------
# Step 4: real probes (corrected fixture mechanism, strictly sequential)
# ---------------------------------------------------------------------------

def prepare_target(iter_key: str, rounds: int) -> str:
    target = os.path.join(DATA_ROOT, "targets", iter_key)
    if os.path.exists(target):
        fail(f"probe target dir already exists for {iter_key}: {target}")
    shutil.copytree(FIXTURE_TARGET_SRC, target)
    for script in sorted(os.listdir(os.path.join(target, "scripts"))):
        if script.endswith(".sh"):
            os.chmod(os.path.join(target, "scripts", script), 0o755)
    gauntlet_md = os.path.join(target, "GAUNTLET.md")
    with open(gauntlet_md) as f:
        md = f.read()
    if "default: mock-agent" not in md:
        fail(f"{iter_key}: the corrected fixture target does not carry the "
             "mock-agent runner default (addendum-1: NO runner sed)")
    if rounds != FIXTURE_DEFAULT_MAX_ROUNDS:
        lever = f"max_rounds: {FIXTURE_DEFAULT_MAX_ROUNDS}"
        if lever not in md:
            fail(f"{iter_key}: GAUNTLET.md does not contain the expected "
                 f"fixture-default lever {lever!r}")
        md = md.replace(lever, f"max_rounds: {rounds}", 1)
    with open(gauntlet_md, "w") as f:
        f.write(md)
    with open(gauntlet_md) as f:
        md2 = f.read()
    if rounds != FIXTURE_DEFAULT_MAX_ROUNDS and \
            f"max_rounds: {rounds}" not in md2:
        fail(f"{iter_key}: max_rounds sed did not take effect")
    return target


def run_probe(iteration: int, candidate: dict) -> tuple[dict, dict]:
    """Run ONE real gauntlet probe; return (probe_observation, run_meta).

    The route wall clock is checked by the caller before dispatch so a stop
    here never discards already-collected evidence."""
    cid = candidate["candidate_id"]
    scenario = candidate["bounded_input"]["scenario"]
    rounds = candidate["bounded_input"]["max_rounds"]
    iter_key = f"i{iteration}-{cid}"
    probe_id = f"probe-{iter_key}"
    target = prepare_target(iter_key, rounds)
    state_dir = os.path.join(DATA_ROOT, "runs", iter_key, "state")
    artifact_dir = os.path.join(DATA_ROOT, "runs", iter_key, "evidence")
    os.makedirs(state_dir, exist_ok=True)
    os.makedirs(artifact_dir, exist_ok=True)
    db_path = os.path.join(state_dir, prov.DB_FILENAME)
    if os.path.exists(db_path):
        fail(f"{iter_key}: state db already exists before the run: {db_path}")
    seed = f"{ROUTE_ID}-i{iteration}-{cid}-seed"

    env = dict(os.environ)
    for key in [k for k in env if k.startswith("GAUNTLET_")]:
        del env[key]  # defensive isolation: only the addendum env is admitted
    env.update({
        "GAUNTLET_STATE_DIR": state_dir,
        "GAUNTLET_ARTIFACT_DIR": artifact_dir,
        "GAUNTLET_MOCK_SCENARIO": scenario,
        "GAUNTLET_ID_SEED": seed,
        "GAUNTLET_CLOCK": "deterministic",
        "GAUNTLET_MAX_ATTEMPTS_PER_UNIT": "2",
        "GAUNTLET_TOOL_TEST": os.path.join(target, "scripts", "check.sh"),
        "PYTHONDONTWRITEBYTECODE": "1",
        **REQUIRED_CONTROL_ENV,
    })
    cmd = ["timeout", str(PROBE_TIMEOUT_S), GAUNTLET_BIN, "run", PROBE_CMD_WORD]
    started_at = utcnow()
    t0 = time.monotonic()
    proc = subprocess.run(cmd, capture_output=True, text=True, cwd=target,
                          env=env)
    wall = round(time.monotonic() - t0, 3)
    finished_at = utcnow()

    timed_out = proc.returncode == 124
    db_exists = os.path.isfile(db_path)
    settle_count = 0
    raw_terminal_state = None
    if db_exists:
        digest = sha256_file(db_path)
        try:
            log = prov.read_event_log(db_path)
        except Exception as exc:  # unreadable db degrades to a run_error
            log = {"run_terminal_states": [],
                   "settlement_committed_count": 0}
            print(f"[probe] {probe_id}: state db unreadable: {exc}", flush=True)
        states = log["run_terminal_states"]
        settle_count = log["settlement_committed_count"]
        if len(states) == 1:
            raw_terminal_state = states[0]
        elif len(states) > 1:
            fail(f"{probe_id}: expected at most one run_terminal event, "
                 f"found {len(states)}")
    else:
        digest = prov.ZERO_SHA

    terminal_class = prov.classify_terminal(raw_terminal_state)
    observation = {
        "probe_id": probe_id,
        "candidate_id": cid,
        "correlation_id": f"{ROUTE_ID}-iter{iteration}",
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
    # Mechanism-provenance gate (absorb-time, fail-closed): the claimed
    # consequence must be backed by the run's own durable state database.
    ok, reason = prov.validate(observation)
    if not ok:
        fail(f"{probe_id}: provenance validation refused this route's own "
             f"observation: {reason}")
    meta = {
        "probe_id": probe_id,
        "candidate_id": cid,
        "iteration": iteration,
        "gauntlet_id_seed": seed,
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
        "max_rounds_sed_applied": rounds != FIXTURE_DEFAULT_MAX_ROUNDS,
        "note": ("state_db_sha256 is the digest of zero bytes when no state "
                 "db was produced (run_error); the path is the expected "
                 "frozen-mechanism path for this candidate"
                 if not db_exists else
                 "state_db_sha256 is the digest of the real gauntlet state "
                 "database"),
    }
    print(f"[probe] {probe_id} scenario={scenario} rounds={rounds} -> "
          f"{terminal_class} (exit={proc.returncode}, wall={wall}s, "
          f"settlements={settle_count})", flush=True)
    return observation, meta


# ---------------------------------------------------------------------------
# Step 5: bounded judgment (observation records only; never generation context)
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


def judge_probe(surface: dict, corr: str) -> tuple[dict, dict]:
    probe_id = surface["probe_id"]
    cid = surface["candidate_id"]
    label = None
    path_record = {"probe_id": probe_id, "candidate_id": cid, "attempts": []}
    for tag, provider, model in (
            ("primary", PRIMARY_PROVIDER, PRIMARY_MODEL),
            ("substitution", SUB_PROVIDER, SUB_MODEL)):
        prompt = judgment_prompt(surface)
        resp = call_provider(provider, model, prompt)
        provider_calls[-1]["purpose"] = f"probe_judgment:{probe_id}"
        raw_path = save_raw(corr, f"judgment-{probe_id}-{tag}.raw.txt",
                            resp["raw"])
        prompt_path = os.path.join(
            RAW_DIR, f"{corr}-judgment-{probe_id}-{tag}.prompt.txt")
        with open(prompt_path, "w") as f:
            f.write(prompt)
        if resp["status"] != "raw":
            path_record["attempts"].append({
                "attempt": tag, "provider": provider, "model": model,
                "outcome": resp["status"], "raw_output_path": raw_path})
            print(f"[judge] {probe_id} {tag} {provider}: {resp['status']}",
                  flush=True)
            continue
        parsed, err = parse_judgment(resp["raw"])
        outcome = "ok" if parsed is not None else f"schema_failure: {err}"
        path_record["attempts"].append({
            "attempt": tag, "provider": provider, "model": model,
            "outcome": outcome, "raw_output_path": raw_path,
            "abstention": parsed == "insufficient_evidence" if parsed else None})
        print(f"[judge] {probe_id} {tag} {provider}: {outcome} label={parsed}",
              flush=True)
        if parsed is not None:
            label = parsed
            break
    if label is None:
        label = MECHANICAL_TERMINAL_MAP[surface["terminal_class"]]
        path_record["attempts"].append({
            "attempt": "deterministic-fallback",
            "provider": "deterministic-terminal-mapping-v1",
            "model": "n/a",
            "outcome": f"ok (mechanics-only mechanical mapping "
                       f"{surface['terminal_class']} -> {label}; never counted "
                       "as a provider result)"})
        judge_identity = "deterministic-terminal-mapping-v1 (mechanics-only)"
    else:
        winning = path_record["attempts"][-1]
        judge_identity = f"{winning['provider']}:{winning['model']}"
    judgment = {
        "judgment_id": f"judg-{cid}",
        "candidate_id": cid,
        "correlation_id": corr,
        "label": label,
        "observation_refs": [probe_id],
        "judge_identity": judge_identity,
        "judged_at_utc": utcnow(),
    }
    err = rc.validate("judgment_result", judgment)
    if err is not None:
        fail(f"internal: invalid judgment_result for {cid}: {err}")
    return judgment, path_record


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

ITERATION_FILES = [
    "neighborhood.yaml",
    "generation-record.yaml",
    "candidates.json",
    "admission-decisions.json",
    "probe-observations.json",
    "probe-run-meta.yaml",
    "judgment-surfaces.json",
    "judgment-results.json",
    "judgment-record.yaml",
    "selection-record.json",
    "route-state-after.yaml",
]


def main() -> int:
    route_started = time.monotonic()
    os.makedirs(RAW_DIR, exist_ok=True)
    if os.path.exists(os.path.join(ROUTE_DIR, "route-record.yaml")) or \
            any(os.path.isdir(os.path.join(ROUTE_DIR, e)) for e in os.listdir(ROUTE_DIR)
                if e.startswith("iteration-")):
        fail("append-only violation: a previous t23-route-001 run's records "
             "already exist under route/; a correction uses a round-2 script "
             "and its own record dirs, never an overwrite")
    if not os.access(GAUNTLET_BIN, os.X_OK):
        fail(f"gauntlet binary missing or not executable: {GAUNTLET_BIN}")
    if not os.path.isdir(FIXTURE_TARGET_SRC):
        fail(f"corrected fixture target missing: {FIXTURE_TARGET_SRC}")
    if os.path.exists(DATA_ROOT):
        fail(f"probe data root already exists: {DATA_ROOT}")
    os.makedirs(DATA_ROOT, exist_ok=False)

    import_verification = verify_frozen_inputs()
    with open(os.path.join(T21_DIR, "operations_catalog.yaml")) as f:
        catalog = yaml.safe_load(f)
    if len(catalog["operations"]) != 24:
        fail("operations catalog does not contain the frozen 24 operations")

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

    stop_condition = None
    stop_note = ""
    iterations_summary: list[dict] = []
    total_probes = 0

    try:
        for iteration in range(1, MAX_ITERATIONS + 1):
            corr = f"{ROUTE_ID}-iter{iteration}"
            iter_dir = os.path.join(ROUTE_DIR, f"iteration-{iteration}")
            os.makedirs(iter_dir, exist_ok=False)
            check_route_wall(route_started)

            # 1. bounded neighborhood
            neighborhood = build_neighborhood(route_state, catalog, iteration,
                                              corr)
            dump_yaml(os.path.join(iter_dir, "neighborhood.yaml"), neighborhood)
            print(f"[neighborhood:i{iteration}] "
                  f"{neighborhood['neighborhood_size']} operations presented",
                  flush=True)
            if neighborhood["neighborhood_size"] == 0:
                raise StopRoute(
                    "no_eligible_candidate",
                    "the bounded neighborhood is empty: every catalog "
                    "operation is probed, denied, or unaffordable on this route")

            # 2. generate K=3 candidates (attempts are collected outside so a
            # mid-generation wall stop still leaves its partial record)
            gen_attempts: list = []
            try:
                candidates, generation_record, provider_path = \
                    generate_candidates(neighborhood, route_state,
                                        route_started, gen_attempts)
            except StopRoute as stop:
                dump_yaml(os.path.join(iter_dir,
                                       "generation-record-partial.yaml"),
                          {"schema": "godspeed-bounded-judgment."
                                     "t23-generation-record-partial",
                           "route_id": ROUTE_ID, "iteration": iteration,
                           "correlation_id": corr, "attempts": gen_attempts,
                           "stopped": f"{stop.condition}: {stop.note}"})
                raise
            dump_yaml(os.path.join(iter_dir, "generation-record.yaml"),
                      generation_record)
            dump_json(os.path.join(iter_dir, "candidates.json"), candidates)
            for c in candidates:
                err = rc.validate("candidate_move", c)
                if err is not None:
                    fail(f"internal: invalid candidate "
                         f"{c.get('candidate_id')!r}: {err}")

            # 3. admission through the frozen T21 module (live route state)
            admissions = []
            for c in candidates:
                decision = adm.classify(c, route_state, catalog,
                                        payment=route_state["probes_remaining"])
                err = rc.validate("admission_decision", decision)
                if err is not None:
                    fail(f"internal: invalid admission_decision: {err}")
                admissions.append(decision)
                print(f"[admit:i{iteration}] {c['candidate_id']} -> "
                      f"{decision['classification']}", flush=True)
            dump_json(os.path.join(iter_dir, "admission-decisions.json"),
                      admissions)
            admitted = [c for c, a in zip(candidates, admissions)
                        if a["classification"] == "admitted"]
            if not admitted:
                raise StopRoute(
                    "no_eligible_candidate",
                    "no candidate of this iteration was admitted; the route "
                    "cannot advance with the frozen K=3 generation ladder")

            # 4. real probes, strictly sequential (one gauntlet process at a
            # time); a wall-clock stop between probes finalizes the partial
            # iteration below instead of discarding collected evidence
            observations, metas = [], []
            surfaces, judgments, judgment_paths = [], [], []
            mid_stop = None
            for c in admitted:
                try:
                    check_route_wall(route_started)
                except StopRoute as stop:
                    mid_stop = stop
                    break
                obs, meta = run_probe(iteration, c)
                observations.append(obs)
                metas.append(meta)
                total_probes += 1
            dump_json(os.path.join(iter_dir, "probe-observations.json"),
                      observations)
            dump_yaml(os.path.join(iter_dir, "probe-run-meta.yaml"),
                      {"schema": "godspeed-bounded-judgment.t23-probe-run-meta",
                       "route_id": ROUTE_ID, "iteration": iteration,
                       "fixture_mechanism": (
                           "addendum-1: fresh tests/fixtures/target copy per "
                           "probe; no runner sed; max_rounds sed only when the "
                           "probe value != 4; GAUNTLET_ID_SEED unique per probe"),
                       "note": "typed sidecar for exit codes, wall seconds and "
                               "the per-probe GAUNTLET_ID_SEED; the frozen "
                               "probe_observation schema has an exact field "
                               "set and refuses extra fields",
                       "probes": metas})

            # 5. bounded judgment from observation records ONLY
            for obs in observations:
                surface = {
                    "probe_id": obs["probe_id"],
                    "candidate_id": obs["candidate_id"],
                    "correlation_id": obs["correlation_id"],
                    "operation": obs["operation"],
                    "bounded_input": obs["bounded_input"],
                    "terminal_class": obs["terminal_class"],
                    "settlement_committed_count":
                        obs["settlement_committed_count"],
                    "state_db_sha256": obs["state_db_sha256"],
                    "mechanism": obs["mechanism"],
                    "started_at_utc": obs["started_at_utc"],
                    "finished_at_utc": obs["finished_at_utc"],
                }
                surfaces.append(surface)
                judgment, path_record = judge_probe(surface, corr)
                judgments.append(judgment)
                judgment_paths.append(path_record)
            dump_json(os.path.join(iter_dir, "judgment-surfaces.json"), surfaces)
            dump_json(os.path.join(iter_dir, "judgment-results.json"), judgments)
            dump_yaml(os.path.join(iter_dir, "judgment-record.yaml"),
                      {"schema": "godspeed-bounded-judgment.t23-judgment-record",
                       "route_id": ROUTE_ID, "iteration": iteration,
                       "judge_input_rule": "observation records only (typed "
                                           "terminal, settlement_committed "
                                           "count, artifact digest, timing); "
                                           "never the builder/provider "
                                           "narrative, never the route goal",
                       "probe_judgments": judgment_paths})

            # absorb probes into the route state (evidence is never discarded)
            label_by_candidate = {j["candidate_id"]: j["label"]
                                  for j in judgments}
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
            selection = sp.select(candidates, admissions, observations,
                                  judgments, route_state,
                                  payment=route_state["probes_remaining"],
                                  correlation_id=corr, iteration=iteration)
            dump_json(os.path.join(iter_dir, "selection-record.json"),
                      selection)
            print(f"[select:i{iteration}] "
                  f"retained={selection['retained_candidate_id']!r} "
                  f"eligible={selection['eligible_candidate_ids']}", flush=True)

            if selection["retained_candidate_id"] != "":
                cid = selection["retained_candidate_id"]
                cand = next(c for c in candidates
                            if c["candidate_id"] == cid)
                obs = next(o for o in observations
                           if o["candidate_id"] == cid)
                route_state["retained_moves"].append({
                    "candidate_id": cid,
                    "correlation_id": corr,
                    "scenario": cand["bounded_input"]["scenario"],
                    "max_rounds": cand["bounded_input"]["max_rounds"],
                    "expected_role": cand["expected_role"],
                    "judgment_label": label_by_candidate[cid],
                })
                err = rc.validate("route_state", route_state)
                if err is not None:
                    fail(f"internal: invalid route_state after retention: {err}")

            goal = goal_evaluation(route_state)
            dump_yaml(os.path.join(iter_dir, "route-state-after.yaml"),
                      {"schema": "godspeed-bounded-judgment.t23-route-state",
                       "route_id": ROUTE_ID, "iteration": iteration,
                       "correlation_id": corr,
                       "provider_path": provider_path,
                       "goal_evaluation": goal,
                       "route_state": route_state})
            iterations_summary.append({
                "iteration": iteration,
                "correlation_id": corr,
                "provider_path": provider_path,
                "probes_run": len(observations),
                "retained_candidate_id": selection["retained_candidate_id"],
                "goal_evaluation": goal,
            })
            if mid_stop is not None:
                raise mid_stop

            # frozen stop conditions, checked in the frozen list order
            if goal["goal_met"]:
                raise StopRoute("route_settled",
                                "corrected route goal met on this route's "
                                "own ledger (baseline + discriminator + "
                                "boundary r*/r*-1)")
            if route_state["probes_remaining"] <= 0:
                raise StopRoute("probe_budget_exhausted",
                                "the frozen 12-probe budget is spent")
            run_errors = [p for p in route_state["probed"]
                          if p["terminal_class"] == "run_error"]
            if len(run_errors) >= 3 and len(run_errors) == \
                    len(route_state["probed"]):
                raise StopRoute(
                    "unrecoverable_execution_failure",
                    "every probe on this route (>= 3 distinct candidates) "
                    "ended in a run error with no typed terminal anywhere")
            if len(route_state["retained_moves"]) >= MAX_RETAINED:
                raise StopRoute(
                    "no_eligible_candidate",
                    "the frozen retained-move depth cap (5) is reached with "
                    "the goal unmet; the route_state schema refuses deeper "
                    "routes")
    except StopRoute as stop:
        stop_condition = stop.condition
        stop_note = stop.note
        print(f"[stop] {stop.condition}: {stop.note}", flush=True)
    if stop_condition is None:
        stop_condition = "iteration_limit_reached"
        stop_note = (f"the frozen iteration limit ({MAX_ITERATIONS}) was "
                     "reached with the goal unmet")

    settled = stop_condition == "route_settled"
    moves = [{"scenario": m["scenario"], "max_rounds": m["max_rounds"],
              "terminal_class": next(p["terminal_class"]
                                     for p in route_state["probed"]
                                     if p["candidate_id"] == m["candidate_id"]
                                     and p["correlation_id"] ==
                                     m["correlation_id"])}
             for m in route_state["retained_moves"]]
    route_record = {
        "route_id": ROUTE_ID,
        "correlation_id": f"{ROUTE_ID}-iter{len(iterations_summary)}",
        "settled": settled,
        "stop_condition": stop_condition,
        "moves": moves,
        "fingerprint": rc.route_fingerprint(moves),
        "correlation_ids": [f"{ROUTE_ID}-iter{i}" for i in
                            range(1, len(iterations_summary) + 1)],
        "policy_version": sp.POLICY_VERSION,
        "prereg_sha256": PREREG_SHA256,
        "opened_at_utc": route_state["created_at_utc"],
        "closed_at_utc": utcnow(),
    }
    err = rc.validate("route_record", route_record)
    if err is not None:
        fail(f"internal: invalid route_record: {err}")
    final_goal = goal_evaluation(route_state)
    dump_yaml(os.path.join(ROUTE_DIR, "route-record.yaml"), {
        "schema": "godspeed-bounded-judgment.t23-route-record",
        "correction_round": CORRECTION_ROUND,
        "fixture_mechanism": (
            "addendum-1 corrected mechanism: fresh tests/fixtures/target copy "
            "per probe; runners.default already mock-agent (NO runner sed); "
            "GAUNTLET.md budget.max_rounds (fixture default 4) sed'd only for "
            "probe values != 4; addendum env incl. per-probe GAUNTLET_ID_SEED; "
            "GAUNTLET_CLOCK=deterministic + required controls; timeout 240"),
        "probe_data_root": DATA_ROOT,
        "goal_definition": {"goal": final_goal["goal"],
                            "prior_knowledge_note": PRIOR_KNOWLEDGE_NOTE},
        "stop_note": stop_note,
        "goal_evaluation": final_goal,
        "route_state": route_state,
        "route_record": route_record,
        "iterations": iterations_summary,
        "note": "a probe terminal is a probe consequence, never route "
                "settlement; settled is decided ONLY by the corrected goal "
                "evaluation over this route's own ledger and is recomputed "
                "independently by verify_route.py",
    })

    payment = {
        "schema": "godspeed-bounded-judgment.t23-payment-counters",
        "task": "T23",
        "route_id": ROUTE_ID,
        "correction_round": CORRECTION_ROUND,
        "wall_clock_seconds": round(time.monotonic() - route_started, 3),
        "provider_calls": {
            "count": len(provider_calls),
            "by_identity": [
                {"seq": i + 1, "purpose": c["purpose"],
                 "provider": c["provider"], "model": c["model"],
                 "outcome": c["outcome"],
                 "started_at_utc": c["started_at_utc"],
                 "duration_seconds": c["duration_seconds"]}
                for i, c in enumerate(provider_calls)
            ],
        },
        "probes_run": total_probes,
        "tool_executions": total_probes,
        "failed_executions": sum(1 for p in route_state["probed"]
                                 if p["terminal_class"] == "run_error"),
        "tokens_and_cost": "n/a: the frozen provider seam (opencode run) does "
                           "not expose token/cost accounting; honest gap per "
                           "prereg payment_counters_frozen.tokens_and_cost",
        "operator_decisions": 0,
        "probe_data_root": DATA_ROOT,
        "note": "teeth tool executions are counted separately in the teeth "
                "records (this file covers the route run only)",
    }
    dump_yaml(os.path.join(ROUTE_DIR, "payment-counters.yaml"), payment)

    print(f"[done] route {ROUTE_ID}: settled={settled} "
          f"stop={stop_condition}; probes={total_probes}; "
          f"retained={len(route_state['retained_moves'])}; "
          f"fingerprint={route_record['fingerprint']!r}; "
          f"wall={payment['wall_clock_seconds']}s", flush=True)
    return 0 if settled else 2


if __name__ == "__main__":
    sys.exit(main())
