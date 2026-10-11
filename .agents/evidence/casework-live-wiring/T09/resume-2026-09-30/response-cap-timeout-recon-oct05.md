# T09 response-cap timeout reconnaissance — 2026-10-05

## Scope and limits

Read-only source investigation requested after the fresh independent cancellation verifier reported race-test failures in `TestInspectOverLimitResponseRetriesOnceOnFreshConnection` and `TestOverLimitMutationAndRecoveryResponsesNeverResendMutation` (reported elapsed times about 3.1s and 3.7s). I did not run tests, compile, inspect environment/argv, or change source. Those failure timings are relayed from the verifier, not reproduced here.

## Source findings

- Current `client.go` diff adds `context.AfterFunc` in `conn.call` (`client.go:191-215`): when its context ends, the callback closes the socket; the deferred owner waits for an executing callback and marks the connection dead. It does not add a second read or response-size scan.
- The call already sets socket write/read deadlines to the earlier of its timeout and context deadline (`client.go:216-236`). `roundTrip` derives a `RequestTimeout` context (`client.go:419-428`), discards failed connections, and only retries an inspect while `ctx.Err() == nil` (`client.go:429-454`). Mutations instead enter status recovery only when their request context remains live.
- The fixtures are **32 MiB + 1 byte**, not 16 MiB: `approvedResponseLineLimit = 32 << 20` and both failing tests allocate `strings.Repeat("x", approvedResponseLineLimit+1)` (`response_limit_test.go:20-21, 249-251, 284-287`). The mutation case also sends an oversized status response on its first status read (`:294-302`).
- Both tests use `testConfig` (`response_limit_test.go:267,307`), whose `RequestTimeout` is 2 seconds and `RecoveryBudget` 3 seconds (`client_test.go:113-123`). They pass `context.Background()` (`response_limit_test.go:273,318`). The config default outside this helper is 15 seconds (`client.go:85-87`).
- `readBoundedLine` accumulates into a growing byte slice and detects overflow only once bytes beyond the configured cap have arrived (`client.go:252-276`). The fake server writes the whole returned string with a 5-second socket write deadline (`client_test.go:91-93`). Under a race build, transferring/processing a full 32 MiB payload can therefore consume the test's 2-second request budget; if so, the read deadline/context expiry wins before the cap detector, and `roundTrip` intentionally does not retry/recover once its request context is expired.

## Assessment

The source supports a timeout-path explanation, and the large fixture plus short test timeout makes race-build/host throughput a credible contributor. It does **not** prove that resource pressure caused these failures or exclude a regression from the callback change. In particular, callback closure at context expiry can race with the socket read and change the low-level error observed, but its wrapper retains the context error and the existing roundTrip expiry guards determine that a timed-out call cannot continue safe retry/recovery. No source evidence justifies increasing a timeout.

## Bounded next check

After the sole compiler owner is available, make a sequential, identical full-module race comparison using the current tree and a Go `-overlay` that supplies the pre-callback `client.go` from the parent revision. Keep fixtures, host, build flags, and command identical; record each exact exit and elapsed time. This isolates the only current `client.go` delta without editing tracked source. If both versions time out, investigate race-build/host throughput; if only the current version fails, review callback/read error ordering and connection-dead marking. Do not change timeout values based only on this report.

## Retrieval note

Graft calls located the client path and exact fixture/test symbols before source-span inspection; retrieval saved approximately 119,451 tokens (~$0.10) across three calls.
