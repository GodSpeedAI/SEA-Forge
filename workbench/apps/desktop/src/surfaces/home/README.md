# Home Surface

## Overview
The canonical CORE-centered orientation state. Home IS the CORE-centered state: no secondary object owns focus, globally relevant objects arrange around CORE, and the persistent Composer is available. Home renders live data from identity, contract, cases, and approvals hooks, projected via `projectHomeOrientation`, and reads connection state via `describeConnection`. Styling is in the co-located `HomeSurface.module.css`.

## Interface
- `HomeSurface()` — the surface component [HomeSurface.tsx:HomeSurface]

## Invariants
- Only source-backed values render; unanswered sources read as unknown, never as zero [HomeSurface.tsx:HomeSurface]

## Side Effects
- Reads server views through `useApprovals`, `useCaseList`, `useServerContract`, and `useIdentity` from `src/hooks` [HomeSurface.tsx:HomeSurface]

## Verification
- Run: `bun run test` (from workbench/)
