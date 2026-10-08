# Watcher terminal TDD source preparation result — 2026-10-08

## Authority and scope

This is a source-preparation receipt for the full assignment [run-observation-watcher-terminal-tdd-source-assignment-oct08.md](run-observation-watcher-terminal-tdd-source-assignment-oct08.md), SHA-256 `bfcfa5cd59f53bf5d81de7a349eac669e8899a8b842c863cb4cdb0f580a5ec82`. The assignment released only the manager lease metadata/signature update, four manual fixture callsite adaptations, and one new authority/terminal TDD fixture. It expressly prohibited implementing the worker/manager algorithm and running compiler, tests, formatter, scanner, build, or Git operations. This result records source presence only; it does not claim compilation, RED, runtime behavior, or approval.

The assignment’s full input chain remains preserved at its original immutable paths: Unit 1 assignment and lifecycle preregistration; revision 6 and its clarifications; combined-image and lifecycle architecture corrections; the Phase B algorithm grant/result/rejection; the watcher-terminal proposal, supplement, root clarifications, and independent approval. The exact immediate assignment and current source are authoritative for this bounded step. No earlier record was overwritten.

## Exact source preimages

Before edits, native source strings were captured in the adjacent immutable JSON files and independently checked for declared digest and byte count:

| Source | Preimage SHA-256 | Bytes | Current SHA-256 | Bytes |
|---|---|---:|---|---:|
| `run_observation_manager.go` | `d7be1d7a724ed65575ccaaf35cdf1aa5af133dbdd4952d592699b7eb443b6350` | 26,384 | `47c95f3ba90abb4355f02d626e664a49027f602e9ac47771c40eda0ba8f22780` | 26,548 |
| `run_observation_manager_failure_test.go` | `ef587b0bd64fe42d5fb56369f620f969610a5bac3d04cd7578911472b1a4893c` | 46,429 | `ccbbe234785365a266cd7ea10737158ca3eea2d4a8a88a9f28c24b7260b190f4` | 47,657 |
| New `run_observation_manager_authority_terminal_test.go` | — | — | `4934e9c5e8a5506f219f2c873856452e97f4c971f4c7a233dbc480fa79df7367` | 29,548 |

The preimage records are `run-observation-watcher-terminal-tdd-manager-preimage-oct08.json` and `run-observation-watcher-terminal-tdd-failure-preimage-oct08.json`. Programmatic read-only verification recomputed their source SHA-256 and byte lengths; both matched their declarations. The manager and failure-fixture diffs were reviewed against those exact captured strings. Changes are scoped to the released declarations/callsites; no unrelated source path was edited.

## Changes made

- In `run_observation_manager.go`, the lease now carries the approved immutable `caller runObservationCaller` and `asOfCursor string`; `beginPrepareOperation` requires both arguments and stores them; production `prepare` passes the already resolved caller and guarded cursor. There is no optional/default mode and no algorithm behavior change.
- In `run_observation_manager_failure_test.go`, the four manual begin sites in `TestRunObservationManagerFailureStopWinsBetweenReserveAndLaunch`, `TestRunObservationManagerFailureDetachWinsBeforeFirstReadUsesDrainingCode4`, `TestRunObservationManagerFailureClaimThenStopUsesWorkerContinuation`, and `TestRunObservationManagerFailureDetachAndSelectedAReleaseHaveOneOwner` now derive caller from each test’s real request identity, obtain the cursor through the real manager guard, fail setup on errors, and pass those required values. Existing test assertions and identities remain in place.
- The new test file adds seven top-level groups under the required `TestRunObservationManagerAuthorityTerminal` prefix. They cover fitting terminal final value for owner and shared waiter; recurring terminal candidate whose inner value fits but full outer retained image exceeds the cap; auth invalidation before and during an actual read; cursor change during an actual read; invalid watcher removal while an authorized shared survivor remains; and final Prepare handoff rechecks across auth/cursor changes and empty/error list results. The fitting-terminal test checks both DTO keys and completed/accepted values, retained-current key/availability/final values, reads 1/0, both eligible leases, no later read, and worker completion. The recurring cap case checks safe prior-marker retention, no candidate disclosure, no new claim, and cleanup only after worker completion.

The new fixture uses actual public `prepare` and actual list/trace port callbacks, controlled by channels at their real call boundaries. It does not add test hooks, write manager state to manufacture transitions, use sleeps, or change production worker code.

## Frozen source verification and limits

The following eight pre-existing files were read and remained byte-identical by SHA-256: worker `84382ffd0cd77e1c85bd11b607c1427075c4899e1c534fbe7368b0af5ad7c9cc`; original manager fixture `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`; key `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b`; retained version `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`; retained-version tests `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`; policy tests `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28`; poller image `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9`; retained-image tests `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256`.

No compiler, test, formatter, scanner, Graft build, or Git command was run for this step. The source is ready for the separately assigned independent source critic; any compile/RED/runtime statement awaits that review and a later explicit execution grant. This receipt makes no claim that tests pass or that the manager algorithm is implemented.
