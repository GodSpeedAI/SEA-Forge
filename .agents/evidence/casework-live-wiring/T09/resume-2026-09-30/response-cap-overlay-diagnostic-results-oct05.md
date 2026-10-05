# Response-cap timeout overlay diagnostic results

Date: 2026-10-05

## Scope and decision

Executed exactly the two serialized diagnostic commands prepared in
`response-cap-overlay-diagnostic-preparation-oct05.md`: baseline HEAD overlay
first, current source second. Each command used `-race -count=3 -parallel=1`
and the same two response-cap tests. Both commands fully exited before the next
command's preflight/run. I do not approve or reject the cancellation client
prerequisite from this diagnostic alone.

Verified source identities:

- Baseline snapshot: `8cdfc52a68c07df84e1f88506163da1a8dab212004d45689a4c8ab8007b3b61a`.
- Current client: `e0d3c12c1af75b35c041889a45db7da3978db26bb0226e38149b07ef0f885efe`.
- Overlay changes only `client.go`; production/test sources were not edited.

## Command results

Shared settings for both: `GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2
GOFLAGS=-p=1 GOLDEN_UPDATE=0`, `GOCACHE=/tmp/sea-cancellation-oct05/gocache`,
and `GOPATH=/tmp/sea-cancellation-oct05/gopath`.

Baseline ran first:

```sh
env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 \
  GOCACHE=/tmp/sea-cancellation-oct05/gocache \
  GOPATH=/tmp/sea-cancellation-oct05/gopath \
  go test -race -count=3 -parallel=1 \
  -overlay=/tmp/sea-cancellation-oct05/client-head-overlay.json \
  ./internal/adapters/sfwp \
  -run '^(TestInspectOverLimitResponseRetriesOnceOnFreshConnection|TestOverLimitMutationAndRecoveryResponsesNeverResendMutation)$'
```

Exit 1, package elapsed `14.948s`. In the three repetitions:

- `TestInspectOverLimitResponseRetriesOnceOnFreshConnection` failed twice
  (4.20s and 3.25s) with the request deadline expiring before safe retry
  success; one repetition passed.
- `TestOverLimitMutationAndRecoveryResponsesNeverResendMutation` failed once
  (2.42s) because correlation recovery did not return the recorded outcome;
  two repetitions passed.

Then current source, only removing `-overlay`:

```sh
env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 \
  GOCACHE=/tmp/sea-cancellation-oct05/gocache \
  GOPATH=/tmp/sea-cancellation-oct05/gopath \
  go test -race -count=3 -parallel=1 \
  ./internal/adapters/sfwp \
  -run '^(TestInspectOverLimitResponseRetriesOnceOnFreshConnection|TestOverLimitMutationAndRecoveryResponsesNeverResendMutation)$'
```

Exit 1, package elapsed `11.567s`. In the three repetitions:

- `TestInspectOverLimitResponseRetriesOnceOnFreshConnection` failed once
  (2.13s) with request deadline expiry; two repetitions passed.
- `TestOverLimitMutationAndRecoveryResponsesNeverResendMutation` failed once
  (3.96s) because correlation recovery did not return the recorded outcome;
  two repetitions passed.

Both failure classes occur with pre-callback HEAD as well as current source.
This establishes that the callback change is not required for these timeout
failures to occur. The differing counts and timings do not isolate whether the
callback changes their frequency or error ordering. The recon's throughput and
short-timeout explanation remains plausible but unproven; no timeout change or
runtime repair follows from this result.

## Host evidence and exact captures

Before each command, an escalated preflight recorded UTC time,
`MemTotal`/`MemAvailable`/swap values, and the full PID/command/RSS process list.
Baseline preflight recorded 8,132,712 kB total, 2,810,544 kB available, and
1,362,532 kB swap free. Current preflight recorded 8,132,712 kB total,
2,725,892 kB available, and 1,378,952 kB swap free. No compiler process was
active in either preflight.

Exact raw, exit, and preflight files are in this directory with prefix
`responsecapdiag-`:

- `baseline-preflight.raw` / `.exit`
- `baseline.raw` / `.exit`
- `current-preflight.raw` / `.exit`
- `current.raw` / `.exit`

Each repository capture was copied dynamically from its `/tmp` capture using
the native patch tool and compared byte-for-byte with `cmp` (all eight
comparisons exited 0). No manual output transcription was used.

## Boundary and token return

No source, test, timeout, status, Git, or debt files were changed. No further
gates were run. The compiler token for this bounded diagnostic is returned to
root. The comparison is diagnostic evidence only and does not supersede the
independent cancellation fixture/runtime gate result.
