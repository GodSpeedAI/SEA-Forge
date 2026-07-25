#!/usr/bin/env sh
set -eu

SCRIPT=$(cd "$(dirname "$0")/.." && pwd)/check-agent-context.sh
TMP_ROOT=${TMPDIR:-/tmp}/sea-forge-context-test-$$
trap 'rm -rf "$TMP_ROOT"' EXIT HUP INT TERM

make_repo() {
  repo=$1
  mkdir -p "$repo/.agents"
  git -C "$repo" init -q
  git -C "$repo" config user.email test@example.invalid
  git -C "$repo" config user.name "Context Test"
  cat >"$repo/.agents/CURRENT_STATUS.md" <<'EOF'
# Current Status
Updated: 2026-07-10
## Objective
Test the context gate.
## Worktree State
Clean at handoff.
## Changed Files
- `.agents/CURRENT_STATUS.md`
## Completed
- Context recorded.
## Verification
- Test fixture created.
## Remaining
- None.
## Blockers
None.
## Decisions
- Keep the gate portable.
EOF
  printf '# Agent rules\n' >"$repo/AGENTS.md"
  git -C "$repo" add .
  git -C "$repo" commit -qm baseline
}

expect_pass() {
  name=$1
  shift
  if ! "$@" >"$TMP_ROOT/output" 2>&1; then
    printf 'FAIL: %s should pass\n' "$name" >&2
    cat "$TMP_ROOT/output" >&2
    exit 1
  fi
}

expect_fail() {
  name=$1
  shift
  if "$@" >"$TMP_ROOT/output" 2>&1; then
    printf 'FAIL: %s should fail\n' "$name" >&2
    exit 1
  fi
}

mkdir -p "$TMP_ROOT"

repo="$TMP_ROOT/valid"
make_repo "$repo"
expect_pass "complete handoff" sh -c "cd '$repo' && '$SCRIPT'"

repo="$TMP_ROOT/missing-section"
make_repo "$repo"
grep -v '^## Changed Files$' "$repo/.agents/CURRENT_STATUS.md" >"$repo/status.tmp"
mv "$repo/status.tmp" "$repo/.agents/CURRENT_STATUS.md"
expect_fail "missing required section" sh -c "cd '$repo' && '$SCRIPT'"

repo="$TMP_ROOT/stale-dirty"
make_repo "$repo"
printf 'changed\n' >>"$repo/AGENTS.md"
expect_fail "project change without handoff update" sh -c "cd '$repo' && '$SCRIPT'"

repo="$TMP_ROOT/fresh-dirty"
make_repo "$repo"
printf 'changed\n' >>"$repo/AGENTS.md"
printf '\nWork in progress.\n' >>"$repo/.agents/CURRENT_STATUS.md"
expect_pass "project and handoff changed together" sh -c "cd '$repo' && '$SCRIPT'"

repo="$TMP_ROOT/fresh-committed"
make_repo "$repo"
printf 'changed\n' >>"$repo/AGENTS.md"
printf '\nCommitted handoff.\n' >>"$repo/.agents/CURRENT_STATUS.md"
git -C "$repo" add .
git -C "$repo" commit -qm change
expect_pass "committed change includes handoff" sh -c "cd '$repo' && CONTEXT_BASE_REF=HEAD^ '$SCRIPT'"

repo="$TMP_ROOT/stale-committed"
make_repo "$repo"
printf 'changed\n' >>"$repo/AGENTS.md"
git -C "$repo" add .
git -C "$repo" commit -qm change
expect_fail "committed change omits handoff" sh -c "cd '$repo' && CONTEXT_BASE_REF=HEAD^ '$SCRIPT'"

printf 'context gate tests passed\n'
