# Independent source review: async polling deadline repair

## Verdict

**Reject source readiness pending the successful-path browser close bound.** This review is
source-only; no test, build, or browser command was run here. The prior source finding about
the 90-second poll deadline is fixed, but one close still uses the general 45-second command
timeout rather than the required five-second bound.

## Verified corrections and retained proof

- `runNativeBrowserPhase` derives one 90-second context from `t.Context()` and uses it for
  phase start and every poll. Each command's 45-second child timeout cannot extend that
  parent deadline.
- The 200 ms wait selects on the same phase context, so deadline cancellation interrupts the
  wait as well as CDP calls.
- Failure diagnostics share one separate 10-second context. Failure then tries to close the
  owned browser session under a five-second context, reports a close failure, and marks the
  session closed only after a successful command.
- The driver still waits for the exact existing `runNativeEventPhase` promise to resolve with
  a valid phase result or fail on a rejected, malformed, missing, or unexpected envelope.
  The start acknowledgement is not treated as success.
- The Vite hook and original TypeScript assertions remain unchanged. The hook only destroys
  the downstream response for incomplete upstream event responses. Phase one requires actual
  native error callbacks, retry from K, ordered resync/snapshot at L, matching retained/current
  snapshots, and pending-retry disposal. Phase two asserts the post-L state still has one
  server-recorded request before the second request from retained C, then requires only
  snapshot L. Both check retained Store history and cleanup.
- `t.Context()` is supported by this repository's Go 1.27 module/toolchain.

## Required remaining correction

After the two phases pass, `TestNativeEventSourceResyncRetryAndRetentionReplay` explicitly
closes the browser at `native_events_live_test.go:266-269` using
`runNativeBrowser(context.Background(), session, "close")`. That path gets the helper's
45-second timeout. Use the same explicit five-second context policy as failure cleanup, surface
the close error, and keep the cleanup flag behavior so a failed close is retried by the bounded
test cleanup. Do not change the native phases or assertions.

The registered fallback test cleanup is correctly bounded to five seconds and reports its
error. Vite cleanup remains process-owned and awaited. A close failure may cause a second
bounded five-second attempt; retain and document that failure-path allowance.

## Builder result is not independent evidence

The builder reports a native race pass at
`/tmp/sea-t08-native-artifacts/native-go-race-async-poll.log` (74.418 seconds). I did not rerun
it because the successful-path close still exceeds the authorized source bound. The shared
local/live and UI gates remain unrun by this reviewer.

## Hashes reviewed

```text
845c4f494087f251c7d07b1f9806b9702c2a61b8a706c073821aec007aadb548  apps/godspeed-casework-go/internal/server/native_events_live_test.go
ba29bdccfd5bc375ef52b7dd6366be15c8d04cc3924352f8c09137ef69c6ab80  apps/godspeed-cognitive-ui/e2e/native-events-live.ts
```
