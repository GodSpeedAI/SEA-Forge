# Independent source review: private local ordinary cursor implementation

Date: 2026-10-07  
Scope: source and test fixture review only; no Bun, compiler, typecheck, runtime, scanner, or Git command.  
**Verdict: REJECT source approval pending one callback-isolation defect.** This is not a runtime result or production approval.

## Reviewed authorization and identities

I read the complete implementation assignment in `local-ordinary-cursor-implementation-record-oct07.md`, including its full quoted original instructions and frozen-input identities; the separate root two-ordinal settlement decision; the accepted revision 4 proposal, independent review and citation erratum; original phase 1, revision 2/3/4, fixture repair and bounds-test review assignments; order RED2 review/verdict; bounds RED01 verdict and root acceptance; and the current complete adapter, new settlement tests, frozen order/bounds tests, existing local tests, port types and conformance source.

| Artifact | SHA-256 |
| --- | --- |
| reviewed `localAdapter.ts` | `828069b6c09d07c728778356c8921bfa553201f7882ffbbdfe83c86d96c21074` |
| reviewed `localAdapter.settlementAtomicity.test.ts` | `35f2a8c93a5213fc22ac0a6fc4ee2963e330eebc3c951125e6d242c448775819` |
| implementation result record | `16f1936b3878526c05c8c0fbbdc2c73cdd86389acbd63524803899748d4b1334` |
| original implementation assignment record | `21f835418a4d3380469309abc0f6d4bee421c6d9723e5870bed8f18364d9b019` |
| root settlement decision | `af9732f2061ee94141e069fc62b9a1f2ba89b229ee35edeb75b80d9f5cd32077` |
| accepted proposal revision 4 | `6d7baafd41a098cc24556b32ea222c999ffe4ea116998740339acd2086131873` |
| revision 4 independent review | `893ddd2e2e75ed63b14689fa19ddbbb8f00d6e11e5725b523e57922eaba359ab` |
| revision 4 citation erratum | `95ccd15e5fb00ace6a478204be1745d6ab693c6821f9f88f285ed81105018c6f` |
| frozen order test | `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1` |
| frozen bounds test | `134ba6162b60eddcf46b9b92347a8da4feda8cca1a26b68ae31edb1922285e8a` |
| frozen conformance | `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c` |
| existing local test | `07dbef23421395a251d6526febdab30b0ec51d0c87dc68f7a57d0de1c0fcc35b` |

The four frozen artifact hashes match the implementation record. Graft was used first, followed by direct source reads and line-level cross-checks. The implementation assignment narrows source changes to the private adapter and a new focused test, so the proposal's separately described shared-conformance branch adjustment was not made; that is consistent with the later bounded assignment and exact unchanged `edf8...` identity, not an unexplained edit omission. Root's settlement clarification is explicit and is applied as a stricter atomic reservation of two adjacent ordinals. No new public interface, schema, dependency, HTTP behavior, or `execution_observation` behavior appears in the reviewed changes.

## Main algorithm and supplemental tests

Except for the finding below, I found the implementation aligned with the accepted private design and root settlement decision:

* `parseLocalCursor` accepts decimal digit pairs, requires safe integers and the local sequence ceiling, and `localCursorSeed` requires exact final snapshot/trajectory string equality before case installation (`localAdapter.ts:107-115,610-614`). `epochText` is retained for formatting; the generated sequence uses ten-digit minimum padding. The template commit stages its initial snapshot, point, event and allocator installation together (`:369-448`). This preserves opaque epoch spelling while comparisons use numeric epoch/sequence.
* `allocateOrdinaryEvents` checks the previous frontier and capacity, constructs and validates candidate ordinals, calls preparation before mutation, validates all prepared events, then synchronously commits the final allocator sequence and staged history (`:627-682`). Preparation of snapshot, point, event and new history arrays is done in `prepareSnapshotAppend` (`:572-608`). Ordinary progress uses the shared allocator without retaining a revision (`:503-518`); snapshot append uses it with new immutable history (`:521-534`).
* Settlement reserves `count=2`, prepares its snapshot at the first candidate and side event at the second, then commits one allocator frontier and one new snapshot/point before publishing the ordered pair (`:536-569,664-678`). With frontier `...9999999998`, the capacity check rejects before invoking `prepare`; with `...9999999997`, the two generated cursors are consecutive and distinct. `getSnapshotAt` remains exact lookup into snapshots (`:208-213`), so the side-event cursor intentionally has no retained snapshot.
* The queue is per subscriber, capped at 64 pending events, and receives both members of a publication batch before any asynchronous callback can run (`:685-735`). The single zero-delay timer, active checks, FIFO drain, self-disposal, queue/timer clearing and `onError` containment are in `:737-810`. Scheduling errors are caught after commit, dispose only the affected subscriber, and are notified without escaping `append` (`:737-748`). The successful cursor is not rolled back after delivery failure.
* The new test uses actual public `dispatchIntent` and the actual start snapshot before seeding its private boundary; it picks the maximum-delay scheduled execution timer as the settlement callback (`settlementAtomicity.test.ts:68-101`). Its one-slot case checks exactly one error and unchanged snapshot/point arrays/frontier with no settlement events (`:104-131`). Its two-slot case checks snapshot then side-event, exact `...9999999998`/`...9999999999` cursors, one retained point, frontier at the ceiling, resolvable snapshot cursor and unresolvable side cursor (`:133-170`). These are focused supplement tests, correctly described as unrun and not baseline-RED evidence.
* The frozen order fixture covers public progress/snapshot/settlement ordering, next snapshot after side event, exact history lookups, progress retaining no snapshot/trajectory point, and per-case isolation (`localAdapter.cursorOrder.test.ts:167-285`). The bounds fixture covers parsing/floors, seed checks, queue64/65, callback behavior, ceiling and ordinary progress/settlement exhaustion, and unsubscribe/timer failure (`localAdapter.cursorBounds.test.ts:120-525`). The accepted RED records limit what those cases have proved at runtime; this source review does not convert them into GREEN results.

## Required finding

**Required — make thrown-value normalization nonthrowing and add a targeted source test before source approval.** `asError` returns an `Error` or calls `String(value)` without a guard (`localAdapter.ts:117-119`). The `onEvent` catch disposes the current subscriber and then calls `asError(error)` before notifying it (`:765-770`). JavaScript permits throwing any value; an object may implement `Symbol.toPrimitive`/`toString` that itself throws. In that case `asError` throws from inside the catch, `onError` is skipped, and the timer callback leaks a new uncaught exception. The same conversion is also used in allocation and timer-scheduling catches (`:468,680,746`). This violates revision 4's unqualified requirement to catch `onEvent` exceptions, report once, dispose only that watcher and continue other subscribers; the existing bounds test exercises only `throw new Error(...)` (`localAdapter.cursorBounds.test.ts:349-367`).

Required follow-up: normalize arbitrary thrown values without allowing conversion itself to escape (for example, contain coercion and use a fixed fallback `Error`), and add a deterministic callback test throwing a non-Error object with a throwing coercion hook. Assert the drain does not throw, the affected subscriber is disposed/notified once, and a healthy subscriber still receives its event. Apply this as a fresh, narrowly scoped builder change; this review made no source or test edit.

## Event-reference isolation: scoped risk, not an added contract gate

`publishBatch` queues the same `publication.event` object into each subscriber (`localAdapter.ts:704-720,723-734`). The canonical `StreamEvent` fields are mutable, not `readonly` (`.agents/reports/interface-contracts/typescript/types.ts:438-443`). A callback that mutates `cursor` or `payload` can therefore change what a later subscriber sees, even though the event payload was cloned away from retained history during preparation. I did not elevate this to a required finding: accepted revision 4 defines FIFO and thrown-callback isolation but does not specify mutation isolation, deep immutability, or one event-object identity per watcher. Changing this behavior should be considered explicitly in a future bounded decision; cloning after commit would itself need safe failure semantics. No such contract or fix is silently assumed here.

## Deviations, limits, and disposition

The only material algorithm clarification versus revision 4 is the later root decision that settlement is one logical operation reserving two ordinals atomically. It is implemented as required. The shared conformance correction remains intentionally out of this source assignment, with its frozen hash unchanged. The reviewer cannot approve source for gates until the thrown-value normalization gap is corrected and independently reviewed. No test/compiler/runtime/gate or production approval follows from this source review. The actual accepted RED evidence remains bounded by its own independent verdicts; the two new settlement tests remain unexecuted.

Graft retrieval for this review saved approximately 43,030 tokens (~$0.03). This is retrieval accounting, not implementation verification.
