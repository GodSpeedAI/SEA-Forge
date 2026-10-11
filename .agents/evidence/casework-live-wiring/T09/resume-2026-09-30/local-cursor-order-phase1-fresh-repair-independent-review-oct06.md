# Local cursor-order fixture fresh repair — independent source review

Date: 2026-10-06  
Verdict: **SOURCE APPROVED for root consideration of a bounded semantic RED.**
No runtime, compile, or actual RED claim is made.

## Frozen inputs and review boundary

- Original Phase 1 assignment: `local-cursor-order-phase1-original-assignment-oct06.md`.
- Fixture repair assignment: `local-cursor-order-fixture-repair-original-assignment-oct06.md`.
- Prior fresh-repair rejection, SHA-256 `7469415c1a0578b29206a0d504713060bb1b3d636323eb0a02d0128621e6722a`:
  `local-cursor-order-phase1-repair-independent-review-oct06.md`.
- Accepted proposal revision 4, SHA-256 `6d7baafd41a098cc24556b32ea222c999ffe4ea116998740339acd2086131873`:
  `local-subscription-ordered-cursor-concrete-proposal-revision4-oct06.md`, with navigation correction in `local-subscription-ordered-cursor-revision4-review-citation-erratum-oct06.md`.
- Candidate test, SHA-256 `c625cb812d5d67ac3428d1f97e6ba452c39acc327828b2f62dd72a4ba53f8fac`:
  `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorOrder.test.ts`.
- Shared conformance remains exact at `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c`.
- Local production source remains exact at `d8b0feb02e489587672968c7450373f000ba02e3fd3d0d509888ca1e0f595f34`.

I read the complete candidate, both assignments, the full prior rejection and
the complete accepted revision 4 proposal. I checked the existing public port
signature, concrete subscription/emit implementation, append/progress/execute
paths, and retained snapshot/history methods. No source or test execution was
performed. Conformance and production identities match the frozen values.

## Prior rejection findings

1. **Async callback and existing port type: repaired.** The omitted-boundary
   callback is now `async`; the four-argument subscription with `onError` is
   invoked through `const port: CaseworkPort = adapter`, so it uses the existing
   interface at `src/ports/contract.ts:204-216` without adding a production
   overload. The source file remains untouched at the frozen hash.
2. **An actual future-floor semantic RED: repaired.** The new
   `same-epoch future floor suppresses its boundary allocation` test first
   reads the current trajectory head under a manually advanced timer, sets the
   valid floor to exactly `nextCursor(head)`, subscribes, then synchronously
   appends that exact cursor and drains its queued callback. Current production
   `subscribeEvents` accepts but ignores `_since` and registers the callback
   (`localAdapter.ts:168-173`); `append` creates the next cursor and `emit`
   schedules callbacks for currently registered listeners (`:402-435`). Thus
   current code delivers the event at the supplied floor and violates the
   assertion that the floor is exclusive. The accepted revision 4 contract
   explicitly permits a same-epoch future floor and requires delivery only
   above it. This distinguishes old and proposed behavior without relying on a
   missing export, test expectation alone, or a pre-subscription event that
   was never queued for the later listener.
3. **Required H+1/H+2 cases remain supplemental and separate.** The omitted
   and supplied-revision tests still assert their requested distinct
   pre-registration H+1 / post-registration H+2 chronology. Their callbacks
   are drained with bounded `drain`; these cases are not misrepresented as the
   semantic floor RED. The omitted case now checks no `onError` and calls
   through the existing interface.

## Full assignment coverage and source support

- Public execution still uses `dispatchIntent`; its accepted intent shape is
  present in `localAdapter.test.ts:262-271`. The manual clock advances the
  dispatch wait before awaiting the response, then `drain` advances scheduled
  progress, completion, settling progress, settlement snapshot, and side-event
  callbacks in due-time/insertion order. The test checks visible progress,
  settlement, unique cursors and strict numeric epoch/sequence order across all
  events, side event strictly after settlement snapshot, non-snapshot exact
  lookup, and a subsequent snapshot after the complete event sequence. Source
  cursor reuse that these assertions expose is at `localAdapter.ts:378-399`.
- Immutable history captures trajectory and snapshots, fires progress, then
  compares the captured trajectory and exact snapshots unchanged; it separately
  checks that progress does not add a point, that a later append adds exactly
  one point, and that exact lookup of the progress cursor rejects. The async
  read ordering fires each controlled timer before awaiting (`test.ts:200-243`;
  source `localAdapter.ts:146-166`).
- Case isolation captures both actual case heads and checks the exact next
  cursor for each case, in addition to event routing. It retains unsubscribe
  before queued callback drain. Source append increments from the requested
  case's own snapshot head (`localAdapter.ts:402-430`); current `emit` captures
  the listener in a timer callback without an active recheck (`:433-435`), so
  the disposal assertion is an independent expected semantic RED.
- Manual timers preserve relative deadlines and insertion order for ties,
  cap scheduling and virtual time, and `drain` has an explicit callback bound
  (`test.ts:6-43,62-70`). Every test restores global timer functions in
  `finally`. No sleeps or retries were added. All awaited adapter reads and
  dispatch receive explicit timer advancement before awaiting.
- Whitebox access remains limited to the existing synchronous `append` and
  `progress` methods (`localAdapter.ts:391-431`); real public dispatch remains
  covered. No production/API/conformance change or additional test file is
  present.

## Compile/type review and limits

By source inspection, the two prior compile blockers are removed: `await` is
inside an async Bun test callback, and four-argument `subscribeEvents` is
called on a `CaseworkPort`-typed value. The interface has that fourth
`onError` callback; the concrete adapter remains unchanged. Imports and helper
types refer to existing source declarations. This is a source-level
type-correctness review, not a compile result.

I found no new fixture blocker. The required parser/invalid-input, FIFO64/error,
ceiling/exhaustion, constructor validation, and observation-event units remain
explicitly held and are not claimed by this focused file. The supplied and
omitted H+1/H+2 scenarios are useful chronology coverage but, as the prior
review explained, are not independently semantic REDs against the old listener
registration behavior. The additional future-floor test supplies the missing
source-distinguishing assertion.

## Expected semantic RED boundaries

Based on frozen production source only, expected failures include:

- The future-floor test receives its cursor-equal-to-floor append because the
  current concrete subscription ignores the boundary.
- Public execution sees reused progress cursors and a settlement side event
  equal to its settlement snapshot (`localAdapter.ts:378-399`), violating
  uniqueness/strict numeric order and exact non-snapshot lookup.
- Unsubscribe-before-drain may invoke the already-captured callback because
  current `emit` does not recheck subscription activity (`:433-435`).

The ordinary omitted/supplied H+1/H+2 tests, case-local cursor increments,
historical snapshot immutability, and progress-only trajectory count are
expected to pass against the old subject. No assertion outcome was observed;
these are predictions, not runtime RED evidence.

## Verdict and next step

**Approve the fixture source shape for root's decision on one bounded actual
RED run.** The candidate addresses both compile/type findings and introduces a
source-supported exclusive-future-floor case while retaining all focused
obligations and frozen source identities. Root must review this complete result
and decide whether to grant the actual RED under its resource/ownership
protocol. This review grants no compiler token and makes no runtime or
production acceptance claim.

No tests, compiler, scanner, Graft build after the source retrieval, Git write,
or network command was run. No existing source, test, conformance file, status,
or debt record was edited.
