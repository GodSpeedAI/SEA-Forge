# CORE renderer provenance

## Donor project

**Gargantua** — a single-file real-time GPU-raymarched Kerr black-hole demo
(relativistic lensing, Doppler beaming, volumetric accretion disk, Gaussian
bloom, anamorphic streak, ACES composite). No license header ships in the
donor file; the donor is treated as an internal white-label source: it is
**not** redistributed, only adapted in place. If the donor's license is later
identified as requiring attribution/redistribution notices, add them here and
next to the extracted modules.

## Golden reference

`workbench/apps/desktop/public/reference/gargantua.html` — a byte-identical
copy of the supplied donor (`sha256:9cfdc399…c93cd`, verified at copy time).
**Never mutate it.** It is served statically so the extracted renderer can be
compared against the original side by side (`/reference/gargantua.html` in
dev builds). All extraction PRs should visually compare against it.

## Method

White-label extraction, not reimplementation:

copy → preserve → separate → relocate → wrap → map → integrate.

Each subsystem was copied verbatim into its destination module, then adapted
only mechanically (imports/exports, globals → constructor state, DOM lookups
→ injected references). Donor comments are preserved where they explain
render behavior.

## Module map (donor source → destination)

| Donor block (`gargantua.html`) | Destination |
|---|---|
| `NOISE_GLSL` | `core/shaders/noise.glsl.ts` (verbatim) |
| `QUAD_VERT`, `QUAD_VERT3` | `core/shaders/quad.vert.glsl.ts` (verbatim) |
| `SCENE_FRAG` | `core/shaders/core.frag.glsl.ts` (verbatim template + same `${NOISE}` splice via `buildSceneFrag`) |
| `BRIGHT_FRAG` | `core/shaders/post.frag.glsl.ts` + `bright.frag.glsl.ts` shim (verbatim) |
| `BLUR_FRAG` | `core/shaders/post.frag.glsl.ts` + `blur.frag.glsl.ts` shim (verbatim) |
| `STREAK_FRAG` | `core/shaders/post.frag.glsl.ts` + `streak.frag.glsl.ts` shim (verbatim) |
| `FINAL_FRAG` | `core/shaders/post.frag.glsl.ts` + `composite.frag.glsl.ts` shim (verbatim) |
| `makePass`, `quadGeo`, `orthoCam` | `core/pipeline/RenderPass.ts` |
| `makeRT`, rt pyramid, `resize` targets | `core/pipeline/RenderTargets.ts` |
| `scenePass` (+ `sceneFragFinal`) | `core/pipeline/ScenePass.ts` |
| `brightPass` | `core/pipeline/BrightPass.ts` |
| `blurPass` + `gaussian` | `core/pipeline/BloomPass.ts` |
| `streakPass` | `core/pipeline/StreakPass.ts` |
| `finalPass` | `core/pipeline/CompositePass.ts` |
| `step` render portion | `core/pipeline/RenderPipeline.ts` (same order + constants) |
| `QUALITY`, `qi`, `qAvg`, `lastQChange`, per-frame block | `core/quality/AdaptiveQuality.ts` |
| `cam`, `updateCamera` | `core/camera/CoreCamera.ts` |
| pointer/wheel/key handlers | `core/camera/OrbitInteraction.ts` |
| `bhMain`, `blit`, `schedule`/`frame`/`step`, fps | `core/CoreRenderer.ts` |
| donor HUD/overlay HTML+CSS | `core/chrome/*` (visual treatment preserved, semantics white-labeled) |

## Material deviations from the donor

- **D-01 — THREE sourcing.** Donor loads `three.min.js` r128 from cdnjs at
  runtime. The app imports `three@0.180.0` from its Bun workspace (offline
  desktop builds cannot depend on a CDN; Tauri CSP forbids it). No shader or
  algorithm change follows from this.
- **D-02 — `PlaneBufferGeometry` → `PlaneGeometry`.** The donor's r128 API
  was removed in modern three; `PlaneGeometry` is the same buffer-backed
  2×2 fullscreen quad. One identifier, no behavior change.
- **D-03 — fatal surface.** Donor `bhFatal` wrote into `#fatal` DOM directly.
  The renderer reports `{ title, message, detail }` via `onFatal` and the
  white-labeled `RenderFailureSurface` renders it. Same failure taxonomy
  (WebGL2 unavailable / context creation failed / context lost / init
  failed), same messages.
- **D-04 — HUD sliders → dev panel.** Donor slider wiring moved to
  `dev/CoreTuningPanel.tsx` (same ranges/defaults/formatters); product chrome
  no longer exposes mass/spin/temp/bloom as user semantics.
- **D-05 — `H` key.** Donor toggled `#hud/#title/#readout/#hint` visibility
  by id. The renderer emits `core:toggle-chrome`; `CoreViewport` owns chrome
  visibility. Same key, same effect, no id coupling.
- **D-06 — pointer-capture guard.** Donor `setPointerCapture` unguarded; the
  extraction try/catches it (must not throw on platforms without capture).
- **D-07 — `resize` split.** Donor `resize()` sized renderer + rebuilt all
  targets + set two uniforms in one function. Preserved as
  `RenderPipeline.resize` (identical math); `CoreRenderer` adds the listener
  lifecycle around it.

No numerical constants, shader bodies, pass ordering, weights, thresholds,
hysteresis bounds, orbit factors, or timing values were changed.
