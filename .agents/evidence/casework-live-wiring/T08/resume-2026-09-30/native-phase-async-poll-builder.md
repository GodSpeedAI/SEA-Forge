# T08 native phase asynchronous browser driver

Status: builder evidence; independent review is still required. The native T08 gate passed its first complete actual-browser run after this change.

## Diagnosis and repair

The earlier actual Go race run reached a CDP failure while evaluating the entire phase promise in one `agent-browser eval`: `CDP command timed out: Runtime.evaluate`. It had also recorded two actual upstream-abort/downstream-destroy markers, but returned no phase result, so that run did not prove the browser's native `onerror` callback or the recovery assertions.

The installed `agent-browser` 0.38.1 README documents a 25,000 ms default operation timeout and a 30,000 ms CLI IPC read timeout, and warns that raising the operation timeout beyond 30 seconds can cause `EAGAIN` (`/home/sprime01/.local/share/mise/installs/npm-agent-browser/0.38.1/node_modules/agent-browser/README.md`, “Default Timeout”, lines 1230–1243). The phase performs real login, API, snapshot, retry, and disconnect work, so a single evaluation can exceed that command window.

`native_events_live_test.go` now starts the exact existing `runNativeEventPhase(phase)` promise once in a unique global key in the isolated browser session. Short CDP evaluations poll that same promise for its actual resolved result or rejection. The Go driver fails on rejection, malformed state, CDP errors, or a 90-second overall deadline. It never treats the start acknowledgment as phase success. A resolved phase result is removed from the page global; the browser session is still closed by the existing test cleanup. The phase implementation and its assertions in `native-events-live.ts` were not changed by this driver repair.

The private generated Vite proxy continues to act only on the owned `/phase1/api/events` and `/phase2/api/events` test paths. On a real incomplete upstream event response, its configured proxy callback destroys the corresponding downstream response. The phase's expected close counts remain asserted as two for phase one and one for phase two. The integration test's full pass therefore includes the actual Chromium-native EventSource callbacks, authenticated retry requests, retained-cursor and snapshot checks, disposal checks, and phase-two C-to-L replay assertions.

## Verification

Host preflight immediately before the run showed 1.8 GiB available memory, writable `/tmp/sea-t08-native-gocache`, and no active Go test, Vite, agent-browser, server, Cargo, or Rust compiler process. The run used the approved host execution path and low-memory Go settings:

```text
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOCACHE=/tmp/sea-t08-native-gocache \
  go test -p=1 -parallel=1 -race -tags=live \
  -run '^TestNativeEventSourceResyncRetryAndRetentionReplay$' -count=1 ./internal/server
```

Result: exit 0; `ok github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/server 74.418s`. Captured command output: `/tmp/sea-t08-native-artifacts/native-go-race-async-poll.log`.

The prior failed CDP run and its failed cells remain preserved. The successful run's temporary diagnostics were removed by normal Go temporary-directory cleanup. After completion, no owned Go test, Vite, agent-browser, kernel server, Cargo, or Rust compiler process remained. `gofmt` and `git diff --check` completed for the edited Go test file.

## Material difference and remaining review

The browser test driver changed from one long-running CDP evaluation to a start-and-poll driver with a unique page key and a bounded 90-second wall deadline. This addresses the documented command-level evaluation limit while keeping the same native browser, promise, session, and phase assertions. No production adapter, authority path, persisted schema, dependency, event source, or assertion was changed as part of this repair.

This is builder evidence, not independent approval or a claim that T08 is fully closed. An independent critic still needs to inspect the source change and verify that the passing native run satisfies the original interruption/recovery contract.
