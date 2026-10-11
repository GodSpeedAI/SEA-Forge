# Private Next TDD fresh fixture repair

Date: 2026-10-09. Immutable source-preparation result. This record establishes
fixture source readiness only; it is not compiler, test, runtime, or behavior
evidence.

## Governing grant and review basis

This fresh repair was prepared under:

- `run-observation-private-next-assignment-oct09.md` and mandatory
  `run-observation-private-next-assignment-addendum-oct09.md` (assignment
  readiness independently approved as ccc150c1; source phase release is
  preserved in the original `private-next-tdd-builder-result-oct09.md`).
- `run-observation-private-next-notifier-proof-addendum-oct09.md` and
  `run-observation-private-next-projection-failure-addendum-oct09.md`, with
  document review `private-next-projection-failure-independent-review-oct09.md`
  (76ebcbf1).
- `private-next-initial-unavailable-fixture-correction-oct09.md`, with its
  independent review `private-next-initial-unavailable-correction-independent-review-oct09.md`
  (060a5349). This correction preserves the accepted initial-read cleanup
  contract and supersedes only the unreachable initial-read recovery setup.
- Rejected fixture review `private-next-tdd-independent-source-review-oct09.md`
  and its frozen input `private-next-tdd-builder-result-oct09.md`.
- Fresh source release from root after independent review 060a5349.

The original four-file boundary remains authoritative. Only
`run_observation_next_test.go` changed. No lifecycle implementation, notifier
implementation, production helper, manager field, delta helper, assertion
outside this fixture unit, or public contract was changed.

## Source identities after final edit

| File | SHA-256 | Scope |
|---|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | `b45dc39bc7466160cec9bc441d2635d577ceae69f64f165f19dbca7f7605737d` | unchanged |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go` | `2e5ef4ee58b1c821b6b5a9fdee83a2bce38f504dae3001f08e2b73306f350766` | unchanged |
| `apps/godspeed-casework-go/internal/server/run_observation_next.go` | `f362b461744b7b474f71a6d91c01f01eba76752c4c82761af5fa06ddb3db4015` | unchanged scaffold/stubs |
| `apps/godspeed-casework-go/internal/server/run_observation_next_test.go` | `72c379991b6256ac38c77602e33127c9ac7a8671657a89d97406eee7e14b9620` | fixture repair only |

## Rejection-finding disposition

| Finding | Repair and source evidence | Fixture oracle |
|---|---|---|
| 1. Static compile defects | Added the missing `nil` results to all four list-reader callbacks. Converted every `manager.cohorts[lease]` boolean use to comma-ok membership. | Callbacks in retention recovery, cancellation, Stop, and worker-join fixtures; membership assertions in detach, Stop, worker-join, and notifier tests. |
| 2. Hydration pruning not established | Replaced the undersized single-run setup with eight terminal runs and 1,024 long-ID frames per run, matching the proven aggregate cap shape in `run_observation_manager_test.go`. | `TestRunObservationNextAcceptsTerminalCaptureWithoutReplayingHydratedFrames` asserts aggregate DTO pruning, then verifies successful terminal Next returns all runs with nonnil empty delta frames. |
| 3. Vacuous all-or-nothing case and unspecified recovery | The two-run fixture starts each real worker at ordinal 1, waits for actual generation-2 reads with ordinal-2 frames, and only then injects failure on the second pure projector call. The first candidate must contain a newly advanced frame. Full watermark maps must remain unchanged; the approved disposition is terminal drain, not retry. | `TestRunObservationNextDiscardsWholeMultiRunCandidateAfterOneProjectionFails` asserts two projections, first-candidate advancement, full watermark equality, joined actual workers, and removed cohort/poller capacity. |
| 4. Initial no-current setup | Applied correction 060a5349: no same-entry recovery claim. A real accepted-list initial read error is held, then Prepare must return its unavailable wrapper with no watermark, no retained lease/poller ref, and actual worker JOIN. The existing shared-initial-failure regression remains untouched. | `TestRunObservationNextInitialReadUnavailableHasNoWatermarkOrPollerRef`. |
| 5. Authorization sequencing gaps | Added post-wake authorization rejection before projection, final cursor mutation while the projector is paused, and a blocked authorization callback while detach claims the registered operation. Authorization wrappers are installed before Prepare and discriminate Next through a context marker. The post-wake case also confirms the seeded wake was consumed before the second authorization check failed. | `TestRunObservationNextRechecksAuthorizationAfterWake`, `TestRunObservationNextRechecksCursorBeforeFinalDisclosure`, and `TestRunObservationNextBlockedAuthCallbackIsOwnedByDetach`. |
| 6. Barrier/notifier cleanup could strand drain | Added once-backed `releaseRunObservationNextBarrier`, used for each projector/read/auth gate. The notifier target test registers an actual target and schedules once-backed `sendPollerNotifierTargets` cleanup immediately after registration. | Existing capture, detach, cancellation, Stop, worker-join and notifier-ownership tests now release gates/targets on every exit path. |
| 7. No nonempty gap-order oracle | Added two actual workers with baseline, middle, and latest reads. It waits for generation 2 (`middle` retained), then generation 3 (`middle` omitted but still ledgered), before Next. The runs are supplied in reverse order. | `TestRunObservationNextSortsNonemptyWindowGapsByRunIDBytes` asserts two nonempty gaps ordered `a-run-gaps`, `z-run-gaps`, their exact evicted IDs/ordinals, and the correspondingly sorted runs. |

## Limits and deviations

- This is source-only fixture preparation. No Go compiler, tests, race run,
  formatter, gate, Git, status, or debt command was run.
- The private Next implementation and production notifier path remain inert
  scaffolding. Behavioral RED is owned by root after independent source review.
- Initial list-unavailable remains separate from accepted-list initial trace
  failure. The latter proves cleanup only; no detached-entry recovery is
  claimed. Actual recovery remains limited to later read/retention markers on
  retained attachments.
- The injected projector error follows the independently reviewed terminal
  disposition. It proves no partial watermark commit and actual teardown; it
  does not claim recovery or worker-publication evidence.
- No material deviation from the authorized four-file scope or assignment
  remains. The only test-claim change is the narrow initial-unavailable
  correction approved by 060a5349.
