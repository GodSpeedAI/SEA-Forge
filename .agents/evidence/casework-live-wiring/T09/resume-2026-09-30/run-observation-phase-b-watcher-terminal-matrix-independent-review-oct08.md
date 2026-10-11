# Independent DOCONLY watcher and terminal matrix supplement review

Date: 2026-10-08

## Review assignment and limits

Read the complete original watcher/terminal architecture assignment, the
complete matrix-supplement assignment, the actual architecture proposal and
additive supplement, the prior independent source rejection and matrix
rejection, the root terminal/surviving-watcher clarification, and the governing
revision 6 clauses. Decide whether the supplement resolves both prior findings
with feasible tests at actual production boundaries and preserves every
existing constraint. Verify the frozen implementation/fixture identities.
Write a proposal-only verdict. No source edits, tests, compiler, formatter,
scanner, build, Graft build, status/debt edit, Git operation, or runtime claim is
authorized.

## Reviewed evidence

- Original architecture-repair assignment:
  `run-observation-phase-b-watcher-terminal-repair-assignment-oct08.md`,
  SHA-256 `7ffdd48ccbfd69d4ff0e8bf5d7812405b7385651c5b5396e8b4c30b4e8deb852`.
- Architecture proposal:
  `run-observation-phase-b-watcher-terminal-repair-proposal-oct08.md`,
  SHA-256 `de8c1647153e06a5fc492671d14c36d3e562f1cd1f079086b62546304e90f0ad`.
- Root clarification:
  `run-observation-phase-b-watcher-terminal-root-clarification-oct08.md`,
  SHA-256 `92c0a5bcbdc309c1f0e09c3ac4bf61dc350aee685985a5a824ee2dff72cabba5`.
- Prior matrix rejection:
  `run-observation-phase-b-watcher-terminal-independent-review-oct08.md`,
  SHA-256 `7108ed75a5e86ffcda16d356d6f2e3da46269f756d65019636ce788cd19aaa77`.
- Complete new supplement assignment:
  `run-observation-phase-b-watcher-terminal-matrix-supplement-assignment-oct08.md`,
  SHA-256 `f31d15ef7f611322cc4941f4b1bb6a29660edb5b8ea0845c04345cf4c1bfedf5`.
- Actual additive supplement:
  `run-observation-phase-b-watcher-terminal-matrix-supplement-oct08.md`,
  SHA-256 `fe1e9e0502bf861b4153887734007e6db1486aea9a731d308e7abc58d0e18116`.
- Original algorithm assignment and source review remain
  `run-observation-phase-b-algorithm-source-assignment-oct08.md`
  (SHA-256 `e5fc9e0aa9a8549aa75bcf78040b7012e79e8a882a176965dc80ce6eace8ef42`)
  and `run-observation-phase-b-algorithm-independent-review-oct08.md`
  (SHA-256 `ecf2e06f34a57257cc99480dc17a41b40cdaff0a34248ceeb7c940c91473b644`).
  Governing revision 6 clauses are in
  `run-observation-manager-concrete-proposal-revision6-oct06.md`.

## Frozen identity verification

I re-hashed the two implementation files and all eight frozen fixture/helper
files. The values match the prior source review and supplement's stated frozen
preimage identities:

| File | SHA-256 |
|---|---|
| `run_observation_manager.go` | `d7be1d7a724ed65575ccaaf35cdf1aa5af133dbdd4952d592699b7eb443b6350` |
| `run_observation_poller_worker.go` | `84382ffd0cd77e1c85bd11b607c1427075c4899e1c534fbe7368b0af5ad7c9cc` |
| `run_observation_manager_failure_test.go` | `ef587b0bd64fe42d5fb56369f620f969610a5bac3d04cd7578911472b1a4893c` |
| `run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

This is an identity check only. No code or test command was run.

## Verdict

**APPROVE the DOCONLY proposal for bounded TDD source preparation.** The
supplement resolves the prior early-handoff coverage gap with four deterministic
subcases and strengthens the fitting-terminal case to cover owner/shared read
counts, worker completion, cache reuse, and read-claim refusal. This approval
is limited to preparing and separately reviewing the proposed TDD changes. It
does not approve those test edits, production source changes, test execution,
GREEN, runtime correctness, lifecycle completion, or a public release.

## Evidence and compatibility

1. **The added handoff matrix reaches the missing real branch boundaries.**
   The supplement holds the actual list callback after signaling entry, changes
   the test-owned current-session or guard dependency, and then releases the
   callback. Its four cells cross invalid authorization versus changed cursor
   with empty versus unavailable/error list. These cover the two previously
   missing early-handoff paths without writing manager/lease/poller state or
   introducing a production hook. The expected typed error, zero event and nil
   lease are appropriate fail-closed outcomes. Read-only manager-lock
   inspection after Prepare returns is compatible with exact cleanup ownership;
   the test does not replace or perform cleanup itself.

2. **It accurately allows a subcase to pass on the old implementation.**
   The supplement predicts cursor-invalid plus empty list may fail already:
   `prepare` guards after successful list return (`run_observation_manager.go:405-407`).
   Conversely, the list-error branch constructs and returns an unavailable DTO
   at `:386-403`, before that later guard, and the current implementation has
   no final reauthorization at `:549-553`. The supplement explicitly requires
   actual per-cell results and does not require every subcase to be RED. This
   avoids manufacturing failure and keeps the test useful as a regression
   check.

3. **Terminal assertions now prove the shared/cache contract and no new read
   claim.** The strengthened case uses an owner and a shared waiter while a
   real initializer read is held. It checks owner count one, shared count zero,
   both final DTOs and leases, real `workerDone`, stable actual-call count, and
   refusal of `claimPollerReadStart` after completion. These assertions match
   the root clarification: publish valid accepted terminal current and
   readiness, let the worker return without another read, retain the valid
   current state for Prepare/cache reuse, do not wait for DTO/creator completion
   in the worker, and do not drain valid leases merely because terminal state
   fit. Capacity remains subject to actual Detach/Stop completion.

4. **Shared authorization and the original private architecture remain intact.**
   The supplement explicitly preserves all six prior scenarios and all twelve
   frozen fixture assertions. It does not change the two required private
   values (`caller runObservationCaller`, `asOfCursor string`), required
   `beginPrepareOperation` arguments, four real-identity fixture setup edits,
   existing current-session/perspective rules, guard/cursor/parent checks,
   independent invalid-watcher drain, or authorized shared-survivor behavior.
   It adds no public policy, API, schema, result code, production hook, new
   field, mode, counter, interface, or cancellation spy. The six-case proposal
   and four matrix cells remain a narrowly scoped test-first task.

5. **Material differences from the supplement assignment:** none found in
   scope. The assignment requested exactly the four dependency/list-result
   cells and terminal owner/shared/cache assertions; the supplement contains
   them and states that outcomes must be reported individually. Its identity
   references to the original failure and manager fixtures are abbreviated
   hashes, but the full exact hashes are preserved above and in the prior
   review. No omitted requirement alters the DOCONLY verdict.

## Limits

This is not evidence that the proposed tests compile, run, or fail for the
intended reason. It does not establish that an implementation will honor the
matrix, that context/current/guard races are resolved at runtime, or that the
worker's stop/join behavior is correct. A new source-preparation assignment,
source review, and separately authorized actual RED gate remain necessary.
