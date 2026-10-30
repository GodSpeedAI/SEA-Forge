# Judgment Surface

## Overview
Bounded human judgment over pending approvals, composed over the existing approval inbox (`ApprovalInboxPage`). Approvals round-trip through `approval.decide`; nothing here invents judgment semantics.

## Interface
- `JudgmentSurface()` — the surface component [JudgmentSurface.tsx:JudgmentSurface]

## Invariants
- ["None"]

## Side Effects
- ["None"]

## Verification
- Run: `bun run test` (from workbench/)
