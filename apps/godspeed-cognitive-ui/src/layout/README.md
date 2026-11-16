# Layout

## Overview
Pure, data-driven layout (`src/layout/layout.ts`): (world, surface, focus, overrides) → placements keyed by object id, for the orbital, causal, judgment, design and comparison surfaces. Geometry is authored in "mock pixels" (the 1672×941 reference images) relative to the current center of gravity, then converted to world units with the parent's frame scale, so entering an object is a real camera zoom.

## Interface
- `layout(input: LayoutInput): LayoutResult` — the single entry point: placements, orbits and links for a surface [layout.ts:layout, layout.ts:LayoutInput]
- `childrenOf(snap, id)` — hierarchy query used by the scene and panels [layout.ts:childrenOf]
- `canonical(snap)` — canonical ordering/coordinates map for a snapshot [layout.ts:canonical]
- `causalMembers(snap, centerId)` — the objects of a causal reading at a local center (drives the causal surface and the "causal view" command) [layout.ts:causalMembers]
- `cameraFor(pos, frame, fit, centerScreenY)` — camera framing for a target position [layout.ts:cameraFor]
- Constants `DESIGN_W = 1672`, `DESIGN_H = 941`, `FOCAL_Y = 0.46` [layout.ts:DESIGN_W, layout.ts:DESIGN_H, layout.ts:FOCAL_Y]
- Output shapes come from `src/model/types.ts`: `Placement`, `Orbit`, `Link`, `LayoutResult` [types.ts:Placement, types.ts:LayoutResult]

## Invariants
- Pure function of (world, surface, focus, overrides): no timers, no store, no DOM [layout.ts:layout]
- Every hierarchy level is 10× smaller than its parent, so zooming without clicking reveals the same nested world [layout.ts]
- Nothing here recognises particular object ids; layout is driven by the projected world (kinds, parents, actions) [layout.ts]

## Side Effects
- ["None"]

## Verification
- Run: `bun test` (working directory: `apps/godspeed-cognitive-ui`; layout tests: `src/layout/layout.test.ts`)
