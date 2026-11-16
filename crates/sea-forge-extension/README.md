# sea-forge-extension

## Overview
SEA Forge crate for extension descriptor and registry contracts (`Cargo.toml` description; spec-full §7.0b). Owns the persisted extension registry (trust levels, statuses, registration and authorization rules) and the stable projection-adapter ABI, plus the pure CEP-0008 v1 flat-profile envelope adapter. Depends on `sea-forge-core`, `sea-forge-ledger`, and `sea-forge-authority`.

## Interface
- Source contract: [./src/README.md] — `ExtensionRegistry`, `TrustLevel`, `ExtensionStatus`, `ProjectionAdapter` ABI, `validate_descriptor`, CEP-0008 module [src/lib.rs:ExtensionRegistry], [src/lib.rs:ProjectionAdapter], [src/cep0008.rs:project_to_cep0008_flat_v1]
- Workspace manifest: `Cargo.toml` (package `sea-forge-extension`; depends on `sea-forge-core` 0.3.0, `sea-forge-ledger` 0.3.0, `sea-forge-authority` 0.3.0)

## Invariants
- Crate is `#![forbid(unsafe_code)]` [src/lib.rs:forbid attribute]
- The registry is persisted at `<root>/extensions/registry.json` (F-12) [src/lib.rs:ExtensionRegistry]
- Re-registration is idempotent only for byte-identical descriptors (SUP-09i); a same-version content change must fail loudly [src/lib.rs:register_built_in]
- Immutable runtime adapters require a version bump for any descriptor change; silently repointing an installed endpoint is forbidden [src/lib.rs:register_immutable_runtime_adapter]
- The `ProjectionAdapter` trait is a stable ABI that must not change before plugins exist [src/lib.rs:ProjectionAdapter]
- The CEP-0008 adapter is pure and deterministic, fail-closed on malformed inputs, and claims only partial CEP-0008 semantic conformance; the native envelope is preserved unchanged [src/cep0008.rs:module doc]

## Side Effects
- Filesystem: registry load/save/exists read and write `<root>/extensions/registry.json`; `load_verified` reads the extension's ledger stream [src/lib.rs:ExtensionRegistry]

## Verification
- Run: `cargo test -p sea-forge-extension` (repo-wide: `just test`)
