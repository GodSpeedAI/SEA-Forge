# Independent review: local subscription cursor adjudication

Date: 2026-10-06  
Verdict: **PARTIAL ACCEPTANCE; revise the proposed progress-cursor remedy and regression assertions before treating the cursor issue as resolved.** Documentation-only review; no implementation or tests were run.

## Reviewed identities and source basis

- Adjudication reviewed: `renderer-chunks-local-subscription-contract-adjudication-oct06.md`, SHA-256 `31e9ccb36f19fc21750c22782ebd11c4c5ff85ac0a7fd39ae2827f58135eb190`.
- Normative spec reviewed: `.agents/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md`, SHA-256 `6820481381435a6aea7acae829cabe0ad2cb12217108af620385cef013aa1bb3`.
- Mock source pattern reviewed: `.agents/reports/interface-contracts/typescript/mock-adapter.ts`, SHA-256 `6f0a878c00ed8d40c7c22afcfe363fd5bafa7d8fd2727189968db1b60888c4bf`.
- Relevant implementation evidence: `CaseworkPort` (`apps/godspeed-cognitive-ui/src/ports/contract.ts:204-216`), local adapter (`src/adapters/local/localAdapter.ts:168-173,347-399,433-435`), HTTP routing (`src/adapters/http/httpCaseworkAdapter.ts:340-359,392-427`), shared conformance (`src/adapters/conformance/caseworkPortConformance.ts:90-139`), local progress test (`src/adapters/local/localAdapter.test.ts:179-191`), T09 extension (`.agents/reports/casework-live-wiring/t09-contract-extension-proposal.md:17,99-103`).

Graft retrieval located the conformance, adapter, and subscription path before direct source verification. No source, test, status, debt, Git, build, or runtime action was performed.

## Findings accepted

### A. The future-only conformance setup compares different cursors

The test captures the trajectory head at `caseworkPortConformance.ts:109`, but subscribes using the earlier `snapshot.cursor` at `:111-115`; it then requires every received event to be greater than the captured head at `:134-138`. After the accepted intent these values can differ. A progress event at the captured head is beyond the cursor actually supplied, so this test cannot classify that event as replay relative to its own `sinceCursor`.

The minimal conformance correction is to subscribe from `headCursorBeforeResume` in this future-only branch and keep the strict `event.cursor > headCursorBeforeResume` assertion. This makes the resume boundary and assertion describe the same future-only contract.

### B. Equal or older ordinary events are suppressed by the HTTP adapter

The native route initializes `lastCursor` from `sinceCursor` and drops `cursor <= lastCursor` (`httpCaseworkAdapter.ts:309-311,340-347`). Fetch streaming delivers only `event.cursor > lastCursor` (`:418-426`). The §6.2 recovery examples reconnect from the last delivered cursor and continue with later cursor values (normative spec `:204-225`). `CaseworkPort` calls the parameter `sinceCursor` but does not itself document inclusive/exclusive equality (`contract.ts:210-215`), so exclusivity is an inference from the normative recovery example and both concrete HTTP consumers, not an explicit comment on the interface.

With that qualification, an ordinary `execution_progress` at exactly the subscriber’s supplied/current cursor is not a new delivered event under the existing clients. The distinct `execution_observation` same-head exemption in §6.3 (`specification:227-235`) does not extend to progress events.

### C. The stated conformance symptom is consistent with late scheduled local delivery

`LocalContractAdapter.subscribeEvents` ignores `_since` and registers its listener directly (`localAdapter.ts:168-173`). Execution schedules delayed progress phases (`:347-366`); the callback reads the then-current snapshot cursor (`:391-399`); listener delivery itself is deferred at zero delay (`:433-435`). Consequently an earlier dispatch’s progress callback can reach a newly registered listener at the current head. This supports the adjudication’s diagnosis of the local subscription boundary and scheduling race.

## Mandatory qualification to the proposed remedy

The adjudication’s steps 1 and 2 are a coherent immediate correction: use the actual head as the future-only resume point and make the local subscription honor its exclusive resume boundary / monotonic ordinary-event rule. Its step 3 and deterministic test requirement are not yet a source-grounded implementation contract.

The adjudication proposes giving progress events a strictly advancing cursor while asserting that the case-revision cursor does not change merely because progress was observed (`adjudication:82-93,105-117`). `StreamEvent` exposes only one `cursor`; `CaseworkPort.subscribeEvents` accepts that same cursor type as `sinceCursor` (`contract.ts:204-216`). The local adapter currently has no separate stream-cursor allocator: progress reuses the current snapshot cursor. A new stream-only cursor that can advance independently of case revisions would need an explicit rule for how it is ordered against subsequent snapshot cursors, what a `sinceCursor` means across those domains, and how reconnect/replay handles that cursor. The adjudication acknowledges that it does not prescribe this mechanism, but the proposed test assumes the mechanism exists.

The cited canonical mock does not prove the proposed “progress advances while case-revision head remains unchanged” behavior. Its `nextCursor()` increments the same `seq` used to create snapshot cursors (`mock-adapter.ts:76-79,369-370`); each progress phase calls `nextCursor()` (`:325-340`); and its trajectory reports that `seq` as `CurrentHead` (`:250-288`). Thus the mock’s progress advances the cursor view used by snapshots/trajectory. It is evidence that its mock stream assigns advancing cursor values, but not evidence for an independent stream cursor that leaves the case-revision head unchanged.

The existing local adapter test explicitly expects at least one progress event (`localAdapter.test.ts:179-191`). Under the HTTP adapter’s strict monotonic filter, local progress that reuses the cursor of an already delivered snapshot will be discarded. The source therefore has two coupled obligations: local subscription boundary conformance and a defined cursor model for any progress that is meant to remain visible after a snapshot at the same head. A timer test cannot resolve that contract choice by manufacturing a cursor.

**Required before approving the full remedy:** root/spec owners must either (a) define and authorize the separate ordered event-cursor/reconnect model, including its relationship to case snapshot cursors, or (b) keep local progress on real case cursors and treat equal/older progress as filtered, revising the local progress behavior/test expectation only through an explicit contract decision. Do not claim that the current mock settles case-revision preservation.

## Smallest safe immediate remedy and deterministic test plan

For the present mismatch, correct the future-only conformance branch to subscribe from captured head `H`, and have local subscription delivery reject ordinary events at or behind its last cursor. Keep the strict `> H` assertion. This does not assign a synthetic cursor to progress.

Use the existing manual timer harness pattern in `httpCaseworkAdapter.nativeEvents.test.ts:4-39`:

1. Use a fresh local adapter and record its current trajectory head `H`. Start/arrange an execution progress timer whose local progress callback will carry the unchanged cursor `H`; subscribe in future-only mode with `sinceCursor=H` before advancing that timer.
2. Advance only the scheduled progress timer and its zero-delay delivery timer. Assert that an event at `H` is not delivered. Also inject or arrange an older-cursor ordinary frame and assert it is not delivered.
3. Cause one genuine case mutation to create `H2 > H`; drain its delivery timer and assert the snapshot at `H2` is received once and in order. Confirm trajectory history contains that genuine revision, with no extra revision appended by progress delivery.
4. Drain a pending progress callback that still carries `H2`; assert it does not duplicate an already delivered ordinary cursor. Unsubscribe before draining any remaining timers and assert no callback occurs afterward.

This test establishes the immediate strict-boundary behavior and the current local limitation honestly. It does **not** prove that later progress phases should be visible with a new cursor. If that visibility is required, add a separate test only after an authorized cursor/replay contract specifies the source of the advancing cursor and its ordering against real case revisions. Preserve the existing progress expectation as an explicit unresolved mismatch until that decision; do not weaken it silently.

## Uncertainty and conclusion

The exclusive `sinceCursor` interpretation is strongly supported by current HTTP behavior and §6.2, but the port signature does not spell it out. The future-only test setup defect is direct. Progress visibility and cursor ownership remain under-specified: §6.1 illustrates a progress cursor after a snapshot, while §6.3 explicitly limits non-advancing same-head treatment to `execution_observation`; neither section explains how independent progress cursors replay alongside revision history. The adjudication correctly rejects equal-head delivery for an ordinary subscription, but its proposed independent cursor plus unchanged revision-head assertion exceeds the evidence currently supplied.

No implementation, test, or build is authorized or claimed by this review. Root retains semantic decision and release authority.
