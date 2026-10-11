# Local cursor bounds fresh repair 2: independent source review

**Date:** 2026-10-06  
**Verdict:** **REJECT — do not run the focused Bun test yet.** The fixture has a statically identifiable unresolved helper at line 413. Fix that compile blocker and the listed source-level coverage gaps, then request another source review before the single authorized runtime attempt.

## Scope and evidence

This is a source-only review of `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorBounds.test.ts`, compared with the original ordered-cursor proposal/revision 4, the prior rejection `local-cursor-bounds-fresh-repair-independent-source-review-oct06.md`, and the bounded repair instructions recorded in `local-cursor-bounds-fresh-repair2-record-oct06.md`. No test, compiler, typecheck, Git, or build command was run.

SHA-256 identities checked:

| Artifact | SHA-256 |
| --- | --- |
| reviewed `localAdapter.cursorBounds.test.ts` | `3d3e1da51057fafde43508b76ef4e8a33d2f4f91324924ec2abddd53db62f576` |
| frozen `localAdapter.cursorOrder.test.ts` | `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1` |
| production `localAdapter.ts` | `d8b0feb02e489587672968c7450373f000ba02e3fd3d0d509888ca1e0f595f34` |
| conformance fixture | `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c` |
| preserved `.3a801b86.original.raw` | `3a801b86b5e39457f6b057777bbd29ea535634005eb8bb0211d2d888aaf14e11` |
| preserved `.884987.original.raw` | `884987fb78dbe869c11addba3bc9a6e76101d41ce4af14f50156a47d5e31bc2f` |

The prior fixture hashes and the frozen production/test/conformance files remain unchanged. No production source or prior review record was modified.

## Blocking defect

`localAdapter.cursorBounds.test.ts:413` calls `readSnapshot(adapter, timers)`, but this file neither declares nor imports `readSnapshot`. The only matching helper found is local to the separate `cursorOrder.test.ts`; it is not exported and cannot satisfy this reference. This is an unresolved identifier and a compile blocker by static inspection. Add a local helper or inline the controlled `getSnapshot` setup before any runtime attempt.

## Repaired findings confirmed

- The template case test now subscribes with the actual committed `caseId` and appends to that same case (`:135-153`), fixing the prior wrong-case setup.
- The zero-padded future floor is derived from the live head cursor and checks equality suppression followed by delivery of the next ordinal (`:159-185`).
- Both overflow scenarios drain only the healthy subscriber while retaining the first subscriber's pending work for events 1–64; each explicitly asserts zero errors after 64 and triggers isolated overflow on event 65 (`:239-277`, `:337-364`).
- `seedCursorForBoundary` updates the snapshot, point, and optional root-selected `ordinaryFrontiers` seam (`:83-100`); the allocation test compares a detached frontier copy rather than an alias (`:366-385`). This supports the baseline without that future private map and the specified future implementation with it.
- The settlement scenario now starts a real public `EXECUTE_ITEM` through `dispatchIntent`, fires the latency timer before awaiting, identifies the initial snapshot delivery and a positive-delay execution timer, verifies the start cursor, then seeds the ceiling and fires the maximum-delay settlement timer (`:409-448`). This uses the actual action timer path instead of hand-calling settlement. Source confirms `dispatchIntent` appends the start snapshot before invoking `execute` (`localAdapter.ts:177-274`), and `execute` schedules execution phases and settlement (`:347-388`). The maximum-delay selection avoids a brittle fixed global timer number.
- Timer helpers are bounded and restored in `finally` across the test cases (`:8-55` and test-level `finally` blocks). The mutation of `NORTHSTAR_TRAJECTORY` is also restored in `finally` (`:125-132`).

## Remaining source-level issues

1. **The separate ordinary future-floor test still hardcodes the fixture's current `1.0000000007` / `1.0000000008` values** (`:217-233`). The new leading-zero test derives its floor from the head, but this second core floor test will silently become invalid if the fixture head changes. Derive its head `H` and test `H+1`/`H+2`, as required by the repair instructions.
2. **The 64/65 test proves healthy-watcher FIFO and isolated slow-watcher overflow, but never drains the slow watcher's actual 64-item backlog.** Its `overflowEvents` stays empty through the 65th event and the watcher is then disposed (`:247-271`). Thus the test name's “64 pending events stay FIFO” is not directly asserted for the pending queue; add an exact-64 case that drains the slow subscriber and compares the full ordered sequence, while retaining the isolated 65th-overflow case.
3. **Progress exhaustion lacks the frontier atomicity assertion.** It asserts one error and no event (`:388-407`) but does not snapshot/compare the frontier or cursor-bearing history. A broken implementation could mutate the allocator before reporting exhaustion and still pass. Capture a detached pre-call frontier plus relevant history and assert they remain unchanged.
4. **The repair record and test differ on timer-control helper wording.** The record describes a `runUntilPattern` helper, but this test uses explicit `runHandle` calls (`:419-432`). The implemented ordering is sound and avoids awaiting before firing dispatch latency; update the record language or establish that it described intent rather than the final fixture.

## Per-test assessment

The constructor seed agreement test compares final snapshot and point and restores its deliberate fixture mutation (`:119-133`). The malformed/unsafe/wrong-epoch/over-ceiling table checks one error, no event, and safe disposal for each input (`:188-215`). The leading-zero floor test is based on the actual head (`:159-185`). Reentrant delivery checks FIFO, and self-disposal independently prevents a queued callback (`:279-315`). Throwing `onEvent` and throwing `onError` have isolated healthy-subscriber assertions (`:317-364`). Ceiling allocation checks no snapshot/point mutation and conditionally checks the future allocator map (`:366-385`). Timer scheduling failure is exercised after commit, with one error and no later callback (`:454-485`).

The settlement setup's identified target is selected from the initial snapshot's actual advertised `EXECUTE_ITEM` action (`:413-420`); production source appends the start event and schedules execution via that public dispatch path. Its remaining blocker is the missing helper, not the chosen settlement sequence.

## Disposition

Do not run Bun yet. First resolve the unbound `readSnapshot`, derive the ordinary future-floor boundary from the live head, directly prove FIFO for a drained 64-event backlog, and assert allocator/history immutability on progress exhaustion. Preserve the frozen cursor-order and production/conformance files. Then provide the corrected fixture for source re-review; only after approval should the root run the one focused RED attempt.
