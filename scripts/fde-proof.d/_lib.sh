#!/usr/bin/env sh
# Shared settings for fde-proof.d scripts (WP-19). Source it: `. "$(dirname "$0")/_lib.sh"`.
# Siblings are other repositories on their `fde/integration` branch; a sibling that is absent, at an
# unexpected commit or without its tool yields NOT_RUN (exit 12), never PASS.
FDE_ROOT=$(cd "$(dirname "$0")/../.." && pwd)
RT_DIR=${FDE_REALITYTRACE_DIR:-$HOME/projects/_wt/fde-sxr-integration}
COG_DIR=${FDE_COGNATE_DIR:-$HOME/projects/_wt/fde-cognate-integration}
HZ_DIR=${FDE_HORIZON_DIR:-$HOME/projects/_wt/fde-horizon-integration}
RT_EXPECT=${FDE_REALITYTRACE_COMMIT:-4930b7d}
COG_EXPECT=${FDE_COGNATE_COMMIT:-db788d1}
HZ_EXPECT=${FDE_HORIZON_COMMIT:-fd2e19b}
DF_DEV_BIN=${FDE_DOMAINFORGE_DEV_BIN:-$HOME/projects/_wt/tmp/df-3960326/domainforge}
DF_RELEASE_BIN=${FDE_DOMAINFORGE_BIN:-$HOME/projects/_wt/tmp/df022-cli/domainforge-0.22.0-x86_64-unknown-linux-gnu/domainforge}
RT_BIN=${FDE_REALITYTRACE_BIN:-$RT_DIR/target/debug/realitytrace}
CELL=${FDE_S1_CELL:-}
export TMPDIR=${TMPDIR:-$HOME/projects/_wt/tmp}
not_run() { echo "REASON: $*"; echo "NOT_RUN: $*" >&2; exit 12; }
partial() { echo "REASON: $*"; echo "PARTIAL: $*" >&2; exit 10; }
# need_sibling <dir> <expected-commit-prefix> <label>
need_sibling() {
  [ -d "$1/.git" ] || [ -f "$1/.git" ] || not_run "$3 worktree absent: $1"
  head=$(git -C "$1" rev-parse --short=7 HEAD 2>/dev/null) || not_run "$3 is not a git worktree: $1"
  case "$head" in "$2"*) ;; *) not_run "$3 is at $head, expected $2 (set FDE_*_COMMIT to accept another commit knowingly)";; esac
}
need_cell() { [ -n "$CELL" ] && [ -d "$CELL/raw" ] || not_run "no S1 episode cell (FDE_S1_CELL); run scripts/fde-s1-run.sh first"; }
cargo_lock() { # cargo_lock <dir> <cargo args...>
  d=$1; shift
  ( cd "$d" && RUSTC_WRAPPER= CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
    flock -o "${FDE_CARGO_LOCK:-$HOME/projects/_wt/cargo.lock}" cargo "$@" )
}
# cargo_pass <dir> <min-passed> <cargo test args...>: 0 only when the run is green with >= min tests passed
# and no producer test printed "NOT_RUN:" (a skipped DomainForge-dependent test is not a pass).
cargo_pass() {
  d=$1; min=$2; shift 2
  out=$(mktemp "$TMPDIR/fde-pass.XXXXXX")
  cargo_lock "$d" "$@" >"$out" 2>&1; code=$?
  tail -n 40 "$out" >&2
  if [ "$code" -ne 0 ] || grep -q '^test result: FAILED' "$out"; then rm -f "$out"; echo "FAIL: cargo $* failed" >&2; return 1; fi
  if grep -q 'NOT_RUN:' "$out"; then rm -f "$out"; echo "REASON: a producer test skipped itself (NOT_RUN) for cargo $*"; return 12; fi
  passed=$(sed -n 's/^test result: ok\. \([0-9][0-9]*\) passed.*/\1/p' "$out" | awk '{s+=$1} END{print s+0}')
  rm -f "$out"
  [ "$passed" -ge "$min" ] || { echo "FAIL: cargo $* ran $passed tests, need >= $min" >&2; return 1; }
}
# rt_test <min> <test-target-args...>: a RealityTrace cargo test with the DomainForge dev binary and the SOP model.
rt_test() {
  min=$1; shift
  need_sibling "$RT_DIR" "$RT_EXPECT" RealityTrace
  [ -x "$DF_DEV_BIN" ] || not_run "DomainForge dev binary missing: $DF_DEV_BIN"
  REALITYTRACE_DOMAINFORGE_BIN=$DF_DEV_BIN FDE_SOP_MODEL=$FDE_ROOT/.sea/interaction/fde-sop/fde-sop.sea \
    cargo_pass "$RT_DIR" "$min" test --locked "$@"
}
# cell_status <id>: ok | fail | absent  (written by scripts/fde-s1-run.sh into $CELL/results/<id>.json)
cell_status() {
  f=$CELL/results/$1.json
  [ -n "$CELL" ] && [ -f "$f" ] || { echo absent; return; }
  python3 -c "import json,sys;print(json.load(open(sys.argv[1])).get('status','fail'))" "$f"
}
# finish <rc> : propagate a helper's exit (0 pass, 12 not run, else fail)
