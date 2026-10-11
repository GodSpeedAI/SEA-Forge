# Stage 4 settlement — SEA-Forge semantic-world binding

Date: 2026-10-04. Branch `migration/cep-world-ref`. Spec: `specs/cep-world-ref-migration.spec.md` (Stage 4). Plan: `plans/2026-10-04-stage4-world-binding.md`.
Operator decision: SEA-Forge recomputes the digest itself; it never trusts a sender's `world_ref`.

## Delivered (crate `sea-forge-domainforge`)
- `world.rs`: `WorldRegistry` (append-only cache by canonical `world_ref`), `WorldBindingError` (typed refusals, converts to `ForgeError`).
- `register_source_set` / `register_identity`: identity derived by DomainForge, digest recomputed, idempotent, one canonical name per digest.
- `verify_snapshot`: real CEP `semantic_snapshot` -> `scope.world_ref` + `domainforge.identity` -> recompute -> registry lookup -> identity equals registered.
- `bind` / `verify_model_ref`: `DomainModelRef.world_ref` (new, `Option`, serde-default, omitted when absent) ties SEA-Forge's `d_content_hash` and `semantic_closure_hash` to the registered world.
- Fail closed on: missing/malformed ref, missing identity, invalid-model flag, digest mismatch (tampered identity or forged ref), unknown world, identity differing from registered, model-ref mismatch, unbound model ref.

## Verification
- `tests/world_binding.rs`: 11 tests, built on real DomainForge 0.19.0 `build_cep_envelope` output (positive path included).
- `just test`: 1,121 passed, 0 failed (was 1,110). Existing `DomainModelRef` records still deserialize.
- Ledger-neutral: registry is a derived in-memory cache; no storage assumed (pg0 untouched).

## Known limits / debt
- Registry is in-memory; durable persistence of worlds is deferred with the pg0 decision.
- Authority/governance paths do not yet *require* a bound `world_ref` (Stage 5/6 consume it).
- `verify_snapshot` checks DomainForge identity integrity, not full CEP profile conformance (cep validators own that).
