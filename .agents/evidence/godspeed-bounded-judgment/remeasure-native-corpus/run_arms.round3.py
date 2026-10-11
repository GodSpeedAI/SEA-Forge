#!/usr/bin/env python3
"""Round-3 continuation pass: resume of the round-2 pass
(run_arms.round2.py), which was interrupted mid-arm2 after nat-05 sample 2.

Correction content (the ONLY changes from run_arms.round2.py):
  C1. resume-safe call-slot resolution: a slot whose preserved evidence
      already exists is reconstructed from disk and NEVER re-rolled —
        - deepseek raw present          -> reused primary output;
        - deepseek error + muse raw     -> the completed substitution is
          reused (the round-2 preserved_call would have re-rolled the
          deepseek leg of such a slot, e.g. nat-02-s2);
        - deepseek error + muse error   -> both legs failed (typed result);
        - deepseek error only           -> the policy's single substitution
          was never attempted (interruption) and is attempted exactly once;
        - nothing present               -> fresh call under the frozen
          substitution policy;
  C2. the living results file is updated IN PLACE (round-2 rebuilt it from
      scratch, which would have discarded recorded per-call wall-time
      metadata of already-completed calls);
  C3. deviation DV-5 records the round-2 interruption; the two completed
      nat-05 call slots are reconstructed from their preserved raw outputs
      with duration_lost_in_interruption: true (their wall time is an honest
      lower bound);
  C4. the historical-freshness comparison is reported for all 12 route rows
      (operator directive) beside the 4-lying-row block.

Everything else (prereg checks, fail-closed DB re-hashes, template pinning,
prompt assembly, parsers, aggregation, scoring, payment counters, per-row
incremental results writes) is the frozen round-2 logic, imported unchanged
from run_arms.round2.py.

No provider call is ever re-rolled. Frozen before its first execution;
never overwritten.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import importlib.util
import json
import os

OUT_DIR = ("/home/sprime01/projects/sea-rs/.agents/evidence/"
           "godspeed-bounded-judgment/remeasure-native-corpus")
R2_PATH = os.path.join(OUT_DIR, "run_arms.round2.py")

spec = importlib.util.spec_from_file_location("remeasure_round2_runner",
                                              R2_PATH)
r2 = importlib.util.module_from_spec(spec)
sys.modules["remeasure_round2_runner"] = r2
spec.loader.exec_module(r2)

PRIMARY = r2.PRIMARY
SUB = r2.SUB
MODEL = r2.MODEL
ARM2_RAW = r2.ARM2_RAW
ARM2_SURF = r2.ARM2_SURF
ARM2B_RAW = r2.ARM2B_RAW

DV5 = {
    "id": "DV-5",
    "issue": (
        "the round-2 pass was interrupted mid-arm2 (after nat-05 sample 2, "
        "before sample 3): 14 of 45 arm2 call slots had completed with "
        "preserved raw outputs (including the nat-02-s2 slot whose deepseek "
        "leg timed out and whose single muse substitution completed), but "
        "the results file had only persisted the arm2 blocks of "
        "nat-01..nat-04 — the per-call wall-time metadata of the two "
        "completed nat-05 slots (s1, s2) was lost with the interrupted "
        "process"),
    "reading": (
        "resume as round-3 (run_arms.round3.py): preserved raw outputs are "
        "re-used verbatim and never re-rolled (a naive re-run of "
        "run_arms.round2.py would have re-rolled the nat-02-s2 deepseek leg "
        "because only its error file and muse raw are on disk); any "
        "interrupted slot's un-attempted single substitution is completed "
        "exactly once; the two nat-05 call records are reconstructed from "
        "their preserved raw outputs with duration_lost_in_interruption: "
        "true, so arm2 wall time is an honest lower bound; the results file "
        "is updated in place"),
    "kind": "execution-interruption-recovery",
}


def preserved_slot(raw_dir: str, stem: str, prompt: str) -> dict:
    """Resume-safe version of the frozen preserved_call policy (C1).

    Returns the same shape as run_arms.round2.preserved_call. An existing
    prompt file is drift-checked; a missing one is written. No completed
    call slot is ever re-rolled."""
    os.makedirs(raw_dir, exist_ok=True)  # C1 of round-2 (the round-1 defect)
    prompt_path = os.path.join(raw_dir, f"{stem}.prompt.txt")
    if os.path.isfile(prompt_path):
        with open(prompt_path) as f:
            if f.read() != prompt:
                r2.stop(f"prompt drift at {prompt_path} (template/corpus "
                        f"changed since an earlier pass)")
    else:
        with open(prompt_path, "w") as f:
            f.write(prompt)
    ds_raw = os.path.join(raw_dir, f"{stem}-deepseek.raw.txt")
    ds_err = os.path.join(raw_dir, f"{stem}-deepseek.error.txt")
    muse_raw = os.path.join(raw_dir, f"{stem}-muse.raw.txt")
    muse_err = os.path.join(raw_dir, f"{stem}-muse.error.txt")

    def ds_fail_record(status_note, err_path):
        return {"call": 1, "provider": PRIMARY, "model": MODEL[PRIMARY],
                "status": status_note, "error_path": err_path,
                "started_at_utc": None, "duration_seconds": None}

    # (1) primary raw preserved -> reused, never re-rolled
    if os.path.isfile(ds_raw):
        with open(ds_raw) as f:
            raw = f.read()
        calls = [{"call": 1, "provider": PRIMARY, "model": MODEL[PRIMARY],
                  "status": "raw (preserved; never re-rolled)",
                  "raw_path": ds_raw, "started_at_utc": None,
                  "duration_seconds": None,
                  "duration_lost_in_interruption": True}]
        return {"calls": calls, "status": "raw", "provider_used": PRIMARY,
                "raw_path": ds_raw, "substituted": False, "raw_text": raw}
    # (2) primary failed AND its single muse substitution completed
    if os.path.isfile(muse_raw):
        if not os.path.isfile(ds_err):
            r2.stop(f"{stem}: muse raw without a recorded deepseek error "
                    f"(substitution-policy anomaly)")
        with open(muse_raw) as f:
            raw = f.read()
        calls = [ds_fail_record(
            "provider_failed (preserved; exact kind recorded at call time)",
            ds_err)]
        calls[-1]["duration_lost_in_interruption"] = True
        calls.append({"call": 2, "provider": SUB, "model": MODEL[SUB],
                      "status": "raw (preserved; never re-rolled)",
                      "raw_path": muse_raw, "started_at_utc": None,
                      "duration_seconds": None,
                      "duration_lost_in_interruption": True})
        return {"calls": calls, "status": "raw", "provider_used": SUB,
                "raw_path": muse_raw, "substituted": True, "raw_text": raw}
    # (3) both legs failed and are recorded -> typed result, no label
    if os.path.isfile(ds_err) and os.path.isfile(muse_err):
        calls = [ds_fail_record(
            "provider_failed (preserved; exact kind recorded at call time)",
            ds_err)]
        calls[-1]["duration_lost_in_interruption"] = True
        calls.append({"call": 2, "provider": SUB, "model": MODEL[SUB],
                      "status": "provider_failed (preserved; exact kind "
                                "recorded at call time)",
                      "error_path": muse_err, "started_at_utc": None,
                      "duration_seconds": None,
                      "duration_lost_in_interruption": True})
        return {"calls": calls, "status": "provider_failed",
                "provider_used": None, "raw_path": None,
                "substituted": True, "raw_text": None}
    # (4) primary failed and recorded; its single substitution was never
    # attempted (interruption) -> complete the policy exactly once
    if os.path.isfile(ds_err):
        calls = [ds_fail_record(
            "provider_failed (preserved; exact kind recorded at call time)",
            ds_err)]
        calls[-1]["duration_lost_in_interruption"] = True
        print(f"[arms3]   primary failure already recorded; completing the "
              f"one muse-spark-free substitution", flush=True)
        sub = r2.timed_call(SUB, prompt)
        calls.append(r2.call_record(2, sub))
        if sub["status"] == "raw":
            with open(muse_raw, "w") as f:
                f.write(sub["raw"])
            calls[-1]["raw_path"] = muse_raw
            return {"calls": calls, "status": "raw", "provider_used": SUB,
                    "raw_path": muse_raw, "substituted": True,
                    "raw_text": sub["raw"]}
        with open(muse_err, "w") as f:
            f.write(sub.get("raw", "") or "")
        calls[-1]["error_path"] = muse_err
        return {"calls": calls, "status": sub["status"],
                "provider_used": None, "raw_path": None,
                "substituted": True, "raw_text": None}
    # (5) fresh slot: primary call, then the frozen single-substitution policy
    resp = r2.timed_call(PRIMARY, prompt)
    calls = [r2.call_record(1, resp)]
    if resp["status"] != "raw":
        with open(ds_err, "w") as f:
            f.write(resp.get("raw", "") or "")
        calls[-1]["error_path"] = ds_err
        print(f"[arms3]   primary {resp['status']}: one "
              f"muse-spark-free substitution", flush=True)
        sub = r2.timed_call(SUB, prompt)
        calls.append(r2.call_record(2, sub))
        if sub["status"] == "raw":
            with open(muse_raw, "w") as f:
                f.write(sub["raw"])
            calls[-1]["raw_path"] = muse_raw
            return {"calls": calls, "status": "raw", "provider_used": SUB,
                    "raw_path": muse_raw, "substituted": True,
                    "raw_text": sub["raw"]}
        with open(muse_err, "w") as f:
            f.write(sub.get("raw", "") or "")
        calls[-1]["error_path"] = muse_err
        return {"calls": calls, "status": sub["status"],
                "provider_used": None, "raw_path": None,
                "substituted": True, "raw_text": None}
    with open(ds_raw, "w") as f:
        f.write(resp["raw"])
    calls[-1]["raw_path"] = ds_raw
    return {"calls": calls, "status": "raw", "provider_used": PRIMARY,
            "raw_path": ds_raw, "substituted": False, "raw_text": resp["raw"]}


def slot_status_from_calls(calls):
    """Scoring-consistent counters for one slot's call records (same
    semantics as the verification gate's payment recompute)."""
    d_calls = d_failed = d_sub = 0
    for cl in calls:
        if cl["provider"] == PRIMARY:
            if cl["status"].startswith("raw"):
                d_calls += 1
            else:
                d_failed += 1
        elif cl["provider"] == SUB and cl["status"].startswith("raw"):
            d_sub += 1
    return d_calls, d_failed, d_sub


def main() -> int:
    if os.environ.get("PYTHONDONTWRITEBYTECODE") != "1":
        r2.stop("export PYTHONDONTWRITEBYTECODE=1 before running")
    if r2.sha256_file(r2.PREREG_PATH) != r2.PREREG_SHA256:
        r2.stop("prereg sha256 mismatch")
    manifest_sha = r2.sha256_file(r2.MANIFEST_PATH)
    with open(r2.DECISIONS_PATH) as f:
        if "D-2026-09-19-RM-02" not in f.read():
            r2.stop("decision D-2026-09-19-RM-02 not found (phase-0 ordering)")
    if manifest_sha != "e2419c2ccd79cd84761354471a796141196a85b8ffe34322531339b74220be36":
        r2.stop("corpus manifest sha256 drifted from the RM-02 frozen digest")
    manifest = r2.load_yaml(r2.MANIFEST_PATH)
    rows_m = manifest["rows"]
    if len(rows_m) != 15:
        r2.stop(f"manifest has {len(rows_m)} rows, expected 15")
    for row in rows_m:  # fail-closed DB re-hash at resume start
        if r2.sha256_file(row["state_db_path"]) != row["db_sha256"]:
            r2.stop(f"{row['id']}: DB pin mismatch at resume start")

    with open(r2.RESULTS_PATH) as f:
        results = json.load(f)
    if len(results["rows"]) != 15 or \
            any("arm1" not in r for r in results["rows"]) or \
            results["arm_status"].get("arm1") != "run":
        r2.stop("results file is not in the post-arm1 carry-over state")
    if results["corpus_manifest"]["sha256"] != manifest_sha:
        r2.stop("results file pins a different corpus manifest sha256")

    # ---------- round-2 template pinning (re-validated, never changed) ----
    prefix, _, _ = r2.pin_round2_template()
    prefix2b = prefix.replace(r2.ARM2B_JSON_OLD,
                              r2.ARM2B_JSON_NEW + r2.ARM2B_NOTE)
    if results["templates"]["arm2"]["template_prefix_sha256"] \
            != r2.sha256_text(prefix) or \
            results["templates"]["arm2b"]["template_prefix_sha256"] \
            != r2.sha256_text(prefix2b):
        r2.stop("recorded round-2 template hashes do not reproduce")

    if not any(d["id"] == "DV-5" for d in results["deviations"]):
        results["deviations"].append(DV5)
    if not any(p.get("pass", "").startswith("round-3")
               for p in results["execution_passes"]):
        results["execution_passes"].append({
            "pass": "round-3 (run_arms.round3.py, resume of the interrupted "
                    "round-2 pass)",
            "arms_executed": ["arm2 (completion)", "arm2b"],
            "note": ("preserved arm2 raw outputs re-used verbatim, never "
                     "re-rolled (including the completed muse substitution "
                     "of nat-02-s2); the remaining arm2 calls and all arm2b "
                     "calls execute fresh under the frozen substitution "
                     "policy; results file updated in place (DV-5)")})
    def save():
        with open(r2.RESULTS_PATH, "w") as f:
            json.dump(results, f, indent=1, sort_keys=False)
            f.write("\n")

    def row_artifacts(row_m, kind):
        """Rebuild the row's surface, prompt and paths for arm2 (kind
        'arm2') or arm2b (kind 'arm2b', extended template)."""
        obs, run_meta = r2.resolve_obs(row_m)
        record = r2.bs.build_surface(obs, run_meta)
        if record.get("status") != "ok" or not record.get("surface"):
            r2.stop(f"{row_m['id']}: surface build failed: {record}")
        surface = record["surface"]
        if surface["settlement_committed_count"] != \
                row_m["actual_settled_units"]:
            r2.stop(f"{row_m['id']}: surface settlement count disagrees "
                    f"with manifest")
        blob = json.dumps(surface, sort_keys=True, indent=1)
        if kind == "arm2":
            prompt = prefix + r2.TEMPLATE_SPLIT + blob
            prompt_path = os.path.join(ARM2_RAW, f"arm2-{row_m['id']}.prompt.txt")
            surf_path = os.path.join(ARM2_SURF,
                                     f"{row_m['id']}.surface.json")
        else:
            prompt = prefix2b + r2.TEMPLATE_SPLIT + blob
            prompt_path = os.path.join(
                ARM2B_RAW, f"arm2b-{row_m['id']}.prompt.txt")
            surf_path = None
        return record, surface, blob, prompt, prompt_path, surf_path

    def ensure_prompt(prompt_path, prompt):
        if os.path.isfile(prompt_path):
            with open(prompt_path) as f:
                if f.read() != prompt:
                    r2.stop(f"prompt drift at {prompt_path}")
        else:
            with open(prompt_path, "w") as f:
                f.write(prompt)

    # ======================================================================
    # PHASE 2 (resumed) — arm2 completion
    # ======================================================================
    print("[arm2] resuming (complete preserved slots re-validated, never "
          "re-rolled)", flush=True)
    for i, row_m in enumerate(rows_m):
        rid = row_m["id"]
        existing = next(r for r in results["rows"] if r["id"] == rid)
        record, surface, blob, prompt, prompt_path, surf_path = \
            row_artifacts(row_m, "arm2")
        if "arm2" in existing:
            # completed row: validate the preserved block against disk
            a2 = existing["arm2"]
            if a2["surface_sha256"] != r2.sha256_text(blob) or \
                    a2["prompt_sha256"] != r2.sha256_text(prompt):
                r2.stop(f"{rid}: preserved arm2 block fails re-validation")
            ensure_prompt(prompt_path, prompt)
            labels = []
            for s in a2["samples"]:
                if s["raw_path"] is None or \
                        not os.path.isfile(s["raw_path"]):
                    r2.stop(f"{rid} sample {s['sample']}: preserved raw "
                            f"missing at resume")
                with open(s["raw_path"]) as f:
                    raw = f.read()
                parsed = r2.parse_arm_domain(raw, "label", r2.ARM2_DOMAIN)
                if not r2.parsed_equal(parsed, s["parsed"]):
                    r2.stop(f"{rid} sample {s['sample']}: preserved raw "
                            f"re-parse disagrees with the recorded record")
                labels.append(s["label"])
            maj, tally = r2.majority_of(labels)
            if maj != a2["majority_label"] or tally != a2["majority_tally"]:
                r2.stop(f"{rid}: preserved majority fails re-validation")
            print(f"[arm2] row {i + 1}/15 {rid}: validated "
                  f"(preserved, no calls)", flush=True)
            continue
        # incomplete row (nat-05) or fresh row: surface file + prompt
        if surf_path is not None:
            if os.path.isfile(surf_path):
                with open(surf_path) as f:
                    saved = json.load(f)
                if json.dumps(saved["surface"], sort_keys=True, indent=1) \
                        != blob:
                    r2.stop(f"{rid}: preserved surface file disagrees with "
                            f"the re-built surface")
            else:
                with open(surf_path, "w") as f:
                    json.dump({"schema": record["schema"],
                               "probe_id": record["probe_id"],
                               "correction_round": record["correction_round"],
                               "addendum2_sha256":
                                   record["addendum2_sha256"],
                               "status": record["status"],
                               "surface": surface,
                               "provenance": record["provenance"]}, f,
                              indent=1, sort_keys=True)
                    f.write("\n")
        ensure_prompt(prompt_path, prompt)
        samples = []
        for s in (1, 2, 3):
            pc = preserved_slot(ARM2_RAW, f"arm2-{rid}-s{s}", prompt)
            if pc["status"] == "raw":
                parsed = r2.parse_arm_domain(pc["raw_text"], "label",
                                             r2.ARM2_DOMAIN)
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
                "correct": (label == row_m["truth_class"]) if label else None})
            print(f"[arm2] row {i + 1}/15 sample {s}: label={label} "
                  f"truth={row_m['truth_class']} "
                  f"provider={pc['provider_used']}", flush=True)
        labels = [s["label"] for s in samples]
        maj, tally = r2.majority_of(labels)
        ok_labels = [l for l in labels if l]
        modal = max(tally.values()) if tally else 0
        existing["arm2"] = {
            "surface_path": surf_path,
            "surface_sha256": r2.sha256_text(blob),
            "prompt_path": prompt_path,
            "prompt_sha256": r2.sha256_text(prompt),
            "samples": samples,
            "labels_ok": ok_labels,
            "majority_label": maj,
            "majority_tally": tally,
            "majority_correct": (maj == row_m["truth_class"]),
            "single1_label": labels[0],
            "single1_correct": (labels[0] == row_m["truth_class"])
                               if labels[0] else None,
            "agreement_modal_of_cast": modal,
            "agreement_rate": modal / 3.0,
            "all_three_agree": (len(ok_labels) == 3
                                and len(set(ok_labels)) == 1),
        }
        save()
    results["arm_status"]["arm2"] = "run"
    save()

    # ---------------- arm2 scoring (frozen round-2 semantics) -------------
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
    s_single = r2.slice_scores([arm2_single_entry(r) for r in a2_deepseek])
    s_maj = r2.slice_scores([arm2_majority_entry(r) for r in a2_deepseek])
    route_rows = [r for r in results["rows"]
                  if r["historical_round2_label"] is not None]
    arm2_scoring = {
        "aggregation_rule": ("majority over the 3 samples; >=2 of 3 cast "
                             "votes needed; ties and rows with <2 parseable "
                             "samples -> insufficient_evidence (DV-2)"),
        "single_sample1_deepseek_slice": s_single,
        "majority_deepseek_slice": s_maj,
        "substituted_slice_never_pooled": {
            "n_rows": len(a2_sub),
            "row_ids": [r["id"] for r in a2_sub],
            "single_sample1": r2.slice_scores(
                [arm2_single_entry(r) for r in a2_sub]) if a2_sub else None,
            "majority": r2.slice_scores(
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
        "historical_freshness_route_rows": [
            {"id": r["id"], "probe_id": r["probe_id"],
             "truth": r["truth_class"],
             "fresh_majority_label": r["arm2"]["majority_label"],
             "fresh_labels": r["arm2"]["labels_ok"],
             "preserved_round2_label": r["historical_round2_label"],
             "fresh_matches_preserved": (
                 r["arm2"]["majority_label"]
                 == r["historical_round2_label"])}
            for r in route_rows],
        "historical_freshness_route_summary": {
            "n_route_rows": len(route_rows),
            "n_fresh_matches_preserved": sum(
                1 for r in route_rows
                if r["arm2"]["majority_label"]
                == r["historical_round2_label"]),
            "pooling_note": ("fresh labels are reported beside the preserved "
                             "round-2 labels; no pooling in either "
                             "direction")},
    }
    results["arm2_scoring"] = arm2_scoring
    save()
    print(f"[arm2] scoring: single acc={s_single['accuracy']} "
          f"majority acc={s_maj['accuracy']} "
          f"stability="
          f"{arm2_scoring['stability']['mean_agreement_rate_deepseek_rows']}",
          flush=True)

    # ======================================================================
    # PHASE 3 — arm2b (exploratory graded progress, 1 sample/row, DV-1)
    # ======================================================================
    print("[arm2b] begin (15 rows x 1 deepseek call, exploratory)",
          flush=True)
    for i, row_m in enumerate(rows_m):
        rid = row_m["id"]
        existing = next(r for r in results["rows"] if r["id"] == rid)
        if "arm2b" in existing:
            print(f"[arm2b] row {i + 1}/15 {rid}: already recorded",
                  flush=True)
            continue
        _, _, _, prompt, prompt_path, _ = row_artifacts(row_m, "arm2b")
        pc = preserved_slot(ARM2B_RAW, f"arm2b-{rid}", prompt)
        if pc["status"] == "raw":
            parsed = r2.parse_arm_domain(pc["raw_text"], "label",
                                         r2.ARM2_DOMAIN, with_estimate=True)
        else:
            parsed = {"status": pc["status"]}
        label = parsed.get("answer") if parsed.get("status") == "ok" else None
        est = parsed.get("settled_units_estimate") \
            if parsed.get("status") == "ok" else None
        est_status = parsed.get("estimate_status") \
            if parsed.get("status") == "ok" else pc["status"]
        abs_err = abs(est - row_m["actual_settled_units"]) \
            if est_status == "valid" else None
        existing["arm2b"] = {
            "prompt_path": prompt_path,
            "prompt_sha256": r2.sha256_text(prompt),
            "calls": pc["calls"],
            "provider_used": pc["provider_used"],
            "substituted": pc["substituted"],
            "status": parsed.get("status", pc["status"]),
            "raw_path": pc["raw_path"], "parsed": parsed,
            "label": label,
            "estimate": est, "estimate_status": est_status,
            "actual_settled_units": row_m["actual_settled_units"],
            "abs_error": abs_err,
        }
        save()
        print(f"[arm2b] row {i + 1}/15 {rid}: label={label} "
              f"estimate={est}({est_status}) "
              f"actual={row_m['actual_settled_units']}", flush=True)
    results["arm_status"]["arm2b"] = "run"
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
        lo, hi = r2.cp_ci(hits, n_valid)
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
    save()

    # ---------------- payment counters (recomputed from the records) ------
    def arm_counters(arm_key):
        d_calls = d_failed = d_sub = 0
        wall = 0.0
        for r in results["rows"]:
            if arm_key == "arm2":
                items = [s["calls"] for s in r["arm2"]["samples"]]
            else:
                items = [r[arm_key]["calls"]]
            for calls in items:
                dc, df, ds_ = slot_status_from_calls(calls)
                d_calls += dc
                d_failed += df
                d_sub += ds_
                for cl in calls:
                    if cl.get("duration_seconds"):
                        wall += cl["duration_seconds"]
        return {"deepseek_calls": d_calls, "deepseek_failed": d_failed,
                "muse_substitutions": d_sub}, round(wall, 3)

    pay = results.setdefault("payment", {})
    total_by_arm = {}
    for arm_key in ("arm1", "arm2", "arm2b"):
        counters, wall = arm_counters(arm_key)
        pay[arm_key] = counters
        total_by_arm[arm_key] = (counters["deepseek_calls"]
                                 + counters["deepseek_failed"]
                                 + counters["muse_substitutions"])
    pay["wall_seconds_by_arm"] = {
        arm_key: arm_counters(arm_key)[1]
        for arm_key in ("arm1", "arm2", "arm2b")}
    pay["total_calls_by_arm"] = total_by_arm
    pay["mechanical_layer_provider_calls"] = 0
    pay["learned_layer_total_provider_calls"] = sum(total_by_arm.values())
    pay["note"] = (
        "call counts include scored deepseek calls, recorded deepseek "
        "failures and muse substitutions; the mechanical consequence layer "
        "requires zero provider calls; arm2 wall time is a lower bound: the "
        "two completed nat-05 slots reconstructed at resume carry no "
        "duration (DV-5)")
    save()
    print(f"[arms3] complete: total provider calls="
          f"{pay['learned_layer_total_provider_calls']} "
          f"status={results['arm_status']}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
