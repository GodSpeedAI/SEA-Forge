#!/usr/bin/env bash
# Run the SEA promotion/re-pin test against a real, disposable RealityTrace ledger.
set -euo pipefail

rt=${FDE_REALITYTRACE_DIR:-$HOME/projects/_wt/fde-sxr-integration}
if [[ ! -f "$rt/sxr-cli/examples/nt07_persist.rs" ]]; then
    echo "NOT_RUN: joined NT-07 requires $rt/sxr-cli/examples/nt07_persist.rs; set FDE_REALITYTRACE_DIR to a RealityTrace checkout with the fixture" >&2
    exit 10
fi
command -v python3 >/dev/null
here=$(cd "$(dirname "$0")" && pwd)
root=$(mktemp -d "${TMPDIR:-/tmp}/sea-nt07.XXXXXX")
trap 'rm -rf -- "$root"' EXIT
lock=${FDE_CARGO_LOCK:-$HOME/projects/_wt/cargo.lock}
export RUSTC_WRAPPER='' CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0

cargo_locked() {
    if [[ -f "$lock" ]]; then
        flock -o "$lock" cargo "$@"
    else
        cargo "$@"
    fi
}

build_rt() (
    cd "$rt"
    cargo_locked build --locked -p realitytrace --bin realitytrace --example nt07_persist --message-format=json
)
build_rt > "$root/rt-build.jsonl"
cargo_locked test --locked -p sea-forge-realitytrace --test sec12_trace --no-run --message-format=json > "$root/sea-build.jsonl"

executable() {
    python3 - "$1" "$2" <<'PY'
import json
import sys
from pathlib import Path

with Path(sys.argv[1]).open() as report:
    rows = map(json.loads, report)
    paths = [row["executable"] for row in rows if row.get("reason") == "compiler-artifact"
             and row["target"]["name"] == sys.argv[2] and row.get("executable")]
if len(paths) != 1:
    sys.exit(f"expected one executable for {sys.argv[2]}, got {len(paths)}")
print(paths[0])
PY
}

SEA_NT07_EMITTER=$(executable "$root/sea-build.jsonl" sec12_trace)
export SEA_NT07_EMITTER
SEA_NT07_PERSIST=$(executable "$root/rt-build.jsonl" nt07_persist)
export SEA_NT07_PERSIST
SEA_NT07_REALITYTRACE=$(executable "$root/rt-build.jsonl" realitytrace)
export SEA_NT07_REALITYTRACE
export XDG_DATA_HOME="$root/data" XDG_STATE_HOME="$root/state" XDG_RUNTIME_DIR="$root/run"
unset SXR_DAEMON_SOCKET
mkdir "$XDG_DATA_HOME" "$XDG_STATE_HOME" "$XDG_RUNTIME_DIR" "$root/repo"
git -C "$root/repo" init -q
git -C "$root/repo" -c user.email=nt07@example.invalid -c user.name=NT-07 \
    -c commit.gpgsign=false -c core.hooksPath=/dev/null commit --allow-empty -qm 'NT-07 fixture'
"$SEA_NT07_REALITYTRACE" repo enroll "$root/repo" --json > "$root/enrolled.json"
SEA_NT07_REPO_ID=$(python3 -c 'import json, sys; print(json.load(sys.stdin)["repo_id"])' < "$root/enrolled.json")
export SEA_NT07_REPO_ID
ledger_dir="$XDG_DATA_HOME/sxr/repos/$SEA_NT07_REPO_ID"
SEA_NT07_CHECKOUT_ID=$(python3 -c 'import json, sys; print(json.load(sys.stdin)["checkout_id"])' < "$ledger_dir/registry.json")
export SEA_NT07_CHECKOUT_ID
export SEA_NT07_DB="$ledger_dir/ledger.db" SEA_NT07_PROOF=1
"$here/_cargo-test.sh" sea-forge-server conformance_world_pin 1 wp18_promote_and_re_pin
