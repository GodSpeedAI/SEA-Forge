# Independent review: focused ordinary-cursor attempt 01

Date: 2026-10-07  
Disposition: **REJECT this attempt as verification evidence; the observed Bun failure is a test-harness contradiction, not evidence of a production unsubscribe defect. A separate captured typecheck also has two test-only narrowing errors. No implementation approval or new runtime authorization follows.**

## Assignment and review scope

The complete implementation assignment and result are preserved in:

| Record | SHA-256 |
|---|---|
| `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/local-ordinary-cursor-implementation-original-record-oct07.md` | `21f835418a4d3380469309abc0f6d4bee421c6d9723e5870bed8f18364d9b019` |
| `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/local-ordinary-cursor-implementation-result-oct07.md` | `16f1936b3878526c05c8c0fbbdc2c73cdd86389acbd63524803899748d4b1334` |

The review assignment was to inspect the actual focused runtime attempt and full source/fixture against the original builder instructions, accepted revision 4 proposal, root settlement atomicity decision, hostile-value repair and its independent review, and frozen order/bounds files. It was read-only: no test rerun, edit, compiler, scanner, or Git action.

Relevant authority and evidence read:

- Revision 4 proposal SHA `6d7baafd41a098cc24556b32ea222c999ffe4ea116998740339acd2086131873`; its independent review SHA `893ddd2e2e75ed63b14689fa19ddbbb8f00d6e11e5725b523e57922eaba359ab`.
- Root two-ordinal settlement decision at `local-settlement-two-ordinal-atomicity-root-decision-oct07.md`; the separate builder decision SHA is `af9732f2061ee94141e069fc62b9a1f2ba89b229ee35edeb75b80d9f5cd32077`.
- Hostile-value repair record SHA `a42853c232e447eeb17e9c21bf51ad3438dfd76afcb934ae991409228b27695f`; repair result SHA `5331dd593115e06afdb01a1f437685e8674887312c21676fe274b26d6aefc26f`; independent source review SHA `b7c8551f95c4e61faacc1a2c88f2c1759f8a7b67515f209c4e3331dbd422cb31`.
- Root-provided focused attempt artifacts: preflight SHA `e97361d29e422f65e91304dfeade267e0db0f7d517e5fefeedec617932e0af7e`, Bun output SHA `b6ebdfdd0eb01a85c89c470ba35f71321b97234046249dc1892ff4bc263bdd8a`, exit SHA `cf205dbb8cea84897b488abcc281bf96698d5e94b1096b16657b4caba9082a22`.
- Root-provided typecheck artifacts: preflight SHA `d75e52154d4b40e4b187117103dad44ae14c05cca170cf7a8e3dedcc6022ab13`, output SHA `9adbe09f7c39918b7d375e11d7c6d9e47bcc0c6c28c96b0efb1ad96ad8e62f78`, exit SHA `5f8bd21203fdbe78b3339b6c8102f5a67cea53316e7b8797aa7aea63e498d985`.

## Candidate identities

| Artifact | SHA-256 |
|---|---|
| Current `localAdapter.ts` | `19c767cee3da406f5729a29177f360638d2b6ac3830294126f564c5d857b4700` |
| Frozen `localAdapter.cursorOrder.test.ts` | `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1` |
| Frozen `localAdapter.cursorBounds.test.ts` | `134ba6162b60eddcf46b9b92347a8da4feda8cca1a26b68ae31edb1922285e8a` |
| Settlement atomicity test | `35f2a8c93a5213fc22ac0a6fc4ee2963e330eebc3c951125e6d242c448775819` |
| Hostile-value test | `31f8a71c2fa550ef0ea6f22259e25c48742a836fea85e52616d6da3356913920` |

The preflight's five listed identities match the present files. The attempt contains four test files, 23 tests, 204 expectations, and reports 22 passing / 1 failing in 222 ms. The captured typecheck exits 2 with exactly two diagnostics.

## Runtime failure classification

The reported exception is `Error: no pending timer` in the manual harness `runNext()` at `localAdapter.cursorOrder.test.ts:27-31`, reached from test line 279. Immediately before it, the test appends one Northstar event, confirms the cursor, then calls `stopFirst()` at lines 276-278. `stopFirst` is the unsubscribe closure returned for the Northstar subscriber.

That setup leaves no runnable timer. Revision 4 requires unsubscribe to mark the subscriber inactive, remove it, cancel its drain timer, and empty its queue (`local-subscription-ordered-cursor-concrete-proposal-revision4-oct06.md:184-193`). The second, Template-case subscriber has no queued event, so it has no timer either. The order-test timer harness filters cancelled timers before `runNext()` selects one (`cursorOrder.test.ts:27-31`). Consequently line 279 throws by design after successful cancellation. The local adapter's current `disposeSubscriber` calls `clearTimeout`, marks inactive, clears queue, and removes the subscriber (`localAdapter.ts:785-801`).

The frozen bounds test independently encodes the same accepted behavior: append queued event, unsubscribe, assert `timers.hasPending()` is false, run all remaining timers, then assert no event/error (`cursorBounds.test.ts:494-506`). This is a direct source-level agreement with the revision 4 contract. The failing order-test fixture instead asks `runNext()` to fire a cancelled timer when no other active subscriber has queued work. **No production unsubscribe defect is demonstrated by this exception.**

The failure occurs before assertions at `cursorOrder.test.ts:280-285`; these were not reached in attempt 01:

- no delivery to either subscriber after the unsubscribe boundary;
- the independent Template case allocates the expected next cursor;
- its later genuine event is delivered to the still-active Template subscriber.

Thus those behaviors are not established by this attempt, even though the fixture source intends to check them. Its `finally` restores manual timer globals (`:287-289`); normal stop closures are after the failing line and so are not reached. The remaining adapter/subscriber objects are local to this test; no production or cross-test leak is shown in the captured output, but the test does not take its normal explicit unsubscribe path on failure.

Smallest fixture-level correction for the released line is to stop trying to run a canceled timer. The helper already has `hasPending()`; a direct assertion that no timer remains is consistent with the accepted contract and the frozen bounds test. Alternatively the existing bounded `drain(timers)` helper can be awaited from this already-async test. Root retains authority to release a narrowly scoped fixture correction and require the exact `373a` preimage be preserved first. This review made no edit and does not release a test change.

## Captured typecheck findings

The separately captured `bun run typecheck` output reports exactly:

- `localAdapter.cursorBounds.test.ts:475`: `response.new_cursor` has type `string | undefined` when compared with `startSnapshot!.cursor`.
- `localAdapter.settlementAtomicity.test.ts:99`: same optional `new_cursor` comparison.

These are test assertions, not production algorithm diagnostics. The existing response contract permits the cursor field to be absent, while these test paths require it after an asserted successful dispatch. Preserve that logical requirement during any authorized fixture repair by asserting cursor presence before equality (for example, a branch/throw that narrows the value, or `toBeDefined()` followed by a non-null assertion in the equality). Do not cast away the absence possibility or remove the cursor equality assertion. No file was changed here.

## Other attempt results and material deviations

All other printed cursor-order cases pass in the captured run. All 14 cursor-bounds tests pass, including the explicit unsubscribe timer cancellation case, and the hostile-value and two settlement-atomicity tests pass. These are the only runtime statements supported by this capture. Since the Bun command exits 1 and the test above stops before its own assertions, this is not a focused suite GREEN and not a valid product-behavior RED for unsubscribe. The tsc attempt separately remains red on the two optional-field assertions.

Against the original implementation assignment/result, this review found no source identity mismatch and no changed frozen `373a`/`134ba` inputs. The complete result documents the accepted private cursor behavior, settlement atomicity clarification, hostile-value hardening, scoped tests, and explicitly pending release. The observed deviation in this attempt is fixture setup at `cursorOrder.test.ts:279`, plus the two type-only fixture errors. No algorithm defect is evidenced; no full matrix, all UI gates, or production integration is approved by this assessment.

## Review boundary and next step

Disposition is limited to classification: attempt 01 failed because of an invalid controlled-timer expectation, not because the unsubscribe implementation failed to cancel delivery. Correct the single released test setup without weakening its behavior assertions, preserve the exact frozen test preimage first, separately narrow both optional `new_cursor` assertions, then require fresh root-owned focused runtime/typecheck evidence. No changes, reruns, compiler actions, gates, or Git commands were performed in this review.

Graft saved ~10,679 tokens (<$0.01) this turn (1 query).
