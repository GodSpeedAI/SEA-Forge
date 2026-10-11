# Private run-observation manager — concrete proposal revision 5

Date: 2026-10-06  
Status: DOCONLY proposal for fresh independent review. No manager source,
caller, public contract, or runtime work is released.

## Decision boundary and changes from revision 4

Revision 5 preserves the exact initial `contract.RunTraceObservation` DTO and
the accepted `boundRunObservationHydration` helper. It replaces bounded-window
ID comparison with an exact ever-observed ID ledger scoped to each live
`(case_id, run_id)` registry entry. A private first-observation ordinal is
assigned when an ID first enters an accepted source version. IDs are opaque
exact text; this ordinal is not parsed from or inferred from `event_id`, a
timestamp, or `TotalFrameCount`.

The ledger and all serialized safe retained poller data share a proposed
1 MiB per-poller JSON budget. The budget is explicitly **proposal-only** and
requires independent review and operator approval before implementation. A
candidate exceeding it is rejected atomically; no new frames/IDs/ordinals are
published and no previously accepted ledger identity is discarded. If a newer
source read cannot be retained, the poller marks its current version
unavailable so an older buffer is never represented as current. `Next` returns
a typed unavailable error, zero delta, and no watermark advance until a later
candidate fits. For a new selected initial candidate that cannot fit, it is
classified as unavailable (`A`), not shared-poller capacity omission (`C`).

Revision 5 also bounds `Next` to one in-flight call per lease, uses one
immutable captured poller-version set per wake, and does not retry a lost
global-generation race. It commits only this lease's watermarks after final
authorization and present-context checks. A later poll remains pending for the
next call. A changed present-context cursor returns typed unavailable with no
delta or watermark advance. This is still as-of-cursor evidence, not continuous
kernel truth.

Nothing here changes public SSE/cursor/DTO/schema behavior, execution
authority, kernel behavior, client physical admission, or the meaning of a
case cursor. Root retains architecture, policy, implementation, and
integration decisions.

## Preserved accepted bounds and cohort accounting

The manager uses process-shared `(case_id, run_id)` keys and at most 16
poller-registry entries, counting initializing, stopping, and draining
entries. The accepted client remains sole owner of the two-physical-read
concurrency limit, one fresh-connection retry for a read-only transport
failure, post-write cooldown, cancellation callback, and connection/pool
retirement. The approved 32 MiB per-response-line client cap is a separate
unimplemented prerequisite; the manager does not implement it or claim
present response-allocation bounds. The manager adds no physical limiter and
does not retry a read itself. Polling is at least one second apart per run.
Each source snapshot
contains at most the adapter's 1,024 retained safe frames. A cohort selects at
most eight readable summaries and starts at most eight logical initial trace
reads; cache reuse or joining an initializer is not a logical read. There is
one scoped `run.list` request, whose upstream enumeration is not bounded by
the case filter or these counts.

For a successfully decoded list, let `R=len(readable Runs)`, `U=len(scoped
UnreadableIDs)`, `S=min(8,R)`, and for the selected `S` rows let `V` be exact
validated rows (including verified current-buffer reuse), `A` be unavailable,
malformed, denied, owner-mismatched, orphaned, or unretainable candidates,
and `C` be selected rows for which the shared registry has no available slot
or the exact entry is still draining. Require `V+A+C=S`; set
`O=(R-S)+C`. Do not backfill after `A` or `C`, promote unreadable IDs, or
inspect rows outside the selector's first `S` candidates. `U` remains a
separate count. Settlement is never inferred.

All six optional count pointers are present, including zero, only after a
complete decoded list. The hydration read budget is `Limit=8`, exact logical
`ReadRunTrace` starts in `ReadsAttempted`, and root's executor interpretation
is `Exhausted=(ReadsAttempted==8 && R>8)`. Cached/shared reuse does not count;
fewer than eight calls and `R==8` are false. This is a root-directed working
interpretation, not a claim that the approved schema's causal wording is
unambiguous or that operator policy was amended.

For a successful list, outcome precedence is `unavailable` when `U>0` or
`A>0`, then `capacity_limited` when `O>0`, then `no_runs` when `R==0 && U==0`,
otherwise `complete`. Only `V` rows appear in `Runs`; unavailable selected
rows have no run payload. A refused, over-cap, or undecodable list returns the
contract DTO with `run_list_state=unavailable`,
`observation_state=unavailable`, nonnil empty `Runs`, and all optional counts
absent. It starts no candidate reads. Its completed empty lease reports
`ErrRunListUnavailable` from `Next`. Authorization, guard, cancellation,
stop, or assembly errors instead return a typed error, zero initial wrapper,
and nil lease.

The eight-selection and eight-read limits overlap. The chosen exhaustion
formula means `R>8` with eight actual logical reads reports true, even though
the same additional rows are also past the selection bound. This remains a
known schema interpretation caveat for review; it is not reopened as an
unspecified behavior.

## Exact contract DTO and initial envelope

The initial value is exactly `contract.RunTraceObservation` with its current
field names, JSON tags, and enum values (`contract/contract.go:186-238,413-442`):
case and observation times; list/cohort states; optional listed, selected,
validated, unreadable, unavailable, and omitted counts; hydration read budget;
and a nonnil `Runs` array. Each run preserves exact case/run/plan identities,
run and observation times, execution and settlement standings, validated or
unavailable run state, frames, total/retained/omitted counts, and truncation.
Optional frame command status and exit-code pointers remain independent.
Never introduce local replacement DTOs or enum spellings.

Build the complete DTO, then call `boundRunObservationHydration` exactly once.
It deep-copies nested mutable values, validates frame/count/timestamp bounds,
measures serialized DTO JSON only, and removes the globally oldest parsed
timestamps with full RunID and EventID byte tie-breakers, preserving each
run's frame order and updating retained/omitted/truncated counts. It preserves
run metadata even when a run has zero retained frames. A helper error is
`ErrInitialAssemblyUnavailable`, zero wrapper, nil lease, and owned rollback;
do not reject before pruning or drop required metadata. This accepted 1 MiB
initial DTO cap is separate from the unapproved per-poller budget and is not
SSE framing or a heap bound.

The private wrapper contains `Observation` and the actual case `AsOfCursor`;
the cursor is not serialized in the DTO. Shared buffer reuse preserves the
timestamp of that exact accepted poller version. Each newly accepted source
version records its actual capture time once as RFC3339Nano; attaching a
watcher or reusing an unchanged buffer does not restamp it as fresh. A
`RunDelta.ObservedAt` names the captured version time from which its current
state/window was built.

## Private types and exact API

These are proposal-only private types. An ordinal is local to one registry
entry lifetime and is assigned in first-observation order while scanning a
source response's retained frames. It has no public cursor meaning.

```go
type pollerKey struct { CaseID, RunID string }

type observedIdentity struct {
    EventID string // exact opaque bytes as represented by the Go string
    FirstOrdinal uint64
}
type retainedObservedFrame struct {
    Frame contract.RunTraceFrame
    FirstOrdinal uint64
}
type pollerVersion struct {
    Generation uint64 // private accepted-read version, not a frame count
    ObservedAt string  // versionObservedAt, capture time for this accepted read
    Snapshot ports.RunTraceSnapshot
    Frames []retainedObservedFrame // current source window in source order
    HighestObservedOrdinal uint64
}

type WindowState struct {
    Generation uint64
    TotalFrameCount int // exact current RunTraceSnapshot source total
    RetainedFrameCount int // len(current source window)
    OmittedFrameCount int // total minus retained source window length
    Truncated bool // omitted > 0
}
type RunDelta struct {
    RunID string
    PlanItemID string
    ObservedAt string // capture time of this immutable version
    Execution contract.RunExecutionStanding
    Settlement contract.RunSettlementStanding
    ObservationState contract.RunTraceRunObservationState
    Frames []contract.RunTraceFrame // first-observed ordinal > lease watermark
    Window WindowState // current source window, not delta frame count
}
type RunWindowGap struct {
    RunID string
    PreviousOrdinal uint64
    CurrentOrdinal uint64
    EvictedObservedEventIDs []string // exact IDs first seen after watermark, absent now
    EvictedObservedEventCount int // exactly len(EvictedObservedEventIDs)
    CurrentWindow WindowState // counts from this captured source version
    UnobservedLossPossible bool
    UnknownMissedFrameCount bool // true; source gives no exact cross-read gap count
}
type observationWatermark struct {
    HighestObservedOrdinal uint64
    Execution contract.RunExecutionStanding
    Settlement contract.RunSettlementStanding
    ObservationState contract.RunTraceRunObservationState
    Window WindowState
}
type ObservationDelta struct {
    CaseID string
    AsOfCursor string // private case knowledge at last guard check
    ObservedAt time.Time
    Runs []RunDelta // nonnil and sorted by exact RunID bytes
    WindowGaps []RunWindowGap // nonnil; exact IDs, no invented numeric loss
}

type CohortRequest struct {
    CaseID string
    Identity *requestIdentity // exact middleware identity, never caller DTO data
}
type InitialCohort struct {
    Observation contract.RunTraceObservation
    AsOfCursor string
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

`Next` errors are typed `ErrLeaseCanceled`, `ErrLeaseStopped`,
`ErrPresentContextUnavailable`, `ErrAuthorizationUnavailable`,
`ErrRunListUnavailable`, `ErrPollerRetentionUnavailable`,
`ErrNextAlreadyInProgress`, or `ErrManagerStopped`. Every error returns the
zero `ObservationDelta` and advances no watermark. The caller must detach and
drain after terminal authorization/context/retention errors. A drain timeout
leaves state counted as draining and does not claim cleanup.

## Source continuity and registry-lifetime identity ledger

The producer source is not a durable sequence contract. The run reader accepts
a well-formed JSONL prefix, may return no rows for unreadable/oversized input,
and stops at malformed tails. The recorder's `tev` sequence starts at zero per
recorder, can advance before failed writes, and omits run identity. Its error
append helper derives a new ID from parseable row count, not the highest prior
ID. The live Go adapter sees one response's recognized total, rejects duplicate
IDs within that response, and keeps at most the last 1,024 allowlisted frames
in source order. It supplies no source generation, durable high-water mark,
cross-read append-only proof, or exact missed-row count. Therefore neither
`event_id`, timestamp, per-response total, nor ID sorting establishes source
continuity (`run-get-trace-source-continuity-recon-oct06.md`, anchors below).

For each live poller registry entry, maintain `SeenByID map[string]uint64` and
`HighestObservedOrdinal`. IDs are exact and scoped by the entry's exact
`(CaseID,RunID)` key. For each accepted complete `ReadRunTrace` snapshot:

1. Validate exact identities and frame safety at the existing adapter seam.
   Preserve its current total and retained-window counts; do not reinterpret
   source row count as a sequence or cross-read counter.
2. Start from a copy of the committed ledger and ordinal. Scan the returned
   retained frames in source order. A previously unseen exact ID receives the
   next ordinal; an existing ID keeps its original ordinal, even if the
   source later reintroduces it or changes its payload. Never parse or sort an
   ID to choose an ordinal. Check `uint64` ordinal exhaustion before commit;
   exhaustion is retention-unavailable, not wraparound.
3. Build immutable retained-frame records from the current source window,
   pairing each current frame value with its ledger ordinal. The current
   window may repeat an old ID; a lease never receives that ID again merely
   because it reappeared. Frame payload changes under a previously observed
   ID do not establish a new identity and are not re-emitted as a new frame.
4. Atomically accept the whole new version, copied ledger, next ordinal,
   source-window counts, stands, and timestamp only if the per-poller budget
   fits. No partially updated ledger/window is visible.

The ledger is never evicted while its registry entry lives. Removing the
registry entry after final joined drain discards its history; a later poller
and new cohort may observe/replay those IDs again. That lifetime boundary is
explicit and is not an exactly-once promise across registry removal, process
restart, or source rewrite. While an entry remains live, reintroduced IDs are
not new.

Each lease stores one scalar `HighestObservedOrdinal` watermark per attached
poller plus the last emitted execution/settlement/observation state and
window. On initial capture, set the watermark to the poller's highest ordinal
**before passing any DTO through helper pruning**. This prevents intentionally
omitted initial frames from being misreported or replayed in `Next`. It also
means an ID first seen by the shared poller before this lease attached is
historical to this lease, whether still retained or already evicted. Cache
reuse preserves that version and timestamp.

For a captured current version, include current retained frames whose first
ordinal exceeds the lease watermark, in source-window order. Then advance the
candidate watermark to the captured version's highest ordinal. For any
ledger ID with ordinal greater than the previous lease watermark that is
absent from the captured current source window, include its exact ID in
`EvictedObservedEventIDs`; `EvictedObservedEventCount` is exactly that array's
length. These are identities the poller accepted but the lease could not
receive from the current window. This exact count is not a source missed-row
count. Source rows that never appeared in a Go response, disappeared before
an accepted poll, or were lost to journal prefix/truncation remain unknown.
When a gap is present, `UnknownMissedFrameCount=true`; never synthesize a
numeric missed-frame count from `TotalFrameCount`, ordinal difference, or
event ID spelling. `UnobservedLossPossible` is true when there may have been
source rows outside the observed retained window or intermediate accepted
versions were coalesced; it does not assert that any specific source row was
lost.

Standing-only changes are represented by a `RunDelta` with empty `Frames`;
private generation changes are not frame counts. A run delta is included when
there is a new-to-lease frame, standing/observation-state change, or truthful
current window metadata change. `Window` always uses the captured source
snapshot's `TotalFrameCount`, retained length, their difference, and derived
truncation; totals may move either direction between responses. A source
response with no new ID can still produce a standing/window delta.

## Proposed serialized per-poller retention budget

The additional 1 MiB limit is a policy proposal, not an accepted runtime
limit. Before implementation it requires independent architecture review and
operator approval. It is separate from both the accepted 1 MiB initial DTO
envelope and accepted 1,024-frame source window. It makes no Go heap, total
process RSS, in-flight response, transport, directory-enumeration, or
whole-cohort memory guarantee.

For deterministic admission, define a private versioned accounting image
serialized with `encoding/json` from explicit structs and slices, never Go
maps. It includes all safely retained, value-bearing per-poller data:

- exact case/run/plan-item identities, immutable first accepted time,
  accepted version time/generation, highest ordinal, next poll time metadata,
  current execution/settlement/run state, source total, retained/omitted and
  truncation values, and current availability/terminal marker;
- the complete current retained safe frame values paired with their ordinal,
  in source order, including optional command status/exit-code values;
- every `SeenByID` pair of exact ID and first ordinal, sorted by unsigned
  UTF-8 byte sequence of the exact ID solely for deterministic encoding;
- any additional safe metadata/auxiliary scalar or slice that would otherwise
  be retained and could grow. Locks, channels, contexts, reference counts,
  fixed-size control structs, decoded-but-uncommitted responses, and transport
  buffers are outside this serialized-value budget and must not be represented
  as bounded by it.

The schema for this private image has a fixed version tag. Compute
`len(json.Marshal(candidateImage))`; the inclusive ceiling is exactly
`1<<20` bytes. Marshal failure or a greater length is a budget rejection.
New fields cannot silently be excluded: any added retained safe field requires
updating the image and budget tests before source approval. Sorting is only for
budget serialization; event ordinals and delivery order remain source order.

For a new initializer, an over-budget first candidate is not installed into
the registry; the selected row is `A`, not `C`, and it contributes no
validated payload. For an existing poller, an over-budget candidate commits
neither its new IDs/ordinals nor its buffer/counts/stands/timestamp. Keep the
last valid version and its ledger unchanged for rollback/audit, but set an
explicit current-retention-unavailable marker and the failed candidate
generation. Wake leases. They must not receive the old version as though it
were current: `Next` returns `ErrPollerRetentionUnavailable`, zero delta, and
no watermark changes. A later complete candidate may recover only if it fits
with the entire previously committed ledger and current safe metadata; it
cannot reset the ledger to gain room. On recovery commit the new accepted
version atomically, clear the marker, and wake leases. No new IDs from a
rejected candidate count as accepted/ever-observed because no identity entry
was safely committed or disclosed; the unknown gap caveat remains.

If the source reports a terminal standing in an over-budget candidate, the
proposal records a terminal-retention-unavailable marker: do not publish the
unretained terminal state or stale prior version as current, return the typed
unavailable result, and stop further polling only after the read has returned
and the worker can be joined. The final state is not claimed disclosed. This
terminal-overflow disposition is a proposal requiring review. Nonterminal
retention failure may be retried on the next one-second-eligible poll; it
does not clear the marker until a full candidate fits. Initial DTO helper
failure remains a whole-Prepare typed error with owned rollback, unlike an
unfit selected poller candidate which is counted in `A`.

## Poller, lease, concurrency, and authorization ownership

Manager lock ownership is limited to registry/lease state transitions,
immutable version/ledger pointer swaps, counts, and reference ownership. No
network or adapter call, session/guard check, callback, channel send, cancel,
wait, or join occurs under the lock.

```go
type pollerState uint8 // initializing, running, stopping, draining, removed
type leaseState uint8  // preparing, active, draining, removed
type pollerAvailability uint8
const (
    pollerCurrent pollerAvailability = iota
    pollerRetentionUnavailable
    pollerReadUnavailable
)
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
type presentContextChecker func(string) (runObservationPresentContext, error)
type watcherIdentity struct {
    sessionID string
    actorID string
    role string
    validUntil time.Time
    requestContext context.Context
    developmentBearer bool
    // No bearer token; perspective is re-verified from the current claim.
}
type watcherAuthorizer interface {
    Capture(context.Context, *requestIdentity) (watcherIdentity, error)
    Recheck(context.Context, watcherIdentity) error
}
type observationErrorKind uint8
const (
    observationCapacity observationErrorKind = iota
    prepareCanceledError
    prepareStoppedError
    presentContextUnavailable
    authorizationUnavailable
    runListUnavailable
    initialAssemblyUnavailable
    selectedRunUnavailable
    leaseCanceled
    leaseStopped
    managerStopped
    pollerRetentionUnavailable
    pollerReadUnavailable
    nextAlreadyInProgress
    drainTimedOut
)
type observationError struct { Kind observationErrorKind; Cause error }
func (e *observationError) Error() string
func (e *observationError) Unwrap() error
type runRef struct { key pollerKey; entry *pollerEntry; ownerLease uint64 }
type initializerToken struct { key pollerKey; generation uint64; done <-chan struct{} }
type attachResult struct {
    disposition attachDisposition
    ref *runRef
    initializer *initializerToken // sole right to publish new initialization
    ready <-chan struct{}         // shared initializer completion
}
type pollerEntry struct {
    key pollerKey
    planItemID string
    state pollerState
    generation uint64
    firstAcceptedAt string // immutable time of first accepted read in this entry
    current *pollerVersion // immutable; swapped atomically
    seenByID map[string]uint64
    highestOrdinal uint64
    availability pollerAvailability
    terminal bool
    refs map[uint64]struct{}
    workerCancel context.CancelFunc
    workerDone chan struct{}
    ready chan struct{}
    notifierRefs int
    // additional fixed lifecycle fields/channels are explicitly owned here
}
type cohortLease struct {
    id uint64
    state leaseState
    refs map[pollerKey]*runRef
    watermarks map[pollerKey]observationWatermark
    wake chan struct{} // capacity-one coalescing hint, never carries payload
    stop chan struct{}
    nextInFlight bool
    identity watcherIdentity // no bearer token retained
    asOfCursor string
    prepareDone chan struct{}
    operations sync.WaitGroup
}
type RunObservationManager struct {
    mu sync.Mutex
    entries map[pollerKey]*pollerEntry
    leases map[uint64]*cohortLease
    nextLeaseID uint64
    stopping bool
    list runListReader
    traces ports.RunTracePort
    authorize watcherAuthorizer
    guard presentContextChecker // delegates to the accepted Store/Relay guard
}
type runListReader interface {
    RunsListForCase(context.Context, ports.CaseRef) (ports.RunListResult, error)
}
func NewRunObservationManager(
    runList runListReader,
    traces ports.RunTracePort,
    auth watcherAuthorizer,
    guard presentContextChecker,
) *RunObservationManager
func (m *RunObservationManager) attach(
    context.Context, *cohortLease, ports.RunSummary,
    runObservationPresentContext,
) (attachResult, error)
```

The structs describe required ownership, not permission to retain secret
credentials. Session watcher identity records exact session ID, actor/role
claim, request context, deadline/revocation basis, and current verified
perspective data required to recheck; never retain the bearer token. Each
watcher is authorized independently. Check request context and
`SessionStore.Current(id)` for session identity/current/deadline/revocation and
exact claim equality, then call the existing `PerspectiveVerifier` seam before
list, each selected read/attach publication, initial handoff, and each `Next`
disclosure. A watcher failure removes only its own refs. Existing
development-only static bearer identity remains as configured: there is no
session-current or revocation lookup for bearer mode, and request context is
its only local lifetime. Shared data never grants access.

Call the implemented private `checkRunObservationPresentContext` helper before
those relevant boundaries, outside the manager lock. It reads only the newest
retained revision; validates exact case/cursor/facts/record/overview/horizon
identity, nonnil horizon items and unique nonblank parent IDs; and requires
Relay's observed case cursor to equal that newest cursor. Its returned parent
map is copied and the result is as-of that check. Before disclosure, recheck
the guard and require the exact captured `AsOfCursor`; if it changed, return
`ErrPresentContextUnavailable`, zero delta and no watermark advance, without
retrying the guard loop. This does not atomically bind the Store and Relay
observations, lease a parent set, establish continuous kernel truth, add a
readiness frontier, or authorize public V4 wiring. Evidence: accepted guard
source/review references below.

The request context belongs to `Prepare` and its initial list/trace work. Each
poller worker has a manager-owned context; the first watcher never owns shared
poller lifetime. Prepare installs an idempotent rollback owner before list or
trace work. Each `Prepare` reserves a provisional lease, and only final
handoff under lock changes `preparing` to `active` and returns the DTO/lease.
All whole-Prepare errors return a zero wrapper and nil lease.

`attach` under lock verifies exact case/run/plan item, lease state, and copied
parent membership; it either reuses a verified current entry, joins its shared
initializer, reserves exactly one new slot/initializer, or acquires no ref
with a typed candidate disposition. Same key with conflicting plan item is
unavailable, never overwritten. At most 16 entries count including draining.
No active key is evicted for capacity. Reads and initializer waits run outside
the lock; only the initializer token can publish its result. Joining an
initializer shares its single logical call.

Every poller worker enforces the one-second floor, publishes immutable
version/ledger snapshots, and signals active leases using a nonblocking
capacity-one wake hint. Hints contain no data and may coalesce. A failed
`ReadRunTrace` produces no candidate version or ledger update; mark the
poller's current observation unavailable and make `Next` return typed
unavailable rather than pass the old version off as the result of that failed
read. A later fully accepted read may clear this read-failure marker. Terminal
accepted state is published once, then polling stops. No callbacks or network
waits happen under the lock.

One `Next` at a time is allowed per lease. It waits outside the lock for a
coalescing wake, caller cancellation, lease detach, or manager stop. Under lock
it claims `nextInFlight`, captures one immutable version (or
retention-unavailable marker) per attached key and the corresponding lease
watermarks, then unlocks.
Concurrent `Next` returns `ErrNextAlreadyInProgress` and zero. It rechecks
authorization and the present-context cursor outside the lock, builds a full
candidate `ObservationDelta` and candidate scalar watermarks from only those
captured versions, and performs final authorization/context checks outside
the lock. It does not loop or recapture on poller generation change. Reacquire
lock once: if this same lease is active and still owns the serial Next token,
atomically commit only its candidate watermarks and return that truthful
captured-version delta. If any poller advanced meanwhile, preserve/set that
lease's wake hint so the next `Next` captures it. The returned delta is
explicitly as-of its captured versions, not a claim of continuous latest
state. On cancellation, guard/auth failure, stop, retention marker, or
assembly error, return zero delta and advance none. No global-generation busy
retry exists.

On any whole-Prepare failure, its rollback owner marks the provisional lease
draining under lock, prevents new refs/notifier refs, and detaches exactly its
refs once. It cancels only entries with no other authorized watcher or
manager-owned completion. It then cancels, waits for notifier refs, joins its
list call, any initializer it owns or made watcherless, lease operations and
poller worker outside the lock. A shared initializer needed by another lease
survives. Slots remain counted in draining until actual calls return and
`workerDone` closes. A timeout leaves the entry/lease draining and counted;
it never permits premature slot reuse. `StopAndDrain` closes admission under
lock, transitions leases/entries idempotently, then performs cancellation and
joins outside the lock. No send can race channel close: notifier references
are acquired under lock, sent/released outside, and draining waits for their
release before close/removal.

## Initial capture, deltas, and typed outcomes

For a lease attaching to a current entry, initial DTO construction captures
the immutable source version and its `HighestObservedOrdinal` before helper
pruning, and initializes the lease watermark from that version's state/window.
The helper may prune frames from the initial DTO, but that omission is already
represented by its exact retained/omitted/truncated fields and is not a later
poller-loss gap. First accepted observation time of an existing shared version
is preserved.

For `Next`, `Frames` only contain current retained entries with ordinal above
the captured watermark. Dedupe is by ledger ordinal, not event ID parsing or
per-response set comparison. New frames are emitted in source response order;
run deltas and gaps sort by full RunID bytes for stable output. A standing-only
generation change emits a run delta with a nonnil empty frame array. Frame
counts are never calculated from generation difference. Current source
window's `TotalFrameCount` is copied from that captured response without
assuming monotonicity, and the retained/omitted/truncated equations describe
that version only.

If a later current window omits a first-observed ID whose ordinal exceeds the
lease watermark, list the exact ID and exact array length as an
observed-but-no-longer-retained gap. `UnknownMissedFrameCount` stays true:
per-read source totals, 1,024-frame windows, prefix-tolerant producer reads,
and opaque event IDs do not prove how many source rows were absent between
observations. Do not claim the known ID count equals source loss. Source rows
never returned by an accepted poll remain unknown.

| Condition | Prepare/Next result | State and count behavior |
|---|---|---|
| Invalid identity, failed guard/auth, canceled request before reservation | Typed Prepare error, zero wrapper, nil lease | No list/read/ref |
| Sixteen proposed poller entries occupied | `ErrObservationCapacity` | No ref; proposal still needs policy approval |
| Refused/over-cap/undecodable completed list | Unavailable DTO; empty completed lease | Counts absent; no reads; Next returns `ErrRunListUnavailable` |
| Successful decoded empty list | `no_runs` DTO, active empty lease | Explicit zero count pointers |
| Selected row denied/malformed/mismatch/orphan/unretainable | DTO succeeds, count in `A` | No payload/backfill; over-budget initial candidate is `A`, not `C` |
| Poller slot full or exact key draining | DTO succeeds, `C` and `O` | No ref/read for that candidate |
| Initial DTO helper fails, including irreducible metadata | Typed error, zero wrapper, nil lease | Roll back and join every owned ref/work |
| Retained candidate exceeds 1 MiB proposed poller budget | Initial candidate `A`; existing poller typed unavailable | No new frames/IDs/ordinals published; prior valid buffer is not served as current after newer failure |
| Later complete poll fits with retained ledger | Poller recovers | Atomic version commit, clear unavailable marker, wake leases |
| Changed final present-context cursor | `Next` typed unavailable, zero delta | No watermark advance or retry loop |
| Concurrent second `Next` on same lease | `ErrNextAlreadyInProgress`, zero delta | First call retains ownership; no overlapping watermark commits |
| Cancellation/auth/guard/stop/assembly error during `Next` | Typed error, zero delta | No watermark advance; caller drains after terminal error |
| Failed Prepare after partial attach | Typed error, zero wrapper, nil lease | Owned rollback; actual read/worker joins before slot release |
| Drain timeout | Typed timeout | Remains counted draining; no slot reuse |

## Required bounded review and later test-first decomposition

No tests, compiler, source, runtime, or public approval are claimed by this
proposal. Before a separately authorized implementation, a different
independent reviewer must inspect the originals, revision 4/rejection, source
continuity recon, this full revision, and exact cited code. The following
deterministic fixtures are required before root grants a compiler owner:

1. **DTO/helper:** every DTO field/state/count pointer, explicit zeros, read
   budget and exhaustion cases, optional frame pointers, exact JSON cap,
   globally oldest timestamp and full ID ties, retained counts/ordering,
   non-aliasing, zero-frame metadata preservation, typed irreducible failure.
2. **Cohort:** decoded empty vs list unavailable; `R/U/S/V/A/C/O` equations,
   precedence, no backfill, no unreadable promotion, selector order, logical
   reads vs client physical retry, exact cache/initializer reuse, root's
   `R=8/9`, reads `7/8` exhaustion interpretation.
3. **Registry ledger continuity:** exact opaque IDs including strings that
   resemble `tev_0001`, arbitrary nonnumeric IDs, duplicate response rows
   rejected at adapter seam, repeats/reordering/payload rewrite do not create
   new ordinals, first-observation response order, no redisclosure after
   eviction/reappearance while entry lives, explicit replay after removal and
   new entry, ordinal overflow fails closed.
4. **Lease watermark/gaps:** initial high-water before helper prune; no initial
   replay or false loss for helper-pruned frames; newly seen IDs delivered
   once while retained; an observed ID evicted before `Next` appears in the
   exact known-ID gap list/count; source unseen/missed total stays unknown;
   nonmonotonic source `TotalFrameCount`; standing-only update has empty
   frames and no generation-derived frame count.
5. **Budget:** stable canonical JSON image exact 1 MiB boundary, all ledger
   entries plus safe retained payload/metadata/aux fields included, marshal
   failure, no map-order dependency, atomic rejection, no seen-ID eviction,
   failed candidate IDs do not partially commit, current availability marker,
   typed Next zero/no-advance, later fit recovery, terminal-overflow policy,
   initial candidate `A` vs registry-capacity `C`. Keep proposal unapproved
   absent operator decision.
6. **Poller/Next race:** one immutable version snapshot per wake, build off
   lock, final auth/guard, poller advances during build, captured delta remains
   truthful as-of version and next wake is preserved; no global retry loop;
   changed case cursor returns typed zero/no-advance; one Next per lease and
   typed concurrent call; per-lease watermarks commit atomically and never
   affect other leases.
7. **Authorization/guard:** exact per-watcher session current/deadline/revoked
   claim/perspective checks; bearer only under existing dev-only configuration
   with no invented revocation; exact case/run/item and parent; newest-only
   present guard cursor match; guard change before final disclosure; no stale
   fallback or continuous-freshness assertion.
8. **Rollback/ownership:** initializer single-flight; failed Prepare after
   list/first attach/helper failure/final handoff; shared watcher survives;
   idempotent detach and concurrent Stop; no calls/callbacks/waits/cancel/join
   under lock; read/client retirement callback and actual worker join before
   removal or slot reuse; timeout remains draining; no channel send/close race.
9. **Bounds/terminal state:** 16 process-shared poller slots include draining;
   accepted shared two-read/retry/cooldown ownership; one-second poll floor;
   1,024 current source frames; accepted initial DTO helper cap; proposed
   per-poller budget only if separately approved; terminal successful final
   version then stop; terminal retention-overflow follows explicit reviewed
   unavailable policy. No heap/process-memory guarantee.

All runtime evidence, RED/GREEN, server race, module race, and canonical gates
remain owned by root's sole compiler executor with fresh resource ownership
and joined-result protocol. This proposal authorizes none of those actions.

## Source anchors and material caveats

- T09 approved proposal and accepted bounds/count semantics:
  `.agents/reports/casework-live-wiring/t09-contract-extension-proposal.md:27-45`;
  run trace is observational, not a case revision:
  `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml:82-106`.
- Exact DTO/enums and contract checks:
  `apps/godspeed-casework-go/internal/contract/contract.go:186-238,413-442`;
  `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-hydration-cap-root-acceptance-oct06.md`.
- Accepted initial helper behavior:
  `apps/godspeed-casework-go/internal/server/run_observation_hydration_cap.go:22-78,80-149,177-194`;
  it globally prunes the oldest parsed timestamp frames, preserves metadata,
  and returns typed unavailable only if metadata itself cannot fit.
- Run trace contract/total/window:
  `apps/godspeed-casework-go/internal/ports/run_trace.go:5-30`;
  `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go:67-177`.
- Reader and producer continuity limits:
  `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-get-trace-source-continuity-recon-oct06.md`;
  exact checked spans: `crates/sea-forge-server/src/sfwp/run_views.rs:417-436,835-857,947-957`,
  `crates/sea-forge-trace/src/lib.rs:78-122`,
  `crates/sea-forge-core/src/types.rs:528-540`, and
  `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go:66-98,103-176`.
- Selector ordering/8-run selection:
  `apps/godspeed-casework-go/internal/server/run_observation_selection.go:64-80`;
  frozen root acceptance is recorded in
  `run-observation-selection-phase2-root-acceptance-oct06.md`.
- Present guard source is implemented and source-reviewed, not a stub:
  `apps/godspeed-casework-go/internal/server/run_observation_present_context.go:25-89`;
  independent Phase 2 review and assignment addendum are
  `run-observation-present-context-phase2-independent-source-review-oct06.md`
  and `run-observation-present-context-phase2-independent-source-review-assignment-addendum-oct06.md`.
  It uses Store newest retained trajectory and Relay observed cursor
  (`internal/projection/store.go:41-53,122-139`,
  `internal/ports/ports.go:266-274`, `internal/server/relay.go:101-150,170-175`).
  The source review approval is not runtime GREEN; that gate remains pending,
  and the helper has no manager caller release from this document.
- Watcher auth source seams:
  `internal/auth/session.go:147-169`, `internal/server/server.go:38-42`,
  `internal/server/session.go:45-70`; `SessionStore.Current` establishes
  session lifetime, not actual drain of a client read.
- Shared physical read client bounds, retries, cooldown, and transport
  retirement are owned by `RunGetAdmission.AcquireRunGet` and
  `RunGetPermit.WriteAttemptStarted/WriteAttemptFinished/Release`
  (`apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission.go:15-25,80-156,193-235`);
  `Client.physicalAttempt` acquires a permit for each actual `run_get`
  attempt and releases it after call/connection cleanup
  (`client.go:489-531`). The production limiter owns exactly two slots and
  one-second post-write cooldown (`run_get_admission.go:28-35,52-56`); its
  acquisition waits with context cancellation and does not evict busy/cooling
  slots. The manager must call through `RunTracePort` and never add a competing
  gate. Entry removal waits for actual read return/client retirement and
  worker completion; session checks alone are insufficient.
- Concrete injected read seams are `ports.RunTracePort.ReadRunTrace(ctx,
  caseID,runID,planItemID)` (`internal/ports/run_trace.go:6-8`) and the
  existing `RunsListForCase(ctx, ports.CaseRef)` method
  (`internal/ports/ports.go:454-471`). The adapter validates exact returned
  case/run/plan identities and per-response duplicate IDs at
  `internal/adapters/sfwp/run_trace.go:67-98,103-176`.
- Existing source totals and the `tev` recorder do not prove immutable prefix,
  ID ordering, continuity, source completeness, or an exact count of missed
  frames. The ledger is manager registry-lifetime state only.

## Preserved decisions, deviations, and uncertainty

This revision preserves revision 4's exact DTO/helper, selector/count rules,
read-budget interpretation, root-approved 16 shared poller entries, eight
selection/eight logical reads, one-second per-run floor, 1,024-frame current
source window, accepted physical client ownership, per-watcher authorization,
present-context guard, failed-Prepare rollback and actual join-before-removal.
It repairs the bounded-watermark continuity gap by using a per-entry exact
ever-observed ledger and scalar first-observation ordinals. It repairs
revision 4's unbounded global-generation retry by defining one captured
immutable version set and one lease-owned watermark commit per `Next`.

Material proposed additions remain unapproved: the per-poller 1 MiB serialized
budget including ledger and all safe metadata/auxiliary value state; its
overflow unavailable/recovery/terminal transitions; the scalar ordinal
ledger; single-flight lease `Next`; and the exact gap signaling fields. The
budget is not a process heap guarantee. Registry removal/new-cohort replay is
an explicit lifetime boundary. `event_id` remains opaque and the source still
does not establish source continuity or numeric missed loss. The root-selected
exhaustion rule preserves its schema ambiguity caveat. Guard reads are
non-atomic and as-of only; no public readiness/frontier, SSE mapper, event
cursor, HTTP route, UI, kernel, or implementation is approved.

No source, test, status, debt, or prior evidence document is changed by this
proposal. No test, compiler, scanner, Graft build, Git, or network action is
authorized or claimed.
