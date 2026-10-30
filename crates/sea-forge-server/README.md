# sea-forge-server

## Overview

SEA Forge concurrent case server (spec-full §10.3, §11.1, M3). The server
listens on a Unix domain socket (Windows: named pipe), accepts NDJSON
requests, and dispatches case runs as subprocesses via `spawn_blocking`.
Kernel logic stays synchronous; Tokio lives only here, making this one of the
two async-edge crates in the workspace (`src/lib.rs` module doc;
`crates/AGENTS.md`). The crate also hosts the SFWP Workbench protocol layer,
identity/peer authentication, delegation execution with audit, the governed
CEP boundary contracts, MCP serving, and Postgres session brokering.

## Interface

- Crate contract lives in [./src/README.md](./src/README.md) (module map with
  per-module symbols).
- Key entry points, cited from source:
  - Server loop: `run(config: ServerConfig)` — `src/lib.rs:run`;
    request handling — `src/lib.rs:handle_request`; request protocol —
    `src/lib.rs:Request`; state — `src/lib.rs:ServerState`.
  - Configuration: `ServerConfig` — `src/config.rs:ServerConfig` (loaded from
    `SEA_FORGE_ROOT` / `SEA_FORGE_SOCKET` / `<root>/server.yaml`).
  - SFWP layer: `src/sfwp` — see [./src/sfwp/README.md](./src/sfwp/README.md).
  - MCP serving: `src/mcp.rs:run` and the `sea-forge-mcp` binary —
    see [./src/bin/README.md](./src/bin/README.md).
- Example programs: [./examples/README.md](./examples/README.md)
  (`journey_gauntlet_bootstrap` cell bootstrap for the UX journey gauntlet).

## Invariants

- Async only at this edge: kernel crates must stay synchronous
  (`lib.rs` module doc; enforced repo-wide by `just no-async-kernel`).
- No CLI contract: `src/main.rs:reject_command_line_arguments` refuses every
  argument before configuration loads (`src/main.rs:NO_ARGUMENTS_MESSAGE`).
- The server refuses every protected verb until identity bindings are
  configured (`src/identity.rs` `Unconfigured` refusal).
- SFWP evolves additively per ADR-003 — new verbs, no reshapes, no wrapper
  frames (`src/sfwp/mod.rs` module doc).
- `#![forbid(unsafe_code)]` at the crate root (`src/lib.rs`, `src/main.rs`).

## Side Effects

- Binds and serves a control-plane socket (Unix domain socket / Windows named
  pipe) (`src/lib.rs:Acceptor`).
- Spawns case-run subprocesses and delegation/agent processes
  (`src/case_dispatch.rs`, `src/delegation.rs`).
- Appends to cell ledgers (case events, delegation audit, sealed transcripts,
  SWE-seed declarations) and reads/writes cell state under the root
  (`src/delegation_audit.rs`, `src/transcript_seal.rs`,
  `src/swe_seed_reconciliation.rs`).
- Makes outbound connections to agent endpoints and Postgres
  (`src/agent_probe.rs:probe`, `src/postgres.rs:PostgresBroker`).

## Verification

- Run: `cargo test -p sea-forge-server` (repo-wide: `just test`)
