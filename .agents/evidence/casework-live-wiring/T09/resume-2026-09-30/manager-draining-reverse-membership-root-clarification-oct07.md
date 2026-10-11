# Draining reverse membership and exact release ownership

Root semantic clarification before Phase B; independent review is required.
No source, fixture, public contract, schema, dependency, cap, or gate changes
are authorized by this record.

## Observed ambiguity

The frozen `TestRunObservationManagerPollerCapacityCountsDrainingEntries`
at `run_observation_manager_test.go:881-897` requires `entry.refs` to retain
one association for the detached/canceled A lease while its actual worker is
held. The exact-ref owner decision requires the first detach transition to
clear that lease's membership atomically, exclude further acquisition, and
own removal once. Revision 6/preregistration require cancellation before JOIN
and retained capacity until actual completion. The newer selected-A race test
checks both maps only after the worker returns and detach completes
(`run_observation_manager_failure_test.go:519-532`). The recon conflict is
recorded in `run-observation-drain-ref-semantics-result-oct07.md`.

The prior phrase "removing remaining exact refs" did not distinguish loss of
read eligibility from completion of the reverse ownership record. It must not
be implemented by weakening the frozen test or releasing capacity early.

## Required interpretation

The lease's `pollers` map represents its eligible attachment membership.
The poller's `refs` map also retains the exact reverse association needed by
the single drain owner until its required actual worker JOIN has completed.
An association with a draining lease is not an eligible watcher.

Under the manager mutex, the first detach owner sets `lease.draining`, captures
its exact entry list locally, and clears its eligible lease membership once.
Selected-A cleanup checks that membership and draining state under the same
mutex; it cannot claim a ref already transferred to the detach owner. Repeated
detach callers join the same drain owner instead of removing membership again.

For an entry made watcherless by that transition, retain `entry.refs[lease]`
as a noneligible drain ownership association while the actual worker is held,
including cancellation/timeout. Cancel outside the mutex. The owner removes
that exact reverse association only after the required worker completion/JOIN,
then removes an unreferenced exact entry and its cohort reservation under the
mutex. A timeout preserves ownership and both capacity reservations. A later
successful join may finish the same drain; it cannot fabricate completion.

Where another eligible lease needs the same worker, remove the departing
lease's reverse association without canceling or joining the surviving shared
worker. Selected-A cleanup while preparing still removes its own exact
membership once; it must retain responsibility for any watcherless actual
worker JOIN before capacity reuse or completion of that owned cleanup.

Every read-start claim must check exact manager entry identity, permitted
phase, manager admission, and at least one lease that is not draining and
whose `lease.pollers[key]` is the exact entry. Raw `len(entry.refs)>0` is not
an eligibility test. This prevents the retained draining association from
causing a later read. No extra token, scalar ref counter, callback, scheduler
hook, state enum, or persistent drain list is introduced by this clarification.
The already permitted draining flag, two maps, completion channels, and local
drain work list suffice. All cancellation, closes, I/O, waits, and JOIN remain
outside the manager mutex. Registered creator work finishes before its own
rollback wait; Stop waits for all creator launch operations before worker JOIN.

## Scope and review

This clarifies the temporal meaning of the existing private reverse map;
it does not add a second release owner or change the single-removal rule.
Logical eligible attachment release occurs atomically at detach; its reverse
ownership association survives only the necessary drain. Full map absence
after JOIN and the original held-worker ref assertion must both remain true.
An independent critic must compare this with all governing clauses and both
fixtures before source release. A concrete contradiction with a normative
spec must be reported rather than encoded silently. No runtime proof or
broader T09 completion is claimed.
