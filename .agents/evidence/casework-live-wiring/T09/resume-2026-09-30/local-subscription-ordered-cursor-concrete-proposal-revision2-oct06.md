# Local ordered-cursor proposal revision 2

Date: 2026-10-06  
Status: fresh complete DOCONLY revision for independent review; implementation is not released.

## Decision and scope

Use one monotonically allocated ordinary-event ordinal per authored local case. Snapshot revisions are a subset of that event sequence: they retain their original cursors and immutable snapshot/trajectory facts, while `execution_progress` and the standalone `settlement_recorded` event consume ordinals without adding revision points. A later snapshot allocates after those side events. `getSnapshotAt` remains an exact lookup among retained snapshots. This is local simulation only; it does not create kernel cursors, alter `execution_observation`, or change a public DTO, port signature, HTTP behavior, schema, or generator.

The two cursor notions are deliberately distinct: trajectory `head_cursor` is the last retained revision cursor; a stream cursor is an ordinary event position and may be greater. This is compatible with available sources: the port defines subscription but does not require every event cursor to resolve as a snapshot (`contract.ts:204-216`); local trajectory head is derived from the last point (`localAdapter.ts:162-166`), and `getSnapshotAt` is exact snapshot lookup (`:146-150`). Normative §6.1 assigns progress a cursor after the snapshot, while §6.3 separately preserves the non-advancing `execution_observation` exception (`04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:195-235`). Root has accepted this local distinction. This document does not claim a new public guarantee beyond that local behavior.

## Private state and initialization

Each `CaseRecord` receives a private cursor allocator and subscriber set. Keep all state per case; no process-global counter, timestamp, or run identifier participates in ordering.

```ts
interface LocalCursorAllocator {
  readonly epochText: string
  readonly epoch: number
  sequence: number
}

interface LocalSubscriber {
  readonly onEvent: (event: StreamEvent) => void
  readonly onError: (error: Error) => void
  active: boolean
  lastDelivered: number
  queue: StreamEvent[] // maximum 64 pending ordinary events
  drainTimer: ReturnType<typeof setTimeout> | undefined
  draining: boolean
}
```

The constructor seeds each initial case from the final snapshot cursor only after validating that the final trajectory point has the same exact cursor string. Template-created cases seed at their initial committed snapshot cursor. Seed validation uses the same rules as subscription input; invalid fixture state fails closed before installing that case. Historical cursors, snapshots, and trajectory points are never renumbered.

The cursor grammar remains decimal `epoch.sequence` (`types.ts:104`); the existing local `padStart(10, '0')` establishes a minimum display width, not a wire maximum (`localAdapter.ts:69`). Root selects a **new private local operational ceiling** of sequence `9_999_999_999`, preserving comparability with existing ten-padded fixture/consumer ordering. It is not a normative public maximum or schema rule. The local allocator never wraps or changes epoch. Generated cursors retain the original seed epoch text and use ten-digit minimum sequence padding.

Parse each supplied component with `/^\d+\.\d+$/` and safe-integer checks. Compare numeric components, never raw variable-width sequence strings. Leading zeroes are accepted as equivalent numeric values for comparison; they are not copied into generated cursors. The allocator preserves its seed epoch text and existing stored cursor strings. Input is rejected with one `onError` and no active registration when malformed, unsafe, above the local sequence ceiling, or from a different numeric epoch. A same-epoch future sequence at or below the ceiling is valid as an exclusive private floor; it receives no replay and no fabricated event, and delivery begins only if a genuine later allocation exceeds it. Seed values above the ceiling or with unsafe/malformed components are invalid. These are authored-local operational checks and do not redefine wire grammar.

## One allocate-and-publish linearization

Every ordinary event uses one synchronous private `allocateAndPublish` path. Given previous sequence `p`, it first verifies `p < 9_999_999_999`, forms candidate `p+1`, formats the candidate cursor, and compares candidate to the **previous** frontier `p`. Only after all fallible snapshot preparation has succeeded does it commit `sequence = candidate`, append prepared snapshot/point state if applicable, then publish the event with that exact candidate. It never compares the event against the frontier after advancing it. No other callsite increments the ordinal or publishes an ordinary event.

For snapshot mutations, clone the prior snapshot's objects, run the edit on the clone, prepare the complete snapshot and trajectory point locally, and validate/build the cursor before changing `snaps`, `points`, or allocator state. On allocation/preparation failure, commit none of them and return no accepted cursor. Commit the new sequence and append the prepared snapshot/point synchronously, then publish its snapshot event. There is no `await` or user callback between commit and publication. If allocation is exhausted, the mutation fails without partial revision/history/frontier change.

For progress and the standalone settlement event, construct payload and timestamp, obtain a candidate, and publish only after committing that candidate. On exhaustion or timer callback failure before publication, stop that one event and report one error to each relevant active subscriber; do not emit a duplicate cursor or fabricate success. Callback failures are isolated per subscriber as described below. No failure rolls back a cursor after it has been committed and an event publication has begun.

Allocation sites preserve current visible behavior while ordering all ordinary events:

| Operation | Ordinary cursor | Retained revision effect |
|---|---|---|
| Snapshot append / committed template snapshot | Allocate one | Append one immutable snapshot and matching trajectory point. |
| Each progress timer callback | Allocate one | None. |
| Completion mutation snapshot | Allocate one | Append one immutable snapshot and matching trajectory point. |
| Settlement mutation snapshot | Allocate one | Append one immutable snapshot and matching trajectory point. |
| Separate `settlement_recorded` after settlement snapshot | Allocate one more | None. |

Current settlement emits the snapshot before its side event (`localAdapter.ts:378-387`); retain that order while giving them distinct cursors. The trajectory head stays at the settlement snapshot, below the later settlement event. Progress remains visible, and prior snapshots/points remain unchanged. For example, after revision `1.0000000042`, accepted mutation may allocate `.0043`, progress `.0044` and `.0045`, completion snapshot `.0046`, settling progress `.0047`, settlement snapshot `.0048`, and settlement side event `.0049`; the revision history ends at `.0048`. The next snapshot is `.0050`. The exact epoch/sequence examples are illustrative; allocator values are seeded from each case's actual final cursor.

## Future-only subscriptions and bounded ordered delivery

The public `subscribeEvents(caseId, sinceCursor, onEvent, onError)` contract stays unchanged. Preserve concrete local callers using three arguments by providing a private/no-op fallback error callback when optional `onError` is absent. Parse and validate `sinceCursor` synchronously before registering. Capture the case allocator's current sequence `F` and set exclusive subscriber floor to `max(parsedSinceSequence, F)`. Register the subscriber before returning. This is future-only and adds no retained replay: it honors a later supplied same-epoch boundary while suppressing any event already allocated before subscription, even if its timer delivery is still queued. Shared conformance must pass the captured trajectory head for the future-only branch as presently written; the private allocator frontier also covers progress/settlement beyond that retained head. Preserve the retained-replay branch's original resume cursor and assertions unchanged.

On publish, visit that case's active subscribers synchronously in registration order. Append the event to each subscriber FIFO. Each subscriber has at most one pending zero-delay drain timer. The drain checks `active` before every callback, dequeues only in FIFO order, rejects/discards any ordinary cursor at or below `lastDelivered`, then advances `lastDelivered` before calling `onEvent`. If a callback disposes itself, stop immediately and discard its remaining queue. `unsubscribe` marks inactive, removes the record, clears its timer, and empties its queue; the active guard still protects an already-running callback. Catch `onEvent` exceptions, call that subscriber's `onError` once, dispose only that subscriber, and continue other subscribers.

The private queue limit is 64 **pending** ordinary events per subscriber. On an attempted 65th pending enqueue, dispose only that subscriber, clear its queued events/timer, and invoke `onError` exactly once with an error stating the local event backlog exceeded 64 and complete delivery is unavailable. Never silently drop an event or evict progress/settlement to claim complete delivery. Other subscribers continue. Reentrant publishing from `onEvent` uses the same FIFO and limit; it cannot create another drain timer. If the initial drain-timer scheduling itself throws, dispose that subscriber and report once. Errors thrown by `onError` are contained and do not affect other subscribers. These per-subscriber rules bound pending queue length, not the total number of subscribers or process-wide memory.

If allocator-seed corruption or an internal non-monotonic candidate is detected, fail closed: publish nothing from that candidate and report once to each affected active subscriber. `execution_observation` remains outside this ordinary allocator and ordering filter; if emitted locally in future, its normative unchanged-cursor handling must be evaluated first and it must not advance `lastDelivered`. This task does not add it.

## Deterministic test-first release plan

No implementation or tests are authorized by this proposal. If root releases a later source task, keep implementation to the local adapter, its tests, and only the required future-only branch adjustment in shared conformance. Use manually controlled timers (the HTTP adapter test helper at `httpCaseworkAdapter.nativeEvents.test.ts:4-39` is a precedent); do not use sleeps, retry luck, or timing-sensitive pass criteria.

Required red/green cases:

1. **Late pre-subscription event:** allocate H+1 and queue its drain, subscribe with supplied H while allocator frontier is H+1, run the old drain, and prove H+1 is suppressed. Allocate H+2 after subscription and prove one delivery above H.
2. **Exclusive/future boundaries:** stale and equal ordinary events are suppressed; a genuine later event is delivered. Valid future same-epoch floor produces no callback until allocator exceeds it. Malformed syntax, unsafe epoch/sequence, different epoch, and sequence above ceiling call `onError` once and create no active subscriber. Leading-zero numeric-equivalent input compares correctly without rewriting emitted/stored cursor text.
3. **Single increasing ordinal and existing obligations:** manually advance progress, completion snapshot, settling progress, settlement snapshot, and settlement side event. Assert all delivered ordinary cursors are distinct and strictly increasing; progress is visible; settlement follows its snapshot; a subsequent snapshot exceeds every earlier event.
4. **Revision immutability/split heads:** retain a pre-progress snapshot and trajectory response, emit progress, and verify exact snapshot lookup and trajectory contents/head are unchanged. After a later snapshot exactly one revision/point is appended above progress. Verify settlement event is above settlement snapshot and does not resolve through `getSnapshotAt`.
5. **Initialization and independent cases:** validate constructor and template-commit seeds against their last snapshot/point; advancing one case cannot affect another's next cursor.
6. **Bounded FIFO:** 64 pending events remain ordered; the attempted 65th disposes that subscriber, clears pending delivery, and yields exactly one backlog `onError`; a second subscriber still receives events. Include reentrant enqueue to capacity, self-disposal, callback throw, throwing error handler, and timer-scheduling failure.
7. **Exhaustion atomicity:** seed a test case at `9_999_999_998`; allocate `9_999_999_999`; the next snapshot attempt returns failure with snapshots, points, frontier, and accepted cursor unchanged. Progress/settlement timer exhaustion reports once and creates no event. No epoch wrap.
8. **Replay preservation:** retained-replay conformance still passes its original resume cursor, exact accepted mutation cursor expectation, and strict replay-prefix ordering. Only future-only conformance uses the captured current head.

Keep the existing local progress and settlement assertions (`localAdapter.test.ts:179-192`) intact. The local future-only conformance must explicitly test queued H+1 suppression because a generic cursor-greater-than-head assertion does not prove that property. `execution_observation` behavior and public wire limits are out of scope.

## Evidence, uncertainty, and deviations

Source anchors: local state/seed construction `localAdapter.ts:52-99`; snapshot lookup and trajectory head `:140-166`; subscription `:168-173`; template case initialization `:312-342`; execution timers and settlement order `:347-389`; progress `:391-399`; append mutation/publication `:401-435`; cursor DTO grammar and stream event kinds `types.ts:104,426-449`; conformance future-only and replay branches `caseworkPortConformance.ts:109-142`; required progress/settlement tests `localAdapter.test.ts:162-192`; manual timer precedent `httpCaseworkAdapter.nativeEvents.test.ts:4-39`.

Normative anchors: `04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:195-235` describes progress cursor advancement, resume cursor usage, and the separate same-real-cursor observation exception. `contract.ts:204-216` leaves local retention behavior and snapshot resolution unspecified. The existing local adapter's future-only behavior is therefore an implementation policy, not a public retained replay promise.

Material decisions incorporated from root: one local case event ordinal; revisions as a subset; future-only private floor; compare candidate against previous frontier then commit/publish that same cursor; per-subscriber 64 pending FIFO with explicit disposal/error; and private 10-digit padded operational ceiling. The ceiling and backlog limit are new authored-local operational constraints, not existing schema or public wire limits. Root has accepted the revision-head/ordinary-stream-frontier distinction for this local simulation.

Material deviations from the rejected complete proposal: its missing queue bound/error/disposal contract is now explicit; the false assertion that ten digits is the existing grammar maximum is corrected to minimum padding plus a new private ceiling; candidate validation now uses the previous frontier and has atomic snapshot preparation/commit ordering; and cursor input cases are specified, including same-epoch future floors and different-epoch rejection. No normative specification, source, fixture, test, public interface, or shared conformance file is changed by this document. No compiler, test, scanner, gate, or Git command was run.

Uncertainty retained for implementation review: whether `onError` callback behavior and three-argument local compatibility exactly match every external concrete-adapter caller must be checked against the callsites before a later source release. Snapshot preparation can be made atomic for ordinary thrown preparation/allocation errors in JavaScript; catastrophic allocation failure during native array mutation is not promised to be recoverable. Queue capacity applies to pending queue items and intentionally does not impose a subscriber-count/global-memory bound. Independent review and a subsequent controlled-timer RED are required before implementation; this proposal itself releases no code.
