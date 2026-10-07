# Local ordered-cursor proposal revision 3

Date: 2026-10-06  
Status: complete DOCONLY repair for independent review; no implementation is released.

## Decision and scope

Use one monotonically allocated ordinary-event ordinal per authored local case. Retained world snapshots and trajectory points are a subset of that sequence. They retain their original cursors and immutable facts. Simulated progress and standalone settlement events consume distinct ordinals without becoming revision points. A subsequent snapshot allocates after those events. `getSnapshotAt` remains an exact lookup among retained snapshots. This is local simulation only: it creates no kernel cursor, changes no `execution_observation` behavior, and changes no public DTO, port signature, HTTP behavior, schema, or generator.

The trajectory head remains the last retained revision cursor; the ordinary stream frontier can be later. This matches the present port, which defines event subscription without promising each event cursor resolves as a snapshot (`apps/godspeed-cognitive-ui/src/ports/contract.ts:204-216`); `getSnapshotAt` is exact lookup and trajectory head comes from the final point (`apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts:146-166`). Normative §6.1 gives progress a later cursor, while §6.3 preserves the unchanged-cursor `execution_observation` exception (`apps/godspeed-cognitive-ui/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:195-235`). This is an authored-local implementation rule, not a new public frontier promise.

## Private state, initialization, and cursor rules

Each case owns its allocator and subscriber records; there is no process-wide counter, timestamp, or run identifier in ordering.

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
  queue: StreamEvent[] // at most 64 pending ordinary events
  drainTimer: ReturnType<typeof setTimeout> | undefined
  draining: boolean
}
```

Seed each initial case from its final snapshot cursor only after checking that the final trajectory point has the same exact cursor string. A template-created case seeds from its initial committed snapshot. Validate seeds by the same syntax/safety/range rules as supplied cursors; invalid fixture state fails closed before installing the case. Do not renumber history.

The existing cursor grammar is decimal `epoch.sequence` (`apps/godspeed-cognitive-ui/src/ports/types.ts:104`). `padStart(10, '0')` is minimum formatting width, not a grammar maximum (`localAdapter.ts:69`). Root selects a **proposed private authored-local ceiling** of sequence `9_999_999_999` to preserve numeric comparability with ten-padded fixture/consumer ordering. It is neither a normative wire maximum nor an implemented policy here. Never wrap or increment epoch. Generated cursors retain seed epoch text and use at least ten sequence digits.

For a supplied string, accept only decimal `epoch.sequence`, safe integer components, matching numeric epoch, and sequence at or below the local proposal ceiling. Leading-zero forms compare by numeric value; stored history and seed text are not rewritten. Different epochs, malformed or unsafe components, and above-ceiling input produce exactly one `onError` and no registration. A safe same-epoch future sequence within the ceiling is a valid exclusive floor: it causes no replay or fabricated event, and delivery begins only after a genuine allocation exceeds it. `undefined` is separately defined below and is not an invalid string.

## Allocate and publish exactly once

All ordinary events use one synchronous private `allocateAndPublish` path. Given prior frontier `p`, check exhaustion, form `candidate = p + 1`, format it, and validate monotonicity against **p before advancing**. Prepare all fallible event/snapshot material first. Then commit `sequence = candidate`, append prepared snapshot and trajectory point if this is a revision, and publish using that same candidate cursor. No callback or `await` occurs between commit and publication; no other site increments or publishes ordinary events. This removes the previous self-comparison bug where a cursor could be compared against the already-advanced frontier.

For snapshot mutation, clone/edit the prior snapshot, prepare the full new snapshot and point locally, and prepare the candidate cursor before touching `snaps`, `points`, or allocator state. On preparation or allocation failure, change none of those and return no accepted cursor. On success, commit cursor and prepared history, then publish. Catastrophic runtime failure during native array mutation is outside the recoverability promise; ordinary thrown preparation/allocation errors are atomic. Exhaustion fails closed without partial revision/history/frontier change.

| Local operation | Cursor allocations | Retained revision effect |
|---|---:|---|
| Snapshot append or committed template snapshot | 1 | Append one immutable snapshot and matching trajectory point. |
| Each progress timer callback | 1 | None. |
| Completion snapshot | 1 | Append one immutable snapshot and matching point. |
| Settlement snapshot | 1 | Append one immutable snapshot and matching point. |
| `settlement_recorded` after settlement snapshot | 1 additional | None. |

Keep current settlement ordering: settlement snapshot first, side event second (`localAdapter.ts:378-387`). Keep visible progress callbacks (`:391-399`). For example, after revision `.0042`, a mutation `.0043`, progress `.0044/.0045`, completion `.0046`, settling progress `.0047`, settlement snapshot `.0048`, and side event `.0049` leave revision head `.0048`; the next snapshot is `.0050`. Illustrative values use the case's real epoch and seed.

## Subscription semantics, including omitted boundary

Keep the public port signature exactly `subscribeEvents(caseId, sinceCursor: string | undefined, onEvent, onError)` (`ports/contract.ts:210-216`). For concrete adapter callers that omit the fourth argument, preserve the existing three-argument call compatibility with a private no-op fallback error callback. The port itself remains unchanged.

At subscription invocation, first find the case and validate a supplied string if one exists. Capture the case's current ordinary allocator frontier `F` synchronously. If `sinceCursor` is `undefined`, this means **no supplied floor**: set the exclusive floor to `F`. If a valid string is supplied, set the floor to `max(parsed numeric sequence, F)`. Both paths are future-only: no retained replay and no callback for already allocated work, including an event whose timer is queued but has not delivered. Register the subscriber and return its ordinary unsubscribe closure. Missing case or invalid supplied cursor reports once and returns a harmless closure without active registration.

The floor is a delivery boundary, not an allocation. An event must have a genuine allocated cursor strictly above it. A safe future supplied floor waits silently until genuine allocation exceeds it. For `undefined`, the captured `F` protects against a pre-subscription queued event. The controlled test must allocate and queue H+1, subscribe with `undefined`, run that old timer and prove no delivery, then allocate H+2 and prove delivery. Keep a separate supplied-H regression: allocate/queue H+1, subscribe with supplied revision head H while frontier is H+1, suppress queued H+1, and deliver later H+2. Do not substitute one case for the other or weaken the supplied-H+1 test.

Shared conformance keeps its future-only branch passing its captured current head; the adapter's private `F` also handles progress/settlement beyond retained trajectory head. The retained-replay branch continues using its original resume cursor and all original expectations. No HTTP public frontier getter or replay changes.

## FIFO delivery, cleanup, and failures

On publication, synchronously visit active case subscribers in registration order. Append each event to each FIFO. At most one zero-delay drain timer exists per subscriber. The drain checks `active` before every callback, dequeues FIFO order, suppresses cursors `<= lastDelivered`, and advances `lastDelivered` before invoking `onEvent`. If the callback unsubscribes itself, stop and clear remaining events. Unsubscribe marks inactive, removes the record, cancels its timer, and clears its FIFO; the active check also protects an already-started drain.

Each subscriber has a proposed capacity of 64 **pending** ordinary events. On attempted enqueue 65, dispose only that subscriber, cancel its timer, clear queued items, and call `onError` exactly once with a message that the backlog exceeded 64 and complete delivery is unavailable. Do not silently lose an event, evict progress/settlement, or report complete delivery. Other subscribers proceed independently. Reentrant publication from `onEvent` appends to the same FIFO and cannot create another drain timer. Catch an `onEvent` throw, report once to that subscriber, dispose it, and continue other subscribers. Contain throws from `onError`. A timer-scheduling throw likewise disposes and reports once. Callback failure does not roll back a cursor already committed for publication. Allocator corruption/nonmonotonicity fails closed before publication and reports once to affected active subscribers.

The capacity is a proposed per-subscriber pending-item bound, not a process-wide memory bound or subscriber-count limit. `execution_observation` remains outside this ordinary allocator/filter; its normative same-real-cursor behavior must not advance `lastDelivered` if ever emitted locally. This proposal does not add it.

## Deterministic test-first matrix for a later authorized implementation

This document authorizes no source or test edit. If a later task releases implementation, limit it to the local adapter, its tests, and the necessary future-only branch of shared conformance. Use a manually controlled timer (precedent: `httpCaseworkAdapter.nativeEvents.test.ts:4-39`), never sleeps/retry luck.

1. **Undefined boundary regression:** queue H+1 before subscribing with `undefined`; flush old timer and assert suppression; allocate genuine H+2 and assert one event and an unsubscribe function. This directly covers the only revision 2 review finding.
2. **Supplied boundary regression:** queue H+1, subscribe with supplied revision H, suppress H+1, deliver H+2. Keep this explicit and separate from case 1.
3. **Exclusive/future/invalid inputs:** stale/equal suppressed; future same-epoch floor waits until real allocation exceeds it; malformed, unsafe, different epoch, and above-ceiling produce exactly one error and no active subscriber; leading zeros compare numerically without rewriting cursor strings.
4. **Ordering and behavior:** manually advance progress, completion, settling progress, settlement snapshot, settlement side event, then another snapshot. Assert unique strictly increasing ordinary cursors, visible progress, settlement side event after snapshot, next snapshot after every preceding event, and unchanged original progress/settlement expectations (`localAdapter.test.ts:162-192`).
5. **Immutable history and split head:** retain old snapshot/trajectory results, emit progress, prove they are unchanged; exact lookup still resolves only snapshots; a later snapshot appends one point above intervening events; settlement side event remains unresolvable as a snapshot.
6. **Initialization and isolation:** verify initial/template seed matches final snapshot and point; corrupt/malformed seed fails closed; advancing one case does not affect another.
7. **Bounded/reentrant FIFO:** 64 events ordered; event 65 produces one backlog error, disposes only that subscriber, clears queue/timer; another subscriber continues. Cover reentrant enqueue, self-disposal, throwing `onEvent`, throwing `onError`, and timer scheduling failure.
8. **Exhaustion atomicity:** at `.9999999998`, allocate `.9999999999`; next snapshot fails with snapshots, points, frontier, and accepted cursor unchanged. Progress/settlement exhaustion reports once without an event. No wrap.
9. **Replay preservation:** retained-replay conformance retains original resume cursor, exact mutation cursor, and strict replay prefix. Only future-only conformance supplies captured current head.

## Source anchors, decision trace, and deviations

Relevant implementation anchors: initial cases and cursor formatting `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts:52-99`; lookup/head `:140-166`; subscription `:168-173`; template initialization `:312-342`; execution, settlement and progress `:347-399`; append/publication `:401-435`; cursor grammar and stream event kinds `apps/godspeed-cognitive-ui/src/ports/types.ts:104,426-449`; port `apps/godspeed-cognitive-ui/src/ports/contract.ts:204-216`; conformance future-only/replay branches `apps/godspeed-cognitive-ui/src/ports/caseworkPortConformance.ts:109-142`; visible progress/settlement tests `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.test.ts:162-192`; controlled timer precedent `apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.nativeEvents.test.ts:4-39`.

Revision 2 already repaired the original independent rejection's four findings: allocate/publish linearization and snapshot preparation atomicity; per-subscriber bounded FIFO/error/disposal; correction that ten digits are minimum padding plus a proposed private ceiling; and exact malformed/unsafe/epoch/future/leading-zero input rules. It also preserved immutable revisions, visible progress/settlement, local case isolation, and distinct replay versus future-only behavior. Revision 2 independent review found exactly one remaining omission: `sinceCursor === undefined` had no registration/floor semantics despite existing port callers using it. Revision 3 now defines it as no supplied floor, uses captured `F`, returns ordinary unsubscribe, never replays, and adds its own controlled queued-event regression while retaining the separate supplied-H test.

Material deviations from revision 2: only the omitted-boundary behavior and its direct deterministic test were added. Prior limits and accepted root semantics are carried forward without broadening them. The 64-event FIFO and `9_999_999_999` ceiling remain **proposals only** pending review and approval; no policy is implemented or assumed by this document. No tests, source, public contract, normative spec, or shared conformance file changed. No test/compiler/scanner/gate/Git command was run. A fresh independent critic must review the original assignments, complete revision 2 and review, and this complete revision 3 before any source task is considered.

Uncertainty remains limited to callsite compatibility for concrete three-argument callers and catastrophic native allocation failure, as previously disclosed. Verify callers if implementation is later authorized. This document is a complete reviewable proposal, not independent acceptance, runtime proof, or authorization to change public/kernel cursor behavior.
