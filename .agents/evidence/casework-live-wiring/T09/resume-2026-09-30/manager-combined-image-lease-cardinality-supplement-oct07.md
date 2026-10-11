# Lifecycle test matrix supplement: per-Prepare lease cardinality

Date: 2026-10-07  
Status: immutable design evidence for independent review; no lifecycle source or
test release is granted by this document.

## Evidence and scope

This supplement repairs only the ambiguous lifecycle test 3 in
`manager-combined-image-protocol-repair-supplement-oct07.md`. The independent
review rejected that case because it described multiple Prepare calls as if
they shared one lease drain. The correction below follows the per-Prepare
ownership model in the original Unit 1 contract and the root's ref-ownership
decision.

Reviewed evidence identities:

- Rejection: `manager-combined-image-protocol-repair-supplement-independent-review-oct07.md`, SHA-256 `026aa35a6326ed1c3f8223ddda80e8b2d2fc70a3ad00e09dc6d9665edba5e9cf`.
- Prior supplement being corrected: `manager-combined-image-protocol-repair-supplement-oct07.md`, SHA-256 `474fcc42c544c0a3dc21092cf3dbf532f6c7541d2ca6ca07fb76deffe4b85307`.
- Prior assignment: `manager-combined-image-protocol-supplement-assignment-oct07.md`, SHA-256 `408df95df56b5ebae6dfee1c82afb00ada18d9b7c69a0449c3dbbc7d3f5b3522`.
- Root decision: `manager-signal-ref-ownership-root-decisions-oct07.md`, SHA-256 `5b9888e5a7212b8d0696f790822f08e718dd03a71f51b28c2bec2ea0098f8df4`.
- Assignment for this supplement, archived before authoring: `manager-combined-image-lease-cardinality-supplement-assignment-oct07.md`, SHA-256 `63301123330c5e0ba0112b81e4b6ff626e00c508490183dc77bb14bd9d367f1a`.

The current manager scaffold confirms the cardinality: `runObservationManager.prepare`
returns one `*runObservationLease` per call (`run_observation_manager.go:125-129`),
and each lease has its own poller-membership map (`:82-85`). A poller has a refs
set keyed by lease pointers (`:73-80`), so distinct leases can share one poller.
The frozen lifecycle fixture
`run_observation_manager_test.go:555-639` already starts independent Prepare
calls while entries initialize; it remains unchanged.

## Replacement for prior future lifecycle test 3

### 3A. Global Stop with several pending Prepare leases

Arrange at least two independent Prepare calls for the same admitted key while
its single poller entry is initializing. Confirm by identity that each call
owns a distinct lease and that the shared poller has one exact ref for each
lease. Hold the initializer so all Prepare calls are pending, then invoke
manager-wide Stop.

Required assertions:

1. Stop wakes every pending Prepare. Under the actual manager mutex, each
   lease's first drain transition is the sole owner that clears that lease's
   remaining exact refs. No transition clears another lease's refs, and no
   awakened waiter removes an already-absent ref or decrements twice.
2. Every Prepare finishes its own operation registration before joining its
   own lease drain completion. A drain completion must not wait on an
   unfinished operation that is itself waiting for that completion.
3. With no surviving lease after global Stop, the shared worker becomes
   watcherless and is cancelled outside the mutex. Manager teardown joins that
   worker exactly once. Reservation/capacity remains occupied through actual
   worker return and JOIN.
4. No pending Prepare publishes a lease or starts/continues a read after Stop
   wins. Each stop-before-first-read Prepare returns its typed stop error, zero
   wrapper and nil lease. Its worker follows the already-approved code 4,
   phase 2 or 3, nil-current, no-read, ready-once behavior.
5. The test observes one worker, one ready resolution, one workerDone close and
   one manager JOIN for the shared poller, while observing one ref removal and
   one drain completion per Prepare lease.

Do not describe these Prepare calls as sharing a lease or a drain completion.
The manager-wide stop signal is shared; lease cleanup ownership and completion
are per call.

### 3B. One pending Prepare rolls back while a distinct lease survives

Arrange two independent authorized Prepare calls whose distinct leases refer
to the same initializing poller. Hold initialization pending. Cancel or
otherwise trigger failed-Prepare rollback for exactly one call while leaving
the other call authorized and active.

Required assertions:

1. The rolled-back Prepare's first lease-drain transition under the actual
   manager mutex owns clearing only that lease's refs, exactly once. The other
   lease's membership remains present; no absent ref is removed a second time.
2. Because the surviving lease still watches the poller, rollback does not
   cancel the shared worker. Cancellation, if any later becomes appropriate,
   remains outside the manager mutex.
3. The failed Prepare completes its own operation registration before joining
   its lease drain. It returns the specified typed error, zero wrapper and nil
   lease; it cannot self-join through an operation that remains unfinished.
4. The surviving Prepare remains authorized and receives the single shared
   initializer result. Assert one initializer publication and no duplicate
   read or worker. If the held real first read returns unavailable, retain the
   separately approved successful-A semantics for that surviving Prepare;
   this is distinct from stop-before-read code 4.
5. The surviving lease is later released through its own exact membership and
   drain protocol, and the worker is cancelled/joined only when its final
   eligible lease ref is gone.

This case is request/Prepare rollback, not manager-wide Stop. It must not
cancel a worker still watched by another authorized lease.

## Unchanged lifecycle matrix and constraints

The existing detach-versus-selected-A-cleanup race cases remain as written:
the selected A cleanup while preparing and lease detach compete under the
actual mutex, and exactly one removes the selected exact membership. The
other observes absence and does not decrement or cancel. Awakened waiters
recheck lease state and join that lease's owner. The stop-before-first-read
case remains code 4 with phase 2 or 3, nil current, no read, ready resolved
once outside the mutex, and capacity retained until worker return/JOIN. A
real first-read unavailable result remains successful A with a nonnil lease.

This supplement adds no image fields, initializer codes, phase values,
accounting terms, caps, wrapper encodings or byte-budget cases. The pure
encoder unit remains a separate assignment and release. The lifecycle source
and lifecycle tests remain unreleased pending root and independent-critic
approval.

## Material differences from the rejected supplement

- Replaces its single ambiguous “multiple Prepare waiters / same drain” case 3
  with two deterministic cases: global Stop across distinct per-Prepare leases,
  and one Prepare rollback while a distinct authorized lease survives.
- Makes lease identity, per-lease ref ownership, per-lease drain completion,
  and shared-poller identity separate asserted facts.
- Adds explicit operation-finish-before-own-drain-join assertions and exact
  per-lease removal cardinality; preserves one shared-worker cancellation/JOIN
  only when no eligible lease refs remain.
- Does not alter the prior cases, encoding design, image controls, caps,
  fixtures, or release boundary.

## Citation erratum

The earlier `manager-combined-image-protocol-repair-supplement-assignment-oct07.md`
contains a review-hash placeholder rather than the actual review identity.
That older immutable record is not changed here. The actual reviewed rejection
identity is the SHA-256 listed above: `026aa35a6326ed1c3f8223ddda80e8b2d2fc70a3ad00e09dc6d9665edba5e9cf`.
