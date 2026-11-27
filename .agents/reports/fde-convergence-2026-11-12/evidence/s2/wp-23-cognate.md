# WP-23 evidence: assignment binding (Cognate half)

Worktree `~/projects/_wt/fde-cognate-integration`, branch `fde/integration`, started from `08fcb78`. Commits (all pushed with `git push origin fde/integration`):
`509e199` feat(runtime): bind runs to assignments (WP-23); `96f8300` test(realitytrace): AT-17 lifecycle end to end; `b38100c` test(runtime): assignment mutation coverage; refuse an assignment on the Connect wire.

## Specification used (read, not reinterpreted)

SEA-Forge `contracts/fde/assignment.v1.json` (sha256 `34782bbd...`), `frozen-reference.v1.json` (sha256 `7b0954ac...`, the `episode_ref` pattern the assignment schema `$ref`s), fixtures `fixtures/assignment/`, README section 11, decisions D26, D27, D34. Both digests equal the SEA-Forge `DIGESTS` file. The SEA-Forge copies were not touched. The Rust reference check (`crates/sea-forge-realitytrace/tests/fde_contracts.rs`, `s2::assignment_sequence`) was read for check order and the transition table; the Cognate state machine matches it.

## Built (paths under `packages/`)

- `runtime-api/schemas/assignment.v1.json`, `frozen-reference.v1.json`: byte copies; sha256 pinned in `runtime-api/test/assignment.test.ts`. `package.json` `files` gains `schemas`; devDependency `ajv ^8.17.1` (already in the lock via realitytrace; `bun.lock` gets only the three lines for it).
- `runtime-api/src/assignment.ts` (pure, no clock/I/O): id grammar (`parseAssignmentId`, `formatAssignmentId`, `parseEpisodeRef`); validators `validateAssignment`, `validateAssignmentEvent`, `validateAssignmentError` (schema shape with JSON pointers, then the semantic rules: id agrees with `episode_ref`, repeated fields agree with the id, unique currencies); budget helpers (`budgetBreaches`, `addSpend`, `spendOf`); state machine `stepAssignment` / `runAssignmentLifecycle` (checks in the contract order: id, episode, transition, link expiry, duplicate currency, budget); `AssignmentRefusedError` (message starts with the exact `ASSIGNMENT_*` code, `.assignmentCode`). Constants (id pattern and length bound, currency enum, unit rules per currency, required lists, sizes) are read from the vendored schema, not copied.
- `runtime-api/src/types.ts`: `startRun` request gains `assignment?` and `linkExpiresAt?`; `AssignmentView`, `AssignmentService`.
- `runtime-bun/src/assignment.ts`: fold of the run stream into the assignment state; `createAssignmentService` (`getAssignment`, `interruptAssignment`, `resumeAssignment`, `recordSpend`), exposed as `Runtime.assignments`. New authorized actions `assignment.interrupt|resume|spend` (default-deny policies must allow them).
- `runtime-bun/src/service.ts`: `startRun` validates the assignment (contract code on refusal), requires `caseContext` (worldRef, semanticRef, role, kind are not in the assignment) and refuses disagreement (case/step: `ASSIGNMENT_ID_INVALID`; episodeRef: `ASSIGNMENT_EPISODE_CHANGED`; assignmentId: `ASSIGNMENT_ID_INVALID`), fills `episodeRef` and `assignmentId` into the frozen context, freezes `assignment` (brief, budget, ids) and `linkExpiresAt` on `run.started`, commits `assignment.issued` and `assignment.started` in the same append, and takes a unique `assignments` record per tenant and id (a second run for an issued id is `ASSIGNMENT_ID_INVALID`; an idempotent redelivery of the same start returns the same run). `cancelRun` records `assignment.cancelled` before `run.cancelled`.
- `runtime-bun/src/executor.ts`: a run whose assignment is `interrupted` is not executed (also after a restart); on completion/failure the terminal `assignment.completed|failed` (carrying cumulative spend) is recorded before `run.completed|failed`.
- `realitytrace/src/interaction-event.ts`: `assignment.interrupted`, `assignment.resumed`, `assignment.spend` are consequential interaction events (status `interrupted`, `succeeded`, `failed` when an overrun).
- `protocol-connect/src/index.ts`: the remote client refuses `assignment` (the wire has no field) instead of dropping it.

## CEP extension and assignment id

The vendored `godspeed.interaction_event.v1` already has `context.assignment_id` and `correlation.assignment_id` (pattern `^assign:[^:\s]+:[^:\s]+:[0-9]+:[0-9]+$`), so no workaround was needed and the vendored CEP schema is untouched. Every consequential event of an assignment run carries the id in both; verified against the schema in AT-17 tests.

## Behavior decisions

- Spend is reported (`recordSpend`: amount added per currency, or `"unknown"`); Cognate measures nothing by itself. Initial spend is 0 for all five (nothing spent at issue, as in the fixtures); an `unknown` report makes that currency `unknown` for good (never counted as 0).
- Overrun: one commit holds `assignment.spend` (cumulative spend including the overrun, `exceeded` list, code), `assignment.failed` (lifecycle `Event` carrying the last in-budget spend and reason `ASSIGNMENT_BUDGET_EXCEEDED: ...`, so every lifecycle event stays acceptable to the contract) and `run.failed` (error starts with the code). The running pass is aborted, open continuations cancelled. The spend call returns the view (status `failed`, `budgetExceeded`); it does not throw. Later spend on the stopped run is refused `ASSIGNMENT_TRANSITION_INVALID`.
- Resume reuses the run and the id (`resumeAssignment`); it is refused `ASSIGNMENT_LINK_EXPIRED` when now is after the `linkExpiresAt` given at start (no override at resume: a new link means a new delegation, a new `k`, a new run). Interrupted/resumed/redelivered requests are idempotent: event ids derive from scope, key and request, so a redelivery is a no-op; the same key with a different request is `already_exists`.
- A run with no assignment: unchanged (no new required field, no assignment events, no extra outbox intents; `getAssignment`/`interruptAssignment` on it is `failed_precondition`).

## Tests and counts

- `bun test packages/runtime-api/test/assignment.test.ts`: 72 pass, 0 fail (190 expect calls).
- `bun test packages/runtime-bun/test/assignment.test.ts`: 19 pass, 0 fail.
- `bun test packages/realitytrace/test/assignment-at17.test.ts`: 4 pass, 0 fail.
- `bun test packages/protocol-connect/test/assignment-wire.test.ts`: 1 pass, 0 fail.
- Gates: `bun run typecheck` clean; `bun run check`: `Substrate check passed: 81 requirements (81 implemented, 0 blocked, 0 candidate); no freeze claimed.`; `bun scripts/check-readmes.mjs`: `check-readmes: OK`.
- Full suite (`bun run test`, pg0 up, `SXR_BIN` set): `1223 pass`, `32 skip`, `0 fail`, `Ran 1255 tests across 145 files. [238.26s]` (baseline 1127 / 32 / 0; +96 = 72 + 19 + 4 + 1).

## Fixture and scenario coverage (every file of the vendored contract set runs)

- valid: 7/7 (Assignment 3, Event 3, Error 1): accepted by Ajv on the vendored schema and by the Cognate validator.
- invalid: 24/24: refused by Ajv and by the Cognate validator, and the Cognate issues include the manifest's pointer; every id-grammar fixture also carries `ASSIGNMENT_ID_INVALID`.
- semantic_invalid: 5/5: schema-valid under Ajv, refused with the manifest's exact code (`ASSIGNMENT_ID_INVALID` x3, `ASSIGNMENT_BUDGET_INVALID` x2).
- lifecycle scenarios: 10/10 with the expected outcome or code (ok x3; `ID_INVALID`, `EPISODE_CHANGED`, `TRANSITION_INVALID` x3, `LINK_EXPIRED`, `BUDGET_EXCEEDED`). A test asserts the scenarios cover every code except `BUDGET_INVALID` (a payload rule, covered by the two semantic fixtures). The fixture directory and manifest are vendored under `runtime-api/test/fixtures/assignment/` (manifest sha256 pinned; a test checks every file is listed).
- Own cases added where the contract fixtures leave a guard unobserved: id vs episode with agreeing fields (Assignment and Event).

## Mutation checks (guard disabled, tests failed, restored; harness re-applies and restores the file each time)

All 35 killed: M1 id length bound; M2 id-vs-episode agreement (first survived: the fixtures are caught by the redundant fields check, so an own test was added, then killed); M3 fields-repeat-id; M4/M4b duplicate currency in budget / spend; M5 lifecycle id change; M6 episode change; M7 transition table; M8 link expiry (3 fail, incl. the service test); M9 budget exceeded in the state machine; M10 unknown limit read as 0; M11 unknown spend counted as 0; M12/M13/M14 case/step, episode, id agreement in `startRun`; M15 payload validation in `startRun`; M16 one-run-per-id record; M17 interrupted run still executes (first survived; a restart test was added, then killed); M18 link expiry not stored at start; M19 `run.failed` not appended on overrun; M20 overrun not detected in `recordSpend`; M21 redelivery replay check; M22 terminal assignment event not recorded; M23 `assignment.cancelled` not recorded; M24 assignment events not interaction events; M25 interrupt does not stop the running pass (first survived, protected only by the stream-version conflict; a side-effect assertion was added, then killed); M26 resume does not re-execute; M27 spend-while-running guard (first survived by coincidence; own test added, then killed); M28 unit enum per currency; M29 money ISO pattern; M30 untrusted label; M31 reason required; M32 five-entry size; M33 negative amount; M34 additional properties; plus the wire guard in `protocol-connect`.

## AT-17 lifecycle part (Cognate): status

Met in Cognate: open from an assignment (id, episode_ref, brief, budget frozen on the run), every consequential event carries `assignment_id` in `context` and `correlation` and validates against the vendored CEP schema, interrupt, resume with the same id (episode unchanged), complete; budget exceed stops the run with `ASSIGNMENT_BUDGET_EXCEEDED` and records the overrun; spend per currency or `unknown`; redelivery idempotent (one record per event, byte-identical extension). Tests: `realitytrace/test/assignment-at17.test.ts` and `runtime-bun/test/assignment.test.ts`. Not part of this half: verification bound to `episode_ref` (WP-43/44), the AT-17 harness `--only AT-17` run against SEA-Forge, context bundle labelling beyond storing `trust: untrusted_reference`.

## Not done

- Connect/proto wire for `caseContext` and `assignment`: needs proto regeneration, not small; the remote client now refuses `assignment` visibly (`failed_precondition`); `caseContext` over the wire remains as in WP-15 (dropped).
- `assignment.issued/started/completed/failed/cancelled` are not interaction events (run.* events already carry the extension); only interrupted, resumed and spend are.
- No automatic time/attention/money measurement: spend is reported by the caller.
- No `delegation-link` object is read: Cognate takes `linkExpiresAt` as a parameter on `startRun`.
- `assignment.v1`'s `Error` payload is validated but Cognate raises `AssignmentRefusedError` (code in message and `.assignmentCode`), not the wire `Error` body.
- Signatures (WP-33), Workbench UI, SEA-Forge server: out of scope.
- Interactions with `RunView`: no `interrupted` run status; an interrupted run reports `running` (the assignment view has the status). Changing `RunView.status` would touch proto, ag-ui and clients.

## What the Workbench half needs

Call `startRun` with `assignment` (from the SEA-Forge delegate result) plus `caseContext` and `linkExpiresAt` (the delegation link's `expires_at`), then `Runtime.assignments` (`interruptAssignment`, `resumeAssignment`, `recordSpend`, `getAssignment`) for the operator controls; an in-process runtime or a new wire (proto fields for `assignment`, `linkExpiresAt`, and the `AssignmentService` RPCs) is needed to do it across Connect. Show `AssignmentView.status`, `spend` against `assignment.budget`, `budgetExceeded`, `brief` (context refs labelled untrusted). Action policy must allow `assignment.interrupt|resume|spend`.
