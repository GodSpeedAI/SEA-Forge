# Watcher terminal fixture repair result — 2026-10-08

This records a fixture-only repair. It does not claim test, compile, or runtime
verification, and it does not authorize lifecycle implementation.

## Frozen identity

The only edited source path is
`apps/godspeed-casework-go/internal/server/run_observation_manager_authority_terminal_test.go`.
Its current SHA-256 is
`51f73f94fed07eb97fd7b9b59d76cbf1ce65278d3acaca5b778c3a91782ad5c8` (41,008
bytes). The exact pre-edit source is preserved in
`run-observation-watcher-terminal-fixture-repair-preimage-oct08.json`; the
decoded source is 29,548 bytes with SHA-256
`4934e9c5e8a5506f219f2c873856452e97f4c971f4c7a233dbc480fa79df7367`.

Ten other files were read and verified unchanged:

| File | SHA-256 |
|---|---|
| `run_observation_manager.go` | `47c95f3ba90abb4355f02d626e664a49027f602e9ac47771c40eda0ba8f22780` |
| `run_observation_poller_worker.go` | `84382ffd0cd77e1c85bd11b607c1427075c4899e1c534fbe7368b0af5ad7c9cc` |
| `run_observation_manager_failure_test.go` | `ccbbe234785365a266cd7ea10737158ca3eea2d4a8a88a9f28c24b7260b190f4` |
| `run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

## Repairs in the fixture

* The outer-cap oracle independently marshals the complete ordered wrapper with
  the helper’s raw JSON as `current`, proves that full wrapper exceeds the cap,
  and checks the bounded encoder returns nil bytes and the fixed unavailable
  error. It no longer uses the production encoder to establish the candidate
  outer-size oracle.
* Failure cases for revocation before read, during read, and cursor change
  during read inspect cohort and poller ownership immediately after failed
  Prepare; cases with a started worker also require `workerDone` already closed.
* The existing held-read shared-survivor case retains its assertions and now
  checks both directions of invalid lease membership and the surviving
  cohort/ref.
* A second group-6 case checks an invalid initiator before the port while a
  valid shared survivor remains. Its barrier is in the existing authorization
  dependency and is entered only when the callback context is pointer-identical
  to a registered poller context under the manager mutex. The mutex is released
  before waiting; no context methods run under it. This avoids relying on a
  specific number of Prepare authorization checks. The existing Current and
  present-context dependencies observe revoked/current values after release.

The resulting test inventory has seven top-level test functions, including
both group-6 subtests and four final-handoff table cases. No assertion result is
claimed: tests, compilation, formatting, and runtime checks were not run.
Expected review focus is that the new pre-port case gates actual worker
authorization through an existing dependency, detects a trace call that wins
before that dependency, and preserves the shared survivor path. The independent
critic should review the complete original repair assignment, exact preimage,
and this frozen source before any RED run.
