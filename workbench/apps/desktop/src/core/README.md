# Core

## Overview
The CORE renderer subsystem: a persistent, GPU-raymarched WebGL2 viewport (Kerr black-hole scene with bloom, streak, and composite post-processing) at the center of the spatial Workbench. The renderer was extracted white-label from the Gargantua donor; provenance, golden reference rules, and the donor-to-module map are recorded in `PROVENANCE.md` (must-read for changes here).

## Subdirectories & Boundaries
- Camera: [./camera/README.md] - `CoreCamera` orbit state (idle auto-orbit, inertia, polar clamping) and `OrbitInteraction` pointer/wheel wiring.
- Chrome: [./chrome/README.md] - Overlay UI: identity badge, context readout, interaction hints, render failure surface.
- Pipeline: [./pipeline/README.md] - WebGL2 render pipeline: scene/bright/bloom/streak/composite passes over shared render targets.
- Quality: [./quality/README.md] - Frame-time-driven `AdaptiveQuality` ladder with hysteresis and cooldown.
- Shaders: [./shaders/README.md] - GLSL sources as TS string exports, verbatim from the donor.

## Interface
- `CoreFatalInfo`, `CoreRendererOptions`, `CoreRenderer` — renderer class owning the canvas, camera, interaction, pipeline, and adaptive quality; exposes the `canvas` and rebuilds targets on resize/context loss [CoreRenderer.ts:CoreRenderer]
- `CoreViewportProps`, `CoreViewport({...})` — React wrapper mounting `CoreRenderer` on a canvas element and rendering `RenderFailureSurface` on fatal errors [CoreViewport.tsx:CoreViewport]
- `DEFAULT_VISUAL_PARAMS`, `CoreVisualParams`, `CoreFocusKind`, `CoreFocus`, `CoreCameraIntent`, `CoreVisualState`, `HOME_VISUAL_STATE`, `homeVisualState()` — visual state the shell projects into renderer intents [CoreVisualState.ts:CoreVisualState]

## Invariants
- The golden reference `public/reference/gargantua.html` must never be mutated; extraction PRs compare against it [PROVENANCE.md:Golden reference]
- Extraction is copy/preserve/adapt, not reimplementation: donor blocks map to modules per the `PROVENANCE.md` table [PROVENANCE.md:Module map]
- The viewport is mounted once and never remounted on focus or surface change (enforced by the shell) [../shell/AppShell.tsx:AppShell]

## Side Effects
- Creates a WebGL2 context on the canvas (`canvas.getContext("webgl2")`), registers `webglcontextlost` handling and a `window` resize listener, and issues per-frame GPU draw calls [CoreRenderer.ts:CoreRenderer]

## Verification
- Run: `bun run test` (from workbench/)
