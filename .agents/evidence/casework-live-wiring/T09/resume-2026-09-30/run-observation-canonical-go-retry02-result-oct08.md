# Canonical Go retry02 result

Date: 2026-10-08

Assignment: `run-observation-canonical-go-retry02-assignment-oct08.md`,
SHA-256 `2d921f8e18f5cb7056623bd981088b234cda592efc6075bb6d67a49294f24e16`.
Execution instructions: `run-observation-canonical-go-retry02-execution-assignment-oct08.md`.
The original canonical grant and formatter-only review remain controlling.

## Gate and preflight

Command, run once from the repository root:

```sh
env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 JUST_TEMPDIR=/tmp just casework-go-check
```

UTC: `2026-10-08T14:20:31,264858739+00:00`; HEAD:
`7c65be70ecf14c77df1a7749e9fa5526e28eabc1`. Preflight exit was 0.
MemAvailable was 2,911,432 KiB; SwapFree was 3,685,820 KiB. No competing
compiler processes were visible in the preflight namespace. Both memory
thresholds passed. Process visibility is namespace-local.

Source identities recorded in the actual preflight:

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

These are the eleven required identities. The ten unchanged files matched the
prior canonical preflight; the formatter-only fixture identity is the sole
source change.

## Actual outcome

Gate exit was 1. Captured stdout was 54,047 bytes
(SHA-256 `c2ba1f932ea20ea67063bb38ea04b52246521fedeb1858377ff4fc650f0dd1c5`);
stderr was 58 bytes (SHA-256
`c77658a06edef106296a5d3c7ef9a4fff5e3c5a8c543d29c05779e264d0f3d2c`). Three
packages reported failures: `internal/adapters/sfwp`, `internal/auth`, and
`internal/server`. The captured output contains 228 `operation not permitted`
listener errors, including Unix socket setup errors and `httptest` TCP
listener failures. This is an actual failed canonical gate. No permission
retry or further gate was run. No lifecycle/T09 completion is claimed.

## Captures and comparisons

Each archive below contains the exact original capture as Base64, its kind,
and original `/tmp` path. The decoded bytes were compared with the untouched
original; all six comparisons were true.

| Kind | Original path | Bytes | Original SHA-256 | Archive SHA-256 | Compare |
|---|---|---:|---|---|---|
| command | `/tmp/sea-observation-canonical-go-retry02-itCfab/command.raw` | 223 | `8ed7019c579aef82703af01d7dbee4e417fe25307bef7333315be65ce2e03c3e` | `f92938beee202346deda7eff8a19697d75f69bca2c527af86d54eba55bb16fea` | true |
| preflight | `/tmp/sea-observation-canonical-go-retry02-itCfab/preflight.raw` | 2706 | `da0a7018af2c0f19e94b3690a84c0cd31354671170021a5a5f0867c8efa59f59` | `d07fbb10b542478873c0743c1eddb2c9f837a995129b24065c5ebf1cf900c57a` | true |
| preflight-exit | `/tmp/sea-observation-canonical-go-retry02-itCfab/preflight.exit.raw` | 17 | `e396bbad2c5a2d243598acb1036bff9f6405c1515f9e364c9e7b0499e10bc275` | `2c41657b6194bf910fc441e0ab83984c412e135ed4f247d09d48830d16075ff6` | true |
| stdout | `/tmp/sea-observation-canonical-go-retry02-itCfab/stdout.raw` | 54047 | `c2ba1f932ea20ea67063bb38ea04b52246521fedeb1858377ff4fc650f0dd1c5` | `221a80c49965d9fb400620d5baded0b45dc84753a740fc5014902258cb886b66` | true |
| stderr | `/tmp/sea-observation-canonical-go-retry02-itCfab/stderr.raw` | 58 | `c77658a06edef106296a5d3c7ef9a4fff5e3c5a8c543d29c05779e264d0f3d2c` | `279db7a0ec702cedeacd03bac5eb3fc60a6d724b2544d99cc43b46e9b78d7464` | true |
| exit | `/tmp/sea-observation-canonical-go-retry02-itCfab/exit.raw` | 2 | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` | `b613fc0507038db3afe7988735314ea05d349b571af332acda6befb135e67133` | true |

The six immutable archives are the corresponding
`run-observation-canonical-go-retry02-{command,preflight,preflight-exit,stdout,stderr,exit}-oct08.raw.json`
files in this evidence directory. No source or test files were changed for
this retry.
