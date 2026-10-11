# Independent review: private run-observation manager proposal revision 2

Date: 2026-10-06  
Reviewed artifact: `run-observation-manager-concrete-proposal-revision2-oct06.md`, SHA-256 `00a2fb58ff9cc3c6e8c6df116b425af822e2a94b22167192edbb0aeff7bc3761`.  
Disposition: **REJECT as a complete private release blueprint; preserve public/integration HOLD.** This is a documentation review only, not implementation, runtime, or operator approval.

## Findings resolved from revision 1

The proposal materially repairs each prior rejection item:

1. **Present-context guard is now concrete.** `PresentContextGuard.Check` owns history/relay dependencies, derives parent IDs from latest retained `Facts.Horizon`, verifies case/facts/horizon/cursor identity, rejects nil history/facts and cursor gaps, and produces private `GuardedCase`. It repeats at the described read/publication/pull boundaries. This matches the actual `Store.Trajectory` cold/evicted behavior (`internal/projection/store.go:122-139`), optional `Revision.Facts` (`:41-53`), and relay's independent observed cursor updates before capture/append (`internal/server/relay.go:101-150,170-175`). The proposal correctly disclaims atomicity and continuous kernel truth.
2. **Exhaustion has an explicit executor choice.** The formula is `reads_attempted == 8 && R > 8`, with cached/shared initialization excluded from actual logical calls. The proposal preserves and explains the simultaneous selection/read-limit ambiguity rather than claiming the DTO resolves it. This follows the revision 2 assignment's root-directed executor interpretation.
3. **Bearer policy is explicit and source-consistent.** It permits only the current dev-only static-token identity under existing validated config, retains no token, uses request context as the bearer lifetime boundary, and rechecks kernel perspective. Existing `AuthOptions` explicitly calls `StaticToken` DEV-ONLY and config-restricted (`internal/server/session.go:45-70`); `requestIdentity` carries session-or-bearer source (`:64-82`). No SessionStore revocation guarantee is claimed for bearer.
4. **Aggregate limits and byte risk are surfaced.** The new 16-cohort/128-attachment limit is distinct from the accepted 16-poller map bound; full capacity fails before list/read and has no invented counts. Capacity-one wake hints avoid payload backlog. The proposal quantifies that frame and line limits alone do not bound heap/RSS and labels a separate 1 MiB serialized per-poller retention budget as a new approval-required policy.

The original shared `(case_id, run_id)` poller key, 16 entries including draining, no eviction, accepted shared physical admission ownership, one-second floor, eight selected/read limits, count equations, 1,024 source frame cap, 1 MiB initial JSON cap, and no-public-wiring boundary are represented faithfully. Actual ports confirm list and trace source shapes (`internal/ports/ports.go:291-295,442-459`; `internal/ports/run_trace.go:5-30`); safe trace projection verifies exact identity/standing and caps at 1,024 (`internal/adapters/sfwp/run_trace.go:66-176`). Existing `SessionStore.Current` returns detached claim/deadline/revocation state without extending idle lifetime (`internal/auth/session.go:146-169`), and server perspective verification is a separate check (`internal/server/server.go:385-400`); the proposal correctly retains both checks.

## Blocking design omissions

### 1. `Prepare` failure does not specify release of its reserved lease or attached pollers

`Prepare` first reserves a `preparing` cohort lease (revision 2, “Cohort assembly,” step 1), then can fail at list refusal/transport/decode/guard (`step 2`), post-list auth/guard (`step 3`), or per-candidate authorization/guard/read/identity validation (`steps 5-6`). It returns an error/no lease in these cases, but no transition or cleanup rule states how the preparing lease is removed, how any pollers attached earlier in the candidate loop are detached, or how pending reads are canceled and joined. Since callers receive no `CohortLease`, caller cleanup cannot be assumed. Repeated failed requests can therefore exhaust the proposed 16-lease table or leave run attachments behind.

Specify a single rollback path for every failed `Prepare`: mark the provisional lease draining under the same lock; stop further additions; detach all run references acquired by this cohort; cancel only pollers that no other watcher needs; join this preparation's list/read work and notifier refs outside the lock; and free the lease/map entries only after completion. Test failures after reservation, after list, after the first successful attach, during a shared initializer, and during shutdown. This is essential to the proposed capacity and lifecycle guarantees.

### 2. The initial observation versus `Next` delta contract is ambiguous

`Prepare` returns only `CohortLease`, and the lease exposes `Next(ctx) (ObservationDelta, error)`. Yet the proposal discusses an initial full observation/count DTO, then says `Next` computes a bounded-window delta. It does not state whether the first `Next` returns the complete initial payload, whether the initial result is delivered separately, or how the 1 MiB initial envelope limit is tied to a particular return value. The private delta type is not defined sufficiently to establish the initial count/status fields or later-frame merge semantics.

Define the exact first-result shape and sequence (for example, make the first successful pull return a typed initial cohort, and subsequent pulls return deltas), exact private result/error variants, and how a lease's per-run watermark initializes and advances. Keep eventual public DTO/SSE mapping held, but make the private producer-consumer contract unambiguous.

### 3. Required attach boundary and method semantics are not concrete enough

The assignments require concrete constructor/attach/lease/stop interfaces. Revision 2 provides `Prepare`, `CohortLease`, `Next`, and `DetachAndDrain`, but no exact private `attach` signature/result for reserving/joining pollers, nor a concrete `SessionCurrent` interface shape. Its prose describes single-flight and candidate attachment but leaves the initializer ownership transfer, returned reference, error result, and rollback path to interpretation. Define the private attach method and its typed outcomes, including exact current cache, shared initializer, new reservation, capacity, draining, and mismatched-identity outcomes.

The general authorization dependency can remain a seam, but its existing methods and expected values should be written explicitly: `Current(sessionID) (auth.CurrentSessionState, bool)` and `VerifyPerspective(ctx, ports.ActorClaim) error`, with request identity type/fields and bearer-vs-session branch identified. Sources provide these exact shapes (`internal/auth/session.go:147-169`; `internal/server/server.go:41-42`; `internal/server/session.go:64-70`).

## New policy proposals remain held

The 16-cohort lease cap and 1 MiB-per-poller serialized retention cap are correctly labeled as new and approval-required. They must not be treated as accepted merely because they appear in this revision. The byte budget also changes the retained set after polls beyond the original 1,024-frame window, so later `omitted/truncated` accounting and “new frame IDs still in the window” behavior must be explicit for initial versus ongoing results. Its stated serialized-byte bound is not a Go heap/RSS bound; the proposal correctly preserves that limitation. Root/operator review must select or reject these policies before any implementation that enforces them.

The unbounded bytes of in-flight 32 MiB line responses and decoded list bodies are also correctly distinguished from retained-buffer bounds. The proposal's theoretical 16-list-call and 2-physical-run-get figures are simple upper-bound products, not measurements; source-level confirmation of the actual client-wide concurrency/budget under mixed verbs remains necessary before any whole-process memory claim.

## Source/contract assessment and verdict

The guard inputs exist as described: `Revision.Facts` contains optional `CaseFacts` (`internal/projection/store.go:41-53`), `CaseFacts` contains `Cursor` and `Horizon` (`internal/projection/builder.go:34-50`), `CaseHorizon.Items` is the existing parent source (`internal/ports/ports.go:266-274`), and server dependencies expose `Trajectory` and `CursorForCase` (`internal/server/server.go:56-70`). `Authority.CaseHorizon` constructs a nonnil `Items` slice even when empty (`internal/adapters/sfwp/authority.go:172-193`), so rejecting a nil captured horizon while accepting a nonnil empty horizon is compatible with the current authority adapter. The stored horizon remains only as-of-cursor knowledge, as the proposal says.

The frame representation is safe and countable from current DTOs, but an exact private `ObservationDelta`/initial cohort model is still necessary to judge whether pruning preserves per-run order and truthful totals (`internal/ports/run_trace.go:10-30`). The execution plan honestly gates the extra byte policy and no test/runtime evidence is claimed.

**Verdict:** revision 2 resolves the four revision 1 findings at an architectural level but remains incomplete as the requested concrete private blueprint due to failed-`Prepare` rollback, the initial-versus-delta contract, and the missing explicit attach operation. Preserve the proposal and prior review unchanged. Request a bounded revision 3 addressing these items and keep source, tests, public integration, and runtime approval held. No tests, compiler, scanner, Graft build, Git, status, debt, or network operation was run.
