# sea-forge-spec-pipeline

## Overview

SEA Forge spec-to-code pipeline and DomainForge projections (spec-full §7.8,
§10.7, M5). Deterministic stage chaining, generated-zone guard, proof
classification, and DomainForge projections, all as pure logic. The pipeline
runs as ordinary governed work — no separate engine; the CLI wires this crate
into the case-run lifecycle (`src/lib.rs` module doc).

## Interface

- Crate contract lives in [./src/README.md](./src/README.md) (single module
  `src/lib.rs`).
- Key entry points, cited from source:
  - `compute_stage_hash` / `compute_chain_hash` / `compute_input_hash` —
    `src/lib.rs:compute_stage_hash`.
  - `validate_stage_order` / `validate_stage_prerequisites` /
    `quarantine_stage` — `src/lib.rs:validate_stage_order`.
  - `compute_proof_classification` —
    `src/lib.rs:compute_proof_classification`.
  - Generated-zone guard: `is_generated_zone` /
    `check_generated_zone_edit` — `src/lib.rs:check_generated_zone_edit`.
  - Projections: `project_model` / `build_projection_record` /
    `compute_rebuild_hash` — `src/lib.rs:project_model`.
  - `process_pipeline(run)` / `verify_byte_identity` —
    `src/lib.rs:process_pipeline`.

## Invariants

- Stage order and prerequisites are validated before processing
  (`src/lib.rs:validate_stage_order`).
- Generated zones are never hand-edited; edits are rejected
  (`src/lib.rs:check_generated_zone_edit`).
- `#![forbid(unsafe_code)]`; pure logic, no engine
  (`src/lib.rs` module doc).

## Side Effects

- ["None"] — pure functions over pipeline/projection types; no disk, network,
  process, or lock operations (`src/lib.rs`).

## Verification

- Run: `cargo test -p sea-forge-spec-pipeline` (repo-wide: `just test`)
