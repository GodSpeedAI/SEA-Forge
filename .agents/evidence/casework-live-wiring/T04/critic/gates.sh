#!/usr/bin/env bash
# Independent critic gate run for T04 units B+C (2026-09-23).
# TRUE exit capture: every command's exit code is echoed after it runs.
# Cargo lock contention with a parallel critic is expected: retry once on
# lock-wait failures (exit 134/"Blocking waiting for file lock" symptom).
REPO=/home/sprime01/projects/sea-rs
LOGDIR="$REPO/.agents/evidence/casework-live-wiring/T04/critic"
cd "$REPO" || exit 99

run_with_retry() {
  local name="$1"; shift
  local attempt log
  for attempt in 1 2; do
    log="$LOGDIR/gate-${name}-attempt${attempt}.log"
    echo "=== $name attempt $attempt: $* ==="
    "$@" >"$log" 2>&1
    local code=$?
    echo "EXIT=$code (attempt $attempt, log: gate-${name}-attempt${attempt}.log)" | tee -a "$LOGDIR/gates-exit-codes.txt"
    if [ $code -eq 0 ]; then return 0; fi
    if grep -q "Blocking waiting for file lock" "$log" 2>/dev/null; then
      echo "lock-wait failure; retrying once"; sleep 20; continue
    fi
    return $code
  done
  return 1
}

: > "$LOGDIR/gates-exit-codes.txt"

run_with_retry 3crate cargo test -p sea-forge-cli -p sea-forge-case-runner -p sea-forge-server
run_with_retry mutations cargo test -p sea-forge-server --test sfwp_case_mutations
run_with_retry supervisor cargo test -p sea-forge-server --test sfwp_supervisor
run_with_retry templates cargo test -p sea-forge-server --test case_templates_live
run_with_retry fmt cargo fmt --all -- --check
run_with_retry no-async-kernel just -f "$REPO/justfile" -d "$REPO" no-async-kernel

echo "=== ALL GATES DONE ==="
cat "$LOGDIR/gates-exit-codes.txt"
