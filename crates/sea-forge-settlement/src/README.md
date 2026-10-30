# sea-forge-settlement — src

## Overview

Implementation of the `sea-forge-settlement` crate: SEA Forge settlement
evaluation. Evaluates settlement claims into `SettlementEvent`s, and provides
the settlement-declaration ledger: hash-chained declarations appended to
JSONL, with pluggable settlement authority (local authority or SWE-seed
transport-backed authority).

## Interface

- Module tree: `declaration` (`src/lib.rs:declaration`), re-exported by
  `lib.rs` (`lib.rs:pub use declaration::{...}`).
- Claim evaluation (`lib.rs`):
  - `settle(claim, workspace, run_dir) -> Result<SettlementEvent, ForgeError>`
    (`lib.rs:settle`): evaluates authority verdicts and execution results into
    a `SettlementEvent`. Escalate verdicts escalate, Deny rejects, write-only
    items settle with an honest `write_only` basis — never a fabricated
    process result (F-10, `lib.rs:settle`).
  - `evaluate_agent_output(output, required) -> Option<bool>`
    (`lib.rs:evaluate_agent_output`).
  - `validate_response_schema(schema, output) -> bool`
    (`lib.rs:validate_response_schema`).
  - `evaluate_batch(...)` (`lib.rs:evaluate_batch`).
- Declaration ledger (`declaration.rs`):
  - `compute_declaration_hash(decl)` — canonical hash excluding
    `declaration_hash` (`declaration.rs:compute_declaration_hash`).
  - `append_declaration(path, decl)` — append one declaration to a JSONL file
    (`declaration.rs:append_declaration`).
  - `append_declaration_ledgered(...)` /
    `append_declaration_ledgered_once(...)` — append through a
    `LedgerStream`, the `_once` variant guarding against duplicates
    (`declaration.rs:append_declaration_ledgered`,
    `declaration.rs:append_declaration_ledgered_once`).
  - `load_declarations(path)` — read declarations back
    (`declaration.rs:load_declarations`).
  - Authority abstraction: trait `SettlementAuthority`
    (`declaration.rs:SettlementAuthority`), `LocalSettlementAuthority`
    (`declaration.rs:LocalSettlementAuthority`).
  - SWE-seed authority: `SweSeedTransport` trait (`declaration.rs:SweSeedTransport`),
    `CommandSweSeedTransport` (`declaration.rs:CommandSweSeedTransport`),
    `SweSeedSettlementAuthority<T>` (`declaration.rs:SweSeedSettlementAuthority`),
    `SweSeedResponse` (`declaration.rs:SweSeedResponse`).

## Invariants

- A write-only item has no process result to settle; it settles with the
  honest `write_only` basis — never a fabricated process result (F-10,
  `lib.rs:settle`).
- Declaration hash covers canonical content excluding `declaration_hash`
  (`declaration.rs:compute_declaration_hash`).
- The declaration ledger is append-only JSONL; duplicate-sensitive appends go
  through `append_declaration_ledgered_once`
  (`declaration.rs:append_declaration_ledgered_once`).

## Side Effects

- Disk writes: `append_declaration` / `append_declaration_ledgered` append
  declarations to JSONL ledger files under the run directory
  (`declaration.rs:append_declaration`).
- Disk reads: `settle` reads workspace/run artifacts to build the settlement
  basis (`lib.rs:settle`); `load_declarations` reads a JSONL ledger
  (`declaration.rs:load_declarations`).
- Process spawns: `CommandSweSeedTransport` runs the configured external
  swe-seed transport command (`declaration.rs:CommandSweSeedTransport`).
- Shared state: the swe-seed transport serializes concurrent invocations with
  an `AtomicBool` guard and threads (`declaration.rs` imports:
  `AtomicBool`, `Arc`, `thread`).

## Verification

- Run: `cargo test -p sea-forge-settlement` (repo-wide: `just test`)
