# sea-forge-authority

## Overview
SEA Forge authority fabric: policy, identity, and governance verdicts
(Cargo.toml). Houses the `PolicyAuthorityEngine` that evaluates governance
candidates into `AuthorityDecision` records, mints move-only `ActionGrant`
proofs for exact actions in exact contexts, implements the CEP authority loop
for Cognate authority requests (including obligation settlement from
RealityTrace's ledger), and runs the pinned OPA CLI as a fail-closed
governance candidate.

## Interface
- Modules: `cep` (CEP authority loop + obligation settlement) and `opa` (OPA/Rego candidate) [src/lib.rs:pub mod].
- Re-exports: `OpaBinding`, `OpaCandidate`, `OpaTrace`, `OPA_VERSION` [src/lib.rs:pub use].
- Key symbols:
  - `PolicyAuthorityEngine` (`new`, `evaluate`, `grant`, `grant_after_approval`, `grant_after_approval_idempotent`) [src/lib.rs:PolicyAuthorityEngine]
  - `ActionGrant` (`authorize`, `authorize_with_controls`, `authorize_execution`) [src/lib.rs:ActionGrant]
  - Policy/identity types: `AuthorityPolicyBundle`, `PolicyRule`, `PolicySurfaces`, `SelfDisclosureGrant`, `IdentityPolicy`, `GovernanceVerdict`, `GovernanceDisposition` [src/lib.rs]
  - CEP: `parse_request`, `CepAuthorityService`, `SettlementCriteria`, `cep_disposition` [src/cep.rs]
  - OPA: `OpaBinding`, `OpaCandidate`, `OpaTrace`, `REGO_DECISION_CONTRACT` [src/opa.rs]
- Source tree contract: [src/README.md]; obligation submodule: [src/cep/README.md]

## Invariants
- `ActionGrant` is move-only, not serializable or cloneable; grants never bypass a committed append-only authority decision [src/lib.rs:ActionGrant].
- CEP parsing is fail-closed; dispositions are never collapsed to a boolean; the envelope subject is recorded but never trusted for role [src/cep.rs:module docs].
- OPA is fail-closed: any evaluation problem yields a `Deny` candidate, and only pinned `opa` 1.21.1 is accepted [src/opa.rs:module docs].
- Envelope/packet/claim size bounds enforced [src/cep.rs:MAX_ENVELOPE_BYTES].

## Side Effects
- Reads policy bundles from disk; spawns the external `opa` binary; reads bundle integrity files [src/lib.rs:AuthorityPolicyBundle::load], [src/opa.rs].
- Commits authority decisions append-only to the `cognate-authority` ledger stream [src/cep.rs:LEDGER_ID].
- Keeps engine-internal issued-decision and opaque-constraint registries under `Mutex` [src/lib.rs:PolicyAuthorityEngine].

## Verification
- Run: `cargo test -p sea-forge-authority` (repo-wide: `just test`)
