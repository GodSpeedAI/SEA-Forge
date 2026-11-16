# sea-forge-authority — src

## Overview
Source tree for the policy authority kernel. `lib.rs` owns the
`PolicyAuthorityEngine`, `ActionGrant` minting/authorization, policy bundles,
and grant validation; `cep.rs` implements the CEP authority loop (Cognate
`authority_request`/`authority_decision` envelopes) with the `obligation`
submodule for ledger-based obligation settlement; `opa.rs` runs the pinned OPA
CLI over DomainForge Rego bundles as a fail-closed governance candidate.

## Interface
- Module tree, declared in [lib.rs]: `cep` ([cep.rs], incl. `cep/obligation.rs` — see [cep/README.md]) and `opa` ([opa.rs]).
- Re-export from [lib.rs]: `OpaBinding`, `OpaCandidate`, `OpaTrace`, `OPA_VERSION` [opa.rs].
- Core types ([lib.rs]): `GovernanceDisposition`, `GovernanceVerdict`, `ResolutionPolicy`, `ResolvedGovernance`, `ActionGrant`, `ExecutionGrantContext`, `AuthorityPolicyBundle`, `SettlementAuthorityDescriptor`, `IntegrityLedgerPolicy`, `WitnessPolicy`, `PolicySource`, `PermissionGrant`, `SodRule`, `ConfiguredIdentity`, `PolicyEngineConfig`, `IdentityPolicy`, `PolicySurfaces` (`FileSurface`, `ExternalApiSurface`, `GitCommitSurface`, `GithubPrSurface`, `PromptRiskSurface`), `SelfDisclosureGrant`, `SelfDisclosureSurface`, `PolicyRule`, `PolicyAuthorityEngine`, `OpaqueConstraint`, `AuthorityEvaluation`.
- Constants: `VALID_CLAIM_CLASSES`, `HIGH_RISK_CLASSES`, `CELL_BINDING_DIMENSIONS` [lib.rs].
- Engine entry points: `PolicyAuthorityEngine::new`, `load_opaque_constraints`, `evaluate`, `grant`, `grant_after_approval`, `grant_after_approval_idempotent`, `evaluate_at` [lib.rs:PolicyAuthorityEngine]; `ActionGrant::authorize`, `authorize_with_controls`, `authorize_execution` [lib.rs:ActionGrant]; helpers `resolve_candidates`, `scope_allows`, `command_allowed` [lib.rs].
- CEP surface ([cep.rs]): `parse_request`, `CepAuthorityService`, `CepDecision`, `SettlementCriteria`, `ParsedRequest`, `VerifiedTransition`, `TransitionFact`, `ApprovalState`, `ApprovalStanding`, `EvidenceSource`, `Corroboration`, `SettlementOutcome`, `EvidenceAck`, `CepAuthorityError`, `OperationKind`, `cep_disposition`; constants `REQUEST_PROFILE`, `DECISION_PROFILE`, `PROFILE_VERSION`, `MAX_ENVELOPE_BYTES`, `MAX_PACKET_BYTES`, `LEDGER_ID`, `MAX_CLAIMS`, `MAX_CLAIM_LENGTH`, `TRANSITION_KINDS`, `COMPATIBILITIES`.
- OPA surface ([opa.rs]): `OpaBinding`, `OpaCandidate`, `OpaTrace`, `OPA_VERSION = "1.21.1"`, `REGO_DECISION_CONTRACT = "domainforge.rego.decision/v1"`.

## Invariants
- `ActionGrant` is a move-only proof that one exact action may execute in one exact context; it is intentionally not serializable or cloneable [lib.rs:ActionGrant].
- Grants carry the canonical executable identity (`untrusted_executable`) verified by the decision so the spawn path can re-check it; never serialized [lib.rs:ActionGrant F-25.h].
- CEP envelopes are parsed fail-closed; the pinned world must be registered; dispositions are never collapsed to a boolean; boundary/degraded decisions keep constraints, policy basis, and controls [cep.rs:module docs].
- The evaluated CEP actor is the verified caller supplied by the server's identity gate; the envelope's subject is recorded, never trusted for role [cep.rs:module docs].
- Authority decisions are committed append-only before being returned [cep.rs:module docs].
- OPA is fail-closed: every evaluation problem (missing binary, version drift, timeout, malformed/inconsistent decision) becomes a `Deny` candidate; only the pinned `opa` version 1.21.1 is accepted [opa.rs:module docs], [opa.rs:OPA_VERSION].
- Envelope/packet size bounds: `MAX_ENVELOPE_BYTES = 64 KiB`, `MAX_PACKET_BYTES = 256 KiB`, `MAX_CLAIMS = 16`, `MAX_CLAIM_LENGTH = 512` [cep.rs].

## Side Effects
- `PolicyAuthorityEngine` keeps issued decisions and opaque constraints in internal `Mutex` state [lib.rs:issued_decisions], [lib.rs:opaque_constraints].
- Policy bundle loading reads bundle and source files from disk (`fs::read_to_string`, `fs::read`) [lib.rs:AuthorityPolicyBundle::load].
- OPA evaluation reads provenance/manifest/bundle files and spawns the external pinned `opa` CLI process via `Command::new` [opa.rs:bundle integrity], [opa.rs:opa invocation].
- CEP decisions are committed to the `cognate-authority` ledger stream via `LedgerStream` [cep.rs:LEDGER_ID].

## Verification
- Run: `cargo test -p sea-forge-authority` (repo-wide: `just test`)
