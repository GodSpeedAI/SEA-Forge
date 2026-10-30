# contracts/scripts

## Overview

Build tooling for the `@sea-forge/contracts` package. The single script, `generate.ts`, is the SFWP contract generator: it is the only sanctioned way to produce the contents of `../generated/` (a generated zone, documented in [../generated/README.md]). It reads the Rust-emitted JSON Schemas in `../schema/*.schema.json`, which are the single source of truth and are never hand-edited, and emits a TypeScript interface plus a precompiled AJV validator for each schema, plus an `index.ts` barrel [generate.ts:main].

## Interface

- `main()` — async entry point invoked at module load; invoked via the package script `bun run scripts/generate.ts` [package.json:scripts.generate] [generate.ts:main]
- Uses `compile` from `json-schema-to-typescript` and `Ajv2020` + `standaloneCode` from `ajv/dist/2020.js` / `ajv/dist/standalone/index.js` [generate.ts:imports]
- No exported public symbols; the script is a build step, not a library.

## Invariants

- Deterministic, byte-identical output: schemas are read in sorted order, the banner is timestamp-free, and barrel exports are sorted; regenerating with unchanged schemas must produce a byte-identical diff — this is the drift-detection contract the `just workbench-contracts-gate` relies on [generate.ts:main].
- `../schema/*.schema.json` is the single source of truth; only files ending in `.schema.json` are processed [generate.ts:main].
- The output directory is rebuilt from scratch so deleted schemas leave no stale files [generate.ts:rmSync].
- Validators must be precompiled with AJV standalone code because the packaged Tauri renderer has a strict CSP and must never need `unsafe-eval` to validate trusted server responses [generate.ts:main].
- Compilation options are fixed: `additionalProperties: false`, `declareExternallyReferenced: true`, `allErrors: true`, `strict: false`, `code: { esm: true, source: true }` [generate.ts:compile, generate.ts:Ajv2020].
- On any error the script writes the stack to stderr and exits with status 1 [generate.ts:main.catch].

## Side Effects

- Deletes and recreates `../generated/` (`rmSync` + `mkdirSync`) [generate.ts:main].
- Writes `../generated/<Name>.ts`, `../generated/<Name>.validator.ts`, and `../generated/index.ts` for each schema [generate.ts:writeFileSync].
- Writes a status line to stdout listing the generated type count [generate.ts:process.stdout.write].
- No network access; reads only the local schema directory.

## Verification

- Run: `cd workbench && bun run generate:contracts && git diff --exit-code -- packages/contracts/generated` (canonical gate: `just workbench-contracts-gate` [justfile:workbench-contracts-gate]). The script itself has no dedicated unit test.
