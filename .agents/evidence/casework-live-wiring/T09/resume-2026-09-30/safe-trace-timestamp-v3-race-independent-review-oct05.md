# Independent runtime review: safe trace timestamp repair

Date: 2026-10-05. Result: **FOCUSED V3 SUITE REJECTED; no runtime approval.**
The command joined with actual test exit 1. No retry or broader gate was run.

## Reviewed source and test

The source review is recorded in
`safe-trace-production-timestamp-repair-independent-review-oct05.md` (SHA-256
`e2b9693070db1870e24797b082c8f51d2b4aafbbbb4c865a6239f9ff5978991b`). The exact frozen inputs were:

| Input | SHA-256 |
|---|---|
| `run_trace.go` | `722e32701b4105b0ff4df3c4acb28e34d696521f10f05171f44afbd927077a1d` |
| `run_trace_test.go` | `ee544329730af2dfab4cdfb34cd001f2184879d1fc5ffad285b09564221f7b53` |
| source approval | recorded separately; no runtime approval implied |

The focused race suite ran the requested pattern and emitted failures only for
the following assertions. Timestamp invalid/valid boundary cases emitted no
failure in this run; this result is not a substitute for broader verification.

## Exact command and resource preflight

The command was run through the approved escalated local Unix-socket fixture
path with the requested limits and cache:

```sh
env GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 go test -race -count=1 -parallel=1 ./internal/adapters/sfwp -run '^TestReadRunTraceV3'
```

The exact preflight, raw output, and actual inner exit captures are archived
beside this record and byte-compared with the originals in `/tmp`:

| Capture | SHA-256 |
|---|---|
| `safe-trace-timestamp-v3-race-preflight-20261005T235322Z.txt` | `90cc5dbe926cfd3f91183c6670730c48a67b1a0aed433caf3816084a0dfb8f71` |
| `safe-trace-timestamp-v3-race-raw-20261005T235400Z.txt` | `b49dbf0fd2ba7874d86b4184023fa398d65e834def86fc03e03e5ae321659efa` |
| `safe-trace-timestamp-v3-race-exit-20261005T235400Z.txt` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` |

The Go command's actual captured exit is `1`. The wrapper process exited zero
only after writing and reporting that actual result. Preflight measured
`MemAvailable` 2,842,480 kB and `SwapFree` 4,800,616 kB. Its full unfiltered
namespace-visible `ps -eo pid,comm,rss` output is archived; this container's
process namespace is visibility-limited and the capture is not represented as
host-wide process visibility. Compiler-slot serialization was controlled by
the parent assignment.

## Failures requiring resolution

1. **Successful zero-frame expected value conflicts with the approved
   erratum.** `TestReadRunTraceV3AllowlistIdentityAndRetention` fails for
   returned-row count 0 at `run_trace_test.go:452-457`: `make(..., 0, 0)` creates
   a nonnil empty expected slice, but the approved erratum requires nil frames
   for empty success, and the implementation returns nil. Update only the
   expected zero representation to comply with the erratum; do not change
   nonempty or error assertions.
2. **Raw invalid JSON cases disagree with the observed authority client
   boundary.** `TestReadRunTraceV3RejectsRawInvalidJSON` expects
   `apperr.KindUnavailable` for invalid JSON text and array/string/number JSON
   roots (`run_trace_test.go:532-542`). The actual `Client.Do` path returns
   `apperr.KindInternal` with a response-line decode error before the adapter
   can decode `resp.Raw`; the focused run reports those four mismatches.
   The `null` JSON root case did not fail. This assertion/contract boundary
   needs root adjudication: preserve authority refusal classes while ensuring
   malformed disclosure responses have the intended unavailable classification.
   Do not normalize all `Do` errors indiscriminately.

## Final status

The timestamp source gate was approved, but the focused V3 race suite failed,
so the adapter is not runtime-approved. No further compiler or scanner command
was run. Full SFWP and full-module gates remain held. This record does not
approve broader behavior, settlement, pollers, or publication.
