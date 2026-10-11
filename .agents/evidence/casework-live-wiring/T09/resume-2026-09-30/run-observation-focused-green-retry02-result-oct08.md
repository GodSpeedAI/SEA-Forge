# Focused race retry 02 result

Date: 2026-10-08

## Assignment and command

Retry assignment: run-observation-focused-green-retry02-assignment-oct08.md, SHA-256 1ef5657bf949f8d555ba31ffe6d58f3a6c30ffe02b140660f966b1f7769cfa90. Archived full execution instruction: run-observation-focused-green-retry02-execution-assignment-oct08.md, SHA-256 1ef5657bf949f8d555ba31ffe6d58f3a6c30ffe02b140660f966b1f7769cfa90. Full original focused assignment: run-observation-watcher-terminal-focused-green-assignment-oct08.md, SHA-256 5cc99e87cdeefd13846af5afe9c5a4281ff4d9258bd71dd874cf0539ff8e5eda. Prior failed run preserved at run-observation-watcher-terminal-focused-green-result-oct08.md, SHA-256 dab62acbba5d783b4051dd45dd691c8e5a2fdff95f7bb32f7b320bbbb3e24404.
Actual command: env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 JUST_TEMPDIR=/tmp go test -race -count=1 -parallel=1 -timeout=120s -run '^TestRunObservation' -v ./internal/server.
UTC: 2026-10-08T12:57:10.339771+00:00; HEAD: 7c65be70ecf14c77df1a7749e9fa5526e28eabc1; preflight exit: 0; test exit: 0.
MemAvailable: 2900576 KiB; SwapFree: 3769420 KiB. Competing compiler processes visible in the preflight namespace: []. Namespace visibility does not prove global host absence.

## Source identities from the actual preflight

| Path | Bytes | SHA-256 | Matches required identity |
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

## Actual test result

Test stdout: 13373 bytes, SHA-256 49eff077badf19ec9212fa824f5a77eeca7bf157fe04f4127d3e3c5437511391; stderr: 0 bytes. The command exited zero. Captured package summary: ok  	github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/server	6.574s.

Parsed verbose outcomes: 53 top-level passed, 0 failed, 0 skipped; 22 nested passed, 0 failed, 0 skipped. All seven authority/terminal fixture groups passed. No failure or race-detector diagnostic appears in captured output. This is evidence for this one focused invocation only, not full-module GREEN, lifecycle/T09 completion, or public wiring.

## Capture archives and byte comparisons

Six actual capture pairs were preserved; each decoded archive was compared to the untouched original and matched byte-for-byte.

| Kind | Archive | Original /tmp path | Bytes | SHA-256 | Compare |
|---|---|---|---:|---|---|
| command | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-focused-green-retry02-command-oct08.raw.json | /tmp/sea-observation-focused-green-retry02-ZdNqcb/command.raw | 297 | a38125aabf6c9164e874198d8e546ab9fa138b5f1bb5865a5db89edc86fcc8bb | true |
| preflight | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-focused-green-retry02-preflight-oct08.raw.json | /tmp/sea-observation-focused-green-retry02-ZdNqcb/preflight.raw | 3292 | 105e08831a84f285e6ce0a790cbd2709d9a3011dad57a2a46690ba6d1d1e9395 | true |
| preflight-exit | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-focused-green-retry02-preflight-exit-oct08.raw.json | /tmp/sea-observation-focused-green-retry02-ZdNqcb/preflight.exit.raw | 1 | 5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9 | true |
| stdout | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-focused-green-retry02-stdout-oct08.raw.json | /tmp/sea-observation-focused-green-retry02-ZdNqcb/stdout.raw | 13373 | 49eff077badf19ec9212fa824f5a77eeca7bf157fe04f4127d3e3c5437511391 | true |
| stderr | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-focused-green-retry02-stderr-oct08.raw.json | /tmp/sea-observation-focused-green-retry02-ZdNqcb/stderr.raw | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 | true |
| exit | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-focused-green-retry02-exit-oct08.raw.json | /tmp/sea-observation-focused-green-retry02-ZdNqcb/exit.raw | 1 | 5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9 | true |

Originals remain in /tmp/sea-observation-focused-green-retry02-ZdNqcb. No second gate was run.

