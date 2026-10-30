# Zoom Controller

## Overview
The zoom boundary: zoom changes representational depth (surface → detail), never mere pixel magnification. The renderer maps a depth step to a modest orbit-radius change around the Home framing (`CoreRenderer.setZoom`). A single `zoomController` instance lives for the application lifetime.

## Interface
- `ZoomController` — integer depth state with `zoomIn`/`zoomOut`/`setDepth` and listener subscription [ZoomController.ts:ZoomController]
- `zoomController` — application-lifetime singleton [ZoomController.ts:zoomController]

## Invariants
- Depth is clamped to non-negative values (`Math.max(0, depth)`) [ZoomController.ts:setDepth]

## Side Effects
- ["None"]

## Verification
- Run: `bun run test` (from workbench/)
