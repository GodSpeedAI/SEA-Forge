#!/usr/bin/env bash
# Staged-file secret scanning canaries (SEC round 3).
#
# Runs gitleaks with this repository's configuration against a throwaway
# repository holding synthetic tokens generated at run time (never stored in
# source), and reports which states each scan mode catches:
#   staged new file, staged modification, unstaged-only change, staged then
#   reverted in the worktree, committed, allowlisted path, path adjacent to
#   an allowlist, clean negative control. Every report must be redacted.
#
# Usage: scripts/security/staged-secret-canaries.sh [OUT_DIR]
# Exits non-zero when an expectation fails. Prints one line per case.
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
out="${1:-$(mktemp -d)}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

token() { # A GitHub-PAT-shaped synthetic token, random per run.
  printf 'ghp_%s' "$(LC_ALL=C tr -dc 'A-Za-z0-9' </dev/urandom | head -c 36)"
}

fresh() {
  rm -rf "$work/r"
  mkdir -p "$work/r"
  git -C "$work/r" init -q -b main
  git -C "$work/r" config user.email canary@example.invalid
  git -C "$work/r" config user.name canary
  git -C "$work/r" config commit.gpgsign false
  cp "$repo_root/.gitleaks.toml" "$work/r/.gitleaks.toml"
  printf 'seed\n' >"$work/r/README"
  git -C "$work/r" add -A
  git -C "$work/r" commit -q -m seed
}

# scan MODE CASE -> prints leak count; keeps the redacted report.
scan() {
  local mode=$1 name=$2 report="$out/$2.$1.json" status=0
  case $mode in
  staged) (cd "$work/r" && gitleaks git --staged --no-banner --redact --exit-code 1 \
    --report-format json --report-path "$report" . >"$out/$name.$mode.log" 2>&1) || status=$? ;;
  history) (cd "$work/r" && gitleaks git --no-banner --redact --exit-code 1 \
    --report-format json --report-path "$report" . >"$out/$name.$mode.log" 2>&1) || status=$? ;;
  esac
  if [ "$status" -gt 1 ]; then
    echo "gitleaks error in $name/$mode" >&2
    cat "$out/$name.$mode.log" >&2
    exit 2
  fi
  python3 -c 'import json,sys; print(len(json.load(open(sys.argv[1]))))' "$report"
}

failures=0
expect() { # CASE MODE EXPECTED ACTUAL
  local verdict=ok
  [ "$3" = "$4" ] || { verdict=UNEXPECTED; failures=$((failures + 1)); }
  printf '%-34s %-8s expected=%s found=%s %s\n' "$1" "$2" "$3" "$4" "$verdict"
}

secrets=()
remember() { secrets+=("$1"); }

# 1. Staged new file.
fresh; t=$(token); remember "$t"
printf 'token = "%s"\n' "$t" >"$work/r/new.txt"; git -C "$work/r" add new.txt
expect staged_new_file staged 1 "$(scan staged staged_new_file)"

# 2. Staged modification of a tracked file.
fresh; t=$(token); remember "$t"
printf 'token = "%s"\n' "$t" >>"$work/r/README"; git -C "$work/r" add README
expect staged_modification staged 1 "$(scan staged staged_modification)"

# 3. Unstaged-only change: never enters the commit.
fresh; t=$(token); remember "$t"
printf 'token = "%s"\n' "$t" >>"$work/r/README"
expect unstaged_only staged 0 "$(scan staged unstaged_only)"

# 4. Staged, then reverted in the worktree: the index still carries it.
fresh; t=$(token); remember "$t"
printf 'token = "%s"\n' "$t" >"$work/r/hidden.txt"; git -C "$work/r" add hidden.txt
printf 'clean\n' >"$work/r/hidden.txt"
expect staged_then_worktree_reverted staged 1 "$(scan staged staged_then_worktree_reverted)"

# 5. Committed: invisible to a staged scan, caught by the history scan.
fresh; t=$(token); remember "$t"
printf 'token = "%s"\n' "$t" >"$work/r/committed.txt"; git -C "$work/r" add committed.txt
git -C "$work/r" commit -q --no-verify -m committed
expect committed staged 0 "$(scan staged committed)"
expect committed history 1 "$(scan history committed)"

# 6. Allowlisted path: a NEW secret under an allowlisted pattern.
fresh; t=$(token); remember "$t"
mkdir -p "$work/r/123"
printf '{"t":"%s"}\n' "$t" >"$work/r/123/full.jsonl"; git -C "$work/r" add 123/full.jsonl
expect allowlisted_numbered_transcript staged "${EXPECT_NUMBERED:-1}" "$(scan staged allowlisted_numbered_transcript)"
fresh; t=$(token); remember "$t"
mkdir -p "$work/r/src/x.entire/metadata"
printf 'token = "%s"\n' "$t" >"$work/r/src/x.entire/metadata/a.txt"
git -C "$work/r" add src
expect allowlisted_unanchored_entire staged "${EXPECT_UNANCHORED:-1}" "$(scan staged allowlisted_unanchored_entire)"

# 6b. A path listed in a commit-scoped allowlist: a new secret there.
fresh; t=$(token); remember "$t"
mkdir -p "$work/r/crates/sea-forge-agent/src"
printf 'const T: &str = "%s";\n' "$t" >"$work/r/crates/sea-forge-agent/src/delegation.rs"
git -C "$work/r" add crates
expect commit_scoped_path_new_secret staged "${EXPECT_COMMIT_SCOPED:-1}" "$(scan staged commit_scoped_path_new_secret)"

# 6c. Casework evidence path: a new credential-shaped secret there.
fresh; t=$(token); remember "$t"
mkdir -p "$work/r/.agents/evidence/casework-live-wiring/T10"
printf 'token = "%s"\n' "$t" >"$work/r/.agents/evidence/casework-live-wiring/T10/new.txt"
git -C "$work/r" add .agents
expect casework_path_new_pat staged "${EXPECT_CASEWORK:-1}" "$(scan staged casework_path_new_pat)"

# 6d. A value-scoped evidence path: only its exact listed values are exempt.
fresh; t=$(token); remember "$t"
mkdir -p "$work/r/.agents/evidence/security-v1/continuation-20261017/all"
printf '{"idempotency_key": "%s"}\n' "$t" >"$work/r/.agents/evidence/security-v1/continuation-20261017/all/results.json"
git -C "$work/r" add .agents
expect value_scoped_path_new_value staged 1 "$(scan staged value_scoped_path_new_value)"

# 7. Adjacent to an allowlist: must still be found.
fresh; t=$(token); remember "$t"
mkdir -p "$work/r/123"
printf '{"t":"%s"}\n' "$t" >"$work/r/123/other.jsonl"; git -C "$work/r" add 123/other.jsonl
expect adjacent_to_allowlist staged 1 "$(scan staged adjacent_to_allowlist)"

# 8. Negative control: clean staged change.
fresh; printf 'nothing secret\n' >>"$work/r/README"; git -C "$work/r" add README
expect clean_negative_control staged 0 "$(scan staged clean_negative_control)"

# Redaction: no synthetic token appears in any report or log.
leaked=0
for t in "${secrets[@]}"; do
  if grep -rqF "$t" "$out"; then leaked=$((leaked + 1)); fi
done
expect redaction all 0 "$leaked"

echo "reports: $out"
[ "$failures" -eq 0 ]
