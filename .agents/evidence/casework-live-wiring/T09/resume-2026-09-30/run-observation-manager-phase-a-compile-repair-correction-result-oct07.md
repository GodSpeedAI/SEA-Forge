# Phase A caller-binding correction result — 2026-10-07

## Exact preimage and source identity

Only `apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go`
was edited. The exact source preimage was archived before editing at
`run-observation-manager-failure-test-preimage-compile-repair-correction-oct07.json`
(SHA-256
`658de480baeedd9569c2e581e8e56b9e1d5bca072294e78235cee50f7dc6270a`). Its
JSON `source` string decodes to 27,703 UTF-8 bytes and SHA-256
`7d1470f4b930036561e0c0500d58fb41240994e764754977ce406367b38ba6b0`; decoded
byte count and hash match the archive metadata.

The edited source is 27,703 bytes with SHA-256
`e63e050a7999b5c239257e57c6642baa346c59a40dfa1871e58d7b64ef80a5cf`.
The complete corrective instruction was archived before the source edit at
`run-observation-manager-phase-a-compile-repair-correction-assignment-oct07.md`
(SHA-256
`2583a9f9bc774186bc3c817fc96a721bfbfa5831a02bd8b33820292efac9bf63`). The
rejected prior review is
`run-observation-manager-phase-a-compile-repair-independent-review-oct07.md`,
SHA-256
`d59c70c7cb96898e4cfd45e474766739304770fe0899450dc05c75c4f21b617e`.

## Function-qualified binding changes

In `TestRunObservationManagerFailureStopJoinsHeldActualRead`, restored the
fixture binding needed by its `manager.prepare(..., caller)` goroutine:

```diff
- manager, _, _, _ := newRunObservationFailureFixture(...)
+ manager, caller, _, _ := newRunObservationFailureFixture(...)
```

In `TestRunObservationManagerFailureGlobalStopReleasesDistinctPendingLeases`,
removed the unused local binding. The test's distinct caller identities remain
created inside its `prepare(session)` closure:

```diff
- manager, caller, _, _ := newRunObservationFailureFixture(...)
+ manager, _, _, _ := newRunObservationFailureFixture(...)
```

Comparison against the exact prior `4ff2...` JSON source
(`run-observation-manager-failure-test-preimage-compile-repair-oct07.json`)
shows precisely one hunk: the Global Stop binding change above. The held-read
caller and all other source text match that prior source. Comparison against
the defective `7d1470...` preimage shows the two corrections listed above.
All nine cases, assertions, ordering, and session identities were preserved.

## Frozen source checks and limitations

Read-only SHA-256 checks confirmed the manager, worker, original manager test
fixture, and six published primitive files remain unchanged:

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

The prior compile-repair result incorrectly reported that its single binding
change was in Global Stop. The independent review found it had instead removed
the required held-read caller. That prior immutable result is not rewritten;
this correction records the actual function-qualified edits and exact byte
comparison. No compiler, test, formatter, Graft build, scanner, or Git mutation
was run. No compile or runtime claim is made. A different independent critic
must review this exact source result before a fresh compile/RED retry.
