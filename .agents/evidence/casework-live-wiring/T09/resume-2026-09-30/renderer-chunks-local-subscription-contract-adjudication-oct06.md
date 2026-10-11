# Local subscription cursor adjudication — 2026-10-06

## Verdict

Under the current Casework event-stream contract, an `execution_progress` event
whose cursor equals the subscriber's exclusive resume cursor is not a valid
delivered event. Ordinary stream events use strictly advancing cursors; the
client treats a cursor at or behind its last delivered cursor as replay and
filters it. `execution_observation` has a separate, explicit same-head rule,
which does not extend to `execution_progress`.

The observed conformance failure also contains a test setup defect: its
future-only subscription passes the older initial snapshot cursor while
asserting against the newer head captured immediately before subscription. A
progress event at that newer head is beyond the supplied resume cursor, even if
it was already the current case head. The test therefore cannot call that event
a replay relative to the cursor it actually supplied.

## Contract and source evidence

The normative [React Cognitive Environment Contract Specification §6.1–6.3](../../../../reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md)
defines the stream as monotonic, shows `execution_progress` at cursor
`1.0000001205` after a snapshot at `1.0000001204`, and resumes a dropped stream
from its latest delivered cursor (§6.2, lines 195–231). The TypeScript
`CaseworkPort.subscribeEvents` surface names its second parameter `sinceCursor`
([contract.ts:204–216](../../../../../../apps/godspeed-cognitive-ui/src/ports/contract.ts#L204)).
The interface does not independently spell out whether equality is inclusive;
the normative event examples, conformance assertions, and concrete consumers
settle that boundary as exclusive for ordinary cursor-bearing events.

The canonical `MockCaseworkAdapter` allocates `nextCursor()` for each progress
phase before emitting it ([mock-adapter.ts:315–349](../../../../reports/interface-contracts/typescript/mock-adapter.ts#L315)).
The HTTP adapter initializes its resume cursor from `sinceCursor`, rejects any
event with `cursor <= lastCursor`, and advances `lastCursor` only for accepted
events ([httpCaseworkAdapter.ts:297–355](../../../../../../apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.ts#L297)).
Thus an equal-head progress frame is discarded by the standard client even if
its wall-clock timestamp is newer.

The later §6.3 exception is specifically for `execution_observation`: it copies
the latest real case cursor, must not advance that cursor, and is routed before
ordinary cursor comparison ([specification lines 227–233](../../../../reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md#L227)).
No equivalent exception exists for `execution_progress`. The T09 extension
proposal also keeps `execution_progress` only for sources that provide truthful
numeric progress; it does not redefine its cursor semantics
([proposal lines 17 and 99–103](../../../../reports/casework-live-wiring/t09-contract-extension-proposal.md#L17)).

## Failure mechanism

`LocalContractAdapter.subscribeEvents` ignores `_since` and registers the raw
callback ([localAdapter.ts:168–173](../../../../../../apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts#L168)).
After an accepted `EXECUTE_ITEM`, `execute` schedules progress timers
([localAdapter.ts:347–366](../../../../../../apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts#L347));
`progress` reads the current snapshot cursor when its timer fires
([localAdapter.ts:391–399](../../../../../../apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts#L391));
`emit` then schedules listener delivery at delay zero
([localAdapter.ts:433–435](../../../../../../apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts#L433)).
So a progress phase scheduled by an earlier dispatch may be delivered after a
new listener is added, carrying the unchanged case head.

The conformance scenario captures `headCursorBeforeResume` at line 109, but
subscribes with `snapshot.cursor` at lines 111–115, then requires every callback
cursor to exceed the captured head at lines 134–138
([caseworkPortConformance.ts](../../../../../../apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts#L109)).
Those two cursors need not be equal: the accepted intent already advanced the
case between the initial snapshot and this resume. A current-head progress
callback can therefore satisfy `event.cursor > supplied sinceCursor` and still
fail `event.cursor > headCursorBeforeResume`.

The recorded canonical run saw one event at the captured head followed by
duplicate later cursors. The diagnostic correctly isolates the equal-head
event as the failing one and records one passing isolated rerun; that rerun does
not establish determinism or clear the canonical gate
([phase-2 diagnosis](renderer-chunks-phase2-diagnostic-oct06.md)). The separate
listener test's sandbox `EPERM` is unrelated to this cursor finding.

## Smallest coherent remedy

1. In the future-only conformance branch, pass the captured current head as
   `sinceCursor`, rather than the original snapshot cursor. Keep the strict
   future cursor assertion.
2. Make the local adapter honor its resume boundary and suppress ordinary
   events whose cursor is less than or equal to that boundary.
3. Preserve visible progress for an active subscription by assigning progress
   events a cursor that strictly advances the ordinary stream, as the canonical
   mock does. Do not deliver equal-head progress and do not exempt it from
   comparison. Keep the special non-advancing handling limited to
   `execution_observation`.

The second and third changes are both needed for contract behavior: filtering
prevents duplicate delivery at a resume boundary, while a new monotonic progress
cursor prevents the ordinary HTTP client from discarding progress immediately
after an already-delivered snapshot. Cursor allocation must remain ordered with
subsequent case revisions; this adjudication does not prescribe its source
implementation.

## Deterministic regression fixture plan

Use a scoped manual timer queue, following the existing helper pattern in
`httpCaseworkAdapter.nativeEvents.test.ts:4–39`. Do not wait on wall-clock sleeps
or rely on a retry passing.

1. Create the local adapter with a fixed execution speed, capture current head
   `H`, subscribe using `H`, and trigger one accepted execution with the required
   setup timers driven explicitly.
2. Advance the manual clock to the first progress phase and then drain its
   zero-delay listener-delivery timer. Assert the phase is delivered once with a
   cursor strictly greater than `H`; assert the case-revision cursor did not
   change merely because progress was observed.
3. Manually inject/deliver a duplicate progress event at `H` and one older event
   below `H`; assert neither reaches a subscription resumed from `H`. Then
   deliver a progress event at the next stream cursor and assert it is accepted.
4. Assert callback cursor order is strictly increasing and unsubscribe before
   draining any remaining callbacks; no callback may arrive after disposal.

This distinguishes the intended behavior from both defects: a newly timed
callback cannot reuse the head cursor, and a stale/equal cursor cannot be made
"future" by arriving after subscription. Existing history, payload, and cursor
replay assertions remain intact.

## Limits and uncertainty

The exclusive equality rule is an inference from the normative monotonic-stream
section plus the canonical mock, HTTP consumer, and conformance behavior; the
`CaseworkPort` method comment itself does not define inclusive/exclusive
comparison. The current sources disagree: local delivery reuses the head and
ignores `sinceCursor`, while the mock and HTTP adapter require advancing cursors.
If product intent is instead to permit equal-head progress, that exception must
be added to the normative contract and all consumers/conformance rules together;
the existing `execution_observation` exception is not enough evidence to infer
it. No source, test, configuration, dependency, Git, status, debt, or gate was
changed or run for this adjudication. No canonical-gate pass is claimed.
