# Private run-observation manager — concrete proposal

Date: 2026-10-06  
Status: source-grounded proposal for root architecture review; no implementation is released.

## Decision boundary

Keep the operator-approved process-shared key `(case_id, run_id)`, at most 16 manager entries, at most two concurrent physical `run_get` attempts, one-second per-run poll floor, eight selected summaries, and at most eight initial logical trace calls. The manager owns watcher leases, shared poller state, cohort assembly, bounded frame retention, and cancellation/join. The accepted SFWP client owns physical admission, retries, write cooldown, connection cleanup, and permit release. No manager-level physical semaphore or alternate limiter is proposed.

All manager operations remain private to `internal/server`. This proposal adds no public DTO, port, HTTP route, SSE cursor rule, kernel verb, or identity model. The present `/api/events` route has no selected case ID, so the manager cannot be wired to that route under this proposal alone. V4/SSE/frontier wiring stays held.

## Private boundary and proposed types

Implement the deterministic cohort assembler separately from the shared poller registry. It consumes existing ports and a caller-supplied present-case context; the context avoids a new hydration-time `run.list` or `run.get` for parent validation.

```go
type observationRunLister interface {
    RunsListForCase(context.Context, ports.CaseRef) (ports.RunListResult, error)
}

type observationTraceReader interface {
    ReadRunTrace(context.Context, string, string, string) (ports.RunTraceSnapshot, error)
}

type observationWatcher struct {
    id         uint64
    caseID     string
    claim      ports.ActorClaim
    sessionID  string // empty only for an existing development-only bearer
    validUntil time.Time
    revoked    <-chan struct{}
    requestCtx context.Context
    updates    chan observationUpdate // bounded; exact bound/action remains a release prerequisite
}

type observationKey struct{ caseID, runID string }

type presentCaseContext struct {
    CaseID        string
    Cursor        string
    ParentItemIDs map[string]struct{}
    Fresh         bool
}

type observationCohort struct {
    Initial RunTraceObservation
    Leases  []*runObservationLease
}

type observationUpdate struct {
    Cursor string // informational case cursor; never an SSE id
    Run    RunTraceRunObservation
}

type attachResult uint8 // attached, joined-initializer, or capacity-omitted

type observationEntryState uint8 // initializing, running, stopping, draining

type observationEntry struct {
    key       observationKey
    state     observationEntryState
    planItem  string
    current   ports.RunTraceSnapshot
    hasCurrent bool
    watchers  map[uint64]*observationWatcher
    ctx       context.Context
    cancel    context.CancelFunc
    ready     chan struct{} // closes once the initial read succeeds or fails
    done      chan struct{} // closes only after worker/read cleanup has returned
}

type runObservationManager struct {
    mu      sync.Mutex
    closed  bool
    ctx     context.Context
    cancel  context.CancelFunc
    list    observationRunLister
    trace   observationTraceReader
    history RevisionHistory
    sessions *auth.SessionStore
    verifier PerspectiveVerifier
    entries map[observationKey]*observationEntry // len includes initializing/stopping/draining
    workers sync.WaitGroup
}

type runObservationLease struct {
    manager  *runObservationManager
    key      observationKey
    watcherID uint64
    updates  <-chan observationUpdate
    once     sync.Once
}

func newRunObservationManager(
    parent context.Context,
    list observationRunLister,
    trace observationTraceReader,
    history RevisionHistory,
    sessions *auth.SessionStore,
    verifier PerspectiveVerifier,
) *runObservationManager

func (m *runObservationManager) Prepare(
    ctx context.Context,
    watcher observationWatcher,
    present presentCaseContext,
) (observationCohort, error)

func (m *runObservationManager) attach(
    ctx context.Context,
    watcher *observationWatcher,
    run ports.RunSummary,
) (*runObservationLease, attachResult, error)

func (l *runObservationLease) Updates() <-chan observationUpdate
func (l *runObservationLease) Detach()
func (m *runObservationManager) StopAndDrain(context.Context) error
```

`RunTraceObservation` and `RunTraceRunObservation` above refer to the existing contract DTO types; the cohort returns a newly assembled value and never aliases a mutable shared buffer. `Fresh` is a caller assertion backed by the stated stored-cursor/relay comparison, not a manager claim of live kernel truth.

These narrow interfaces are package-private test seams over existing `CaseAuthorityPort.RunsListForCase` and `RunTracePort.ReadRunTrace`; they do not replace or widen the ports. The production authority already implements both. The constructor is a private manager constructor, not a new server/public constructor. A later server assembly bridge requires a separate release because `Server` currently receives a `WorldSource`, not a `RunTracePort`.

`presentCaseContext` contains `CaseID`, the newest stored case cursor, its `Horizon.Items` parent IDs, and a freshness verdict from the caller. `observationCohort` contains the initial DTO and the leases retained for the stream lifetime. `observationUpdate` contains only new safe frames for an already validated `(run_id,event_id)` identity plus the real case cursor as informational metadata; it never carries an SSE id.

## Present-case and watcher preconditions

Before `Prepare` reads the list, the caller must supply a nonblank requested case and a trusted latest-known present context for that same case. The manager rejects a missing `Facts`, mismatched case/horizon identity, blank or duplicate parent IDs, cold history, an evicted revision, or a relay/store cursor gap. It uses `Horizon.Items` captured in the latest retained `Revision.Facts`; it makes no extra `CaseHorizon`, `run.list`, or `run.get` call for this check. This is knowledge as of a real stored cursor, not a claim about continuously current kernel truth.

The current `Store.Trajectory(caseID)` can provide retained revisions and their `Facts.Horizon`; compare its latest stored cursor to `Relay.CursorForCase(caseID)` before treating that context as present. A mismatch means capture has not caught up or failed, so the private manager rejects the context. No case selector currently reaches `/api/events`, and a historical `?cursor=` world response is not present-world context. These are caller preconditions, not new public readiness behavior.

Every watcher owns a separate authorization lease. For session-backed watchers, the caller records the session ID and actor claim after `requireSession`; `Current(id)` must still return the same claim, valid deadline, and revocation signal before list access, each trace start, and every later delivery. Check both the request context and revocation signal. `Current` does not slide `LastSeen` and does not reflect kernel role/delegation changes, so also call the existing `PerspectiveVerifier.VerifyPerspective(ctx, claim)` at those boundaries. A failed session/perspective check detaches that watcher and prevents its disclosure; it does not cancel work needed by another authorized watcher.

Development bearer requests remain confined to the existing development-only mode. A bearer has no `SessionStore.Current` state or revocation signal; its request context is the only local lifetime signal, and its perspective must still be rechecked before each disclosure. This proposal makes no bearer-revocation guarantee. Root must confirm whether observations are allowed for such streams or withheld while ordinary case revisions continue.

The read ports do not accept `OnBehalfOf` and return unpersonalized read models. A successful perspective check is a watcher gate; it does not make `RunsListForCase` or `ReadRunTrace` an actor-attributed kernel read. Every watcher must pass its own gate before shared data is disclosed. No watcher/session/role is part of the approved poller key.

## Cohort assembly and count equations

The existing `selectObservationRuns` orders all readable summaries and selects the first eight. The scoped port validates its arrays, case ownership, standings, identities, and duplicates before returning. Rust filters requested-case ownership before putting IDs in `unreadable`; those IDs are claimed by that case but have no readable summary and are never selector input.

For a completed decoded list, define:

- `R = len(result.Runs)` — readable summary rows.
- `U = len(result.UnreadableIDs)` — scoped case-claimed unreadable IDs.
- `S = min(8, R)` — selected readable candidates.
- `V` — selected rows with an exact case/run/plan-item snapshot and a parent present in the trusted horizon; exact verified cache reuse counts here.
- `A` — selected rows denied, malformed, identity-mismatched, or orphaned. No frames are projected for these rows.
- `C` — selected rows not attached because the shared 16-entry capacity is unavailable, including an exact key still draining.
- `O = (R - S) + C` — readable rows omitted by the cohort selection or shared capacity.
- `reads_attempted` — logical `ReadRunTrace` invocations started for this cohort. Increment immediately before invocation, including a queued failure. Client transport retries do not increment it. Exact shared-current-buffer reuse, or joining another cohort’s in-flight initializer without invoking `ReadRunTrace`, adds zero to this cohort’s count.

The outcomes of selected rows are disjoint: `V + A + C = S`. Read failures are unavailable, never omitted, and never cause replacement selection. Continue through the already selected rows after an individual read failure while the watcher remains valid. Do not start candidates after the first eight, and do not exceed eight calls. If a session/context check fails, abort publication, stop future starts, cancel work no longer needed by any watcher, and join outstanding reads.

| Result | Counts and state |
|---|---|
| `run.list` refused, over-cap, undecodable, or otherwise unavailable | All optional counts absent; `run_list_state=unavailable`, `observation_state=unavailable`, `runs=[]`, no candidate reads. |
| Successful decoded list | All count pointers present, including zeroes; `listed=R`, `selected=S`, `unreadable=U`, `unavailable=A`, `omitted=O`, `validated=V`. |
| No readable or unreadable rows | `no_runs`, with all counts zero and `reads_attempted=0`. An unreadable ID prevents `no_runs`. |
| Any `U>0` or `A>0` | `unavailable` takes precedence over capacity; retain exact counts and any independently validated run observations. |
| No failures and `O>0` | `capacity_limited`; retain known omissions. |
| No failures and `O=0`, with at least one row | `complete`. |

The scoped list can only report the response it produced; the kernel currently suppresses some filesystem enumeration/read failures. `run_list_state=complete` therefore means the list response decoded and passed adapter validation, not that all upstream directories or journals were proven complete.

### Read-budget exhaustion ambiguity

For review, this proposal follows the original manager assignment’s candidate interpretation: `exhausted = (reads_attempted == 8 && R > 8)`. This marks additional readable rows as unvalidated when all eight initial logical calls were actually consumed, even though those rows are also beyond the first-eight selection boundary. Cached reuse does not count; fewer than eight invocations always means `exhausted=false`.

This is not settled policy. Proposal text says `exhausted` is true only when the read limit prevented a known candidate from validation, but also says to set it when the eight-call budget is exhausted while additional known candidates remain. Because the cohort and read limits are both eight, an alternate causal reading treats rows beyond `S` as omitted by selection and sets exhausted false. Existing TypeScript/Go checks constrain the field shape/range, not this formula. Root/critic must resolve the ambiguity before fixture or implementation release.

## Shared poller state and lifecycle

The manager mutex protects only `closed`, the bounded key map, entry state, watcher membership, and immutable current-buffer publication. No session lookup, perspective verification, source call, channel send, timer wait, cancellation join, or other callback runs under this mutex. Copy references under lock, unlock, perform the operation, then reacquire and verify the same entry generation/watcher ID before committing the result.

An entry uses this state machine:

```text
absent -> initializing -> running -> stopping -> draining -> removed
                       \-> stopping -> draining -> removed  (terminal/read failure)
```

- `initializing`: one reserved map entry and one worker own a single initial `ReadRunTrace`; concurrent same-key attachers join its `ready` result rather than starting duplicate calls.
- `running`: one worker owns at most one read at a time and schedules subsequent polls no faster than one second after the prior logical poll finishes. Every call uses the accepted shared SFWP client hook; the manager adds no second physical limiter or two-read semaphore.
- `stopping`: no new poll starts; cancel the entry context. A watcher detaches independently. Cancel only when the last watcher leaves, every watcher fails authorization, the run becomes terminal, or manager shutdown begins.
- `draining`: the map entry still consumes one of the 16 slots until the worker and `ReadRunTrace` return. The accepted client path joins cancellation callbacks and retires/releases the connection before releasing its physical permit; the manager then joins its worker before removing the key.
- `removed`: only after `done` closes and the worker has returned. An attach racing with a draining same-key entry receives a capacity omission for this cohort; it cannot replace that entry early.

The entry context derives from the manager/server lifetime, not from the first watcher request. If one watcher cancels, remaining watchers may still need the in-flight read. Its own context and revocation signal only detach that lease. A last-watcher detach cancels the shared entry; the slot remains occupied until the read drains. A terminal result is delivered once to watchers that pass the final authorization check, then the entry stops and drains. Shutdown marks the manager closed under lock, snapshots and cancels entries outside the lock, and waits for all workers before returning. No goroutine, map entry, or slot survives a successful `StopAndDrain`.

At capacity, never evict an active, initializing, or draining entry. A new key is omitted and increments `C`; its later SSE cohort may retry. Same-key reuse requires exact `RunID`, `CaseID`, and `PlanItemID`, a valid current parent, and a verified current buffer. A mismatched cached identity is never disclosed; report that selected row unavailable rather than silently treating the buffer as a hit. Root should review whether a fresh read can replace a mismatched live entry after it drains.

Each watcher has its own bounded delivery queue and lease. Pollers publish snapshots without blocking on network writes. Before each poll boundary and each watcher delivery, validate that watcher outside the manager lock; detach only watchers whose session/request or current kernel perspective fails. The remaining authorized watchers continue to share the same key. A slow-watcher queue must remain bounded; the exact queue capacity and overflow action need root/critic decision before implementation because the public observation DTO has no dedicated server backpressure field. It must not silently claim complete delivery or grow without bound. The approved 16-entry bound limits pollers, not leases: until a separate bounded watcher/lease policy is selected, concurrent stream attachments can still make queue memory unbounded. No watcher-count limit is invented here.

## Frame and payload bounds

Retain at most 1,024 allowlisted frames per shared run buffer. Initial run observations preserve the port’s `TotalFrameCount`; retained count equals emitted frames; omitted count equals `TotalFrameCount - retained`; `truncated` is true exactly when omitted is positive. Keep frame order within each run. For later polls, diff `(run_id,event_id)` against the bounded buffer and publish only unseen frame IDs. Eviction ends the dedupe guarantee; do not promise unbounded exactly-once behavior.

Marshal the initial `RunTraceObservation` JSON payload (not `event:`/`data:` SSE framing). While it exceeds 1 MiB, remove the globally oldest parsed timestamp frame; ties use full `RunID` bytes, then `EventID` bytes. Preserve the order of remaining frames inside each run and update retained/omitted/truncated counts after each removal. If required metadata alone exceeds the cap, return typed unavailable and emit no observation payload. Do not drop required counts, identities, or runs, and do not fabricate counts. This fail-closed path is a proposal and requires independent review.

## Bounded test-first decomposition

Each source unit starts with a fixture that distinguishes correct behavior from its failure mode and reaches an assertion RED before implementation. The existing selector fixture and selector source are frozen and remain untouched.

1. **Cohort classification** — add only `run_observation_cohort_test.go`, then its private source file. Cover list unavailable vs decoded empty; `R/U/S/V/A/C/O` equations; scoped unreadable IDs; top-eight omission; all selected reads attempted after an individual failure while the watcher remains valid; no replacements; exact run/case/item and parent validation; cache reuse at zero reads; logical retry accounting; and both exhaustion readings (`R=8` vs `R=9`, `reads_attempted=7` vs `8`). Freeze the chosen exhaustion behavior only after root/critic resolves it.
2. **Registry and leases** — add only manager fixtures/tests, then manager source. Cover exact shared key, simultaneous same-key initialization with one call, same run ID in different cases, 16-entry capacity including initialization/draining, no active eviction, same-key draining attach, and concurrent attach/detach without waits under lock.
3. **Lifecycle/auth/drain** — use a blocking fake reader and verifier to cover one watcher leaving while another remains, last-watcher cancellation, revoked/expired session, kernel perspective denial, bearer policy, terminal read, shutdown, cancellation before a later selected start, worker/read join before slot reuse, and no post-revocation fanout. Test strict `ValidUntil` handling without timer spin. Do not claim the auth-store test proves manager drain.
4. **Frame and envelope** — cover safe-field preservation, 1,024-frame rolling retention, new-ID-only updates, bounded watcher backlog policy, deterministic cross-run oldest-frame pruning with timestamp/ID ties, exact byte size at and over 1 MiB, accurate counts, and metadata-only overflow returning unavailable without an oversized payload.
5. **Integration, after separate release** — the server wiring must prove current explicit case input, no SSE `id:` or cursor advancement, separate authorization for each watcher, unchanged historical snapshots, and real temporary-cell frames. UI/schema/golden and public route work remains held for V4.

After source review, root must grant one sequential compiler owner. Proposed later gates are the focused manager race suite, all `internal/server` race tests, full Go module race tests, `just casework-go-check`, and a real temporary-cell integration exercise. Each requires fresh resource preflight, joined exit, and exact capture review under current T09 protocol. None ran for this proposal.

## Review requirements, differences, and open prerequisites

There is no intended difference from the operator-approved shared-key, poll, frame, or cohort limits. This proposal makes explicit the root-assigned meanings for `listed=R`, `unreadable=U`, `selected=min(8,R)`, `validated=V`, `unavailable=A`, and `omitted=(R-S)+C`; the simultaneous selection/read exhaustion meaning remains proposed only. The metadata-only 1 MiB failure returns typed unavailable with no oversized event, as directed by the assignment, but that treatment requires independent review.

Required before implementation: root/critic adjudication of exhaustion semantics; approval of the private bearer behavior and slow-watcher queue policy; confirmation that the latest stored horizon plus relay-cursor equality is the accepted current-parent guard; a case-selection/source injection seam that does not rely on the held public SSE/V4 work; and independent review of the exact constructor, attach, lease, stop, count, and drain contracts. The stored horizon can be stale as-of-cursor and is not continuous kernel truth. Session `Current` is lifetime state, not live kernel role/delegation state. Existing run ports do not carry watcher identity. None of these facts is resolved by this proposal.

No implementation, public/API wiring, compiler, test, scanner, Git, status, debt, or Graft build was run. Root retains architecture acceptance and all gate ownership.

## Direct source and contract references

- Approved cohort, unreadable/unavailable/omitted distinctions, exhaustion wording, 1 MiB pruning, and poller stop: `.agents/reports/casework-live-wiring/t09-contract-extension-proposal.md:31,35,39,41,43,45`.
- Normative observation bounds: `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml:82-106`; canonical event/state description: `.agents/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:227-237`.
- DTO, read budget and absent-vs-zero contract: `apps/godspeed-casework-go/internal/contract/contract.go:167-238`; Go golden bounds/count-presence checks: `internal/contract/golden_test.go:196-237,414-450`; TypeScript/schema currently check type/range and exhaustion description, not formula: `.agents/reports/interface-contracts/typescript/types.ts:353-373`, `schemas/event-stream.schema.json:94-169`, `tests/contract-conformance.test.ts:232-270`.
- Existing source ports and summary/list shapes: `apps/godspeed-casework-go/internal/ports/ports.go:276-296,442-459`; safe trace port: `internal/ports/run_trace.go:5-20`. Scoped list validates all returned rows before exposing them: `internal/adapters/sfwp/authority.go:254-315`. Kernel scopes ownership before collecting unreadables: `crates/sea-forge-server/src/sfwp/run_views.rs:777-833`. Safe trace projection validates exact identities and retains at most 1,024 frames: `internal/adapters/sfwp/run_trace.go:67-177`.
- Current parent check in existing live projection: `apps/godspeed-casework-go/internal/projection/live.go:64-123`. Revision stores immutable captured facts: `internal/projection/store.go:41-53`; per-case history and cold/evicted caveat: `:122-139`; relay capture/store and gap behavior: `internal/server/relay.go:101-150`; case cursor source: `internal/server/relay.go:170-175` (`CursorForCase`).
- Present route context: `/api/world` live vs historical paths `apps/godspeed-casework-go/internal/server/server.go:177-230`; `/api/events` reads only resume cursor and subscribes globally `:293-318`; existing session identity and bearer shape `internal/server/session.go:64-70,85-141`; `SessionStore.Current` detached deadline/revocation semantics `internal/auth/session.go:53-59,146-169`; perspective verifier at `internal/server/server.go:385-400` and delegated identity check in `internal/projection/live.go:317-339`.
- Accepted prerequisite boundaries: safe trace source approval and runtime classifications `safe-trace-runtime-classification-independent-review-oct05.md`; bounded session primitive source/runtime approval `observation-session-lifecycle-production-independent-review-oct05.md` and `observation-session-lifecycle-production-runtime-review-oct05.md`; physical admission root adjudication and independent production source review `run-trace-physical-admission-root-adjudication-oct06.md`, `physical-admission-production-independent-rereview-oct06.md`; selector source/fixture approval `run-observation-selection-final-independent-source-review-oct06.md`, `run-observation-selection-phase2-independent-production-source-review-oct06.md`.
