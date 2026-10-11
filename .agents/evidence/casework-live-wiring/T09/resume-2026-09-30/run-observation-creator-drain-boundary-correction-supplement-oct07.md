# Creator/list drain boundary correction supplement — 2026-10-07

Status: immutable DOCONLY correction after independent rejection. It grants no
source, fixture, compiler, or implementation release. Preserve the prior
assignment, proposal, and review unchanged.

The full corrective instruction is archived at
`run-observation-creator-drain-boundary-correction-assignment-oct07.md`.
This supplement amends the proposal
`run-observation-creator-drain-boundary-proposal-oct07.md` (SHA-256
`3a5e5d70a7adfd177e951dd0118f6f7f32ed2f906306695687fd8f7b7a293119`) only at
the lifecycle order and proposed test matrix below.

## Corrected creator and cancellation order

Every Prepare path owns a local derived request context used only for that
Prepare's list operation and initial readiness waits. On every exit path—normal
success, successful empty result, real initial A, refused-list/empty-lease
result, cancellation, Stop, or other error—the creator first finishes all
uses of that context, calls its local cancel function, and joins its bridge
goroutine. The bridge watches request-context completion or the exact lease's
once-closed `leaseDone`; it cancels only the local derived context. It never
cancels the manager-owned shared poller context or removes an eligible lease
reference. `leaseDone` may remain open for a successful active lease. Only
after list/bridge/owned launch work and creator-owned rollback have returned
does the creator close its exact `creatorDone` and balance the manager creator
WaitGroup once, outside the mutex. It does this before waiting for its own
rollback drain. The existing four manual fixture finish closures must pass the
captured exact lease to `finishPrepareOperation(lease)`; no no-argument or
implicit-current-creator path is used.

After list returns and before any attach or reservation, `reservePollerBatch`
rechecks under `m.mu` that the manager is not stopping, the lease is not
draining, and `m.cohorts[lease]` still identifies that exact lease. If a
cancel/Stop/detach transition already won, the late Prepare returns the
approved typed stop/cancellation error, zero DTO and nil lease, performs its
owned rollback, and starts no trace read. Reservation remains atomic with
membership/capacity updates. If Stop wins only after a batch was reserved, the
creator still launches every entry in its immutable owned batch exactly once
before finishing; the worker follows the existing stopped code-4 path.

Detach signals its own `leaseDone` before waiting, then waits only for that
lease's creator completion and workers it owns or made watcherless. It does
not wait for an unrelated creator or cancel another lease's context. Stop
signals all leases it owns and cancels pollers outside the mutex before global
creator/worker waits. The request bridge and shared worker contexts remain
separate.

## Corrected TDD observations

Keep all nine existing cases and assertions. In the held-list detach case,
assert that Prepare returns the specified typed stop/cancellation/unavailable
error, a zero DTO and nil lease; an ordinary successful unavailable-list DTO
must not satisfy it. While A's list remains held, B's list context remains
live; after A returns and A's exact creator/drain completes, B can return a
successful empty-list Prepare and then detach normally. Observe that the
successful B creator's `creatorDone` is closed when Prepare returns. The
implementation contract closes it only after canceling and joining the local
bridge; source review must verify that order. No extra bridge field or test
hook is introduced.

Add a Stop timeout branch that distinguishes `ErrDrainTimeout` from successful
drain completion. While a list or worker is held, timeout must leave exact
cohort/poller capacity and watcherless reverse ownership counted; it must not
fabricate closure of `creatorDone`, `drainDone`, or `stopDone`. After the actual
list callback returns, bridge JOIN completes, creator finish runs, and actual
workers return and JOIN, a later drain retry may finish and release the exact
owned reservations. Preserve the original detach timeout and held-read
assertions.

The two new held-list cases and four finish-closure shape changes remain the
only failure-fixture extensions. Frozen manager fixture `af2df...` and six
published primitives remain unchanged. No enum, counter, token, callback
field, public surface, schema, or budget change is introduced.

## Review boundary and material changes

This supplement repairs the rejected proposal's missing local-context cancel
on successful and empty/A paths, adds the required typed detach-cancellation
result, prevents post-list attachment after Stop/detach wins, and defines Stop
timeout retention and retry. It adds an observable successful creator
completion check without claiming the fixture can directly observe a joined
goroutine; source review must verify bridge JOIN precedes `creatorDone` closure.
The new independent critic must review this supplement with the complete
original assignment/proposal/rejection before any TDD fixture release. No
Go/test write, compiler, formatter, scanner, explicit Graft build, Git action,
or runtime claim is included.
