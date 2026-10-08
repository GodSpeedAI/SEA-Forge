# Focused watcher-terminal verification result

Date: 2026-10-08

## Command and resource preflight

Execution instruction: run-observation-watcher-terminal-focused-green-execution-assignment-oct08.md, SHA-256 5cc99e87cdeefd13846af5afe9c5a4281ff4d9258bd71dd874cf0539ff8e5eda.
Actual command: env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 JUST_TEMPDIR=/tmp go test -race -count=1 -parallel=1 -timeout=120s -run '^TestRunObservation' -v ./internal/server.
Preflight UTC: 2026-10-08T12:42:05.972655+00:00; HEAD: 7c65be70ecf14c77df1a7749e9fa5526e28eabc1; preflight exit: 0.
MemAvailable: 3186208 KiB; SwapFree: 2567656 KiB; compiler processes visible in this namespace: []. The preflight states that namespace visibility cannot prove global host absence. RAM/swap thresholds passed.

## Source identities from the actual preflight

| Path | Bytes | SHA-256 | Matches required identity |
|---|---:|---|---|
| apps/godspeed-casework-go/internal/server/run_observation_manager.go | 29377 | 22005e5c15d60c4e99768ac6c69da2d211b816ef16c20663231e8a9fc164dc57 | True |
| apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go | 13217 | b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403 | True |
| apps/godspeed-casework-go/internal/server/run_observation_manager_authority_terminal_test.go | 41286 | 889f9648ead4399ef2d89732422ffed9a3d1ff786e4938b2aafaa45862b2752b | True |
| apps/godspeed-casework-go/internal/server/run_observation_manager_test.go | 55815 | af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4 | True |
| apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go | 47657 | ccbbe234785365a266cd7ea10737158ca3eea2d4a8a88a9f28c24b7260b190f4 | True |
| apps/godspeed-casework-go/internal/server/run_observation_key.go | 193 | a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b | True |
| apps/godspeed-casework-go/internal/server/run_observation_retained_version.go | 10456 | 2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd | True |
| apps/godspeed-casework-go/internal/server/run_observation_retained_version_test.go | 26425 | 34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7 | True |
| apps/godspeed-casework-go/internal/server/run_observation_retained_policy_test.go | 13240 | e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28 | True |
| apps/godspeed-casework-go/internal/server/run_observation_poller_image.go | 3208 | 3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9 | True |
| apps/godspeed-casework-go/internal/server/run_observation_manager_retained_image_test.go | 11463 | cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256 | True |

## Actual test outcome

Test process exit: 1; stdout was 14570 bytes (SHA-256 29b3faee979d7922009ed73c1d4a682030690e01fbb52e1d827c32a7e978b8b6) and stderr was 0 bytes. The tests compiled and ran, but the package command failed. Captured verbose output contains 50 of 53 top-level outcomes passing and 3 failing; 22 nested outcomes passing and 0 failing; no skips. No race-detector diagnostic was printed, but this failing run is not a race-clean result.

The three reported top-level assertion failures were:

- TestRunObservationManagerGuardRefusesBeforeListButValidParentReachesRead (captured output line 77): stale-context Prepare returned a populated DTO, nonnil lease, and nil error; the assertion expected a zero DTO, nil lease, and unavailable error.
- TestRunObservationManagerAuthorizationRefusalReturnsNoLeaseBeforeList (captured output line 92): authorized retry downstream calls were perspective=4, list=1, trace=0; the assertion expected one perspective check, one list, and no trace reads.
- TestRunObservationManagerCanceledPartialPrepareJoinsOwnedReadBeforeRetry (captured output line 97): post-rollback calls were list=2, first=1, second=2; the assertion expected 2 each.

The authority/terminal fixture’s seven top-level groups passed. The other failed cases are existing manager tests matched by the requested ^TestRunObservation pattern; this receipt records observed failures without diagnosing them or treating them as expected RED. No second run or broader gate was performed.

## Capture archive and byte comparison

Each archive below contains the untouched original path and raw UTF-8 content. The original bytes were read back and compared with the decoded archive after process join; all comparisons were true.

| Kind | Archive | Original /tmp path | Bytes | SHA-256 | Byte compare |
|---|---|---|---:|---|---|
| command | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-watcher-terminal-focused-green-etaBTw-command-oct08.raw.json | /tmp/sea-observation-watcher-terminal-focused-green-etaBTw/command.raw | 297 | a38125aabf6c9164e874198d8e546ab9fa138b5f1bb5865a5db89edc86fcc8bb | true |
| preflight | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-watcher-terminal-focused-green-etaBTw-preflight-oct08.raw.json | /tmp/sea-observation-watcher-terminal-focused-green-etaBTw/preflight.raw | 3306 | 45d5e4282f904567eb4911a71fe0878769db9fb36d47c98ce83ab4f71ce84c68 | true |
| preflight-exit | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-watcher-terminal-focused-green-etaBTw-preflight-exit-oct08.raw.json | /tmp/sea-observation-watcher-terminal-focused-green-etaBTw/preflight.exit.raw | 1 | 5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9 | true |
| stdout | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-watcher-terminal-focused-green-etaBTw-stdout-oct08.raw.json | /tmp/sea-observation-watcher-terminal-focused-green-etaBTw/stdout.raw | 14570 | 29b3faee979d7922009ed73c1d4a682030690e01fbb52e1d827c32a7e978b8b6 | true |
| stderr | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-watcher-terminal-focused-green-etaBTw-stderr-oct08.raw.json | /tmp/sea-observation-watcher-terminal-focused-green-etaBTw/stderr.raw | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 | true |
| exit | .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-watcher-terminal-focused-green-etaBTw-exit-oct08.raw.json | /tmp/sea-observation-watcher-terminal-focused-green-etaBTw/exit.raw | 1 | 6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b | true |

The preflight-exit capture is additionally archived and compared, yielding six capture pairs total. Original files remain in place. This result supports only the observed focused test outcome; it does not claim lifecycle completion, T09 completion, or public wiring.

