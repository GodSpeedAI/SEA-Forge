# Pages

## Overview
Operate-space page components: readiness, case horizon and creation, approval inbox, operations monitor, evidence, run records, asset catalog, delegation roster and workbench, Thoth, system contract, and the route/standing presentation tables. Pages own governed reads via `src/hooks` and are composed into spatial surfaces by `src/surfaces/*` (the shell routes to them directly for operate paths). `SurfacesPages.tsx` provides alias re-exports and placeholder pages for the spatial surface menu.

## Interface
- `OperateRouteState`, `OperateRouteDefinition`, `OPERATE_ROUTES`, `OPERATE_ROUTE_BY_PATH` — operate route registry keyed by path [operateRoutes.ts:OPERATE_ROUTES]
- `EXECUTION_VARIANT`, `SETTLEMENT_VARIANT`, `CASE_STATE_VARIANT`, `CRITERION_VARIANT`, `VERDICT_VARIANT`, `ASSET_STANDING_VARIANT`, `DELEGATION_STANDING_VARIANT`, `VALUE_SOURCE_LABEL`, `humanize(value)` — governed status variant/presentation maps [standing.ts:EXECUTION_VARIANT]
- `ReadinessPage()` [ReadinessPage.tsx:ReadinessPage]
- `CaseHorizonPage()` [CaseHorizonPage.tsx:CaseHorizonPage]
- `CaseCreationWorkbench()` [CaseCreationWorkbench.tsx:CaseCreationWorkbench]
- `ApprovalInboxPage()` [ApprovalInboxPage.tsx:ApprovalInboxPage]
- `OperationsPage()` [OperationsPage.tsx:OperationsPage]
- `EvidencePage()` [EvidencePage.tsx:EvidencePage]
- `RunRecordPage()` [RunRecordPage.tsx:RunRecordPage]
- `AssetCatalogPage()` [AssetCatalogPage.tsx:AssetCatalogPage]
- `DelegationRoster()` [DelegationRoster.tsx:DelegationRoster]
- `DelegationWorkbench()` [DelegationWorkbench.tsx:DelegationWorkbench]
- `ThothPage()` [ThothPage.tsx:ThothPage]
- `SystemContractPage()` [SystemContractPage.tsx:SystemContractPage]
- `UnbackedSurface({...})` — placeholder for unbacked surface routes [UnbackedSurface.tsx:UnbackedSurface]
- SurfacesPages alias re-exports and placeholders: `ModelsPage`, `MemoryPage`, `CapabilitiesPage`, `ArtifactsPage`, `FederationPage`, plus `ThothPage`/`AssetsPage`/`CasesPage`/`InboxPage`/`OperationsPage`/`AdminPage` re-exports [SurfacesPages.tsx:SurfacesPages]

## Invariants
- ["None"]

## Side Effects
- Pages issue governed server reads/writes through `src/hooks` (host-bridge `sfwp_query`/`sfwp_command`), e.g. approvals decide via `useApprovals` and case creation via the authoring machine's command path [ApprovalInboxPage.tsx:ApprovalInboxPage]

## Verification
- Run: `bun run test` (from workbench/)
