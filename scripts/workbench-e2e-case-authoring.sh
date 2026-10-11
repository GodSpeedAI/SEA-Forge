#!/usr/bin/env bash
#
# Case-authoring proof journeys (scenarios 6 and 7) driven by agent-browser.
#
# Mocked-IPC evidence: this runner installs the agent-browser counterpart of
# the Playwright `e2e/tauriMock.ts` shim (see
# workbench/apps/desktop/e2e-agent-browser/sfwp-case-authoring-mock.js), so
# the real renderer code — TanStack Query hooks, the XState authoring
# machine, the generated AJV contract validators — runs unmodified against
# scripted `case.preflight`/`case.commit`/`request.get_status` answers. It
# must not be described as real-stack coverage: there is no Tauri host and no
# sea-forge-server on the other side of the bridge.
#
# Journey 7 (stale repair): commit #1 is rejected as stale carrying digest A;
# the only repair affordance is a re-preflight that pins digest B; commit #2
# succeeds carrying B. Proves the UI cannot re-commit a stale precondition.
#
# Journey 6 (ambiguous commit): commit #1's response is dropped; the UI offers
# recovery via `request.get_status` only, the commit control stays disabled
# (a programmatic click attempt is a no-op), and exactly one `case_commit`
# ever crosses the bridge. Proves no duplicate side effect.
#
# Mechanics this runner depends on (learned the hard way):
# - The shim is eval'd into a live page, so every navigation after install
#   must be a SPA route change; a full page load would wipe it.
# - Viewport is 1600x1000: below 1420px the evidence drawer overlays the
#   attention rail and intercepts clicks on the Commit control (the same
#   reason the Playwright specs pin this viewport).
# - Status-pill labels render uppercase via CSS; text waits compare
#   uppercased innerText.
# - The stale state is asserted via its alert + "Re-run preflight" control,
#   not the pill label: the pill keeps "Preflight passed" in the stale state
#   because `preflight?.ok` shadows `state` in the label ternary (filed as
#   debt).
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MOCK="$REPO/workbench/apps/desktop/e2e-agent-browser/sfwp-case-authoring-mock.js"
ARTIFACTS="$(mktemp -d /tmp/sea-forge-case-authoring.XXXXXX)"
VITE_PID=""
SESSION="sea-forge-case-authoring-$$"
KEEP_ARTIFACTS="${SEA_FORGE_E2E_KEEP_ARTIFACTS:-0}"
DIGEST_A="sha256:17a4c917a4c917a4c917a4c917a4c917a4c917a4c917a4c917a4c917a4c917a4"
DIGEST_B="sha256:9e2f639e2f639e2f639e2f639e2f639e2f639e2f639e2f639e2f639e2f639e2f"

cleanup() {
    status=$?
    if [ -n "$VITE_PID" ] && kill -0 "$VITE_PID" 2>/dev/null; then
        kill "$VITE_PID" 2>/dev/null || true
        wait "$VITE_PID" 2>/dev/null || true
    fi
    agent-browser --session "$SESSION" close --all >/dev/null 2>&1 || true
    if [ "$status" -ne 0 ] || [ "$KEEP_ARTIFACTS" -eq 1 ]; then
        echo "[case-authoring] retained artifacts: $ARTIFACTS" >&2
    else
        rm -rf "$ARTIFACTS"
    fi
    exit "$status"
}
trap cleanup EXIT

fail() {
    echo "fail: $*" >&2
    KEEP_ARTIFACTS=1
    exit 1
}

command -v agent-browser >/dev/null || fail "agent-browser is required"
[ -f "$MOCK" ] || fail "mock shim missing: $MOCK"
command -v bun >/dev/null || fail "bun is required (workbench toolchain)"

ab() { agent-browser --session "$SESSION" --screenshot-dir "$ARTIFACTS" "$@"; }

# Text waits compare uppercased innerText: pill labels are CSS-uppercased.
wait_text() { ab wait --fn "document.body.innerText.toUpperCase().includes('$1')" >/dev/null; }

url_is() { ab get url | rg -q "$1"; }

commit_disabled() {
    ab eval --stdin <<'EVALEOF'
(() => {
  const b = [...document.querySelectorAll("button")]
    .find((x) => (x.textContent || "").includes("Commit case"));
  return b ? `disabled=${b.disabled}` : "missing";
})()
EVALEOF
}

# agent-browser prints eval results JSON-encoded exactly once (numbers bare,
# strings quoted), so evaluate the raw value and strip the quotes.
mock_field() { ab eval "window.__sfwpMock.$1" | tr -d '"'; }

cd "$REPO/workbench/apps/desktop"
bun run vite --host 127.0.0.1 --port 0 --strictPort >"$ARTIFACTS/vite.log" 2>&1 &
VITE_PID=$!

for _ in $(seq 1 100); do
    url="$(sed -n 's|.*Local: *\(http://127\.0\.0\.1:[0-9][0-9]*\)/.*|\1|p' "$ARTIFACTS/vite.log" | tail -1)"
    [ -n "$url" ] && break
    sleep 0.1
done
[ -n "${url:-}" ] || fail "Vite did not publish a local URL; see $ARTIFACTS/vite.log"

# Boot the renderer bridge-less (it must fail closed, not crash), then install
# the mock into the live document and reach /cases/new through SPA
# navigation, which preserves the shim.
ab open "$url/readiness" >/dev/null
ab set viewport 1600 1000 >/dev/null
ab wait "#main-content" >/dev/null
ab eval --stdin <"$MOCK" >"$ARTIFACTS/mock-install.txt"
rg -q "sfwp mock installed" "$ARTIFACTS/mock-install.txt" || fail "mock shim did not install"

reach_authoring_route() {
    ab find text "Cases" click >/dev/null
    ab find text "Readiness" click >/dev/null
    wait_text "CREATE CASE"
    ab find role button click --name "Create case" >/dev/null
    ab wait --fn "document.body.innerText.includes('Select a template')" >/dev/null
    url_is "/cases/new$" || fail "did not reach /cases/new"
}

fill_draft() {
    ab find role radio click --name "hello_agents@0.1.0" >/dev/null
    ab wait 400 >/dev/null
    ab find label "greeting_name" fill "Operator" >/dev/null
}

run_preflight_until_commit_enabled() {
    ab find role button click --name "Run preflight" >/dev/null
    ab wait --fn "[...document.querySelectorAll('button')].find((x) => (x.textContent || '').includes('Commit case'))?.disabled === false" >/dev/null
}

echo "[case-authoring] journey 7: stale precondition repairs via re-preflight"
reach_authoring_route
ab eval 'window.__sfwpMock.reset("stale_once")' >/dev/null
fill_draft
run_preflight_until_commit_enabled

ab find role button click --name "Commit case" >/dev/null
wait_text "THE TEMPLATE CHANGED SINCE PREFLIGHT"
wait_text "RE-RUN PREFLIGHT"
ab screenshot "$ARTIFACTS/journey7-rejected-as-stale.png" >/dev/null

# The stale state's only lawful action is a re-preflight: committing is off.
[[ "$(commit_disabled)" == *disabled=true* ]] || fail "commit control enabled while rejected_as_stale"
[ "$(mock_field "commits.length")" = "1" ] || fail "expected exactly one commit before repair"
[ "$(mock_field "commits[0].preconditions.records[0].expected_digest")" = "$DIGEST_A" ] \
    || fail "stale commit did not carry digest A"

ab find role button click --name "Re-run preflight" >/dev/null
run_preflight_until_commit_enabled
[ "$(mock_field "preflightDigests.length")" = "2" ] \
    || fail "repair did not run exactly two preflights (digests: $(mock_field "preflightDigests"))"
[ "$(mock_field "preflightDigests[1]")" = "$DIGEST_B" ] \
    || fail "re-preflight did not pin digest B (digests: $(mock_field "preflightDigests"))"

ab find role button click --name "Commit case" >/dev/null
ab wait --fn "location.pathname === '/cases'" >/dev/null
[ "$(mock_field "commits.length")" = "2" ] || fail "expected the repaired commit"
[ "$(mock_field "commits[1].preconditions.records[0].expected_digest")" = "$DIGEST_B" ] \
    || fail "repaired commit did not carry the fresh digest"
[ "$(mock_field "statusCalls")" = "0" ] || fail "journey 7 should never need status recovery"
ab screenshot "$ARTIFACTS/journey7-repaired-and-committed.png" >/dev/null

echo "[case-authoring] journey 6: dropped commit response recovers, never resubmits"
reach_authoring_route
ab eval 'window.__sfwpMock.reset("drop_once")' >/dev/null
fill_draft
run_preflight_until_commit_enabled

ab find role button click --name "Commit case" >/dev/null
wait_text "THE COMMIT RESPONSE WAS LOST"
wait_text "RECOVER OUTCOME"
ab screenshot "$ARTIFACTS/journey6-ambiguous.png" >/dev/null

# A duplicate commit is structurally unreachable: the control is disabled and
# even a programmatic click on it must not reach the bridge.
[[ "$(commit_disabled)" == *disabled=true* ]] || fail "commit control enabled while ambiguous"
ab eval --stdin >"$ARTIFACTS/journey6-forced-click.txt" <<'EVALEOF'
(() => {
  const b = [...document.querySelectorAll("button")]
    .find((x) => (x.textContent || "").includes("Commit case"));
  if (b) b.click();
  return b ? `forced click on disabled=${b.disabled}` : "missing";
})()
EVALEOF
[ "$(mock_field "commits.length")" = "1" ] || fail "duplicate case_commit crossed the bridge"

ab a11y --selector "#main-content" --json >"$ARTIFACTS/journey6-ambiguous.a11y.json" || true
python3 - "$ARTIFACTS/journey6-ambiguous.a11y.json" <<'PY' || fail "ambiguous state has accessibility violations"
import json
import sys

result = json.load(open(sys.argv[1]))
payload = result.get("data", result) if isinstance(result, dict) else result
violations = payload.get("violations", []) if isinstance(payload, dict) else []
if violations:
    raise SystemExit(f"fail: ambiguous state has {len(violations)} accessibility violation(s)")
PY

ab find role button click --name "Recover outcome" >/dev/null
ab wait --fn "location.pathname === '/cases'" >/dev/null
[ "$(mock_field "commits.length")" = "1" ] || fail "recovery resubmitted the commit"
[ "$(mock_field "statusCalls")" = "1" ] || fail "recovery did not consult request.get_status"
[ "$(mock_field "commits[0].preconditions.records[0].expected_digest")" = "$DIGEST_A" ] \
    || fail "the single commit did not carry the preflight digest"
ab screenshot "$ARTIFACTS/journey6-recovered.png" >/dev/null

ab errors >"$ARTIFACTS/errors.txt" || true
ab console >"$ARTIFACTS/console.txt" || true
if [ -s "$ARTIFACTS/errors.txt" ]; then
    fail "page errors during the journeys: $(cat "$ARTIFACTS/errors.txt")"
fi

echo "[case-authoring] both proof journeys passed (mocked IPC; renderer code paths real)"
