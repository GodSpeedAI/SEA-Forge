# Canonical current-source Go gate result

Date: 2026-10-08

Canonical assignment run-observation-canonical-go-current-source-assignment-oct08.md, SHA-256 8ec3e01783dca962b1ba0ff22b11b07a80b4752dcafebf87c8f911661df50172. Archived full execution instruction run-observation-canonical-go-current-source-execution-assignment-oct08.md, SHA-256 8ec3e01783dca962b1ba0ff22b11b07a80b4752dcafebf87c8f911661df50172. Previous focused retry assignment run-observation-focused-green-retry02-assignment-oct08.md, SHA-256 1ef5657bf949f8d555ba31ffe6d58f3a6c30ffe02b140660f966b1f7769cfa90.

Actual command: env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 JUST_TEMPDIR=/tmp just casework-go-check.
UTC: 2026-10-08T13:00:07.898995+00:00; HEAD: 7c65be70ecf14c77df1a7749e9fa5526e28eabc1; preflight exit: 0; command exit: 1.
MemAvailable: 2920036 KiB; SwapFree: 3771452 KiB. Competing compiler/gate processes visible in the preflight namespace: []. Namespace process visibility cannot prove global host absence.

## Current source identities from the preflight

| Path | Bytes | SHA-256 | Match |
|---|---:|---|---|
| apps/godspeed-casework-go/internal/server/run_observation_manager.go | 29377 | 22005e5c15d60c4e99768ac6c69da2d211b816ef16c20663231e8a9fc164dc57 | True |
| apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go | 13217 | b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403 | True |
| apps/godspeed-casework-go/internal/server/run_observation_manager_authority_terminal_test.go | 41286 | 889f9648ead4399ef2d89732422ffed9a3d1ff786e4938b2aafaa45862b2752b | True |
| apps/godspeed-casework-go/internal/server/run_observation_manager_test.go | 56118 | cf7188c23e5bad5f7b0f093561bf85e1e273b48c721a677084429e8038f0ba86 | True |
| apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go | 47657 | ccbbe234785365a266cd7ea10737158ca3eea2d4a8a88a9f28c24b7260b190f4 | True |
| apps/godspeed-casework-go/internal/server/run_observation_key.go | 193 | a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b | True |
| apps/godspeed-casework-go/internal/server/run_observation_retained_version.go | 10456 | 2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd | True |
| apps/godspeed-casework-go/internal/server/run_observation_retained_version_test.go | 26425 | 34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7 | True |
| apps/godspeed-casework-go/internal/server/run_observation_retained_policy_test.go | 13240 | e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28 | True |
| apps/godspeed-casework-go/internal/server/run_observation_poller_image.go | 3208 | 3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9 | True |
| apps/godspeed-casework-go/internal/server/run_observation_manager_retained_image_test.go | 11463 | cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256 | True |

## Actual failure

The command exited 1 before the test stage. Standard output was 0 bytes; standard error was 154 bytes, SHA-256 16491323d82b83d63515bb2e980fdb5344488f8f127a7ba74ee4442c6cac26d7. The captured stderr was:

    casework-go-check: gofmt would rewrite:
    internal/server/run_observation_manager_failure_test.go
    error: recipe `casework-go-check` failed with exit code 1

The canonical recipe stopped because gofmt would rewrite internal/server/run_observation_manager_failure_test.go. No test outcomes were produced. No formatter write, retry, or second gate was run. This is an actual canonical-gate failure, not a passing result.

## Actual capture archive and byte comparisons

All six capture records were decoded and byte-compared with their untouched original files after JOIN; each matched.

| Kind | Archive | Original /tmp path | Bytes | SHA-256 | Compare |
|---|---|---|---:|---|---|
| command | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-canonical-go-command-oct08.raw.json | /tmp/sea-observation-canonical-go-E3JO4A/command.raw | 223 | 8ed7019c579aef82703af01d7dbee4e417fe25307bef7333315be65ce2e03c3e | true |
| preflight | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-canonical-go-preflight-oct08.raw.json | /tmp/sea-observation-canonical-go-E3JO4A/preflight.raw | 3499 | fb07d238c0f15ad444fae9be3bb8afddaba1e20958e070a712705c35ab63145d | true |
| preflight-exit | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-canonical-go-preflight-exit-oct08.raw.json | /tmp/sea-observation-canonical-go-E3JO4A/preflight.exit.raw | 1 | 5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9 | true |
| stdout | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-canonical-go-stdout-oct08.raw.json | /tmp/sea-observation-canonical-go-E3JO4A/stdout.raw | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 | true |
| stderr | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-canonical-go-stderr-oct08.raw.json | /tmp/sea-observation-canonical-go-E3JO4A/stderr.raw | 154 | 16491323d82b83d63515bb2e980fdb5344488f8f127a7ba74ee4442c6cac26d7 | true |
| exit | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-canonical-go-exit-oct08.raw.json | /tmp/sea-observation-canonical-go-E3JO4A/exit.raw | 1 | 6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b | true |

Originals remain in /tmp/sea-observation-canonical-go-E3JO4A. No lifecycle/T09 completion is claimed.

