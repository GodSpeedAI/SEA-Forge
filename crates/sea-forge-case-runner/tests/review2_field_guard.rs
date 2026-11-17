//! WP-21 part B, review round 2 (finding 1): the Field Case write rules hold
//! at the write, on every path that completes a gate item or closes the case,
//! including the advance engine's automatic milestone and close.

use sea_forge_case_runner::case_ops::{self, advance::AdvanceScope};
use sea_forge_core::types::*;
use sea_forge_core::RECORD_VERSION;
use serde_json::json;
use std::fs;
use std::path::Path;

const CASE: &str = "case-0001";

fn item(id: &str, kind: ItemKind, depends_on: &[&str]) -> PlanItem {
    PlanItem {
        external_ref: None,
        plan_item_id: id.into(),
        name: id.into(),
        operations: vec![],
        entry_criteria: vec![],
        entry_criteria_mode: EntryCriteriaMode::Any,
        exit_criteria: vec![],
        settlement_criteria: SettlementCriteria::default(),
        settlement_criteria_ref: None,
        item_kind: kind,
        sandbox_class: None,
        parent_stage: None,
        markers: ItemMarkers {
            required: true,
            ..Default::default()
        },
        max_instances: 1,
        depends_on: depends_on.iter().map(|s| s.to_string()).collect(),
        environment: None,
        proposed_by: None,
    }
}

/// A Field Case whose human task `task-a` is done and whose next item is an
/// automatic milestone `milestone`.
fn seed(root: &Path, milestone: &str) {
    let dir = root.join("cases").join(CASE);
    fs::create_dir_all(&dir).unwrap();
    let case = Case {
        version: RECORD_VERSION.into(),
        case_id: CASE.into(),
        intent: Intent {
            intent_id: "int_1".into(),
            summary: "fixture".into(),
            actor_id: "operator_runner".into(),
            process_id: "proc_1".into(),
            created_at: "2026-09-23T00:00:00Z".into(),
        },
        state: CaseState::Active,
        plan_ref: "plan.json".into(),
        run_ids: vec![],
        stages: vec![],
        close_reason: None,
        created_at: "2026-09-23T00:00:00Z".into(),
        closed_at: None,
    };
    let plan = CasePlan {
        version: "0.2".into(),
        plan_id: "plan_1".into(),
        case_id: CASE.into(),
        run_id: "run_1".into(),
        intent_id: "int_1".into(),
        items: vec![
            item("task-a", ItemKind::HumanTask, &[]),
            item(milestone, ItemKind::Milestone, &["task-a"]),
        ],
        template_ref: Some("fde.field_case@1".into()),
        job_contract_ref: None,
    };
    fs::write(dir.join("case.json"), serde_json::to_vec(&case).unwrap()).unwrap();
    fs::write(dir.join("plan.json"), serde_json::to_vec(&plan).unwrap()).unwrap();
    fs::write(dir.join("case-events.jsonl"), b"").unwrap();
    // README section 6: every write on a Field Case rests on its document.
    fs::write(
        dir.join("field_case.json"),
        json!({"case_id": CASE, "revision": 1}).to_string(),
    )
    .unwrap();
    case_ops::append_case_event(
        root,
        CASE,
        "operator_runner",
        TraceKind::HumanTaskCompleted,
        Some("task-a"),
        json!({}),
    )
    .unwrap();
}

/// The ledger the server's writer would leave: hash-chained (the guard
/// verifies the chain before it admits anything).
fn records(root: &Path, lines: &[serde_json::Value]) {
    let mut prev = "0".repeat(64);
    let mut text = String::new();
    for (i, line) in lines.iter().enumerate() {
        let mut record = line.clone();
        record["seq"] = json!(i as u64 + 1);
        record["prev_hash"] = json!(prev);
        record["hash"] = json!(case_ops::case_guard::digest_of(&record).unwrap());
        prev = record["hash"].as_str().unwrap().to_string();
        text.push_str(&format!("{record}\n"));
    }
    fs::write(
        root.join("cases").join(CASE).join("field-records.jsonl"),
        text,
    )
    .unwrap();
}

fn event_kinds(root: &Path) -> Vec<String> {
    fs::read_to_string(root.join("cases").join(CASE).join("case-events.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap()["kind"].to_string())
        .collect()
}

fn pass(root: &Path) -> Result<&'static str, sea_forge_core::errors::ForgeError> {
    let mut run =
        |_: &PlanItem, _: &str| -> Result<SettlementEvent, _> { unreachable!("no episode runs") };
    case_ops::advance::advance_case(
        root,
        "operator_runner",
        CASE,
        AdvanceScope::All,
        &mut |_| {},
        &mut run,
    )
    .map(|o| o.state)
}

/// An automatic gate milestone is no way around the gate: with no assessment
/// the engine pass leaves it unachieved, and the plan does not close.
#[test]
fn review2_engine_does_not_achieve_an_unassessed_gate() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), "gate_prove_value");
    let state = pass(dir.path()).expect("a blocked pass is not an error");
    let kinds = event_kinds(dir.path());
    assert!(
        !kinds.iter().any(|k| k.contains("milestone_achieved")),
        "{state}: {kinds:?}"
    );
    assert!(
        !kinds.iter().any(|k| k.contains("case_closed")),
        "{kinds:?}"
    );
    // A blocked assessment does not pass either.
    records(
        dir.path(),
        &[json!({"kind": "gate.assessed", "gate": "prove_value", "state": "blocked"})],
    );
    pass(dir.path()).unwrap();
    assert!(!event_kinds(dir.path())
        .iter()
        .any(|k| k.contains("milestone_achieved")));
}

/// R11 on the engine's own close: a finished plan does not close without a
/// case-level close disposition; with one it does.
#[test]
fn review2_engine_close_needs_a_close_disposition() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), "ms-plain");
    let refused = pass(dir.path()).expect_err("no close without a disposition");
    assert!(
        refused.to_string().contains("CLOSE_DISPOSITION_REQUIRED"),
        "{refused}"
    );
    assert!(!event_kinds(dir.path())
        .iter()
        .any(|k| k.contains("case_closed")));
    records(
        dir.path(),
        &[
            json!({"kind": "disposition.recorded", "level": "case", "disposition": "close-complete"}),
        ],
    );
    assert_eq!(pass(dir.path()).unwrap(), "completed");
    assert!(event_kinds(dir.path())
        .iter()
        .any(|k| k.contains("case_closed")));
}

/// The same rule for a plan that is not a Field Case: untouched.
#[test]
fn review2_a_plain_case_closes_as_before() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), "gate_prove_value");
    let plan_path = dir.path().join("cases").join(CASE).join("plan.json");
    let mut plan: serde_json::Value =
        serde_json::from_slice(&fs::read(&plan_path).unwrap()).unwrap();
    plan["template_ref"] = json!(null);
    fs::write(&plan_path, plan.to_string()).unwrap();
    assert_eq!(pass(dir.path()).unwrap(), "completed");
}
