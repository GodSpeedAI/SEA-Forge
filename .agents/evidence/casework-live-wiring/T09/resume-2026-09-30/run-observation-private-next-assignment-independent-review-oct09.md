# Independent review: private Next assignment

## Verdict and scope

**Approved for bounded assignment readiness only**, with the sequencing
clarification in `run-observation-private-next-assignment-addendum-oct09.md`.
This approves preparation of the specified private Next source/TDD unit; it does
not approve implementation, tests, runtime behavior, public readiness, or T09
settlement. The source release remains held as stated by the assignment.

## Evidence reviewed

- `run-observation-private-next-assignment-oct09.md`, especially transaction,
  wake, capture, commit, drain, and test requirements at lines 23–81.
- `run-observation-private-next-assignment-addendum-oct09.md`, especially
  operation ownership and the pre-wait, post-wake, and final authorization
  checkpoints at lines 6–14; concurrent-call behavior and the exact projector
  seam at lines 16–26.
- `run-observation-next-integration-root-decisions-oct08.md`, lines 8–23,
  25–47, and 49–66, plus the accepted revision 6 proposal/addendum,
  retention-initializer correction revision 3, and private-policy approval.
- Current implementation anchors: `run_observation_manager.go` lines 618–675
  for authorization/present-context checks and the Prepare-specific disclosure
  predicate; lines 767–862 for single-owner drain, off-lock cancellation, and
  creator/worker joins. `run_observation_poller_worker.go` lines 49–128 and
  161–224 for the mandatory production worker path, outside-lock watcher
  authorization, membership recheck, and read-start boundary.
  `run_observation_delta.go` lines 48–172 for the existing pure per-run
  projection and candidate watermark behavior.

## Findings

The original assignment required two authorization/present-context checks but
did not define their ordering around the wait. The addendum makes three
checkpoints explicit: after registering the operation and before waiting;
after wake and before capture/projection; and immediately before disclosure.
Callbacks stay outside `manager.mu`. Registering the operation first lets the
drain owner account for a callback that blocks. The addendum also correctly
limits these to as-of observations rather than promising continuous revocation
or atomic Store/Relay state.

The initial wake rule is sufficient for both complete-empty leases and a
successful terminal initial result, while unavailable list state returns
without waiting. The empty case therefore does not depend on a worker publish,
and terminal `retainedCurrent` remains deliverable after `workerDone`, matching
the root decision and revision 6 retention semantics.

The transaction is coherent: one admitted call captures exact entry/current
pointers, copied identity ledgers, and that lease’s prior watermarks under the
mutex; projects only those immutable captures outside it; and commits every
candidate watermark together only after exact lease/token and forward/reverse
membership checks. It explicitly permits `current` to advance during
projection, preserves the coalesced wake for the next call, and forbids
recapture/retry. The typed projector seam is specifically the existing
`buildRunObservationRunDelta` function; production and barrier tests use the
same transaction helper. These choices preserve the pure helper contract and
avoid a manager-global hook or alternate test lifecycle.

Drain behavior also matches the accepted ownership model. An admitted terminal
failure schedules the single asynchronous drain owner and releases its own
operation before any teardown wait. The owner joins creators, operations,
notifier references, and actual workers outside the mutex before releasing
capacity. Timeouts retain counted ownership, and a lease leaving shared work
does not cancel work required by a surviving authorized lease. A concurrent
second `Next` is a separate typed already-in-progress result: it owns no token
or operation, consumes no wake, and does not invalidate or drain the admitted
call.

## Material differences and limits

The addendum is a material sequencing clarification to the less explicit
revision 6 wording: it adds a pre-wait check and states the post-wake check
before capture. It does not change the private authority model, as-of
limitations, public contract, retention/capacity policy, or lease ownership.
No other material deviation from the accepted revision 6/addendum/correction
and private-policy decisions was found in this assignment. No implementation,
test, compiler, formatter, or runtime gate was run for this review.
