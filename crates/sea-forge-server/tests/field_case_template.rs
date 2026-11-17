//! WP-21 part B: the template `fde.field_case@1` and the Field Case document
//! table are checked against the declared model and the frozen schema, so a
//! drift fails here (E section 3: "a test fails if its operations or gates
//! differ from the model's").

use sea_forge_core::types::ItemKind;
use sea_forge_planner::case_engine::{next_case_actions, validate_proposal, CaseAction};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn repo(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

/// The `gate_id` of every `EvidenceGate` instance of the model, in file order.
fn model_gates() -> Vec<String> {
    let model = fs::read_to_string(repo(".sea/interaction/fde-sop/fde-sop.sea")).unwrap();
    model
        .lines()
        .filter_map(|l| l.trim().strip_prefix("gate_id: \""))
        .map(|rest| rest.trim_end_matches(['"', ',']).to_string())
        .collect()
}

fn schema(name: &str) -> Value {
    serde_json::from_str(&fs::read_to_string(repo("contracts/fde").join(name)).unwrap()).unwrap()
}

#[test]
fn template_gates_are_the_models_gates() {
    let model = model_gates();
    assert_eq!(model.len(), 9, "the model declares nine gates: {model:?}");
    let template = sea_forge_planner::fde_field_case_template();
    let gates: Vec<String> = template
        .plan
        .items
        .iter()
        .filter(|i| i.item_kind == ItemKind::Milestone)
        .map(|i| i.plan_item_id.strip_prefix("gate_").unwrap().to_string())
        .collect();
    assert_eq!(
        gates, model,
        "template gate milestones differ from the model"
    );
    // The same nine, in the same order, everywhere the contracts and the server name them.
    assert_eq!(sea_forge_planner::FDE_GATES.to_vec(), model);
    assert_eq!(
        sea_forge_server::sfwp::field_continuation::GATES.to_vec(),
        model
    );
    let v2 = schema("case-record-append.v2.json");
    let enum_of = |v: &Value| -> Vec<String> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(
        enum_of(&v2["$defs"]["GateAssessedPayload"]["properties"]["gate"]["enum"]),
        model
    );
    let cont = schema("continuation.v1.json");
    assert_eq!(
        enum_of(
            &cont["$defs"]["Continuation"]["properties"]["gate_evidence"]["items"]["properties"]
                ["gate"]["enum"]
        ),
        model
    );
    // Hard gates are in the model's gate set.
    for hard in sea_forge_server::sfwp::field_case_kinds::HARD_GATES {
        assert!(model.iter().any(|g| g == hard), "{hard}");
    }
}

#[test]
fn template_operations_are_the_models_operations() {
    // A gate assessment and a disposition are records (`assess_gate`,
    // `record_disposition` in the model), never plan item operations: the
    // template carries none, and the model names both.
    let model = fs::read_to_string(repo(".sea/interaction/fde-sop/fde-sop.sea")).unwrap();
    for op in ["operation assess_gate", "operation record_disposition"] {
        assert!(model.contains(op), "the model no longer declares `{op}`");
    }
    let template = sea_forge_planner::fde_field_case_template();
    assert!(template.plan.items.iter().all(|i| i.operations.is_empty()));
    assert!(template.plan.repeated.is_empty());
}

#[test]
fn template_instantiates_and_nothing_advances_by_itself() {
    let template = sea_forge_planner::fde_field_case_template();
    assert_eq!(
        (template.name.as_str(), template.version.as_str()),
        ("fde.field_case", "1")
    );
    assert!(sea_forge_planner::is_built_in_template_ref(
        "fde.field_case@1"
    ));
    let mut plan =
        sea_forge_planner::instantiate(&template, &BTreeMap::new(), "case-0001", "run-1", "int-1")
            .unwrap();
    validate_proposal(&mut plan).unwrap();
    assert_eq!(plan.items.len(), 10);
    // Activities repeat (the repetition marker), may be omitted and run in any
    // order (no entry criteria, no dependency, activated by hand).
    let activity = plan
        .items
        .iter()
        .find(|i| i.plan_item_id == "activity")
        .unwrap();
    assert!(activity.markers.repetition && activity.markers.manual_activation);
    assert!(activity.max_instances > 1);
    assert!(plan
        .items
        .iter()
        .all(|i| i.entry_criteria.is_empty() && i.depends_on.is_empty()));
    // Activities may be omitted; a gate is required (the plan cannot complete by
    // itself while one is unresolved).
    assert!(!activity.markers.required);
    assert!(plan
        .items
        .iter()
        .filter(|i| i.item_kind == ItemKind::Milestone)
        .all(|i| i.markers.required));
    // Nothing advances automatically: the only actions open to the engine are
    // the manual enablements.
    let actions = next_case_actions(&plan.items, &[]);
    assert_eq!(actions.len(), 10, "{actions:?}");
    assert!(
        actions.iter().all(|a| matches!(a, CaseAction::Enable(_))),
        "{actions:?}"
    );
}

#[test]
fn template_is_stored_and_loaded_pinned() {
    let dir = tempfile::tempdir().unwrap();
    sea_forge_planner::store_builtin(
        dir.path(),
        "model:demo",
        &"0".repeat(64),
        sea_forge_planner::DEFAULT_TOPOLOGY_ENDPOINT_REF,
    )
    .unwrap_or_else(|e| panic!("{e:?}"));
    let loaded = sea_forge_planner::load_pinned(dir.path(), "fde.field_case@1").unwrap();
    assert_eq!(loaded, sea_forge_planner::fde_field_case_template());
    assert!(dir.path().join("templates/fde.field_case@1.yaml").exists());
    // A name cannot step out of templates/.
    for bad in ["..@1", ".x@1", "x..y@1", "x.@1", "a/b@1"] {
        assert!(
            sea_forge_planner::templates::parse_template_ref(bad).is_err(),
            "{bad}"
        );
    }
}

#[test]
fn document_table_is_the_frozen_schema() {
    let schema = schema("field-case.v1.json");
    let groups = schema["properties"]["fields"]["properties"]
        .as_object()
        .unwrap();
    let want: Vec<(String, Vec<String>)> = groups
        .iter()
        .map(|(g, v)| {
            (
                g.clone(),
                v["required"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_str().unwrap().to_string())
                    .collect(),
            )
        })
        .collect();
    let have: Vec<(String, Vec<String>)> = sea_forge_core::field_case::GROUPS
        .iter()
        .map(|(g, f)| {
            (
                (*g).to_string(),
                f.iter().map(|x| (*x).to_string()).collect(),
            )
        })
        .collect();
    let mut have = have;
    let mut want = want;
    have.sort();
    want.sort();
    assert_eq!(have, want);
    assert_eq!(have.iter().map(|(_, f)| f.len()).sum::<usize>(), 45);
    let doc = sea_forge_core::field_case::opened_document("case-0001");
    sea_forge_server::sfwp::field_case_kinds::document_is_valid(&doc).unwrap();
    assert_eq!(doc["sop"]["sha256"], sea_forge_core::field_case::SOP_SHA256);
    // The registry's read-only paths are fields of the table.
    let registry = schema_registry();
    let reserved: Vec<String> = registry["reserved_paths_after_freeze"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    let mut have: Vec<String> = sea_forge_server::sfwp::field_case_kinds::RESERVED_PATHS
        .iter()
        .map(|s| s.to_string())
        .collect();
    have.sort();
    let mut want = reserved;
    want.sort();
    assert_eq!(have, want);
}

fn schema_registry() -> Value {
    serde_json::from_str(
        &fs::read_to_string(repo(
            "contracts/fde/fixtures/case-record-kinds/valid/registry.json",
        ))
        .unwrap(),
    )
    .unwrap()
}
