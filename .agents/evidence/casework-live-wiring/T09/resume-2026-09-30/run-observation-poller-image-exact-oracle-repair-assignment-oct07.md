# Exact nested-image oracle fixture repair assignment

Date: 2026-10-07  
Status: immutable source-only assignment archived before the fixture edit.

## Complete original root instruction

> ROOT EXPLICIT FRESH FIXTURE REPAIR RELEASE: independentcritic confirmed final5f4efixture materialgap exacthelperembedding; finalimmutablerejectionforthcoming. Read full originalassignment5c8+release4877+source result4fdc+criticnewrejection oncevisible BEFOREpatch. Archive this ENTIREoriginalinstruction NEWimmutable assignment beforeedits. Edit ONLY run_observation_manager_retained_image_test.go: currentbranch independently build full exactorderedcanonicalexpectedbytes from fixed literalprefix schema/phase1/init1/current + marshalRunObservationRetainedImage(state) + closingbrace, assert entire output equal (ensurescurrentlast,no extra fields, exactrawhelperobject/ledger/metadata). Exactcap+1 independently build same canonicalcandidatewrapper from candidateinner, assertactualoraclelen=cap+1 and innerfits, retaintimestampdelta1 explanation; do notuseproductionencoder outputtoestablishcandidateoracle. Preserveall7toptests, oldassertions, frozen5 and source stub32f57byteidentical. Nativeapply_patch only; source reads first. No tests/compiler/formatter/scanner/Git/nestedagents. Resultnewimmutable exacthashes/fullmaterialdeviations. Differentrenderercritic must review originalinstructions+result beforeRED. WAITactualfinalrejectionartifactvisibility before sourceedit.

Root subsequently confirmed the exact rejection artifact is present and that
this fresh fixture-only release stands. Its path is
`run-observation-poller-image-testfirst-source-independent-review-oct07.md`.

## Full review and release identities

- Original fixture assignment: `run_observation_poller_image_testfirst_assignment-oct07.md`, SHA-256 `5c8b57bdf3b462fc366783bb304f1aff45af7c4f36a7e58dc3e7a55dada3a2c0`.
- Original two-file release: `run_observation_poller_image_testfirst_release_assignment-oct07.md`, SHA-256 `4877d97deee28b572b6df553a4e7ac47e51aec26216c22e54e998ee9de49b5e8`.
- Original source result: `run-observation-poller-image-testfirst-source-result-oct07.md`, SHA-256 `4fdcde6655b358da2f6308697fc378e9738092a418c447bc50c816c7d0c01947`.
- Final fixture rejection: `run-observation-poller-image-testfirst-source-independent-review-oct07.md`; exact hash to record in the result receipt after read.
- Frozen encoder seam: `apps/godspeed-casework-go/internal/server/run_observation_poller_image.go`, SHA-256 `32f57bd8aa6edb9c20b645e3138b4b9e9fc02076518a34587253a3af31af328e`; this file must remain byte-identical.
- Pre-repair fixture: `apps/godspeed-casework-go/internal/server/run_observation_manager_retained_image_test.go`, SHA-256 `5f4e1253fb3656df570a8f941cbca1686f03a6443b56e9821a7b08e9cae06e70`.

The review finding is that the current-branch test did not prove the wrapper's
`current` bytes are the complete canonical helper image. The repair must add
an independent expected-byte oracle, not use output from
`marshalRunObservationPollerImage` to construct the expectation.

## Exact edit and assertions

Only edit `run_observation_manager_retained_image_test.go`. Preserve the seven
top-level test names and all existing assertions. In the current-branch case,
call the unchanged `marshalRunObservationRetainedImage(state)`, then assemble
the full expected wrapper as literal ordered prefix
`{"schema_version":"run-observation-poller-v1","phase":1,"init_result":1,"current":`
+ helper bytes + `}`. Compare all encoder output bytes to this expected value.
This makes the helper object raw, complete, canonical, and the final wrapper
field, including its retained ledger and metadata.

For the exact-cap-plus-one case, preserve the existing timestamp extension
`2026-10-07T12:00:00.1Z` to `2026-10-07T12:00:00.12Z` and its measured inner
delta assertion. Build an independent candidate expected wrapper from the
same literal ordered prefix + candidate helper bytes + closing brace. Assert
candidate helper length remains below the helper cap and the independent
expected wrapper length is exactly `runObservationRetainedImageMaxBytes+1`.
Do not use the production encoder's candidate output to calculate or establish
this oracle. Keep the actual production call only for the expected nil/error
refusal assertion.

Preserve all seven top-level tests and prior checks, all five frozen manager,
manager-test, helper, base-helper-test, and policy-test files, and the throwing
encoder seam. Do not run tests, compiler, formatter, scanner, or Git. Use
native `apply_patch` only. Afterward write a new immutable result receipt with
the full source-result/release/rejection hashes, new exact hashes, frozen-file
hashes, material deviations, and explicit no-test/compiler statement. A
different renderer critic must review the original instructions plus that
result before any RED run.
