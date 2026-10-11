# Safe-trace Phase1 independent review: Unix socket environment failure

Date: 2026-10-05. Fixture source review: the one-line Close repair is correct and its reconstructed pre-repair SHA matches the prior frozen fixture. Runtime disposition: **REJECT for intended assertion RED in the default tool sandbox**; the test command compiled and began running, but fixture setup failed before assertions.

## Source identities and repair

The fresh builder changed only `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go`, replacing `_ = f.client.Close()` with `f.client.Close()`. Current test SHA is fc9bbb748766dd2590b73c2a0ad1b0309e9dc62c2bd7d163e862d310a7b391cd; port and typed stub remain 88191de13d6a21df988dcc3cb834df9015cdb6c2c2aa73a555e10583b108daf5 and 944cf97fd4d6b055f6ec3940086eb3f7ccbce28b7ad0642d023c1a80870c0e51. I reversed only that Close line in a temporary reconstruction; its SHA exactly matches frozen pre-repair e8859aa2e88632f03301cb4f4d97b9039e9105a96a2eaa2c97092f2d104a1155, and diff shows only the one-line replacement. The close call now matches its void API signature.

## Focused command result

After a fresh host preflight and no active compiler, this only authorized Go command was run with the existing writable /tmp GOCACHE and required limits:

`GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 go test -race -count=1 -parallel=1 ./internal/adapters/sfwp -run '^TestReadRunTraceV3'`

It joined exit 1. The package compiled and selected tests began, but every test requiring the fixture failed at `net.Listen("unix", ...)` with `setsockopt: operation not permitted`. Thus the temporary-unavailable stub assertion was not reached, no behavior RED is proven, and fixture close/join was not exercised at runtime.

The preflight was captured at 2026-10-05 22:44:26 UTC, with full `/proc/meminfo` and unfiltered PID/comm/RSS table. It recorded MemAvailable 2,636,272 kB and SwapFree 4,721,680 kB; only codex, bash, and ps were present before the command. Post-run scan found no lingering compiler; MemAvailable was 2,686,064 kB and SwapFree 4,781,644 kB. Exact preflight, full test output and exit are adjacent in `safe-trace-phase1-red-critic-fc9bbb74-preflight.raw`, `safe-trace-phase1-red-critic-fc9bbb74.raw`, and `safe-trace-phase1-red-critic-fc9bbb74.exit`, byte-matched to the originals under `/tmp`.

## Material fixture cleanup observation

When `net.Listen` fails, `newRunTraceFixture` has already created its temp directory but calls `t.Fatal` without removing that directory. This suite run may therefore have left `sfwp-run-trace-*` directories under `/tmp`. I did not delete them or alter source; this bounded setup-failure cleanup gap is reported for the builder/root to assess.

## Disposition

The source matrix remains as accepted by the exact-hash READY review, with the cleanup call compile blocker repaired. The focused default-sandbox run is still not an assertion RED. No production implementation or source change was made by this critic; no broader Go gate ran. Any further attempt must use a separately authorized mechanism that permits the fixture's local Unix socket and must retain the same kernel/project restrictions, limits, full preflight, and exact captures.
