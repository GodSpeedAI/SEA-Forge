use sea_forge_authority::{
    resolve_candidates, AuthorityPolicyBundle, GovernanceDisposition as D, GovernanceVerdict,
    PolicyAuthorityEngine, ResolutionPolicy,
};
use std::collections::{BTreeMap, BTreeSet};

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
