#!/usr/bin/env bash
#
# Renderer-only, non-mocked evidence using agent-browser. This runner must not
# be described as real-stack coverage: Chrome has no Tauri IPC bridge. Its
# asserted outcome is the safe one for that condition — visible unavailability,
# not a fabricated successful projection.
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACTS="$(mktemp -d /tmp/sea-forge-agent-browser.XXXXXX)"
VITE_PID=""
SESSION="sea-forge-renderer-$$"
KEEP_ARTIFACTS="${SEA_FORGE_E2E_KEEP_ARTIFACTS:-0}"

cleanup() {
    status=$?
    if [ -n "$VITE_PID" ] && kill -0 "$VITE_PID" 2>/dev/null; then
        kill "$VITE_PID" 2>/dev/null || true
        wait "$VITE_PID" 2>/dev/null || true
    fi
    agent-browser --session "$SESSION" close --all >/dev/null 2>&1 || true
    if [ "$status" -ne 0 ] || [ "$KEEP_ARTIFACTS" -eq 1 ]; then
        echo "[agent-browser] retained artifacts: $ARTIFACTS" >&2
    else
        rm -rf "$ARTIFACTS"
    fi
    exit "$status"
}
trap cleanup EXIT

command -v agent-browser >/dev/null || {
    echo "fail: agent-browser is required; install the project-approved CLI first" >&2
    exit 1
}

cd "$REPO/workbench/apps/desktop"
bun run vite --host 127.0.0.1 --port 0 --strictPort >"$ARTIFACTS/vite.log" 2>&1 &
VITE_PID=$!

for _ in $(seq 1 100); do
    url="$(sed -n 's|.*Local: *\(http://127\.0\.0\.1:[0-9][0-9]*\)/.*|\1|p' "$ARTIFACTS/vite.log" | tail -1)"
    [ -n "$url" ] && break
    sleep 0.1
done
[ -n "${url:-}" ] || {
    echo "fail: Vite did not publish a local URL; see $ARTIFACTS/vite.log" >&2
    KEEP_ARTIFACTS=1
    exit 1
}

agent-browser --session "$SESSION" --screenshot-dir "$ARTIFACTS" open "$url/readiness" \
    >"$ARTIFACTS/open.txt"
agent-browser --session "$SESSION" wait "#main-content" >"$ARTIFACTS/wait.txt"
agent-browser --session "$SESSION" screenshot "$ARTIFACTS/readiness-no-bridge.png" \
    >"$ARTIFACTS/screenshot.txt"
agent-browser --session "$SESSION" snapshot >"$ARTIFACTS/readiness-no-bridge.snapshot"
agent-browser --session "$SESSION" a11y --selector "#main-content" --json \
    >"$ARTIFACTS/readiness-no-bridge.a11y.json"
agent-browser --session "$SESSION" console >"$ARTIFACTS/console.txt"
agent-browser --session "$SESSION" errors >"$ARTIFACTS/errors.txt"

if ! rg -q "Readiness projection unavailable|Cell unavailable" "$ARTIFACTS/readiness-no-bridge.snapshot"; then
    echo "fail: renderer did not fail closed without the native Tauri bridge" >&2
    KEEP_ARTIFACTS=1
    exit 1
fi

python3 - "$ARTIFACTS/readiness-no-bridge.a11y.json" <<'PY'
import json
import sys

result = json.load(open(sys.argv[1]))
payload = result.get("data", result) if isinstance(result, dict) else result
violations = payload.get("violations", []) if isinstance(payload, dict) else []
if violations:
    raise SystemExit(f"fail: no-bridge renderer has {len(violations)} accessibility violation(s)")
PY

echo "[agent-browser] renderer-only no-bridge evidence passed"
