# Run observation manager Unit1 RED01: independent negative-evidence verdict

Date: 2026-10-07  
Disposition: **ACCEPT as one focused semantic expected-RED observation only**

## Assignment, approval, and scope

The original Unit1 assignment is `run-observation-manager-unit1-original-assignment-oct06.md` (SHA-256 `de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916`). It authorizes a private unexported test-first scaffold, not the manager algorithm or integration. The repair 3 independent source review (`run-observation-manager-unit1-repair3-cleanup-independent-source-review-oct07.md`, SHA-256 `5133db7ed624733183e88b93bd5ae55a540a1ed754dd2b5e82c6d94cc8cb601b`) approved only a separately authorized focused expected-RED case. Root selected `TestRunObservationManagerCanceledPartialPrepareJoinsOwnedReadBeforeRetry` and was sole runtime/compiler owner. This review is read-only and did not rerun anything.

Reviewed candidate identities match the approved source floor:

- `run_observation_manager.go`: SHA-256 `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`.
- `run_observation_manager_test.go`: SHA-256 `b1b585320ef65ed1cb27a2b618bcca4a8b2e368c9646e96a6229e6855e91c72a`.
- Source approval/repair 3 review: SHA-256 `5133db7ed624733183e88b93bd5ae55a540a1ed754dd2b5e82c6d94cc8cb601b`.

The complete approved source design and scope are retained in the original assignment, revision 6 proposal and corrections, and root private-design decision. This result authorizes no implementation, green behavior, integration, or T09 settlement.

## Captured command and artifacts

Root reports the sole command was:

```text
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/<setup-fix1-cache> CARGO_BUILD_JOBS=1 go test -race -count=1 -parallel=1 -timeout=60s ./internal/server -run '^TestRunObservationManagerCanceledPartialPrepareJoinsOwnedReadBeforeRetry$'
```

The output is a test failure, exit code 1, with the package completing in 0.234s. The reported isolated test duration is 0.00s. The captured preflight reports `MemAvailableMiB 4151 SwapFreeMiB 9066`, exact candidate hashes above, and `resource and source floors PASS; root sole heavy owner`.

Root archived the three capture files in `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/`:

| Capture | SHA-256 |
|---|---|
| `manager-unit1-red01-preflight.raw` | `467a3d35cc174704ee7ee565dc35a375a876a3bbb3fd86373f1cca240fd5be53` |
| `manager-unit1-red01-go-output.raw` | `0938468dde42ee19796659f6dd54c37fc4e8fcd90f77c8fa03de3e47a9ebf6b4` |
| `manager-unit1-red01-exit.raw` | `cf205dbb8cea84897b488abcc281bf96698d5e94b1096b16657b4caba9082a22` |

I independently verified those three hashes match the corresponding `/tmp/sea-manager-unit1-red01-oct07.*.raw` capture copies. The capture contents show no compile error, timeout, panic, fixture deadlock, or cleanup error.

## Exact observed result and semantic classification

The output is:

```text
--- FAIL: TestRunObservationManagerCanceledPartialPrepareJoinsOwnedReadBeforeRetry (0.00s)
    run_observation_manager_test.go:1020: Prepare returned before the canceled partial-Prepare read: got semantic failure event={EventType: Cursor: Timestamp: Payload:{CaseID: ObservedAt: RunListState: ObservationState: ListedRunCount:<nil> SelectedRunCount:<nil> ValidatedRunCount:<nil> UnreadableRunCount:<nil> UnavailableRunCount:<nil> OmittedRunCount:<nil> HydrationReadBudget:{Limit:0 ReadsAttempted:0 Exhausted:false} Runs:[]}} lease=<nil> err=unavailable: run observation manager unit 1 is not wired (run_observation); expected successful attachment reaching the controlled read
FAIL
FAIL github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/server 0.234s
FAIL
```

This is the intended **semantic stub RED**: `prepare` returned its explicit typed-unavailable “unit 1 is not wired” result with zero DTO and nil lease before the controlled trace read could start. The helper at test line 1020 is designed to fail immediately on that semantic result rather than misreport a timer timeout. The source confirms `prepare` first validates caller identity and dependencies, runs injected authorization and the present-context guard, then returns the deliberate unwired error (`run_observation_manager.go:125-145`). The output's specific `unit 1 is not wired` error distinguishes it from those earlier guard/authorization failures.

This is not a setup, compile, timeout, or cleanup failure. The actual read-start/cancel/return/retry behavior was never entered because the stub deliberately does not call the list or trace ports. The bounded cleanup ran after the failure; there was no work-owned poller to cancel or join, and no cleanup error appears in the capture.

## Assertions reached and not reached

Reached: the test's fail-fast helper observed `prepare` returning before the controlled read and identified the deliberate typed-unavailable stub response, zero event, and nil lease. This supplies one narrow negative observation that the manager behavior remains unwired.

Not reached: controlled read start; caller cancellation during partial initialization; actual read return and worker retirement; failed-Prepare rollback join; retry admission after rollback; retry fake release/success; call-count assertions; shared-initializer surviving-ref behavior; cohort or poller capacity behavior; and any later Unit1 cases. No claim is made that any of those tests were run, failed, or passed. In particular, this is not “all Unit1 tests RED,” nor evidence for lifecycle correctness.

## Limits

This independent verdict accepts only that the one approved focused command produced a genuine semantic expected-RED against the deliberately unwired stub, with capture provenance and candidate source identity aligned. It does not approve an algorithm, implementation, source behavior beyond the stub classification, runtime GREEN, broader test suite, integration, or settlement. The operator/compiler owner must make any next runtime decision separately.
