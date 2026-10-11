# Independent source review: run-observation manager Unit 1 TESTFIRST scaffold

Date: 2026-10-06  
Reviewed source/test identities: `run_observation_manager.go` SHA-256 `ee7afab1b22c011cf79b602f415336d459fda346aad1bf0c942a4464326ff878`; `run_observation_manager_test.go` SHA-256 `2b2d1fbcd3bfb7e7c27254673ca46b2deb30d9edd4ac862fa464e2a9b18334b6`.  
Reviewed assignment: `run-observation-manager-unit1-original-assignment-oct06.md`, SHA-256 `de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916`.  
Disposition: **REJECT as ready for the assigned full focused RED.** The scaffold is intentionally unwired and no manager implementation is claimed, but it adds a prohibited test-only production hook, omits direct draining-capacity coverage, and some lifecycle fixtures fail at channel setup time before their semantic assertions can run.

## Inputs and scope

I read the full Unit 1 original assignment and builder record, the root private design decision and its independent review, and the frozen revision 6 proposal/addenda/corrections and response-line-cap erratum. I read the complete two Go files and checked their current hashes against the builder record. I ran no test, compiler, scanner, formatter, Git command, or runtime gate.

## Blocking findings

### 1. `afterAttach` is a test-only production hook

`run_observation_manager.go:55-57` adds an `afterAttach` callback field explicitly described as a “deterministic test barrier.” The shared-initializer test assigns it at `run_observation_manager_test.go:623-628`. The original assignment explicitly forbids new test-only production hooks. The fact that the current stub does not call this field does not remove the production seam from the proposed source; a future implementation would need to invoke it for the test to work.

Remove the callback field and its use. Keep synchronization within the private manager contract and deterministic fake ports, without adding a manager callback solely to coordinate tests.

### 2. Tests do not prove cohort or poller slots remain counted while draining

`TestRunObservationManagerReservesPreparingCohortsBeforeList` (`run_observation_manager_test.go:298-388`) proves 16 preparing reservations and 16 completed active leases consume the proposed cohort cap. It then detaches them synchronously after their work has completed. It never attempts a seventeenth cohort while a cohort is draining on owned work.

`TestRunObservationManagerPollerCapCountsInitializingEntries` (`:390-484`) proves that 16 initializing entries block another selected run. It does not exercise a stopping/draining poller whose read/client retirement or worker join has not completed. The canceled partial-Prepare test (`:515-594`) checks same-key retry ordering after one failed rollback; it does not prove the global 16-entry capacity remains occupied by a draining entry. The stop-timeout test (`:679-730`) closes manager admission globally, so refusal there does not distinguish draining-slot accounting from manager-wide stop.

Add bounded fixtures with different keys that put the 16th cohort and 16th poller entry into draining while owned work is held. Attempt another cohort/poller before release and assert refusal/capacity-limited behavior without a new list/read. Release the fake call, join, then prove capacity is released only after completion. Keep channel ordering explicit; do not use sleeps to establish the transition.

### 3. The shared-initializer fixture cannot reach its semantic assertions against this stub

The `prepare` stub returns `run observation manager unit 1 is not wired` at `run_observation_manager.go:119-120` without invoking the fake list or trace ports. In `TestRunObservationManagerSharedInitializerSurvivesInitiatorCancellation`, the first wait is for the fake trace's `readStarted` signal (`run_observation_manager_test.go:641`); it cannot occur with this stub. Even if that wait were removed, the second-ref synchronization depends on `afterAttach`. Thus the fixture fails at its timeout/setup precondition rather than reaching its assertions about owner cancellation, one shared read, and a surviving waiter. The other channel-driven lifecycle tests likewise depend on list/read callbacks that the scaffold deliberately never reaches.

This does not invalidate those cases as future implementation requirements, but the current suite cannot be presented as an actual semantic RED for those behaviors. Remove the prohibited hook and either make the focused RED command exercise a deterministic assertion the stub can reach, or revise the test-first seam so the lifecycle fixtures reach explicit semantic assertions without test-only production instrumentation. Do not claim a behavioral failure for the shared-initializer or draining contracts until the corresponding assertions are reached.

## Requirements already represented

- The source remains unexported and unwired; `prepare`, `stopAndDrain`, and `detachAndDrain` clearly return a typed unavailable stub rather than claiming an implemented manager.
- The caller snapshot holds only source, session ID, and actor claim; it has no bearer/CSRF token or mutable session pointer. Its test exercises the bearer/no-session snapshot.
- The private list, trace, session-current, perspective, and present-context seams use existing repository return types. The prepare path invokes injected authorization and the existing guard before its deliberate unwired response.
- The tests cover empty observation shape, valid and stale present context, parent filtering, trace identity mismatch, preparing/active cohort limit, initializing poller limit, failed-Prepare nil-lease behavior, partial cancellation with held read, shared initializer survival, timeout and repeated stop/detach calls.
- The files do not implement `Next`, SSE, caller wiring, the ID ledger, retention budget, polling cadence, or public behavior, as required for this unit.

These checks do not cure the three blocking findings above. The present source stub is not source approval and no actual RED is asserted here.

## Verdict and next step

Preserve the frozen files and this review. A fresh bounded correction should remove `afterAttach`, add direct cohort/poller draining-capacity tests, and make the shared-initializer lifecycle fixture reach its semantic assertions without that production hook. Then a separate compiler owner may run the specifically authorized focused RED; review the joined output before any production manager implementation is released. No algorithm, integration, public stream, or T09 settlement is approved.
