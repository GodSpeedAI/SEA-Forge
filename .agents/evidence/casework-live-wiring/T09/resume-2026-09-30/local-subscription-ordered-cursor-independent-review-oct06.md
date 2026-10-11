# Local ordered-cursor proposal: independent review

Date: 2026-10-06  
Verdict: **REJECT pending bounded corrections and explicit root decisions.** This is a DOCONLY review; no source, tests, gates, or implementation were changed or run.

## Reviewed identities

- Original assignment: `local-subscription-ordered-cursor-proposal-original-assignment-oct06.md`, SHA-256 `6028c2e99831f67a5ab3b974e955fd9b1fc6d7e1ac7c012ef93944bcd409d4c8`.
- Full proposal: `local-subscription-ordered-cursor-concrete-proposal-oct06.md`, SHA-256 `6f1b431963d7c49b76e4c93d4cd3e2ba9a0f920dc7ec6201949da846aed30658`.
- Prior cursor adjudication: `renderer-chunks-local-subscription-contract-adjudication-oct06.md`, SHA-256 `31e9ccb36f19fc21750c22782ebd11c4c5ff85ac0a7fd39ae2827f58135eb190`.
- Prior independent partial review: `renderer-chunks-local-subscription-contract-independent-review-oct06.md`.

## Findings that block acceptance

### 1. Allocator high-water validation can reject every newly allocated event

The proposal says `lastSequence` is the per-case allocator state (lines 26–32), then assigns each event a new ordinal (lines 54–66), enqueues each allocated event (line 72), and requires an ordinary event cursor to be greater than “allocator high-water” before delivery (line 74). The allocator high-water after allocation is that event's cursor. Applying the stated check at delivery therefore rejects the event that advanced the high-water, including every progress, snapshot, and settlement event.

The intended safe rule needs one unambiguous linearization point. For example, a private allocate-and-publish operation can compare its candidate against the *previous* high-water, advance the allocator once, then enqueue that same candidate without comparing it to the now-advanced value. Alternatively, distinguish allocated and published frontiers and specify their transitions. The proposal must state which value each comparison uses and test it; leaving this as an implementation interpretation is not sufficient for cursor correctness.

### 2. Per-subscriber queue has no bound or overflow behavior

`LocalSubscriber.queue` is an unconstrained `StreamEvent[]` (lines 34–42), and the delivery design appends every allocated event before its FIFO drain runs (line 72). Unsubscribe clears it, but an active slow or reentrant listener has no stated queue ceiling or overload behavior. The assignment requires ordered delivery and visible progress/settlement, so silent event dropping is not acceptable; the proposal must either use the simpler guarded FIFO timer model already present in this adapter or specify a finite queue limit, deterministic overflow/error behavior, and how required events remain truthful. The root's requested queue-bound review remains unresolved.

### 3. The claimed existing ten-digit maximum is not established by cited sources

The proposal calls `<epoch>.<10-digit-sequence>` the existing grammar and treats `9_999_999_999` as the maximum representable value (line 50). The actual type comment says `<epoch>.<seq>` (`types.ts:104`); the wire contract test accepts any decimal digit count with `/^\d+\.\d+$/` (`wireContract.test.ts:416`); and local formatting uses `padStart(10, '0')` (`localAdapter.ts:69`), which supplies a minimum width but does not reject an eleventh digit. The exact ten-digit assertion in `northstarData.test.ts` applies to fixture cursors, not the cursor grammar. These sources do not establish the asserted maximum.

This matters because HTTP stream adapters compare cursor strings lexicographically (`httpCaseworkAdapter.ts:345–347,418–425`). Keeping local sequence strings at exactly ten digits within one fixed epoch preserves that ordering; an unbounded increment that grows to eleven digits does not. The proposal may request a local operational ceiling, but it must identify it as a newly imposed local bound, justify it against the existing consumers/contract, validate the supplied cursor and allocator state against it, and obtain the required root decision. It must not present the ceiling as already encoded by `types.ts`.

### 4. Cursor input boundary cases are not specified precisely enough

Lines 50 and 70 propose parsing `sinceCursor`, comparing it with the local high-water, and reporting invalid inputs through `onError`, but the accepted grammar and edge behavior are not defined. In particular, the proposal fixes the case epoch and also accepts an explicitly later `sinceCursor`. A cursor with a later epoch than the fixture epoch will remain above every cursor this fixed-epoch allocator can generate; the subscription would silently receive nothing forever. Values outside JavaScript's exact integer range also cannot safely be parsed into the proposed `number` fields and compared numerically. Specify whether wrong-epoch, out-of-bound, unsafe-integer, malformed, and genuinely future same-epoch cursors fail through `onError` or have another explicit behavior. Add each boundary to the deterministic cases before implementation approval.

## Requirements and evidence reviewed

The core design direction is coherent in several respects: one allocator per `CaseRecord`; snapshot cursors also advance ordinary event order; progress and settlement get distinct ordinals; later snapshots follow earlier side events; historical snapshots and trajectory points stay revision-only; the settlement snapshot precedes its separate side event; and `execution_observation` is expressly excluded (proposal lines 8, 26, 52–66, 74, 91). Constructor and template-commit initialization are identified. The proposal preserves retained-replay inputs/assertions and does not add a public port or DTO field (lines 76, 82–95). Its manual-timer test plan covers cursor ordering, stale/equal filtering, per-case ownership, queued unsubscribe, immutable history, replay mode, and the required prior progress/settlement expectations.

The future-only conformance scenarios were checked directly. `localAdapter.test.ts:32–33` selects `resume: 'future-only'`; the HTTP fixture at `:250` selects `resume: 'replay-retained'`. Thus the local adapter can use its private captured allocator high-water to suppress an already allocated callback even when that callback's cursor is above the trajectory revision head. The proposal's controlled test explicitly queues a pre-subscription `H+1` progress callback, subscribes at `H`, and requires suppression (proposal lines 88–89). The retained-replay HTTP branch remains unchanged. My earlier concern that the HTTP retained-replay branch itself needed a new event-frontier getter was a false positive and is withdrawn.

The source does not require `TemporalTrajectoryResponse.head_cursor` to equal the latest ordinary event cursor: local `queryTemporalTrajectory` derives it from the last retained point (`localAdapter.ts:162–166`), the HTTP trajectory validator requires it to equal the last point (`httpCaseworkAdapter.ts:105–107`), and `getSnapshotAt` is exact snapshot lookup (`localAdapter.ts:140–150`). The normative §6.1 examples assign `execution_progress` a cursor after a snapshot, while §6.3 separately carves out non-advancing `execution_observation` (`04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:195–235`). The port has no promise that every stream event cursor resolves to a snapshot (`contract.ts:204–216`). Therefore, treating the trajectory head as the retained revision head and the stream cursor as an ordinary event frontier appears compatible with the available source contract; `getSnapshotAt` rejecting a progress-only or settlement-side-event ordinal is consistent with revision-only history. Root still owns the explicit semantic acceptance requested by the proposal (lines 101–106).

Other directly checked source behavior supports the rest of the proposed scope: progress currently reuses snapshot cursor at `localAdapter.ts:391–399`; append increments from snapshot alone at `:401–429`; settlement emits its side event with the snapshot cursor at `:378–387`; subscription disposal is currently a bare set delete at `:168–173`; and existing tests require visible progress and settlement at `localAdapter.test.ts:179–192`. HTTP consumers reject ordinary cursors at or below the supplied boundary (`httpCaseworkAdapter.ts:345–347,418–425`). These observations establish why the proposed work is needed, but do not cure the four blocking definition gaps above.

## Required next step

Revise the proposal only: define allocator/high-water comparison order, resolve queue bounding/overflow without weakening ordered progress and settlement visibility, correct the cursor-format/max claim with explicit boundary rules, and preserve the already specified revision-head/stream-frontier distinction and controlled pre-subscription test. Then request root's explicit architecture decisions on revision head versus event frontier, future-only semantics, `getSnapshotAt`, and the local exhaustion/error policy. No implementation is approved by this review.
