# Corrected manager read-start lifecycle proposal

Date: 2026-10-07  
Status: documentation and test-first proposal only. No source or fixture release.

## Decision and evidence basis

This is a fresh proposal after the independent rejection of
`manager-read-start-testability-proposal-oct07.md` (SHA-256
`2700427c0e63f9780c47062fba2f5c7d04c07992551498b6323a015ed08790bb`) and its
review (SHA-256
`2a00b7bf44f0039f27ea6231c544cbf9e00061060660a8d53919dd02f8043ffb`). It
applies `manager-worker-launch-root-clarification-oct07.md` without rewriting
either prior record. The root clarification specifies code 4 / phase 2 or 3,
nil current, one readiness resolution and zero actual calls when cancellation
prevents the first port invocation; it also requires every newly reserved
worker to launch exactly once, even if Stop wins before launch.

The governing contract remains Unit 1 original assignment
`de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916`, lifecycle
preregistration `f48df1c85ca67ce8dafa3d21b82f08a8c39aae1ec710d4ee78ce617e215869f5`,
revision 6 `60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b`,
addendum `9439cbfe8cb30fdf7b1beb41c617ff031ab383296b220317a8b7f529ce3b859a`,
retention corrections 1/2/3
(`449c73301de85794e9aec3093e82a88fdba8c60e46f207cd30454a15f1ce0524`,
`3494e3de99b2f5d3fe725a85971dccd0d70cef226da257ce237cce7fba2d72e1`,
`b8077e9b086f4fe3f15b9733ad3f1cb6fc6f87ca22c5a989494216c50e26d15e`), root
ref-ownership decision `5b9888e5a7212b8d0696f790822f08e718dd03a71f51b28c2bec2ea0098f8df4`,
and root read-eligibility decision
`e1b1db0e7155ff162869c54845d0251783c72e4ed2837f3961ac91e58df602ac`. The
independent review of the eligibility decision is
`0e6f3385f703aaf3eeb5832b7b66414dec2d85984afafea3f2f1e4b9a830612c`.

Current identities rechecked before this proposal: manager scaffold
`run_observation_manager.go` SHA-256
`ab9f1c35757aecd8c8c02de300a95205a1bd07aa5e39ebe8ae056dbc8878fe62`; frozen
manager fixture `run_observation_manager_test.go` SHA-256
`af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`; extracted
key `run_observation_key.go` SHA-256
`a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b`; encoder
source `run_observation_poller_image.go` SHA-256
`3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9`; encoder
fixture `run_observation_manager_retained_image_test.go` SHA-256
`cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256`. The
root clarification explains the fixture transition from `8e741...` to
`cf501...` and manager declaration move from `fa1601...` to `ab9f1c...`;
historical evidence remains untouched.

## Minimum private production boundary

Implement the lifecycle in three paths only after separate root release and
independent source review:

1. Modify `apps/godspeed-casework-go/internal/server/run_observation_manager.go`
   for manager lifecycle state and the real reservation, lease, and Stop
   transitions.
2. Add
   `apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go`
   for the sole worker and its real read path.
3. Add
   `apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go`
   for the new lifecycle matrix.

The manager source is the only existing source exception in this lifecycle
unit; the other two are new files. Keep the existing manager fixture byte
identical at `af2df...`. Keep `run_observation_key.go`, the pure retained helper
and its fixture, the policy fixture, the image encoder and its fixture
byte-identical at the hashes above and their existing recorded hashes. Do not
edit a caller, public surface, DTO, schema, code range, byte limit, cap,
physical adapter policy, or dependency.

The smallest production method set is:

```go
func (m *runObservationManager) beginPrepareOperation() bool
func (m *runObservationManager) finishPrepareOperation()
func (m *runObservationManager) reservePollerBatch(/* selected rows, lease */) ([]*runObservationPoller, error)
func (m *runObservationManager) launchPollerBatch(owned []*runObservationPoller)
func (m *runObservationManager) claimPollerReadStart(entry *runObservationPoller) bool
func (m *runObservationManager) runPoller(entry *runObservationPoller)
func (m *runObservationManager) runPollerFromClaim(entry *runObservationPoller, eligible bool)
func (m *runObservationManager) stopAndDrain(ctx context.Context) error
```

Names and signatures are private proposal details, but the boundaries are
required. `beginPrepareOperation` acquires the manager mutex, refuses after
Stop has closed admission, and registers the creator operation before any
reservation. `finishPrepareOperation` balances that registration exactly
once after the Prepare has launched all owned workers and completed its own
rollback bookkeeping, but before it waits for a drain that includes its work.
`reservePollerBatch` uses that same mutex to create/attach exact
keys and returns an immutable list containing every entry for which this
Prepare owns the sole launch. No other Prepare or Stop launches those entries.
`prepare` must call `launchPollerBatch` outside the mutex exactly once for the
whole list before waiting on any `ready` channel or returning/rolling back.
This remains true when Stop or request cancellation won after reservation.

`stopAndDrain` sets `stopping` and snapshots/cancels under the lock, then
cancels, waits for creator operations, and joins workers outside it. It must
not wait for `workerDone` for a reserved entry until the creator has completed
its launch batch. The creator finishes its registered operation before
waiting for any drain it initiated; it never waits on its own operation or
uses a rollback wait that includes itself. Stop keeps reservations counted
until actual worker completion and JOIN. Repeated Stop/detach callers recheck
the state and use the existing drain ownership rather than independently
releasing a lease's refs.

`claimPollerReadStart` is the accepted eligibility linearization point. Under
`m.mu`, it allows a read only when manager admission is open, exact map
membership still identifies `entry`, the phase permits reading, and at least
one eligible lease ref remains. It does no read, cancellation, signal, send,
wait, or JOIN; it does not increment `ReadsAttempted`. The worker calls it
before each initial and recurring read. After unlock, the worker rechecks its
manager-owned context; only the actual `ReadRunTrace` call increments the
logical attempt count. All adapter calls and retirement remain outside `mu`.

`runPollerFromClaim` is the worker's actual continuation after the eligibility
decision. `runPoller` obtains the decision and immediately hands the result to
this production continuation; tests may sequence the same two methods to
place Stop between them. This is a private worker boundary reused by runtime,
not an injected callback, scheduler hook, sleep, fake pre-call port barrier,
or test-only field. The continuation owns the real result/readiness path and
worker completion. No persistent claim/token field is needed.

The only additional persistent lifecycle state proposed is:

- `runObservationManager.stopping bool` and a creator-operation
  `sync.WaitGroup`, both coordinated with `m.mu`. A positive Add occurs under
  that mutex before reservation; Stop sets `stopping` under it before calling
  Wait outside it, so no later Add races with Wait.
- On each existing poller, `phase`, `initResult`, and `current`, using only
  the already approved fixed phase/result codes and retained-state pointer.
  Existing `ready`, `ctx/cancel`, `workerDone`, exact `key`, per-lease `refs`,
  and lease membership continue to provide ownership. No read-start counter,
  initializer generation, generic token, callback field, extra worker, or
  serialized wrapper field is introduced.

If the first eligibility check fails, or succeeds but cancellation prevents
the actual first call, the worker commits no current value, records existing
initializer code 4 and phase 2 (stopping) or 3 (draining), resolves `ready`
once outside `mu`, and completes/closes its actual `workerDone`. The Prepare
returns typed stop/cancellation error with zero DTO and nil lease after its
owned rollback. Its `ReadsAttempted` remains zero. A cancellation that prevents
a later read does not replace an already accepted current value with this
initializer failure. When a port call has actually begun, its claim remains
owned until the call returns, existing retirement completes, and the worker
is JOINed. A prior claim may physically enter the port after Stop's wall-clock
request; no later claim may pass after Stop wins the mutex. This is the
linearization semantics accepted by the root and independent reviewers.

## Compile-safe, test-first matrix

The new failure test file exercises production methods above. It does not
modify the frozen manager fixture or set lifecycle fields directly.

1. **Stop wins before eligibility, through real reserve/launch.** Register a
   creator operation; use `reservePollerBatch` to obtain one actual owned
   initializing entry and immutable launch list, but do not launch yet. Start
   real `stopAndDrain`; synchronize by observing its actual `stopping` state
   under `m.mu`. Then use the creator's real `launchPollerBatch` and finish
   its operation. Assert one launch, `claimPollerReadStart` refusal, no port
   invocation, code 4, phase 2 or 3, nil current, one `ready` resolution,
   worker completion/JOIN, and retained capacity until the join. Stop cannot
   return before creator completion and actual worker completion. This is the
   production reservation-to-launch interval required by root; no test writes
   `stopping` or simulates a worker result.
2. **Claim wins, then cancellation prevents the first invocation.** Reserve a
   referenced entry through production methods. Call
   `claimPollerReadStart` and assert it succeeds; before the worker continuation
   runs, execute the real Stop transition so its manager context is canceled.
   Then call `runPollerFromClaim(entry, true)`, the same worker continuation
   called by `runPoller`. Assert zero port calls/attempts, code 4, phase 2/3,
   nil current, exactly one readiness resolution, actual workerDone completion,
   failed Prepare result, and capacity retained through JOIN. The explicit
   boundary orders the two real production stages without a test callback or
   scheduler hook.
3. **Actual held call and JOIN.** Use the existing trace-port fixture seam
   after actual port entry to hold the call. Stop while it is held, verify the
   entry remains registered/counted and no new read claim succeeds, then
   release it. Assert port return, workerDone, Stop JOIN, and only then
   capacity reuse. The manager test proves lifecycle ownership at the port
   boundary; it does not replace the adapter's separate physical admission
   and retirement proof.
4. **Shared initializer under global Stop.** Two separate pending Prepare
   leases share one initializing poller. Global Stop wakes both. Each Prepare
   removes only its own exact refs and completes its own registered operation
   before awaiting its own lease drain. The manager joins the one shared
   worker once; neither lease remains and no ref is removed twice.
5. **One Prepare cancels while another authorized lease survives.** Two
   distinct Prepare leases share the initializer. Roll back one pending
   Prepare; it removes its refs exactly once and does not cancel the worker
   while the other lease remains eligible. The surviving initializer result
   proceeds and the second Prepare completes normally.
6. **Detach/A cleanup race.** Race a preparing lease's selected-A cleanup
   against its detach/drain transition. Under `m.mu`, exactly one sees and
   removes each membership. An awakened waiter rechecks draining and joins
   the same drain owner; it does not decrement absent refs or cancel work
   required by another lease.
7. Preserve existing all-eight-selected-start-before-any-wait behavior and
   the current manager fixture unchanged. Keep success/error DTO, exact key,
   selection, counts, per-poller canonical image, and physical retry behavior
   assertions governed by their existing fixtures and decisions.

The deterministic cancellation-after-claim case depends on the private
`runPollerFromClaim` worker continuation being the exact path used by runtime.
An independent source critic must reject an implementation that forks test and
runtime continuations. If this continuation cannot be implemented without
introducing a test-only hook/state field, return the test matrix to root for
revision before any source release; do not substitute a timing race.

## Resolved findings and deviations

- The earlier recon proposal omitted a complete outcome for claim-success then
  cancellation before port entry. This proposal applies the root's exact code
  4 / phase 2-or-3 / nil-current / ready-once / zero-call / real-workerDone
  outcome and preserves the actual-call count rule.
- The earlier stop-before-claim fixture assumed an entry could be registered
  but not launched without proving production could reach that interval. The
  clarification now requires an immutable owned launch list and unconditional
  once-only launch after unlock; the test stages those same production methods.
- The fixture identity is reconciled to current `cf501...`; `8e741...` remains
  historical. The manager identity is current `ab9f1c...` after the separate
  key extraction; `fa1601...` remains historical. The key extraction itself
  and all pure encoder/helper fixtures stay frozen.
- The proposal preserves the mutex eligibility linearization point. It does
  not promise wall-clock callback entry before Stop when the worker claim won
  first, does not count claims as actual reads, and does not claim that a
  manager fixture proves adapter retirement internals.
- The lifecycle source/test paths above are proposed for a fresh root release
  only. No source/test/compiler/formatter/scanner/Git/build work, runtime
  result, or implementation approval is included in this record.
