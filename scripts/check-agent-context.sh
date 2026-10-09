#!/usr/bin/env sh
set -eu

STATUS_YAML=.agents/CURRENT_STATUS.yaml
STATUS_MD=.agents/CURRENT_STATUS.md

fail() {
  printf 'context_error: %s\n' "$1" >&2
  printf 'next move: update %s and %s, then rerun scripts/check-agent-context.sh\n' "$STATUS_YAML" "$STATUS_MD" >&2
  exit 1
}

[ -f AGENTS.md ] || fail 'AGENTS.md is missing'
[ -f .agents/AGENTS.md ] || fail '.agents/AGENTS.md is missing'
[ -f "$STATUS_YAML" ] || fail "$STATUS_YAML is missing"
[ -f "$STATUS_MD" ] || fail "$STATUS_MD is missing"

validate_output=$(python3 - "$STATUS_YAML" "$STATUS_MD" <<'PY' 2>&1 || true
import sys, json
from datetime import datetime

yaml_path = sys.argv[1]
md_path = sys.argv[2]

try:
    with open(yaml_path, "r", encoding="utf-8") as f:
        yaml_text = f.read()
except Exception as e:
    print(f"Failed to read {yaml_path}: {e}")
    sys.exit(1)

if not yaml_text.endswith("\n"):
    print("CURRENT_STATUS.yaml must end with a newline")
    sys.exit(1)

lines = yaml_text.rstrip("\n").split("\n")
if len(lines) == 0 or len(lines) % 2 != 0:
    print(f"CURRENT_STATUS.yaml must contain complete two-line documents (separated by '---'), got {len(lines)} lines")
    sys.exit(1)

prev_rev = 0
last_snapshot = None

required_fields = ["revision", "recorded_at", "stage", "summary", "verified", "limits", "next", "evidence", "spec", "ledger"]

for i in range(0, len(lines), 2):
    sep = lines[i]
    content = lines[i + 1]
    doc_num = i // 2 + 1
    if sep != "---":
        print(f"Status document {doc_num}: missing '---' separator")
        sys.exit(1)
    try:
        data = json.loads(content)
    except Exception as e:
        print(f"Status document {doc_num}: invalid one-line JSON-compatible YAML: {e}")
        sys.exit(1)
    if not isinstance(data, dict):
        print(f"Status document {doc_num}: entry must be a JSON object")
        sys.exit(1)
    for field in required_fields:
        if field not in data:
            print(f"Status document {doc_num}: missing required field '{field}'")
            sys.exit(1)
    rev = data["revision"]
    if not isinstance(rev, int) or isinstance(rev, bool) or rev <= prev_rev:
        print(f"Status document {doc_num}: revision must be integer strictly greater than {prev_rev}, got {rev}")
        sys.exit(1)
    prev_rev = rev

    rec = data["recorded_at"]
    if not isinstance(rec, str):
        print(f"Status document {doc_num}: recorded_at must be a string")
        sys.exit(1)
    try:
        datetime.fromisoformat(rec.replace("Z", "+00:00"))
    except Exception as e:
        print(f"Status document {doc_num}: invalid recorded_at timestamp '{rec}': {e}")
        sys.exit(1)

    for list_field in ["verified", "limits", "evidence"]:
        if not isinstance(data[list_field], list):
            print(f"Status document {doc_num}: '{list_field}' must be a list")
            sys.exit(1)

    for str_field in ["stage", "summary", "next", "spec", "ledger"]:
        if not isinstance(data[str_field], str) or not data[str_field].strip():
            print(f"Status document {doc_num}: '{str_field}' must be a non-empty string")
            sys.exit(1)

    last_snapshot = data

try:
    with open(md_path, "r", encoding="utf-8") as f:
        md_text = f.read()
except Exception as e:
    print(f"Failed to read {md_path}: {e}")
    sys.exit(1)

rev_marker = f"**Status revision:** {last_snapshot['revision']}"
summary_marker = f"**Summary:** {last_snapshot['summary']}"

if rev_marker not in md_text:
    print(f"CURRENT_STATUS.md does not match latest revision: expected '{rev_marker}'")
    sys.exit(1)

if summary_marker not in md_text:
    print(f"CURRENT_STATUS.md does not match latest summary: expected '{summary_marker}'")
    sys.exit(1)

PY
)

if [ -n "$validate_output" ]; then
  fail "$validate_output"
fi

base_ref=${CONTEXT_BASE_REF:-}
if [ -z "$base_ref" ] && [ "${CI:-}" = "true" ] && git rev-parse --verify HEAD^ >/dev/null 2>&1; then
  base_ref=HEAD^
fi

base_changes=
if [ -n "$base_ref" ] && git rev-parse --verify "$base_ref" >/dev/null 2>&1; then
  base_changes=$(git diff --name-only "$base_ref"...HEAD)
fi

project_changes=$(
  {
    git diff --name-only
    git diff --cached --name-only
    git ls-files --others --exclude-standard
    printf '%s\n' "$base_changes"
  } | sort -u | grep -Ev '^(\.agents/CURRENT_STATUS\.(yaml|md)|target|\.logs)(/|$)' || true
)

yaml_changed=$(git status --porcelain -- "$STATUS_YAML")
if printf '%s\n' "$base_changes" | grep -Fqx "$STATUS_YAML"; then
  yaml_changed=committed
fi

md_changed=$(git status --porcelain -- "$STATUS_MD")
if printf '%s\n' "$base_changes" | grep -Fqx "$STATUS_MD"; then
  md_changed=committed
fi

if [ -n "$project_changes" ] && { [ -z "$yaml_changed" ] || [ -z "$md_changed" ]; }; then
  fail 'project files changed without corresponding CURRENT_STATUS.yaml and CURRENT_STATUS.md updates'
fi

printf 'status check passed\n'
