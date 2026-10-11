# Phase B focused expected-RED result — 2026-10-08

## Scope and command

This records the single focused expected-RED run authorized by `run-observation-manager-phase-b-focused-red-execution-assignment-oct07.md`. It is evidence for the test-first prerequisite only; it does not approve manager behavior or claim a green lifecycle implementation.

Working directory: `/home/sprime01/projects/sea-rs/apps/godspeed-casework-go`.

Exact command and environment:

```sh
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 JUST_TEMPDIR=/tmp go test -race -count=1 -parallel=1 -timeout=60s -run '^TestRunObservationManagerFailure' -v ./internal/server
```

The command completed with process exit status 1. Captured verbose output records 12 test cases run, 0 passing, and 12 failing. All failures were ordinary assertions or bounded fixture waits against intentionally unwired Phase B stubs. Four cases reported the unavailable `beginPrepareOperation` stub; five list/read cases observed the unwired `prepare` stub before expected list/read state; three cases timed out their fixture's two-second wait for a read/start that the stub does not perform. No Go compiler error, test-harness timeout, panic, or race warning appeared in the captured output. The run therefore establishes expected RED only; later assertions after each failure were not reached and are not claimed as verified.

## Preflight identities and resources

The actual preflight captured UTC `2026-10-08T03:49:32Z`, `MemAvailable=3,343,648 KiB`, `SwapFree=3,565,852 KiB`, `COMPETING_PROCESSES=none`, and Git HEAD `7c65be70ecf14c77df1a7749e9fa5526e28eabc1`. It inventoried the required 12 failure tests.

All ten source identities recorded by the actual preflight:

| Source | SHA-256 |
| --- | --- |
| `internal/server/manager.go` | `b7f0511b5a1690253d3adb0132beafc106d3b03f5218c6bde3f1c40f4217884a` |
| `internal/server/manager_failure_test.go` | `ef587b0bd64fe42d5fb56369f620f969610a5bac3d04cd7578911472b1a4893c` |
| `internal/server/poller_worker.go` | `f90cd397cd983e42e53498f13baa89fbfeac54e07eb3847b3228592ddd4d929f` |
| `internal/server/manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `internal/server/run_observation_key.go` | `a6a0604c` |
| `internal/server/run_observation_retained.go` | `2157583f` |
| `internal/server/run_observation_retained_image.go` | `base34` |
| `internal/server/run_observation_poller_image.go` | `3dba418f` |
| `internal/server/run_observation_poller_image_test.go` | `cf501bf7` |
| `internal/server/run_observation_primitives_policy.go` | `e156c9cf` |

The final six abbreviated identities are the exact prefixes recorded in the full raw preflight; the first four are full SHA-256 values. No source or test file was changed for this run.

## Actual captures and byte checks

The five original capture files are in `/tmp/phase-b-red.nrOxaO`. Each was archived immediately after the process joined as a lossless JSON string wrapper under the BASE directory. Decoded archive bytes were compared to the corresponding original; all five comparisons returned `cmp` status 0.

| Capture | Original SHA-256 / bytes | Immutable archive | Archive SHA-256 |
| --- | --- | --- | --- |
| Preflight stdout `/tmp/phase-b-red.nrOxaO/preflight.raw` | `98b753d8c4edfbc8f9d24e9903e4a0e0fb49a851578be059229d56309206be1c` / 2687 | `run-observation-manager-phase-b-focused-red-preflight-oct08.raw.json` | `45a47e6cfd322d38ee1bd93ed4645e0ed23628c6fda7739bc4c054fdaffc6236` |
| Preflight exit `/tmp/phase-b-red.nrOxaO/preflight.exit.raw` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` / 2 | `run-observation-manager-phase-b-focused-red-preflight-exit-oct08.raw.json` | `ef699b14481b08f5d76204811e4f6ac1afdb834e7ec727579e3f92df26bb1e66` |
| Exact command `/tmp/phase-b-red.nrOxaO/command.raw` | `edb582f97be766322e2287c217b24eff901a3b18483fa9e68bf30063f96349a9` / 306 | `run-observation-manager-phase-b-focused-red-command-oct08.raw.json` | `1f8d4978ad81183440a9e2e57a735ac502e1684249410b0a1f76725691b9d385` |
| Test output `/tmp/phase-b-red.nrOxaO/output.raw` | `95051f1b8bcc7e12fe235a73bef2387303868241185da86094cc8aff87e143bc` / 5758 | `run-observation-manager-phase-b-focused-red-output-oct08.raw.json` | `dc992eb63dbba43e920446dc0914f43727247fc2f2270572edbcd7475c166cd6` |
| Test exit `/tmp/phase-b-red.nrOxaO/exit.raw` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` / 2 | `run-observation-manager-phase-b-focused-red-exit-oct08.raw.json` | `af8765b63f988bfb2fb920b48aa7947a6a2cf122377575ae90a6a148e1fb27fd` |

No further test, formatter, build, or verification gate was run. The exact captured output and preflight are the source of truth for this bounded result; no package-wide or runtime correctness conclusion is implied.
