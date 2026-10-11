# Drain/ref semantics reconciliation — 2026-10-07

## Finding

The proposed split—clear `lease.pollers` at detach, retain the reverse
`entry.refs[lease]` relation as non-eligible drain ownership until actual worker
JOIN, then remove it—is **not explicitly authorized** by the cited protocol.
It is a plausible way to satisfy the frozen observation and the newer
post-JOIN fixture, but it assigns a second meaning to the existing `entry.refs`
map. The documents call those entries lease refs and do not define a retained
drain-owner ref or say that the two membership maps may intentionally disagree.
Root needs to decide this semantic before Phase B relies on it.

## Exact assertions and clauses

- Frozen `TestRunObservationManagerPollerCapacityCountsDrainingEntries`,
  `apps/godspeed-casework-go/internal/server/run_observation_manager_test.go:850–897`,
  cancels `case_A` detach while its eight reads are held, then inspects both
  `case_A` and `case_B`. At lines 881–897 it requires every poller entry to
  remain registered, have `len(entry.refs)==1`, a live context, and an open
  `workerDone`; its failure text says “want retained ref and live worker.” At
  lines 903–912 it requires A's context canceled and B's context live. Thus
  A's lease is draining/canceled while `entry.refs` still contains one lease
  association; capacity remains occupied.
- Frozen `TestRunObservationManagerFailureDetachAndSelectedAReleaseHaveOneOwner`,
  `run_observation_manager_failure_test.go:471–535`, only asserts after the held
  read has been released and `detachAndDrain` has returned: both
  `lease.pollers[entry.key]` and `entry.refs[lease]` must be absent (lines
  519–532). It is compatible with either immediate ref deletion or deletion at
  completed drain, so it does not settle the interim protocol.
- Root's `manager-signal-ref-ownership-root-decisions-oct07.md:9–24` says the
  first detach/drain transition owns removing the lease's remaining exact refs
  and clears “the lease's ref membership” atomically; awakened Prepare waiters
  must not remove them again. A selected-A cleanup races for that same exact
  membership, and exactly one transition wins. It does not identify whether
  “membership” means only `lease.pollers`, both maps, or allow a draining
  association to remain in `entry.refs`.
- Revision 6, `run-observation-manager-concrete-proposal-revision6-oct06.md:286–292`,
  says all refs belong to one lease and are detached exactly once; a slot is
  removed only after no lease refs/notifier refs remain, owned work is canceled
  as appropriate, and `workerDone` closes. At lines 315–324, rollback marks a
  lease draining, detaches each acquired ref once, then waits/joins outside the
  lock; capacity remains counted until work joins. That sequencing reads as ref
  removal before JOIN, while the frozen test reads `entry.refs` during the
  blocked-JOIN interval.
- Retention correction revision 3 lines 23–44 likewise says the stop marker
  precedes JOIN and capacity remains counted until read retirement/JOIN and
  reference drains, but does not redefine `entry.refs` as drain ownership.

## Reconciliation boundary

The test and the post-JOIN exact-membership assertion can coexist temporally if
`entry.refs` intentionally keeps a draining lease pointer until worker JOIN,
while `lease.pollers` is atomically cleared and `lease.draining` excludes that
pointer from read eligibility. The current private shape provides the boolean
`draining` and both maps (`run_observation_manager.go:70–87`), so this would not
necessarily need a new field. However, no cited approved clause explicitly
permits this split interpretation, and “remove exact refs” plus “refs belong to
one lease and are detached exactly once” weighs against it. Deleting the A
entry ref immediately, on the other hand, would fail the frozen line 895
assertion. No safe implementation choice can be inferred from these sources
alone without clarifying whether the reverse-map association is a live lease
ref or a non-eligible drain-owned retention record.

This is an unresolved test/protocol semantic conflict, not evidence to weaken
the frozen test or add state. Root retains the architecture decision and should
make the precise `entry.refs` lifecycle explicit before Phase B. No source,
fixture, test, compiler, Graft, scanner, or Git mutation was performed.
