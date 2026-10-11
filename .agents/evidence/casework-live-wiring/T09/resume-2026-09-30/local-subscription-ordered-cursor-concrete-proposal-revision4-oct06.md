# Local ordered-cursor proposal revision 4

Date: 2026-10-06  
Status: complete DOCONLY citation-corrected revision for independent review;
implementation remains held.

## Decision and scope

Use one monotonically allocated ordinary-event ordinal per authored local case.
Retained world snapshots and trajectory points are a subset of that sequence:
they keep their original cursors and immutable facts. Simulated progress and
the standalone settlement event consume distinct ordinals without becoming
revision points. A later snapshot allocates after those events. `getSnapshotAt`
remains an exact lookup among retained snapshots. This concerns only authored
local simulation. It creates no kernel cursor, changes no
`execution_observation` behavior, and changes no public DTO, port signature,
HTTP behavior, or schema.

The trajectory head remains the last retained revision cursor; the ordinary
stream frontier may be later. The port allows ordinary event subscriptions
without promising that every stream cursor resolves to a snapshot
(`apps/godspeed-cognitive-ui/src/ports/contract.ts:204-216`). The local
adapter's `getSnapshotAt` is exact retained-snapshot lookup and its trajectory
head comes from the last point (`src/adapters/local/localAdapter.ts:146-166`).
Normative §6.1 gives `execution_progress` a later event cursor than the
preceding snapshot; §6.3 separately says an informational
`execution_observation` copies the latest real case cursor and does not advance
the client's case cursor (`.agents/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:195-235`).
This local distinction introduces no public frontier promise.

## Private state, initialization, and cursor semantics

Each case owns a private allocator and its subscribers; no process-wide
counter, timestamp, or run ID participates in ordering.

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

Seed each constructor-installed case from its last snapshot only after
validating that the final trajectory point has the same exact cursor string.
Seed a template-committed case from its initial committed snapshot cursor.
Validate seeds using the same syntax, numeric safety, epoch, and local-ceiling
rules as supplied cursors; malformed fixture state fails closed before that
case is installed. Never renumber existing snapshots or trajectory points.
Current constructor state is at
`apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts:52-99`; the
template-created case is installed at `:276-343`.

The canonical TS snapshot field is typed as `string` with a format example,
not an executable validator (`.agents/reports/interface-contracts/typescript/types.ts:101-110`).
The `CURSOR_RE` expression at
`apps/godspeed-cognitive-ui/src/ports/wireContract.test.ts:416` is test-only;
it is `/^\d+\.\d+$/`, and its snapshot/observation assertions are at
`:489-495,604-624`. It permits variable digit counts and does not implement
runtime subscription validation. The actual local comparator is
`src/ports/project.ts:145-150`; it converts components with `Number` and
compares them, but does not validate the cursor grammar or safe-integer range.
The subscription parser below must perform those checks itself.

`padStart(10, '0')` in the local adapter is minimum formatting width
(`src/adapters/local/localAdapter.ts:68-70`), not a wire grammar maximum. Use
the root-selected **new private authored-local operational ceiling**
`9_999_999_999` for the sequence so generated values remain comparable with
existing ten-padded fixtures/consumers. This is not a normative public maximum
or existing parser rule. Never wrap or increment the epoch. Preserve the
allocator seed's original epoch text; generate sequence text with ten-digit
minimum padding while the proposed ceiling holds.

Parse supplied cursors as exact decimal `epoch.sequence` text with
`/^\d+\.\d+$/`; convert both components only after checking `Number.isSafeInteger`.
Require numeric epoch equality with the case seed and sequence not above the
private ceiling. Compare numerically, never by variable-width lexical sequence
text. Leading zeroes are accepted as numerically equivalent and are not copied
into generated cursors; existing stored/seed strings are not rewritten.
Malformed syntax, unsafe components, different numeric epoch, and sequence
above the ceiling call `onError` exactly once and install no active
subscription. Validate seeds with the same rules. A same-epoch future sequence
within the ceiling is a valid exclusive floor: it causes no replay and no
fabricated event; delivery begins only after a genuine allocation exceeds it.

## One allocate-and-publish linearization

Every ordinary event uses one synchronous private `allocateAndPublish` path.
Given previous sequence `p`, check that `p < 9_999_999_999`, compute candidate
`p + 1`, format it, and verify it advances past **p before changing the
frontier**. Prepare all fallible event/snapshot data first. Then commit
`sequence = candidate`, append prepared snapshot/trajectory state if this is a
revision, and publish that same candidate cursor. Do not compare a candidate
with the already-advanced frontier. No other callsite increments the ordinal
or publishes an ordinary event. There is no `await` or user callback between
commit and publication.

For a snapshot mutation, clone the previous snapshot's objects, apply the edit
to that clone, and build the complete next snapshot, point, and candidate
cursor before changing `snaps`, `points`, or allocator state. If preparation or
allocation fails, change none of those and return no accepted cursor. On
success, synchronously commit the cursor and prepared revision/point, then
publish. The promise covers ordinary thrown preparation/allocation failures;
catastrophic native allocation failure during JavaScript array mutation is not
claimed recoverable. Exhaustion never partially changes revision, points, or
frontier. Progress/settlement payload or timer-callback failure before cursor
commit stops that event and reports once to each relevant active subscriber;
it emits no event and fabricates no accepted cursor. Once an event cursor is
committed and publication begins, callback failure does not roll it back.

| Local operation | Ordinary event ordinals | Retained revision effect |
|---|---:|---|
| Snapshot append or committed template snapshot | 1 | Append one immutable snapshot and matching trajectory point. |
| Each progress timer callback | 1 | None. |
| Completion snapshot | 1 | Append one immutable snapshot and matching point. |
| Settlement snapshot | 1 | Append one immutable snapshot and matching point. |
| `settlement_recorded` after settlement snapshot | 1 additional | None. |

Preserve current settlement order: settlement snapshot first, separate
`settlement_recorded` second (`localAdapter.ts:367-388`). Keep progress events
visible (`:391-399`) and prior snapshots/points unchanged. For illustration,
from revision `1.0000000042`, a mutation may allocate `.0043`, progress `.0044`
and `.0045`, completion `.0046`, settling progress `.0047`, settlement
snapshot `.0048`, and settlement side event `.0049`. Revision history ends at
`.0048`; a later snapshot is `.0050`. Values are illustrative and use the
case's actual seed epoch and sequence.

## Subscription behavior, including omitted and supplied floors

Keep public `CaseworkPort.subscribeEvents(caseId, sinceCursor, onEvent, onError)`
unchanged (`apps/godspeed-cognitive-ui/src/ports/contract.ts:204-216`). Preserve
concrete local adapter callers that pass three arguments by making its
implementation's fourth callback optional and substituting a private no-op
`onError`; do not alter the public port signature.

At invocation, find the case and validate a supplied string before registering.
Capture that case's allocator frontier `F` synchronously. `sinceCursor ===
undefined` means **no supplied floor**, not invalid input: use `F` as the
exclusive floor, install no replay, report no input error, and return the
ordinary unsubscribe closure. For a valid supplied string, use the maximum of
its parsed numeric sequence and `F`. Both modes are future-only; a preexisting
event is suppressed even when its delivery timer is queued. A missing case
or invalid supplied string calls `onError` once, installs no active record,
and returns a harmless unsubscribe closure.

A floor does not allocate an event. A safe same-epoch future floor remains
silent until a genuine allocation exceeds it. For `undefined`, the captured
frontier prevents a queued pre-subscription event from becoming post-subscribe
delivery merely because its timer fires later.

The current concrete conformance implementation subscribes using
`snapshot.cursor` before it branches between replay and future-only
(`apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts:109-139`).
The authorized *proposal scope* includes a branch-only correction: capture the
trajectory head as it already does, pass that captured `headCursorBeforeResume`
for `resume === 'future-only'`, and keep the original supplied snapshot/resume
cursor for `resume === 'replay-retained'`. Keep replay's exact accepted cursor
and chronological-prefix expectations unchanged. Do not describe the current
conformance code as already supplying the future-only head.

The future-only conformance's generic `event.cursor > headCursorBeforeResume`
assertion (`caseworkPortConformance.ts:134-139`) does not prove that a queued
H+1 event is suppressed. Add both deterministic local adapter tests below:

1. **Omitted boundary:** allocate H+1 and queue its drain, subscribe with
   `undefined`, run the already-queued timer and prove H+1 is suppressed;
   allocate H+2 after registration and prove one genuine delivery. The call
   returns a normal unsubscribe and causes no `onError`.
2. **Supplied revision boundary:** allocate/queue H+1, then subscribe with
   supplied revision H while allocator frontier is H+1; prove queued H+1 is
   suppressed and a later H+2 is delivered. Keep this separate from the
   omitted-boundary case; neither replaces the other.

## Ordered bounded delivery and cleanup

On publication, synchronously visit that case's active subscribers in
registration order and append to each subscriber FIFO. Each has at most one
zero-delay drain timer. The drain checks `active` before each callback, dequeues
in FIFO order, ignores any ordinary event at or below `lastDelivered`, then
advances `lastDelivered` before invoking `onEvent`. Self-unsubscribe stops the
drain and clears remaining items. Unsubscribe marks inactive, removes the
record, cancels the timer, and empties the FIFO; an active check also prevents
delivery from an already-started drain.

The proposed **64 pending ordinary event** capacity is per subscriber. On an
attempted 65th enqueue, dispose only that subscriber, cancel its timer, clear
its FIFO, and call its `onError` exactly once with a clear message that backlog
exceeded 64 and complete delivery is unavailable. Never silently drop events,
evict progress/settlement to claim completeness, or stop other subscribers.
Reentrant publication from `onEvent` uses the same FIFO/limit and cannot
schedule a second drain. Catch `onEvent` exceptions, report once to that
subscriber, dispose it, then continue other subscribers. Contain exceptions
from `onError`. If timer scheduling throws, dispose and report once. A
seed-corruption or internal nonmonotonic-candidate failure publishes nothing
and reports once to each affected active subscriber.

Queue capacity bounds pending items only; it is not a subscriber-count or
process-wide memory bound. The
`execution_observation` event type remains outside the ordinary allocator and
filter. If ever emitted locally, its §6.3 unchanged-real-cursor behavior must
be evaluated separately and must not advance `lastDelivered`; this proposal
does not add it.

## Test-first acceptance matrix for a later authorized source task

No implementation or test edit is authorized by this proposal. If later
released, source scope is the local adapter, its tests, and the necessary
future-only branch correction in shared conformance only. Use controlled
manual timers; an existing test helper precedent is
`apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.nativeEvents.test.ts:4-39`.
New tests must not use sleeps, retry luck, or timing-sensitive pass criteria.

1. **Undefined-boundary RED/GREEN:** queue H+1 before subscribing with
   `undefined`; manually fire its old timer and assert no delivery; allocate
   H+2 and assert exactly one delivery plus a returned unsubscribe function.
2. **Supplied-boundary RED/GREEN:** queue H+1, subscribe with supplied H,
   suppress H+1, deliver H+2. Retain this independently from test 1.
3. **Exclusive and input cases:** stale/equal events do not deliver; genuine
   later events do. Safe future same-epoch floor is silent until passed by an
   actual allocation. Malformed syntax, unsafe epoch/sequence, different
   numeric epoch, and sequence above the private ceiling call one error and
   install no subscriber. Leading-zero numeric equivalents compare correctly
   without rewriting stored/emitted cursors. Verify invalid constructor seed
   fails closed.
4. **Single ordinal/order and existing visible behavior:** manually advance a
   snapshot mutation, progress callbacks, completion snapshot, settling
   progress, settlement snapshot, and side event. Assert all ordinary cursors
   are distinct and strictly increasing, progress and settlement remain
   delivered, settlement follows its snapshot, and a subsequent snapshot
   exceeds every preceding event. Keep the existing visible assertions at
   `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.test.ts:162-192`.
5. **Immutable revisions and split heads:** retain a previous snapshot and
   trajectory result, emit progress, and prove their contents/head do not
   change. Exact lookup resolves retained snapshots only. A later snapshot
   adds exactly one revision/point after intervening side events; settlement
   side-event cursor does not resolve via `getSnapshotAt`.
6. **Case initialization/isolation:** validate initial and template-commit
   seeds against final snapshot and trajectory point; advancing one case does
   not affect another case's next cursor.
7. **Bounded FIFO and cleanup:** 64 pending events stay ordered; attempted
   event 65 reports one backlog error, disposes that subscriber and clears
   queued work/timer; another subscriber continues. Cover reentrant enqueue,
   self-disposal, throwing `onEvent`, throwing `onError`, and scheduling throw.
8. **Exhaustion atomicity:** with sequence `9_999_999_998`, allocate
   `9_999_999_999`; the next snapshot fails with snapshots, points, frontier,
   and accepted cursor unchanged. Progress/settlement timer exhaustion reports
   once and emits no event. Assert there is no epoch wrap.
9. **Replay compatibility:** retained-replay conformance keeps its original
   resume cursor, exact accepted mutation cursor, and strict replay-prefix
   ordering. Only future-only conformance uses captured trajectory head.

## Citation corrections and source evidence

The earlier revision 3 proposal is preserved unchanged; its independent review
rejected it for citations only. This revision corrects the following material
source errors rather than silently changing old evidence:

1. Revision 3 cited
   `apps/godspeed-cognitive-ui/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md`.
   The real normative source is
   `.agents/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:195-235`.
2. Revision 3 cited nonexistent
   `apps/godspeed-cognitive-ui/src/ports/types.ts`. The cursor member is only
   a string/comment example in
   `.agents/reports/interface-contracts/typescript/types.ts:101-110`; canonical
   stream event kinds/envelope types are at `:426-448`. The executable
   decimal-regex check lives in test code at
   `apps/godspeed-cognitive-ui/src/ports/wireContract.test.ts:416` with
   snapshot and observation cursor assertions at `:489-495,604-624`. None of
   those is a production runtime subscription parser. Numeric comparison lives
   in `src/ports/project.ts:145-150` and does not prove or validate syntax.
3. Revision 3 cited nonexistent
   `apps/godspeed-cognitive-ui/src/ports/caseworkPortConformance.ts`. The actual
   shared fixture is
   `apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts:109-150`.
   Source inspection also corrected the prior claim that future-only already
   passes captured head: line 113 currently passes `snapshot.cursor` before
   the replay/future branch at `:118-139`. The proposed branch-only correction
   above names the necessary behavior and preserves retained replay.

Relevant implementation spans verified for this proposal:

- Case record shape, constructor fixtures and padding:
  `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts:52-99`.
- Exact snapshot lookup, trajectory head, current future-only subscription:
  `localAdapter.ts:140-173`.
- Template-created case and initial event:
  `localAdapter.ts:276-343`.
- Execution timers, settlement snapshot/side-event ordering and progress
  cursor reuse: `localAdapter.ts:347-399`.
- Snapshot append and asynchronous event delivery:
  `localAdapter.ts:401-435`.
- Existing progress/settlement behavior assertions:
  `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.test.ts:162-192`.
- Port signature:
  `apps/godspeed-cognitive-ui/src/ports/contract.ts:204-216`.
- Actual conformance branching and historical immutability:
  `apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts:109-150`.
- Normative progress and informational-observation rules:
  `.agents/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:195-235`.

## Preserved decisions, deviations, and uncertainty

Revision 4 preserves all accepted revision 2/3 semantics and error behavior:
one ordinary event ordinal per case; revisions as immutable subset; distinct
progress/settlement event ordinals; exact snapshot lookup; no epoch wrap;
`9_999_999_999` proposed local ceiling; candidate checked against previous
frontier and same-cursor commit/publication; thrown preparation/allocation
failure atomicity; queue capacity 64; one onError and no registration for
invalid inputs; safe future floor; leading-zero numeric comparison; and
`undefined` as no supplied floor using captured `F`, with unsubscribe and no
replay. Existing progress and settlement visibility/order and all assigned
controlled-timer cases remain required.

Material corrections relative to revision 3 are citation/source mapping only,
plus correction of the conformance branch fact: the current future-only branch
does not yet pass captured `headCursorBeforeResume`, so revision 4 explicitly
proposes the branch-only adjustment required by the original assignment.
Retained replay continues passing the original snapshot/resume cursor. No
source, fixture, test, public interface, normative spec, or generator changed.

The operational ceiling and FIFO limit remain local proposals, not implemented
rules, schema constraints, or global memory guarantees. Remaining source-level
uncertainty is whether the optional `onError` fallback preserves every concrete
adapter caller; verify all callers if source work is separately authorized.
Ordinary thrown preparation/allocation errors can be staged before mutation;
catastrophic native allocation failure during array mutation is not promised
recoverable. Independent review and the required controlled-timer REDs remain
prerequisites. This note is not independent approval, runtime proof, or source
authorization.
