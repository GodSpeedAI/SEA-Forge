# sea-forge-server — src/bin

## Overview

Helper executables built from the `sea-forge-server` crate. The crate's real
server entry point is `src/main.rs` (the `sea-forge-server` binary); the two
binaries here cover MCP serving and SFWP schema generation.

## Interface

- `sea-forge-mcp` (`sea-forge-mcp.rs:main`): MCP server entry point. Takes
  exactly one argument, a trusted config JSON path
  (`usage: sea-forge-mcp <trusted-config.json>`), and delegates to
  `sea_forge_server::mcp::run(config_path)` (`sea-forge-mcp.rs:main`).
  Refuses extra arguments.
- `gen_sfwp_schema` (`gen_sfwp_schema.rs:main`): SFWP JSON Schema generator
  (Task 3, Rust half). Emits one `<TypeName>.schema.json` per SFWP-layer
  contract type into `workbench/packages/contracts/schema/` using `schemars`
  over the exported `sea_forge_server::sfwp::*` types
  (`gen_sfwp_schema.rs` module doc). Regenerated deterministically via
  `just workbench-contracts-generate`; a drift test fails if a contract type
  changes without regeneration.

## Invariants

- `sea-forge-mcp` accepts exactly one argument; any other argument count is a
  usage error (`sea-forge-mcp.rs:main`).
- `gen_sfwp_schema` output must be byte-deterministic: schemas are generated
  from the Rust type definitions and a drift test enforces regeneration after
  contract changes (`gen_sfwp_schema.rs` module doc).

## Side Effects

- `gen_sfwp_schema` writes schema files into
  `workbench/packages/contracts/schema/` (`gen_sfwp_schema.rs` module doc).
- `sea-forge-mcp` starts listeners/servers per its trusted config via
  `sea_forge_server::mcp::run` (`sea-forge-mcp.rs:main`).

## Verification

- Run: `cargo test -p sea-forge-server` (repo-wide: `just test`)
