# Guards

## Overview
Client-side governance guard layer. `guards.ts` defines the guard taxonomy (guard IDs, verdicts, context, results) and pure evaluation; `protectedAction.ts` decides whether a protected operation may proceed; `useGuardContext.ts` binds guard inputs to the readiness/identity data from the hooks layer; `GovernedDenialSurface.tsx` renders the UI when a guard denies. The router blocks routes on the blocking guards.

## Interface
- `GuardId`, `GuardVerdict`, `GuardContext`, `GuardResult` — guard taxonomy [guards.ts:GuardId]
- `evaluateGuard(guardId, context)` — pure guard evaluation [guards.ts:evaluateGuard]
- `ProtectedOperation`, `ProtectedActionRefusal`, `ProtectedActionDecision`, `evaluateProtectedAction(...)` — protected-operation decision logic [protectedAction.ts:evaluateProtectedAction]
- `useGuardContext()` — assembles `GuardContext` from readiness/identity views [useGuardContext.ts:useGuardContext]
- `BLOCKING_GUARDS` — `["G1", "G2", "G9"]`, the guards that block routing [useGuardContext.ts:BLOCKING_GUARDS]
- `GovernedDenialSurfaceProps` and `GovernedDenialSurface` — denial UI with retry [GovernedDenialSurface.tsx:GovernedDenialSurface]

## Invariants
- Guard verdicts are limited to `"passed" | "failed" | "indeterminate" | "not_applicable"` [guards.ts:GuardVerdict]
- Only `operator` and `system` are self-sponsoring roles (`SELF_SPONSORING_ROLES`) [guards.ts:SELF_SPONSORING_ROLES]
- Guards G1, G2, and G9 block navigation to guarded routes [useGuardContext.ts:BLOCKING_GUARDS]

## Side Effects
- `useGuardContext` consumes TanStack Query data from `../hooks/useReadiness` and `../hooks/useIdentity` (server reads over the host bridge) [useGuardContext.ts:useGuardContext]

## Verification
- Run: `bun run test` (from workbench/)
