# Independent integration runtime review: bounded trace and auth prerequisites

Date: 2026-10-06. Verdict: **APPROVED for the bounded trace and auth
prerequisites covered by these source reviews and runtime gates.** This is not
approval of full T09/SSE/poller integration.

## Frozen source and independent reviews

The trace files remain at the identities independently reviewed and exercised:

| Source | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go` | `3b6b0a7320c73143aca0b101b89e3c64472e85d43f76f00976aac4a21d692d39` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go` | `dd97281929948a99a279c6133fc7568bb7fdf6a686e85b8bfdd0a89a323da78e` |
| `apps/godspeed-casework-go/internal/auth/session.go` | `d1c9daabfb758de94ea20ea0b7db7fbce1fad3a4ec43f6dee461d428b9972aa5` |
| `apps/godspeed-casework-go/internal/auth/session_observation_test.go` | `8de408be7efc31d6e19d73870bee541d82f127272b5c2ab048dbee6ba3212e3f` |

Trace source approval is in
`safe-trace-runtime-classification-independent-review-oct05.md` (SHA-256
`3cffd0540748d2aabcd36cb25b740076e0ff91ec53fe24c3f3bcc10e20452ebd`); its
runtime record, including focused and full SFWP race runs, is
`safe-trace-runtime-classification-independent-runtime-review-oct06.md` (SHA
`bcb65a47bf44ef574d0fb99ba8e939288e6bb9b4de87b73c77fdc9c5b0ab6b75`). Auth
source and runtime reviews are respectively
`observation-session-lifecycle-production-independent-review-oct05.md`
(SHA `c36695b7c3fb54d5cb8cbd9969676943c7d70791f7960312edf60d839040d553`)
and `observation-session-lifecycle-production-runtime-review-oct05.md`
(SHA `8930d812ddfdbf60c8c88b5e72eb9222d378b291b8190e454c2b97206347d017`).
The reviewed auth change is limited to the bounded SessionStore lifecycle
prerequisite; it does not claim that pending reads are canceled or drained.

## Additional integration gates run

After both source reviews and prerequisite gates were accepted, this verifier
ran two further commands sequentially. Each used the approved cache and limits
(`GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1`,
`GOMEMLIMIT=256MiB`, `GOGC=50`, `GOMAXPROCS=2`, `GOFLAGS=-p=1`,
`GOLDEN_UPDATE=0`). The commands ran through the authorized escalated local
Unix/loopback fixture path.

1. Full module race suite:
   `go test -race -count=1 -parallel=1 ./...` from
   `apps/godspeed-casework-go`. Actual joined exit: **0**. Every package with
   tests reported `ok`; packages without tests reported `[no test files]`.
2. Canonical repository gate: `just casework-go-check` from repository root.
   The recipe runs `gofmt -l .`, `go vet ./...`, and `go test ./...`. Actual
   joined exit: **0**, ending with `casework-go-check: format, vet and tests
   green`.

Each command had a fresh preceding `/proc/meminfo` and full unfiltered
namespace-visible `ps -eo pid,comm,rss` preflight. Process visibility is
namespace-limited, not a claim about the host's complete process table.
Preflights, full raw output, and actual inner exit files are archived alongside
this report and byte-compared with their `/tmp` captures.

| Gate | Preflight SHA-256 | Raw SHA-256 | Exit SHA-256 | Actual exit |
|---|---|---|---|---|
| Full module race | `44f86488e5466b4248273277d773b830df943582ba95cf02cd1192aa349d47d0` | `1e531d1db832d3c6fce011628b512405ad46149e7e1fe9e9b3af899416d5a637` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `0` |
| `just casework-go-check` | `7aa1debce20f247e5346fe40a098459bc6b0a52dad2da99cd34fe752ae042762` | `1f96dfc0a197e7757de95465768f496d39b1ec1285306104d12af269025412d2` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `0` |

The current trace evidence bundle also includes the earlier focused V3 race
and full SFWP race passes. The prerequisite auth runtime record reports
focused and full auth race passes with actual exit 0 and byte-verified
captures. No gate was retried after an actual failure.

## Claims and boundaries

These results support the bounded trace projection and bounded auth-store
prerequisite source claims, and show the whole Go module race suite plus the
canonical Go format/vet/test gate pass on the frozen tree. They do not prove
server watcher behavior, deadline re-arm timing, cancellation or draining of
pending trace reads, shared-poller ownership, stale-buffer suppression, SSE
fan-out, UI behavior, or full T09 settlement. No live gates were enabled. No
scanner, Git operation, status update, debt update, or source change was made.
