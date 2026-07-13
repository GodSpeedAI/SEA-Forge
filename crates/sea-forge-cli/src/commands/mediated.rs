use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::{
    errors::ForgeError,
    types::{Actor, ActorRole, ActorType, AuthorityAction, BindingResolution, IdentityBinding},
};
use sea_forge_ledger::LedgerStream;
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
    let engine = PolicyAuthorityEngine::new(bundle)?;
    let actor = Actor {
        actor_id: actor_id.into(),
        role: ActorRole::Operator,
    };
    let decision = engine.evaluate(AuthorityEvaluation {
        actor: &actor,
        binding: IdentityBinding {
            principal: actor_id.into(),
            actor_type: ActorType::Human,
            binding_resolution: BindingResolution::LocalDefault,
            identity_binding_source: "cli-read".into(),
            sponsor: None,
        },
        run_id: "read_ingress",
        plan_item_id: "read_ingress",
        sequence: 1,
        action,
        workspace_root: root,
        evidence_refs: vec![],
        artifacts_root: None,
        timeout_secs: None,
        env_keys: Default::default(),
    })?;
    let committed = stream.commit_typed("authority_decision", vec![], &decision, vec![])?;
    let grant = engine.grant(&decision, &committed, action)?;
    grant.authorize(action, "read_ingress", "read_ingress", root)
}

pub fn policy_path(root: &Path, explicit: Option<&Path>) -> std::path::PathBuf {
    explicit
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.join("authority/active-policy.json"))
}
