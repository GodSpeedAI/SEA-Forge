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

## Stage 6 — Cognate <-> SEA-Forge CEP authority loop

Transport (operator-confirmed "go on" to the recommendation): SEA-Forge's existing Unix-socket NDJSON server (ADR-003 additive verb `authority_request`). Cognate (Bun) connects with a unix socket client. Identity is the server's existing SO_PEERCRED gate; the verified caller is the evaluated actor, the envelope's subject is recorded, never trusted for role.

- R1 SEA-Forge: `sea-forge-authority::cep` parses a CEP `authority_request` (profile `godspeed.authority_request`), requires its `scope.world_ref` to be registered in the Stage 4 `WorldRegistry`, evaluates it with the real `PolicyAuthorityEngine` as `AuthorityAction::Reserved { cognate_action | cognate_capability }`, commits the `AuthorityDecision` append-only (idempotent by operation_id), and returns a CEP `authority_decision` envelope with lineage to the request.
- R2 Dispositions map without loss: allow, deny, escalate, boundary and degraded keep constraints, policy_basis and compensating controls; nothing becomes a boolean. Unknown action surface -> deny (engine default).
- R3 Fail closed: malformed request, unknown/unregistered world, wrong kind/profile, oversize, cep authority disabled -> typed error, no allow.
- R4 Cognate: `SeaForgeAuthority implements GovernanceAuthority`: builds the request envelope, sends it, validates the response (kind, profile, lineage == request id, world_ref echo, operation_id echo, known decision, constraints present for constrained dispositions) and maps to Governor dispositions. Any transport or validation failure -> `unknown` (refused by the Governor).
- R5 Constraint handling: Governor honors `boundary`/`degraded`/`constrained_allow` only when constraints are present AND an enforcer for each constraint is registered; otherwise refuses. Stage 6 ships the mechanism with no enforcers registered by default, so these still refuse.
- Out of scope: escalation continuation (Stage 7), world loading beyond server config, per-subject policy (engine rules key on role + operation kind only; recorded as debt).
