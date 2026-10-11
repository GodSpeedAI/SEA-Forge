# Independent source review: bounded native browser driver

## Verdict

**Source approved for an independent native runtime rerun.** This is not approval of the
builder's runtime claim or of full T08. The shared and UI gates remain outstanding.

## Corrections verified

- `runNativeBrowserPhase` derives one 90-second context from `t.Context()` and passes it to
  start and every poll. The per-command 45-second timeout is a child of that context, so it
  cannot extend the phase deadline. The 200 ms wait selects on the same context.
- Failure diagnostics share one 10-second context. Browser closure then gets a separate
  five-second context. Close failures are recorded, and `browserClosed` becomes true only
  after a successful close.
- All three owned session close paths are bounded to five seconds: fallback test cleanup
  (`native_events_live_test.go:235-243`), normal successful close (`266-270`), and phase
  failure close (`372-386`). A failed failure-path close is retried by fallback cleanup under
  a second five-second bound; the failure is surfaced.
- Polling waits for the same `runNativeEventPhase(phase)` promise's actual result. The
  start response is only an acknowledgement. Missing, malformed, unexpected, rejected, or
  timed-out states fail.
- The private Vite hook, TypeScript phases, transport, and authority behavior are unchanged.
  The strict native callbacks, cursor and event assertions remain intact. For phase two, the
  post-L state must still contain only the original server-recorded request; the later actual
  request from retained C and the only `snapshot(L)` are then checked while C and L are
  retained.
- `t.Context()` is available in the pinned Go 1.27 module/toolchain. `git diff --check`
  completed successfully.

## Runtime evidence needed

Run the independent serialized native race test and preserve its exact command, host
preflight, raw output, exit status, diagnostics, and source hash. The builder reports a prior
native pass in `/tmp/sea-t08-native-artifacts/native-go-race-async-poll.log`; that remains
non-independent builder evidence. T08 stays open until independent native, shared local/live,
and required UI gates pass with limitations recorded.

## Source hashes

```text
c7530522053e952fd36c13a55af7ea44e62f16275e26dd583b88b3d0b9a270b1  apps/godspeed-casework-go/internal/server/native_events_live_test.go
ba29bdccfd5bc375ef52b7dd6366be15c8d04cc3924352f8c09137ef69c6ab80  apps/godspeed-cognitive-ui/e2e/native-events-live.ts
```
