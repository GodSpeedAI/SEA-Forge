#!/usr/bin/env sh
# Helper for fde-proof.d scripts (WP-18): _rt-test.sh <min-passed> <filter>...
# The RealityTrace half of a cross-repo test: runs `cargo test -p sxr-observed --test wp18 <filter>` in the
# RealityTrace worktree (FDE_REALITYTRACE_DIR, default ~/projects/_wt/fde-sxr-integration) under the same
# low-memory settings and lock as _cargo-test.sh. Exit 0: the half ran and passed. Exit 10 (PARTIAL): the
# RealityTrace repository is not there, so the half did not run; a caller must never turn that into PASS.
# Any other exit: the half ran and failed.
set -eu
min=$1; shift
rt=${FDE_REALITYTRACE_DIR:-$HOME/projects/_wt/fde-sxr-integration}
if [ ! -f "$rt/sxr-observed/tests/wp18.rs" ]; then
  echo "NOT_RUN: the RealityTrace half needs $rt/sxr-observed/tests/wp18.rs (set FDE_REALITYTRACE_DIR); reporting PARTIAL, never PASS" >&2
  exit 10
fi
lock=${FDE_CARGO_LOCK:-$HOME/projects/_wt/cargo.lock}
export RUSTC_WRAPPER= CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
for filter in "$@"; do
  out=$(mktemp "${TMPDIR:-/tmp}/fde-rt-test.XXXXXX")
  set +e
  if [ -e "$lock" ]; then
    (cd "$rt" && flock -o "$lock" cargo test -p sxr-observed --locked --test wp18 "$filter") 2>&1 | tee "$out"
  else
    (cd "$rt" && cargo test -p sxr-observed --locked --test wp18 "$filter") 2>&1 | tee "$out"
  fi
  code=$?
  set -e
  passed=$(sed -n 's/^test result: ok\. \([0-9][0-9]*\) passed.*/\1/p' "$out" | tail -1)
  rm -f "$out"
  [ "$code" -eq 0 ] || { echo "FAIL: RealityTrace cargo test $filter exited $code" >&2; exit 1; }
  [ "${passed:-0}" -ge "$min" ] || { echo "FAIL: RealityTrace $filter ran ${passed:-0} tests, need >= $min" >&2; exit 1; }
done
