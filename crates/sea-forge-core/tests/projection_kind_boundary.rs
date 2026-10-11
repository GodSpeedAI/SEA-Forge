//! M9 (E11) ProjectionKind compatibility boundary.
//!
//! Proves that adding `ProjectionKind::{Kg, SelfModelSnapshot}` is safe at the
//! enum layer: an old reader whose enum lacks the new variants either (a) still
//! reads every legacy record unchanged, or (b) receives a clean serde error
//! (no panic) on a record carrying a new variant. The deeper structural guard —
//! that old readers never reach this deserialization because the new records
//! live under a distinct ledger `record_kind` — is proven in
//! `sea-forge-ledger/tests/projection_boundary.rs` and exercised for E6 import
//! in `sea-forge-cell`.

use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::{
    ProjectionKind, ProjectionRecord, ProjectionStatus, ProjectionValidation, StageFile,
};
use serde::Deserialize;
use serde_json::json;

/// A frozen pre-M9 reader enum: every variant that existed before Kg/SelfModelSnapshot.
/// This models a binary built from the b351c95 baseline reading a record written by M9.
#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
enum LegacyProjectionKind {
    Sea,
    Calm,
    Rdf,
    Sbvr,
    Shacl,
    Manifest,
    KgEvent,
    CapabilityRecord,
    MemoryIndex,
    CapitalRecord,
    ImplementationDefined,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct LegacyProjectionRecord {
    projection_id: String,
    projection_kind: LegacyProjectionKind,
}

fn record_for(kind: &ProjectionKind) -> ProjectionRecord {
    ProjectionRecord {
        projection_id: "proj_01".into(),
        projection_kind: kind.clone(),
        adapter_ref: "sea-forge-domainforge".into(),
        case_id: "case_01".into(),
        run_id: "run_01".into(),
        domain_model_ref: None,
        source_refs: vec!["models/seaforge-system@0.1.0.sea".into()],
        input_hash: "sha256:input".into(),
        output_refs: vec![StageFile {
            path: "kg/model.ttl".into(),
            sha256: "sha256:out".into(),
            generated: true,
            ..Default::default()
        }],
        quarantine_refs: vec![],
        validation: ProjectionValidation {
            status: ProjectionStatus::Accepted,
            validator_ref: "domainforge-core".into(),
            basis: vec![],
        },
        authority_refs: vec![],
        evidence_refs: vec![],
        settlement_ref: None,
        created_at: "2026-07-16T00:00:00Z".into(),
        rebuild_hash: "sha256:rebuild".into(),
    }
}

#[test]
fn new_variants_round_trip_in_current_reader() {
    for kind in [ProjectionKind::Kg, ProjectionKind::SelfModelSnapshot] {
        let value = serde_json::to_value(record_for(&kind)).unwrap();
        // Wire names match spec-adlc-thoth §3.1.
        let wire = value
            .get("projection_kind")
            .and_then(|v| v.as_str())
            .unwrap();
        let expected = match kind {
            ProjectionKind::Kg => "kg",
            ProjectionKind::SelfModelSnapshot => "self_model_snapshot",
            _ => unreachable!(),
        };
        assert_eq!(wire, expected);
        let back: ProjectionRecord = serde_json::from_value(value).unwrap();
        assert_eq!(back.projection_kind, kind);
    }
}

#[test]
fn legacy_records_still_deserialize_in_old_reader() {
    // No regression: pre-M9 variants deserialize in both old and new readers.
    for kind in [ProjectionKind::Calm, ProjectionKind::Rdf] {
        let value = serde_json::to_value(record_for(&kind)).unwrap();
        serde_json::from_value::<LegacyProjectionRecord>(value).unwrap();
    }
}

#[test]
fn old_reader_rejects_new_variants_cleanly_without_panic() {
    // If an old reader ever attempted enum deserialization on a new-variant
    // record, serde returns a normal Err — never a panic or mid-replay crash.
    // The structural record_kind boundary (see ledger test) ensures this path
    // is never reached in practice; this asserts the fallback is still safe.
    for kind in [ProjectionKind::Kg, ProjectionKind::SelfModelSnapshot] {
        let value = serde_json::to_value(record_for(&kind)).unwrap();
        let result = serde_json::from_value::<LegacyProjectionRecord>(value);
        assert!(
            result.is_err(),
            "{kind:?} should not deserialize for a legacy reader"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.to_lowercase().contains("variant")
                || err.to_lowercase().contains("kg")
                || err.to_lowercase().contains("self_model_snapshot"),
            "error should name the unknown variant: {err}"
        );
    }
}

#[test]
fn unknown_projection_kind_string_is_rejected_not_silently_accepted() {
    // A typo'd or malicious kind string is a clean error, never an unknown default.
    let bad = json!({"projection_id":"p","projection_kind":"not_a_real_kind"});
    assert!(serde_json::from_value::<ProjectionRecord>(bad).is_err());
}

#[test]
fn self_model_error_class_is_distinct() {
    let err = ForgeError::SelfModel("bundled model hash drift".into());
    assert_eq!(err.class(), "self_model_error");
    assert!(err.to_string().contains("bundled model hash drift"));
    // Blast-radius isolation is enforced by call sites (self-model + ask only);
    // the class string is the machine-readable diagnostic.
}

#[test]
fn snapshot_id_uses_smsnap_prefix() {
    let id = sea_forge_core::ids::snapshot_id().unwrap();
    assert!(
        id.starts_with("smsnap_") && id.len() > "smsnap_".len(),
        "snapshot id must use the smsnap_ prefix: {id}"
    );
}
