# Projections

## Overview
Pure view-model functions that turn server views/query results into the shapes the spatial surfaces render. No I/O and no React state: every function takes inputs and returns a projection, so rendering stays deterministic and testable.

## Interface
- `ProjectionStatus` — shared projection status type [projection.ts:ProjectionStatus]
- `HomeOrientation`, `projectHomeOrientation(...)` — CORE-centered home orientation [HomeProjection.ts:projectHomeOrientation]
- `CoreVisualSignals`, `CoreVisualIntent`, `projectCoreVisual(...)`, `activityFromPendingWork(...)` — visual intent for the CORE renderer [CoreVisualProjection.ts:projectCoreVisual]
- `Affordance`, `projectAffordances(...)` — surface affordances for the Composer [AffordanceProjection.ts:projectAffordances]
- `JudgmentSummary`, `projectJudgment(...)` — approval-inbox summary [JudgmentProjection.ts:projectJudgment]
- `EvidenceSummary`, `projectEvidenceList(...)` — evidence index summary [EvidenceProjection.ts:projectEvidenceList]
- `CaseSummary`, `projectCaseList(...)` — case list summary [CaseProjection.ts:projectCaseList]
- `TimelinePosition`, `projectTimelinePosition(...)` — timeline position [TimelineProjection.ts:projectTimelinePosition]

## Invariants
- ["None"]

## Side Effects
- ["None"]

## Verification
- Run: `bun run test` (from workbench/)
