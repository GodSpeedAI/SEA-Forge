# sea-forge-cli

## Overview
SEA Forge one-shot CLI: run, validate, inspect, recall (Cargo.toml). The
binary `sea-forge` is the human/operator command surface of the governed
capability-execution kernel: it parses a clap grammar, dispatches to thin
command adapters, and exits with the governed dispatch code. Governed
mutations live in shared kernel crates (`sea-forge-case-runner`,
`sea-forge-core`) so the SFWP server and this CLI execute the same
implementation.

## Interface
- Binary target: `sea-forge` [Cargo.toml:[[bin]]].
- Command grammar and dispatch: `main`, `dispatch` [src/main.rs].
- Pipelines: `run_intent` (intent runs), `run_plan`/`run_plan_with_approval_extension` (plan runs) [src/pipeline.rs], [src/plan_pipeline.rs].
- Approvals view: `append`, `check_expiry`, `latest_status`, `load_all`, `pending_for_case` (re-exports from `sea_forge_core::approvals`) [src/approvals.rs].
- Subcommand implementations: see [src/commands/README.md].
- Source tree contract: [src/README.md]

## Invariants
- `#![forbid(unsafe_code)]` [src/main.rs].
- The approvals journal fold has a single owner in `sea-forge-core`; the CLI never re-implements it [src/approvals.rs:module docs].
- Governed case mutations have exactly one shared implementation used by CLI and server [src/commands/case.rs:module docs].
- Process exit code equals the dispatch result [src/main.rs:main].

## Side Effects
- Creates run/case state, appends to case ledgers and JSONL traces/evidence, writes artifacts under the state root [src/pipeline.rs:run_intent].
- Executes sandboxed task episodes via the sandbox/runtime crates [src/plan_pipeline.rs].
- Spawns governed side effects only through authorized kernel paths (mediated authority, approvals, case ops) [src/commands/README.md].

## Verification
- Run: `cargo test -p sea-forge-cli` (repo-wide: `just test`)
