# SEA Forge Workbench Agent Guide

This directory is the Workbench frontend: a React 19 renderer built with Vite
and Bun. It is a separate Bun workspace from the repository root — see
`docs/decisions/ADR-004-workbench-stack.md` for the stack decision (its status
update records that the Tauri desktop shell was removed on 2026-10-13), and the
root `AGENTS.md` for the kernel this frontend is a client of.

The Tauri 2 host (`apps/desktop/src-tauri`) was removed on 2026-10-13. The
Workbench is a web app now. The renderer still calls `@tauri-apps/api`
(`invoke`/`listen`) for its data path, so it has no backend until that bridge is
replaced; that follow-up is tracked in `.agents/OBSERVED_DEBT.md`.

For any Workbench task, load `.agents/skills/building-sea-forge-workbench/SKILL.md`
first. Use `just` from the repository root; it is the only command interface
agents should invoke.

## Commands

```sh
just workbench-check              # frozen install + generated-contract gate + renderer typecheck, build, tests
just workbench-dev-up             # renderer-only Vite server
just workbench-dev-down
just workbench-contracts-generate # regenerate Rust schemas and TS/AJV projections
just workbench-contracts-gate     # verify generated-zone drift and UI token copy
```

The kernel gates (`just check`, `just test`) never need Bun and must stay green
regardless of anything in this directory.

## E2E Testing & Evidence Rules

- `just workbench-e2e-agent-browser` is renderer and accessibility evidence only.
- `just workbench-e2e-case-authoring` uses a mocked `__TAURI_INTERNALS__` IPC bridge; it does not prove integrated server behavior.
- No recipe currently provides integrated (real server + real bridge) evidence for the renderer.

## Boundaries

- **The renderer never touches the Unix socket, `.sea-forge/` files, or SQL
  directly.** Backend access goes through a typed bridge, never a generic
  `invoke_backend(method, arbitrary_json)` escape hatch. The bridge is currently
  absent (see above).
- **TanStack Query owns inspection views, never canonical mutation.**
  Protected commands go through the SFWP command client; views update from
  source-backed state, not optimistic writes.
- **XState machines are frontend interaction state only.** They never
  duplicate canonical backend reducers or invent settlement/authority truth.
- Astryx (`@astryxdesign/*`) is design substrate, not the semantic owner.
  SEA Forge state vocabulary, canonical tokens, and domain meaning are owned
  here, projected onto Astryx via `packages/sea-forge-astryx-theme` — never
  fork Astryx's generated component CSS.

## Generated-zone rules

- `packages/contracts/schema/*.schema.json` (from Task 3 on): JSON Schema
  emitted from the types listed in `crates/sea-forge-server/src/sfwp/mod.rs`'s
  `SCHEMA_TYPES`. Never hand-edit; run `just workbench-contracts-generate`.
- `packages/contracts/generated/` (from Task 3 on): `@sea-forge/contracts`,
  built from `packages/contracts/schema/` by `just workbench-contracts-generate`
  (`packages/contracts/scripts/generate.ts`, using `json-schema-to-typescript`
  + Ajv standalone generation — see `docs/decisions/ADR-005-sfwp-schema-generation.md`).
  One `<Name>.ts` type and one static `<Name>.validator.ts` per schema, plus a
  barrel `index.ts`. Validators must not compile schemas at browser runtime
  (a strict CSP is expected for the renderer). Never hand-edit; regenerate and
  commit the diff.
  `crates/sea-forge-server/tests/conformance_sfwp.rs`'s
  `generated_schemas_are_committed_and_current` test fails the Rust gate if a
  contract type changes without regenerating the schema; the plan's Task 3
  gate (`just workbench-contracts-gate`) fails the frontend gate the same way
  on the TS side.
- `packages/sea-forge-ui-tokens/sea-forge.tokens.css` is a byte-for-byte
  copy-projection of `.agents/specs/frontend/colors_and_type.css`. Never
  hand-edit it directly — edit the spec source and re-copy; the contracts gate
  fails if they diverge.
- `apps/desktop/dist/` and `node_modules/` are build output — see `.gitignore`,
  never commit.

## Workspace layout

```text
workbench/
  package.json, bunfig.toml          # Bun workspace root
  apps/desktop/                      # Vite + React 19 renderer
  packages/
    sea-forge-ui-tokens/             # canonical token copy-projection
    sea-forge-astryx-theme/          # SEA Forge tokens projected onto Astryx
    sea-forge-ui-components/         # semantic components (from Task 4)
    contracts/
      schema/                        # Rust-emitted JSON Schema (Task 3)
      generated/                     # generated TS types + AJV validators (Task 3)
```
