# Phase B test-first source preparation — result — 2026-10-07

## Assignment, preimages, and source identity

The complete release is archived at
`run-observation-manager-phase-b-tdd-source-preparation-assignment-oct07.md`,
SHA-256 `34224d769f5b802cb7cc31b005ae330f671598590e809aede649cd98dc0a89a7`.
The two exact UTF-8 source preimages are JSON records with path, SHA-256,
byte count, and lossless full source:

| Path | Preimage JSON | Original SHA-256 | Bytes | Post-edit SHA-256 |
|---|---|---|---:|---|
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | `run-observation-manager-phase-b-tdd-manager-preimage-oct07.json` | `f9321a620ad64546e735b936d0b58b9f514eab0d7c6d0325f1e7a82f37c5e314` | 6,924 | `9e8cb97a0197b1382162c0ce91501538e7d308b380765d845edda328f0a2441d` |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go` | `run-observation-manager-phase-b-tdd-failure-fixture-preimage-oct07.json` | `e63e050a7999b5c239257e57c6642baa346c59a40dfa1871e58d7b64ef80a5cf` | 27,703 | `06c239e865aeca39fdbbe6f63b92cd23e3c181f1e0af071a6fb9b4b4c0724a1f` |

Before source edits, both decoded archived source strings matched the original
source bytes exactly (cmp 0); decoded lengths and SHA-256 values match the
table. The independent document review approving this preparation-only release
is `run-observation-creator-drain-final-wording-independent-review-oct07.md`,
SHA-256 `03243da7879ce821eed31fe23e7640138e78f57613b608cc687322811f06fe99`.
It approves TDD source preparation only.

## Exact source changes

In `run_observation_manager.go`, the private lease gains only
`creatorDone chan struct{}` and `leaseDone chan struct{}`.
`finishPrepareOperation` now accepts its exact lease, and
`reservePollerBatch` now requires `prepareDone <-chan struct{}`. Both
method bodies remain the original deliberate unwired stubs; no lifecycle,
context-bridge, reservation, drain, or worker algorithm was added.

In `run_observation_manager_failure_test.go`, only the four named fixture
functions changed their local `finishOnce.Do` closure to pass that function's
captured exact lease and their reserve call to pass `lease.leaseDone`:

- `TestRunObservationManagerFailureStopWinsBetweenReserveAndLaunch`
- `TestRunObservationManagerFailureDetachWinsBeforeFirstReadUsesDrainingCode4`
- `TestRunObservationManagerFailureClaimThenStopUsesWorkerContinuation`
- `TestRunObservationManagerFailureDetachAndSelectedAReleaseHaveOneOwner`

All nine existing cases and their assertions remain. Three tests were added;
each waits for actual list-callback entry and fails immediately if Prepare
returns before that entry:

1. `TestRunObservationManagerFailureStopJoinsHeldListCreator`: Stop cancels
   the held list, keeps the exact cohort and creator open until release, rejects
   another Prepare before another list/read, then completes after typed
   zero-wrapper/nil-lease failure and creator completion.
2. `TestRunObservationManagerFailureDetachJoinsOnlyHeldListCreator`: A and B
   use separate held list calls. Detach A cancels/joins A while B's context and
   creator remain live; A returns typed zero-wrapper/nil-lease failure and
   completes its drain while B is held; B then returns a successful empty-list
   lease with closed creator completion and detaches normally.
3. `TestRunObservationManagerFailureCallerCancelHeldReadableListDoesNotReserve`:
   caller cancellation is observed while the real callback remains held, then
   the callback returns a readable candidate with nil error. Prepare must return
   typed cancellation with zero wrapper/nil lease, no trace read or candidate
   reservation, and closed creator completion before its cohort is removed.
   A later empty-list Prepare proves the manager can be reused afterward.

The shared helper `waitForRunObservationFailureListEntry` uses the existing
two-second bounded channel pattern; it is not a lifecycle hook. Test callbacks
use explicit release channels and cleanup. There are no sleeps, fake port
barriers, cancel-function replacement, or private state writes.

## Frozen source identities

All eight protected files were rehashed after the edits:

| File | SHA-256 |
|---|---|
| `run_observation_poller_worker.go` | `f90cd397cd983e42e53498f13baa89fbfeac54e07eb3847b3228592ddd4d929f` |
| `run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |

## Limits and next review

This is source-preparation evidence, not lifecycle implementation. No
formatter, compiler, test, scanner, Git operation, or Graft build was run.
The read-only Graft freshness check reports no committed graph and stale
structure for the edited symbols; rebuilding is outside this release. No
compile, expected RED, lifecycle behavior, or runtime approval is claimed.
The next gate is a different independent source critic reviewing the complete
original release and exact change; afterward Root must separately authorize
a resource-checked 12-case focused RED.

