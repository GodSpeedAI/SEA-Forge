use sea_forge_core::{types::*, RECORD_VERSION};
use sea_forge_settlement::{settle, LocalSettlementAuthority, SettlementAuthority};
use std::{collections::BTreeMap, path::Path};

fn legacy_claim() -> SettlementClaim {
    SettlementClaim {
        run_id: "run".into(),
        plan_item_id: "item_01".into(),
        criteria_ref: None,
        criteria: SettlementCriteria {
            require_exit_zero: true,
            ..Default::default()
        },
        execution: None,
        authority_verdicts: vec![Verdict::Allow],
        evaluator_scores: BTreeMap::new(),
        batch: None,
    }
}

fn modern_claim() -> SettlementClaim {
    SettlementClaim {
        run_id: "run".into(),
        plan_item_id: "item_01".into(),
        criteria_ref: Some("crit_abcdef".into()),
        criteria: SettlementCriteria {
            require_exit_zero: true,
            ..Default::default()
        },
        execution: None,
        authority_verdicts: vec![Verdict::Allow],
        evaluator_scores: BTreeMap::new(),
        batch: None,
    }
}

#[test]
fn legacy_claim_evaluates_and_marks_unattributed() {
    let event = settle(&legacy_claim(), Path::new("."), Path::new(".")).unwrap();
    assert_eq!(event.status, SettlementStatus::Rejected);
    assert!(event.basis.contains(&"legacy_unattributed_criteria".into()));
    assert!(event.criteria_ref.is_none());
}

#[test]
fn modern_claim_records_criteria_ref() {
    let event = settle(&modern_claim(), Path::new("."), Path::new(".")).unwrap();
    assert_eq!(event.criteria_ref, Some("crit_abcdef".into()));
    assert!(!event.basis.contains(&"legacy_unattributed_criteria".into()));
}

#[test]
fn legacy_unattributed_cannot_qualify_strong() {
    // Settlement qualification in Task 11 must reject any event whose basis
    // contains legacy_unattributed_criteria. This test asserts the marker is
    // emitted before that gate exists.
    let event = settle(&legacy_claim(), Path::new("."), Path::new(".")).unwrap();
    assert!(event.basis.contains(&"legacy_unattributed_criteria".into()));
}

#[test]
fn settlement_event_round_trips_criteria_ref() {
    let event = settle(&modern_claim(), Path::new("."), Path::new(".")).unwrap();
    let serialized = serde_json::to_string(&event).unwrap();
    let round_tripped: SettlementEvent = serde_json::from_str(&serialized).unwrap();
    assert_eq!(round_tripped.criteria_ref, Some("crit_abcdef".into()));
    assert_eq!(round_tripped.version, RECORD_VERSION);
}

// ── Task 13B: Thoth-authored claim SoD at the settlement boundary ──

fn declaration_request(
    declarer_id: &str,
    authored_by: Option<&str>,
) -> SettlementDeclarationRequest {
    SettlementDeclarationRequest {
        settlement_ref: "set_thoth_sod".into(),
        run_id: "run_thoth_sod".into(),
        case_id: "case_thoth_sod".into(),
        plan_item_id: "item_thoth_sod".into(),
        claim_manifest_sha256: "sha256:claim-manifest".into(),
        criteria_ref: "crit_thoth_sod".into(),
        criteria_sha256: "sha256:criteria".into(),
        criteria_record_hash: "sha256:criteria-record".into(),
        criteria_declared_at: "2026-07-13T00:00:00Z".into(),
        execution_started_at: "2026-07-14T00:00:00Z".into(),
        job_contract_ref: None,
        origin_refs: vec![OriginRef {
            kind: OriginRefKind::Intent,
            reference: "int_thoth_sod".into(),
            sha256: "sha256:origin".into(),
            role: OriginRole::DesiredResult,
            evidence_refs: vec![],
            domain_model_ref: None,
        }],
        verifier_ref: "builtin:local".into(),
        verifier_sha256: "sha256:verifier".into(),
        acting_entity_id: "entity_acting".into(),
        requested_strength: SettlementStrength::Local,
        declarer: Declarer {
            actor_id: declarer_id.into(),
            authority_ref: "swe_seed".into(),
            role: "R-AA".into(),
            standing_basis: "external_verification".into(),
        },
        variation_tags: BTreeMap::new(),
        disruption_tags: vec![],
        orchestration_burden: None,
        source_evidence_refs: vec!["evd_thoth_sod".into()],
        authored_by: authored_by.map(String::from),
    }
}

#[test]
fn thoth_sod_same_author_denies_declaration() {
    let authority = LocalSettlementAuthority::new("entity_acting");
    let req = declaration_request("thoth", Some("thoth"));
    let decl = authority.declare(&req).unwrap();
    assert_eq!(decl.status, DeclarationStatus::Rejected);
}

#[test]
fn thoth_sod_different_author_allows_declaration() {
    let authority = LocalSettlementAuthority::new("entity_acting");
    let req = declaration_request("operator", Some("thoth"));
    let decl = authority.declare(&req).unwrap();
    assert_eq!(decl.status, DeclarationStatus::Accepted);
    assert_eq!(decl.authored_by.as_deref(), Some("thoth"));
}

#[test]
fn thoth_sod_copied_claim_cannot_bypass() {
    // A "copy" of the same authored claim submitted under the same declarer
    // denies identically — copying carries the same authored_by/declarer.
    let authority = LocalSettlementAuthority::new("entity_acting");
    let original = declaration_request("thoth", Some("thoth"));
    let copied = declaration_request("thoth", Some("thoth"));
    assert_eq!(
        authority.declare(&original).unwrap().status,
        DeclarationStatus::Rejected
    );
    assert_eq!(
        authority.declare(&copied).unwrap().status,
        DeclarationStatus::Rejected
    );
}

#[test]
fn thoth_sod_relabeled_claim_cannot_bypass() {
    // Relabeling the declarer's role/standing_basis doesn't change its
    // actor_id identity — the SoD binding survives the relabel.
    let authority = LocalSettlementAuthority::new("entity_acting");
    let mut req = declaration_request("thoth", Some("thoth"));
    req.declarer.role = "R-relabeled".into();
    req.declarer.standing_basis = "relabeled_basis".into();
    let decl = authority.declare(&req).unwrap();
    assert_eq!(decl.status, DeclarationStatus::Rejected);
}

#[test]
fn thoth_sod_replayed_claim_cannot_bypass() {
    // Replaying the identical request again still denies — the check is
    // deterministic over (authored_by, declarer), not one-shot.
    let authority = LocalSettlementAuthority::new("entity_acting");
    let req = declaration_request("thoth", Some("thoth"));
    for _ in 0..2 {
        assert_eq!(
            authority.declare(&req).unwrap().status,
            DeclarationStatus::Rejected
        );
    }
}
