# Phase A focused expected-RED execution assignment — 2026-10-07

**Status: prepared only; no heavy token or command execution is authorized by
this record.** Wait for a separate explicit root grant before any preflight or
test command.

## Complete approval basis

This execution is bounded by the full original Phase A release assignment
`run-observation-manager-phase-a-testfirst-assignment-oct07.md` (SHA-256
`7211787b2f4a07db3d8cddb031ee7f644f38ac5b5331f12ab0de54d5fec66565`), the
repair assignment
`run-observation-manager-phase-a-fixture-repair-assignment-oct07.md` (SHA-256
`dc37aa4b06d995431fadaa581ce9a87e85fc62db7514534523bf1d6d81722d3e`), the
independent source approval
`run-observation-manager-phase-a-fixture-independent-review-oct07.md`
(SHA-256 `0112e6441f30c0ce9ddad695b07de7792dc326afd3f70bd05aa7947fa96d6fcb`),
and its wording erratum
`run-observation-manager-phase-a-fixture-independent-review-erratum-oct07.md`
(SHA-256 `f5112a643afccb40f11ae6651f74fa1a7a78ef9db272565e68b270e7462363ce`).
Those immutable records preserve the full release and repair instructions,
the exact reviewed source scope, and the bounded source-readiness verdict.

## Complete execution instruction received

> PREPARE ONLY pending focused Phase A expected RED execution, NO heavy
> permission yet. Archive full original Phase A 721, repair dc37, source
> APPROVE 0112 plus erratum f511 with entire execution instruction in a new
> immutable assignment. After ROOT explicit heavy grant (normal push session
> 62474 still running), from primary Go APP use env
> `GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1'
> GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1
> CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 JUST_TEMPDIR=/tmp` and run
> `go test -race -count=1 -parallel=1 -timeout=60s -run
> '^TestRunObservationManagerFailure' -v ./internal/server`.
>
> Fresh UTC/resource preflight: available RAM at least 1200 MiB, free swap at
> least 512 MiB, no competing Go/Rust compiler, actual current HEAD at grant,
> and all nine source hashes: manager f932, worker f90, failure fixture 4ff2,
> existing manager fixture af, and six committed primitives. Record the nine
> case inventory. Capture actual preflight/output/exit (and preflight exit if
> captured) to unique `/tmp` originals. After JOIN, before any other gate,
> immediately archive every capture immutably and compare each archive to its
> original. For CR content use a native escaped-JSON lossless wrapper and
> decoded byte comparison; do not use shell `cp` or an escalated writer. For
> literal raw LF content, read original JSON strings preserving tabs and CR,
> require archive destinations absent, then write natively. Do not change
> source/tests, format, scan, mutate Git, or make another attempt. Expected
> result is nine registered cases that compile and fail through ordinary
> assertions caused by the intentionally unwired lifecycle, not a compile/name,
> environment, race, or unintended-pass issue. Record exact failure points and
> later assertions not reached; do not claim GREEN behavior. A source compile
> defect requires rejection and a fresh builder. The existing manager fixture
> is included in package compilation but is not selected by the test filter;
> the nine new cases are the bounded prerequisite. The current source basis
> also includes
> `manager-successful-prepare-read-count-root-decision-oct07.md` for future
> algorithm work, not new RED assertions. Return the heavy token after all
> captures and bounded receipt are complete. Wait for explicit ROOT grant
> before commands.

## Frozen source inventory required at each preflight

| Path | SHA-256 |
| --- | --- |
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | `f9321a620ad64546e735b936d0b58b9f514eab0d7c6d0325f1e7a82f37c5e314` |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go` | `f90cd397cd983e42e53498f13baa89fbfeac54e07eb3847b3228592ddd4d929f` |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go` | `4ff2fe1c922d6ae7e232d5a58f201298e232807437967fe22a9316dcb45aa0f3` |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `apps/godspeed-casework-go/internal/server/run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `apps/godspeed-casework-go/internal/server/run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `apps/godspeed-casework-go/internal/server/run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `apps/godspeed-casework-go/internal/server/run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

## Exact test inventory

The filter must select these nine cases, all unchanged at the reviewed fixture
hash:

1. `TestRunObservationManagerFailureStopWinsBetweenReserveAndLaunch`
2. `TestRunObservationManagerFailureDetachWinsBeforeFirstReadUsesDrainingCode4`
3. `TestRunObservationManagerFailureClaimThenStopUsesWorkerContinuation`
4. `TestRunObservationManagerFailureStopJoinsHeldActualRead`
5. `TestRunObservationManagerFailureGlobalStopReleasesDistinctPendingLeases`
6. `TestRunObservationManagerFailureCanceledPrepareLeavesAuthorizedLease`
7. `TestRunObservationManagerFailureDetachAndSelectedAReleaseHaveOneOwner`
8. `TestRunObservationManagerFailureSharedFirstReadErrorIsSuccessfulA`
9. `TestRunObservationManagerFailureOversizedNilKeyIsAWithoutAdmission`

## Gate protocol and stop conditions

Each actual command requires its own fresh resource/hash preflight after the
explicit grant. Run the single focused command only. Preserve exact original
preflight, output, and exit bytes in unique `/tmp` paths. After command JOIN,
archive and byte-compare all captures before any other gate or response. No
manual transcription or overwrite is permitted. If a source identity or
resource precondition fails, stop before the command and report the observed
block. If compilation fails, identify the exact compiler error and stop for a
fresh repair; do not reclassify it as expected RED. If tests fail, accept only
ordinary assertion failures attributable to the intentionally unwired stubs;
report actual selected/registered case counts and precise failure points, with
no runtime or implementation approval. No retry or additional gate is
authorized by this assignment.
