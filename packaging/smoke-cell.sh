#!/bin/sh
# Functional smoke of installed SEA-Forge binaries (Linux and macOS).
#
# Creates a fresh cell in a temp directory, lists its runs with the CLI, starts
# the daemon against that cell, waits for its socket to appear, then stops it.
#
# Usage: packaging/smoke-cell.sh <sea-forge> <sea-forge-server>
set -eu

if [ "$#" -ne 2 ]; then
    echo "usage: $0 <sea-forge> <sea-forge-server>" >&2
    exit 2
fi
cli=$1
server=$2

work=$(mktemp -d /tmp/sfsmoke.XXXXXX)
server_pid=""
cleanup() {
    if [ -n "$server_pid" ]; then
        kill "$server_pid" 2>/dev/null || true
        wait "$server_pid" 2>/dev/null || true
    fi
    rm -rf "$work"
}
trap cleanup EXIT

cell="$work/cell"
mkdir -p "$cell"

"$cli" --version
"$cli" runs --root "$cell"
echo "smoke: CLI read the fresh cell (exit 0)"

SEA_FORGE_ROOT="$cell" "$server" > "$work/server.stdout" 2> "$work/server.log" &
server_pid=$!

i=0
while [ "$i" -lt 60 ]; do
    if ! kill -0 "$server_pid" 2>/dev/null; then
        echo "error: sea-forge-server exited before creating its socket" >&2
        cat "$work/server.log" >&2
        exit 1
    fi
    if find "$cell" -type s | grep -q .; then
        break
    fi
    sleep 1
    i=$((i + 1))
done

if ! find "$cell" -type s | grep -q .; then
    echo "error: no socket appeared under the cell within 60s" >&2
    cat "$work/server.log" >&2
    exit 1
fi
echo "smoke: sea-forge-server running, socket: $(find "$cell" -type s | head -n 1)"
echo "smoke: PASS"
