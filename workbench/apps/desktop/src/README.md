# src

## Overview
Application source for the desktop Workbench renderer (React 19 + TypeScript + Vite). `main.tsx` mounts the app (StrictMode, astryx theme, TanStack Query client, TanStack Router); `router.tsx` defines the full route tree with guard-based blocking; `ProofPage.tsx` and `statusMachine.ts` are the proof/status verification page and its XState machine.

## Subdirectories & Boundaries
- Core renderer: [./core/README.md] - Persistent WebGL2 CORE viewport, camera, pipeline, quality, shaders, chrome, and provenance record.
- Shell: [./shell/README.md] - `AppShell` spatial composition, surface rail, evidence context, connection state.
- Composer: [./composer/README.md] - Persistent context-bound command composer.
- Spatial: [./spatial/README.md] - Focus/zoom/time-version controllers and `Surface`/`SpatialObject`/`Relationship`/`Artifact` primitives.
- Surfaces: [./surfaces/artifacts/README.md], [./surfaces/case/README.md], [./surfaces/case-design/README.md], [./surfaces/causal/README.md], [./surfaces/execution/README.md], [./surfaces/home/README.md], [./surfaces/judgment/README.md], [./surfaces/semantic-beat/README.md], [./surfaces/temporal/README.md] - Spatial surfaces composing the operate pages.
- Pages: [./pages/README.md] - Operate pages, route registry, and standing/variant presentation.
- Hooks: [./hooks/README.md] - Governed-query data layer over the Tauri host bridge.
- Guards: [./guards/README.md] - Governance guard taxonomy, evaluation, blocking-guard context, denial surface.
- Machines: [./machines/README.md] - XState case-authoring machine with no-side-effect-safe commit handling.
- Projections: [./projections/README.md] - Pure view-model projection functions.
- Dev: [./dev/README.md] - CORE tuning panel overlay.

## Interface
- `router` — `createRouter({ routeTree })` [router.tsx:router]
- Route paths: `/`, `/readiness`, `/thoth`, `/assets`, `/delegate`, `/models`, `/cases`, `/cases/new`, `/inbox`, `/operations`, `/evidence`, `/runs/$runId`, `/memory`, `/capabilities`, `/artifacts`, `/federation`, `/admin` [router.tsx:rootRoute]
- `ProofPage()` — proof/status verification page driving `statusMachine` [ProofPage.tsx:ProofPage]
- `statusMachine` — XState 5 machine modeling status verification [statusMachine.ts:statusMachine]

## Invariants
- Routes guarded by `BLOCKING_GUARDS` (G1, G2, G9) render `GovernedDenialSurface` instead of the destination when a guard fails; guard evaluation is via `evaluateGuard` + `useGuardContext` [router.tsx:RootComponent]

## Side Effects
- `main.tsx` mounts React into `document.getElementById("root")` and loads global stylesheets (astryx reset/theme, ui-tokens, sea-forge-astryx theme) [main.tsx:createRoot]

## Verification
- Run: `bun run test` (from workbench/)
