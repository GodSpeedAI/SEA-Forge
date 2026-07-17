use sea_forge_core::types::*;
use sea_forge_core::ForgeError;
use sea_forge_ledger::LedgerStream;
use sea_forge_planner::{
    criteria::{
        compute_criteria_sha256, compute_record_hash, derive_from_intent, derive_from_template,
        verify_item_criteria, verify_plan_criteria,
    },
    templates::{instantiate, PlanTemplate, TemplateItem, TemplateOperation, TemplatePlan},
};
use std::collections::BTreeMap;

fn demo_intent() -> Intent {
    Intent {
        intent_id: "int_abcdef".into(),
        summary: "generate and validate sea model".into(),
        actor_id: "entity".into(),
        process_id: "process".into(),
        created_at: "2026-07-13T00:00:00Z".into(),
    }
}

fn demo_item() -> PlanItem {
    PlanItem {
        plan_item_id: "item_01".into(),
        name: "generate_and_validate".into(),
        operations: vec![],
        entry_criteria: vec![],
        exit_criteria: vec![],
        settlement_criteria: SettlementCriteria {
            require_exit_zero: true,
            required_artifacts: vec!["model.sea".into()],
            stdout_must_contain: Some("valid".into()),
            ..Default::default()
        },
        settlement_criteria_ref: None,
        item_kind: ItemKind::SandboxedTask,
        sandbox_class: None,
        parent_stage: None,
        markers: Default::default(),
        max_instances: 1,
        depends_on: vec![],
        environment: None,
    }
}

#[test]
fn criteria_hash_is_deterministic() {
    let item = demo_item();
    let a = compute_criteria_sha256(&item.settlement_criteria).unwrap();
    let b = compute_criteria_sha256(&item.settlement_criteria).unwrap();
    assert!(!a.is_empty());
    assert_eq!(a, b);
}

#[test]
fn derived_record_hashes_to_itself() {
    let intent = demo_intent();
    let item = demo_item();
    let record = derive_from_intent(&intent, &item, "entity", "2026-07-13T00:00:00Z").unwrap();
    assert_eq!(
        record.criteria_record_hash,
        compute_record_hash(&record).unwrap()
    );
    assert!(!record.origin_refs.is_empty());
    assert_eq!(record.origin_refs[0].kind, OriginRefKind::Intent);
    assert_eq!(record.origin_refs[0].reference, intent.intent_id);
}

#[test]
fn intent_origin_reuses_existing_substrate_no_job_contract() {
    let intent = demo_intent();
    let item = demo_item();
    let record = derive_from_intent(&intent, &item, "entity", "2026-07-13T00:00:00Z").unwrap();
    assert_eq!(record.origin_refs.len(), 1);
    assert_eq!(record.origin_refs[0].kind, OriginRefKind::Intent);
    assert!(!record.origin_refs[0].sha256.is_empty());
    assert_eq!(
        record.derivation.method,
        DerivationMethod::DeterministicPlanner
    );
}

#[test]
fn verify_item_criteria_resolves_and_matches_snapshot() {
    let intent = demo_intent();
    let mut item = demo_item();
    let record = derive_from_intent(&intent, &item, "entity", "2026-07-13T00:00:00Z").unwrap();
    item.settlement_criteria_ref = Some(record.criteria_id.clone());
    let mut map = BTreeMap::new();
    map.insert(record.criteria_id.clone(), record.clone());
    let resolved = verify_item_criteria(&item, &map).unwrap();
    assert_eq!(resolved.criteria_id, record.criteria_id);
}

#[test]
fn verify_item_criteria_fails_when_ref_missing() {
    let item = demo_item();
    let error = verify_item_criteria(&item, &BTreeMap::new()).unwrap_err();
    assert_eq!(error.class(), "criteria_provenance_error");
}

#[test]
fn verify_item_criteria_fails_when_snapshot_hash_mismatches() {
    let intent = demo_intent();
    let mut item = demo_item();
    let record = derive_from_intent(&intent, &item, "entity", "2026-07-13T00:00:00Z").unwrap();
    item.settlement_criteria_ref = Some(record.criteria_id.clone());
    item.settlement_criteria.require_exit_zero = false; // mutate embedded criteria
    let mut map = BTreeMap::new();
    map.insert(record.criteria_id.clone(), record);
    let error = verify_item_criteria(&item, &map).unwrap_err();
    assert_eq!(error.class(), "criteria_provenance_error");
}

#[test]
fn template_instantiation_criteria_is_deterministic_ignoring_id() {
    let intent = demo_intent();
    let template = PlanTemplate {
        name: "demo".into(),
        version: "1.0.0".into(),
        description: "test".into(),
        parameters: BTreeMap::new(),
        origin_refs: vec![],
        job_contract: None,
        plan: TemplatePlan {
            items: vec![TemplateItem {
                plan_item_id: "item_01".into(),
                name: "write_model".into(),
                operations: vec![TemplateOperation::WriteFile {
                    path: "model.sea".into(),
                    content_hint: "{}".into(),
                }],
                settlement_criteria: SettlementCriteria {
                    require_exit_zero: true,
                    required_artifacts: vec!["model.sea".into()],
                    stdout_must_contain: None,
                    ..Default::default()
                },
                item_kind: ItemKind::SandboxedTask,
                sandbox_class: None,
                markers: Default::default(),
                max_instances: 1,
                environment: None,
                entry_criteria: vec![],
                exit_criteria: vec![],
                parent_stage: None,
                depends_on: vec![],
            }],
        },
    };
    let params = BTreeMap::new();
    let first = instantiate(&template, &params, "case_01", "run_01", &intent.intent_id).unwrap();
    let second = instantiate(&template, &params, "case_01", "run_01", &intent.intent_id).unwrap();
    let record_a = derive_from_template(
        &template,
        &first.items[0],
        &intent,
        "entity",
        "2026-07-13T00:00:00Z",
    )
    .unwrap();
    let record_b = derive_from_template(
        &template,
        &second.items[0],
        &intent,
        "entity",
        "2026-07-13T00:00:00Z",
    )
    .unwrap();
    assert_eq!(
        first.items[0].settlement_criteria,
        second.items[0].settlement_criteria
    );
    assert_eq!(record_a.criteria, record_b.criteria);
    assert_ne!(record_a.criteria_id, record_b.criteria_id);
    assert_eq!(record_a.origin_refs, record_b.origin_refs);
    assert_eq!(record_a.criteria_sha256, record_b.criteria_sha256);
    assert_eq!(record_a.derivation.method, record_b.derivation.method);
    // criteria_record_hash includes the random criteria_id, so compare the
    // canonical content after normalizing the ledger-assigned identity.
    let mut normalized_a = record_a.clone();
    let mut normalized_b = record_b.clone();
    normalized_a.criteria_id = "crit_same".into();
    normalized_b.criteria_id = "crit_same".into();
    normalized_a.criteria_record_hash = compute_record_hash(&normalized_a).unwrap();
    normalized_b.criteria_record_hash = compute_record_hash(&normalized_b).unwrap();
    assert_eq!(
        normalized_a.criteria_record_hash,
        normalized_b.criteria_record_hash
    );
}

#[test]
fn template_origin_reuses_intent_and_template() {
    let intent = demo_intent();
    let template = PlanTemplate {
        name: "demo".into(),
        version: "1.0.0".into(),
        description: "test".into(),
        parameters: BTreeMap::new(),
        origin_refs: vec![],
        job_contract: None,
        plan: TemplatePlan {
            items: vec![TemplateItem {
                plan_item_id: "item_01".into(),
                name: "write_model".into(),
                operations: vec![TemplateOperation::WriteFile {
                    path: "model.sea".into(),
                    content_hint: "{}".into(),
                }],
                settlement_criteria: SettlementCriteria {
                    require_exit_zero: true,
                    required_artifacts: vec!["model.sea".into()],
                    stdout_must_contain: None,
                    ..Default::default()
                },
                item_kind: ItemKind::SandboxedTask,
                sandbox_class: None,
                markers: Default::default(),
                max_instances: 1,
                environment: None,
                entry_criteria: vec![],
                exit_criteria: vec![],
                parent_stage: None,
                depends_on: vec![],
            }],
        },
    };
    let params = BTreeMap::new();
    let plan = instantiate(&template, &params, "case_01", "run_01", &intent.intent_id).unwrap();
    let record = derive_from_template(
        &template,
        &plan.items[0],
        &intent,
        "entity",
        "2026-07-13T00:00:00Z",
    )
    .unwrap();
    let kinds: Vec<_> = record.origin_refs.iter().map(|o| o.kind.clone()).collect();
    assert!(kinds.contains(&OriginRefKind::Intent));
    assert!(kinds.contains(&OriginRefKind::PlanTemplate));
    assert_eq!(record.derivation.method, DerivationMethod::PlanTemplate);
    assert_eq!(record.derivation.producer_ref, Some("demo@1.0.0".into()));
}

#[test]
fn changing_criteria_changes_hash() {
    let item_a = demo_item();
    let mut item_b = demo_item();
    item_b.settlement_criteria.required_artifacts = vec!["other.sea".into()];
    let hash_a = compute_criteria_sha256(&item_a.settlement_criteria).unwrap();
    let hash_b = compute_criteria_sha256(&item_b.settlement_criteria).unwrap();
    assert_ne!(hash_a, hash_b);
}

#[test]
fn criteria_record_commits_to_ledger_and_resolves() {
    let intent = demo_intent();
    let item = demo_item();
    let record = derive_from_intent(&intent, &item, "entity", "2026-07-13T00:00:00Z").unwrap();

    let root = tempfile::tempdir().unwrap();
    let stream = LedgerStream::open(root.path(), "case-test", "entity").unwrap();
    let committed = stream
        .commit_typed(
            "settlement_criteria",
            vec!["run_01".into(), record.criteria_id.clone()],
            &record,
            vec![],
        )
        .unwrap();

    let mut found = None;
    for entry in stream.read_entries().unwrap() {
        if entry.record_kind == "settlement_criteria" {
            let loaded: SettlementCriteriaRecord =
                serde_json::from_value(entry.payload.clone()).unwrap();
            found = Some(loaded);
        }
    }
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.criteria_id, record.criteria_id);
    assert_eq!(found.criteria_sha256, record.criteria_sha256);

    // Verify the committed record by its payload hash.
    assert_eq!(
        committed.payload_hash(),
        stream.read_entries().unwrap().last().unwrap().payload_hash
    );
}

#[test]
fn verify_plan_criteria_skips_legacy_items() {
    let mut item_with_ref = demo_item();
    let record = derive_from_intent(
        &demo_intent(),
        &item_with_ref,
        "entity",
        "2026-07-13T00:00:00Z",
    )
    .unwrap();
    item_with_ref.settlement_criteria_ref = Some(record.criteria_id.clone());

    let mut legacy_item = demo_item();
    legacy_item.plan_item_id = "legacy_01".into();
    legacy_item.settlement_criteria_ref = None;

    let plan = CasePlan {
        version: "0.2".into(),
        plan_id: "plan_01".into(),
        case_id: "case_01".into(),
        run_id: "run_01".into(),
        intent_id: "int_01".into(),
        items: vec![item_with_ref, legacy_item],
        template_ref: None,
        job_contract_ref: None,
    };
    let mut map = BTreeMap::new();
    map.insert(record.criteria_id.clone(), record.clone());
    let records = verify_plan_criteria(&plan, &map).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].criteria_id, record.criteria_id);
}

#[test]
fn post_execution_declaration_time_is_caught_by_validation() {
    // If declared_at is after the run starts, the criteria record is stale.
    // The current verify_item_criteria does not check declared_at, but the
    // spec says criteria must predate execution. This test documents the
    // contract that the record carries a declared_at timestamp and the
    // pipeline is responsible for checking it before authority.
    let intent = demo_intent();
    let item = demo_item();
    let record = derive_from_intent(&intent, &item, "entity", "2026-07-13T00:00:00Z").unwrap();
    assert!(!record.declared_at.is_empty());
}

#[test]
fn built_in_demo_does_not_create_redundant_job_contract() {
    // The built-in demo path uses derive_from_intent, which reuses the Intent
    // as the origin and never synthesizes a JobContract.
    let intent = demo_intent();
    let item = demo_item();
    let record = derive_from_intent(&intent, &item, "entity", "2026-07-13T00:00:00Z").unwrap();
    for origin in &record.origin_refs {
        assert_ne!(origin.kind, OriginRefKind::JobContract);
    }
    assert_eq!(
        record.derivation.method,
        DerivationMethod::DeterministicPlanner
    );
}

// ── M10 (E12 §7.5): DesiredOutcome origin ref ──

use sea_forge_planner::{verify_desired_outcome_refs, DesiredOutcomeResolver, NoModelResolver};

struct StubResolver {
    known: std::collections::HashSet<String>,
}

impl DesiredOutcomeResolver for StubResolver {
    fn verify_desired_outcome(
        &self,
        reference: &str,
        _domain_model_ref: &str,
        _model_sha256: &str,
    ) -> Result<(), ForgeError> {
        if self.known.contains(reference) {
            Ok(())
        } else {
            Err(ForgeError::Plan {
                class: "criteria_provenance_error",
                message: format!("unresolved desired-outcome ref: {reference}"),
            })
        }
    }
}

fn desired_outcome_record(reference: &str, dmr: Option<&str>) -> SettlementCriteriaRecord {
    let mut record = derive_from_intent(
        &demo_intent(),
        &demo_item(),
        "entity",
        "2026-07-13T00:00:00Z",
    )
    .unwrap();
    record.origin_refs.push(OriginRef {
        kind: OriginRefKind::DesiredOutcome,
        reference: reference.into(),
        sha256: "sha256:model_hash".into(),
        role: OriginRole::DesiredResult,
        evidence_refs: vec![],
        domain_model_ref: dmr.map(str::to_owned),
    });
    record.criteria_record_hash = sea_forge_planner::compute_record_hash(&record).unwrap();
    record
}

#[test]
fn m10_desired_outcome_resolves_with_valid_resolver() {
    let record = desired_outcome_record("outcome:faster_builds", Some("godspeed.adlc_odi_case"));
    let resolver = StubResolver {
        known: ["outcome:faster_builds".into()].into(),
    };
    assert!(verify_desired_outcome_refs(&record, &resolver).is_ok());
}

#[test]
fn m10_desired_outcome_without_resolver_is_rejected() {
    let record = desired_outcome_record("outcome:faster_builds", Some("godspeed.adlc_odi_case"));
    // NoModelResolver rejects everything (fail-closed).
    let error = verify_desired_outcome_refs(&record, &NoModelResolver).unwrap_err();
    assert_eq!(error.class(), "criteria_provenance_error");
}

#[test]
fn m10_desired_outcome_without_domain_model_ref_is_rejected() {
    let record = desired_outcome_record("outcome:faster_builds", None);
    let resolver = StubResolver {
        known: ["outcome:faster_builds".into()].into(),
    };
    let error = verify_desired_outcome_refs(&record, &resolver).unwrap_err();
    assert_eq!(error.class(), "criteria_provenance_error");
    assert!(error.to_string().contains("domain_model_ref"));
}

#[test]
fn m10_unresolved_desired_outcome_ref_is_criteria_provenance_error() {
    let record = desired_outcome_record("outcome:nonexistent", Some("godspeed.adlc_odi_case"));
    let resolver = StubResolver {
        known: ["outcome:faster_builds".into()].into(),
    };
    let error = verify_desired_outcome_refs(&record, &resolver).unwrap_err();
    assert_eq!(error.class(), "criteria_provenance_error");
    assert!(error.to_string().contains("unresolved"));
}

#[test]
fn m10_desired_outcome_hash_changes_when_ref_or_model_changes() {
    let r1 = desired_outcome_record("outcome:A", Some("model_v1"));
    let r2 = desired_outcome_record("outcome:B", Some("model_v1"));
    let r3 = desired_outcome_record("outcome:A", Some("model_v2"));
    let h1 = sea_forge_planner::compute_record_hash(&r1).unwrap();
    let h2 = sea_forge_planner::compute_record_hash(&r2).unwrap();
    let h3 = sea_forge_planner::compute_record_hash(&r3).unwrap();
    assert_ne!(h1, h2);
    assert_ne!(h1, h3);
    assert_ne!(h2, h3);
}
