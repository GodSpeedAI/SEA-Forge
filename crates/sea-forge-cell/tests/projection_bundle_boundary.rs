//! M9 (E11) E6 federation boundary for ProjectionKind::{Kg, SelfModelSnapshot}.
//!
//! Proves that a self-model projection file carrying a new ProjectionKind
//! variant round-trips through SeaCell bundle export/import without issue:
//! import validates every file by sha256+size and NEVER deserializes
//! `ProjectionRecord`, so the new enum variants cannot affect federation.

use sea_forge_cell as cell;
use sea_forge_core::types::{
    ProjectionKind, ProjectionRecord, ProjectionStatus, ProjectionValidation, StageFile,
};
use sha2::{Digest, Sha256};
use std::fs;

const RUN_FILES: &[&str] = &[
    "plan.json",
    "trace.jsonl",
    "evidence.jsonl",
    "authority.json",
    "settlement.json",
    "semantic-envelope.json",
];

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

/// Export a run whose artifacts include self-model projections with the new
/// ProjectionKind variants, then import on a fresh root and confirm the files
/// round-trip byte-identical (content-addressed), proving import never needs to
/// understand the new enum variants.
#[test]
fn self_model_projections_round_trip_through_federation() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    let run_id = "run_20260716T000000Z_aaaaaa";
    let run_dir = src.path().join(".sea-forge/runs").join(run_id);
    fs::create_dir_all(run_dir.join("artifacts/self-model")).unwrap();
    for name in RUN_FILES {
        fs::write(run_dir.join(name), b"{}\n").unwrap();
    }

    let kg_bytes = serde_json::to_vec_pretty(&projection_record(ProjectionKind::Kg)).unwrap();
    let snap_bytes =
        serde_json::to_vec_pretty(&projection_record(ProjectionKind::SelfModelSnapshot)).unwrap();
    fs::write(run_dir.join("artifacts/self-model/kg.json"), &kg_bytes).unwrap();
    fs::write(
        run_dir.join("artifacts/self-model/snapshot.json"),
        &snap_bytes,
    )
    .unwrap();

    let bundle = src.path().join(".sea-forge/export/boundary.tar");
    let manifest = cell::export(src.path(), &[run_id.to_string()], &[], &bundle).unwrap();

    // The two self-model projection files are bundled.
    assert!(manifest
        .files
        .iter()
        .any(|f| f.path.ends_with("self-model/kg.json")));
    assert!(manifest
        .files
        .iter()
        .any(|f| f.path.ends_with("self-model/snapshot.json")));

    // Import on a fresh root succeeds — files are verified by hash, not by
    // deserializing ProjectionKind, so the new variants are irrelevant here.
    let imported = cell::import(dst.path(), &bundle).unwrap();
    let imported_root = dst
        .path()
        .join(".sea-forge/imported")
        .join(&imported.cell_id);

    let round_tripped_kg = fs::read(
        imported_root
            .join("runs")
            .join(run_id)
            .join("artifacts/self-model/kg.json"),
    )
    .unwrap();
    let round_tripped_snap = fs::read(
        imported_root
            .join("runs")
            .join(run_id)
            .join("artifacts/self-model/snapshot.json"),
    )
    .unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&round_tripped_kg)),
        sha256_hex(&kg_bytes)
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(&round_tripped_snap)),
        sha256_hex(&snap_bytes)
    );

    // And the new-variant records still deserialize correctly post-import for
    // an M9-aware consumer (the import itself did not need to).
    let r: ProjectionRecord = serde_json::from_slice(&round_tripped_kg).unwrap();
    assert_eq!(r.projection_kind, ProjectionKind::Kg);
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn federation_proof_is_by_hash_not_enum_deserialization() {
    // Defensive: even a deliberately alien projection file imports fine as long
    // as its hash matches, because import treats bundle entries as opaque bytes.
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    let run_id = "run_20260716T000001Z_bbbbbb";
    let run_dir = src.path().join(".sea-forge/runs").join(run_id);
    fs::create_dir_all(run_dir.join("artifacts")).unwrap();
    for name in RUN_FILES {
        fs::write(run_dir.join(name), b"{}\n").unwrap();
    }
    // Alien shape with the new variant string — import does not parse it.
    fs::write(
        run_dir.join("artifacts/alien.json"),
        b"{\"projection_kind\":\"self_model_snapshot\",\"future\":\"field\"}",
    )
    .unwrap();

    let bundle = src.path().join(".sea-forge/export/alien.tar");
    cell::export(src.path(), &[run_id.to_string()], &[], &bundle).unwrap();
    cell::import(dst.path(), &bundle).expect("import is hash-based, enum-agnostic");
}
