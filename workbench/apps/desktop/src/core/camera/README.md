# Core Camera

## Overview
Camera state and pointer interaction for the CORE renderer (extracted from the Gargantua donor; see `/home/sprime01/projects/sea-rs/workbench/apps/desktop/src/core/PROVENANCE.md`). `CoreCamera` owns spherical orbit state with idle auto-orbit and inertia; `OrbitInteraction` binds pointer/wheel input from the canvas to camera impulses.

## Interface
- `CoreCameraState` — spherical camera state record [CoreCamera.ts:CoreCameraState]
- `HOME_CAMERA_STATE` — canonical home framing [CoreCamera.ts:HOME_CAMERA_STATE]
- `CORE_FOV`, `CORE_TAN_HALF_FOV` — field-of-view constants [CoreCamera.ts:CORE_FOV]
- `CoreCamera` — orbit camera class (applyWheelZoom, idle auto-orbit, inertia decay via `INERTIA_DECAY = 0.94`) [CoreCamera.ts:CoreCamera]
- `OrbitInteractionCallbacks` — input wiring contract [OrbitInteraction.ts:OrbitInteractionCallbacks]
- `OrbitInteraction` — pointer drag / wheel / keyboard handler (`DRAG_FACTOR = 0.0032`; wheel zoom delegated to `CoreCamera.applyWheelZoom`) [OrbitInteraction.ts:OrbitInteraction]

## Invariants
- Polar angle is clamped to `[PHI_MIN, PHI_MAX]` (`0.12` .. `PI - 0.12`), never poles [CoreCamera.ts:PHI_MIN]
- Idle auto-orbit engages only after `IDLE_MS = 3500` without input, at `AUTO_ORBIT_RATE = 0.028` [CoreCamera.ts:IDLE_MS]

## Side Effects
- `OrbitInteraction` attaches `pointerdown` on the canvas (with best-effort `setPointerCapture`) and `pointermove`/`pointerup`/`pointercancel` on `window` [OrbitInteraction.ts:OrbitInteraction]

## Verification
- Run: `bun run test` (from workbench/)
