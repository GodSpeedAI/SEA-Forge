//! M9 (E11) ledger compatibility boundary for ProjectionKind::{Kg, SelfModelSnapshot}.
//!
//! Proves the structural isolation the spec relies on: self-model projections
//! are persisted under a DISTINCT ledger `record_kind` (`self_model_projection`).
//! Then `LedgerStream::verify()` (the replay path) stays green on a stream that
//! contains new-variant records (it hashes payloads as opaque JSON and never
//! deserializes `ProjectionKind`), and a pre-M9 reader that filters by its own
//! known `record_kind` values SKIPS the new record entirely (clean skip, no
//! deserialization attempt, no mid-replay failure), while an M9 reader that
//! filters by `self_model_projection` deserializes it. Both new variants are
//! exercised.

use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::{
    ProjectionKind, ProjectionRecord, ProjectionStatus, ProjectionValidation, StageFile,
};
use sea_forge_ledger::types::LedgerStream;
use serde::Deserialize;
use tempfile::tempdir;

/// Frozen pre-M9 reader: it only loads the record_kind values it knows about.
const LEGACY_PROJECTION_KIND: &str = "projection";
const SELF_MODEL_PROJECTION_KIND: &str = "self_model_projection";

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct LegacyProjectionRecord {
    projection_id: String,
    projection_kind: serde_json::Value, // legacy reader treats kind as opaque
}

fn projection_record(kind: ProjectionKind) -> ProjectionRecord {
    ProjectionRecord {
        projection_id: "proj_smsnap_01".into(),
        projection_kind: kind,
        adapter_ref: "sea-forge-self-model".into(),
        case_id: "case_01".into(),
        run_id: "run_01".into(),
        domain_model_ref: None,
        source_refs: vec!["models/seaforge-system@0.1.0.sea".into()],
        input_hash: "sha256:input".into(),
        output_refs: vec![StageFile {
            path: "self-model/kg/model.ttl".into(),
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

/// A pre-M9 typed loader: it scans the ledger for the record_kind it understands
/// (`projection`) and deserializes only those. This mirrors how every existing
/// consumer in the workspace reads typed records (filter on record_kind first).
fn legacy_load_projection_records(
    stream: &LedgerStream,
) -> Result<Vec<LegacyProjectionRecord>, ForgeError> {
    let mut out = Vec::new();
    for entry in stream.read_entries()? {
        if entry.record_kind == LEGACY_PROJECTION_KIND {
            out.push(serde_json::from_value::<LegacyProjectionRecord>(
                entry.payload,
            )?);
        }
        // Any other record_kind — including self_model_projection — is skipped
        // without ever attempting ProjectionKind deserialization.
    }
    Ok(out)
}

fn new_load_self_model_projections(
    stream: &LedgerStream,
) -> Result<Vec<ProjectionRecord>, ForgeError> {
    let mut out = Vec::new();
    for entry in stream.read_entries()? {
        if entry.record_kind == SELF_MODEL_PROJECTION_KIND {
            out.push(serde_json::from_value::<ProjectionRecord>(entry.payload)?);
        }
    }
    Ok(out)
}

#[test]
fn ledger_verify_is_immune_to_new_projection_variants() {
    for kind in [ProjectionKind::Kg, ProjectionKind::SelfModelSnapshot] {
        let dir = tempdir().unwrap();
        let stream = LedgerStream::open(dir.path(), "case_test", "operator_local").unwrap();

        // A realistic mixed stream: pre-M9 records alongside the new kind.
        stream
            .append(
                "authority_decision",
                vec!["run_01".into()],
                serde_json::json!({"decision":"allow"}),
                vec![],
            )
            .unwrap();
        let record = projection_record(kind);
        stream
            .append(
                SELF_MODEL_PROJECTION_KIND,
                vec!["smsnap_01".into()],
                serde_json::to_value(record).unwrap(),
                vec![],
            )
            .unwrap();
        stream
            .append(
                "settlement_event",
                vec!["run_01".into()],
                serde_json::json!({"status":"accepted"}),
                vec![],
            )
            .unwrap();

        // Replay/verify hashes payloads as opaque JSON; the new variant never
        // touches ProjectionKind deserialization here.
        stream.verify().expect("ledger verify must be green");
    }
}

#[test]
fn legacy_reader_skips_new_record_kind_without_deserializing() {
    for kind in [ProjectionKind::Kg, ProjectionKind::SelfModelSnapshot] {
        let dir = tempdir().unwrap();
        let stream = LedgerStream::open(dir.path(), "case_test", "operator_local").unwrap();

        // One legacy projection + one new self-model projection.
        stream
            .append(
                LEGACY_PROJECTION_KIND,
                vec!["proj_old".into()],
                serde_json::json!({"projection_id":"proj_old","projection_kind":"calm"}),
                vec![],
            )
            .unwrap();
        stream
            .append(
                SELF_MODEL_PROJECTION_KIND,
                vec!["proj_new".into()],
                serde_json::to_value(projection_record(kind)).unwrap(),
                vec![],
            )
            .unwrap();

        // Legacy reader finds exactly its own record; the new one is skipped,
        // so it never attempts to deserialize the Kg/SelfModelSnapshot variant.
        let legacy = legacy_load_projection_records(&stream).unwrap();
        assert_eq!(
            legacy.len(),
            1,
            "legacy reader must skip self_model_projection"
        );
        assert_eq!(legacy[0].projection_id, "proj_old");

        // M9 reader finds the new record and deserializes the new variant fine.
        let new_records = new_load_self_model_projections(&stream).unwrap();
        assert_eq!(new_records.len(), 1);
        assert!(matches!(
            new_records[0].projection_kind,
            ProjectionKind::Kg | ProjectionKind::SelfModelSnapshot
        ));

        // And the full stream still verifies.
        stream.verify().unwrap();
    }
}

#[test]
fn legacy_reader_never_panics_when_new_kind_present() {
    // Hostile-shaped payload under the new kind must not affect a legacy reader
    // that filters by record_kind — it simply never reads the payload.
    let dir = tempdir().unwrap();
    let stream = LedgerStream::open(dir.path(), "case_test", "operator_local").unwrap();
    stream
        .append(
            SELF_MODEL_PROJECTION_KIND,
            vec!["x".into()],
            serde_json::json!({"projection_id":"x","projection_kind":"self_model_snapshot","totally":"alien","structure":[1,2,3]}),
            vec![],
        )
        .unwrap();

    let legacy = legacy_load_projection_records(&stream).unwrap();
    assert!(legacy.is_empty(), "legacy reader ignores the alien record");
    stream.verify().unwrap();
}
