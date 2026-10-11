# Unit1 list-refusal fixture repair result

Date: 2026-10-07. Source-only fixture repair; no tests, compiler, formatter, scanner, typecheck, Git, or runtime command was run.

## Frozen identities

- `run_observation_manager.go` SHA-256 `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` (unchanged).
- `run_observation_manager_test.go` SHA-256 `87be7ae28010132c3e5ddbaa9ad0469ed5891061b88a005f86d8d70b977124c0` (edited fixture source).
- Exact pre-edit source/test JSON archives and byte/hash claims are recorded in the frozen assignment record `run_observation_manager_list_refusal_fixture_repair_record_oct07.md`; the local `.raw` add-file attempts are not exact and are not used as preimages.

## Fixture changes

Replaced the misleading list-error-as-failed-Prepare test with `TestRunObservationManagerAuthorizationRefusalReturnsNoLeaseBeforeList`. It now uses the real injected authorization check with a stale current-session claim, expects a typed refusal/zero DTO/nil lease, and asserts the list and perspective verifier were not called. It then corrects the current claim and verifies the next Prepare reaches the explicit unwired stub, still without calling `run.list`. The test name and assertions no longer claim an owned list reservation was rolled back on an auth failure. The separate partial-Prepare cancellation/retry test remains in place for that future lifecycle contract.

Added `TestRunObservationManagerRefusedListReturnsUnavailableDTOAndEmptyLease`. It specifies the revision 6 unavailable initial DTO, including nonnil empty `Runs`, absent optional counts, and no candidate trace reads, and requires a nonnil empty private lease. This assertion is an unrun future-contract test against the current deliberately unwired stub; no RED/GREEN result is claimed.

The exact currently declared private `runObservationLease` has only `manager` and `pollers`; no `Next` method or list-unavailable lease-state representation exists. The new test therefore does not invent or invoke `Next`; `Next() == ErrRunListUnavailable` remains a clearly documented future implementation obligation from revision 6, outside this fixture-only repair.

## Whole-fixture search and scope

Searched every `runObservationManagerListFunc` fixture and `RunListResult{}` error return in `run_observation_manager_test.go`. The only misuse was the replaced fixture. The canceled list path in `TestRunObservationManagerReservesPreparingCohortsBeforeList` returns `ctx.Err()` in response to its explicit cancellation channel and is not a run-list refusal. No production source, lifecycle declaration, selector, guard, hydration behavior, contract, dependency, or architecture changed.

The previous exact source and test identities are backed by root's lossless UTF-8 JSON preimages. My local `.raw` attempts have an extra terminal LF, are retained with their actual nonmatching hashes in the assignment record, and must not be described as byte-exact copies. All changes are TESTONLY. This result does not assert that the new tests compile or run.
