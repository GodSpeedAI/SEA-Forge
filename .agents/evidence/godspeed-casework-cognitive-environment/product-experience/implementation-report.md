# Product-experience implementation run (2026-09-19)

Operator-ordered takeover: "finish the product experience" of the GodSpeed casework cognitive
environment, with the operator's visual correction and black-hole Core reference images applied
mid-run. This report records what was built, what was verified, and what remains fixture. It is an
implementation report, NOT a task settlement: plan tasks whose substance now exists (T04, T06, T07,
T08, T09, T10) remain formally unsettled because the plan's preregistration/teeth/confirmation
process was not executed for them in this run.

## What was built

**UI core extensions (T06/T07/T08/T09 semantics, `apps/godspeed-cognitive-ui/src/core`)**
- Semantic zoom as a representational rule: `visibleObjects(state)` resolves the visible SET from
  (surface, focus, zoom band) plus narration overlays; grandchildren carrying attention surface at
  local scale and recede when the projection says settled. Wheel + double-click map to `setZoom`.
- Temporal world switching: `stepTime`/`setTimePosition`/`returnToNow` drive `snapshotAt(cursor)`
  through the provider; the world becomes its earlier self (objects that did not exist yet
  disappear); live SSE pushes are held aside while historical; two-position comparison
  (`setTimeComparison`) renders both cached projections side by side with ghost markers for
  objects absent on one side.
- Choreography: `NarrationBeat.directives` (focus, camera, emphasize/deEmphasize, reveal/conceal,
  contextual relationships, annotations, artifact materialization, comparison, temporal steps)
  applied through the same action vocabulary a human uses; pause/resume holds position;
  interruption preserves everything said; a new narration starts from a quiet world (unpinned
  artifacts dematerialize, overlays clear).
- Artifact lifecycle: materialize-at-minimal, advance minimal/summary/source, close; pinning is a
  `persist-artifact` intent that crosses the interaction boundary or does not happen.
- Contracts extended additively (T01 shapes preserved): `WorldObject.parentId/note/attention`,
  `WorldSnapshot.provenance`, `WorldAdapter.snapshotAt?`, `ZoomLevel`, `NarrationDirective`,
  richer `UiContextSnapshot`, `IntentKind += persist-artifact`, `ArtifactAdapter.read(ref, level?)`.
  All T03 teeth (purity, provider-swap scenario, adapter replacement, narration interruption) run
  unchanged and green.

**Spatial host (`src/host/scene`, R3F/three)**
- Core: a billboarded raymarched gravitational lens (photon geodesic bending, event-horizon
  capture, procedural lensed starfield, warm/cool photon-ring glow with faint doppler asymmetry,
  slow sky rotation for motion in the lensing), dissolving into the page field; plus a sparse
  scene-wide star-dust shell. Restrained on the light field, instrument-panel in dark mode.
- Orbital arrangement: attention as visual physics (consequence pulls inward and larger; quiet
  recedes); even fan spacing for requires-judgment satellites at system scale; stable identity
  across rearrangements (nodes ease, never teleport); camera travels through the same world for
  focus; contextual thin relationship lines only where they help the current thought; comparison
  as a rearrangement with a divider hairline.

**Artifact runtime (`src/host/artifacts`)**: cards anchored to their bound object's projected
screen point; Open expands spatially toward the foreground with the world receding behind a veil;
Close contracts back toward the origin; level stepper; Keep (pin) crosses the boundary. Renderer
families all lazy-loaded: quotation (with CodeJar excerpt editing at source), sanitized semantic
HTML document (#hit highlight + scroll), diff (react-diff-view), table (TanStack table/virtual;
react-data-grid toggle + SheetJS xlsx export at source), chart (restrained SVG bars; see decision
D-3), PDF (pdfjs with bundled worker), plain-text fallback.

**Composer + narration presentation (`src/host/composer`)**: bottom-center pill; suggestions;
context chips (focus, time, selection carried); narration as a single caption line with
Pause/Resume/Dismiss and beat dots; compact semantic residue ("Recap") on demand; no transcript.

**Controls/chrome**: temporal strip (scrub, prev/next, Now, two-tap Compare), ⌘K search palette,
outline panel (the list alternative to the spatial graph), quiet provenance mark with typed
refusal notices, keyboard legend, aria-live scene announcer, dark/light theme with persistence,
prefers-reduced-motion honored in DOM and scene.

**CopilotKit seam (`src/host/agent`)**: context registered (surface/focus/objects/time/artifacts)
and UI-owned actions (focus, open artifact, compare, move through time, propose consequence) —
lazily mounted only when a runtime URL is configured; absent otherwise by design.

**Go boundary (`apps/godspeed-casework-go`)**: `internal/projection` (embedded canonical fixture,
monotonic revision store, SSE replay-then-live), `internal/coordinator` (intent allowlist,
staleness by cursor, idempotency by id+body, fixture authority allowlist with zero-effects denial,
leases claimed/active/released, staged revisions 1151-1153: layer appears, checks run, world
quiets in ordinary language), `internal/artifactstore`, `internal/server` (healthz, world,
world?cursor, time, events SSE, intents, artifacts list/persist/get; strict JSON; 1 MiB cap;
loopback-only CORS for the colocated UI), `-serve` mode in cmd with graceful shutdown.
Canonical dataset `northstar.world.json` authored once in the UI tree and embedded in Go with a
byte-equality drift test.

**Operator recipes**: `casework-go-up/-down/-status`, `casework-demo-up` (justfile, casework
group).

## Verification (this run)

- GATE_UI (`just casework-ui-check`): PASS — frozen install, typecheck, build, 72 tests / 471
  assertions (core purity/scenario/narration/choreography/zoom, Northstar providers, scenario
  agent, renderers incl. sanitizer/decode/registry). Suite run 5x consecutively with 0 failures
  after hardening one timing-flaky T03 assertion (bounded wait instead of a fixed sleep).
- GATE_GO (`just casework-go-check`): PASS — gofmt clean, vet clean, all package tests green
  (incl. new CORS test; also verified under -race and -count=5 by the implementing agent).
- GATE_SPEC_TRACE (plan validator): PASS — spec hash, 86/86 requirements, DAG, gate activation.
- Production build: PASS — `vite build` green; heavy families ship as separate async chunks
  (verified: no shiki/xlsx/pdfjs/react-data-grid/diff markers in the entry chunk); `vite preview`
  served the built app and entry asset (HTTP 200).
- Live walkthrough (screenshots in `screenshots/`, agent-browser against the running Go boundary +
  dev server, provenance `go:fixture:northstar`): opening composition; focus-as-entering (search,
  camera, receded world); semantic zoom to detail (wheel + double-click); "What changed
  overnight" 4-beat choreography (emphasis physics, artifact materialization, contextual
  relationship); excerpt card expanding to the sanitized document panel and back; "Is this
  related" comparison arrangement with annotation; narrated history walk (world became cursor-980
  then 900, assumption resurfacing, artifact quotes) and return to now; "Show me exactly why it
  failed" causal surface with provider-contract artifact; diff artifact at source level; Keep
  crossing the boundary ("Kept"); dark mode; two full consequential runs ("Have the agent
  implement that") with SSE revisions 1151-1153 and 1154-1156: compatibility layer visible while
  progressing, world settled and quiet with every requires-judgment ring released and ordinary-
  language notes ("Ready after security review", "10/10 verified").
- Refusal path: with the boundary absent the same request is refused (`unavailable`) and the typed
  refusal notice shows "nothing changed"; no local conclusion.

## Honest limits (unchanged claims, restated)

- No SEA Forge, Gauntlet, or GitHub integration: the Go server's authority is a fixture allowlist
  labeled `go:fixture:northstar`; nothing here is evidence of governed integration (the plan's
  T05/T11 under GATE_SEAFORGE/GATE_GAUNTLET remain the governing path).
- Durability is process-lifetime (in-memory); leases clear on restart (RECOV-001 semantics need
  the real persistence path).
- CopilotKit: seam implemented, no runtime configured this milestone; the scripted scenario agent
  drives the same vocabulary.
- The scenario agent is a scripted fixture keyed to the Northstar dataset; it degrades honestly
  for unknown questions.
- Plan tasks T04/T06/T07/T08/T09/T10: implemented substance recorded here; formal settlement
  (preregistration, teeth execution per task, independent confirmation for P3 tasks) not
  performed in this run. T11's GATE_INTEGRATED recipe does not exist yet by design.

## Addendum: game-first reframe (2026-09-20)

The operator's follow-up reframe ("think of this as a video game; the first frame is Core,
composer, white space, nothing else") was applied as a presentation pass over the same
architecture:

- First frame: full-viewport white world; the raymarched Core exactly centered; the composer as a
  narrow search field bottom-center (widens only when focused); peripheral chrome (keys, theme,
  outline, provenance) faded to 0.42 opacity until approached; the temporal strip hidden entirely
  unless standing in history or comparing; composer context chips removed. Emptiness is the
  resting design.
- Core whisper: a greeting on wake ("Good morning"), then "1 thing needs you" as bare ivory type
  inside the event horizon, driven by the projection's attention set, fading with the world's
  wake. No card, no badge.
- Wake physics: one rAF loop (module-level, no React re-renders) writes a `--wake` custom
  property; pointer activity wakes labels, discs, peripheral chrome and Core's lens over ~2.6s of
  attention and lets them settle when idle. The world begins almost asleep.
- Click = travel: clicking a body (disc or label) moves the camera into it and makes it the local
  world; clicking the centered body steps deeper (semantic zoom); shift-click selects without
  traveling (for compare); double-click opens the bound artifact; Esc/Core-home unwinds one
  level. Arrow keys move attention without camera travel.
- Orbit: top-level bodies revolve almost imperceptibly around the current center from their
  authored angles (never a random re-layout); requires-judgment satellites fan beside their
  parent with the resting relationship hairline barely visible.
- Resting relationships: only attention-involving connections render at system scale, at very low
  opacity; knowledge-graph lines appear on focus, selection, hover or narration.

Verified live against the restarted Go boundary (fresh world at cursor 1150): asleep first frame;
awake frame with satellites and whisper; camera travel into Northstar (Core collapses to the home
mark, children resolve); Esc back out. Typecheck clean; 72/72 tests green; screenshots 24-31 in
`screenshots/`.
