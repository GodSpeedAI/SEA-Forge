# SEA Forge Workbench Agent Guide

This directory is the desktop frontend: a Tauri 2 host + React 19 renderer
that talks to `sea-forge-server` only through a typed Rust bridge over the
existing Unix-socket NDJSON surface. It is a separate Bun workspace and a
separate Cargo workspace from the repository root — see
`docs/decisions/ADR-004-workbench-stack.md` for why, and the root `AGENTS.md`
for the kernel this frontend is a client of.

For any Workbench task, load `.agents/skills/building-sea-forge-workbench/SKILL.md`
first. Use `just` from the repository root; it is the only command interface
agents should invoke.

## Commands

```sh
just workbench-check              # frozen install + renderer + host gates
just workbench-dev-up             # renderer-only Vite server
just workbench-dev-down
just workbench-tauri-dev          # desktop window with debug sidecar
just workbench-host-build         # standalone Tauri host crate
just workbench-tauri-test         # host lint, tests, and formatting
just workbench-contracts-generate # regenerate Rust schemas and TS/AJV projections
just workbench-contracts-gate     # verify generated-zone and boundary drift
just workbench-package            # release sidecar + packaged desktop bundle
```

The kernel gates (`just check`, `just test`) never need Bun and must stay green
regardless of anything in this directory.

## Boundaries

- **The renderer never touches the Unix socket, `.sea-forge/` files, or SQL
  directly.** All of that is the Tauri host's job
  (`apps/desktop/src-tauri/src/`), exposed to React through typed commands
  and channels — never a generic `invoke_backend(method, arbitrary_json)`
  escape hatch.
- **TanStack Query owns inspection views, never canonical mutation.**
  Protected commands go through the SFWP command client (Tauri command →
  host → socket); views update from source-backed state, not optimistic
  writes.
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
  barrel `index.ts`. Validators must not compile schemas at browser runtime:
  the packaged Tauri renderer has a strict CSP. Never hand-edit; regenerate and
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
- `apps/desktop/src-tauri/target/`, `apps/desktop/dist/`, and `node_modules/`
  are build output — see `.gitignore`, never commit.

## Workspace layout

```text
workbench/
  package.json, bunfig.toml          # Bun workspace root
  apps/desktop/                      # Vite + React 19 renderer
    src-tauri/                       # Tauri 2 host (own Cargo workspace)
      src/socket.rs                  # NDJSON client: call(), FIFO response pairing
      src/bridge.rs                  # closed SfwpQuery/SfwpCommand Tauri commands
      src/events.rs                  # durable cursor, reconnect, gap recovery
  packages/
    sea-forge-ui-tokens/             # canonical token copy-projection
    sea-forge-astryx-theme/          # SEA Forge tokens projected onto Astryx
    sea-forge-ui-components/         # semantic components (from Task 4)
    contracts/
      schema/                        # Rust-emitted JSON Schema (Task 3)
      generated/                     # generated TS types + AJV validators (Task 3)
```
