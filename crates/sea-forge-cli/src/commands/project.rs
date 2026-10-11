//! `sea-forge project` (spec-audit-remediation Task 10B): a thin adapter
//! wiring a real CLI invocation through Task 10A's case-runner stage path
//! and Task 5's validated DomainForge model, then independently settling
//! CALM/RDF (or other) `ProjectionRecord`s from that one model.
//!
//! SEA Forge (this command), not DomainForge, owns every authorized output
//! write; `sea_forge_domainforge::{load_validate, project}` are pure,
//! in-memory, side-effect-free calls (§10.4a).

use sea_forge_core::{
    errors::ForgeError,
    ids,
    types::{
        PipelineRoute, ProjectionKind, ProjectionRecord, ProjectionStatus, ProjectionValidation,
        SettlementEvent, SettlementStatus, SpecPipelineRun, SpecPipelineStage, StageFile,
        StageKind, StageStatus,
    },
    RECORD_VERSION,
};
use sea_forge_domainforge::{SeaSourceSet, SourceFile};
use sea_forge_ledger::LedgerStream;
use serde_json::json;
use std::{fs, path::Path};

pub struct ProjectOptions<'a> {
    pub entry: &'a Path,
    pub root: &'a Path,
    pub policy: &'a Path,
    pub entity: &'a str,
    pub projections: &'a [ProjectionKind],
}

fn sha256(bytes: &[u8]) -> String {
    sea_forge_evidence::sha256_bytes(bytes)
}

fn self_exe() -> Result<String, ForgeError> {
    let path = std::env::current_exe()
        .and_then(|p| p.canonicalize())
        .map_err(|e| ForgeError::io("resolve current executable", e))?;
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| ForgeError::Input("non-utf8 executable path".into()))
}

/// One canonical-order stage: writes real content to `stage_dir`, wires its
/// input to the prior stage's output by path+hash, and returns the
/// `SpecPipelineStage` plus its own `(path, sha256)` for the next stage.
fn write_stage(
    stage_dir: &Path,
    stage_id: &str,
    kind: StageKind,
    file_name: &str,
    content: &[u8],
    predecessor: Option<(&str, &str)>,
    exe: &str,
) -> Result<(SpecPipelineStage, (String, String)), ForgeError> {
    let path = stage_dir.join(file_name);
    fs::write(&path, content).map_err(|e| ForgeError::io("write stage content", e))?;
    let hash = sha256(content);
    let rel_path = file_name.to_string();
    let inputs = match predecessor {
        Some((pred_path, pred_hash)) => vec![StageFile {
            path: pred_path.into(),
            sha256: pred_hash.into(),
            generated: false,
            ..Default::default()
        }],
        None => vec![],
    };
    let stage = SpecPipelineStage {
        stage_id: stage_id.into(),
        kind,
        inputs,
        outputs: vec![StageFile {
            path: rel_path.clone(),
            sha256: hash.clone(),
            generated: true,
            ..Default::default()
        }],
        command: Some(vec![
            exe.into(),
            "stage-check".into(),
            path.display().to_string(),
            hash.clone(),
        ]),
        status: StageStatus::Pending,
        quarantine_ref: None,
        settlement_basis: vec![],
    };
    Ok((stage, (rel_path, hash)))
}

fn commit_projection(
    stream: &LedgerStream,
    model: &sea_forge_domainforge::DomainModel,
    kind: &ProjectionKind,
    case_id: &str,
    run_id: &str,
    source_refs: &[String],
    input_hash: &str,
) -> Result<bool, ForgeError> {
    let adapter_ref = "sea-forge-domainforge@0.1";
    let projection_id = ids::random_id("proj")?;
    let model_ref_value = serde_json::to_value(&model.model_ref)?;
    let created_at = chrono::Utc::now().to_rfc3339();

    let (record, accepted) = match sea_forge_domainforge::project(model, kind) {
        Ok(outputs) => {
            let record = sea_forge_spec_pipeline::build_projection_record(
                &projection_id,
                kind.clone(),
                adapter_ref,
                case_id,
                run_id,
                Some(&model_ref_value),
                source_refs.to_vec(),
                input_hash,
                &outputs,
                &created_at,
            )?;
            (record, true)
        }
        Err(error) => {
            let descriptor_sha256 = sea_forge_domainforge::ADAPTER_DESCRIPTOR_SHA256;
            let rebuild_hash = sea_forge_spec_pipeline::compute_rebuild_hash(
                adapter_ref,
                &model_ref_value,
                source_refs,
                descriptor_sha256,
                input_hash,
                &[],
            )?;
            let record = ProjectionRecord {
                projection_id: projection_id.clone(),
                projection_kind: kind.clone(),
                adapter_ref: adapter_ref.into(),
                case_id: case_id.into(),
                run_id: run_id.into(),
                domain_model_ref: Some(model_ref_value),
                source_refs: source_refs.to_vec(),
                input_hash: input_hash.into(),
                output_refs: vec![],
                quarantine_refs: vec![format!("quarantine/projection_{projection_id}.json")],
                validation: ProjectionValidation {
                    status: ProjectionStatus::Rejected,
                    validator_ref: descriptor_sha256.into(),
                    basis: vec![format!("projection_failed:{error}")],
                },
                authority_refs: vec![],
                evidence_refs: vec![],
                settlement_ref: None,
                created_at,
                rebuild_hash,
            };
            (record, false)
        }
    };

    stream.commit_typed(
        "projection_record",
        vec![case_id.into(), run_id.into(), projection_id.clone()],
        &record,
        vec![],
    )?;
    let settlement = SettlementEvent {
        version: RECORD_VERSION.into(),
        settlement_id: ids::random_id("set")?,
        run_id: run_id.into(),
        status: if accepted {
            SettlementStatus::Accepted
        } else {
            SettlementStatus::Rejected
        },
        basis: vec![if accepted {
            "projection_validated".into()
        } else {
            "projection_validation_failed".into()
        }],
        review_required: false,
        settled_at: chrono::Utc::now().to_rfc3339(),
        criteria_ref: None,
    };
    stream.commit_typed(
        "settlement_event",
        vec![
            case_id.into(),
            run_id.into(),
            settlement.settlement_id.clone(),
        ],
        &settlement,
        vec![],
    )?;
    Ok(accepted)
}

pub fn execute(options: ProjectOptions<'_>) -> Result<u8, ForgeError> {
    fs::create_dir_all(options.root).map_err(|e| ForgeError::io("create state root", e))?;
    let root = options
        .root
        .canonicalize()
        .map_err(|e| ForgeError::io("canonicalize root", e))?;

    let entry_content =
        fs::read_to_string(options.entry).map_err(|e| ForgeError::io("read entry file", e))?;
    let entry_uri = options
        .entry
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("model.sea")
        .to_string();
    let source_set = SeaSourceSet {
        entry_uri: entry_uri.clone(),
        files: vec![SourceFile {
            uri: entry_uri,
            sha256: sha256(entry_content.as_bytes()),
            content: entry_content.clone(),
        }],
    };
    let model = sea_forge_domainforge::load_validate(&source_set)?;

    let pipeline_id = ids::random_id("pipe")?;
    let case_id = ids::case_id()?;
    let run_id = ids::run_id()?;
    let intent_id = ids::random_id("int")?;
    let exe = self_exe()?;
    let stage_dir = root.join("spec-pipelines").join(&pipeline_id);
    fs::create_dir_all(&stage_dir).map_err(|e| ForgeError::io("create stage dir", e))?;

    let (adr, adr_out) = write_stage(
        &stage_dir,
        "stage_adr",
        StageKind::Adr,
        "adr.md",
        b"# ADR\n\nAccepted decision: build the demo domain model.\n",
        None,
        &exe,
    )?;
    let (prd, prd_out) = write_stage(
        &stage_dir,
        "stage_prd",
        StageKind::Prd,
        "prd.md",
        b"# PRD\n\nProduct requirements for the demo domain model.\n",
        Some((&adr_out.0, &adr_out.1)),
        &exe,
    )?;
    let (sds, sds_out) = write_stage(
        &stage_dir,
        "stage_sds",
        StageKind::Sds,
        "sds.md",
        b"# SDS\n\nSoftware design spec for the demo domain model.\n",
        Some((&prd_out.0, &prd_out.1)),
        &exe,
    )?;
    let (sea, sea_out) = write_stage(
        &stage_dir,
        "stage_sea",
        StageKind::Sea,
        "model.sea",
        entry_content.as_bytes(),
        Some((&sds_out.0, &sds_out.1)),
        &exe,
    )?;
    let ast_content = format!("{:#?}", model.graph);
    let (ast, ast_out) = write_stage(
        &stage_dir,
        "stage_ast",
        StageKind::Ast,
        "model.ast.json",
        ast_content.as_bytes(),
        Some((&sea_out.0, &sea_out.1)),
        &exe,
    )?;
    let ir_content = serde_json::to_string_pretty(&model.model_ref)?;
    let (ir, ir_out) = write_stage(
        &stage_dir,
        "stage_ir",
        StageKind::Ir,
        "model.ir.json",
        ir_content.as_bytes(),
        Some((&ast_out.0, &ast_out.1)),
        &exe,
    )?;
    let manifest_content = serde_json::to_string_pretty(&json!({
        "domainforge_version": model.model_ref.domainforge_version,
        "semantic_model_sha256": model.model_ref.semantic_model_sha256,
        "source_refs": model.model_ref.source_refs,
    }))?;
    let (manifest, manifest_out) = write_stage(
        &stage_dir,
        "stage_manifest",
        StageKind::Manifest,
        "model.manifest.json",
        manifest_content.as_bytes(),
        Some((&ir_out.0, &ir_out.1)),
        &exe,
    )?;
    let calm_outputs = sea_forge_domainforge::project(&model, &ProjectionKind::Calm)?;
    let calm_content = calm_outputs.get("calm.json").cloned().unwrap_or_default();
    let (generated_contract, _) = write_stage(
        &stage_dir,
        "stage_generated_contract",
        StageKind::GeneratedContract,
        "generated_contract.json",
        calm_content.as_bytes(),
        Some((&manifest_out.0, &manifest_out.1)),
        &exe,
    )?;

    let stages = vec![adr, prd, sds, sea, ast, ir, manifest, generated_contract];
    let plan = sea_forge_planner::stage_case_plan(&stages, &case_id, &run_id, &intent_id)?;
    let outcome = sea_forge_case_runner::run_stage_case(
        &root,
        options.policy,
        options.entity,
        &plan,
        stages,
    )?;

    let stream = LedgerStream::open(&root, format!("case-{case_id}"), options.entity)?;
    let mut pipeline_run = SpecPipelineRun {
        version: RECORD_VERSION.into(),
        pipeline_id: pipeline_id.clone(),
        case_id: case_id.clone(),
        run_id: run_id.clone(),
        context_id: Some(stage_dir.display().to_string()),
        domain_model_ref: Some(serde_json::to_value(&model.model_ref)?),
        route: PipelineRoute::FullSpecToRuntime,
        authority_refs: vec![],
        evidence_refs: vec![],
        settlement_ref: None,
        stages: outcome.stages.clone(),
        proof_classification: sea_forge_core::types::ProofClassification::AuthorityOnly,
    };
    sea_forge_spec_pipeline::process_pipeline(&mut pipeline_run)?;
    stream.commit_typed(
        "spec_pipeline_run",
        vec![case_id.clone(), run_id.clone(), pipeline_id.clone()],
        &pipeline_run,
        vec![],
    )?;

    let source_refs: Vec<String> = model
        .model_ref
        .source_refs
        .iter()
        .map(|r| format!("{}:{}", r.uri, r.sha256))
        .collect();
    let input_hash = sea_forge_spec_pipeline::compute_input_hash(&source_refs)?;
    let mut accepted = 0;
    let mut quarantined = 0;
    for kind in options.projections {
        let ok = commit_projection(
            &stream,
            &model,
            kind,
            &case_id,
            &run_id,
            &source_refs,
            &input_hash,
        )?;
        if ok {
            accepted += 1;
        } else {
            quarantined += 1;
        }
    }

    println!("case_id={case_id}");
    println!("case_state={}", outcome.state);
    println!(
        "proof_classification={:?}",
        pipeline_run.proof_classification
    );
    println!("projections_accepted={accepted}");
    println!("projections_quarantined={quarantined}");

    Ok(if outcome.state == "completed" && quarantined == 0 {
        0
    } else {
        1
    })
}
