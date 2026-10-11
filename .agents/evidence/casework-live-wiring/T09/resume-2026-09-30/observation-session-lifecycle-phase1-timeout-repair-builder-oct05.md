# Observation session lifecycle Phase 1 timeout repair builder record

Date: 2026-10-05. Fresh bounded source-only repair after the independent
review rejected the five-second race-test timeout as per-iteration rather
than test-wide. This record requests fresh independent review; it is not an
approval or assertion-RED claim.

## Original assignment and binding instructions

The original Phase 1 assignment, preserved in
`observation-session-lifecycle-phase1-builder-oct05.md`, was to read the
approved root proposal, its independent review, existing auth session/store
tests and identity type, then own only `internal/auth/session.go`, a new
`internal/auth/session_observation_test.go`, and an immutable record. Add the
detached `CurrentSessionState` shape (`Claim ports.ActorClaim`, `ValidUntil
time.Time`, `Revoked <-chan struct{}`) and temporary `Current(id string)`
stub returning zero state and false. Do not implement revocation or alter
Start, Resolve, Destroy, Sweep, or eviction. Add focused coverage for
non-sliding detached reads; strict idle and absolute boundaries; Resolve
sliding idle but fixed absolute expiry; every removal path and stable channel
closure; repeated deletion, unrelated-session safety, capacity bounds, and
synchronized lookup/delete races. The stub's failure must be at a live-Current
behavioral assertion, not setup, compilation, or a blocked peer. No test,
compile, scanner, Git, hook, status, or debt commands.

The follow-up repair assignment was to change only
`apps/godspeed-casework-go/internal/auth/session_observation_test.go` after
the review finding that its five-second timer was recreated in each of the
16 race iterations. Keep all 16 iterations, the bounded buffered completion
join and its happens-before guarantee, and all assertions and lifecycle
coverage. Use one shared five-second timer for the entire race test, created
before the loop and stopped by a defer. Do not edit production source; the
approved `session.go` hash must remain
`f8f0df88d283ef0c28057f08fcdd6b23c7111c89abd68e23b52f754eb9d17b48`. Do not
compile, scan, run tests, or perform Git/status/debt operations. Freeze the
source and provide a new immutable builder record containing the original
instructions, full actual diff, hashes, and deviations for independent review.

The proposal approval is limited to the auth-store prerequisite. It does not
approve the later SSE/poller implementation, cancellation/drain behavior, or
full Unit 5C. Timeout reports failure but cannot cancel a goroutine blocked in
these context-free store methods; the test makes no claim that it can forcibly
terminate such a goroutine.

## Source identities

Before this repair, the frozen fixture SHA-256 was
`5bbc19a4f1719defd79c564623603a64e2daa391a7db9ebe04821b94c34dd934`, matching
the fixture-repair record and review. After this repair, the fixture SHA-256
is `8de408be7efc31d6e19d73870bee541d82f127272b5c2ab048dbee6ba3212e3f`.
Production `session.go` remains SHA-256
`f8f0df88d283ef0c28057f08fcdd6b23c7111c89abd68e23b52f754eb9d17b48`.

## Full actual source diff

```diff
 func TestSessionStoreCurrentRacingDestroyNeverMissesRevocation(t *testing.T) {
+	timer := time.NewTimer(5 * time.Second)
+	defer timer.Stop()
+
 	for i := 0; i < 16; i++ {
@@
 		}()
 		close(start)
-		timer := time.NewTimer(5 * time.Second)
-		defer timer.Stop()
 		for worker := 0; worker < 2; worker++ {
```

One timer now bounds all 16 iterations cumulatively. If an iteration times out,
`t.Fatal` exits the test and runs the deferred `Stop`; each worker reports
completion through the existing capacity-two channel, and a successful test
still receives both completion values in every iteration before reading
`got`/`found`. The start barrier, race outcomes, assertions, and lifecycle
matrix are unchanged.

## Deviations and verification boundary

No material deviation. Only the assigned test file and this new record were
written. No production source, dependency, identity, lifecycle behavior,
assertion, iteration count, join structure, evidence from a test run, or
unrelated file was changed. No compiler, test, scanner, Git, hook, status, or
debt operation was run. This source-only handoff makes no compilation, RED,
runtime, or test-pass claim; it awaits a fresh independent review.
