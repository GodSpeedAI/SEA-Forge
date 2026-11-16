# sea-forge-cell — src

## Overview
Source tree for SEA Forge SeaCell federation (spec-full §7.4): cell identity
(`cell.json`), tar bundle export/import with atomic-reject (§14.8), and
template adoption gating. Kernel crate: synchronous, no network, no async;
`#![forbid(unsafe_code)]` [lib.rs:module docs], [lib.rs:forbid(unsafe_code)].

## Interface
- Module tree, declared in [lib.rs]: `bundle` ([bundle.rs]), `cell` ([cell.rs]), `template` ([template.rs]).
- Re-exports from [lib.rs]: `export`, `import`, `imported_template_path`, `read_manifest` [bundle.rs]; `ensure`, `is_known_schema`, `read_cell` (`read`), `CellRecord` [cell.rs]; `adopt` [template.rs].

## Invariants
- Cell identity lives at `<state_root>/cell.json` with schema `cell.v1`; ids must be `cell_<8 hex>` (13 chars) or reads fail with `cell_id_error` [cell.rs:SCHEMA_VERSION], [cell.rs:read].
- The passed root is the state root (F-12): no extra `.sea-forge` prefix, avoiding a second cell identity [cell.rs:ensure F-12].
- Bundle bytes are untrusted by construction (§14.8): reads are capped at `MAX_BUNDLE_BYTES = 256 MiB`, enforced by the read itself (`take`), not a racy metadata check [bundle.rs:MAX_BUNDLE_BYTES], [bundle.rs:read_bundle_bytes].
- Export fails closed when runs were requested but zero run-evidence files resolve — a bundle silently containing no evidence is worse than an error (F-12) [bundle.rs:export].
- Only evidence-relevant run files are bundled; `workspace/` is transient scratch and is excluded [bundle.rs:RUN_EVIDENCE_FILES].
- Import is atomic-reject: entries land in staging and are moved into place by `fs::rename`, with prior imports cleared first (§14.8) [bundle.rs:import].
- Template adoption is inert until adopted (§10.6); `cell_id` is rejected at the boundary if not a valid id segment, blocking path traversal (F-14) [template.rs:adopt].
- Adoption copies rather than moves, preserving the imported provenance trail [template.rs:adopt].

## Side Effects
- Creates and writes `<root>/cell.json` atomically with `OpenOptions::create_new` on first call [cell.rs:ensure].
- Reads and writes tar bundles; creates/removes staging and final import directories (`fs::remove_dir_all`, `fs::rename`, `fs::create_dir_all`) [bundle.rs:export], [bundle.rs:import].
- Copies adopted template files into `<root>/templates/` [template.rs:adopt].

## Verification
- Run: `cargo test -p sea-forge-cell` (repo-wide: `just test`)
