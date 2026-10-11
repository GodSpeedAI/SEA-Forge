# Local ordered-cursor proposal revision 2 — independent review

Date: 2026-10-06  
Verdict: **REJECT pending one bounded subscription-input clarification.** This is a document-only review. No source, tests, or runtime gates were changed or run.

## Frozen identities and review boundary

- Original architecture assignment: `local-subscription-ordered-cursor-proposal-original-assignment-oct06.md`, SHA-256 `6028c2e99831f67a5ab3b974e955fd9b1fc6d7e1ac7c012ef93944bcd409d4c8`.
- Revision 2 repair assignment: `local-subscription-ordered-cursor-revision2-original-assignment-oct06.md`, SHA-256 `6375957abaffb8c9b051133776869ab8d63b01405b6eb35cdfe7a56418ecabde`.
- Revision 2 proposal: `local-subscription-ordered-cursor-concrete-proposal-revision2-oct06.md`, SHA-256 `a46ece1d0feb86ca679f485c4cf32ee566bff531f7adcc8fa1d630503d248456`.
- Prior rejection: `local-subscription-ordered-cursor-independent-review-oct06.md`, SHA-256 `fe154fbe0e8a8279cf9a300c79178c40a17e9f55412bfdf20f5e6426e3dc682c`.

The proposal is compared with both original assignments, the prior rejection and root's revision 2 decisions, plus the cited local adapter, port, conformance, HTTP cursor, and normative source. This verdict concerns proposal completeness only; it does not authorize code or prove runtime behavior.

## The four prior findings are addressed

1. **Allocation linearization:** the proposal now computes a candidate from previous sequence `p`, validates the candidate against `p`, prepares a snapshot before mutation, then commits the same cursor and publishes it without comparing against the advanced frontier (`revision2:47-58`). It gives failure atomicity for preparation/allocation failures and states the limitation for catastrophic native allocation failure (`:50-54,129-130`). The allocation table includes snapshot, progress, completion, settlement snapshot, and settlement side event (`:60-69`). This directly fixes the prior high-water self-rejection.
2. **Bounded per-subscriber queue:** each FIFO is capped at 64 pending events. Attempt 65 disposes only the subscriber, clears pending items/timer, and calls its error callback once with a completeness warning; other subscribers continue. Reentrancy, self-disposal, callback throws, error-callback throws, and timer scheduling failure are specified and included in test cases (`:73-82,100-105`). This is the exact local bound root assigned, not a process-wide subscriber-memory guarantee, and the proposal says so (`:82,130`).
3. **Cursor width:** the proposal correctly states that decimal grammar and `padStart(10)` establish no existing ten-digit maximum. It marks `9_999_999_999` as a new local operational ceiling chosen by root, uses fixed-width generated sequences under that ceiling, forbids wrap, and specifies allocation-exhaustion tests (`:32-34,55-58,94-95,119`).
4. **Supplied cursor cases:** malformed, unsafe, above-ceiling and different-numeric-epoch strings are rejected through one error without registration; safe same-epoch future floors are accepted without replay/fabricated events; numeric comparison and leading-zero behavior are explicit; seed validation is covered (`:36-40,73-75,91-95`).

The root-approved retained-revision-head/ordinary-stream-frontier distinction is also implemented consistently in the proposal. The future-only branch captures private allocator frontier `F` and uses the maximum with the supplied exclusive boundary; it explicitly requires the controlled queued `H+1` event to be suppressed when subscription occurs at revision head `H` (`:73-75,87-89,106`). The retained-replay branch remains unchanged (`:75,113,127`). This matches the source split: `CaseworkPort` does not require stream cursors to resolve to retained snapshots (`ports/contract.ts:204-216`); local `getSnapshotAt` does exact snapshot lookup and trajectory head derives from the last point (`adapters/local/localAdapter.ts:146-166`). Normative §6.1 gives progress its own later cursor while §6.3 keeps `execution_observation` at the latest real case cursor (`04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:195-235`). The corrected proposal does not conflate those cases.

The proposal preserves visible progress/settlement, settlement-after-snapshot ordering, immutable prior revisions, exact snapshot lookup, and case-local allocator ownership (`:60-69,86-89,115-118`). Its exact-source anchors identify the current behavior that must change: initial/template state and padding (`localAdapter.ts:52-99`), exact snapshot lookup and trajectory head (`:146-166`), subscription (`:168-173`), template-created state (`:312-342`), execution and settlement order (`:347-399`), append and publication (`:401-435`), current progress cursor reuse (`:391-399`), and existing visible progress/settlement assertions (`localAdapter.test.ts:162-192`). The manual-timer helper is a concrete precedent (`httpCaseworkAdapter.nativeEvents.test.ts:4-39`). No unsupported HTTP replay/frontier change is requested.

## Remaining blocking clarification: `sinceCursor === undefined`

The source port explicitly accepts an omitted boundary: `CaseworkPort.subscribeEvents(caseId, sinceCursor: string | undefined, onEvent, onError)` (`ports/contract.ts:210-216`). Existing concrete local callers use `undefined` (for example `localAdapter.test.ts:179-192`), and the adapter's current concrete signature also accepts it (`localAdapter.ts:168`). Revision 2 declares `sinceCursor: string | undefined` in the unchanged interface, but then says “Parse and validate `sinceCursor` synchronously” and sets `exclusiveFloor = max(parsedSinceSequence, F)` (`revision2:73-75`) without defining the value or path for `undefined`.

That leaves an ordinary existing callsite without a defined registration outcome, despite the proposal's claim of exact input behavior and concrete future-only semantics. The required repair is small: state that `undefined` means no supplied exclusive floor and therefore uses captured allocator frontier `F`, with no replay; provide the normal unsubscribe closure; and test that omitted-boundary subscription suppresses an already queued event and receives a later genuine event. It must not turn `undefined` into an error or weaken the `H+1` supplied-head test. The proposal may keep the optional/no-op fallback for concrete three-argument callers, while retaining the port signature unchanged.

This is the only unresolved material issue I found. The review does not reopen the rejected proposal's four findings and does not repeat the earlier false concern about needing an HTTP public-frontier getter; the future-only and retained-replay branches are distinct in the actual conformance fixture (`caseworkPortConformance.ts:109-142`).

## Verdict

**Reject pending the omitted-boundary rule and its deterministic case.** All four prior rejection findings and the root-directed event-frontier decisions are otherwise addressed with concrete state, ordering, failure behavior, and test obligations. Preserve this record and the earlier rejection as immutable history. No implementation or runtime approval follows this document.
