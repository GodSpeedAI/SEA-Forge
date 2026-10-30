# Scene

## Overview
The rendered world: one WebGL canvas (Core, particles, dust), one SVG layer (orbits, links), and one DOM layer (objects, excerpts). React renders structure (which nodes exist, which representation); `SceneRuntime` owns everything that changes every frame — the camera, per-object tweens, LOD, and the imperative writes to all three layers from the same camera (`src/scene/runtime.ts`).

## Interface
- `SceneRuntime` — the per-frame engine: `start()`, `dispose()`, `resize(w, h)`, `flyTo(cam)`, `currentCamera()`, `renderIds()`, `placement(id)`, `attachSvg(svg)`, `attachCore(core)` [runtime.ts:SceneRuntime]
- `Scene` — the React composition of the three layers; `snapshotView(store, runtime)` reads runtime state for React [Scene.tsx:Scene, Scene.tsx:snapshotView]
- `SceneServices` — the callbacks the scene needs from the app: the artifact service, `invoke(object, action)` (consequential ones open judgment), and `centerActions(state)` (representation switches at the current local center) [Scene.tsx:SceneServices]
- `ObjectNode` — one world object; its representation is chosen by semantic LOD, never by scaling the same thing up: dot → labeled sphere → sphere with its parts [ObjectNode.tsx:ObjectNode]
- `CoreCanvas` — the only WebGL in the app: singularities, accretion particles, dust field [coreCanvas.ts:CoreCanvas, coreCanvas.ts:Singularity]
- `LodTracker` / `nextLod(prev, apparentPx)` — semantic LOD with hysteresis; thresholds `LOD_UP = [34, 120, 420]` px, `HYSTERESIS = 0.78` [lod.ts:LodTracker, lod.ts:nextLod, lod.ts:LOD_UP, lod.ts:HYSTERESIS]

## Invariants
- One canvas, one WebGL context: `CoreCanvas` is the only WebGL in the app and knows nothing about the world model — the scene runtime feeds it screen-space values derived from the same camera as the DOM layer every frame [coreCanvas.ts:CoreCanvas]
- Position, size (`--r`) and opacity are written per frame by the scene runtime, not by React [ObjectNode.tsx:ObjectNode]
- Object representation switches by semantic LOD with hysteresis, never by scaling a single representation [lod.ts:nextLod]
- The past is read-only: backend-listed actions are absent in the past (`onInvoke` absent when `past`) [ObjectNode.tsx:ObjectNodeProps]

## Side Effects
- Per-frame imperative DOM writes, SVG path updates and WebGL draws driven by `requestAnimationFrame` [runtime.ts:SceneRuntime.start, coreCanvas.ts:CoreCanvas]
- WebGL context creation and teardown in `CoreCanvas` (attach/dispose) [coreCanvas.ts:CoreCanvas]
- Camera flights and tweens mutate runtime camera state on each frame [runtime.ts:SceneRuntime.flyTo]

## Verification
- Run: `bun test` (working directory: `apps/godspeed-cognitive-ui`; LOD tests: `src/scene/lod.test.ts`)
