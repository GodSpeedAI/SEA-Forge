# sea-forge-extension/src

## Overview
Source for the `sea-forge-extension` crate: SEA Forge's extension descriptor and registry contracts (spec-full §7.0b). `lib.rs` owns the persisted extension registry (trust levels, statuses, registration/authorization rules) and the projection-adapter ABI; `cep0008.rs` is a pure adapter projecting native `SemanticEnvelope`s into the CEP-0008 v1 flat wire profile.

## Interface
- Modules: root (`lib.rs`), `cep0008` [lib.rs:pub mod cep0008]
- `ExtensionRegistry` — registry persisted at `<root>/extensions/registry.json` (F-12), with `load`, `load_verified`, `save`, `exists`, `register_built_in`, `register_immutable_runtime_adapter`, `import_authorized`, `adopt`, `is_active` [lib.rs:ExtensionRegistry]
- `RegistryEntry` — `extension_id`, `version`, `descriptor_sha256`, `trust_level`, `status`, optional `authority_ref` [lib.rs:RegistryEntry]
- `TrustLevel` — `BuiltIn`, `FirstParty`, `Workspace`, `Imported`, `Quarantined` [lib.rs:TrustLevel]
- `ExtensionStatus` — `Active`, `Disabled`, `Quarantined`, `Superseded` [lib.rs:ExtensionStatus]
- `ExtensionAuthorization` — carries an `ActionGrant`, `run_id`, `plan_item_id`, `workspace_root`, and the committed `authority_ref` backing an extension action [lib.rs:ExtensionAuthorization]
- `ExtensionInstallRecord`, `Compatibility` — install history and kernel/version compatibility bounds [lib.rs:ExtensionInstallRecord], [lib.rs:Compatibility]
- `ProjectionAdapter` — stable trait ABI for projection adapters: `plan`, `project`, `validate`, `explain`, `rebuild_hash` [lib.rs:ProjectionAdapter]
- `ProjectionInputs`, `ProjectionPlan`, `ProjectionOutput`, `OutputRef`, `ProjectionRecord`, `ProjectionValidation` — projection run and persisted-record types [lib.rs:ProjectionInputs], [lib.rs:ProjectionRecord]
- `validate_descriptor(descriptor: &ExtensionDescriptor) -> Result<(), ForgeError>` — descriptor validation per spec-full §7.0b/§12 M0 (sha256 grammar, canonical surfaces) [lib.rs:validate_descriptor]
- Re-exports from `sea-forge-core::types`: `ContractRef`, `ProjectionKind`, `ProjectionRef`, `ProjectionStatus` [lib.rs:re-export]
- CEP-0008 module: `Cep0008Provenance`, `Cep0008FlatEnvelopeV1`, `Cep0008ProjectionInput`, `project_to_cep0008_flat_v1`, and profile constants `PROFILE_ID_GODSPEED_V1_FLAT`, `PROFILE_ID_CEP0008_FLAT_V1`, `SCHEMA_DIGEST_V1_FLAT` [cep0008.rs:Cep0008FlatEnvelopeV1], [cep0008.rs:project_to_cep0008_flat_v1], [cep0008.rs:PROFILE_ID_GODSPEED_V1_FLAT]

## Invariants
- Crate is `#![forbid(unsafe_code)]` [lib.rs:forbid attribute]
- Built-in registration is idempotent only when the descriptor is byte-identical; the same `(id, version)` with a different hash is a build bug and must fail loudly, never silently keep the old implementation (SUP-09i) [lib.rs:register_built_in]
- Immutable runtime adapters: a descriptor change must use a new version; silently repointing an installed endpoint (same version, different descriptor hash) is forbidden, and a versioned change replaces the prior entry in place [lib.rs:register_immutable_runtime_adapter]
- Descriptors pass `validate_descriptor` before any registration path; descriptor hashes must be `sha256:` with 64 lowercase hex digits [lib.rs:validate_descriptor]
- `ProjectionAdapter` is a stable contract that must not change before plugins exist; it currently has no implementations beyond a test stub [lib.rs:ProjectionAdapter]
- CEP-0008 projection is pure, deterministic (identical inputs produce identical serialized bytes), and fail-closed (malformed or missing required inputs return `schema_error`) [cep0008.rs:project_to_cep0008_flat_v1]
- `Cep0008FlatEnvelopeV1` and `Cep0008Provenance` are `deny_unknown_fields`, so no unexpected field can appear in serialized output [cep0008.rs:Cep0008FlatEnvelopeV1]
- The output claims only partial CEP-0008 semantic conformance; the native `SemanticEnvelope` is preserved unchanged [cep0008.rs:module doc]
- `PROFILE_ID_GODSPEED_V1_FLAT` is condition-based sunset (G1): zero new G1 consumers permitted [cep0008.rs:PROFILE_ID_GODSPEED_V1_FLAT]

## Side Effects
- Filesystem: `ExtensionRegistry::load`/`save`/`exists` read and write `<root>/extensions/registry.json`; `load_verified` reads the extension's ledger stream to verify entries [lib.rs:ExtensionRegistry]
- None from `cep0008`: no filesystem, ledger, network, or clock operations [cep0008.rs:module doc]

## Verification
- Run: `cargo test -p sea-forge-extension` (repo-wide: `just test`)
