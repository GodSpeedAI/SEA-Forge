# Private run observation manager — concrete proposal revision 4

Date: 2026-10-06  
Status: DOCONLY proposal for fresh independent architecture review. No source,
caller, public contract, or runtime work is released.

## Decision boundary and repaired findings

This revision addresses the revision 3 independent review: the initial value is
the accepted `contract.RunTraceObservation` DTO, bounded by the existing
`boundRunObservationHydration` helper; every private delta/gap/error type is
defined; `Prepare` and its request have one unambiguous context; and cursor,
generation, and retention claims follow the actual trace adapter rather than
assuming monotonic event IDs or append-only source history.

The initial DTO contains exactly the existing contract fields and JSON shape
(`internal/contract/contract.go:186-238`). A private wrapper carries the
as-of-case cursor and internal lease metadata outside the DTO. The wrapper and
lease metadata are not included in the 1 MiB payload measurement. No public
HTTP/SSE mapping, cursor/frontier change, kernel read, identity expansion,
route, or implementation is authorized. Root retains architecture and
integration decisions.

Preserve the approved process-shared `(case_id,run_id)` key, 16-entry registry
including initializing/draining entries, eight selected summaries, at most
eight initial logical trace reads, 1-second per-run poll floor, 1,024 frames
per retained run window, and shared-client ownership of physical admission,
retry, post-write cooldown, cancellation callbacks, and connection retirement.
The manager adds no competing physical limiter. The additional maximum of 16
cohort leases and 1 MiB serialized retained bytes per poller remain proposals
only; each requires independent review and operator approval before source
work. Neither is assumed by this proposal's correctness or memory claims.

## Existing contract and source facts

The canonical initial DTO has `CaseID`, `ObservedAt`, `RunListState`,
`ObservationState`, six optional count pointers, `HydrationReadBudget` with
`Limit`, `ReadsAttempted`, and `Exhausted`, and a nonnil `Runs` array. Each run
has its own `ObservedAt`, exact identities, execution and settlement standings,
run observation state, frames, and total/retained/omitted/truncated values.
Frame command status and exit code are independent optional pointers
(`contract.go:196-238`). Allowed vocabularies are the current contract arrays:
execution `pending|enabled|active|completed|failed|terminated`, settlement
`unsettled|accepted|rejected|escalated`, run state `validated|unavailable`,
list state `complete|unavailable`, and cohort state
`complete|no_runs|capacity_limited|unavailable`
(`contract.go:413-442`). No alternate local enum or renamed status is used.

`boundRunObservationHydration(contract.RunTraceObservation)` deep-clones count
pointers, run/frame slices, and optional frame pointers; validates at most eight
runs, at most 1,024 frames per run, count consistency, and RFC3339Nano frame
timestamps; and measures actual `encoding/json` bytes. If oversized, it removes
the deterministic globally oldest frames and updates only retained/omitted/
truncated values. It retains every run and all metadata. If metadata alone
cannot fit, it returns typed unavailable and the zero DTO
(`internal/server/run_observation_hydration_cap.go:22-78,80-149,177-194`). The
manager must call this exact helper on the fully assembled initial DTO. It must
not reject a frame-heavy DTO before pruning or implement a second byte-bound
algorithm.

`RunTraceSnapshot.TotalFrameCount` counts allowlisted frames in the returned
`run.get` trace array before the adapter's retention window; it is not a claim
of source journal completeness (`internal/ports/run_trace.go:13-30`). The SFWP
adapter validates exact case/run/plan identities and separate standing
vocabularies, skips unknown string frame kinds, rejects malformed known rows,
deduplicates event IDs within the response, and retains the last 1,024
allowlisted frames in response order while reporting the accurate allowlisted
total (`internal/adapters/sfwp/run_trace.go:67-177`). It does not provide a
monotonic event ID, sequence, cross-poll append-only guarantee, or exact count
of frames a consumer missed between observations. Manager comparisons are by
exact `(RunID, EventID)` identity; cursor-like ordering is never inferred from
IDs, timestamps, or `TotalFrameCount`.

The existing selector is called once on the complete decoded readable list;
it orders the complete candidates and selects at most eight
(`internal/server/run_observation_selection.go:64-80`). The approved root
acceptance freezes selector source and fixture at hashes
`b9de3dae19ea9f473eaa0bbf680d06bcab3975195d90eea3bc0e40866c1ca63d` and
`1e2295a37aec4383f2473226680129f26394253e0ed43d6f05120ef6b30bada4`.

## Private data and API contract

All following types are package-private proposal types. The initial DTO is
retained intact inside the wrapper; only `Observation` is serialized and sent
to the later, separately authorized mapper.

```go
type CohortRequest struct {
    CaseID string
    Identity *requestIdentity // exact middleware identity; never request DTO data
}

type InitialCohort struct {
    Observation contract.RunTraceObservation
    AsOfCursor string // private real case cursor; excluded from DTO/payload size
}

type WindowState struct {
    Generation uint64
    TotalFrameCount int // exact current RunTraceSnapshot source total
    RetainedFrameCount int // len(current source window)
    OmittedFrameCount int // total minus current source window length
    Truncated bool // omitted > 0
}

type RunDelta struct {
    RunID string
    PlanItemID string
    ObservedAt string // immutable first accepted poller observation time
    Execution contract.RunExecutionStanding
    Settlement contract.RunSettlementStanding
    ObservationState contract.RunTraceRunObservationState
    Frames []contract.RunTraceFrame // exact IDs new to this lease and still retained
    Window WindowState // describes the current full source window, not Frames
}

type RunWindowGap struct {
    RunID string
    PreviousGeneration uint64
    CurrentGeneration uint64
    EvictedUnseenEventIDs []string // exact IDs known from this lease's prior window
    CurrentWindow WindowState
    UnobservedLossPossible bool // true when polls coalesced or unseen IDs left the ring
    UnknownMissedFrameCount bool // always true absent source-proven exact count
}

type runObservationWatermark struct {
    Generation uint64
    Execution contract.RunExecutionStanding
    Settlement contract.RunSettlementStanding
    ObservationState contract.RunTraceRunObservationState
    SeenEventIDs map[string]struct{} // exact IDs in the last source window, not an unbounded ledger
}

type prepareCancelCause uint8
const (
    prepareCanceled prepareCancelCause = iota // request context/session ended
    prepareStopped                            // manager shutdown won before handoff
    prepareGuardLost                          // latest retained context no longer agrees
    prepareAuthLost                           // this watcher's auth/perspective failed
    prepareAssemblyFailed                     // DTO/hydration helper failed
)

type observationErrorKind uint8
const (
    observationCapacity observationErrorKind = iota
    prepareCanceledError
    prepareStoppedError
    presentContextUnavailable
    authorizationUnavailable
    runListUnavailable
    selectedRunUnavailable
    initialAssemblyUnavailable
    leaseCanceled
    leaseStopped
    managerStopped
    drainTimedOut
)
type observationError struct { Kind observationErrorKind; Cause error }
func (e *observationError) Error() string
func (e *observationError) Unwrap() error

type ObservationDelta struct {
    CaseID string
    AsOfCursor string // private as-of case knowledge; not a kernel cursor
    ObservedAt time.Time
    Runs []RunDelta // nonnil, sorted by exact RunID bytes for stable private output
    WindowGaps []RunWindowGap // nonnil; known evictions only, never guessed counts
}

type CohortLease interface {
    Next(context.Context) (ObservationDelta, error)
    DetachAndDrain(context.Context) error
}

func (m *RunObservationManager) Prepare(
    context.Context, CohortRequest,
) (InitialCohort, CohortLease, error)
func (m *RunObservationManager) StopAndDrain(context.Context) error
```

`Prepare`'s single context is the request lifetime and cancellation context for
the scoped list and this cohort's logical initial reads. The manager-owned
poller context is separate and manager-scoped; it cannot be owned by the first
watcher's context. Session deadlines/revocation are checked explicitly. For
bearer requests, request context is the only local lifetime signal. There is
no duplicate `RequestContext` field.

On success, `Observation.Runs` and `WindowGaps` are nonnil arrays. `Prepare`
returns the initial DTO once; its first `Next` never repeats it. Each run's
`Frames` in `RunDelta` contains only frames newly observed by that lease and
still present in the latest bounded source window. Those delta frames are not
the DTO's retained-frame array: `Window` separately reports the exact current
source-window counts and always satisfies `RetainedFrameCount == len(current
source window)`, `OmittedFrameCount == TotalFrameCount - RetainedFrameCount`,
and `Truncated == (OmittedFrameCount > 0)`. The count never claims how many
frames this watcher received.

`ErrObservationCapacity`, `ErrPrepareCanceled`, `ErrPrepareStopped`,
`ErrPresentContextUnavailable`, `ErrAuthorizationUnavailable`,
`ErrRunListUnavailable`, and `ErrInitialAssemblyUnavailable` are typed
whole-prepare errors. Every error return from `Prepare` returns exactly the
zero `InitialCohort`, nil lease, and one typed error; no partial count pointers,
IDs, or frames escape. `ErrSelectedRunUnavailable` is a private candidate
disposition, not a whole-prepare error. The existing hydration helper's typed
unavailable result for invalid or irreducibly oversized DTO is a whole-prepare
failure and its zero DTO is preserved. `Next` errors are typed
`ErrLeaseCanceled`, `ErrLeaseStopped`, `ErrPresentContextUnavailable`,
`ErrAuthorizationUnavailable`, or `ErrManagerStopped`; an error produces the
zero `ObservationDelta` and does not advance any watermark. A timeout from
detach/stop is reported and leaves relevant state counted as draining.

## Present-context and watcher authority

The manager accepts no caller-provided parent set or freshness assertion. Its
guard depends on the existing retained history and relay cursor seams:

```go
type PresentContextGuard struct {
    history interface { Trajectory(caseID string) []projection.Revision }
    relay interface { CursorForCase(caseID string) (string, bool) }
}
func (g *PresentContextGuard) Check(caseID string) (runObservationPresentContext, error)
```

The accepted private context result is the existing
`runObservationPresentContext{caseID,cursor,parentIDs}`; the current
`checkRunObservationPresentContext` remains an unavailable stub pending its
own authorized implementation (`internal/server/run_observation_present_context.go`).
The check must require the exact newest retained case row, matching revision,
snapshot, facts, and record/overview/horizon identities and cursors, nonnil
horizon items (nonnil empty means known empty), unique nonblank item IDs, and an
observed relay cursor equal to the retained head. It returns a copied parent
set. Sources are `projection/store.go:41-53,122-139`,
`projection/builder.go:34-51`, `ports/ports.go:266-274`,
`server/relay.go:101-150,170-175`, and the guard assignment/review. This is
as-of-cursor evidence from two non-atomic observations. It is not continuous
kernel truth, a lease on parent state, a readiness frontier, or kernel
authority. The guard is rechecked outside locks before each list/read,
publication, initial handoff, and `Next` disclosure. No `LiveSource.Facts`
read or public V4 behavior is proposed.

`CohortRequest.Identity` is the exact authenticated `*requestIdentity` built by
middleware (`internal/server/session.go:64-70`); callers do not reconstruct it.
Session watches retain the exact session ID, detached actor/role claim, request
context, `ValidUntil`, and revocation state. Before each sensitive boundary,
check context, `SessionStore.Current(id)` for current/deadline/revocation and
exact claim equality, then `PerspectiveVerifier.VerifyPerspective(ctx, claim)`
using its real signature (`internal/auth/session.go:147-169`,
`internal/server/server.go:38-42`). A watcher failure detaches only that
watcher. For bearer mode accept only existing configured development-only
static bearer identities; retain actor/role and request context but never the
token. Recheck perspective on every boundary. Bearer has no `Current` lookup
or claimed revocation; request context is its only local lifetime. Existing
configuration restrictions remain unchanged (`session.go:45-62`). Sharing a
buffer is not authorization.

## Initial cohort outcome and exact accounting

For a successful decoded list, let `R=len(result.Runs)`, `U=len(result.UnreadableIDs)`,
`S=min(8,R)`, and among selected rows let `V` be validated exact identities
(including verified cache reuse), `A` selected unavailable/malformed/denied/
mismatched/orphaned rows, and `C` selected rows blocked by full or draining
shared registry capacity. Set `O=(R-S)+C`; require `V+A+C=S`. `U` is separate
and no unreadable ID is promoted to a readable row. Do not backfill after `A`
or `C`; do not read candidates outside the selector's first `S` rows.

For completed list, populate all six DTO count pointers, including explicit
zeroes; set read budget `Limit=8`, exact number of logical initial
`ReadRunTrace` calls started (0..8), and the approved exhaustion interpretation
as a proposal: `Exhausted=(ReadsAttempted==8 && R>8)`. A physical retry is not a
logical call; exact current cache and joining an in-flight initializer do not
increment this cohort's read count. No more than eight logical calls are
started.

The run-list contract has no pagination and an adapter-accepted decoded result
does not prove upstream directory completeness. `RunListState=complete` means
only that this scoped response decoded and passed its adapter validation. If
the list is refused, over-cap, or undecodable, return a successful terminal
initial outcome with `RunListState=unavailable`,
`ObservationState=unavailable`, `Runs=[]`, and all optional counts absent;
there are no candidate reads or lease refs. This is not a whole-prepare Go
error: it is the accepted DTO's unavailable outcome. The returned lease is an
already-completed empty lease whose `Next` reports `ErrRunListUnavailable`;
the caller can disclose the exact DTO once and cannot mistake it for an empty
successful list. Authorization, guard, cancellation, stop, and assembly
failures instead return typed error and zero DTO/lease.

When list succeeds, outcome precedence is: any `U>0` or `A>0` gives
`ObservationState=unavailable`; otherwise `O>0` gives `capacity_limited`;
otherwise `R==0 && U==0` gives `no_runs`; otherwise `complete`. Only `V`
verified selected rows appear in `Runs`, with per-run `ObservationState=validated`.
`unavailable` selected outcomes have counts but no run payload. Settlement is
never inferred from process exit or trace contents.

The exhaustion formula remains unresolved: schema prose says true only when the
read limit prevented validation of a known candidate, but both selection and
read limits are eight, so candidates after `S` are also beyond the selection
boundary. `R=8` and eight reads must be false under the proposal; `R>8` and
eight actual logical reads is provisionally true. Cached/shared reuse and fewer
than eight calls remain false. Root and a fresh critic must decide this causal
interpretation before implementation; this document does not claim the
contract unambiguous.

## Poller, attach, and ownership state

```go
type pollerKey struct { CaseID, RunID string }
type pollerState uint8 // initializing, running, stopping, draining, removed
type leaseState uint8  // preparing, active, draining, removed
type attachDisposition uint8
const (
    attachCurrent attachDisposition = iota
    attachSharedInitializer
    attachNewInitializer
    attachCapacityBlocked
    attachDraining
    attachIdentityMismatch
    attachInvalid
)
type runRef struct { key pollerKey; entry *pollerEntry; ownerLease uint64 }
type initializerToken struct { key pollerKey; generation uint64; done <-chan struct{} }
type attachResult struct {
    disposition attachDisposition
    ref *runRef
    initializer *initializerToken // only for new initializer owner
    ready <-chan struct{}           // current/shared initializer result
}
func (m *RunObservationManager) attach(
    context.Context, *cohortLease, ports.RunSummary,
    runObservationPresentContext,
) (attachResult, error)
```

Before calling `attach`, the caller checks authorization and guard outside the
mutex. Under the mutex, attach verifies manager/lease state, exact case/run/
plan-item identity, and guarded parent membership. An exact cached entry gets
one lease-owned watcher ref. A same-key initializing entry gets a ref and
shared `ready` result; it does not cause a second logical read. A new key
reserves one slot and initializing entry with generation, ready/done channels,
manager-derived cancel context, and first ref; its token is the only right to
publish that initializer's result. Capacity or draining acquires no ref and
counts as `C`. Conflicting plan identity and invalid parent/identity acquire no
ref and count as `A`. Waits, source calls, authorization, guard, cancellation,
and joins all run outside the mutex. Exact `ReadRunTrace(ctx,case,run,item)` is
the existing port (`internal/ports/run_trace.go:5-8`).

The 16-entry limit counts initializing, stopping, and draining entries. Never
evict an active key. A draining exact key yields `C`; slot reuse waits for its
actual worker/read completion. Same key with a different plan item is never
joined or overwritten. Reuse requires current exact case/run/item identity,
present parent, and a verified buffer. Initial `ObservedAt` is immutable on
cache/shared reuse; a genuinely new poller stamps once at first accepted read.
The registry stores bounded source windows, exact IDs, and standing values;
polling is no faster than once per second per run. A terminal source standing
publishes its final accepted observation then stops. No physical retry,
cooldown, or two-read gate is duplicated in this manager.

## Initial watermarks and subsequent deltas

Each poller has a generation incremented for each accepted source snapshot;
generation is a private local observation version, not a case/event cursor.
Each lease stores one `runObservationWatermark` per attached key: last accepted
generation, last disclosed standing tuple, and exact event IDs in the last
source window. This is a bounded comparison set, not a durable/unbounded seen
ledger. During
`Prepare`, while lease is `preparing`, capture each attached current snapshot,
copy it into the DTO candidate, and initialize that lease's generation,
standing, and seen-ID watermark to the exact captured window. Thus later
`Next` cannot replay initial frames. Initial buffer reuse preserves poller's
original observation time.

After successful handoff, each accepted poll increments generation and emits
only a nonblocking capacity-one wake hint to active leases. The hint carries no
payload/count; coalescing is expected. `Next` waits outside the lock for hint,
caller cancellation, lease detach, or manager stop; then it rechecks auth and
present context, snapshots current poller generations/windows under lock, and
builds a delta outside lock. It includes new exact event IDs from current
windows plus a `RunDelta` when standing or run state changed, even if `Frames`
is empty. Current `WindowState` counts always describe the full current
source window and obey the source-total equations above. Runs and gaps are
stable-sorted by complete RunID bytes. No source ID or timestamp ordering is
used to invent order.

If an ID present in a lease's prior observed window is now absent and was not
disclosed, record that exact ID in `EvictedUnseenEventIDs`. If generation
advanced while wake hints coalesced, or an unseen ID is no longer available,
set `UnobservedLossPossible=true`. `UnknownMissedFrameCount` stays true and no
numeric missed-frame count is asserted: `TotalFrameCount` describes one source
response before retention, not a validated monotonic counter across reads.
The gap's `CurrentWindow` contains truthful current total/retained/omitted and
truncated fields, not a fabricated loss total. IDs are deduplicated only while
present in the bounded source window / lease watermark. The guarantee is
bounded and does not promise exactly-once delivery across eviction, source
rewrite, or process restart.

Construct a candidate delta and next watermarks outside the mutex. Reacquire
the lock and commit them atomically only if lease remains active and captured
entry generations are still the committed source windows. If any generation
changed, discard the candidate and watermark copy, retain/restore a wake hint,
and take a new snapshot; do not return a stale candidate or commit partial
watermarks. Continue until one captured generation set is still current at
commit or the caller cancels/lease stops, in which case return zero delta and
advance no watermark. The one-second poll floor bounds source poll frequency;
new local generations can arrive only from those completed polls. On
cancellation, authorization/guard failure, stop, or assembly error, return
zero delta and advance no watermark. External calls, waits, callbacks, sends,
cancellation, and joins never run under manager lock.

## Initial DTO and envelope bound

Build `contract.RunTraceObservation` using every accepted field and enum:
non-nil `Runs`; exact per-run observation/standing states and metadata;
accurate source total; `RetainedFrameCount=len(Frames)`;
`OmittedFrameCount=TotalFrameCount-RetainedFrameCount`; and
`Truncated=(OmittedFrameCount>0)`. Keep every optional count pointer present
after decoded list, including zero, and absent only for list-unavailable DTO.
Preserve `RunTraceHydrationReadBudget` limit/read-count/exhausted values and
optional `ExecutionStatus`/`ExitCode` pointers independently. Preserve
identities, state fields, run order, and run objects even if all frames are
removed by envelope pruning.

Pass the complete DTO exactly once through
`boundRunObservationHydration`. That helper measures serialized DTO JSON only,
not private cursor/lease wrapper or SSE framing; it performs deterministic
oldest parsed timestamp, then full RunID bytes, then EventID bytes pruning,
per-run order preservation, exact count update and deep clone. It validates
limits and count/timestamp invariants. A typed helper error returns
`ErrInitialAssemblyUnavailable`, zero `InitialCohort`, nil lease, and rollback
of every acquired ref; do not send an oversized/partial DTO or drop metadata.
This reuses the approved bounded behavior; it is not a new 1 MiB-per-poller
retention policy. The helper is the sole initial-envelope cap authority.

## Prepare lifecycle and mandatory failed-Prepare rollback

Lease lifecycle is `preparing -> active -> draining -> removed`; poller
lifecycle is `initializing -> running -> stopping -> draining -> removed`.
Every `Prepare` reserves a provisional lease and installs one idempotent
rollback owner before any list or trace work. Because error returns transfer no
lease, caller cleanup is never assumed.

For every whole-prepare failure, including context/auth/guard loss, list
infrastructure error (if not represented as the DTO unavailable terminal
outcome), initializer failure that invalidates assembly, envelope-helper error,
stop race, and final handoff failure:

1. Under lock mark the provisional lease `draining`, prohibit new refs/notifier
   refs, and detach its refs exactly once. A concurrent `StopAndDrain` uses the
   same idempotent transition.
2. Decrement only that lease's refs. Cancel a poller only when no other
   authorized watcher/ref or manager-owned completion needs it; never cancel a
   shared initializer still needed by another lease.
3. Keep every canceled entry in the 16-slot count while `ReadRunTrace` and its
   accepted client cancellation/pool-retirement path are outstanding. Release
   the mutex before cancel, notifier waits, operation joins, or worker joins.
4. Join the failed prepare's list operation, any initializer it owns or made
   watcherless, notifier references, and lease operations outside the mutex.
   A shared initializer needed by another lease remains owned by that entry;
   this lease may detach without canceling it.
5. Mark lease removed and free poller slots only after actual calls return and
   the poller worker's `done` closes. A timeout returns an error and leaves
   relevant state counted as draining; it does not claim cleanup.

Selected candidate disposition `A` releases only that candidate ref and
continues other already-selected rows if watcher authorization remains valid.
It cancels/joins an initializer only if that ref was its last authorized need.
Capacity/draining `C` acquired no ref. A later whole failure rolls back every
prior ref. Manager shutdown closes admissions under lock, marks leases
draining, then cancels and joins outside lock. If handoff already linearized,
Prepare succeeded and shutdown drains the active lease; otherwise Prepare
returns typed stopped with zero result. No channel close races with a sender:
notifier refs are acquired under lock, sent/released outside, and detach waits
for them before close.

## Stop, delivery and failure table

| Boundary | Result | Ownership and cleanup |
|---|---|---|
| Missing/invalid request identity, failed guard/auth, canceled context before reservation | Typed error; zero wrapper, nil lease | No list/read/ref starts |
| 16 proposed cohort slots occupied (proposal only) | `ErrObservationCapacity`; zero result | No list/read; requires approval before implementation |
| Refused/over-cap/undecodable completed list | Exact unavailable DTO, absent counts, nonnil completed empty lease | No candidate read/ref; next returns list-unavailable |
| Decoded empty successful list | Exact DTO with all zero pointers, `no_runs`, `ReadsAttempted=0` | Active/empty lease with no poller refs |
| Selected row unavailable, malformed, denied, mismatched, or orphaned | DTO succeeds; counts `A`, no run payload | Detach candidate only; no backfill |
| Poller capacity full or exact key draining | DTO succeeds; counts `C` and `O` | No ref; no replacement selection |
| Initial DTO helper rejects invariant or irreducible metadata overflow | Typed unavailable error; zero result/nil lease | Roll back all prior refs, join owned work |
| Context/guard/auth lost after partial attach | Typed error; zero result/nil lease | Idempotent rollback, detach own refs, cancel only watcherless entries, join |
| Shared initializer failure | Candidate unavailable for each waiting cohort | Each detaches own ref; other entries/watchers continue |
| Cancellation while joining shared initializer | `ErrPrepareCanceled`; zero result | Detach only this watcher; shared initializer survives if needed |
| Stop before activation | `ErrPrepareStopped`; zero result | Same rollback; Stop joins preparation |
| Stop after activation | Prepare result remains valid | Next returns stopped; detach/manager joins |
| Drain timeout | Typed timeout/error | Lease/poller remain counted `draining`; retry may continue join |

`SessionStore.Current` is authorization lifetime state; it does not prove
transport drain. The poller `done` signal follows return from the accepted
trace client path after its cancellation callback and pool-retirement cleanup.
The accepted shared physical-admission implementation's actual completion
must be reverified at source review before any production slot-removal claim.

## Proposed aggregate and memory bounds, explicitly unapproved

The possible 16-lease cap includes preparing and draining leases, each with at
most eight run refs. It would cap attachments at 128 but does not by itself
bound all bytes. The separate proposed 1 MiB serialized safe-buffer budget per
poller would cap at most 16 MiB serialized retained safe payload across 16
entries. Both are proposals, not authorization or existing behavior. The
serialized cap does not bound Go heap overhead, in-flight decoded responses,
line buffers, scoped-list enumeration, or transport bytes. The 1 MiB initial
DTO cap does not substitute for either policy. No OOM/process-RSS guarantee is
claimed. Until those policy decisions are approved, implementations must not
silently assume either limit.

## Bounded test-first review matrix for any later released source task

Each fixture must first distinguish correct behavior from the held subject;
no test/compiler/run result is claimed here.

1. **DTO exactness and helper:** assert every DTO field, enum, count pointer
   presence/zero, read-budget value, run state, standing, pointer optionality,
   count equation, arrays, deep-copy behavior, and exact helper pruning at/over
   1 MiB. Verify metadata-only overflow returns typed unavailable and zero DTO;
   never fail merely because pre-prune frames exceed the cap.
2. **Cohort classification:** successful decoded empty versus list unavailable;
   `R/U/S/V/A/C/O` equations, count precedence, no backfill, no unreadable
   promotion, exact identities/parents, cache reuse at zero reads, logical
   calls versus physical retries, and exhausted ambiguity cases `R=8/9`,
   reads `7/8`; do not freeze exhaustion until root resolves it.
3. **Delta semantics:** initial watermark prevents replay; ID matching is exact
   and case/run scoped; standing-only updates produce a delta with empty frame
   array; coalesced hints use current buffer; exact known evicted IDs are
   reported; unknown missed count remains unknown; rewrite/nonmonotonic IDs do
   not produce fabricated cursor order or loss count; failed/canceled Next
   leaves watermark unchanged.
4. **Attach registry:** exact current cache, same-key shared initializer,
   single new initializer, same run/different case, plan mismatch, invalid
   parent, full 16 entries, draining key, ref ownership and whether each path
   charges a logical read. Assert no same-key double call.
5. **Failed Prepare rollback:** list failure, cancellation after reservation,
   guard/auth after list, after first attach and later whole failure, helper
   overflow after partial attach, shared initializer cancellation, concurrent
   stop at each boundary, and final handoff race. Prove own refs detach once,
   other watcher survives, actual read/client cleanup joins before slot reuse,
   and no partial output escapes.
6. **Authority and present-context:** session current/deadline/revocation,
   claim change, perspective loss before every boundary; dev-only bearer,
   request-context lifetime, no token retention or invented revocation; nil
   versus empty horizon, exact captured references/cursor and no stale-parent
   fallback. Explicitly do not assert continuous truth.
7. **Poller and shutdown:** one-second floor, terminal final state, last
   watcher, all watchers unauthorized, notifier send/close safety, cancellation
   and actual transport join, draining slot counted, timeout remains draining,
   no callbacks/waits/joins under lock. No manager duplicate physical limiter.
8. **Bounds and independent proof:** 1,024 source window and DTO helper cap;
   if separately approved, 16 cohort and per-poller serialized cap, with no
   whole-process heap claim. A different source critic reviews all originals
   and full result. Root separately grants one compiler owner and sequential
   focused RED/GREEN, server race, module race, canonical gate; each requires
   fresh resource capture and joined exact result under current protocol.

## Differences, unresolved decisions, and prerequisites

Revision 4 changes revision 3's custom `InitialCohort`/`InitialRun` payload to
a wrapper around the exact contract DTO and mandates the accepted hydration
helper. Its count pointers, `HydrationReadBudget`, all enum states, nested run
state, and optional frame pointers are preserved; frame pruning cannot become a
pre-pruning rejection. The delta/gap types now state generation, exact frame
identity, current source-window count semantics, known evictions, and unknown
missed counts without assuming append-only history or monotonic IDs. The
duplicate request-context field is removed: the Prepare context is the request
and per-cohort initial-read cancellation context; polling remains manager-owned.

Remaining decisions: root/critic must resolve the exhaustion causal ambiguity;
the additional 16-cohort admission cap and 1 MiB-per-poller serialized budget
need separate review and operator approval; the present-context guard is still
a separately held stub; actual cancellation/pool-retirement completion must be
reconfirmed before source release; and a private caller/integration assignment
is still required. Public mapper, HTTP/SSE route, event cursor/frontier, UI,
kernel, and live implementation remain held. None of this proposal approves
runtime behavior or source implementation.

## Source references

- Approved private-source counts, read budget, envelope, poller and client bounds:
  `.agents/reports/casework-live-wiring/t09-contract-extension-proposal.md:31-45`.
- Normative observation rules: `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml:82-106`;
  canonical contract prose `.agents/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:227-237`.
- Exact DTO/enums: `apps/godspeed-casework-go/internal/contract/contract.go:186-238,413-442`;
  contract assertions `internal/contract/golden_test.go:196-245,300-302,414-450`.
- DTO bound helper: `apps/godspeed-casework-go/internal/server/run_observation_hydration_cap.go:22-78,80-149,177-194`;
  its production assignment and independent source acceptance remain the authority for helper behavior.
- Trace total/window behavior: `apps/godspeed-casework-go/internal/ports/run_trace.go:5-30`;
  `internal/adapters/sfwp/run_trace.go:67-177`.
- Selector: `apps/godspeed-casework-go/internal/server/run_observation_selection.go:64-80` and root acceptance `run-observation-selection-phase2-root-acceptance-oct06.md`.
- Guard evidence: `internal/projection/store.go:41-53,122-139`,
  `internal/projection/builder.go:34-51`, `internal/ports/ports.go:266-274`,
  `internal/server/relay.go:101-150,170-175`; held guard
  `internal/server/run_observation_present_context.go` and its original assignment.
- Auth seams: `internal/auth/session.go:147-169`,
  `internal/server/server.go:38-42`, `internal/server/session.go:45-70`.

No source, test, status, debt, or other document is changed by this proposal.
No test, compiler, scanner, Graft build, Git, or network command is authorized
or claimed.
