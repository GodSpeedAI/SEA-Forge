"""Capture existing regressions separately from the deliberately RED acceptance suite."""

import argparse
import datetime
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--server-only", action="store_true", help="resume server baseline after fixing fixture prerequisites")
    parser.add_argument("--required-gates", action="store_true", help="capture canonical workspace tests and minimum proofs")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False, mode=0o700)
    # Existing M16 server regressions require the real built sea-forge CLI.
    if args.required_gates:
        commands = [["just", "test"], ["just", "proof"]]
    else:
        commands = [["just", "build"]]
        if not args.server_only:
            commands.extend([["just", "crate-test", "sea-forge-" + crate]
                             for crate in ("authority", "runtime", "sandbox", "ledger", "settlement")])
        # Only new acceptance tests are excluded. Existing regression assertions,
        # including all original identity and settlement tests, are unchanged.
        commands.append(["cargo", "test", "-p", "sea-forge-server", "--locked", "--", "--skip", "security_v1"])
    records = []
    for index, command in enumerate(commands):
        try:
            completed = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=900)
            output = completed.stdout + "\n" + completed.stderr
            exit_code = completed.returncode
        except (OSError, subprocess.SubprocessError) as error:
            output, exit_code = str(error), None
        name = f"{index:02}-regression.log"
        (args.output / name).write_text(output)
        records.append({"command": command, "exit_code": exit_code, "log": name,
                        "sha256": hashlib.sha256(output.encode()).hexdigest()})
        print(" ".join(command) + ": " + str(exit_code), flush=True)
    (args.output / "results.json").write_text(json.dumps({
        "recorded_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "commands": records}, indent=2) + "\n")
    return 0 if all(record["exit_code"] == 0 for record in records) else 1


if __name__ == "__main__":
    sys.exit(main())
