# Poller image focused GREEN independent acceptance — 2026-10-07

## Bounded verdict

**Accept the pure encoder unit for its focused source and fixture scope.** The post-repair formatting diagnostic had empty output and command status 0. The exact focused test command passed under the race detector: 7 top-level tests and all 3 nested validation subtests passed. The seven source identities in the fresh preflight match the approved encoder/fixture plus five frozen manager/helper/policy files.

This does not establish manager lifecycle behavior, read suppression, worker join/capacity behavior, full-package or full-module Go correctness, T09 completion, or runtime integration. The manager remains intentionally unwired.

## Actual commands and preflights

Formatting diagnostic:

```text
cd apps/godspeed-casework-go && gofmt -d internal/server/run_observation_poller_image.go internal/server/run_observation_manager_retained_image_test.go
```

The fresh preflight at 2026-10-07T18:32:52Z recorded `MemAvailableKiB=3120168`, `SwapFreeKiB=11146584`, no competing compiler process, and Go 1.27.1. All seven expected source hashes matched. `gofmt -d` output was empty (0 bytes) and captured invocation status was 0.

Focused GREEN command:

```text
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 go test -v -race -count=1 -parallel=1 -timeout=60s ./internal/server -run '^TestRunObservationPollerImage'
```

Its separate fresh preflight at 2026-10-07T18:34:02Z recorded `MemAvailableKiB=3081324`, `SwapFreeKiB=11077560`, no competing compiler process, and Go 1.27.1. It again recorded all seven expected source hashes:

- Encoder: `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9`.
- Focused fixture after five-field formatting repair: `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256`.
- Frozen manager: `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`.
- Frozen manager fixture: `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`.
- Frozen retained helper: `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`.
- Frozen retained helper fixture: `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`.
- Frozen policy fixture: `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7`.

The actual test process exited 0. Captured output reports all seven test functions PASS, including three nested invalid-phase, invalid-initializer-result, and current-key-mismatch subtests; `-race` reported no issue. The focused package line reports `ok .../internal/server 2.124s`.

## Capture provenance and exact comparisons

The actual originals remain in these unique temporary directories. Every listed actual/original pair compared byte-for-byte equal (`cmp` exit 0):

| Actual original | Immutable archive | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| `/tmp/poller-image-green-retry-bEjMgt/preflight.raw` | `run-observation-poller-image-green-gofmt-preflight-oct07.raw` | 1300 | `f11a4537a89dfe7d39624b9d4ab066d7cf5d3c0a71846e2d2c0f587946ab972a` |
| `/tmp/poller-image-green-retry-bEjMgt/preflight.exit.raw` | `run-observation-poller-image-green-gofmt-preflight-exit-oct07.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| `/tmp/poller-image-green-retry-bEjMgt/gofmt.output.raw` | `run-observation-poller-image-green-gofmt-output-oct07.raw` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `/tmp/poller-image-green-retry-bEjMgt/gofmt.exit.raw` | `run-observation-poller-image-green-gofmt-exit-oct07.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| `/tmp/poller-image-green-test-8sWTTH/preflight.raw` | `run-observation-poller-image-green-preflight-oct07.raw` | 1438 | `156ffdb3659999ab3d55f4f6271c6d14cddb1a44a825db14d79aa6cbfc7a68f0` |
| `/tmp/poller-image-green-test-8sWTTH/preflight.exit.raw` | `run-observation-poller-image-green-preflight-exit-oct07.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| `/tmp/poller-image-green-test-8sWTTH/test.output.raw` | `run-observation-poller-image-green-output-exact-root-oct07.raw` | 1829 | `809720a6df88052daa6d5b52b423d28fe6c70c925a8cc8963423b04ee7ef412d` |
| `/tmp/poller-image-green-test-8sWTTH/test.exit.raw` | `run-observation-poller-image-green-exit-oct07.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |

### Archive correction and limitation

The first native-patch transcription of the focused test output was inaccurate: `run-observation-poller-image-green-output-oct07.raw` contains spaces at bytes 1748 and 1822 where the untouched actual output has tabs. Its SHA-256 is `578a6c123290bacc28460ced7f46859177b084e4519f8482dded99a7f80f6675`, and it is **not** an accepted capture. It remains unchanged as a record of the failed comparison.

Following root's explicit safe resolution, I read the untouched original into the patch content without manual transcription and archived a separate copy as `run-observation-poller-image-green-output-exact-root-oct07.raw`. That archive compares equal to the actual original, with matching 1829-byte length and SHA-256 `809720a6df88052daa6d5b52b423d28fe6c70c925a8cc8963423b04ee7ef412d`. No source/test content was changed to resolve the evidence mismatch.

The earlier RED capture remains a separate expected-RED result; it is not combined with this GREEN run. No other gate or command was run under this grant. The sole heavy token is returned to root.
