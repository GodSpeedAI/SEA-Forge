# Isolated primitives canonical retry and full-module race result

Date: 2026-10-07  
Status: canonical retry and full-module race both passed under the specifically authorized normal listener permissions.

## Scope and source identity

The execution instruction and environment-only retry variation are archived
in `run-observation-primitives-canonical-retry-escalation-assignment-oct07.md`
(SHA-256 `170e37db8cc853194e384d6d8defb11333aaa7b935995adaf451d669d5b170fc`).
The initial sandbox-only canonical attempt and its socket `EPERM` failure are
preserved separately in
`run-observation-primitives-canonical-retry-fullrace-result-oct07.md`; that
attempt was not overwritten.

All three gate preflights recorded isolated root
`/tmp/sea-casework-primitives-74f86f0-oct07`, HEAD
`74f86f0964f92c5b1715eb0703a9b880d28cccae`, and exactly six untracked
primitive source/test paths. Each preflight confirms the same six SHA-256
identities, and all six primary-to-isolated comparisons are zero. No tracked
tests were changed, removed, or omitted from the isolated package tree.

| File | SHA-256 |
|---|---|
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

## Gate 1: canonical package check

The fresh preflight at `2026-10-07T22:18:30Z` recorded
`MemAvailable=2688696 kB`, `SwapFree=10265628 kB`, zero competing Go
processes, zero competing Rust compiler processes, expected HEAD/status, and
matching source hashes. The actual command was run from the isolated root
with `sandbox_permissions=require_escalated`, as specifically authorized
after the earlier local-listener sandbox denial:

```sh
env JUST_TEMPDIR=/tmp GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 just casework-go-check
```

It exited `0`. The actual output states `casework-go-check: format, vet and
tests green`; every listed test package reports `ok`, and the command’s
no-test-file packages report `? ... [no test files]`.

## Gate 2: full-module race

Only after Gate 1 passed and its four captures were archived and compared, a
new preflight was taken at `2026-10-07T22:20:11Z`. It recorded
`MemAvailable=2729440 kB`, `SwapFree=10265712 kB`, zero competing Go
processes, zero competing Rust compiler processes, the same HEAD/status, and
the same six identities/comparisons. The actual command was run from
`/tmp/sea-casework-primitives-74f86f0-oct07/apps/godspeed-casework-go` with
the same environment and specifically authorized normal listener
permissions:

```sh
env JUST_TEMPDIR=/tmp GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 go test -race -count=1 -parallel=1 -timeout=60s ./...
```

It exited `0`. The full actual output lists successful package results for
the Go module; packages without test files are marked `[no test files]`. No
race or package failure is present in the captured output.

## Actual capture archives

Each listed original and archive was compared byte-for-byte (`cmp=0`). The
preflight exit capture is `0\n` for each gate; test/canonical exits are also
`0\n`.

| Capture | Actual original | Archive | Bytes | SHA-256 |
|---|---|---|---:|---|
| Canonical retry preflight | `/tmp/sea-primitives-canonical-retry-critic-20261007/preflight.raw` | `run-observation-primitives-canonical-retry-preflight-oct07.raw` | 1,841 | `48f02be99a73c9636981e177a366a44c93531a9c5880c4bfcb50b532522284bb` |
| Canonical retry preflight exit | `/tmp/sea-primitives-canonical-retry-critic-20261007/preflight.exit` | `run-observation-primitives-canonical-retry-preflight-exit-oct07.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| Canonical stdout+stderr | `/tmp/sea-primitives-canonical-retry-critic-20261007/canonical.stdout.raw` | `run-observation-primitives-canonical-retry-output-oct07.raw` | 1,427 | `c8f78fcba881d8f6078e612d41be8d887b5461485b58b558192bd8dbf6babf89` |
| Canonical exit | `/tmp/sea-primitives-canonical-retry-critic-20261007/canonical.exit` | `run-observation-primitives-canonical-retry-exit-oct07.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| Full-race preflight | `/tmp/sea-primitives-fullrace-critic-20261007/preflight.raw` | `run-observation-primitives-fullrace-preflight-oct07.raw` | 1,841 | `4ddf37d7d4024e543701d0b335b7257d624f122b2faf940b4b8247c3db45c3c6` |
| Full-race preflight exit | `/tmp/sea-primitives-fullrace-critic-20261007/preflight.exit` | `run-observation-primitives-fullrace-preflight-exit-oct07.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| Full-race stdout+stderr | `/tmp/sea-primitives-fullrace-critic-20261007/fullrace.stdout.raw` | `run-observation-primitives-fullrace-output-oct07.raw` | 1,380 | `accb557a72b6e2cdb617eae624842147a65fa34016a50dfc60807811f0f0949d` |
| Full-race exit | `/tmp/sea-primitives-fullrace-critic-20261007/fullrace.exit` | `run-observation-primitives-fullrace-exit-oct07.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |

The captures are actual redirected outputs, not manually transcribed logs.
The earlier four captures from the sandbox-denied attempt remain intact in
the separate initial-result record. No source, fixture, dependency, generator,
Git state, or test selection changed. This result verifies the isolated six-
file primitive scope only; it is not evidence that the unpublished manager
scaffold/failure fixture is implemented, that lifecycle cases pass, or that
T09/public integration is complete.
