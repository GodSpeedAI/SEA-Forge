# Response-cap timeout baseline overlay diagnostic preparation

Date: 2026-10-05

## Scope and execution boundary

Read the root-requested `response-cap-timeout-recon-oct05.md` and independently
prepared a Go `-overlay` baseline for a serialized current-vs-HEAD comparison.
This is preparation only: no compiler, test, or benchmark command was run.
The Rust critic currently owns the compiler token; both proposed commands below
remain unexecuted pending root's explicit token transfer.

## Verified source inputs

- Current `apps/godspeed-casework-go/internal/adapters/sfwp/client.go` SHA-256:
  `e0d3c12c1af75b35c041889a45db7da3978db26bb0226e38149b07ef0f885efe`.
- Original `HEAD:apps/godspeed-casework-go/internal/adapters/sfwp/client.go`
  SHA-256: `8cdfc52a68c07df84e1f88506163da1a8dab212004d45689a4c8ab8007b3b61a`.
- Frozen HEAD bytes are at
  `/tmp/sea-cancellation-oct05/client-head-baseline.go`. They were obtained
  dynamically from `git show HEAD:apps/godspeed-casework-go/internal/adapters/sfwp/client.go`,
  written with the native patch tool, and compared byte-for-byte using
  `git show ... | cmp - /tmp/sea-cancellation-oct05/client-head-baseline.go`
  (exit 0). Its SHA-256 matches the HEAD value above.
- Overlay JSON is
  `/tmp/sea-cancellation-oct05/client-head-overlay.json`. It maps the absolute
  current client path
  `/home/sprime01/projects/sea-rs/apps/godspeed-casework-go/internal/adapters/sfwp/client.go`
  to `/tmp/sea-cancellation-oct05/client-head-baseline.go`.

The baseline/current diff is confined to `conn.call` (`client.go:191-250`):
the current version adds a context `AfterFunc`, joins a running close callback,
marks the checked-out connection dead, and wraps cancellation in unavailable.
No test, fixture, timeout, response cap, or other Go source is substituted by
the overlay.

## Failure context and proposed identical test target

The recon reports race failures in:

- `TestInspectOverLimitResponseRetriesOnceOnFreshConnection`
  (`response_limit_test.go:249-280`)
- `TestOverLimitMutationAndRecoveryResponsesNeverResendMutation`
  (`response_limit_test.go:284-330`)

Both send a response of `approvedResponseLineLimit + 1`, where the limit is
32 MiB (`response_limit_test.go:21,251,286`). Their `testConfig` uses a
two-second `RequestTimeout` and three-second `RecoveryBudget`
(`client_test.go:113-123`). The reported race-run timings and failure are
from the recon/fresh verifier; I did not reproduce them. The recon identifies
payload transfer/read cost and context expiry as plausible, but unproven, and
also flags callback/read error ordering as an alternative.

Use the exact same focused race/count-three command and environment for both
revisions, from `apps/godspeed-casework-go`; run one to completion and retain
its exit/output before starting the next. Capture a fresh actual-host
RAM/process preflight immediately before each command, with the assigned
Go limits and task cache paths. Do not change source, fixture, or timeout.

Baseline command (the sole difference is `-overlay`):

```sh
env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 \
  GOCACHE=/tmp/sea-cancellation-oct05/gocache \
  GOPATH=/tmp/sea-cancellation-oct05/gopath \
  go test -race -count=3 -parallel=1 \
  -overlay=/tmp/sea-cancellation-oct05/client-head-overlay.json \
  ./internal/adapters/sfwp \
  -run '^(TestInspectOverLimitResponseRetriesOnceOnFreshConnection|TestOverLimitMutationAndRecoveryResponsesNeverResendMutation)$'
```

Current command:

```sh
env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 \
  GOCACHE=/tmp/sea-cancellation-oct05/gocache \
  GOPATH=/tmp/sea-cancellation-oct05/gopath \
  go test -race -count=3 -parallel=1 \
  ./internal/adapters/sfwp \
  -run '^(TestInspectOverLimitResponseRetriesOnceOnFreshConnection|TestOverLimitMutationAndRecoveryResponsesNeverResendMutation)$'
```

Keep tests, Go toolchain, host, cache, flags, and order constant. Record both
outcomes even if the first fails; no timeout increase or source repair is
authorized by this preparation. This diagnostic can compare current callback
code with pre-callback HEAD only; it does not by itself approve broader T09.

## Filesystem boundary

Only the new baseline snapshot and overlay JSON were written under `/tmp`, and
this new diagnostic preparation note was added. No tracked source, fixture,
Git metadata, status, debt, or earlier review/evidence artifact was modified.
