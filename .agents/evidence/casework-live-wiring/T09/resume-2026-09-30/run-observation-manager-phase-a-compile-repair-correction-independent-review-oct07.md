# Independent review: Phase A caller binding correction

**Verdict: SOURCE READY for the separately authorized focused RED retry only.** This is a source review, not a compiler or runtime result.

## Exact source comparison

The corrective assignment SHA-256 is `2583a9f9bc774186bc3c817fc96a721bfbfa5831a02bd8b33820292efac9bf63`; its result is `run-observation-manager-phase-a-compile-repair-correction-result-oct07.md`. The archived correction preimage JSON decodes to 27,703 bytes with SHA-256 `7d1470f4b930036561e0c0500d58fb41240994e764754977ce406367b38ba6b0`, matching the stated source preimage. The current `run_observation_manager_failure_test.go` is 27,703 bytes, SHA-256 `e63e050a7999b5c239257e57c6642baa346c59a40dfa1871e58d7b64ef80a5cf`.

Comparison of that preimage to the current file shows exactly the two authorized function-qualified changes:

- In `TestRunObservationManagerFailureStopJoinsHeldActualRead` (starts at line 253), `manager, caller, _, _` is restored. The held-read goroutine still passes `caller` to `manager.prepare` (line 273), so the needed local is defined.
- In `TestRunObservationManagerFailureGlobalStopReleasesDistinctPendingLeases` (starts at line 316), the unused fixture binding is now `manager, _, _, _`. This test creates its distinct callers in the `prepare(session)` closure, retaining the required identities without the unused local.

The exact earlier `4ff2fe1c922d6ae7e232d5a58f201298e232807437967fe22a9316dcb45aa0f3` source preimage independently confirms the net correction changes only the Global Stop binding. Thus the held-read binding has been restored and the original unused binding removed. All nine `TestRunObservationManagerFailure...` cases remain declared: stop-before-launch; detach-before-first-read; claimed-worker continuation; held actual read join; global stop with distinct pending leases; canceled Prepare with surviving lease; detach/selected-A single owner; shared first-read error A; and oversized nil-key rejection. No assertion or test body changed beyond the two bindings.

## Frozen identities and instruction differences

Read-only SHA-256 checks match the corrective result for the manager (`f9321a620ad64546e735b936d0b58b9f514eab0d7c6d0325f1e7a82f37c5e314`), worker (`f90cd397cd983e42e53498f13baa89fbfeac54e07eb3847b3228592ddd4d929f`), original manager fixture (`af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`), and six published primitives (`a6f0114d…`, `2157583f…`, `34df05f1…`, `e156c9cf…`, `3dba418f…`, `cf501bf7…`). Full hashes are in the builder result.

The correction follows the full assignment: it edits only the fixture file; restores the required held-read binding; removes the unused Global Stop binding; preserves the nine-case matrix, assertions, and caller identities; and changes no frozen file. It explicitly corrects the prior builder result’s false claim about which occurrence changed. No material deviation was found.

## Limit

No compiler, test, formatter, scanner, or Git mutation was run. This verdict establishes source-level readiness for the focused RED retry only. It does not claim compilation, expected assertion failures, lifecycle correctness, or runtime behavior. A separate root heavy-token grant is required before any execution.
