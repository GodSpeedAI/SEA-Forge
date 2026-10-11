# Retained-version helper atomicity fixture repair — original assignment

Date: 2026-10-07. Immutable original assignment record, written before editing
the test source. Scope is test-only and source-only; no compile, test, scanner,
typecheck, or Git command.

## Full fresh root assignment

> Fresh DIFFERENT bounded retained-helper TESTONLY repair. Sourceonly no
> compile/tests/scanner/Git; UIverifierownscompiler. Read full ORIGINAL
> helperinstructions package
> `run_observation_retained_version_helper_assignment_oct07.md` +
> source_receipt/hashcorrection, source/go test; BASE rootretaineddecomposition
> and REJECT
> `run-observation-retained-version-helper-independent-source-review-oct07.md`
> SHA `3123e59424ad5cc46fb2b5a41d3cb4d115d4027d4ade8cf3e99a05332c1ff889`.
> Repair allfindings with minimalfocusedassertions/tests; helperproduction
> `d8df...` unchangedexplicitbuilderstub+encoder. Immediateexactpreimage test
> `5365c1ff...` via nativeJSON BEFOREedit and decodedhash equality,
> originalNEWfullassignmentrecord BEFOREediting; ifnativecopy needsroot ask.
> Requiredcoverage: fullpriormarshal byte equality before/afterreject;
> exact output state metadata/window/ledger equality exceptfixedavailability
> marker andsamecanonicalimagebyteLENGTH; full A/B/new ledger equal recovery;
> terminalno candidateframes/data retained andallpriorsafevaluesunchanged
> exceptmarker; acceptedexactboundarymarshal len==1MiB and
> rejectedcandidatecomputedimage 1MiB+1; repeat/disappear/reappear
> exactordinals andnevermutate prior; duplicatesONEwindow INVALID consistent
> traceport143-146 (no winnerpolicy); generationoverflow andordinaloverflow
> atomic no wrap; pointers noaliasincludingprior/state clones;
> fixedwidthmarkers atcountermax non-growing. No helperalgorithm/Next/
> lifecycleintegration, no code sourcechangejusttoaccepttest, nodeps/
> publiccontracts. Code/testscopedstructuresprivate. Ifsourcedeclaration
> changes strictlyrequiredforobservableassertions reportrootbeforeediting.
> Fresh differentguardcritic afterward originalfullinstructions+result;
> explicit deviations/no runtimeclaims, immutable records. Rootretainsemantics.

## Root semantic clarification

> Private generation semantic clarification forfixtures: retained-state
> `Generation` is accepted immutable version serial, initialize 1; each
> accepted candidate previous.Generation+1; refusal/control-marker
> transitionpreserves generation and AcceptedAt. MaxUint64 prior then new
> candidate cannot wrap or advance →GenerationOverflow safepriorstate/ledger
> unchanged. Separate initializer/lifecycle ownershiptoken notsamecounter;
> lifecyclefullpollerimage mustaccountthatifretainedoutsidehelper.
> Newoptionalpointer prior/result clone test mustfirstassert nonnil/distinct
> beforemutatingavoidnilpanic masking. Rootratifies routineprivateencoding
> semantics; noPublicspec/schema change.

## Prior instructions, review, and source identities

The full earlier retained-helper instructions, source receipt, and final hash
correction are preserved at:

- `run_observation_retained_version_helper_assignment_oct07.md`
- `run_observation_retained_version_helper_source_receipt_oct07.md`
- `run_observation_retained_version_helper_hash_correction_oct07.md`

The frozen helper source is `run_observation_retained_version.go`, SHA-256
`d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331`, 5,836
bytes. The test preimage is
`run_observation_retained_version_test.go`, SHA-256
`5365c1ff47cdf0006868b102a1fdcdf9c77a12d51cee4ab06b04af536f494978`, 12,464
bytes. The helper remains an always-rejected builder stub; the encoder is
unchanged. The independent source review rejected the test fixture bundle for
four incomplete assertion groups: refusal atomicity/recovery, terminal
nonretention, measured image boundaries, and identity/counter edge cases.

## Required test-only repair scope

Modify only `run_observation_retained_version_test.go`. Keep
`run_observation_retained_version.go` at the frozen hash. Add the smallest
focused assertions needed to cover all rejected findings:

1. For nonterminal overbudget refusal, prove caller prior's complete canonical
   bytes before and after the call are equal; prove result state equals the
   complete prior safe state except for the fixed retention-unavailable code;
   prove result canonical bytes have identical length and exactly match that
   expected marker-only image. Recovery must preserve the complete A/B/new
   ledger and first ordinals.
2. For terminal refusal, prove no candidate frame, ID, metadata, timestamp, or
   payload is retained. Compare the complete result state and image to the
   prior state with only its fixed terminal marker changed.
3. Measure canonical image bytes for accepted exact-boundary candidate at
   exactly `1<<20`, and computed rejected irreducible metadata image at exactly
   `1<<20 + 1`.
4. Verify one-window duplicate event IDs are invalid per
   `internal/adapters/sfwp/run_trace.go:143-146`; do not invent winner
   selection. Extend disappearance/reappearance coverage so existing exact
   IDs retain original ordinals, source order remains exact, and prior state is
   never mutated.
5. Cover generation overflow as clarified above and strengthen ordinal
   overflow to prove no wrap or partial mutation. Add a prior-to-result pointer
   clone case; check optional pointers are nonnil and distinct before
   dereferencing/mutating. With generation and high-water at `MaxUint64`, every
   fixed-width availability marker preserves canonical byte length.

No helper algorithm, manager lifecycle, `Next`, auth, I/O, public contract,
dependency, or integration change is authorized. If the test cannot express
these assertions against existing declarations, report the gap to root before
changing source. A different guard critic receives the original assignment
and full result. Report exact final hashes and diff; state explicit deviations
and make no runtime claim.

## Exact test preimage preservation

Before any edit, read-only decoding of
`.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/retained-test-5365-before-atomicity-repair-oct07.json`
produced 12,464 UTF-8 bytes and SHA-256
`5365c1ff47cdf0006868b102a1fdcdf9c77a12d51cee4ab06b04af536f494978`; the live
test file had the same byte count and hash, with decoded-byte equality
`True`. This is an exact source JSON wrapper, not runtime evidence.

No test source has been edited yet. Any future test-only diff will be recorded
in a separate immutable result receipt.
