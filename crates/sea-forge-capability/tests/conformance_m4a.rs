use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use sea_forge_capability::promotion::{
    build_capability_record, declaration_qualifies, default_v02_policy, parse_fixed,
    rebuild_capability, require_proven, save_policy,
};
use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::*;
use sea_forge_settlement::declaration::{
    append_declaration, LocalSettlementAuthority, SettlementAuthority, SweSeedResponse,
    SweSeedSettlementAuthority, SweSeedTransport,
};

// ── Test helpers ──

struct FixedTransport {
    response: SweSeedResponse,
}

impl SweSeedTransport for FixedTransport {
    fn submit(&self, _req: &SettlementDeclarationRequest) -> Result<SweSeedResponse, ForgeError> {
        Ok(SweSeedResponse {
            attestation_ref: self.response.attestation_ref.clone(),
            attribution_confidence: self.response.attribution_confidence.clone(),
            gaming_exposure: self.response.gaming_exposure.clone(),
            hidden_debt_blindness: self.response.hidden_debt_blindness.clone(),
            feedback_delay_ms: self.response.feedback_delay_ms,
        })
    }
}

fn good_reliability() -> SweSeedResponse {
    SweSeedResponse {
        attestation_ref: "swe_seed_attest_001".into(),
        attribution_confidence: "0.950000".into(),
        gaming_exposure: "0.050000".into(),
        hidden_debt_blindness: "0.100000".into(),
        feedback_delay_ms: 5000,
    }
}

fn gameable_reliability() -> SweSeedResponse {
    SweSeedResponse {
        attestation_ref: "swe_seed_attest_gameable".into(),
        attribution_confidence: "0.950000".into(),
        gaming_exposure: "0.800000".into(),
        hidden_debt_blindness: "0.100000".into(),
        feedback_delay_ms: 5000,
    }
}

fn low_attribution_reliability() -> SweSeedResponse {
    SweSeedResponse {
        attestation_ref: "swe_seed_attest_low_attr".into(),
        attribution_confidence: "0.300000".into(),
        gaming_exposure: "0.050000".into(),
        hidden_debt_blindness: "0.100000".into(),
        feedback_delay_ms: 5000,
    }
}

fn make_envelope(run_id: &str, capability: &str, result: SettlementStatus) -> SemanticEnvelope {
    SemanticEnvelope {
        version: "0.1".into(),
        run_id: run_id.into(),
        case_ref: "case_test".into(),
        intent: Intent {
            intent_id: "int_test".into(),
            summary: format!("test {capability}"),
            actor_id: "entity_a".into(),
            process_id: "proc_test".into(),
            created_at: "2026-07-14T00:00:00Z".into(),
        },
        plan_ref: "plan_test".into(),
        template_ref: None,
        authority_decisions: vec![],
        evidence_refs: vec!["evd_0001".into()],
        settlement_ref: "set_0001".into(),
        capability_delta: CapabilityDelta {
            attempted_capability: capability.into(),
            result,
        },
        attribution: Attribution {
            entity_id: "entity_a".into(),
            process_id: "proc_test".into(),
            session_id: "sess_test".into(),
        },
        artifact_refs: vec![],
        extension_refs: vec![],
        projection_refs: vec![],
        cell_id: None,
    }
}

#[allow(clippy::too_many_arguments)]
fn make_decl_req(
    run_id: &str,
    plan_item_id: &str,
    acting_entity: &str,
    declarer_id: &str,
    strength: SettlementStrength,
    criteria_declared_at: &str,
    execution_started_at: &str,
    variation: Vec<(&str, &str)>,
    disruptions: Vec<&str>,
    burden: Option<&str>,
) -> SettlementDeclarationRequest {
    let mut variation_tags = BTreeMap::new();
    for (k, v) in variation {
        variation_tags.insert(k.into(), v.into());
    }
    SettlementDeclarationRequest {
        settlement_ref: "set_0001".into(),
        run_id: run_id.into(),
        case_id: "case_test".into(),
        plan_item_id: plan_item_id.into(),
        claim_manifest_sha256: "sha256:abc123".into(),
        criteria_ref: "crit_abc123".into(),
        criteria_sha256: "sha256:def456".into(),
        criteria_record_hash: "sha256:ghi789".into(),
        criteria_declared_at: criteria_declared_at.into(),
        execution_started_at: execution_started_at.into(),
        job_contract_ref: None,
        origin_refs: vec![OriginRef {
            kind: OriginRefKind::Intent,
            reference: "int_test".into(),
            sha256: "sha256:origin1".into(),
            role: OriginRole::DesiredResult,
            evidence_refs: vec![],
            domain_model_ref: None,
        }],
        verifier_ref: "builtin:local".into(),
        verifier_sha256: "sha256:verifier".into(),
        acting_entity_id: acting_entity.into(),
        requested_strength: strength,
        declarer: Declarer {
            actor_id: declarer_id.into(),
            authority_ref: "swe_seed".into(),
            role: "R-AA".into(),
            standing_basis: "external_verification".into(),
        },
        variation_tags,
        disruption_tags: disruptions.into_iter().map(String::from).collect(),
        orchestration_burden: burden.map(String::from),
        source_evidence_refs: vec!["evd_0001".into()],
        authored_by: None,
    }
}

fn setup_root(root: &Path) {
    fs::create_dir_all(root.join(".sea-forge/settlement")).unwrap();
    fs::create_dir_all(root.join(".sea-forge/capabilities/policies")).unwrap();
}

fn write_envelopes(root: &Path, envelopes: &[SemanticEnvelope]) {
    let path = root.join(".sea-forge/capabilities.jsonl");
    let mut data = String::new();
    for env in envelopes {
        data.push_str(&serde_json::to_string(env).unwrap());
        data.push('\n');
    }
    fs::write(path, data).unwrap();
}

const CAP: &str = "generate_and_validate_sea_model";

// ── Tests ──

#[test]
fn m4a_raw_counts_match_5_mixed_runs() {
    let root = tempfile::tempdir().unwrap();
    setup_root(root.path());

    let envelopes = vec![
        make_envelope("run_001", CAP, SettlementStatus::Accepted),
        make_envelope("run_002", CAP, SettlementStatus::Accepted),
        make_envelope("run_003", CAP, SettlementStatus::Accepted),
        make_envelope("run_004", CAP, SettlementStatus::Rejected),
        make_envelope("run_005", CAP, SettlementStatus::Escalated),
    ];
    write_envelopes(root.path(), &envelopes);

    let policy = default_v02_policy();
    save_policy(root.path(), &policy).unwrap();

    let record = rebuild_capability(root.path(), CAP, &policy).unwrap();

    assert_eq!(record.counts.accepted, 3);
    assert_eq!(record.counts.rejected, 1);
    assert_eq!(record.counts.escalated, 1);
    assert_eq!(record.status, CapabilityStatus::Attempted);
}

#[test]
fn m4a_local_declaration_zero_qualifying_weight() {
    let root = tempfile::tempdir().unwrap();
    setup_root(root.path());

    let envelopes = vec![make_envelope("run_001", CAP, SettlementStatus::Accepted)];
    write_envelopes(root.path(), &envelopes);

    let local = LocalSettlementAuthority::new("entity_a");
    let req = make_decl_req(
        "run_001",
        CAP,
        "entity_a",
        "entity_b", // different declarer
        SettlementStrength::Local,
        "2026-07-13T00:00:00Z",
        "2026-07-14T00:00:00Z",
        vec![],
        vec![],
        None,
    );
    let decl = local.declare(&req).unwrap();
    assert_eq!(decl.status, DeclarationStatus::Accepted);
    assert_eq!(decl.strength, SettlementStrength::Local);
    assert!(!decl.qualifies_for_capability);

    append_declaration(
        &root.path().join(".sea-forge/settlement/declarations.jsonl"),
        &decl,
    )
    .unwrap();

    let policy = default_v02_policy();
    save_policy(root.path(), &policy).unwrap();

    let record = rebuild_capability(root.path(), CAP, &policy).unwrap();

    // Observed (counts from envelope) but zero qualifying weight
    assert_eq!(record.counts.accepted, 1);
    assert_eq!(record.qualifying.declaration_count, 0);
    assert_eq!(parse_fixed(&record.qualifying.accepted_weight), 0);
}

#[test]
fn m4a_post_hoc_criteria_integrity_failure() {
    let root = tempfile::tempdir().unwrap();
    setup_root(root.path());

    write_envelopes(
        root.path(),
        &[make_envelope("run_001", CAP, SettlementStatus::Accepted)],
    );

    let transport = FixedTransport {
        response: good_reliability(),
    };
    let authority = SweSeedSettlementAuthority::new(transport, "entity_a");

    // Criteria declared AFTER execution started
    let req = make_decl_req(
        "run_001",
        CAP,
        "entity_a",
        "entity_b",
        SettlementStrength::Strong,
        "2026-07-15T00:00:00Z", // criteria_declared_at
        "2026-07-14T00:00:00Z", // execution_started_at (before criteria!)
        vec![],
        vec![],
        None,
    );
    let decl = authority.declare(&req).unwrap();

    // Integrity failure → status rejected, does not qualify
    assert_eq!(decl.status, DeclarationStatus::Rejected);
    assert!(!decl.qualifies_for_capability);

    append_declaration(
        &root.path().join(".sea-forge/settlement/declarations.jsonl"),
        &decl,
    )
    .unwrap();

    let policy = default_v02_policy();
    save_policy(root.path(), &policy).unwrap();
    let record = rebuild_capability(root.path(), CAP, &policy).unwrap();
    assert_eq!(record.qualifying.declaration_count, 0);
}

#[test]
fn m4a_self_declaration_integrity_failure() {
    let root = tempfile::tempdir().unwrap();
    setup_root(root.path());

    write_envelopes(
        root.path(),
        &[make_envelope("run_001", CAP, SettlementStatus::Accepted)],
    );

    let transport = FixedTransport {
        response: good_reliability(),
    };
    let authority = SweSeedSettlementAuthority::new(transport, "entity_a");

    // Declarer IS the acting entity
    let req = make_decl_req(
        "run_001",
        CAP,
        "entity_a",
        "entity_a", // same as acting_entity!
        SettlementStrength::Strong,
        "2026-07-13T00:00:00Z",
        "2026-07-14T00:00:00Z",
        vec![],
        vec![],
        None,
    );
    let decl = authority.declare(&req).unwrap();

    assert_eq!(decl.status, DeclarationStatus::Rejected);
    assert!(!decl.qualifies_for_capability);
    assert!(!decl.independence.independent);

    append_declaration(
        &root.path().join(".sea-forge/settlement/declarations.jsonl"),
        &decl,
    )
    .unwrap();

    let policy = default_v02_policy();
    save_policy(root.path(), &policy).unwrap();
    let record = rebuild_capability(root.path(), CAP, &policy).unwrap();
    assert_eq!(record.qualifying.declaration_count, 0);
}

#[test]
fn m4a_gameable_feedback_weight_below_threshold() {
    let root = tempfile::tempdir().unwrap();
    setup_root(root.path());

    write_envelopes(
        root.path(),
        &[make_envelope("run_001", CAP, SettlementStatus::Accepted)],
    );

    // High gaming exposure → weight below 0.8 threshold
    let transport = FixedTransport {
        response: gameable_reliability(),
    };
    let authority = SweSeedSettlementAuthority::new(transport, "entity_a");

    let req = make_decl_req(
        "run_001",
        CAP,
        "entity_a",
        "entity_b",
        SettlementStrength::Strong,
        "2026-07-13T00:00:00Z",
        "2026-07-14T00:00:00Z",
        vec![("model_scale", "small"), ("input_type", "text")],
        vec![],
        Some("0.800000"),
    );
    let decl = authority.declare(&req).unwrap();
    assert_eq!(decl.status, DeclarationStatus::Accepted);
    // Weight should be low: 0.95 * 0.20 * 0.90 ≈ 0.171
    let weight = parse_fixed(&decl.reliability.weight);
    assert!(
        weight < 200_000,
        "expected weight < 0.2, got {}",
        decl.reliability.weight
    );

    append_declaration(
        &root.path().join(".sea-forge/settlement/declarations.jsonl"),
        &decl,
    )
    .unwrap();

    let policy = default_v02_policy();
    save_policy(root.path(), &policy).unwrap();
    let record = rebuild_capability(root.path(), CAP, &policy).unwrap();
    // Declaration doesn't qualify because weight < min_reliability_weight (0.8)
    assert_eq!(record.qualifying.declaration_count, 0);
}

#[test]
fn m4a_low_attribution_weight_below_threshold() {
    let root = tempfile::tempdir().unwrap();
    setup_root(root.path());

    write_envelopes(
        root.path(),
        &[make_envelope("run_001", CAP, SettlementStatus::Accepted)],
    );

    let transport = FixedTransport {
        response: low_attribution_reliability(),
    };
    let authority = SweSeedSettlementAuthority::new(transport, "entity_a");

    let req = make_decl_req(
        "run_001",
        CAP,
        "entity_a",
        "entity_b",
        SettlementStrength::Strong,
        "2026-07-13T00:00:00Z",
        "2026-07-14T00:00:00Z",
        vec![("model_scale", "small"), ("input_type", "text")],
        vec![],
        Some("0.800000"),
    );
    let decl = authority.declare(&req).unwrap();
    // Low attribution: 0.30 * 0.95 * 0.90 ≈ 0.256
    let weight = parse_fixed(&decl.reliability.weight);
    assert!(
        weight < 300_000,
        "expected weight < 0.3, got {}",
        decl.reliability.weight
    );

    append_declaration(
        &root.path().join(".sea-forge/settlement/declarations.jsonl"),
        &decl,
    )
    .unwrap();

    let policy = default_v02_policy();
    save_policy(root.path(), &policy).unwrap();
    let record = rebuild_capability(root.path(), CAP, &policy).unwrap();
    assert_eq!(record.qualifying.declaration_count, 0);
}

#[test]
fn m4a_three_qualifying_declarations_promotion_to_proven() {
    let root = tempfile::tempdir().unwrap();
    setup_root(root.path());

    let envelopes = vec![
        make_envelope("run_001", CAP, SettlementStatus::Accepted),
        make_envelope("run_002", CAP, SettlementStatus::Accepted),
        make_envelope("run_003", CAP, SettlementStatus::Accepted),
    ];
    write_envelopes(root.path(), &envelopes);

    let transport = FixedTransport {
        response: good_reliability(),
    };
    let authority = SweSeedSettlementAuthority::new(transport, "entity_a");

    let decl_path = root.path().join(".sea-forge/settlement/declarations.jsonl");

    // Three qualifying declarations spanning all required variation dimensions
    // with decreasing orchestration burden (baseline 0.9 → current 0.7)
    let variations = [
        (
            vec![("model_scale", "small"), ("input_type", "text")],
            vec!["disk_full"],
            Some("0.900000"),
        ),
        (
            vec![("model_scale", "large"), ("input_type", "image")],
            vec![],
            Some("0.800000"),
        ),
        (
            vec![("model_scale", "medium"), ("input_type", "audio")],
            vec![],
            Some("0.700000"),
        ),
    ];

    for (i, (var, disruptions, burden)) in variations.iter().enumerate() {
        let req = make_decl_req(
            &format!("run_{:03}", i + 1),
            CAP,
            "entity_a",
            "entity_b",
            SettlementStrength::Strong,
            "2026-07-13T00:00:00Z",
            "2026-07-14T00:00:00Z",
            var.clone(),
            disruptions.clone(),
            *burden,
        );
        let decl = authority.declare(&req).unwrap();
        assert_eq!(decl.status, DeclarationStatus::Accepted);
        assert!(decl.qualifies_for_capability);
        append_declaration(&decl_path, &decl).unwrap();
    }

    let policy = default_v02_policy();
    save_policy(root.path(), &policy).unwrap();

    let record = rebuild_capability(root.path(), CAP, &policy).unwrap();

    assert_eq!(record.qualifying.declaration_count, 3);
    assert_eq!(
        record.status,
        CapabilityStatus::Proven,
        "status should be proven, got {:?}",
        record.status
    );
    // Coverage: both dimensions covered with >= 2 distinct values each
    assert_eq!(parse_fixed(&record.variation.coverage_ratio), 1_000_000);
    // Recovery: disk_full recovered
    assert_eq!(parse_fixed(&record.recovery.recovery_ratio), 1_000_000);
    // Regression weight < 0.5
    assert!(parse_fixed(&record.qualifying.regression_weight) < 500_000);
    // Burden reduction
    assert!(record.orchestration.reduction.is_some());
}

#[test]
fn m4a_repeated_variation_no_coverage_increase() {
    let root = tempfile::tempdir().unwrap();
    setup_root(root.path());

    write_envelopes(
        root.path(),
        &[
            make_envelope("run_001", CAP, SettlementStatus::Accepted),
            make_envelope("run_002", CAP, SettlementStatus::Accepted),
            make_envelope("run_003", CAP, SettlementStatus::Accepted),
        ],
    );

    let transport = FixedTransport {
        response: good_reliability(),
    };
    let authority = SweSeedSettlementAuthority::new(transport, "entity_a");

    let decl_path = root.path().join(".sea-forge/settlement/declarations.jsonl");

    // Same variation value repeated 3 times
    for i in 1..=3 {
        let req = make_decl_req(
            &format!("run_{:03}", i),
            CAP,
            "entity_a",
            "entity_b",
            SettlementStrength::Strong,
            "2026-07-13T00:00:00Z",
            "2026-07-14T00:00:00Z",
            vec![("model_scale", "small"), ("input_type", "text")], // same every time
            vec!["disk_full"],
            Some("0.900000"),
        );
        let decl = authority.declare(&req).unwrap();
        append_declaration(&decl_path, &decl).unwrap();
    }

    let policy = default_v02_policy();
    save_policy(root.path(), &policy).unwrap();

    let record = rebuild_capability(root.path(), CAP, &policy).unwrap();

    // Counts rise (3 declarations) but coverage does not (only 1 distinct value per dimension)
    assert_eq!(record.qualifying.declaration_count, 3);
    // Coverage < 1.0 because min_distinct_values_per_dimension is 2
    let coverage = parse_fixed(&record.variation.coverage_ratio);
    assert!(
        coverage < 1_000_000,
        "expected coverage < 1.0, got {}",
        record.variation.coverage_ratio
    );
    // Not proven because coverage fails
    assert_ne!(record.status, CapabilityStatus::Proven);
}

#[test]
fn m4a_regression_contraction() {
    let root = tempfile::tempdir().unwrap();
    setup_root(root.path());

    write_envelopes(
        root.path(),
        &[
            make_envelope("run_001", CAP, SettlementStatus::Accepted),
            make_envelope("run_002", CAP, SettlementStatus::Accepted),
            make_envelope("run_003", CAP, SettlementStatus::Accepted),
            make_envelope("run_004", CAP, SettlementStatus::Rejected),
        ],
    );

    let transport = FixedTransport {
        response: good_reliability(),
    };
    let authority = SweSeedSettlementAuthority::new(transport, "entity_a");

    let decl_path = root.path().join(".sea-forge/settlement/declarations.jsonl");

    // 3 qualifying + 1 rejected-strong (regression)
    let variations = [
        (
            vec![("model_scale", "small"), ("input_type", "text")],
            vec!["disk_full"],
            Some("0.900000"),
        ),
        (
            vec![("model_scale", "large"), ("input_type", "image")],
            vec![],
            Some("0.800000"),
        ),
        (
            vec![("model_scale", "medium"), ("input_type", "audio")],
            vec![],
            Some("0.700000"),
        ),
    ];

    for (i, (var, disruptions, burden)) in variations.iter().enumerate() {
        let req = make_decl_req(
            &format!("run_{:03}", i + 1),
            CAP,
            "entity_a",
            "entity_b",
            SettlementStrength::Strong,
            "2026-07-13T00:00:00Z",
            "2026-07-14T00:00:00Z",
            var.clone(),
            disruptions.clone(),
            *burden,
        );
        let decl = authority.declare(&req).unwrap();
        append_declaration(&decl_path, &decl).unwrap();
    }

    // Add a rejected-strong declaration (regression)
    // To create a rejected-strong declaration, we need integrity to fail but strength=strong
    let req_regress = make_decl_req(
        "run_004",
        CAP,
        "entity_a",
        "entity_b",
        SettlementStrength::Strong,
        "2026-07-13T00:00:00Z",
        "2026-07-14T00:00:00Z",
        vec![],
        vec![],
        None,
    );
    // Manually create a rejected-strong declaration
    let mut regress_decl = authority.declare(&req_regress).unwrap();
    regress_decl.status = DeclarationStatus::Rejected;
    regress_decl.qualifies_for_capability = false;
    // Recompute hash after mutation
    regress_decl.declaration_hash =
        sea_forge_settlement::compute_declaration_hash(&regress_decl).unwrap();
    append_declaration(&decl_path, &regress_decl).unwrap();

    let policy = default_v02_policy();
    save_policy(root.path(), &policy).unwrap();

    let record = rebuild_capability(root.path(), CAP, &policy).unwrap();

    // Regression weight > 0 → contraction
    assert!(parse_fixed(&record.qualifying.regression_weight) > 0);
    assert!(record
        .contraction_reasons
        .contains(&"regression".to_string()));
}

#[test]
fn m4a_rebuild_byte_identical_modulo_rebuilt_at() {
    let root = tempfile::tempdir().unwrap();
    setup_root(root.path());

    write_envelopes(
        root.path(),
        &[
            make_envelope("run_001", CAP, SettlementStatus::Accepted),
            make_envelope("run_002", CAP, SettlementStatus::Accepted),
            make_envelope("run_003", CAP, SettlementStatus::Accepted),
        ],
    );

    let transport = FixedTransport {
        response: good_reliability(),
    };
    let authority = SweSeedSettlementAuthority::new(transport, "entity_a");

    let decl_path = root.path().join(".sea-forge/settlement/declarations.jsonl");

    let variations = [
        (
            vec![("model_scale", "small"), ("input_type", "text")],
            vec!["disk_full"],
            Some("0.900000"),
        ),
        (
            vec![("model_scale", "large"), ("input_type", "image")],
            vec![],
            Some("0.800000"),
        ),
        (
            vec![("model_scale", "medium"), ("input_type", "audio")],
            vec![],
            Some("0.700000"),
        ),
    ];

    for (i, (var, disruptions, burden)) in variations.iter().enumerate() {
        let req = make_decl_req(
            &format!("run_{:03}", i + 1),
            CAP,
            "entity_a",
            "entity_b",
            SettlementStrength::Strong,
            "2026-07-13T00:00:00Z",
            "2026-07-14T00:00:00Z",
            var.clone(),
            disruptions.clone(),
            *burden,
        );
        let decl = authority.declare(&req).unwrap();
        append_declaration(&decl_path, &decl).unwrap();
    }

    let policy = default_v02_policy();
    save_policy(root.path(), &policy).unwrap();

    let envelopes =
        sea_forge_capability::load_envelopes(&root.path().join(".sea-forge/capabilities.jsonl"))
            .unwrap();
    let declarations = sea_forge_capability::load_declarations(
        &root.path().join(".sea-forge/settlement/declarations.jsonl"),
    )
    .unwrap();

    // Build twice with same rebuilt_at → byte-identical
    let r1 = build_capability_record(
        CAP,
        &envelopes,
        &declarations,
        &policy,
        "2026-07-14T12:00:00Z",
    );
    let r2 = build_capability_record(
        CAP,
        &envelopes,
        &declarations,
        &policy,
        "2026-07-14T12:00:00Z",
    );

    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(
        j1, j2,
        "rebuild must be byte-identical with same inputs and rebuilt_at"
    );

    // Different rebuilt_at → only that field differs
    let r3 = build_capability_record(
        CAP,
        &envelopes,
        &declarations,
        &policy,
        "2026-07-15T00:00:00Z",
    );
    assert_ne!(r1.rebuilt_at, r3.rebuilt_at);
    assert_eq!(r1.status, r3.status);
    assert_eq!(r1.qualifying, r3.qualifying);
}

#[test]
fn m4a_require_proven_denies_not_proven() {
    let root = tempfile::tempdir().unwrap();
    setup_root(root.path());

    // Only 1 run, no qualifying declarations → not proven
    write_envelopes(
        root.path(),
        &[make_envelope("run_001", CAP, SettlementStatus::Accepted)],
    );

    let policy = default_v02_policy();
    save_policy(root.path(), &policy).unwrap();

    let result = require_proven(root.path(), CAP, &policy.name, &policy.policy_sha256);
    assert!(
        result.is_err(),
        "require_proven should deny when not proven"
    );

    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("not proven"),
        "error should cite 'not proven', got: {err}"
    );
    assert!(
        err.contains(&policy.policy_sha256),
        "error should cite policy hash, got: {err}"
    );
}

#[test]
fn m4a_require_proven_allows_when_proven() {
    let root = tempfile::tempdir().unwrap();
    setup_root(root.path());

    write_envelopes(
        root.path(),
        &[
            make_envelope("run_001", CAP, SettlementStatus::Accepted),
            make_envelope("run_002", CAP, SettlementStatus::Accepted),
            make_envelope("run_003", CAP, SettlementStatus::Accepted),
        ],
    );

    let transport = FixedTransport {
        response: good_reliability(),
    };
    let authority = SweSeedSettlementAuthority::new(transport, "entity_a");

    let decl_path = root.path().join(".sea-forge/settlement/declarations.jsonl");

    let variations = [
        (
            vec![("model_scale", "small"), ("input_type", "text")],
            vec!["disk_full"],
            Some("0.900000"),
        ),
        (
            vec![("model_scale", "large"), ("input_type", "image")],
            vec![],
            Some("0.800000"),
        ),
        (
            vec![("model_scale", "medium"), ("input_type", "audio")],
            vec![],
            Some("0.700000"),
        ),
    ];

    for (i, (var, disruptions, burden)) in variations.iter().enumerate() {
        let req = make_decl_req(
            &format!("run_{:03}", i + 1),
            CAP,
            "entity_a",
            "entity_b",
            SettlementStrength::Strong,
            "2026-07-13T00:00:00Z",
            "2026-07-14T00:00:00Z",
            var.clone(),
            disruptions.clone(),
            *burden,
        );
        let decl = authority.declare(&req).unwrap();
        append_declaration(&decl_path, &decl).unwrap();
    }

    let policy = default_v02_policy();
    save_policy(root.path(), &policy).unwrap();

    let result = require_proven(root.path(), CAP, &policy.name, &policy.policy_sha256);
    assert!(result.is_ok(), "require_proven should allow when proven");
    let record = result.unwrap();
    assert_eq!(record.status, CapabilityStatus::Proven);
}

#[test]
fn m4a_policy_change_contraction() {
    let root = tempfile::tempdir().unwrap();
    setup_root(root.path());

    write_envelopes(
        root.path(),
        &[
            make_envelope("run_001", CAP, SettlementStatus::Accepted),
            make_envelope("run_002", CAP, SettlementStatus::Accepted),
            make_envelope("run_003", CAP, SettlementStatus::Accepted),
        ],
    );

    let transport = FixedTransport {
        response: good_reliability(),
    };
    let authority = SweSeedSettlementAuthority::new(transport, "entity_a");

    let decl_path = root.path().join(".sea-forge/settlement/declarations.jsonl");

    let variations = [
        (
            vec![("model_scale", "small"), ("input_type", "text")],
            vec!["disk_full"],
            Some("0.900000"),
        ),
        (
            vec![("model_scale", "large"), ("input_type", "image")],
            vec![],
            Some("0.800000"),
        ),
        (
            vec![("model_scale", "medium"), ("input_type", "audio")],
            vec![],
            Some("0.700000"),
        ),
    ];

    for (i, (var, disruptions, burden)) in variations.iter().enumerate() {
        let req = make_decl_req(
            &format!("run_{:03}", i + 1),
            CAP,
            "entity_a",
            "entity_b",
            SettlementStrength::Strong,
            "2026-07-13T00:00:00Z",
            "2026-07-14T00:00:00Z",
            var.clone(),
            disruptions.clone(),
            *burden,
        );
        let decl = authority.declare(&req).unwrap();
        append_declaration(&decl_path, &decl).unwrap();
    }

    // First policy: proven
    let policy1 = default_v02_policy();
    save_policy(root.path(), &policy1).unwrap();
    let record1 = rebuild_capability(root.path(), CAP, &policy1).unwrap();
    assert_eq!(record1.status, CapabilityStatus::Proven);

    // Second policy: raise min_declarations to 5 → no longer proven
    let mut policy2 = default_v02_policy();
    policy2.min_declarations = 5;
    policy2.name = "stricter_v02".into();
    policy2.policy_sha256 = sea_forge_capability::compute_policy_hash(&policy2).unwrap();
    save_policy(root.path(), &policy2).unwrap();

    let record2 = rebuild_capability(root.path(), CAP, &policy2).unwrap();
    assert_ne!(record2.status, CapabilityStatus::Proven);
}

// ── Task 13B: Thoth-authored claim SoD at the capability-promotion boundary ──
//
// These declarations are constructed directly (bypassing `declare()`
// entirely) to prove the promotion boundary re-checks authorship
// independently — a copied or replayed declaration record that never went
// through settlement's own accept-time check still cannot qualify.

fn qualifying_declaration(declarer_id: &str, authored_by: Option<&str>) -> SettlementDeclaration {
    SettlementDeclaration {
        version: "0.2".into(),
        declaration_id: "decl_thoth_sod".into(),
        settlement_ref: "set_thoth_sod".into(),
        run_id: "run_thoth_sod".into(),
        case_id: "case_thoth_sod".into(),
        plan_item_id: CAP.into(),
        claim_manifest_sha256: "sha256:claim-manifest".into(),
        status: DeclarationStatus::Accepted,
        strength: SettlementStrength::Strong,
        qualifies_for_capability: true,
        criteria_ref: "crit_thoth_sod".into(),
        criteria_sha256: "sha256:criteria".into(),
        criteria_record_hash: "sha256:criteria-record".into(),
        criteria_declared_at: "2026-07-13T00:00:00Z".into(),
        job_contract_ref: None,
        origin_refs: vec![OriginRef {
            kind: OriginRefKind::Intent,
            reference: "int_thoth_sod".into(),
            sha256: "sha256:origin".into(),
            role: OriginRole::DesiredResult,
            evidence_refs: vec![],
            domain_model_ref: None,
        }],
        verifier_ref: "swe_seed@1".into(),
        verifier_sha256: "sha256:verifier".into(),
        verification_evidence_refs: vec!["evi_gate".into()],
        declarer: Declarer {
            actor_id: declarer_id.into(),
            authority_ref: "swe_seed".into(),
            role: "R-AA".into(),
            standing_basis: "external_verification".into(),
        },
        independence: DeclarationIndependence {
            acting_entity_id: "entity_acting".into(),
            independent: true,
            basis: "external_declarer_differs_from_actor".into(),
        },
        reliability: DeclarationReliability {
            feedback_delay_ms: 0,
            attribution_confidence: "1.000000".into(),
            gaming_exposure: "0.000000".into(),
            hidden_debt_blindness: "0.000000".into(),
            weight: "1.000000".into(),
            basis: "swe_seed_adapter".into(),
        },
        variation_tags: BTreeMap::new(),
        disruption_tags: vec![],
        orchestration_burden: None,
        issued_at: "2026-07-14T00:00:00Z".into(),
        source_evidence_refs: vec!["evi_gate".into()],
        adapter_attestation_ref: Some("attestation:swe-seed:1".into()),
        authored_by: authored_by.map(String::from),
        declaration_hash: String::new(),
    }
}

#[test]
fn thoth_sod_same_author_cannot_qualify_for_promotion() {
    let policy = default_v02_policy();
    let decl = qualifying_declaration("thoth", Some("thoth"));
    assert!(!declaration_qualifies(&decl, &policy));
}

#[test]
fn thoth_sod_different_author_can_qualify_for_promotion() {
    let policy = default_v02_policy();
    let decl = qualifying_declaration("operator", Some("thoth"));
    assert!(declaration_qualifies(&decl, &policy));
}

#[test]
fn thoth_sod_copied_declaration_cannot_bypass_promotion() {
    // A declaration "copied" straight into the promotion pipeline (never
    // through `declare()`) still fails the re-check.
    let policy = default_v02_policy();
    let original = qualifying_declaration("thoth", Some("thoth"));
    let copied = original.clone();
    assert!(!declaration_qualifies(&original, &policy));
    assert!(!declaration_qualifies(&copied, &policy));
}

#[test]
fn thoth_sod_relabeled_declaration_cannot_bypass_promotion() {
    // Relabeling the declarer's role/standing_basis doesn't change actor_id.
    let policy = default_v02_policy();
    let mut decl = qualifying_declaration("thoth", Some("thoth"));
    decl.declarer.role = "R-relabeled".into();
    decl.declarer.standing_basis = "relabeled_basis".into();
    assert!(!declaration_qualifies(&decl, &policy));
}

#[test]
fn thoth_sod_replayed_declaration_cannot_bypass_promotion() {
    // Re-evaluating the identical record repeatedly still denies.
    let policy = default_v02_policy();
    let decl = qualifying_declaration("thoth", Some("thoth"));
    for _ in 0..2 {
        assert!(!declaration_qualifies(&decl, &policy));
    }
}
