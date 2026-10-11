# Run observation manager Unit 1 lifecycle implementation preregistration

Date: 2026-10-07. Immutable plan before implementation release. Source-only
planning; no source/test edits, compiler, test, scanner, formatter, typecheck,
or Git command was performed for this record.

## Assignment and release boundary

The current assignment is to prepare the next bounded Go manager Unit 1
lifecycle implementation plan against the existing scaffold and fixture. The
builder must not edit production or test source until root verifies the fresh
`af2df` fixture RED and explicitly releases implementation. No Server, HTTP,
SSE, V4, public contract, schema, kernel writer/frontier, dependency, or auth
policy integration is included. The private units may use only unexported
names, current ports and DTO types, and source-grounded auth/present-context
contracts. Root retains semantic and architecture decisions.

The frozen original assignment is
`.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-manager-unit1-original-assignment-oct06.md`
(SHA-256 `de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916`).
It authorizes a test-first private `prepare`/admission/owned-rollback unit,
including 16 preparing/active/draining cohorts, 16 initializing/stopping/
draining pollers, exact key and parent validation, shared initialization,
rollback that cancels only watcherless work, actual read/retirement/worker
join before capacity release, and idempotent stop/detach with timeout retaining
capacity. It excludes full `Next`/delta, SSE, auth-policy, JSON-budget, and
integration implementation. The new root direction expands this next bounded
unit only to cover the private prepare/shared-initializer/poller lifecycle,
initial DTO, recurring reads needed for draining, and stop/detach ownership;
it still explicitly excludes `Next`, public wiring, and production exports.

The source identities at planning time are:

| File | SHA-256 |
|---|---|
| `run_observation_manager.go` | `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` |
| `run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |

The test hash is the current repaired fixture identity, not evidence of a test
run. The earlier accepted RED at another fixture hash is not transferable to
this version. Fresh RED capture and root release remain pending. Existing
exact preimages for source and the prior list-refusal fixture version were
archived by root as UTF-8 JSON content in
`run_observation_manager.go.before-list-fixture-oct07.json` and
`run_observation_manager_test.go.before-list-fixture-oct07.json`; this record
does not claim those archives contain the newer `af2df` test version.

## Governing decisions and source contracts

Use revision 6 and its addendum, initializer/retention corrections 2 and 3,
response-line-cap erratum, independent reviews, and root's private-design
decision. The root decision explicitly accepts the private 16 cohort slots
(including preparing/active/draining), 128 attachment ceiling, and canonical
`1<<20` current retained-value image per poller. It authorizes these private
choices without authorizing public release. Never reset or evict the exact
entry-lifetime ledger to fit. Nonterminal retention failure is atomic and
recoverable only with a later complete candidate fitting the preserved ledger;
terminal over-budget state is not disclosed and bars future read starts before
join. Fixed-width internal lifecycle state keeps a failure marker representable
at the exact byte boundary. This is not a heap/RSS bound.

The 32 MiB Go inbound serialized response-line cap already exists before JSON
decode and includes LF; it is not a source-file or heap cap. The adapter retains
at most 1,024 safe frames after decode. The separate Rust 64 MiB whole-journal
limit and malformed-tail prefix behavior remain CW-23 debt. Do not claim a
kernel per-row cap or complete trace inventory.

Relevant source anchors inspected:

* `run_observation_manager.go`: private manager, list/trace/session/guard
  dependencies, `prepare`, `stopAndDrain`, and `detachAndDrain` are present;
  the last three return the explicit “unit 1 is not wired” unavailable stub.
  The poller scaffold has `refs`, `ready`, manager-owned context/cancel, and
  `workerDone`, but no actual state machine/version/initializer result yet.
* `run_observation_selection.go`: `selectObservationRuns` validates execution
  standing and selects by active/pending/completed rank, recency, and exact
  `RunID` tie-break. Reuse this helper unchanged.
* `run_observation_hydration_cap.go`: `boundRunObservationHydration` deep-copies
  and serializes the full DTO, permits exactly `1<<20`, removes globally oldest
  frames by timestamp then exact RunID/EventID, repairs counts, and returns
  unavailable only when metadata cannot fit. Call it once on the complete
  initial DTO; do not implement a parallel pruner.
* `run_observation_present_context.go`: the guard checks newest Store
  trajectory, snapshot/Facts/case/cursor identity, Horizon parent uniqueness,
  and Relay cursor equality. It is an as-of check and takes no kernel Facts
  read. Keep it outside the manager mutex.
* `ports.RunTracePort.ReadRunTrace` returns `RunTraceSnapshot`; `run_trace.go`
  validates safe standing/frames and retains the last 1,024 source-order frames.
  Its call includes the existing SFWP physical admission/retry/retirement
  ownership. Do not add a manager physical limiter or retry path.

## Bounded file plan after release

1. **Manager types/state in `run_observation_manager.go`.** Keep private
   constructor and seam. Add the actual manager-protected lease and poller
   transition representation needed for reservation, attach references,
   initializer generation, readiness, worker lifetime, stopping/draining, and
   idempotent owner join. Make lease reservation atomic before list/read work;
   include preparing, active, and draining in the 16 cohort count and
   initializing/stopping/draining in the 16 poller count. Reserve all selected
   initializers before waiting for any one to finish, so the 16-entry test
   cannot pass by serializing two groups of eight. Reject capacity before
   forbidden list/read work; do not evict live entries.
2. **Prepare orchestration in the same file.** Validate request identity,
   authorize and capture present context outside the lock; list exactly once;
   distinguish list refusal from failed Prepare. Select only the existing
   selector's first eight; do not backfill or promote unreadables. For a
   successful list set all six optional counts, with `S=min(8,R)`, `V+A+C=S`,
   `O=(R-S)+C`, and root's executor interpretation
   `Exhausted=(ReadsAttempted==8 && R>8)`. Count actual logical trace-call
   starts; shared/cache reuse is zero. Refused/over-cap/undecodable list yields
   exact unavailable DTO, nonnil empty Runs, absent optional counts, no trace
   call, and completed empty lease. Auth/guard/cancellation/stop/irreducible
   assembly error yields typed error, zero wrapper, nil lease, with internal
   rollback. Populate the exact contract DTO including frame counts and
   optional pointers, then call the existing hydration cap once.
3. **Shared initializer and poller worker.** Attach exact `(case,run,plan item)`
   identity only after selected-parent validation. Under the manager mutex,
   acquire/release exact lease refs and issue a generation-bound manager-owned
   initializer token. Initiator context owns its wait/list, not shared worker
   publication. Cancelling one watcher removes only its ref; cancel shared
   initialization only when no authorized ref remains. A waiter failure rolls
   back its own ref without stranding or revoking another watcher's work.
   Publish only for matching generation/token. All port I/O, authorization,
   guard calls, callbacks, cancellation, waits, and joins occur outside lock.
   Do not release any entry or cohort capacity until actual port return and
   worker `workerDone` join (and owned refs/operations/notifiers drain).
4. **Recurring worker/drain.** Use one worker/read at a time per poller and a
   minimum one-second interval. Retain one owned frame representation, clone
   optional pointers at ingestion and DTO projection, and enforce 1,024 source
   frames. If implementing the retained ever-seen ledger/current value in this
   unit, include every retained safe value and exact ID/ordinal pair in the
   deterministic canonical `1<<20` image before publication; reject atomically
   and honor revisions 2/3's transient versus terminal refusal rules. Never
   omit this bound. If that value-state cannot be implemented coherently in
   this slice, stop before fake placeholder state becomes an apparent
   production implementation and return the scope dependency to root.
5. **Stop/detach/rollback.** Transition to draining under the same mutex that
   gates reference/notifier/operation acquisition. Make close/cancel once-only;
   never close a wake channel. Cancel outside the lock, join actual workers and
   list operations outside the lock, and only remove/reuse capacity on complete
   drain. A timeout returns the typed drain error and leaves the entry counted.
   Concurrent stop/detach must be idempotent. Do not add `Next` or an invented
   lease method to make this unit appear complete.
6. **Focused fixture extension, only if root releases a fixture change.** Add
   tests before implementation for exact initial DTO/list refusal, selected
   calls/count equations, all-eight-started-before-waiting capacity behavior,
   16 cohort and 16 poller boundaries, shared initializer owner cancellation
   with a real second ref already registered, partial Prepare rollback, actual
   worker JOIN and timeout retention, stop admission closure, one-second
   minimum cadence, current-context/parent mismatch, exact pointer-copy
   behavior, and irreducible hydration failure. Use channel-controlled
   blockers and bounded waits; tests must fail at semantic assertions before
   cleanup. No sleeps to create races.

## Required ownership and behavior invariants

* Manager mutex protects only maps/counts/state/reference acquisition and
  immutable version publication. No callback, adapter call, I/O, channel send,
  cancel, WaitGroup wait, or worker JOIN is performed under it.
* The request context cannot own shared initializer lifetime. Generation tokens
  protect against stale completion; manager-owned worker alone closes its
  `workerDone` after the actual trace call and retirement return.
* A watcher leaving never cancels work still needed by another authorized
  attachment. A slot remains occupied during every actual read, retirement,
  operation/ref/notifier drain, and JOIN; timeout is not completion.
* Failed Prepare always has an internal rollback owner because caller receives
  nil lease. List refusal is the separate exception returning its specified
  unavailable DTO and completed empty lease.
* Successful list has all count pointers present (including zero); unavailable
  list has all six absent. No backfill, unreadable promotion, candidate read
  beyond selected prefix, or claim of settlement.
* Existing trace port owns physical two-attempt admission, fresh-connection
  retry, cooldown, cancellation, and client retirement; manager counts one
  logical call and never bypasses it.
* The initial DTO 1 MiB cap is separate from the per-poller retained image.
  The latter is canonical serialized current state only, not total transient
  heap/RSS. Preserve exact opaque event IDs and source ordering.

## Scope dependencies and unresolved boundary

The revision 6 full proposal includes ledger/window/`Next` types and behavior,
while this lifecycle unit explicitly excludes `Next`. The current source has
no `Next` method, and the fixture's list-refusal empty lease currently cannot
assert `ErrRunListUnavailable` on `Next`; that remains an implementation
obligation for a later unit, not permission to add it here. A recurring poller
that retains current versions/ledger state cannot sidestep the approved
per-poller serialized budget. Root must decide whether the complete bounded
retained-state publisher is in this release or whether this task stops at the
initializer/drain lifecycle seam; do not land a placeholder that reads and
retains unbounded state. `RunTraceSnapshot.TotalFrameCount` is a count from the
adapter's returned response, not a source-continuity guarantee. The already
documented kernel malformed-tail/whole-journal false-empty risk remains open.

## Evidence/claim limits

No tests, compiler, runtime, scanners, Git, or source edits were performed for
this plan. The previous RED (at a different fixture hash) proves only that one
earlier selected case reached the unwired stub; it proves no rollback, retry,
capacity, poller, or join behavior. The fresh `af2df` fixture must be reviewed
and run by root before implementation release. A source critic must review
the released implementation before any runtime GREEN claim. Private lifecycle
completion is not public integration or T09 settlement.
