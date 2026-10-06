# Run observation hydration cap — independent Phase 1 fixture rereview

Date: 2026-10-06  
Verdict: **REJECT fixture release pending one focused assertion repair.** The production file remains the authorized typed-unavailable stub. No compile/test/runtime gate was run; actual assertion RED is unproven.

## Frozen identities

- Stub `apps/godspeed-casework-go/internal/server/run_observation_hydration_cap.go`: SHA-256 `1ebfa2e2a002b3c86ca4fd8b599e8b572929feb8ebaa192326658b3c3b8227d7` (unchanged from Phase 1).
- Fixture `apps/godspeed-casework-go/internal/server/run_observation_hydration_cap_test.go`: SHA-256 `bf21b94192d510364c60720d174c3ec9f91b71c5e8a58e4c43a0d4947877aa9c`.
- Compared the full fixture against both `run-observation-hydration-cap-original-assignment-oct06.md` and `run-observation-hydration-cap-fixture-repair-original-assignment-oct06.md`, the Phase 1 rejection, and the RunTrace DTO/golden contract.

## Prior Phase 1 findings now corrected

1. `hydrationRun` now uses valid execution `active`, settlement `unsettled`, and nested observation state `validated` (`run_observation_hydration_cap_test.go:26-34`; vocabularies in `internal/contract/contract.go:434-442`).
2. Empty variadic constructors now allocate nonnil zero-length `Frames` and `Runs`; the clone helper conditionally allocates when the input slice is nonnil, preserving nil versus nonnil-empty shape while separately copying nested frames and optional pointers (`:26-53,56-91`). Empty runs/frames therefore serialize as arrays, consistent with canonical checks (`internal/contract/golden_test.go:214-219,250-257`).
3. The cap-plus-one success snapshots the input, verifies it after trimming, verifies independent run/frame backing slices, mutates output identity, optional metadata and all six cohort count pointers, and rechecks the original (`:168-204`). This now exercises non-aliasing on an actual trimming branch with a surviving frame.
4. The nine-run case has `Limit=8`, `ReadsAttempted=8`, selected/validated counts 8 and omitted count 1 (`:376-386`), so it isolates the over-eight-runs condition from the independent read-budget range (`golden_test.go:211-218`).

## Remaining mandatory finding

### R1 · No assertion checks exact metadata when trimming empties a run

The assignment requires keeping required run metadata and updating `TotalFrameCount`, `RetainedFrameCount`, `OmittedFrameCount`, and `Truncated` exactly for every run affected by frame removal (original assignment, “Eventual behavior pinned”). Multiple deterministic cases remove the sole/oldest frame of a run and expect that run to remain with an empty frame list: equal-instant run-ID tie at `:222-228`, event-ID tie at `:229-235`, and the natural cross-run oversized case at `:264-300`. These cases assert run identities and surviving event IDs, but never assert the emptied run’s `TotalFrameCount` / `RetainedFrameCount` / `OmittedFrameCount` / `Truncated`. The cap-plus-one and ring-omission cases do assert counts, but each leaves one frame in the affected run (`:168-204,303-342`).

Thus a production implementation that mishandles metadata when a run’s final retained frame is removed could satisfy every current expected assertion. This is distinct from the already covered partial-run count update. Retaining the run ID in an expected map does not prove its required frame metadata is accurate.

**Required repair:** add explicit expected metadata for at least one trim case that removes all frames from a run (preferably the multi-run case): preserve its original total, expect zero retained, increment omitted exactly once, set truncated true, retain run identity/standing/observation fields, and assert the full relevant values. Keep the current tie and survivor expectations unchanged.

## Remaining coverage and scope

The fixture now contains well-formed empty/short inputs, exact cap and cap-plus-one sizing from independent `encoding/json`, globally oldest and equal-instant offset ordering, complete RunID and EventID byte-order examples, unsorted per-run survivor order, naturally oversized cross-run frames, pre-existing ring omission accounting, all six count pointers, metadata-only overflow, >8 runs, >1024 frames, inconsistent counts and invalid timestamps. The nine-run budget fields remain within their independent range. The frozen source is still only the required `KindUnavailable` stub and signature; no algorithm or wiring has been introduced.

The expected run-ID tie result is correct under bytewise full-ID ordering: `run-1` sorts before `run-10` because it is the shorter prefix, and is the removed frame at the equal instant. The event-ID result correctly removes `evt-a` before `evt-z`. These are source-level oracle checks; no implementation was executed. Other than R1, no material deviation from either original assignment was identified.

**No runtime verdict:** this is fixture source review only. Root may separately decide whether to authorize actual RED after the fixture repair is independently accepted. No source edits, tests, compiler, scanner, Git/status/debt operation, or Graft build was performed. Root retains release authority.
