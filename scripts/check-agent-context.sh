#!/usr/bin/env sh
set -eu

STATUS_FILE=.agents/CURRENT_STATUS.md
REQUIRED_SECTIONS='Objective
Worktree State
Changed Files
Completed
Verification
Remaining
Blockers
Decisions'

fail() {
  printf 'context_error: %s\n' "$1" >&2
  printf 'next move: update %s, then rerun scripts/check-agent-context.sh\n' "$STATUS_FILE" >&2
  exit 1
}

[ -f AGENTS.md ] || fail 'AGENTS.md is missing'
[ -f "$STATUS_FILE" ] || fail "$STATUS_FILE is missing"

if ! grep -Eq '^Updated: [0-9]{4}-[0-9]{2}-[0-9]{2}$' "$STATUS_FILE"; then
  fail "$STATUS_FILE needs an Updated: YYYY-MM-DD line"
fi

printf '%s\n' "$REQUIRED_SECTIONS" | while IFS= read -r section; do
  grep -Fqx "## $section" "$STATUS_FILE" || fail "$STATUS_FILE is missing the '$section' section"
done

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
  } | sort -u | grep -Ev '^(\.agents/CURRENT_STATUS\.md|target/|\.logs/)$' || true
)

status_changed=$(git status --porcelain -- "$STATUS_FILE")
if printf '%s\n' "$base_changes" | grep -Fqx "$STATUS_FILE"; then
  status_changed=committed
fi

if [ -n "$project_changes" ] && [ -z "$status_changed" ]; then
  fail 'project files changed without a corresponding CURRENT_STATUS.md update'
fi

printf 'context check passed\n'
