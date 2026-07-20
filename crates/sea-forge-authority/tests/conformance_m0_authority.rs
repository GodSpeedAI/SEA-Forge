use sea_forge_authority::{
    resolve_candidates, AuthorityEvaluation, AuthorityPolicyBundle, GovernanceDisposition as D,
    GovernanceVerdict, PolicyAuthorityEngine, ResolutionPolicy,
};
use sea_forge_core::types::{Actor, ActorRole, AuthorityAction};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn candidate(disposition: D) -> GovernanceVerdict {
    GovernanceVerdict {
        engine: format!("{disposition:?}"),
        disposition: disposition.clone(),
        boundaries: if disposition == D::Boundary {
            BTreeMap::from([("workspace".into(), BTreeSet::from(["/safe".into()]))])
        } else {
            BTreeMap::new()
        },
        compensating_controls: if disposition == D::Degraded {
            BTreeSet::from(["audit".into()])
        } else {
            BTreeSet::new()
        },
        reason: "fixture".into(),
        evidence_refs: vec!["evi_fixture".into()],
    }
}

#[test]
fn m8_transition_rule_missing_required_contract_is_schema_error() {
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(
        r#"version: "0.1"
rules:
  - name: incomplete-transition
    verdict: allow
    actor_role: operator
    operation_kind: transition_artifact_stage
"#,
    )
    .unwrap();
    let error = PolicyAuthorityEngine::new(bundle).err().unwrap();
    assert_eq!(error.class(), "schema_error");
}

#[test]
fn settlement_authority_descriptor_selects_one_strong_adapter_and_local_cannot_be_strong() {
    let valid: AuthorityPolicyBundle = serde_yaml::from_str(
        r#"version: "0.1"
settlement_authorities:
  - authority_ref: local
    adapter: local
    trust_anchor_ref: local-kernel
    declarer_actor_id: operator
    permitted_declarer_roles: [operator]
    standing_basis: local-only
  - authority_ref: swe_seed_prod
    adapter: swe_seed
    command: ["/opt/swe-seed", "declare"]
    timeout_secs: 30
    trust_anchor_ref: key://swe-seed/current
    declarer_actor_id: swe_seed
    permitted_declarer_roles: [R-AA]
    standing_basis: externally-attested
    may_issue_strong: true
rules: []
"#,
    )
    .unwrap();
    let engine = PolicyAuthorityEngine::new(valid.clone());
    assert!(engine.is_ok());
    assert_eq!(
        valid.strong_settlement_authority().unwrap().authority_ref,
        "swe_seed_prod"
    );

    let invalid: AuthorityPolicyBundle = serde_yaml::from_str(
        r#"version: "0.1"
settlement_authorities:
  - authority_ref: local
    adapter: local
    trust_anchor_ref: local-kernel
    declarer_actor_id: operator
    permitted_declarer_roles: [operator]
    standing_basis: local-only
    may_issue_strong: true
rules: []
"#,
    )
    .unwrap();
    assert_eq!(
        PolicyAuthorityEngine::new(invalid).err().unwrap().class(),
        "settlement_authority_config_error"
    );
}

#[test]
fn m8_attestation_rule_missing_role_constraints_is_schema_error() {
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(
        r#"version: "0.1"
rules:
  - name: incomplete-attestation
    verdict: allow
    actor_role: operator
    operation_kind: attest_artifact_identity
    ledger: ifl
    degraded_mode: forbidden
"#,
    )
    .unwrap();
    let error = PolicyAuthorityEngine::new(bundle).err().unwrap();
    assert_eq!(error.class(), "schema_error");
}

#[test]
fn m8_capitalization_schema_requires_approval_promote_and_value_evidence() {
    for (requires_approval, modes, evidence) in [
        ("false", "[promote]", "[adoption]"),
        ("true", "[derive]", "[adoption]"),
        ("true", "[promote]", "[]"),
    ] {
        let yaml = format!(
            r#"version: "0.1"
rules:
  - name: capitalize
    verdict: allow
    actor_role: operator
    operation_kind: transition_artifact_stage
    transition_kind: capitalize
    from_stage: product
    to_stage: capital
    modes: {modes}
    gate_profile_ref: capital@1
    required_settlement_strength: strong
    qualifying_value_evidence_kinds: {evidence}
    requires_approval: {requires_approval}
"#
        );
        let bundle: AuthorityPolicyBundle = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(
            PolicyAuthorityEngine::new(bundle).err().unwrap().class(),
            "schema_error"
        );
    }
}

#[test]
fn m12_agent_probe_requires_exact_endpoint_policy_and_grant_binding() {
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(
        r#"version: "0.1"
policy_surfaces:
  external_api:
    mode: deny-by-default
    allow_hosts: [127.0.0.1]
rules:
  - name: allow-agent-probe
    verdict: allow
    actor_role: operator
    operation_kind: agent_probe
"#,
    )
    .unwrap();
    let binding = bundle.resolve_identity("operator_local", ActorRole::Operator);
    let engine = PolicyAuthorityEngine::new(bundle).unwrap();
    let actor = Actor {
        actor_id: "operator_local".into(),
        role: ActorRole::Operator,
    };
    let action = AuthorityAction::AgentProbe {
        endpoint_ref: "local-test".into(),
        descriptor_config_sha256: "sha256:descriptor".into(),
        provider_kind: "openai_compatible".into(),
        scheme: "http".into(),
        host: "127.0.0.1".into(),
        port: 8080,
        path: "/".into(),
        model: "test-model".into(),
        max_request_bytes: 1024,
        max_response_bytes: 2048,
        timeout_secs: 5,
        credential_ref: Some("TEST_KEY".into()),
        prompt_sha256: "sha256:prompt".into(),
    };
    let decision = engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding,
            run_id: "run_agent_probe",
            case_id: "case_agent_probe",
            plan_item_id: "item_agent_probe",
            sequence: 1,
            action: &action,
            workspace_root: Path::new("/tmp/workspace"),
            evidence_refs: vec!["intent:agent_probe".into()],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            environment: None,
        })
        .unwrap();
    assert_eq!(decision.verdict, sea_forge_core::types::Verdict::Allow);

    let root = tempfile::tempdir().unwrap();
    let stream =
        sea_forge_ledger::LedgerStream::open(root.path(), "agent-probe-authority", "test").unwrap();
    let committed = stream
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    engine
        .grant(&decision, &committed, &action, None)
        .expect("exact agent probe action should mint a grant");

    let substituted = match action.clone() {
        AuthorityAction::AgentProbe {
            endpoint_ref,
            descriptor_config_sha256,
            provider_kind,
            scheme,
            host,
            port,
            path,
            max_request_bytes,
            max_response_bytes,
            timeout_secs,
            credential_ref,
            prompt_sha256,
            ..
        } => AuthorityAction::AgentProbe {
            endpoint_ref,
            descriptor_config_sha256,
            provider_kind,
            scheme,
            host,
            port,
            path,
            model: "different-model".into(),
            max_request_bytes,
            max_response_bytes,
            timeout_secs,
            credential_ref,
            prompt_sha256,
        },
        _ => unreachable!(),
    };
    assert!(engine
        .grant(&decision, &committed, &substituted, None)
        .is_err());
}

#[test]
fn m13_agent_task_requires_exact_endpoint_policy_and_grant_binding() {
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(
        r#"version: "0.1"
policy_surfaces:
  external_api:
    mode: deny-by-default
    allow_hosts: [127.0.0.1]
rules:
  - name: allow-agent-task
    verdict: allow
    actor_role: operator
    operation_kind: agent_task
"#,
    )
    .unwrap();
    let binding = bundle.resolve_identity("operator_local", ActorRole::Operator);
    let engine = PolicyAuthorityEngine::new(bundle).unwrap();
    let actor = Actor {
        actor_id: "operator_local".into(),
        role: ActorRole::Operator,
    };
    let action = AuthorityAction::AgentTask {
        endpoint_ref: "local-test".into(),
        descriptor_config_sha256: "sha256:descriptor".into(),
        provider_kind: "openai_compatible".into(),
        scheme: "http".into(),
        host: "127.0.0.1".into(),
        port: 8080,
        path: "/".into(),
        model: "test-model".into(),
        max_request_bytes: 4096,
        max_response_bytes: 8192,
        timeout_secs: 30,
        credential_ref: Some("TEST_KEY".into()),
        instruction_sha256: "sha256:instruction".into(),
        max_turns: 5,
        token_budget: Some(4096),
    };
    let decision = engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding,
            run_id: "run_agent_task",
            case_id: "case_agent_task",
            plan_item_id: "item_agent_task",
            sequence: 1,
            action: &action,
            workspace_root: Path::new("/tmp/workspace"),
            evidence_refs: vec!["intent:agent_task".into()],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            environment: None,
        })
        .unwrap();
    assert_eq!(decision.verdict, sea_forge_core::types::Verdict::Allow);

    let root = tempfile::tempdir().unwrap();
    let stream =
        sea_forge_ledger::LedgerStream::open(root.path(), "agent-task-authority", "test").unwrap();
    let committed = stream
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    engine
        .grant(&decision, &committed, &action, None)
        .expect("exact agent task action should mint a grant");

    // Substituting max_turns must break the grant binding.
    let substituted = match action.clone() {
        AuthorityAction::AgentTask {
            endpoint_ref,
            descriptor_config_sha256,
            provider_kind,
            scheme,
            host,
            port,
            path,
            model,
            max_request_bytes,
            max_response_bytes,
            timeout_secs,
            credential_ref,
            instruction_sha256,
            token_budget,
            ..
        } => AuthorityAction::AgentTask {
            endpoint_ref,
            descriptor_config_sha256,
            provider_kind,
            scheme,
            host,
            port,
            path,
            model,
            max_request_bytes,
            max_response_bytes,
            timeout_secs,
            credential_ref,
            instruction_sha256,
            max_turns: 99,
            token_budget,
        },
        _ => unreachable!(),
    };
    assert!(engine
        .grant(&decision, &committed, &substituted, None)
        .is_err());
}

fn expected(left: &D, right: &D) -> D {
    if left == &D::Deny || right == &D::Deny {
        D::Deny
    } else if left == &D::Escalate || right == &D::Escalate {
        D::Escalate
    } else if left == &D::Boundary || right == &D::Boundary {
        D::Boundary
    } else if left == &D::Degraded || right == &D::Degraded {
        D::Degraded
    } else {
        D::Allow
    }
}

#[test]
fn complete_pairwise_matrix_is_commutative() {
    let dispositions = [D::Allow, D::Deny, D::Escalate, D::Boundary, D::Degraded];
    let policy = ResolutionPolicy {
        allow_degraded: true,
    };
    for left in &dispositions {
        for right in &dispositions {
            let forward = resolve_candidates(
                &[candidate(left.clone()), candidate(right.clone())],
                &policy,
            );
            let reverse = resolve_candidates(
                &[candidate(right.clone()), candidate(left.clone())],
                &policy,
            );
            assert_eq!(forward, reverse, "{left:?} with {right:?}");
            assert_eq!(forward.disposition, expected(left, right));
        }
    }
}

#[test]
fn resolution_is_idempotent_and_group_order_independent() {
    let policy = ResolutionPolicy {
        allow_degraded: true,
    };
    for disposition in [D::Allow, D::Deny, D::Escalate, D::Boundary, D::Degraded] {
        assert_eq!(
            resolve_candidates(&[candidate(disposition.clone())], &policy),
            resolve_candidates(
                &[candidate(disposition.clone()), candidate(disposition)],
                &policy,
            )
        );
    }
    let first = resolve_candidates(
        &[
            candidate(D::Allow),
            candidate(D::Boundary),
            candidate(D::Escalate),
        ],
        &policy,
    );
    let regrouped = resolve_candidates(
        &[
            candidate(D::Escalate),
            candidate(D::Allow),
            candidate(D::Boundary),
        ],
        &policy,
    );
    assert_eq!(first, regrouped);
}

#[test]
fn domainforge_authority_is_model_semantic() {
    let source = include_str!("../../sea-forge-domainforge/tests/fixtures/demo.sea");
    let source_hash = {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(source.as_bytes()))
    };
    let model = sea_forge_domainforge::load_validate(&sea_forge_domainforge::SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![sea_forge_domainforge::SourceFile {
            uri: "demo.sea".into(),
            sha256: source_hash,
            content: source.into(),
        }],
    })
    .unwrap();
    let matching = sea_forge_domainforge::evaluate_authority(
        &model,
        "write_file",
        "Sample.sea",
        vec!["evi_model".into()],
    )
    .unwrap();
    let unrelated = sea_forge_domainforge::evaluate_authority(
        &model,
        "write_file",
        "Unknown.sea",
        vec!["evi_model".into()],
    )
    .unwrap();
    assert_eq!(
        matching.normalized_disposition,
        sea_forge_domainforge::CandidateDisposition::Allow
    );
    assert_eq!(
        unrelated.normalized_disposition,
        sea_forge_domainforge::CandidateDisposition::Deny
    );
}
