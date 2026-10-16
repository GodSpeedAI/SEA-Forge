"""Capture real verification commands; never reinterpret their exit status."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument("output")
parser.add_argument("command", nargs=argparse.REMAINDER)
args = parser.parse_args()
command = args.command[1:] if args.command[:1] == ["--"] else args.command
if not command:
    parser.error("a command is required")
repo = Path(__file__).resolve().parents[2]
output = repo / args.output
if ".." in output.parts:
    parser.error("traversal is forbidden")
output.relative_to(repo / ".agents/evidence/security-v2")
for ancestor in [output, *output.parents]:
    if ancestor.is_symlink():
        parser.error("symlinked evidence path")
output.mkdir(parents=True, mode=0o700, exist_ok=False)
started = datetime.now(timezone.utc).isoformat()
with (output / "command.log").open("xb") as log:
    result = subprocess.run(command, cwd=repo, env=os.environ.copy(), stdout=log, stderr=subprocess.STDOUT, check=False)
record = {
    "command": command, "started_at": started,
    "finished_at": datetime.now(timezone.utc).isoformat(),
    "exit_code": result.returncode,
    "log_sha256": hashlib.sha256((output / "command.log").read_bytes()).hexdigest(),
    "source_head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip(),
}
(output / "result.json").write_text(json.dumps(record, indent=2) + "\n")
print(f"{output.relative_to(repo)}/result.json: exit {result.returncode}")
raise SystemExit(result.returncode)
