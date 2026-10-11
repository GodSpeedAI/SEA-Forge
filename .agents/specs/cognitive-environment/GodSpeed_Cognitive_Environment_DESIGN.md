# GodSpeed Cognitive Environment — DESIGN.md

> Category: Governed Cognitive Environment / Casework Operating System  
> Status: Canonical visual and interaction grammar for OpenDesign prototyping  
> Scope: React cognitive environment over the GodSpeed System Front End, SEA-Forge case subsystem, Gauntlet execution, and RealityTrace history/evidence

---

## 0.1 READ FIRST: Implementation Brief (operator-confirmed 2026-09-22)

This section is the entry point for any agent building this UI. Read it, then Appendix A (the operator's own clarifications), then open the mock images in `mocks/` next to this file. You should not need to do repository recon to start. The pointers you need are below.

**Precedence:** §0.1 and Appendix A override the rest of this document on implementation. They also override the following older sources:

- the 2026-09-22 Gargantua/CORE addendum in `docs/decisions/ADR-006-godspeed-casework-ui-replacement.md`;
- the "R3F/Three required" and "raymarched Core" wording in `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml`;
- anything left over from the deleted host.

The rest of this document still governs visual grammar, semantic distinctions and copy.

### 0.1.1 What we are building, in one paragraph

GodSpeed is a **persistent 2.5D world with a camera**, not a set of pages. You manage governed work (cases, plan items, approvals, agent runs, evidence, settlement) by moving through that world:

- You pan and zoom freely.
- Clicking an object flies the camera into it.
- Crossing zoom thresholds changes the *representation* of what you see: semantic level of detail (LOD).
- AI answers arrive as **semantic beats**. A beat is a short caption plus a coordinated change to the scene: focus, rearrangement, relationships, time, and small **artifacts**.
- Artifacts start as tiny, incisive excerpts. They can expand into a real working pane docked on the right, then collapse back into the world exactly where you were.
- The world must *behave* like a scene. Nothing about it has to *be* WebGL.

### 0.1.2 Why earlier attempts failed (do not repeat)

1. **Everything went into React Three Fiber (R3F).** Text, labels, panels and interactive artifacts are hard in WebGL. Agents spent their effort on renderer plumbing, not on the experience.
2. **Gargantua (`.tmp/SEA-Forge-UI-mocks/gargantua.html`) was adopted as the Core.** It is a physically accurate Kerr black hole on a *dark* background, with raymarching and a 5-pass bloom pipeline. The mocks show a *stylized* bronze swirl on *white*. It also needed its own WebGL context composited under a second R3F canvas. `gargantua.html` is at most a mood reference. Do not port it.
3. **Borrowed frameworks pulled against the design.** CopilotKit is built around chat sidebars, and this design forbids chat. The app also pulled in d3-force, elkjs, PDF.js, SheetJS, mammoth, react-data-grid and Shiki before the home screen even worked.
4. **Process displaced the product.** Task gates, purity tests, byte-parity provenance and frozen contracts consumed the effort while Home still did not look like mock 01.
5. **Salvage.** Every new attempt built on the previous host, so its problems carried forward. **The old host was deleted on 2026-09-22. Do not restore it, copy from it, or rebuild its architecture.** (A backup tarball exists outside the repo. Do not use it.)

### 0.1.3 Rendering architecture (hybrid, one camera)

```
WorldState   objects by id, relationships, time cursor, active surface, focus stack,
             open artifacts, selection, active narrative/beat
    │
layout(world, surface, focus, time) → target {x, y, depth} per object id   (pure function)
    │
Camera { x, y, zoom, tilt }   written by: pan, wheel/pinch, focus flights, beats
    │                         (restrained 2.5D: bounded tilt/orbit, no free flight)
    ├── WebGL layer (one Three.js canvas, one context): Core visual, particle field,
    │     depth/parallax. It reads the same camera. Nothing else lives here.
    └── World DOM layer: one container with a single CSS transform derived from the camera
          ├── object nodes: each picks its LOD representation from the effective zoom
          ├── relationship lines (SVG in world space), revealed on hover, focus or beat
          └── small artifacts anchored beside their source object
Screen-space DOM: composer, artifact dock (right third), time strip, judgment controls,
                  Core/home anchor (only when not at Home), live/provenance mark
```

Rules:

- **One camera is the source of truth.** The WebGL and DOM layers derive from it every frame, so they can never drift apart.
- **Identity-preserving motion.** Objects are keyed by id. A layout change tweens each object from its current position to its new one. Objects never disappear and respawn. Entering objects fade or scale in near their parent, and leaving objects collapse toward theirs.
- **Semantic LOD.** Each object kind defines 3–4 representations, for example dot → labeled sphere → card with its parts → source. Thresholds use hysteresis. Zooming out collapses detail again. Never fake semantic zoom by scaling the same thing up.
- **Surfaces are layouts, not routes.** Orbital (default), causal, compare, temporal and judgment are different `layout()` functions over the same ids. Dense workbench modes (case design, deep execution inspection) are separate *modes* that may add rails and tabs. They must never leak into Home.
- **Time.** Setting the time cursor swaps the world snapshot for the one at that cursor, then the same identity-preserving transition runs. You can still pan, zoom and focus in the past. In the past everything is read-only.
- **The Core visual** must match mocks 01 and 05 (light) and 13 and 06 (dark): a stylized dark disc, a swirling bronze accretion of streaks and particles, subtle lensing at most. A single cheap fragment shader or a particle system is enough. Dark mode is designed separately, not inverted.
- **Artifacts** move through the states `excerpt → expanded → dismissed | pinned`.
  - Expanding snapshots `{camera, focus, time, surface, layout}` and animates the artifact out of its source. The world compresses left and the dock takes about a third of the viewport with sharp edges.
  - Collapsing reverses the animation and restores the snapshot exactly.
  - Artifacts are ephemeral. Only an explicit pin sends `persist-artifact` to the backend.
  - Heavy renderers (diff, grid, PDF, docx, chart, code) are added only when that artifact kind is being built, and they are lazy-loaded.
- **Beats.** A beat is plain data:

  ```ts
  { caption, focus?, surface?, reveal?, dim?, relationships?, artifacts?, time?, holdMs? }
  ```

  A narrative is a list of beats. A small player applies each beat through the same action dispatcher. Any user input pauses the player and records a checkpoint. "Continue" resumes from that checkpoint. The Core caption holds only the few words of the current thought. There is no transcript.
- **One action vocabulary for humans and agents:** `focus, back, home, select, reveal, connect, arrange(surface), compare, setTime, returnToNow, materializeArtifact, expandArtifact, collapseArtifact, pinArtifact, invoke(action)`.
  - View actions stay local.
  - Governed actions (`invoke`, pin) go to the Go service as intents. Authority is decided there.
  - Agents never produce coordinates and never touch Three.js internals.
- **Composer context.** The composer always knows the focus, selection, zoom/LOD, time, open artifact, selected text and visible objects. It sends these with every request, so "Why?" works without extra context.
- **Attention is spatial.** It is shown through distance, scale, opacity, motion, clarity and relationship emphasis, not badges. Settled work goes quiet. Anything requiring judgment is hard to miss, and not by color alone.
- **Accessibility.** Keyboard equivalents for every action. An outline/list view of the visible world. Reduced motion.

Suggested stack (keep it small): React 19 + Vite + TypeScript, `three` (no R3F required; allowed only if it simplifies the Core canvas), a tiny store (`useSyncExternalStore` or zustand), and a hand-rolled or `motion` tween. Add nothing else until a milestone needs it.

### 0.1.4 Backend and contracts (so you don't need recon)

- **Stack:** React UI → Go service `apps/godspeed-casework-go` (HTTP JSON + SSE, `127.0.0.1:4179`) → SEA-Forge (Rust) over SFWP (Unix socket). The Go service is the UI's only backend. The UI never talks to SFWP, the ledger or Gauntlet directly.
- **Endpoints** (`internal/server/server.go:59-66`; wire details in `apps/godspeed-cognitive-ui/reference/WIRE.md`):

  | Endpoint | Purpose |
  |---|---|
  | `GET /api/healthz` | health check |
  | `GET /api/world` | world snapshot at a cursor |
  | `GET /api/time` | revision positions |
  | `GET /api/events?last=<cursor>` | SSE stream: replay + live + heartbeat; on a gap, fetch a fresh snapshot |
  | `POST /api/intents` | submit an intent; the response is `accepted` or `refused`, plus a reason and a lease |
  | `GET /api/artifacts` | list artifacts |
  | `POST /api/artifacts` | persist (pin) an artifact |
  | `GET /api/artifacts/{ref}?level=minimal\|summary\|source` | fetch an artifact at a disclosure level |

- **Shapes:** `apps/godspeed-cognitive-ui/contracts/*.ts` (world, interaction, temporal, artifact, agent). These are reference shapes that match the wire format. You may change them. Positions sent by the backend are hints only. The UI's `layout()` owns placement.
- **Demo data:** `apps/godspeed-cognitive-ui/reference/northstar.world.json` is the Northstar world the mocks depict. Build against a local fixture first, and wire in Go second.
- **Full interface specs** (read only if you need detail): `.agents/reports/interface-contracts/`, where `04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md` and `03-GO-SYSTEM-FRONTEND-PORTS-SPECIFICATION.md` are the relevant ones. `09-COLD-AGENT-HANDOFF-AND-WALKTHROUGH.md` is the orientation guide.
- **Domain rules the UI must respect:**
  - Actions appear only if the backend lists them on the object. The UI never computes authority.
  - Execution completing is not settlement. Keep the two visibly distinct.
  - IDs are opaque.
  - Scrubbing into the past is read-only.
  - Use plain language, not case-engine jargon ("Needs your approval", not "Escalated authority").

### 0.1.5 Mock file names on disk

The names in §0 use an older numbering. The actual files (in `mocks/`) are:

| §0 name | File on disk |
|---|---|
| 01 home light | `01-home-light.png` |
| 02 semantic beat + incisive artifact | `02-semanticbeat-incisive-artifact.png` |
| 03 semantic beat + expanded diff | `03-semantic-beat-expanded-diff.png` |
| 04 multi-artifact beat | `04-multi-artifact-beat.png` |
| 05 focused case light | `05-focused-case-light.png` |
| 06 expanded artifact | `07-expanded-artifact.png` |
| 07 temporal rewind | `08-temporal-rewind.png` |
| 08 causal rearrangement | `09-casual-rearrangement.png` |
| 09 human judgment | `10-human-judgement.png` |
| 10 autonomous execution | `11-work-handed-off-autonomous-execution.png` |
| 11 case design time | `12-case-design-time.png` |
| 12 focused case dark | `06-focused-case-dark.png` |
| 13 home dark | `13-home-dark.png` |

The sidebars in `11-…` and `12-…` belong to contextual dense workbench modes. They are not the global shell.

### 0.1.6 Milestone 1: prove the scene model (fixture only, no backend, no process gates)

The code lives in `apps/godspeed-cognitive-ui/src` (currently a placeholder). Milestone 1 is done when the operator watches it and it feels like the mocks:

1. **Home** looks like mock 01 (and mock 13 in dark mode). The idle world wakes subtly when the pointer moves.
2. **Free navigation.** Pan, wheel/pinch zoom and restrained tilt all work. Crossing at least one LOD threshold changes the representation in both directions.
3. **Focus.** Clicking Northstar flies the camera in, and the layout matches mock 05. Core shrinks to the top-left anchor. Esc/back returns you through the focus levels, and the anchor returns you Home.
4. **Beats.** Typing "Why did the pilot fail?" plays 3–4 beats as in Appendix A §5: captions, rearrangement into the causal layout (mock 09), and one small document-excerpt artifact. Clicking during playback pauses it, and "continue" resumes.
5. **Artifacts.** The excerpt expands into the right dock (mocks 03 and 07). Closing it restores the exact prior camera, focus and layout.
6. **Time.** Scrubbing back (mock 08) removes and changes objects through identity-preserving transitions. Pan and zoom still work in the past.

Later milestones cover the Go wiring over SSE, human judgment (mock 10), autonomous execution receding (mock 11), the case design mode (mock 12), multi-artifact beats (mock 04), real artifact renderers, and accessibility completeness.

---

## 0. Authority and Reference Order

This file defines the **visual and interaction grammar** of the GodSpeed Cognitive Environment.

The reference images are not loose inspiration. They are **canonical visual states** of one product and should be reconstructed as one coherent interactive system.

When implementing or prototyping:

1. **Reference images govern visual composition, proportion, density, and look.**
2. **This DESIGN.md governs interaction grammar, semantic distinctions, state behavior, and transitions between those images.**
3. **The frozen frontend/API contract governs data meaning and lawful actions.**
4. **Backend systems remain authoritative for governed truth.**

Do not reinterpret the images into a conventional SaaS product.
Do not redesign them into a dashboard system.
Do not add navigation, cards, panels, or chat history simply because they are common web patterns.

### Canonical reference image set

Use these names consistently:

1. `01-home-light.png`
2. `02-semantic-beat-incisive-artifact.png`
3. `03-semantic-beat-expanded-diff.png`
4. `04-multi-artifact-beat.png`
5. `05-focused-case-light.png`
6. `06-expanded-artifact.png`
7. `07-temporal-rewind.png`
8. `08-causal-rearrangement.png`
9. `09-human-judgment.png`
10. `10-work-handed-off-autonomous-execution.png`
11. `11-case-design-time.png`
12. `12-focused-case-dark.png`
13. `13-home-dark.png`

The image set is a visual state machine, not a gallery of unrelated screens.

---

# 1. Product Idea

GodSpeed is a **persistent, agent-directed cognitive environment for governed work**.

The user should feel that they are looking at and moving through the work itself, not operating a project-management application.

The governing experience is:

```text
current world
→ focus
→ increased representational resolution
→ inspect / ask / act
→ execution and observation
→ reality changes
→ world reorganizes
→ unresolved consequence becomes perceptible
→ settlement quiets the world
```

The primary renderer is a **real-time spatial scene**.

Think:

```text
game world
+ knowledge graph
+ semantic zoom
+ evidence workbench
+ governed case system
```

not:

```text
dashboard
+ cards
+ chat sidebar
+ graph widget
```

The astronomy language is **visual physics only**. It is not product ontology.

Do not call work items planets, moons, stars, or orbits in the product language.

---

# 2. Semantic Ground Rules

The interface must preserve these distinctions:

- **Possibility is not affordance.** An affordance is visible, reachable, payable, governable, executable/recoverable, and settleable from the current horizon.
- **Execution is not settlement.** A run may complete while settlement remains evaluating, rejected, unknown, or escalated.
- **Agent termination is not settlement.**
- **Repository state is not case authority.**
- **Representation is not reality.**
- **A cognitive artifact is not durable case material until a governed persistence action succeeds.**
- **Access or one successful attempt is not capability.**
- **Role boundaries determine which higher-authority case operations a user can perform.**

The frontend should hide CMMN/case-management machinery without hiding its consequences.

A user should see:

- `Needs your approval`
- `Ready after security review`
- `You can reopen this work`
- `Waiting on customer evidence`
- `Available to add if needed`

rather than backend sentry, plan-item, or case-engine terminology.

---

# 3. The World Model

The cognitive environment has a small compositional grammar:

- **Core** — broad system orientation and home.
- **Object** — something worth perceiving, focusing, inspecting, acting on, or asking about.
- **Relationship** — a useful connection between objects.
- **Surface** — a purpose-specific arrangement of the same underlying world.
- **Focus** — the current local world / center of gravity.
- **Semantic Zoom** — increased representational resolution.
- **Time / Version** — the same world at another consequential state.
- **Composer** — persistent natural-language/voice intent entry.
- **Artifact** — bounded interactive materialized cognition.
- **Semantic Beat** — one thought expressed as one coordinated visual state.

The world is persistent. Pages are not the primary mental model.

---

# 4. Visual Theme

## 4.1 Light mode

Light mode is the default visual reference.

Use:

- pure or near-pure white background;
- very large amounts of negative space;
- black / dark-charcoal typography;
- very thin low-opacity relationships;
- restrained warm-white / pale-amber energy around Core;
- tiny accents only when semantic state requires them;
- crisp, quiet, premium composition.

The feel should be:

- Apple-like restraint;
- Linear-like precision;
- NASA-like operational clarity;
- game-world spatial continuity;
- knowledge-graph legibility;
- cinematic without spectacle.

Avoid:

- generic SaaS blue;
- card soup;
- gradients used as decoration;
- large glass panels;
- neon cyberpunk;
- sci-fi HUD clutter;
- decorative dashboards.

## 4.2 Dark mode

Dark mode is a **designed inversion**, not a CSS color inversion.

At Home:

- background becomes deep near-black / navy;
- Core becomes a luminous **blue-fire sun / vortex**;
- orbital relationships become cool blue;
- objects remain abstract scene entities, not literal planets;
- the world may contain subtle star/nebula texture, but it must remain quiet and sparse.

Reference: `13-home-dark.png`.

When focused inside a case, use the same dark visual physics while preserving the focused-case grammar. Reference: `12-focused-case-dark.png`.

**Important:** Home is already Core. Do not show a separate Core/Home button while the user is at Home. The compact Core/Home anchor appears only after the user has focused into another object/world.

---

# 5. Core

## 5.1 Home light

Reference: `01-home-light.png`.

Core is centered.

It is a black-hole-like scene object with:

- opaque black center;
- subtle lensing;
- restrained warm accretion ring;
- sparse swirling particulate matter;
- slow low-amplitude motion;
- slight asymmetry so it does not look like a CSS circle.

The effect should feel alive before the user consciously notices the animation.

## 5.2 Home dark

Reference: `13-home-dark.png`.

Core becomes a blue-fire luminous center.

The dark-mode Core may retain a dark inner void, but the dominant read is a radiant blue sun/vortex with energetic filament-like accretion.

## 5.3 Core as an information surface

Core may display a **very small semantic residue** inside or immediately integrated into its center.

Examples:

- `Good progress`
- `1 thing needs you`
- `1 path diverged`
- `All quiet`

This is not an assistant transcript.
It is not chain-of-thought.
It is not a chat response.
It is the smallest language necessary to frame the current semantic beat.

---

# 6. Cognitive Objects

At broad scale, cognitive objects are scene entities, not cards.

They may appear as:

- small spheres;
- dark or luminous discs;
- compact irregular scene objects;
- points with labels;
- subtle haloed nodes.

In light mode they should **not look like literal planets**.

Their appearance communicates salience through:

- distance from current center;
- scale;
- opacity;
- clarity;
- motion;
- glow/edge intensity;
- relationship emphasis.

Routine or settled material becomes quieter.
Unresolved consequential material becomes perceptually easier to find.

Avoid badge-driven attention overload.

---

# 7. Relationships and Spatial Physics

Relationships are graph-like but contextual.

Use:

- thin curves;
- faint lines;
- arcs;
- proximity;
- containment;
- spoke arrangements;
- sequence;
- causal chains;
- orbit-like placement.

Most relationships should remain low-opacity or invisible until useful.

The environment behaves like a gravitational knowledge graph:

- current center is the local center of gravity;
- related objects arrange around it;
- salience affects apparent weight;
- focus changes the center;
- semantic beats may temporarily rearrange the same objects.

Do not render permanent graph spaghetti.

---

# 8. Home

References:

- `01-home-light.png`
- `13-home-dark.png`

Home should be visually sparse.

The basic frame is:

```text
              sparse cognitive objects

                   CORE


          bottom-centered composer
```

When nothing needs the user, it is acceptable for the screen to contain almost nothing except Core and the composer.

Emptiness communicates low unresolved complexity.

No persistent sidebar.
No dashboard header.
No card grid.
No activity stream.
No chat transcript.
No Core button, because the user is already at Core.

---

# 9. Composer

The composer is the primary persistent conventional control.

It sits bottom-center and should look like:

- Google Search restraint;
- Apple Spotlight precision;
- a modern command field;
- not a chatbot.

Idle state:

```text
[ search icon   Ask, find, understand, or act…      voice ]
```

Properties:

- horizontally centered;
- compact height;
- modest width;
- soft but not excessive radius;
- restrained border/shadow;
- light mode white-on-white separation;
- dark mode subtle blue/cool outline;
- may expand slightly when active;
- never becomes a permanent chat panel.

The composer receives current:

- surface;
- focus;
- selection;
- temporal position;
- visible objects;
- open artifact context.

The user should not restate context already visible on screen.

---

# 10. Focused Case / Entering an Object

References:

- `05-focused-case-light.png`
- `12-focused-case-dark.png`

Focus is **camera movement**, not page navigation.

When the user focuses an object:

1. the camera travels toward it;
2. the object becomes the local center of gravity;
3. the broad system world recedes;
4. related internal objects emerge around the new center;
5. Core becomes a small home/orientation anchor;
6. Escape/back/Core unwinds one focus level.

The user should feel:

> I moved into Northstar.

not:

> I opened the Northstar detail page.

The small Core/Home anchor is appropriate here because the user is no longer at Home.

---

# 11. Semantic Zoom

Zoom changes representation, not merely geometry.

Example:

```text
Northstar
↓
Goal / Evidence / Implementation / Rollout
↓
Claims API / Session Migration / Secondary Coverage
↓
PR / replay / test / observed condition
↓
actual diff / source / evidence / document
```

At greater depth:

- labels become more concrete;
- internal structure appears;
- source artifacts become reachable;
- low-level operational material remains hidden until needed.

Never fake semantic zoom by making the same object larger.

---

# 12. Semantic Beats

Reference: `02-semantic-beat-incisive-artifact.png`.

A semantic beat is:

> **one thought, one visual state**.

The scene itself is part of the answer.

A beat may coordinate:

- camera focus;
- object reveal/hide;
- relationship emphasis;
- object rearrangement;
- minimal center text;
- artifact materialization;
- temporal movement;
- evidence insertion.

The center text is semantic framing only.

Do not show:

- private reasoning;
- thinking transcript;
- complete response transcript;
- long prose walls.

Use this production law:

> One thought. One visual state. Show the relationship. Reveal only what is needed. Change when the thought changes. Prove claims with reality.

---

# 13. Artifacts

Artifacts are bounded interactive representations materialized because the current thought needs more resolution.

They are ephemeral by default.

Possible artifact types:

- code diff;
- code excerpt;
- document excerpt;
- Word document;
- PDF;
- TanStack chart;
- TanStack table;
- spreadsheet slice;
- evidence record;
- API definition;
- timeline;
- causal diagram.

## 13.1 Incisive artifact

Reference: `02-semantic-beat-incisive-artifact.png`.

Show only the smallest useful evidence.

Example: a tiny diff excerpt containing the exact changed lines that matter.

The artifact includes an explicit expand affordance.

## 13.2 Multi-artifact composition

Reference: `04-multi-artifact-beat.png`.

Several artifacts may coexist when the thought genuinely requires them.

Example composition:

- TanStack chart;
- TanStack table;
- Word-document excerpt;
- spatial cognitive objects;
- optional code/diff working surface.

The artifacts remain bounded and contextual, not a dashboard grid.

## 13.3 Expanded artifact

References:

- `03-semantic-beat-expanded-diff.png`
- `06-expanded-artifact.png`

When expanded, an artifact becomes a sharp-edged working surface docked to the right.

Target behavior:

- approximately one-third of the viewport;
- flush to the top/right/bottom where appropriate;
- **square/sharp outer corners**;
- no floating rounded container;
- the world is pushed/compressed left rather than obscured;
- spatial context remains visible;
- the source object remains perceptually connected to the artifact;
- closing the artifact restores the prior scene and camera context.

The artifact is a browser-native precision surface.

For code/diff:

- dark code surface is acceptable in light mode;
- syntax highlighting is dense and legible;
- code metadata and source identity remain visible;
- use real diff semantics;
- do not turn it into a full IDE unless the user explicitly chooses `Open in Editor`.

---

# 14. Temporal Rewind

Reference: `07-temporal-rewind.png`.

History is not an activity-log page.

The **world becomes its earlier state**.

At another temporal position:

- objects may disappear;
- earlier assumptions may reappear;
- labels may change;
- relationships may change;
- evidence availability may change;
- current objects may become ghosted or not-yet-present.

A restrained temporal control may appear above the composer only while temporal navigation is active.

Support:

- previous consequential state;
- next consequential state;
- scrub through meaningful versions;
- compare two positions;
- return to now.

The source history is immutable.

---

# 15. Causal Rearrangement

Reference: `08-causal-rearrangement.png`.

The same source objects may leave their ordinary orbital arrangement and smoothly reorganize into a causal surface.

Example:

```text
Expected Workflow
→ Implementation Assumption
→ Customer Record
→ Missing Secondary Coverage
→ Observed Failure
```

The arrangement itself is part of the answer.

The world should animate into this configuration, preserve object identity, and be able to return to its previous arrangement.

Do not navigate to a `Causal Analysis` page.

---

# 16. Human Judgment / Authority Moment

Reference: `09-human-judgment.png`.

Human authority should appear only when it materially changes what can happen next.

The environment should narrow attention toward the consequential object and reveal a compact right-side decision surface.

The panel should:

- use sharp or very low-radius geometry;
- show why judgment is required;
- show exact lawful actions supplied by the backend;
- expose evidence/context one interaction away;
- distinguish approve/reject/escalate or other actual role-authorized operations;
- never invent actions for visual symmetry.

Only one action should receive dominant emphasis.

The frontend does not infer authority.

---

# 17. Work Handed Off / Autonomous Execution

Reference: `10-work-handed-off-autonomous-execution.png`.

When the user delegates governed work:

- the world should show that work has left the human;
- the relevant object can become subtly active;
- progress is visible when the user intentionally inspects it;
- background work does not flood the main scene with process telemetry.

An explicit operational surface may expose:

- bounded run progress;
- current step;
- changed files;
- evidence;
- logs;
- agent/run details.

This is a **deep operational view**, not the default Home shell.

Keep the distinction visible:

```text
Execution: RUNNING / SUCCEEDED / FAILED
Settlement: PENDING / EVALUATING / ACCEPTED / REJECTED / ESCALATED
```

Execution success never visually implies settlement acceptance.

---

# 18. Case Design Time

Reference: `11-case-design-time.png`.

Design time is intentionally more workbench-like because the user is authoring the governed structure itself.

This is one of the few places where a denser shell is appropriate.

The view may include:

- left navigation for design functions;
- central spatial case model;
- right-side structured editor;
- tabs for model/roles/evidence/preview/versions;
- stage sequence;
- role configuration;
- evidence configuration;
- preview/simulation;
- version/draft status.

The center remains the current case model, not a form page.

The right-side editor may expose CMMN-derived case concepts in human-operable language without requiring the user to understand CMMN.

Draft state must be explicit.

Do not imply commitment until the governed commit succeeds.

---

# 19. Agent Interaction

The agent operates the same represented world as the human.

Agent tools should be semantic operations such as:

- `focus(object)`
- `back()`
- `reveal(objects)`
- `hide(objects)`
- `connect(a, b)`
- `highlight(object)`
- `arrange(surface)`
- `compare(a, b)`
- `setTemporalPosition(version)`
- `returnToNow()`
- `materializeArtifact(ref)`
- `collapseArtifact(id)`
- `invoke(action)`

The agent must not manipulate Three.js coordinates directly.

Substantive explanation behaves like an **interruptible in-engine cutscene**:

- narration may direct camera and scene;
- user can click/inspect/branch at any time;
- the sequence pauses while the user explores;
- `Continue` resumes from the semantic beat.

The primary answer medium is the changing world, not a transcript.

---

# 20. Attention Discipline

Human attention is expensive.

The environment should surface only changes that materially affect:

- judgment;
- authority;
- knowledge;
- recovery;
- consequential next moves.

Normal events should remain quiet:

- agent started;
- tool called;
- commit created;
- routine check passed;
- normal background work.

Consequential events become perceptible:

- assumption disproven;
- human judgment required;
- authority escalated;
- evidence failed;
- settlement rejected;
- recovery capacity shrinking;
- new affordance opened.

As work settles, the interface should become quieter.

---

# 21. Semantic Color Roles

Use color only as a redundant signal.
Every semantic state also needs text and/or shape.

Recommended semantic roles:

- **focus/current path:** blue;
- **healthy/verified/allowed:** green;
- **attention/escalation/unresolved:** warm amber/orange;
- **failure/denial/rejection:** red;
- **quarantined/untrusted:** violet;
- **unknown/pending/background:** gray.

Avoid treating `green` as generic success.
Always preserve the state domain:

- `Execution succeeded`
- `Settlement accepted`
- `Authority allowed`
- `Evidence complete`

These are distinct.

---

# 22. Typography

Use a restrained premium sans for user-facing language and a precise mono for code/machine values.

Recommended stacks:

```css
--font-sans: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
--font-mono: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
```

Typography should feel:

- quiet;
- precise;
- slightly technical;
- never promotional.

Avoid oversized marketing typography.

Object labels should remain camera-facing and highly legible.
Use DOM text overlays where that improves clarity.

---

# 23. Geometry and Elevation

## World objects

Use soft scene depth, not UI-card elevation.

## Composer

May use moderate pill geometry.

## Small contextual artifacts

May use modest radius.

## Expanded working panels

Use square or very low-radius outer geometry.
Do not create floating rounded drawers.

## Decision panels

Prefer rectilinear, operational geometry.

Avoid:

- glassmorphism as the main language;
- deep shadow stacks;
- rounded-card soup;
- excessive radius.

---

# 24. Motion

Motion is semantic punctuation.

Use motion for:

- camera focus/back;
- semantic zoom;
- object rearrangement;
- artifact expand/collapse;
- temporal transitions;
- one-time attention arrival;
- Core ambient life.

Do not animate ordinary background updates.

Ambient motion should be low frequency and subtle.

Core can continuously breathe/swirls subtly.
Cognitive objects may drift/orbit very slowly.
Relationships may pulse only when relevant.

Respect reduced-motion settings.

---

# 25. Interaction Vocabulary

Keep direct interaction small and consistent.

Suggested grammar:

- click/tap — select;
- Enter / Space / double-click — focus;
- Escape — back one focus level;
- `/` or Cmd/Ctrl+K — composer;
- Tab / arrows — move among useful nearby objects where appropriate;
- explicit temporal control only in temporal mode.

The user should learn the environment through repetition, not documentation.

---

# 26. Design-Time vs Run-Time Shell Rule

The product has two visual densities.

## Run-time cognitive environment

Default.

- world-first;
- no persistent left navigation;
- minimal chrome;
- scene is primary;
- composer persistent;
- controls appear contextually.

## Design/deep operational workbench

Used when the user intentionally enters:

- case design;
- deep code/diff inspection;
- execution diagnostics;
- evidence matrices;
- other precision-heavy work.

Then more conventional rails/panels/tabs are allowed, but the cognitive world should remain visible or recoverable and the product semantics must remain intact.

Do not let workbench density leak back into Home.

---

# 27. Accessibility

Required:

- WCAG AA text contrast;
- visible keyboard focus;
- keyboard access to primary actions;
- list/table alternative for graph-dependent information;
- reduced-motion mode;
- 200% zoom without losing actions/state;
- screen-reader labels for objects, relationships, states, and actions;
- color never used as sole state signal.

Spatial presentation must never be the only way to access governed information.

---

# 28. Voice and Copy

Use calm operational precision.

Good:

- `1 thing needs you`
- `Execution completed. Settlement evaluation continues.`
- `Ready after security review`
- `Waiting on customer evidence`
- `This assumption no longer holds.`
- `No path is currently spendable.`

Avoid:

- `Congratulations!`
- `Great job!`
- `Mission accomplished!`
- `Something went wrong.`
- long explanatory prose when the scene can carry the relationship.

---

# 29. Anti-Targets

If the prototype primarily resembles any of these, revise it:

- Jira;
- Linear clone;
- Trello;
- React Flow demo;
- Obsidian graph;
- ChatGPT/Claude chat;
- Grafana/Datadog dashboard;
- VS Code shell;
- CRM;
- generic mission-control dashboard;
- generic Three.js space demo.

Borrow mechanics, not product form.

---

# 30. OpenDesign Instructions

When reconstructing the prototype from the reference images:

1. Treat the 13 images as canonical states of one interface.
2. Do not invent a new design system that competes with them.
3. Reconstruct shared typography, spacing, object geometry, line weights, artifact surfaces, composer geometry, semantic accents, and dark/light behavior from the images.
4. Build a **single interactive stateful prototype**, not 13 static pages.
5. Preserve scene continuity between states.
6. Animate identity-preserving transitions rather than hard page cuts where feasible.
7. Home must remain sparse.
8. Focus must feel like camera movement into an object.
9. Semantic beats must make arrangement part of the answer.
10. Artifact expansion must push/compress the scene and dock sharply on the right.
11. Temporal rewind must transform the world rather than open a history page.
12. Causal explanation must rearrange existing objects.
13. Human judgment must surface role-authorized actions only when required.
14. Autonomous execution must not become the default product shell.
15. Case design time may use a denser workbench shell.
16. Dark mode is explicitly designed; do not mechanically invert light mode.
17. Do not show a Core/Home button on Home itself.
18. Show the Core/Home anchor only after the user has focused into another object/world.
19. Do not introduce a persistent transcript.
20. Do not expose backend CMMN/case-management jargon unless the user deliberately enters a technical/deep inspection context.

## Prototype flow to implement

At minimum wire these transitions:

```text
01 Home Light
  ↓ focus object
05 Focused Case Light
  ↓ semantic question
02 Semantic Beat + Incisive Artifact
  ↓ expand artifact
03 Semantic Beat + Expanded Diff
  ↓ compose multiple evidence forms
04 Multi-Artifact Beat
  ↓ deeper artifact inspection
06 Expanded Artifact
  ↓ move backward in time
07 Temporal Rewind
  ↓ ask why
08 Causal Rearrangement
  ↓ consequence requires authority
09 Human Judgment
  ↓ delegate approved work
10 Autonomous Execution
  ↓ edit underlying case model
11 Case Design Time
```

Theme variants:

```text
01 Home Light ↔ 13 Home Dark
05 Focused Case Light ↔ 12 Focused Case Dark
```

---

# 31. Final Visual Acceptance

The prototype is visually correct when:

- Home reads as a quiet world, not an app shell;
- Core is unmistakably the center of gravity;
- cognitive objects feel spatial and graph-related without becoming literal planets;
- focus feels like entering an object;
- semantic zoom changes representation;
- semantic beats make the scene part of the answer;
- artifacts materialize from context and deepen progressively;
- expanded artifacts become real working surfaces without destroying orientation;
- time changes the world itself;
- causal explanation is a rearrangement of the same world;
- human authority appears at the moment of leverage;
- delegated work becomes quieter unless inspected;
- case design time is dense enough for precision without redefining the whole product;
- light and dark modes preserve the same cognitive grammar;
- no private reasoning or full assistant transcript becomes the UI.

The desired user experience is:

> **I am looking at my current world. I move closer to what matters. The system rearranges reality into the smallest representation from which I can understand or act. I can touch the evidence. I can delegate the work. The world changes when reality changes.**

---

# Appendix A. Operator Clarifications (verbatim, 2026-09-22)

> Binding. Written by the operator after reviewing the failed builds. Where anything else in this document seems to conflict, this appendix and §0.1 win.

Your read is broadly right. The main thing I want to add is that there are a few interaction ideas that are more load-bearing than the mocks alone may make obvious, especially **artifacts, semantic beats, and free spatial navigation**.

A good mental model is still: **a persistent 2.5D world with a camera, not a graph rendered inside a webpage**. I do not care whether every label and panel is literally rendered in Three.js—in fact, text-heavy surfaces, documents, diffs, tables, charts, forms, etc. probably should be DOM—but the *world itself* needs scene-like behavior: camera continuity, pan, zoom, focus, semantic LOD, object identity, spatial relationships, and smooth rearrangement.

## A.1 The user can pan and zoom freely

We've focused a lot on click-to-focus, but I don't want navigation limited to that.

The user should be able to:

* pan around the current world;
* wheel/trackpad/pinch zoom;
* move closer/farther without selecting something;
* orbit/inspect the current spatial arrangement within restrained bounds;
* click an object to focus it;
* back out through previous focus levels;
* return to Core/Home.

This should feel more like navigating a game/map/3D workspace than scrolling a page.

But zoom is also **semantic**. As you cross useful zoom thresholds, the representation changes:

```text
Northstar
    ↓
Goal / Evidence / Implementation / Release
    ↓
Secondary Coverage / Claims API / Session Migration
    ↓
PR / Test / Observation
    ↓
actual diff / source / document / evidence
```

So there are really two things happening together: camera zoom; representational resolution. Zooming out should collapse detail again.

I do not want arbitrary free-flight 3D. It is more like a controlled 2.5D scene with depth, parallax, pan, camera movement, and semantic LOD.

## A.2 Artifacts are not ordinary panels

This is probably the biggest thing to understand.

An **artifact** is a temporary piece of working cognition that appears because the current thought needs more resolution.

Examples: one incisive diff; a quotation; three rows from a spreadsheet; a small chart; a table; part of a Word document; a PDF passage; source code; an API definition; a screenshot; a causal diagram; a timeline; an evidence record.

The initial artifact should usually be **small and incisive**.

For example, if the agent says:

> "This is the assumption that changed the outcome."

it should not immediately open a giant document viewer. A small artifact may appear next to the relevant object showing only the sentence that matters. The user can then expand it.

## A.3 Expanded artifacts are a different state of the same artifact

When an artifact is expanded, it becomes a real working surface. For the desktop mockups, I like the behavior we established:

* the artifact expands into a sharp-edged pane on the right;
* perhaps around one-third of the viewport;
* the spatial world compresses/pushes left;
* the world remains visible;
* the user retains spatial orientation;
* the artifact is fully interactive.

So a diff becomes a real diff viewer. A Word artifact becomes the actual document viewer. A table becomes the real interactive table. A chart becomes the real chart. A spreadsheet excerpt becomes a proper grid.

The user should be able to: scroll; search; select text; inspect details; interact; ask the agent about the current selection.

Closing the expanded artifact should return the user to **exactly the previous camera, focus, time and arrangement**. It should feel as if the artifact expanded out of the world and then collapsed back into it.

## A.4 Artifacts are normally ephemeral

An artifact exists because the current reasoning needs it. Its normal lifecycle is something like:

```text
materialize → inspect → possibly manipulate → dismiss
```

or:

```text
materialize → inspect → "keep this" → preserve / pin
```

So the system should not automatically turn every AI-generated visual into permanent case state. Temporary representation and durable evidence/state are different things.

## A.5 Semantic beats are the main AI response mechanism

I do not want the AI's substantive response to primarily be a transcript.

A **semantic beat** is approximately one meaningful thought represented as one coherent visual state.

For example — User: *"Why did the pilot fail?"*

* **Beat 1:** "The implementation itself worked." Implementation moves forward. A tiny verified indicator appears.
* **Beat 2:** "The assumption about secondary coverage was wrong." Implementation recedes. The relevant assumption and customer record come forward. A small document artifact appears containing the exact evidence.
* **Beat 3:** "That caused this path to diverge." The world rearranges into the causal structure.
* **Beat 4:** "This is the resulting failure." The observed outcome comes forward.

The **movement, arrangement, relationships and artifacts are part of the answer**. The text should therefore be very short.

The little text inside Core/current center is not: hidden chain-of-thought; a transcript; the full assistant response. It is closer to the caption for the current thought. For example `1 path diverged` or `Secondary coverage changed the outcome`.

## A.6 Semantic beats and artifacts work together

Artifacts are ingredients inside semantic beats. A beat might: focus an object; move several objects; reveal a relationship; dim irrelevant objects; materialize a document excerpt; materialize a chart; annotate an observation; move time backward; then pause.

A later beat may reuse an artifact already on screen, replace it, collapse it or introduce another. That is what the multi-artifact mock was trying to show.

For example, a single thought may require: a TanStack chart showing the behavioral change; a TanStack table showing which cohort caused it; a Word-document excerpt containing the customer's stated requirement. Those three artifacts together might constitute the evidence for the beat. They are not three dashboard widgets permanently attached to the case.

## A.7 Semantic beats are interactive, not video

This is effectively an **interactive in-engine cutscene**.

While an explanation is happening, the user can interrupt. They might: click the customer record; expand the document; select a sentence; ask "Why does this matter?"; pan somewhere else; zoom into Implementation.

The current semantic sequence should pause/branch. When the user says *"Continue"*, the system should be able to resume from the previous narrative position. So we need some concept like:

```text
Narrative
    └── Beat
          ├── minimal narration
          ├── focus
          ├── scene arrangement
          ├── relationships
          ├── artifacts
          ├── temporal position
          └── resumable checkpoint
```

It does not need to become a huge DSL.

## A.8 Surfaces are rearrangements of one world

Things like causal explanation, comparison, history, execution, customer-ready explanation are not necessarily pages. They are often **different arrangements of the same objects**.

Normal Northstar arrangement:

```text
       Goal

Evidence   Northstar   Implementation

      Customer Workflow
```

Ask *"Why did this fail?"* — same underlying objects animate into:

```text
Expected
    ↓
Assumption
    ↓
Missing condition
    ↓
Observed failure
```

Ask *"Compare this with the normal path."* — they rearrange side-by-side. No route change is required.

## A.9 Time works the same way

"History" should not primarily mean an event-log page. When I say *"Walk me through how we got here."* the world itself should become its earlier version. At an earlier point: some objects do not exist yet; old assumptions reappear; relationships differ; evidence is missing; state labels differ.

The user can still pan, zoom, focus and inspect while standing in that earlier representation. Then we can move forward again. RealityTrace is one of the things making that temporal reconstruction possible.

## A.10 Spatial continuity matters

When the representation changes, preserve identity. Objects should generally animate from their old positions to their new positions rather than disappear and randomly respawn elsewhere. Camera transitions matter for the same reason. Artifacts should expand from their source. Closing them should collapse back toward their source. The point is preserving the user's mental model.

## A.11 Attention is communicated spatially

I do not want every backend event turned into a badge. The scene should communicate importance through: distance; scale; opacity; motion; clarity; depth; relationship emphasis.

If something is settled and needs nothing, it should become quiet. If something needs human judgment, it should become perceptually hard to miss. This is why "quiet UI" is important to the design.

## A.12 The composer is persistent, but chat is not

The composer remains near the bottom center across normal world states. It knows: current focus; current selection; current zoom/representation; current time; open artifact; selected document text; visible world.

So I should be able to simply say *"Why?"* or *"Compare these."* or *"Go back before this changed."* without re-explaining context. The answer then modifies the environment rather than accumulating chat bubbles.

## A.13 The agent manipulates semantic operations, not pixels

Humans and agents should effectively share actions such as:

```text
focus(object)
back()
select(object)
reveal(objects)
connect(a, b)
arrange(surface)
compare(a, b)
setTime(version)
returnToNow()
materializeArtifact(ref)
expandArtifact(id)
collapseArtifact(id)
invoke(action)
```

The agent should not be generating pixel coordinates or directly manipulating Three.js internals.

## A.14 The sidebars in the mocks are contextual, not the global runtime shell

The normal spatial runtime views—Home, focused case, semantic explanations, temporal views, causal views—should **not** suddenly grow a permanent Linear/Jira-style sidebar.

The denser sidebars/panels in some mocks represent specific workbench states. The clearest example is **case design time**. When someone is deliberately constructing a case, precision matters, so a denser editor/workbench with stages, roles, triggers, evidence, milestones, versions, etc. is appropriate. Likewise, if the user deliberately drills into execution details, a denser operational inspection surface is reasonable.

But after simply handing work to an agent, I do not want the default world to become a giant agent-control dashboard. Normal autonomous execution should mostly recede into the background.

```text
normal runtime               → sparse spatial world
deep operational inspection  → denser workbench allowed
case design time             → denser workbench intentionally allowed
```

Those are modes of the same product, not one permanent shell.

## A.15 Hybrid implementation is welcome

I do not need labels, documents, charts and buttons forced into WebGL merely to say we used R3F. A hybrid may actually be the right implementation:

```text
R3F / Three  → Core, spatial world, camera, depth, particles,
               relationships where appropriate, scene transitions
DOM / SVG    → labels, composer, documents, diffs, charts, tables,
               authority surfaces, precise controls
```

What I don't want to lose is the **scene model**. If the orbiting world becomes ordinary absolutely positioned DOM that happens to animate, but we still get genuine pan/zoom, camera continuity, semantic LOD, spatial focus and smooth rearrangement, that may be perfectly fine. The important thing is that it behaves like a world rather than like pages.

## A.16 Smaller details that matter

* mouse/pointer movement can subtly "wake" the idle world;
* relationships can reveal on hover/focus rather than always being visible;
* Core is not shown as a Home button when you're already at Home;
* once you've entered something else, Core becomes the small return-home anchor;
* dark mode is a designed alternate visual system, not simply inverted CSS;
* human judgment controls only appear when actual authority is required;
* role-specific actions come from the backend contract;
* execution completion and settlement remain visibly distinct;
* source/evidence should always be reachable from claims;
* a user can preserve an artifact explicitly rather than everything becoming permanent automatically;
* accessibility should have keyboard equivalents and a non-spatial/list representation where necessary.

**DOM/SVG where that's cheaper and clearer, custom graphics only where graphics actually carry meaning.**

The thing I would preserve very strongly is this:

> The primary interaction model is still a persistent spatial environment with a camera. Pan, zoom, focus, time, rearrangement and artifacts change the user's representation of the same underlying world.

If we can get that behavior while dramatically simplifying the implementation, that's closer to what I want than maintaining a technically impressive R3F architecture that keeps failing to produce the experience.

