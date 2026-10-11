# Focused watcher/terminal RED result — 2026-10-08

## Authorization, command, and actual preflight

This execution used the sole-owner grant in
`run-observation-watcher-terminal-focused-red-assignment-oct08.md` (SHA-256
`74e82cb6a6b7bb0c141459ea7127c486856cb994a523ea394ec9a9d78a031aa0`). The
entire assignment was archived before execution as
`run-observation-watcher-terminal-focused-red-execution-assignment-oct08.md`;
its SHA-256 is the same `74e82cb6a6b7bb0c141459ea7127c486856cb994a523ea394ec9a9d78a031aa0`.

Actual preflight timestamp: `2026-10-08T05:09:39Z`. HEAD was
`7c65be70ecf14c77df1a7749e9fa5526e28eabc1`. `MemAvailable` was 3,489.0 MiB
and `SwapFree` was 2,864.0 MiB. Preflight exit was 0. The compiler-match
section contained only the preflight's own Bash wrapper (PID 2) and `awk`
(PID 44); the process snapshot showed no competing compiler/build/test
process. The shell/awk matches were caused by scanning the preflight command
text itself, not by another compiler process.

The actual command ran from
`/home/sprime01/projects/sea-rs/apps/godspeed-casework-go`:

```text
env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 JUST_TEMPDIR=/tmp go test -race -count=1 -parallel=1 -timeout=60s -run '^TestRunObservationManagerAuthorityTerminal' -v ./internal/server
```

The process joined with exit 1. Its package output ends in the ordinary `FAIL`
summary for `github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/server`.

## Source identities observed at preflight

All eleven source files were hashed and byte-counted from their actual bytes
before execution. The post-gate read-only recomputation matched every identity.

| Source path | SHA-256 | Bytes |
|---|---|---:|
| `apps/godspeed-casework-go/internal/server/run_observation_manager_authority_terminal_test.go` | `ea181a2f5781a7b11fd683642cca89b89c2d902ea44d8ea276a5c9c5c1d4a829` | 41,237 |
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | `47c95f3ba90abb4355f02d626e664a49027f602e9ac47771c40eda0ba8f22780` | 26,548 |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go` | `ccbbe234785365a266cd7ea10737158ca3eea2d4a8a88a9f28c24b7260b190f4` | 47,657 |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go` | `84382ffd0cd77e1c85bd11b607c1427075c4899e1c534fbe7368b0af5ad7c9cc` | 8,873 |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` | 55,815 |
| `apps/godspeed-casework-go/internal/server/run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` | 193 |
| `apps/godspeed-casework-go/internal/server/run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` | 10,456 |
| `apps/godspeed-casework-go/internal/server/run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` | 26,425 |
| `apps/godspeed-casework-go/internal/server/run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` | 13,240 |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` | 3,208 |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` | 11,463 |

The source inventory reported seven top-level test functions: fitting terminal,
recurring outer-cap refusal, revoke-before-read, revoke-during-read,
cursor-change-during-read, invalid watcher/shared survivor, and final Prepare
handoff. There were six nested outcomes: two group-6 scenarios and four
handoff table cells.

## Actual focused outcomes

All seven top-level functions failed. Five of the six nested cases failed; the
`cursor-invalid-empty-list` handoff case passed. These are actual observed
outcomes, not predicted counts.

| Top-level group / nested case | Actual result | Direct failure evidence |
|---|---|---|
| `TestRunObservationManagerAuthorityTerminalFittingTerminalKeepsFinalValueAndStops` | FAIL | The worker started a later actual call; trace call count was 2 (`output` line 2). |
| `TestRunObservationManagerAuthorityTerminalRecurringOuterCapRefusalDrainsAfterJoin` | FAIL | After the held actual call was released and worker completion observed, the test timed out waiting for the terminal-refusal cancellation signal (`output` line 4). This is the bounded assertion timeout, not a process/tool timeout. |
| `TestRunObservationManagerAuthorityTerminalRevocationBeforeSelectedReadFailsClosed` | FAIL | Revoked-before-read Prepare returned a normal DTO, non-nil lease, and one attempted read rather than typed unavailable/zero DTO/nil lease (`output` line 6). |
| `TestRunObservationManagerAuthorityTerminalRevocationDuringReadPreventsPublication` | FAIL | Revoked-during-read Prepare returned a normal DTO and non-nil lease instead of fail-closed output (`output` line 8). |
| `TestRunObservationManagerAuthorityTerminalCursorChangeDuringReadPreventsPublication` | FAIL | Changed-cursor Prepare returned a normal DTO and non-nil lease instead of fail-closed output (`output` line 10). |
| `...InvalidWatcherPreservesSharedSurvivor/during-held-read` | FAIL | The invalid shared initializer owner returned a normal DTO and non-nil lease (`output` line 13). |
| `...InvalidWatcherPreservesSharedSurvivor/invalid-initiator-before-port-survivor-authorizes` | FAIL | The actual trace port began before the initiating worker entered the existing authorization dependency, and the invalid initiator returned a normal DTO/non-nil lease (`output` lines 16–17). This is the expected missing worker-authorization boundary. |
| `...RechecksFinalPrepareHandoff/auth-invalid-empty-list` | FAIL | Auth-invalid empty-list handoff returned a successful no-runs DTO and non-nil lease (`output` line 20). |
| `...RechecksFinalPrepareHandoff/auth-invalid-unavailable-list` | FAIL | Auth-invalid unavailable-list handoff returned an unavailable DTO and non-nil lease rather than typed failure (`output` line 22). |
| `...RechecksFinalPrepareHandoff/cursor-invalid-empty-list` | PASS | The actual output reports `--- PASS` (`output` line 24); this is the allowed current-code pass. |
| `...RechecksFinalPrepareHandoff/cursor-invalid-unavailable-list` | FAIL | Cursor-invalid unavailable-list handoff returned an unavailable DTO and non-nil lease (`output` line 26). |

The focused package compiled and ran under `-race`; the output contains no
compile/setup failure, panic, race report, or environment error. The behavioral
RED is supported by the observed missing authorization/terminal/cleanup
behavior. The one passing cursor-invalid/empty-list cell is permitted by the
assignment. The second group stopped at its bounded cancellation-signal
assertion, so later assertions in that function were not reached. Tests were
not broadened to other packages; this result makes no GREEN, lifecycle, or T09
settlement claim.

## Exact capture archive and comparison

Every `raw` value in the five JSON wrappers below was decoded and byte-compared
to its untouched `/tmp` original. Each comparison returned 0 before this
receipt was written. Original and wrapper identity tables were derived from
the actual captured bytes.

| Kind | Original `/tmp` path | Original bytes / SHA-256 | Immutable BASE wrapper | Wrapper bytes / SHA-256 | Compare |
|---|---|---|---|---|---:|
| preflight | `/tmp/watcher-terminal-focused-red.DUHxkr/preflight.raw` | 93,398 / `52e66387c60403b7a77ce34f0bbe04dd5491d9fbea2f5ee1daeaaaf6fe43478d` | `run-observation-watcher-terminal-focused-red-preflight-oct08.raw.json` | 95,884 / `88c0bc397d235508ba05d8dd1bbb32f2d1831274aaa3521247e2892476537a62` | 0 |
| preflight exit | `/tmp/watcher-terminal-focused-red.DUHxkr/preflight.exit.raw` | 2 / `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `run-observation-watcher-terminal-focused-red-preflight-exit-oct08.raw.json` | 120 / `9ef6364bb520533b99eda9ec32dec745f4906c18bf03061ed2e449630bd8e4a6` | 0 |
| command | `/tmp/watcher-terminal-focused-red.DUHxkr/command.raw` | 320 / `cc74103fca0fa0973b54a4b19a8aa5985daf333a30b49d6ff3a4ed8f009de50e` | `run-observation-watcher-terminal-focused-red-command-oct08.raw.json` | 424 / `2d7170815c59768d6a0e9056498703eab8fc02a76629b915e34caebd663a0942` | 0 |
| output | `/tmp/watcher-terminal-focused-red.DUHxkr/output.raw` | 10,789 / `7a4c8a38511f18e79d0b0e35b251b2a9619e258083c876da90d9478d4cabced7` | `run-observation-watcher-terminal-focused-red-output-oct08.raw.json` | 10,932 / `402ce44df91e07dd04cf33e1b59c68ab6c8a13821433c6b37e5380a301f39e12` | 0 |
| exit | `/tmp/watcher-terminal-focused-red.DUHxkr/exit.raw` | 2 / `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` | `run-observation-watcher-terminal-focused-red-exit-oct08.raw.json` | 100 / `03b595eb487f39927c0f4a26c10e364060a36491bc58f904300a828c6d573807` | 0 |

No additional gate, source edit, formatter, scanner, Graft build, or Git
operation was performed. The sole heavy token is returned to root after this
joined run, exact archive comparisons, and receipt.
