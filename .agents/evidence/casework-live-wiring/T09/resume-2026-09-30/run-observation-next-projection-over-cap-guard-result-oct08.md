# Next over-cap guard — source change result

Date: 2026-10-08. This is the narrowly released algorithm guard only. It is not lifecycle, Next integration, T09, or runtime completion.

## Full original grant

GUARD RELEASE actualRED verifiedroot: testfails nilerr only, no compileerrors; all6originals archived/rootcmp complete. Freshguardbuilder (different fromoriginalrenderer) allowed ONLYdelta.go add `len(state.Frames) > runObservationRetainedFrameLimit` to existing initial unavailable conditional preservingalllogic. Tests frozen66e368ff; old11frozen. Nativeapply_patch writer; no tests/compiler/Git/gates yet. Optionalread-onlygofmt-d verifyclean noformatwriter. Write NEW immutable run-observation-next-projection-over-cap-guard-result-oct08.md FULLoriginalguardgrant exactpreposthash/diff preservedtestidentity/original12? old11 hashes directverify. No fullalgorithm/t09claim; independentNextcriticwillownserializedfocusedrace+canonicalGo gates. No othersemanticchanges.

## Exact change and identities

Only `apps/godspeed-casework-go/internal/server/run_observation_delta.go` changed. The existing initial unavailable-input condition now also rejects `len(state.Frames) > runObservationRetainedFrameLimit`. No other condition, control flow, error, output, or algorithm logic changed.

- Authorized preimage: 5,541 bytes, SHA-256 `3c81aa06b4f2062e0d3af314635c4c141c9b407d6cee32afe579a03a391cbbb4`.
- Result: 5,599 bytes, SHA-256 `5241e656d65f86a315e37df44b48d5eaae865c92cf80cc306c9bf4fc0fd7e128`.
- Removing the one added guard line from the result reconstructs the authorized preimage byte-for-byte by length and SHA-256.

The test fixture remains frozen at `apps/godspeed-casework-go/internal/server/run_observation_delta_test.go`, 15,232 bytes, SHA-256 `66e368ffdea1839aca74a6c242cb947f6d97fca45a2e228574ec4d1c3adeedbc`. Its 1,025-frame malformed-input case had a focused RED recorded by root: exit 1, empty stderr, sole failure was `error = <nil>, want typed unavailable for over-cap captured window`; no compile/setup error. Root reports all six actual gate captures archived and compared before this guard grant. This builder did not rerun or rearchive the gate.

## Frozen neighboring sources

All eleven source files listed in the accepted TDD preparation receipt were independently rehashed and matched exactly:

| File | SHA-256 |
|---|---|
| `run_observation_manager.go` | `22005e5c15d60c4e99768ac6c69da2d211b816ef16c20663231e8a9fc164dc57` |
| `run_observation_poller_worker.go` | `b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403` |
| `run_observation_manager_authority_terminal_test.go` | `889f9648ead4399ef2d89732422ffed9a3d1ff786e4938b2aafaa45862b2752b` |
| `run_observation_manager_test.go` | `cf7188c23e5bad5f7b0f093561bf85e1e273b48c721a677084429e8038f0ba86` |
| `run_observation_manager_failure_test.go` | `8c71197adc61bf0f0a3fb0a0fdf413ff41f1fdfa54e8a4af0c4a3abf63cb8df8` |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

No test, compiler, formatter, scanner, build, or Git command was run for this guard change. Independent review and root-owned focused/canonical verification remain outstanding. No broader algorithm or completion claim is made.
