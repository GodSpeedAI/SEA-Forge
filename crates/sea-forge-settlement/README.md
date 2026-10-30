# sea-forge-settlement

## Overview

SEA Forge settlement evaluation. Turns settlement claims — authority verdicts
plus execution results — into `SettlementEvent`s, and owns the settlement
declaration ledger: hash-computed, append-only JSONL declarations with
pluggable settlement authority (local, or SWE-seed via an external transport
command).

## Interface

- Crate contract lives in [./src/README.md](./src/README.md) (modules
  `lib.rs`, `declaration.rs`).
- Key entry points, cited from source:
  - `settle(claim, workspace, run_dir)` — `src/lib.rs:settle`.
  - `evaluate_agent_output` / `validate_response_schema` / `evaluate_batch` —
    `src/lib.rs:evaluate_batch`.
  - Declaration ledger: `append_declaration`,
    `append_declaration_ledgered`, `append_declaration_ledgered_once`,
    `compute_declaration_hash`, `load_declarations` —
    `src/declaration.rs:append_declaration`.
  - Authority: `SettlementAuthority`, `LocalSettlementAuthority`,
    `SweSeedSettlementAuthority<T>`, `SweSeedTransport`,
    `CommandSweSeedTransport` — `src/declaration.rs:SettlementAuthority`.

## Invariants

- Write-only items settle on the honest `write_only` basis, never a fabricated
  process result (F-10, `src/lib.rs:settle`).
- Declarations are append-only and canonically hash-chained (hash excludes
  `declaration_hash`) (`src/declaration.rs:compute_declaration_hash`).

## Side Effects

- Appends settlement declarations to JSONL ledgers under the run directory
  (`src/declaration.rs:append_declaration`).
- Spawns the external swe-seed transport command when the SWE-seed authority
  is used (`src/declaration.rs:CommandSweSeedTransport`).

## Verification

- Run: `cargo test -p sea-forge-settlement` (repo-wide: `just test`)
