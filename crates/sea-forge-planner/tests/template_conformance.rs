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
