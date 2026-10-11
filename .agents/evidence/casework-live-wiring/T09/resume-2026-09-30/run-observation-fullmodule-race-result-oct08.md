# Full-module Go race result

Date: 2026-10-08

Execution assignment: `run-observation-fullmodule-race-execution-assignment-oct08.md`.
The full original is `run-observation-fullmodule-race-assignment-oct08.md`.
Canonical listener retry03 passed and is recorded separately.

## Command and preflight

The single race command ran from `apps/godspeed-casework-go` via normal
`require_escalated` listener permission:

```sh
env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 JUST_TEMPDIR=/tmp go test -race -count=1 -parallel=1 -timeout=120s ./...
```

UTC: `2026-10-08T14:29:24,137051507+00:00`; HEAD:
`7c65be70ecf14c77df1a7749e9fa5526e28eabc1`. Preflight exit was 0.
MemAvailable was 2,662,156 KiB; SwapFree was 3,767,588 KiB. No competing
compiler processes were visible in the preflight namespace. Both memory
thresholds passed. Process visibility is namespace-local.

The eleven actual source identities matched the canonical retry03 preflight:

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

The command exited 0 with empty stderr. Stdout was 1,381 bytes, SHA-256
`ffbeb521ee486a6426a9d02f8eace1ff592814a367173056c065027447f1171b`.
It reports 10 packages `ok` and 5 packages with no test files. This is the
result of this uncached full-module race invocation only; it is not a T09
settlement, public wiring, or broader release claim.

## Captures and comparisons

Each archive contains the exact original capture as Base64, kind, and original
`/tmp` path. Each decoded archive was compared byte-for-byte with its
untouched original; all six comparisons were true.

| Kind | Original path | Bytes | Original SHA-256 | Archive SHA-256 | Compare |
|---|---|---:|---|---|---|
| command | `/tmp/sea-observation-fullmodule-race-bro0xb/command.raw` | 255 | `c6b1230056d6ce2a06a99619dcf7c5778141526dddab32b1d38110f85bdc33c0` | `ea20401b30323addb13b21ab234fd6297a6fb6dcb9fd5cd29b92110489762724` | true |
| preflight | `/tmp/sea-observation-fullmodule-race-bro0xb/preflight.raw` | 2706 | `d5c4337fd6b22fbe8c0b10f46bc6de232d3d6be2609d10bffb3e02fc50e95050` | `bee099303aba33cb375bdf4ffbf71ffeb1e7864e3ac84b07ef4b59bbf6759625` | true |
| preflight-exit | `/tmp/sea-observation-fullmodule-race-bro0xb/preflight.exit.raw` | 17 | `e396bbad2c5a2d243598acb1036bff9f6405c1515f9e364c9e7b0499e10bc275` | `84bec7c3584a20a96961e95747118e9f1bda73da98548dc8d31af437febf1493` | true |
| stdout | `/tmp/sea-observation-fullmodule-race-bro0xb/stdout.raw` | 1381 | `ffbeb521ee486a6426a9d02f8eace1ff592814a367173056c065027447f1171b` | `4781749848c77b4ba2867a1d1a910884f6415ec4ce52117d7bda8ee6bd015562` | true |
| stderr | `/tmp/sea-observation-fullmodule-race-bro0xb/stderr.raw` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | `75e4d22c2cadd1818a603fb284630e4815fbfee42dac32771e1e6d84dcead402` | true |
| exit | `/tmp/sea-observation-fullmodule-race-bro0xb/exit.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `395ec329956bcfea8873ec811734c845b54d9d081f02a62e82328c89a7a52063` | true |

The archives are the corresponding
`run-observation-fullmodule-race-{command,preflight,preflight-exit,stdout,stderr,exit}-oct08.raw.json`
files in this evidence directory. No source or test files were changed. No
second gate was run.
