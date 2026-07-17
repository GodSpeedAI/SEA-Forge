use sea_forge_core::types::{ItemKind, ItemMarkers, SettlementCriteria};
use sea_forge_planner::templates::{
    instantiate, load, load_pinned, ParameterDef, PlanTemplate, TemplateItem, TemplateOperation,
    TemplatePlan,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("sea-forge-template-{name}-{nonce}"));
    fs::create_dir_all(&root).unwrap();
    root
}

fn template() -> PlanTemplate {
    PlanTemplate {
        name: "demo".into(),
        version: "1.0.0".into(),
        description: "deterministic template".into(),
        parameters: BTreeMap::from([
            (
                "output".into(),
                ParameterDef {
                    param_type: "path".into(),
                    required: true,
                    default: None,
                },
            ),
            (
                "message".into(),
                ParameterDef {
                    param_type: "string".into(),
                    required: false,
                    default: Some("hello".into()),
                },
            ),
        ]),
        origin_refs: vec![],
        job_contract: None,
        plan: TemplatePlan {
            items: vec![TemplateItem {
                plan_item_id: "item_01".into(),
                name: "write_message".into(),
                operations: vec![TemplateOperation::WriteFile {
                    path: "${output}".into(),
                    content_hint: "${message}".into(),
                }],
                settlement_criteria: SettlementCriteria {
                    require_exit_zero: false,
                    required_artifacts: vec!["${output}".into()],
                    stdout_must_contain: Some("${message}".into()),
                    ..Default::default()
                },
                item_kind: ItemKind::SandboxedTask,
                sandbox_class: Some("local".into()),
                markers: ItemMarkers::default(),
                max_instances: 1,
                environment: None,
                entry_criteria: vec![],
                exit_criteria: vec![],
                parent_stage: None,
                depends_on: vec![],
            }],
        },
    }
}

#[test]
fn template_instantiation_is_byte_deterministic_and_records_provenance() {
    let params = BTreeMap::from([
        ("output".into(), "result.txt".into()),
        ("message".into(), "stable".into()),
    ]);
    let first = instantiate(&template(), &params, "case_01", "run_01", "int_01").unwrap();
    let second = instantiate(&template(), &params, "case_01", "run_01", "int_01").unwrap();
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );
    assert_eq!(first.template_ref.as_deref(), Some("demo@1.0.0"));
    assert_eq!(
        first.items[0].settlement_criteria.required_artifacts,
        ["result.txt"]
    );
}

#[test]
fn template_missing_required_param_is_input_error() {
    let error =
        instantiate(&template(), &BTreeMap::new(), "case_01", "run_01", "int_01").unwrap_err();
    assert_eq!(error.class(), "input_error");
}

#[test]
fn template_forbidden_argv0_substitution_is_schema_error_at_load() {
    let root = temp_root("argv0");
    let path = root.join("bad@1.0.0.yaml");
    fs::write(
        &path,
        r#"
name: bad
version: 1.0.0
parameters:
  command: { type: string, required: true }
plan:
  items:
    - plan_item_id: item_01
      name: bad
      operations:
        - kind: execute_command
          argv: ["${command}"]
          cwd: "."
      settlement_criteria:
        require_exit_zero: true
        required_artifacts: []
        stdout_must_contain: null
        require_approval: false
"#,
    )
    .unwrap();
    let error = load(&path).unwrap_err();
    assert_eq!(error.class(), "schema_error");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn template_pin_rejects_same_version_with_changed_bytes() {
    let root = temp_root("pin");
    let templates = root.join("templates");
    fs::create_dir_all(&templates).unwrap();
    let path = templates.join("demo@1.0.0.yaml");
    fs::write(&path, serde_yaml::to_string(&template()).unwrap()).unwrap();
    load_pinned(&root, "demo@1.0.0").unwrap();
    fs::write(
        &path,
        format!(
            "{}\n# changed\n",
            serde_yaml::to_string(&template()).unwrap()
        ),
    )
    .unwrap();
    let error = load_pinned(&root, "demo@1.0.0").unwrap_err();
    assert_eq!(error.class(), "template_changed_error");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn template_path_param_rejects_parent_escape() {
    let params = BTreeMap::from([("output".into(), "../outside".into())]);
    let error = instantiate(&template(), &params, "case_01", "run_01", "int_01").unwrap_err();
    assert_eq!(error.class(), "input_error");
}

// ── M10 (E12): control-flow vocabulary projection ──

fn control_flow_template() -> PlanTemplate {
    use sea_forge_core::types::{Sentry, SentryPredicate, SentryTrigger};
    PlanTemplate {
        name: "flow".into(),
        version: "0.1.0".into(),
        description: "stages + sentries + deps".into(),
        parameters: BTreeMap::new(),
        origin_refs: vec![],
        job_contract: None,
        plan: TemplatePlan {
            items: vec![
                TemplateItem {
                    plan_item_id: "stage_frame".into(),
                    name: "Frame".into(),
                    operations: vec![],
                    settlement_criteria: SettlementCriteria::default(),
                    item_kind: ItemKind::Stage,
                    sandbox_class: None,
                    markers: ItemMarkers::default(),
                    max_instances: 1,
                    environment: None,
                    entry_criteria: vec![],
                    exit_criteria: vec![],
                    parent_stage: None,
                    depends_on: vec![],
                },
                TemplateItem {
                    plan_item_id: "item_design".into(),
                    name: "Design".into(),
                    operations: vec![],
                    settlement_criteria: SettlementCriteria::default(),
                    item_kind: ItemKind::SandboxedTask,
                    sandbox_class: Some("local".into()),
                    markers: ItemMarkers {
                        repetition: true,
                        ..Default::default()
                    },
                    max_instances: 3,
                    environment: None,
                    entry_criteria: vec![],
                    exit_criteria: vec![],
                    parent_stage: Some("stage_frame".into()),
                    depends_on: vec![],
                },
                TemplateItem {
                    plan_item_id: "item_sim".into(),
                    name: "Simulation".into(),
                    operations: vec![],
                    settlement_criteria: SettlementCriteria::default(),
                    item_kind: ItemKind::SandboxedTask,
                    sandbox_class: Some("local".into()),
                    markers: ItemMarkers::default(),
                    max_instances: 1,
                    environment: None,
                    // Reactivation sentry: design re-enters when sim is rejected.
                    entry_criteria: vec![Sentry {
                        on: SentryTrigger {
                            source: "item_sim".into(),
                            event: "settlement_status".into(),
                        },
                        if_predicate: Some(SentryPredicate::SettlementStatus {
                            status: "rejected".into(),
                        }),
                    }],
                    exit_criteria: vec![],
                    parent_stage: Some("stage_frame".into()),
                    depends_on: vec![],
                },
            ],
        },
    }
}

#[test]
fn control_flow_fields_project_byte_for_byte() {
    let tmpl = control_flow_template();
    let plan = instantiate(&tmpl, &BTreeMap::new(), "c1", "r1", "i1").unwrap();
    // Stage item
    assert_eq!(plan.items[0].item_kind, ItemKind::Stage);
    assert!(plan.items[0].parent_stage.is_none());
    // Design item: repetition + parent_stage
    assert_eq!(plan.items[1].parent_stage.as_deref(), Some("stage_frame"));
    assert!(plan.items[1].markers.repetition);
    assert_eq!(plan.items[1].max_instances, 3);
    // Simulation item: source-bound sentry projected
    assert_eq!(plan.items[2].entry_criteria.len(), 1);
    let sentry = &plan.items[2].entry_criteria[0];
    assert_eq!(sentry.on.source, "item_sim");
    assert_eq!(sentry.on.event, "settlement_status");
    // Byte-deterministic: same template + params ⇒ identical plan
    let plan2 = instantiate(&tmpl, &BTreeMap::new(), "c1", "r1", "i1").unwrap();
    assert_eq!(
        serde_json::to_vec(&plan).unwrap(),
        serde_json::to_vec(&plan2).unwrap()
    );
}

#[test]
fn old_template_without_control_flow_fields_still_deserializes() {
    let yaml = r#"
name: legacy
version: "0.1.0"
description: no control-flow fields
plan:
  items:
    - plan_item_id: item_01
      name: lone
      settlement_criteria:
        require_exit_zero: true
        required_artifacts: []
        stdout_must_contain: null
        require_approval: false
"#;
    let root = temp_root("legacy");
    let path = root.join("legacy@0.1.0.yaml");
    fs::write(&path, yaml).unwrap();
    let tmpl = load(&path).unwrap();
    let plan = instantiate(&tmpl, &BTreeMap::new(), "c", "r", "i").unwrap();
    assert!(plan.items[0].entry_criteria.is_empty());
    assert!(plan.items[0].exit_criteria.is_empty());
    assert!(plan.items[0].parent_stage.is_none());
    assert!(plan.items[0].depends_on.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sentry_source_param_substitution_is_forbidden() {
    let yaml = r#"
name: bad_sentry
version: "0.1.0"
parameters:
  src: { type: string, required: true }
plan:
  items:
    - plan_item_id: item_01
      name: one
      entry_criteria:
        - on: { source: "${src}", event: settlement_status }
      settlement_criteria:
        require_exit_zero: false
        required_artifacts: []
        stdout_must_contain: null
        require_approval: false
    - plan_item_id: ${src}
      name: src_placeholder
      settlement_criteria:
        require_exit_zero: false
        required_artifacts: []
        stdout_must_contain: null
        require_approval: false
"#;
    let root = temp_root("sentry_param");
    let path = root.join("bad_sentry@0.1.0.yaml");
    fs::write(&path, yaml).unwrap();
    let error = load(&path).unwrap_err();
    assert_eq!(error.class(), "schema_error");
    fs::remove_dir_all(root).unwrap();
}
