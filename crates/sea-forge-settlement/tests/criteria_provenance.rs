use sea_forge_core::{types::*, RECORD_VERSION};
use sea_forge_settlement::settle;
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
