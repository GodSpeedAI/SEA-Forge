use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::{
    errors::ForgeError,
    types::{Actor, ActorRole, AuthorityAction},
};
use sea_forge_ledger::{signing::read_verifying_key, LedgerManager, LedgerStream};
use std::path::Path;

pub fn authorize_read(
    root: &Path,
    policy_path: &Path,
    actor_id: &str,
    action: &AuthorityAction,
) -> Result<(), ForgeError> {
    let bundle = AuthorityPolicyBundle::load(policy_path)?;
    authorize_with_bundle(root, actor_id, action, bundle)
}

pub fn authorize_with_bundle(
    root: &Path,
    actor_id: &str,
    action: &AuthorityAction,
    bundle: AuthorityPolicyBundle,
) -> Result<(), ForgeError> {
    let stream = LedgerStream::open(root, "authority-reads", actor_id)?;
    stream.commit_typed("authority_policy", vec![], &bundle, vec![])?;
    let integrity = bundle.integrity_ledger.clone();
    let binding = bundle.resolve_identity(actor_id, ActorRole::Operator);
    let domainforge_model = bundle.load_domainforge_model()?;
    let domainforge_candidate = domainforge_model
        .as_ref()
        .map(|model| {
            sea_forge_authority::DomainForgeCandidate::evaluate(
                model,
                action,
                vec![format!(
                    "domain-model:{}",
                    model.model_ref.semantic_model_sha256
                )],
            )
        })
        .transpose()?;
    let engine = PolicyAuthorityEngine::new(bundle)?;
    engine.load_opaque_constraints(crate::pipeline::load_opaque_constraints(root)?)?;
    let evidence = stream.commit_typed(
        "authority_evidence",
        vec![actor_id.into()],
        &serde_json::json!({"kind": "ingress_authority_request", "action": action}),
        vec![],
    )?;
    let actor = Actor {
        actor_id: actor_id.into(),
        role: ActorRole::Operator,
    };
    let decision = engine.evaluate(AuthorityEvaluation {
        actor: &actor,
        binding,
        run_id: "read_ingress",
        case_id: "read_ingress",
        plan_item_id: "read_ingress",
        sequence: 1,
        action,
        workspace_root: root,
        evidence_refs: vec![evidence.entry_ulid().into()],
        artifacts_root: None,
        timeout_secs: None,
        env_keys: Default::default(),
        domainforge_candidate: domainforge_candidate.as_ref(),
    })?;
    let committed = stream.commit_typed("authority_decision", vec![], &decision, vec![])?;
    crate::pipeline::rebuild_authority_mirrors(root, &stream, &committed)?;
    if integrity.required_for_side_effects {
        let key_dir = integrity.signing_key_dir.as_deref().ok_or_else(|| {
            ForgeError::Internal("ledger_integrity_error: signing_key_dir is required".into())
        })?;
        LedgerManager::new(root)?.create_pre_action_assurance(
            key_dir,
            &integrity.signing_key_id,
            actor_id,
            &integrity
                .witnesses
                .iter()
                .map(|witness| {
                    (
                        witness.witness_id.clone(),
                        witness.key_dir.clone(),
                        witness.key_id.clone(),
                    )
                })
                .collect::<Vec<_>>(),
            integrity.min_witnesses,
            &[&committed],
        )?;
    }
    let grant = engine.grant(&decision, &committed, action, None)?;
    grant.authorize(action, "read_ingress", "read_ingress", root)
}

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

pub fn record_assurance<T: serde::Serialize>(
    root: &Path,
    policy_path: &Path,
    actor_id: &str,
    record_kind: &str,
    record: &T,
) -> Result<String, ForgeError> {
    let (label, checkpoint_hash) = verified_assurance(root, policy_path, actor_id)?;
    let Some(checkpoint_hash) = checkpoint_hash else {
        return Ok("legacy_digest_only".into());
    };
    let manager = LedgerManager::new(root)?;
    if manager.global_checkpoint_covers_record(&checkpoint_hash, record_kind, record)? {
        Ok(label)
    } else {
        Ok("legacy_digest_only".into())
    }
}
