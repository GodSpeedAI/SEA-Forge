#!/usr/bin/env bash
# Boots a REAL sea-forge-server binary on a fresh temp cell and runs the
# critic's runtime attack (cycle refusal + input refusal + positive control).
set -u
REPO=/home/sprime01/projects/sea-rs
CRITIC="$REPO/.agents/evidence/casework-live-wiring/T04/critic"
CELL=$(mktemp -d /tmp/sf-t04critic-XXXX)
export SEA_FORGE_ROOT="$CELL"
export SEA_FORGE_SOCKET="$CELL/server.sock"
mkdir -p "$CELL"
uid=$(id -u)
cat > "$CELL/server.yaml" <<YAML
identity:
  bindings:
    - uid: $uid
      actor_id: operator_local
      roles: ["operator"]
YAML
cat > "$CELL/policy.yaml" <<'YAML'
version: "0.1"
rules:
  - name: allow-reopen
    verdict: allow
    actor_role: operator
    operation_kind: case_reopen
  - name: allow-propose
    verdict: allow
    actor_role: operator
    operation_kind: discretionary_task_add
  - name: allow-terminate
    verdict: allow
    actor_role: operator
    operation_kind: case_terminate
  - name: allow-human-task
    verdict: allow
    actor_role: operator
    operation_kind: human_task_completion
  - name: allow-command
    verdict: allow
    actor_role: operator
    operation_kind: execute_command
    argv0: sea-forge
YAML
BIN=$(ls -1 "$REPO/target/debug/sea-forge-server" 2>/dev/null || ls -1 "$REPO/target/release/sea-forge-server" 2>/dev/null)
if [ -z "$BIN" ]; then
  echo "no prebuilt server binary; building (debug)..."
  cargo build --manifest-path "$REPO/Cargo.toml" -p sea-forge-server --bin sea-forge-server || exit 99
  BIN="$REPO/target/debug/sea-forge-server"
fi
echo "server binary: $BIN"
echo "cell: $CELL"
"$BIN" > "$CRITIC/attack-server.log" 2>&1 &
SRV=$!
for _ in $(seq 1 200); do [ -S "$SEA_FORGE_SOCKET" ] && break; sleep 0.1; done
if [ ! -S "$SEA_FORGE_SOCKET" ]; then
  echo "FAIL: socket never appeared"; kill "$SRV" 2>/dev/null; cat "$CRITIC/attack-server.log"; exit 1
fi
echo "socket up: $SEA_FORGE_SOCKET"
python3 "$CRITIC/attack.py" "$SEA_FORGE_SOCKET" "$CELL"
CODE=$?
echo "ATTACK_EXIT=$CODE"
kill "$SRV" 2>/dev/null
wait "$SRV" 2>/dev/null
rm -rf "$CELL"
exit $CODE
