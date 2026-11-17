# src

## Overview
The source tree of the GodSpeed cognitive environment UI (React 19 + Vite + three.js, Bun toolchain). Architecture, top-down: `main.tsx` selects one of two sources behind the single contract port (live gateway for production builds; the local contract-conformant adapter via dynamic import for dev/test), and everything above the port sees only the internal projection. `src/build/rendererChunkContract.ts` enforces the nine-lazy-renderer chunk contract at build time (its Vite plugin lives in `vite.config.ts`).

## Subdirectories & Boundaries
- Ports (contract boundary): [./ports/README.md] - The port types (CaseworkPort, SessionPort, TemplateSourcePort, NarrationPort) and the contract→internal projection; the only place contract shapes are read.
- Adapters: [./adapters/README.md] - The live HTTP adapter (Go gateway over fetch+SSE), the local contract-conformant dev/test fixture, and the shared conformance suite.
- App: [./app/README.md] - The `App` shell, the single intent path (VAR-007), composer command routing, live event wiring, and the case-design/proposal flow.
- Artifacts: [./artifacts/README.md] - Payload parsing, the per-ref artifact service, and the lazy renderer registry (nine distinct chunks).
- Renderers: [./artifacts/renderers/README.md] - The nine source-level artifact renderers with search highlighting and graph layout.
- Model: [./model/README.md] - Internal state types, reducer/store, presentation derivation, and view selection (live / historical / compare / design).
- Layout: [./layout/README.md] - Pure, data-driven placements for the orbital, causal, judgment, design and comparison surfaces.
- Camera: [./camera/README.md] - Pure projection, pan/zoom/tilt, and camera flight math.
- Scene: [./scene/README.md] - The per-frame runtime, one WebGL canvas (CoreCanvas), SVG link layer, DOM object layer and semantic LOD.
- Narrative: [./narrative/README.md] - Narration conductor, beat player (pause/checkpoint/resume), the scripted local agent, and governed Thoth Ask narration.
- UI: [./ui/README.md] - The React panels and chrome (composer, judgment, dock, time strip, login, workbench, design panels).

Directly in this directory:
- `main.tsx` — boot: source selection (`import.meta.env.DEV` + `VITE_CASEWORK_SOURCE`), local test/recovery switches (`?speed=N`, `?failArtifact=<ref>`, `?corruptArtifact=<ref>`, `?agent=off`, `?agentFailAfter=N`), session resolution and login screen [main.tsx]
- `build/rendererChunkContract.ts` — `assertRendererChunks(bundle)`: the build fails unless each of the nine renderer modules is emitted as exactly one distinct dynamic-entry chunk imported by the registry chunk [build/rendererChunkContract.ts:assertRendererChunks]
- `vite-env.d.ts` — Vite client types.
