# Run observation selection — focused actual-RED runtime verdict

Date: 2026-10-06  
Verdict: **ACCEPT the assigned focused actual-RED boundary only.** The source remains a typed-unavailable stub. This is not algorithm/runtime approval and does not release broader gates.

## Frozen sources

| Path | SHA-256 before and after gate |
|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_selection.go` | `1e5cdf404b96bd206f2fc1ad9b6f96365740e8e529079f65aaac5673c1176c50` |
| `apps/godspeed-casework-go/internal/server/run_observation_selection_test.go` | `1e2295a37aec4383f2473226680129f26394253e0ed43d6f05120ef6b30bada4` |

The source hash pair was recorded in the byte-exact preflight and rechecked after the joined test command; both matched.

## Resource preflight and command

Immediately before launch, captured resource preflight reported 2,576,556,032 bytes available RAM (about 2.4 GiB) and 1,368,858,624 bytes free swap (about 1.3 GiB), above the assignment thresholds. `/tmp` had 1,406,943,232 bytes available and the designated Go cache used 314,439,630 bytes. These are the reported process/container-visible values, not a claim about resources outside that namespace. The accepted preflight capture is `run-observation-selection-red-preflight-03-oct06.txt`; `cmp` against its `/tmp` raw capture exited 0.

From `apps/godspeed-casework-go`, with `GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1`, ran:

```text
go test -race -v -count=1 -parallel=1 -timeout=90s ./internal/server -run '^TestSelectObservationRuns'
```

The process was launched through a capture wrapper and joined as session `66279`; exit code was 1. Raw output and exit status are archived as `run-observation-selection-red-focused-01-oct06.raw` and `.exit`. Both archive files byte-compared equal to their unique `/tmp` captures (`cmp` exit 0).

## Observed results and proof boundary

- The command reached all four `TestSelectObservationRuns...` top-level functions and all three `empty`, `empty_nonnil`, and `short` subtests. There were no setup, build, timeout, or race-detector errors.
- The two ranking cases and three successful empty/short subtests failed because the intentionally frozen stub returned typed unavailable where successful selection was expected. These are expected assertion RED failures; later assertions within each failed subtest did not execute.
- `TestSelectObservationRunsRejectsUnknownExecutionWithoutPartialSelection` passed against the stub. That pass establishes only that the fixture's unavailable/no-partial/input-preservation assertion is reachable; the stub's universal error does not prove selector algorithm behavior.
- The focused command used the race detector and returned promptly. No no-spin or pool assertions are in this selector fixture.

No source or test files were edited. No broader package/module/canonical gate, scanner, algorithm implementation, Graft build, Git/status/debt update, or live integration was run. Root must separately authorize any next gate or Phase 2 implementation.
