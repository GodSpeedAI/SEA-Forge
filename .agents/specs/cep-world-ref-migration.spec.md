# CEP-0008 / world_ref migration spec (v0.1.0)

Normative for branch `migration/cep-world-ref`. Source brief: operator's staged migration prompt (2026-10-04). Stage ledger: `.agents/reports/cep-world-migration/stage0-migration-map.md`.

## Invariants
1. DomainForge owns semantic meaning and canonical identity (`DomainModelIdentity::canonical_digest()`). No second ontology, hash scheme, or universal protocol.
2. CEP-0008 is the semantic interoperability protocol, not the transport. Native internals stay native; CEP is required only where meaning, authority, observation, evidence, settlement, identity, provenance, completeness or omissions cross a subsystem boundary.
3. `world_ref` (`world:<name>@sha256:<digest>`) is one immutable semantic-world revision, defined once in DomainForge. `world_alias` is mutable and must resolve to a `world_ref` before consequential execution. `world_label` is presentation only. Durable records pin `world_ref`. Unknown or unverifiable identity fails closed or escalates.
4. Never collapse: representation/reality, observation/occurrence, data/evidence, claim/evidence, completion/settlement, possibility/permissibility, authority/capability.
5. SEA-Forge keeps governed execution authority and settlement. Its ledger stays append-only, hash-chained, MMR-committed. pg0 migration is out of scope.
6. Governance modes are `sea-forge` and `off` only. No implicit fallback; outage under `sea-forge` never becomes ungoverned execution.
7. One DomainForge release (target 0.18.2) across every active consumer; historical fixtures may differ only when marked.

## Stage gates
Each stage is settled only when its gate passes (Stage 1: every active consumer on 0.18.2, builds, existing tests pass, no runtime dependency on an older version). A stage is committed and pushed before the next begins.

## Stage 4 — SEA-Forge semantic-world binding

Operator decision (2026-10-04): SEA-Forge recomputes the world digest itself from the carried
`DomainModelIdentity`; it never trusts a sender's `world_ref` assertion.

Requirements (crate `sea-forge-domainforge`, module `world`):
- R1 `DomainModelRef` gains `world_ref: Option<String>` (serde default; absent on pre-binding records).
- R2 `WorldRegistry` caches verified worlds keyed by `world_ref`; register-only, append-only (a world is never replaced).
- R3 `register_source_set(name, source_set)` derives identity via DomainForge, mints the `world_ref`, registers it.
- R4 `verify_snapshot(envelope)` fails closed when: `scope.world_ref` missing/malformed; identity extension missing/malformed; recomputed digest != `world_ref`; model flagged invalid; world unknown to the registry; presented identity differs from the registered one.
- R5 `bind(model_ref, world_ref)` / `verify_model_ref` tie SEA-Forge's own `d_content_hash` + `semantic_closure_hash` to the registered world; mismatch fails closed.
- R6 Ledger-neutral: no storage assumption; registry is a derived cache, world_ref values are plain strings suitable for append-only records.
Out of scope: persistence of the registry (pg0 not started), transitions (Stage 10).
