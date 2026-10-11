# Physical run.get admission — focused assertion-RED attempt 3 review

Date: 2026-10-06  
Verdict: **REJECT — unexpected race in test fixture; assertion-RED is not acceptable.**  
Compiler ownership: this reviewer held the sole token for the focused attempt. The command completed and its process joined before this verdict. No source repair or further run was made.

## Frozen identities

| Path | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/adapters/sfwp/client.go` | `d2f993d7dda3a5d63a414ef5a64d6f995c04de15f94537374940dc6c031027c6` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission.go` | `f661bd8f8d9046d6eac29738d8521a48ca964361db2e107151cdedce9a0d62a7` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission_test.go` | `dae0406d503e16546b24c507b02db63489dd8b06cd381b34fd045326d1983d58` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/client_run_get_admission_test.go` | `3466befd626c75785c03ce90c659351ce470f5499ecf69ee5fb8bd18aadcf6cb` |

## Preflight and command

Attempt 3 preflight: `/tmp/physical-admission-assertion-red-attempt3-20261006.preflight`; available RAM 2374 MiB, free swap 1993 MiB, assigned GOCACHE existed. `gofmt -d` emitted no diff and exit 0 (`/tmp/physical-admission-assertion-red-attempt3-20261006.gofmt.{raw,exit}`).

The assigned exact focused race command was run with `GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1`, `-race -count=1 -parallel=1 -timeout=90s`, and the assigned nine-function regex in `apps/godspeed-casework-go`. The first sandboxed attempt failed only because local fixture listeners were denied (`setsockopt: operation not permitted`). The subsequent run used the narrowly approved escalation for these local fake Unix-socket fixtures only; the binds succeeded. It exited 1 in 0.189s. Raw and exit are preserved at `/tmp/physical-admission-assertion-red-attempt3-20261006.raw` and `.exit` (exit content `1`).

## Observed results

- **Unexpected race:** `TestClientAdmissionCountsAllFourRunGetRetryAttempts` captures `calls` in its fake-server handler at `client_run_get_admission_test.go:88–98`. `fakeServer` serves accepted connections in separate goroutines; each handler increments/reads this plain integer at line 89. The race detector reported concurrent read/write accesses from two `fakeServer.serve` goroutines, and the test was marked failed for both its expected missing admission hook and `race detected during execution`. This is a fixture race, not an intended assertion RED. Make the handler's response sequencing race-safe (for example, atomic counter or mutex) without changing the four-response sequence, then obtain a different fresh source review before another compiler assignment.
- **Expected absent-hook failures reached:** `TestClientUsesAdmissionOnlyForPhysicalRunGetAttempts` got `admission run IDs=[]`; `TestClientAdmissionCountsAllFourRunGetRetryAttempts` also observed no admission acquisitions; payload/LF event test saw only writes and no permit events; partial/error and blocked write tests saw no permit-start/finish accounting; cancellation cleanup saw close events but no permit-release event.
- **Expected stub failures reached:** `TestRunGetLimiterContract` constructor-dimension subtest passed; each acquisition-dependent subtest failed at its first expected operation with the declared typed-unavailable stub. The no-spin scenario itself was not reached because its initial `permitA` acquisition failed immediately. No limiter behavior is proved.
- **Other selected behavior:** `TestClientDoesNotRouteAskOrMutationThroughRunGetAdmission` and `TestMutationTransportFailureIsNotResentThroughAdmission` emitted no failures, so their selected assertions appear to pass. The cancellation test stops at the expected permit-release ordering failure; its later pool observation is unreached. Passing unaffected assertions do not compensate for the race.
- All nine top-level test functions were selected by the regex. The five fake-server listener tests that failed in attempt 2 were able to bind in attempt 3. This attempt is not accepted as actual assertion RED because the race detector is red.

## Capture preservation limitation

The shell was unable to copy the `/tmp` captures into this `.agents` directory because the filesystem reports it read-only. The original attempt-3 preflight, formatter, raw, and exit files remain in `/tmp` at the paths above; the raw/exit were not overwritten. Root should perform or confirm byte-exact native archival and retain both the sandbox setup failure and this race failure.

No production behavior, full package/module gate, or T09 settlement is approved. The next step is a distinct bounded test-only repair and independent source review; no additional compilation was performed under this token.
