#!/usr/bin/env bash
#
# Seed a demonstration cell with real, inspectable records.
#
# Every record this writes is produced by really running the real kernel: real
# runs really authorized, really settled, really committed to the append-only
# ledger. Nothing is hand-written and nothing is copied in from a fixture. If
# you open a seeded record and follow its evidence links, everything checks out
# exactly as if you had done the work yourself — which is the only kind of demo
# data a governed-evidence system is allowed to have.
#
#   scripts/seed-cell.sh [cell-root]
#
# Default root is $HOME/.sea-forge-demo, deliberately NOT $HOME/.sea-forge,
# which may be a real cell holding real work.
#
# Undo with scripts/reset-cell.sh.
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CELL="${1:-${SEA_FORGE_DEMO_ROOT:-$HOME/.sea-forge-demo}}"
STAMP="$CELL/.seeded-demo-cell"

# The socket lives outside the cell root: a cell under $HOME can easily exceed
# the 95-byte Unix socket path budget, and the server's own remedy for that is
# to point SEA_FORGE_SOCKET at a short path while leaving the records in place.
SOCK="${SEA_FORGE_DEMO_SOCKET:-/tmp/sea-forge-demo.sock}"

say() { printf '[seed] %s\n' "$*"; }

if [ -e "$CELL" ] && [ ! -e "$STAMP" ]; then
    echo "refusing: $CELL exists but was not created by this script." >&2
    echo "          It may be a real cell. Pick another root or remove it yourself." >&2
    exit 1
fi

say "building the binaries this seed drives"
cargo build -q --manifest-path "$REPO/Cargo.toml" -p sea-forge-cli -p sea-forge-server --bins
CLI="$REPO/target/debug/sea-forge"
SERVER="$REPO/target/debug/sea-forge-server"

# Re-seeding must be safe, so start from a clean cell rather than appending a
# second copy of everything to the ledgers.
if [ -e "$STAMP" ]; then
    say "existing demo cell found; resetting it first"
    "$REPO/scripts/reset-cell.sh" "$CELL" --yes
fi

mkdir -p "$CELL"
: >"$STAMP"

# Two actors on this uid, so separation of duty is visible: one operator raises
# work, a different one clears it.
uid="$(id -u)"
cat >"$CELL/server.yaml" <<YAML
identity:
  bindings:
    - uid: $uid
      actor_id: operator_a
      roles: [operator]
    - uid: $uid
      actor_id: operator_b
      roles: [operator]
YAML

# `<cell>/sea-forge` must be the SERVER: policy matches on argv0's file name,
# and the authority engine additionally requires argv0 to canonicalize to the
# process actually executing it — which, for work the server submits in
# process, is the server. The CLI goes beside it under its own name.
ln -sf "$SERVER" "$CELL/sea-forge"
ln -sf "$CLI" "$CELL/sea-forge-cli"

say "cell root: $CELL"

# --- CLI-driven runs: one accepted, one denied ------------------------------

cat >"$CELL/demo-permissive.yaml" <<'YAML'
version: "0.1"
rules:
  - name: allow-model-write
    verdict: allow
    actor_role: operator
    operation_kind: write_file
    path_prefix: ""
  - name: allow-self-validate
    verdict: allow
    actor_role: operator
    operation_kind: execute_command
    argv0: sea-forge
YAML

printf '%s\n' 'version: "0.1"' 'rules: []' >"$CELL/demo-deny.yaml"

say "running work that policy permits"
"$CLI" run --root "$CELL" --policy "$CELL/demo-permissive.yaml" \
    --intent "Generate and validate a simple DomainForge .sea model" \
    >"$CELL/.seed-accepted.out"
accepted_run="$(sed -n 's/^run_id=//p' "$CELL/.seed-accepted.out")"
say "  accepted run: $accepted_run"

say "running work that policy denies"
# Exit 3 is the denial's own exit code, not a script failure.
denied_exit=0
"$CLI" run --root "$CELL" --policy "$CELL/demo-deny.yaml" \
    --intent "Generate and validate a simple DomainForge .sea model" \
    >"$CELL/.seed-denied.out" || denied_exit=$?
if [ "$denied_exit" -ne 3 ]; then
    echo "expected a denial (exit 3), got exit $denied_exit" >&2
    exit 1
fi
denied_run="$(sed -n 's/^run_id=//p' "$CELL/.seed-denied.out")"
say "  denied run: $denied_run (no workspace contents — invariant AUTH-01)"

say "realizing the Genesis self-model so the cell can answer questions"
"$CLI" self-model --root "$CELL" rebuild >/dev/null

# --- Server-driven records: escalation and approval -------------------------

say "starting a server to seed escalations"
SEA_FORGE_ROOT="$CELL" SEA_FORGE_SOCKET="$SOCK" "$SERVER" \
    >"$CELL/.seed-server.log" 2>&1 &
server_pid=$!
# Always stop the server this script started, however the script ends.
trap 'kill "$server_pid" 2>/dev/null || true; wait "$server_pid" 2>/dev/null || true' EXIT

for _ in $(seq 1 200); do
    [ -S "$SOCK" ] && break
    sleep 0.05
done
if [ ! -S "$SOCK" ]; then
    echo "the seeding server never published $SOCK:" >&2
    tail -20 "$CELL/.seed-server.log" >&2
    exit 1
fi

python3 "$REPO/scripts/seed_approvals.py" "$CELL" "$SOCK"

say "done"
cat <<EOF

  Seeded cell: $CELL

  Open it in the Workbench:
      SEA_FORGE_ROOT=$CELL SEA_FORGE_SOCKET=$SOCK just workbench-demo

  Or inspect it from the command line:
      $CLI case list --root $CELL

  Remove everything this created:
      just cell-reset
EOF
