# Hooks

## Overview
Data layer for the Workbench renderer: TanStack Query hooks that read and write governed server views over the Tauri host bridge, plus the shared governed-query transport (`governedQuery.ts`) and error mapping (`bridgeError.ts`). Every inspect hook follows the same three steps: send the query, refuse the server's typed error envelope, then validate the body against its generated contract.

## Interface
- `queryGoverned<T>(...)`, `rejectGovernedError(...)`, `describeAjv(...)`, `GovernedViewError` (alias `CaseViewError`) — the one path from a renderer question to a validated SFWP answer [governedQuery.ts:queryGoverned]
- `toError(reason)`, `BridgeGovernedError` — error normalization [bridgeError.ts:toError]
- `useServerContract()`, `ServerContract`, `NegotiationResult`, `MethodStanding`, `standingOf(...)`, `STANDING_PRESENTATION`, `SURFACED_METHODS`, `CLIENT_PROTOCOL_VERSION = "1"`, `CONTRACT_QUERY_KEY` — server contract negotiation [useServerContract.ts:useServerContract]
- `useIdentity()`, `IdentityState`, `ResolvedActor`, `CellInfo`, `SupervisionState`, `selectedActorId()`, `cellNameFromRoot(...)`, `IDENTITY_QUERY_KEY`, `CELL_QUERY_KEY` — actor identity and cell info [useIdentity.ts:useIdentity]
- `useOperationsStream()`, `OperationsStream`, `useRequestStatus(...)`, `settlementOf(...)`, `SettlementStanding`, `EVENTS_QUERY_KEY`, `REQUEST_STATUS_QUERY_KEY` — governed event stream and request status [useOperationsStream.ts:useOperationsStream]
- `useReadiness(intendedOperation?)`, `UseReadinessResult`, `READINESS_QUERY_KEY` — readiness view [useReadiness.ts:useReadiness]
- `useCases()` family: `useCaseList()`, `useCaseOverview(caseId)`, `useCaseHorizon(caseId)`, `CASES_QUERY_KEY` [useCases.ts:useCases]
- `useApprovals(caseId?)`, `Verdict`, `DecisionOutcome`, `VERDICT_PAST`, `APPROVALS_QUERY_KEY` — approval inbox [useApprovals.ts:useApprovals]
- `useDelegations()`, `CancelOutcome`, `DELEGATIONS_QUERY_KEY` — delegation reads [useDelegations.ts:useDelegations]
- `useDelegationPreview(request)`, `DelegationPreviewRequest`, `delegationPreviewQueryKey(...)` — delegation preview [useDelegationPreview.ts:useDelegationPreview]
- `useRuns(caseId?)` family: `useRunList`, `useRunRecord`, `RUNS_QUERY_KEY` [useRuns.ts:useRuns]
- `useAssets()`, `ASSETS_QUERY_KEY` [useAssets.ts:useAssets]
- `useCaseEntryOptions()`, `CASE_ENTRY_OPTIONS_QUERY_KEY` [useCaseEntryOptions.ts:useCaseEntryOptions]
- `useThoth()`, `AskInput`, `Answer`, `QuestionKind`, `QUESTION_KINDS` — Thoth Q&A [useThoth.ts:useThoth]
- `useGovernedEventInvalidation(...)` — cache invalidation on governed events [useGovernedEventInvalidation.ts:useGovernedEventInvalidation]

## Invariants
- A governed error body is checked before schema validation; denial is a governed outcome, not a malformed response, and must never be reported as "failed contract validation" [governedQuery.ts:queryGoverned]
- The client speaks `CLIENT_PROTOCOL_VERSION = "1"` and surfaces only the methods in `SURFACED_METHODS` [useServerContract.ts:CLIENT_PROTOCOL_VERSION]
- Contract and identity queries deliberately do not refetch on window focus or interval, so standing does not flicker with server restarts [useServerContract.ts:useServerContract]

## Side Effects
- All server reads/writes go through the Tauri host bridge: `invoke("sfwp_query", ...)` for queries, `invoke("sfwp_command", ...)` for commands, `invoke("sfwp_request_status", ...)` for status [useServerContract.ts:useServerContract]
- `useOperationsStream` subscribes to host events via `listen(...)` from `@tauri-apps/api/event` (no host bridge: the stream still re-reads on demand) [useOperationsStream.ts:useOperationsStream]
- `useIdentity` persists the acting identity under a session-storage key (`window.sessionStorage` get/set/remove) [useIdentity.ts:selectedActorId]

## Verification
- Run: `bun run test` (from workbench/)
