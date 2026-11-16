# sea-forge-capability — src

## Overview
Source tree for the capability layer: memory extraction/recall over JSONL
stores ([memory.rs]) and capability promotion from settled declarations
([promotion.rs]), plus canonical extension-descriptor validation in
[lib.rs].

## Interface
- Module tree, declared in [lib.rs]: `memory` ([memory.rs]), `promotion` ([promotion.rs]).
- Root-level API ([lib.rs]): `validate_extension_descriptor(&ExtensionDescriptor)` — validates authority surface membership and sha256 reference form.
- Re-exports from [lib.rs]: `append_capability_record`, `build_capability_record`, `compute_policy_hash`, `default_v02_policy`, `format_fixed`, `load_declarations`, `load_envelopes`, `parse_fixed`, `rebuild_capability`, `require_proven`, `save_policy` [promotion.rs].
- Memory API ([memory.rs]): `compute_dedup_key`, `extract_from_envelope`, `append_memory_items`, `load_memory_items`, `MemoryRecallQuery`, `recall_memory`, `rebuild_index`, `query_index`, `recall_with_fallback`, `MAX_RECALL_LIMIT = 50`.
- Promotion API ([promotion.rs]): `parse_fixed`, `format_fixed` (6-decimal fixed point, i64 millionths, `SCALE = 1_000_000`), `default_v02_policy`, `compute_policy_hash`, `save_policy`, `load_policy`, `declaration_qualifies`, `load_envelopes`, `build_capability_record`, `rebuild_capability`, `require_proven`, `load_declarations`, `append_capability_record`.

## Invariants
- Fixed-point arithmetic uses i64 millionths with saturating ops and char-boundary truncation so pathological magnitudes clamp rather than panic or wrap (SUP-05) [promotion.rs:parse_fixed].
- SoD boundary (spec-adlc-thoth §10.3, T13B) is re-checked independently of settlement: a declaration whose author equals the declarer never counts toward capability promotion [promotion.rs:declaration_qualifies].
- `declaration_qualifies` additionally requires: `Accepted` status, `Strong` strength, `qualifies_for_capability`, declared independence, weight >= policy minimum, non-empty `criteria_ref`, and source evidence for any variation tags [promotion.rs:declaration_qualifies].
- `require_proven` fails closed with `settlement_integrity_error` unless the capability's status is `Proven` or `Metabolized` under a policy whose name matches the reference and whose sha256 is pinned [promotion.rs:require_proven].
- Dedup keys are sha256 over (kind, trimmed lowercased statement, entity id) [memory.rs:compute_dedup_key].

## Side Effects
- Memory store: appends items to a JSONL file, creating parent directories; reads items; writes a recall index via temp file + `fs::rename`; opens/updates files with `OpenOptions` [memory.rs:append_memory_items], [memory.rs:rebuild_index], [memory.rs:recall_with_fallback].
- Promotion: writes/reads the capability promotion policy JSON and capability records under the workspace root (`fs::write`, `fs::read`, `fs::create_dir_all`); loads envelopes and declarations from JSONL files [promotion.rs:save_policy], [promotion.rs:rebuild_capability], [promotion.rs:load_declarations].

## Verification
- Run: `cargo test -p sea-forge-capability` (repo-wide: `just test`)
