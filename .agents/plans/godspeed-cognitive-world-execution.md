# GodSpeed Cognitive World: Executable Experience Plan

Updated: 2026-09-20  
State: Authority correction and donor-based 3D scene are integrated; rich-mode narration and full journey acceptance remain open. Preserve all valid work and unresolved failures.  
Governing authority: `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml`; the existing `godspeed-casework-cognitive-environment.plan.yaml` governs settlement and legacy UI removal. This checklist tracks the requested frontend experience and does not mark those settlement tasks complete.

## Working rules

- The architect assigns bounded implementation tasks after Graft and lexical narrowing. Luna owns routine source work. Terra receives an already narrowed seam only when integration radius justifies it. The architect owns architecture, rendered inspection, integration review, and final verification.
- The React world consumes cognitive projections and available actions. Consequential outcomes come from the Go/application boundary and are never asserted by local UI state. Mock behavior stays visibly labeled.
- Work grows as one persistent scene, not independent page or panel routes. Delete superseded presentation only after dependency and blast-radius inspection; preserve core contracts, adapters, generators, and useful tests. The separate legacy Tauri/Gauntlet removal remains governed by T12–T14 of the existing plan.
- Donors live in `.tmp/donor/`. The user explicitly prefers copying coherent licensed source slices wholesale, then tailoring them to GodSpeed. Record repository, pinned commit, source path, license, modifications, and retained notice for every copied or adapted source. Do not copy media without its own license.
- A checked box requires the named evidence. Passing typecheck, tests, or a worker report alone does not constitute visual acceptance. Each visible slice is run and inspected before advancing.
- Preserve preexisting worktree changes. No blanket reset, clean, or unrelated formatting. Run heavy gates serially.

## Baseline and ownership

- Existing UI target: `apps/godspeed-cognitive-ui/`; frozen handoff: `.agents/reports/interface-contracts/`.
- Existing runnable demo: `just casework-demo-up` (`http://127.0.0.1:4178`). The Go demo currently labels its fixture projection; it is not governed integration evidence.
- The 2026-09-20 opening screenshot showed overlapping objects and an overactive custom Core. A focused view and composer journey were inspected. The initial `just casework-ui-check` passed, but these visual defects kept acceptance open. The subsequent authority correction is complete: spec v0.2.3 and plan v0.2.4 are hash-bound, the validator passed, and the focused interface contract test passed 9 tests. Those checks clear the authority gate; they do not settle rendered Core or camera acceptance.
- Agent assignments: Luna, spatial donor/layout; Luna, licensed black-hole donor transplant; Luna, frozen-contract audit. Camera and later integration work receive separate scoped assignments after those reports.

## Deterministic search and agent dispatch

For this indexed repository, the architect starts with one `graft map` or a scoped `graft ask ... --source` when the seam is unknown. Known symbols use `graft grep`; signature changes and deletions use `graft callers ... --depth 2` (or `--depth all` for a multi-file refactor); candidate files use `graft skeleton`. Then use `rg --files` for inventory and `rg` for exact paths/literals, and read only the narrowed ranges. Donor repositories are not indexed by Graft: use `rg --files`, `rg` for candidate mechanics and license files, then read only those files and their immediate imports. Never hand an agent a repository-wide “explore” task or a dump of broad search output.

Before each dispatch, the architect supplies one affordance, candidate files/symbols, dependencies, target interface, expected artifact, and verification command. The worker reports exact paths/lines, mechanic retained, donor semantics discarded, license, changes, tests, and remaining ambiguity. The architect reviews the actual diff and render. If a first search is weak, switch to the appropriate lexical or symbol tool; do not repeat broad reasoning.

| Slice / bounded seam | Worker | Deterministic narrowing before dispatch | Worker deliverable and check |
| --- | --- | --- | --- |
| 0: old presentation removal | **Luna** after architect identifies exact shell and callers | `graft grep` entry symbol; `graft callers ... --depth all`; `rg --files` for routes/styles/tests | Remove only obsolete visual shell/CSS and route; preserve core/adapters; `just casework-ui-check` |
| 0: donor licensing and mechanics | **Luna** (separate spatial and Core tasks) | `rg --files .tmp/donor`; `rg` for `LICENSE`, camera/orbit/shader symbols; targeted imports | Pinned commits, license record, smallest mechanic slice; no media copying |
| 0/4: frozen contract mapping | **Luna** for inventory, **Terra** only if typed shape conversion spans client, Go wire, and UI core | `graft ask` scoped to `apps/godspeed-cognitive-ui`; `graft callers` on adapter methods; exact `rg` over handoff TypeScript | Typed adapter and contract tests; no backend semantic rediscovery |
| 1: Core transplant | **Luna** | Donor shader/source file inventory and license; `graft skeleton`/`callers` for `CoreObject` | Licensed donor mechanic replaces custom Core; provenance; screenshot and build |
| 1: object spacing, composer, relationships | **Luna**, one affordance per task | `graft ask` for arrangement and display; `rg` exact CSS/component identifiers; inspect fixture object coordinates | Focused layout tests, clean idle/narration render, `just casework-ui-check` |
| 2: camera travel and recursive focus | **Terra** after architect narrows `CameraIntent`, `CameraRig`, `SceneCanvas`, `layout`, and focus actions | `graft callers focus --depth 2`, `graft skeleton` candidate files, exact transition tests | World-space camera journey with stable anchor/back path; mid-transition and settled browser evidence |
| 3: semantic zoom and comparison | **Luna** for single-module tests/layout; **Terra** if scene/core transition crosses multiple systems | `graft callers setZoom --depth 2`; `graft grep` comparison commands; targeted tests | Distinct resolution bands and uncluttered same-world comparison |
| 4/5: projection, time, role actions | **Luna** for adapters/tests; **Terra** only for live transport or temporal reconstruction spanning core/Go/scene | `graft ask` scoped to adapters/core; `graft callers` on intent and temporal ports; exact handoff TS files | Mock journey and role tests, typed Go integration, honest provenance |
| 5: artifacts and lazy loading | **Luna** for individual renderers and registry; **Terra** for scene-to-DOM continuity if needed | `graft grep` artifact refs; `graft callers resolve --depth 2`; inspect registry and one renderer pattern | Anchor/expand/close continuity and chunk evidence |
| 6: CopilotKit and cutscene | **Terra** for choreography seam; **Luna** for isolated semantic command tests | `graft ask` agent-command flow, `graft callers` on shared core actions, exact narration tests | Same action vocabulary, interrupt/resume browser journey, adapter tests |
| 7: accessibility/performance/verification | **Luna** for bounded fixes and tests; architect personally verifies integrated experience | Browser a11y/performance output; `rg` exact failing selector; Graft callers before code changes | Targeted fixes and gate logs; architect owns final 24-point journey and acceptance |

**Terra dispatch gate:** an already narrowed seam must have substantial integration radius (camera/focus, shader adaptation beyond a local transplant, scene/DOM artifact continuity, temporal reconstruction, CopilotKit choreography, or cross-module TypeScript/performance failure). Luna remains the default. The architect does not perform routine implementation or use an agent as a filesystem search engine.

## Execution checklist

### 0. Preserve and narrow

- [x] Record the preexisting dirty inventory and audit the active renderer entry/presentation with Graft before removing anything. The current `main.tsx` mounts one scene-based `App.tsx`; no legacy route, router, or conventional panel shell remains. Old fixture providers are referenced only by core tests and were preserved. Evidence: Luna's scoped entry/caller audit and the preserved `git status --short` inventory; no deletion was warranted.
- [x] Keep five pinned donors in `.tmp/donor/`: Tycho, Racing Game, blackhole-ts, BLACK-HOLE, and threejs-galaxy-shader. Evidence: `.agents/reports/godspeed-casework-cognitive-environment/godspeed-cognitive-world-donor-provenance-2026-09-20.md` records each checkout's remote, exact HEAD, license/notice status, candidate source paths, adapted versus inspection-only scope, and the BLACK-HOLE unresolved tracked-license caveat; no unresolved-license source is copied.
- [x] Audit the frozen TypeScript client/mock against the current UI and Go transport seam. Resolve any shape mismatch at a typed adapter boundary before claiming contract parity. Evidence: spec/plan authority correction bound to `09d4292b956b09066b84bb03014413cae2b14a2c89f5338b34b6ebf542ab1dae`, validator PASS, and focused Bun conformance test (9 pass) across prose, TypeScript, schemas, examples, and tests. Full draft-2020-12 semantic validation remains an explicit limitation.

### 1. First frame: persistent world

- [x] Full viewport, near-white field, centered Core, sparse 3D CognitiveObjects, faint contextual relationships, and one bottom-center search composer; no alternate conventional route. Evidence: architect-inspected `/tmp/godspeed-responsive-final-1280.png`, `/tmp/godspeed-responsive-final-390-clean.png`, and fresh `/tmp/godspeed-final-320.png`. Narrow system labels move to OUTLINE to preserve scene clarity.
- [x] Replace the custom Core with the licensed blackhole-ts geodesic/accretion mechanic, tailored to a legible 3D black hole with lensing, an inclined slow-moving disk, sparse matter, and reduced-motion support. Evidence: `COREOBJECT_PROVENANCE.md`, `/tmp/godspeed-core3d-final.png`, two motion-frame pairs, scene tests, and architect-inspected render.
- [x] Separate system-scale objects deterministically while preserving salience and stable identity. Evidence: rich Casework spacing tests (11 focused pass), `/tmp/godspeed-rich-spacing-final-opening.png` and `/tmp/godspeed-rich-spacing-final-task-focus.png` inspected by architect; supplied `spatial_layout` remains unchanged.
- [x] Keep composer compact while idle and during narration; long captions wrap in a bounded surface and narrow provenance/outline stay above it. Evidence: `/tmp/godspeed-caption-desktop-narration.png`, `/tmp/godspeed-caption-narrow-fixed.png`, `/tmp/godspeed-final-320.png`, architect-inspected.
- [x] Run `just casework-ui-check`, inspect browser errors, and correct the largest perceptual mismatch before slice 2. Latest integrated gate: 95 tests, typecheck and production build PASS; browser checks so far show no runtime errors. Re-run after artifact timing edit before final claim.

### 2. Travel and recursive focus

- [x] Click or keyboard focus moves the camera toward the selected object's world position. The object becomes the local center; prior world recedes; Core is a small home anchor. Back/home restores orientation. Evidence: 11 journey tests, `/tmp/godspeed-northstar-current-focus.png`, `42-camera-return-home-settled.png`, and architect browser journey.
- [x] Focusing a child repeats the same transition at the next depth without route navigation. Evidence: recursive focus journey tests and architect-inspected focused hierarchy; no router exists in the renderer.

### 3. Semantic resolution and layout

- [ ] System, local, and detail bands reveal different object structure, notes, relationships, and affordances, beyond geometric scale. Evidence: same object at each band and core tests.
- [x] Dynamic contextual graph and object/time comparison rearrange the same world without permanent graph clutter. Evidence: causal `/tmp/godspeed-final-causal-comparison.png`; clean temporal `/tmp/godspeed-time-comparison-clean.png`; comparison layout/artifact tests. Temporal comparison is capped to four ranked objects per side and suppresses ordinary artifact cards.

### 4. Frozen projection and action boundary

- [x] Consume the supplied rich mock through a typed UI adapter and exercise role-aware objects/actions, governed work, subscribed execution update, evidence, history, artifacts, and settlement distinctions. Evidence: canonical-mock adaptation tests (12 focused pass), frozen package conformance (9 pass), and architect browser journey from Ready → In Progress → Done & Proven → report.
- [x] Display only actions supplied by the projection; submit consequential intents through the application adapter; update consequence only from a later projection. Evidence: accepted/refused/stale/role tests, visible fixture provenance, `/tmp/godspeed-action-awaiting.png`, and post-completion artifact timing tests.

### 5. Time and artifacts

- [x] Historical position changes the world snapshot; previous/next, comparison, and return-to-now work. Historical mutation remains disabled. Evidence: `/tmp/godspeed-final-history2.png`, `/tmp/godspeed-time-comparison-clean.png`, temporal and action-boundary tests.
- [x] An artifact emerges from its scene object as a bounded excerpt, expands to readable DOM, and closes back to the exact prior camera/world context. Evidence: `/tmp/godspeed-artifact-panel-fixed-expanded.png`, `/tmp/godspeed-artifact-close-restored.png`, browser DOM/no-skeleton check, loader regression and renderer-family tests.
- [x] Heavy renderers load only when needed. Evidence: production chunks plus registry tests resolving all lazy renderer families.

### 6. Agent choreography

- [x] CopilotKit maps 11 semantic commands through `CognitiveEnvironment`; grammar has no coordinates or direct Three mutation. Evidence: parser/forwarding tests, default build excludes CopilotKit, configured build emits its lazy chunk. No live Copilot endpoint was available, so live service demonstration remains an external integration limitation.
- [x] A causal query and object comparison visibly rearrange the scene. Narration is interruptible and resumes from a saved semantic beat. Evidence: `/tmp/godspeed-final-causal-comparison.png`, `/tmp/godspeed-final-history-paused.png`, pause/resume tests. Temporal comparison visual acceptance remains separately open because its first browser render was cluttered.

### 7. Integration, polish, and acceptance

- [ ] Real Go adapter where available provides live snapshots/events and typed refusals. A fixture-backed Go demo is labeled as fixture and cannot count as SEA-Forge/Gauntlet authority proof. Evidence: boundary/transport tests and provenance in the UI.
- [ ] Check keyboard paths, screen-reader outline, contrast, reduced motion, narrow layout, frame cost, and browser errors. Keyboard/outline, contrast, reduced-motion behavior, 320/390 layouts and zero browser errors are observed; a formal frame-cost profile remains unrecorded.
- [ ] Architect personally performs the 24-point Northstar journey in the user's request and records each observed result, failure, and correction. No agent self-assessment substitutes for this.
- [ ] Run `just casework-ui-check`, relevant Go/contract gates, `just context-check`, inspect final diff, and `graft build`. Record any unavailable integration gate as pending, never green.
- [ ] Update `.agents/CURRENT_STATUS.md` and `.agents/current_status.yml` with accomplished slices, evidence, blockers, and next move. Keep existing settlement state honest.

## Current next move

The donor-based 3D Core and celestial bodies have architect-inspected rendered evidence. Rich Casework opening/focused spacing and role action pending feedback now pass focused tests and browser inspection. A rich-mode causal question still routes into the unrelated Northstar scenario; Terra owns that narrowed agent-adapter seam. The latest full `just casework-ui-check` passed 84 tests before spacing/pending-action edits; rerun after the current agent edit. Then personally finish the 24-point reference journey, record observed gaps, update handoff state, and run `just context-check`. Preserve all dirty/untracked work.
