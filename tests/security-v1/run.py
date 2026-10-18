"""Run affordable frozen gates, keeping failures and coverage gaps visibly non-green."""

import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import uuid

from mcp_probe import probe as mcp_probe
from postgres_probe import probe as postgres_probe

ROOT = Path(__file__).resolve().parents[2]
INVENTORY = json.loads((Path(__file__).parent / "inventory.json").read_text())


def collect_rust(gate, output):
    # The existing crate-test recipe has no argument for --nocapture or a test
    # binary. Use Cargo for this missing recipe; existing regression gates remain required.
    command = ["cargo", "test", "-p", "sea-forge-server", "--locked", "--test",
               "security_acceptance_v1", "security_v1_" + gate.lower(), "--", "--nocapture"]
    try:
        completed = subprocess.run(command, cwd=ROOT,
                                   capture_output=True, text=True, timeout=900)
    except (OSError, subprocess.SubprocessError) as error:
        return [], {"command": command, "reason": type(error).__name__ + ": " + str(error)}
    log = completed.stdout + "\n" + completed.stderr
    (output / ("rust-" + gate + ".log")).write_text(log)
    rows = []
    for line in log.splitlines():
        if "SECURITY_V1 {" in line:
            try:
                rows.append(json.loads(line.split("SECURITY_V1 ", 1)[1]))
            except ValueError:
                pass
    return rows, {"command": command, "exit_code": completed.returncode,
                  "log_sha256": hashlib.sha256(log.encode()).hexdigest()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("gate", choices=["A", "B", "C", "all"])
    parser.add_argument("--output", required=True, type=Path,
                        help="new observer-owned directory outside attacker workspace; existing paths refused")
    args = parser.parse_args()
    output_path = args.output.absolute()
    observer_root = ROOT / ".agents/evidence/security-v1"
    if ".." in args.output.parts or not output_path.is_relative_to(observer_root):
        parser.error("observer output must be under .agents/evidence/security-v1, outside all disposable agent workspaces")
    if any(parent.is_symlink() for parent in [output_path, *output_path.parents]):
        parser.error("observer output and its parents must not be symlinks")
    args.output.mkdir(parents=True, exist_ok=False, mode=0o700)
    run_id = str(uuid.uuid4())
    results, commands = [], []
    for gate in ("ABC" if args.gate == "all" else args.gate):
        cases = [case for case in INVENTORY["cases"] if case["gate"] == gate]
        measured = []
        if any(case["probe"] == "rust" for case in cases):
            measured, command = collect_rust(gate, args.output)
            commands.append(command)
        if gate == "A":
            measured.extend(mcp_probe())
            measured.append(postgres_probe())
        indexed = {row["case_id"]: row for row in measured}
        for case in cases:
            row = indexed.get(case["id"])
            if row is None:
                gap = case["probe"] == "gap"
                row = {"case_id": case["id"], "classification": "NOT-YET-EXECUTABLE" if gap else "INFRASTRUCTURE-BLOCKED",
                       "attempted": False, "reason": case.get("reason", "probe did not produce complete evidence; inspect command log"),
                       "positive_control": False, "cleanup": "no completed probe evidence"}
            row.update(gate=gate, observer_run_id=run_id)
            row.setdefault("trusted_identity", "not observed")
            row.setdefault("claimed_identity", "not observed")
            row.setdefault("action_id", case["id"])
            row.setdefault("decision", "not observed")
            row.setdefault("observed_effect", "not observed; see per-attempt observations when present")
            row.setdefault("run_id", run_id)
            results.append(row)
    report = {"suite": INVENTORY["suite"], "observer_run_id": run_id,
              "recorded_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
              "baseline_commit": INVENTORY["baseline_commit"], "commands": commands, "results": results,
              "observer_directory":{"uid":args.output.stat().st_uid,
                  "mode":oct(args.output.stat().st_mode & 0o777),
                  "scope_limit":"trusted test driver; mode bits do not isolate arbitrary same-UID host code"}}
    (args.output / "results.json").write_text(json.dumps(report, indent=2) + "\n")
    for row in results:
        print(row["case_id"] + ": " + row["classification"])
    print("Evidence: " + str(args.output / "results.json"))
    # Coverage blockers are never an exit-zero gate. Rust failures with a partial
    # emitted report also prevent success, even if other cases happened to pass.
    green = all(row["classification"] == "PASS" for row in results) and all(
        command.get("exit_code") == 0 for command in commands)
    return 0 if green else 1


if __name__ == "__main__":
    sys.exit(main())
