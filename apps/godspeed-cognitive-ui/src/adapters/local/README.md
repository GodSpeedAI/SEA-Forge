# Local Contract Adapter (dev/test fixture)

## Overview
A local, contract-conformant stand-in for the Go system front end (`src/adapters/local/localAdapter.ts`). It implements the contract `CaseworkPort` plus the additive session (`SessionPort`) and template (`TemplateSourcePort`) surfaces over authored Northstar data. Authority is decided the way the contract describes (role-aware, stale-projection and duplicate checks), and governed execution is simulated as contract events (`execution_progress`, then `snapshot`, then `settlement_recorded`). This is NOT backend integration; every screen that uses it is labelled "Local contract adapter".

## Interface
- `LocalContractAdapter` — implements `CaseworkPort`, `SessionPort`, `TemplateSourcePort` [localAdapter.ts:LocalContractAdapter]
  - Session: `session()`, `login()`, `logout()`; self-reports `LOCAL_IDENTITY` (or starts logged out when `identity: null`) [localAdapter.ts:LOCAL_IDENTITY]
  - Reads: `getSnapshot`, `getSnapshotAt`, `queryTemporalTrajectory`, `resolveArtifact`
  - Authority: `dispatchIntent` — typed refusal kinds `AUTHORITY_DENIED`, `UNAUTHORIZED_ROLE`, `SOD_VIOLATION`, `STALE_PROJECTION`, `JUSTIFICATION_REQUIRED`, `UNAVAILABLE`, `INVALID`, plus `DUPLICATE_IN_FLIGHT` for a repeated `intent_id` [localAdapter.ts:LocalContractAdapter.dispatchIntent]
  - Events: `subscribeEvents` — future-only fixture stream with per-subscriber queue and ordered drain [localAdapter.ts:LocalContractAdapter.subscribeEvents]
  - Templates: `getTemplates`, `preflightTemplate` backed by `LOCAL_TEMPLATES` [localTemplates.ts:LOCAL_TEMPLATES]
- `LocalAdapterOptions` — `speed` (execution multiplier), `failArtifacts`, `corruptArtifacts`, `latency`, `identity` [localAdapter.ts:LocalAdapterOptions]
- `NORTHSTAR_CASE_ID` (re-exported here so the boot path imports the fixture through this module alone) [localAdapter.ts:NORTHSTAR_CASE_ID]
- Fixture data: `buildNorthstarHistory`, `NORTHSTAR_TRAJECTORY` [northstarData.ts:buildNorthstarHistory, northstarData.ts:NORTHSTAR_TRAJECTORY]; template case `TEMPLATE_CASE_ID` / `buildTemplateHistory` [templateData.ts:TEMPLATE_CASE_ID, templateData.ts:buildTemplateHistory]; artifact fixtures `PAYLOADS`, `ARTIFACT_TITLES` [payloads.ts:PAYLOADS, payloads.ts:ARTIFACT_TITLES]; `templateByRef`, `localPreflight`, `digestOf` [localTemplates.ts:templateByRef, localTemplates.ts:localPreflight, localTemplates.ts:digestOf]

## Invariants
- Seed integrity: a case's final snapshot and final trajectory cursor must match exactly, and must parse as a local `epoch.sequence` cursor, or `localCursorSeed` throws [localAdapter.ts:localCursorSeed, localAdapter.ts:LOCAL_CURSOR_RE]
- Authority refusals follow the contract: stale `client_cursor` yields `STALE_PROJECTION`; a stale preflight digest refuses with `STALE_PROJECTION` before commit [localAdapter.ts:LocalContractAdapter.dispatchIntent]
- Events are published with monotonic per-case cursor allocation (epoch padded to 10 digits, sequence ceiling 9,999,999,999) [localAdapter.ts:LOCAL_SEQUENCE_CEILING, localAdapter.ts:LocalContractAdapter.publishBatch]
- Local cursors are future-only: subscribers receive only events after the cursor at subscription time (asserted by the conformance suite's `future-only` mode) [caseworkPortConformance.ts:CaseworkPortConformanceScenario.resume]

## Side Effects
- Simulated asynchronous execution with `setTimeout`-based phases, progress events and settlements on an in-memory case store [localAdapter.ts:LocalContractAdapter.execute, localAdapter.ts:LocalContractAdapter.progress]
- Test/recovery hooks: artifact refs listed in `failArtifacts` fail resolution; `corruptArtifacts` resolve to malformed payloads [localAdapter.ts:LocalAdapterOptions]
- No network or disk access; all state is in-memory fixtures (Northstar history, trajectory JSON, artifact payloads) [northstarData.ts, payloads.ts]
- The `src/adapters/local/*.md` and `*.raw`/`*.json` repair-record files in this directory are evidence artifacts from earlier repair sessions, not source code.

## Verification
- Run: `bun test` (working directory: `apps/godspeed-cognitive-ui`; adapter tests: `src/adapters/local/localAdapter.test.ts`, `localAdapter.cursorBounds.test.ts`, `localAdapter.cursorOrder.test.ts`, `localAdapter.hostileThrownValues.test.ts`, `localAdapter.settlementAtomicity.test.ts`, `northstarData.test.ts`)
