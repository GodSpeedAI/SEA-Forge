# Task 16 M8 Substrate Reconciliation

Updated: 2026-07-15

## Inspected and reused unchanged

- `sea-forge-core::types`: `ArtifactDescriptor`, `ArtifactStage`, `ApprovalRequest`, `SettlementCriteriaRecord`, `SettlementEvent`, `SettlementDeclaration`, `AuthorityDecision`, `CasePlan`, and `PlanItem`.
- `sea-forge-ledger`: canonical hashing, verified streams, `LedgerStream::commit_typed`, append ordinals, and committed-record references.
- `sea-forge-authority`: `PolicyAuthorityEngine`, exact one-use `ActionGrant`, `AuthorityAction::Reserved`, RBAC, SoD, and fail-closed resolution.
- Planner criteria derivation, M7 `EnvironmentSpec`/`Evaluator`, settlement declarations, `DomainModelRef`, and projection rebuild conventions.

## Extended

- Existing policy rules now carry and enforce spec-full §8.2 transition and attestation fields against exact action parameters.
- Existing settlement, approval-resolution, evidence, and declaration bodies are ledgered before compatibility views.
- Existing plan-case execution runs M8 gate evaluators through the M7 authority/runtime path.
- Existing CLI mediation passes exact context-bound grants to opaque M8 commit and attestation boundaries.

## Added because no equivalent existed

- `sea-forge-artifact-ip`: additive v0.2 registration identity, inherited lineage, immutable gate profiles, TransitionTokens, typed ledger reference validation, and rebuildable catalog/capital projections.
- Ledgered rights and value-evidence records. Value evidence resolves accepted out-of-case run evidence and settlement; a caller boolean or relabeled generic evidence is insufficient.
- Opaque `AuthorizedTransition` and `AuthorizedAttestation` values.
- Default IFL adapter backed by the M0 ledger, returning `ifl:token:<ledger>:<append_ordinal>`.

## Deliberately rejected duplicates

- No second authority/evaluator engine, criteria store, settlement protocol, semantic identity system, artifact database, mutable stage/reuse count, migration, or replacement artifact ID.
- No persisted `Operation::ArtifactTransition`; the existing case plan plus exact `Reserved` authority action remains the boundary.

## Resolved capability (post-continuation)

- End-to-end CLI capitalization is implemented: the resume command performs
  pending-transition resume after approval, invokes a configured strong
  external `SettlementAuthority` transport, and finalizes with a one-token
  declaration. The earlier "remaining missing capability" note below was a
  pre-continuation snapshot and is no longer accurate; see
  `crates/sea-forge-cli/src/commands/resume.rs` and the M8 conformance tests
  for the resolved status.
