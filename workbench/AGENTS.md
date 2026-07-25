# SEA Forge Workbench Agent Guide

This directory is the desktop frontend: a Tauri 2 host + React 19 renderer
that talks to `sea-forge-server` only through a typed Rust bridge over the
existing Unix-socket NDJSON surface. It is a separate Bun workspace and a
separate Cargo workspace from the repository root — see
`docs/decisions/ADR-004-workbench-stack.md` for why, and the root `AGENTS.md`
for the kernel this frontend is a client of.

For any Workbench task, load `.claude/skills/building-sea-forge-workbench/SKILL.md`
first; this file only covers local commands and boundaries.

## Commands

```sh
# From the repository root
just workbench-check          # install (frozen) + check + build + test

# From workbench/
bun install
bun run dev                   # renderer only (Vite), no desktop window
bun run check                 # tsc --noEmit + oxlint
bun run build                 # tsc -b + vite build
bun run test                  # vitest run

# From workbench/apps/desktop
bun run tauri dev             # launches the desktop window
bun run tauri build           # packaged bundle (Task 14)
cargo build --manifest-path src-tauri/Cargo.toml   # host crate only
```

The kernel gates (`devbox run -- just check`, `just test`) never need Bun and
must stay green regardless of anything in this directory.

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
  emitted by `cargo run -p sea-forge-server --bin gen_sfwp_schema` from the
  types listed in `crates/sea-forge-server/src/sfwp/mod.rs`'s `SCHEMA_TYPES`.
  Never hand-edit; regenerate on the Rust side, not here.
- `packages/contracts/generated/` (from Task 3 on): `@sea-forge/contracts`,
  built from `packages/contracts/schema/` by `bun run generate:contracts`
  (`packages/contracts/scripts/generate.ts`, using `json-schema-to-typescript`
  + `ajv` — see `docs/decisions/ADR-005-sfwp-schema-generation.md`). One
  `<Name>.ts` type and one `<Name>.validator.ts` AJV validator per schema,
  plus a barrel `index.ts`. Never hand-edit; regenerate and commit the diff.
  `crates/sea-forge-server/tests/conformance_sfwp.rs`'s
  `generated_schemas_are_committed_and_current` test fails the Rust gate if a
  contract type changes without regenerating the schema; the plan's Task 3
  gate (`bun run generate:contracts && git diff --exit-code
  packages/contracts/generated`) fails the frontend gate the same way on the
  TS side.
- `packages/sea-forge-ui-tokens/sea-forge.tokens.css` is a byte-for-byte
  copy-projection of `.agents/specs/frontend/colors_and_type.css`. Never
  hand-edit it directly — edit the spec source and re-copy; `bun run
  check-drift` (in that package) fails if they diverge.
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
