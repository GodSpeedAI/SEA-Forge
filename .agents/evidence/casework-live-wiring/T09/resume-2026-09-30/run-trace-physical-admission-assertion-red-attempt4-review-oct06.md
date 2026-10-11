# Physical run.get admission — focused assertion-RED attempt 4

Date: 2026-10-06  
Verdict: **ACCEPT as actual focused assertion RED for the test-first fixture boundary.**  
Compiler ownership: assigned independent critic; the real Go process completed and joined before this verdict. No production source or full gate was run.

## Frozen source identities

| Path | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/adapters/sfwp/client.go` | `d2f993d7dda3a5d63a414ef5a64d6f995c04de15f94537374940dc6c031027c6` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission.go` | `f661bd8f8d9046d6eac29738d8521a48ca964361db2e107151cdedce9a0d62a7` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission_test.go` | `dae0406d503e16546b24c507b02db63489dd8b06cd381b34fd045326d1983d58` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/client_run_get_admission_test.go` | `becd4266dca4a28e7641f5eaab1dafe5dc932e04723895cf5e795e7671524103` |

The retry-counter atomic repair was independently source-approved in `run-trace-physical-admission-retry-counter-source-review-oct06.md`. It changes only the fake-server counter/import; the four response cases and all assertions remain unchanged.

## Preflight and formatting

Fresh preflight: `physical-admission-attempt4-preflight-oct06.txt` reports 2551 MiB available RAM, 1885 MiB free swap, the assigned cache present, and only the current shell/ps/sort process entries. `gofmt -d` on both fixture files emitted zero bytes and exited 0; the zero-byte stdout is recorded from `/tmp/physical-admission-assertion-red-attempt4-20261006.gofmt.raw` rather than represented as a nonempty capture.

## Command and exact result

Working directory: `apps/godspeed-casework-go`. The assigned nine-function test selection and build controls were unchanged; `-v` was added solely to establish reached test groups. The local-only sandbox escalation was used because attempt 2 demonstrated `setsockopt: operation not permitted` for the fixture Unix listeners. The escalated attempt had no setup error.

```text
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 go test -race -count=1 -parallel=1 -timeout=90s -v ./internal/adapters/sfwp -run '^(TestRunGetLimiterContract|TestClientUsesAdmissionOnlyForPhysicalRunGetAttempts|TestClientAdmissionCountsAllFourRunGetRetryAttempts|TestClientDoesNotRouteAskOrMutationThroughRunGetAdmission|TestMutationTransportFailureIsNotResentThroughAdmission|TestClientAdmissionMarksPayloadAndLFWriteBoundary|TestClientAdmissionFinishesAtFinalAttemptedWriteOnPayloadAndLFFailure|TestClientWaitsForBlockedPayloadAndLFAttemptBeforeFinish|TestClientAdmissionReleaseFollowsCancellationAndConnectionRetirement)$'
```

Actual exit: **1**, with expected assertion failures. The preflight, raw output, and exit files under this directory are byte-exact copies of the originals in `/tmp/physical-admission-assertion-red-attempt4-20261006.{preflight,raw,exit}`; `cmp` succeeded for each.

## Reached groups and interpretation

Verbose output contains `=== RUN` for **all nine selected top-level functions**. There was no compiler/type error, sandbox setup failure, timeout, or race report.

- **Expected missing-hook RED:** `TestClientUsesAdmissionOnlyForPhysicalRunGetAttempts` observes no admission ID; `TestClientAdmissionCountsAllFourRunGetRetryAttempts` confirms the four-request path but gets zero admission acquisitions; payload/LF ordering, payload/LF failure accounting, blocked-write finish ordering, and cancellation release ordering all show the absent hook/permit events. These are the intended distinctions while the client remains unchanged.
- **Expected stub RED:** `TestRunGetLimiterContract` reaches its subtests. The constructor dimensions pass; ordinary acquisition-dependent subtests fail immediately with the fixed typed-unavailable stub. The already-canceled context subtest passes the typed-unavailable assertion, which does not establish queued wait behavior.
- **Preserved unaffected behavior:** `TestClientDoesNotRouteAskOrMutationThroughRunGetAdmission` and `TestMutationTransportFailureIsNotResentThroughAdmission` pass. Ask/mutation behavior therefore remains untouched by the absent run_get hook.
- **Explicit unproven portions:** the mandatory no-spin subtest is entered but stops at its initial `permitA` acquisition; it does not reach timer-count, cancellation-wait, or real-release/reacquire assertions. The callback/pool test fails on the absent permit-release event before the later pool-at-release observations. These are not production/limiter proofs and cannot be claimed green from this intentionally incomplete stub.

The failure is an actual compiled assertion RED for the current absence of the limiter algorithm and client hook. It satisfies the assigned test-first proof boundary only. It does not approve runtime implementation, manager logical-budget behavior, hydration, SSE/UI, or T09 settlement. Earlier compile failure, sandbox setup failure, and race failure remain preserved separately and are not overwritten by this accepted attempt.
