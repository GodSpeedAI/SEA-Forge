# Phase A focused expected-RED execution result — 2026-10-07

**Result: REJECTED before test execution because compilation failed.** The
focused RED did not reach any test assertion. This result is not a RED pass,
runtime result, or lifecycle approval. No repair or retry was performed.

## Assignment and preflight

The execution followed
`run-observation-manager-phase-a-focused-red-execution-assignment-oct07.md`
(SHA-256
`b7fc844d44f76083b81028a0038f54f1e92804d8225a672cfe9a0068db52082e`) after
root's explicit sole-heavy-token grant. It used the primary Go app at
`/home/sprime01/projects/sea-rs/apps/godspeed-casework-go`.

Actual preflight UTC was `2026-10-07T22:58:52Z`; local HEAD was
`7c65be70ecf14c77df1a7749e9fa5526e28eabc1`. Available memory was
`3993596 kB` (~3900 MiB) and free swap was `8792860 kB` (~8586 MiB); both
exceeded the required minimums. The process check found zero competing Go/Rust
compiler processes. The preflight records the exact ten source/fixture hashes
and the nine selected test names/count. It exited 0.

## Actual command and outcome

Environment:

```text
GOMEMLIMIT=256MiB
GOGC=50
GOMAXPROCS=2
GOFLAGS=-p=1 -count=1
GOLDEN_UPDATE=0
GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1
CARGO_BUILD_JOBS=1
RUST_TEST_THREADS=1
JUST_TEMPDIR=/tmp
```

Command:

```text
go test -race -count=1 -parallel=1 -timeout=60s -run '^TestRunObservationManagerFailure' -v ./internal/server
```

The actual output reports a package build failure at
`internal/server/run_observation_manager_failure_test.go:323:11`:
`declared and not used: caller`. The `GlobalStopReleasesDistinctPendingLeases`
fixture binds `manager, caller, _, _` but does not use `caller`. The process
exited 1. No test case ran; no number of assertion failures or passes is
claimed. This is a source compile failure, not the expected ordinary assertion
RED from the intentionally unwired lifecycle stubs. A fresh fixture-only repair
and independent source review are required before another execution grant.

## Actual captures and immediate archive comparison

The originals were created in `/tmp/sea-casework-phasea-red-20261007T225840Z/`.
Immediately after the command joined, all four were added as new immutable
evidence files with native `apply_patch`; each archive was compared with its
original using `cmp` before any further gate. All four comparisons returned 0.

| Capture | Original bytes | Original SHA-256 | Immutable archive |
| --- | ---: | --- | --- |
| Preflight | 2649 | `4d545ef177bfc6d6281846469b890a793daad23ad07be1912e5517e36d3a3a40` | `run-observation-manager-phase-a-red-preflight-oct07.raw` |
| Preflight exit | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `run-observation-manager-phase-a-red-preflight-exit-oct07.raw` |
| Test output | 350 | `3d4faed250c99bd5bf8cff0a376fee1f5c5957a77b8187567c54eeba572e0368` | `run-observation-manager-phase-a-red-output-oct07.raw` |
| Test exit | 2 | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` | `run-observation-manager-phase-a-red-exit-oct07.raw` |

Each archive has the same byte count and SHA-256 as its corresponding `/tmp`
original. The output and exit originals remain unchanged in `/tmp`.

## Scope and limitations

No source/test changes, formatting, scanner, extra gate, retry, or Git mutation
was performed. The three Phase A paths remain at the preflight hashes. The
focused command was attempted once and stopped at package compilation. No
expected-RED, assertion-reachability, race-cleanliness, runtime, manager
lifecycle, or T09 completion claim is supported. The sole heavy token is
returned after this capture receipt.
