# Spatial

## Overview
Spatial UI layer for the Workbench: the focus, zoom, and time/version controllers that define spatial state, plus `primitives.tsx`, the React framing primitives (`Surface`, `SpatialObject`, `Relationship`, `Artifact`) that every surface composes with.

## Subdirectories & Boundaries
- Focus: [./focus/README.md] - Global focus targets (core/object/anchor), `focusController` singleton, and focus transition timing.
- Time: [./time/README.md] - Time/version positions as kernel-minted opaque cursors; `timeVersionController` singleton.
- Zoom: [./zoom/README.md] - Representational depth control; `zoomController` singleton mapped by the renderer to orbit radius.

## Interface
- `SurfaceProps`, `Surface` — titled spatial surface frame [primitives.tsx:Surface]
- `SpatialObjectProps`, `SpatialObject` — focusable object node [primitives.tsx:SpatialObject]
- `RelationshipProps`, `Relationship` — labeled edge between objects [primitives.tsx:Relationship]
- `ArtifactProps`, `Artifact` — artifact node [primitives.tsx:Artifact]

## Invariants
- ["None"]

## Side Effects
- ["None"]

## Verification
- Run: `bun run test` (from workbench/)
