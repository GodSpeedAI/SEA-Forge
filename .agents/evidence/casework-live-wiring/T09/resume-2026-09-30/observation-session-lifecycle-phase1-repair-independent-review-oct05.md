# Independent review: repaired observation session lifecycle Phase 1

Date: 2026-10-05. Verdict: **REJECT source readiness pending the specified
test-wide timeout correction**. The original unused-local compile blocker is
fixed and the file is gofmt-clean. No source was changed; no compiler/test/
scanner/Git/status/debt command was run.

## Review evidence

The repair record states the original test SHA-256
`78cbe26fffc70837eb27ce597872ad9a1dfe7fe70cce745441ae10a721c2c6aa` and
repaired file hash
`5bbc19a4f1719defd79c564623603a64e2daa391a7db9ebe04821b94c34dd934`; both
match the current files. `session.go` remains at the approved Phase 1
declaration/stub hash `f8f0df88d283ef0c28057f08fcdd6b23c7111c89abd68e23b52f754eb9d17b48`.
The only functional test edit is `clock` → `_` in the Current/Destroy race
fixture. `gofmt -d` emits no changes.

The complete lifecycle matrix remains: detached claim and non-sliding
`LastSeen`; earliest deadline; exact and just-after idle and absolute expiry;
Resolve sliding idle with fixed absolute expiry; stable channel closure for
Destroy, Resolve expiry, Current expiry, Sweep expiry, and capacity eviction;
idempotent removal, other-session preservation, capacity bound; and concurrent
Current/Destroy lookup. The synchronized test clock guards all reads/writes.
In the race test, each worker writes `got`/`found` before sending to the
capacity-two `completed` channel; receiving two completion values establishes
the happens-before edges before the main goroutine reads those results. The
start channel releases both workers together. Declarations/stub and identity
scope remain unchanged; no removal path was omitted.

## Remaining blocker and cleanup limit

In `TestSessionStoreCurrentRacingDestroyNeverMissesRevocation`, lines 252-284,
the five-second timer is created inside the 16-iteration loop (lines 262-263).
That bounds each iteration separately, not the entire race test to the
five-second total timeout required by the fresh review assignment. A slow but
non-deadlocking sequence can therefore consume up to roughly sixteen timeout
windows. Use one deadline/timer for the whole test (or one shared absolute
deadline) before this fixture is source-ready.

If an implementation truly deadlocks inside `Current` or `Destroy`, the
timeout can fail the test but cannot cancel a goroutine blocked on the store
mutex: these APIs take no context. The completion channel is buffered, so a
worker that eventually returns will not then block trying to report
completion. The test must not claim that timeout cleanup forcibly terminates a
deadlocked worker. This limitation is inherent to this fixture and is not an
additional requested source change unless a safe cleanup seam is available.

The prior rejection's unused variable and formatting findings are resolved.
No other material deviations from the original Phase 1 assignment or approved
session lifecycle proposal were found. Approval remains limited to test/stub
source readiness, not runtime revocation, server SSE integration, or pending
read cancellation/drain.
