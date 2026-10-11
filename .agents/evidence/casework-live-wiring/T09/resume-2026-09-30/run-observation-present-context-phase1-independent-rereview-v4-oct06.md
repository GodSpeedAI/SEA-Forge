# Present-context guard Phase 1 fixture — independent rereview

Date: 2026-10-06  
Verdict: **APPROVE fixture for root consideration of the bounded focused RED gate.** This is source and test-fixture review only. No test, compiler, scanner, Graft build, Git, or network command was run. This verdict does not claim actual RED, algorithm correctness, or production approval.

## Frozen inputs

- Original assignment `run-observation-present-context-original-assignment-oct06.md`: SHA-256 `f15838d11468f6bade76a7a759ebc9676c7d1c9e5d260c64440d0de3df9e262d`.
- Fixture repair assignment `run-observation-present-context-fixture-repair-original-assignment-oct06.md`: SHA-256 `ab4e72b1f3fe8305c56f59099ee36b0ae65ccbd1426b76920b4425365d227d3e`.
- Prior review `run-observation-present-context-phase1-independent-review-oct06.md`: SHA-256 `d715a0b5ff36171e0403a2692fff62e0c5bdc42781e28ca1eb4bb7f377f91c16`; addendum: SHA-256 `dbdb11ab9d46fa6d18eccd6b23ce8aa3d5d55c99145c54cfba64de3972889734`.
- Stub `apps/godspeed-casework-go/internal/server/run_observation_present_context.go`: SHA-256 `565ad7a604bf68ab7667c901eaa23928d28476150fdfa82da9e170ba9e61ee86` (unchanged).
- Repaired fixture `apps/godspeed-casework-go/internal/server/run_observation_present_context_test.go`: SHA-256 `fffa033505fa19b937da12963df91a0cdc2af30f39ecb50031fc2a17ec64a67b`.

## Source scope and fixture shape

The stub still defines only the private history and relay seams, the private result, and `checkRunObservationPresentContext`; all calls return a zero result with typed unavailable (`run_observation_present_context.go:8-35`). This matches the deliberately unavailable Phase 1 assignment. It contains no validation algorithm, caller, constructor integration, authority/kernel read, or fabricated ready result.

The fixture constructs actual `projection.Revision`, snapshot, facts, and horizon values with nonempty exact cursor/case fields and a nonnil horizon (`run_observation_present_context_test.go:34-54`). Its history fake returns the supplied revision slice, making separate input and baseline ownership meaningful (`:13-21`). The repair remains within the assigned test file; no source or other test changed.

## Prior findings resolved

1. **Independent, shape-preserving baseline.** The clone helper now retains nil versus nonnil empty for `VisibleObjects`, strings, actions, overview lists, horizon items, and other cloned slices by allocating with `make` when the source is nonnil (`:70-181`). It deep-copies the fixture's populated `Overview.Stages` (`:115-121`) and nested mutable values represented by the clone helper. Rejection tests pass a separately cloned input and compare it to a distinct clone (`:273-285`); success cases do the same (`:369-384`). Invalid-input cases also clone history and relay maps before calling (`:222-242`). The previous empty-versus-nil false failure and self-alias comparisons are removed.

2. **Independent cursor defects.** The nonblank revision-cursor case changes only `Revision.Cursor` while snapshot, facts, and relay retain `cursor/exact` (`:253-265,273-285`). The blank case sets revision, snapshot, facts, and relay cursor views blank together (`:261-265,277-285`), isolating rejection of a blank compared cursor from a cursor mismatch. Other snapshot and facts cursor cases remain one-field mutations (`:255-256`).

3. **Distinguishable newest-only rule.** The older valid row has `cursor/old`, the newest row has invalid nil horizon items, and Relay advertises `cursor/old` (`:298-309`). Falling back to the older row would satisfy the relay equality, so the expected unavailable result now specifically exercises rejection of the invalid newest row rather than being masked by a later mismatch. The input baseline is independently cloned.

4. **Optional Relay lookup after invalid history.** Wrong-case history now checks the exact requested history case and only constrains Relay arguments if Relay is called (`:445-461`). It no longer requires an unnecessary Relay read after an already-invalid latest row. The valid-history relay-gap table still requires an exact requested-case Relay call (`:320-352`).

5. **Parent ownership and history progression.** Populated and nonnil-empty horizons assert exact opaque IDs/cursor, input immutability, result-map independence from source mutation and caller mutation, and independence of a repeated result (`:354-406`). Advancing Relay without a captured row must fail; after a matching newer row is retained, the newer parent set excludes the removed parent while the earlier result remains unchanged (`:408-443`).

6. **Unavailable/error contract.** Blank case, nil dependencies, empty history, independent identity/reference/cursor defects, nil horizon, blank/duplicate IDs, missing or blank Relay cursor, advancement, and opaque cursor mismatch remain covered (`:209-352`). Failure assertions require typed unavailable and an exact zero result (`:199-207`).

## Remaining limits and disposition

I found no remaining material fixture omission against the two assignments. The `Overview.Stages` slice is populated and cloned; other mutable nested values used by the fixture are either cloned by type or are absent/zero. Relay-call behavior is constrained only where the original contract requires an exact-case query, and each history lookup is checked against the requested case.

The source is still an unconditional stub. By inspection, positive populated/empty success tests fail at the first `err != nil` assertion (`:373-376`), so later result, immutability, copied-set, repeat, and history-advance assertions are not reached in this Phase 1 run. Negative typed-error/zero-result checks can pass without input-specific validation and do not prove the eventual algorithm. No runtime outcome is claimed here. The original limitations remain: this is only an as-of-check guard prerequisite, does not guarantee continuous kernel freshness or prevent races after the check, and releases no watcher, manager, caller, or public readiness.

There is no implementation or runtime release in this review. Root retains the decision whether to grant the actual focused RED gate with its own resource and capture requirements.
