# Phase A one-binding fixture compile repair result — 2026-10-07

## Exact preimage and source result

Only `apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go`
was edited. Before the edit, the exact current source was archived as native
JSON in
`run-observation-manager-failure-test-preimage-compile-repair-oct07.json`.
Decoded preimage verification returned 27,708 bytes and SHA-256
`4ff2fe1c922d6ae7e232d5a58f201298e232807437967fe22a9316dcb45aa0f3`, matching
the JSON metadata. The post-edit source is 27,703 bytes with SHA-256
`7d1470f4b930036561e0c0500d58fb41240994e764754977ce406367b38ba6b0`.

The complete repair instruction was archived before source editing at
`run-observation-manager-phase-a-compile-repair-assignment-oct07.md`, SHA-256
`b5689c684013ab24a76289383f9a008a6efc364c08ef1f2db4cce12cf03da0e0`.

## Exact source delta

At `TestRunObservationManagerFailureGlobalStopReleasesDistinctPendingLeases`,
the single binding changed from:

```go
manager, caller, _, _ := newRunObservationFailureFixture(...)
```

to:

```go
manager, _, _, _ := newRunObservationFailureFixture(...)
```

No session identities or `prepare` closure calls were changed. The unified
preimage comparison showed only this one-line difference. The unused local
`caller` was not consumed elsewhere in the test.

## Preserved identities and readiness limits

Read-only SHA-256 checks confirmed these frozen sources remain exact:

| Path | SHA-256 |
| --- | --- |
| `run_observation_manager.go` | `f9321a620ad64546e735b936d0b58b9f514eab0d7c6d0325f1e7a82f37c5e314` |
| `run_observation_poller_worker.go` | `f90cd397cd983e42e53498f13baa89fbfeac54e07eb3847b3228592ddd4d929f` |
| `run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

No compiler, formatter, Graft build, scanner, test, or Git mutation was run.
This result makes no compile-readiness, assertion, focused-RED, or lifecycle
runtime claim. The previous compiler result remains the authority for the
earlier failed attempt; no retry was made here. A different independent source
reviewer must accept this exact delta before root's separately authorized
compile/RED retry.

Role provenance: this author created the initial `3b0...` Phase A fixture but
did not author the later `4ff2...` fixture repair. This was a bounded compile
repair to the current fixture, not self-approval. No other issue was changed.
