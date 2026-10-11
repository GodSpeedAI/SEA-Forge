# T03 settlement report — the React UI core and local world adapter

- **Task:** T03 (P2, confirmation: peer). **Gates:** `GATE_SPEC_TRACE` PASS · `GATE_UI`
  (`just casework-ui-check`: frozen install, `tsc --noEmit`, `vite build`, `bun test`) PASS.
- **Date:** 2026-09-20. **Settles:** REQ-GOAL-002, REQ-GOAL-003, REQ-OUT-002, REQ-ARCH-003, REQ-ARCH-004.
- **Preregistration:** frozen at 2026-09-20T00:13:15Z, before any app file, recipe, or test was written.
- **Resource preflight:** `raw-logs/resource-preflight.log` — MemAvailable 2.04 GiB (2141308 kB),
  swap 1.55 GiB (1626988 kB; 1 GiB = 1048576 kB), PSI ~0, no competing builds. All T03 gates are Bun/TypeScript work
  (no Rust compilation); they ran one at a time without pressure.

## What was built

| Artifact | Role |
|---|---|
| `apps/godspeed-cognitive-ui/src/core/` | The renderer-independent UI core: `model.ts` (SurfaceProjection / CognitiveObject / CognitiveRelationship shapes over the T01 contract types, TemporalContext state, CognitiveArtifact descriptor, action-outcome and refusal vocabulary), `store.ts` (the local state container), `actions.ts` (the application action vocabulary: focus, moveFocus, setSurface, select, resolve/disclosure cycling, propose, time stepping and return-to-now, narration append/interrupt), `engine.ts` (composes adapters + store + actions), `ports.ts` (catalog and scene-renderer seams) |
| `apps/godspeed-cognitive-ui/src/adapters/fixture/` | The fixture world adapter set — a representative FDE casework world (harbour dredging permit: 3 surfaces, 7 objects, 6 relationships, 3 bounded artifacts) with history revisions, plus `alt-provider.ts`: a second, structurally different provider (adjacency map, lazy lenses, different kinds/ids/salience) implementing the same ports for the provider-swap tooth |
| `apps/godspeed-cognitive-ui/src/adapters/test/` | No-op/test adapters: unavailable agent, scripted agent (for the interruption test), recording scene renderer |
| `apps/godspeed-cognitive-ui/src/host/` | The runnable React 19 host: composes the fixture environment, renders surfaces/objects/disclosure/panels, and maps mouse + keyboard onto the same action vocabulary the tests call; `bootstrap.tsx` is the only file that chooses the adapter set |
| `apps/godspeed-cognitive-ui/src/core/purity.test.ts` | The mechanical core-purity gate (no react/three/copilotkit/transport/backend imports or identifiers in `src/core`), enforced inside the test suite so `GATE_UI` observes it every run |
| `justfile` → `casework-ui-check`, `casework-ui-up`, `casework-ui-down`, `casework-ui-status` | The named `GATE_UI` recipe and the operator controls |
| `apps/godspeed-cognitive-ui/README.md` | Documents the architecture, the commands, and the one-line what-this-is / what-this-is-not note |

## Gate evidence

```
$ python3 .agents/plans/validate-godspeed-casework-cognitive-environment.py
PASS: spec hash, 86/86 requirements, 15-task DAG, gate activation, and UI removal

$ just casework-ui-check
casework-ui-check: frozen install, typecheck, build and tests green
```

Raw logs: `raw-logs/gate-casework-ui-check.log`, `raw-logs/bun-test-full.log` (8 tests, 226
assertions, 0 fail), `raw-logs/operator-cycle.log`.

## Teeth (executed)

`teeth/run-teeth.sh` → exit 0, `TEETH RESULT: all three teeth behaved as specified`
(`teeth/teeth-run.log`):

1. **Scene and agent adapters replaced by no-op/test adapters** — the recording scene renderer and
   the unavailable agent pass; the core's tests execute against the fixture adapter and narration
   degrades honestly (`available=false`, "unavailable, not failed"). The interruption test proves
   the agent seam is interruptible by construction with a scripted agent.
2. **Fixture provider swapped for a structurally different provider** — ONE shared interaction
   scenario (`runInteractionScenario`) passes under both providers; zero hardcoded ids anywhere in
   it (every id is derived from the state the environment loaded, which is what makes the swap a
   real test rather than a tautology). The interaction grammar is unchanged by the provider.
3. **Purity gate non-vacuity (real injection)** — with `import { useState } from 'react'` written
   into `src/core/`, the purity scan FAILS (`zz_tooth_injected_react.ts: imports react`); with the
   file removed it passes again. The gate can fail for the right reason.

## `done_when` adjudication

1. **Fixture-driven UI core executes without backend or scene dependencies.** MET — the engine runs
   against the fixture adapter with the scene replaced by a recording no-op and the agent by an
   unavailable adapter (tooth 1). No SEA Forge, GitHub, Gauntlet, CopilotKit, transport, or renderer
   library appears under `src/core` (purity test, red-injected in tooth 3).
2. **Shared application actions drive focus/selection/temporal/artifact state.** MET — every state
   transition in the suite (and in the host's keyboard/mouse handlers) goes through the same action
   vocabulary; the scenario covers focus, move, select, surface policy (switching clears both),
   disclosure cycling (minimal → summary → source, then clamped), uncatalogued objects reporting no
   disclosure, time stepping/return-to-now, refused consequential intents, and narration state.
3. **Adapter replacement tests pass.** MET — teeth 1–3 above.
4. **All applicable global gates remain green.** MET — `GATE_SPEC_TRACE` and `GATE_UI` both exit 0.
   `GATE_SEAFORGE` (activates T05) was not run and is not claimed; the Rust-wide gates remain
   deferred under the build-resource policy recorded in the handoff (MemAvailable ≈ 2.1–2.3 GiB
   through this session, thinner than the T00 baseline window).

## REQ adjudication (what each settled requirement can and cannot claim)

- **REQ-GOAL-002 (interaction model independent of SEA-Forge, Go transport, Three.js, CopilotKit,
  repository providers, artifact renderers):** the interaction model and core are demonstrably
  independent (tests + mechanical scan). The "React cognitive environment" itself exists as a React
  19 host, so the requirement's subject exists rather than being vacuous.
- **REQ-GOAL-003 (small compositional grammar: surfaces, objects, relationships, focus, zoom, time/
  version, composer interaction, bounded artifacts):** the core representations and actions cover
  the grammar's T03 slice. **Semantic zoom and composer interaction are NOT implemented** — they are
  the spatial scene's (T06) and composer's (T08) mechanics; the grammar here is the vocabulary they
  will extend, which is what T03's plan steps actually demand.
- **REQ-OUT-002 (render projections and maintain local state without becoming authority):** by
  construction — state is a projection + interaction record; the only consequential path is an
  intent that the fixture adapter refuses; nothing in the core can settle, approve, or persist case
  truth.
- **REQ-ARCH-003 (core executable against a local fixture adapter without SEA-Forge, GitHub,
  Gauntlet, CopilotKit):** demonstrated by every test run; the process-level claim (no backend
  process contacted) holds because no transport code exists in the core and the fixture adapters
  are in-memory.
- **REQ-ARCH-004 (replacing the Go projection transport must not require redesigning the
  interaction grammar):** the provider-swap tooth is the design-level proof at the world-port seam.
  The actual Go transport arrives at T10; the tooth proves the grammar's independence, not the
  transport.

## Honest limits

- The host is a deliberately small DOM surface, not the spatial scene: no R3F/Three.js, no semantic
  zoom, no composer (T06/T08 own those). Time positions are metadata state only — projecting
  historical world *content* is T07.
- The demo page and README state plainly that the app is fixture-backed and connected to nothing.
- `bun.lock` is committed for frozen installs; `node_modules/` and `dist/` are ignored in the app's
  own `.gitignore`.

## Preserved corrections (round 0 → 1)

1. **The engine passed the interaction adapter where a dispatch function was expected** — caught by
   the shared scenario (`dispatch is not a function`), fixed by adapting at the call site.
2. **The first shared scenario hardcoded fixture object ids** and so failed under the alt provider —
   which was exactly the fixture-leak the tooth exists to catch. The scenario now derives every id
   from the loaded state, making the provider swap a real falsification attempt.
3. **The purity scan flagged the word "Gauntlet" in the engine's doc comment.** The scan was right
   to be suspicious of prose; the comment was reworded rather than the scan weakened.
4. **Type-level fixes found by `tsc`:** wrong import depth for the adapter files, a port signature
   whose `observe` returned `void` where the store's unsubscribe is required, and an over-wide
   catalog constructor parameter.
