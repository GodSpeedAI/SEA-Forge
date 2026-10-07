# Private run observation manager — concrete proposal revision 3

Date: 2026-10-06  
Status: complete DOCONLY repair for independent architecture review; implementation remains held.

## Decision boundary and source basis

This is a complete private producer/consumer and lifecycle proposal. It incorporates revision 2's present-context guard, explicit exhaustion interpretation, development-only bearer rule, proposed aggregate cohort cap, bounded pull hints, accepted count semantics, and proposed retention-byte budget. Revision 3 addresses the independent review's three blocking findings: failed-`Prepare` rollback, exact initial-result versus subsequent-delta contract, and an explicit private attach operation with typed outcomes and ownership.

No public HTTP/SSE mapping, cursor/frontier change, kernel read, identity expansion, route, runtime behavior, or implementation is authorized. Root retains architecture and integration decisions. The proposed additional 16-cohort bound and 1 MiB-per-poller retained-byte policy are **proposals only**, each requiring independent review and operator approval before any implementation relies on them.

Actual source anchors: retained `Revision`/`Trajectory` behavior and optional facts are `apps/godspeed-casework-go/internal/projection/store.go:41-53,122-139`; `CaseFacts.Cursor` and `Horizon` are `internal/projection/builder.go:34-51`; `CaseHorizon.Items` is `internal/ports/ports.go:266-274`; relay cursor observation/lookup is `internal/server/relay.go:101-150,170-175`; `Server` already depends on history and relay cursor interfaces at `internal/server/server.go:56-70`; `PerspectiveVerifier.VerifyPerspective(ctx, ports.ActorClaim) error` is `internal/server/server.go:38-42`; `requestIdentity` has `Identity`, `Session`, and `Source` fields at `internal/server/session.go:64-70`; the development-only `StaticToken` and resolved `BearerIdentity` configuration are at `:45-62`; `SessionStore.Current(id) (CurrentSessionState, bool)` is `internal/auth/session.go:147-169`; safe trace identity/frame DTOs are `internal/ports/run_trace.go:5-30`; scoped list results and horizon types are `internal/ports/ports.go:266-306`.

The normative private-source contract remains the approved T09 extension. Existing `(case_id,run_id)` sharing, 16 pollers including draining, at most two physical run-get attempts, accepted post-write cooldown, no faster than one logical poll start per second per run, deterministic at-most-eight selection and eight initial logical reads, count meanings, 1,024 retained frames, and initial JSON payload cap of 1 MiB are retained. The manager does not duplicate or bypass the shared physical-admission owner.

## Private inputs, present-context guard, and authority

The manager does not hydrate a missing case view. It uses the current relay history and observed cursor only:

```go
type RevisionHistory interface {
    Trajectory(caseID string) []projection.Revision
}
type RelayCursors interface {
    CursorForCase(caseID string) (cursor string, ok bool)
}
type GuardedCase struct {
    caseID    string
    cursor    string
    parentIDs map[string]struct{}
}
type PresentContextGuard struct { history RevisionHistory; relay RelayCursors }
func (g *PresentContextGuard) Check(caseID string) (GuardedCase, error)
```

`Check` requires a newest retained row for the exact case. It rejects cold/evicted history; disagreement among row, snapshot, facts, record/overview/horizon references; absent facts; mismatch between `Facts.Cursor` and revision cursor; nil horizon items (nonnil empty means a complete empty horizon); blank or duplicate item IDs; absent relay cursor; or relay cursor unequal to the newest retained revision cursor. It copies item IDs into an owned private set. A caller cannot supply parent IDs or a freshness assertion. The private result is as-of-cursor knowledge: history and relay are not atomically read, and the horizon may change after each successful check. Recheck before every sensitive boundary; do not call `LiveSource.Facts` or claim continuous kernel truth or the held public V4 readiness frontier.

Use the exact existing private auth seams:

```go
type SessionCurrent interface {
    Current(sessionID string) (auth.CurrentSessionState, bool)
}
type PerspectiveVerifier interface {
    VerifyPerspective(ctx context.Context, actor ports.ActorClaim) error
}
```

`requestIdentity` is the server's existing authenticated `*requestIdentity` (`Identity auth.Identity`, `Session *auth.Session`, `Source "session"|"bearer"`), passed from middleware, never reconstructed from a request DTO. A session watcher retains session ID, detached original actor/role claim, request context, `ValidUntil`, and revocation state. At initial disclosure, each list/read, poll authorization boundary, buffer publication, and every pull/fanout boundary, check context, `Current(sessionID)`, exact claim equality, deadline, revoked state and `VerifyPerspective`. A departure detaches that watcher only.

For bearer mode, allow only the existing validated dev-only static-token request identity with configured `BearerIdentity`; retain actor/role and request context, never token material. Recheck kernel perspective at each boundary. Request context is the only bearer lifetime signal: do not claim `SessionStore.Current` or bearer revocation. Existing config validation continues to reject static bearer auth outside development. This does not broaden identity policy.

## Exact private values, outputs, and interfaces

All types below are private implementation proposals; the initial result is a private hydration DTO, not the public SSE/HTTP DTO. It carries only selected, identity-validated safe snapshots and counts:

```go
type CohortRequest struct {
    CaseID string
    Identity *requestIdentity
    RequestContext context.Context
}
type InitialRun struct {
    RunID, PlanItemID, Execution, Settlement string
    ObservedAt time.Time
    Frames []ports.RunTraceFrame // safe, bounded initial frame window
    TotalFrameCount, RetainedFrameCount, OmittedFrameCount int
    Truncated bool
}
type InitialCohort struct {
    CaseID string
    AsOfCursor string
    ObservedAt time.Time
    Runs []InitialRun
    ListedRunCount, UnreadableRunCount, SelectedRunCount int
    ValidatedRunCount, UnavailableRunCount, OmittedRunCount int
    Status ObservationStatus
    Exhausted bool
}
type ObservationDelta struct {
    CaseID string
    AsOfCursor string
    ObservedAt time.Time
    Runs []RunDelta // only newly observed safe frames/standing changes
    WindowGaps []RunWindowGap // exact loss from bounded frame window, if any
}
type CohortLease interface {
    Next(ctx context.Context) (ObservationDelta, error)
    DetachAndDrain(ctx context.Context) error
}
func (m *RunObservationManager) Prepare(
    ctx context.Context, req CohortRequest,
) (InitialCohort, CohortLease, error)
func (m *RunObservationManager) StopAndDrain(ctx context.Context) error
```

`Prepare` returns the initial result **directly and exactly once**. A successful return has a nonnil lease. A whole-prepare failure returns zero `InitialCohort`, nil lease, and one typed error; no partial initial DTO or counts escape. Selected-run unavailability is different: the prepare succeeds with an initial result whose `UnavailableRunCount`/status reflect those candidates and a lease for the other validated runs. After that return, every successful `lease.Next` is a delta only; it never repeats the initial cohort or initial counts. Delta includes only frame IDs/standing changes newer than its watermark while retained, plus an exact window-gap record if a bounded window has moved past unseen frames. Public mapping remains held.

Typed whole-prepare failures are: `ErrObservationCapacity`, `ErrPrepareCanceled`, `ErrPrepareStopped`, `ErrPresentContextUnavailable`, `ErrAuthorizationUnavailable`, `ErrRunListUnavailable`, `ErrInitialEnvelopeTooLarge`, and `ErrInitialAssemblyUnavailable`. They disclose no initial DTO/count pointers. `ErrSelectedRunUnavailable` is an internal candidate disposition, not a whole-prepare error; it increments `A` and assembly continues. Capacity/draining for a selected row is a counted disposition `C`. Only malformed/mismatched/denied/orphaned or failed selected trace candidates are `A`; list/guard/authorization failures that invalidate the whole initial cohort fail `Prepare`.

### Poller and attach types

```go
type pollerKey struct { CaseID, RunID string }
type pollerState uint8 // initializing, running, stopping, draining, removed
type leaseState uint8  // preparing, active, draining, removed
type attachDisposition uint8
const (
    attachCurrent attachDisposition = iota // exact verified current cache
    attachSharedInitializer                // exact key initialization already reserved
    attachNewInitializer                   // this call reserved initialization
    attachCapacityBlocked                  // 16-entry poller table full
    attachDraining                         // exact key is being removed
    attachIdentityMismatch                 // same key metadata conflicts
    attachInvalid                          // selected identity/parent invalid
)
type runRef struct { key pollerKey; entry *pollerEntry; ownerLease uint64 }
type initializerToken struct { key pollerKey; generation uint64; done <-chan struct{} }
type attachResult struct {
    disposition attachDisposition
    ref *runRef // owned by the provisional lease on success
    initializer *initializerToken // only for attachNewInitializer
    ready <-chan struct{} // current/shared entry readiness
}
func (m *RunObservationManager) attach(
    ctx context.Context, lease *cohortLease, row ports.RunSummary, guard GuardedCase,
) (attachResult, error)
```

`attach` is called only after the caller has performed fresh watcher authorization and `guard.Check` outside the mutex. It takes no network or callback capability. Under the mutex it verifies manager/lease state and guard-bound parent membership, then atomically resolves the exact `(CaseID,RunID)` key:

| Disposition | Atomic action | Caller action outside lock |
|---|---|---|
| `attachCurrent` | Verify cached case/run/plan-item identity and current parent; add one lease-owned watcher ref. | Recheck guard/auth; snapshot exact current buffer and preserve its original `ObservedAt`. No logical read. |
| `attachSharedInitializer` | Verify exact selected identity; add this lease's watcher ref to existing `initializing` entry. | Wait on `ready` with request/manager cancellation outside lock. No logical read charged to this cohort. |
| `attachNewInitializer` | Reserve one map slot and `initializing` entry with `ready/done`, owner generation and first watcher ref. | Recheck guard/auth, perform one logical `ReadRunTrace` outside lock, validate and publish; token permits only that initializer generation to complete. |
| `attachCapacityBlocked` | No reference acquired. | Count selected row as `C`; no list/read retry and no replacement selection. |
| `attachDraining` | No reference acquired. | Count selected row as `C`; no new same-key initializer until removal completes. |
| `attachIdentityMismatch` / `attachInvalid` | No reference acquired. | Count selected row as `A`; never expose it. |

The same key with a conflicting plan item is never joined or overwritten. A current or shared buffer is revalidated for exact `CaseID`, `RunID`, and `PlanItemID` and membership in the freshly guarded parent set before reuse/disclosure. The initial `ReadRunTrace(ctx, caseID, runID, planItemID)` uses the existing port (`internal/ports/run_trace.go:5-8`); any returned identity mismatch is selected unavailable, never cached as valid.

## Count equations and initial outcome semantics

Use the existing deterministic selector once over the complete decoded readable result. Let `R = len(Runs)`, `U = len(UnreadableIDs)`, `S = min(8,R)`, and for the selected rows `V` validated/reused/shared-initialized, `A` selected unavailable, `C` selected blocked by poller capacity/draining, and `O = (R-S)+C`. Then `V+A+C=S`; unreadable rows are separately counted in `U`. No unreadable identifier is silently promoted into a readable candidate. Read only selected rows; never backfill after an `A` or `C`.

If list is refused, transport/decode fails, guard is stale, auth fails, manager stops, or request context ends before handoff, fail the whole `Prepare` with zero initial DTO/counts. Once a complete list is decoded, emit all applicable count fields including zero. Status precedence remains: list/unreadable/selected validation failure => `unavailable`; else `O>0` => `capacity_limited`; else `R=0 && U=0` => `no_runs`; else `complete`. Only `V` validated rows enter `InitialCohort.Runs`. Counts never infer settlement.

Use the root-directed exhaustion interpretation `Exhausted = (logical_initial_reads_started == 8 && R > 8)`. Cached buffer reuse and shared initialization are not logical reads. Fewer than eight actual logical calls never exhaust. This treats the eight-selector/eight-read boundary as jointly binding even when remaining readable rows were also outside selection; the schema causal phrase “read limit prevented validation” remains ambiguous. This is an executor interpretation, retained for independent/root resolution before fixture implementation, not a claim the schema is unambiguous.

The initial result's JSON encoding (not SSE framing) is at most 1 MiB. Build and measure the entire private initial DTO before handoff. If metadata alone or any required initial payload makes it exceed that bound, return `ErrInitialEnvelopeTooLarge` and no payload/counts; run refs acquired in preparation are rolled back. Do not truncate IDs or invent counts to fit.

## Exact initialization, watermark, and delta contract

The lease owns a capacity-one wake-hint channel, never queued payloads. Pollers own bounded per-run snapshots and generation numbers; leases own a map of per-run watermarks.

1. During `Prepare`, each successful attached row contributes an exact initial snapshot `g` and its safe retained frame set. A fresh read's successful buffer publication records `g`; cache/shared initializer reuse retains the existing first `ObservedAt`. Under the mutex, atomically copy the initial rows and record the corresponding per-run generation and frame-ID watermark into the still-`preparing` lease.
2. Build the entire bounded `InitialCohort` outside the mutex. Recheck auth and `PresentContextGuard` before each disclosure boundary and before final handoff. If any whole-cohort check, context, stop, or encoding check fails, rollback; do not return a partial initial object or lease.
3. At final handoff, take the mutex and linearize only if manager is open, request context alive, lease still `preparing`, and all current watcher checks already passed immediately before lock acquisition. Mark lease `active` and transfer caller ownership. This lock transition is the success point. Return the already-built initial value and lease. A concurrent stop after that point may drain the just-returned lease; its next pull then returns typed stopped/revoked error.
4. Later buffer publication increments a poller generation and sends a nonblocking hint to active leases with a notification reference. `Next(ctx)` waits outside the lock for a hint, caller cancellation, lease stop, or manager shutdown. On wake it rechecks watcher auth and present context outside lock, then snapshots current per-run buffers/generations under lock. It computes only deltas since the lease watermark and updates watermarks atomically only after delta construction succeeds and the lease remains active. Thus a failed/canceled `Next` does not consume frames.
5. If ring eviction removed frames after the watermark, `Next` reports a `RunWindowGap` with exact total/retained/omitted counts and only the currently retained newer frame IDs. It makes no unbounded exactly-once promise. A capacity-one hint may coalesce many publications; the buffers and generations, not hint count, carry truth.

No network wait, session/perspective check, guard check, channel wait/send, callback, cancellation, or join runs under the manager mutex. A producer acquires a notifier reference under lock only for an active lease, unlocks to send the coalesced hint, then releases the reference. Detach marks the lease draining before further notifier refs can be acquired, waits outside lock for existing notifier refs, then closes its channel. No direct callbacks exist in this private design.

## Prepare state machine and mandatory rollback ownership

The manager has at most 16 process-shared poller entries, counting `initializing`, `stopping`, and `draining`; no active eviction. It also proposes a separate maximum of 16 cohort leases, including `preparing` and `draining`, each with at most eight selected refs (at most 128 attachments). This aggregate cap is an **unapproved proposal** distinct from the 16-poller policy. At capacity, fail before list/read with `ErrObservationCapacity` and no fabricated counts. Polling remains at most once per second per key. Shared physical two-attempt admission, retry and post-write cooldown remain owned solely by the accepted shared client hook.

Lease states: `preparing -> active -> draining -> removed`. Poller states: `initializing -> running -> stopping -> draining -> removed`; terminal state, initializer/read failure, or no remaining watchers transitions through stop/drain. Every reservation/ref has an explicit owner lease ID. Every `Prepare` installs one deferred single rollback path immediately after reserving its provisional lease. A `Prepare` that returns an error never transfers a lease; therefore only the manager can and must clean its provisional state.

**Rollback for every whole-prepare failure** (guard/auth/list/trace-infrastructure failure, invalidated context, initial encoding failure, stop race, or initial result handoff failure):

1. Under the mutex, transition the provisional lease `preparing -> draining`, set rollback cause, and forbid any further attachment or wake reference. Make the operation idempotent so `Prepare` and `StopAndDrain` can race safely.
2. Under that same lock, detach every `runRef` acquired by this lease, decrement its attachment count exactly once, and remove the lease from the set of leases allowed to receive notifications. Do not detach refs owned by another lease.
3. For each referenced poller, cancel only if after this decrement it has no other authorized watcher/ref and no manager-owned reason to finish. Never cancel a poller still needed by another watcher, including one sharing an initializer. Mark any canceled entry `stopping`/`draining` and keep its slot counted.
4. Release the mutex. Join this prepare's in-flight list call and any initializer/read it owns or caused to become watcherless; wait for its notifier refs and lease operations to finish. A shared initializer still needed by another watcher is not canceled or joined as this lease's work; this lease waits only if necessary to decide its own initial result, and cancellation detaches it without affecting the other watcher.
5. Only after owned work and notifier refs have completed may rollback mark the lease `removed`. Remove a poller map entry and free its slot only after its actual source call returns through the existing client cancellation/pool-retirement cleanup and poller worker closes `done`. A drain timeout returns an error and leaves the relevant lease/poller counted in `draining`; it never fabricates cleanup.

Selected-run unavailable (`A`) is not a whole-prepare failure: release just that candidate's ref if one was acquired, stop/cancel its initializer only if no other watcher needs it, join an exclusively owned canceled read before freeing its slot, then continue the initial result. Capacity/draining (`C`) acquires no ref. If a later candidate fails whole-prepare after earlier candidates attached, the single rollback releases all earlier refs as above.

List/read operations are accounted as lease operations before leaving the lock; completion decrements them via a close-on-zero signal outside the external call. `StopAndDrain` first closes admissions under lock and marks every lease draining, then cancels entries outside lock and waits for preparation-operation, notifier and poller completion outside lock. A preparing caller detects stopped state before attach and final handoff, runs the same rollback path, and reports `ErrPrepareStopped`. If it already crossed the active handoff linearization, stop drains that active lease normally. A timeout leaves the manager closed and all unfinished entries counted; subsequent `StopAndDrain` may continue joining.

### Failure and race table

| Failure/race | Returned result | Owned cleanup |
|---|---|---|
| Context/auth/guard fails before reservation | typed error, no DTO/lease | no lease/ref exists |
| 16 cohort slots occupied | `ErrObservationCapacity`, no DTO/lease/list/read | no reservation to roll back |
| List refused, transport or decode failure after reservation | `ErrRunListUnavailable`, zero DTO/nil lease | provisional lease rollback; wait list op completion |
| Guard/auth becomes invalid after list or between candidate reads | typed whole-prepare error, zero DTO/nil lease | rollback all earlier refs; cancel only watcherless pollers; join owned work |
| Selected trace denied, malformed, mismatched, orphaned, or source error | initial DTO succeeds with that row in `A`/unavailable | release candidate ref; cancel/join only if no other watcher needs it |
| Selected poller table full or key draining | initial DTO succeeds with that row in `C`/capacity-limited | no ref acquired |
| Shared initializer fails | selected row becomes `A` for each waiting cohort; unrelated watchers unaffected | detach each failed cohort ref; initializer stops/joins under normal watcher count rules |
| Initial envelope >1 MiB | `ErrInitialEnvelopeTooLarge`, zero DTO/nil lease | rollback every acquired ref and join owned work |
| Request cancellation while waiting on shared initializer | `ErrPrepareCanceled`, zero DTO/nil lease | detach this watcher; do not cancel/join initializer still needed by others |
| `StopAndDrain` before final activation | `ErrPrepareStopped`, zero DTO/nil lease | same idempotent rollback; stop joins preparation operation |
| Stop after active handoff | `Prepare` succeeded; lease Next returns stopped after drain | normal `DetachAndDrain`/Stop join |
| Cancellation/stop during a new initializer | no successful initial DTO for canceled cohort | cancel only if watcher count reaches zero; wait actual client cleanup before slot removal |
| Partial attach then later whole failure | zero DTO/nil lease | detach every prior ref; preserve buffers for other watchers; join only owned/now-unneeded work |

`SessionStore.Current` only reports session lifetime and detached state; it does not prove a trace transport has drained. Poller completion is not signaled until the `ReadRunTrace` caller returns after the accepted client's actual cancellation and pool-retirement path.

## Shared poller, cache, and delivery lifecycle

For an exact key, first attachment reserves an `initializing` entry and ready/done channels under lock. It performs one logical `ReadRunTrace` outside lock; physical retries stay under the shared client owner and do not change logical counts. Concurrent attaches join `ready` and do not issue another read. The first observation timestamp is immutable for this buffer. Validate case/run/plan identity, known current parent, safe DTO, accepted execution/settlement fields and frame limit before publishing. Before buffer publication, repeat that watcher's auth and guard; if the initiating watcher left but another authorized watcher remains, validate the surviving watchers against current parent/auth and keep work alive for them. Never use the first request context as the shared poller's sole lifetime context.

On each later poll, check every relevant attached watcher before the read and before publishing a result; invalid watchers are detached independently. If none remain, cancel. A terminal run stops polling and ends further deltas after publishing its final validated state. Detaching one lease decrements only its refs. When last watcher leaves, mark stopping, cancel, join actual call and worker, then remove. No poller slot is freed on cancel request alone.

`Next` is a pull from current validated buffers, not fanout of an unbounded frame queue. A capacity-one channel carries wake hints only. If the caller is slow, coalesced hints do not erase buffer truth; bounded frame eviction yields explicit `RunWindowGap`. `Next` does no work after lease drain or failed auth/guard and returns typed error; it does not infer settlement or claim global ordering/exact-once beyond the retained source window.

## Frame, byte, and payload bounds

Preserve at most 1,024 safe frames per run, the accurate source `TotalFrameCount`, and `retained=len(frames)`, `omitted=total-retained`, `truncated=(omitted>0)`. Only frames currently in the ring can be in later deltas. The existing 1 MiB initial JSON cap applies to the complete `InitialCohort` encoding.

Frame and source line limits do not bound retained heap: two physical trace attempts can expose at least 64 MiB raw line bytes concurrently at 32 MiB per line; 16 cohorts could issue up to 16 scoped lists (theoretical 512 MiB raw list lines); 16 pollers ×1,024 retained frames is 16,384 frames, and IDs approaching a 32 MiB line can dominate storage. These are upper-bound illustrations, not measured concurrency or memory. No OOM or process-RSS guarantee follows.

Revision 2 proposed, for review only, at most 1 MiB canonical JSON bytes retained per poller (16 MiB for 16 entries), charging safe serialized frames and identity/standing metadata, while also enforcing the 1,024-frame cap. On overflow, drop globally oldest parsed timestamp, then full RunID bytes, then EventID bytes, preserving each run's relative order and recalculating truthful counts. If a required identity or single frame cannot fit, fail that selected run unavailable; never truncate identity or silently omit mandatory metadata. Initial envelope metadata alone over 1 MiB is a typed whole-prepare failure. This byte policy is not implemented, approved, or assumed. It bounds serialized retained bytes only, not Go heap, decoded/in-flight payloads, list fan-out, or allocator overhead. It needs independent architecture and operator approval separate from the proposed 16-lease cap.

## Deterministic test-first matrix for later separately authorized implementation

No test or source change is authorized now. If root later releases implementation, each failure must have a deterministic test before its fix, no sleep/retry-luck criteria, and a focused RED observed by the assigned compiler owner. The 1 MiB retention-budget cases remain blocked until approval of that policy.

1. **Guard:** missing/cold/evicted history, nil facts, mismatched revision/snapshot/facts/record/overview/horizon refs, cursor mismatch/gap, nil versus empty items, blank/duplicate parents, and success with copied IDs; every rejection proves no list/read/disclosure.
2. **Exact initial/delta sequence:** `Prepare` returns one bounded initial result plus lease; initial includes approved hydration fields and all count values (including zeros); first `Next` after no new work waits; after a new frame/standing update it returns only that delta, not initial DTO/counts; canceled/failed `Next` leaves watermark unchanged; success advances watermark once; coalesced hints yield current buffer state; evicted unseen frames produce exact window-gap counts.
3. **Counts and typed outcomes:** list refuse/transport/decode/guard failure yields typed whole-prepare error, zero result/nil lease and absent counts; successful decoded empty gives present zero counts/no-runs; selected validation failures are `A` and do not stop unrelated candidates; capacity/draining are `C`; no backfill; `V+A+C=S`; `O=(R-S)+C`; status precedence; unreadable IDs separate; exhaustion tests 8 reads with `R>8`, 8 with `R=8`, cached/shared reuse, and fewer than 8 actual reads.
4. **Attach table:** exact current cache with original observation time; shared initializer; unique new initializer; 16-slot capacity; draining key; same key with conflicting plan item; invalid/missing-parent identity; each disposition asserts exact ref ownership and whether a logical read occurs.
5. **Every rollback boundary:** fail list after reservation; guard/auth after list; failure before first attach; after first successful attach and later selected hard error; partial candidate attach then initial envelope failure; context canceled waiting on shared initializer; initializer error with multiple waiters; a failed watcher detaching without canceling another watcher; last-watcher cancel/join; ensure lease/poller slots remain counted until owned operations, notifications, client cleanup, and worker completion actually join.
6. **Prepare/Stop races:** Stop before reservation, while list is in flight, while waiting initializer, during final build, immediately before handoff, and immediately after active handoff. Exactly one rollback/removal, no returned partial DTO, no Add/Wait race, no freed slot before join; a timeout leaves truthful draining state.
7. **Auth/bearer:** session `Current` failure, changed claim/role, deadline/revoked, perspective loss before each list/read/publish/initial return/delta pull; dev bearer only with validated config, request-context lifetime, repeated perspective check, no token retention or invented revocation guarantee.
8. **Poller lifecycle/concurrency:** exact `(case,run)` key; same-key single flight; other watcher survives initializer-owner cancellation; selected cache becomes orphaned on changed horizon and is not reused; terminal, last watcher, all unauthorized, and shutdown stop/join; one-second floor; no manager duplicate physical limiter; no external call/check/wait/callback/join under lock; capacity-one notifications have safe close/send refs.
9. **Bounds:** 1,024-frame counts and 1 MiB initial envelope exact boundary; if separately approved, 1 MiB-per-poller serialized retention boundary, single oversized frame/identity failure, cross-run oldest/tie ordering, per-run order and exact totals. No test may imply a whole-process heap bound.
10. **Independent proof:** source reviewer is not builder; root separately assigns compiler owner, the actual focused RED, focused race test, then serial broader gates. No test result or source acceptance is claimed by this proposal.

## Material deviations, findings addressed, and remaining prerequisites

Revision 2's three omissions are repaired concretely: every failed `Prepare` owns one idempotent rollback even though it returns no lease; `Prepare(ctx,req)` returns `(InitialCohort,CohortLease,error)` with typed whole-prepare failures versus counted selected-row unavailability; and `attach(ctx,lease,row,guard)` returns a typed disposition/ref/initializer token covering cache, shared/new initialization, capacity, draining, identity mismatch, and invalid input. Initial hydration/counts return once from `Prepare`; subsequent `Next` calls return deltas only, with watermark initialization and commit rules explicit. Context, stop, initial encoding, shared initializer, partial attach and notification/ref cleanup races are specified.

Previously accepted limits and semantics are preserved: 16 poller entries and physical admission ownership, maximum eight selected rows and initial logical reads, one-second per-run poll floor, count equations/precedence, 1,024 frames, 1 MiB initial JSON cap, identity/perspective guard, as-of cursor disclaimer, no callbacks or joins under lock, no settlement inference, and no public mapping/frontier change. Proposed additional 16-cohort admission and 1 MiB-per-poller retention budgets are still unapproved proposals, not behavior. The revision 2 exhaustion boundary `reads_attempted == 8 && R > 8` remains explicitly flagged for root/critic resolution due to causal wording ambiguity.

Remaining prerequisites before any source work: fresh independent architecture review of this complete blueprint; root decision on the exhaustion ambiguity; independent review and operator approval of the distinct 16-cohort cap; independent review and operator approval of the 1 MiB-per-poller retained-byte policy; source-level confirmation of actual transport cancellation/pool-retirement completion before slot removal; a separately released private caller/integration assignment; and, later, separately approved public mapping/frontier work. No source/tests/status/debt, public wire, or implementation were changed by this document. No test, compiler, scanner, gate, Graft build, Git, or network command was run.
