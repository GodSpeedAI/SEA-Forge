# sea-forge-spec-pipeline — src

## Overview

Implementation of the `sea-forge-spec-pipeline` crate: SEA Forge spec-to-code
pipeline (spec-full §7.8, §10.7, M5) and DomainForge projections. Provides the
pure logic of the pipeline — deterministic stage chaining with hashes, stage
ordering and prerequisites, proof classification, quarantine, the
generated-zone guard, and projection record building. The pipeline runs as
ordinary governed work — no separate engine; this crate provides the pure
logic, and the CLI wires it into the case-run lifecycle (`src/lib.rs` module
doc).

## Interface

- Stage hashing (`lib.rs`):
  - `compute_stage_hash(stage)` — content hash of a single stage (covers
    stage_id, kind, inputs, outputs, command, status; excludes
    quarantine_ref and settlement_basis governance metadata)
    (`lib.rs:compute_stage_hash`).
  - `compute_chain_hash(stages)` — hash over the ordered stage chain
    (`lib.rs:compute_chain_hash`).
  - `compute_input_hash(source_refs)` / `hash_content(content)` —
    (`lib.rs:compute_input_hash`, `lib.rs:hash_content`).
- Stage validation and ordering (`lib.rs`):
  - `validate_stage_order(stages)` (`lib.rs:validate_stage_order`).
  - `validate_stage_prerequisites(...)` (`lib.rs:validate_stage_prerequisites`).
  - `quarantine_stage(stage, reason)` (`lib.rs:quarantine_stage`).
- Proof classification: `compute_proof_classification(stages) ->
  ProofClassification` (`lib.rs:compute_proof_classification`).
- Generated-zone guard (`lib.rs`):
  - `is_generated_zone(path)` (`lib.rs:is_generated_zone`).
  - `check_generated_zone_edit(path)` — reject edits to generated zones
    (`lib.rs:check_generated_zone_edit`).
- Projections (`lib.rs`):
  - `project_model(...)` (`lib.rs:project_model`).
  - `compute_rebuild_hash(...)` (`lib.rs:compute_rebuild_hash`).
  - `build_projection_record(...)` (`lib.rs:build_projection_record`).
- Run processing and byte identity:
  - `process_pipeline(run: &mut SpecPipelineRun)` (`lib.rs:process_pipeline`).
  - `verify_byte_identity(a, b) -> Result<bool, ForgeError>` —
    (`lib.rs:verify_byte_identity`).

## Invariants

- Stage hashes are deterministic and content-derived: they cover stage_id,
  kind, inputs, outputs, command, status, and exclude governance metadata
  (quarantine_ref, settlement_basis) (`lib.rs:compute_stage_hash`).
- Stage order and prerequisites are validated before processing; violations
  are `ForgeError`s, not warnings (`lib.rs:validate_stage_order`,
  `lib.rs:validate_stage_prerequisites`).
- Generated zones may not be hand-edited: `check_generated_zone_edit` rejects
  edits to paths classified as generated (`lib.rs:check_generated_zone_edit`).
- Pure logic crate: `#![forbid(unsafe_code)]`; no engine of its own — the CLI
  wires it into the case-run lifecycle as governed work
  (`src/lib.rs` module doc).

## Side Effects

- ["None"] — the crate implements pure functions over `SpecPipelineRun` /
  `SpecPipelineStage` values and in-memory file lists; it performs no disk,
  network, process, or lock operations (`src/lib.rs`).

## Verification

- Run: `cargo test -p sea-forge-spec-pipeline` (repo-wide: `just test`)
