use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole, AuthorityAction, Verdict};
use sea_forge_ledger::LedgerStream;
use serde_json::json;

fn action(server: &str, tool: &str, request_digest: &str) -> AuthorityAction {
    AuthorityAction::Reserved {
        resource_type: "mcp_tool_call".into(),
        resource_id: json!([server, tool]).to_string(),
        parameters: json!({"server":server,"tool":tool,"request_sha256":request_digest}),
    }
}

#[test]
fn mcp_scope_and_exact_request_digest_share_the_existing_authority_grant() {
    let root = tempfile::tempdir().unwrap();
    let bundle: AuthorityPolicyBundle =
        sea_forge_core::yaml::from_str(include_str!("../../../tests/security-v2/mcp-policy.yaml"))
            .unwrap();
    let actor = Actor {
        actor_id: "metadata-workload".into(),
        role: ActorRole::Operator,
    };
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let evaluate = |action: &AuthorityAction| {
        engine
            .evaluate(AuthorityEvaluation {
                actor: &actor,
                binding: bundle.resolve_identity(&actor.actor_id, actor.role.clone()),
                run_id: "run-mcp-unit",
                case_id: "case-mcp-unit",
                plan_item_id: "tool",
                sequence: 1,
                action,
                workspace_root: root.path(),
                evidence_refs: vec![],
                artifacts_root: None,
                timeout_secs: Some(10),
                env_keys: Default::default(),
                opa_candidate: None,
                environment: None,
            })
            .unwrap()
    };
    let allowed = action("security-upstream", "metadata.read", &"a".repeat(64));
    let decision = evaluate(&allowed);
    assert_eq!(decision.verdict, Verdict::Allow);
    for denied in [
        action("other-server", "metadata.read", &"a".repeat(64)),
        action("security-upstream", "audit.delete", &"a".repeat(64)),
    ] {
        assert_eq!(evaluate(&denied).verdict, Verdict::Deny);
    }
    let ledger = LedgerStream::open(root.path(), "run-mcp-unit", "metadata-workload").unwrap();
    let committed = ledger
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    engine
        .grant(&decision, &committed, &allowed, None)
        .unwrap()
        .authorize(&allowed, "run-mcp-unit", "tool", root.path())
        .unwrap();
    let changed_request = action("security-upstream", "metadata.read", &"b".repeat(64));
    assert!(engine.grant(&decision, &committed, &allowed, None).is_err());
    let decision = evaluate(&allowed);
    let committed = ledger
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    assert!(engine
        .grant(&decision, &committed, &allowed, None)
        .unwrap()
        .authorize(&changed_request, "run-mcp-unit", "tool", root.path())
        .is_err());
    let mut unscoped = bundle;
    unscoped.rules[0].boundary_constraints.remove("mcp_tool");
    assert!(PolicyAuthorityEngine::new(unscoped).is_err());
}
