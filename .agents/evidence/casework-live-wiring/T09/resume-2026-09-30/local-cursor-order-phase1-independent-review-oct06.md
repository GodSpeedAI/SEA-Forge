# Local ordinary cursor ordering Phase 1 — independent source review

Date: 2026-10-06  
Verdict: **REJECT; fixture repair required before any actual RED run.** Source-only review; no source/test changes or execution performed.

## Reviewed artifacts and boundary

- Original Phase 1 assignment: `local-cursor-order-phase1-original-assignment-oct06.md`.
- Complete accepted proposal revision 4: `local-subscription-ordered-cursor-concrete-proposal-revision4-oct06.md`, SHA-256 `6d7baafd41a098cc24556b32ea222c999ffe4ea116998740339acd2086131873`.
- Revision 4 independent architecture review: `local-subscription-ordered-cursor-revision4-independent-review-oct06.md`, SHA-256 `893ddd2e2e75ed63b14689fa19ddbbb8f00d6e11e5725b523e57922eaba359ab`.
- Review citation erratum: `local-subscription-ordered-cursor-revision4-review-citation-erratum-oct06.md` (section-name correction only; no semantic change).
- New local fixture: `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorOrder.test.ts`, SHA-256 `c41a5f372a2d65c9a7b8d8f8e5a6003390c9bbc268dc620ce9259ff98917b236`.
- Shared conformance fixture: `apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts`, SHA-256 `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c`.

I read both complete test/conformance files, verified their imports and private whitebox signatures against the current local adapter, and inspected the read-only HEAD diff for the conformance file. The conformance diff changes only the resume argument: future-only receives captured `headCursorBeforeResume`, replay still receives `snapshot.cursor`. The replay branch and its strict accepted-cursor/chronological-prefix checks are otherwise unchanged. That change matches the assignment and proposal; it does not imply the local adapter implementation now honors the floor.

## Blocking findings

### 1. The immutable-history test blocks before it can assert anything

In the test `progress leaves retained snapshots unchanged and each new snapshot adds one history point`, `localAdapter.cursorOrder.test.ts:170` awaits `adapter.queryTemporalTrajectory(...)` before calling `timers.runNext()` at line 171. `queryTemporalTrajectory` awaits `this.wait()` (`localAdapter.ts:162-166`), and `this.wait()` schedules `setTimeout` (`:453-455`). Since the test has replaced global timers with a manual queue, that promise cannot settle until the test fires the queued timer; the test is awaiting before doing so. This is a hang, not an expected semantic RED. Capture the promise, run its timer, then await it, following `readSnapshot`'s pattern at test lines 47-50. No later assertions in that test are reached as written.

### 2. Execution cursor assertions contradict the accepted ordinary-event contract

The accepted design allocates distinct increasing ordinary ordinals to snapshots, every `execution_progress`, and the `settlement_recorded` side event; it explicitly places the side event after the settlement snapshot (revision 4 section “One allocate-and-publish linearization” and acceptance item 4). But the execution test filters every progress event out before checking uniqueness/order (`localAdapter.cursorOrder.test.ts:149-152`) and then explicitly requires the settlement event cursor to equal the immediately preceding settlement snapshot cursor (`:153-154`). Those conditions cannot accept the proposal: the progress cursor reuse that is the central bug remains unasserted, and the settlement equality assertion directly demands the old duplicated cursor. A correct implementation with a distinct later settlement-side-event cursor would fail this fixture. Assert strict increasing order across **all** emitted ordinary events including progress, and assert side-event cursor is strictly greater than the preceding snapshot cursor. Then assert the later appended snapshot exceeds the last cursor across the full event sequence, not the progress-filtered subset (`:155-159`).

The source confirms why these assertions matter. `progress` currently copies the latest snapshot cursor (`localAdapter.ts:391-399`); settlement emits the side event with `snap.cursor` after `append` emits the settlement snapshot (`:378-389`); `append` independently increments the revision cursor and emits a snapshot (`:402-431`). The old code therefore reuses cursors for both progress and settlement side events.

### 3. The two floor tests do not drain the later subscriber's callback

In the omitted-boundary case, after appending H+2 at test line 87 with both `prior` and `late` listeners active, line 88 calls `timers.runNext()` once and line 89 expects `late` to have received H+2. In the supplied-H case, H+2 is appended at line 116 and line 117 fires only one timer before line 118 expects the second subscriber's event. The current `emit` schedules one timer per listener (`localAdapter.ts:433-435`), and the accepted design uses a separate FIFO/drain timer for each subscriber (revision 4 section “Ordered bounded delivery and cleanup”). In registration order, one call drains `prior`/`existing`; the target subscriber's timer remains pending. Thus both tests can fail even when that target subscriber is correctly queued for delivery. They need a bounded timer-drain helper or explicitly fire both callbacks before asserting. Keep the old H+1 suppression assertion separate in each case.

### 4. Manual timer draining has no bound

The `drain` helper at test lines 53-59 loops until no callback remains, with no maximum callback count or failure guard. The timer queue grows without a limit at lines 8-20. A regression that continually reschedules work can hang this focused fixture indefinitely. The test harness should enforce a finite callback budget and fail with pending timer diagnostics when exceeded. `finally` restoration is correctly present in every test, which limits global timer contamination even on assertion failure; this does not bound a runaway drain.

## Coverage assessment

The accepted parts are concrete and source-grounded:

- The whitebox append helper casts only to the real existing `append(caseId,eventType,summary,intent,edit): XSnapshot` shape (`localAdapter.ts:402-431`); it adds no production export or fixture option. The progress cast matches the existing private `progress(caseId,run_id,phase,progress_percent,log_line)` behavior (`:391-399`).
- The public execution test dispatches a real intent through the current adapter API. Its `ns-release` / `APPROVE_HUMAN_TASK` shape matches the existing accepted-intent helper (`localAdapter.test.ts:262-271`), so it does exercise the execute timers rather than replacing execution with a test-only call.
- Omitted and supplied boundaries are distinct cases. Both queue an event before the second subscription; the omitted case uses `undefined`, and the supplied case uses a captured revision head. The first queued H+1 assertions are meaningful against the current `subscribeEvents` implementation, which ignores `_since` and registers listeners directly (`localAdapter.ts:168-173`); future-only suppression is expected to be a semantic RED. The supplied case obtains its head while correctly firing the query timer (`localAdapter.cursorOrder.test.ts:56-59`).
- The unsubscribe-before-drain case exercises an actual queued callback and verifies it is suppressed after unsubscribe (`localAdapter.cursorOrder.test.ts:211-228`). The current `emit` captures the listener in a timer closure without rechecking membership, so this is also a meaningful semantic RED (`localAdapter.ts:433-435`).
- The progress-history scenario, once its initial timer hang is fixed, intends to compare full trajectory and exact prior snapshots before/after a progress-only event and ensure progress cursors do not resolve via exact `getSnapshotAt` (`localAdapter.ts:146-151,162-166`; test `:166-205`). Its assertions express the right immutable-history contract, but currently are unreachable because of finding 1.
- The per-case listener routing test proves that an event for the template case reaches that case's subscriber and not the Northstar subscriber (`localAdapter.cursorOrder.test.ts:211-228`). It does **not** prove allocator independence: it does not capture both cases' starting heads or compare each case's next cursor progression. The only assertion about the template event cursor is truthiness (`:225-227`). Add a cursor-state assertion if “independent cases advance independently” is meant to prove independent allocator state rather than listener routing alone.

## Deliberately held scope and compatibility

The fixture correctly does not implement or claim coverage for invalid cursor parsing/seeds, queue capacity 64 and overflow/onError behavior, counter ceiling/exhaustion, or `execution_observation`; the Phase 1 assignment explicitly holds these for later deterministic fixtures. No public port signature changes are made. The conformance correction is precisely branch-only and retains its original replay cursor and assertions (`caseworkPortConformance.ts:109-150`, verified diff).

The added file appears source-compatible by inspection: its imported contract types and `LocalContractAdapter` methods exist; the whitebox signatures match current production methods; timer types use the platform `TimerHandler`/`ReturnType<typeof setTimeout>` already used by this UI environment. This is not an actual typecheck claim. No compiler or test was run, so compile and runtime outcomes remain unverified.

## Verdict and next move

**REJECT the fixture as insufficient for an actual semantic RED.** The history test hangs, the execution assertions conflict with the accepted distinct-ordinal semantics, both two-subscriber floor tests stop before the intended listener drains, and the timer drain is unbounded. The all-event uniqueness/order, distinct settlement cursor, next-snapshot-after-progress boundary, and allocator-level case isolation claims therefore are not proven. Preserve the current artifacts; a fresh builder should repair only the fixture, then a separate critic should re-review before root assigns a real RED. Do not implement the cursor algorithm, parser, queue64, or ceiling from this review.

No source/test changes, tests, compiler, scanner, Graft build, Git write, or network command was performed. Retrieval calls in this review reported savings: 19,561 tokens ($0.02), 5,318 tokens (<$0.01), and 2,072 tokens (<$0.01), approximately 26,951 tokens total (<$0.04).
