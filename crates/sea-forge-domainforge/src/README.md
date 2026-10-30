# sea-forge-domainforge/src

## Overview
Source for the `sea-forge-domainforge` crate: SEA Forge's first-party semantic adapter over the pinned external `domainforge-core` crate (spec-full §7.0a). It validates a `.sea` source set into an in-memory `DomainModel`, projects validated models into CALM/RDF/KG artifacts, and binds semantic envelopes to verified "world" identities. `lib.rs` holds the load/validate/project contract; `world.rs` holds the semantic-world binding and registry.

## Interface
- Modules: root (`lib.rs`), `world` [lib.rs:pub mod world]
- `load_validate(source_set: &SeaSourceSet) -> Result<DomainModel, ForgeError>` — parse and validate a `SeaSourceSet` into a `DomainModel` [lib.rs:load_validate]
- `project(model: &DomainModel, kind: &ProjectionKind) -> Result<BTreeMap<String, String>, ForgeError>` — project a model into CALM (`calm.json`), RDF (`model.ttl`, `model.rdf`), or KG artifacts as a sorted path-to-content map [lib.rs:project]
- `SeaSourceSet`, `SourceFile` — entry `.sea` source plus namespace-registry/imported files, each with `uri`/`sha256`/`content` [lib.rs:SeaSourceSet], [lib.rs:SourceFile]
- `DomainModel`, `DomainModelRef`, `SourceRef` — in-memory derived view and its stable reference (graph, source refs, adapter/parse/semantic hashes, optional `world_ref`) [lib.rs:DomainModel], [lib.rs:DomainModelRef], [lib.rs:SourceRef]
- `ADAPTER_DESCRIPTOR_SHA256`, `EXPECTED_DOMAINFORGE_VERSION` — pinned adapter descriptor digest and expected DomainForge version (`"0.21.1"`) [lib.rs:ADAPTER_DESCRIPTOR_SHA256], [lib.rs:EXPECTED_DOMAINFORGE_VERSION]
- Finite-limit constants: `MAX_SOURCE_COUNT` (64), `MAX_AGGREGATE_BYTES` (1 MiB), `MAX_IMPORT_DEPTH` (16), `MAX_AST_NODES` (10,000), `MAX_NESTING_DEPTH` (256) [lib.rs:MAX_SOURCE_COUNT], [lib.rs:MAX_AGGREGATE_BYTES], [lib.rs:MAX_IMPORT_DEPTH], [lib.rs:MAX_AST_NODES], [lib.rs:MAX_NESTING_DEPTH]
- `parse_identity_json(json: &str) -> Result<DomainModelIdentity, WorldBindingError>` — parse a `DomainModelIdentity`, refusing unknown keys [world.rs:parse_identity_json]
- `WorldRegistry` — in-memory, append-only cache of verified worlds keyed by canonical `world:<name>@sha256:<digest>` text [world.rs:WorldRegistry]
- `WorldBindingError` — every variant is a refusal (missing/invalid identity, digest mismatch, unknown world, identity-role error, unresolvable model) [world.rs:WorldBindingError]
- Re-export of `domainforge_core::application::world::WorldRef` [lib.rs:WorldRef]

## Invariants
- Crate is `#![forbid(unsafe_code)]` [lib.rs:forbid attribute]
- The public contract is synchronous and side-effect-free: `load_validate`, and `project` are pure; `project` explicitly performs no filesystem writes, no network calls, no CLI invocations [lib.rs:module doc], [lib.rs:project]
- CALM projections strip the non-deterministic `sea:timestamp` metadata field so the same graph produces byte-identical output across calls [lib.rs:project]
- The adapter is pinned to `domainforge-core = "=0.21.1"` [Cargo.toml:domainforge-core] and expects `EXPECTED_DOMAINFORGE_VERSION = "0.21.1"` [lib.rs:EXPECTED_DOMAINFORGE_VERSION]
- All external parser/model input is bounded by the `MAX_*` constants; `MAX_NESTING_DEPTH` bounds the external pest parser's recursion so a crafted model cannot overflow the stack (SUP-01) [lib.rs:MAX_NESTING_DEPTH]
- SEA Forge never trusts a sender's `world_ref`: the identity digest is recomputed from the carried `DomainModelIdentity` with the pinned DomainForge implementation, and binding fails closed on anything unknown, malformed, tampered, mismatched or invalid [world.rs:module doc]
- The `WorldRegistry` is append-only: a registered world is never replaced or removed; a *prior* identity is never a place to act [world.rs:WorldRegistry]
- `parse_identity_json` rejects documents containing fields the identity digest does not cover [world.rs:parse_identity_json]
- `DomainModel` is an in-memory derived view, not persisted truth [lib.rs:DomainModel]

## Side Effects
- None. No filesystem, ledger, network, or clock operations in either module; the world registry assumes no storage backend.

## Verification
- Run: `cargo test -p sea-forge-domainforge` (repo-wide: `just test`)
