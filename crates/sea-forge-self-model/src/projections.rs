//! Self-model projections through the ProjectionRecord ABI (spec-adlc-thoth
//! §3.1, §10.1, slice 1.5). Produces KG/CALM/JSON(self_model_snapshot)
//! ProjectionRecords that are deterministic, rebuildable, and carry full
//! provenance. Two rebuilds from the same sources are byte-identical modulo
//! `created_at`.

use crate::{canonical_sha256, ComposedModel, SelfModelSnapshot};
use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::{
    ProjectionKind, ProjectionRecord, ProjectionStatus, ProjectionValidation, StageFile,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const ADAPTER_REF: &str = "sea-forge-self-model";

/// One self-model projection: its ProjectionRecord plus the materialized output
/// bytes keyed by the same full path used in `record.output_refs`.
#[derive(Clone, Debug)]
pub struct SelfModelProjection {
    pub record: ProjectionRecord,
    pub outputs: BTreeMap<String, String>,
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Deterministic projection id from kind + snapshot hash prefix, so rebuilds of
/// the same snapshot produce byte-identical records (modulo created_at).
fn deterministic_projection_id(kind_label: &str, snapshot_hash: &str) -> String {
    let digest = snapshot_hash
        .strip_prefix("sha256:")
        .unwrap_or(snapshot_hash);
    let prefix = digest.len().min(16);
    format!("proj_{kind_label}_{}", &digest[..prefix])
}

fn build(
    kind_label: &str,
    kind: ProjectionKind,
    raw_outputs: BTreeMap<String, String>,
    source_refs: Vec<String>,
    input_hash: &str,
    snapshot: &SelfModelSnapshot,
    created_at: &str,
) -> Result<SelfModelProjection, ForgeError> {
    // Re-key outputs under self-model/<kind_label>/ and build output_refs.
    let mut outputs: BTreeMap<String, String> = BTreeMap::new();
    let mut output_refs: Vec<StageFile> = Vec::with_capacity(raw_outputs.len());
    for (path, content) in raw_outputs {
        let full = format!("self-model/{kind_label}/{path}");
        let hash = format!("sha256:{}", sha256_hex(content.as_bytes()));
        output_refs.push(StageFile {
            path: full.clone(),
            sha256: hash,
            generated: true,
            ..Default::default()
        });
        outputs.insert(full, content);
    }
    output_refs.sort_by(|a, b| a.path.cmp(&b.path));

    let rebuild_input = serde_json::json!({
        "adapter_ref": ADAPTER_REF,
        "source_refs": source_refs,
        "input_hash": input_hash,
        "output_refs": output_refs,
    });
    let rebuild_hash = canonical_sha256(&rebuild_input)?;

    let record = ProjectionRecord {
        projection_id: deterministic_projection_id(kind_label, &snapshot.snapshot_hash),
        projection_kind: kind,
        adapter_ref: ADAPTER_REF.into(),
        case_id: "self-model".into(),
        run_id: snapshot.snapshot_id.clone(),
        domain_model_ref: None,
        source_refs,
        input_hash: input_hash.into(),
        output_refs,
        quarantine_refs: vec![],
        validation: ProjectionValidation {
            status: ProjectionStatus::Accepted,
            validator_ref: "sea-forge-self-model".into(),
            basis: vec!["deterministic_rebuild".into()],
        },
        authority_refs: vec![],
        evidence_refs: vec![],
        settlement_ref: None,
        created_at: created_at.into(),
        rebuild_hash,
    };
    Ok(SelfModelProjection { record, outputs })
}

/// Build the three self-model projections (KG, CALM, self_model_snapshot).
/// Pure: no filesystem writes. Deterministic for fixed sources + created_at.
pub fn project_self(
    composed: &ComposedModel,
    snapshot: &SelfModelSnapshot,
    created_at: &str,
) -> Result<Vec<SelfModelProjection>, ForgeError> {
    use sea_forge_domainforge::project;
    let source_refs: Vec<String> = composed
        .system_model_ref()
        .source_refs
        .iter()
        .map(|s| s.uri.clone())
        .collect();
    let model_input_hash = composed.system_model_ref().semantic_model_sha256.clone();

    let kg_outputs = project(&composed.system, &ProjectionKind::Kg)?;
    let calm_outputs = project(&composed.system, &ProjectionKind::Calm)?;

    let snap_value = serde_json::to_value(snapshot)?;
    let snap_bytes = serde_json::to_vec_pretty(&snap_value)?;
    let snap_str = String::from_utf8(snap_bytes)
        .map_err(|e| sea_forge_core::ForgeError::Serialization(format!("snapshot utf-8: {e}")))?;
    let mut snap_outputs = BTreeMap::new();
    snap_outputs.insert("snapshot.json".into(), snap_str);

    Ok(vec![
        build(
            "kg",
            ProjectionKind::Kg,
            kg_outputs,
            source_refs.clone(),
            &model_input_hash,
            snapshot,
            created_at,
        )?,
        build(
            "calm",
            ProjectionKind::Calm,
            calm_outputs,
            source_refs.clone(),
            &model_input_hash,
            snapshot,
            created_at,
        )?,
        build(
            "self_model_snapshot",
            ProjectionKind::SelfModelSnapshot,
            snap_outputs,
            source_refs,
            &snapshot.snapshot_hash,
            snapshot,
            created_at,
        )?,
    ])
}

/// Recompute the rebuild_hash of a projection record from its stored fields and
/// confirm it matches. Drift ⇒ `self_model_error` (used to reject a pre-generated
/// projection whose output hash no longer matches its bytes).
pub fn verify_projection(record: &ProjectionRecord) -> Result<(), ForgeError> {
    let rebuild_input = serde_json::json!({
        "adapter_ref": record.adapter_ref,
        "source_refs": record.source_refs,
        "input_hash": record.input_hash,
        "output_refs": record.output_refs,
    });
    let expected = canonical_sha256(&rebuild_input)?;
    if expected != record.rebuild_hash {
        return Err(sea_forge_core::ForgeError::SelfModel(format!(
            "self_model_error: projection rebuild_hash mismatch for {}: declared={} computed={expected}",
            record.projection_id, record.rebuild_hash
        )));
    }
    Ok(())
}
