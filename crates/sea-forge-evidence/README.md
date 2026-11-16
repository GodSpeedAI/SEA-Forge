# sea-forge-evidence

## Overview
SEA Forge crate for evidence and artifact capture (`Cargo.toml` description). Provides hashing/canonicalization primitives shared across the kernel, a sequence-checked JSONL evidence journal with power-loss-safe durability, and artifact capture that copies outputs into the run's artifacts directory with symlink-safe writes.

## Interface
- Source contract: [./src/README.md] — `sha256_bytes`, `sha256_file`, `canonical_json`, `hash_canonical`, `JsonlEvidenceWriter`, `capture_file` [src/lib.rs:JsonlEvidenceWriter], [src/lib.rs:capture_file]
- Workspace manifest: `Cargo.toml` (package `sea-forge-evidence`; depends on `sea-forge-core` and `sea-forge-sandbox` 0.3.0)

## Invariants
- Evidence journals are created exclusively (`create_new`) and every append is sequence-checked, keeping the journal gap-free [src/lib.rs:create], [src/lib.rs:append_prepared]
- Durability discipline (F-25.r): appends flush and `sync_data` — flush alone is kill-9-safe, not power-loss-safe [src/lib.rs:append_prepared]
- Artifact capture writes through the sandbox's no-follow primitives (`safe_join`, `safe_write_within`) so a symlink swapped in at the artifact path cannot redirect the capture (F-25.i) [src/lib.rs:capture_file]
- Canonicalization delegates to the single shared primitive in `sea-forge-core` (SUP-09d) [src/lib.rs:canonical_json]

## Side Effects
- Filesystem: creates the artifacts directory, copies artifact files, creates and appends to the evidence JSONL journal, and syncs it per append [src/lib.rs:capture_file], [src/lib.rs:JsonlEvidenceWriter]

## Verification
- Run: `cargo test -p sea-forge-evidence` (repo-wide: `just test`)
