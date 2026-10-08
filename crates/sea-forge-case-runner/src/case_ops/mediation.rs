//! Mediated authority checks for governed case mutations.
//!
//! Extracted verbatim from `sea-forge-cli`'s `commands/mediated.rs` so that a
//! case mutation authorized here (by the CLI or by the server, in-process) and
//! one authorized by any other CLI command run through the same engine share
//! one implementation. The stream, evidence and mirror writes are part of the
//! contract: a mutation is not authorized until its decision is ledgered.

use sea_forge_authority::{
    ActionGrant, AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine,
};
use sea_forge_core::{
    errors::ForgeError,
    types::{Actor, ActorRole, AuthorityAction, AuthorityDecision},
};
use sea_forge_ledger::{CommittedRecordRef, LedgerManager, LedgerStream};
use std::path::Path;

use super::authority_views::{load_opaque_constraints, rebuild_authority_mirrors};

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

/// Same as [`authorize_read`], but lets the caller peek at a value derived
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

pub(super) fn authorize_with_bundle_callback<T>(
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
    engine.load_opaque_constraints(load_opaque_constraints(root)?)?;
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
    rebuild_authority_mirrors(root, &stream, &committed)?;
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
