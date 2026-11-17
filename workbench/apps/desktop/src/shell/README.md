# Shell

## Overview
The spatial application shell. `AppShell` composes the persistent CORE viewport (mounted once, never remounted on focus/surface change), the route content framed as spatial surfaces, the persistent context-bound Composer, and transient overlays (identity, readout, hints, evidence drawer, cell availability alert). `SurfaceRail` is the surface navigation rail; `EvidenceContext` distributes evidence records to the drawer; `connectionState.ts` derives human-readable connection status.

## Interface
- `AppShellProps` and `AppShell` — root spatial composition [AppShell.tsx:AppShell]
- `SurfaceRail` — surface navigation rail over the fixed `SURFACES` list [SurfaceRail.tsx:SurfaceRail]
- `EvidenceContextValue`, `EvidenceContextProvider`, `useEvidenceContext()` — evidence drawer context [EvidenceContext.ts:EvidenceContextProvider]
- `describeConnection(...)` — connection-state presentation [connectionState.ts:describeConnection]

## Invariants
- `CoreViewport` is persistent and mounted once; it never remounts on focus or surface change [AppShell.tsx:AppShell]
- Path-to-surface mapping is by route prefix via `SURFACE_BY_PATH` / `surfaceIdFor` [AppShell.tsx:SURFACE_BY_PATH]

## Side Effects
- `AppShell` subscribes to router location and to `focusController`/`zoomController` changes, and re-renders the viewport accordingly [AppShell.tsx:AppShell]

## Verification
- Run: `bun run test` (from workbench/)
