//! A settlement declaration is rejected as post-hoc when its criteria were declared AFTER execution began.
//! That is a comparison of instants, not text, and a stamp that does not parse cannot show the criteria came
//! first, so it is rejected (fail closed).
use sea_forge_core::types::*;
use sea_forge_settlement::{LocalSettlementAuthority, SettlementAuthority};

fn request(declared: &str, started: &str) -> SettlementDeclarationRequest {
    SettlementDeclarationRequest {
        settlement_ref: "set_x".into(),
        run_id: "run_x".into(),
        case_id: "case_x".into(),
        plan_item_id: "item_x".into(),
        claim_manifest_sha256: "sha256:claim".into(),
        criteria_ref: "crit_x".into(),
        criteria_sha256: "sha256:crit".into(),
        criteria_record_hash: "sha256:crit-record".into(),
        criteria_declared_at: declared.into(),
        execution_started_at: started.into(),
        job_contract_ref: None,
        origin_refs: vec![OriginRef {
            kind: OriginRefKind::Intent,
            reference: "int_x".into(),
            sha256: "sha256:intent".into(),
            role: OriginRole::DesiredResult,
            evidence_refs: vec![],
            domain_model_ref: None,
        }],
        verifier_ref: "harness".into(),
        verifier_sha256: "sha256:verifier".into(),
        acting_entity_id: "operator_local".into(),
        requested_strength: SettlementStrength::Local,
        declarer: Declarer {
            actor_id: "independent_declarer".into(),
            authority_ref: "local".into(),
            role: "R-AA".into(),
            standing_basis: "test".into(),
        },
        variation_tags: Default::default(),
        disruption_tags: vec![],
        orchestration_burden: None,
        source_evidence_refs: vec!["evi_x".into()],
        authored_by: None,
        authorization_ref: None,
        authorization_request_sha256: None,
        authorization_content_sha256: None,
    }
}

fn status(declared: &str, started: &str) -> DeclarationStatus {
    LocalSettlementAuthority::new("operator_local")
        .declare(&request(declared, started))
        .unwrap()
        .status
}

#[test]
fn criteria_declared_before_or_at_execution_are_accepted_in_any_spelling() {
    for (declared, started) in [
        ("2026-01-01T00:00:00Z", "2026-01-01T00:00:01Z"),
        ("2026-01-01T00:00:00Z", "2026-01-01T00:00:00+00:00"),
        ("2026-01-01T05:30:00+05:30", "2026-01-01T00:00:00Z"),
        // Earlier by an instant although the text sorts later.
        ("2026-01-01T00:00:00Z", "2025-12-31T20:00:00-05:00"),
        (
            "2026-01-01T00:00:00.123Z",
            "2026-01-01T00:00:00.123456+00:00",
        ),
    ] {
        assert_eq!(
            status(declared, started),
            DeclarationStatus::Accepted,
            "{declared} vs {started}"
        );
    }
}

#[test]
fn criteria_declared_after_execution_are_rejected_in_any_spelling() {
    for (declared, started) in [
        ("2026-01-01T00:00:02Z", "2026-01-01T00:00:01Z"),
        (
            "2026-01-01T00:00:00.123457Z",
            "2026-01-01T00:00:00.123456+00:00",
        ),
        // Later by an instant although the text sorts earlier.
        ("2025-12-31T20:00:01-05:00", "2026-01-01T01:00:00Z"),
        ("2026-01-01T05:30:01+05:30", "2026-01-01T00:00:00Z"),
    ] {
        assert_eq!(
            status(declared, started),
            DeclarationStatus::Rejected,
            "{declared} vs {started}"
        );
    }
}

#[test]
fn an_unparsable_stamp_is_rejected() {
    for (declared, started) in [
        ("not a time", "2026-01-01T00:00:01Z"),
        ("2026-01-01T00:00:00Z", "not a time"),
        ("", "2026-01-01T00:00:01Z"),
        ("2026-01-01T00:00:00", "2026-01-01T00:00:01Z"),
    ] {
        assert_eq!(
            status(declared, started),
            DeclarationStatus::Rejected,
            "{declared:?} vs {started:?}"
        );
    }
}
