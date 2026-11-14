#!/usr/bin/env sh
# Helper for fde-proof.d scripts (WP-14): _cargo-test.sh <package> <test-target> <min-passed> <filter>...
# Runs each filter under the low-memory cargo environment (flock on the shared lock when it exists)
# and succeeds only when every filter reports "test result: ok" with at least <min-passed> tests
# passed. A filter matching zero tests is a failure, never a pass.
set -eu
pkg=$1; target=$2; min=$3; shift 3
lock=${FDE_CARGO_LOCK:-$HOME/projects/_wt/cargo.lock}
export RUSTC_WRAPPER= CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
for filter in "$@"; do
  out=$(mktemp "${TMPDIR:-/tmp}/fde-cargo-test.XXXXXX")
  set +e
  if [ -e "$lock" ]; then
    flock -o "$lock" cargo test -p "$pkg" --test "$target" "$filter" 2>&1 | tee "$out"
  else
    cargo test -p "$pkg" --test "$target" "$filter" 2>&1 | tee "$out"
  fi
  code=$?
  set -e
  passed=$(sed -n 's/^test result: ok\. \([0-9][0-9]*\) passed.*/\1/p' "$out" | tail -1)
  rm -f "$out"
  [ "$code" -eq 0 ] || { echo "FAIL: cargo test $filter exited $code" >&2; exit 1; }
  [ "${passed:-0}" -ge "$min" ] || { echo "FAIL: $filter ran ${passed:-0} tests, need >= $min" >&2; exit 1; }
done
