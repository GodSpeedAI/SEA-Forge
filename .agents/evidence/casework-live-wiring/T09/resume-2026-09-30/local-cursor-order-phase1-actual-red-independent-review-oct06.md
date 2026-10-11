# Local cursor-order Phase 1 — independent actual RED review

Date: 2026-10-06  
Verdict: **REJECT the captured run as sufficient actual RED evidence for releasing the ordering algorithm.** It demonstrates three real behaviors the approved design must fix, but one of the four failures is a fixture setup error that prevents required history assertions from running. No implementation or source release follows from this review.

## Frozen source, assignments, and output evidence

- Original fixture assignment `local-cursor-order-phase1-original-assignment-oct06.md`: SHA-256 `42f13a986ef8dbc8e46899458f08e7f40c9e7c30765b5ba389e94ee8278b9268`.
- Fixture repair assignment `local-cursor-order-fixture-repair-original-assignment-oct06.md`: SHA-256 `f01bfe20e2ebc48fe94b5c4b4107d833fe6aa352ea06cd1c875f3272459c1cca`.
- First independent source rejection `local-cursor-order-phase1-independent-review-oct06.md`: SHA-256 `022e8401f69941e5b7aeff756a472a748b8ede82c6545db605b44a3e84e66c20`.
- Repair rejection `local-cursor-order-phase1-repair-independent-review-oct06.md`: SHA-256 `7469415c1a0578b29206a0d504713060bb1b3d636323eb0a02d0128621e6722a`.
- Fresh source approval for a bounded RED `local-cursor-order-phase1-fresh-repair-independent-review-oct06.md`: SHA-256 `1a3e91e51b0e0f36b6ee05ef9ef094b6527f01d92673ffd336b34cb9c5414be5`.
- Test `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorOrder.test.ts`: SHA-256 `c625cb812d5d67ac3428d1f97e6ba452c39acc327828b2f62dd72a4ba53f8fac`.
- Production `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts`: SHA-256 `d8b0feb02e489587672968c7450373f000ba02e3fd3d0d509888ca1e0f595f34`.
- Conformance `apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts`: SHA-256 `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c`.
- Root's immutable direct-copy artifacts compare byte-for-byte to the three actual `/tmp` originals (each `cmp` exit 0). Preflight: `ui-cursor-order-red01-preflight-direct-original-copy.raw`, SHA-256 `78dd1a15463439e7566a7a2b7ff84a3ba81bf7525840017d7134d9e72f12a7c3`; at 20:56:29Z, MemAvailable 3,237,019,648 bytes and SwapFree 1,685,548,864 bytes. Bun output: `ui-cursor-order-red01-bun-output-direct-original-copy.raw`, SHA-256 `4d31b7e2ca43947ed546d8cd4f73310525f5dcfa57126dc771bc6fc55d249419` (56,335 bytes). Exit artifact: `ui-cursor-order-red01-exit-direct-original-copy.raw`, SHA-256 `cf205dbb8cea84897b488abcc281bf96698d5e94b1096b16657b4caba9082a22` (`exit=1`). The original `/tmp` output path is named `...go-output.raw` but contains Bun output; the repository copy labels it `bun-output`, preserving and disclosing that naming mismatch.

The Bun command ran six focused tests in session 23252, which was joined. The command wrapper returned 0 while Bun returned 1. The run reports `2 pass`, `4 fail`, `21 expect() calls`, and 965 ms.

## Three source-semantic failures

1. **Future floor is ignored.** `same-epoch future floor suppresses its boundary allocation...` fails at test line 129: the append cursor equals the valid exclusive future floor, yet the listener receives that snapshot. Current `subscribeEvents` ignores `_since` and adds the listener unconditionally (`localAdapter.ts:168-173`); append emits to active listeners (`:402-435`). The approved contract permits this same-epoch future floor and requires delivery only above it. This is a genuine semantic RED.

2. **Settlement side-event cursor resolves a retained snapshot.** The public execution test fails at line 189 because `getSnapshotAt` resolves when the fixture requires rejection for the settlement side-event cursor. Production appends the settlement snapshot and emits `settlement_recorded` with exactly `snap.cursor` (`localAdapter.ts:378-388`); `getSnapshotAt` finds the retained snapshot by exact cursor (`:146-150`). This is a genuine semantic RED for the snapshot/history distinction.

3. **Queued callback is delivered after unsubscribe.** The case-local/disposal test appends while subscribed, unsubscribes, then drains the queued timer; it fails at line 280 because the removed listener still receives the event. `emit` captured `fn` in a timer closure and never checks active membership at callback time (`localAdapter.ts:433-435`). This is a genuine semantic RED.

## Blocking fixture setup failure and reached assertions

`progress leaves retained snapshots unchanged...` fails at test line 248 with `no pending timer`. The test emits progress to a temporary listener, and its trajectory-before/after and exact-snapshot comparisons pass (`:231-245`). It then unsubscribes at line 245, appends at line 247, and calls `runNext()` at line 248. No listeners remain, so `emit` schedules no callback (`localAdapter.ts:433-435`); the helper correctly throws when the queue is empty (`test.ts:27-35`). This is a harness/setup failure, not evidence of a production ordering defect. It stops the required one-point-after-append and progress-cursor `getSnapshotAt` assertions (`:249-256`), so those assertions are unobserved.

The public execution test's first failure at line 189 also stops all later cursor uniqueness, strict epoch/sequence ordering, side-event-greater-than-snapshot, and subsequent-snapshot assertions (`:190-207`). Those requirements exist in the fixture but this run did not exercise them. The disposal test fails before its template-case event assertion at line 285; exact second-case allocator progression/routing is therefore unreached. The raw output, not the earlier static prediction, controls these boundaries.

## Passing tests and held assertions

The undefined-boundary H+1/H+2 test passes, as does the supplied-revision H+1/H+2 test. In each, the subscriber was registered after the preexisting event was emitted, so current listener-set behavior naturally does not queue that event for the new subscriber; these passes are chronology coverage, not proof the old implementation honors a resume floor. The new future-floor test supplies the real floor RED.

The public execution test also reaches its exact `getSnapshotAt` check, where it exposes settlement cursor aliasing; it does not reach the later all-event order checks. The history test reaches and passes its immutability comparisons for the preexisting trajectory/snapshots after a progress event, but not the subsequent one-point and progress-cursor assertions. No claim is made for parser/invalid-input, FIFO64/error, ceiling/exhaustion, constructor validation, or `execution_observation`, which remain outside this focused unit.

## Disposition

The output is a valid joined Bun run and confirms the three semantic failures above, but the extra `no pending timer` failure is a fixture defect and required cursor-order assertions remain unreached. Therefore the overall focused UI RED fixture is **REJECTED for algorithm-release purposes**. A different builder should repair only the bounded fixture setup, followed by a new independent source review and a fresh actual RED run. No ordering algorithm or broader UI release is authorized here.

This review only inspected the frozen files and archived run. It ran no Bun test, compiler, scanner, Graft build, Git command, network request, or source edit.
