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

## Stage 7 — Escalation and durable continuation

Written after the implementation (the plan was held in the session, not in this file first); recorded here so the
requirements the code and tests answer are explicit. Process deviation noted in the settlement.

- R1 An `escalate` whose cause is a policy rule requiring approval issues an approval (`apr-<operation_id>`, expiring, default 24h); any other escalation (unresolved identity, active opaque constraint, unavailable engine) issues none and stays a plain refusal.
- R2 Approval standing is derived only from the append-only `cognate-authority` ledger: the escalation entry (key `cep:<op>`), the resolution (key `cep-approval-resolve:<apr>`), the use (key `cep-approval-use:<apr>`). No second journal. Resolution and use are single-shot by ledger idempotency under the exclusive lock.
- R3 Only an actor that policy allows (`approval_resolution` rule) and that is not the requester may resolve; the attempt, allowed or refused, is a committed decision.
- R4 Revalidation: a request carrying `approval_id` must reference an approved, unexpired, unused approval for exactly the same world, operation kind, name, resource, subject and requester, with lineage to the escalated request. Policy is evaluated again now. An approval satisfies a policy escalation only: it never overrides a deny, and adds nothing when policy now allows outright (not consumed).
- R5 Cognate: an escalated capability call inside a run records the escalation once, then waits on a durable `governance-approval` continuation (expiring with the approval window). Any resume only triggers revalidation; the authority alone decides. A rejected, expired or used approval closes the wait and fails the run. The provider never runs before an allow.
- R6 `Runtime.pollApprovals()` (optionally timed) resumes approved waits and cancels closed ones, idempotently.
- Out of scope: escalated actions (startRun etc.) wait nowhere and are refused with the approval id; surfacing Cognate approvals in the SEA-Forge workbench approval inbox (separate ledger stream, not the case approvals journal).

## Stage 8 — execution -> RealityTrace -> evidence -> settlement

Written before implementation. Roles are fixed by the migration brief and are the point of the stage:
Cognate operates, RealityTrace observes, SEA-Forge settles. Execution is not evidence; evidence is not settlement.

Findings from recon (observed with the released RealityTrace 0.3.0):
- `sxr` writes its own closed CEP envelopes (`observation.recorded`, `evidence.created` as `evidence_packet`, `settlement.registered` as `settlement_packet`); `scope` is limited to repo/checkout/run keys (no `world_ref`), lineage is empty, kinds are fixed by record. Changing that needs a RealityTrace release, so this stage does not modify it (debt).
- An invocation observation yields evidence with `direction: unknown`, so sxr's own settlement is `unsettled`. Completion alone cannot settle; supporting evidence needs a verifier.
- sxr's settlement is RealityTrace's threshold judgment about its question. SEA-Forge's settlement is the governed acceptance of the operation. They are different facts and are never conflated.

Requirements:
- R1 Cognate emits a GodSpeed `execution_trace` envelope per governed capability invocation: `scope.world_ref`, `lineage_refs` to the authority decision envelope, `godspeed.execution_trace {operation_id, authority_decision_ref, correlation_id}`, the outcome as completion only. The authority reference reaches the trace through the recorded decision metadata, not by lookup at emission time.
- R2 The trace is delivered to RealityTrace as the observation content (hash-addressed by sxr); Cognate's correlation ids keep riding in `extensions.cognate`.
- R3 Cognate packages RealityTrace's recorded evidence and question into a GodSpeed `evidence_packet` (lineage to the trace envelope, same world, provenance naming the sxr ledger positions, items exactly as sxr recorded them: direction and reliability are never upgraded).
- R4 SEA-Forge `authority_evidence` accepts an evidence packet only if: profile and kind are right; the world is registered and equals the authority decision's world; `authority_decision_ref` names a committed ALLOW decision for the same operation; items are well-formed. It commits the packet append-only (idempotent by envelope id). It never settles.
- R5 SEA-Forge `authority_settle` evaluates the committed evidence for an operation against criteria declared in cell config and bound into the allow decision at decision time (`settlement-criteria:<sha256>` in `policy_refs`); if the configured criteria changed since, it refuses (no moving goalposts). Policy must allow the caller the `settlement_declaration` surface. Outcomes: `settled` (enough supporting evidence at or above the minimum reliability and none contradicting), `rejected` (any contradicting evidence), `unsettled` (otherwise, including trace-only). It returns a `settlement_packet` and commits it; `settled` and `rejected` are final for the operation, `unsettled` can be re-evaluated when the evidence set changes.
- R6 Conformance: the three packets validate under cep's own validator.
- Out of scope: SWE_SEED transport of the settlement packet (Stage 9), Gauntlet as a verifier (Stage 9), modifying RealityTrace.
