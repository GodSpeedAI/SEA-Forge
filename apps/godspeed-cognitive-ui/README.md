# @godspeed/cognitive-ui

The GodSpeed cognitive environment: a persistent 2.5D world with one camera, over the Go
casework service (`apps/godspeed-casework-go`). Spec:
`.agents/specs/cognitive-environment/GodSpeed_Cognitive_Environment_DESIGN.md` (§0.1 + Appendix A).
Build log and verification: `.agents/specs/cognitive-environment/RUN_PLAN.md` and `verification/`.

## Run

Two sources behind the one contract port (`src/ports/contract.ts`), chosen in `src/main.tsx`:

* **Live (the production path).** `bun run build` always selects the live source: the Go gateway
  (`apps/godspeed-casework-go`) over same-origin HTTP+SSE with a gateway session (login screen when
  unauthenticated), real governed intents, and Thoth Ask narration. The local adapter is behind a dynamic
  import so it is tree-shaken out of production assets (the bundle check in `e2e/live/bundle.ts` scans the built assets). The status
  bar says **Live gateway** with the origin. The gateway serves the build when its config sets
  `serve.static_root` to `apps/godspeed-cognitive-ui/dist`.
* **Local (dev and tests only).** A dev server (`import.meta.env.DEV`) defaults to the local contract-conformant
  adapter (`src/adapters/local/`, Northstar data, scripted local agent); the status bar says **Local contract
  adapter (not the Go service)**. Set `VITE_CASEWORK_SOURCE=live` on the dev server to use the gateway instead.

```
just casework-ui-up          # dev server at http://127.0.0.1:4178 (vite proxies /api to 127.0.0.1:4179)
just casework-ui-check       # frozen install, typecheck, build, tests
```

To run the live stack locally (kernel, gateway, cell) use the recipes in
`apps/godspeed-casework-go/README.md` ("Run it locally (live)"): `just casework-cell-init`,
`casework-server-up`, `casework-live-go-up`, then `VITE_CASEWORK_SOURCE=live` for the dev server or a
`dist` build served by the gateway. `configs/live-serve.json` uses dev auth (any password for `operator`
or `rso`); production uses `local` or `oidc` auth. See the Go README for auth modes and the production bind rule.

- **Try (local source):** move the pointer (the world wakes), click Projects then Northstar, click an object's
  "▤ artifacts" pill and a card to open the right-hand viewer, use the center chips (Causal view, History,
  Compare with earlier, Design case), ask "Why did the pilot fail?", hover Release and choose "Approve release →",
  or type "let the agent approve the release" to watch authority refuse the agent.
- **Artifacts:** the nine source renderers (diff, text, markdown, table, chart, json, graph, trace, timeline)
  are separate lazy chunks (`src/artifacts/registry.tsx`); the build fails if they are not nine distinct
  chunks (`rendererChunkContractPlugin` in `vite.config.ts`).
- **Switches (local source, tests and recovery):** `?speed=N`, `?beatPace=N`, `?failArtifact=<ref>`,
  `?corruptArtifact=<ref>`, `?failRenderer=<kind>`, `?agent=off`, `?agentFailAfter=N`.

## E2E ladders (agent-browser)

* **Local ladder:** `bun run e2e` (dev server up via `just casework-ui-up`) runs J0-J9 plus RECOVERY through
  the real rendered UI against the local adapter. Evidence:
  `.agents/evidence/godspeed-casework-cognitive-environment/ui-journeys/latest`. See `e2e/README.md` and
  `.agents/reports/godspeed-cognitive-ui-functional/02-JOURNEY-CUBE.md`.
* **Live ladder:** `just casework-e2e-live` (= `bun e2e/run.ts --live`; needs bun, agent-browser, a built kernel,
  go, and ports 4179/4180 free). The harness owns the stack: fresh temp cell, production UI build plus bundle
  scan, kernel and gateway serving `dist`, then journeys L0 (readiness and identity), L1 (template to commit),
  L2 (horizon standing), L3 (discretionary add), L4 (execute), L5 (sign-off: operator denied, R-SO approves in a
  separate session), L6 (settlement and evidence dock), L7 (lifecycle), L8 (Thoth ask narration), L9 (L0-L8 in one
  session with a clean console) and L-RECOV (corrupt artifact, SSE drop, kernel and gateway `kill -9`). Each
  journey asserts the UI and the durable ledger delta. Flags pass through, e.g. `--only L0,L1`. Teeth (negative
  self-checks): `--tooth stub-gateway|shared-session|shared-cookie|console-error`. Evidence:
  `.agents/evidence/casework-live-wiring/T10/run-<timestamp>/` (`latest` symlink). The harness uses dev auth.
* Load test and metrics: `just casework-load`, see the Go README.

Live limitations: no kernel resume after an approval (CW-45), so L7 continues the lifecycle by hand; no
`execution_progress` frames, so progress is never shown; after a gateway restart the page needs a reload and
artifact errors are cached per ref until reload (CW-46).

## Keyboard

| Key | Action |
|---|---|
| Esc | back one level (closes dock/judgment/workbench first) |
| h / Home | Core (Home) |
| `/` or Ctrl/Cmd+K | composer |
| Tab, then Enter or Space | focus an object |
| arrows, + / − | pan, zoom |
| Shift/right-drag | restrained orbit (tilt) |
| t, `[` `]` | time strip, step back/forward (the past is read-only) |
| o | outline (non-spatial list of what's on screen) |
| d | dark / light |
| Space | pause / resume the current explanation ("continue" also resumes) |

`prefers-reduced-motion` makes the camera arrive instead of fly and slows the Core.

## Layout of the code

```
src/ports/     contract port (types from .agents/reports/interface-contracts) + projection to the internal model
src/adapters/  http (live gateway adapter) and local (dev/test contract-conformant adapter) + conformance tests
src/model/     internal model, reducer, world view (live / historical / compare / design)
src/layout/    pure, data-driven layout: orbital, causal, judgment, design, comparison annotations
src/camera/    projection, pan/zoom/tilt, flights
src/scene/     runtime (per-frame camera, tweens, LOD, SVG), CoreCanvas (the only WebGL), ObjectNode (LOD reps)
src/artifacts/ payload parsing, artifact service (resolveArtifact), lazy source renderers + error boundary
src/narrative/ narration port conductor, beat player (pause/checkpoint/resume), scripted local agent
src/ui/        composer, chrome, excerpt, dock, time strip, compare bar, judgment, execution, workbench, design, outline
src/app/       App shell, composer commands, intent path (human = agent path), live events, case design helpers
e2e/           affordance-dependency ladders (agent-browser, real pointer input): journeys/ local, journeys-live/ + live/ live
```

Deep links (verification only; all reachable through the UI): `?focus=`, `surface=causal`, `time=<cursor>`,
`timeline=1`, `awake=1`, `theme=dark`, `judge=<object>`.

## Subdirectories & Boundaries

- src (source tree): [./src/README.md] - Boot/source selection, contract port, internal model, scene, artifacts, narrative, UI.
- src/ports: [./src/ports/README.md] - The contract port (`CaseworkPort`, `SessionPort`, `TemplateSourcePort`, `NarrationPort`) and the contract→internal projection.
- src/adapters: [./src/adapters/README.md] - Live HTTP adapter, local contract adapter, shared port conformance suite.
- src/app: [./src/app/README.md] - App shell, the one intent path (VAR-007), composer commands, live events, proposal flow.
- src/artifacts: [./src/artifacts/README.md] - Payload parsing, artifact service, lazy renderer registry; renderers in [./src/artifacts/renderers/README.md].
- src/model: [./src/model/README.md] - Internal state, reducer/store, derivation, view selection.
- src/layout: [./src/layout/README.md] - Pure, data-driven surface layouts.
- src/camera: [./src/camera/README.md] - Pure projection, pan/zoom/tilt, flights.
- src/scene: [./src/scene/README.md] - Per-frame runtime, CoreCanvas (the only WebGL), ObjectNode, semantic LOD.
- src/narrative: [./src/narrative/README.md] - Narration conductor, beat player, scripted local agent, Thoth Ask narration.
- src/ui: [./src/ui/README.md] - React panels and chrome.

Other in-repo assets (not README-linked here): `e2e/` (journey ladders, see `e2e/README.md`), `reference/WIRE.md`,
`t08c/` (live-stack fixture config), `vite.config.ts` (dev/preview server on 127.0.0.1:4178 with `/api` proxied to
127.0.0.1:4179 and `rendererChunkContractPlugin`).

## Interface

- `App` — the shell; composes scene, panels, composer, narrative player and live events [src/app/App.tsx:App]
- `CaseworkPort` (+ `SessionPort`, `TemplateSourcePort`, `NarrationPort`) — the one contract boundary; shapes come
  verbatim from `.agents/reports/interface-contracts/typescript/types`, spec-04-only shapes declared here
  [src/ports/contract.ts:CaseworkPort]
- `HttpCaseworkAdapter` — the live gateway source (fetch + SSE, session identity, typed refusals)
  [src/adapters/http/httpCaseworkAdapter.ts:HttpCaseworkAdapter]
- `LocalContractAdapter` — the dev/test contract-conformant fixture source
  [src/adapters/local/localAdapter.ts:LocalContractAdapter]
- `runCaseworkPortConformance` — the shared behavioral suite both adapters must pass
  [src/adapters/conformance/caseworkPortConformance.ts:runCaseworkPortConformance]
- `createStore` / `reduce` — the internal store and reducer every surface reads [src/model/store.ts:createStore, src/model/store.ts:reduce]
- `projectSnapshot` / `projectHistory` — contract→internal projection, the only place contract shapes are read
  [src/ports/project.ts:projectSnapshot, src/ports/project.ts:projectHistory]
- `createIntentPath` — the one path consequential work takes out of the UI, human or agent (VAR-007)
  [src/app/intents.ts:createIntentPath]
- `createArtifactService` / `parseArtifact` — per-ref artifact resolution and payload parsing
  [src/artifacts/service.ts:createArtifactService, src/artifacts/model.ts:parseArtifact]
- `assertRendererChunks` — build-time nine-renderer chunk contract [src/build/rendererChunkContract.ts:assertRendererChunks]

## Invariants

- Everything crosses the port in contract shapes; replacing the local adapter with the Go adapter must not change
  anything above `src/ports/contract.ts` [src/ports/contract.ts:CaseworkPort]
- The UI never decides authority: refusals surface as the backend phrased them, and world changes arrive only as new
  snapshots [src/app/intents.ts:createIntentPath]
- Snapshots append history and never rewrite it; the past is read-only [src/app/live.ts:connectLive, src/model/store.ts:isPast]
- The client never asserts identity; identity comes from the gateway session or the local fixture's self-report
  [src/adapters/http/httpCaseworkAdapter.ts:HttpCaseworkAdapter.getSnapshot, src/adapters/local/localAdapter.ts:LOCAL_IDENTITY]
- Production builds always select the live source; the local adapter is behind a dynamic import and must never ship in
  production assets [src/main.tsx]
- The nine source renderers are nine distinct lazy chunks; the build fails otherwise (VAR-006)
  [src/build/rendererChunkContract.ts:assertRendererChunks]
- Execution progress is shown only when the stream reported it; an honest pill shows the phase, not a guessed number
  [src/app/live.ts:connectLive]
- Narration directives use only the UI's declared vocabulary; grounded Thoth beats never invent claims, citations or
  directives [src/narrative/conduct.ts:beatFromNarration, src/narrative/thothNarration.ts:beatsOfAnswer]

## Side Effects

- HTTP + SSE to the Go gateway (`/api/world`, `/api/intents`, `/api/events`, `/api/artifacts/{digest}`, `/api/trajectory`,
  `/api/templates`, `/api/ask`, `/api/auth/*`, `/api/session`) with double-submit CSRF
  [src/adapters/http/httpCaseworkAdapter.ts:HttpCaseworkAdapter]
- One WebGL context (CoreCanvas) plus per-frame DOM/SVG writes from the scene runtime
  [src/scene/coreCanvas.ts:CoreCanvas, src/scene/runtime.ts:SceneRuntime]
- Timers for simulated local execution, beat playback, outage grace and recovery probes
  [src/adapters/local/localAdapter.ts:LocalContractAdapter.execute, src/narrative/player.ts:startNarrativePlayer, src/app/live.ts:connectLive]

## Verification

- Run: `bun test` (working directory: `apps/godspeed-cognitive-ui`)
- Full app gate: `just casework-ui-check` (frozen install, typecheck, build incl. the renderer chunk contract, tests);
  E2E ladders: `bun run e2e` (local) and `just casework-e2e-live` (live).
