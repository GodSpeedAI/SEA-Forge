# sea-forge-astryx-theme/src

## Overview

Source of the `@sea-forge/astryx-theme` package: SEA Forge's projection onto the Astryx design system. It consists of two files that together bind the Astryx "neutral" theme to SEA Forge's canonical design tokens under the desktop application's strict Content Security Policy:

- `theme.ts` — the TypeScript entry point (`main`/`types` in `../package.json`). It re-exports the prebuilt neutral theme projection from `@astryxdesign/theme-neutral/built` as `seaForgeTheme` [theme.ts:seaForgeTheme].
- `theme.css` — a static stylesheet that maps Astryx theme CSS custom properties (`--color-background-*`, `--color-text-*`, `--font-family-*`) onto SEA Forge token variables (`--surface-*`, `--fg-*`, `--color-focus-primary`, `--font-sans`, `--font-mono`) inside `@layer astryx-theme` and `@scope ([data-astryx-theme="neutral"]) to ([data-astryx-theme])` [theme.css].

## Interface

- `export const seaForgeTheme = neutralTheme` — the only TypeScript export; imported from `@astryxdesign/theme-neutral/built` [theme.ts:seaForgeTheme]
- `theme.css` is consumed as a stylesheet (it defines no classes; it customizes CSS custom properties within the Astryx theme scope) [theme.css:@scope].

## Invariants

- Astryx's `defineTheme` runtime style-injection path must not be used: it is incompatible with the packaged desktop application's strict Tauri CSP. The theme must remain the static `__built` neutral projection plus the static stylesheet beside it [theme.ts:comment, theme.css:header-comment].
- The theme name must remain `"neutral"` so Astryx's generated component CSS continues to apply [theme.ts:comment].
- The `theme.css` variable mapping is the binding layer between Astryx component CSS and the SEA Forge token set published by `@sea-forge/ui-tokens` (e.g. `--color-success: var(--color-authority-allowed)`, `--color-error: var(--color-danger)`, `--color-warning: var(--color-attention)`) [theme.css:@scope].

## Side Effects

- `["None"]` — the module performs no I/O, no DOM mutation, and no network calls. `theme.css` affects rendering only when loaded into the document by the consuming application.

## Verification

- No test script exists for this package (`../package.json` defines no `scripts`). Type-level assurance comes from the Workbench check gate: `just workbench-check` (repo root justfile).
