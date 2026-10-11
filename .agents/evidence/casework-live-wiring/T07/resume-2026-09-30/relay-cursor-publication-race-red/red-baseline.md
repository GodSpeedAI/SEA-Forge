# Relay cursor publication race — independent red baseline

Date: 2026-09-30. Branch/HEAD under review: `casework/live-wiring` / `ff7fe6f`. New test snapshot: `relay_cursor_publication_test.go` (sha256 recorded by file-copy command in parent transcript). No production relay edits were present for this run.

## Pre-run resource check

Immediately before the successful focused run, a host `/proc/meminfo` and active-process scan reported:

```text
MemTotal=8132708 KiB
MemAvailable=2921316 KiB
SwapTotal=12582912 KiB
SwapFree=8713344 KiB
active_compiler_processes=[]
```

The check was repeated after one setup failure; there were still no active compiler processes. No other build/test was running. The sole compile token remained with this verifier for this run.

## Attempt 1: cache path was read-only (no package test executed)

Command:

```text
GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 -p=1 -run '^(TestWaitForCaseAdvanceReturnsOnlyAfterRevisionIsQueryable|TestWaitForCaseAdvanceSkipsCaptureAndAppendFailures)$' ./internal/server
```

Working directory: `/home/sprime01/projects/sea-rs/apps/godspeed-casework-go`.

Captured exit: 1.

Captured output:

```text
# ./internal/server
open /home/sprime01/.cache/go-build/e1/e1bb4a9a774672bc2349acd9783ef8d37aa9caacea143bccec05aaf30569db1d-d: read-only file system
FAIL	./internal/server [setup failed]
FAIL
```

This was a Go cache permission setup failure, not a test result. No escalation/bypass was used; the retry moved only Go's build cache into `/tmp`.

## Attempt 2: focused independent race baseline

Command:

```text
GOMAXPROCS=2 GOFLAGS=-p=1 GOCACHE=/tmp/t07-final.uUPtaO/go-cache go test -race -count=1 -p=1 -run '^(TestWaitForCaseAdvanceReturnsOnlyAfterRevisionIsQueryable|TestWaitForCaseAdvanceSkipsCaptureAndAppendFailures)$' ./internal/server
```

Working directory: `/home/sprime01/projects/sea-rs/apps/godspeed-casework-go`.

Captured exit: 1 (expected red).

Exact captured output:

```text
--- FAIL: TestWaitForCaseAdvanceReturnsOnlyAfterRevisionIsQueryable (0.00s)
    relay_cursor_publication_test.go:99: WaitForCaseAdvance returned observed cursor "01BBB" before retained publication
--- FAIL: TestWaitForCaseAdvanceSkipsCaptureAndAppendFailures (0.01s)
    --- FAIL: TestWaitForCaseAdvanceSkipsCaptureAndAppendFailures/capture_error (0.00s)
        relay_cursor_publication_test.go:158: WaitForCaseAdvance returned observed cursor "01BBB" before retained publication
    --- FAIL: TestWaitForCaseAdvanceSkipsCaptureAndAppendFailures/store_append_refusal (0.01s)
        relay_cursor_publication_test.go:158: WaitForCaseAdvance returned observed cursor "01BBB" before retained publication
FAIL
FAIL	github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/server	0.462s
FAIL
```

## Test/source review

The tests pause the first `Facts` call after `Relay.accept` updates `Head` and `CursorForCase`. They assert `WaitForCaseAdvance` remains blocked during capture, and that a successful return is already queryable through both `Store.At` and `Trajectory`. The failure table covers capture error and `Store.Append` refusal, verifies the observed cursor is still available to the stale-intent path, and expects the waiter to wake only after the next retained cursor. Existing fixtures are in the same package and are not build-tagged.

The red result is the expected defect: all three paths woke on observed `01BBB` before the revision was retained. This confirms the live `new_cursor` race. Test helper `requireWaitStillBlocked` uses a 25 ms negative wait; under extreme scheduler starvation that could theoretically mask a premature return, so the builder should consider whether a stronger synchronization assertion is feasible without adding production hooks.

No implementation change was made after this red run. Root has the exact output and has been told the compile token is released for the next authorized phase.
