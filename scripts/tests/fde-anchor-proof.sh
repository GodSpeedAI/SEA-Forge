#!/usr/bin/env bash
# Tests for scripts/fde-anchor.sh and scripts/fde-proof.sh (WP-09).
# Fixture tests run in a throwaway repository with a bare origin. The final section runs the
# real AT-01 in this worktree and needs the published tag fde-anchor/prereg/fde-s1 (skipped
# with a failure message when the tag is absent: a skipped prerequisite is not green).
set -eu

SRC=$(cd "$(dirname "$0")/../.." && pwd)
TMP_ROOT=${TMPDIR:-/tmp}/fde-anchor-proof-test-$$
trap 'rm -rf "$TMP_ROOT"' EXIT HUP INT TERM
# Run from a git hook, GIT_DIR and friends would redirect git into the real repository.
for var in $(git rev-parse --local-env-vars); do unset "$var"; done
unset GIT_CONFIG_COUNT GIT_CONFIG_KEY_0 GIT_CONFIG_VALUE_0 2>/dev/null || true

fails=0
ok() { printf 'ok:   %s\n' "$1"; }
no() { printf 'FAIL: %s\n' "$1"; fails=$((fails + 1)); }
expect_rc() { # desc expected-rc  cmd...
  desc=$1; want=$2; shift 2
  set +e; out=$("$@" 2>&1); got=$?; set -e
  last_out=$out
  if [ "$got" -eq "$want" ]; then ok "$desc (exit $got)"; else no "$desc: want exit $want, got $got"; printf '%s\n' "$out" | sed 's/^/      /'; fi
}
expect_in() { case "$last_out" in *"$2"*) ok "$1";; *) no "$1: output lacks '$2'"; printf '%s\n' "$last_out" | sed 's/^/      /';; esac; }

mkdir -p "$TMP_ROOT/origin.git" "$TMP_ROOT/w"
git init -q --bare "$TMP_ROOT/origin.git"
W=$TMP_ROOT/w
git init -q -b main "$W"
git -C "$W" remote add origin "$TMP_ROOT/origin.git"
git -C "$W" config user.email test@example.invalid
git -C "$W" config user.name "Anchor Test"
mkdir -p "$W/scripts" "$W/contracts/fde" "$W/.agents/preregistrations/fde-s1"
cp "$SRC/scripts/fde-anchor.sh" "$SRC/scripts/fde-proof.sh" "$SRC/scripts/check-fde-contract-digests.sh" "$W/scripts/"
cp "$SRC"/contracts/fde/*.json "$SRC/contracts/fde/DIGESTS" "$W/contracts/fde/"
P=$W/.agents/preregistrations/fde-s1
printf 'preregistration_id: fde-s1\ntests:\n- id: AT-77\n  command: x\n' >"$P/tests.yaml"
printf 'alpha\n' >"$P/a.md"
printf 'beta\n' >"$P/b.md"
A=$W/scripts/fde-anchor.sh

digest=$(cd "$W" && scripts/fde-anchor.sh manifest fde-s1)
git -C "$W" add . && git -C "$W" commit -qm prereg && git -C "$W" push -q origin main

expect_rc "publish refuses a digest that is not the manifest digest" 3 "$A" publish prereg fde-s1 "$(printf '0%.0s' $(seq 64))"
expect_rc "publish refuses an id that is not ref-safe" 2 "$A" publish prereg 'bad id' "$digest"
expect_rc "publish --push creates and pushes the tag" 0 "$A" publish prereg fde-s1 "$digest" --push
expect_rc "verify exits 0 on the untouched pre-registration" 0 "$A" verify prereg/fde-s1
expect_rc "verify accepts a bare id and --remote" 0 "$A" verify fde-s1 --remote
expect_in "tag message is the anchor JSON with the digest" "$digest"
expect_rc "publishing the same anchor twice is refused" 3 "$A" publish prereg fde-s1 "$digest" --push
expect_in "refusal says the tag exists" "tag exists"
other=$(printf '1%.0s' $(seq 64))
expect_rc "a different digest under the same tag name is refused" 3 "$A" publish prereg fde-s1 "$other"

printf 'gamma\n' >"$P/b.md"
expect_rc "tampering with one file makes verify fail" 1 "$A" verify prereg/fde-s1
expect_in "verify names the changed file" "file changed: b.md"
(cd "$W" && scripts/fde-anchor.sh manifest fde-s1 >/dev/null)
expect_rc "tampering with the file and regenerating the manifest still fails" 1 "$A" verify prereg/fde-s1
expect_in "the regenerated-manifest case still names the file" "file changed: b.md"
git -C "$W" checkout -q -- .
printf 'extra\n' >"$P/c.md"
expect_rc "an added file makes verify fail" 1 "$A" verify prereg/fde-s1
expect_in "verify names the added file" "file added: c.md"
rm "$P/c.md"; rm "$P/a.md"
expect_rc "a removed file makes verify fail" 1 "$A" verify prereg/fde-s1
expect_in "verify names the removed file" "file removed: a.md"
git -C "$W" checkout -q -- .
expect_rc "verify passes again once restored" 0 "$A" verify prereg/fde-s1

git -C "$W" tag -d fde-anchor/prereg/fde-s1 >/dev/null
expect_rc "verify --remote fails when the local tag is gone" 1 "$A" verify prereg/fde-s1 --remote
git -C "$W" fetch -q origin 'refs/tags/*:refs/tags/*'
git -C "$W" push -q origin :refs/tags/fde-anchor/prereg/fde-s1
expect_rc "verify --remote fails when the tag is missing on origin" 1 "$A" verify prereg/fde-s1 --remote
expect_in "the failure says the tag is not on origin" "not found on origin"

# fde-proof.sh in the fixture: re-publish a fresh anchor for an unmodified tree, then run.
git -C "$W" push -q origin refs/tags/fde-anchor/prereg/fde-s1
expect_rc "proof: an unwired test is NOT_RUN and the run is not green" 1 "$W/scripts/fde-proof.sh" --slice s1 --only AT-77
expect_in "proof reports NOT_RUN" "NOT_RUN"
st=$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["results"][-1]["status"])' "$W/.tmp/fde-proof/s1/results.json")
[ "$st" = NOT_RUN ] && ok "results.json records NOT_RUN, not PASS" || no "results.json status is $st"
expect_rc "proof: --only with an unknown id is a usage error" 2 "$W/scripts/fde-proof.sh" --slice s1 --only AT-99
expect_rc "proof: --replay is a stub that exits 4" 4 "$W/scripts/fde-proof.sh" --slice s1 --replay run-1
expect_rc "proof: --mutate is a stub that exits 4" 4 "$W/scripts/fde-proof.sh" --slice s1 --mutate NT-01
printf 'tampered\n' >>"$P/a.md"
expect_rc "proof refuses to run on a tampered pre-registration" 3 "$W/scripts/fde-proof.sh" --slice s1 --only AT-77
expect_in "the refusal names the changed file" "file changed: a.md"

# Real worktree: AT-01 runs and passes (needs the published anchor).
cd "$SRC"
if git rev-parse -q --verify refs/tags/fde-anchor/prereg/fde-s1 >/dev/null; then
  expect_rc "real: scripts/fde-proof.sh --slice s1 --only AT-01 passes" 0 scripts/fde-proof.sh --slice s1 --only AT-01
  expect_in "real: AT-01 is PASS" "AT-01"
  rst=$(python3 -c 'import json,sys;print([r["status"] for r in json.load(open(sys.argv[1]))["results"] if r["id"]=="AT-01"][0])' .tmp/fde-proof/s1/results.json)
  [ "$rst" = PASS ] && ok "real: results.json AT-01 is PASS" || no "real: AT-01 status $rst"
  expect_rc "real: an unwired test (AT-04) is NOT_RUN and the run exits non-zero" 1 scripts/fde-proof.sh --slice s1 --only AT-04
else
  no "real: tag fde-anchor/prereg/fde-s1 is not published; AT-01 end-to-end not run"
fi

if [ "$fails" -ne 0 ]; then printf '%s test(s) failed\n' "$fails"; exit 1; fi
echo "all fde-anchor / fde-proof tests passed"
