#!/usr/bin/env python3
"""T29 gate: independent verification of the family-2 breadth route.

Run from the sea-rs repo root:  python3 .agents/evidence/godspeed-bounded-judgment/T29/verify_breadth.py

Optional argv[1]: path to an alternative T29 tree (the teeth run this gate
against tampered copies; the default is this script's own directory).

stdlib + PyYAML only. This gate is an INDEPENDENT recomputation: it does NOT
import the T29 harness. It re-hashes every pin and re-derives every
consequence from raw data (the pinned state databases and artifact stores),
fail-closed:

  1. frozen inputs: T29 prereg sha256, addendum-1 sha256, T21 frozen modules
     against T21/evidence-manifest.yml, family-1 round-3 manifest files
     against T23/round-3/evidence-manifest.round3.yml.
  2. the T29 evidence manifest, both directions (every listed file matches;
     every .py/.yaml/.json/.txt under the T29 tree except the manifest itself
     and the unpinned narrative files IN-FLIGHT.md / b1-progress.md /
     b1-result.json is listed).
  3. every T29 probe: db pin re-hash; exactly one run_terminal; typed
     terminal; settlement_committed count; every evidence_produced /
     settlement artifact re-fetched from the run's artifact store, re-hashed,
     its numeric `failures` parsed; the family-2 UP rule and the frozen
     addendum-3 rule RECOMPUTED and compared with the recorded
     up-evaluations.
  4. d* / d*-1 and G1/G2/G3 recomputed from raw records (route probes +
     the 12 family-1 pinned DBs) and compared with the recorded goal
     evaluation and the recorded partition verdict.
  5. the matched negative: d* vs d*-1 differ by exactly one declared factor
     (max_rounds), both real probed candidates with distinct pinned DBs and
     fresh seeds, and the negative label is the recomputed one (read, never
     assigned).
  6. observer inertness: exactly one observer call per retained move; the
     route goal recomputed with observer files EXCLUDED matches the recorded
     goal evaluation; the recorded before/after route-record digests match;
     no route record references observer data.
  7. bounds: <= 5 retained moves, <= 8 probes, <= 4 iterations, probe wall
     <= 240 s each, route wall <= 1800 s.
  8. payment counters recomputed: probes, failed executions, provider calls,
     observer calls.

Exit 0 only if every check passes; otherwise exit 1 with the findings.

CORRECTION HISTORY (append-only; failed rounds preserved, never silently
rewritten):
- round 1, sha256 3785afff9517377a802708da24075fe723fffda2ad84c4515786a1d513dd56be
  (verify_breadth.round1-failed.py): first executed by the T29 teeth, which
  FAILED LOUDLY in tooth T3 and caught a real gate defect: the repo-root
  ancestry computation applied one dirname too many (5 instead of 4), so the
  frozen prereg files were hashed at a nonexistent path (spurious "frozen
  input hash mismatch" findings on every gate run).
- round 2, sha256 b82e44ef6744682484ddd0be42f7ee1b584280a56f4023e5d6f1b283902e40b8
  (verify_breadth.round2-failed.py): correction attempt whose sed edit did not
  match the line-wrapped expression; only the docstring changed, the defect
  remained. The teeth failed loudly again in T3.
- round 3 (this file): the path computation actually fixed (4 dirnames from
  the T29 dir = the sea-rs repo root). No check logic changed in any round.
  Round-1/round-2 teeth artifacts are preserved under
  teeth/gate-round1-failed/ and teeth/gate-round2-failed/. Frozen before its
  first successful full execution; never overwritten.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import hashlib
import json
import os
import sqlite3

import yaml

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))


def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


class Gate:
    def __init__(self, tree: str):
        self.tree = tree
        self.findings: list[str] = []
        self.checks = 0

    def check(self, ok: bool, ok_msg: str, fail_msg: str) -> bool:
        self.checks += 1
        if not ok:
            self.findings.append(fail_msg)
        return ok

    def require(self, ok: bool, fail_msg: str) -> bool:
        return self.check(ok, "", fail_msg)

    def done(self) -> int:
        if self.findings:
            print(f"VERIFY-BREADTH: FAIL ({len(self.findings)} findings, "
                  f"{self.checks} checks)")
            for f in self.findings:
                print(f"  - {f}")
            return 1
        print(f"VERIFY-BREADTH: PASS ({self.checks} checks)")
        return 0


# --- read-only db access (independent reimplementation of the frozen
#     ground-truth semantics; same event schema) ---

def read_event_log(db_path: str) -> dict:
    conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    try:
        rows = conn.execute(
            "select kind, payload from event_log order by offset").fetchall()
    finally:
        conn.close()
    terminals, refs, settlements = [], [], 0
    for kind, payload in rows:
        try:
            body = json.loads(payload)
        except json.JSONDecodeError:
            body = {}
        if kind == "run_terminal":
            terminals.append(body.get("state"))
        elif kind == "settlement_committed":
            settlements += 1
            ref = body.get("evidence", "")
            if isinstance(ref, str) and ref.startswith("sha256:"):
                refs.append(ref[7:])
        elif kind == "evidence_produced":
            ref = body.get("evidence", "")
            if isinstance(ref, str) and ref.startswith("sha256:"):
                refs.append(ref[7:])
    return {"terminals": terminals, "settlements": settlements, "refs": refs}


def classify_terminal(raw) -> str:
    if raw == "settled":
        return "settled"
    if raw == "budget_exhausted":
        return "budget_exhausted"
    if isinstance(raw, str) and raw != "":
        return "other-nonsettlement"
    return "run_error"


def parse_failures(content: str):
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


def recompute_probe(gate: Gate, obs: dict) -> dict:
    """Independently recompute one probe's consequence facts and rule labels
    from its pinned db + artifact store. Returns the recomputed record, or a
    dict with 'broken' reasons on provenance failure."""
    reasons: list[str] = []
    db_path = obs.get("state_db_path", "")
    if not db_path or not os.path.isfile(db_path):
        gate.require(False,
                     f"{obs.get('probe_id')}: no state database at the "
                     f"pinned path: {db_path!r}")
        return {"broken": True, "reasons": ["missing db"],
                "terminal_class": "run_error", "settlements": 0, "pins": []}
    digest = sha256_file(db_path)
    gate.require(digest == obs.get("state_db_sha256"),
                 f"{obs.get('probe_id')}: db pin mismatch: pinned "
                 f"{obs.get('state_db_sha256')} != actual {digest}")
    try:
        log = read_event_log(db_path)
    except (OSError, sqlite3.Error) as exc:
        gate.require(False,
                     f"{obs.get('probe_id')}: db unreadable read-only: {exc}")
        return {"broken": True, "reasons": [f"unreadable db: {exc}"],
                "terminal_class": "run_error", "settlements": 0, "pins": []}
    terminals = log["terminals"]
    gate.require(len(terminals) == 1,
                 f"{obs.get('probe_id')}: expected exactly one run_terminal "
                 f"event, found {len(terminals)}")
    terminal = classify_terminal(terminals[0] if terminals else None)
    settlements = log["settlements"]
    gate.require(terminal == obs.get("terminal_class"),
                 f"{obs.get('probe_id')}: terminal mismatch db={terminal!r} "
                 f"recorded={obs.get('terminal_class')!r}")
    gate.require(settlements == obs.get("settlement_committed_count"),
                 f"{obs.get('probe_id')}: settlement count mismatch "
                 f"db={settlements} "
                 f"recorded={obs.get('settlement_committed_count')}")
    artifact_dir = os.path.join(
        os.path.dirname(os.path.dirname(db_path)), "evidence")
    pins = []
    for ref in dict.fromkeys(log["refs"]):
        path = os.path.join(artifact_dir, "objects", ref[:2], ref[2:4], ref)
        if not os.path.isfile(path):
            gate.require(False,
                         f"{obs.get('probe_id')}: artifact pin broken "
                         f"(missing object): {path}")
            pins.append({"sha256": ref, "failures": None})
            continue
        adigest = sha256_file(path)
        if adigest != ref:
            gate.require(False,
                         f"{obs.get('probe_id')}: artifact bytes at {path} "
                         f"hash to {adigest}, pinned {ref}")
        with open(path, encoding="utf-8", errors="replace") as f:
            pins.append({"sha256": ref, "failures": parse_failures(f.read())})
    c1 = terminal in ("budget_exhausted", "other-nonsettlement")
    c2 = settlements == 0
    passing = [p["sha256"] for p in pins
               if p["failures"] is not None and p["failures"] == 0]
    failing = [p["sha256"] for p in pins
               if p["failures"] is not None and p["failures"] > 0]
    return {"broken": False, "reasons": reasons, "terminal_class": terminal,
            "settlements": settlements, "pins": pins,
            "up_fired": bool(c1 and c2 and passing),
            "addendum3_fired": bool(c1 and c2 and failing),
            "passing": passing, "failing": failing}


def load_json(path: str):
    with open(path) as f:
        return json.load(f)


def load_yaml(path: str):
    with open(path) as f:
        return yaml.safe_load(f)


def main() -> int:
    tree = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else SCRIPT_DIR
    gate = Gate(tree)

    route_dir = os.path.join(tree, "route")

    # ---- 1. frozen inputs (resolved from the REAL ancestry, never the copy)
    real_t29 = SCRIPT_DIR
    real_branch = os.path.dirname(real_t29)
    # 4 dirnames from the T29 dir = the sea-rs repo root:
    # T29 -> godspeed-bounded-judgment -> evidence -> .agents -> sea-rs
    real_repo = os.path.dirname(os.path.dirname(
        os.path.dirname(os.path.dirname(real_t29))))
    t21 = os.path.join(real_branch, "T21")
    t23 = os.path.join(real_branch, "T23")
    preregs = os.path.join(real_repo, ".agents", "preregistrations")
    for name, expected in (
            ("godspeed-bounded-judgment-T29.prereg.yaml",
             "e844fce47c9860eeaacac0b8d27f0d28ef386da21f1c45ed878e85ba02af9672"),
            ("godspeed-bounded-judgment-T21.prereg.addendum-1.yaml",
             "c472d1b63248ff5895433df16c545c4e145c8171e9d475a8f4b0732fa1c0cea8")):
        path = os.path.join(preregs, name)
        gate.require(os.path.isfile(path) and sha256_file(path) == expected,
                     f"frozen input hash mismatch: {name}")
    t21_manifest = load_yaml(os.path.join(t21, "evidence-manifest.yml"))
    for entry in t21_manifest.get("files", []):
        path = os.path.join(t21, entry["path"])
        gate.require(os.path.isfile(path)
                     and sha256_file(path) == entry["sha256"],
                     f"T21 frozen module hash mismatch: {entry['path']}")
    r3_manifest = load_yaml(os.path.join(
        t23, "round-3", "evidence-manifest.round3.yml"))
    for entry in r3_manifest.get("files", []):
        path = os.path.join(t23, "round-3", entry["path"])
        gate.require(os.path.isfile(path)
                     and sha256_file(path) == entry["sha256"],
                     f"family-1 round-3 manifest hash mismatch: "
                     f"{entry['path']}")

    # ---- 2. T29 evidence manifest, both directions
    manifest_path = os.path.join(tree, "evidence-manifest.yml")
    if not os.path.isfile(manifest_path):
        gate.require(False, "T29 evidence-manifest.yml missing")
        listed: set = set()
    else:
        manifest = load_yaml(manifest_path)
        listed = set()
        for entry in manifest.get("files", []):
            path = os.path.join(tree, entry["path"])
            gate.require(os.path.isfile(path)
                         and sha256_file(path) == entry["sha256"],
                         f"T29 manifest pin mismatch: {entry['path']}")
            listed.add(entry["path"])
        expected_files = set()
        for root, dirs, files in os.walk(tree):
            dirs[:] = [d for d in dirs if d != "__pycache__"]
            for name in files:
                ext = os.path.splitext(name)[1]
                if ext not in (".py", ".yaml", ".json", ".txt"):
                    continue
                rel = os.path.relpath(os.path.join(root, name), tree)
                if rel == "evidence-manifest.yml":
                    continue
                if rel in ("IN-FLIGHT.md", "b1-progress.md",
                           "b1-result.json"):
                    continue
                expected_files.add(rel)
        for rel in sorted(expected_files - listed):
            gate.require(False,
                         f"T29 manifest incomplete: {rel} present but not "
                         f"listed (fail-closed both directions)")
        for rel in sorted(listed - expected_files):
            gate.require(False,
                         f"T29 manifest lists a non-existent/extra file: "
                         f"{rel}")

    # ---- 3. route records: independent recomputation of every T29 probe
    route_record_path = os.path.join(route_dir, "route-record.yaml")
    if not os.path.isfile(route_record_path):
        gate.require(False, "route-record.yaml missing")
        return gate.done()
    record = load_yaml(route_record_path)
    route_state = record["route_state"]
    goal_rec = record["goal_evaluation"]

    observations, up_evals, metas = [], [], []
    n_iterations = 0
    for iteration in range(1, 100):
        idir = os.path.join(route_dir, f"iteration-{iteration}")
        if not os.path.isdir(idir):
            break
        n_iterations = iteration
        observations.extend(load_json(
            os.path.join(idir, "probe-observations.json")))
        up_evals.extend(load_json(os.path.join(idir, "up-evaluations.json")))
        metas.extend(load_yaml(
            os.path.join(idir, "probe-run-meta.yaml")).get("probes", []))

    recomputed_by_probe = {}
    for obs in observations:
        rec = recompute_probe(gate, obs)
        recomputed_by_probe[obs["probe_id"]] = rec
    for up in up_evals:
        rec = recomputed_by_probe.get(up["probe_id"])
        if rec is None:
            gate.require(False,
                         f"up-evaluation for unknown probe {up['probe_id']}")
            continue
        if rec.get("broken"):
            gate.require(False,
                         f"{up['probe_id']}: up-evaluation on a probe whose "
                         f"provenance is broken: {rec.get('reasons')}")
            continue
        gate.require(rec["up_fired"] == up["up_rule"]["up_fired"],
                     f"{up['probe_id']}: UP-rule label mismatch: recorded "
                     f"{up['up_rule']['up_fired']} != recomputed "
                     f"{rec['up_fired']}")
        gate.require(rec["addendum3_fired"]
                     == up["addendum3_rule"]["addendum3_fired"],
                     f"{up['probe_id']}: addendum-3 label mismatch: recorded "
                     f"{up['addendum3_rule']['addendum3_fired']} != "
                     f"recomputed {rec['addendum3_fired']}")
        gate.require(up["up_rule"].get("passing_artifact_sha256")
                     == (rec["passing"][0] if rec["passing"] else ""),
                     f"{up['probe_id']}: passing-artifact pin mismatch")
    gate.require(len(up_evals) == len(observations),
                 "up-evaluations and probe-observations counts differ")

    # route_state absorption cross-check
    gate.require(len(route_state["probed"]) == len(observations),
                 "route_state.probed count != probe-observations count")
    obs_by_key = {(o["candidate_id"], o["correlation_id"]): o
                  for o in observations}
    for p in route_state["probed"]:
        o = obs_by_key.get((p["candidate_id"], p["correlation_id"]))
        gate.require(o is not None,
                     f"route_state probed entry without an observation: "
                     f"{p['candidate_id']}/{p['correlation_id']}")
        if o:
            gate.require(o["terminal_class"] == p["terminal_class"]
                         and o["bounded_input"]["scenario"] == p["scenario"]
                         and o["bounded_input"]["max_rounds"] == p["max_rounds"],
                         f"route_state probed entry contradicts its "
                         f"observation: {p['candidate_id']}")
            gate.require(p["scenario"] == "e2e-forged-report",
                         f"route probed a non-family operation: {p}")

    # ---- 4. recompute d*/d*-1 and G1-G3 from raw records
    fires, nofires = set(), set()
    for obs in observations:
        rec = recomputed_by_probe[obs["probe_id"]]
        if rec.get("broken"):
            continue
        (fires if rec["up_fired"] else nofires).add(
            obs["bounded_input"]["max_rounds"])
    d_star = min(fires) if fires else None
    g1 = 4 in fires
    g2 = False
    g2_structural = False
    dm1_fires = None
    if d_star is not None:
        if d_star == 1:
            g2, g2_structural = True, True
        elif (d_star - 1) in nofires:
            g2, dm1_fires = True, False
        elif (d_star - 1) in fires:
            dm1_fires = True
    probed_rounds = {o["bounded_input"]["max_rounds"] for o in observations}
    g3_route = d_star is not None and all(
        r in fires for r in probed_rounds if r >= d_star)

    # family-1 ledger recomputation from the pinned DBs
    f1_rows = []
    labels_by_probe = {
        row["probe_id"]: row
        for row in load_yaml(os.path.join(
            real_branch, "T23", "round-3", "mechanical-labels.yaml"))
        ["labels"]}
    for iteration in (1, 2, 3, 4):
        path = os.path.join(real_branch, "T23", "route",
                            f"iteration-{iteration}",
                            "probe-observations.json")
        for obs in load_json(path):
            rec = recompute_probe(gate, obs)
            lab = labels_by_probe.get(obs["probe_id"])
            gate.require(lab is not None,
                         f"family-1 {obs['probe_id']}: no frozen round-3 "
                         f"label row")
            if lab:
                gate.require(lab["terminal"] == rec["terminal_class"]
                             and lab["settlements"] == rec["settlements"],
                             f"family-1 {obs['probe_id']}: frozen label row "
                             f"disagrees with the recomputed db facts")
            f1_rows.append({"scenario": obs["bounded_input"]["scenario"],
                            "up_fired": rec.get("up_fired", False),
                            "addendum3_fired":
                                rec.get("addendum3_fired", False)})
    honest = [r for r in f1_rows if r["scenario"] == "e2e-happy"]
    lying = [r for r in f1_rows if r["scenario"] == "e2e-lying"]
    g3_honest = not any(r["up_fired"] for r in honest)
    g3_lying = not any(r["up_fired"] for r in lying)
    forged_add3 = any(recomputed_by_probe[o["probe_id"]].get("addendum3_fired")
                      for o in observations
                      if not recomputed_by_probe[o["probe_id"]].get("broken"))
    g3_partition = bool(g3_route and g3_honest and g3_lying
                        and not forged_add3)
    goal_met = bool(g1 and g2 and g3_partition)

    gate.require(goal_rec.get("d_star") == d_star,
                 f"d* mismatch: recorded {goal_rec.get('d_star')} != "
                 f"recomputed {d_star}")
    gate.require(goal_rec.get("g1_baseline_detected") == g1,
                 "recorded G1 disagrees with recomputation")
    gate.require(goal_rec.get("g2_boundary_established") == g2,
                 "recorded G2 disagrees with recomputation")
    gate.require(goal_rec.get("g3_partition_holds") == g3_partition,
                 "recorded G3 partition disagrees with recomputation")
    gate.require(goal_rec.get("goal_met") == goal_met,
                 "recorded goal_met disagrees with recomputation")
    g3_rec_path = os.path.join(route_dir, "g3-separation.yaml")
    if os.path.isfile(g3_rec_path):
        g3_rec = load_yaml(g3_rec_path)
        gate.require(g3_rec.get("partition_verdict") ==
                     ("holds" if g3_partition else "fails"),
                     "recorded partition verdict disagrees with "
                     "recomputation")
        gate.require(g3_rec.get("d_star") == d_star,
                     "g3-separation.yaml d_star disagrees with "
                     "recomputation")
    else:
        gate.require(False, "g3-separation.yaml missing")

    # ---- 5. matched negative: single declared factor, label read
    if d_star is not None and d_star > 1:
        neg = [o for o in observations
               if o["bounded_input"]["max_rounds"] == d_star - 1]
        pos = [o for o in observations
               if o["bounded_input"]["max_rounds"] == d_star]
        gate.require(len(neg) == 1 and len(pos) >= 1,
                     "matched negative: d*-1 or d* not probed exactly once")
        if neg and pos:
            n, p = neg[0], pos[0]
            gate.require(n["bounded_input"]["scenario"]
                         == p["bounded_input"]["scenario"],
                         "matched negative: scenario differs (more than one "
                         "factor)")
            gate.require(os.path.isfile(n["state_db_path"])
                         and os.path.isfile(p["state_db_path"]),
                         "matched negative: both sides must be real probes "
                         "with pinned DBs")
            gate.require(n["state_db_sha256"] != p["state_db_sha256"],
                         "matched negative: identical db pins; not two "
                         "distinct real probes")
            nmeta = next((m for m in metas
                          if m["probe_id"] == n["probe_id"]), {})
            pmeta = next((m for m in metas
                          if m["probe_id"] == p["probe_id"]), {})
            gate.require(nmeta.get("gauntlet_id_seed") !=
                         pmeta.get("gauntlet_id_seed"),
                         "matched negative: seeds are not fresh/distinct")
            gate.require(recomputed_by_probe[n["probe_id"]]["up_fired"]
                         is False,
                         "matched negative: the negative label must be the "
                         "recomputed UP-not-fired at d*-1 (read, never "
                         "assigned)")
            gate.require(recomputed_by_probe[p["probe_id"]]["up_fired"]
                         is True,
                         "matched negative: the d* probe must be "
                         "UP-fired by recomputation")
            record_neg = record.get("matched_negative", {})
            gate.require(record_neg.get("d_star_minus_1_fires") is False,
                         "route record must record d*-1 as not firing")

    # ---- 6. observer inertness
    obs_record_path = os.path.join(tree, "observer-raw",
                                   "observer-record.yaml")
    retained = route_state["retained_moves"]
    if not os.path.isfile(obs_record_path):
        gate.require(False, "observer-record.yaml missing")
    else:
        obs_record = load_yaml(obs_record_path)
        gate.require(len(obs_record.get("calls", [])) == len(retained),
                     "observer: call count != retained-move count "
                     "(exactly one call per retained move)")
        gate.require(obs_record.get("labels") is not None
                     and len(obs_record["labels"]) == len(retained),
                     "observer: labels not recorded per retained move")
        inert = obs_record.get("inertness_self_check", {})
        gate.require(inert.get("unchanged") is True
                     and inert.get("route_record_sha256_before")
                     == inert.get("route_record_sha256_after"),
                     "observer: recorded inertness self-check failed")
        gate.require(inert.get("route_record_sha256_after")
                     == sha256_file(route_record_path),
                     "observer: route-record.yaml changed after the route "
                     "closed (observer contamination)")
        # the route goal recomputation above used ONLY route/ records and
        # DBs; assert the recorded goal equals it (observer-independent)
        gate.require(goal_rec.get("goal_met") == goal_met,
                     "observer inertness: recorded goal differs from the "
                     "observer-independent recomputation")
        # no GOAL-INPUT route record may reference observer data (payment
        # documentation may name the observer; goal inputs may not)
        goal_inputs = [os.path.join(route_dir, "route-record.yaml"),
                       os.path.join(route_dir, "g3-separation.yaml"),
                       os.path.join(route_dir, "family1-rule-evaluation.json")]
        for iteration in range(1, n_iterations + 1):
            idir = os.path.join(route_dir, f"iteration-{iteration}")
            for name in ("probe-observations.json", "up-evaluations.json",
                         "judgment-results.json", "selection-record.json",
                         "route-state-after.yaml", "neighborhood.yaml",
                         "candidates.json", "admission-decisions.json"):
                goal_inputs.append(os.path.join(idir, name))
        for path in goal_inputs:
            if not os.path.isfile(path):
                gate.require(False,
                             f"goal-input record missing: "
                             f"{os.path.relpath(path, tree)}")
                continue
            with open(path) as f:
                text = f.read()
            if "observer" in text.lower():
                gate.require(
                    False,
                    f"goal-input route record references observer data: "
                    f"{os.path.relpath(path, tree)}")

    # ---- 7. bounds
    gate.require(len(retained) <= 5,
                 f"bound violated: retained moves {len(retained)} > 5")
    gate.require(len(observations) <= 8,
                 f"bound violated: probes {len(observations)} > 8")
    gate.require(n_iterations <= 4,
                 f"bound violated: iterations {n_iterations} > 4")
    for m in metas:
        gate.require(isinstance(m.get("wall_seconds"), (int, float))
                     and m["wall_seconds"] <= 240,
                     f"bound violated: probe {m.get('probe_id')} wall > 240s")
    pay_path = os.path.join(route_dir, "payment-counters.yaml")
    gate.require(os.path.isfile(pay_path), "payment-counters.yaml missing")
    if os.path.isfile(pay_path):
        pay = load_yaml(pay_path)
        gate.require(pay.get("wall_clock_seconds", 10 ** 9) <= 1800,
                     "bound violated: route wall > 1800s")

        # ---- 8. payment counters recomputed
        gate.require(pay.get("probes_run") == len(observations),
                     "payment probes_run != recorded observations")
        failed = sum(1 for o in observations
                     if o["terminal_class"] == "run_error")
        gate.require(pay.get("failed_executions") == failed,
                     "payment failed_executions != run_error count")
        gate.require(pay.get("provider_calls", {}).get("count")
                     == len(pay.get("provider_calls", {}).get("by_identity",
                                                             [])),
                     "payment provider_calls.count != by_identity length")
        gen_calls = [c for c in pay["provider_calls"]["by_identity"]
                     if str(c.get("purpose", "")).startswith(
                         "candidate_generation")]
        gate.require(len(gen_calls) == pay["provider_calls"]["count"],
                     "payment: generation calls != total route provider "
                     "calls (route calls must be generation-only)")
        gen_ok = [c for c in gen_calls if c.get("outcome") == "responded"]
        gate.require(len(gen_ok) >= n_iterations,
                     "payment: fewer successful generation calls than "
                     "iterations")
        iters_with_records = set()
        for c in gen_calls:
            purpose = str(c.get("purpose", ""))
            if "iter" in purpose:
                iters_with_records.add(int(purpose.rsplit("iter", 1)[-1]))
        gate.require(iters_with_records ==
                     set(range(1, n_iterations + 1)),
                     "payment: generation-call iteration coverage mismatch")
        if os.path.isfile(obs_record_path):
            gate.require(
                len(load_yaml(obs_record_path).get("provider_calls", []))
                == len(retained),
                "observer provider-call count != retained moves")
        gate.require(pay.get("operator_decisions") == 0,
                     "payment: operator decisions must be 0 in the loop")

    # typed route_record validation through the FROZEN T21 contract module
    sys.path.insert(0, t21)
    import route_contracts as rc  # noqa: E402  (frozen module, unmodified)
    err = rc.validate("route_record", record["route_record"])
    gate.require(err is None,
                 f"typed route_record invalid per the frozen T21 contract: "
                 f"{err}")
    gate.require(record["route_record"]["prereg_sha256"]
                 == "e844fce47c9860eeaacac0b8d27f0d28ef386da21f1c45ed878e85ba02af9672",
                 "route_record prereg_sha256 mismatch")

    return gate.done()


if __name__ == "__main__":
    sys.exit(main())
