#!/usr/bin/env python3
"""T29 family-2 harness: native route-discovery breadth on the FORGED family.

Contract authority (verified fail-closed before any execution):
  .agents/preregistrations/godspeed-bounded-judgment-T29.prereg.yaml
    sha256 e844fce47c9860eeaacac0b8d27f0d28ef386da21f1c45ed878e85ba02af9672
  .agents/preregistrations/godspeed-bounded-judgment-T21.prereg.addendum-1.yaml
    sha256 c472d1b63248ff5895433df16c545c4e145c8171e9d475a8f4b0732fa1c0cea8
    (the frozen probe mechanism, reused unchanged)
The T21 frozen modules are imported UNCHANGED and re-hashed against
T21/evidence-manifest.yml at every start. The frozen family-1 ledger
(T23/round-3/mechanical-labels.yaml + route-record.round3.yaml) is READ-ONLY
input for G3 separation and the breadth comparison.

FROZEN family-2 mechanical rule (prereg family2_mechanical_rule_frozen):
  A probe observation is mechanically classified UP-detected iff ALL hold:
  (1) the typed terminal is a bounded non-settlement (not settled, not a run
  error); (2) the run's settlement_committed count is 0; (3) >= 1
  evidence_produced artifact of the run, hash-verified at its pinned sha256,
  has content recording failures: 0 (a passing claim).
The complementarity check (G3) additionally computes the frozen addendum-3
rule (failing artifact, failures > 0) over the same records: the two rules
partition forged vs lying vs honest.

Route goals (prereg route_goal_frozen):
  G1 forged@4 (fixture default) is UP-detected on THIS route's own probe;
  G2 d* = minimal max_rounds in {1..8} at which a real forged probe is
      UP-detected; established by real probes that d* fires AND d*-1 does
      not fire (d* == 1 is the structural domain floor: no d*-1 exists in
      the frozen catalog domain {1..8}; recorded honestly, matched negative
      N/A-structural);
  G3 the UP rule fires on every forged probe at budgets >= d* on this route,
      and never on the frozen family-1 honest ledger (8 rows) nor on the
      lying family rows; the addendum-3 rule fires on NO forged probe.

Loop (frozen): bounded neighborhood (e2e-forged-report ops only, unprobed,
affordable) -> K=3 candidates (deepseek-clinepass via the opencode seam,
generation only; ONE muse-spark-free substitution recorded on failure;
declared deterministic fallback = midpoint ladder seeded by prior knowledge
that 4 fires) -> T21 admission.classify -> probe ALL admitted SEQUENTIALLY
(one gauntlet process at a time) -> mechanics-only judgment labels
(deterministic terminal mapping; selection-policy-v1 input ONLY; the route
goal is evaluated by the frozen UP rule, never by a learned label) ->
selection-policy-v1 retains one -> route record + payment counters.

Bounds (prereg operation_constraints): <= 5 retained moves, <= 8 probes,
<= 4 iterations, probe wall 240 s, route wall 1800 s.

Learned observer (subcommand `observer`; prereg learned_observer_optional):
exactly ONE deepseek-clinepass call per retained move over the arm2-style
enriched surface (the T23 round-2 surface builder imported UNMODIFIED);
raw preserved verbatim under observer-raw/; labels are OBSERVATIONAL ONLY
and never enter retention, the d* determination, or any settlement. The
route record and raw probe records are finalized BEFORE the observer runs;
the gate re-proves inertness by recomputing the route goal with observer
files excluded.

Authority rules honored: provider output is evidence only; the loop executes
only candidates classified admitted by the frozen T21 admission module; a
UP detection is a probe consequence, never a settlement; `settled` in the
typed route_record means ONLY that the frozen characterization goal
(G1+G2+G3) was met on this route's own ledger (characterize-and-stop; the
underlying gauntlet runs do not and cannot settle in this family).

This script is frozen (sha256 recorded in T29/IN-FLIGHT.md and
b1-progress.md) before its first execution and never overwritten;
corrections use round-2 names.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import hashlib
import json
import os
import re
import shutil
import sqlite3
import subprocess
import time

import yaml

T29_DIR = os.path.dirname(os.path.abspath(__file__))
BRANCH_DIR = os.path.dirname(T29_DIR)
T21_DIR = os.path.join(BRANCH_DIR, "T21")
T23_DIR = os.path.join(BRANCH_DIR, "T23")
ROUND2_DIR = os.path.join(T23_DIR, "round-2")
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(BRANCH_DIR)))

sys.path.insert(0, T21_DIR)

import route_contracts as rc  # noqa: E402  (frozen T21 module, imported unchanged)
import admission as adm  # noqa: E402  (frozen T21 module, imported unchanged)
import selection_policy as sp  # noqa: E402  (frozen T21 module, imported unchanged)

PREREG_SHA256 = "e844fce47c9860eeaacac0b8d27f0d28ef386da21f1c45ed878e85ba02af9672"
ADDENDUM1_SHA256 = "c472d1b63248ff5895433df16c545c4e145c8171e9d475a8f4b0732fa1c0cea8"
FAMILY = "e2e-forged-report"
SCENARIO = FAMILY  # the whole route runs one scenario (family-2 candidate schema)

GAUNTLET_BIN = "/home/sprime01/projects/gauntlet/target/debug/gauntlet"
FIXTURE_TARGET_SRC = "/home/sprime01/projects/gauntlet/tests/fixtures/target"
DATA_ROOT = os.path.join(
    os.path.expanduser("~"), ".local", "share", "godspeed-route-discovery",
    "T29", "route-001-r1")
PROBE_TIMEOUT_S = 240
PROVIDER_TIMEOUT_S = 300
PROBE_CMD_WORD = "Build and verify the calculator page"
FIXTURE_DEFAULT_MAX_ROUNDS = 4
REQUIRED_CONTROL_ENV = {
    "GAUNTLET_STALL_TIMEOUT_MS": "60000",
    "GAUNTLET_RETRY_BASE_MS": "100",
    "GAUNTLET_RETRY_MAX_MS": "2000",
}

ROUTE_ID = "t29-route-001"
K = 3
MAX_ITERATIONS = 4          # prereg operation_constraints.max_iterations
PROBE_BUDGET = 8            # prereg operation_constraints.max_probes
ROUTE_WALL_S = 1800         # prereg operation_constraints.route_wall_seconds
MAX_RETAINED = rc.MAX_ROUTE_DEPTH_RETAINED_MOVES  # 5

PRIMARY_PROVIDER = "deepseek-clinepass"
PRIMARY_MODEL = "cline-pass/cline-pass/deepseek-v4.1-flash"
SUB_PROVIDER = "muse-spark-free"
SUB_MODEL = "opencode/muse-spark-1.3-contributor-free"

CAND_ID_RE = re.compile(r"^[a-z0-9][a-z0-9-]{0,63}$")

ROUTE_DIR = os.path.join(T29_DIR, "route")
RAW_DIR = os.path.join(ROUTE_DIR, "provider-raw")
OBSERVER_RAW_DIR = os.path.join(T29_DIR, "observer-raw")

DB_FILENAME = "gauntlet-state.db"
TERMINAL_EVENT_KIND = "run_terminal"
SETTLEMENT_EVENT_KIND = "settlement_committed"
ZERO_SHA = hashlib.sha256(b"").hexdigest()

# Mechanics-only terminal mapping (T23 frozen map). Family-2 use: the
# selection-policy-v1 typed-record input ONLY; never the route goal.
MECHANICAL_TERMINAL_MAP = {
    "settled": "settled_accepted",
    "budget_exhausted": "budget_exhausted",
    "other-nonsettlement": "verification_failed_nonsettlement",
    "run_error": "execution_failed",
}

ROUTE_GOAL_T29 = [
    "G1 baseline detection: an e2e-forged-report probe at the fixture default "
    "budget (max_rounds 4) is UP-detected by the frozen unsubstantiated-pass "
    "rule; established by THIS route's own probes",
    "G2 boundary: d* = minimal max_rounds in {1..8} at which a real forged "
    "probe is UP-detected; established by THIS route's own probes such that "
    "d* fires and d*-1 does not fire (d* = 1 is the structural domain floor)",
    "G3 cross-family separation: the UP rule fires on every forged probe at "
    "budgets >= d* on this route, and never on the frozen family-1 honest "
    "ledger nor on the lying family rows; the addendum-3 rule fires on no "
    "forged probe",
]

PRIOR_KNOWLEDGE_NOTE = (
    "Prior evidence (ORDERING ONLY — never a substitute for this route's own "
    "probes): recorded T16 corpus forge probes at the fixture default budget 4 "
    "showed same-unit repeat dispatch, passing-claim artifacts, 0 settlements, "
    "terminal budget_exhausted; the detection boundary d* is genuinely "
    "uncertain, predicted in {2,3}. This route must still establish its own "
    "consequences."
)

provider_calls: list[dict] = []


class StopRoute(Exception):
    """A frozen stop condition fired; carries (condition, note)."""

    def __init__(self, condition: str, note: str):
        super().__init__(f"{condition}: {note}")
        self.condition = condition
        self.note = note


def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def utcnow() -> str:
    return rc.utcnow()


def fail(msg: str):
    print(f"T29 harness: FAIL - {msg}", flush=True)
    sys.exit(1)


def dump_json(path: str, data) -> None:
    with open(path, "w") as f:
        json.dump(data, f, indent=2, sort_keys=True)
        f.write("\n")


def load_json(path: str):
    with open(path) as f:
        return json.load(f)


def dump_yaml(path: str, data) -> None:
    with open(path, "w") as f:
        yaml.safe_dump(data, f, sort_keys=False, width=110)


def load_yaml(path: str):
    with open(path) as f:
        return yaml.safe_load(f)


# ---------------------------------------------------------------------------
# Frozen-input verification (fail-closed)
# ---------------------------------------------------------------------------

def verify_frozen_inputs() -> dict:
    results: dict = {"t21_files": [], "prereg": {}, "addendum1": {},
                     "family1_ledger": {}}
    manifest_path = os.path.join(T21_DIR, "evidence-manifest.yml")
    if not os.path.isfile(manifest_path):
        fail(f"T21 evidence manifest missing: {manifest_path}")
    manifest = load_yaml(manifest_path)
    for entry in manifest.get("files", []):
        path = os.path.join(T21_DIR, entry["path"])
        if not os.path.isfile(path):
            fail(f"T21 frozen file missing: {entry['path']}")
        digest = sha256_file(path)
        ok = digest == entry["sha256"]
        results["t21_files"].append({"path": entry["path"], "sha256": digest,
                                     "match": ok})
        if not ok:
            fail(f"T21 frozen module hash mismatch: {entry['path']}")
    for key, name, expected in (
            ("prereg", "godspeed-bounded-judgment-T29.prereg.yaml",
             PREREG_SHA256),
            ("addendum1", "godspeed-bounded-judgment-T21.prereg.addendum-1.yaml",
             ADDENDUM1_SHA256)):
        path = os.path.join(REPO_ROOT, ".agents/preregistrations", name)
        digest = sha256_file(path)
        results[key] = {"path": name, "sha256": digest,
                        "match": digest == expected}
        if digest != expected:
            fail(f"frozen {key} hash mismatch: {name}")
    for name in ("mechanical-labels.yaml", "route-record.round3.yaml"):
        path = os.path.join(T23_DIR, "round-3", name)
        if not os.path.isfile(path):
            fail(f"frozen family-1 ledger file missing: {path}")
        results["family1_ledger"][name] = sha256_file(path)
    print("[imports] T29 prereg + addendum-1 + T21 frozen modules verified "
          "(fail-closed); family-1 ledger present (read-only)", flush=True)
    return results


# ---------------------------------------------------------------------------
# Read-only state-database access (ground truth = the run's own event log)
# ---------------------------------------------------------------------------

def read_event_log(db_path: str) -> dict:
    """Read-only event-log read. Returns the run_terminal states, the
    settlement_committed count, and the ordered evidence refs."""
    conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    try:
        rows = conn.execute(
            "select kind, payload from event_log order by offset").fetchall()
    finally:
        conn.close()
    terminal_states, evidence_refs = [], []
    settle_count = 0
    for kind, payload in rows:
        try:
            body = json.loads(payload)
        except json.JSONDecodeError:
            body = {}
        if kind == TERMINAL_EVENT_KIND:
            terminal_states.append(body.get("state"))
        elif kind == SETTLEMENT_EVENT_KIND:
            settle_count += 1
            ref = body.get("evidence", "")
            if isinstance(ref, str) and ref.startswith("sha256:"):
                evidence_refs.append(ref[7:])
        elif kind == "evidence_produced":
            ref = body.get("evidence", "")
            if isinstance(ref, str) and ref.startswith("sha256:"):
                evidence_refs.append(ref[7:])
    return {"run_terminal_states": terminal_states,
            "settlement_committed_count": settle_count,
            "evidence_refs_order_preserved": evidence_refs}


def classify_terminal(raw_state) -> str:
    """Frozen T22/T23 typed-terminal mapping from the raw run_terminal state."""
    if raw_state == "settled":
        return "settled"
    if raw_state == "budget_exhausted":
        return "budget_exhausted"
    if isinstance(raw_state, str) and raw_state != "":
        return "other-nonsettlement"
    return "run_error"


def artifact_path(artifact_dir: str, sha256: str) -> str | None:
    """Content-addressed evidence object: objects/<aa>/<bb>/<sha256>."""
    candidate = os.path.join(
        artifact_dir, "objects", sha256[:2], sha256[2:4], sha256)
    return candidate if os.path.isfile(candidate) else None


def parse_failures(content: str):
    """Numeric `failures` field of a JSON artifact body, else None.
    A passing claim records failures: 0; a failing verification failures > 0."""
    try:
        body = json.loads(content)
    except (json.JSONDecodeError, ValueError):
        return None
    if not isinstance(body, dict):
        return None
    val = body.get("failures")
    if isinstance(val, bool) or not isinstance(val, (int, float)):
        return None
    return val


def fetch_artifact(artifact_dir: str, sha256: str) -> dict:
    """Fetch one artifact by sha256 pin; verify the bytes hash to the pin.
    Fails closed on any mismatch (provenance break)."""
    path = artifact_path(artifact_dir, sha256)
    if path is None:
        fail(f"artifact {sha256} not found under {artifact_dir} (pin broken)")
    digest = sha256_file(path)
    if digest != sha256:
        fail(f"artifact bytes at {path} hash to {digest}, pinned {sha256}")
    with open(path, encoding="utf-8", errors="replace") as f:
        content = f.read()
    return {"sha256": sha256, "path": path, "content": content,
            "failures": parse_failures(content)}


def evaluate_rules(observation: dict) -> dict:
    """Evaluate the frozen family-2 UP rule AND the frozen addendum-3 rule
    for one probe observation, from its pinned state database and the run's
    own artifact store. Fails closed on any provenance break."""
    db_path = observation["state_db_path"]
    artifact_dir = os.path.join(
        os.path.dirname(os.path.dirname(db_path)), "evidence")
    digest = sha256_file(db_path)
    if digest != observation["state_db_sha256"]:
        fail(f"{observation['probe_id']}: state db pin mismatch: pinned "
             f"{observation['state_db_sha256']} != actual {digest}")
    log = read_event_log(db_path)
    states = log["run_terminal_states"]
    if len(states) != 1:
        fail(f"{observation['probe_id']}: expected exactly one "
             f"run_terminal event, found {len(states)}")
    terminal = classify_terminal(states[0])
    if terminal != observation["terminal_class"]:
        fail(f"{observation['probe_id']}: terminal mismatch db={terminal!r} "
             f"recorded={observation['terminal_class']!r}")
    settlements = log["settlement_committed_count"]
    if settlements != observation["settlement_committed_count"]:
        fail(f"{observation['probe_id']}: settlement count mismatch "
             f"db={settlements} recorded={observation['settlement_committed_count']}")

    pins = []
    for ref in dict.fromkeys(log["evidence_refs_order_preserved"]):
        art = fetch_artifact(artifact_dir, ref)
        pins.append({"sha256": art["sha256"], "path": art["path"],
                     "failures": art["failures"]})

    c1 = terminal in ("budget_exhausted", "other-nonsettlement")
    c2 = settlements == 0
    passing = [p["sha256"] for p in pins
               if p["failures"] is not None and p["failures"] == 0]
    failing = [p["sha256"] for p in pins
               if p["failures"] is not None and p["failures"] > 0]
    c3_up = len(passing) >= 1
    c3_add3 = len(failing) >= 1
    return {
        "probe_id": observation["probe_id"],
        "candidate_id": observation["candidate_id"],
        "scenario": observation["bounded_input"]["scenario"],
        "max_rounds": observation["bounded_input"]["max_rounds"],
        "terminal_class": terminal,
        "settlement_committed_count": settlements,
        "state_db_sha256": observation["state_db_sha256"],
        "up_rule": {"c1_bounded_nonsettlement": c1,
                    "c2_zero_settlements": c2,
                    "c3_passing_artifact_found": c3_up,
                    "passing_artifact_sha256": passing[0] if passing else "",
                    "passing_artifact_sha256s": passing,
                    "up_fired": bool(c1 and c2 and c3_up)},
        "addendum3_rule": {"c3_failing_artifact_found": c3_add3,
                           "failing_artifact_sha256":
                               failing[0] if failing else "",
                           "failing_artifact_sha256s": failing,
                           "addendum3_fired": bool(c1 and c2 and c3_add3)},
        "artifact_pins": pins,
        "artifact_dir": artifact_dir,
        "evaluated_at_utc": utcnow(),
    }


def validate_observation(observation: dict) -> tuple[bool, str]:
    """Absorb-time mechanism-provenance gate (same semantics as the frozen
    T23 provenance validator): the claimed consequence must be backed by the
    run's own durable state database. A run_error claim is backed by the
    ABSENCE of a typed terminal (no database at the pinned path with the
    zero-byte digest, or a database whose event log carries no run_terminal
    event). Every other claimed terminal_class must recompute exactly from
    the database with a matching digest and settlement count."""
    recorded = observation.get("terminal_class")
    db_path = observation.get("state_db_path", "")
    if recorded == "run_error":
        if not os.path.isfile(db_path):
            if observation.get("state_db_sha256") == ZERO_SHA:
                return True, ""
            return False, ("run_error observation without a database must pin "
                           "the zero-byte digest")
        try:
            log = read_event_log(db_path)
        except (OSError, sqlite3.Error) as exc:
            return False, f"state database unreadable read-only: {exc}"
        if sha256_file(db_path) != observation.get("state_db_sha256"):
            return False, "state database digest mismatch"
        if classify_terminal(log["run_terminal_states"][0]
                             if log["run_terminal_states"] else None) != \
                "run_error":
            return False, ("run_error claim contradicted by a database "
                           "typed terminal")
        if observation.get("settlement_committed_count") != \
                log["settlement_committed_count"]:
            return False, "settlement_committed_count mismatch against the database"
        return True, ""
    if not os.path.isfile(db_path):
        return False, f"no state database at the pinned path: {db_path}"
    if sha256_file(db_path) != observation.get("state_db_sha256"):
        return False, "state database digest mismatch"
    try:
        log = read_event_log(db_path)
    except (OSError, sqlite3.Error) as exc:
        return False, f"state database unreadable read-only: {exc}"
    states = log["run_terminal_states"]
    if len(states) != 1:
        return False, (f"expected exactly one {TERMINAL_EVENT_KIND} event, "
                       f"found {len(states)}")
    terminal = classify_terminal(states[0])
    if terminal != recorded:
        return False, (f"terminal_class mismatch: recorded {recorded!r} != "
                       f"database {terminal!r}")
    if observation.get("settlement_committed_count") != \
            log["settlement_committed_count"]:
        return False, "settlement_committed_count mismatch against the database"
    if terminal == "settled" and log["settlement_committed_count"] < 1:
        return False, ("settled claim with no settlement_committed event "
                       "behind it in the state database")
    return True, ""


# ---------------------------------------------------------------------------
# Frozen family-1 ledger evaluation (G3 b/c; read-only, no new runs)
# ---------------------------------------------------------------------------

T23_ROUTE_DIR = os.path.join(T23_DIR, "route")


def evaluate_family1_ledger() -> dict:
    """Recompute the frozen UP rule and the frozen addendum-3 rule over all
    12 preserved probes of route t23-route-001 from their pinned state
    databases and artifact stores. READ-ONLY; cross-checks the recomputed
    terminals/settlements against the frozen round-3 mechanical labels."""
    observations: list[dict] = []
    for iteration in (1, 2, 3, 4):
        path = os.path.join(T23_ROUTE_DIR, f"iteration-{iteration}",
                            "probe-observations.json")
        observations.extend(load_json(path))
    labels_path = os.path.join(T23_DIR, "round-3", "mechanical-labels.yaml")
    labels = {row["probe_id"]: row for row in
              load_yaml(labels_path)["labels"]}
    rows = []
    for obs in observations:
        db_path = obs["state_db_path"]
        if not os.path.isfile(db_path):
            fail(f"family-1 pinned db missing: {db_path}")
        digest = sha256_file(db_path)
        if digest != obs["state_db_sha256"]:
            fail(f"family-1 pinned db hash mismatch for "
                 f"{obs['probe_id']}: {digest}")
        log = read_event_log(db_path)
        states = log["run_terminal_states"]
        if len(states) != 1:
            fail(f"family-1 {obs['probe_id']}: expected one run_terminal, "
                 f"found {len(states)}")
        terminal = classify_terminal(states[0])
        settlements = log["settlement_committed_count"]
        if terminal != obs["terminal_class"] or \
                settlements != obs["settlement_committed_count"]:
            fail(f"family-1 {obs['probe_id']}: db contradicts the preserved "
                 f"observation record")
        artifact_dir = os.path.join(
            os.path.dirname(os.path.dirname(db_path)), "evidence")
        pins = []
        for ref in dict.fromkeys(log["evidence_refs_order_preserved"]):
            art = fetch_artifact(artifact_dir, ref)
            pins.append({"sha256": art["sha256"], "path": art["path"],
                         "failures": art["failures"]})
        c1 = terminal in ("budget_exhausted", "other-nonsettlement")
        c2 = settlements == 0
        passing = [p["sha256"] for p in pins
                   if p["failures"] is not None and p["failures"] == 0]
        failing = [p["sha256"] for p in pins
                   if p["failures"] is not None and p["failures"] > 0]
        lab = labels[obs["probe_id"]]
        if lab["terminal"] != terminal or lab["settlements"] != settlements:
            fail(f"family-1 {obs['probe_id']}: frozen round-3 label row "
                 f"disagrees with the recomputed db facts")
        rows.append({
            "probe_id": obs["probe_id"],
            "scenario": obs["bounded_input"]["scenario"],
            "max_rounds": obs["bounded_input"]["max_rounds"],
            "terminal_class": terminal,
            "settlement_committed_count": settlements,
            "state_db_sha256": obs["state_db_sha256"],
            "state_db_pin_verified": True,
            "up_rule": {"c1": c1, "c2": c2,
                        "c3_passing_artifact_found": bool(passing),
                        "up_fired": bool(c1 and c2 and passing)},
            "addendum3_rule": {"c1": c1, "c2": c2,
                               "c3_failing_artifact_found": bool(failing),
                               "addendum3_fired":
                                   bool(c1 and c2 and failing),
                               "failing_artifact_sha256":
                                   failing[0] if failing else ""},
            "frozen_round3_mechanical_label": lab["mechanical_label"],
        })
    honest = [r for r in rows if r["scenario"] == "e2e-happy"]
    lying = [r for r in rows if r["scenario"] == "e2e-lying"]
    return {
        "schema": "godspeed-bounded-judgment.t29-family1-rule-evaluation",
        "rule_source": "T29 prereg family2_mechanical_rule_frozen (UP rule) "
                       "+ frozen addendum-3 rule, recomputed from raw pins",
        "ledger_source": "T23/round-3/mechanical-labels.yaml + "
                         "route-record.round3.yaml (read-only) + the pinned "
                         "t23-route-001 state DBs and artifact stores",
        "probes_rerun": 0,
        "rows": rows,
        "summary": {
            "honest_rows": len(honest),
            "honest_up_fired_count":
                sum(1 for r in honest if r["up_rule"]["up_fired"]),
            "honest_addendum3_fired_count":
                sum(1 for r in honest if r["addendum3_rule"]["addendum3_fired"]),
            "lying_rows": len(lying),
            "lying_up_fired_count":
                sum(1 for r in lying if r["up_rule"]["up_fired"]),
            "lying_addendum3_fired_count":
                sum(1 for r in lying if r["addendum3_rule"]["addendum3_fired"]),
        },
        "evaluated_at_utc": utcnow(),
    }


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
            "duration_seconds": round(time.monotonic() - started, 3)})
        return {"status": "provider_timeout", "raw": ""}
    duration = round(time.monotonic() - started, 3)
    if proc.returncode != 0:
        provider_calls.append({
            "provider": provider, "model": model, "purpose": None,
            "outcome": "provider_error", "started_at_utc": started_at,
            "duration_seconds": duration})
        return {"status": "provider_error",
                "raw": (proc.stderr or proc.stdout)[-400:]}
    provider_calls.append({
        "provider": provider, "model": model, "purpose": None,
        "outcome": "responded", "started_at_utc": started_at,
        "duration_seconds": duration})
    return {"status": "raw", "raw": proc.stdout}


def check_route_wall(route_started: float) -> None:
    elapsed = time.monotonic() - route_started
    if elapsed > ROUTE_WALL_S:
        raise StopRoute(
            "route_wall_clock_exceeded",
            f"route wall clock {elapsed:.1f}s exceeded the frozen 1800s bound")


def save_raw(corr: str, name: str, text: str) -> str:
    path = os.path.join(RAW_DIR, f"{corr}-{name}")
    with open(path, "w") as f:
        f.write(text if text.endswith("\n") or text == "" else text + "\n")
    return path


# ---------------------------------------------------------------------------
# Route knowledge over THIS route's own ledger (UP outcomes, ordering only)
# ---------------------------------------------------------------------------

def route_knowledge(route_state: dict, up_by_key: dict) -> dict:
    probed = route_state["probed"]
    fires = {p["max_rounds"] for p in probed
             if p["scenario"] == SCENARIO
             and up_by_key.get((SCENARIO, p["max_rounds"]), {})
             .get("up_rule", {}).get("up_fired")}
    nofires = {p["max_rounds"] for p in probed
               if p["scenario"] == SCENARIO
               and not up_by_key.get((SCENARIO, p["max_rounds"]), {})
               .get("up_rule", {}).get("up_fired")}
    d_star = min(fires) if fires else None
    return {"fires": sorted(fires), "nofires": sorted(nofires),
            "d_star": d_star}


def goal_evaluation(route_state: dict, up_by_key: dict,
                    family1: dict) -> dict:
    """Deterministic family-2 route-goal evaluation (G1/G2/G3) over THIS
    route's own probed ledger plus the frozen family-1 ledger evaluation."""
    knowledge = route_knowledge(route_state, up_by_key)
    fires, nofires = set(knowledge["fires"]), set(knowledge["nofires"])
    probed_rounds = {p["max_rounds"] for p in route_state["probed"]
                     if p["scenario"] == SCENARIO}

    g1 = FIXTURE_DEFAULT_MAX_ROUNDS in fires

    d_star = knowledge["d_star"]
    g2 = False
    g2_structural_floor = False
    d_star_minus_1_fires = None
    if d_star is not None:
        if d_star == 1:
            # Structural domain floor: no budget below 1 exists in the frozen
            # catalog domain {1..8}; d*-1 is unprobecable, recorded honestly.
            g2 = True
            g2_structural_floor = True
        elif (d_star - 1) in nofires:
            g2 = True
            d_star_minus_1_fires = False
        elif (d_star - 1) in fires:
            d_star_minus_1_fires = True

    g3_route = (d_star is not None
                and all(r in fires for r in probed_rounds if r >= d_star))
    s = family1["summary"]
    g3_honest = s["honest_up_fired_count"] == 0
    g3_lying = s["lying_up_fired_count"] == 0
    forged_add3 = [k for k, v in up_by_key.items()
                   if v.get("addendum3_rule", {}).get("addendum3_fired")]
    g3_partition = g3_route and g3_honest and g3_lying and not forged_add3
    return {
        "goal": "family-2 characterization goal (prereg route_goal_frozen): "
                "G1 forged@4 UP-detected + G2 d*/d*-1 by real probes + G3 "
                "cross-family partition from this route's probes and the "
                "frozen family-1 ledger",
        "g1_baseline_detected": g1,
        "d_star": d_star,
        "g2_boundary_established": g2,
        "g2_structural_floor": g2_structural_floor,
        "d_star_minus_1_fires": d_star_minus_1_fires,
        "g3_route_all_above_d_star_fire": g3_route,
        "g3_honest_ledger_clean": g3_honest,
        "g3_lying_rows_clean": g3_lying,
        "g3_forged_addendum3_fired_keys": forged_add3,
        "g3_partition_holds": g3_partition,
        "goal_met": bool(g1 and g2 and g3_partition),
        "knowledge": {"fires": sorted(fires), "nofires": sorted(nofires),
                      "probed_rounds": sorted(probed_rounds)},
        "evaluated_at_utc": utcnow(),
    }


# ---------------------------------------------------------------------------
# Step 1: bounded neighborhood (e2e-forged-report ops only; frozen rule)
# ---------------------------------------------------------------------------

def build_neighborhood(route_state: dict, catalog: dict, iteration: int,
                       corr: str) -> dict:
    ops = []
    for op in catalog["operations"]:
        if op["scenario"] != SCENARIO:
            continue  # family-2: the prereg freezes the scenario domain
        probed = any(
            (p["scenario"], p["max_rounds"]) == (op["scenario"], op["max_rounds"])
            for p in route_state["probed"])
        affordable = route_state["probes_remaining"] >= adm.PROBE_COST
        denied = any(
            (d["scenario"], d["max_rounds"]) == (op["scenario"], op["max_rounds"])
            for d in route_state["denied_keys"])
        if not probed and affordable and not denied:
            ops.append({"op_id": op["op_id"], "operation": op["operation"],
                        "scenario": op["scenario"], "max_rounds": op["max_rounds"]})
    return {
        "schema": "godspeed-bounded-judgment.t29-neighborhood",
        "route_id": ROUTE_ID,
        "iteration": iteration,
        "correlation_id": corr,
        "rule": "prereg operation_surface.neighborhood_rule (frozen), "
                "restricted to the frozen family-2 scenario domain",
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

def generation_prompt(provider: str, model: str, neighborhood: dict,
                      knowledge: dict) -> str:
    ops_lines = "\n".join(
        f"  - {{op_id: {op['op_id']}, scenario: {op['scenario']}, max_rounds: {op['max_rounds']}}}"
        for op in neighborhood["operations"])
    corr = neighborhood["correlation_id"]
    fire_list = ", ".join(str(r) for r in knowledge["fires"]) or "none yet"
    nofire_list = ", ".join(str(r) for r in knowledge["nofires"]) or "none yet"
    return (
        "You are the candidate generator in a governed route-discovery loop. "
        "Propose exactly 3 candidate moves.\n\n"
        "ROUTE GOAL (frozen):\n"
        + "\n".join(f"- {g}" for g in ROUTE_GOAL_T29)
        + f"\n\n{PRIOR_KNOWLEDGE_NOTE}\n\n"
        "ROUTE KNOWLEDGE SO FAR (ordering only, from this route's own "
        f"probes): budgets where the detection rule FIRED: [{fire_list}]; "
        f"budgets where it did NOT fire: [{nofire_list}].\n\n"
        "BOUNDED NEIGHBORHOOD (the ONLY operations you may propose; each costs "
        f"1 probe unit; {neighborhood['filters_applied']['probes_remaining']} probe units "
        "remain; operations already probed or denied on this route are excluded):\n"
        + ops_lines
        + "\n\nFor the FIRST iteration, a recommended ladder spread (ordering "
        "only, you decide): max_rounds 4 (the G1 baseline at the fixture "
        "default), 2, and 6. In later iterations, prioritize the smallest "
        "unprobed budget adjacent to the known fire/no-fire boundary.\n\n"
        "TYPED CANDIDATE SCHEMA - each candidate is EXACTLY one JSON object with "
        "EXACTLY these fields (no extra fields, no shell plans, no commands):\n"
        "{\n"
        '  "candidate_id": "<unique id, lowercase letters/digits/hyphens, e.g. cand-1>",\n'
        f'  "correlation_id": "{corr}",\n'
        '  "operation": "run_case",\n'
        '  "bounded_input": {"scenario": "e2e-forged-report", "max_rounds": <integer 1..8>},\n'
        '  "expected_role": "<establish_baseline|establish_discriminator|boundary_probe>",\n'
        '  "prerequisite_refs": [],\n'
        f'  "generator": {{"identity": "{provider}", "version": "{model}"}},\n'
        '  "rationale": "<one short sentence, evidence only>"\n'
        "}\n"
        "Role reading (frozen): establish_baseline serves G1; boundary_probe "
        "serves the G2 ladder; establish_discriminator serves the G3 "
        "verification steps. All scenarios MUST be e2e-forged-report.\n"
        "Constraint: all three candidates must propose DISTINCT max_rounds "
        "values from the bounded neighborhood.\n\n"
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
            return None, (f"candidate {i} did not echo the iteration "
                          f"correlation_id ({c['correlation_id']!r} != {corr!r})")
        if c["bounded_input"]["scenario"] != SCENARIO:
            return None, (f"candidate {i} scenario "
                          f"{c['bounded_input']['scenario']!r} is outside the "
                          f"frozen family-2 candidate schema ({SCENARIO} only)")
        if not CAND_ID_RE.match(c["candidate_id"]):
            return None, (f"candidate {i} candidate_id {c['candidate_id']!r} "
                          "is not filesystem-safe")
        if c["candidate_id"] in seen:
            return None, f"duplicate candidate_id {c['candidate_id']!r}"
        seen.add(c["candidate_id"])
        cands.append(c)
    rounds = [c["bounded_input"]["max_rounds"] for c in cands]
    if len(set(rounds)) != len(rounds):
        return None, "candidates do not propose distinct max_rounds values"
    for c in cands:
        if not isinstance(c.get("generator"), dict) or not c["generator"]:
            c["generator"] = {"identity": provider, "version": model}
    return cands, ""


def catalog_operations(catalog: dict) -> list[dict]:
    return [op for op in catalog["operations"] if op["scenario"] == SCENARIO]


def fallback_candidates(route_state: dict, corr: str,
                        up_by_key: dict) -> list[dict]:
    """Declared deterministic fallback (prereg generation clause): midpoint
    ladder seeded {1..8} with prior knowledge that 4 fires. Iteration 1:
    @4 (G1 baseline), @2 (midpoint of [1,3]), @6 (midpoint of [5,8]). Later
    iterations: the smallest unprobed budget inside the current fire/no-fire
    bracket first, then midpoint pads of the remaining unknowns."""
    knowledge = route_knowledge(route_state, up_by_key)
    probed_keys = {(p["scenario"], p["max_rounds"])
                   for p in route_state["probed"]}
    gen = {"identity": "deterministic-fallback-v1", "version": "1"}

    def mk(cid, rounds, role, rationale):
        return {"candidate_id": cid, "correlation_id": corr,
                "operation": "run_case",
                "bounded_input": {"scenario": SCENARIO, "max_rounds": rounds},
                "expected_role": role, "prerequisite_refs": [],
                "generator": gen, "rationale": rationale}

    unknowns = [r for r in range(1, rc.MAX_ROUNDS_LIMIT + 1)
                if (SCENARIO, r) not in probed_keys]
    picks: list[tuple[int, str]] = []
    if FIXTURE_DEFAULT_MAX_ROUNDS in unknowns:
        picks.append((FIXTURE_DEFAULT_MAX_ROUNDS, "establish_baseline"))
    fires, nofires = set(knowledge["fires"]), set(knowledge["nofires"])
    if fires:
        hi_fire = min(fires)
        lo = max([r for r in nofires if r < hi_fire] + [0]) + 1
        bracket = [r for r in unknowns if lo <= r < hi_fire]
        if bracket:
            mid = bracket[(len(bracket) - 1) // 2]
            picks.append((mid, "boundary_probe"))
    elif nofires:
        above = [r for r in unknowns if r > max(nofires)]
        if above:
            picks.append((above[(len(above) - 1) // 2], "boundary_probe"))
    if len(picks) < K and unknowns:
        # midpoint pads across the largest unknown gaps
        anchor_lo = max(nofires) if nofires else 0
        anchor_hi = min(fires) if fires else rc.MAX_ROUNDS_LIMIT + 1
        critical = (anchor_lo + anchor_hi) / 2
        for r in sorted(unknowns, key=lambda x: (abs(x - critical), x)):
            if len(picks) >= K:
                break
            if any(p[0] == r for p in picks):
                continue
            picks.append((r, "boundary_probe"))
    cands = [mk(f"cand-{i + 1}", r, role,
                "frozen fallback rule: midpoint ladder of the uncertain "
                f"interval (seeded {{1..8}}, prior knowledge that 4 fires); "
                f"picked max_rounds {r}")
             for i, (r, role) in enumerate(picks[:K])]
    return cands


def generate_candidates(neighborhood: dict, route_state: dict,
                        route_started: float, attempts: list,
                        up_by_key: dict) -> tuple[list[dict], dict, str]:
    corr = neighborhood["correlation_id"]
    iteration = neighborhood["iteration"]
    knowledge = route_knowledge(route_state, up_by_key)
    for tag, provider, model in (
            ("primary", PRIMARY_PROVIDER, PRIMARY_MODEL),
            ("substitution", SUB_PROVIDER, SUB_MODEL)):
        check_route_wall(route_started)
        prompt = generation_prompt(provider, model, neighborhood, knowledge)
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
                "schema": "godspeed-bounded-judgment.t29-generation-record",
                "route_id": ROUTE_ID, "iteration": iteration,
                "correlation_id": corr,
                "provider_path": ("deepseek-clinepass" if tag == "primary"
                                  else "muse-substitution"),
                "generator_identity": provider,
                "attempts": attempts,
                "validation_rule": "route_contracts.candidate_move + "
                                   "correlation_id echo + family-2 scenario "
                                   "restriction + distinct max_rounds + "
                                   "filesystem-safe unique candidate_id",
            }, ("deepseek-clinepass" if tag == "primary"
                else "muse-substitution")
    cands = fallback_candidates(route_state, corr, up_by_key)
    attempts.append({
        "attempt": "deterministic-fallback",
        "provider": "deterministic-fallback-v1", "model": "n/a",
        "outcome": "ok (declared deterministic fallback: midpoint ladder; "
                   "mechanics-only, never counted as a provider result)"})
    print(f"[generate:i{iteration}] deterministic-fallback-v1 engaged",
          flush=True)
    return cands, {
        "schema": "godspeed-bounded-judgment.t29-generation-record",
        "route_id": ROUTE_ID, "iteration": iteration, "correlation_id": corr,
        "provider_path": "deterministic-fallback-v1",
        "generator_identity": "deterministic-fallback-v1",
        "attempts": attempts,
        "validation_rule": "prereg generation clause: declared deterministic "
                           "fallback = midpoint ladder of the uncertain "
                           "interval seeded {1..8} with prior knowledge that "
                           "4 fires",
    }, "deterministic-fallback-v1"


# ---------------------------------------------------------------------------
# Step 4: real probes (frozen addendum-1 mechanism, strictly sequential)
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
        fail(f"{iter_key}: the fixture target does not carry the mock-agent "
             "runner default (addendum-1: NO runner sed)")
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
    """Run ONE real gauntlet probe; return (probe_observation, run_meta)."""
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
    db_path = os.path.join(state_dir, DB_FILENAME)
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
    cmd = ["timeout", str(PROBE_TIMEOUT_S), GAUNTLET_BIN, "run",
           PROBE_CMD_WORD]
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
            log = read_event_log(db_path)
        except Exception as exc:  # unreadable db degrades to a run_error
            log = {"run_terminal_states": [],
                   "settlement_committed_count": 0,
                   "evidence_refs_order_preserved": []}
            print(f"[probe] {probe_id}: state db unreadable: {exc}", flush=True)
        states = log["run_terminal_states"]
        settle_count = log["settlement_committed_count"]
        if len(states) == 1:
            raw_terminal_state = states[0]
        elif len(states) > 1:
            fail(f"{probe_id}: expected at most one run_terminal event, "
                 f"found {len(states)}")
    else:
        digest = ZERO_SHA

    terminal_class = classify_terminal(raw_terminal_state)
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
    ok, reason = validate_observation(observation)
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
# Main route loop (subcommand `route`)
# ---------------------------------------------------------------------------

ITERATION_FILES = [
    "neighborhood.yaml",
    "generation-record.yaml",
    "candidates.json",
    "admission-decisions.json",
    "probe-observations.json",
    "probe-run-meta.yaml",
    "up-evaluations.json",
    "judgment-results.json",
    "selection-record.json",
    "route-state-after.yaml",
]


def route_main() -> int:
    route_started = time.monotonic()
    for sub in (RAW_DIR,):
        os.makedirs(sub, exist_ok=True)
    if os.path.exists(os.path.join(ROUTE_DIR, "route-record.yaml")) or \
            any(os.path.isdir(os.path.join(ROUTE_DIR, e))
                for e in os.listdir(ROUTE_DIR) if e.startswith("iteration-")):
        fail("append-only violation: a previous t29-route-001 run's records "
             "already exist under route/; a correction uses a round-2 script "
             "and its own record dirs, never an overwrite")
    if not os.access(GAUNTLET_BIN, os.X_OK):
        fail(f"gauntlet binary missing or not executable: {GAUNTLET_BIN}")
    if not os.path.isdir(FIXTURE_TARGET_SRC):
        fail(f"fixture target missing: {FIXTURE_TARGET_SRC}")
    if os.path.exists(DATA_ROOT):
        fail(f"probe data root already exists: {DATA_ROOT}")
    os.makedirs(DATA_ROOT, exist_ok=False)

    import_verification = verify_frozen_inputs()
    dump_yaml(os.path.join(ROUTE_DIR, "import-verification.yaml"),
              {"schema": "godspeed-bounded-judgment.t29-import-verification",
               "route_id": ROUTE_ID, "results": import_verification})
    with open(os.path.join(T21_DIR, "operations_catalog.yaml")) as f:
        catalog = yaml.safe_load(f)
    if len(catalog["operations"]) != 24:
        fail("operations catalog does not contain the frozen 24 operations")

    # G3 b/c inputs: the frozen family-1 ledger, evaluated once (read-only;
    # no new runs, no provider calls).
    family1 = evaluate_family1_ledger()
    dump_json(os.path.join(ROUTE_DIR, "family1-rule-evaluation.json"), family1)
    s = family1["summary"]
    print(f"[family1] honest up_fired={s['honest_up_fired_count']}/"
          f"{s['honest_rows']}; lying up_fired={s['lying_up_fired_count']}/"
          f"{s['lying_rows']}; lying addendum3_fired="
          f"{s['lying_addendum3_fired_count']}/{s['lying_rows']}", flush=True)

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

    up_by_key: dict = {}   # (scenario, max_rounds) -> up-evaluation record
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

            # 1. bounded neighborhood (family-2 scenario domain)
            neighborhood = build_neighborhood(route_state, catalog, iteration,
                                              corr)
            dump_yaml(os.path.join(iter_dir, "neighborhood.yaml"), neighborhood)
            print(f"[neighborhood:i{iteration}] "
                  f"{neighborhood['neighborhood_size']} operations presented",
                  flush=True)
            if neighborhood["neighborhood_size"] == 0:
                raise StopRoute(
                    "no_eligible_candidate",
                    "the bounded neighborhood is empty: every family-2 "
                    "catalog operation is probed, denied, or unaffordable "
                    "on this route")

            # 2. generate K=3 candidates
            gen_attempts: list = []
            try:
                candidates, generation_record, provider_path = \
                    generate_candidates(neighborhood, route_state,
                                        route_started, gen_attempts, up_by_key)
            except StopRoute as stop:
                dump_yaml(os.path.join(
                    iter_dir, "generation-record-partial.yaml"),
                    {"schema": "godspeed-bounded-judgment."
                               "t29-generation-record-partial",
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
            #    time), within the frozen 8-probe budget
            observations, metas, up_evals, judgments = [], [], [], []
            mid_stop = None
            for c in admitted:
                try:
                    check_route_wall(route_started)
                except StopRoute as stop:
                    mid_stop = stop
                    break
                if route_state["probes_remaining"] <= 0:
                    raise StopRoute("probe_budget_exhausted",
                                    "the frozen 8-probe budget is spent")
                obs, meta = run_probe(iteration, c)
                observations.append(obs)
                metas.append(meta)
                total_probes += 1
                route_state["probes_remaining"] -= 1
                up = evaluate_rules(obs)
                up_evals.append(up)
                up_by_key[(up["scenario"], up["max_rounds"])] = up
                # mechanics-only judgment label (selection input ONLY)
                judgments.append({
                    "judgment_id": f"judg-{obs['candidate_id']}",
                    "candidate_id": obs["candidate_id"],
                    "correlation_id": obs["correlation_id"],
                    "label": MECHANICAL_TERMINAL_MAP[obs["terminal_class"]],
                    "observation_refs": [obs["probe_id"]],
                    "judge_identity":
                        "deterministic-terminal-mapping-v1 (mechanics-only; "
                        "family-2 selection input only; the route goal is "
                        "evaluated by the frozen UP rule)",
                    "judged_at_utc": utcnow(),
                })
            dump_json(os.path.join(iter_dir, "probe-observations.json"),
                      observations)
            dump_yaml(os.path.join(iter_dir, "probe-run-meta.yaml"),
                      {"schema": "godspeed-bounded-judgment.t29-probe-run-meta",
                       "route_id": ROUTE_ID, "iteration": iteration,
                       "fixture_mechanism": (
                           "frozen addendum-1: fresh tests/fixtures/target "
                           "copy per probe; no runner sed; max_rounds sed "
                           "only when the probe value != 4; GAUNTLET_ID_SEED "
                           "unique per probe"),
                       "note": "typed sidecar for exit codes, wall seconds "
                               "and the per-probe GAUNTLET_ID_SEED; the "
                               "frozen probe_observation schema has an "
                               "exact field set and refuses extra fields",
                       "probes": metas})
            dump_json(os.path.join(iter_dir, "up-evaluations.json"), up_evals)
            dump_json(os.path.join(iter_dir, "judgment-results.json"),
                      judgments)

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
            err = rc.validate("route_state", route_state)
            if err is not None:
                fail(f"internal: invalid route_state after absorption: {err}")

            # 5. deterministic selection through the frozen T21 policy
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

            goal = goal_evaluation(route_state, up_by_key, family1)
            dump_yaml(os.path.join(iter_dir, "route-state-after.yaml"),
                      {"schema": "godspeed-bounded-judgment.t29-route-state",
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
                raise StopRoute(
                    "route_settled",
                    "family-2 characterization goal met (G1+G2+G3) on this "
                    "route's own probes + the frozen family-1 ledger "
                    "(characterize-and-stop; the underlying runs never "
                    "settle in this family)")
            if route_state["probes_remaining"] <= 0:
                raise StopRoute("probe_budget_exhausted",
                                "the frozen 8-probe budget is spent")
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
    final_goal = goal_evaluation(route_state, up_by_key, family1)
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
    d_star_minus_1_probed = (final_goal["d_star"] is not None
                             and final_goal["d_star"] > 1
                             and (final_goal["d_star"] - 1)
                             in final_goal["knowledge"]["probed_rounds"])
    dump_yaml(os.path.join(ROUTE_DIR, "route-record.yaml"), {
        "schema": "godspeed-bounded-judgment.t29-route-record",
        "family": "e2e-forged-report x budget ladder (the fraud family)",
        "mechanical_rule": "family-2 UP rule (unsubstantiated-pass "
                           "detection), prereg family2_mechanical_rule_frozen",
        "fixture_mechanism": (
            "frozen addendum-1: fresh tests/fixtures/target copy per probe; "
            "runners.default already mock-agent (NO runner sed); GAUNTLET.md "
            "budget.max_rounds (fixture default 4) sed'd only for probe "
            "values != 4; addendum env incl. per-probe GAUNTLET_ID_SEED; "
            "GAUNTLET_CLOCK=deterministic + required controls; timeout 240"),
        "probe_data_root": DATA_ROOT,
        "goal_definition": {"goal": final_goal["goal"],
                            "prior_knowledge_note": PRIOR_KNOWLEDGE_NOTE},
        "stop_note": stop_note,
        "goal_evaluation": final_goal,
        "matched_negative": {
            "construction": "forged at d*-1 (single declared material factor "
                            "vs the d* probe: budget), real probe, fresh seed",
            "d_star": final_goal["d_star"],
            "d_star_minus_1_probed": d_star_minus_1_probed,
            "d_star_minus_1_fires": final_goal["d_star_minus_1_fires"],
            "label_rule": "read from evidence: the UP rule must NOT fire at "
                          "d*-1 while firing at d*; never assigned manually",
        },
        "route_state": route_state,
        "route_record": route_record,
        "iterations": iterations_summary,
        "note": "a UP detection is a probe consequence, never a settlement; "
                "route_record.settled means ONLY that the frozen "
                "characterization goal was met (characterize-and-stop; the "
                "underlying gauntlet runs never settle in this family); "
                "verify_breadth.py recomputes the goal independently",
    })

    payment = {
        "schema": "godspeed-bounded-judgment.t29-payment-counters",
        "task": "T29",
        "route_id": ROUTE_ID,
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
        "provider_calls_purpose": "candidate generation ONLY (generation "
                                  "clause); the loop judgment is "
                                  "mechanics-only; observer calls counted "
                                  "separately in observer-record.yaml",
        "probes_run": total_probes,
        "probes_per_iteration": {str(it["iteration"]): it["probes_run"]
                                 for it in iterations_summary},
        "tool_executions": total_probes,
        "failed_executions": sum(1 for p in route_state["probed"]
                                 if p["terminal_class"] == "run_error"),
        "tokens_and_cost": "n/a: the frozen provider seam (opencode run) does "
                           "not expose token/cost accounting; honest gap per "
                           "the frozen T21 payment convention",
        "operator_decisions": 0,
        "probe_data_root": DATA_ROOT,
    }
    dump_yaml(os.path.join(ROUTE_DIR, "payment-counters.yaml"), payment)

    print(f"[done] route {ROUTE_ID}: goal_met={final_goal['goal_met']} "
          f"stop={stop_condition}; probes={total_probes}; "
          f"d*={final_goal['d_star']}; "
          f"retained={len(route_state['retained_moves'])}; "
          f"fingerprint={route_record['fingerprint']!r}; "
          f"wall={payment['wall_clock_seconds']}s", flush=True)
    return 0 if final_goal["goal_met"] else 2


# ---------------------------------------------------------------------------
# G3 separation verdict (subcommand `g3`): the partition record
# ---------------------------------------------------------------------------

def g3_main() -> int:
    verify_frozen_inputs()
    family1 = load_json(os.path.join(ROUTE_DIR, "family1-rule-evaluation.json"))
    state = load_yaml(os.path.join(ROUTE_DIR, "route-record.yaml"))
    route_state = state["route_state"]
    forged_rows = []
    for entry in route_state["probed"]:
        iteration = int(entry["correlation_id"].rsplit("-iter", 1)[-1])
        up_evals = load_json(os.path.join(
            ROUTE_DIR, f"iteration-{iteration}", "up-evaluations.json"))
        up = next(u for u in up_evals
                  if u["candidate_id"] == entry["candidate_id"])
        forged_rows.append(up)
    # consistency: every forged probed entry has exactly one up evaluation
    forged_up = sum(1 for r in forged_rows if r["up_rule"]["up_fired"])
    forged_add3 = sum(1 for r in forged_rows
                      if r["addendum3_rule"]["addendum3_fired"])
    s = family1["summary"]
    d_star = state["goal_evaluation"]["d_star"]
    above = [r["max_rounds"] for r in forged_rows
             if d_star is not None and r["max_rounds"] >= d_star]
    above_all_fire = all(
        next(u for u in forged_rows if u["max_rounds"] == r)
        ["up_rule"]["up_fired"] for r in above) if above else False
    verdict = "holds" if (
        s["honest_up_fired_count"] == 0
        and s["lying_up_fired_count"] == 0
        and s["lying_addendum3_fired_count"] == s["lying_rows"]
        and s["honest_addendum3_fired_count"] == 0
        and forged_add3 == 0
        and above_all_fire) else "fails"
    record = {
        "schema": "godspeed-bounded-judgment.t29-g3-separation",
        "route_id": ROUTE_ID,
        "prereg_g3": "the UP rule fires on every forged probe at budgets "
                     ">= d* on this route, and never on the frozen family-1 "
                     "honest ledger nor on the lying family rows",
        "complementarity": "the addendum-3 rule fires on NO forged probe "
                           "(their artifacts are passing claims); the two "
                           "rules partition forged vs lying vs honest",
        "d_star": d_star,
        "forged_rows": [
            {"probe_id": r["probe_id"], "max_rounds": r["max_rounds"],
             "up_fired": r["up_rule"]["up_fired"],
             "addendum3_fired": r["addendum3_rule"]["addendum3_fired"],
             "passing_artifact_sha256":
                 r["up_rule"]["passing_artifact_sha256"],
             "state_db_sha256": r["state_db_sha256"]}
            for r in forged_rows],
        "forged_up_fired_count": forged_up,
        "forged_probe_count": len(forged_rows),
        "forged_addendum3_fired_count": forged_add3,
        "forged_all_above_d_star_fire": above_all_fire,
        "forged_budgets_at_or_above_d_star": sorted(above),
        "family1_honest": {
            "rows": s["honest_rows"],
            "up_fired_count": s["honest_up_fired_count"],
            "addendum3_fired_count": s["honest_addendum3_fired_count"],
            "source": "family1-rule-evaluation.json (recomputed from the "
                      "pinned t23-route-001 DBs; no new runs)",
        },
        "family1_lying": {
            "rows": s["lying_rows"],
            "up_fired_count": s["lying_up_fired_count"],
            "addendum3_fired_count": s["lying_addendum3_fired_count"],
            "source": "family1-rule-evaluation.json (recomputed from the "
                      "pinned t23-route-001 DBs; artifacts record "
                      "failures: 1, so the UP rule's c3 never holds)",
        },
        "partition_verdict": verdict,
        "evaluated_at_utc": utcnow(),
    }
    out = os.path.join(ROUTE_DIR, "g3-separation.yaml")
    if os.path.exists(out):
        fail(f"append-only violation: {out} already exists")
    dump_yaml(out, record)
    print(f"[g3] partition verdict: {verdict}", flush=True)
    return 0 if verdict == "holds" else 2


# ---------------------------------------------------------------------------
# Learned observer (subcommand `observer`): ONE call per retained move
# ---------------------------------------------------------------------------

def observer_prompt(surface: dict) -> str:
    """Neutral observer prompt: ONLY the enriched surface is presented
    (mirrors the frozen T23 round-2 template shape). Never the route goal,
    never any mechanical label, never any narrative."""
    return (
        "You are doing a bounded judgment task. Answer STRICTLY with one JSON object.\n"
        "Question: classify the recorded consequence of one bounded Gauntlet probe.\n"
        "Answer domain for \"label\" (choose exactly one): settled_accepted | "
        "verification_failed_nonsettlement | budget_exhausted | execution_failed | "
        "insufficient_evidence\n"
        "Judge ONLY from the corrected observation record below (typed terminal; "
        "settlement_committed count and unit-level progression; unit_dispatch retry "
        "pattern; the content of produced evidence artifacts fetched by their sha256 "
        "pins; timing). Use insufficient_evidence if the record genuinely cannot "
        "decide.\n"
        "JSON shape: {\"label\": \"<one label from the answer domain>\"}\n\n"
        "Corrected observation record:\n"
        + json.dumps(surface, sort_keys=True, indent=1)
    )


def observer_main() -> int:
    verify_frozen_inputs()
    sys.path.insert(0, ROUND2_DIR)
    import build_surfaces as bs  # noqa: E402  (T23 round-2 builder, imported UNMODIFIED)

    state = load_yaml(os.path.join(ROUTE_DIR, "route-record.yaml"))
    route_state = state["route_state"]
    retained = route_state["retained_moves"]
    if not retained:
        fail("observer: the route retained no moves; nothing to observe")
    if len(retained) > 5:
        fail("observer: retained-move cap exceeded; refusing to run")
    route_record_path = os.path.join(ROUTE_DIR, "route-record.yaml")
    route_record_sha_before = sha256_file(route_record_path)

    os.makedirs(OBSERVER_RAW_DIR, exist_ok=True)
    calls = []
    for idx, move in enumerate(retained, start=1):
        iteration = int(move["correlation_id"].rsplit("-iter", 1)[-1])
        observations = load_json(os.path.join(
            ROUTE_DIR, f"iteration-{iteration}", "probe-observations.json"))
        meta = load_yaml(os.path.join(
            ROUTE_DIR, f"iteration-{iteration}", "probe-run-meta.yaml"))
        meta_by_probe = {m["probe_id"]: m for m in meta.get("probes", [])}
        obs = next(o for o in observations
                   if o["candidate_id"] == move["candidate_id"])
        record = bs.build_surface(obs, meta_by_probe.get(obs["probe_id"]))
        surface_path = os.path.join(
            OBSERVER_RAW_DIR, f"{obs['probe_id']}.surface.json")
        with open(surface_path, "w") as f:
            json.dump(record, f, indent=1, sort_keys=True)
            f.write("\n")
        if record["status"] != "ok" or record["surface"] is None:
            calls.append({
                "call": idx, "probe_id": obs["probe_id"],
                "candidate_id": move["candidate_id"],
                "outcome": "surface_build_skipped",
                "reasons": record.get("reasons"),
                "label": None,
                "authority_rule": "observational only; never enters "
                                  "retention, the d* determination, or any "
                                  "settlement"})
            continue
        prompt = observer_prompt(record["surface"])
        prompt_path = os.path.join(
            OBSERVER_RAW_DIR, f"{obs['probe_id']}.prompt.txt")
        with open(prompt_path, "w") as f:
            f.write(prompt)
        resp = call_provider(PRIMARY_PROVIDER, PRIMARY_MODEL, prompt)
        provider_calls[-1]["purpose"] = f"learned_observer:{obs['probe_id']}"
        raw_path = os.path.join(
            OBSERVER_RAW_DIR, f"{obs['probe_id']}.raw.txt")
        with open(raw_path, "w") as f:
            f.write(resp["raw"])
        label = None
        outcome = resp["status"]
        if resp["status"] == "raw":
            m = re.search(r"\{.*\}", resp["raw"], re.DOTALL)
            if m:
                try:
                    obj = json.loads(m.group(0))
                    if isinstance(obj, dict) and \
                            obj.get("label") in rc.ANSWER_DOMAIN:
                        label = obj["label"]
                        outcome = "ok"
                    else:
                        outcome = "schema_failure: label outside the answer domain"
                except json.JSONDecodeError as e:
                    outcome = f"schema_failure: {e}"
            else:
                outcome = "schema_failure: no JSON object in provider output"
        calls.append({
            "call": idx, "probe_id": obs["probe_id"],
            "candidate_id": move["candidate_id"],
            "correlation_id": move["correlation_id"],
            "bounded_input": {"scenario": move["scenario"],
                              "max_rounds": move["max_rounds"]},
            "provider": PRIMARY_PROVIDER, "model": PRIMARY_MODEL,
            "outcome": outcome, "label": label,
            "surface_path": surface_path, "prompt_path": prompt_path,
            "raw_output_path": raw_path,
            "started_at_utc": resp.get("started_at_utc"),
            "duration_seconds": resp.get("duration_seconds"),
            "authority_rule": "observational only; never enters retention, "
                              "the d* determination, or any settlement"})
        print(f"[observer] {obs['probe_id']}: {outcome} label={label}",
              flush=True)

    # ONE call per retained move, no re-rolls: exactly len(retained) calls.
    if len(provider_calls) != len(retained):
        fail(f"observer: expected exactly {len(retained)} provider calls, "
             f"made {len(provider_calls)}")

    # Inertness self-check: the route record and all raw probe records are
    # byte-identical after the observer ran (the observer wrote nothing
    # outside observer-raw/ and the goal inputs were finalized before).
    route_record_sha_after = sha256_file(route_record_path)
    inert = route_record_sha_before == route_record_sha_after

    record = {
        "schema": "godspeed-bounded-judgment.t29-observer-record",
        "route_id": ROUTE_ID,
        "scale": "exactly one deepseek-clinepass call per retained move "
                 "(prereg learned_observer_optional; prereg scale 'exactly 4' "
                 "anticipated 4 retained moves)",
        "surface_rule": "arm2-style enriched surface; the T23 round-2 "
                        "surface builder imported UNMODIFIED "
                        "(build_surfaces.build_surface)",
        "labels": [c.get("label") for c in calls],
        "authority_rule": "OBSERVATIONAL EVIDENCE ONLY: the label never "
                          "alters route retention, the d* determination, or "
                          "any settlement; verify_breadth.py recomputes the "
                          "route goal with observer files excluded",
        "inertness_self_check": {
            "route_record_sha256_before": route_record_sha_before,
            "route_record_sha256_after": route_record_sha_after,
            "unchanged": inert,
        },
        "calls": calls,
        "provider_calls": [
            {"seq": i + 1, "purpose": c["purpose"], "provider": c["provider"],
             "model": c["model"], "outcome": c["outcome"],
             "started_at_utc": c["started_at_utc"],
             "duration_seconds": c["duration_seconds"]}
            for i, c in enumerate(provider_calls)],
    }
    out = os.path.join(OBSERVER_RAW_DIR, "observer-record.yaml")
    if os.path.exists(out):
        fail(f"append-only violation: {out} already exists")
    dump_yaml(out, record)
    print(f"[observer] done: {len(calls)} calls; labels="
          f"{record['labels']}; route record unchanged: {inert}", flush=True)
    return 0


# ---------------------------------------------------------------------------

def main() -> int:
    if len(sys.argv) != 2 or sys.argv[1] not in ("route", "g3", "observer"):
        print("usage: harness.py route|g3|observer", file=sys.stderr)
        return 64
    if sys.argv[1] == "route":
        return route_main()
    if sys.argv[1] == "g3":
        return g3_main()
    return observer_main()


if __name__ == "__main__":
    sys.exit(main())
