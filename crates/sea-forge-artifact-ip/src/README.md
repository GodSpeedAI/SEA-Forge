# sea-forge-artifact-ip — src

## Overview
Single-module source tree for the artifact-to-IP recognition overlay (milestone
M8) [lib.rs:module docs]. Ledger records are the only truth; the catalog and
capital files are deterministic views rebuilt from them. The module implements
artifact registration, rights profiles, value evidence, lifecycle/transition
authorization and validation, attestation, and materialized-view rebuild.

## Interface
All symbols live in [lib.rs]:
- Record/data types: `ArtifactRegistrationRecord`, `ArtifactRegistrationInput`, `VerifiedArtifactRegistrationInput`, `ArtifactLifecycleInput`, `ArtifactLifecycleRecord`, `ValueEvidenceRef`, `ValueEvidenceRecord`, `ValueEvidenceSourceRecord`, `RightsProfileRecord`, `AuthorizedRightsProfile`, `AuthorizedTransition`, `AuthorizedArtifactLifecycle`, `AuthorizedAttestation`, `TransitionToken`, `TransitionInput`, `TransitionProposal`, `PendingArtifactTransition`, `ArtifactTransitionTerminal`, `ArtifactTransitionClaimManifest`, `ResumedArtifactTransition`, `ArtifactGateProfile`, `ArtifactState`, `SemanticAnchor`, `GateResult`, `Attestation`, `AttestationPolicy`, plus enums `IdentityStatus`, `TransitionKind`, `TransitionMode`, `LifecycleStatus`, `SemanticReferenceClass`, `ArtifactTransitionTerminalStatus`, `ArtifactTransitionTerminalReason`
- Registration: `register`, `register_verified`, `append_registration`, `content_identity`
- Transitions: `transition`, `validate_before_transform`, `validate_proposal_before_transform`, `canonical_transition_action`, `authorize_transition`, `append_transition`, `seal_pending_transition`, `commit_pending_transition`, `load_pending_transition`, `resume_pending_transition`, `load_completed_transition`, `commit_transition_terminal`, `load_transition_terminal`, `seal_claim_manifest`, `commit_claim_manifest`
- Lifecycle: `canonical_lifecycle_action`, `authorize_lifecycle`, `append_lifecycle`
- Rights/attestation: `canonical_rights_action`, `authorize_rights_profile`, `authorize_attestation`, `attest`, `ArtifactAttestor`, `DefaultArtifactAttestor`, `append_rights_profile`, `store_gate_profile`, `resolve_gate_profile`, `read_gate_profile`
- Evidence and references: `append_value_evidence`, `append_value_evidence_source`, `append_domain_model_ref`
- Views: `rebuild`, `rebuild_with_lifecycle`, `rebuild_materialized_views`, `load_ledger_records`, `LedgerFacts`
- Record version constant: `M8_RECORD_VERSION = "0.2"` [lib.rs:M8_RECORD_VERSION]

## Invariants
- Ledger records are the only truth; catalog and capital files are deterministic views rebuilt from them [lib.rs:module docs].
- Record schema version is pinned: `M8_RECORD_VERSION = "0.2"` [lib.rs:M8_RECORD_VERSION].
- Artifact IDs must be safe single-segment filenames [lib.rs:register ("artifact_id must be a safe single-segment filename")].
- Promotion transitions must preserve one artifact and content identity [lib.rs:validate_transition_shape].
- Each value-evidence source must have at least one accepted parent establishing its `from_stage`; no omitted parent (§M8 multi-source invariant) [lib.rs:validate_ledger_references].
- Required attestation must forbid degraded mode [lib.rs:authorize_attestation].
- A retry of an already-terminal transition case must reach the idempotent terminal path [lib.rs:commit_transition_terminal].

## Side Effects
- Writes gate profile JSON under the workspace via `fs::create_dir_all`/`fs::write` [lib.rs:store_gate_profile].
- Rebuilds materialized catalog and capital views: writes the catalog file and removes stale state/capital directories via `fs::write`, `fs::remove_file`, `fs::remove_dir_all` [lib.rs:rebuild_materialized_views].
- Appends records to ledger streams via `LedgerManager`/`LedgerStream` from `sea-forge-ledger` [lib.rs:append_registration], [lib.rs:append_transition_inner], and related `append_*` functions.

## Verification
- Run: `cargo test -p sea-forge-artifact-ip` (repo-wide: `just test`)
