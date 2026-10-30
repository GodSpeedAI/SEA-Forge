# Workbench Blueprint

## Overview
Bun workspace for the SEA Forge Workbench frontend: the React 19 web renderer (`apps/desktop`) plus four private `@sea-forge/*` packages (contracts, astryx-theme, ui-components, ui-tokens). The Tauri desktop shell was removed on 2026-10-13; the renderer talks to the kernel host bridge instead. Generated schemas/tokens are produced by package scripts and must not be hand-edited.

## Subdirectories & Boundaries
- [apps](./apps/README.md) - Groups the single Workbench frontend application, `desktop`.
- [packages](./packages/README.md) - Groups the four private `@sea-forge/*` workspace packages, including the SFWP contracts generated zone.

## Global Invariants
- Generated zones are regenerated, never hand-edited: `packages/contracts/generated` is emitted by `packages/contracts/scripts/generate.ts` (drift gate: `just workbench-contracts-gate`).
- `sea-forge.tokens.css` must stay a byte-copy of `/.agents/specs/frontend/colors_and_type.css` (enforced by `packages/sea-forge-ui-tokens/scripts/check-drift.mjs`).

## Global Side Effects
- `None` at this level; build/dev commands (`bun run dev|build|check|test`) are declared in the child contracts.

## Verification
- Run: `bun run test` (from `workbench/`); full Workbench gate: `just workbench-check` (see `workbench/AGENTS.md`).
