# Phase B four-binding repair result — 2026-10-07

## Provenance and preserved preimages

The full repair instruction was archived before edits in
`run-observation-manager-phase-b-four-binding-repair-assignment-oct07.md`.
The original source-preparation release is
`run-observation-manager-phase-b-tdd-source-preparation-assignment-oct07.md`
(SHA-256 `34224d769f5b802cb7cc31b005ae330f671598590e809aede649cd98dc0a89a7`);
its result is `run-observation-manager-phase-b-tdd-source-preparation-result-oct07.md`
(SHA-256 `a2d1c9e7061b48ba0919373421391ef91d830067703d9d1b30bb9fa0a31311b4`),
with citation erratum `run-observation-manager-phase-b-tdd-source-preparation-review-citation-erratum-oct07.md`
(SHA-256 `fbc3c58b888644cd80ef106e38fb4a58ec0f7e226b3d53b89b7cef30d50cc327`).
The actual independent rejection is
`run-observation-manager-phase-b-tdd-source-independent-review-oct07.md`
(SHA-256 `38e4130200659ef4f5a38f3a5a9d68b824ed5aed33afc1d7d854d3f9dccdd913`).

Before source edits, exact current-file JSON preimages were archived:

| Source | Archived JSON | Decoded bytes | Decoded SHA-256 |
| --- | --- | ---: | --- |
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | `run-observation-manager-phase-b-manager-current-preimage-oct07.json` | 7,041 | `9e8cb97a0197b1382162c0ce91501538e7d308b380765d845edda328f0a2441d` |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go` | `run-observation-manager-phase-b-failure-current-preimage-oct07.json` | 46,418 | `06c239e865aeca39fdbbe6f63b92cd23e3c181f1e0af071a6fb9b4b4c0724a1f` |

For each JSON record, decoding its `source` as UTF-8 reproduced the byte count
and SHA-256 above exactly. These are the exact two current source preimages,
not reconstructions of the older Phase A files.

## Exact four repairs

The preimage-to-current unified diff contains only these four bindings:

1. In `run_observation_manager.go`, the first two `reservePollerBatch`
   parameters are now named `lease *runObservationLease` and
   `rows []ports.RunSummary`. Required `prepareDone <-chan struct{}` and the
   deliberate unavailable stub body are unchanged.
2. In `TestRunObservationManagerFailureStopJoinsHeldListCreator`, the cohort
   lookup now uses `_, registeredLease := manager.cohorts[lease]`.
3. In `TestRunObservationManagerFailureCallerCancelHeldReadableListDoesNotReserve`,
   the held-state lookup now uses `_, stillRegistered := manager.cohorts[lease]`.
4. In that same caller-cancellation test, the post-return lookup is split into
   `_, remainingCohort := manager.cohorts[lease]` and
   `remainingPollers := len(manager.pollers)`.

No assertions, function bodies beyond these bindings, test order, signatures,
or behavior were otherwise changed. No other files were edited.

## Fixture inventory and frozen sources

The fixture contains 12 tests: the original nine
`StopWinsBetweenReserveAndLaunch`,
`DetachWinsBeforeFirstReadUsesDrainingCode4`,
`ClaimThenStopUsesWorkerContinuation`, `StopJoinsHeldActualRead`,
`GlobalStopReleasesDistinctPendingLeases`,
`CanceledPrepareLeavesAuthorizedLease`,
`DetachAndSelectedAReleaseHaveOneOwner`,
`SharedFirstReadErrorIsSuccessfulA`, and
`OversizedNilKeyIsAWithoutAdmission`; plus the three held-list cases
`StopJoinsHeldListCreator`, `DetachJoinsOnlyHeldListCreator`, and
`CallerCancelHeldReadableListDoesNotReserve`. The nine original cases remain
present. This static repair does not claim they compiled or ran.

All eight protected files retain their authorized hashes:

| File | SHA-256 |
| --- | --- |
| `run_observation_poller_worker.go` | `f90cd397cd983e42e53498f13baa89fbfeac54e07eb3847b3228592ddd4d929f` |
| `run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |

## Limits and next review

No compiler, test, formatter, scanner, Graft build, or Git action was run.
The manager methods remain deliberate unwired stubs; this is not lifecycle
implementation, source/runtime approval, or expected-RED evidence. A different
critic must review the complete original release, this repair assignment, the
exact source diff, and this result before any 12-case focused RED execution.
