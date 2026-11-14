#!/usr/bin/env bash
# S1 gate (WP-19): run the three `program` episodes of the frozen pre-registration through the stack in a
# fresh scratch cell under $HOME/projects/_wt/tmp/fde-s1/<stamp>, keeping every raw record. Prints the cell
# path on its last line. See scripts/fde-s1/run.py for what each step does and which gaps are recorded.
#
#   scripts/fde-s1-run.sh            full run (builds, three real cargo test runs: slow, needs >= 3500 MB free)
#   FDE_S1_DEBUG_ARGV='["true"]'     plumbing check only; the cell is marked debug and the proof runner refuses it
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
export TMPDIR=${TMPDIR:-$HOME/projects/_wt/tmp}
base=$TMPDIR/fde-s1; mkdir -p "$base"
avail=$(free -m | awk '/^Mem:/{print $7}')
[ "$avail" -ge "${FDE_S1_MIN_FREE_MB:-3500}" ] || { echo "fde-s1-run: only ${avail} MB available (need ${FDE_S1_MIN_FREE_MB:-3500}); stop other heavy jobs" >&2; exit 2; }
for t in bun git python3 sqlite3 flock cargo; do command -v "$t" >/dev/null || { echo "fde-s1-run: $t missing" >&2; exit 2; }; done
lock=${FDE_CARGO_LOCK:-$HOME/projects/_wt/cargo.lock}
cargo_env=(RUSTC_WRAPPER= CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0)
( cd "$root" && flock -o "$lock" env "${cargo_env[@]}" cargo build --locked -p sea-forge-server --example fde_s1_driver ) >&2
seed=${FDE_RT_SEED_BIN:-$HOME/projects/_wt/tmp/rt-seed-target/debug/rt-seed}
if [ ! -x "$seed" ]; then
  ( cd "$root/scripts/fde-s1/rt-seed" && flock -o "$lock" env "${cargo_env[@]}" CARGO_TARGET_DIR=$HOME/projects/_wt/tmp/rt-seed-target cargo build ) >&2
fi
stamp=$(date -u +%Y%m%dT%H%M%SZ)
cell=$base/$stamp
python3 "$root/scripts/fde-s1/run.py" "$cell" pipeline >&2 || { echo "fde-s1-run: pipeline failed; partial cell kept at $cell" >&2; ln -sfn "$cell" "$base/latest"; echo "$cell"; exit 1; }
ln -sfn "$cell" "$base/latest"
echo "$cell"
