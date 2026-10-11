# Independent review of Phase A one-binding compile repair — 2026-10-07

**Verdict: REJECT.** The binding change was applied to the wrong test
function. The actual fixture retains the unused `caller` in the Global Stop
test and removes the caller required by the held-read test. The expected RED
precondition remains unmet.

## Evidence and exact identities

I read the full one-binding repair assignment
`run-observation-manager-phase-a-compile-repair-assignment-oct07.md` (SHA-256
`b5689c684013ab24a76289383f9a008a6efc364c08ef1f2db4cce12cf03da0e0`), repair
result
`run-observation-manager-phase-a-compile-repair-result-oct07.md` (SHA-256
`7cb1bf55dc482eed7d9351be6bf9928ec23cfaa19b11bff51dc2e5b02c2a9ae1`), prior
focused compiler failure result (SHA-256
`54b5ffb7bfdc6eca1fc751f904bfc07837a885994c9ce4da38567ba6f4e62fa0`), and
prior Phase A fixture source approval and erratum. The repair's source preimage
is `run-observation-manager-failure-test-preimage-compile-repair-oct07.json`.

Using its `source` JSON string without adding a newline, I verified that the
decoded preimage hashes to
`4ff2fe1c922d6ae7e232d5a58f201298e232807437967fe22a9316dcb45aa0f3`. The
actual current fixture hashes to
`7d1470f4b930036561e0c0500d58fb41240994e764754977ce406367b38ba6b0`. The
preimage diff contains exactly one binding hunk, but it is at the wrong site:

```diff
- manager, caller, _, _ := newRunObservationFailureFixture(...) // held-read test
+ manager, _, _, _ := newRunObservationFailureFixture(...)      // held-read test
```

## Material defect

In `TestRunObservationManagerFailureStopJoinsHeldActualRead`, the current
binding at `run_observation_manager_failure_test.go:261` is
`manager, _, _, _`, but the goroutine still passes `caller` to `prepare` at
line 273. This reintroduces the original undefined identifier and invalidates
the prior source approval for the repair target.

In `TestRunObservationManagerFailureGlobalStopReleasesDistinctPendingLeases`,
the binding at line 323 remains `manager, caller, _, _`. The local `caller` is
unused; the two Prepare goroutines instead construct their distinct identities
inside the `prepare(session)` closure at lines 331–335 using
`runObservationManagerCaller(session)`. Thus the original compile failure at
line 323 remains.

The repair result claims the change was made in the Global Stop test and
describes removal of its unused caller. That claim does not match the actual
preimage-to-current diff or source. The single change instead removed the
required held-read caller. Both compile defects therefore remain: an undefined
identifier in the held-read test and an unused local in the Global Stop test.
The expected focused assertion RED is not ready to run.

## Limits

No compiler, test, formatter, scanner, source edit, or Git mutation was run.
This review does not infer compiler output from source; it records that the
previous observed compiler failure remains and identifies the current
source-level defects. A fresh bounded builder must move the binding change to
the authorized Global Stop occurrence, preserve the held-read binding, publish
new exact identities/result, and receive a new independent review before any
compile grant.
