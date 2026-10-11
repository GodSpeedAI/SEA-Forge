# CORE Architecture — Gargantua White-Label Integration

CORE is the persistent visual representation of the governed capability
kernel: the Gargantua single-file WebGL black-hole implementation, extracted
verbatim into maintainable modules, white-labeled, and integrated behind a
narrow API. Application code never touches GPU internals.

## 1. Golden reference

`workbench/apps/desktop/public/reference/gargantua.html` — byte-identical
copy of the supplied donor (`sha256:9cfdc399…c93cd`). Never mutate it. Served
statically (dev: `/reference/gargantua.html`) for side-by-side comparison
with the extracted renderer.

## 2. Where CORE rendering lives

`workbench/apps/desktop/src/core/`:

- `CoreViewport.tsx` — persistent canvas, mounted once per app lifetime.
- `CoreRenderer.ts` — lifecycle + frame loop; the ONLY module that owns the
  `THREE.WebGLRenderer`.
- `CoreVisualState.ts` — visual-intent types + Home defaults (NOT domain truth).
- `camera/CoreCamera.ts`, `camera/OrbitInteraction.ts` — donor orbit math + input.
- `quality/AdaptiveQuality.ts` — donor ladder + hysteresis.
- `pipeline/` — `RenderPipeline`, `RenderTargets`, `RenderPass`, `ScenePass`,
  `BrightPass`, `BloomPass`, `StreakPass`, `CompositePass`.
- `shaders/` — `noise`, `quad.vert`, `core.frag` (scene), `post.frag`
  (bright/blur/streak/composite) + task-named re-export shims.
- `chrome/` — white-labeled donor overlays (identity, readout, hints, failure).
- `PROVENANCE.md` — donor→module map + every material deviation (D-01…D-07).

## 3. Donor-code modules

Everything under `core/shaders/`, `core/camera/`, `core/quality/`, and
`core/pipeline/` is substantially verbatim donor code (mechanical moves only:
imports/exports, globals→state, DOM→injected refs). Proven by
`core/donorParity.test.ts`, which diffs each shader against the golden file.

## 4. Renderer public API

`start() stop() resize() dispose() setVisualState() setFocus()
setCameraIntent() setZoom()` (+ read-only `cameraState qualityIndex
focusState zoomDepth activityLevel`). No Three.js types cross `src/core/`.

## 5. Pipeline order (donor, preserved)

scene raymarch → bright extraction → bloom pyramid → anamorphic streak →
final composite → screen. Weights, thresholds, stretches, and the
downsample-then-double-blur halo sequence are unchanged.

## 6. Semantic vs visual state

`CoreVisualState` is visual intent consumed by the renderer; nothing
authoritative derives from it. Domain truth flows one way:

SEA-Forge/domain → system front end → ports/adapters (`hooks/`, Tauri
bridge) → `projections/` → spatial UI → `CoreVisualProjection` → renderer.

`CoreVisualProjection` is deliberately restrained: mass/spin stay at donor
defaults; activity moves bloom ±0.15 / temp ±0.05. No `if case.X then
blackHole.spin = …` mappings exist anywhere.

## 7. Spatial grammar

`src/spatial/`: `Surface Object Relationship Artifact` primitives;
`focus/FocusController` (semantic intent; Home = CORE-centered default) +
`focus/FocusTransition`; `zoom/ZoomController` (representational depth);
`time/TimeVersionController` (opaque kernel cursors, null = live head).

## 8. Home / CORE

`/` renders `HomeSurface`: no secondary focus, CORE centered, live
orientation objects (cases, judgment, integrity) arranged around it,
Composer available. `FocusController.returnHome()` + renderer's
`setCameraIntent({ returnHome: true })` restore the canonical framing.
No "CORE page", no "Home card".

## 9. Adding a new surface

1. Wrap the capability's governed reads (existing hook/port, or the narrowest
   new port with a documented why) in `src/surfaces/<name>/<Name>Surface.tsx`.
2. Frame with `<Surface id title>`; focus changes go through
   `focusController`; time pins through `timeVersionController`.
3. Register the route in `src/router.tsx` + the rail entry in
   `src/shell/SurfaceRail.tsx` + the path→surface id in `AppShell.tsx`.
4. Project any new domain→UI mapping in `src/projections/` (pure functions
   + tests). Never invent semantics to populate the surface.

## 10. Domain → UI path

Hooks (`useCases useApprovals useIdentity useServerContract` → Tauri bridge
→ Unix-socket SFWP) feed projections, which feed surfaces/Composer/readout.
Mutations travel the existing SFWP command client; views re-read
source-backed state. Guards (`guards/`) still gate the shell.

## 11. Renderer tuning

`src/dev/CoreTuningPanel.tsx` (dev builds only): donor slider ranges,
defaults, and formatters for mass/spin/temp/bloom + FPS. Product chrome
never exposes renderer physics.

## 12. Removed old UI

`Sidebar GlobalHeader JourneyRibbon` (+ CSS/tests), `mockup.css`,
`mockupFidelity.test.ts`, `shellKeyboard.test.tsx`, and the `/`→Readiness
index (now `HomeSurface`; Readiness stays at `/readiness`). All underlying
pages, hooks, guards, and contracts retained and composed as surfaces.

## 13. Deviations from donor behavior

None semantic. Seven mechanical deviations D-01…D-07 in `PROVENANCE.md`
(npm three, `PlaneGeometry`, fatal callback, `H` event, tuning relocation,
pointer-capture guard, resize split). Visual/behavioral equivalence rests on
byte-parity shader tests + preserved constants; side-by-side GPU comparison
against `/reference/gargantua.html` remains for a GPU host (recorded
limitation — this migration ran headless).
