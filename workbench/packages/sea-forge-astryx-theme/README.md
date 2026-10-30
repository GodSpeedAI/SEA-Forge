# @sea-forge/astryx-theme (sea-forge-astryx-theme)

## Overview

Bun workspace package (`@sea-forge/astryx-theme`, private, `"type": "module"`) that adapts the Astryx design system to SEA Forge. It has one purpose: keep the Astryx "neutral" theme working inside the packaged desktop application's strict Tauri Content Security Policy. Instead of Astryx's `defineTheme` runtime style injection (which would require `unsafe-eval`), the package exports the prebuilt neutral theme projection and a static stylesheet that maps Astryx theme variables onto SEA Forge's canonical token set [src/theme.ts:comment, src/theme.css:header-comment].

Peer dependencies: `@astryxdesign/core` and `@astryxdesign/theme-neutral` at 0.1.8 [package.json:peerDependencies]. Entry point: `main`/`types` = `./src/theme.ts` [package.json:main].

## Interface

- [./src/README.md] — contract for `src/`, containing:
  - `export const seaForgeTheme = neutralTheme` from `src/theme.ts` (the package's only TypeScript export) [src/theme.ts:seaForgeTheme]
  - `src/theme.css` — static `@layer astryx-theme` / `@scope ([data-astryx-theme="neutral"])` stylesheet binding Astryx variables (`--color-background-*`, `--color-text-*`, `--color-accent`, `--font-family-*`) to SEA Forge tokens (`--surface-*`, `--fg-*`, `--color-authority-*`, `--font-sans`, `--font-mono`) [src/theme.css:@scope].

## Invariants

- No runtime theme generation: the theme must stay the `__built` neutral projection plus the static stylesheet, because Astryx's `defineTheme` path is incompatible with the strict CSP [src/theme.ts:comment].
- The theme name must remain `"neutral"` so Astryx's generated component CSS applies [src/theme.ts:comment].
- The CSS variable mapping in `src/theme.css` is the coupling point to `@sea-forge/ui-tokens` values and must be updated there, not by redefining tokens here.

## Side Effects

- `["None"]` — the package exports a constant object and a stylesheet; it performs no I/O, network access, or DOM mutation on import. The stylesheet affects rendering only once loaded by the consuming application.

## Verification

- The package defines no scripts/tests (`package.json` has no `scripts`). It is type-checked as part of the workspace: `just workbench-check` from the repo root, or `bun run check` in `../sea-forge-ui-components/`.
