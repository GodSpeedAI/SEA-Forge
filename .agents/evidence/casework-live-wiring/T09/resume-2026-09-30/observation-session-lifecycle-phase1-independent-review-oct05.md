# Independent review: observation session lifecycle Phase 1

Date: 2026-10-05. Verdict: **REJECT source readiness**. The frozen fixture has
a guaranteed Go compile error and its concurrency join is unbounded. No source
was changed; no compiler/test/scanner/Git/status/debt command was run. Read-only
`gofmt -l` reports the new test file as not formatted.

## Blocking findings

1. `TestSessionStoreCurrentRacingDestroyNeverMissesRevocation` declares
   `clock, store := newObservationSessionStore(...)` at
   `internal/auth/session_observation_test.go:253`, but never references
   `clock` in that function. Go rejects this local as declared and not used.
   This prevents compilation before any intended `Current` behavioral
   assertion, violating the original Phase 1 requirement that the frozen
   assertion RED be a live-Current failure rather than a compile/setup error.
   The builder record's claim that the fixture is ready for an assertion RED
   is therefore unsupported. Remove the unused binding or use only the store
   result in a new builder pass, then independently re-freeze/review.

2. The same race test releases two workers and calls `workers.Wait()` directly
   (`:256-272`) with no bounded completion path. The workers are simple
   `Current`/`Destroy` calls today, but the fixture is intended to expose
   locking and lookup/deletion races. A deadlock in the implementation under
   test would hang the focused test instead of producing a bounded failure.
   The original assignment requires an intended assertion RED without a
   blocked test/peer; add a bounded join that reports failure and still
   accounts for worker cleanup. This is independent of the compile blocker.

## Coverage and scope otherwise checked

The stub and declarations match the requested shape: `CurrentSessionState`
contains only `Claim`, `ValidUntil`, and receive-only `Revoked`
(`session.go:51-57`); `Current` returns zero/false (`:143-146`). Existing
store methods are unchanged. The fixture covers exact and just-after idle and
absolute boundaries, no-sliding/detached claim, Resolve sliding versus fixed
absolute expiry, all removal paths (Destroy, Resolve expiry, Current expiry,
Sweep expiry, capacity eviction), repeated removal, unrelated-session
preservation, bounded capacity, and a synchronized Current/Destroy race.
Its clock protects reads and writes, and the race test's result variables are
read only after the WaitGroup join. No session credentials or IDs are printed.

The builder-record hashes match the current files:

- `session.go`: `f8f0df88d283ef0c28057f08fcdd6b23c7111c89abd68e23b52f754eb9d17b48`
- `session_observation_test.go`:
  `78cbe26fffc70837eb27ce597872ad9a1dfe7fe70cce745441ae10a721c2c6aa`
- builder record:
  `c9cad7b48a9997bffe6999c3c172512052cccb4a5db45797965ab2809d6e0124`

No scope expansion or implementation of revocation was found. This rejection
concerns Phase 1 source readiness only; no compiler token or later auth-store
implementation is approved by it. Repair through a fresh bounded builder, then
return the new immutable source and record for another independent review.
