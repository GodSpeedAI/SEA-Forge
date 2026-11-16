# workbench/packages

## Overview

Workspace-packages root of the SEA Forge Workbench (Bun workspace, React 19 renderer). This directory holds no direct source: it groups the four private `@sea-forge/*` workspace packages that carry the renderer's contract layer, design tokens, theme adaptation, and governed-UI component library. All packages are TypeScript/ESM (`"type": "module"`) and are wired together with `workspace:*` dependencies (`@sea-forge/ui-components` depends on `@sea-forge/astryx-theme` and `@sea-forge/ui-tokens`) [sea-forge-ui-components/package.json:dependencies]. Contract generation and token drift for these packages are gated repo-wide by `just workbench-contracts-gate` [justfile:workbench-contracts-gate].

## Subdirectories & Boundaries

- `contracts/`: [./contracts/README.md] — `@sea-forge/contracts`: TypeScript interfaces and precompiled AJV validators generated from the Rust-emitted JSON Schemas in `schema/`; only public surface is `./generated/index.ts`. `generated/` is a generated zone (generator: `scripts/generate.ts`); `scripts/` holds that generator.
- `sea-forge-astryx-theme/`: [./sea-forge-astryx-theme/README.md] — `@sea-forge/astryx-theme`: CSP-safe static adaptation of the Astryx "neutral" theme (`seaForgeTheme` re-export plus a static CSS variable projection in `src/`).
- `sea-forge-ui-components/`: [./sea-forge-ui-components/README.md] — `@sea-forge/ui-components`: nine React governed-UI components (`GovernedStatusPill`, `DualStateIndicator`, `SourceFreshnessBadge`, `IntegrityIndicator`, `ProtectedActionButton`, `WhyStatePanel`, `EvidenceDrawer`, `AuthorityBoundaryPanel`, `AvailabilityLadder`) with CSS Modules, vitest suite, and Storybook stories under `src/stories/`.
- `sea-forge-ui-tokens/`: [./sea-forge-ui-tokens/README.md] — `@sea-forge/ui-tokens`: canonical design-token stylesheet (`sea-forge.tokens.css`) published as `./tokens.css`; enforced byte-copy of `/.agents/specs/frontend/colors_and_type.css` via `scripts/check-drift.mjs`.

## Verification

- Run: `bun run test` from `workbench/` (workspace tests; the vitest suite lives in `sea-forge-ui-components`). Repo gates: `just workbench-check` (includes the contracts/token drift gate `just workbench-contracts-gate`).
