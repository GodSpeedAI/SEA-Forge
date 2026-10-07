# Local cursor ordering Phase 1 — independent RED2 review

Date: 2026-10-06  
Verdict: **Accept the captured run as bounded negative evidence against the frozen pre-fix adapter.** This supports only the four observed behaviors below; it releases no ordering implementation and makes no full UI, canonical-gate, or integration claim.

## Reviewed inputs and frozen identities

- Original Phase 1 assignment: `local-cursor-order-phase1-original-assignment-oct06.md`, SHA-256 `42f13a986ef8dbc8e46899458f08e7f40c9e7c30765b5ba389e94ee8278b9268`.
- Fixture repair assignment: `local-cursor-order-fixture-repair-original-assignment-oct06.md`, SHA-256 `f01bfe20e2ebc48fe94b5c4b4107d833fe6aa352ea06cd1c875f3272459c1cca`.
- Previous RED1 rejection: `local-cursor-order-phase1-actual-red-independent-review-oct06.md`, SHA-256 `c2c65c25af3679a9a392b89fa4dcdd86bb863e36a810be47ae5aac463f2df531`.
- Current fixture source review: `local-cursor-order-phase1-freshrepair373a-independent-source-review-oct06.md`. It records the one-line bounded drain repair and source-only approval; that approval is not runtime evidence.
- Root's fixture source acceptance: `local-cursor-order-phase1-root-source-acceptance-oct06.md`.
- Test `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorOrder.test.ts`: SHA-256 `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1`.
- Frozen production `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts`: SHA-256 `d8b0feb02e489587672968c7450373f000ba02e3fd3d0d509888ca1e0f595f34`.
- Frozen conformance `apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts`: SHA-256 `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c`.

The source and test hashes match the source-reviewed freeze. Production and conformance were not changed for this RED2 run.

## Capture integrity and observed run

I independently hashed and compared all three archived records to their actual `/tmp` originals; every `cmp` exited 0:

| Artifact | `/tmp` original and archived copy | SHA-256 |
|---|---|---|
| Preflight | `/tmp/sea-casework-20261006-ui-cursor-order-red02-preflight.raw` and `ui-cursor-order-red02-preflight.raw` | `d02a284ba2fc69ae9a90e03b7c691feac7752beae67d4d6efb19a0754a608872` |
| Bun output | `/tmp/sea-casework-20261006-ui-cursor-order-red02-bun-output.raw` and `ui-cursor-order-red02-bun-output.raw` | `19781fc438013e4ec32af891c7b228408a653093e79c02ce470173084671cc2e` |
| Exit | `/tmp/sea-casework-20261006-ui-cursor-order-red02-exit.raw` and `ui-cursor-order-red02-exit.raw` | `cf205dbb8cea84897b488abcc281bf96698d5e94b1096b16657b4caba9082a22` |

Archived paths are under `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/`. The preflight is timestamped `2026-10-06T21:08:19Z` and records 2,985,750,528 available RAM bytes, 1,803,755,520 free swap bytes, and 1,193,013,248 bytes available on `/tmp`, plus the three frozen source hashes. The joined focused Bun run returned exit 1 and reports **2 passed, 4 failed, 24 expect calls, six tests, 340 ms**. There is no `no pending timer` error in this output.

## The four observed semantic failures

1. **Future cursor floor is ignored.** In `localAdapter.cursorOrder.test.ts:114-139`, the test reads the actual head, computes a valid same-epoch next cursor, subscribes with that future floor, and appends exactly at that cursor. The exact cursor precondition passes at line 127; after draining, line 129 fails because the event was delivered to the listener instead of suppressed. This distinguishes current behavior from the requested floor semantics. The later append above the floor and its expected H+2 delivery at lines 131-134 are **unreached** because the boundary assertion fails first. In production, `subscribeEvents` names the argument `_since` and ignores it while registering the listener (`localAdapter.ts:168-174`); `emit` queues the event for currently registered listeners (`:433-435`).

2. **Settlement side-event cursor resolves a retained snapshot.** The public execution test reaches successful dispatch, drains the execution, observes progress, locates settlement, and confirms a snapshot immediately precedes the settlement side event (`test.ts:167-181`). It then calls `getSnapshotAt` with the settlement event cursor; line 189 fails because the promise resolves where rejection is required. Production appends the settlement snapshot and emits the settlement event using the same `snap.cursor` (`localAdapter.ts:378-388`), while `getSnapshotAt` searches retained snapshots by exact cursor (`:146-151`). The later all-event cursor uniqueness and strict-order loop, explicit side-event-greater-than-snapshot assertion, and subsequent snapshot assertions (`test.ts:190-207`) are **unreached**.

3. **Progress cursor resolves a retained snapshot.** The history test reaches and passes the progress event check and before/after trajectory and snapshot equality checks (`test.ts:214-245`). It then appends one immutable history point and reaches the final trajectory assertions; the point count and new tail cursor checks pass (`:247-253`). Exact lookup using the progress event cursor resolves rather than rejects, so the assertion at line 256 fails. Production progress borrows the latest retained snapshot cursor (`localAdapter.ts:391-399`), which the exact lookup can resolve (`:146-151`). The old RED1 fixture's post-unsubscribe `runNext()` setup error is absent: line 248 now uses bounded `await drain(timers)`, so all history and progress-cursor assertions described above execute.

4. **Unsubscribe does not cancel an already queued callback.** The case-local test obtains both heads, subscribes, appends a Northstar event, checks the next Northstar cursor, unsubscribes, and drains that queued timer (`test.ts:262-279`). At line 280 the first listener already contains the event, so the expectation of no delivery fails. Production `emit` closes over each listener function in `setTimeout(() => fn(e), 0)` and the callback does not check whether the listener was subsequently removed (`localAdapter.ts:433-435`). The empty second-listener assertion and the later template-case append, exact template allocator increment, and delivery assertion (`test.ts:281-285`) are **unreached**.

The two passing tests are the omitted-boundary chronology test (`:85-112`) and supplied revision H+1/H+2 chronology test (`:141-165`). These passes are supplementary: an event emitted before a late listener is registered was never queued for that listener, so those tests alone do not prove floor filtering. RED2's same-epoch future-floor test is the direct floor failure.

## Disposition and limits

The corrected fixture provides a valid focused RED for four distinct pre-fix behaviors. Its one-line bounded drain repair removed the prior no-pending-timer setup failure, allowing the history immutability and one-point assertions to run. The output contains four expected semantic failures and no harness/setup failure.

This evidence does **not** show that the later cursor uniqueness/order assertions or second-case allocator checks pass; they were stopped by earlier failures as listed above. It does not cover parser/invalid-input behavior, FIFO64/error handling, sequence ceiling/exhaustion, constructor validation, the full UI suite, a canonical check, or manager/live-system integration. It provides no positive evidence for a cursor algorithm, implementation correctness, source GREEN, or full T09 completion. Root must separately decide which additional bounded fixtures and gates are required before any implementation release.

No test or compiler command was run by this reviewer. No production/test source, Git state, scanner, Graft build, or network operation was changed or invoked for this review.
