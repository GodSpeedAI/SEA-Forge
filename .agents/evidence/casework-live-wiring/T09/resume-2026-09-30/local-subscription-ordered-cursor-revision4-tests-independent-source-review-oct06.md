# Independent source review: local ordered-cursor bounds fixture

Date: 2026-10-06  
Artifact reviewed: `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorBounds.test.ts`  
Artifact SHA-256: `884987fb78dbe869c11addba3bc9a6e76101d41ce4af14f50156a47d5e31bc2f`  
Disposition: **REJECT as a complete test-only fixture.** Several tests identify real gaps, but the 65-event test contradicts the specified per-subscriber queue behavior, one case fails from an empty-timer setup error, and material accepted obligations are not covered. This source review is not a runtime or compile result and releases no implementation.

## Inputs and review boundary

Reviewed the complete local cursor root assignment, complete revision 4 proposal, revision 4 independent review and its citation erratum, the revision 4 test-review assignment, the full new bounds test, the full pre-existing frozen cursor-order test, and relevant current local adapter and fixture source. The reviewed design is `local-subscription-ordered-cursor-concrete-proposal-revision4-oct06.md`; earlier cited proposal line ranges are offset, so section names and the actual source spans are used here.

The frozen existing test remains byte-identical to the reviewed `373a` artifact: `localAdapter.cursorOrder.test.ts` SHA-256 is `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1`. The new artifact hash is recorded above. This review made no edits to either test or production source. No Bun, typecheck, test, build, Git, or network command was run; all RED/pass expectations below are source-level predictions only.

## Blocking findings

### 1. The 65-event fixture overflows both subscribers

At test lines 159-163, the `overflow` subscriber and the supposedly healthy subscriber are both registered before 65 synchronous appends. No timer is advanced until after all 65 events are published. Under the proposal's exact 64-pending-event limit, both subscriber queues therefore receive event 65 while still holding 64 pending entries. Both must be disposed and each must receive one overflow error. The assertions at lines 168-174 instead require the healthy subscriber to have no error and receive all 65 queued events plus event 66. Those expectations are incompatible with the accepted per-subscriber policy; a correct implementation would fail this test.

The helper only provides FIFO `runNext`/`runAll` (lines 26-45), and the test does not interleave any timer drain with publication. Merely registering healthy first would not establish continuity while the other watcher stays backlogged, because its already-scheduled timer remains ahead of future work. Repair needs deterministic selective timer control or another sequence that drains the healthy subscriber between publications while leaving the slow subscriber's timer pending. It must separately prove exactly 64 pending events stay ordered, then prove the 65th disposes only the slow subscriber and later events still reach the healthy one.

### 2. Leading-zero test throws before its intended assertion

At lines 82-88, the test appends `beforeRegistration` before subscribing, when the adapter has no listener. Current `emit` schedules callbacks only for listeners present at publication (`localAdapter.ts:433-435`), so this append creates no timer. The subsequent `timers.runNext()` at line 87 throws `no pending timer` before the leading-zero/floor assertions. This is a harness/setup error, not a semantic RED.

The boundary/future-floor test at lines 129-148 already supplies a useful real semantic RED for the ignored `_since` behavior: with a valid future floor equal to the next allocation, current `subscribeEvents` registers without filtering (`localAdapter.ts:168-173`) and delivers the boundary event. The separate leading-zero case should be reworked to use a deterministic future boundary with a zero-padded numeric equivalent, or otherwise assert a distinct numeric-equivalence behavior without calling `runNext` when no timer exists. Avoid duplicating the ordinary future-floor case without adding an observable distinction.

### 3. Reentrant FIFO ordering is not asserted

At lines 188-201, the reentrant listener enqueues an event on its first callback and self-unsubscribes on its second. The queued reentrant event is then deliberately canceled, so `reentrant === [first, second]` cannot establish where the reentrant event would have appeared in FIFO order. The healthy listener only has a length assertion (`3`), not an exact cursor sequence. This proves neither reentrant FIFO order nor that reentrant publication does not create a competing drain. Keep self-unsubscribe as its own assertion and add an active listener case that observes the reentrant event after events already queued before it.

### 4. Required callback-error and allocator-exhaustion behavior is absent

The fixture exercises a throwing `onEvent` and one timer-scheduling failure, but it has no throwing `onError` callback case. The proposal requires containing `onError` throws so they do not interrupt other subscribers (revision 4, “Ordered bounded delivery and cleanup”; test matrix item 7). The existing `onEvent` test is useful: current `emit` invokes the listener without a catch, so the manually fired callback throws out of `runAll` rather than reporting one error and allowing the healthy callback to continue.

No test reaches the proposed allocator boundary `9_999_999_999`, verifies successful allocation at that ceiling, and then verifies the next snapshot/progress/settlement attempt refuses without changing snapshots, points, frontier, or publishing a cursor. The `1.10000000000` input at lines 100-110 only checks subscription-input validation above the ceiling; it does not test allocator exhaustion or atomicity. Revision 4 test matrix item 8 requires this. No test proves progress or settlement timer exhaustion reports once without emitting, or that epoch does not wrap. A private test setup can seed the private allocator/cursor state through white-box access if its proposed representation is fixed; if the representation is not safely injectable, report that constraint and keep the boundary uncovered rather than infer coverage from invalid-input tests.

## Coverage by requested behavior

| Obligation | Source-level assessment |
|---|---|
| Decimal syntax, leading-zero numeric equality | Invalid strings/unsafe components/wrong epoch/over-ceiling are enumerated. The leading-zero test currently throws on `runNext` because no timer was created; it does not reach the intended assertion. |
| Invalid supplied cursor: one error, no registration | Lines 100-126 exercise a useful observable: append after invalid registration, drain, require one error and no event. Against current source, the listener registers and receives the event, so this is a meaningful semantic RED. |
| Valid same-epoch future floor | Lines 129-148 are a meaningful semantic RED: production ignores `_since`, so it delivers the exact boundary which should be suppressed. |
| Constructor seed agreement/failure | Lines 63-77 check default final snapshot/point equality, then mutate the shared Northstar trajectory point and restore it in `finally`; construction should reject a snapshot/point mismatch. Current constructor shallow-copies the point array and does not validate, so mismatch rejection is a real RED. It does not test malformed/unsafe/other-epoch/over-ceiling seed syntax or the template-committed case seed. |
| Per-subscriber FIFO 64/65, healthy continuity | Invalid as written: both subscribers get 65 pending entries and must overflow. No exact-64 ordered-success assertion exists. |
| Reentrant ordering/self-dispose/throw isolation | Self-unsubscribe is exercised and should fail current captured-timeout behavior. Reentrant FIFO ordering is not distinguished because that reentrant item is canceled before delivery. Throwing `onEvent` has a useful case; throwing `onError` is absent. |
| Timer cleanup/schedule failure after commit | Lines 220-247 exercise `finally` restoration and cancellation before drain; current emitter cannot clear a queued timeout, so the `hasPending` assertion is an intended RED. Schedule-failure test is source-distinguishing: current `append` commits snapshot/point then `emit` lets `setTimeout` throw (`localAdapter.ts:401-430,433-435`), while the proposal requires contain/report once after commit. The fixture doesn't assert post-failure trajectory/point state explicitly, though returned committed snapshot is checked on the intended implementation. |
| Shared fixture mutation cleanup | The only shared fixture mutation is restored in `finally` (lines 69-76). Each test that installs timer replacements restores both global timer functions in `finally` (all test bodies, lines 80-250). |
| Existing cursor-order test preservation | Hash matches the frozen reviewed `373a` test. Existing test covers chronology, event order, immutable retained history, case-locality and pre-drain unsubscribe, but does not fill the new bounds/parser/exhaustion gaps. |

## Additional limits and source checks

- Manual timers are deterministic for present cases: callbacks are selected in insertion order, scheduled delays are intentionally ignored, and each installed helper is restored in `finally`. Unlike the frozen test helper, this helper has no maximum callback/scheduling guard and no selective-run facility. The lack of selective control directly blocks the current healthy-continuity test shape; bounded callback protection would also make failure behavior clearer.
- `appendFixture` invokes private `append` for deterministic mutation. This is a private white-box dependency, not a public-interface change. The callback subscription helper casts to the existing four-argument `CaseworkPort`; the production adapter currently declares only three parameters, which is why its ignored `_since` behavior remains observable. No test-only production export is introduced.
- Constructor mismatch injection is restored safely, but it relies on the imported trajectory objects being mutable. `northstarData.ts` exports the JSON trajectory as `TemporalCheckpoint[]`; the constructor stores a shallow array copy (`localAdapter.ts:82-84`), so mutating the final point changes the seed presented to a later constructor while leaving the independently built snapshot cursor untouched.
- The initial template fixture is installed with matching snapshot/point cursors in the constructor (`localAdapter.ts:84-98`). Template-committed cases are created from one `snap.cursor` copied to the new point (`:313-340`). The new test does not exercise either committed-case equality or a mismatch/failure path. Current `commitCase` derives both from the same local snapshot and has no input seam for an invalid cursor, so a failure-path test may require an explicitly documented private fixture seam; do not claim the constructor test covers it.
- Current `append` derives the next sequence directly from the retained snapshot cursor and mutates history before scheduling delivery (`localAdapter.ts:401-430`). `progress` reuses the latest snapshot cursor (`:391-399`), and settlement emits a second event with its snapshot cursor (`:378-388`). These spans confirm that boundary filtering alone cannot cover the accepted shared-allocation and exhaustion requirements.
- No conformance fixture was changed by this test artifact. The accepted revision 4 proposal still requires future-only conformance to pass captured trajectory head while retained replay preserves its original cursor; these new tests do not verify that branch behavior. This remains a separate source/test obligation, not a reason to edit conformance here.

## Required repair before reconsideration

1. Remove the no-pending-timer call from the leading-zero case and make the case actually distinguish numeric equivalence from string comparison with a meaningful source-level RED.
2. Rewrite the overflow case so only one subscriber remains backlogged at its 65th pending event; add an exact 64-event FIFO success assertion and prove the healthy subscriber continues through later events.
3. Assert reentrant FIFO order with a listener that remains active through delivery; keep self-dispose as an independent assertion.
4. Add an `onError`-throws isolation case.
5. Add near-ceiling allocator success/refusal atomicity and progress/settlement no-publication assertions, or record a concrete untestable seam limitation against the accepted matrix.
6. Cover template-committed seed agreement and explicitly handle the lack of a reachable invalid committed-seed path without adding production interfaces or changing fixture scope.
7. Preserve/freeze the existing cursor-order test and rerun independent source review. Root alone owns any compile or runtime authorization.

## Verdict

**Reject this test-only artifact for now.** The invalid-input, ordinary future-floor, constructor mismatch, unsubscribe, callback-throw, and scheduling-failure cases contain useful intended checks, and global/shared state restoration is correct. However, the leading-zero test has an unrelated setup failure, the queue-capacity test expects behavior prohibited by the proposal, and required FIFO, error-containment, template-seed, and allocator-exhaustion behavior is missing or ineffective. No compile status or actual RED is claimed. No implementation approval follows from this review.
