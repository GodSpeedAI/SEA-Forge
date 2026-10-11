# Independent Store green review and native T08 rejection

## Verdict

**Scoped approval: the projection Store retention-index repair.** The repair passed the
focused repeated race tests, the projection package race suite, the canonical Casework Go
check, and the full Go module race suite. The source change matches the bounded assignment
below and has no material deviation.

**T08 remains rejected.** After a separate nil-to-empty serialization repair, the native
browser run still failed before proving the required interruption/reconnect sequence. The
current failure is that Chromium did not invoke the native EventSource error callback after
the harness closed one Go-side connection behind the private Vite proxy. This is consistent
with an upstream-only close, but does not itself prove that diagnosis. No native retry or
recovery result is approved.

## Store assignment and source review

The original bounded production assignment was limited to `Store.Append`: after actual
retention eviction, reindex every surviving cursor to its new slice index once under the
existing mutex. Preserve typed evicted-cursor refusal, revision cloning, ordering, broadcast
and subscriber behavior; do not add an `At` guard, allocate a replacement index, change APIs,
schemas or dependencies, or reindex on non-evicting appends.

The independently inspected change in
`apps/godspeed-casework-go/internal/projection/store.go` sets an `evicted` flag in the existing
trim loop and, only when eviction occurred, rewrites each survivor's existing `byCursor` entry
to its current index. There is no `At` behavior change. The new
`apps/godspeed-casework-go/internal/projection/store_retention_test.go` checks retention-one
eviction and lookup, repeated retention-two wraps, each retained and evicted cursor, index
alignment/bounds, and replay ordering. Existing clone tests cover richer revision contents.

The prior independent red report records the failure this repair addresses:
`../store-retention-red-evidence/independent-red-review.md`. Its two raw runs showed an
out-of-range panic at retention one and a wrong revision at retention two. The production
repair fixes the stale slice-index mapping without masking `At` failures.

## Executed Store gates

Each Go command used `GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1
GOCACHE=/home/sprime01/.cache/go-build`; commands were serialized with `-p=1
-parallel=1` where applicable. Direct host memory/compiler preflights preceded each run; the
recorded available memory was above 1 GiB, and no competing Go, Rust, Cargo or Vite compiler
was active.

| Command | Exit | Raw evidence |
| --- | ---: | --- |
| `go test -p=1 -parallel=1 -race -run '^TestStoreRetention(OneKeepsLatestAtAndEvictsPrevious\|TwoKeepsEveryRetainedIndexAndReplaysInOrder)$' -count=25 ./internal/projection` | 0 | `focused-count25.log` |
| `go test -p=1 -parallel=1 -race -count=1 ./internal/projection` | 0 | `projection-package-race.log` |
| `just casework-go-check` | 0 | `casework-go-check.log` |
| `go test -p=1 -parallel=1 -race -count=1 ./...` | 0 | `go-module-race.log` |

Each raw log includes the full environment/command line and `EXIT_CODE`. The canonical check
reports format, vet and tests green. The full-module race log shows all Go packages completed.

## Native T08 attempts and material divergence

The earlier native run `native-live-race.log` exited 1 at the browser's zero-event-request
assertion. The Go state helper copied an empty request slice from `nil`, which JSON encoded as
`null`, while the strict browser assertion reads `.length`. A fresh test-only repair changed
the copy to `append([]nativeEventRequest{}, p.requests...)`; the current
`native_events_live_test.go` retains strict assertions and makes the empty result encode as an
array. No transport, authority, adapter, or production behavior changed in that repair.

The new run `native-live-race-after-empty-requests-fix.log` also exited 1. It passed phase-one
initial resync/snapshot setup and reached the actual interruption step. The browser awaited its
first native EventSource error callback and timed out with `errors=[]`. The test-only
`closeStreams` endpoint reported one closed connection; source inspection shows that the
tracked connection is attached to the Go server request context while the browser talks
through Vite's HTTP proxy. The saved diagnostics report Vite module HTTP 200, successful module
import, no browser window/console errors, and an empty Vite stderr log. These observations
establish the failed callback assertion; they do not establish that the browser's downstream
connection was closed or that a native retry occurred.

The failed run preserved diagnostics at
`/tmp/sea-t08-native-diagnostic-2900998077` and test cells at
`/tmp/sea-t08-f14-failed-cell-799881055/cell` and
`/tmp/sea-t08-f14-failed-cell-1955156880/cell`. The original empty-array failure remains
preserved in `native-live-race.log`; neither run was overwritten. The earlier native Store
panic report and its erratum also remain unchanged. In particular,
`../native-import-diagnosis-errata.md` corrects the earlier diagnosis: that historical run
proved HTTP 200 plus a Store `At` panic only; it did not prove initial SSE or recovery.

Next native verification needs an actual browser-facing connection interruption through the
owned proxy/test topology, with the native `onerror` assertion intact, an observed real
reconnect request, and the required authoritative replay assertions. Do not convert the
timeout into a passing result or substitute a fabricated EventSource error.

## SHA-256 manifest

Source at this review:

```text
044a44b4b42efb6e79f7bf0a61d003d174d17500f147d2eb80e1a61779e43e64  apps/godspeed-casework-go/internal/projection/store.go
c146c7655748fcba8e96f0b9f760604d316b84e9dd95ba6cc1cb8308daf89c25  apps/godspeed-casework-go/internal/projection/store_retention_test.go
89111084e8e43722a771803c19c71296dc75f77f0b35696f435d79f6cadd8552  apps/godspeed-casework-go/internal/server/native_events_live_test.go
ba29bdccfd5bc375ef52b7dd6366be15c8d04cc3924352f8c09137ef69c6ab80  apps/godspeed-cognitive-ui/e2e/native-events-live.ts
```

Raw gate logs:

```text
563427d2619929c2af5cb4746bae499be068c14f1253f1efacc74b162214732b  focused-count25.log
25afbed6f86d83e279a56f2d3c737d07d57bb3d0af053ec6384611533e6b3fac  projection-package-race.log
3af9518e78992ac2e9a93af7725d11cc80e3f7cb9329e1764fe37369052689c4  casework-go-check.log
f8e938c852daf6ee05636543920554b5516284d8d89e774b30ea4cf34b08507c  go-module-race.log
ad164cecaa4e2004f794bcf53179c09efd4f98235d837f034752118dd27405df  native-live-race.log
226b5f67c3e297792b383d092bd212e8d73f3ebd9579c7e264e79c204ac149fa  native-live-race-after-empty-requests-fix.log
```
