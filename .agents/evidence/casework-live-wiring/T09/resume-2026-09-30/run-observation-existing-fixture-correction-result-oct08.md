# Existing manager fixture correction result

Date: 2026-10-08

## Scope and source identities

Only `apps/godspeed-casework-go/internal/server/run_observation_manager_test.go` changed. Original source: 55,815 bytes, SHA-256 `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`. Final source: 56,118 bytes, SHA-256 `cf7188c23e5bad5f7b0f093561bf85e1e273b48c721a677084429e8038f0ba86`.

Exact native preimage archive: `run-observation-existing-fixture-correction-preimage-oct08.json`, SHA-256 `6ade27afc2b7719955d72b9c6909d6b46f14369bb337e6079be17060e3eb7153`. The wrapper declares the source path, original SHA and 55,815-byte size. Its decoded `content` was compared to the source before edits; equality was true.

## Bounded changes

1. `TestRunObservationManagerGuardRefusesBeforeListButValidParentReachesRead`: the Store revision remains at `cursor-old`; before `Prepare`, the existing fake Relay cursor is set to `cursor-current` under its mutex. This makes the Store/Relay mismatch real. The stale guard still must return unavailable with zero DTO/nil lease and zero list/read calls; the valid parent still must reach exactly one list and one trace read. No fixture-name or arbitrary-cursor production behavior is implied.
2. `TestRunObservationManagerAuthorizationRefusalReturnsNoLeaseBeforeList`: preserved the pre-admission refusal's zero perspective/list work, exact retry DTO, one list, zero trace reads, and successful lease/drain. The authorized empty retry now requires exactly four perspective checks, matching the four mandatory authorization boundaries observed in the actual failing run.
3. `TestRunObservationManagerCanceledPartialPrepareJoinsOwnedReadBeforeRetry`: added a buffered first-read signal, emitted only by the first actual `run-first` trace callback invocation. The test waits for both the first and second actual read starts before cancellation. Existing return/JOIN, retry, capacity and exact final `list=2`, `first=2`, `second=2` assertions remain.

Mechanical comparison decoded the preimage and applied five unique, function-local text replacements (each old anchor occurred exactly once). The resulting expected source was byte-equal to the final source (`true`); no other source-text delta was found.

## Frozen identities verified after edit

Production and other fixture files:

| File | SHA-256 |
|---|---|
| `run_observation_manager.go` | `22005e5c15d60c4e99768ac6c69da2d211b816ef16c20663231e8a9fc164dc57` |
| `run_observation_poller_worker.go` | `b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403` |
| `run_observation_manager_authority_terminal_test.go` | `889f9648ead4399ef2d89732422ffed9a3d1ff786e4938b2aafaa45862b2752b` |
| `run_observation_manager_failure_test.go` | `ccbbe234785365a266cd7ea10737158ca3eea2d4a8a88a9f28c24b7260b190f4` |

Six published primitive files:

| File | SHA-256 |
|---|---|
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

All match the frozen identities in the focused-green preflight receipt. The original 53-test focused run reported these three relevant failures: stale Store/Relay fixture returned a populated DTO, the authorized retry observed four perspective checks, and the partial cancellation test observed first/second read counts of 1/2 instead of 2/2. The repair makes those cases exercise the intended mismatch and actual-start ordering while retaining their assertions.

No tests, compiler, formatter, build, scanner, or Git command was run for this repair. This is a source-frozen fixture correction only; it does not claim a new passing runtime gate or lifecycle approval. Independent source review is still required.
