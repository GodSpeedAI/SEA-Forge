#!/usr/bin/env bash
#
# Real Linux Workbench smoke. It is intentionally strict: every record comes
# from the normal seed path; every UI interaction goes through the compiled
# Tauri application; no `__TAURI_INTERNALS__` mock is ever loaded.
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PREFLIGHT=0
FILTER=""
if [ "${1:-}" = "--preflight" ]; then
    PREFLIGHT=1
else
    FILTER="${1:-}"
fi
TOOLS="$REPO/workbench/.tools/tauri-driver-2.0.6/bin"
TAURI_DRIVER="$TOOLS/tauri-driver"
APP="${SEA_FORGE_E2E_APP:-$REPO/workbench/apps/desktop/src-tauri/target/release/sea-forge-workbench}"
ARTIFACTS="$(mktemp -d /tmp/sea-forge-real-e2e.XXXXXX)"
CELL="$ARTIFACTS/cell"
SOCKET="$ARTIFACTS/server.sock"
DRIVER_PID=""
KEEP_ARTIFACTS=0
DRIVER_PORT=""
NATIVE_PORT=""
FRESH_CELL=0
CELL_SEEDED=0

# The initialization proof must begin with no root at all.  Other smoke
# scenarios need the governed seed fixture, so do not blur the two states.
if [[ "$FILTER" =~ initialization ]]; then
    FRESH_CELL=1
fi

cleanup() {
    status=$?
    if [ -n "$DRIVER_PID" ] && kill -0 "$DRIVER_PID" 2>/dev/null; then
        kill "$DRIVER_PID" 2>/dev/null || true
        wait "$DRIVER_PID" 2>/dev/null || true
    fi
    if [ "$CELL_SEEDED" -eq 1 ]; then
        SEA_FORGE_DEMO_SOCKET="$SOCKET" "$REPO/scripts/reset-cell.sh" "$CELL" --yes \
            >"$ARTIFACTS/reset.log" 2>&1 || true
    fi
    if [ "$status" -ne 0 ] || [ "$KEEP_ARTIFACTS" -eq 1 ]; then
        echo "[real-e2e] retained artifacts: $ARTIFACTS" >&2
    else
        rm -rf "$ARTIFACTS"
    fi
    exit "$status"
}
trap cleanup EXIT

preflight() {
[ -x "$TAURI_DRIVER" ] || {
    echo "fail: missing pinned tauri-driver at $TAURI_DRIVER" >&2
    echo "next move: cargo install tauri-driver --version 2.0.6 --locked --root workbench/.tools/tauri-driver-2.0.6" >&2
    exit 1
}
WEBKIT_DRIVER="$(command -v WebKitWebDriver || true)"
[ -n "$WEBKIT_DRIVER" ] || {
    echo "fail: native Linux WebKit driver is unavailable (expected WebKitWebDriver)" >&2
    echo "next move: sudo apt-get install webkit2gtk-driver" >&2
    exit 1
}
command -v xvfb-run >/dev/null || {
    echo "fail: native Linux display server is unavailable (expected xvfb-run)" >&2
    echo "next move: sudo apt-get install xvfb" >&2
    exit 1
}
}

preflight
if [ "$PREFLIGHT" -eq 1 ]; then
    echo "[real-e2e] native driver prerequisites available"
    exit 0
fi

# A fixed WebDriver port makes one failed or concurrent run able to masquerade
# as the next run's driver. Reserve a distinct pair for this invocation instead.
read -r DRIVER_PORT NATIVE_PORT <<EOF
$(python3 - <<'PY'
import socket

listeners = []
try:
    for _ in range(2):
        listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        listener.bind(("127.0.0.1", 0))
        listeners.append(listener)
    print(" ".join(str(listener.getsockname()[1]) for listener in listeners))
finally:
    for listener in listeners:
        listener.close()
PY
)
EOF

[ -x "$APP" ] || {
    echo "fail: compiled Workbench not found at $APP" >&2
    echo "next move: just workbench-package" >&2
    exit 1
}

if [ "$FRESH_CELL" -eq 0 ]; then
    SEA_FORGE_DEMO_SOCKET="$SOCKET" "$REPO/scripts/seed-cell.sh" "$CELL" \
        >"$ARTIFACTS/seed.log" 2>&1
    CELL_SEEDED=1
fi

# The single driver process is the only long-running process this runner owns.
# The native driver owns the app it launches; deleting the session closes that
# app, whose own supervisor then settles the sidecar it started.
SEA_FORGE_ROOT="$CELL" \
SEA_FORGE_SOCKET="$SOCKET" \
SEA_FORGE_SERVER_BIN="$REPO/target/release/sea-forge-server" \
LIBGL_ALWAYS_SOFTWARE=1 \
GDK_BACKEND=x11 \
xvfb-run --auto-servernum --server-args="-screen 0 1280x860x24" \
"$TAURI_DRIVER" --port "$DRIVER_PORT" --native-port "$NATIVE_PORT" --native-driver "$WEBKIT_DRIVER" \
    >"$ARTIFACTS/tauri-driver.log" 2>&1 &
DRIVER_PID=$!

DRIVER_READY=0
for _ in $(seq 1 100); do
    if ! kill -0 "$DRIVER_PID" 2>/dev/null; then
        echo "fail: the Tauri driver exited before becoming ready" >&2
        sed -n '1,120p' "$ARTIFACTS/tauri-driver.log" >&2 || true
        exit 1
    fi
    if DRIVER_PORT="$DRIVER_PORT" python3 - <<'PY' >/dev/null 2>&1
import os
import socket

s = socket.create_connection(("127.0.0.1", int(os.environ["DRIVER_PORT"])), 0.2)
s.close()
PY
    then
        DRIVER_READY=1
        break
    fi
    sleep 0.1
done

[ "$DRIVER_READY" -eq 1 ] || {
    echo "fail: the Tauri driver did not become ready on port $DRIVER_PORT" >&2
    sed -n '1,120p' "$ARTIFACTS/tauri-driver.log" >&2 || true
    exit 1
}

SMOKE_ARGS=()
if [ "$FRESH_CELL" -eq 1 ]; then
    SMOKE_ARGS+=(--fresh-cell)
fi

python3 "$REPO/scripts/workbench_tauri_webdriver_smoke.py" \
    --driver "http://127.0.0.1:$DRIVER_PORT" \
    --application "$APP" \
    --socket "$SOCKET" \
    --artifacts "$ARTIFACTS" \
    --filter "$FILTER" \
    "${SMOKE_ARGS[@]}"

echo "[real-e2e] native Tauri smoke passed"
