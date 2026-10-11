# Independent review: retained policy fixture

Date: 2026-10-07  
Disposition: **REJECT this fixture for focused semantic RED pending two test-helper repairs**

## Scope and identities

Source-only review of the full original policy-fixture assignment, original helper assignment, algorithm preregistration, root algorithm decisions, prior independent algorithm review, the complete new policy test, and both frozen helper source/test files. No compiler, test, scanner, formatter, typecheck, runtime, Git, or source edit was performed.

| Artifact | SHA-256 |
|---|---|
| Policy-fixture assignment | `ed290103a075fae97b438d3ed755ac4ef6e3553ed8712a973efbb5f607504a83` |
| New policy fixture under review | `d1b919e4e432afc42940fb6b4da632c8d631a9ff016ed8e0e2eea0075523c8f0` |
| Retained helper, unchanged | `d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331` |
| Original focused fixture, unchanged | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| Original helper assignment | `cc1fed59eb1f440e53756d3f2adee79369d0c57f17f800481b9c92a1093c446a` |
| Algorithm preregistration | `b8f0a5693cc69423df1a85a9952a941b6e88ae910e0efcb50e7034b3b9b7bdb4` |
| Root algorithm decisions | `9dcf3886f98d4416fd1bbcded12ecc939a9f5281b9bebbe3e929dd865185aa3a` |
| Prior independent review containing the four findings | `0e42531893a16ef91be307deb1d4747ce98e2e45f5749d4a636adfc8a1605c44` |

Only the new file was requested for this repair. The helper and original fixture hashes match the frozen identities. No actual RED is established here; the helper remains an always-rejected stub at `run_observation_retained_version.go:67-80`.

## Four prior findings: addressed in source

The new tests materially address all four gaps in the prior review:

1. `TestRunObservationRetainedPolicyOverflowMarkersAreExactAndAtomic` now pins nonterminal ordinal overflow to retention-unavailable, terminal ordinal overflow to terminal-retention-failure, and generation overflow to stop-scheduling (`run_observation_retained_policy_test.go:58-115`).
2. `TestRunObservationRetainedPolicyTerminalityUsesExecutionOnly` exercises each terminal execution value with unsettled settlement, plus active execution with accepted and rejected settlements (`:117-151`). Values are valid: the contract enumerates these execution/settlement standings at `internal/contract/contract.go:434-436`; helper-built command frame status is `completed`, also allowed at `:426-432`. Candidate size comes from a long opaque event ID, not invalid enum or timestamp data (`:132-137`).
3. `TestRunObservationRetainedPolicyFinalMarkersRefuseLaterCandidates` tries a later fitting candidate against both terminal-retention-failure and stop-scheduling markers and checks refusal/marker preservation (`:153-173`).
4. `TestRunObservationRetainedPolicyPreservesResponseCounts` verifies total 5 with a one-frame window gives retained 1, omitted 4 and truncated true, then accepts a later total of 2 and verifies counts again (`:175-211`). This separates source response count from ledger size and exercises a decreasing total, consistent with `ports.RunTraceSnapshot` (`internal/ports/run_trace.go:10-20`).

The new extra policy cases are also aligned with the root's authorization: read-unavailable recovery; requested/snapshot and prior/requested key mismatches; negative and below-window totals (`:213-266`). They use valid test identities and return invalid/nil for those malformed counts/keys. No external-interface or lifecycle test is introduced. The fixture uses all new test names with the required `TestRunObservationRetainedPolicy` prefix.

## Blocking fixture defects

### 1. `requireRetainedPolicySafeCopy` captures the caller state after the operation

The helper first runs `priorBefore := cloneRetainedTestState(prior)` at line 18. Every call site passes `prior` only after `buildRunObservationRetainedCandidate` returned; therefore this clone cannot prove the input was unchanged during candidate construction. The ordinal-overflow cases (`:77-88`), terminality table (`:131-141`), and final-marker cases (`:157-170`) have no separate pre-call clone. A builder that mutates the caller's map/slice/pointers and returns a matching copy can satisfy the helper's equality check because its `want` is based on already-mutated state. This contradicts the assignment's required atomic prior-state proof and root decision that refusals never mutate prior state (`retained-publisher-algorithm-root-decisions-oct07.md:5`).

Minimal repair: capture `priorBefore` and its canonical image in each test before invoking the candidate builder, then pass those expected values to the assertion helper (or make a helper that receives a pre-call snapshot). Keep the pointer/map mutation probe to establish return-copy independence, but compare the original caller state against the pre-call clone, not a post-call clone.

### 2. The safe-copy helper mutates the result before the generation test's length assertion

`requireRetainedPolicySafeCopy` intentionally changes returned frame pointers to `"result-only"` and `81`, adds a `"result-only"` ledger entry, then returns (`:50-55`). The generation-overflow caller invokes it at line 107 and only afterward marshals the changed result at lines 108-114 and requires its length to equal the pre-call `priorImage`. A correct algorithm returns the prior safe image with only the stop-scheduling marker, whose fixed-width representation is the same length as the prior. The helper then lengthens the optional status from `completed` to `result-only`, changes the one-digit exit code to two digits, and adds a ledger row, so the subsequent image length is larger. Thus a correct implementation is forced to fail this assertion because the test mutated its result first.

Minimal repair: do all value/image/length assertions before invoking a mutation-based alias probe, or make the reusable safe-copy assertion non-mutating and move alias mutation into a separate test after all image checks. Do not weaken the fixed-length marker assertion.

The generation case's separate pre-call `before` clone correctly checks the input caller was not mutated (`:92-107`), but its later image-length assertion remains invalid for the reason above. For ordinal and final-marker cases, the first defect also leaves the caller-input atomicity assertion incomplete.

## Re-review requirements and boundaries

After both repairs, re-review the complete new fixture and verify the pre-call snapshot is truly taken before every builder invocation whose caller-owned prior is under test. Ensure the helper's deliberate deep-copy probe cannot affect later assertions. The intended exact marker, terminal-classification, later-candidate refusal, source-count, key, total, and recovery assertions should remain intact.

The design itself remains the root-ratified bounded private helper from the preregistration and root decisions: 1 MiB canonical retained-image budget, 1,024-frame window, exact opaque ID lifetime ledger, source-order ordinals, execution-only terminality, fixed-width marker behavior, and immutable candidate replacement. This review rejects only this new fixture's readiness to prove those semantics. It approves no helper algorithm, actual RED, lifecycle, manager, `Next`, integration, or runtime behavior. A separate current-candidate focused RED remains held pending source repair/re-review and root release.

No test or compiler claim is made. Graft retrieval saved ~64,411 tokens (~$0.05) this turn.
