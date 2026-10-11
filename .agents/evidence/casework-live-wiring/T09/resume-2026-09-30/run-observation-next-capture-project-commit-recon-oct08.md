# Private Next capture/project/commit source recon

Date: 2026-10-08. Read-only source map for a later bounded private Next unit;
this grants no implementation or runtime work.

## Inputs and current source

Reviewed revision 6 (`60498c53`), its addendum (`9439cbfe`), retention
initializer correction 3 (`b8077e9b`), Next root decisions (`afcf4d2c`), and
initial lease decomposition (`f36f6271`). The current status says initial lease
TDD is source-reviewed but its focused RED is pending; list-state/map
initialization and watermark seeding remain a prerequisite, not completed
behavior.

Current source identities: manager `e2b77094`, manager tests `08852d59`, worker
`b0fde3fe`, pure delta `5241e656`, delta tests `48c6d28e` (full SHA-256 values
are available by hashing the named files; no source was changed here). Graft
was attempted first, but the repository has no graph manifest; anchors below
were checked directly.

## Existing path and reusable projection

- `runObservationLease` holds exact poller refs, caller/cursor, `listState`,
  and scalar watermark map (`manager.go:81-91`). At present, `beginPrepareOperation`
  initializes only the poller map and lifecycle channels (`:114-129`); no
  source path initializes `watermarks` or sets `listState`. `prepare` handles
  complete-empty and list-error separately (`:396-413`), captures each
  accepted `entry.current` (`:518-535`), then hydrates/prunes before its final
  disclosure gate (`:558-569`). Initial bookkeeping must land first.
- Poller `current` is an immutable retained-state pointer (`manager.go:69-79`;
  retained state in `run_observation_retained_version.go:38-54`). The worker
  authorizes each eligible watcher outside the mutex (`worker.go:161-241`),
  then rechecks exact manager entry, phase, pointer, and ref membership before
  publishing `entry.current` under the mutex (`:367-417`). A terminal accepted
  snapshot stops recurring reads (`:291-296,420-429`).
- `buildRunObservationRunDelta(state, copiedLedger, prior)` is the reusable
  pure per-run projector (`delta.go:48-52`). It validates exact IDs/ordinals
  and complete ledger coverage through captured `H`, ignores ledger ordinals
  above `H`, emits source-ordered frames for `P < ordinal <= H`, creates sorted
  exact-ID gaps for evictions, and returns a candidate scalar watermark
  (`:53-88,90-168`). It returns zero outputs on malformed/unavailable input.
  Existing tests cover boundaries/order, later window/ledger, standings-only
  changes, pointer/input copying, reappearing IDs, malformed input, frame cap,
  and valid empty terminal state (`run_observation_delta_test.go:24-351`).

## Next capture/commit requirements and test gaps

The root decision requires one mutex capture of each exact entry/current
pointer, copied `SeenByID` ledger, and that lease's prior watermark, followed
by assembly outside the mutex. Reauthorize/guard outside the mutex. Commit the
whole set of candidate watermarks only after final context and exact lease,
cohort, manager entry, forward-ref, and reverse-ref checks under the mutex. A
worker may publish while assembly runs: this call returns the captured version
and does not reject it merely because `entry.current` has advanced; that newer
version belongs to a later Next. Any run error yields zero aggregate and no
watermark changes (`root decisions`, “Captures, wake ownership and commit”).

Do not reuse `leaseCanDisclose` verbatim: it is Prepare-specific, accepts
`prepareDone`, and requires `entry.current == captured state`
(`manager.go:617-647`). `authorizeLeasePresent` is a reusable outside-lock
auth/guard starting point, but Next supplies its own caller context and needs
its own locked handoff predicate (`:590-615`). No `Next` method or manager-level
Next test currently exists. Required focused tests remain: successful
all-run commit; one-run projection failure leaves every prior watermark intact;
poll publication during assembly returns only captured `H` and commits only
that candidate; auth/guard/context refusal and detach/ref-membership loss
commit nothing. Pure-helper boundary tests already cover projection math and
should remain separate.

Wake coalescing, notifier ownership, operation registration, and drain/join
changes are explicitly later work; do not add or redesign them in this
capture/project/commit unit. No source, test, compiler, formatter, gate, or Git
operation was run for this recon.
