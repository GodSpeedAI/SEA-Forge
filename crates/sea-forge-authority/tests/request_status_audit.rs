use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole, AuthorityAction, Verdict};
use serde_json::json;

fn verdict(policy: &str, request_id: &str) -> Verdict {
    let root = tempfile::tempdir().unwrap();
    let policy_path = root.path().join("policy.yaml");
    std::fs::write(&policy_path, policy).unwrap();
    let bundle = AuthorityPolicyBundle::load(&policy_path).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let actor = Actor {
        actor_id: "auditor".into(),
        role: ActorRole::Operator,
    };
    let action = AuthorityAction::Reserved {
        resource_type: "request_status_audit".into(),
        resource_id: request_id.into(),
        parameters: json!({"request_id": request_id}),
    };
    engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding: bundle.resolve_identity(&actor.actor_id, actor.role.clone()),
            run_id: "audit",
            case_id: "audit",
            plan_item_id: "request_status_audit",
            sequence: 1,
            action: &action,
            workspace_root: root.path(),
            evidence_refs: vec![],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            opa_candidate: None,
            environment: None,
        })
        .unwrap()
        .verdict
}

#[test]
fn request_status_audit_requires_an_exact_request_rule() {
    let exact = "version: '0.1'\nrules:\n  - name: audit-one\n    verdict: allow\n    actor_role: operator\n    operation_kind: request_status_audit\n    resources: [req-one]\n";
    let broad = "version: '0.1'\nrules:\n  - name: audit-all\n    verdict: allow\n    actor_role: operator\n    operation_kind: request_status_audit\n";
    assert_eq!(verdict(exact, "req-one"), Verdict::Allow);
    assert_eq!(verdict(exact, "req-two"), Verdict::Deny);
    assert_eq!(verdict(broad, "req-one"), Verdict::Deny);
}
