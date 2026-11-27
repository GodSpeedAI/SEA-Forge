# @sea-forge/contracts (contracts)

## Overview

Bun workspace package (`@sea-forge/contracts`, private, `"type": "module"`) that supplies the TypeScript/TypeScript-validatable contract layer between the SEA Forge Rust kernel and the Workbench renderer. The Rust side emits JSON Schemas into the repository-level `contracts/schema/` (the single source of truth, never hand-edited); the generator in `scripts/` compiles each schema into a TypeScript interface and a precompiled AJV validator under `generated/`, which is the package's only public entry point (`exports["."] = "./generated/index.ts"`) [package.json:exports].

Dependencies: `ajv` (8.20.0) and `json-schema-to-typescript` (15.0.4) [package.json:dependencies].

## Interface

- `./generated/` — generated zone (no README: the generator owns the directory): 64 `<Name>.ts` interfaces, 64 `<Name>.validator.ts` AJV validators, and the `index.ts` barrel re-exporting `type { Name }` and `validate as validateName` for every schema in `contracts/schema/`.
- [./scripts/README.md] — `generate.ts`, the SFWP contract generator (`bun run generate`).
- `../../../contracts/schema/` (repository level, not inside this package) — 64 Rust-emitted `*.schema.json` files, read by `scripts/generate.ts`; not in this contract set's write scope. Examples: `CaseSummary.schema.json`, `RunRecord.schema.json`, `PreflightResult.schema.json`, `CaseTaskApproval.schema.json`.
- Package script: `generate` → `bun run scripts/generate.ts` [package.json:scripts].

## Invariants

- `contracts/schema/*.schema.json` is the single source of truth; the contents of `generated/` are fully derived and must regenerate byte-identically (deterministic, timestamp-free output) [scripts/generate.ts:BANNER].
- Generated validators are precompiled AJV standalone code so the packaged renderer never needs `unsafe-eval` under its strict CSP [scripts/generate.ts:main].
- Repository gate: `just workbench-contracts-gate` regenerates and fails if `packages/contracts/generated` drifts from the committed schemas or contains untracked files [justfile:workbench-contracts-gate].
- `generated/` must never be hand-edited.

## Side Effects

- Only via `scripts/generate.ts`: deletes/recreates `generated/` and writes the derived `.ts` files there [scripts/generate.ts:rmSync, scripts/generate.ts:writeFileSync]. The package's runtime exports are pure modules.

## Verification

- Run: `just workbench-contracts-gate` (regeneration + drift check); direct form: `cd workbench && bun run generate:contracts`. The package has no test script of its own.
