# Private run-observation manager — concrete proposal revision 6

Date: 2026-10-06  
Status: DOCONLY proposal for independent architecture review. No manager source,
caller, public contract, or runtime work is released.

## Decision boundary

This is a complete replacement blueprint for review, not a source assignment.
It preserves the exact existing `contract.RunTraceObservation` initial DTO and
calls `boundRunObservationHydration` once on the complete DTO. It keeps the
entry-lifetime exact opaque-ID ledger, first-observation ordinals, scalar lease
watermarks, truthful source-window counts, and one captured immutable version
set per `Next` from revision 5. It incorporates every revision 5 independent
review finding and makes the ownership and timestamp rules below normative for
this private proposal.

No HTTP/SSE mapper, event cursor/frontier, kernel Facts read, public interface,
schema, authority rule, or UI change is proposed. The present-context check is
only the existing Store/Relay comparison and is an as-of observation, not an
atomic or continuous-truth guarantee. Root retains all policy, implementation,
and integration decisions.

## Bounds and admission

The already accepted bounds remain: at most 16 process-shared `(case_id,
run_id)` poller entries, counting initializing/stopping/draining entries;
at most eight selected readable summaries and eight actual logical initial
trace-read starts per successful list; one scoped `run.list`; at least one
second between polls for a run; and at most 1,024 safe frames in each source
snapshot. A physical retry is not another logical read. The shared production
`RunGetAdmission` remains sole owner of two physical `run_get` attempts, the
read-only fresh-connection retry, post-write cooldown, cancellation callback,
and connection/pool retirement. The manager calls only `RunTracePort`; it adds
no physical gate, retry, or alternate cancellation path. Its capacity is not
released until the actual read returns, client retirement finishes, and the
poller worker joins.

**Proposed, not approved:** cap all cohort leases at 16, counting preparing,
active, and draining leases. Each lease selects at most eight rows, so no more
than 128 lease-to-run attachments can exist. Reservation occurs before list
or trace work. At capacity, `Prepare` returns typed `ErrCohortCapacity`, zero
initial wrapper, and nil lease without starting list/read work or fabricating
DTO counts. A draining lease consumes its slot until its owned operations and
per-target notifier references are joined. No live lease is evicted. This
policy is independent of the accepted 16-poller bound and requires independent
review and operator approval before implementation or integration.

**Proposed, not approved:** each poller entry's ever-observed ID ledger,
current frame window, and all other retained value-bearing safe metadata share
one combined serialized JSON budget of exactly `1<<20` bytes. This is a
deterministic retained-value policy, not a heap, RSS, in-flight response,
transport, directory-enumeration, or whole-cohort bound. It requires
independent review and operator approval. The accepted 1 MiB initial DTO JSON
cap is separate. The separate approved 32 MiB source-line cap is an
unimplemented prerequisite and is not supplied by this proposal.

Even if both proposed limits are later approved, 16 serialized budgets imply
at most 16 MiB of those serialized retained per-poller values, plus at most
16 MiB of concurrently assembled initial DTO JSON before pruning, plus up to
128 lease structures and unbounded-in-this-accounting transport/decoded
temporaries. These are serialized-image figures, not Go heap upper bounds.
The source-line limit does not bound arbitrarily large metadata IDs and
directory enumeration. No practical total-process memory/OOM guarantee is
claimed. If review requires such a guarantee, a separate aggregate byte and
input-enumeration policy is required; it is outside this proposal and must not
be silently added.

## Cohort selection, counts, and exact DTO

For a successfully decoded list let `R=len(readable Runs)`, `U=len(scoped
UnreadableIDs)`, and `S=min(8,R)`. The deterministic accepted selector chooses
the first `S` candidates. Among only those rows, `V` is exact validated
ownership including exact current-buffer reuse; `A` is unavailable, malformed,
denied, mismatched, orphaned, or unretainable; `C` is blocked by shared poller
capacity or an exact entry still draining. Require `V+A+C=S`; set
`O=(R-S)+C`. Thus `R=V+A+C+(R-S)`, and `U` is separate. Never backfill after
`A` or `C`, promote unreadable IDs, or inspect candidates outside the selected
prefix. Settlement is never inferred.

On a complete decoded list, all six optional counts are present, including
zero. `Limit=8`; `ReadsAttempted` counts actual logical `ReadRunTrace` starts,
not physical retries or cache/shared-initializer reuse. Use the root-directed
executor interpretation `Exhausted=(ReadsAttempted==8 && R>8)`. In particular,
`R==8` is false, fewer than eight actual starts is false, and `R>8` with eight
starts is true even where the eight-selection bound also excludes candidates.
The approved schema's causal wording remains ambiguous; this is an executor
interpretation, not a schema or operator-policy amendment.

Outcome precedence on a successful list is `unavailable` if `U>0 || A>0`,
then `capacity_limited` if `O>0`, then `no_runs` if `R==0 && U==0`, otherwise
`complete`. Only `V` rows appear in `Runs`. A refused, over-cap, or undecodable
list yields the exact DTO with `run_list_state=unavailable`,
`observation_state=unavailable`, nonnil empty `Runs`, and all optional counts
absent; it starts no candidate read. It returns a completed empty lease whose
`Next` returns `ErrRunListUnavailable`. A failed auth/guard, context
cancellation, manager stop, or irreducible initial DTO assembly returns a
typed error, zero wrapper, nil lease, and performs owned rollback.

Preserve exact contract field names, JSON tags, enum strings, and optional
frame pointer values. Build the full `contract.RunTraceObservation` first,
call `boundRunObservationHydration` exactly once, and allow it to prune only
after the initial lease high-water marks have been captured. Preserve metadata
and per-run frame order; do not introduce a local replacement DTO. The helper
is authoritative for global oldest-timestamp pruning, exact RunID/EventID
tie-breaks, updated counts, and irreducible-metadata failure.

## Private API and types

All types and methods below are private proposal names; they do not authorize
new exported/public interfaces.

```go
type pollerKey struct { CaseID, RunID string }

type observedIdentity struct { EventID string; FirstOrdinal uint64 }
// The one owned frame representation. Pointer members are deep-copied at
// ingestion and cloned again when materializing a DTO/delta.
type retainedObservedFrame struct {
    Frame ports.RunTraceFrame
    FirstOrdinal uint64
}
type pollerVersion struct {
    Generation uint64
    ObservedAt string // RFC3339Nano capture time for this accepted read
    CaseID, RunID, PlanItemID string
    Execution, Settlement string
    TotalFrameCount int
    Frames []retainedObservedFrame // source order; sole retained frame storage
    HighestObservedOrdinal uint64
}
// Do not retain a ports.RunTraceSnapshot with its Frames slice alongside
// pollerVersion.Frames. Copy scalar metadata and frame records exactly once.

type WindowState struct {
    Generation uint64
    TotalFrameCount, RetainedFrameCount, OmittedFrameCount int
    Truncated bool
}
type RunDelta struct {
    RunID, PlanItemID, ObservedAt string
    Execution contract.RunExecutionStanding
    Settlement contract.RunSettlementStanding
    ObservationState contract.RunTraceRunObservationState
    Frames []contract.RunTraceFrame // nonnil, in captured source order
    Window WindowState
}
type RunWindowGap struct {
    RunID string
    PreviousOrdinal, CurrentOrdinal uint64
    EvictedObservedEventIDs []string // exact IDs, deterministic byte order
    EvictedObservedEventCount int
    CurrentWindow WindowState
    UnobservedLossPossible bool
    UnknownMissedFrameCount bool
}
type observationWatermark struct {
    HighestObservedOrdinal uint64
    Execution contract.RunExecutionStanding
    Settlement contract.RunSettlementStanding
    ObservationState contract.RunTraceRunObservationState
    Window WindowState
}
type ObservationDelta struct {
    CaseID, AsOfCursor string
    ObservedAt time.Time
    Runs []RunDelta // nonnil, exact RunID byte order
    WindowGaps []RunWindowGap // nonnil
}
type CohortRequest struct { CaseID string; Identity *requestIdentity }
type InitialCohort struct {
    Observation contract.RunTraceObservation
    AsOfCursor string
}
type CohortLease interface {
    Next(context.Context) (ObservationDelta, error)
    DetachAndDrain(context.Context) error
}
func NewRunObservationManager(
    runList runListReader, traces ports.RunTracePort,
    auth watcherAuthorizer, guard presentContextChecker,
) *RunObservationManager
func (m *RunObservationManager) Prepare(
    context.Context, CohortRequest,
) (InitialCohort, CohortLease, error)
func (m *RunObservationManager) StopAndDrain(context.Context) error
func (m *RunObservationManager) attach(
    context.Context, *cohortLease, ports.RunSummary,
    runObservationPresentContext,
) (attachResult, error)
```

`pollerVersion` is immutable after publication. Ingestion copies the frame
slice and separately copies each `ExecutionStatus` and `ExitCode` pointee;
neither the adapter's response nor a DTO/delta recipient can mutate committed
state. There is only one retained frame value per event in `Frames`, paired
with its ordinal. `Snapshot.Frames` is never retained. When projecting into the
initial DTO or a delta, copy frame values and optional pointers again. The
budget image serializes each committed frame value once, with its ordinal;
DTO/delta output is transient and is not also counted as poller-retained data.

`RunDelta.ObservedAt` and the initial DTO run's `ObservedAt` both come from the
exact captured `pollerVersion.ObservedAt`. Reusing a current version preserves
its timestamp. Every successfully accepted later source read gets its own
capture timestamp. No separate `firstAcceptedAt` is retained or consulted.
Capturing one version fixes its timestamp together with its window and state.

The typed error set includes `ErrCohortCapacity`, `ErrObservationCapacity`,
`ErrPrepareCanceled`, `ErrPrepareStopped`, `ErrPresentContextUnavailable`,
`ErrAuthorizationUnavailable`, `ErrRunListUnavailable`,
`ErrInitialAssemblyUnavailable`, `ErrLeaseCanceled`, `ErrLeaseStopped`,
`ErrManagerStopped`, `ErrPollerRetentionUnavailable`,
`ErrPollerReadUnavailable`, `ErrNextAlreadyInProgress`, and `ErrDrainTimeout`.
Every `Next` error returns a zero `ObservationDelta` and commits no watermark.
After a terminal auth/context/unavailable error the owner detaches and drains.
Timeout never frees capacity.

## State, lock, and exact ownership

Manager mutex protects only state transitions, maps/counts, immutable pointer
swaps, and reference acquisition. No list/read, authorization, guard, callback,
channel send, cancellation, wait, or join occurs while it is held.

```go
type pollerState uint8 // initializing, running, stopping, draining, removed
type leaseState uint8  // preparing, active, draining, removed
type pollerAvailability uint8 // current, retentionUnavailable, readUnavailable
type attachDisposition uint8 // current, sharedInitializer, newInitializer,
                             // pollerCapacity, cohortCapacity, draining,
                             // identityMismatch, invalid
type pollerEntry struct {
    key pollerKey
    planItemID string
    state pollerState
    current *pollerVersion
    seenByID map[string]uint64
    highestOrdinal uint64
    availability pollerAvailability
    failedGeneration uint64
    terminal bool
    leases map[uint64]struct{}
    workerCancel context.CancelFunc
    workerDone chan struct{} // closed only by worker after actual return
    ready chan struct{}      // initializer completion
}
type cohortLease struct {
    id uint64
    state leaseState
    refs map[pollerKey]*runRef
    watermarks map[pollerKey]observationWatermark
    wake chan struct{} // capacity one; NEVER closed
    leaseDone chan struct{} // cancellation signal; closed exactly once
    doneOnce sync.Once
    notifyWG sync.WaitGroup
    operations sync.WaitGroup
    nextInFlight bool
    identity watcherIdentity // never stores bearer token
    asOfCursor string
}
type RunObservationManager struct {
    mu sync.Mutex
    entries map[pollerKey]*pollerEntry
    leases map[uint64]*cohortLease
    stopping bool
    list runListReader
    traces ports.RunTracePort
    authorize watcherAuthorizer
    guard presentContextChecker
}
```

**Notifier rule:** notification ownership is per target lease/channel, never a
single ambiguous entry count. While holding the manager mutex, a publisher
acquires one `target.notifyWG.Add(1)` for each eligible lease and records those
specific targets. It unlocks, makes a nonblocking send on each target's
capacity-one `wake`, then calls that target's `Done` exactly once in a defer,
including if the send is coalesced. A lease transitions to draining under the
same mutex that gates acquisition, so no new notifier Add can occur after
drain begins. Its owner calls `notifyWG.Wait()` outside the mutex before
finishing detach. No notifier closes `wake`; no send-after-close is possible.
`leaseDone` is a separate once-closed cancellation signal and is never sent to.
The poller worker alone closes its own `workerDone` after the adapter call and
client retirement have actually returned. No waiter closes worker-owned
channels.

`operations` follows the same rule: Add only while the lease is active and
under the manager mutex; drain changes the state under that mutex before
waiting outside it. All refs belong to one lease and are detached exactly once.
The 16-lease count includes preparing/active/draining. The 16-entry count
includes initializing/stopping/draining. A slot is removed only after no
lease refs/notifier refs remain, owned poller work is canceled as appropriate,
and `workerDone` has closed. Concurrent stop/detach is idempotent.

### Prepare, attach, rollback, and shared initialization

`Prepare` first validates request identity and the exact present context outside
the lock, then reserves a preparing lease slot under lock before doing list or
trace work. If manager stop or the proposed 16-cohort limit rejects it, no
list/read begins. It stores no credentials beyond the existing watcher
identity requirements. The request context owns only this Prepare's list and
initial wait; a shared poller worker has a manager-owned cancellation context,
so one watcher's departure cannot cancel work needed by another.

`attach` has a typed disposition for a verified current cache, joining an
existing initializer, owning one newly reserved initializer, poller capacity,
cohort capacity, draining key, identity mismatch, and invalid candidate.
Under lock it checks exact case/run/plan-item identity, lease eligibility, and
membership in the copied current Horizon parent set. Conflicting plan identity
for one key is unavailable, never overwritten. It acquires a ref only after
those checks. The initializer owner alone may publish; waiters join its
completion and do not start another logical trace read. All adapter calls and
waits happen outside the lock. A verified shared current buffer is reused
without a logical read and preserves that version's timestamp.

Every failure after lease reservation has an internal rollback owner because
the caller receives no lease on error. Rollback marks the provisional lease
draining under lock, which bars new refs, operation Adds, and notifier Adds;
detaches each acquired ref once; cancels a poller only when no other authorized
lease still needs it; and closes `leaseDone` once. Outside the lock it waits
for target notifier refs and lease operations, joins its list call, joins an
initializer it owns or made watcherless, and joins any poller worker it owns
or made watcherless. Shared work with another eligible watcher continues.
Capacity remains counted until all actual owned work has returned and joins
complete. `ErrDrainTimeout` leaves the lease/entry in draining and counted;
only a later successful owner join can remove it. `StopAndDrain` closes
admission under lock, performs the same transitions idempotently, and joins
outside the lock.

### Authorization and present-context checks

Each watcher is independently checked before list, every selected trace
read/attach publication, initial handoff, and each later disclosure. Session
watchers use `SessionStore.Current(id)` for current identity/deadline/revocation,
verify exact actor/role claim equality, and call the current perspective
verifier. Existing development-only static bearer behavior remains bounded by
request context and configured actor/role; it has no invented session-current
or revocation promise, and no bearer value is retained. Shared buffers never
confer authorization.

Call the existing `checkRunObservationPresentContext` outside the manager
lock. It uses newest retained Store history and `Relay.CursorForCase`, validates
case/cursor/Facts/Record/Overview/Horizon identities, nonnil horizon items,
unique nonblank parents, and exact cursor equality. It performs no kernel
Facts read. Copy the derived parent set for validation. Recheck auth and guard
immediately before disclosure and require the captured `AsOfCursor` to remain
equal; a change returns `ErrPresentContextUnavailable`, zero delta, and no
watermark advancement. Store/Relay checks are non-atomic and do not guarantee
continuous truth or authorize public readiness/frontier behavior.

## Exact ledger, read failures, version capture, and deltas

The source is not a durable sequence contract. The Rust reader can return no
rows for unreadable/oversized data and can stop at a malformed tail; the
recorder's `tev` counter is per recorder and advances before fallible writes;
the Go adapter retains at most the last 1,024 allowlisted frames in source
order and reports one response's recognized total. No source generation,
cross-read append-only proof, durable high-water mark, or exact missed-row
count exists. `event_id`, timestamp, and `TotalFrameCount` are opaque/current
response facts, not cross-read sequence keys.

For each entry lifetime, maintain exact `SeenByID map[string]uint64` and a
monotone `HighestObservedOrdinal`. On a fully decoded, validated read, copy
the ledger and ordinal, scan returned retained frames in source order, and
assign the next ordinal only to an exact unseen string. Repeated IDs retain
their first ordinal even after eviction/reappearance or payload rewrite. Never
parse, normalize, or sort IDs to choose delivery order. Check ordinal overflow
before accepting. Atomically publish the complete immutable version, copied
ledger, counts, stands, timestamp, and ordinal only if the proposed combined
budget fits. No rejected partial ID becomes accepted or disclosed.

On a failed `ReadRunTrace`, publish no candidate version and no ledger change.
Set `availability=readUnavailable` plus a failed-generation marker and wake
attached leases. `Next` returns **`ErrPollerReadUnavailable`**, zero delta, and
no watermark advance; it must not present the previous buffer as current.
A later complete successful read may recover by committing a full version
against the unchanged ledger and then clearing the marker. This read-failure
state and recovery rule are private manager behavior; no source continuity is
inferred.

For a candidate rejected by the proposed combined retention budget, commit
neither IDs/ordinals nor frame/window/stand/timestamp state. Keep the previous
immutable version only for internal reference, mark current retention
unavailable with the failed generation, wake leases, and return
`ErrPollerRetentionUnavailable` with zero delta/no watermark advance. Never
serve the old version as current. Recovery requires a complete later candidate
that fits with the entire old ledger; never reset/evict the ledger to make
room. A terminal state in an unretainable candidate is not disclosed and
stops further reads only after the actual read returns and the worker is
joined. These budget overflow, terminal, and recovery policies remain
proposal-only pending review and operator approval.

At initial attachment, capture the exact immutable version and copy its
`HighestObservedOrdinal` into the lease watermark **before** constructing and
pruning the initial DTO. Thus helper-pruned frames are neither replayed in
`Next` nor reported as a poller eviction gap. Each initial run DTO's
`ObservedAt` is copied from that captured version's `ObservedAt`. Cache reuse
keeps that version time; a newly accepted poll has a distinct capture time.

For one `Next`, serialize concurrent calls per lease. Wait outside locks for
`wake`, caller cancellation, `leaseDone`, or manager stop. Under lock, capture
one immutable version (or typed unavailable marker) per attached key plus
this lease's scalar watermarks, then unlock. Recheck auth/guard, build a full
candidate delta and watermarks only from that captured set, then do final
auth/guard checks before disclosure. Do not retry to catch a global generation
or recapture a newer version. Under lock, if the same lease is still active and
owns its single-Next token, commit only its candidate watermarks and return
the captured-version delta. If a poll advanced while building, preserve/set
the coalesced wake hint for the next call. The current delta is explicitly
as-of its captured versions.

Frames are current retained frame values whose first ordinals exceed the
previous watermark, in current source-window order. Advance the candidate
watermark to the captured version's highest ordinal. For IDs above the prior
watermark absent from that current window, report the exact opaque IDs in
`EvictedObservedEventIDs`; the count equals its length. `UnknownMissedFrameCount`
is true when a gap is reported; do not convert ordinals or response totals to
a source-loss count. Source rows never observed remain unknown. Standing-only
or window-only change may yield an empty-frame delta. `Window` is exactly the
captured response total, retained length, their difference, and derived
truncation; totals can move either direction. Use exact RunID byte order for
delta/gap ordering, while preserving frame source order. A `Next` error,
cancellation, auth/guard change, stop, or assembly error returns zero and
commits no watermarks.

## Proposed combined serialized retention image

This private accounting image has an explicit version tag, explicit structs
and slices (no map iteration), and includes exactly once every retained
value-bearing field: exact case/run/plan identities; version generation and
capture time; highest and next ordinal; execution/settlement/run/availability
and terminal state; next-poll scalar if retained; source total and
retained/omitted/truncated metadata; the sole current frame records with
optional pointer values and ordinals in source order; every exact seen-ID and
first-ordinal pair sorted only for canonical encoding by unsigned UTF-8 byte
sequence; and every other retained safe scalar/slice that could grow. No
retained safe value may be excluded as an auxiliary field. `Snapshot.Frames`
is absent, so frame values are not double-counted. Locks, channels, contexts,
wait-group/reference state, fixed-size controls, decoded uncommitted
responses, and transport buffers are outside this serialized-value budget
and are not claimed bounded by it.

Marshal the candidate image with `encoding/json`; accept only when encoded
length is `<= 1<<20`. Marshal error or greater size rejects atomically. New
retained fields require updating this image and exact-boundary tests before
source review. For a first selected candidate that does not fit, count it in
`A`, not `C`, and install no poller. For an existing entry, mark retention
unavailable as above. Budget and overflow semantics remain unapproved.

## Outcome table

| Condition | Result | Counts/state |
|---|---|---|
| Proposed 16 cohort slots occupied | `ErrCohortCapacity`, zero wrapper, nil lease | No list/read; preparing/active/draining all consume slots |
| Accepted 16 poller slots occupied or exact key draining | Selected row in `C` | Counts successful list truthfully; no backfill |
| Failed/refused/over-cap/undecodable list | Unavailable DTO, completed empty lease | Count pointers absent; no candidate reads; `Next` is `ErrRunListUnavailable` |
| Successfully decoded empty list | `no_runs`, active empty lease | Every count pointer present as zero |
| Selected invalid/denied/mismatch/orphan row | Row in `A` | No run payload; no backfill |
| Successful validated candidate/cache | Row in `V` | Exact ownership and parent validated; cache does not count as read |
| Initial helper irreducible metadata error | `ErrInitialAssemblyUnavailable`, nil lease | Roll back all refs/work and join before capacity release |
| Failed later trace read | `ErrPollerReadUnavailable`, zero delta | No ledger/version change; no watermark move; later complete read may recover |
| Proposed retained image over budget | `ErrPollerRetentionUnavailable`, zero delta | No partial commit/reset; no old version as current; proposed policy unapproved |
| Later full candidate fits old ledger | Normal captured version | Atomic recovery and wake; no ledger loss |
| Auth or final present cursor check fails | Typed unavailable, zero delta | No watermark move; detach/drain on terminal error |
| Another `Next` already owns lease token | `ErrNextAlreadyInProgress` | First call retains token; no overlapping commits |
| Failed `Prepare` after partial attachment | Original typed Prepare error, nil lease | Internal rollback owns all cleanup; caller cannot be relied on |
| Drain timeout | `ErrDrainTimeout` | Remains draining and counted; no slot reuse |
| Terminal state fits and is accepted | Final version disclosed, then poll stops | Worker actually joins before slot release |
| Terminal state is in unfit candidate | Typed retention unavailable | Not disclosed; stop only after read return/join; proposal unapproved |

## Test-first review matrix and gates

No test or runtime evidence is claimed. Before any separate source assignment,
a different reviewer must inspect the original assignments, full revisions 5
and 6, revision 5 rejection, source-continuity recon/review, accepted selector
and hydration evidence, and source anchors. The subsequent implementation
assignment must require focused RED before code and these distinct fixtures:

1. **DTO/helper:** exact DTO fields/enums, present zero counts versus absent
   failure counts, `R=8/9` and reads `7/8` exhaustion, optional frame pointers,
   exact cap and tie ordering, preserved metadata/counts/order, non-aliasing,
   zero-frame runs, typed irreducible failure, initial DTO timestamp equals
   captured version timestamp on cache reuse and after a changed version.
2. **Cohort/list:** `R/U/S/V/A/C/O` equations and outcome precedence, no
   backfill/unreadable promotion, selector order, logical versus physical
   attempts, exact verified cache/shared initializer behavior, proposed
   cohort limit before list/read, and preparing/active/draining slot release
   only after joins. Proposed cap tests do not imply approval.
3. **Ledger/window:** arbitrary opaque IDs including `tev_0001`, repeated,
   reordered, rewritten, evicted then reintroduced IDs; source order; exact
   entry-lifetime replay boundary; ordinal overflow; nonmonotonic source total;
   first-observation high-water before initial pruning; exact observed-eviction
   IDs/count; unknown source loss; standing/window-only empty frames.
4. **Read and retention failure:** failed read produces the declared typed
   `ErrPollerReadUnavailable`, zero delta, no watermark change, does not serve
   previous version, then later complete recovery; budget exact byte boundary,
   marshal error, all retained metadata/aux/ledger included once, no map-order
   dependence, atomic reject/no ID acceptance, typed unavailable/no advance,
   later fit with old ledger, terminal-overflow behavior, initial `A` versus
   poller `C`. All budget behavior is held pending approval.
5. **Next races:** one captured version set; poll advances during assembly;
   auth and guard before and after; final cursor mismatch returns typed
   unavailable/zero/no advance; no generation retry; preserve later wake;
   only this lease's watermarks commit; concurrent Next is typed and zero.
6. **Ownership/rollback:** every failure after reservation/list/partial attach/
   helper error/final handoff; initializer owner/waiter race; shared watcher
   survives detach; concurrent Stop; callbacks, waits, sends, cancels and joins
   all outside manager lock; cancellation only without another authorized
   watcher; `workerDone` only after actual client read/retirement; timeouts
   stay counted; per-target notifier Add/Done exactly once on send and
   coalesced paths; draining blocks Add before Wait; wake never closes;
   once-closed leaseDone; no send-after-close.
7. **Authorization/present guard:** session Current, deadline, revoked state,
   claim and current perspective on every boundary; configured development
   bearer with no session/revocation claim or retained token; exact identities
   and Horizon parent membership; newest-only Store/Relay equality; no kernel
   Facts extra read; as-of/non-atomic limitation explicit.
8. **Bounds/client:** 16 poller entries include initialization/drain, accepted
   shared two-physical attempt limiter/retry/cooldown/cancellation/retirement,
   one-second floor, 1,024 frame window, separate accepted initial DTO cap,
   separate unapproved 16 lease/128 attachment and 1 MiB combined retained
   image; no heap/process-memory claim.

All focused RED/GREEN, full race/module/canonical gates, runtime captures, and
compiler ownership remain separately assigned to root's sole compiler owner
under fresh-resource and joined-result protocol. This document authorizes
none of those operations.

## Evidence anchors and factual limits

- Normative T09 extension and observational boundary:
  `.agents/reports/casework-live-wiring/t09-contract-extension-proposal.md:27-45`;
  `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml:82-106`.
- Exact DTO/enums and accepted hydration helper:
  `apps/godspeed-casework-go/internal/contract/contract.go:186-238,413-442`;
  `internal/server/run_observation_hydration_cap.go:22-78,80-149,177-194`.
- Snapshot and pointer-valued frame fields:
  `apps/godspeed-casework-go/internal/ports/run_trace.go:5-30`.
  `RunTraceSnapshot` owns `Frames`; `RunTraceFrame` has optional
  `ExecutionStatus *string` and `ExitCode *int64`, motivating the single owned
  frame representation and deep clone rule.
- Adapter identity/duplicate/frame-window behavior:
  `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go:66-98,103-177`.
- Source continuity limitations:
  `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-get-trace-source-continuity-recon-oct06.md`;
  `crates/sea-forge-server/src/sfwp/run_views.rs:417-436,835-857,947-957`;
  `crates/sea-forge-trace/src/lib.rs:78-122`;
  `crates/sea-forge-core/src/types.rs:528-540`.
- Selector and root acceptance:
  `apps/godspeed-casework-go/internal/server/run_observation_selection.go:64-80`;
  `run-observation-selection-phase2-root-acceptance-oct06.md`.
- Present-context implementation and retained Store/Relay seams:
  `apps/godspeed-casework-go/internal/server/run_observation_present_context.go:25-89`;
  `internal/projection/store.go:41-53,122-139`;
  `internal/ports/ports.go:266-274`;
  `internal/server/relay.go:101-150,170-175`. The guard is source-reviewed,
  not a runtime GREEN or manager-callsite approval.
- Authorization seams:
  `internal/auth/session.go:147-169`, `internal/server/server.go:38-42`,
  `internal/server/session.go:45-70`. Session currentness does not itself
  prove a client read drained.
- Shared physical ownership and retirement:
  `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission.go:15-25,80-156,193-235`;
  `apps/godspeed-casework-go/internal/adapters/sfwp/client.go:489-531`.
  The manager must retain actual join-before-removal semantics.
- Injected ports:
  `apps/godspeed-casework-go/internal/ports/run_trace.go:5-8` and
  `apps/godspeed-casework-go/internal/ports/ports.go:454-471`.

## Deviations and unresolved approvals

Revision 6 preserves all accepted revision 5 decisions: exact DTO/helper;
process-shared 16 poller entries; 8-row selection and logical read limits;
`V+A+C=S`, `O=(R-S)+C`, and `Exhausted=(ReadsAttempted==8 && R>8)` with the
schema ambiguity disclosed; one-second poll floor; accepted shared physical
admission/cooldown/cancellation/retirement ownership; present-context guard;
per-watcher auth; exact entry-lifetime ID ledger; source-order ordinals;
scalar per-lease watermarks; no generation retry; and rollback/join before
capacity release.

It repairs revision 5 by adding the missing proposed 16-cohort cap and
128-attachment maximum; enumerating `ErrPollerReadUnavailable` with zero
delta/no advance and recovery; defining initial and delta timestamps from the
exact captured version and removing `firstAcceptedAt`; retaining only one
deep-copied frame+ordinal value representation; and specifying per-target
notifier Add/Done ownership, nonclosing wake channel, once-closed `leaseDone`,
and detach wait outside locks.

The 16-cohort/128-attachment admission rule, combined serialized 1 MiB
ever-observed ledger/window/auxiliary-value budget, and its overflow,
unavailable, terminal, and recovery transitions remain **proposals requiring
independent architecture review and operator approval**. They are not treated
as approved limits or an implementation basis. The root exhaustion rule is an
executor interpretation, not a schema amendment. Existing 32 MiB source-line
cap remains an unimplemented prerequisite. Source continuity and unobserved
loss remain unknown; entry removal permits later replay. Store/Relay guard
remains as-of and non-atomic. No source, tests, public contract, runtime,
frontier, or T09 settlement is approved by this document.

This revision is intended to close the review gaps, not to represent review
acceptance. No tests, compiler, scanner, Graft build, Git, or network action
was run or claimed. Earlier proposal/review documents remain unchanged.
