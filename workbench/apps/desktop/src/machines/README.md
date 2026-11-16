# Machines

## Overview
XState 5 state machine for governed case authoring. `caseAuthoringMachine` drives the draft → preflight → commit → status-recovery flow so a case commit can never silently lose track of an in-flight server request.

## Interface
- `CaseDraft`, `CommitSuccess`, `RejectedAsStale` — draft and commit outcome types [caseAuthoringMachine.ts:CaseDraft]
- `caseAuthoringMachine` — the `setup({}).createMachine(...)` machine [caseAuthoringMachine.ts:caseAuthoringMachine]
- Internal actors: `preflightActor`, `commitActor`, `statusRecoveryActor` (fromPromise-based) [caseAuthoringMachine.ts:commitActor]

## Invariants
- The commit actor is handed its request id; it never mints one, so `ambiguous` always has an id to poll via `request.get_status` [caseAuthoringMachine.ts:commitActor]
- An error is treated as no-side-effect only if the server certifies `no_side_effect === true`; anything else (including `request_timeout`) is ambiguous, on the safe side by default [caseAuthoringMachine.ts:provesNoSideEffect]

## Side Effects
- `commitActor` calls the host bridge `invoke("sfwp_command", { command: { verb: "case_commit", ... } })`, passing `actAs: selectedActorId()` when set [caseAuthoringMachine.ts:commitActor]
- `statusRecoveryActor` calls `invoke("sfwp_request_status", { requestId })` [caseAuthoringMachine.ts:statusRecoveryActor]
- `preflightActor` performs a preflight query over the host bridge [caseAuthoringMachine.ts:preflightActor]

## Verification
- Run: `bun run test` (from workbench/)
