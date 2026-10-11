# Independent runtime review: safe trace classification repair

Date: 2026-10-06. Verdict: **APPROVED for the focused V3 and full SFWP package
race suites only.** Broader module, settlement, and T09 claims are not approved.

## Reviewed source and binding contract

The independently source-approved implementation and fixture remain unchanged
at the post-run hashes:

| File | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go` | `3b6b0a7320c73143aca0b101b89e3c64472e85d43f76f00976aac4a21d692d39` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go` | `dd97281929948a99a279c6133fc7568bb7fdf6a686e85b8bfdd0a89a323da78e` |
| source review record | `3cffd0540748d2aabcd36cb25b740076e0ff91ec53fe24c3f3bcc10e20452ebd` |

The focused runtime rejection is resolved in accordance with root adjudication:
the successful zero-frame expected value now matches the nil-empty erratum;
the port maps only typed `apperr.Error` response-decoder failures (`Op ==
"decode"`) to the fixed unavailable error, first preserving any `Refusal` in
the chain. Other `Client.Do` errors pass through unchanged. The new
`internal_failure` refusal regression and the original refusal assertions pass
as part of the runs below. V3 timestamp validation, bounded newest-1024
retention, duplicate checking across the full response, source-order/count
rules, and safe projection assertions were source-reviewed and exercised by
the focused/package suites.

## Commands and resource controls

Both commands used the authorized escalated local Unix-socket fixture path and
the same limits/cache. They ran sequentially, with the second command started
only after the focused command’s actual exit 0 was recorded:

```sh
env GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 go test -race -count=1 -parallel=1 ./internal/adapters/sfwp -run '^TestReadRunTraceV3'
```

```sh
env GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 go test -race -count=1 -parallel=1 ./internal/adapters/sfwp
```

The preflights contain actual `/proc/meminfo` RAM/swap and full unfiltered
namespace-visible `ps -eo pid,comm,rss`; process visibility is namespace
limited, while heavy-command serialization was controlled by the root's sole
compiler-token transfer. Each preflight, raw output, and actual exit file is
archived alongside this record and byte-compared with its `/tmp` original.

| Run | Preflight SHA-256 | Raw SHA-256 | Exit SHA-256 | Actual exit |
|---|---|---|---|---|
| Focused V3 | `65c4296162c2e37d9db6276d91e2faa2eb3bb68a89141e91b0fc074332e40018` | `5462d27ce03c49214d649d7e74907bba891e92f15d70139bfeef25dce993516e` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `0` |
| Full SFWP | `8c930131a058a32957159991475dc3483c94baf4d30470470a4bec2942e0296d` | `fc293b359af32720edee53edc20d178e935f7b800607d6b4f1b468917a9f242e` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `0` |

The original raw logs are one-line `ok` results (focused 2.867s; full package
24.160s). The evidence directory also contains an initial incorrectly spaced
full-suite raw-copy attempt; it is explicitly **not** the capture. The
`safe-trace-classification-full-sfwp-raw-native-exact-20261006T000500Z.txt`
copy is the corrected exact-byte artifact and matches the raw `/tmp` file.

## Result and limits

Both authorized race commands passed with actual joined exit 0. The focused
run covered the corrected zero-row case, malformed-response decode mapping,
timestamp syntax boundaries, and refusal preservation; the full SFWP package
race suite passed on the same frozen source. No retry, full-module test,
canonical `just` gate, scanner, Git operation, status update, or debt operation
was run. This approves only these two runtime claims and does not settle other
T09 subsystems or publication.
