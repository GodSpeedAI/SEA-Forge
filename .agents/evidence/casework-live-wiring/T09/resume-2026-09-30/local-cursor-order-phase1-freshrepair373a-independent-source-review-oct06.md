# Local ordinary cursor ordering Phase 1 — fresh repair source review

Date: 2026-10-06  
Verdict: **SOURCE APPROVED for root consideration of one bounded semantic RED.** No runtime, compile, or test claim.

## Reviewed artifacts and frozen source

- Original assignment: `local-cursor-order-phase1-original-assignment-oct06.md`.
- Fixture repair assignment: `local-cursor-order-fixture-repair-original-assignment-oct06.md`.
- Prior fixture review and the six repaired findings: `local-cursor-order-phase1-fresh-repair-independent-review-oct06.md`.
- Rejected actual RED review: `local-cursor-order-phase1-actual-red-independent-review-oct06.md`, SHA-256 `c2c65c25af3679a9a392b89fa4dcdd86bb863e36a810be47ae5aac463f2df531`.
- Preserved direct Bun output: `ui-cursor-order-red01-bun-output-direct-original-copy.raw`, SHA-256 `4d31b7e2ca43947ed546d8cd4f73310525f5dcfa57126dc771bc6fc55d249419`; its source path naming mismatch is disclosed in `local-cursor-order-phase1-actual-red-capture-path-addendum-oct06.md`.
- Candidate test `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorOrder.test.ts`, SHA-256 `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1`.
- Frozen production `localAdapter.ts`, SHA-256 `d8b0feb02e489587672968c7450373f000ba02e3fd3d0d509888ca1e0f595f34`.
- Frozen conformance `caseworkPortConformance.ts`, SHA-256 `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c`.

I read both complete fixture assignments, the prior source reviews, the complete current test, complete conformance source, archived actual RED, and relevant adapter/port source. I inspected the current changed fixture line and made an in-memory inverse-only comparison: changing only `await drain(timers)` after `appendFixture(adapter, 'one immutable history addition')` back to `timers.runNext()` hashes to the previously reviewed fixture `c625cb812d5d67ac3428d1f97e6ba452c39acc327828b2f62dd72a4ba53f8fac`. Thus the candidate delta from that fixture is exactly this one history-timer line. This inverse comparison is structural verification, not original runtime evidence.

## One-line fixture repair and the old setup failure

At test lines 247-251, after the progress listener is unsubscribed, the test appends an immutable history point and now calls bounded `await drain(timers)` before its separate trajectory query. Current `emit` schedules subscriber callbacks only when listeners exist (`localAdapter.ts:433-435`); with this listener removed, the append has no callback timer to fire. Therefore the prior `timers.runNext()` threw `no pending timer`, as the archived actual RED review records, and prevented the post-append history and progress-cursor lookup assertions from running. The bounded drain correctly handles the valid zero-queued-callback state. The later `queryTemporalTrajectory` still schedules its own read delay via `wait()` (`localAdapter.ts:162-166,453-455`), and the fixture explicitly advances that timer at line 250 before awaiting.

No assertion, fixture value, test order, helper, import, timer logic, API, source, or conformance assertion changed. The single-line repair removes a harness error; it is not presented as a production semantic fix.

## Prior six fixture findings: all repaired in the current candidate

1. **Async callback / existing port type:** the omitted-boundary test callback is `async`; the four-argument subscription is made through an existing `CaseworkPort`-typed reference, so its `onError` argument matches `ports/contract.ts:204-216`. No production overload was added.
2. **Distinct future-floor RED:** the same-epoch future-floor test queries the head with a manual timer, uses `nextCursor(head)` as a valid exclusive floor, appends exactly at that floor, and expects suppression (`test.ts:114-138`). Production accepts but ignores `_since` and registers the listener (`localAdapter.ts:168-173`); append emits to it (`:402-435`). This is a real source-distinguishing assertion, unlike the pre-registration H+1 chronology alone.
3. **All ordinary event cursors:** the public execution fixture includes progress, snapshots, and side events in cursor uniqueness and strict numeric epoch/sequence ordering (`test.ts:167-207`). It requires settlement side event strictly after its snapshot and the subsequent snapshot after all preceding ordinary events.
4. **Bounded delivery work:** manual timer scheduling is capped at 500 callbacks and virtual time at 60,000 ms; `drain` also has a finite callback guard. It sorts by due time and insertion order, and each test restores global timers in `finally` (`test.ts:6-43,62-70`). No sleeps or retry loops.
5. **Exact independent case allocator assertions:** the case test captures both initial heads and asserts exact next cursors for Northstar and template cases, in addition to routing/disposal (`test.ts:262-285`).
6. **History and exact lookup:** the test captures trajectory and retained snapshots before progress, compares them unchanged after progress, confirms a later append adds exactly one revision point, and requests the progress event cursor through exact snapshot lookup (`test.ts:214-256`; source `localAdapter.ts:146-166`). The fixed timer ordering now permits these assertions to execute.

The test still keeps omitted-boundary and supplied-`H` H+1/H+2 scenarios separate (`test.ts:85-112,141-165`). The first proves no `onError` for omitted floor and a returned unsubscribe. The supplied revision test asserts pre-registration H+1 suppression and later H+2 delivery. These are useful chronology checks but are expected to pass on the old listener implementation and are not represented as the semantic RED.

## Current source expectations and unreached assertions

The frozen production source is a genuine pre-fix subject:

- Supplied `_since` is ignored (`localAdapter.ts:168-173`), so a future-floor append is delivered; expected semantic failure at test line 129.
- Settlement creates a snapshot and emits a side event with the same `snap.cursor` (`:378-388`). Exact `getSnapshotAt` searches retained snapshots by cursor (`:146-150`), so the side-event cursor resolves; expected semantic failure at test line 189.
- `progress` reuses the latest retained snapshot cursor (`:391-399`), so querying that progress cursor resolves a snapshot; after the new timer repair, expected semantic failure at test line 256.
- `emit` captures the listener in the timeout and does not recheck membership at drain time (`:433-435`), so unsubscribe-before-drain still calls it; expected semantic failure at line 280.

These are source-derived predictions, not a new RED result. In the public execution test, the first settlement lookup assertion at line 189 fails against current source before the later all-event cursor uniqueness/order and subsequent-snapshot checks can execute. In the case-local test, the unsubscribe assertion at line 280 fails before the later template-case cursor assertion. The fixture contains the intended assertions, but their current-runtime reachability is not claimed. In the history test the immutability comparisons and one-point history assertion should execute after the repaired no-op drain, then the progress cursor lookup is expected to fail. Omitted and supplied H+1/H+2 chronology is expected to pass against current listener registration. No runtime claim relies on the archived run alone.

The conformance diff remains frozen and branch-only as required: future-only passes captured `headCursorBeforeResume`, replay continues using exact original `snapshot.cursor`, and replay's strict accepted-cursor/prefix assertions remain intact (`caseworkPortConformance.ts:109-150`). Production and conformance hashes match assignment identities.

## Scope not covered and disposition

This focused file does not prove parser/invalid cursor behavior, bounded FIFO-64 capacity/error reporting, counter ceiling/exhaustion, constructor validation, full adapter conformance, or `execution_observation`. These remain separately held. No cursor algorithm or implementation is authorized by this fixture review.

**Approve this fixture's source shape for root to consider one bounded actual RED.** The only delta from the previously source-approved candidate removes the demonstrable zero-queue setup failure. This verdict does not assert the repaired suite ran, grants no compiler token, and does not release production changes. Root must make the separate actual-RED/resource decision.

No compiler, tests, scanner, Git, Graft build, or network action was run for this review; no source, test, conformance, status, or debt file was changed.
