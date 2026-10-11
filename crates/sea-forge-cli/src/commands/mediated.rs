//! Thin CLI surface over the shared mediated-authority library.
//!
//! The mediation core (bundle load, opaque constraints, evidence, decision
//! commit, authority mirrors, side-effect assurance, grant mint) lives in
//! `sea_forge_case_runner::case_ops::mediation` so case mutations authorized
//! here and mutations authorized in-process by the SFWP server share one
//! implementation. Cell-assurance inspection and policy-path resolution remain
//! CLI concerns and stay below.

use sea_forge_authority::AuthorityPolicyBundle;
use sea_forge_core::errors::ForgeError;
use sea_forge_ledger::{signing::read_verifying_key, LedgerManager};
use std::path::Path;

pub use sea_forge_case_runner::case_ops::mediation::{
    authorize_action, authorize_read, authorize_read_with_grant, with_authorized_action,
    with_authorized_action_binding, with_authorized_action_in_context, AuthorityContext,
};

pub fn policy_path(root: &Path, explicit: Option<&Path>) -> std::path::PathBuf {
    explicit
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.join("authority/active-policy.json"))
}

fn verified_assurance(
    root: &Path,
    policy_path: &Path,
    actor_id: &str,
) -> Result<(String, Option<String>), ForgeError> {
    let manager = LedgerManager::new(root)?;
    for stream_id in manager.list_stream_ids()? {
        manager.open_stream(&stream_id, "assurance")?.verify()?;
    }
    if !root.join("ledgers/global-checkpoints.jsonl").is_file() {
        return Ok(("local_tamper_evident".into(), None));
    }
    // Migrated roots are historical snapshots; their assurance level is fixed at legacy.
    if root.join("migration.json").is_file() {
        return Ok(("legacy_digest_only".into(), None));
    }
    let policy = AuthorityPolicyBundle::load(policy_path)?;
    let key_dir = policy
        .integrity_ledger
        .signing_key_dir
        .as_deref()
        .ok_or_else(|| ForgeError::Internal("ledger_integrity_error: signer unavailable".into()))?;
    let verifying_key = read_verifying_key(key_dir, &policy.integrity_ledger.signing_key_id)?;
    manager.verify_global_checkpoints(&verifying_key)?;
    if policy.integrity_ledger.min_witnesses == 0 {
        let checkpoint_hash = manager
            .read_global_checkpoints()?
            .last()
            .map(|checkpoint| checkpoint.global_checkpoint_hash.clone());
        return Ok(("checkpoint_signed".into(), checkpoint_hash));
    }
    let checkpoints = manager.read_global_checkpoints()?;
    let checkpoint = checkpoints
        .get(checkpoints.len().saturating_sub(2))
        .ok_or_else(|| {
            ForgeError::Internal("ledger_integrity_error: witnessed checkpoint missing".into())
        })?;
    let witness_keys = policy
        .integrity_ledger
        .witnesses
        .iter()
        .map(|witness| {
            Ok((
                witness.witness_id.clone(),
                read_verifying_key(&witness.key_dir, &witness.key_id)?,
            ))
        })
        .collect::<Result<Vec<_>, ForgeError>>()?;
    let witness_assurance = manager.verify_witness_receipts(
        &checkpoint.global_checkpoint_hash,
        &witness_keys,
        actor_id,
        policy.integrity_ledger.min_witnesses,
    )?;
    if witness_assurance != sea_forge_ledger::AssuranceLevel::ExternallyVerified {
        return Err(ForgeError::Internal(
            "ledger_integrity_error: insufficient independent witnesses".into(),
        ));
    }
    Ok((
        "externally_verified".into(),
        Some(checkpoint.global_checkpoint_hash.clone()),
    ))
}

pub fn assurance(root: &Path, policy_path: &Path, actor_id: &str) -> Result<String, ForgeError> {
    verified_assurance(root, policy_path, actor_id).map(|(label, _)| label)
}
