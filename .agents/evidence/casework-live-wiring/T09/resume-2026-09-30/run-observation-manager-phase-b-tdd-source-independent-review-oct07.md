# Independent review: Phase B TDD source preparation — 2026-10-07

**Verdict: REJECT source readiness.** The source diff stays within the authorized two files and the new tests substantially match the released matrix, but the manager signature and three map-membership assertions contain static Go type/syntax errors. No compiler or test was run; this is a source-level finding only.

## Reproducible source evidence

The full original release SHA-256 is `34224d769f5b802cb7cc31b005ae330f671598590e809aede649cd98dc0a89a7`; its exact scope is archived in the assignment file listed below. The preimage JSONs decode with `jq -rj` to the declared exact byte counts and original hashes: manager 6,924 bytes / `f9321a620ad64546e735b936d0b58b9f514eab0d7c6d0325f1e7a82f37c5e314`, fixture 27,703 bytes / `e63e050a7999b5c239257e57c6642baa346c59a40dfa1871e58d7b64ef80a5cf`. Direct unified diffs from the decoded preimages show only the authorized manager lease fields/signatures and the four finish closures, four reservation arguments, one list-entry helper, and three appended tests.

## Concrete blockers

1. In `run_observation_manager.go:123-127`, `reservePollerBatch` mixes unnamed first and second parameters with the named third parameter `prepareDone`. Go requires parameter names to be consistently present or omitted for a signature; this declaration is not valid Go. Name the first parameters (or omit the name on `prepareDone`) in a fresh builder repair while preserving the required third channel argument and deliberate stub body.

2. In `run_observation_manager_failure_test.go:747-750`, `manager.cohorts` is declared `map[*runObservationLease]struct{}` at manager line 51. `registeredLease := manager.cohorts[lease]` therefore yields `struct{}`, but line 749 uses it with `!` as a boolean. Use the map's comma-ok result or an explicit membership comparison.

3. In `run_observation_manager_failure_test.go:1056-1060`, `stillRegistered := manager.cohorts[lease]` again yields `struct{}` and is used as a boolean with `!` at line 1059.

4. In `run_observation_manager_failure_test.go:1082-1086`, `remainingCohort` is likewise the `struct{}` map value, but line 1085 uses it as a boolean in `if remainingCohort || ...`.

These are actual source incompatibilities visible from the declaration and the map value type, not inferred test failures. The result records no compiler invocation, and this review makes no compiler-exit claim.

## Remaining source review findings

The intended source delta is otherwise tightly scoped: the lease adds exactly `creatorDone` and `leaseDone`; `finishPrepareOperation` takes the exact lease; `reservePollerBatch` adds required `prepareDone`; both bodies remain the deliberate unwired stubs. The fixture changes only the four authorized wrappers/call sites and adds the three named held-list tests. All nine original test functions remain. The new tests use actual `prepare` list callbacks, bounded channels, existing `apperr.KindUnavailable`, read-only manager mutex inspections, and cleanup release functions; no callback hook, cancel spy, private-state assignment, or sleep was added. The successful B path waits for its exact `creatorDone`; test code does not claim to observe the bridge goroutine directly.

The fixture still uses the `map[*runObservationLease]struct{}` registry correctly as a set for read-only iteration; the three invalid expressions above are the only direct map-value boolean uses found by exhaustive search in this file. Four exact call-shape changes and the test additions are evidenced by the preimage diff. Result says 12 total cases (nine retained plus three new) and eight protected source hashes; current hashes match that record. Worker, original manager fixture, and six primitives remain unchanged.

## Scope and next step

This is a fresh-builder repair loop for the invalid signature and map-membership checks. Do not treat the unwired stubs as lifecycle behavior. No formatter, compiler, test, scanner, Git, or Graft build was run. TDD source readiness remains unapproved until a corrected frozen result receives another independent source review. Root must separately grant any expected-RED execution.
