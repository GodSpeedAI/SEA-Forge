# React + TypeScript + Vite

This template provides a minimal setup to get React working in Vite with HMR and some Oxlint rules.

Currently, two official plugins are available:

- [@vitejs/plugin-react](https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react) uses [Oxc](https://oxc.rs)
- [@vitejs/plugin-react-swc](https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react-swc) uses [SWC](https://swc.rs/)

## React Compiler

The React Compiler is not enabled on this template because of its impact on dev & build performances. To add it, see [this documentation](https://react.dev/learn/react-compiler/installation).

## Expanding the Oxlint configuration

If you are developing a production application, we recommend enabling type-aware lint rules by installing `oxlint-tsgolint` and editing `.oxlintrc.json`:

```json
{
  "$schema": "./node_modules/oxlint/configuration_schema.json",
  "plugins": ["react", "typescript", "oxc"],
  "options": {
    "typeAware": true
  },
  "rules": {
    "react/rules-of-hooks": "error",
    "react/only-export-components": ["warn", { "allowConstantExport": true }]
  }
}
```

See the [Oxlint rules documentation](https://oxc.rs/docs/guide/usage/linter/rules) for the full list of rules and categories.

## Overview

The `desktop` Bun/React workspace app is the Workbench frontend renderer: a React 19 + TypeScript + Vite application (package name `desktop`, private, ESM) presenting the SEA Forge Workbench as a spatial UI around a persistent WebGL2 CORE viewport. Server interaction goes exclusively through the Tauri host bridge (`@tauri-apps/api` `invoke`/`listen`) against the SFWP query/command/request-status verbs, with response bodies validated against the generated `@sea-forge/contracts` schemas via AJV. Workspace dependencies: `@sea-forge/contracts`, `@sea-forge/ui-components`, `@sea-forge/ui-tokens`, `@sea-forge/astryx-theme`, plus `@tanstack/react-query`, `@tanstack/react-router`, `xstate`/`@xstate/react`, `three`, `@stylexjs/stylex`, and `react-hook-form` [package.json:desktop].

The e2e directories (`e2e/`, `e2e-agent-browser/`) and Playwright config cover browser-level E2E evidence (`workbench-e2e-*` gates) and are out of scope for unit-level verification.

## Interface

- `src/main.tsx` — application entry: mounts `<RouterProvider router={router} />` inside StrictMode, `QueryClientProvider`, and the `seaForgeTheme` Theme [src/main.tsx:createRoot]
- `src/router.tsx` — `router` and the full route tree (`/`, `/readiness`, `/thoth`, `/assets`, `/delegate`, `/models`, `/cases`, `/cases/new`, `/inbox`, `/operations`, `/evidence`, `/runs/$runId`, `/memory`, `/capabilities`, `/artifacts`, `/federation`, `/admin`) [src/router.tsx:router]
- Scripts from `package.json`: `dev` (vite), `build` (`tsc -b && vite build`), `check` (`tsc -b --noEmit && oxlint`), `test` (vitest run), `e2e` (`scripts/workbench-e2e-agent-browser.sh`), `e2e:mocked` (playwright test), `lint` (oxlint), `preview` [package.json:scripts]
- Child contract links: [./src/README.md] (with its `core`, `shell`, `composer`, `spatial`, `surfaces/*`, `pages`, `hooks`, `guards`, `machines`, `projections`, `dev` sub-contracts)

## Invariants

- All server interaction flows through the Tauri host bridge verbs (`sfwp_query`, `sfwp_command`, `sfwp_request_status`); governed denial bodies are checked before schema validation [src/hooks/governedQuery.ts:queryGoverned]
- The golden reference `public/reference/gargantua.html` is a byte-identical donor copy (sha256 verified) and must never be mutated [src/core/PROVENANCE.md:Golden reference]
- Routes gated by the blocking guards (G1, G2, G9) render `GovernedDenialSurface` instead of the destination when a guard fails [src/router.tsx:RootComponent]

## Side Effects

- `dev`/`preview`/`build` start Vite/TypeScript build processes; `build` writes the production bundle to `dist/` [package.json:scripts]
- The renderer reads/writes browser session storage for the acting identity and issues host-bridge IPC calls and Tauri event subscriptions at runtime [src/hooks/useIdentity.ts:selectedActorId]

## Verification

- Run: `bun run test` (from workbench/)
- Typecheck + lint: `bun run --cwd apps/desktop check`
- Build: `bun run --cwd apps/desktop build`
- E2E (separate gate): `bun run --cwd apps/desktop e2e`
