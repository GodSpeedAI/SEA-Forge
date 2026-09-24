# @godspeed/cognitive-ui

The GodSpeed cognitive environment: a persistent 2.5D world with one camera, over the Go
casework service (`apps/godspeed-casework-go`). Spec:
`.agents/specs/cognitive-environment/GodSpeed_Cognitive_Environment_DESIGN.md` (§0.1 + Appendix A).
Build log and verification: `.agents/specs/cognitive-environment/RUN_PLAN.md` and `verification/`.

## Run

```
just casework-ui-up          # dev server at http://127.0.0.1:4178
just casework-ui-check       # frozen install, typecheck, build, tests
```

- **World source.** The UI talks only to the contract port (`src/ports/contract.ts`), which this phase serves from a
  local, contract-conformant adapter (`src/adapters/local/`). The status bar says **Local contract adapter**.
  The Go system front end is intentionally not connected yet. See
  `.agents/reports/godspeed-cognitive-ui-functional/03-CONTRACT-PORTS-AND-SEAMS.md`.
- **Try:** move the pointer (the world wakes), click Projects then Northstar, click an object's "▤ artifacts" pill
  and a card to open the right-hand viewer, use the center chips (Causal view, History, Compare with earlier,
  Design case), ask "Why did the pilot fail?", hover Release and choose "Approve release →", or type
  "let the agent approve the release" to watch authority refuse the agent.
- **E2E ladder:** `bun run e2e` (dev server must be up) runs J0→J9 plus RECOVERY through the real rendered UI; see
  `e2e/README.md` and `.agents/reports/godspeed-cognitive-ui-functional/02-JOURNEY-CUBE.md`.
- **Switches (tests and recovery):** `?speed=N`, `?beatPace=N`, `?failArtifact=<ref>`, `?corruptArtifact=<ref>`,
  `?failRenderer=<kind>`, `?agent=off`, `?agentFailAfter=N`.

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
src/adapters/  local contract-conformant adapter: contract data at rest, authority, execution events
src/model/     internal model, reducer, world view (live / historical / compare / design)
src/layout/    pure, data-driven layout: orbital, causal, judgment, design, comparison annotations
src/camera/    projection, pan/zoom/tilt, flights
src/scene/     runtime (per-frame camera, tweens, LOD, SVG), CoreCanvas (the only WebGL), ObjectNode (LOD reps)
src/artifacts/ payload parsing, artifact service (resolveArtifact), lazy source renderers + error boundary
src/narrative/ narration port conductor, beat player (pause/checkpoint/resume), scripted local agent
src/ui/        composer, chrome, excerpt, dock, time strip, compare bar, judgment, execution, workbench, design, outline
src/app/       App shell, composer commands, intent path (human = agent path), live events, case design helpers
e2e/           affordance-dependency ladder (agent-browser, real pointer input)
```

Deep links (verification only; all reachable through the UI): `?focus=`, `surface=causal`, `time=<cursor>`,
`timeline=1`, `awake=1`, `theme=dark`, `judge=<object>`.
