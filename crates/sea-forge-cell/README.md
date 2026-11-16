# sea-forge-cell

## Overview
SEA Forge SeaCell federation: cell identity, bundle export/import
(spec-full §7.4, Cargo.toml). Synchronous kernel crate with no network and no
async; `#![forbid(unsafe_code)]` [src/lib.rs]. Owns the per-cell identity
record, evidence-relevant run bundling with atomic-reject import, and gated
template adoption.

## Interface
- Modules: `bundle` (tar export/import), `cell` (identity), `template` (adoption) [src/lib.rs:pub mod].
- Re-exports: `export`, `import`, `imported_template_path`, `read_manifest`, `ensure`, `read_cell`, `is_known_schema`, `CellRecord`, `adopt` [src/lib.rs:pub use].
- Source tree contract: [src/README.md]

## Invariants
- Cell ids are `cell_<8 hex>` under the state root, schema `cell.v1` [src/cell.rs:SCHEMA_VERSION].
- Bundle reads are memory-bounded at 256 MiB and enforced by the read itself; bundle bytes are untrusted by construction (§14.8) [src/bundle.rs:MAX_BUNDLE_BYTES].
- Export fails closed when requested runs resolve to zero evidence files (F-12) [src/bundle.rs:export].
- Import is atomic-reject via staging + rename [src/bundle.rs:import].
- Imported templates are inert until adopted; adoption rejects traversal-shaped cell ids (F-14) and copies (never moves) to preserve provenance [src/template.rs:adopt].

## Side Effects
- Writes `cell.json`, bundle tars, staged import directories, and adopted template copies under the state root [src/cell.rs:ensure], [src/bundle.rs:import], [src/template.rs:adopt].

## Verification
- Run: `cargo test -p sea-forge-cell` (repo-wide: `just test`)
