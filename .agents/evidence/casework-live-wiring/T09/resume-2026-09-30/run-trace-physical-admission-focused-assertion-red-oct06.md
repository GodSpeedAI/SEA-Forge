# Physical run.get admission — focused assertion-RED attempt

Date: 2026-10-06  
Verdict: **REJECT — compile/setup failure; no fixture assertion ran.**  
Compiler ownership: assigned to this independent critic for this focused command only; process/session joined before this record. No source repair or rerun was made.

## Frozen source identities

| File | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/adapters/sfwp/client.go` | `d2f993d7dda3a5d63a414ef5a64d6f995c04de15f94537374940dc6c031027c6` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission.go` | `f661bd8f8d9046d6eac29738d8521a48ca964361db2e107151cdedce9a0d62a7` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission_test.go` | `d9334d3e1ea1ad61e0c89e68bb18de29ae77dfac33fd83db87dfed3a58e35ac8` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/client_run_get_admission_test.go` | `3466befd626c75785c03ce90c659351ce470f5499ecf69ee5fb8bd18aadcf6cb` |

## Preflight and formatting

Fresh preflight was captured at `/tmp/physical-admission-assertion-red-20261006.preflight.archive`. It reported 2201 MiB available RAM, 2231 MiB free swap, and the assigned GOCACHE existed. `gofmt -d` on both changed fixture files emitted no diff.

## Command and exact result

Working directory: `apps/godspeed-casework-go`.

```text
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 go test -race -count=1 -parallel=1 -timeout=90s ./internal/adapters/sfwp -run '^(TestRunGetLimiterContract|TestClientUsesAdmissionOnlyForPhysicalRunGetAttempts|TestClientAdmissionCountsAllFourRunGetRetryAttempts|TestClientDoesNotRouteAskOrMutationThroughRunGetAdmission|TestMutationTransportFailureIsNotResentThroughAdmission|TestClientAdmissionMarksPayloadAndLFWriteBoundary|TestClientAdmissionFinishesAtFinalAttemptedWriteOnPayloadAndLFFailure|TestClientWaitsForBlockedPayloadAndLFAttemptBeforeFinish|TestClientAdmissionReleaseFollowsCancellationAndConnectionRetirement)$'
```

Actual exit: **1**. Raw output and exit were copied byte-for-byte to `/tmp/physical-admission-assertion-red-20261006.raw.archive` and `/tmp/physical-admission-assertion-red-20261006.exit.archive`; `cmp` passed for both archive copies.

```text
# github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/adapters/sfwp [github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/adapters/sfwp.test]
internal/adapters/sfwp/run_get_admission_test.go:283:3: undefined: t
FAIL    github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/adapters/sfwp [build failed]
FAIL
```

The compiler stops in `queuedRunGetWaiter.cleanup` at `run_get_admission_test.go:283`: the method calls `t.Errorf`, but has no `t` receiver/parameter. This is a source compile error, not an expected stub or missing-hook assertion. No focused test group ran. Specifically, `TestRunGetLimiterContract`, run_get-only routing, all-four-attempt counting, Ask/mutation behavior, payload/LF ordering and failure cases, blocked-write behavior, and callback/pool-retirement ordering are all **unreached**. There is no actual assertion RED evidence from this attempt.

The next step belongs to a different builder under a fresh bounded repair assignment: fix the cleanup reporter's scope while preserving cancellation and bounded join behavior, then obtain a fresh source review before any new compiler ownership. Do not interpret this failure as fixture approval, production verification, or T09 settlement.
