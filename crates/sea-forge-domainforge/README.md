# sea-forge-domainforge

## Overview
SEA Forge's first-party DomainForge semantic adapter (`Cargo.toml` description; spec-full §7.0a). It wraps the pinned external `domainforge-core` crate to validate `.sea` source sets into semantic models, project them into CALM/RDF/KG artifacts, and bind semantic envelopes to verified world identities. Pure, synchronous, and side-effect-free by contract.

## Interface
- Source contract: [./src/README.md] — `load_validate`, `project`, `DomainModel`/`DomainModelRef`, `WorldRegistry`/`WorldBindingError`, pinned limits and version [src/lib.rs:load_validate], [src/lib.rs:project], [src/world.rs:WorldRegistry]
- Workspace manifest: `Cargo.toml` (package `sea-forge-domainforge`; pinned `domainforge-core = "=0.21.1"`, `sea-forge-core` 0.3.0)

## Invariants
- Public contract is synchronous and side-effect-free: `load_validate` and `project` are pure, with no filesystem, network, or CLI use [src/lib.rs:module doc]
- Crate is `#![forbid(unsafe_code)]` [src/lib.rs:forbid attribute]
- Adapter is pinned to DomainForge 0.21.1 (`EXPECTED_DOMAINFORGE_VERSION`) and bounded by the `MAX_*` resource limits, including parser-recursion bounding (`MAX_NESTING_DEPTH`, SUP-01) [src/lib.rs:EXPECTED_DOMAINFORGE_VERSION], [src/lib.rs:MAX_NESTING_DEPTH]
- World binding never trusts a sender's `world_ref` and fails closed on anything unknown, malformed, tampered, mismatched, or invalid [src/world.rs:module doc]
- CALM projections are byte-identical for the same graph (non-deterministic `sea:timestamp` stripped) [src/lib.rs:project]

## Side Effects
- None in the library contract; the crate performs no filesystem, ledger, network, or clock operations.

## Verification
- Run: `cargo test -p sea-forge-domainforge` (repo-wide: `just test`)
