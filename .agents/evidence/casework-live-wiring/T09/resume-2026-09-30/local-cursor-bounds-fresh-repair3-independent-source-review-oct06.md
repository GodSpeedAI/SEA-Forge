# Local cursor bounds fresh repair 3: independent source review

**Date:** 2026-10-06  
**Verdict:** **APPROVE for the root-owned focused runtime RED attempt.** Source review found no remaining setup, deadlock, or assertion defect in this bounds fixture. This is not a compile or runtime result and does not approve production implementation.

## Scope and byte identities

Reviewed the complete test file, repair 3 record, original bounds/test-review assignments, revision 4 proposal, earlier independent reviews including the repair 2 rejection, frozen cursor-order test, and current local adapter/timer source. Checked all 14 tests and their private-state accesses. No compiler, typecheck, test, Bun, runtime, Git, or build command was run.

| Artifact | SHA-256 |
| --- | --- |
| reviewed `localAdapter.cursorBounds.test.ts` | `134ba6162b60eddcf46b9b92347a8da4feda8cca1a26b68ae31edb1922285e8a` |
| preserved `.3d3e1da.original.raw` | `3d3e1da51057fafde43508b76ef4e8a33d2f4f91324924ec2abddd53db62f576` |
| frozen `localAdapter.cursorOrder.test.ts` | `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1` |
| production `localAdapter.ts` | `d8b0feb02e489587672968c7450373f000ba02e3fd3d0d509888ca1e0f595f34` |
| shared `caseworkPortConformance.ts` | `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c` |

The frozen test, production adapter, and conformance file retain their recorded hashes. The only assigned source artifact changed since repair 2 is the bounds test; its previous bytes are preserved at the recorded raw file.

## Findings resolved

- **Unbound helper:** `readSnapshot` is now defined at test lines 106–110. On the inspected baseline, `LocalContractAdapter` construction schedules no timers; `getSnapshot` awaits `wait()`, which schedules one timer (`localAdapter.ts:140-144,453-455`). Thus `runNext()` in the helper fires the pending call's controlled latency callback before adopting the promise. This removes the repair 2 static compile blocker and does not await before advancing its timer.
- **Dynamic future boundary:** both future-floor tests derive H from the final snapshot (`:223-245` and `:165-187`). The ordinary test suppresses exact H+1 and receives H+2; the leading-zero test uses a numerically equivalent padded floor, suppresses equality, and receives the next allocation. The expected cursor width/epoch text are derived from the captured cursor.
- **Exact queue boundary:** the 64/65 scenario selectively drains the healthy subscriber and explicitly checks zero slow-subscriber errors before event 65 (`:251-289`). A separate test queues exactly 64 events without draining, then compares the drained cursor list in order and checks no error (`:291-309`). Event 65 then disposes only the backlogged subscriber, while the healthy subscriber receives the event and continues (`:271-284`). The throwing-`onError` scenario also asserts zero calls after 64 before event 65 (`:369-396`). These tests now cover both pending FIFO acceptance and isolated overflow.
- **Frontier atomicity:** the test helper always updates final snapshot and trajectory point and updates `ordinaryFrontiers` only when the optional future private map exists (`:83-100`). The ceiling and progress tests make detached copies of frontier fields (`:398-447`), so in-place mutation cannot be hidden by aliasing. History arrays are independently cloned and compared.
- **Public start and settlement timer:** the settlement test reads a real initial snapshot, selects an advertised `EXECUTE_ITEM`, invokes public `dispatchIntent`, fires its captured latency handle before awaiting, then identifies and verifies the start-delivery timer (`:449-476`). Production dispatch appends the start snapshot before invoking `execute` (`localAdapter.ts:177-274`). The test seeds to the ceiling only after that successful start and selects the largest-delay timer from that dispatch's captured positive-delay registrations (`:478-487`); current source schedules four phase timers, completion, settling progress, and settlement in order, with settlement at the largest delay (`:347-388`). This avoids dependence on a global timer ID and does not execute unrelated timers before the targeted exhaustion assertion.
- **Record timer wording:** repair 3 correctly describes the final fixture's explicit handle selection. It does not claim a `runUntilPattern` helper.

## Correction to repair 3 record

The repair 3 record says the prior review claimed there were no explicit zero-error checks after 64. That is inaccurate. The repair 2 review explicitly credited `overflowErrors` empty after event 64 (`localAdapter.cursorBounds.test.ts:256-258` in the then-reviewed fixture) and `onErrorCalls === 0` after 64 (`:353-354`). The new fixture retains both checks and adds the separate exact-64 slow-queue drain. This correction is recorded here without editing the immutable repair 2 review or repair 3 record.

## Test and assertion audit

1. Constructor seed agreement compares the last snapshot and point; its deliberately mutated shared trajectory cursor is restored in `finally` (`:125-139`).
2. Template commit checks its matching seed and a subsequent allocation; the subscription and append both use the returned committed case ID (`:141-163`). The private `commitCase` helper exercises a valid committed seed. Current production derives both cursors from the same built snapshot (`localAdapter.ts:313-340`), so a mismatched committed-seed path is not injectable without a production seam; the fixture makes no claim to cover that impossible baseline path.
3. Leading-zero numeric equality, invalid decimal/safe-range/wrong-epoch/ceiling floors, and valid future boundary behavior have controlled timer drains and intended assertions (`:165-249`). Invalid floors each check one error, no received event after a later append, and safe unsubscribe.
4. Overflow, exact-64 FIFO, reentrant FIFO, self-unsubscribe, throwing `onEvent`, and throwing `onError` each have independent assertions (`:251-396`). The handle rule in the dual-subscriber tests follows registration order: the slow listener is registered first, the healthy listener second, and each latest handle is therefore the healthy drain. It does not inspect production subscriber internals.
5. Snapshot exhaustion verifies the accepted ceiling ordinal, failed next allocation, unchanged snapshot/point history, detached frontier, and last accepted cursor (`:398-418`). Progress exhaustion checks one error, no event, unchanged histories and frontier, and no cursor advance (`:420-447`). Settlement exhaustion verifies a normal start first, then checks no thrown timer callback, one error, no side event, no partial history, and the retained ceiling (`:449-492`).
6. Unsubscribe cancels a queued drain; scheduling failure is injected after commit and must be reported once without later callback delivery (`:494-525`). Current `append` commits snapshot and point before `emit` schedules delivery (`localAdapter.ts:401-435`), so this is aimed at the intended post-commit failure behavior.

All installed timer globals are restored in test-level `finally` blocks; the shared trajectory mutation is restored likewise. The fixture helper caps scheduled callbacks and `runAll` iterations (`:8-55`). No fixture mutation or timer leak is apparent from source. The optional `ordinaryFrontiers` seam supports the root-selected future map while the baseline without that map can still reach intended semantic RED assertions. In particular, on the current baseline settlement append succeeds and the assertion for its missing exhaustion error fails before the later optional-frontier expectation; the absent map is not a setup failure.

## Material scope notes

This file is a bounds/validation supplement to the frozen `localAdapter.cursorOrder.test.ts`, not a replacement for the whole revision 4 matrix. The frozen test continues to contain omitted and supplied floor behavior, public execution ordering, retained-history checks, and case-local delivery. The repair stayed test-only and did not change the frozen file, production adapter, public seam, dependencies, or configuration. The source review establishes plausible deterministic RED distinctions from the inspected baseline, but only the root-owned focused Bun attempt can establish whether the file compiles and how its assertions execute.

## Disposition

Source review is approved for one root-owned focused runtime RED attempt. Preserve the frozen files and report the actual command/result separately; do not treat this review as evidence of compilation or passing behavior. No source/test execution was performed here.
