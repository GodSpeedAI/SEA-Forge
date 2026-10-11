# T08 independent final confirmation — 2026-09-30

## Verdict

**Independent confirmation approves T08.** The authorized shared adapter suite, live
conformance, native browser EventSource recovery and retention proofs, and canonical UI gate
pass. I found no production API, authority, schema, dependency, or adapter implementation
change in the T08 repairs under review. Root retains task settlement, status, and commit.

## Scope and source review

T08's original plan requires the HTTP `CaseworkPort` to exercise the shared local/live
conformance suite, keep production bundles HTTP-only for default/local/invalid source
selection, surface typed stale refusals without replaying intents, and prove real server
history/SSE behavior. The T08 test helpers also require real native `EventSource`, authenticated
session use, retained cursor proof, actual stream drops, retries, and disposal.

The reviewed changes are scoped to the shared conformance tests, isolated live/browser
harnesses, and live test stack. The retention configurable stack helper was changed before the
recorded global Go gates. The native Go test has the `live` build tag and was independently
compiled and run below. No production Go handler, HTTP adapter, UI runtime, public contract, or
dependency changed in this review.

The shared replay assertion now permits an ordered retained prefix before the exact accepted
receipt cursor: it waits for the first cursor at or beyond the receipt, rejects a cursor that
passes the receipt, and validates strictly increasing cursors from the subscribed snapshot
through the exact receipt. This is an intentional correction to `Store.Subscribe`'s retained
chronological replay behavior. The live-only branch and other suite assertions remain intact.
Focused fixtures accept a valid prefix followed by the receipt, reject a prefix without the
receipt by timeout, and reject unordered prefix frames. The local fixture retains its real
dispatch/first subscription; only the replay delivery is controlled.

The native snapshot helper compares every schema-defined stable snapshot field, the full
sorted `visible_objects` values including optional object extensions, actions, attention,
perspective, and `x`. Historical SSE snapshots still compare exactly, including timestamp,
against authenticated `Store.At(K/L)`. Each fresh `/world` read must first have the expected
K/L cursor; current equality uses the same stable-field helper on both sides and separately
validates the returned timestamp. It excludes only the per-render timestamp. The preceding
failed comparison log contains no field-level diff, so the actually differing field in that
rejected run remains unknown; source establishes that `/world` gets a fresh render time, but
that cause is not claimed as an observed fact. The rejected raw log is preserved byte-identical
as `native-current-comparison-reject.log`.

## Native live EventSource proof

The independent command ran serially on the approved actual host with `GOMEMLIMIT=256MiB`,
`GOGC=50`, `GOMAXPROCS=2`, `GOFLAGS=-p=1`, the existing writable Go cache, `-p=1`,
`-parallel=1`, `-race`, the `live` tag, and `-count=1`. Preflight found 2,606,032 kB available,
no active Go/Cargo/Bun/Vite compiler, an executable existing `sea-forge-server` target, and a
writable Go cache. Exit 0; 35.679 seconds. Raw log:
`native-full-comparison-final-independent-green.log` (SHA-256
`7005c167d2c8faea764ff8f74fdd65f3bf1f6f6a37a6449781be663896e571fc`). The test has no skip
path: missing `agent-browser` is fatal, and both phase runners and their kernel evidence
assertions execute unconditionally.

The test uses isolated real cell/kernel and server stacks, Chromium through `agent-browser`,
the HTTP adapter, and the browser's native `EventSource` (explicitly checked as native). The
browser logs in and uses the authenticated session for snapshot and EventSource requests; no
cookie values are recorded. A private Vite proxy destroys an incomplete downstream SSE
response when the real upstream stream aborts, because closing the Go-side connection behind
the proxy alone did not trigger browser `onerror`. This is test-only fault injection at the
owned proxy: there is no EventSource mock, fabricated feed/error, or production transport
change. The native error callbacks, reconnect requests, cursors, and event counts remain strict.

Retention one starts at sole retained C, executes the actual `task_prepare` item, verifies
durable activation/settlement/completion and K, and receives exactly `resync(K), snapshot(K)`.
The streamed K matches the full authenticated `Store.At(K)` snapshot. After a real downstream
drop and native error, one protected `ADD_DISCRETIONARY_WORK` mutation is committed while the
stream is down. A retry from K is held until the relay has settled L; the test verifies one
`plan_mutated` durable delta, then exact `resync(L), snapshot(L)`, full `Store.At(L)` equality,
and fresh same-cursor stable-field equality. K is evicted and L is the sole retained revision.
The second real drop disposes the adapter while its capped retry is pending; after waiting
longer than the retry cap, there is no third request, active stream, subscriber, or post-dispose
callback.

Retention two starts from C while preserving its predecessor, establishes one actual stream,
drops it, commits one governed plan mutation to L while disconnected, and verifies no retry
request had started at the point L was retained. The actual retry then uses C and delivers only
`snapshot(L)`—no resync—while both C and L remain queryable. The test checks the real retained
Store revisions, one durable plan-mutation delta, one native error/reconnect, the exact replay
cursor, and complete disposal without an extra request. Thus the frame is retained replay,
not merely a possible future live delivery.

The private-proxy hook expects exactly two incomplete downstream closes in phase one and one
in phase two. Native source assertions verify the expected callback/request totals and final
kernel/store state. The test's successful execution with no skip path therefore covers both
phases and all of those assertions, even though normal `go test` output is non-verbose.

## Shared live and UI gates

The independent shared live command was `CASEWORK_LIVE=1 bun run e2e/live-conformance.ts`.
It exited 0 using the owned isolated gateway/cell runner. Its sanitized transcript records
health and readiness, anonymous refusal, operator session/login, template preflight, PROPOSE,
target-specific action offers, the exact accepted-cursor replay and retained history checks,
typed `STALE_PROJECTION` refusal with unchanged durable trajectory, and a separate R-SO session.
The suite invokes dispatch once for the stale case and the adapter does not retry that typed
refusal; there is no explicit raw transport-request counter for that particular refusal, so
this review does not claim a measured HTTP request count. The runner verifies its own gateway
exit and removes only its successful owned temporary run root. Transcript:
`shared-live-independent-green.log`, SHA-256
`377a1ef9b27a12238d03074521f32160b94e2ef3f58ea5d81c83daad4d82a707`.

The canonical `XDG_RUNTIME_DIR=/tmp just casework-ui-check` first failed in the ordinary
sandbox because Bun's explicit loopback-listener test received `EPERM`; its TypeScript and
production build stages passed. The same canonical gate then ran in the user-authorized direct
host context and exited 0: frozen install, `tsc --noEmit`, pinned `NODE_ENV=production` build,
and all 258 tests / 1,271 assertions passed. The loopback test passed without alteration. Raw
pass log `canonical-ui-final-directhost-green.log` has SHA-256
`a3016de8af46305adaaafe9b13b523dd4ed3dac48f1fd401a4ece4b5cc487cf8`.

The three canonical production builds for default, `VITE_CASEWORK_SOURCE=local`, and invalid
source each exited 0. Their outputs are byte-identical and contain the HTTP adapter without
local adapter, fixture, Northstar, or local narrator markers. These use the package's pinned
production build script; earlier raw Vite commands inheriting `NODE_ENV=development` are
retained as diagnostics and are not counted as production proof. The canonical default build
was also rerun by `casework-ui-check`.

## Reused global evidence and limits

- The four-crate Rust gate passed 594 tests, 0 failed, 4 existing ignored. T08 changed no Rust
  source; see T07's independent final gate record and its `09-four-crate-cargo-test.log`.
- T07 `go vet ./...` and full module `go test -race -count=1 ./...` passed after the T08
  `internal/livestack/stack.go` test helper change (its source timestamp precedes those logs).
  The only new Go test source is `-tags=live`; it has its fresh independent pass above.
- The local browser ladder's prior 11/11 journeys (78/78 steps) is reused from
  `T08/resume-2026-09-29/local-ladder-round8/results.md`; T08 changed harness/tests, not UI
  runtime code, matching the plan's baseline warning to rerun the local ladder only when UI
  runtime changes.
- Canonical default/local/invalid production build logs are preserved under this directory.
  No raw development-mode build is counted.
- T07's live authorization/session/CSRF/origin/history/revoked-replay teeth remain supported
  by its independent verification record; T08 does not alter that production server code.
- `just casework-e2e-live` is introduced by T10 and is not a T08 gate. T09 source preparation
  remains unimplemented and outside this confirmation.

## Source and evidence manifest

```text
c7e79435774e006063288a63b1f72b3e52f47fd2f4769cc2559bb240707330b6  apps/godspeed-cognitive-ui/e2e/native-events-live.ts
a8f7db6735ca6483357fd324bd18406cf4db35a8831e448d48931aff9f4cacb3  apps/godspeed-cognitive-ui/e2e/live-conformance.ts
07dbef23421395a251d6526febdab30b0ec51d0c87dc68f7a57d0de1c0fcc35b  apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.test.ts
b2514a8ec08c077a9c250394a58ec68219ed1e8bef3f80c6f848a8fad39b3366  apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts
c7530522053e952fd36c13a55af7ea44e62f16275e26dd583b88b3d0b9a270b1  apps/godspeed-casework-go/internal/server/native_events_live_test.go
de60d75f3cb91aab30726545355b93cd612f1d94b2cc5f771c52c101525c769a  apps/godspeed-casework-go/internal/livestack/stack.go

7005c167d2c8faea764ff8f74fdd65f3bf1f6f6a37a6449781be663896e571fc  native-full-comparison-final-independent-green.log
1bafbefee2a80fed98b862e55bda64df09f3ba0459bbfdaeb533781542233aa6  native-current-comparison-reject.log
377a1ef9b27a12238d03074521f32160b94e2ef3f58ea5d81c83daad4d82a707  shared-live-independent-green.log
a3016de8af46305adaaafe9b13b523dd4ed3dac48f1fd401a4ece4b5cc487cf8  canonical-ui-final-directhost-green.log
e72d2bbba3c8dbeb0b5228520115c1938e9687be2f6cc12c334012e2e19e1c31  canonical-production-default.log
2d58913e6a6f25af77db6afe55b7e50277408d95d58ed8eb4e5c7f82ab180a09  canonical-production-local.log
5504cf3221672f8add9c89c33341c6068fc2c46b030f69f93b9e8080a943610b  canonical-production-invalid.log
```

