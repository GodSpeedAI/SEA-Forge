# Local ordinary cursor ordering Phase 1 repair — independent source review

Date: 2026-10-06  
Verdict: **REJECT; compile blockers and ineffective floor REDs remain.** Source-only review; no compilation, runtime, or test claim.

## Frozen inputs and review boundary

- Original Phase 1 assignment: `local-cursor-order-phase1-original-assignment-oct06.md`.
- Fresh fixture-repair assignment: `local-cursor-order-fixture-repair-original-assignment-oct06.md`.
- Prior independent rejection: `local-cursor-order-phase1-independent-review-oct06.md`.
- Accepted architecture proposal: `local-subscription-ordered-cursor-concrete-proposal-revision4-oct06.md`, SHA-256 `6d7baafd41a098cc24556b32ea222c999ffe4ea116998740339acd2086131873`; its review's proposal-line references are subject to `local-subscription-ordered-cursor-revision4-review-citation-erratum-oct06.md`.
- Repaired local fixture: `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorOrder.test.ts`, SHA-256 `d1b3421f821f6ef671ab5595c060bdc006bcfdf3fbcc8c7c47cf33d38a0777cf`.
- Preserved shared conformance file: `apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts`, SHA-256 `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c`.
- Frozen production source: `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts`, SHA-256 `d8b0feb02e489587672968c7450373f000ba02e3fd3d0d509888ca1e0f595f34`.

I read the complete two assignments, prior rejection, accepted proposal/review/erratum, full repaired fixture, full affected conformance source, and relevant production/API spans. The read-only diff confirms conformance remains exactly the intended branch-only argument change: future-only passes `headCursorBeforeResume`; retained replay passes `snapshot.cursor`; replay assertions and timeout are unchanged (`caseworkPortConformance.ts:109-150`). Production `localAdapter.ts` matches the frozen source hash and was not changed.

## The six prior fixture findings

1. **History-test timer sequencing: repaired.** The fixture now captures `beforePending`, runs the manual timer, and only then awaits at `localAdapter.cursorOrder.test.ts:190-192`. The later trajectory/snapshot reads likewise run their timers before awaiting (`:207-228`). The independent-case head reads also follow that order (`:240-245`). This removes the prior hang.
2. **All-event cursor ordering and settlement distinction: repaired.** The execution test now includes every event cursor, including progress, in uniqueness and numeric epoch/sequence ordering (`:162-172`); it requires the settlement side event to advance beyond the preceding snapshot (`:170-172`) and the subsequent snapshot to advance beyond the last event (`:173-179`). It separately verifies the settlement side-event cursor does not resolve as a snapshot (`:154-161`). This now agrees with revision 4's ordinary-event and retained-revision distinction.
3. **Draining callbacks for both subscribers: structurally repaired.** Both H+2 cases call `await drain(timers)` before asserting the target subscriber's delivery (`:100-106,129-132`), instead of assuming one timer handles every subscriber. The omitted-boundary assertion does await an asynchronous test callback only after the compile error listed below is fixed.
4. **Timer bounds and ordering: repaired by inspection.** Scheduling is capped at 500 callbacks, virtual advancement at 60 seconds, drain execution at 500 callbacks, and timers are selected by due time then insertion order (`:6-43,62-70`). Every test restores global timer functions in `finally`. No sleeps or retry loops were added.
5. **Independent case allocator state: repaired by inspection.** The test reads both case heads and requires each synchronous append to equal that case's own `nextCursor` (`:240-255`), then verifies case-local event delivery (`:246-257`). This is stronger than routing/truthiness alone and would expose a shared counter advancing the second case.
6. **Immutable history and exact lookup: repaired by inspection.** It captures the initial trajectory and exact snapshots, emits a progress event, compares trajectory and snapshots unchanged, appends one revision/point, and requires the progress cursor lookup to fail (`:186-228`). Execution also checks the settlement side-event cursor against exact lookup (`:154-161`). These assertions are not runtime proof and have not been executed.

## Remaining blocking findings

### 1. Omitted-boundary test is syntactically invalid and calls a nonexistent concrete overload

The first `test` callback is declared non-async (`localAdapter.cursorOrder.test.ts:85`), but contains `await drain(timers)` at lines 101 and 105. This is a TypeScript syntax/type error: `await` is not permitted in that callback as written. The same test calls `adapter.subscribeEvents` with four arguments at line 94. The variable is concretely typed as `LocalContractAdapter`; its actual method takes three parameters (`localAdapter.ts:168-173`). The four-argument `onError` parameter exists on the `CaseworkPort` interface (`ports/contract.ts:204-216`), but that does not add a four-argument call signature to a variable typed as the concrete class. The fixture therefore cannot compile against the current concrete API. Mark the callback async and invoke the API through an existing `CaseworkPort`-typed reference (or otherwise keep within existing public types); do not add a production overload, which is outside the repair assignment.

These are source-level conclusions only; I did not invoke a compiler.

### 2. Omitted and supplied H+1 floor cases do not distinguish the old behavior

Both cases allocate/publish H+1 before adding the `late`/`received` subscriber (`:91-98` and `:121-128`). Current `subscribeEvents` ignores `_since` but adds a listener only when `subscribeEvents` is called (`localAdapter.ts:168-173`). Current `emit` synchronously iterates the listeners present at event publication and schedules one callback per such listener (`:433-435`). Therefore the later subscriber never has an H+1 callback queued: it was not in the listener set when H+1 was emitted. The old implementation and the proposed future-only implementation both deliver no H+1 to that subscriber, then deliver a genuine H+2 after registration. Once the compile errors are repaired, the assertions in both floor tests are expected to pass against the old ignored-cursor implementation; they are not semantic REDs for `sinceCursor` floor handling.

This fails the original requirement that tests distinguish correct behavior from the current implementation and the repair assignment's demand for focused RED authorization. The tests need a source-supported scenario where the ignored boundary changes an observable result. For example, a post-subscription ordinary event whose current cursor is equal to or below the supplied/captured floor can expose the ignored filter, while the corrected allocator emits it above the floor; or another carefully bounded whitebox path must demonstrably enqueue work for the relevant subscriber before its callback drains. Preserve the separately required H+1/H+2 chronology as supplementary coverage, but do not claim it proves cursor-floor filtering unless an event is actually pending for that subscriber. Root/critic should agree the revised RED observes current production behavior before implementation is released.

## Expected semantic REDs versus unproved behavior

Source inspection predicts meaningful failures in other cases under frozen production source:

- The public execution test should fail its all-event uniqueness/order or distinct settlement-cursor assertions: `progress` currently copies the latest snapshot cursor (`localAdapter.ts:391-399`), and the settlement side event currently reuses `snap.cursor` (`:378-388`).
- The progress exact-lookup assertion should fail because `getSnapshotAt` resolves retained snapshots by exact cursor (`:146-151`), while current progress reuses the current snapshot cursor.
- Unsubscribe-before-drain should fail because `emit` has already captured the listener in a timer closure and the callback does not recheck subscription activity (`:433-435`).
- The two H+1 floor assertions are expected to pass old behavior for the reason above; the independent-case sequence assertions are expected to pass because `append` derives the next value from that case's own snapshot head (`:402-430`). The trajectory immutability comparisons around progress are expected to pass old behavior because current progress does not mutate revisions; the non-snapshot cursor lookup assertion is the relevant RED there.

No test has been executed, and the compile blockers prevent treating any predicted result as observed evidence. No compiler/test failure, passing test, or actual RED is claimed.

## Held scope and material deviations

Parser/invalid-input behavior, queue64/error handling, sequence ceiling/exhaustion, constructor seed validation, and `execution_observation` remain deliberately untested in this Phase 1 file, as both assignments require. The conformance change is unchanged and correct for the authorized branch-specific source adjustment; retained replay continues to receive the exact original `snapshot.cursor` with its exact accepted-cursor and strict-prefix checks.

No unauthorized source or conformance change was found. The repaired test file itself contains only focused test helpers/cases; its whitebox append/progress shapes match current private methods (`localAdapter.ts:391-399,402-431`), and its public dispatch intent matches the existing accepted intent pattern (`localAdapter.test.ts:262-271`). The material remaining deviations are fixture-level: the first test cannot compile as written; both floor tests lack a current-source semantic RED; and actual RED evidence is still absent. No implementation or production algorithm should be released based on this fixture.

## Verdict

**REJECT for focused RED authorization.** The repair correctly fixes the six cited fixture defects by inspection, but introduces/retains two compile blockers in the omitted-boundary test and does not make either H+1 floor case fail against the old source. A fresh different builder must repair only the test file; then a different critic should inspect the new hashes and source-grounded RED shape before root grants actual RED. Production remains frozen.

No tests, compiler, scanner, Graft build, Git write, network, source edit, status edit, or debt edit was performed. This review is not runtime proof.

🌱 Graft saved ~10,372 tokens (<$0.01) this turn.
