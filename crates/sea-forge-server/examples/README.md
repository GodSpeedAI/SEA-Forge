# sea-forge-server — examples

## Overview

Runnable example programs for the `sea-forge-server` crate. These are
build targets (`cargo run -p sea-forge-server --example <name>`) that
bootstrap real cell state for end-to-end harnesses; they are not part of the
library contract.

## Interface

- `journey_gauntlet_bootstrap` (`journey_gauntlet_bootstrap.rs`): one-shot cell
  bootstrap for the UX Journey Settlement Gauntlet
  (`.agents/reports/ux-journey-settlement/harness/gauntlet_runner.py`). Writes
  exactly the files a fresh cell root needs before the real
  `sea-forge-server` binary is worth starting — a `server.yaml`, a template
  file, and a `policy.yaml` — using the same real types the server loads
  (`ServerConfig`, `IdentityBindings`, `sequential_agents_template`), not
  hand-authored YAML (`journey_gauntlet_bootstrap.rs` module doc).
  - Run: `cargo run -p sea-forge-server --example journey_gauntlet_bootstrap`
  - Env: `SEA_FORGE_ROOT` (required, empty/fresh cell root);
    `SEA_FORGE_GAUNTLET_ENDPOINT_URL` (required, base URL of a running
    OpenAI-compatible stub endpoint kept alive by the harness)
    (`journey_gauntlet_bootstrap.rs` module doc).
  - Exercises the built-in template `sequential_agents@0.1.0`
    (`journey_gauntlet_bootstrap.rs:TEMPLATE_REF`), proven end to end by
    `crates/sea-forge-server/tests/conformance_case_authoring.rs`.

## Invariants

- The bootstrap uses the real server types (`ServerConfig`,
  `IdentityBindings`, `sequential_agents_template`) so the produced YAML
  cannot drift from the real schema (`journey_gauntlet_bootstrap.rs` module doc).
- It only writes into the cell root given via `SEA_FORGE_ROOT` and then exits;
  the harness starts the actual `sea-forge-server` binary against that root
  (`journey_gauntlet_bootstrap.rs` module doc).

## Side Effects

- Writes `server.yaml`, an identity binding configuration, a template file,
  and `policy.yaml` under the `SEA_FORGE_ROOT` cell root
  (`journey_gauntlet_bootstrap.rs` module doc, via
  `sea_forge_server::config::SOCKET_FILE_NAME`).

## Verification

- Run: `cargo test -p sea-forge-server` (repo-wide: `just test`)
