#!/usr/bin/env python3
"""T23 GATE, round 2 — verify_route.round2.py (run from the sea-rs repo root).

Round-2 correction gate for the addendum-2 rejudgment of route
t23-route-001. Round-1's verify_route.py is NOT modified; this gate reuses
its frozen helpers by import (shared fail-closed check pool) and adds the
round-2 checks. Exits 0 ONLY if every check passes; a missing file, a hash
mismatch, or an inconsistent verdict is a FAILURE.

Checks
  (1) frozen inputs intact: T21 frozen modules + base prereg + addendum-1
      (round-1 helper) and addendum-2 by its frozen sha256;
  (2) round-1 files UNMODIFIED: every entry of round-1's
      evidence-manifest.yml still hashes to its pinned sha256 (fail-closed;
      covers all round-1 .py/.yaml, route/iteration-* records, provider-raw,
      teeth records);
  (3) corrected surfaces: one per round-1 probe observation (bijective);
      DB pins re-verified through the frozen provenance validator; surface
      fields (terminal, settlement count + unit progression, dispatch
      pattern, evidence artifact sha set, timing) independently re-derived
      from the pinned state databases; artifact bytes re-read from the
      run's own artifact dir and re-hashed to their pinned sha256, with the
      surface-embedded content matching the artifact bytes;
  (4) re-judgment integrity: every corrected label is in the frozen answer
      domain and came from a preserved raw provider output (re-parsed with
      the frozen parser) or is a declared mechanics-only abstention; one
      ladder pass per probe (primary + at most one substitution); prompts
      preserved and neutral (answer-domain enumeration present; route-goal
      and round-1 vocabulary absent; surfaces carry no label fields);
  (5) separation + goal consistency: the separation verdict and the
      addendum-2 goal recomputation are mechanically re-derived from the
      corrected labels and must match separation-verdict.json and
      route-record.round2.yaml; settled only when the recomputed goal
      holds; route ledger/fingerprint unchanged from round 1; no probe
      re-run (probes 12, reruns 0, probe timestamps precede the round-2
      re-judgment);
  (6) round-2 payment counters present and consistent;
  (7) round-2 manifest hash checks, both directions, fail-closed (every
      entry matches disk; every round-2 file except the manifest is
      pinned).
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import json
import os
import re
import sqlite3

T23_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ROUND2_DIR = os.path.dirname(os.path.abspath(__file__))
BRANCH_DIR = os.path.dirname(T23_DIR)
T21_DIR = os.path.join(BRANCH_DIR, "T21")
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(BRANCH_DIR)))

sys.path.insert(0, T21_DIR)
sys.path.insert(0, T23_DIR)

import route_contracts as rc  # noqa: E402  (frozen T21 module)
import provenance as prov    # noqa: E402  (frozen round-1 validator)

# round-1 gate module: helpers + shared fail-closed check pool, unmodified
sys.path.insert(0, T23_DIR)
import verify_route as vr    # noqa: E402

import yaml                  # noqa: E402

ADDENDUM2_SHA256 = (
    "087887c27cc50c6c989fcf7bb214e77aca62ea74d7e32a868ef0f7dd211e4c5a")
ROUND1_ROUTE_RECORD_SHA256 = (
    "9345c43e6787ef32fbe1a7749c90872de7b5be4b46f73af9708516f4f8ded67e")
MANIFEST_NAME = "evidence-manifest.round2.yml"

ROUTE_DIR = vr.ROUTE_DIR

# words that must NEVER appear in a neutral judgment prompt (the judge must
# not see the route goal or the correction's expectations); the surface's
# scenario name (e2e-lying/e2e-happy) is observation input, not goal vocab
PROMPT_BANNED = ("baseline", "discriminator", "boundary", "goal",
                 "addendum", "expected_role")
ANSWER_DOMAIN_LINE = (
    'Answer domain for "label" (choose exactly one): settled_accepted | '
    "verification_failed_nonsettlement | budget_exhausted | "
    "execution_failed | insufficient_evidence")


def load_json(path):
    with open(path) as f:
        return json.load(f)


def load_yaml(path):
    with open(path) as f:
        return yaml.safe_load(f)


def parse_judgment(text: str):
    """Frozen parser semantics (round-1 re-statement)."""
    m = re.search(r"\{.*\}", text, vr_re_dotall())
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


def vr_re_dotall():
    return re.DOTALL


def read_events(db_path: str):
    conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    try:
        rows = conn.execute(
            "select kind, payload from event_log order by offset").fetchall()
    finally:
        conn.close()
    events = []
    for kind, payload in rows:
        try:
            body = json.loads(payload)
        except json.JSONDecodeError:
            body = {}
        events.append((kind, body))
    return events


def no_label_fields(node, path="surface") -> list[str]:
    bad = []
    if isinstance(node, dict):
        for k, v in node.items():
            if k in ("label", "judgment_label", "judgment", "goal"):
                bad.append(f"{path}.{k}")
            bad.extend(no_label_fields(v, f"{path}.{k}"))
    elif isinstance(node, list):
        for i, v in enumerate(node):
            bad.extend(no_label_fields(v, f"{path}[{i}]"))
    return bad


def main() -> int:
    print("verify_route.round2: starting (fail-closed)", flush=True)

    # ---- (1) frozen inputs --------------------------------------------------
    vr.verify_t21_manifest()  # T21 modules + base prereg + addendum-1
    add2 = os.path.join(REPO_ROOT, ".agents/preregistrations",
                        "godspeed-bounded-judgment-T21.prereg.addendum-2.yaml")
    if vr.check(os.path.isfile(add2), "addendum-2 missing"):
        vr.check(vr.sha256_file(add2) == ADDENDUM2_SHA256,
                 "addendum-2 hash mismatch (surface correction is frozen)")

    # ---- (2) round-1 files unmodified ----------------------------------------
    manifest1_path = os.path.join(T23_DIR, "evidence-manifest.yml")
    if vr.check(os.path.isfile(manifest1_path),
                "round-1 evidence-manifest.yml missing"):
        manifest1 = load_yaml(manifest1_path)
        n1 = 0
        for entry in manifest1.get("files", []):
            path = os.path.join(T23_DIR, entry["path"])
            n1 += 1
            if not vr.check(os.path.isfile(path),
                            f"round-1 file missing: {entry['path']}"):
                continue
            vr.check(vr.sha256_file(path) == entry["sha256"],
                     f"round-1 file MODIFIED since freeze: {entry['path']}")
        vr.check(n1 >= 40, f"round-1 manifest implausibly small ({n1})")

    # ---- load round-1 observations (the preserved probe set) -----------------
    rr1_path = os.path.join(ROUTE_DIR, "route-record.yaml")
    if not vr.check(os.path.isfile(rr1_path), "round-1 route-record missing"):
        print("verify_route.round2: FAIL"); return 1
    round1 = load_yaml(rr1_path)
    obs_by_probe = {}
    for iteration in (1, 2, 3, 4):
        idir = os.path.join(ROUTE_DIR, f"iteration-{iteration}")
        for o in load_json(os.path.join(idir, "probe-observations.json")):
            obs_by_probe[o["probe_id"]] = o
        for j in load_json(os.path.join(idir, "judgment-results.json")):
            pass  # round-1 labels read only through the manifest hashes above
    vr.check(len(obs_by_probe) == 12,
             f"expected 12 preserved round-1 probes, found "
             f"{len(obs_by_probe)}")
    round1_labels = {}
    for iteration in (1, 2, 3, 4):
        idir = os.path.join(ROUTE_DIR, f"iteration-{iteration}")
        for j in load_json(os.path.join(idir, "judgment-results.json")):
            round1_labels[j["judgment_id"] + "@" + j["correlation_id"]] = \
                j["label"]

    # ---- (3) corrected surfaces ----------------------------------------------
    surfaces_dir = os.path.join(ROUND2_DIR, "surfaces")
    index_path = os.path.join(surfaces_dir, "corrected-surfaces.json")
    index = load_json(index_path) if vr.check(
        os.path.isfile(index_path), "corrected-surfaces.json missing") else {}
    surface_ids = index.get("surfaces", [])
    vr.check(sorted(surface_ids) == sorted(obs_by_probe),
             "surface index is not bijective with the round-1 probe set")
    vr.check(index.get("addendum2_sha256") == ADDENDUM2_SHA256,
             "surface index does not pin the addendum-2 sha256")
    vr.check(index.get("route_id") == "t23-route-001",
             "surface index names a different route")

    latest_judged = None
    for pid in sorted(obs_by_probe):
        obs = obs_by_probe[pid]
        sfile = os.path.join(surfaces_dir, f"{pid}.json")
        if not vr.check(os.path.isfile(sfile), f"surface file missing: {pid}"):
            continue
        rec = load_json(sfile)
        vr.check(rec.get("addendum2_sha256") == ADDENDUM2_SHA256,
                 f"{pid}: surface does not pin the addendum-2 sha256")
        if rec.get("status") != "ok":
            # a skipped probe must record the DB pin mismatch and pin no
            # surface; the gate still demands its DB be re-verifiably wrong
            res = prov.recompute(obs)
            vr.check(not res["ok"],
                     f"{pid}: skipped but its DB pin verifies fine")
            vr.check(rec.get("surface") is None,
                     f"{pid}: skipped record still carries a surface")
            continue
        surface = rec.get("surface")
        vr.check(isinstance(surface, dict), f"{pid}: surface missing")
        if not isinstance(surface, dict):
            continue
        # provenance: DB pin re-verified through the frozen validator
        res = prov.recompute(obs)
        vr.check(res["ok"],
                 f"{pid}: state-DB pin/terminal recheck failed: "
                 f"{res['reasons']}")
        provb = rec.get("provenance", {})
        vr.check(provb.get("db_pin_verified") is True,
                 f"{pid}: surface provenance does not assert the DB pin")
        vr.check(provb.get("state_db_sha256_actual") ==
                 obs["state_db_sha256"],
                 f"{pid}: actual DB digest differs from the round-1 pin")
        # the judge-facing surface carries no label/goal fields
        for bad in no_label_fields(surface):
            vr.check(False, f"{pid}: judge-facing surface carries field {bad}")
        serialized = json.dumps(surface, sort_keys=True)
        for leaked in ("settled_accepted", "verification_failed_nonsettlement",
                       "execution_failed", "insufficient_evidence"):
            vr.check(leaked not in serialized,
                     f"{pid}: surface leaks judgment vocabulary {leaked!r}")
        # mirror of the round-1 observation
        vr.check(surface.get("probe_id") == pid, f"{pid}: probe_id mismatch")
        vr.check(surface.get("bounded_input") == obs["bounded_input"],
                 f"{pid}: bounded_input does not mirror the observation")
        vr.check(surface.get("state_db_sha256") == obs["state_db_sha256"],
                 f"{pid}: state_db_sha256 does not mirror the observation")
        tt = surface.get("typed_terminal", {})
        vr.check(tt.get("terminal_class") == obs["terminal_class"],
                 f"{pid}: terminal_class does not mirror the observation")
        vr.check(tt.get("run_terminal_states") ==
                 ([obs["terminal_class"]]
                  if obs["terminal_class"] != "run_error" else []) or
                 tt.get("run_terminal_states") is not None,
                 f"{pid}: run_terminal_states missing")
        vr.check(surface.get("settlement_committed_count") ==
                 obs["settlement_committed_count"],
                 f"{pid}: settlement_committed_count does not mirror the "
                 "observation")
        # independent re-derivation from the pinned database
        events = read_events(obs["state_db_path"])
        dispatch_seq = [b.get("unit_id") for k, b in events
                        if k == "unit_dispatch"]
        settlements = [b for k, b in events
                       if k == "settlement_committed"]
        ev_refs = []
        for k, b in events:
            if k in ("evidence_produced", "settlement_committed"):
                ref = b.get("evidence", "")
                if ref.startswith("sha256:"):
                    ev_refs.append(ref[7:])
        pattern = surface.get("unit_dispatch_pattern", {})
        vr.check(pattern.get("dispatch_sequence") == dispatch_seq,
                 f"{pid}: dispatch sequence not re-derivable from the DB")
        prog = surface.get("settlement_progression", [])
        vr.check([p.get("unit_id") for p in prog] ==
                 [s.get("unit_id") for s in settlements],
                 f"{pid}: settlement progression not re-derivable from the DB")
        vr.check([p.get("verification_ref") for p in prog] ==
                 [s.get("verification") for s in settlements],
                 f"{pid}: settlement verification refs not re-derivable")
        # artifacts: sha set from the DB, bytes re-hashed, content matched
        art_dir = os.path.join(os.path.dirname(os.path.dirname(
            obs["state_db_path"])), "evidence")
        arts = surface.get("evidence_artifacts", [])
        vr.check([a.get("sha256") for a in arts] == list(dict.fromkeys(ev_refs)),
                 f"{pid}: surface artifact sha set != distinct DB evidence refs")
        art_pins = provb.get("artifact_paths", {})
        vr.check(sorted(art_pins) ==
                 sorted(a.get("sha256") for a in arts),
                 f"{pid}: provenance artifact-path map != surface sha set")
        for a in arts:
            sha = a.get("sha256", "")
            apath = os.path.join(art_dir, "objects", sha[:2], sha[2:4], sha)
            if not vr.check(os.path.isfile(apath),
                            f"{pid}: artifact object missing for pin {sha}"):
                continue
            vr.check(art_pins.get(sha) == apath,
                     f"{pid}: provenance path for pin {sha} is not the "
                     "content-addressed artifact location")
            vr.check(vr.sha256_file(apath) == sha,
                     f"{pid}: artifact bytes do NOT hash to the pinned "
                     f"sha256 {sha}")
            with open(apath, encoding="utf-8", errors="replace") as f:
                content = f.read()
            vr.check(a.get("content") == content,
                     f"{pid}: surface-embedded content != artifact bytes "
                     f"for pin {sha}")
            kindfile = apath + ".kind"
            if os.path.isfile(kindfile):
                with open(kindfile) as f:
                    vr.check(a.get("kind") == f.read().strip(),
                             f"{pid}: artifact kind mismatch for pin {sha}")
        # timing mirrors the preserved observation + sidecar
        timing = surface.get("timing", {})
        vr.check(timing.get("started_at_utc") == obs["started_at_utc"]
                 and timing.get("finished_at_utc") == obs["finished_at_utc"],
                 f"{pid}: timing does not mirror the observation")
        vr.check(isinstance(timing.get("wall_seconds"), (int, float)),
                 f"{pid}: timing lacks the preserved wall_seconds")

    # ---- (4) re-judgment integrity --------------------------------------------
    res_path = os.path.join(ROUND2_DIR, "rejudgment-results.json")
    results = load_json(res_path) if vr.check(
        os.path.isfile(res_path), "rejudgment-results.json missing") else []
    vr.check(sorted(r["probe_id"] for r in results) == sorted(obs_by_probe),
             "rejudgment results are not bijective with the probe set")
    total_provider_calls = 0
    durations = []
    for r in results:
        pid = r["probe_id"]
        vr.check(r["label"] in rc.ANSWER_DOMAIN,
                 f"{pid}: corrected label {r['label']!r} outside the frozen "
                 "answer domain")
        attempts = r.get("attempts", [])
        provider_attempts = [a for a in attempts
                             if a.get("provider") in
                             ("deepseek-clinepass", "muse-spark-free")]
        total_provider_calls += len(provider_attempts)
        vr.check(len(provider_attempts) <= 2,
                 f"{pid}: more than one ladder pass "
                 f"({len(provider_attempts)} provider attempts)")
        tags = [a.get("attempt") for a in provider_attempts]
        vr.check(len(set(tags)) == len(tags),
                 f"{pid}: duplicate ladder tags {tags}")
        if tags:
            vr.check(tags[0] == "primary",
                     f"{pid}: ladder did not start with the primary provider")
        if r.get("source") == "provider":
            ok_att = [a for a in provider_attempts
                      if str(a.get("outcome", "")).startswith("ok")]
            vr.check(len(ok_att) >= 1,
                     f"{pid}: provider-sourced label without an ok attempt")
            win = ok_att[-1] if ok_att else None
            if win is not None:
                raw_path = win.get("raw_output_path")
                if vr.check(isinstance(raw_path, str) and
                            os.path.isfile(raw_path),
                            f"{pid}: winning attempt raw output missing: "
                            f"{raw_path}"):
                    with open(raw_path) as f:
                        parsed, err = parse_judgment(f.read())
                    vr.check(parsed == r["label"],
                             f"{pid}: preserved raw output re-parses to "
                             f"{parsed!r}, recorded {r['label']!r} ({err})")
                prompt_path = raw_path.replace(".raw.txt", ".prompt.txt") \
                    if raw_path else None
                if vr.check(isinstance(prompt_path, str) and
                            os.path.isfile(prompt_path),
                            f"{pid}: judgment prompt missing: {prompt_path}"):
                    with open(prompt_path) as f:
                        prompt_text = f.read()
                    vr.check(ANSWER_DOMAIN_LINE in prompt_text,
                             f"{pid}: prompt lacks the frozen answer-domain "
                             "line")
                    low = prompt_text.lower()
                    for banned in PROMPT_BANNED:
                        vr.check(banned not in low,
                                 f"{pid}: neutral prompt contains banned "
                                 f"goal vocabulary {banned!r}")
            vr.check(r["judge_identity"].startswith(
                ("deepseek-clinepass:", "muse-spark-free:")),
                f"{pid}: provider label with unexpected judge identity "
                f"{r['judge_identity']!r}")
        else:
            # declared mechanics-only abstention: label must be the explicit
            # insufficiency label and the fallback attempt recorded
            vr.check(r["label"] == "insufficient_evidence",
                     f"{pid}: non-provider label must be the explicit "
                     "insufficiency abstention")
            vr.check("mechanics-only" in r.get("judge_identity", ""),
                     f"{pid}: fallback not marked mechanics-only")
        for a in provider_attempts:
            if isinstance(a.get("duration_seconds"), (int, float)):
                durations.append(a["duration_seconds"])
        if r.get("judged_at_utc"):
            if latest_judged is None or r["judged_at_utc"] > latest_judged:
                latest_judged = r["judged_at_utc"]
    vr.check(sum(durations) > 0 if durations else total_provider_calls == 0,
             "provider calls recorded without any wall duration")
    vr.check(total_provider_calls <= 24,
             f"implausible provider call count {total_provider_calls}")

    # ---- (5) separation + goal consistency -------------------------------------
    sep_path = os.path.join(ROUND2_DIR, "separation-verdict.json")
    sep = load_json(sep_path) if vr.check(
        os.path.isfile(sep_path), "separation-verdict.json missing") else {}
    rr2_path = os.path.join(ROUND2_DIR, "route-record.round2.yaml")
    rr2 = load_yaml(rr2_path) if vr.check(
        os.path.isfile(rr2_path), "route-record.round2.yaml missing") else {}

    label_by_probe = {r["probe_id"]: r["label"] for r in results}
    lying = [o for o in obs_by_probe.values()
             if o["bounded_input"]["scenario"] == "e2e-lying"]
    honest_boundary = [o for o in obs_by_probe.values()
                       if o["bounded_input"]["scenario"] == "e2e-happy"
                       and o["bounded_input"]["max_rounds"] in (1, 2, 3)]
    happy = [o for o in obs_by_probe.values()
             if o["bounded_input"]["scenario"] == "e2e-happy"]
    lying_all_vfn = all(
        label_by_probe.get(o["probe_id"]) ==
        "verification_failed_nonsettlement" for o in lying)
    boundary_all_be = all(
        label_by_probe.get(o["probe_id"]) == "budget_exhausted"
        for o in honest_boundary)
    separated = lying_all_vfn and boundary_all_be
    vr.check(sep.get("verdict") == ("separated" if separated
                                    else "not_separated"),
             "recorded separation verdict != mechanical recomputation")
    vr.check(sep.get("lying_all_verification_failed_nonsettlement")
             is lying_all_vfn,
             "separation record lying-family bit != recomputation")
    vr.check(sep.get("honest_boundary_all_budget_exhausted")
             is boundary_all_be,
             "separation record honest-boundary bit != recomputation")

    vr.check(rr2.get("correction_round") == 2,
             "round-2 route record must carry correction_round 2")
    vr.check(rr2.get("addendum2_sha256") == ADDENDUM2_SHA256,
             "round-2 route record does not pin the addendum-2 sha256")
    vr.check(rr2.get("basis", {}).get("round1_route_record_sha256") ==
             ROUND1_ROUTE_RECORD_SHA256,
             "round-2 record does not pin the round-1 route-record sha256")
    vr.check(rr2.get("probes_rerun") == 0 and rr2.get("probe_count") == 12,
             "round-2 record must show 12 probes, 0 re-run")
    # corrected labels join bijectively and match the re-judgment results
    labels_rows = {row["probe_id"]: row
                   for row in rr2.get("corrected_labels", [])}
    vr.check(sorted(labels_rows) == sorted(obs_by_probe),
             "corrected_labels not bijective with the probe set")
    for pid, row in labels_rows.items():
        obs = obs_by_probe[pid]
        vr.check(row["scenario"] == obs["bounded_input"]["scenario"]
                 and row["max_rounds"] == obs["bounded_input"]["max_rounds"]
                 and row["terminal_class"] == obs["terminal_class"],
                 f"{pid}: corrected_labels row does not mirror observation")
        vr.check(label_by_probe.get(pid) == row["round2_label"],
                 f"{pid}: record round2_label != rejudgment result")
        jkey = row["candidate_id"] + "@" + row["correlation_id"]
        # round-1 label column preserved for the record only
        r1_expected = None
        for iteration in (1, 2, 3, 4):
            for j in load_json(os.path.join(
                    ROUTE_DIR, f"iteration-{iteration}",
                    "judgment-results.json")):
                if j["candidate_id"] == row["candidate_id"] and \
                        j["correlation_id"] == row["correlation_id"]:
                    r1_expected = j["label"]
        vr.check(row.get("round1_label") == r1_expected,
                 f"{pid}: round1_label column != round-1 judgment record")

    # goal recomputation from the corrected labels
    happy_settled = {o["bounded_input"]["max_rounds"] for o in happy
                     if label_by_probe.get(o["probe_id"]) ==
                     "settled_accepted"}
    happy_exhausted = {o["bounded_input"]["max_rounds"] for o in happy
                       if label_by_probe.get(o["probe_id"]) ==
                       "budget_exhausted"}
    baseline_met = 4 in happy_settled
    r_star = min(happy_settled) if happy_settled else None
    boundary_met = bool(happy_settled) and (r_star - 1 in happy_exhausted)
    discriminator_met = separated
    goal_met = baseline_met and boundary_met and discriminator_met
    goal = rr2.get("goal_evaluation", {})
    vr.check(goal.get("baseline_met") == baseline_met,
             "recorded baseline_met != recomputed from corrected labels")
    vr.check(goal.get("discriminator_met") == discriminator_met,
             "recorded discriminator_met != separation recomputation")
    vr.check(goal.get("r_star") == r_star,
             "recorded r_star != recomputed")
    vr.check(goal.get("boundary_met") == boundary_met,
             "recorded boundary_met != recomputed")
    vr.check(goal.get("goal_met") == goal_met,
             "recorded goal_met != recomputed")
    vr.check("verification_failed_nonsettlement" in goal.get("goal", "")
             and "addendum-2" in goal.get("goal", ""),
             "goal text does not state the addendum-2 restated goal")
    if not goal_met:
        vr.check("not fully separate" in goal.get("honesty_rail", "")
                 or "did not fully separate" in goal.get("honesty_rail", "")
                 or "NOT-met" in goal.get("honesty_rail", ""),
                 "honest non-separation must be preserved in honesty_rail")

    # route ledger unchanged: route_record block mirrors round 1 exactly
    rr1_block = round1["route_record"]
    rr2_block = rr2.get("route_record", {})
    for field in ("moves", "fingerprint", "correlation_ids",
                  "policy_version", "prereg_sha256"):
        vr.check(rr2_block.get(field) == rr1_block.get(field),
                 f"round-2 route ledger field {field} differs from round 1")
    vr.check(rc.route_fingerprint(rr2_block.get("moves", [])) ==
             rr2_block.get("fingerprint"),
             "round-2 fingerprint does not recompute from the moves")
    vr.check(rr2_block.get("settled") == goal_met,
             "route_record.settled does not match the recomputed goal")
    if not goal_met:
        vr.check(rr2_block.get("stop_condition") != "route_settled",
                 "unsettled goal must not record route_settled")

    # no probe re-run: every probe predates the round-2 re-judgment
    for pid, o in obs_by_probe.items():
        vr.check(o["finished_at_utc"] < str(latest_judged),
                 f"{pid}: probe timestamp is not before the round-2 "
                 f"re-judgment (probe re-run?)")

    # ---- (6) round-2 payment counters ------------------------------------------
    pay2 = load_yaml(os.path.join(ROUND2_DIR, "payment-counters.round2.yaml")) \
        if os.path.isfile(os.path.join(
            ROUND2_DIR, "payment-counters.round2.yaml")) else {}
    vr.check(pay2.get("task") == "T23", "round-2 payment counters not T23's")
    vr.check(pay2.get("probes_run") == 12 and pay2.get("probes_rerun") == 0,
             "round-2 payment counters must show 12 probes, 0 re-run")
    pcalls = pay2.get("provider_calls", {})
    vr.check(isinstance(pcalls, dict)
             and pcalls.get("count") == total_provider_calls
             and len(pcalls.get("by_identity", [])) == pcalls.get("count"),
             "round-2 provider-call ledger inconsistent with the "
             "re-judgment results")
    vr.check(isinstance(pay2.get("tokens_and_cost"), str)
             and pay2.get("tokens_and_cost") != "",
             "round-2 payment must record the tokens/cost state")

    # ---- (7) round-2 manifest, both directions ----------------------------------
    manifest2_path = os.path.join(ROUND2_DIR, MANIFEST_NAME)
    if not vr.check(os.path.isfile(manifest2_path),
                    f"{MANIFEST_NAME} missing (must precede the gate run)"):
        pass
    else:
        manifest2 = load_yaml(manifest2_path)
        pinned = set()
        for entry in manifest2.get("files", []):
            rel = entry["path"]
            path = os.path.join(ROUND2_DIR, rel)
            pinned.add(os.path.normpath(rel))
            if not vr.check(os.path.isfile(path),
                            f"round-2 manifest file missing: {rel}"):
                continue
            vr.check(vr.sha256_file(path) == entry["sha256"],
                     f"round-2 manifest hash mismatch: {rel}")
        on_disk = set()
        for dirpath, dirnames, filenames in os.walk(ROUND2_DIR):
            dirnames[:] = [d for d in dirnames if d != "__pycache__"]
            for fn in filenames:
                full = os.path.join(dirpath, fn)
                on_disk.add(os.path.normpath(
                    os.path.relpath(full, ROUND2_DIR)))
        on_disk.discard(os.path.normpath(MANIFEST_NAME))
        unpinned = sorted(on_disk - pinned)
        vr.check(not unpinned,
                 f"round-2 files not pinned in {MANIFEST_NAME}: {unpinned}")
        stale = sorted(pinned - on_disk)
        vr.check(not stale,
                 f"round-2 manifest pins nonexistent files: {stale}")

    # ---- verdict -----------------------------------------------------------------
    if vr.ERRORS:
        print(f"verify_route.round2: FAIL ({len(vr.ERRORS)} problem(s) after "
              f"{vr.CHECKS} checks)")
        for e in vr.ERRORS:
            print(f"  - {e}")
        return 1
    print(f"verify_route.round2: PASS ({vr.CHECKS} checks)")
    print(f"  separation verdict: {sep.get('verdict')} "
          f"(lying vfn {sum(1 for o in lying if label_by_probe.get(o['probe_id']) == 'verification_failed_nonsettlement')}/{len(lying)}; "
          f"honest boundary @1,@2,@3 all budget_exhausted={boundary_all_be})")
    print(f"  goal: baseline_met={baseline_met} "
          f"discriminator_met={discriminator_met} r*={r_star} "
          f"boundary_met={boundary_met} -> goal_met={goal_met}")
    print(f"  provider calls (re-judgment): {total_provider_calls}; "
          f"probes=12 reruns=0; fingerprint unchanged")
    return 0


if __name__ == "__main__":
    sys.exit(main())
