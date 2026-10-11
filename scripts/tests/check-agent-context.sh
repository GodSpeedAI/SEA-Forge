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
  printf '# Agent rules\n' >"$repo/AGENTS.md"
  printf '# Workbench rules\n' >"$repo/.agents/AGENTS.md"
  cat >"$repo/.agents/CURRENT_STATUS.yaml" <<'EOF'
---
{"revision": 1, "recorded_at": "2026-10-08T18:00:00Z", "stage": "Testing stage", "summary": "Baseline test status.", "verified": ["unit tests pass"], "limits": ["none"], "next": "Continue testing", "evidence": [".agents/evidence/test.md"], "spec": "specs/test-spec.md", "ledger": ".agents/DEBT.md"}
EOF
  cat >"$repo/.agents/CURRENT_STATUS.md" <<'EOF'
# Current status

**Status revision:** 1

**Stage:** Testing stage

**Summary:** Baseline test status.

**Verified:**
- unit tests pass

**Limits:**
- none

**Next:** Continue testing

**Evidence:**
- .agents/evidence/test.md
EOF
  git -C "$repo" add .
  git -C "$repo" commit -qm baseline
}

append_status() {
  repo=$1
  rev=$2
  summary=$3
  cat >>"$repo/.agents/CURRENT_STATUS.yaml" <<EOF
---
{"revision": $rev, "recorded_at": "2026-10-08T18:10:00Z", "stage": "Testing stage", "summary": "$summary", "verified": ["all passed"], "limits": ["none"], "next": "Next move", "evidence": [".agents/evidence/test.md"], "spec": "specs/test-spec.md", "ledger": ".agents/DEBT.md"}
EOF
  cat >"$repo/.agents/CURRENT_STATUS.md" <<EOF
# Current status

**Status revision:** $rev

**Stage:** Testing stage

**Summary:** $summary

**Verified:**
- all passed

**Limits:**
- none

**Next:** Next move

**Evidence:**
- .agents/evidence/test.md
EOF
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

repo="$TMP_ROOT/missing-yaml"
make_repo "$repo"
rm "$repo/.agents/CURRENT_STATUS.yaml"
expect_fail "missing CURRENT_STATUS.yaml" sh -c "cd '$repo' && '$SCRIPT'"

repo="$TMP_ROOT/missing-md"
make_repo "$repo"
rm "$repo/.agents/CURRENT_STATUS.md"
expect_fail "missing CURRENT_STATUS.md" sh -c "cd '$repo' && '$SCRIPT'"

repo="$TMP_ROOT/malformed-yaml-json"
make_repo "$repo"
cat >>"$repo/.agents/CURRENT_STATUS.yaml" <<'EOF'
---
{"revision": 2, invalid json
EOF
expect_fail "malformed json in yaml" sh -c "cd '$repo' && '$SCRIPT'"

repo="$TMP_ROOT/non-monotonic-revision"
make_repo "$repo"
append_status "$repo" 1 "Duplicate revision."
expect_fail "non-monotonic revision" sh -c "cd '$repo' && '$SCRIPT'"

repo="$TMP_ROOT/mismatched-human-view"
make_repo "$repo"
sed -i 's/\*\*Status revision:\*\* 1/\*\*Status revision:\*\* 99/' "$repo/.agents/CURRENT_STATUS.md"
expect_fail "mismatched human view" sh -c "cd '$repo' && '$SCRIPT'"

repo="$TMP_ROOT/stale-dirty"
make_repo "$repo"
printf 'changed\n' >>"$repo/AGENTS.md"
expect_fail "project change without handoff update" sh -c "cd '$repo' && '$SCRIPT'"

repo="$TMP_ROOT/partial-dirty"
make_repo "$repo"
printf 'changed\n' >>"$repo/AGENTS.md"
cat >>"$repo/.agents/CURRENT_STATUS.yaml" <<'EOF'
---
{"revision": 2, "recorded_at": "2026-10-08T18:10:00Z", "stage": "Testing stage", "summary": "Only yaml updated.", "verified": ["all passed"], "limits": ["none"], "next": "Next move", "evidence": [".agents/evidence/test.md"], "spec": "specs/test-spec.md", "ledger": ".agents/DEBT.md"}
EOF
expect_fail "project change with only yaml updated" sh -c "cd '$repo' && '$SCRIPT'"

repo="$TMP_ROOT/fresh-dirty"
make_repo "$repo"
printf 'changed\n' >>"$repo/AGENTS.md"
append_status "$repo" 2 "Work in progress."
expect_pass "project and both handoff files changed together" sh -c "cd '$repo' && '$SCRIPT'"

repo="$TMP_ROOT/fresh-committed"
make_repo "$repo"
printf 'changed\n' >>"$repo/AGENTS.md"
append_status "$repo" 2 "Committed handoff."
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
