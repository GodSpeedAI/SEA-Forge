# T08 native live retry repair handoff

## Assignment and rejected gap

This repair addresses the independent static rejection of the first native EventSource
builder. The rejection identified that phase one closed its actual stream, awaited one
`onError`, disposed immediately, and asserted the request count stayed at one. The original
phase-one requirement also called for an authorized kernel mutation while disconnected and a
real native retry that resynchronizes from interrupted cursor K to retained cursor L.

The original assignment remains recorded in `native-live-builder.md`. This repair changes only
`apps/godspeed-casework-go/internal/server/native_events_live_test.go` and
`apps/godspeed-cognitive-ui/e2e/native-events-live.ts`; this note is the only new evidence file.
The retention stack helper, production retention, EventSource adapter, Vite configuration,
dependencies, server wiring, and phase-two retention-two proof were left untouched by this
repair.

## Source changes and material differences

Phase one now keeps the authenticated browser subscription alive after the first forced close.
After observing the first native `onError`, it fetches the authoritative snapshot at K,
requires that `ADD_DISCRETIONARY_WORK` is offered, and sends that intent through the real
session-authenticated HTTP adapter with K as its client cursor. It verifies the mutation is
accepted and produces one durable `plan_mutated` trace and a new retention-one head L.

The test harness now holds the second actual `/api/events` request before passing it to the
kernel-backed API. This test-only gate makes ordering deterministic when the fixed 1.5 second
browser retry expires before the authorized mutation finishes. The browser observes the real
retry request and its `last=K` query. It then calls a private harness release endpoint; that
endpoint waits until the durable case event count and relay/store head remain stable for at
least 250 ms before releasing the request. The request then reaches the real API and replays
the retained window. No SSE frames, kernel outcomes, snapshots, or errors are synthesized.

The browser asserts the retry delivers exactly `resync_required(L)` followed by `snapshot(L)`
at the same cursor. The snapshot must match both authenticated `Store.At(L)` and a fresh
authenticated current snapshot and show the newly authorized plan item. The browser then
forces a second close, waits for the second native error, disposes while the retry timer is
pending, waits longer than the 1.5 second retry cap, and checks that no third request,
subscriber, stream, or callback appears.

The existing phase-two retention-two scenario remains the same: its genuine mutation retains
C and appends L, reconnects from C, replays only `snapshot(L)` without a resync event, and
disposes the live stream. Its durable `plan_mutated` assertion and request-count proof remain.

## Source assertion anchors

- Go test request gate and actual request tracking: `native_events_live_test.go`,
  `serveAPI` and `nativeEventHarness.ServeHTTP` (`/api/events`, `/__native-test/release/`).
- Go stable-head barrier: `nativeRelaySettled`, which compares durable event count and relay
  head to the projection-store head until stable.
- Go final phase-one evidence: `assertPhase1KernelEvidence` checks C→K→L ordering, one durable
  plan mutation, exact event order/cursors, two errors, exactly two event requests, reconnect
  from K, retained `Store.At(L)`, and eviction of K under retention one.
- Browser phase-one mutation/retry path: `native-events-live.ts`, `phaseOne`; it verifies the
  action is offered, uses the authenticated adapter, checks the actual pending retry query,
  releases only after mutation completion, and compares the streamed L snapshot with
  `getSnapshotAt(L)` and fresh `getSnapshot`.
- Browser terminal disposal path: `phaseOne` second disconnect and the 1.75 second wait after
  disposal.
- Retention-two preserved path: `native-events-live.ts`, `phaseTwo`, plus Go
  `assertPhase2KernelEvidence`.

## Verification state

No Go test, Bun command, browser command, compiler, build, vet, or runtime check was run by this
builder. `git diff --check` is permitted for the parent handoff but does not establish Go,
TypeScript, browser, or live-kernel correctness. The source is ready for the requested fresh
`t08_final_static` review. Full Bun/live verification remains pending with the root-held
compile token, and no passing result or acceptance is claimed here.
