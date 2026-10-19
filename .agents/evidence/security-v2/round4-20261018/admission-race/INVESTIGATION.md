# B — admission overflow flake: investigation state (2026-10-18)

Failure under investigation (recorded in round 3 `hook-env-escape.txt`):
`conformance_sfwp::request_admission_refuses_overflow_and_cancels_waiters_without_effects`
fails "pre-admission timeout reached provider" (5 vs 4), 1/13 on the round-3
branch and 1/40 on unmodified main, during gated full-suite pushes. A
fast-failure variant (~0.2 s) was also reported.

## Reproduction attempts (worktree security/round4-20261018, test binary
conformance_sfwp-05e7886f2515d8e3, instrumented: per-accept timestamps,
phase timeline, DEBUG tracing with test-writer)

- single test, idle host: 60/60 pass (`run-*.log`)
- single test, 4 CPU hogs on 6 cores: 60/60 pass (`loaded-run-*.log`)
- whole conformance_sfwp binary, 4 CPU hogs: 20/20 pass (`full-run-*.log`)
- full `cargo test -p sea-forge-server`, hogs: in progress (`suite-run-*.log`)

140 consecutive passes, no reproduction yet.

## Static analysis (crates/sea-forge-server/src/lib.rs)

- Permits: `request_admission` = Semaphore(max_concurrent_runs=4),
  `request_admission_waiters` = Semaphore(8), run-level `semaphore` = 4. No
  `add_permits` anywhere; permits are only taken (try, immediate; acquire,
  queued) and dropped after `dispatch_admitted` returns.
- A 5th provider dial requires a request holding BOTH an admission permit
  and a run permit. The four actives hold both and hang at the provider with
  a 30 s endpoint timeout; `bounded_admitted` timeouts at REQUEST_TIMEOUT=10 s
  deliberately do NOT abort admitted work ("work continues"), so no permit
  can be released before ~t=30 s — the test asserts at ~t=10 s.
- The WAITING→GRANTED/CANCELLED compare-exchange handoff is correct on both
  losing sides (task drops ticket+permit; wrapper aborts and awaits).
- Exactly one HTTP request per probe (`OpenAiCompatibleProvider::complete`,
  single POST; `register_endpoint` is ledger-only; no model-resolution call;
  reqwest: no proxy, no redirects, no retry, single resolve_to_addrs).

## Working hypotheses (unproven, reproduction pending)

1. An active probe returns early — e.g. a WSL2 loopback transport failure
   (RST) kills its HTTP request — releasing both permits; a waiter is then
   admitted and dials the 5th connection. Server behavior would be CORRECT
   in that case (a finished request frees capacity); the test's
   `connections == 4` would be a false proxy for "no waiter admitted".
2. A panic inside an active's spawned task (JoinError path) would also
   release its permit; no panic site is apparent on the probe path.
3. The ~0.2 s fast variant may be `Client::connect`'s unwrap under accept
   backlog pressure (test-robustness, distinct from admission semantics).

## Next

The running full-suite loop under load reproduces the original trigger
conditions most closely. On a captured failure, the instrumentation prints:
accept timestamps (which dial was 5th and when), "request admitted" debug
lines (which request_id), and deadline/cancellation warns.

## Update 2026-10-19

Eight further full `cargo test -p sea-forge-server` suite runs under load
(suite-run-2..9): all pass. Cumulative: 140 targeted runs + 9 full-suite
runs, zero reproductions. The static analysis stands: with both semaphores
held by hanging probes until t~30s, no permit can be released before the
~10s assertions, and no double-dial path exists; the leading explanation
remains an environmental transient (WSL2 loopback failure completing an
active probe early, which would make waiter admission correct behavior and
the test's connection-count a false proxy). No fix without a captured
failure; the instrumented harness from this round (accept timestamps,
admission debug lines) is available in git history for the next occurrence.
