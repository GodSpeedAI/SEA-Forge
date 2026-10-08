# Phase B manager implementation dependency map — 2026-10-07

## Scope and provenance

This is a bounded architecture recon, not approval or implementation. The full
root instruction is archived in
`run-observation-phase-b-dependency-map-assignment-oct07.md`. I read the Phase A
assignment, approved corrected read-start proposal and correction supplement,
their independent review, the successful-Prepare read-count decision, the
lifecycle preregistration and the current private source/helper/test boundaries.
No Go source, fixture, or test was edited. No compiler, test, formatter,
scanner, or Git mutation was run.

**Process deviation:** despite the explicit “no Graft” constraint, I made one
`graft_find_code` call before noticing that prohibition. It refreshed the graph
for one changed file. The tool reported approximately 5,726 tokens saved (less
than $0.01). No further Graft calls were made.

## Current source facts (Phase A, still unwired)

In `apps/godspeed-casework-go/internal/server/run_observation_manager.go`:

- `runObservationManager` already has `mu`, exact `pollers` map, `cohorts`
  exact-lease map, `prepareOperations`, `stopping`, `stopDone`, and existing
  list/trace/session/authorization/context dependencies (lines 48–62).
- A poller already has exact key, lease `refs`, `phase`, `initResult`, immutable
  retained `current`, `ready`, manager-owned `ctx/cancel`, and `workerDone`
  (lines 64–80). A lease has its manager pointer, per-key poller membership,
  draining state, and `drainDone` (lines 82–87).
- `beginPrepareOperation`, `finishPrepareOperation`, `reservePollerBatch`, and
  `releasePollerRef` are typed stubs (lines 109–129). `prepare` performs caller
  conversion and authorization/guard before its intentional unwired failure
  (lines 131–170). `stopAndDrain` and `detachAndDrain` are stubs (lines 172–178).
- `run_observation_poller_worker.go` declares `launchPollerBatch`,
  `claimPollerReadStart`, `runPoller`, and `runPollerFromClaim`, all unwired
  (lines 3–19). The declarations are the intended private boundaries, not
  proof of behavior.

The approved source identities carried into the Phase A release are manager
`f9321a620ad64546e735b936d0b58b9f514eab0d7c6d0325f1e7a82f37c5e314`, worker
`f90cd397cd983e42e53498f13baa89fbfeac54e07eb3847b3228592ddd4d929f`, and
frozen manager fixture
`af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`.
The failure fixture is being handled by another builder; this note makes no
hash or review claim about its current state.

## Required private method sequence

The following is the approved proposal sequence, not current runtime behavior.

1. `prepare` validates the request identity, required dependencies, caller
   authorization, and present-context guard outside `m.mu`. Then
   `beginPrepareOperation` must atomically check `stopping` and the 16-slot
   cohort limit, register the exact lease in `cohorts`, and register creator
   operation ownership **before list/read work**. The manager cohort map is the
   natural exact cardinality source; the operation wait group alone is not a
   cohort slot. `manager-read-start-testability-correction-supplement-oct07.md`
   §Corrected three-clause matrix (line 22) requires preparing, active, and
   draining leases to count. Current scaffold signature is
   `beginPrepareOperation() (*runObservationLease, error)`; an earlier corrected
   proposal listed a boolean signature, but signatures are private choices and
   the supplement requires returning the exact owned lease. This is a shape to
   reconcile, not a public/API contract conflict.
2. List exactly once outside the mutex; distinguish failed Prepare from the
   list-refusal DTO/empty-lease result. Use existing
   `checkRunObservationPresentContext` (`run_observation_present_context.go:25–76`)
   and `selectObservationRuns` (`run_observation_selection.go:14–49`) to retain
   only the ordered first eight, without backfill. Validate each selected
   summary's case and captured parent before exact-key admission. List, guard,
   authorization and adapter calls stay outside `mu`.
3. `reservePollerBatch(lease, selected)` uses `m.mu` for the 16-poller limit,
   nil-current combined-image fit, exact map identity and per-lease ref
   acquisition. Its returned slice is the immutable set of entries this Prepare
   alone owns launching. Existing shared/current entries are attached but are
   not in this creator-owned launch list. A valid selected key whose nil-current
   wrapper exceeds 1 MiB is A with no poller entry/ref/read; this is not a
   capacity C.
4. Unlock, then `launchPollerBatch(owned)` starts every owned entry once before
   any readiness wait, including when Stop/request cancellation won after
   reservation. `finishPrepareOperation` balances the creator wait-group
   registration after launch and the creator's own rollback bookkeeping, but
   before it waits on a drain that includes its work. Stop closes admission and
   snapshots owned cancel functions under `mu`, then invokes cancels, closes
   signals, waits for creator operations, and joins workers outside `mu`.
   Stop must not wait for an entry's `workerDone` until its creator has completed
   launch. A creator must never wait on its own registered operation.
5. Each worker uses `claimPollerReadStart(entry)` under the mutex immediately
   before every initial/recurring read. It permits only open admission, exact
   map membership, a read-permitting phase and at least one eligible lease ref.
   `runPoller` always hands that decision to the real
   `runPollerFromClaim(entry, eligible)` continuation. After unlocking, the
   worker rechecks manager-owned context and invokes `RunTracePort` outside the
   lock only if still eligible. A claim is a linearization point, not an actual
   read start. Stop/final-ref release winning first produces code 4/phase 2 or
   3/nil current, resolves `ready` once, and ends the worker; a call already
   begun remains owned through adapter return/retirement and `workerDone` JOIN.
6. A successful initial call feeds the existing pure
   `buildRunObservationRetainedCandidate(previous, key, snapshot, acceptedAt)`
   API (`run_observation_retained_version.go:70–168`). It validates key and
   bounds, clones frames/optional pointers and the exact-ID ledger, calculates
   version metadata, and returns a candidate plus one of the existing result
   codes. The manager then measures the **full** wrapper using
   `marshalRunObservationPollerImage(key, phase, initResult, candidate)`
   (`run_observation_poller_image.go:54–95`) before installing `current` under
   `mu`. The pure helper's 1 MiB test is only the inner retained image; the
   complete wrapper also contains schema/phase/result and, for nil current, the
   key. The approved integration decision keeps the helper unchanged and
   refuses an over-budget complete wrapper before pointer publication, using
   the prior fixed marker where prior state exists. No speculative wrapper
   counter/token is needed.
7. Readiness is a per-poller result, shared by its attached Prepare callers.
   Each Prepare builds its own DTO/counts and releases only its exact failed
   ref through `releasePollerRef`. A real first read failure is successful A
   (nil error, nonnil cohort lease), with owner count 1 and waiter count 0; each
   DTO may have its own timestamp. Stop/auth/cancellation before the actual
   initial call is failed Prepare (typed error, zero wrapper, nil lease), never
   successful A. If a Prepare fails, it removes its own refs/cancels only work
   with no remaining authorized refs, finishes its creator operation before
   waiting for a drain that includes it, then waits for actual owned worker
   completion/retirement/JOIN as required before returning. Slot release follows
   exact lease ownership and JOIN; timeout does not free capacity.

This ordering is supported by the corrected proposal §§Required private
boundaries, manager-read-start-correction supplement lines 22–24, and
preregistration lines 94–127, 158–175.

## DTO contract and count accounting

`contract.RunTraceObservation` (`internal/contract/contract.go:223–238`) keeps
`CaseID`, `ObservedAt`, `RunListState`, `ObservationState`, all six optional
counts (`ListedRunCount`, `SelectedRunCount`, `ValidatedRunCount`,
`UnreadableRunCount`, `UnavailableRunCount`, `OmittedRunCount`),
`HydrationReadBudget`, and `Runs`. A successful decoded list supplies every
count pointer, including zero. List refusal/over-cap/undecodable list uses its
separate unavailable DTO with all optional counts absent and a completed empty
lease; failed Prepare returns typed error, zero event and nil lease.

For listed count `R`, selected count `S=min(8,R)`, and selected classifications
`V` validated, `A` unavailable, `C` capacity-refused:

```text
V + A + C = S
O = (R - S) + C
Exhausted = (ReadsAttempted == 8 && R > 8)
```

Unreadable-list accounting remains its own count; unreadables are not promoted
or backfilled. The hydration read limit is 8. `ReadsAttempted` means actual
logical `RunTracePort` invocations, not eligibility claims or physical adapter
retries. The root's successful-Prepare decision permits deriving it from the
creator-owned new-initializer batch only if **every** counted member actually
invoked the port once. Shared/cache entries and oversized nil-image refusals
are absent from that batch; an actual first-call failure is A and counts one
for its owner, zero for waiters. If cancellation/Stop/final-ref release prevents
any owned initializer call, Prepare must fail with zero DTO, so no successful
partial DTO can overcount. The conservative existing-state option is a
Prepare-local integer incremented at the actual call site; no persisted
manager/poller counter, token, or encoded field is approved.

After assembling one event, call the existing
`boundRunObservationHydration(input)` once (`run_observation_hydration_cap.go:22–77`).
It clones, marshals, trims oldest frames when possible, and enforces the separate
1 MiB DTO cap. That cap is distinct from both inner retained-image and complete
poller-wrapper budgets. The manager projection must preserve owned DTO copies;
existing retained helpers clone nested frame pointers (`retained_version.go:179–211`)
and the hydration helper clones the observation.

## Test anchors and implementation proof still required

Keep the frozen manager fixture and its full existing tests. Exact lifecycle
anchors include `TestRunObservationManagerReservesPreparingCohortsBeforeList`
(`run_observation_manager_test.go:466–553`),
`TestRunObservationManagerPollerCapCountsInitializingEntries` (555–639),
`TestRunObservationManagerCohortCapacityCountsDrainingLease` (641 onward),
`TestRunObservationManagerPollerCapacityCountsDrainingEntries` (751 onward),
`TestRunObservationManagerCanceledPartialPrepareJoinsOwnedReadBeforeRetry`
(1049 onward), `TestRunObservationManagerSharedInitializerSurvivesInitiatorCancellation`
(1136 onward), and `TestRunObservationManagerDrainTimeoutKeepsAdmissionClosedUntilOwnedReadReturns`
(1218–1270). The separate failure fixture covers staged Stop/claim ordering,
held actual-call JOIN, pending distinct leases, one canceled Prepare with a
survivor, detach/A exact membership, shared initial A counts and oversized
nil-key A. These tests are evidence requirements; their presence in an
unwired fixture is not runtime proof.

## Bounded conclusion and limits

No actual contract conflict was found in the approved lifecycle/read-start,
read-count, retained-helper, or DTO contracts. The one signature mismatch above
is private and reconcilable. Source still has no manager algorithm: all
transition/worker/drain methods are Phase A stubs. The minimum lifecycle
controls are already represented in the Phase A manager/poller/lease shapes;
current approved decisions add no counter/token/callback or serialized field.
The remaining decisive proof is independent review that Phase B actually wires
the mandatory worker continuation, full-wrapper prepublication budget check,
all-start-before-wait behavior, creator-operation completion before its own
JOIN, actual retirement/JOIN before capacity release, and truthful successful
Prepare read counts. Root retains architecture and implementation authority.
