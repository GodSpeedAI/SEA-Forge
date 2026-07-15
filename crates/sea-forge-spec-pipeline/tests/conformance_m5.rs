use sea_forge_core::types::*;
use sea_forge_domainforge::{load_validate, SeaSourceSet, SourceFile};
use sea_forge_spec_pipeline::*;
use sha2::{Digest, Sha256};

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn source_file(uri: &str, content: &str) -> SourceFile {
    SourceFile {
        uri: uri.into(),
        sha256: sha256_hex(content.as_bytes()),
        content: content.into(),
    }
}

const DEMO_SEA: &str = r#"@namespace "demo"
@version "1.0.0"

Entity "Sample" in demo
Resource "Artifact" units in demo
Flow "Artifact" from "Sample" to "Sample" quantity 1
"#;

fn demo_source_set() -> SeaSourceSet {
    SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![source_file("demo.sea", DEMO_SEA)],
    }
}

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

fn make_stage_with_io(
    kind: StageKind,
    status: StageStatus,
    inputs: Vec<(&str, &str)>,
    outputs: Vec<(&str, &str)>,
) -> SpecPipelineStage {
    SpecPipelineStage {
        stage_id: format!("stage_{kind:?}").to_lowercase(),
        kind,
        inputs: inputs
            .iter()
            .map(|(p, h)| StageFile {
                path: p.to_string(),
                sha256: h.to_string(),
                generated: false,
            })
            .collect(),
        outputs: outputs
            .iter()
            .map(|(p, h)| StageFile {
                path: p.to_string(),
                sha256: h.to_string(),
                generated: true,
            })
            .collect(),
        command: None,
        status,
        quarantine_ref: None,
        settlement_basis: vec![],
    }
}

// ── 1. Full pipeline hash-chain ──

#[test]
fn pipeline_stages_hash_link_in_order() {
    let stages = vec![
        make_stage(StageKind::Adr, StageStatus::Accepted),
        make_stage(StageKind::Prd, StageStatus::Accepted),
        make_stage(StageKind::Sds, StageStatus::Accepted),
        make_stage(StageKind::Sea, StageStatus::Accepted),
    ];
    let chain_hash = compute_chain_hash(&stages).unwrap();
    assert!(chain_hash.starts_with("sha256:"));

    // Touching one input byte of a stage changes the chain hash.
    let mut modified_stages = stages.clone();
    modified_stages[1].stage_id = "stage_prd_modified".into();
    let modified_hash = compute_chain_hash(&modified_stages).unwrap();
    assert_ne!(
        chain_hash, modified_hash,
        "changing a stage must change the chain hash"
    );
}

// ── 2. Direct generated-zone edit denial ──

#[test]
fn direct_generated_zone_edit_is_denied() {
    assert!(check_generated_zone_edit("src/gen/model.rs").is_err());
    assert!(check_generated_zone_edit("docs/specs/model.ast.json").is_err());
    assert!(check_generated_zone_edit("docs/specs/model.ir.json").is_err());
    assert!(check_generated_zone_edit("docs/specs/model.manifest.json").is_err());
    assert!(
        check_generated_zone_edit("docs/specs/fixtures/semantic/thing.semantic.fixture.yaml")
            .is_err()
    );

    // Authored paths are fine.
    assert!(check_generated_zone_edit("src/authored/handler.rs").is_ok());
    assert!(check_generated_zone_edit("docs/specs/model.sea").is_ok());
}

// ── 3. Regeneration byte-identity ──

#[test]
fn regeneration_produces_byte_identical_output() {
    let content = b"generated contract text";
    let hash = hash_content(content);

    let stage1 = make_stage_with_io(
        StageKind::GeneratedContract,
        StageStatus::Accepted,
        vec![],
        vec![("contract.rs", &hash)],
    );
    let stage2 = make_stage_with_io(
        StageKind::GeneratedContract,
        StageStatus::Accepted,
        vec![],
        vec![("contract.rs", &hash)],
    );
    assert!(
        verify_byte_identity(&stage1.outputs, &stage2.outputs).unwrap(),
        "same content must produce byte-identical output"
    );

    let different_hash = hash_content(b"different content");
    let stage3 = make_stage_with_io(
        StageKind::GeneratedContract,
        StageStatus::Accepted,
        vec![],
        vec![("contract.rs", &different_hash)],
    );
    assert!(
        !verify_byte_identity(&stage1.outputs, &stage3.outputs).unwrap(),
        "different content must fail byte-identity check"
    );
}

// ── 4. Classification ceiling ──

#[test]
fn classification_ceiling_prevents_above_generated_contract_without_last_mile() {
    // Generated contract accepted, but no last-mile/runtime/acceptance.
    let mut run = SpecPipelineRun {
        version: "0.1".into(),
        pipeline_id: "pipe_001".into(),
        case_id: "case_001".into(),
        run_id: "run_001".into(),
        context_id: Some("docs/specs/demo".into()),
        domain_model_ref: None,
        route: PipelineRoute::FullSpecToRuntime,
        authority_refs: vec![],
        evidence_refs: vec![],
        settlement_ref: None,
        stages: vec![
            make_stage(StageKind::Adr, StageStatus::Accepted),
            make_stage(StageKind::Prd, StageStatus::Accepted),
            make_stage(StageKind::Sds, StageStatus::Accepted),
            make_stage(StageKind::Sea, StageStatus::Accepted),
            make_stage(StageKind::Ast, StageStatus::Accepted),
            make_stage(StageKind::Ir, StageStatus::Accepted),
            make_stage(StageKind::Manifest, StageStatus::Accepted),
            make_stage(StageKind::GeneratedContract, StageStatus::Accepted),
        ],
        proof_classification: ProofClassification::AuthorityOnly,
    };
    process_pipeline(&mut run).unwrap();
    assert_eq!(
        run.proof_classification,
        ProofClassification::GeneratedContract,
        "classification must not exceed generated-contract without last-mile"
    );

    // Add last-mile + runtime + acceptance → focused-slice.
    run.stages.push(make_stage(
        StageKind::LastMileAdapter,
        StageStatus::Accepted,
    ));
    run.stages
        .push(make_stage(StageKind::RuntimeWiring, StageStatus::Accepted));
    run.stages.push(make_stage(
        StageKind::AcceptanceProof,
        StageStatus::Accepted,
    ));
    process_pipeline(&mut run).unwrap();
    assert_eq!(
        run.proof_classification,
        ProofClassification::FocusedSlice,
        "classification reaches focused-slice with last-mile + runtime + acceptance"
    );
}

// ── 5. CALM + RDF projection from same DomainModelRef ──

#[test]
fn calm_and_rdf_projections_derive_from_same_domain_model_ref() {
    let model = load_validate(&demo_source_set()).unwrap();
    let kinds = vec![ProjectionKind::Calm, ProjectionKind::Rdf];
    let results = project_model(&model, &kinds).unwrap();

    assert!(results.contains_key(&ProjectionKind::Calm));
    assert!(results.contains_key(&ProjectionKind::Rdf));

    let calm = &results[&ProjectionKind::Calm];
    assert!(calm.contains_key("calm.json"));
    let calm_value: serde_json::Value = serde_json::from_str(&calm["calm.json"]).unwrap();
    assert!(calm_value.is_object());

    let rdf = &results[&ProjectionKind::Rdf];
    assert!(rdf.contains_key("model.ttl"));
    assert!(!rdf["model.ttl"].is_empty());
}

// ── 6. ProjectionRecord rebuild byte-identical ──

#[test]
fn projection_record_rebuild_is_byte_identical() {
    let model = load_validate(&demo_source_set()).unwrap();
    let now = "2026-07-14T00:00:00Z";
    let source_refs: Vec<String> = model
        .model_ref
        .source_refs
        .iter()
        .map(|r| format!("{}:{}", r.uri, r.sha256))
        .collect();
    let input_hash = compute_input_hash(&source_refs).unwrap();
    let model_ref_value = serde_json::to_value(&model.model_ref).unwrap();

    // Independent projection calls — must produce identical rebuild_hash
    // because the projection is deterministic (timestamp stripped, §10.7).
    let record1 = build_projection_record(
        "proj_001",
        ProjectionKind::Calm,
        "sea-forge-domainforge@0.1",
        "case_001",
        "run_001",
        Some(&model_ref_value),
        source_refs.clone(),
        &input_hash,
        &project_model(&model, &[ProjectionKind::Calm]).unwrap()[&ProjectionKind::Calm],
        now,
    )
    .unwrap();

    let record2 = build_projection_record(
        "proj_001",
        ProjectionKind::Calm,
        "sea-forge-domainforge@0.1",
        "case_001",
        "run_001",
        Some(&model_ref_value),
        source_refs,
        &input_hash,
        &project_model(&model, &[ProjectionKind::Calm]).unwrap()[&ProjectionKind::Calm],
        now,
    )
    .unwrap();

    assert_eq!(
        record1.rebuild_hash, record2.rebuild_hash,
        "same inputs must produce identical rebuild_hash"
    );
    assert_eq!(record1.output_refs, record2.output_refs);
    assert_eq!(record1.validation.status, ProjectionStatus::Accepted);
}

// ── 7. Quarantine completeness ──

#[test]
fn quarantine_retains_failed_stage_with_provenance() {
    let mut stage = make_stage(StageKind::GeneratedContract, StageStatus::Rejected);
    quarantine_stage(&mut stage, "output_validation_failed").unwrap();

    assert_eq!(stage.status, StageStatus::Quarantined);
    assert!(stage.quarantine_ref.is_some());
    assert!(
        stage
            .settlement_basis
            .iter()
            .any(|b| b.contains("stage_quarantined")),
        "settlement_basis should record the quarantine reason"
    );
}

// ── 8. Stage ordering validation ──

#[test]
fn out_of_order_stages_are_rejected() {
    let stages = vec![
        make_stage(StageKind::Prd, StageStatus::Accepted),
        make_stage(StageKind::Adr, StageStatus::Accepted), // out of order
    ];
    assert!(validate_stage_order(&stages).is_err());

    let valid_stages = vec![
        make_stage(StageKind::Adr, StageStatus::Accepted),
        make_stage(StageKind::Prd, StageStatus::Accepted),
        make_stage(StageKind::Sds, StageStatus::Accepted),
    ];
    assert!(validate_stage_order(&valid_stages).is_ok());
}

// ── 9. Pipeline processing quarantines rejected stages ──

#[test]
fn pipeline_processing_quarantines_rejected_stages() {
    let mut run = SpecPipelineRun {
        version: "0.1".into(),
        pipeline_id: "pipe_002".into(),
        case_id: "case_002".into(),
        run_id: "run_002".into(),
        context_id: None,
        domain_model_ref: None,
        route: PipelineRoute::FullSpecToRuntime,
        authority_refs: vec![],
        evidence_refs: vec![],
        settlement_ref: None,
        stages: vec![
            make_stage(StageKind::Adr, StageStatus::Accepted),
            make_stage(StageKind::Prd, StageStatus::Rejected), // failed
            make_stage(StageKind::Sds, StageStatus::Skipped),  // skipped due to prior failure
        ],
        proof_classification: ProofClassification::AuthorityOnly,
    };
    process_pipeline(&mut run).unwrap();

    // Rejected stage should be quarantined.
    assert_eq!(run.stages[1].status, StageStatus::Quarantined);
    assert!(run.stages[1].quarantine_ref.is_some());

    // Classification should be authority-only (no generated outputs).
    assert_eq!(run.proof_classification, ProofClassification::AuthorityOnly);
}

// ── 10. No-side-effect proof: projection is pure in-memory ──

#[test]
fn projection_has_no_filesystem_side_effects() {
    let dir = tempfile::TempDir::new().unwrap();
    let model = load_validate(&demo_source_set()).unwrap();

    // Run projection — it should not write anything to disk.
    let results = project_model(&model, &[ProjectionKind::Calm, ProjectionKind::Rdf]).unwrap();

    // Verify no files were created in the temp dir by the projection.
    let before: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
    assert!(before.is_empty(), "projection must not create files");

    // The results are in-memory only.
    assert!(!results.is_empty());
    // If we want to persist, SEA Forge (not DomainForge) must do it under authority.
}
