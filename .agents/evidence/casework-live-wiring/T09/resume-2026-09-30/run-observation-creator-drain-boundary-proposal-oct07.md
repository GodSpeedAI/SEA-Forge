# Creator/list ownership boundary proposal — 2026-10-07

## Provenance and status

This is a new immutable proposal written under the complete document-only
assignment archived in
[`run-observation-creator-drain-boundary-proposal-assignment-oct07.md`](run-observation-creator-drain-boundary-proposal-assignment-oct07.md).
It is not an implementation approval or runtime proof. Root retains the
architecture decision; a different critic must review this proposal before
any source release.

Evidence basis read for this proposal:

- Original Unit 1 contract: `run-observation-manager-unit1-original-assignment-oct06.md`;
- lifecycle preregistration: `apps/godspeed-casework-go/internal/server/run_observation_manager_unit1_lifecycle_implementation_preregistration_oct07.md`;
- revision 6 addendum and retention corrections 1–3;
- corrected read-start proposal/supplement and root decisions:
  `manager-read-start-testability-corrected-proposal-oct07.md`,
  `manager-read-start-testability-correction-supplement-oct07.md`,
  `manager-read-start-testability-root-decision-oct07.md`,
  `manager-worker-launch-root-clarification-oct07.md`,
  `manager-initial-failure-root-clarification-oct07.md`, and
  `manager-successful-prepare-read-count-root-decision-oct07.md`;
- ref ownership and reverse membership: `manager-signal-ref-ownership-root-decisions-oct07.md`,
  `manager-draining-reverse-membership-root-clarification-oct07.md`, and
  `manager-draining-reverse-membership-independent-review-oct07.md`;
- combined-image constraints: `manager-combined-image-root-decisions-oct07.md`;
- current implementation seams in `run_observation_manager.go` and
  `run_observation_poller_worker.go`; current failure fixture
  `run_observation_manager_failure_test.go`; frozen capacity/drain fixture
  `run_observation_manager_test.go`.

Source facts: current manager has `cohorts`, `prepareOperations`, and
`stopping`; lease has `pollers`, `draining`, and `drainDone`. The current
`beginPrepareOperation` and `finishPrepareOperation` are unwired stubs
(`run_observation_manager.go:109-117`). The worker file provides the actual
private `claimPollerReadStart`, `runPoller`, and mandatory
`runPollerFromClaim` boundaries, currently unwired
(`run_observation_poller_worker.go:7-19`). The frozen capacity test requires a
reverse ref to remain while a canceled actual worker is held
(`run_observation_manager_test.go:879-897`). Four failure-fixture wrappers
currently invoke no-argument finish at lines 131, 181, 224, and 487.

## Proposed smallest private ownership boundary

1. **Reserve creator and lease atomically.** `beginPrepareOperation` checks
   `stopping` and the 16-cohort bound under `m.mu`. On success it constructs
   one exact lease with `creatorDone`, `leaseDone`, and the existing
   `drainDone`; inserts it in `m.cohorts`; and calls
   `m.prepareOperations.Add(1)` before unlocking. On refusal it returns no
   lease and performs no list or trace read. The same mutex orders this
   positive Add before Stop sets `stopping`, so no Add races with Stop's Wait.
   The creator then uses this exact lease throughout Prepare and rollback.

2. **Finish only the owning creator.** Change the private signature to
   `finishPrepareOperation(lease *runObservationLease)`. The creator invokes it
   exactly once after its list call, cancellation bridge, launch batch, and
   any creator-owned rollback work have returned. Outside `m.mu`, it closes
   that lease's `creatorDone` and balances `prepareOperations.Done()` exactly
   once, before waiting on its own drain. Keep once ownership local to the
   creator call path; add no persistent finished flag, token, counter, or
   callback. A creator must never wait for a Stop/Detach operation that is
   itself waiting for that creator.

3. **Bridge a preparing lease to its list context.** Derive a local cancelable
   request context for `RunsListForCase` and the initial readiness wait. Start
   one bounded goroutine per preparing lease that selects between
   `lease.leaseDone` and the local context's `Done`; on lease drain it cancels
   the local context. Join this bridge before creator finish. The cohort limit
   bounds these temporary goroutines. The shared poller context remains
   manager-owned and independent of the request context. Do not retain a
   callback or add a generic test hook.

4. **Detach signals before its waits.** Under `m.mu`, the first detach owner
   marks the exact lease draining, captures its exact entries in local work,
   and clears `lease.pollers` once. That transition claims ownership of closing
   `leaseDone`. After unlocking, close it promptly, cancel only watcherless
   workers, then wait for this lease's `creatorDone` and the required actual
   worker completion/JOIN. Do not wait on the manager-wide creator WaitGroup
   from Detach: another lease may have a separate held list. Timeout preserves
   drain ownership and both capacity reservations; it cannot signal completion
   or release capacity.

5. **Stop signals before global waits.** Under `m.mu`, Stop closes admission by
   setting `stopping`, snapshots leases/work, and claims their drain transitions.
   Outside the lock it closes lease signals and invokes cancels before waiting
   for manager creator registrations, worker completion, and joins. Because
   reservation/Add and stopping are ordered by `m.mu`, no creator can register
   after Stop begins its Wait. All cancellation, channel close, callbacks,
   waits, port I/O, and JOIN stay outside `m.mu`.

6. **Use one eligibility predicate.** Reverse ownership and eligible
   attachment are distinct. Under `m.mu`, read start and shared-worker
   retention require manager admission, exact manager-entry identity, an
   allowed phase, and at least one lease where
   `!lease.draining && lease.pollers[key] == entry`. Never use
   `len(entry.refs) > 0` as eligibility. Detach clears eligibility atomically;
   for a watcherless held worker, `entry.refs[lease]` remains as noneligible
   drain ownership until actual JOIN. On a surviving shared entry, remove the
   departing reverse association without canceling/joining the worker.
   Remove the watcherless reverse association and capacity only after JOIN.
   This is the approved resolution of the apparent frozen-test conflict, not
   a test weakening or second release owner.

## Test-first fixture extension

Keep all nine existing failure-fixture cases and their assertions/order. Patch
only the four function-qualified local finish closures to pass their captured
lease, for example:

```go
finish := func() { finishOnce.Do(func() { manager.finishPrepareOperation(lease) }) }
```

The exact functions are `TestRunObservationManagerFailureStopWinsBetweenReserveAndLaunch`,
`TestRunObservationManagerFailureDetachWinsBeforeFirstReadUsesDrainingCode4`,
`TestRunObservationManagerFailureClaimThenStopUsesWorkerContinuation`, and
`TestRunObservationManagerFailureDetachAndSelectedAReleaseHaveOneOwner`.
Do not change the frozen `run_observation_manager_test.go` or six primitive
files.

Add two tests using the real `RunsListForCase` callback and channels, never
sleep, scheduler hooks, fake port barriers, or private-state writes:

| Test | Staging and required observations |
| --- | --- |
| Global stop owns a held list creator | Start Prepare with a context-aware list callback that signals entry, then waits for both context cancellation and explicit test release. Read-only inspect `manager.cohorts` under `m.mu` to capture its exact lease. Start Stop. Assert the list context is canceled promptly while the callback remains held; cohort/capacity remains reserved, `creatorDone` remains open, Stop has not returned, and another Prepare is refused before list/trace I/O. Explicitly release the callback; assert Prepare returns typed failure with zero DTO and nil lease, creator completion closes, Stop finishes, and its registry is empty. |
| Detach waits for only its own preparing creator | Hold A's real list call and capture A's lease through read-only registry inspection. Start B for the same case with a distinct session; wait for B's real list call and identify B's distinct lease. Detach A. Assert A's list context is canceled, B's remains live, and A detach waits while A's callback is held. Release A; require A Prepare failure and A detach completion while B's callback is still held. Then release B with a valid empty list, require successful B Prepare with a nonnil lease, and perform normal B detach. |

Use the existing bounded channel-wait style (two-second bounds already appear
in the fixtures), not sleeps. These tests demonstrate signal-before-wait,
creator/list completion ownership, exact-lease waiting, and capacity held until
the creator and required actual workers retire. Existing tests continue to
cover actual read JOIN, all-eight-start-before-wait, shared first-read failure
accounting, oversized nil-key A refusal, and retained reverse ownership while
a worker is held.

## Material changes from earlier Phase A and limits

- This corrects the earlier no-argument finish boundary: exact lease is now
  passed so one creator closes only its own `creatorDone`; local once ownership
  remains nonpersistent.
- It adds only the two explicitly required lease channels and exact lease
  parameter, alongside existing cohort, draining, maps, and manager WaitGroup.
  No new public API, DTO/image field, enum, identifier rule, cap, callback,
  scheduler hook, or speculative counter is proposed.
- The new failure fixture is the sole fixture exception for four exact finish
  call shapes and two tests. Frozen manager fixture and six primitive files
  remain unchanged. All nine existing failure cases are retained.
- Reverse membership is retained through JOIN only as drain ownership; the
  eligible map is cleared at detach. This temporal distinction is required by
  both the frozen held-worker assertion and the independently approved exact
  release protocol.
- These are proposed behaviors and private boundaries, not evidence that the
  unwired current stubs implement them. No tests, compiler, formatter, or
  runtime gates were run for this proposal. The exact channel-close and Wait
  ordering, plus both held-list tests, remain for independent review and later
  implementation evidence.

## Review result

No unresolved contradiction was found in the cited contracts after applying
the approved reverse-membership clarification. The proposal preserves the
original actual-call read accounting: eligibility alone is not a read; a
successful initial A requires the actual initial call, while stop/cancel before
that call is typed failed Prepare with zero DTO and nil lease. Root retains the
final architecture decision.
