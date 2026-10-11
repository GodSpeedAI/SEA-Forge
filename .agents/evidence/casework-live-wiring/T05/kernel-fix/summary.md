# T05-found kernel defects — fixed by the orchestrator (2026-09-25)

Found by the T05 completion builder (forbidden to touch crates/), fixed inline:

## K-1: execute_sandbox refused Operation::WriteFile
- Symptom: every write_file episode settled rejected with basis
  [episode_dispatch_error, input_error] ("execution request requires non-empty
  execute_command"); blocked T10-L4 and made T03's templates unexecutable.
- Root cause: the server routed every operation through the sandbox command
  executor; the CLI routes WriteFile through sea_forge_sandbox::materialize
  (pipeline.rs:519). The settlement engine already had the honest design
  (F-10 write_only basis) that the server never used (hardcoded write_only: false).
- Fix (case_dispatch.rs execute_sandbox): WriteFile ops materialize under the
  grant (no CommandStarted/Finished — no command ran), the written file is
  captured as a content-addressed artifact (ArtifactCaptured + evidence record,
  same path stdout/stderr take), and the SettlementClaim carries write_only=true.
- Note for T06/T09/T10 builders (decision log D-3-followups): a write-only
  episode's run trace is authority_evaluated, workspace_created,
  artifact_captured, settlement_recorded, ... — there are NO command frames for
  write-only items (honest: no command ran). L4's command-frame expectations
  apply to execute_command items.

## K-2: event-kind naming concatenated humps
- Symptom: bus frames published `case.trace.itemactivated` while the module doc
  promised snake_case (`item_activated`).
- Fix (sfwp/case_mutations.rs trace_kind_snake): proper camel->snake conversion.
- Callers updated: tests/sfwp_case_mutations.rs event-name literals; Go goldens
  re-captured (testdata/frames.json).

## Verification
- three-crate gate: 58 suites ok / 0 failed (kernel-fix/three-crate-gate.log)
- sfwp_case_mutations 10/10, sfwp_supervisor 4/4, case_templates_live 7/7
- Go: vet 0, go test -race ./... all ok, live suite 13 PASS + 0 subtest skips
  (only the env-gated TestGoldenCapture skips by design), gofmt clean
- Go goldens re-captured against the fixed server (32 frames)
