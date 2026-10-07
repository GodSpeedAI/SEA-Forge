# Local cursor bounds fixture — fresh repair 3 record

Date: 2026-10-06. Source-only TESTONLY repair; no runtime or compiler claim.

## Repair instructions

The fresh-repair instructions for this builder were:

> Fresh repair of current UI bounds 3d3e1da candidate by DIFFERENT builder from current author. Source only, no Bun/test/compiler/Git. Read ORIGINAL bounds assignment, first independent rejection, repair2 record and latest local-cursor-bounds-fresh-repair2-independent-source-review-oct06.md (70876f88) under .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/. Address ALL findings: undefined readSnapshot local controlled timer helper/inline, derive ordinary future floor from captured current head, direct drain of exactly64 slow queued events proving FIFO, progress exhaustion allocator+history immutability. Correct repair record timer explanation in NEW record (not old). Preserve preedit current3d exact bytes in NEW native patch backup. Frozen production d8b0/order373a/conformanceedf unchanged. No public seam, dependency, config, test-only production hooks. Record original full instructions + reviewer findings + resulting exact hashes/deviations immutable NEW record. Graft first/read applicable instructions. Await independent source review before any runtime.

The underlying original proposal assignment is `local-subscription-ordered-cursor-proposal-original-assignment-oct06.md` (SHA-256 `6028c2e99831f67a5ab3b974e955fd9b1fc6d7e1ac7c012ef93944bcd409d4c8`). The original revision 4 builder assignment is `local-subscription-ordered-cursor-revision4-original-assignment-oct06.md` (SHA-256 `0af4db3bfee9d17621eb8f6a1e1f8ebb18c11b456b2f6140accb2a290b31d7b8`). The test-review assignment is `local-subscription-ordered-cursor-revision4-tests-independent-review-assignment-oct06.md` (SHA-256 `3eabff31dedc360898729de0b46469a0892934fbe86d06d7b47d4f9773c467bb`).

The test-review assignment says to compare the fixture against the original assignment, revision 4 proposal/review/erratum, frozen `cursorOrder` test, and actual adapter/timer helpers; inspect all assertions and private accesses; establish deterministic meaningful baseline REDs; cover parsing, safe floors, seed behavior, 64/65 FIFO and continuity, callback isolation, cancellation, and post-commit schedule failure; verify timer and shared-fixture restoration; preserve the frozen test; and make no runtime or compile claim. It forbids editing production, frozen tests, or any other source/test/config file. Root retains compiler and runtime gates.

## Reviews read and findings

The first independent rejection is `local-subscription-ordered-cursor-revision4-tests-independent-source-review-oct06.md` (SHA-256 `c80c591af140180431e9ef505a041d64a359841de9a4556cf311cce08fdb5153`). It identified the earlier leading-zero no-timer setup, both subscribers overflowing at 65, missing active reentrant FIFO assertion, missing throwing-`onError` coverage, and absent allocator exhaustion coverage. Those source findings motivated the prior repair; this record does not amend that review.

The repair 2 record is `local-cursor-bounds-fresh-repair2-record-oct06.md` (SHA-256 `086b1643306a2ebf84317c29a638a5bbf1b43d5addfbfad66e00c9e4deb0e518`). The latest independent rejection is `local-cursor-bounds-fresh-repair2-independent-source-review-oct06.md` (SHA-256 `70876f88463af586b79fffac4777d98de6a04b7a33a298c148a4afc89b249925`); reviewed fixture SHA-256 was `3d3e1da51057fafde43508b76ef4e8a33d2f4f91324924ec2abddd53db62f576`. Its findings were:

1. `readSnapshot(adapter, timers)` was referenced but undefined; this was a static compile blocker.
2. The ordinary future-floor test hardcoded `.0007/.0008` instead of deriving H from the current head.
3. No dedicated case directly drained the slow subscriber's pending 64 events and asserted exact FIFO delivery.
4. Progress exhaustion checked error/no event but not retained snapshot, point, and allocator frontier immutability.
5. The review characterized the repair record's timer wording as inconsistent with the fixture.

The reviewed 3d3e1da source already contains explicit zero-overflow assertions after 64 in both overflow tests (`overflowErrors` is empty; throwing `onError` call count is zero). The source review says these were absent. This repair retains those checks and adds the separate slow-subscriber 64-event drain test, so the accepted boundary is directly established regardless of that discrepancy.

## Changes and timer behavior

Only the assigned bounds test was edited. Added a local `readSnapshot` helper: call `getSnapshot`, fire the installed manual timer with `runNext`, then return the promise. The ordinary future-floor test captures the current final snapshot cursor and derives H+1/H+2 while preserving the fixture's epoch text and minimum sequence width. Added a slow-only case that queues 64 events before draining, then checks the exact cursor order and zero errors. Progress exhaustion now snapshots the histories and copies the optional allocator-frontier fields before calling progress; it checks the cursor-bearing history, last cursor, and frontier remain unchanged after one error and no event.

The actual timer selection remains explicit and handle-based. The settlement scenario reads the initial snapshot with the new helper, invokes public `dispatchIntent`, fires the captured dispatch latency handle, records `scheduledAfter(dispatchWaitHandle)`, fires the zero-delay start delivery, and clears that start event from the assertion list. It then seeds the test case to the ceiling and chooses the execution timer with greatest recorded delay from that captured registration set; `runHandle(selected.handle)` fires it. There is no `runUntilPattern` helper and no global hardcoded settlement handle. This paragraph is the corrected timer-control explanation; earlier repair records remain immutable.

The optional private `ordinaryFrontiers` map remains a conditional test-only white-box seam. The helper always updates the final snapshot and point cursor, and replaces the map entry with `{ epochText, epoch, sequence }` only if the future private map exists. No export, option, dependency, public interface, config, or production hook was added. Template commitment has a valid matching-seed path; an invalid committed seed remains unreachable through the current deterministic private commit path and is not claimed covered.

## Byte identities and verification

Before editing, the exact pre-edit candidate bytes were copied to `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorBounds.test.ts.3d3e1da.original.raw`; the copy hash matches the reviewed candidate: `3d3e1da51057fafde43508b76ef4e8a33d2f4f91324924ec2abddd53db62f576`.

| Artifact | SHA-256 after repair |
|---|---|
| Repaired `localAdapter.cursorBounds.test.ts` | `134ba6162b60eddcf46b9b92347a8da4feda8cca1a26b68ae31edb1922285e8a` |
| Preserved pre-edit `.3d3e1da.original.raw` | `3d3e1da51057fafde43508b76ef4e8a33d2f4f91324924ec2abddd53db62f576` |
| Frozen `localAdapter.cursorOrder.test.ts` | `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1` |
| Production `localAdapter.ts` | `d8b0feb02e489587672968c7450373f000ba02e3fd3d0d509888ca1e0f595f34` |
| Shared `caseworkPortConformance.ts` | `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c` |

No Bun, tests, typecheck, compiler, build, Git, or network command was run. The updated fixture is not independently approved and has no runtime RED result. The next step is a different independent source review of this fixture, preserved original bytes, and this new record. Root approval remains required before any runtime attempt.
