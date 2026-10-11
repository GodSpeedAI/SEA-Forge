# Canonical Go listener retry03 result

Date: 2026-10-08

Assignment: `run-observation-canonical-go-listener-retry03-assignment-oct08.md`.
Execution assignment: `run-observation-canonical-go-listener-retry03-execution-assignment-oct08.md`.
The original canonical grant and retry02 failure remain part of the record.

## Gate and preflight

The exact root command was run once through `tools.exec_command` with
`sandbox_permissions="require_escalated"`, under the separately authorized
normal listener permission retry. The command and environment were unchanged:

```sh
env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 JUST_TEMPDIR=/tmp just casework-go-check
```

UTC: `2026-10-08T14:25:13,985565403+00:00`; HEAD:
`7c65be70ecf14c77df1a7749e9fa5526e28eabc1`. Preflight exit was 0.
MemAvailable was 2,740,748 KiB; SwapFree was 3,766,180 KiB. No competing
compiler processes were visible in the preflight namespace. Both memory
thresholds passed. Process visibility is namespace-local.

All eleven source identities matched the required preflight values:

| Path | Bytes | SHA-256 |
|---|---:|---|
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | 29377 | `22005e5c15d60c4e99768ac6c69da2d211b816ef16c20663231e8a9fc164dc57` |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go` | 13217 | `b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403` |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_authority_terminal_test.go` | 41286 | `889f9648ead4399ef2d89732422ffed9a3d1ff786e4938b2aafaa45862b2752b` |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_test.go` | 56118 | `cf7188c23e5bad5f7b0f093561bf85e1e273b48c721a677084429e8038f0ba86` |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go` | 47658 | `8c71197adc61bf0f0a3fb0a0fdf413ff41f1fdfa54e8a4af0c4a3abf63cb8df8` |
| `apps/godspeed-casework-go/internal/server/run_observation_key.go` | 193 | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `apps/godspeed-casework-go/internal/server/run_observation_retained_version.go` | 10456 | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `apps/godspeed-casework-go/internal/server/run_observation_retained_version_test.go` | 26425 | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `apps/godspeed-casework-go/internal/server/run_observation_retained_policy_test.go` | 13240 | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_image.go` | 3208 | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_retained_image_test.go` | 11463 | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

## Actual result

The gate exited 0. Captured stdout was 1,427 bytes, SHA-256
`7003ecbf100c1f569738dad1a95db5bbd257229e2f7c5f1917d8f05cdb56afa7`; stderr
was empty. Output reports 10 packages `ok`, 5 packages with no test files, and
ends `casework-go-check: format, vet and tests green`. The recipe completed its
format, vet, and test checks. This result is limited to this canonical
invocation; it does not claim full-module race or lifecycle/T09 completion.

## Captures and comparisons

Each archive preserves the exact original capture as Base64, its kind, and
original path. Each decoded archive was compared byte-for-byte with the
untouched original; all six comparisons were true.

| Kind | Original path | Bytes | Original SHA-256 | Archive SHA-256 | Compare |
|---|---|---:|---|---|---|
| command | `/tmp/sea-observation-canonical-go-retry03-X621fl/command.raw` | 223 | `8ed7019c579aef82703af01d7dbee4e417fe25307bef7333315be65ce2e03c3e` | `cc6a58a31d1eb64e453f1788742812ed7b44b6521bf2e73eb8bd2a534f56c190` | true |
| preflight | `/tmp/sea-observation-canonical-go-retry03-X621fl/preflight.raw` | 2706 | `0cd18df17a0963874d0bf646feaf16996aecc6e20618c40d541e51df9ad07949` | `6051810d16967bbe017329df81f540a258811f63c604c2db2853b390cb89bbc2` | true |
| preflight-exit | `/tmp/sea-observation-canonical-go-retry03-X621fl/preflight.exit.raw` | 17 | `e396bbad2c5a2d243598acb1036bff9f6405c1515f9e364c9e7b0499e10bc275` | `dad7987c765d959cc2fd4d98900d969c7012d2031921b05bb935a189cbf2c61a` | true |
| stdout | `/tmp/sea-observation-canonical-go-retry03-X621fl/stdout.raw` | 1427 | `7003ecbf100c1f569738dad1a95db5bbd257229e2f7c5f1917d8f05cdb56afa7` | `976c21e932464980cf19167cb927bb7e20fe365af4dc45a38985b11537abccfd` | true |
| stderr | `/tmp/sea-observation-canonical-go-retry03-X621fl/stderr.raw` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | `c83817e01e1b25fba578fbab0009ac7f7cc7edbccb44f12e33b2f9886689f91c` | true |
| exit | `/tmp/sea-observation-canonical-go-retry03-X621fl/exit.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `1dd810cd636a08549aa84e427416c21ab6d0d76407d58ed4e22f7bd4d3b658f2` | true |

The immutable archives are the corresponding
`run-observation-canonical-go-retry03-{command,preflight,preflight-exit,stdout,stderr,exit}-oct08.raw.json`
files in this evidence directory. No source or test files were changed. No
second gate was run.
