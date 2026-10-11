# Present-context guard prerequisite — independent Phase 1 review

Date: 2026-10-06  
Verdict: **REJECT fixture for bounded repairs; source stub scope is correct.** This is a source/fixture review only. No test, compiler, scanner, Graft build, Git, or network command was run.

## Frozen identities and scope

- Original assignment: `run-observation-present-context-original-assignment-oct06.md`, SHA-256 `f15838d11468f6bade76a7a759ebc9676c7d1c9e5d260c64440d0de3df9e262d`.
- Source stub: `apps/godspeed-casework-go/internal/server/run_observation_present_context.go`, SHA-256 `565ad7a604bf68ab7667c901eaa23928d28476150fdfa82da9e170ba9e61ee86`.
- Fixture: `apps/godspeed-casework-go/internal/server/run_observation_present_context_test.go`, SHA-256 `825ab0dcdb80b27338401aab8227474c4422e1a4aea0b3d22073b0ae05255060`.

This review is limited to the assigned two-file Phase 1 stub and tests. It does not review or release the eventual guard algorithm, manager, caller integration, authorization, or public readiness. Existing `Store.Trajectory` returns retained case revisions as deep copies and documents empty history as unknown/cold/evicted (`internal/projection/store.go:41-53,122-139`). `CaseFacts` has optional pointer ownership with `Record`, `Overview`, `Horizon`, and cursor (`internal/projection/builder.go:34-50`); `CaseHorizon.Items` is a slice and may encode complete empty with a nonnil zero-length value (`internal/ports/ports.go:240-274`). These exact production shapes support the assigned narrow guard seam.

## Source stub scope

The source defines only private unit-prefixed interfaces for `Trajectory(caseID)` and `CursorForCase(caseID)`, the private `{caseID,cursor,parentIDs}` result, and the exact requested function. It ignores its inputs and always returns the zero result with `apperr.KindUnavailable` via `apperr.New` (`run_observation_present_context.go:8-32`). There is no validation algorithm, caller, constructor integration, authority/kernel operation, exported server interface, or fabricated ready state. This matches the assignment's deliberately unavailable Phase 1 stub. No source-scope deviation found.

## Fixture coverage and expected stub behavior

The helper builds actual `projection.Revision`, `contract.CognitiveWorldSnapshot`, `projection.CaseFacts`, and `ports.CaseHorizon` values; cursors and case references are nonempty; horizon items are constructed nonnil even at length zero; populated parents retain full opaque IDs (`run_observation_present_context_test.go:28-54,193-207`). The table independently mutates revision case, snapshot case/cursor, facts cursor, record/overview/horizon refs, blank revision cursor, missing facts, nil horizon, blank parent, and duplicate parent (`:123-154`). Separate tests cover blank case, nil dependencies, empty history, newest-row invalidity, absent/blank/mismatched/advanced relay cursors, populated/empty success, advancing history, parent removal, and wrong-case history (`:102-121,156-191,193-274,276-284`). Error assertions check both typed unavailable and an exactly zero result (`:92-100`).

Because the source is still the specified unconditional stub, static control-flow analysis predicts:

- Blank/nil/empty, identity-defect, latest-invalid, and relay-gap tests that assert only typed unavailable and zero result will pass for the stub regardless of input. That is not evidence that the future algorithm validates those defects.
- Both success subtests in `TestCheckRunObservationPresentContextAcceptsAndCopiesEmptyAndPopulatedHorizons` fail immediately at the `err != nil` assertion (`:211-214`); output equality, input immutability, returned-map copying, source-item mutation, and repeat independence below it are not reached.
- `TestCheckRunObservationPresentContextRechecksAdvancingHistoryAndDoesNotReuseOldParents` stops at its first expected success (`:248-255`), so the simulated relay advance and latest-parent replacement assertions are not reached.
- Relay-gap tests fail their exact-call assertion because the stub makes no `CursorForCase` calls (`:181-188`). The wrong-case test also expects dependency calls, so its later call assertion fails against the stub (`:276-283`). These are expected missing-algorithm red boundaries by inspection; no actual test run or RED capture is claimed.
- The newest-invalid test's typed-error/zero-result check passes the stub, but it does not yet establish no-fallback behavior. The test's older row has `cursor/old`, while the relay advertises `cursor/new` (`:156-165`). An incorrect implementation that falls back to the older row and then correctly compares that old cursor against relay would still return unavailable and pass. Make the fallback distinguishable: arrange the fallback row to match the relay cursor while the newest row is invalid, then require rejection (or assert a separately exposed selection outcome without returning partial state).

The wrong-history-case test also requires `CursorForCase("case_1")` even when the final trajectory row is already rejected for belonging to `case_other` (`:276-283`). The assignment requires an exact requested-case relay check for a valid candidate, but does not require querying relay after an earlier history-identity failure. The fixture should assert that `Trajectory` receives the requested case, and leave relay-call behavior for valid latest history/gap cases; otherwise it prescribes unnecessary internal call order. The relay-gap table already checks the exact relay request on that path (`:181-188`).

## Blocking fixture defects

### 1. No independent nonblank revision-cursor mismatch case

The identity table tests only `Revision.Cursor == ""` (`:135`). The snapshot and facts cursor cases alter those fields while leaving `Revision.Cursor` at its original nonempty value (`:130-131`). No case changes only `Revision.Cursor` to a different nonempty opaque value while all other identity/cursor fields and relay still match. An algorithm that rejects blank revision cursors but accidentally ignores equality of a nonblank `Revision.Cursor` with snapshot/facts/relay can pass the current invalid and positive fixtures. Add that one-field mutation to prove each cursor identity is independently bound to the selected revision.

### 2. Full-input immutability checks use aliased or incomplete baselines

The assignment requires complete source history to remain `DeepEqual` unchanged. In the per-identity test, `before` is passed directly into the fake history, so `history.revisions` and `before` are the same backing slice when compared at `:145-150`; an in-place mutation changes both sides and is invisible. The newest-row test compares `history.revisions` to `inputs` after the same `inputs` slice was passed into the fake (`:160-165`), so that comparison is also aliased.

The clone helper intended to create independent baselines is incomplete: the fixture populates `Facts.Overview.Stages` at `:41`, but the helper copies the `CaseFacts` struct shallowly and does not copy `Overview.Stages` (`:70-89`). Thus the positive test's `inputs` versus `before` comparison (`:208-219`) also shares that slice; mutating it through the function input changes both values and remains undetected. Use a deep independent baseline that clones every populated mutable field, pass a separate source slice to the fake, and compare the post-call source against the untouched baseline on success and rejection. Populate nested slice fields in a realistic fixture only where needed to exercise those copies.

## Required bounded repair and limits

Before fixture approval, independently add (a) a nonblank `Revision.Cursor` mismatch; (b) genuinely independent complete pre-call snapshots for success and failure mutation checks, including `Overview.Stages`; and (c) a no-fallback fixture whose old row would pass relay equality if selected, so only newest-row enforcement causes rejection. Remove or narrow the wrong-case relay-call count assertion unless the exact order is separately made a requirement. Keep all source files and Phase 1 stub unchanged.

The record does not claim actual test execution, successful assertions after early stub failures, algorithm correctness, complete input immutability evidence from the current fixture, or authority to implement the guard. The source stub itself remains correctly scoped and always unavailable. A repaired fixture and independent source review are required before root may grant the focused actual RED gate. Even eventual guard success is only as-of-check knowledge, not authorization, continuously current kernel truth, atomic history/relay state, or public V4 readiness.
