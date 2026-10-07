# Private run observation manager — concrete proposal revision 2

Date: 2026-10-06  
Author: independent Luna proposal builder; no implementation or runtime work performed.

## Decision boundary and inputs

This is one revised private architecture proposal. It addresses the four mandatory findings in `run-observation-manager-concrete-proposal-independent-review-oct06.md` (the absent present-case guard, aggregate lease/queue bound, unresolved exhaustion rule, and bearer policy). It preserves the earlier proposal's accepted bounds and count meanings. It does not approve a route, public API, wire schema, cursor change, identity expansion, implementation, or runtime claim. Root retains architecture acceptance.

The governing source references are the approved T09 extension and its Casework DTO/schema, plus current Go projection, relay, server, auth, and ports code. `internal/projection/store.go:43-53,122-139` defines `Revision` and returns retained cloned trajectory rows; `internal/projection/builder.go:36-51` defines optional `CaseFacts` and its cursor; `internal/ports/ports.go:268-291` defines the case horizon; `internal/server/relay.go:101-150,170-175` advances observed cursors and exposes `CursorForCase`; `internal/server/server.go:57-82` already has history/cursor dependency shapes. `internal/server/session.go:45-79` defines auth options and request identity, while `internal/auth/session.go:147-181` defines the current-session check. Run list and trace DTOs are in `internal/ports/ports.go:293-306` and `internal/ports/run_trace.go:13-30`.

The existing private manager proposal's `(case_id,run_id)` sharing, 16-entry poller bound, 1-second per-run floor, selector/read limits, count model, 1,024-frame retention, initial 1 MiB JSON envelope limit, and no-public-wiring boundary remain. The independent review rejected the proposal only at the four lifecycle/semantics gaps recorded there; this revision changes those points and makes their enforcement boundaries concrete.

## Private dependencies and exact present-context guard

The manager is constructed with already existing dependencies; it performs no new hydration or kernel reads:

```go
type RevisionHistory interface {
    Trajectory(caseID string) []projection.Revision
}
type RelayCursors interface {
    CursorForCase(caseID string) (cursor string, ok bool)
}
type PresentContextGuard struct {
    history RevisionHistory
    relay   RelayCursors
}
type GuardedCase struct {
    caseID     string
    cursor     string
    parentIDs  map[string]struct{}
}
func (g *PresentContextGuard) Check(caseID string) (GuardedCase, error)
```

`GuardedCase` is private and is created only by `Check`; it is not a public token and conveys an as-of-cursor observation, not authorization. `Check` reads the newest retained trajectory row for the exact case and rejects with a typed `ErrPresentContextUnavailable` if: there is no retained row (cold, evicted, or unknown); the row's `CaseID` or `Snapshot.CaseID` differs; `Facts` is nil; `Facts.Cursor` differs from `Revision.Cursor`; `Facts.Record.Ref`, `Facts.Overview.Ref`, or `Facts.Horizon.Ref` does not identify the requested case; `Horizon.Items` is nil (incomplete legacy view; a nonnil empty slice is a complete empty horizon); a parent item ID is blank or duplicated; or `Relay.CursorForCase` is absent or differs from the newest retained revision cursor. It copies the item IDs into a private owned set before returning. It never derives a parent set from request input.

`NewRunObservationManager(..., guard *PresentContextGuard, ...)` requires a nonnil guard backed by the server's existing `RevisionHistory` and `RelayCursors` dependencies. A future `Prepare` call requires a successful guard result; callers cannot populate freshness or parent IDs. `Prepare` repeats `Check` at entry before lease reservation and before any run list, trace read, or disclosure. The coordinator repeats it outside locks immediately before each poller read, before publishing a new shared buffer, and immediately before each watcher fanout callback. A buffer is eligible for reuse only if its `CaseID`, `RunID`, and `PlanItemID` still match the selected row and that `PlanItemID` remains in the newly checked parent set. Otherwise the selected candidate is unavailable; there is no replacement selection.

The guard compares two existing views but cannot make them atomic. Relay's observed cursor can advance after `Trajectory` is copied or after `Check` returns; a horizon can therefore become stale between a successful boundary check and a later network call/callback. Every boundary rechecks, so the guarantee is “latest retained present facts matched the observed relay cursor at this check,” not continuously current kernel truth or an atomic snapshot. A capture failure may advance the relay cursor without appending a revision; that mismatch is rejected as a gap. The manager must not call `LiveSource.Facts` to fill a missing row, read the kernel for the current horizon, or claim that this private check creates the held public V4 readiness frontier.

## Identities, limits, and owned state

The manager key is the exact pair `(caseID, runID)`, not a bare run ID. Each `pollerEntry` stores the immutable `caseID`, `runID`, `planItemID`, first verified observation time, latest owned snapshot, generation number, last logical call start, cancellation function, `ready` and `done` channels, watcher reference count, and terminal/error state. Its identity is checked against every `RunTraceSnapshot`; a reused or newly read snapshot with a case/run/plan mismatch is unavailable and is never published.

The process-shared poller map has at most 16 entries, counting `initializing`, `stopping`, and `draining`. There is no active eviction. A map slot is freed only after the accepted client call has returned through its cancellation and pool-retirement path and the poller worker has closed `done`. Manager code never duplicates the accepted run-get physical limiter, retries, or cooldown; those remain owned by the already accepted shared client hook. The poller may start no more often than once per second per key, measured from completion of the preceding client attempt for safety. `ReadRunTrace` cancellation is considered drained only after its actual caller returns, including the hook's cleanup; `SessionStore.Current` is not transport cleanup evidence.

This proposal adds a separate **proposed** aggregate bound of 16 cohort leases, including `preparing` and `draining` leases. Each lease can attach at most eight selected run entries, so at most 128 watcher-to-run attachments exist across all leases. This is distinct from (and does not replace or reinterpret) the approved 16-poller bound. It is a new admission policy requiring independent architecture review and operator approval before public integration. A full lease table returns the private typed `ErrObservationCapacity` before run.list or trace calls; it does not emit a synthetic observation with invented count values. It never evicts an active lease. The existing case-revision subscription remains a distinct stream and is not rejected by this observation-only bound.

```go
type ManagerOptions struct { Now func() time.Time }
type RunObservationManager struct { /* private mutex, maps, deps, process ctx */ }
type CohortRequest struct { CaseID string; Identity requestIdentity; RequestContext context.Context }
type CohortLease interface {
    Next(ctx context.Context) (ObservationDelta, error)
    DetachAndDrain(ctx context.Context) error
}
func NewRunObservationManager(
    parent context.Context,
    history RevisionHistory,
    relay RelayCursors,
    sessions SessionCurrent,
    perspective PerspectiveVerifier,
    runs ports.RunListPort,
    traces ports.RunTracePort,
    options ManagerOptions,
) (*RunObservationManager, error)
func (m *RunObservationManager) Prepare(ctx context.Context, req CohortRequest) (CohortLease, error)
func (m *RunObservationManager) StopAndDrain(ctx context.Context) error
```

`SessionCurrent` and `PerspectiveVerifier` are the existing server/auth behaviors behind these private seams; this proposal does not create new identity or authority. `ObservationDelta` is private and contains only validated owned run snapshots and the existing observation/count semantics. It is pulled by the future caller; the lease channel carries only a wake hint, never a frame or payload.

Each lease owns `wake chan struct{}` with capacity one. Producers send nonblocking wake hints after publishing a newer generation. A full channel means “a pull is already pending,” not loss of an unbounded payload queue. On wake, `Next` rechecks watcher authorization and present context, snapshots the latest bounded per-run buffers under the manager lock, releases the lock, computes the bounded-window delta, and invokes no external callback while holding the lock. It advances a lease's per-run generation watermark only after the corresponding pull has been successfully returned. If a slow consumer misses frames that have fallen out of the retained ring, the next result reports the exact window gap using retained/total frame counts; it makes no unbounded exactly-once claim. The later public DTO mapping for such a delta remains a separately held integration decision.

To make close/send safe, publishing a wake takes a notification reference under the manager mutex only while the lease is active, then releases the mutex before the nonblocking send and decrements the reference afterward. Detach marks the lease draining under that same mutex, preventing new notification references, then waits for outstanding notifier references outside the mutex before closing `wake`. No network call, `Next` wait, callback, cancellation join, or wait-group wait occurs while the manager mutex is held.

## Watcher authorization and bearer rule

Each lease holds one watcher-specific immutable authorization record, not a shared authorization decision. Cookie/session watchers retain only the session ID, detached claimed actor/role, request context, deadline, and revocation signal. Before the initial disclosure, each run list/read, every later poll, buffer publication, and each fanout/pull boundary, the manager checks that the request context is live; `SessionStore.Current(sessionID)` succeeds; actor/role still match the original claim; the stored deadline has not passed; the revocation signal remains open; and the current `PerspectiveVerifier` accepts that actor/role for the requested case. These checks occur outside the manager mutex. If any check fails, that watcher is detached and receives no later disclosure. Its departure does not cancel a poller while another authorized lease remains attached to that exact run.

The existing dev-only bearer path is explicitly allowed for this private behavior only when the existing validated server configuration has a static bearer token and configured bearer identity. The manager receives the already authenticated `requestIdentity`; it retains only the configured actor/role and request context, never the bearer token. For bearer watchers, the request context bounds lifetime and current kernel perspective is verified at each same boundary. There is no `SessionStore.Current` or bearer revocation guarantee. Existing configuration validation remains authoritative and continues rejecting static bearer configuration outside development mode. No public identity behavior is broadened.

## Cohort assembly and exact count semantics

`Prepare` proceeds in this order:

1. Check request context, authenticate using the existing middleware result, run `PresentContextGuard.Check`, and reserve one `preparing` cohort lease under the manager mutex. Capacity failure is typed and occurs before any list/read. Unlock before any further operation.
2. Recheck authorization and guard immediately before one scoped `RunListPort` call for the case. A refused, transport-failed, undecodable, or guard-stale list returns typed unavailable and no observation/count DTO; all count pointers remain absent.
3. Recheck authorization and the exact current parent set after list. On failure, return typed unavailable without exposing list rows or counts. For a complete result, `R = len(result.Runs)` readable rows and `U = len(result.UnreadableIDs)` scoped unreadable IDs. `U` is counted separately and is never treated as a readable candidate.
4. Call the accepted deterministic selector once over the complete decoded readable set. It yields `S = min(8,R)` candidates in the established order. Do not read rows outside those eight and do not replace any selected failure with a later row.
5. For each selected candidate in selection order, check authorization and guard outside the lock and verify parent membership. Exact current cached snapshot: attach/reuse with no logical read, preserving its original observation time. Same-key initializing snapshot: attach and wait for `ready` outside the lock; this cohort made no logical read. No current entry and map slot available: reserve one `initializing` entry, then call the accepted logical `ReadRunTrace` once outside locks. Only this cohort-started invocation increments its `reads_attempted`; hidden physical retries do not. If the map is full or the key is draining, classify that selected row as capacity-blocked. Returned data must match exact case/run/parent IDs and accepted standing/frame constraints before publication; malformed, denied, mismatched, or orphaned selected rows are unavailable. Never fan out an omitted candidate.
6. Before publishing each newly completed buffer and before returning the initial observation, recheck watcher authorization and the present-context guard. If either fails, suppress disclosure and detach the failed watcher. Other authorized watchers may still use a shared buffer whose identities and current parent ownership remain valid.

Count names and equations are unchanged:

| Quantity | Definition |
| --- | --- |
| `R` / `listed_run_count` | Number of readable summary rows in the completed scoped list. |
| `U` / `unreadable_run_count` | Number of scoped case-claimed `UnreadableIDs`. |
| `S` / `selected_run_count` | `min(8,R)`, selected by the accepted selector. |
| `V` / `validated_run_count` | Selected rows with exact case/run/plan-item ownership and a current known parent verified; includes valid cache reuse and shared initialization. |
| `A` / `unavailable_run_count` | Selected rows denied, malformed, mismatched, or no longer owned by a known parent. |
| `C` / capacity-blocked rows | Selected rows blocked by shared poller capacity or a draining same-key entry. |
| `O` / `omitted_run_count` | `(R-S)+C`; readable rows beyond selection plus selected rows blocked by capacity. |

For a completed decoded list, `V + A + C = S`; unreadable rows remain a separate `U` and are not included in that equation. Readable rows beyond `S` count in `O`, not `A`. A selected validation failure counts `A`, not `C`, and never shifts selection. Failed/refused/undecodable list means every count pointer is absent. After a complete list, all applicable counts—including zero—are present. If list or any scoped unreadable row or selected validation is unavailable, status is `unavailable`; otherwise `O>0` gives `capacity_limited`; otherwise `R=0 && U=0` gives `no_runs`; all remaining completed lists give `complete`. Only `V` owned validated rows contribute observations. Counts do not assert settlement.

### Exhaustion resolution

The executor uses `exhausted = (reads_attempted == 8 && R > 8)`. Both eight-row selection and eight-logical-read limits are binding at the same boundary. When both are reached with additional readable candidates, the manager reports exhausted, because the approved extension explicitly describes exhaustion while additional known candidates remain. This is the chosen executor interpretation of the wording ambiguity: the schema's causal “read limit prevented validation” language could be read more narrowly because candidates after the selector's first eight are also excluded by selection. The field is false with cached reuse, shared initialization, fewer than eight actual logical calls, or `R<=8`. This choice preserves count equations: extra rows remain omitted, not read failures. It is a root-directed interpretation for review, not a claim that the schema text is unambiguous.

## Bounded frames, bytes, and non-claims

Each poller keeps no more than 1,024 safe frames, exact source `TotalFrameCount`, and the observation time associated with the source snapshot. Retained count equals `len(frames)`; omitted is `TotalFrameCount-len(frames)`; truncated is exactly omitted greater than zero. A polling update includes only new frame IDs still present in this bounded ring. Pruning for the initial JSON payload removes globally oldest parsed timestamps first, then full RunID UTF-8 bytes, then EventID UTF-8 bytes, while retaining per-run order. It updates retained/omitted/truncated from the exact retained set; no frame is silently removed while counts imply it is present.

The existing response line cap does not provide a useful retained-heap bound for strings. A single 32 MiB source line can carry one very large ID; the accepted two-physical-run-get bound permits at least 64 MiB of raw line bytes in simultaneous trace responses, before decoded Go object overhead. A 16-lease proposal could also issue up to 16 scoped list calls; at a 32 MiB line cap, that is another theoretical 512 MiB of concurrent raw list response bytes, before decoding. Over time, 16 pollers ×1,024 retained frames =16,384 frames; if each retained metadata ID approaches 32 MiB, raw ID bytes alone could approach 512 GiB. Thus the approved frame count and source line cap are not a practical heap bound, and this proposal makes no OOM-safety claim. No existing per-ID byte limit was found in the cited DTO contract.

For review, I propose an additional private retention budget of at most 1 MiB of canonical JSON bytes per poller, 16 MiB across 16 pollers. Charge exact serialized safe frame bytes plus each run's identity/standing metadata; retain no more than both 1,024 frames and 1 MiB. When a new frame makes the budget exceed either limit, evict globally oldest retained frames using the same timestamp/full-ID order, preserving each run's frame order, and recompute truthful totals/retained/omitted/truncated fields. If a single required run identity/standing record or single frame cannot fit by itself, fail the selected run closed as unavailable; do not truncate an ID or emit a partial identity. The initial whole-envelope 1 MiB JSON limit remains separately enforced; metadata alone exceeding it yields a typed unavailable result and no oversized payload. This is an exact policy proposal, not approved behavior or code. Independent review and operator approval are required before it can alter accepted DTO accounting. It bounds retained serialized bytes, not Go heap/RSS, decode buffers, the 32 MiB line in flight, list fan-out, or allocator overhead. A later integration must separately consider input/body concurrency bounds before claiming a whole-process memory ceiling.

## Lifecycle, locking, cancellation, and drain

Poller states are `initializing -> running -> stopping -> draining -> removed`; failure or terminal standing enters `stopping`. Lease states are `preparing -> active -> draining -> removed`. Both maps count every state except removed. Manager mutex protects maps, state changes, generation/watcher counts, notification references, and the closed bit only. All `RunList`, `ReadRunTrace`, perspective checks, guard checks, `Next` waits, wake sends, cancellations, callbacks, and joins happen outside the mutex.

For same-key initialization, the first caller reserves the entry and its `ready`/`done` channels under lock, then performs the read outside the lock. Concurrent callers attach to that exact entry and await `ready` outside the lock; they do not issue another read. A failure closes `ready` with an error and all attached cohorts classify their selected row as unavailable. A successful read is published only after exact identity, auth, current parent and buffer-bound checks. Poller work uses the manager process context, not the first request's context, so one watcher's cancellation cannot cancel work needed by another; the last authorized attachment causes cancel.

`DetachAndDrain` atomically marks the lease draining, prevents new wake references, removes it from the active-lease set only after its existing operations have been accounted for, decrements run attachments, and marks any now-unneeded poller stopping. It then unlocks, cancels eligible pollers, waits for lease notification/fanout operations and poller `done` channels, and frees slots only after the accepted transport call has returned. A context timeout returns an error and leaves the manager closed/draining entries counted; it never pretends a slot was freed. Callers must not invoke detach synchronously from inside a callback that detach itself must join.

`StopAndDrain` marks the manager closed and all leases/entries draining under lock, preventing new reservations. It cancels outside the lock, waits for any in-flight preparation/list operation, notification/fanout operation, and each poller `done`, then removes entries. If its context expires, it returns the context error while retaining truthful draining state. A later call may continue joining. `StopAndDrain` never waits while holding the manager mutex. On shutdown, no new cohort is accepted. Last watcher departure, terminal run, all watchers losing authorization, and process shutdown all follow the same cancel-then-join rule.

## Lifecycle and outcome table

| Boundary | Success | Failure / result | Locks and disclosure rule |
| --- | --- | --- | --- |
| Prepare identity + guard | Reserve a `preparing` lease if under 16. | Context/auth/guard failure: typed unavailable. Sixteen leases: `ErrObservationCapacity`. | Guard/auth outside lock; reserve only under lock. No list/read/count payload on failure. |
| Scoped list | One complete decoded case list. | Refused/transport/decode/stale-context failure: typed unavailable. | Call outside lock; count pointers absent. |
| Selection | First `min(8,R)` deterministic rows. | No replacement. | Complete readable set only; unreadable IDs stay separate. |
| Selected row | Valid read, current cache, or shared initialization. | `A` for denied/malformed/mismatch/orphan; `C` for map capacity/draining. | Recheck auth/guard before every read/reuse and before publish. |
| Initial envelope | Accurate bounded validated observations. | Metadata-only over 1 MiB: typed unavailable, no oversized payload. | Final authorization + guard outside lock. |
| Poll/pull | Current bounded ring delta. | Stale guard/auth: suppress and detach; slow consumer may have exact window gap. | Coalesced capacity-1 hint; no payload queue. |
| Detach/shutdown | All needed calls joined before slots freed. | Timeout returns error with entries still counted draining. | Cancel/join outside lock; no false cleanup success. |

## Test-first implementation partition and prerequisites

Implementation is not authorized by this proposal. If root separately releases it, add focused tests before implementation in these disjoint units:

1. `present_context_guard_test.go`: fake history/relay tests for absent history, evicted/cold, nil facts, case/snapshot/facts/horizon identity mismatch, facts cursor mismatch, nil versus nonnil-empty horizon, blank/duplicate parents, missing relay cursor, relay gap, and success with copied parent IDs. Each failure asserts list, trace, and sink are untouched.
2. `run_observation_cohort_test.go`: list refused/undecodable versus successful empty; `R/U/S/V/A/C/O` equations and status precedence; selector order; no replacement; exact identity/parent validation; current cache observation-time preservation; same-key initializer sharing; logical call counting; exhaustion at exactly 8 with `R>8`, with `R=8`, and with reuse reducing actual calls below 8.
3. `run_observation_registry_test.go`: 16 entries across every state, no active eviction, 16 cohort leases including preparation/drain, typed capacity failure before list/read, max eight attachments per lease/128 total, exact key includes case, concurrent singleflight, draining-key capacity outcome, one-second completion floor, and no manager-level physical semaphore/retry.
4. `run_observation_authorization_test.go`: session `Current`, exact actor/role, deadline, revocation, perspective changes before initial disclosure, every read, publication, and pull; dev bearer allowed only under validated config, request-context-only lifetime, perspective recheck, no token retention, and no fabricated bearer revocation claim. One watcher leaving must not cancel another authorized attachment.
5. `run_observation_lifecycle_test.go`: blocked reader cancellation then actual hook cleanup/join before map reuse, terminal read, last watcher, all-auth-fail, shutdown, timeout leaving slots counted, notifier send/close race, no callback under mutex, lease preparation versus stop, and slow-reader coalescing/window-gap accounting.
6. `run_observation_memory_test.go`: both 1,024-frame and proposed 1 MiB-per-poller byte caps, exact byte boundary, one oversized frame/identity typed failure, cross-run oldest pruning and full-ID ties, per-run order, exact total/retained/omitted/truncated, and initial whole-envelope cap. This sixth unit is blocked on independent and operator approval of the new private byte policy; do not change the DTO silently.

For each unit, show actual focused RED against the positive assertion before implementation, then the narrow package race test. Independent source critic and compiler owner must be separate from the builder. Serial broader gates, any local-listener elevation, evidence capture, and eventual public integration require their own explicit root assignments. Nothing in this proposal establishes test results.

## Material changes and remaining review

Compared with the original proposal, this revision supplies the enforcement dependency and typed result the first review required; it chooses root's `reads_attempted==8 && R>8` executor interpretation while preserving and documenting the causal wording ambiguity; it explicitly allows only the existing dev bearer identity/request-context path with repeated perspective checks; and it proposes a distinct 16-cohort lease admission cap with coalesced capacity-one wake hints and pull-based bounded deltas. It also quantifies why 16 ×1,024 frames under a 32 MiB line cap is not a heap bound and proposes a separate exact 1 MiB-per-poller serialized-retention policy for approval. It does not change accepted count meanings, frame count, initial envelope limit, physical admission ownership, or held public wiring.

Open before implementation/public wiring: independent architecture and operator approval of the new 16-lease aggregate cap; approval of the separate byte policy and its unavailable accounting; confirmation that request cancellation and existing client cleanup truly join before poller completion; existing middleware must preserve authenticated `requestIdentity` through the future private call site; and root's separate release of a future caller/integration task. The present-context result is an as-of check with unavoidable between-check races. No public cursor/readiness guarantee, exact-once event stream, bounded whole-process heap, settlement, or future-callsite approval is claimed.
