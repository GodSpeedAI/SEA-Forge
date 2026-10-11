# T08 async browser driver and proxy hook: independent source review

## Verdict

**Reject source readiness for independent runtime verification until the phase deadline is
enforced across each CDP call.** The harness changes otherwise preserve the required native
EventSource path and the strict assertions. This is a source-only review: I ran no Go, Bun,
browser, build, or test command.

The builder reports that its native race run passed in 74.418 seconds and saved
`/tmp/sea-t08-native-artifacts/native-go-race-async-poll.log`. That is builder evidence, not an
independent result. This review does not approve T08.

## Required correction: the advertised 90-second phase bound can overrun

`runNativeBrowserPhase` sets a 90-second deadline at
`apps/godspeed-casework-go/internal/server/native_events_live_test.go:315`, but every poll
uses `context.Background()` at line 322. `runNativeBrowser` then gives that individual command
an independent 45-second timeout. A poll started just before the phase deadline can therefore
continue for up to another 45 seconds. On timeout, `failNativeBrowserPhase` performs two more
background browser commands for diagnostics (lines 357-359), each with its own 45-second
limit, before the test reports failure. Thus the 90-second deadline does not bound the
phase-result wait or the error path as described.

Pass the remaining phase deadline into each start/poll command so no CDP call extends the
phase wait. Give diagnostics and cleanup a separate bounded budget and state the resulting
total failure-path bound explicitly. Do not treat the immediate `started` response as proof;
the current code correctly waits for the same browser promise to resolve with the phase result.

## Reviewed proxy interruption path

The private Vite hook in `native_events_live_test.go:139-192` is limited to GET requests for
`/phase1/api/events` and `/phase2/api/events`. It observes the real proxy response, and only
when the upstream response is incomplete and the browser-facing response is still open does it
destroy that downstream `ServerResponse`. It removes its `aborted`, `close`, `error`, `end`,
and downstream `close` listeners on normal completion, downstream close, or the first
incomplete-upstream failure. Its marker includes only phase, event source, and completion state;
the harness requires exactly two markers for phase one and one for phase two.

The installed Vite implementation supports this hook shape: its local
`apps/godspeed-cognitive-ui/node_modules/vite/dist/node/index.d.ts:522` declares
`configure(proxy, options)`, and `dist/node/chunks/config.js:21491-21497` emits `proxyRes`
before the normal `proxyRes.pipe(res)`. The test hook therefore has the actual downstream
response object and can terminate the browser-facing stream; it does not synthesize an
EventSource callback, frame, cursor, or authority result.

The TypeScript phase assertions remain strict and unchanged (`native-events-live.ts` SHA below):

- Phase one requires actual native `onerror` callbacks, an authenticated offline mutation to
  L, a retry request whose query is K, ordered `resync_required(L)` then `snapshot(L)`, matching
  authenticated retained/current snapshots, and disposal with no third request or callbacks.
- Phase two requires a real interruption and offline mutation, one later request from retained
  C, exactly `snapshot(L)` with no resync, and disposal. The post-mutation state assertion
  requires exactly one request before L is observed; the Go harness records each events request
  at handler entry before it reaches the API. The later request and snapshot therefore remain
  observable server-backed replay evidence.

The async driver starts the same exported `runNativeEventPhase(phase)` promise, stores its
resolved or rejected envelope under a unique page key, and polls that key. Missing, malformed,
unexpected, or rejected state fails; only the actual resolved phase result returns success.
Phase functions retain `finally { dispose() }`. Go cleanup attempts browser-session closure,
terminates and waits for its owned Vite process, closes tracked streams, and lets request
context cancellation release a held retry gate. The failure-path browser-close error is
currently ignored by cleanup; the successful explicit close is checked. This review found no
changed adapter, transport, authority, production, or dependency behavior.

## Required next step

Make only the bounded deadline-context correction, preserve the native assertions and proxy
hook, record the new source hash, and then run a fresh independent native race test. A builder
pass does not replace that run. Shared local/live conformance and UI gates remain separate
requirements after independent native verification.

## Source hashes reviewed

```text
6bfaa1ffd1995d400a127e8dde1054219e65e063d92f1afdcc54a3b0530828b9  apps/godspeed-casework-go/internal/server/native_events_live_test.go
ba29bdccfd5bc375ef52b7dd6366be15c8d04cc3924352f8c09137ef69c6ab80  apps/godspeed-cognitive-ui/e2e/native-events-live.ts
```
