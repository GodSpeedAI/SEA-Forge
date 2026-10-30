# sea-forge-artifact-ip

## Overview
SEA Forge artifact-to-IP recognition overlay (milestone M8). Implements
artifact registration, rights profiles, value evidence, lifecycle and
transition authorization/validation, attestation, and deterministic rebuild of
catalog/capital materialized views. Ledger records are the only truth; files
under the workspace are derived views [src/lib.rs:module docs].

## Interface
- Public surface is the single `lib.rs` module; key entry points include:
  - Registration: `register`, `register_verified`, `append_registration`, `content_identity` [src/lib.rs]
  - Transitions: `transition`, `authorize_transition`, `validate_before_transform`, `append_transition`, `resume_pending_transition`, `load_completed_transition` [src/lib.rs]
  - Lifecycle: `authorize_lifecycle`, `canonical_lifecycle_action`, `append_lifecycle` [src/lib.rs]
  - Rights/attestation: `authorize_rights_profile`, `authorize_attestation`, `attest`, `ArtifactAttestor`, `store_gate_profile`, `resolve_gate_profile` [src/lib.rs]
  - Evidence: `append_value_evidence`, `append_value_evidence_source`, `append_domain_model_ref` [src/lib.rs]
  - Views: `rebuild`, `rebuild_with_lifecycle`, `rebuild_materialized_views`, `load_ledger_records` [src/lib.rs]
- Record version: `M8_RECORD_VERSION = "0.2"` [src/lib.rs:M8_RECORD_VERSION]
- Source tree contract: [src/README.md]

## Invariants
- Ledger records are the only truth; catalog and capital files are deterministic views rebuilt from them [src/lib.rs:module docs].
- Artifact IDs must be safe single-segment filenames [src/lib.rs:register].
- Promotion transitions preserve one artifact and content identity [src/lib.rs:validate_transition_shape].
- Value-evidence sources each require at least one accepted parent establishing `from_stage` (§M8 multi-source invariant) [src/lib.rs:validate_ledger_references].
- Required attestation forbids degraded mode [src/lib.rs:authorize_attestation].
- Retries of already-terminal transition cases reach the idempotent terminal path [src/lib.rs:commit_transition_terminal].

## Side Effects
- Appends records to ledger streams via `sea-forge-ledger`'s `LedgerManager`/`LedgerStream` [src/lib.rs:append_registration].
- Writes gate profile and materialized catalog/capital view files under the workspace root; removes stale view state [src/lib.rs:store_gate_profile], [src/lib.rs:rebuild_materialized_views].

## Verification
- Run: `cargo test -p sea-forge-artifact-ip` (repo-wide: `just test`)
