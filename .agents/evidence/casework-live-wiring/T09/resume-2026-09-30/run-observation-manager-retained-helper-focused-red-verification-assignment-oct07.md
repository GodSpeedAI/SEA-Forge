# Focused Unit 1 and retained-helper semantic RED verification assignment

Date: 2026-10-07. Source-only execution plan; written before any new runtime
command. **No compiler/runtime command may run until root explicitly releases
the compiler token.** A prior independent source review approved the manager
scaffold only for a separately authorized focused expected-RED case; the
retained-helper source review approved the fixtures only for focused semantic
RED. This assignment itself grants no runtime permission.

## Governing scope

The full Unit 1 original assignment is
`run-observation-manager-unit1-original-assignment-oct06.md`; it releases only
private, test-first prepare/admission/rollback scaffolding. Root's retained
publisher decomposition releases only the separate pure helper test-first
scaffold. The manager must remain unwired; the retained candidate builder must
remain the deliberate rejected stub. Neither command verifies an algorithm,
implementation, production integration, `Next`, SSE, lifecycle correctness,
or T09 settlement.

The source identities reviewed before this assignment are:

| File | SHA-256 |
|---|---|
| `internal/server/run_observation_manager.go` | `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` |
| `internal/server/run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `internal/server/run_observation_retained_version.go` | `d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331` |
| `internal/server/run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |

Source/test edits are out of scope. Recheck all four hashes immediately before
each attempt. If any differ, stop before running Go and report the mismatch.

## Exact sequential attempts

After explicit root release, perform these two attempts serially. Do not start
attempt 2 until attempt 1's command has joined and its three actual captures
have been archived and hash-checked.

1. Manager focused fixtures: stale-claim refusal with corrected-claim retry,
   refused-list unavailable DTO/empty lease, and canceled partial Prepare
   rollback/read join/retry:

   ```sh
   GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 go test -race -count=1 -parallel=1 -timeout=60s ./internal/server -run '^(TestRunObservationManagerAuthorizationRefusalReturnsNoLeaseBeforeList|TestRunObservationManagerRefusedListReturnsUnavailableDTOAndEmptyLease|TestRunObservationManagerCanceledPartialPrepareJoinsOwnedReadBeforeRetry)$'
   ```

2. Every retained-helper focused fixture (currently eight tests):

   ```sh
   GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 go test -race -count=1 -parallel=1 -timeout=60s ./internal/server -run '^TestRunObservationRetained'
   ```

Before each Go command, capture a fresh preflight containing UTC time,
`MemAvailableMiB`, `SwapFreeMiB`, repository root, the exact four source/test
hashes, and the complete command. Require at least 1200 MiB available memory
and 512 MiB free swap; if below either floor, do not run and report the
measured values. Use the exact command line above, preserving the environment
values and test selectors.

Immediately after each command completes and joins, preserve its complete
stdout/stderr and actual numeric exit status as separate files. Use these six
new archive names in the BASE evidence directory:

* `run-observation-focused-red-manager-preflight-oct07.raw`
* `run-observation-focused-red-manager-output-oct07.raw`
* `run-observation-focused-red-manager-exit-oct07.raw`
* `run-observation-focused-red-retained-preflight-oct07.raw`
* `run-observation-focused-red-retained-output-oct07.raw`
* `run-observation-focused-red-retained-exit-oct07.raw`

Do not overwrite prior evidence. Keep a `/tmp` capture copy if useful, then
compare exact archived bytes and record SHA-256 for every artifact before
returning the compiler token or starting the next attempt.

## Result classification

An expected semantic RED requires the package to compile and the selected
fixtures to fail at their intended semantic assertions against the deliberate
stubs. For the manager command, classify separately each test outcome:

* the stale-claim path must return its intended typed auth refusal/zero
  wrapper/nil lease before downstream work, then the corrected-claim retry
  must expose the current unwired stub failure rather than falsely passing;
* the refused-list test must expose the unwired stub rather than return the
  required unavailable DTO/empty lease;
* the canceled-partial-Prepare test must hit its semantic fail-fast before the
  controlled read rather than timeout or deadlock.

For the retained-helper command, the eight fixtures should fail semantically
because accepted/recovered candidates are unavailable from the explicit
stub. A compiler/type error, package setup failure, race detector report,
timeout, panic, deadlock, cleanup failure, resource failure, or output missing
the expected assertion is not an expected semantic RED. Record the actual
reason and do not describe such a result as stub RED. Do not infer coverage of
any unselected manager lifecycle tests or of behavior the stubs never enter.

## Evidence limits and provenance

The earlier Unit 1 RED01 was accepted only as one focused semantic failure of
`TestRunObservationManagerCanceledPartialPrepareJoinsOwnedReadBeforeRetry`;
its root acceptance and command-provenance correction explain its limited
reach. This assignment proposes additional isolated fixtures but does not
promote that historical result into evidence for them. The prior full manager
source critiques and retained-helper terminal fixture review approve only
source shape suitable for future focused RED, not runtime or implementation.

When runtime is released and complete, write a new immutable result record
with exact commands, captures, capture hashes and comparisons, all test names
and reached failure lines, precise semantic-versus-setup classification,
explicit unexecuted scope, and source hashes. A different verifier reviews
that evidence. Until then, no command, test, compile, RED/GREEN, or runtime
result is claimed.
