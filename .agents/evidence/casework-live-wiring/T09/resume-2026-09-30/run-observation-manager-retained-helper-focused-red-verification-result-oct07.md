# Unit 1 manager and retained-helper focused RED verification result

Date: 2026-10-07. Executed only after root's explicit sole-compiler-token
release. The two assigned commands ran serially. This is bounded test-first
negative evidence, not implementation approval.

## Commands and source identities

The verification assignment is
`run-observation-manager-retained-helper-focused-red-verification-assignment-oct07.md`
(SHA-256 `45682c73c2db251a860aa616a3783b0388b2f074a1b62c3371644681e172e53e`).
Both commands used the listed resource controls, `GOLDEN_UPDATE=0`, the same
explicit GOCACHE, `-race -count=1 -parallel=1 -timeout=60s`, and ran from
`/home/sprime01/projects/sea-rs/apps/godspeed-casework-go`. No files were
modified between attempts.

| Source/test | SHA-256 before and after both attempts |
|---|---|
| `internal/server/run_observation_manager.go` | `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` |
| `internal/server/run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `internal/server/run_observation_retained_version.go` | `d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331` |
| `internal/server/run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |

## Attempt 1: manager focused fixtures

The exact selector was:

```text
^(TestRunObservationManagerAuthorizationRefusalReturnsNoLeaseBeforeList|TestRunObservationManagerRefusedListReturnsUnavailableDTOAndEmptyLease|TestRunObservationManagerCanceledPartialPrepareJoinsOwnedReadBeforeRetry)$
```

Fresh preflight at `2026-10-07T12:44:06.891550+00:00` reported repository
`/home/sprime01/projects/sea-rs`, `MemAvailableMiB=4396`, and
`SwapFreeMiB=12080`; all four hashes matched. All exceeded the assignment
resource floors. The exact command is embedded in the preflight capture.

Exit status: **1**. The package compiled and the three selected tests each
failed on the deliberate unwired manager stub:

* `TestRunObservationManagerAuthorizationRefusalReturnsNoLeaseBeforeList`
  passed its stale-claim refusal/no-downstream assertions, then failed at
  `run_observation_manager_test.go:971`: after the corrected claim, `prepare`
  returned typed `run observation manager unit 1 is not wired` instead of the
  successful no-runs event/empty lease.
* `TestRunObservationManagerRefusedListReturnsUnavailableDTOAndEmptyLease`
  failed at `:1016`: the unwired stub returned a zero event, nil lease, and
  typed unavailable error before the list-refusal DTO/empty-lease behavior.
* `TestRunObservationManagerCanceledPartialPrepareJoinsOwnedReadBeforeRetry`
  failed at `:1099`: the fail-fast helper observed that same semantic stub
  response before the controlled read could start.

These are semantic stub failures, not compiler/setup failures. The actual list,
trace read, cancellation, rollback, worker join, and retry lifecycle were not
entered or verified by this stub.

| Capture | SHA-256 |
|---|---|
| `run-observation-focused-red-manager-preflight-oct07.raw` | `c213482c8e0e18c24401253de9ff909eb75c68a4cd55dc82cf7f9761b8e3ed7d` |
| `run-observation-focused-red-manager-output-oct07.raw` | `ff3700ad56b29591174dbda13eac97325430e155087fb20cc4d70b8610c6e9aa` |
| `run-observation-focused-red-manager-exit-oct07.raw` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` |

## Attempt 2: all retained-helper focused fixtures

This command started only after attempt 1 joined and its preflight/output/exit
captures had been archived and hashed. The selector was
`^TestRunObservationRetained`, covering all eight retained-helper tests.

Fresh preflight at `2026-10-07T12:46:06.942834+00:00` reported repository
`/home/sprime01/projects/sea-rs`, `MemAvailableMiB=4212`, and
`SwapFreeMiB=12080`; all four hashes matched, above the required floors. The
exact command is embedded in the preflight capture.

Exit status: **1**. The package compiled and each of the eight selected tests
failed at its first expected semantic assertion because the explicit helper
stub returns generic `retainedCandidateRejected` (numeric outcome 1) and nil
state:

* `TestRunObservationRetainedCandidateCopiesPointersAndCapturesWindow` —
  `run_observation_retained_version_test.go:95`, wanted accepted candidate.
* `TestRunObservationRetainedCandidateClonesPriorAndSourcePointers` — `:117`,
  wanted initial accepted candidate.
* `TestRunObservationRetainedCandidateKeepsFirstOrdinalsAcrossReorderAndShorterWindow`
  — `:148`, wanted initial accepted candidate.
* `TestRunObservationRetainedCandidateRejectsDuplicateIDsInOneWindow` — `:231`,
  received generic rejected outcome instead of invalid input.
* `TestRunObservationRetainedCandidateRejectsBudgetAtomicallyAndRecovers` —
  `:254`, received generic rejected outcome/nil state instead of typed
  retention-unavailable state.
* `TestRunObservationRetainedCandidateTerminalRefusalKeepsOnlySafePriorState`
  — `:329`, received generic rejected outcome/nil state instead of typed
  terminal-retention state.
* `TestRunObservationRetainedImageFixedWidthControlsAndExactLimit` — `:435`,
  exact-boundary candidate was generically rejected.
* `TestRunObservationRetainedCandidateBoundsWindowAndOrdinalOverflow` — `:468`,
  exact 1,024-frame candidate was generically rejected.

This establishes only semantic expected-RED against the deliberate stub. It
does not verify any retained-candidate algorithm or the later assertions in
these tests. No compiler error, timeout, panic, race report, or cleanup failure
appeared in either command output.

| Capture | SHA-256 |
|---|---|
| `run-observation-focused-red-retained-preflight-oct07.raw` | `d445294b4460f0e28295fe0e87c3565bcb9e948d5b67d12fb7cc5048896afb1c` |
| `run-observation-focused-red-retained-output-oct07.raw` | `51f72fa297984754c42f3e2e1e4e5148cbe7490e4e374b189f6489016567af40` |
| `run-observation-focused-red-retained-exit-oct07.raw` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` |

## Evidence limits

The two package commands prove that the reviewed candidates compiled under the
specified race-enabled focused invocations and that the selected fixtures
reached their expected semantic failures against unwired stubs. They do not
establish manager or retained-helper behavior, green tests, lifecycle
correctness, algorithm correctness, production integration, or T09
settlement. All other manager tests and all package tests outside the two
selectors remain unexecuted. The six capture files are immutable new evidence;
prior RED records remain unchanged.
