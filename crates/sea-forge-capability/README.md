# sea-forge-capability

## Overview
SEA Forge capability memory and extension validation (Cargo.toml). Two
subsystems: a JSONL-backed memory store with deterministic extraction, dedup,
indexing, and recall; and the capability promotion pipeline that turns
strongly-settled declarations into capability records under a hashed policy,
with a fail-closed `require_proven` gate.

## Interface
- Modules: `memory` (memory store and recall) and `promotion` (capability promotion) [src/lib.rs:pub mod].
- Root API: `validate_extension_descriptor` [src/lib.rs].
- Re-exports: `append_capability_record`, `build_capability_record`, `compute_policy_hash`, `default_v02_policy`, `format_fixed`, `load_declarations`, `load_envelopes`, `parse_fixed`, `rebuild_capability`, `require_proven`, `save_policy` [src/lib.rs:pub use].
- Memory: `compute_dedup_key`, `extract_from_envelope`, `append_memory_items`, `load_memory_items`, `recall_memory`, `rebuild_index`, `query_index`, `recall_with_fallback`, `MAX_RECALL_LIMIT` [src/memory.rs].
- Promotion: `parse_fixed`/`format_fixed` (i64 millionths fixed point), `declaration_qualifies`, `require_proven`, `save_policy`/`load_policy`, `rebuild_capability` [src/promotion.rs].
- Source tree contract: [src/README.md]

## Invariants
- SoD boundary (spec-adlc-thoth §10.3, T13B) re-checked at promotion time: self-authored declarations never count toward promotion [src/promotion.rs:declaration_qualifies].
- Fixed-point math is saturating and char-boundary safe (SUP-05) [src/promotion.rs:parse_fixed].
- `require_proven` denies with `settlement_integrity_error` unless the capability is `Proven`/`Metabolized` under the pinned policy hash and matching name [src/promotion.rs:require_proven].
- Dedup keys are sha256 over (kind, normalized statement, entity id) [src/memory.rs:compute_dedup_key].
- Extension descriptors must use canonical authority surfaces and lowercase 64-hex sha256 references [src/lib.rs:validate_extension_descriptor].

## Side Effects
- Reads/writes memory items JSONL and index files under workspace paths (temp-file + rename for the index) [src/memory.rs:append_memory_items], [src/memory.rs:rebuild_index].
- Reads/writes promotion policy and capability record JSON files; loads declarations/envelopes from JSONL [src/promotion.rs:save_policy], [src/promotion.rs:load_declarations].

## Verification
- Run: `cargo test -p sea-forge-capability` (repo-wide: `just test`)
