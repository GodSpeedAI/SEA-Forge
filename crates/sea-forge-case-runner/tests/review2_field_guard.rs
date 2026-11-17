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
        sea_forge_core::field_case::opened_document(CASE).to_string(),
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

/// A Field Case of `items`; `done` completes `task-a` first.
fn seed_items(root: &Path, items: Vec<PlanItem>, done: bool) {
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
        items,
        template_ref: Some("fde.field_case@1".into()),
        job_contract_ref: None,
    };
    fs::write(dir.join("case.json"), serde_json::to_vec(&case).unwrap()).unwrap();
    fs::write(dir.join("plan.json"), serde_json::to_vec(&plan).unwrap()).unwrap();
    fs::write(dir.join("case-events.jsonl"), b"").unwrap();
    // README section 6: every write on a Field Case rests on its document.
    fs::write(
        dir.join("field_case.json"),
        sea_forge_core::field_case::opened_document(CASE).to_string(),
    )
    .unwrap();
    if done {
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
    // A plain case has no document (a Field Case marker).
    fs::remove_file(dir.path().join("cases").join(CASE).join("field_case.json")).unwrap();
    let plan_path = dir.path().join("cases").join(CASE).join("plan.json");
    let mut plan: serde_json::Value =
        serde_json::from_slice(&fs::read(&plan_path).unwrap()).unwrap();
    plan["template_ref"] = json!(null);
    fs::write(&plan_path, plan.to_string()).unwrap();
    assert_eq!(pass(dir.path()).unwrap(), "completed");
}

// ---------------------------------------------------------------------------
// Review round 4: nothing is written before admission; corruption fails
// closed; a closed case takes only the finishing action; one transaction per
// verb.
// ---------------------------------------------------------------------------

fn policy(root: &Path, kinds: &[&str]) -> std::path::PathBuf {
    let rules: String = kinds
        .iter()
        .map(|k| {
            format!("  - name: allow-{k}\n    verdict: allow\n    actor_role: operator\n    operation_kind: {k}\n")
        })
        .collect();
    let path = root.join("policy.yaml");
    fs::write(&path, format!("version: \"0.1\"\nrules:\n{rules}")).unwrap();
    path
}

fn case_json(root: &Path) -> serde_json::Value {
    serde_json::from_slice(&fs::read(root.join("cases").join(CASE).join("case.json")).unwrap())
        .unwrap()
}

/// 1. A state write that ends the case, before its guarded event (CLI resume,
/// server submission), is refused: the helper every writer uses admits first.
#[test]
fn review4_case_json_is_not_written_ahead_of_admission() {
    let dir = tempfile::tempdir().unwrap();
    seed_items(
        dir.path(),
        vec![item("task-a", ItemKind::HumanTask, &[])],
        false,
    );
    let case_dir = dir.path().join("cases").join(CASE);
    let before = fs::read(case_dir.join("case.json")).unwrap();
    let mut terminated = case_json(dir.path());
    terminated["state"] = json!("terminated");
    let got = sea_forge_case_runner::write_json(&case_dir.join("case.json"), &terminated);
    assert!(
        got.is_err(),
        "case.json went terminated with no disposition"
    );
    assert!(got
        .unwrap_err()
        .to_string()
        .contains("CLOSE_DISPOSITION_REQUIRED"));
    assert_eq!(fs::read(case_dir.join("case.json")).unwrap(), before);
    // A submission over the Field Case rewrites neither its plan nor its case.
    let plan_before = fs::read(case_dir.join("plan.json")).unwrap();
    let got = sea_forge_case_runner::write_json(&case_dir.join("plan.json"), &json!({}));
    assert!(got.is_err());
    assert_eq!(fs::read(case_dir.join("plan.json")).unwrap(), plan_before);
    // A write under the run directory of a suspended case is refused too.
    records(
        dir.path(),
        &[json!({"kind": "disposition.recorded", "level": "case", "disposition": "suspend"})],
    );
    let got = sea_forge_case_runner::write_json(
        &case_dir.join("runs").join("run-1").join("settlement.json"),
        &json!({}),
    );
    assert!(got.is_err());
    assert!(!case_dir.join("runs").exists());
}

/// 2. Corruption fails closed whatever the plan says.
#[test]
fn review4_corrupt_stores_fail_closed() {
    for what in ["plan", "aux", "document", "document-schema"] {
        let dir = tempfile::tempdir().unwrap();
        seed_items(
            dir.path(),
            vec![item("task-a", ItemKind::HumanTask, &[])],
            false,
        );
        let case_dir = dir.path().join("cases").join(CASE);
        match what {
            "plan" => fs::write(case_dir.join("plan.json"), "{not json").unwrap(),
            "aux" => fs::write(case_dir.join("field-runs.jsonl"), "{torn\n").unwrap(),
            "document" => fs::write(case_dir.join("field_case.json"), "[]").unwrap(),
            _ => {
                let mut d = sea_forge_core::field_case::opened_document(CASE);
                d["fields"]["framing"] = json!({});
                fs::write(case_dir.join("field_case.json"), d.to_string()).unwrap();
            }
        }
        let got = case_ops::append_case_event(
            dir.path(),
            CASE,
            "operator_local",
            TraceKind::ItemEnabled,
            Some("task-a"),
            json!({}),
        );
        assert!(
            got.as_ref()
                .is_err_and(|e| e.to_string().contains("STORE_INTEGRITY")),
            "{what}: {got:?}"
        );
        assert!(
            event_kinds(dir.path()).is_empty(),
            "{what}: nothing written"
        );
    }
}

/// 3. After a close disposition only an action that FINISHES the plan is
/// admitted: progress that leaves the plan unfinished is refused.
#[test]
fn review4_closed_case_admits_only_the_finishing_action() {
    let dir = tempfile::tempdir().unwrap();
    seed_items(
        dir.path(),
        vec![
            item("task-a", ItemKind::HumanTask, &[]),
            item("task-b", ItemKind::HumanTask, &[]),
        ],
        false,
    );
    records(
        dir.path(),
        &[
            json!({"kind": "disposition.recorded", "level": "case", "disposition": "close-satisfied"}),
        ],
    );
    let write = |kind, item: &str| {
        case_ops::append_case_event(
            dir.path(),
            CASE,
            "operator_local",
            kind,
            Some(item),
            json!({}),
        )
    };
    // task-b still open: completing task-a does not finish the plan.
    let got = write(TraceKind::HumanTaskCompleted, "task-a");
    assert!(got.is_err_and(|e| e.to_string().contains("CASE_CLOSED")));
    assert!(event_kinds(dir.path()).is_empty());
    // Once task-a is done, task-b is the finishing action.
    case_ops::append_case_event(
        dir.path(),
        CASE,
        "operator_local",
        TraceKind::ItemTerminated,
        Some("task-a"),
        json!({}),
    )
    .expect_err("terminating task-a does not finish the plan either");
}

/// 4. One transaction per verb: the completion that would reach a close with
/// no disposition writes nothing, and a retry writes nothing either.
#[test]
fn review4_completion_that_reaches_a_close_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    seed_items(
        dir.path(),
        vec![item("task-a", ItemKind::HumanTask, &[])],
        false,
    );
    let policy = policy(dir.path(), &["human_task_completion"]);
    for _ in 0..2 {
        let got = case_ops::complete_human_task(
            dir.path(),
            &policy,
            "operator_local",
            CASE,
            "task-a",
            None,
        );
        assert!(
            got.as_ref()
                .is_err_and(|e| e.to_string().contains("CLOSE_DISPOSITION_REQUIRED")),
            "{got:?}"
        );
        assert!(
            event_kinds(dir.path()).is_empty(),
            "no completion left behind"
        );
    }
    // With the disposition the same verb completes and closes, once.
    records(
        dir.path(),
        &[
            json!({"kind": "disposition.recorded", "level": "case", "disposition": "close-satisfied"}),
        ],
    );
    assert_eq!(
        case_ops::complete_human_task(dir.path(), &policy, "operator_local", CASE, "task-a", None)
            .unwrap(),
        0
    );
    let kinds = event_kinds(dir.path());
    assert_eq!(kinds.len(), 2, "{kinds:?}");
    assert!(case_ops::complete_human_task(
        dir.path(),
        &policy,
        "operator_local",
        CASE,
        "task-a",
        None
    )
    .is_err());
    assert_eq!(event_kinds(dir.path()).len(), 2, "no repeat");
}
