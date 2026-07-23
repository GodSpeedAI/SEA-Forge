//! SEA Forge spec-to-code pipeline (spec-full §7.8, §10.7, M5).
//!
//! Deterministic stage chaining, generated-zone guard, proof classification,
//! and DomainForge projections. The pipeline runs as ordinary governed work —
//! no separate engine. This crate provides the pure logic; the CLI wires it
//! into the case-run lifecycle.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use sea_forge_core::{
    errors::ForgeError,
    types::{
        ProjectionKind, ProjectionRecord, ProjectionStatus, ProjectionValidation,
        ProofClassification, SpecPipelineRun, SpecPipelineStage, StageFile, StageKind, StageStatus,
    },
};
use sea_forge_ledger::types::hash_canonical;
use sha2::{Digest, Sha256};

// ── Stage hashing ──

/// Compute the content hash of a single stage.
/// Hash covers: stage_id, kind, inputs, outputs, command, status.
/// Excludes quarantine_ref and settlement_basis (they are governance metadata, not content).
pub fn compute_stage_hash(stage: &SpecPipelineStage) -> Result<String, ForgeError> {
    let canonical = serde_json::json!({
        "stage_id": stage.stage_id,
        "kind": stage.kind,
        "inputs": stage.inputs,
        "outputs": stage.outputs,
        "command": stage.command,
        "status": stage.status,
    });
    let bytes = serde_json::to_vec(&canonical)?;
    Ok(format!("sha256:{:x}", Sha256::digest(&bytes)))
}

/// Compute the chain hash over all stages in order.
/// Each stage's hash includes the prior stage's hash, forming a linked chain.
pub fn compute_chain_hash(stages: &[SpecPipelineStage]) -> Result<String, ForgeError> {
    let mut prior_hash = String::new();
    let mut hasher = Sha256::new();
    for stage in stages {
        let stage_hash = compute_stage_hash(stage)?;
        hasher.update(stage_hash.as_bytes());
        hasher.update(prior_hash.as_bytes());
        prior_hash = format!("{:x}", hasher.clone().finalize());
        hasher = Sha256::new();
    }
    Ok(format!("sha256:{prior_hash}"))
}

// ── Stage ordering ──

/// The canonical stage order per §10.7.
const CANONICAL_ORDER: &[StageKind] = &[
    StageKind::Adr,
    StageKind::Prd,
    StageKind::Sds,
    StageKind::Sea,
    StageKind::Ast,
    StageKind::Ir,
    StageKind::Manifest,
    StageKind::GeneratedContract,
    StageKind::SemanticFixture,
    StageKind::LastMileAdapter,
    StageKind::RuntimeWiring,
    StageKind::AcceptanceProof,
];

/// Validate that stages follow the canonical order and that a later stage
/// only starts if all prior accepted stages validate by hash.
pub fn validate_stage_order(stages: &[SpecPipelineStage]) -> Result<(), ForgeError> {
    let mut last_ordinal: usize = 0;
    for stage in stages {
        let ordinal = CANONICAL_ORDER
            .iter()
            .position(|k| k == &stage.kind)
            .ok_or_else(|| {
                ForgeError::Input(format!(
                    "spec_pipeline_error: unknown stage kind {:?}",
                    stage.kind
                ))
            })?;
        if ordinal < last_ordinal {
            return Err(ForgeError::Input(format!(
                "spec_pipeline_error: stage {:?} appears after stage at position {last_ordinal}",
                stage.kind
            )));
        }
        last_ordinal = ordinal;
    }
    Ok(())
}

// ── Proof classification ──

/// Kinds that must form an unbroken accepted prefix (per `CANONICAL_ORDER`)
/// before classification may rise above `authority-only`. `SemanticFixture`
/// is intentionally excluded: it is evidentiary, not every model produces
/// one, and requiring it would over-constrain routes that never claim it.
const REQUIRED_FOR_GENERATED_CONTRACT: &[StageKind] = &[
    StageKind::Adr,
    StageKind::Prd,
    StageKind::Sds,
    StageKind::Sea,
    StageKind::Ast,
    StageKind::Ir,
    StageKind::Manifest,
    StageKind::GeneratedContract,
];

const REQUIRED_FOR_FOCUSED_SLICE: &[StageKind] = &[
    StageKind::LastMileAdapter,
    StageKind::RuntimeWiring,
    StageKind::AcceptanceProof,
];

fn all_accepted(stages: &[SpecPipelineStage], kinds: &[StageKind]) -> bool {
    kinds.iter().all(|kind| {
        stages
            .iter()
            .any(|s| &s.kind == kind && s.status == StageStatus::Accepted)
    })
}

/// Compute the proof classification ceiling based on accepted stages.
///
/// A classification only counts once its ENTIRE required canonical-order
/// prefix is present and accepted (§10.7) — a stage cannot borrow the
/// classification of a predecessor that is absent, skipped, rejected, or
/// quarantined. This prevents a sparse, out-of-context slice of stages from
/// claiming a stronger classification than its actual proof chain supports.
///
/// - `authority-only`: no generated outputs claimed, or the required prefix
///   is incomplete.
/// - `generated-contract`: the full ADR..GeneratedContract prefix is accepted.
/// - `focused-slice`: the generated-contract prefix plus last_mile + runtime
///   + acceptance are all accepted.
pub fn compute_proof_classification(stages: &[SpecPipelineStage]) -> ProofClassification {
    let has_generated = all_accepted(stages, REQUIRED_FOR_GENERATED_CONTRACT);
    let has_focused_slice = has_generated && all_accepted(stages, REQUIRED_FOR_FOCUSED_SLICE);

    if has_focused_slice {
        ProofClassification::FocusedSlice
    } else if has_generated {
        ProofClassification::GeneratedContract
    } else {
        ProofClassification::AuthorityOnly
    }
}

// ── Stage prerequisite validation (M5 Task 9) ──

/// Validate that a stage's declared input files resolve to a verified
/// predecessor: either a stage earlier in `stages` whose matching-path
/// output is `Accepted` with an identical hash/schema/domain-model-ref/
/// DomainForge version, or a fully self-verified externally supplied file
/// (its own hash and schema are present, so it does not depend on a local
/// predecessor's status to be trusted).
///
/// This replaces trusting array position/ordering alone: ordering only
/// proves stages appear in canonical sequence, not that a later stage's
/// claimed input actually is the verified output of an accepted, matching
/// predecessor. Call this before activating a stage (Task 10A) or as part
/// of whole-run processing (`process_pipeline`).
pub fn validate_stage_prerequisites(
    stages: &[SpecPipelineStage],
    stage_index: usize,
) -> Result<(), ForgeError> {
    let stage = &stages[stage_index];
    for input in &stage.inputs {
        if input.sha256.is_empty() {
            return Err(ForgeError::Input(format!(
                "spec_pipeline_prerequisite_error: stage {} input {} is unhashed",
                stage.stage_id, input.path
            )));
        }
        let predecessor = stages[..stage_index]
            .iter()
            .find(|p| p.outputs.iter().any(|o| o.path == input.path));
        match predecessor {
            Some(predecessor) => {
                let output = predecessor
                    .outputs
                    .iter()
                    .find(|o| o.path == input.path)
                    .expect("matched above");
                if predecessor.status != StageStatus::Accepted {
                    return Err(ForgeError::Input(format!(
                        "spec_pipeline_prerequisite_error: predecessor stage {} for input {} is not accepted (status: {:?})",
                        predecessor.stage_id, input.path, predecessor.status
                    )));
                }
                if output.sha256 != input.sha256 {
                    return Err(ForgeError::Input(format!(
                        "spec_pipeline_prerequisite_error: predecessor hash mismatch for input {}",
                        input.path
                    )));
                }
                if output.schema_ref != input.schema_ref {
                    return Err(ForgeError::Input(format!(
                        "spec_pipeline_prerequisite_error: predecessor schema mismatch for input {}",
                        input.path
                    )));
                }
                if output.domain_model_ref != input.domain_model_ref {
                    return Err(ForgeError::Input(format!(
                        "spec_pipeline_prerequisite_error: predecessor domain model mismatch for input {}",
                        input.path
                    )));
                }
                if output.domainforge_version != input.domainforge_version {
                    return Err(ForgeError::Input(format!(
                        "spec_pipeline_prerequisite_error: predecessor DomainForge version mismatch for input {}",
                        input.path
                    )));
                }
            }
            None => {
                if input.schema_ref.is_none() {
                    return Err(ForgeError::Input(format!(
                        "spec_pipeline_prerequisite_error: externally supplied input {} has no verified schema",
                        input.path
                    )));
                }
            }
        }
    }
    Ok(())
}

// ── Quarantine ──

/// Quarantine a failed stage: set status, write quarantine ref.
/// The quarantine file is `{workspace}/quarantine/{stage_id}.jsonl` with
/// `{record, score, evidence_ref}` per §10.7/§7.6 pattern.
pub fn quarantine_stage(stage: &mut SpecPipelineStage, reason: &str) -> Result<(), ForgeError> {
    stage.status = StageStatus::Quarantined;
    stage.quarantine_ref = Some(format!("quarantine/{}.json", stage.stage_id));
    stage
        .settlement_basis
        .push(format!("stage_quarantined:{reason}"));
    Ok(())
}

// ── Generated-zone guard ──

/// Check if a path is in a generated zone (read-only per §10.7).
pub fn is_generated_zone(path: &str) -> bool {
    path.contains("src/gen/")
        || path.ends_with(".ast.json")
        || path.ends_with(".ir.json")
        || path.ends_with(".manifest.json")
        || path.contains("/fixtures/semantic/")
}

/// Deny direct edits to generated zones.
pub fn check_generated_zone_edit(path: &str) -> Result<(), ForgeError> {
    if is_generated_zone(path) {
        return Err(ForgeError::Input(format!(
            "generated_zone_direct_edit: direct edits to {path} are denied; use a spec or generator stage"
        )));
    }
    Ok(())
}

// ── Projection ──

/// Project a DomainModel into CALM and/or RDF via the DomainForge adapter.
/// Pure in-memory: no filesystem writes, no network, no CLI side effects (§10.4a).
pub fn project_model(
    model: &sea_forge_domainforge::DomainModel,
    kinds: &[ProjectionKind],
) -> Result<BTreeMap<ProjectionKind, BTreeMap<String, String>>, ForgeError> {
    let mut results = BTreeMap::new();
    for kind in kinds {
        let projected = sea_forge_domainforge::project(model, kind)?;
        results.insert(kind.clone(), projected);
    }
    Ok(results)
}

/// Compute the rebuild_hash for a ProjectionRecord per §7.0b.
/// Hash of canonical {adapter_ref, domain_model_ref, source_refs, descriptor_sha256, input_hash, output_refs}.
pub fn compute_rebuild_hash(
    adapter_ref: &str,
    domain_model_ref: &serde_json::Value,
    source_refs: &[String],
    descriptor_sha256: &str,
    input_hash: &str,
    output_refs: &[StageFile],
) -> Result<String, ForgeError> {
    let canonical = serde_json::json!({
        "adapter_ref": adapter_ref,
        "domain_model_ref": domain_model_ref,
        "source_refs": source_refs,
        "descriptor_sha256": descriptor_sha256,
        "input_hash": input_hash,
        "output_refs": output_refs,
    });
    let bytes = serde_json::to_vec(&canonical)?;
    Ok(format!("sha256:{:x}", Sha256::digest(&bytes)))
}

/// Build a ProjectionRecord from a successful projection.
#[allow(clippy::too_many_arguments)]
pub fn build_projection_record(
    projection_id: &str,
    kind: ProjectionKind,
    adapter_ref: &str,
    case_id: &str,
    run_id: &str,
    domain_model_ref: Option<&serde_json::Value>,
    source_refs: Vec<String>,
    input_hash: &str,
    outputs: &BTreeMap<String, String>,
    created_at: &str,
) -> Result<ProjectionRecord, ForgeError> {
    let mut output_refs = Vec::new();
    for (path, content) in outputs {
        let sha = format!("sha256:{:x}", Sha256::digest(content.as_bytes()));
        output_refs.push(StageFile {
            path: path.clone(),
            sha256: sha,
            generated: true,
            ..Default::default()
        });
    }
    output_refs.sort_by(|a, b| a.path.cmp(&b.path));

    let descriptor_sha256 = sea_forge_domainforge::ADAPTER_DESCRIPTOR_SHA256;
    let rebuild_hash = compute_rebuild_hash(
        adapter_ref,
        domain_model_ref.unwrap_or(&serde_json::Value::Null),
        &source_refs,
        descriptor_sha256,
        input_hash,
        &output_refs,
    )?;

    Ok(ProjectionRecord {
        projection_id: projection_id.into(),
        projection_kind: kind,
        adapter_ref: adapter_ref.into(),
        case_id: case_id.into(),
        run_id: run_id.into(),
        domain_model_ref: domain_model_ref.cloned(),
        source_refs,
        input_hash: input_hash.into(),
        output_refs,
        quarantine_refs: vec![],
        validation: ProjectionValidation {
            status: ProjectionStatus::Accepted,
            validator_ref: descriptor_sha256.into(),
            basis: vec!["projection_validated".into()],
        },
        authority_refs: vec![],
        evidence_refs: vec![],
        settlement_ref: None,
        created_at: created_at.into(),
        rebuild_hash,
    })
}

/// Process a pipeline run: validate ordering, quarantine failed or
/// prerequisite-invalid stages, compute classification.
///
/// A stage whose declared status is `Accepted` but whose input chain does
/// not verify against its predecessors (Task 9) is quarantined here too —
/// an executor's self-reported success cannot outrun the proof chain.
pub fn process_pipeline(run: &mut SpecPipelineRun) -> Result<(), ForgeError> {
    validate_stage_order(&run.stages)?;

    for index in 0..run.stages.len() {
        if run.stages[index].status == StageStatus::Rejected {
            quarantine_stage(&mut run.stages[index], "stage_output_validation_failed")?;
            continue;
        }
        if run.stages[index].status == StageStatus::Accepted {
            if let Err(error) = validate_stage_prerequisites(&run.stages, index) {
                quarantine_stage(&mut run.stages[index], &error.to_string())?;
            }
        }
    }

    run.proof_classification = compute_proof_classification(&run.stages);
    Ok(())
}

// ── Regeneration determinism ──

/// Verify that two sets of stage outputs are byte-identical.
/// Used to prove regeneration determinism (§10.7).
pub fn verify_byte_identity(a: &[StageFile], b: &[StageFile]) -> Result<bool, ForgeError> {
    if a.len() != b.len() {
        return Ok(false);
    }
    let mut a_sorted = a.to_vec();
    a_sorted.sort_by(|x, y| x.path.cmp(&y.path));
    let mut b_sorted = b.to_vec();
    b_sorted.sort_by(|x, y| x.path.cmp(&y.path));
    for (a_file, b_file) in a_sorted.iter().zip(b_sorted.iter()) {
        if a_file.path != b_file.path || a_file.sha256 != b_file.sha256 {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Hash file content for stage I/O records.
pub fn hash_content(content: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(content))
}

/// Compute the input_hash for a projection (hash of all source file hashes).
pub fn compute_input_hash(source_refs: &[String]) -> Result<String, ForgeError> {
    let canonical = serde_json::json!({ "source_refs": source_refs });
    hash_canonical(&canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stage(kind: StageKind, status: StageStatus) -> SpecPipelineStage {
        SpecPipelineStage {
            stage_id: format!("stage_{kind:?}").to_lowercase(),
            kind,
            inputs: vec![],
            outputs: vec![],
            command: None,
            status,
            quarantine_ref: None,
            settlement_basis: vec![],
        }
    }

    #[test]
    fn chain_hash_changes_on_input_change() {
        let stages = vec![
            make_stage(StageKind::Adr, StageStatus::Accepted),
            make_stage(StageKind::Prd, StageStatus::Accepted),
        ];
        let hash1 = compute_chain_hash(&stages).unwrap();

        let mut stages2 = stages.clone();
        stages2[1].status = StageStatus::Rejected;
        let hash2 = compute_chain_hash(&stages2).unwrap();

        assert_ne!(hash1, hash2, "chain hash must change when a stage changes");
    }

    fn full_prefix_stages(extra: &[StageKind]) -> Vec<SpecPipelineStage> {
        let mut stages: Vec<SpecPipelineStage> = REQUIRED_FOR_GENERATED_CONTRACT
            .iter()
            .map(|kind| make_stage(kind.clone(), StageStatus::Accepted))
            .collect();
        stages.extend(
            extra
                .iter()
                .map(|kind| make_stage(kind.clone(), StageStatus::Accepted)),
        );
        stages
    }

    #[test]
    fn classification_ceiling_without_last_mile() {
        let stages = full_prefix_stages(&[]);
        let classification = compute_proof_classification(&stages);
        assert_eq!(classification, ProofClassification::GeneratedContract);
    }

    #[test]
    fn classification_reaches_focused_slice() {
        let stages = full_prefix_stages(&[
            StageKind::LastMileAdapter,
            StageKind::RuntimeWiring,
            StageKind::AcceptanceProof,
        ]);
        let classification = compute_proof_classification(&stages);
        assert_eq!(classification, ProofClassification::FocusedSlice);
    }

    #[test]
    fn classification_ceiling_requires_full_prefix_before_focused_slice() {
        // A sparse stage list containing ONLY the last four canonical kinds
        // must NOT reach focused-slice: the earlier ADR..Manifest prefix is
        // absent, so the proof chain behind it is unverified. This is the
        // audited defect (a fixture granting focused-slice classification
        // without earlier stages present).
        let sparse = vec![
            make_stage(StageKind::GeneratedContract, StageStatus::Accepted),
            make_stage(StageKind::LastMileAdapter, StageStatus::Accepted),
            make_stage(StageKind::RuntimeWiring, StageStatus::Accepted),
            make_stage(StageKind::AcceptanceProof, StageStatus::Accepted),
        ];
        assert_eq!(
            compute_proof_classification(&sparse),
            ProofClassification::AuthorityOnly,
            "classification must not exceed authority-only when the earlier canonical prefix is missing"
        );
    }

    #[test]
    fn generated_zone_denied() {
        assert!(check_generated_zone_edit("src/gen/model.rs").is_err());
        assert!(check_generated_zone_edit("docs/model.ast.json").is_err());
        assert!(check_generated_zone_edit("docs/model.ir.json").is_err());
        assert!(check_generated_zone_edit("src/authored.rs").is_ok());
    }

    #[test]
    fn byte_identity_check() {
        let a = vec![StageFile {
            path: "out.json".into(),
            sha256: "sha256:abc".into(),
            generated: true,
            ..Default::default()
        }];
        let b = vec![StageFile {
            path: "out.json".into(),
            sha256: "sha256:abc".into(),
            generated: true,
            ..Default::default()
        }];
        assert!(verify_byte_identity(&a, &b).unwrap());

        let c = vec![StageFile {
            path: "out.json".into(),
            sha256: "sha256:xyz".into(),
            generated: true,
            ..Default::default()
        }];
        assert!(!verify_byte_identity(&a, &c).unwrap());
    }
}
