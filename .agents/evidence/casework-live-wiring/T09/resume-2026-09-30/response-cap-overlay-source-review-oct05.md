# Response-cap baseline overlay — independent source readiness review

Date: 2026-10-05

## Decision

The proposed diagnostic is source-ready for root to transfer the compiler token
to its assigned worker. The overlay is a one-entry Go overlay that substitutes
only the absolute `client.go` path with the exact `HEAD` source snapshot. The
proposed baseline/current commands use the same package, test regex, race/count
settings, environment, cache, and timeouts; the baseline command adds only the
overlay flag. This review ran no compiler, tests, or benchmarks and grants no
approval of the cancellation change or broader T09 work.

## Exact source identities

- Current `apps/godspeed-casework-go/internal/adapters/sfwp/client.go`:
  SHA-256 `e0d3c12c1af75b35c041889a45db7da3978db26bb0226e38149b07ef0f885efe`
  (20,006 bytes).
- Baseline `/tmp/sea-cancellation-oct05/client-head-baseline.go`:
  SHA-256 `8cdfc52a68c07df84e1f88506163da1a8dab212004d45689a4c8ab8007b3b61a`
  (19,380 bytes).
- Overlay `/tmp/sea-cancellation-oct05/client-head-overlay.json`:
  SHA-256 `272857546158486fb249a7cb1190a2a8b594c2afa0c4faf7f7d28249b61ace69`
  (174 bytes).

I independently ran
`git show HEAD:apps/godspeed-casework-go/internal/adapters/sfwp/client.go | cmp - /tmp/sea-cancellation-oct05/client-head-baseline.go`;
`cmp` exited 0. The same `git show` stream hashed to the baseline SHA-256
above. No whole file was printed. The overlay JSON parses to exactly one
replacement: absolute current client path
`/home/sprime01/projects/sea-rs/apps/godspeed-casework-go/internal/adapters/sfwp/client.go`
maps to `/tmp/sea-cancellation-oct05/client-head-baseline.go`.

The current-versus-baseline diff is confined to `conn.call`
(`client.go:191-215`). Current source adds the context `AfterFunc`, waits for
the callback if it has started, sets `cn.dead` before the owner unlocks, and
wraps interrupted I/O with the context error. No response-cap algorithm,
timeout, configuration, retry path, or other client source differs in this
diff. It does not substitute or rewrite fixtures.

## Tests, configuration, and command symmetry

The target tests remain the existing 32 MiB + 1 byte response cases:
`response_limit_test.go:20-21,249-281,284-330`. Both proposed commands select
exactly:

`^(TestInspectOverLimitResponseRetriesOnceOnFreshConnection|TestOverLimitMutationAndRecoveryResponsesNeverResendMutation)$`

Both use `-race -count=3 -parallel=1 ./internal/adapters/sfwp`, the same
`GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0`, and
the same `/tmp/sea-cancellation-oct05/gocache` and `gopath`. They retain
`response_limit_test.go` and `client_test.go` unchanged from HEAD; I verified
both with `git diff --quiet HEAD -- <path>` and matching current/HEAD hashes.
`client_test.go:113-123` still sets `RequestTimeout=2s`, `RecoveryBudget=3s`,
and `BackoffBase=20ms`. The only proposed command difference is the baseline
command's `-overlay=/tmp/sea-cancellation-oct05/client-head-overlay.json`.
Order is baseline first, then current, with separate completion/evidence before
the next command.

One literal HEAD difference to preserve in reporting: the worktree's
`client_cancellation_test.go` differs from HEAD. Neither command overlays or
rewrites it, so both use the same current fixture file; the regex does not
select its cancellation tests. This makes the baseline a current-worktree test
run with only the client implementation replaced, not a historical full-HEAD
checkout. Package compilation still includes that current fixture in both
runs.

The test configuration explains the diagnostic target: each response fixture
is `approvedResponseLineLimit + 1`; mutation recovery's first status response
is also oversized. `roundTrip` gives each call the two-second test request
deadline, while recovery has its separate three-second budget. The preparation
correctly treats transfer/processing cost versus callback/read error ordering
as competing hypotheses; source inspection does not establish which causes
the earlier timeouts. The comparison can isolate whether substituting the
pre-callback `conn.call` changes the observed outcome for these same fixtures.

## Command review

The exact baseline and current commands in
`response-cap-overlay-diagnostic-preparation-oct05.md` are symmetric in all
test-affecting arguments and environment. Baseline adds only its `-overlay`
argument; current runs the worktree source directly. Keep the assigned order,
collect fresh host preflight before each command, and wait for the process exit
before starting the next. Do not adjust either timeout or edit any source or
fixture as part of this diagnostic.

## Work performed

Only this new review evidence file was added. Read-only `graft`, `git show`,
`git diff`, `cmp`, hash, size, and overlay-JSON inspection commands were used.
No source, test, status, debt, or Git metadata was modified; no compiler,
test, or benchmark was run.
