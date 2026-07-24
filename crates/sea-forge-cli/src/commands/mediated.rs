use sea_forge_authority::{
    ActionGrant, AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine,
};
use sea_forge_core::{
    errors::ForgeError,
    types::{Actor, ActorRole, AuthorityAction, AuthorityDecision},
};
use sea_forge_ledger::{
    signing::read_verifying_key, CommittedRecordRef, LedgerManager, LedgerStream,
};
use std::path::Path;

/// Mediate an action which will cause a governed side effect after this call.
pub fn authorize_action(
    root: &Path,
    policy_path: &Path,
    actor_id: &str,
    action: &AuthorityAction,
) -> Result<(), ForgeError> {
    with_authorized_action(root, policy_path, actor_id, action, true, |grant| {
        let context = action_context(action);
        grant.authorize(action, context, context, root)
    })
}

pub fn with_authorized_action<T>(
    root: &Path,
    policy_path: &Path,
    actor_id: &str,
    action: &AuthorityAction,
    require_side_effect_assurance: bool,
    callback: impl FnOnce(ActionGrant) -> Result<T, ForgeError>,
) -> Result<T, ForgeError> {
    let bundle = AuthorityPolicyBundle::load(policy_path)?;
    let context = action_context(action);
    authorize_with_bundle_callback(
        root,
        actor_id,
        action,
        bundle,
        AuthorityContext {
            run_id: context,
            case_id: context,
            plan_item_id: context,
            sequence: 1,
        },
        require_side_effect_assurance,
        |grant, _, _| callback(grant),
    )
}

pub struct AuthorityContext<'a> {
    pub run_id: &'a str,
    pub case_id: &'a str,
    pub plan_item_id: &'a str,
    pub sequence: usize,
}

pub fn with_authorized_action_in_context<T>(
    root: &Path,
    policy_path: &Path,
    actor_id: &str,
    action: &AuthorityAction,
    context: AuthorityContext<'_>,
    callback: impl FnOnce(ActionGrant, String) -> Result<T, ForgeError>,
) -> Result<T, ForgeError> {
    let bundle = AuthorityPolicyBundle::load(policy_path)?;
    authorize_with_bundle_callback(
        root,
        actor_id,
        action,
        bundle,
        context,
        true,
        |grant, decision, _| callback(grant, decision.decision_id),
    )
}

pub fn with_authorized_action_binding<T>(
    root: &Path,
    policy_path: &Path,
    actor_id: &str,
    action: &AuthorityAction,
    callback: impl FnOnce(ActionGrant, AuthorityDecision, CommittedRecordRef) -> Result<T, ForgeError>,
) -> Result<T, ForgeError> {
    let bundle = AuthorityPolicyBundle::load(policy_path)?;
    let context = action_context(action);
    authorize_with_bundle_callback(
        root,
        actor_id,
        action,
        bundle,
        AuthorityContext {
            run_id: context,
            case_id: context,
            plan_item_id: context,
            sequence: 1,
        },
        true,
        callback,
    )
}

/// Compatibility wrapper for read-only command callers.
pub fn authorize_read(
    root: &Path,
    policy_path: &Path,
    actor_id: &str,
    action: &AuthorityAction,
) -> Result<(), ForgeError> {
    // Reads govern no side effect, so they must not build or require
    // pre-action side-effect assurance.
    with_authorized_action(root, policy_path, actor_id, action, false, |grant| {
        let context = action_context(action);
        grant.authorize(action, context, context, root)
    })
}

/// Same as `authorize_read`, but lets the caller peek at a value derived
/// from the `ActionGrant` (e.g. a boundary-constraint cap) before it is
/// consumed by `authorize()`. Used by the manager loop to read the
/// authority-granted `max_manager_iterations` boundary (spec §16.2).
pub fn authorize_read_with_grant<T>(
    root: &Path,
    policy_path: &Path,
    actor_id: &str,
    action: &AuthorityAction,
    peek: impl FnOnce(&ActionGrant) -> T,
) -> Result<T, ForgeError> {
    with_authorized_action(root, policy_path, actor_id, action, false, |grant| {
        let context = action_context(action);
        let peeked = peek(&grant);
        grant.authorize(action, context, context, root)?;
        Ok(peeked)
    })
}

fn action_context(action: &AuthorityAction) -> &'static str {
    match action {
        AuthorityAction::Reserved { resource_type, .. }
            if resource_type == "attest_artifact_identity" =>
        {
            "attestation_ingress"
        }
        _ => "read_ingress",
    }
}

fn authorize_with_bundle_callback<T>(
    root: &Path,
    actor_id: &str,
    action: &AuthorityAction,
    bundle: AuthorityPolicyBundle,
    context: AuthorityContext<'_>,
    require_side_effect_assurance: bool,
    callback: impl FnOnce(ActionGrant, AuthorityDecision, CommittedRecordRef) -> Result<T, ForgeError>,
) -> Result<T, ForgeError> {
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
        run_id: context.run_id,
        case_id: context.case_id,
        plan_item_id: context.plan_item_id,
        sequence: context.sequence,
        action,
        workspace_root: root,
        evidence_refs: vec![evidence.entry_ulid().into()],
        artifacts_root: None,
        timeout_secs: None,
        env_keys: Default::default(),
        domainforge_candidate: domainforge_candidate.as_ref(),
        environment: None,
    })?;
    let committed = stream.commit_typed("authority_decision", vec![], &decision, vec![])?;
    crate::pipeline::rebuild_authority_mirrors(root, &stream, &committed)?;
    let assurance = if require_side_effect_assurance && integrity.required_for_side_effects {
        let key_dir = integrity.signing_key_dir.as_deref().ok_or_else(|| {
            ForgeError::Internal("ledger_integrity_error: signing_key_dir is required".into())
        })?;
        Some(
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
            )?,
        )
    } else {
        None
    };
    let grant = engine.grant(&decision, &committed, action, assurance.as_ref())?;
    callback(grant, decision, committed)
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
