# Ports (contract boundary)

## Overview
The UI's contract boundary. Everything the world, history, artifacts, authority, execution and settlement bring into the UI crosses the port in shapes defined by the interface-contract package (`.agents/reports/interface-contracts`); `project.ts` is the only place contract shapes are read. Replacing the local adapter with the Go adapter must not change anything above this file.

## Interface
- `contract.ts` — the port types:
  - `CaseworkPort` — `getSnapshot`, `getSnapshotAt`, `dispatchIntent`, `resolveArtifact`, `queryTemporalTrajectory`, `subscribeEvents` [contract.ts:CaseworkPort]
  - `TemplateSourcePort` — `getTemplates`, `preflightTemplate` (feature-detect with `'getTemplates' in port`) [contract.ts:TemplateSourcePort]
  - `SessionPort` / `SessionIdentity` — identity from the gateway session or self-reported fixture (feature-detect with `'session' in port`) [contract.ts:SessionPort, contract.ts:SessionIdentity]
  - `NarrationPort` / `AgentNarrationStream` / `NarrationBeat` / `NarrationDirective` — the agent side (optional; the UI must work without it) [contract.ts:NarrationPort, contract.ts:AgentNarrationBeat]
  - Spec-04 shapes not yet in `types.ts`: `CognitiveRelationship`, `DisclosureLevel`, `CognitiveArtifact` [contract.ts:CognitiveRelationship, contract.ts:DisclosureLevel, contract.ts:CognitiveArtifact]
  - Proposed optional extensions under `x`: `XObject`, `XSnapshot`, `CausalRepresentation`, `ObjectExtensions`, `ToneHint` [contract.ts:XObject, contract.ts:XSnapshot]
- `project.ts` — contract snapshot → internal world: `projectObject(o, parentFallback)`, `projectSnapshot(snap, opts)`, `projectHistory(...)`, `loadCaseHistory(...)`, `compareCursor(a, b)`, `CORE_ID` [project.ts:projectObject, project.ts:projectSnapshot, project.ts:projectHistory, project.ts:loadCaseHistory, project.ts:compareCursor, project.ts:CORE_ID]

## Invariants
- Everything crossing into the UI does so in the shapes of the interface-contract package; base types are re-exported verbatim from `.agents/reports/interface-contracts/typescript/types` [contract.ts:CaseworkPort, project.ts]
- `project.ts` is the only place contract shapes are read; layouts, scene and panels only ever see the projection, and nothing there recognises particular ids or names [project.ts]
- Presentation is derived locally from kernel fields (`src/model/derive.ts`): live snapshots carry no `x` extensions; authored `x` extensions (the local fixture) win over derivation wherever present [project.ts]
- Case-level lifecycle intents (`REOPEN_CASE`, `TERMINATE_CASE`) ride in the snapshot's `available_actions` because they target the case, not an object [project.ts:CASE_LIFECYCLE_INTENTS]
- An opened event stream alone proves nothing about the source; `subscribeEvents`' `onOpen` carries that caveat (recovery proof lives in `src/app/live.ts`) [contract.ts:CaseworkPort.subscribeEvents]

## Side Effects
- ["None"] (type declarations and pure projection; I/O happens only in adapters)

## Verification
- Run: `bun test` (working directory: `apps/godspeed-cognitive-ui`; tests: `src/ports/project.test.ts`, `wireContract.test.ts`)
