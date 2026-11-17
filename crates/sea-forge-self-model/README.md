# sea-forge-self-model

## Overview

SEA Forge Genesis self-model (spec-adlc-thoth E11, M9). Bundles the canonical
release `.sea` models, computes deterministic release and installation-cell
realizations, projects the self-model through DomainForge, and maintains
immutable, hash-linked self-model snapshots on disk. A synchronous kernel
crate: no async, no network (`src/lib.rs` module doc).

## Interface

- Crate contract lives in [./src/README.md](./src/README.md) (module tree:
  `lib.rs`, `projections.rs`, `store.rs`).
- Key entry points, cited from source:
  - `bundled()` / `verify_bundled()` / `load_composed()` —
    `src/lib.rs:bundled`, `src/lib.rs:verify_bundled`,
    `src/lib.rs:load_composed`.
  - `release_realization()` / `build_cell_realization()` —
    `src/lib.rs:release_realization`, `src/lib.rs:build_cell_realization`.
  - Snapshot assembly: `build_snapshot` / `verify_snapshot` /
    `snapshot_hash` — `src/lib.rs:build_snapshot`.
  - On-disk store: `rebuild`, `current_snapshot`, `ensure_init`,
    `mark_current_stale`, `validate` — `src/store.rs:rebuild`.
  - Self-model projection: `project_self` / `verify_projection` —
    `src/projections.rs:project_self`.

## Invariants

- Bundled model bytes must match the pinned sha256 constants; drift is
  `self_model_error` (`src/lib.rs:SYSTEM_MODEL_SHA256`).
- Snapshots are immutable and canonically hash-linked
  (`src/lib.rs:verify_snapshot`).
- Synchronous kernel crate: no async, no network, `#![forbid(unsafe_code)]`
  (`src/lib.rs` module doc).

## Side Effects

- Reads and writes self-model state (manifest, realizations, snapshots,
  staleness markers) under the cell root's `.sea-forge` state tree
  (`src/store.rs:rebuild`, `src/store.rs:mark_current_stale`).

## Verification

- Run: `cargo test -p sea-forge-self-model` (repo-wide: `just test`)
