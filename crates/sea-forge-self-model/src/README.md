# sea-forge-self-model — src

## Overview

Implementation of the `sea-forge-self-model` crate: SEA Forge Genesis
self-model (spec-adlc-thoth E11, M9). Bundles canonical `.sea` model bytes,
provides deterministic release and cell realizations, DomainForge projections
of the self-model, and immutable snapshots with canonical hash linking.
Synchronous kernel crate: no async, no network (`src/lib.rs` module doc).

## Interface

- Module tree: `projections`, `store` (`src/lib.rs:projections`,
  `src/lib.rs:store`); everything else is in `lib.rs`.
- Bundled assets and pinned hashes (`lib.rs`):
  - `SYSTEM_MODEL_SHA256`, `SEED_MODEL_SHA256` — pinned release sha256 of
    `models/seaforge-system@0.1.0.sea` and `models/adlc-odi-case@0.1.0.sea`,
    included via `include_bytes!` (`lib.rs:SYSTEM_MODEL_SHA256`,
    `lib.rs:SYSTEM_MODEL_BYTES`).
  - `RELEASE_ID = "sea-forge@0.1.0"`, `KERNEL_VERSION = "0.1"`
    (`lib.rs:RELEASE_ID`, `lib.rs:KERNEL_VERSION`).
  - `bundled() -> BundledModels` — release-owned model bytes + pinned
    constants (`lib.rs:bundled`).
  - `verify_bundled(models)` — reject byte drift before validation with
    `self_model_error` (`lib.rs:verify_bundled`).
  - `load_composed(models)` — validate both models through DomainForge
    (`lib.rs:load_composed`); types `BundledModels`, `ModelAsset`,
    `ComposedModel`.
- Realizations (`lib.rs`):
  - `release_realization()` — deterministic release realization
    (SOURCE_DATE_EPOCH-driven) (`lib.rs:release_realization`); type
    `ReleaseRealization`, `ProjectionTarget`, `BundledModelRef`;
    `realization_sha256(r)` (`lib.rs:realization_sha256`).
  - `build_cell_realization(...)` / `cell_realization_sha256` — installation
    cell realization (`lib.rs:build_cell_realization`); types
    `CellRealization`, `ToolchainProbe`, `ProbeResult`, `ExtensionState`.
- Snapshots (`lib.rs`): `build_snapshot`, `snapshot_hash`,
  `verify_snapshot`, `SelfModelSnapshot`, `SnapshotFreshness`
  (`lib.rs:build_snapshot`, `lib.rs:verify_snapshot`).
- Store (`store.rs`): on-disk self-model state under a cell root —
  `rebuild(root, inputs)` (`store.rs:rebuild`), `ensure_init`
  (`store.rs:ensure_init`), `current_snapshot(root)` (`store.rs:current_snapshot`),
  `read_cell_realization` / `read_release_realization`
  (`store.rs:read_cell_realization`), `mark_current_stale` (`store.rs:mark_current_stale`),
  `is_stale` (`store.rs:is_stale`), `validate` (`store.rs:validate`);
  types `SelfModelManifest`, `RebuildInputs`.
- Projections (`projections.rs`): `project_self(...)` — DomainForge projection
  of the self-model (`projections.rs:project_self`); `verify_projection(record,
  outputs_dir)` (`projections.rs:verify_projection`); type
  `SelfModelProjection`.
- Helper: `canonical_sha256<T: Serialize>(value)` — canonical JSON hashing
  (`lib.rs:canonical_sha256`).

## Invariants

- Model bytes are release-owned: `include_bytes!` of the canonical `.sea`
  files; any drift against the pinned sha256 constants is `self_model_error`,
  never a warning (`lib.rs:SYSTEM_MODEL_SHA256`, `lib.rs:verify_bundled`).
- Snapshots are immutable once built and hash-linked: `snapshot_hash` links a
  snapshot to its inputs; `verify_snapshot` re-checks the chain
  (`lib.rs:build_snapshot`, `lib.rs:snapshot_hash`, `lib.rs:verify_snapshot`).
- Realizations are deterministic — no wall-clock or environment leakage beyond
  the declared inputs (`lib.rs:release_realization`, SOURCE_DATE_EPOCH).
- `#![forbid(unsafe_code)]`; synchronous kernel crate — no async, no network
  (`src/lib.rs` module doc).

## Side Effects

- Disk writes under the cell root: `store.rs:rebuild` / `ensure_init`
  materialize the self-model manifest, realizations, and current snapshot;
  `mark_current_stale` rewrites staleness state (`store.rs:rebuild`,
  `store.rs:mark_current_stale`).
- Disk reads: `current_snapshot`, `read_cell_realization`,
  `read_release_realization`, `is_stale`, `validate` read the persisted
  self-model state (`store.rs`).
- Projection outputs: `projections.rs:project_self` produces projection
  records whose outputs `verify_projection` reads back from `outputs_dir`
  (`projections.rs:verify_projection`).

## Verification

- Run: `cargo test -p sea-forge-self-model` (repo-wide: `just test`)
