#!/usr/bin/env python3
"""T23 round-2 re-judgment through the frozen provider ladder (addendum-2).

Re-judges each corrected judgment surface of route t23-route-001 through the
frozen ladder: primary deepseek-clinepass (`opencode run -m
cline-pass/cline-pass/deepseek-v4.1-flash`), ONE muse-spark substitution on
provider_error/provider_timeout/schema_failure, then a declared mechanical
fallback marked mechanics-only that assigns insufficient_evidence (an
abstention; never a fabricated provider result).

Honesty rules: ONE ladder pass per probe; every raw provider output is
preserved verbatim under round-2/provider-raw/ BEFORE parsing; the prompt is
neutral — it presents ONLY the corrected surface (never the route goal,
never round-1 labels, never the addendum's label-semantics wording, never
any narrative). Invalid or abstaining provider output resolves per the
frozen answer domain; a label of insufficient_evidence from a live provider
is a legitimate provider result.

Crash safety: if a raw output file for a probe/ladder tag already exists on
disk, it is reused (never re-rolled) — a resumed run cannot create a second
provider pass for the same probe. This script is frozen before its first
execution and never overwritten.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import json
import os
import re
import subprocess
import time
from datetime import datetime, timezone

T23_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BRANCH_DIR = os.path.dirname(T23_DIR)
T21_DIR = os.path.join(BRANCH_DIR, "T21")
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(BRANCH_DIR)))

sys.path.insert(0, T21_DIR)

import route_contracts as rc  # noqa: E402  (frozen T21 module, unmodified)

ROUND2_DIR = os.path.dirname(os.path.abspath(__file__))
SURFACES_DIR = os.path.join(ROUND2_DIR, "surfaces")
RAW_DIR = os.path.join(ROUND2_DIR, "provider-raw")

PRIMARY_PROVIDER = "deepseek-clinepass"
PRIMARY_MODEL = "cline-pass/cline-pass/deepseek-v4.1-flash"
SUB_PROVIDER = "muse-spark-free"
SUB_MODEL = "opencode/muse-spark-1.3-contributor-free"
PROVIDER_TIMEOUT_S = 300

ADDENDUM2_SHA256 = (
    "087887c27cc50c6c989fcf7bb214e77aca62ea74d7e32a868ef0f7dd211e4c5a")


def utcnow() -> str:
    return datetime.now(timezone.utc).isoformat()


def judgment_prompt(surface: dict) -> str:
    """Neutral round-2 prompt: ONLY the corrected surface is presented.

    Mirrors the round-1 template shape (field-class enumeration + answer
    domain + JSON shape). The addendum's label_semantics wording is
    deliberately excluded: telling the judge when to prefer a label would
    coach it. Use insufficient_evidence if the record cannot decide.
    """
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


def parse_judgment(text: str) -> tuple[str | None, str]:
    """Frozen round-1 parser (same semantics, re-stated here so round-1
    scripts stay untouched): first JSON object, `label` in the domain."""
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


def call_provider(provider: str, model: str, prompt: str) -> dict:
    cmd = ["opencode", "run", "-m", model, prompt]
    started = time.monotonic()
    started_at = utcnow()
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True,
                              timeout=PROVIDER_TIMEOUT_S, cwd=REPO_ROOT)
    except subprocess.TimeoutExpired:
        return {"status": "provider_timeout", "raw": "",
                "started_at_utc": started_at,
                "duration_seconds": round(time.monotonic() - started, 3)}
    duration = round(time.monotonic() - started, 3)
    if proc.returncode != 0:
        return {"status": "provider_error",
                "raw": (proc.stderr or proc.stdout)[-400:],
                "started_at_utc": started_at, "duration_seconds": duration}
    return {"status": "raw", "raw": proc.stdout,
            "started_at_utc": started_at, "duration_seconds": duration}


def main() -> int:
    os.makedirs(RAW_DIR, exist_ok=True)
    index = json.load(open(os.path.join(SURFACES_DIR,
                                        "corrected-surfaces.json")))
    records = index["records"]

    results: list[dict] = []
    path_records: list[dict] = []
    results_path = os.path.join(ROUND2_DIR, "rejudgment-results.json")

    for record in records:
        probe_id = record["probe_id"]
        surface = record["surface"]
        cid = surface["candidate_id"] if surface else None
        corr = surface["correlation_id"] if surface else None
        path_record: dict = {"probe_id": probe_id, "candidate_id": cid,
                             "attempts": []}

        if record["status"] != "ok" or surface is None:
            # DB pin mismatched at surface build: skipped, no provider call;
            # the re-judgment abstains with the failure recorded.
            label = "insufficient_evidence"
            path_record["attempts"].append({
                "attempt": "no-provider-call",
                "provider": "n/a", "model": "n/a",
                "outcome": ("skipped: state-database sha256 pin mismatch at "
                            "surface build; reasons: "
                            f"{record.get('reasons')}"),
                "raw_output_path": None})
            results.append({
                "probe_id": probe_id, "candidate_id": cid,
                "correlation_id": corr, "label": label,
                "judge_identity": "no-provider-call (skipped probe)",
                "judged_at_utc": utcnow(), "source": "skipped",
                "surface_path": os.path.join(
                    SURFACES_DIR, f"{probe_id}.json"),
                "attempts": path_record["attempts"]})
            path_records.append(path_record)
            with open(results_path, "w") as f:
                json.dump(results, f, indent=1, sort_keys=True)
                f.write("\n")
            continue

        prompt = judgment_prompt(surface)
        prompt_path = os.path.join(
            RAW_DIR, f"r2-judgment-{probe_id}-primary.prompt.txt")
        with open(prompt_path, "w") as f:
            f.write(prompt)

        label = None
        for tag, provider, model in (
                ("primary", PRIMARY_PROVIDER, PRIMARY_MODEL),
                ("substitution", SUB_PROVIDER, SUB_MODEL)):
            raw_path = os.path.join(
                RAW_DIR, f"r2-judgment-{probe_id}-{tag}.raw.txt")
            if os.path.isfile(raw_path):
                # resumed run: the preserved raw output of THIS probe's
                # ladder pass is reused; providers are never re-rolled
                with open(raw_path) as f:
                    resp = {"status": "raw", "raw": f.read(),
                            "started_at_utc": "(preserved from an earlier "
                                              "pass of this script)",
                            "duration_seconds": None}
            else:
                resp = call_provider(provider, model, prompt)
                with open(raw_path, "w") as f:
                    f.write(resp["raw"])
            if resp["status"] != "raw":
                path_record["attempts"].append({
                    "attempt": tag, "provider": provider, "model": model,
                    "outcome": resp["status"],
                    "raw_output_path": raw_path,
                    "started_at_utc": resp["started_at_utc"],
                    "duration_seconds": resp["duration_seconds"]})
                print(f"[rejudge] {probe_id} {tag} {provider}: "
                      f"{resp['status']}", flush=True)
                continue
            parsed, err = parse_judgment(resp["raw"])
            outcome = "ok" if parsed is not None else f"schema_failure: {err}"
            path_record["attempts"].append({
                "attempt": tag, "provider": provider, "model": model,
                "outcome": outcome, "raw_output_path": raw_path,
                "started_at_utc": resp["started_at_utc"],
                "duration_seconds": resp["duration_seconds"],
                "abstention": (parsed == "insufficient_evidence"
                               if parsed is not None else None)})
            print(f"[rejudge] {probe_id} {tag} {provider}: {outcome} "
                  f"label={parsed}", flush=True)
            if parsed is not None:
                label = parsed
                break

        if label is None:
            label = "insufficient_evidence"
            path_record["attempts"].append({
                "attempt": "mechanical-fallback",
                "provider": "abstention-fallback-v1 (mechanics-only)",
                "model": "n/a",
                "outcome": ("both ladder attempts failed; the mechanics-only "
                            "fallback abstains with insufficient_evidence; "
                            "never counted as a provider result")})
            judge_identity = ("abstention-fallback-v1 (mechanics-only)")
            source = "mechanical-fallback-abstention"
        else:
            winning = path_record["attempts"][-1]
            judge_identity = f"{winning['provider']}:{winning['model']}"
            source = "provider"

        results.append({
            "probe_id": probe_id, "candidate_id": cid,
            "correlation_id": corr, "label": label,
            "judge_identity": judge_identity,
            "judged_at_utc": utcnow(), "source": source,
            "observation_refs": [probe_id],
            "surface_path": os.path.join(SURFACES_DIR, f"{probe_id}.json"),
            "attempts": path_record["attempts"]})
        path_records.append(path_record)
        with open(results_path, "w") as f:
            json.dump(results, f, indent=1, sort_keys=True)
            f.write("\n")

    with open(os.path.join(ROUND2_DIR, "rejudgment-record.yaml"), "w") as f:
        f.write("# T23 round-2 re-judgment record (addendum-2)\n")
        import yaml
        yaml.safe_dump({
            "schema": "godspeed-bounded-judgment.t23-round2-rejudgment-record",
            "route_id": "t23-route-001",
            "correction_round": 2,
            "addendum2_sha256": ADDENDUM2_SHA256,
            "judge_input_rule": "corrected judgment surfaces only (typed "
                                "terminal; settlement count + unit "
                                "progression; unit_dispatch retry pattern; "
                                "produced evidence artifact contents by "
                                "sha256 pin; timing); never the route goal, "
                                "never round-1 labels, never the addendum "
                                "label-semantics wording, never any "
                                "narrative",
            "ladder": {"primary": f"{PRIMARY_PROVIDER}:{PRIMARY_MODEL}",
                       "substitution": f"{SUB_PROVIDER}:{SUB_MODEL}",
                       "fallback": "abstention-fallback-v1 "
                                   "(mechanics-only, insufficient_evidence)"},
            "one_pass_rule": "one ladder pass per probe; raw outputs "
                             "preserved verbatim before parsing; resumed "
                             "runs reuse preserved raws and never re-roll",
            "probe_judgments": path_records,
        }, f, sort_keys=False, default_flow_style=False)

    labels = {r["probe_id"]: r["label"] for r in results}
    print("[rejudge] labels:", json.dumps(labels, indent=1), flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
