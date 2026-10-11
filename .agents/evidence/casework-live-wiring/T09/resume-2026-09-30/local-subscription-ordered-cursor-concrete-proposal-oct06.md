# Local ordinary-event cursor correction — concrete proposal

Date: 2026-10-06  
Status: fresh source-grounded proposal for root architecture review; no implementation is released.

## Recommendation and compatibility boundary

Use one monotonically allocated ordinary event ordinal per local case, with a snapshot revision being one kind of event that also appends immutable snapshot/history data. Allocate distinct ordinals for each simulated `execution_progress` and `settlement_recorded` event. A later snapshot allocates after all earlier ordinary events. Keep the `execution_observation` same-real-case-cursor rule outside this allocator and ordinary comparison path.

This is implementable behind the existing local adapter without changing the public port, event DTO, or HTTP adapter, provided the contract distinction is explicit in code/tests: `TemporalTrajectoryResponse.head_cursor` and `getSnapshotAt` identify retained snapshot revisions; a normal stream cursor identifies an ordered event position and need not be a retained snapshot cursor. The source already implements `getSnapshotAt` as an exact lookup in `snaps` (`localAdapter.ts:146-150`) and computes trajectory head from the last revision point (`:162-166`). The port does not declare that every event cursor must resolve through `getSnapshotAt` (`ports/contract.ts:204-216`), and the normative recovery example uses the last delivered stream cursor for `?last` (`Spec §6.2, lines 204-225`). The common ordinal keeps those cursor values comparable and monotonic.

There is still a contract gap: neither `TemporalTrajectoryResponse.head_cursor` nor the `sinceCursor` parameter documents whether the trajectory head must equal the newest ordinary stream event. If root/spec owners require that equality, this design is incompatible with preserving revision-only history: progress or settlement ordinals would make the stream frontier exceed the last snapshot. In that case reject this direction and require an explicitly authorized stream-frontier API/contract decision; do not rewrite or add progress points to temporal revision history to fake equality. The sources establish separate revision and event uses, but do not conclusively settle this head/frontier semantic.

## Current source facts

- `CaseRecord` currently holds only `snaps` and `points`; the adapter owns records in a per-instance `cases` map (`localAdapter.ts:52-55,71-74`). Northstar starts from authored immutable snapshot/history fixtures (`:82-83`); template-created cases start with a fixed first snapshot and matching first point (`:312-342`). The current `append` derives a new revision cursor from the latest snapshot alone (`:401-429`), so it can reuse a cursor already used by progress.
- `subscribeEvents` ignores `sinceCursor`, registers a bare callback, and returns a set deletion (`localAdapter.ts:168-173`). `emit` schedules one unguarded zero-delay callback for every current listener (`:433-435`); disposal does not guard callbacks already queued.
- Progress timers are scheduled in `execute` (`localAdapter.ts:347-366`), and `progress` reads the then-current snapshot cursor when the timer fires (`:391-399`). Execution completion appends a snapshot; settlement appends another snapshot and then emits `settlement_recorded` with that same cursor (`:357-388`). Those two ordinary event types can therefore collide with a revision cursor, and late callbacks can outlive subscription changes.
- Shared conformance captures `headCursorBeforeResume`, but currently passes the older initial `snapshot.cursor` in both resume modes (`caseworkPortConformance.ts:109-115`). Its future-only assertion compares callbacks against the captured trajectory head (`:134-138`). The replay-retained branch expects replay after the original snapshot cursor (`:118-132`); that branch must retain its input and expectations.
- Existing local tests require visible progress and settlement (`localAdapter.test.ts:179-192`) and completion-before-settlement ordering (`:162-176`). Keep these obligations. The current checks use wall-clock waits, so add deterministic coverage rather than relying on those delays for the new ordering proof.
- The canonical mock already increments one sequence for progress (`mock-adapter.ts:76-79,325-340`), but its trajectory reports its own sequence as `CurrentHead` (`:250-288`); it does not prove that progress can advance while a local revision-only head stays unchanged. Treat it only as evidence for ordinary monotonic event allocation.
- Spec §6.1 shows progress after a snapshot at the next numeric cursor (`lines 195-202`); §6.2 resumes from the latest delivered cursor (`:204-225`). §6.3 separately requires `execution_observation` to copy a real case cursor and leave the client's revision cursor unchanged (`:227-235`). The shared event type permits progress/settlement event kinds (`types.ts:426-449`), while its observation DTO comment explicitly calls that event cursor a latest real case cursor (`:446-449`). Do not route that exception through the new ordinary ordinal allocator.

## Private state and allocation rules

Extend only private local-adapter state. Each `CaseRecord` owns its cursor allocation state; do not use a process/global counter, another case's counter, wall-clock time, or a run ID to order events.

```ts
interface LocalCursorState {
  readonly epoch: string
  lastSequence: number // integer in [0, 9_999_999_999]
}

interface LocalSubscriber {
  readonly onEvent: (event: StreamEvent) => void
  readonly onError: (error: Error) => void
  active: boolean
  lastDelivered: LocalCursor
  queue: StreamEvent[]
  drainScheduled: boolean
  drainTimer?: ReturnType<typeof setTimeout>
}

interface LocalCursor {
  readonly epoch: string
  readonly sequence: number
}
```

This is illustrative private shape, not a request to export types. Existing generated cursor grammar is `<epoch>.<10-digit-sequence>` (`types.ts:109`; local `pad` is ten digits at `localAdapter.ts:69`). Parse and compare the two decimal components numerically (or by validated decimal-string comparison), not with unpadded lexical comparison. Keep the case's fixture epoch fixed. Sequence `9_999_999_999` is the maximum representable ten-digit value; do not wrap, reuse, or silently increment the epoch. On exhaustion, fail the allocation before changing a snapshot, point, event frontier, or returned intent cursor. For timer-only progress/settlement, stop that event and report the failure once to active subscribers through `onError`; do not emit a duplicate cursor. Invalid supplied cursors likewise report through `onError` and create no active subscription. These local error-path details need focused tests.

Initialize allocator state per existing case from the latest snapshot cursor after checking that its final trajectory point has that same revision cursor. The Northstar fixture and template fixture each receive independent state. A newly committed template case seeds its allocator at its initial snapshot cursor, which is its first revision/event ordinal; its next ordinary event allocates strictly after it. Do not renumber historical fixture cursors.

Allocation sites:

| Operation | Cursor allocated | Revision/history effect |
|---|---|---|
| Existing or newly committed snapshot | One new ordinal, used as the snapshot's cursor and its matching `TemporalCheckpoint.cursor` | Append exactly one immutable snapshot and one trajectory point. |
| Each progress timer callback | One new ordinal used only by its `execution_progress` event | No snapshot or trajectory mutation. |
| Execution-completed mutation | One new ordinal for its snapshot event/revision | Append the existing completion snapshot/history point. |
| Settlement mutation | One new ordinal for its settlement snapshot/revision | Append the existing settlement snapshot/history point. |
| `settlement_recorded` after that snapshot | A further distinct ordinal | Emit the existing settlement payload; do not append another revision point. |

The table's two settlement rows describe the current call sequence at `localAdapter.ts:378-387`: the snapshot emitted by `append` remains first, then the settlement side event follows at the next ordinal. All ordinary events use the same allocator and therefore order by allocation, not by timestamp. No snapshot or trajectory point is inserted for progress or for the standalone settlement event.

For example, from current revision `H`, an accepted-intent snapshot can be `H+1`; successive progress callbacks use `H+2` through `H+5`; completion snapshot is `H+6`; settling progress is `H+7`; settlement snapshot is `H+8`; the separate settlement event is `H+9`; and the next actual snapshot is `H+10`. The trajectory contains revision points only through `H+8`; its head remains `H+8` after the `H+9` settlement event. An old snapshot at `H+1` remains byte-for-byte unchanged. This explicitly represents a revision head below the ordinary stream frontier.

## Future-only subscription and delivery lifecycle

Replace the bare listener callback with a per-subscription record. On `subscribeEvents(caseId, sinceCursor, ...)`, synchronously read that case's current allocator high-water and register the active subscriber before returning. The local adapter is future-only and does not add a replay buffer. Set the initial exclusive boundary to `max(parsed sinceCursor, captured event high-water)`; this both honors an explicitly later `sinceCursor` and prevents a delayed callback from an already allocated event being mistaken for a new event. If `sinceCursor` is omitted, the captured high-water still makes the subscription future-only. The conformance future-only branch supplies its captured `headCursorBeforeResume`; the internal frontier additionally handles the case where progress/settlement has advanced after the latest revision.

On each allocated event, enqueue it in ordinal order for each active case subscriber. Give each subscriber one FIFO zero-delay drain, rather than independent callbacks that can overtake each other. The drain must check `active` before every callback, discard any ordinary event with `cursor <= lastDelivered`, advance `lastDelivered` before invoking `onEvent`, and continue only in allocation/FIFO order. If a callback unsubscribes itself, immediately stop draining it. `unsubscribe` sets `active=false`, removes the subscriber, clears its pending timer, and drops its queue; the active check remains mandatory because a timer may already be running. Catch a listener exception, report it to that subscriber's `onError`, and dispose only that subscriber so another listener still receives later events.

For an ordinary event whose cursor is malformed or not greater than allocator high-water, fail closed rather than deliver it or move the frontier backwards. Allocation and publication happen synchronously in one JS turn, so a new subscriber captures either the old high-water before allocation or the new high-water after allocation; it cannot see a partially committed cursor. `execution_observation`, if ever added to this local adapter, must retain the explicit non-advancing special handling before ordinary cursor comparison and must not update `lastDelivered`; this task does not add that event.

The local future-only adapter does not promise replay for an older `sinceCursor`. That is an existing limitation of local behavior, not a new branch of this proposal. The conformance retained-replay path remains unchanged: it continues to pass the original `snapshot.cursor` and keeps the exact accepted-cursor replay assertions. Only its future-only branch passes `headCursorBeforeResume` (`caseworkPortConformance.ts:109-115`). No HTTP client cursor filters or public port signatures change.

## Test-first bounded implementation scope

Implementation should be a separate release after root architecture acceptance. A scoped manual-timer queue like `httpCaseworkAdapter.nativeEvents.test.ts:4-39` gives deterministic control of delayed progress and zero-delay delivery. Keep changes to these source/test files only:

1. `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.test.ts`: add a local manual timer helper or a narrowly reusable test helper; replace timing-based evidence for the new cursor rules with explicit timer advancement.
2. `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts`: private per-case allocator, revision allocation, progress/settlement allocation, subscriber queue, boundary filter, and disposal guard.
3. `apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts`: choose `headCursorBeforeResume` only for the future-only branch; preserve original `snapshot.cursor` for replay-retained and retain all current replay assertions.

RED/green cases required before implementation approval:

- **Current-head future-only boundary:** dispatch an accepted intent, capture `headCursorBeforeResume`, arrange an already allocated progress callback whose delivery is still queued, subscribe with the captured head, then drain the old callback. It must not be delivered. Trigger the next progress callback after subscription and prove it arrives once with a cursor greater than the captured head.
- **Stale and equal filters:** queue an ordinary event at the supplied boundary and an older ordinary event before the subscription; neither is delivered afterward. A newly allocated event above the boundary is delivered. Assert equality is exclusive without wall-clock delays.
- **One ordinal sequence/order:** manually advance execution timers through progress, completion snapshot, settling progress, settlement snapshot and settlement event. Assert every ordinary delivered cursor is strictly increasing and distinct, event type order remains intact, progress remains visible, and settlement follows its snapshot. Then cause a later actual snapshot and assert its cursor exceeds the last progress/settlement event.
- **Revision immutability and split head/frontier:** capture a prior snapshot and trajectory. After progress, `getSnapshotAt` for the prior snapshot is unchanged, trajectory points/head have not advanced, and the next actual snapshot appends one point at a cursor above progress. After settlement, trajectory head is the settlement snapshot cursor while the settlement side-event cursor is greater; `getSnapshotAt(settlement-event-cursor)` continues to reject because it is not a revision. Assert this behavior only if root accepts the head/frontier distinction above.
- **Per-case ownership:** advance ordinary events in one case and prove another case's next cursor is determined only by its own seed and is unchanged by the first case. Verify each case's initial allocator seed matches its latest snapshot/final trajectory point.
- **Queued disposal:** queue delivery, unsubscribe before its drain timer runs, then run all pending timers. No callback occurs. Also unsubscribe from inside a callback and assert later queued items for that subscriber are dropped.
- **Numeric exhaustion/validation:** from a test-seeded final sequence `9_999_999_998`, allocate exactly `9_999_999_999`; the next attempt fails without wrapping/reusing a cursor or mutating revisions. Invalid cursor input calls `onError` once and leaves no registered listener.
- **Preserve replay mode:** ensure the retained-replay conformance branch still supplies the original resume cursor and still observes the original exact accepted mutation cursor and strictly increasing replay prefix.

The existing progress and settlement assertions at `localAdapter.test.ts:179-192` remain. Do not weaken them to make the boundary filter pass. No ordinary equal-head progress exemption is introduced. No sleeps, retry luck, changed schema, new event kind, or fabricated kernel cursor is used.

## Required architecture decisions and review conditions

Root must explicitly accept or reject the following before source release:

1. **Revision head versus stream frontier:** accept that trajectory/history head stays at the last snapshot revision while ordinary stream event ordinals may be later; otherwise reject this direction rather than changing history silently.
2. **Meaning of local `sinceCursor`:** approve local's existing future-only behavior with a captured allocator-frontier baseline and an exclusive caller boundary. This implementation does not add local replay.
3. **`getSnapshotAt` limitation:** confirm stream ordinals without snapshots are not valid temporal snapshot positions. This is exactly what current local source does for arbitrary non-snapshot cursors, but it becomes common for real progress/settlement events.
4. **Cursor exhaustion and bad inputs:** approve fail-closed, no-wrap behavior and the `onError` treatment for local test-fixture cursors.

Independent review should specifically challenge ordering when a callback is queued before subscription, allocator seeding on both constructor and template-commit paths, equality filtering, settlement's two distinct ordinals, `execution_observation` isolation, and any accidental mutation of captured snapshots/trajectory points. The source evidence supports the private mechanism, but does not substitute for root's decision on the public meaning of revision head versus stream frontier. No code, tests, compiler, gate, build, or Git operation was performed for this proposal.

## Direct references

- Original adjudication: `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/renderer-chunks-local-subscription-contract-adjudication-oct06.md` (SHA-256 `31e9ccb36f19fc21750c22782ebd11c4c5ff85ac0a7fd39ae2827f58135eb190`). Its three-step remedy requires an advancing progress cursor but does not specify the allocator or its ordering against later revisions.
- Full independent partial acceptance: `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/renderer-chunks-local-subscription-contract-independent-review-oct06.md` (partial acceptance; immutable revision head versus stream-cursor ordering flagged as unresolved).
- Normative cursor/event requirements: `.agents/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:195-235`; cursor form and stream DTOs: `.agents/reports/interface-contracts/typescript/types.ts:109,419-449`.
- Port boundary: `apps/godspeed-cognitive-ui/src/ports/contract.ts:204-216`; exact snapshot resolution and trajectory head: `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts:140-166`.
- Per-case initialization, subscription, lifecycle and allocation sites: `localAdapter.ts:52-99,168-173,312-342,347-399,401-435`.
- Conformance branch and preserved replay behavior: `apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts:109-142`.
- Existing progress/settlement obligations: `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.test.ts:162-192`; manual timer precedent: `apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.nativeEvents.test.ts:4-39`.
- Canonical mock allocator is limited evidence: `.agents/reports/interface-contracts/typescript/mock-adapter.ts:76-79,250-288,315-349`.
