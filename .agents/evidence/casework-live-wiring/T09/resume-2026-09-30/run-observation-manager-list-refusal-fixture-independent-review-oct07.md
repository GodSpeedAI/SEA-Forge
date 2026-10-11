# Independent review: Unit 1 list-refusal fixture repair

Date: 2026-10-07  
Disposition: **REJECT full fixture approval pending a future-success retry expectation**

## Scope and source identity

Reviewed the complete Unit 1 original assignment, root private-design decision and review, revision 6 proposal and addenda/corrections/response-line-cap erratum, qualified Unit 1 RED verdict, repair 3 cleanup review, the full candidate manager source/test, list-refusal fixture assignment/result, and the contract DTO declarations.

Current hashes:

| Artifact | SHA-256 |
|---|---|
| `run_observation_manager.go` | `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` |
| `run_observation_manager_test.go` | `87be7ae28010132c3e5ddbaa9ad0469ed5891061b88a005f86d8d70b977124c0` |

I independently decoded the exact preimage JSON archives: manager source 5,488 bytes/SHA `fa1601...a905d`; test 51,535 bytes/SHA `b1b585...1c72a`. The current production source is unchanged. The unified test diff contains only the added `strings` import and edits/addition around the former failed-Prepare list-error test. Historical package `.raw` preimage attempts include an extra LF and are correctly not represented as exact copies.

## Revision 6 distinction and correct portions

Revision 6 requires a refused/over-cap/undecodable `run.list` to return an unavailable initial DTO and completed empty lease: unavailable list and observation states, nonnil empty `Runs`, absent optional count pointers, no candidate trace reads, and `ErrRunListUnavailable` from the later `Next`. Auth/guard/context/stop/irreducible-assembly failures instead return a typed error, zero wrapper and nil lease, with owned rollback as applicable (`run-observation-manager-concrete-proposal-revision6-oct06.md:90-98,451-459`). The current manager source remains the explicit unwired stub and declares no `Next`; the repair correctly refrains from inventing one and marks the later `Next` contract as an implementation obligation.

The new `TestRunObservationManagerRefusedListReturnsUnavailableDTOAndEmptyLease` (`run_observation_manager_test.go:986-1024`) is aligned with the current declarations and revision 6: it injects a typed unavailable list refusal, expects a nonnil empty lease, uses `reflect.DeepEqual` to check the exact event/DTO including unavailable states, timestamp/current cursor, `Limit: 8`, and nonnil empty `Runs`, separately verifies all six optional count pointers are nil, and asserts one list call and zero trace calls. Not asserting `Next` is a candid Unit 1 boundary, not a defect in this fixture-only change.

The replacement auth-refusal test correctly uses a stale current-session claim to produce a real authorization rejection before list/admission, with zero DTO/nil lease and zero list/verifier calls (`:950-959`). The independent cancellation/partial-prepare fixture remains the appropriate owned-rollback/retry coverage. The list fake inventory found no additional misuse: the sole other error return from a list fake is `ctx.Err()` for the explicitly canceled blocked-list path (`:476-479`), not a run-list refusal.

## Blocking finding: authorized retry asserts placeholder behavior

After the auth refusal, `TestRunObservationManagerAuthorizationRefusalReturnsNoLeaseBeforeList` installs the caller's correct current claim, then asserts the subsequent `prepare` must again return `KindUnavailable`, nil lease, zero DTO, and an error containing the literal `"unit 1 is not wired"`; it further requires `listCalls == 0` (`:961-975`). That assertion encodes the current stub implementation as the expected future behavior. When `prepare` is correctly implemented, the corrected caller must pass authorization and guard, call the valid empty list fake, and receive the ordinary successful `no_runs` initial DTO plus nonnil empty lease; the list should have been called once. The current assertion would reject that implementation and prevents this fixture from serving as future-compatible test-first coverage.

Repair minimally: preserve the stale-claim zero-wrapper/nil-lease/no-list/no-verifier assertions. On corrected-claim retry, expect success with `ObservationState == "no_runs"`, an exact successful empty-list DTO (including present zero count pointers and `ReadsAttempted: 0`), a nonnil empty lease, one perspective verification, and exactly one list call with no trace reads. Allow cleanup/stop to own the empty lease or explicitly detach it. Remove the `strings` import if it is no longer used. The unwired stub will then fail at the expected semantic positive assertion, rather than the test blessing its placeholder result. This does not claim that retry or any other behavior was run.

## Disposition and limits

The new list-refusal DTO test is source-level consistent with revision 6 and the declared Unit 1 seam. The stale-auth branch is also correctly classified. I nevertheless reject the **full candidate** because the corrected-identity retry hardcodes an unwired implementation detail and would fail under correct future behavior. Have a fresh bounded source-only correction update that expectation and record new hashes; do not edit this review. No tests, compiler, scanner, formatter, typecheck, Git, or runtime command was run. No RED/GREEN, lifecycle, algorithm, integration, or T09 completion claim follows.

Graft retrieval saved approximately 18,997 tokens (~$0.02) this turn.
