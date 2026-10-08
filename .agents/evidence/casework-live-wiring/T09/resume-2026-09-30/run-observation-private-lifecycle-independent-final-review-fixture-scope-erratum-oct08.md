# Additive scope erratum: separately authorized existing-fixture correction

Date: 2026-10-08

This erratum supplements, and does not overwrite, `run-observation-private-lifecycle-independent-final-review-oct08.md` (SHA-256 `03be81b6a55f7179e49bdb484dfdf790d00e5379d57809418bc8e6ccf0392200`). That review disclosed the later failure-fixture formatting correction but omitted a distinct, separately authorized change to three tests in `run_observation_manager_test.go`.

## Provenance and exact change

The governing fixture-correction assignment is `run-observation-existing-fixture-correction-assignment-oct08.md` (SHA-256 `75cd9bfd5f49ab11dce02f759543ea1e2c20d6f490ee3b10e983463ccbacd982`); its result is `run-observation-existing-fixture-correction-result-oct08.md` (SHA-256 `588375ea008f61ed1380513838c4004f880e13f8dfd177aebe029e5ff48d4f86`), and its independent review is `run-observation-existing-fixture-correction-independent-review-oct08.md` (SHA-256 `d703ba5a` as recorded in that review thread; see the full review artifact for its content). The exact preimage wrapper is `run-observation-existing-fixture-correction-preimage-oct08.json` (SHA-256 `6ade27afc2b7719955d72b9c6909d6b46f14369bb337e6079be17060e3eb7153`). The assignment authorized only three named tests after an exact native preimage of the prior `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` / 55,815-byte file. The resulting test file is `cf7188c23e5bad5f7b0f093561bf85e1e273b48c721a677084429e8038f0ba86` / 56,118 bytes.

The result records five unique function-local replacements confined to:

1. `TestRunObservationManagerGuardRefusesBeforeListButValidParentReachesRead`: the fixture now makes Store revision `cursor-old` disagree with the fake Relay cursor `cursor-current`; existing refusal and positive parent/read assertions remain.
2. `TestRunObservationManagerAuthorizationRefusalReturnsNoLeaseBeforeList`: the successful empty retry expects the four required perspective checks; pre-admission refusal, exact DTO, list/read counts, and successful drain assertions remain.
3. `TestRunObservationManagerCanceledPartialPrepareJoinsOwnedReadBeforeRetry`: the test waits for actual first and second trace callback starts before cancellation; exact two-each final counts, return/JOIN, retry, capacity, zero DTO, and nil-lease assertions remain.

These are fixture-oracle and ordering corrections, not production behavior edits. The separately authorized focused retry subsequently passed 53 top-level and 22 nested outcomes; its result and six raw capture comparisons are recorded in the existing retry02 evidence. The canonical and full-module gates followed. This erratum does not claim this critic authored the corrections or gates.

## Reconciliation and limits

The frozen final review's statement that the manager failure fixture changed only by a formatting correction is accurate for `run_observation_manager_failure_test.go`, but incomplete as a description of all fixture-source changes: `run_observation_manager_test.go` also received the three authorized changes above. The semantic review remains about the actual final manager/worker implementation and retains its bounded Prepare/shared-poller/Stop-and-drain scope; this source-provenance amendment does not enlarge that runtime verdict or establish Next/SSE/T09 completion.

No existing review, result, source, or captured output was overwritten to create this correction. The initial review is retained byte-for-byte; this is an additive immutable record.
