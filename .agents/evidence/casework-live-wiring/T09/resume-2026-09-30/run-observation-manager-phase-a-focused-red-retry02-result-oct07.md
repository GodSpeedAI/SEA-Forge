# Phase A focused RED retry 02 result — 2026-10-07

## Gate and preflight

Executed the single root-authorized focused race command from
`apps/godspeed-casework-go`:

```text
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 JUST_TEMPDIR=/tmp go test -race -count=1 -parallel=1 -timeout=60s -run '^TestRunObservationManagerFailure' -v ./internal/server
```

Fresh preflight at `2026-10-07T23:12:02Z` reported `MemAvailable: 3986500 kB`
(about 3.8 GiB), `SwapFree: 8812624 kB` (about 8.4 GiB), no competing Go/Rust
compiler process (`count: 0`), and HEAD
`7c65be70ecf14c77df1a7749e9fa5526e28eabc1`. All ten preflight source hashes
matched the execution assignment, including corrected fixture
`e63e050a7999b5c239257e57c6642baa346c59a40dfa1871e58d7b64ef80a5cf`; the
nine expected test declarations were present.

## Actual result

The test binary compiled and ran all nine selected cases. Observed summary:
**9 run, 0 passed, 9 failed**, package command exit status `1`, elapsed test
time `6.264s`. The failures were test assertions or bounded fixture waits at
the expected unwired Phase A lifecycle boundary; no compiler error, test
selection error, environment error, panic, or race report appeared.

| Case | First observed failure | Assertions not reached in that case |
| --- | --- | --- |
| `StopWinsBetweenReserveAndLaunch` | line 128: `beginPrepareOperation` returned the typed “unit 1 is not wired” unavailable result instead of a reserved creator lease | reserve/stop ordering, launch, readiness/JOIN and zero-read assertions |
| `DetachWinsBeforeFirstReadUsesDrainingCode4` | line 178: `beginPrepareOperation` returned the same unavailable result instead of a lease | detach/read-start barrier, readiness/JOIN and draining/code-4 assertions |
| `ClaimThenStopUsesWorkerContinuation` | line 221: `beginPrepareOperation` returned unavailable instead of a creator lease | positive claim, Stop continuation, readiness/JOIN and no-read assertions |
| `StopJoinsHeldActualRead` | line 276: held-read Prepare did not satisfy its successful-Prepare contract; the manager returned the unwired unavailable result | poller lookup, held real read, Stop cancellation and worker JOIN checks |
| `GlobalStopReleasesDistinctPendingLeases` | line 341: timed out waiting for the shared initial read | Stop ownership/cancellation and per-lease release/JOIN assertions |
| `CanceledPrepareLeavesAuthorizedLease` | line 422: timed out waiting for the initializer owned by the first Prepare | cancellation and surviving-lease assertions |
| `DetachAndSelectedAReleaseHaveOneOwner` | line 484: `beginPrepareOperation` returned unavailable instead of a preparing lease | held read, detach/selected-A race, exact membership and JOIN checks |
| `SharedFirstReadErrorIsSuccessfulA` | line 557: timed out waiting for the owner's real first trace read | waiter sharing, A result/counts, exact cleanup and worker JOIN checks |
| `OversizedNilKeyIsAWithoutAdmission` | line 634: Prepare returned unavailable instead of successful A with a nonnil cohort lease | A counts, no-admission, allowed list/zero trace reads and lease detach assertions |

This is the expected bounded Phase A RED prerequisite: the intentional
compile-safe lifecycle stubs prevent the positive lifecycle setup and the
tests stop at their first failing setup/assertion or bounded wait. It does not
prove lifecycle behavior, and later assertions listed above were not reached.
The test selection did include all nine expected cases. The original manager
fixture compiled as part of the package but was not selected by this filter.

## Capture provenance and byte verification

The actual captures originated at `/tmp/phasea-red-retry02-20261007T231500Z/`.
All five were immediately preserved in
`run-observation-manager-phase-a-focused-red-retry02-captures-oct07.json`
(SHA-256 `5c1a082ae54644144fa3cd8e29a29bf2c6270b2fbe947e23b70b2b93079e56a9`).
The JSON strings were decoded and compared with each original file's bytes;
all comparisons returned `cmp=0`:

| Original capture | Bytes | SHA-256 | Comparison |
| --- | ---: | --- | ---: |
| `preflight.raw` | 2,719 | `a53a11951dabb498796024d153a43d069192f9a8493b77cd4686fa556c7eadbf` | 0 |
| `preflight.exit.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | 0 |
| `command.raw` | 341 | `7bd9358813254825fcc74fbec690a7292795cc0ba6a5efa6afb2c8f7dc09a43c` | 0 |
| `test.output.raw` | 3,713 | `fae14762de2ed58d7af7c7d73625db7a1aefb67970543d7e2c963a9ad96cc4ed` | 0 |
| `test.exit.raw` | 2 | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` | 0 |

Preflight exit status was `0`; test exit status was `1`. Captures were
archived before reading test output or running any further command. No retry,
formatter, extra gate, source/test edit, scanner, or Git mutation was run.

## Scope and disposition

This independently supports only the focused expected-RED prerequisite for
the exact reviewed Phase A fixture and frozen source set. It does not approve
or verify the lifecycle implementation, does not claim GREEN behavior, and
does not settle the broader T09 milestone. Heavy-token ownership returns to
root after this result.
