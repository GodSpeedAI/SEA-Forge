# UI cache and app integration recon

Read-only source recon, 2026-10-05. This is a bounded test-first implementation proposal, not
implementation approval. Governing decisions: `ui-observation-boundary-proposal.md`,
`ui-observation-boundary-root-decision.md`, and `ui-observation-fixture-source-preparation.md`.

## Existing seams

- `apps/godspeed-cognitive-ui/src/ports/contract.ts:204-216` defines `CaseworkPort.subscribeEvents`
  with four arguments. A fifth optional typed callbacks argument is additive and keeps every
  existing call valid. Implementors include `adapters/http/httpCaseworkAdapter.ts:297`,
  `adapters/local/localAdapter.ts:168`, the conformance scenario, and fakes in
  `ui/journeys.test.tsx:374`. The local adapter can ignore the optional callback; use its source
  event only if it actually has the canonical validated type. Existing legacy `onEvent` must
  continue receiving the original event.
- HTTP native EventSource ingestion is `httpCaseworkAdapter.ts:297-378`; fetch/ReadableStream
  ingestion is `:380-447`; permissive `parseStreamEvent` is `:530-549`. Both currently filter
  against and update `lastCursor` before any observation-specific validation. Native's event
  allowlist currently omits `execution_observation`. Neither URL currently sends `case_id`.
  Both transport URL constructors must send requested `case_id`; tests should assert it and
  preserve ordinary replay behavior.
- The shared TS wire types are imported into the UI port from
  `.agents/reports/interface-contracts/typescript/types.ts:306-447`. They provide structural
  types, not runtime validation. Existing adapter `isRecord` only checks non-null object at
  `httpCaseworkAdapter.ts:78`; the generic event parser validates only event/payload records,
  cursor, and optional payload case equality. It is not sufficient for observation admission.
- `app/live.ts:16-140` owns each subscription. It has local `logs` and `snaps`; ordinary
  snapshot/progress/settlement handling starts at `:92`. `model/types.ts:399-407` has
  `ExecutionState`; `UiState.executions` is at `:467`; `model/store.ts:288-289` replaces an
  execution by plan-item key. `App.tsx:89-103` reconnects on `history.caseId` and returns the
  disconnect cleanup. `App.tsx:354-362` selects and renders `state.executions` against
  `liveSnap` regardless of `state.revision`, so a historical-time guard belongs at this final
  render selection (and must be tested at render level).

## Bounded behavior proposal

Define UI-only typed callback/update/notice shapes in the port layer; do not extend canonical
wire DTOs to describe local admission. Validate an observation strictly before cursor logic,
then call optional `onUpdate(originalValidatedEvent, admittedFramesByRun)` or deliver the legacy
event unchanged. The original envelope and all server retained/omitted counts remain intact.
Every admitted `run_id` is an own-property map entry, including `[]` for metadata-only or fully
deduplicated updates; a capacity-rejected run is absent. Map entries must correspond to unique
run IDs in the envelope and their frames must belong to that run. An admitted unavailable run
keeps its last validated annotation with an explicit unavailable indication; empty frames do
not clear or imply success.

One cache belongs to one subscription and survives native/fetch internal reconnects. Clear it
on unsubscribe. Bounds: 32 run keys, 4,096 pair identities globally, 1,024 per run. Evict
oldest cached identities at either identity bound. At key capacity, terminal-key LRU may evict
terminal entries only; never evict a nonterminal run to admit another nonterminal run. If no
terminal key can be evicted, reject that run's local admission, omit its map entry, emit a typed
UI-local capacity notice (not `onError`), and continue ordinary case revisions. Dedupe only
while the `(run_id,event_id)` identity remains cached; replay after eviction is permitted.
Represent identity as nested native `Map<run_id, Set<event_id>>` (or an unambiguous JSON tuple),
not delimiter concatenation: IDs are arbitrary strings and no new ID grammar is authorized.

Observation routing precedes the ordinary cursor filter in both transports. Keep the source
cursor/timestamp; never compare or mutate `lastCursor` for observations. Ordinary events retain
their existing comparison/update semantics. Include `execution_observation` in native listener
allowlist. Query `case_id` in both native and fetch URLs, with tests proving exact requested-case
scoping and reconnect URL behavior.

The app consumes only the typed update path for annotations and ignores observations in its
legacy event handler, avoiding duplicate application. Store sidecar remains ephemeral and
outside `WorldHistory`; replace annotations by real run/event IDs. Clear sidecar on case change
and `connectLive` disposal, in addition to the subscription cache cleanup. Keep captured
snapshot/run standing immutable and suppress current annotations/execution UI while
`state.revision !== nowRevision(state.history)`.

## Test-first assertion matrix

1. **Compatibility:** four-argument `subscribeEvents` callers and existing implementors remain
   valid; local/fake/conformance paths compile without implementing observation support.
2. **Validation before cursor:** valid observation delivers even with equal/stale/no ordinary
   cursor; does not alter reconnect `last` value. The next ordinary equal/stale frame is filtered
   as before and a later ordinary revision advances. Exercise native and fetch independently.
3. **Transport scope:** both URLs carry encoded `case_id`; wrong/missing observation case is
   rejected before delivery; native allowlist recognizes observation. Reconnect resumes from
   unchanged ordinary cursor.
4. **Canonical integrity:** callback receives the original validated envelope by identity/value,
   unchanged server counts and source timestamp/cursor. Legacy callback also gets original event.
5. **Validation matrix:** exact root/run/frame/budget keys; requested case; exact vocabularies;
   unique run IDs and frame pair identities; run ownership; max cohort/read/frame sizes; count
   relationships; optional `execution_status`/`exit_code` only on `command_finished`; finite,
   integral safe optional numbers; unsafe, fractional, negative and overflow arithmetic rejected.
   Metadata-only valid envelope remains deliverable.
6. **Admission/map:** admitted run owns a map property for `[]`, including metadata-only and
   duplicate-only updates; rejected run has no property. Verify unavailable rows retain previous
   annotation and show unavailable; empty frames do not erase or signal success.
7. **Dedupe/eviction:** `(run_id,event_id)` does not collide across delimiter-containing ID
   pairs; duplicates are suppressed while cached; oldest per-run/global identities evict at
   bounds and replay is allowed after eviction.
8. **Run-key policy:** terminal key LRU; active/nonterminal entries protected; all-protected
   capacity emits typed notice, not error, while later ordinary revision is delivered. Exercise
   exact 32-key/4096-global/1024-per-run edges.
9. **Lifecycle:** cache persists across internal reconnect; unsubscribe clears cache. App-side
   annotations clear on disconnect/dispose and case switch, independently of connection cache.
10. **Rendering/history:** sidecar is not serialized into `WorldHistory`; current annotation
    replaces by real IDs; historical selection hides every current execution annotation/pill/
    panel while captured historical standing remains visible. Assert actual rendered output.

## Unresolved contract precision

The canonical interface spec says “strict”/allowlisted metadata and calls for real identity
dedupe, but generated TS interfaces do not validate unknown keys. The conformance helper
`expectRunTraceObservationShape` in `interface-contracts/tests/contract-conformance.test.ts:232+`
checks exact keys and integer arithmetic, but uses `Number.isInteger`, not
`Number.isSafeInteger`, for `exit_code` and does not comprehensively validate every count/budget
relationship. The wire comments explicitly require `Number.isSafeInteger`; browser runtime
validator behavior is currently absent. The implementation fixture must apply a self-contained
strict UI validator without widening canonical types or assuming generated TS types validate.

The frame invariant `total_frame_count - retained_frame_count === omitted_frame_count` and
`truncated === (omitted_frame_count > 0)` are pinned in interface tests. For JS exactness, require
each operand to be a nonnegative safe integer and validate the subtraction without unsafe
intermediate arithmetic (e.g. reject totals whose exact relationship cannot be represented).
Root decision is still needed for malformed observation behavior (drop silently vs transport
error); the source-preparation note explicitly leaves this unapproved. Terminal-key predicate
also must follow the approved terminal statuses (completed/failed/terminated); do not infer
terminality from settlement alone or invent a freshness rule.

## Minimal ownership set

Unit 6 ingestion/cache: `src/ports/contract.ts`, `src/adapters/http/httpCaseworkAdapter.ts`,
`src/adapters/http/httpCaseworkAdapter.observations.test.ts`, and focused edits to
`src/adapters/http/httpCaseworkAdapter.nativeEvents.test.ts` for native URL/allowlist coverage.
Touch `src/adapters/local/localAdapter.ts` only if needed to preserve typed optional callback
compatibility. Avoid canonical wire DTO/type edits.

Unit 6b app/model/view: `src/app/live.ts`, `src/model/types.ts`, `src/model/store.ts`,
`src/app/App.tsx`, `src/app/live.observations.test.ts`, plus the existing actual-render test file
for historical suppression. Keep fixture ownership separate from unit 6.
