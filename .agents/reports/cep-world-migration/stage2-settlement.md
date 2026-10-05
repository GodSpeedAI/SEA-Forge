# Stage 2 — canonical world_ref: settlement record (2026-10-04)

Repo: DomainForge, branch `feat/world-ref`, commit a3a4157 (pushed, not merged, not released).

## Decisions (operator, 2026-10-04)
- `world_ref` digest is exactly `DomainModelIdentity::canonical_digest()`. No second hash.
- It therefore changes on a DomainForge release and on a comment-only source edit; `semantic_closure_hash` is the field that proves equal meaning.

## What exists
- Format (defined once, `domainforge-core/src/application/world.rs`, documented in `docs/reference/sea-application-contract.md` 14.1): `world:<name>@sha256:<64 lowercase hex>`; alias `world:<name>`; label is presentation only.
- Types: `WorldRef`, `WorldAlias`, `WorldLabel`, `WorldName`, `WorldCatalog` (register, lookup, verify, retarget_alias, resolve_alias, alias_history, pin). Worlds are append-only; a digest has one canonical name; unknown/malformed/mismatching references fail closed.
- `DomainModelIdentity::from_document` (field mapping per reference 14) and `validate` (unsupported scheme and malformed hashes fail closed).
- Thin bindings on `Graph` for Python, TypeScript and WASM: `domain_model_identity_json`, `world_ref_from_identity_json`, `verify_world_ref`, `parse_world_ref`.

## Evidence
- Independent implementation: stdlib-only `tests/fixtures/world_ref/world_ref_reference.py` generates and self-checks `golden-vectors.json` (6 valid, 15 invalid refs, aliases). Rust (3 golden + 16 contract tests), Python binding (24), TypeScript binding (23), WASM (1 added; 23 total) reproduce every digest and accept/reject identical strings.
- Gates: `cargo test -p domainforge-core --features cli` 1267 passed / 0 failed; pytest all pass; vitest 221/221; `just wasm-test` 23/23; clippy -D warnings clean; fmt clean.
- Required behaviors: same world -> same ref (`same_resolved_world_yields_the_same_world_ref`); semantic change -> new ref; mutable alias retarget leaves a pinned historical ref unchanged and verifiable (`alias_retarget_never_rewrites_a_historical_world_ref`); tampered identity fails (`tampered_identity_does_not_verify_against_the_original_ref`).

## Deviation from the brief, stated plainly
The brief expected formatting/comments to leave `world_ref` unchanged. They do not: DomainForge's `source_set_hash` covers exact source text, and the operator chose to keep `world_ref` equal to the canonical digest. A comment-only edit mints a new `world_ref` with an identical `semantic_closure_hash` (asserted, not hidden). Stage 10 transitions should carry that equality as compatibility metadata.

## Gate
Met at DomainForge: two independent implementations verify the same world_ref and a catalog answers "what world is this" without filename or process-memory ambiguity.
Not met downstream: no consumer can use it until DomainForge merges and releases (expected 0.19.0) and the consumers are bumped together (repeat of Stage 1).
