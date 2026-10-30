# Camera

## Overview
Pure projection and camera math for the 2.5D world (`src/camera/camera.ts`): world-to-screen projection with depth and tilt, pan/zoom around a focal point, tilt clamping, and time-based camera flights. No state lives here — the scene runtime (`src/scene/runtime.ts`) owns the camera and drives these functions per frame.

## Interface
- `project(p, cam, vp): Projected` — project a world point to screen coordinates (includes depth-derived scale; `PERSPECTIVE = 1400` world units) [camera.ts:project, camera.ts:PERSPECTIVE]
- `unproject(...)` — inverse mapping from screen to world [camera.ts:unproject]
- `panBy(cam, dx, dy)` / `zoomAt(...)` — focal-point-preserving pan and zoom [camera.ts:panBy, camera.ts:zoomAt]
- `clampTilt(t)` — restrained orbit bounds [camera.ts:clampTilt]
- `lerpCamera(a, b, t)` / `easeInOutCubic(t)` — interpolation helpers [camera.ts:lerpCamera, camera.ts:easeInOutCubic]
- `flightDuration(a, b)` / `sampleFlight(f, now)` — camera flights with deterministic duration and sampling [camera.ts:Flight, camera.ts:flightDuration, camera.ts:sampleFlight]
- Types `Viewport`, `Projected`, `Flight` [camera.ts:Viewport, camera.ts:Projected, camera.ts:Flight]

## Invariants
- Pure math only: no DOM, WebGL, or store access; every function is deterministic from its arguments [camera.ts]
- `project` accounts for camera tilt (rotation around the focal point) and per-point depth scale, so DOM, SVG and WebGL layers can be driven from one camera [camera.ts:project]

## Side Effects
- ["None"]

## Verification
- Run: `bun test` (working directory: `apps/godspeed-cognitive-ui`; camera tests: `src/camera/camera.test.ts`)
