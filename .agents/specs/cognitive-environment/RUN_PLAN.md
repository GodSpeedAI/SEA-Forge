# Cognitive Environment: Unattended Build Run Plan

Owner: main agent (Opus), orchestrating. The operator is away. When the operator says "continue", execute this plan without asking questions.
Spec: `GodSpeed_Cognitive_Environment_DESIGN.md` (§0.1 + Appendix A are binding). Mocks: `mocks/*.png` (1672×941).
App: `apps/godspeed-cognitive-ui` (placeholder `src/main.tsx`; React 19 + Vite + TS + three only). Dev server: `bun run dev` → http://127.0.0.1:4178.

## Operating rules

- **Subagents are always `model: "haiku"`.** Never Sonnet or Opus. Omitting the model inherits Opus, so always pass it explicitly.
- **Haiku**: recon, focused module builds from precise specs (one module or component per agent, with exact file paths, the interface, and acceptance tests), screenshot capture, and first-pass visual diff notes.
- **Main agent**: architecture, the shared types and interfaces (write these *first*, yourself, so Haiku agents build against fixed contracts), integration and wiring, visual judgment against the mocks, and final verification. Spot-check every Haiku report by reading the code or screenshot that matters.
- Run independent Haiku builds in parallel (≤4 at a time). Keep prompts self-contained and point them to §0.1 of the spec.
- **Do not commit, push, stash, reset or clean.** Leave all changes in the working tree for the operator. Never touch files outside `apps/godspeed-cognitive-ui` and `.agents/specs/cognitive-environment` unless strictly required.
- Never restore or read the old host backup tarball. Do not port `gargantua.html`.
- Add npm dependencies only when a milestone requires them, and record why in the Progress Log.
- Keep `bun run typecheck`, `bun run build` and `bun test` green at the end of every milestone. Tests belong next to modules (`*.test.ts`) for pure logic: camera math, layout, LOD thresholds, beat player, artifact snapshot/restore, and time-slice derivation.

## Verification loop (every milestone)

1. Start the dev server (background) if it isn't running.
2. Capture screenshots at 1672×941 with `agent-browser` (run `agent-browser skills get core --full` once for its usage). Also drive the interactions: click, wheel, type in the composer, Esc.
3. Main agent reads the screenshot and the matching mock side by side, and judges composition, proportion, density, typography, Core look, spacing, light and dark mode, and motion (capture mid-transition frames for motion).
4. List the concrete deltas, fix them (delegate the localized fixes), and re-capture. Repeat until the only deltas left are minor.
5. Save final screenshots to `.agents/specs/cognitive-environment/verification/<milestone>/` and log them.

The fidelity bar is that a side-by-side with the mock reads as the same product: same Core look and placement, same sparse white field, same label typography and weight, same composer pill, same orbit lines. The exact sample data may differ.

## Milestones (see spec §0.1.6 and later)

- **M0 Foundation (main agent).**
  - Recon `reference/northstar.world.json` and `contracts/*.ts` (Haiku) to learn the object kinds, parents and time revisions.
  - Write `src/model/types.ts` (WorldState, WorldObject with LOD representations, Camera, Surface, Artifact, Beat, Narrative, Action union), `src/model/store.ts` (a tiny external store plus the `dispatch(action)` reducer signature), and the module layout below.
- **M1 Scene model:**
  - Home matches mock 01 and mock 13.
  - Free pan, zoom and restrained tilt work, with LOD thresholds and hysteresis.
  - Focus flies in and matches mock 05 (dark: mock 06). Core shrinks to its anchor. Back and Home work.
  - Pointer movement wakes the idle world. Relationships reveal on hover.
- **M2 Beats + artifacts:**
  - "Why did the pilot fail?" plays a scripted fixture narrative (no LLM) of 3–4 beats as in Appendix A §5.
  - The layout rearranges into the causal layout (mock 09) with labeled arrows.
  - An excerpt artifact appears and expands into the right dock (mocks 02, 03 and 07). Closing it restores the exact snapshot.
  - Clicking pauses the narrative and "continue" resumes it.
- **M3 Time:** scrubbing on the time strip rewinds (mock 08), with identity-preserving transitions and read-only navigation in the past. "Return to now" works.
- **M4 Judgment + execution:**
  - The human-judgment panel appears only when an action requires authority (mock 10).
  - Delegated work recedes, with inspection on demand (mock 11).
- **M5 Multi-artifact beat** (mock 04): a small chart, a table and a doc excerpt appear together. Add chart and table deps here only if needed, and lazy-load them.
- **M6 Case design mode** (mock 12): a dense workbench mode entered deliberately, which never leaks into Home.
- **M7 Go wiring:** a fixture-provider interface. When `apps/godspeed-casework-go` is running (`just casework-demo-up` or the Go README), load the world, time and artifacts from `/api/*` and SSE, falling back to the local fixture with an honest label. Intents go through `POST /api/intents`.
- **M8 Polish + a11y:** keyboard map (§25), outline/list view, reduced motion, dark-mode pass, and a final full visual sweep of all 13 mocks.

Suggested layout:

```
src/model/     types, store, reducer, fixture loader, time slicing
src/camera/    camera state, input (pan/wheel/pinch/tilt), flyTo tweens
src/layout/    orbital, causal, compare, temporal, judgment (pure)
src/scene/     CoreCanvas (three, one context), WorldLayer (DOM transform), ObjectNode (LOD), Relations (SVG)
src/narrative/ beat player, checkpoints, fixture narratives
src/ui/        Composer, Caption, CoreAnchor, ArtifactExcerpt, ArtifactDock, TimeStrip, JudgmentPanel, ModeShell
src/adapters/  fixture provider, go provider (M7)
```

## Stop conditions

- Stop when M1–M8 are verified, or when M1–M3 are verified at high fidelity and the remaining work is blocked (record the blocker).
- If blocked, record it and move to the next milestone that isn't blocked. Never stop to ask the operator.
- Finish with a short report in the Progress Log: what works, the screenshot paths, known gaps, and how to run.

## Progress Log

- 2026-09-22: Plan written. Old host deleted, spec moved here. Next step: M0.
- M0 done. `src/model/{types,store,fixture,artifacts}.ts`, `src/camera/camera.ts`, `src/scene/lod.ts`, `src/narrative/{player,narratives}.ts` with tests. Deps added: `@fontsource/inter` and `@fontsource/ibm-plex-mono` (bundled fonts, no network). Nothing else.
- M1 done (verified against mocks 01, 05, 13, 06):
  - Layout is authored in mock pixels with nested 10× frames, so free zoom and focus agree.
  - The Core is one Three canvas. Face-on it uses a streak shader tuned by radial color profile against mock 01. Edge-on (focused case) it uses a small Newtonian ray-march for the lensed look.
  - Dark Home renders categories as luminous spheres (mock 13).
  - Idle Home stays pristine; pointer movement wakes chrome, residue and attention satellites.
  - Click-to-focus flight, Escape/back, Home anchor and wheel zoom with LOD hysteresis all work. Camera restore after artifact collapse is exact (checked numerically).
  - Headless Chrome here uses software GL (~1 fps at full resolution), so the canvas has dynamic resolution. `?quality=high` pins full resolution for screenshots.
  - Deep links for verification: `?focus=`, `surface=`, `time=`, `awake=1`, `theme=dark`, `play=`, `expand=`.
- M3 done (mock 08): time strip (scrub, arrows, `[`/`]`, return to now); the past is read-only; "Earlier state" and the revision summary show under the center; ghost and dormant objects.
- M4 done (mocks 10 and 11):
  - Judgment surface: the center shifts left, the decision comes forward with a role pill, and the context fades but stays legible. A square decision panel stays open (pending) until the backend answers, then shows accepted or refused.
  - **Safety fix:** the panel no longer autofocuses Approve. The Enter that submitted "approve the release" used to approve it.
  - After approval, the fixture live adapter appends revisions as the rollout executes. The quiet execution pill shows "Executed · awaiting settlement" before "Settled". Deep inspection (mock 11) is a workbench mode with rails; the world keeps rendering in the middle.
- M5 done (mock 04): the evidence narrative materializes chart, table and doc excerpts at once. Cards take open zones when there are 2 or more.
- M6 done (mock 12): case design is a workbench mode over a separate template world (it never leaks into runtime), with glyph nodes, an in-void title, and the design panel. Save Draft is a governed intent.
- M7 done:
  - The UI loads the Go world (`/api/time` plus `/api/world?cursor=`) when `/api/healthz` answers. Otherwise it uses the fixture, labeled "Fixture". `?source=fixture` forces the fixture (the mocks' dataset); verification screenshots use it.
  - SSE `revision` events append live revisions. Intents POST to `/api/intents`; tested round-trip: Go refused an approval with its own reason, and the panel shows it. Vite proxies `/api` to 4179.
  - Fixture narratives resolve through `model/aliases.ts` against Go ids.
- M2 mostly done: the "why did the pilot fail" narrative plays 4 beats (focus, dim, reveal, record excerpt, causal rearrangement with labeled arrows and role pills, failure revealed last). Pause on input; "continue" resumes. The dock (mock 07) shows the diff; the world compresses left; collapse restores exactly.
- M8 done: keyboard map per §25 (Esc, h, `/` and Ctrl/Cmd+K, Tab plus Enter/Space, arrows, +/−, t, `[`/`]`, o, d). Outline view (`o`) is the non-spatial equivalent. Captions, decisions and past state are announced through an aria-live region. Reduced motion makes the camera arrive instead of fly.
  - Core fidelity pass. Face-on: a two-arm log-spiral whirlpool with long tangential streaks and particles on the arms; radial color profile within ~10 luminance of mock 01. Edge-on: a symmetric lensed disk with a photon rim. Dark: soft luminous blue filaments.
  - The smoke test of the main flows produced no runtime errors. `just casework-ui-check` is green (78 tests).

## Final report (2026-09-22)

**Works.** Home (idle as in mock 01, wakes on pointer movement), free pan/zoom/tilt with semantic LOD, click-to-focus flights, back/Home, beats (why/evidence/history) with pause and "continue", excerpt → dock → exact restore, time travel (read-only past), judgment with backend outcome, simulated execution and a separate settlement, execution-inspect and case-design workbench modes, Go wiring (world, time, SSE, intents), dark mode, and accessibility basics.

**Screenshots.** `verification/final/*.png` (ours) and `verification/compare/*.png` (mock left, ours right) for all 13 mocks. The mock 02/03 comparisons are analogs: those mocks show a different, config-diff world, so the Northstar equivalents were captured instead.

**Known gaps.**
- The Core shader is close but not photographic: fewer glints and less bloom than the mocks, and dark filaments are smoother.
- Go artifact bodies are not fetched. Excerpt and dock renderers use fixture artifacts whose refs match Go's (art-diff-491, art-checks-table). Pinning posts `/api/artifacts`.
- Fixture narratives are scripted; they resolve against Go ids through aliases, but no LLM is wired.
- The JS bundle is 1.16 MB (three.js is not split out).
- Motion was verified numerically, not visually: headless Chrome here uses software GL at ~1 fps.
- The judgment composer is not squared as in mock 10.

**How to run.** `just casework-ui-up` → http://127.0.0.1:4178. The UI auto-uses the Go service on 4179 if it is running; `?source=fixture` shows the mocks' dataset. See `apps/godspeed-cognitive-ui/README.md`.
