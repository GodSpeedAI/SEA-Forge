# Independent source re-review: repaired local cursor bounds fixture

Date: 2026-10-06  
Reviewed artifact: `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorBounds.test.ts`  
SHA-256: `3a801b86b5e39457f6b057777bbd29ea535634005eb8bb0211d2d888aaf14e11`  
Verdict: **REJECT as a complete test-only fixture.** The earlier dual-overflow and empty-timer defects are repaired, but several new assertions still test the wrong case/boundary or can mask state mutation. The settlement test bypasses the start path and depends on a fixed timer ID. Source-only review; no compile or runtime claim.

## Inputs and scope

Read the prior independent rejection `local-subscription-ordered-cursor-revision4-tests-independent-source-review-oct06.md` (SHA-256 `c80c591af140180431e9ef505a041d64a359841de9a4556cf311cce08fdb5153`), original local cursor assignment, complete revision 4 proposal and review/citation erratum, the fresh repair record `local-cursor-bounds-fresh-repair-record-oct06.md`, repaired fixture, frozen cursor-order fixture, current local adapter, and its fixture seeds. The design authority for this check is revision 4's **“Private state, initialization, and cursor semantics,” “One allocate-and-publish linearization,” “Subscription behavior, including omitted and supplied floors,” “Ordered bounded delivery and cleanup,”** and test matrix items 3, 6, 7, and 8. The root-selected private setup contract is an optional `ordinaryFrontiers: Map<string, {epochText, epoch, sequence}>`; test setup must update it when present and always set the private last snapshot/point cursor.

No Bun, typecheck, tests, compiler, build, Git, Graft build, or network action was run. The old rejected test bytes remain preserved at `localAdapter.cursorBounds.test.ts.884987.original.raw` with exact SHA-256 `884987fb78dbe869c11addba3bc9a6e76101d41ce4af14f50156a47d5e31bc2f`. The frozen prior test hash is still `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1`; production `localAdapter.ts` is still `d8b0feb02e489587672968c7450373f000ba02e3fd3d0d509888ca1e0f595f34`; shared conformance remains `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c`. No unrelated source or test change was found by these byte identities.

## Blocking findings

### 1. Template test subscribes to Northstar, then appends to another case

`subscribe` at test lines 101-103 always calls `subscribeEvents(NORTHSTAR_CASE_ID, ...)`. The template test at lines 122-139 commits a new case, obtains its ID, then invokes that helper at line 133 before appending to the committed case at line 134. The new subscription is therefore on Northstar while the event is published to the template case. With case-local listener sets (current `localAdapter.ts:72-73,433-435`), the asserted template event at line 137 cannot be received even with correct cursor behavior. Parameterize the helper with an optional `caseId`, or call the port directly with the committed ID. This is a test setup failure, not a production semantic RED.

The underlying seed check itself is useful: current `commitCase` initializes both `snap.cursor` and the point from that same value (`localAdapter.ts:313-340`), and the test verifies a subsequent allocation. But until it listens on the created case, it does not verify the proposed committed-case delivery/allocator handoff.

### 2. Leading-zero test's alleged boundary is off by one

The last Northstar snapshot in the checked fixture is `1.0000000006` (`data/northstar.snapshots.json:3167`; constructor takes that history and trajectory in `localAdapter.ts:82-84`). The new test uses floor `0001.0000000008` at line 152. Per the root-selected allocator seed, the first allocation is `.0007`, and the second is `.0008`. The exclusive floor therefore suppresses both events: `.0007` is below it and `.0008` equals it. Yet the test calls the first append `boundary`, expects it to equal `.0008` at line 159, and expects the second append to be delivered at line 160. A correct implementation seeded from the fixture would produce `.0007` then `.0008` and fail these assertions.

Derive `H` from the current final snapshot/trajectory in the test and compute floors/cursors from `H`; do not hardcode the fixture's assumed `.0007/.0008` sequence. For a zero-padded numeric-equivalent boundary, allocate the exact event whose numeric sequence equals that floor and assert suppression, then allocate a strictly greater event and assert delivery. Keep this case meaningfully RED against the ignored-`_since` baseline without calling `runNext` when no callback exists.

### 3. The 64/65 tests still do not prove that 64 queued items are accepted

The repair now selectively drains the latest healthy timer while leaving the older subscriber backlogged (`:218-249`, helper `runHandle`/`latestHandle` at `:26-52`). That repairs the prior contradiction: only the slow subscriber reaches 65 pending events while the healthy subscriber is drained each iteration. The healthy path's 64-event FIFO order and later continuity are asserted.

However, neither the overflow test nor the throwing-`onError` test asserts that the slow subscriber has **no error after exactly 64** events. In the overflow test `overflowErrors` is first inspected only after event 65 (`:229-242`). An implementation that incorrectly rejects on event 64 and disposes that watcher would still produce exactly one error by the final assertions, leave `overflowEvents` empty, and allow the healthy watcher to receive 65 and 66. The throwing-error test likewise first checks `onErrorCalls` after event 65 (`:327-335`), so an incorrect threshold of 64 also produces one call and can satisfy it. Add pre-65 assertions that the slow subscriber is still active with no overflow error, then trigger event 65 and assert exactly one disposal/error. The exact 64-pending acceptance boundary is not currently established.

The helper's handle selection does not inspect private production state and is otherwise deterministic for this registration order: overflow is registered first and healthy second, so each append's last schedule handle is healthy's drain. It caps scheduled handles at 1,000 and `runAll` at 500. That source-level assessment assumes the private implementation schedules at most one drain per active subscriber, as revision 4 specifies.

### 4. Snapshot frontier comparison aliases mutable state

At line 352, `frontierBefore` stores the map's object reference directly. The later equality assertion at line 356 compares the map's object with the same object reference. If a failed allocation mutates `sequence` on that object in place before throwing, both references expose the mutated value and the equality assertion passes. The snapshot/point clones catch history mutation but cannot catch this frontier mutation. Copy scalar fields at capture time, e.g. `{ ...frontier }`, and compare against that detached value. This is required for the atomicity claim in revision 4's test matrix item 8.

The helper's boundary setup otherwise follows the root-selected white-box contract: it always changes the adapter-local last snapshot and point, and, when an entry exists, replaces that case's `ordinaryFrontiers` value with the requested exact epoch text/numeric epoch/sequence (`:82-98`). It leaves the current baseline usable when the map is absent. The missing clone is specifically in the failure oracle.

### 5. Settlement exhaustion bypasses the start path and hardcodes timer 7

The test seeds the ordinary frontier and history to the ceiling before calling private `execute` at lines 385-405. The real accepted path calls `append` for the action/start snapshot and only then calls `execute` (`localAdapter.ts:234-270`). If the future implementation makes the start operation allocate an ordinary event or otherwise validates the current frontier at that boundary, seeding to the ceiling before execution makes setup fail before the intended settlement callback is reached. It also bypasses the behavior that connects execution start to timer scheduling.

Run the actual start path at a normal head first, consume/verify its start snapshot/delivery deterministically, then seed the last snapshot/point and optional frontier to the ceiling. Identify the settlement callback from the set of timer handles registered by that execution after the initial start work, not by global literal handle `7`. Current `execute` schedules four phases, completion, settling progress, and settlement in that order (`localAdapter.ts:347-388`), so handle 7 happens to be the final settlement callback only in the current private direct-call setup with no prior timer. The actual start path/public listener or future scheduling additions shift absolute handles, making the literal brittle and potentially directing the test at a progress/completion callback instead of settlement.

After invoking the actual settlement callback, the existing snapshots/points/no-side-event and final-ordinal checks are valuable, but they only prove the desired path if the settlement timer was indeed selected. Current tests do not satisfy that precondition robustly.

## Repair findings that are now resolved

- Original fixture preservation is exact: `.884987.original.raw` matches the previous source-review hash, and the frozen `cursorOrder373a`, production `d8b0feb02e`, and conformance `edf8ed69` hashes remain unchanged.
- The leading-zero test no longer calls `runNext` with no timer, but the replacement has the off-by-one floor described above.
- The slow/healthy selective timer approach avoids overflowing both watchers; exact-64 threshold acceptance still lacks a pre-65 assertion.
- Reentrant FIFO now observes `[first, second, reentrant]` while remaining active (`:257-277`), and self-unsubscribe is independently tested (`:279-289`). These are meaningful distinctions from the previous fixture.
- Separate `onEvent` and throwing-`onError` isolation tests now exist (`:295-341`). Both restore global timers in `finally`.
- Near-ceiling snapshot, progress, and settlement test cases now exist (`:343-409`), but the frontier oracle alias and settlement setup/handle selection block full acceptance.
- The template committed case now has a seed agreement and subsequent-allocation check, but the shared subscribe helper targets the wrong case.
- Global timer replacements are restored in `finally`; shared `NORTHSTAR_TRAJECTORY` mutation remains restored in `finally` (`:106-120`). No shared fixture or timer leak is apparent from source inspection.

## Other coverage and verification limits

Malformed, unsafe-number, wrong-epoch, and above-ceiling supplied floors remain present (`:167-195`) and the invalid-input test appends afterward to detect accidental registration. The ordinary future-floor test remains a source-distinguishing RED against current `subscribeEvents`, whose `_since` argument is ignored and whose listeners receive scheduled events (`localAdapter.ts:168-173,433-435`). The parser/validation behavior remains unrun; no source-level result establishes compile readiness.

Constructor fixture mutation tests snapshot/trajectory disagreement and safely restores the imported point. The template-committed path can generate only a matching cursor today because both values derive from the same `snap.cursor` (`localAdapter.ts:313-340`). The test therefore proves valid seed agreement only; no invalid committed-seed path is claimed. If validation failure at that internally generated boundary cannot be injected without a production seam, keep that limitation explicit rather than widening the test API.

The ceiling tests compare history snapshots/points and `ordinaryFrontiers` on overflow, and progress exhaustion expects one error/no event. The settlement case must be repaired as above. None of these were run, so source-only review cannot establish meaningful runtime RED or resolve harness/type errors not evident by inspection.

## Required repair before approval

1. Make the subscription helper accept and use a case ID, then target the committed template case.
2. Derive leading-zero and expected allocation cursors from captured `H`; suppress the equal-floor event and deliver only a strictly later allocation.
3. Assert the slow subscriber has zero overflow/error calls after 64 items in both threshold fixtures, then verify only event 65 disposes it.
4. Deep-copy the optional frontier scalars before the failed allocation so the atomicity check cannot alias in-place mutations.
5. Exercise settlement exhaustion from a normally started execution, then seed to ceiling and select the settlement callback through a controlled handle range tied to that execution, not hardcoded global handle 7.
6. Preserve all four verified hashes and run another independent source review. Root alone controls any subsequent RED/compiler/runtime decision.

## Verdict

**Reject for now.** The new fixture is a meaningful improvement and repairs most findings from the first review, but its template subscription is case-mismatched, its leading-zero boundary expectation is invalid for the fixture head, its queue tests do not establish the 64-versus-65 threshold, its frontier check aliases the object under test, and its settlement case is brittle and bypasses the actual start path. These are concrete source-level defects, not runtime claims. No source, test, status, or earlier review artifact was changed by this review.
