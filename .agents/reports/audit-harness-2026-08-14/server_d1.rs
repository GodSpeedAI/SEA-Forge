use sea_forge_core::types::{
    CasePlan, ItemKind, ItemMarkers, Operation, PlanItem, SettlementCriteria,
};
use sea_forge_server::{handle_request, Request, ServerConfig, ServerState};
use std::fs;
use std::path::Path;
use std::sync::Arc;

fn policy(root: &Path) -> std::path::PathBuf {
    let path = root.join("allow.yaml");
    fs::write(
        &path,
        "version: \"0.1\"\nrules:\n  - name: allow-command\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n",
    )
    .unwrap();
    path
}

fn trusted_self_executable(dir: &Path) -> std::path::PathBuf {
    let link = dir.join("sea-forge");
    std::os::unix::fs::symlink(std::env::current_exe().unwrap(), &link).unwrap();
    link
}

fn mixed_plan(argv: Vec<String>) -> CasePlan {
    let task = PlanItem {
        plan_item_id: "task_a".into(),
        name: "sandboxed".into(),
        operations: vec![Operation::ExecuteCommand { argv, cwd: ".".into() }],
        entry_criteria: vec![],
        entry_criteria_mode: Default::default(),
        exit_criteria: vec![],
        settlement_criteria: SettlementCriteria::default(),
        settlement_criteria_ref: None,
        item_kind: ItemKind::SandboxedTask,
        sandbox_class: None,
        parent_stage: None,
        markers: ItemMarkers { required: true, ..Default::default() },
        max_instances: 1,
        depends_on: vec![],
        environment: None,
        proposed_by: None,
    };
    let stage = PlanItem {
        plan_item_id: "stage_b".into(),
        name: "stage".into(),
        operations: vec![],
        entry_criteria: vec![],
        entry_criteria_mode: Default::default(),
        exit_criteria: vec![],
        settlement_criteria: SettlementCriteria::default(),
        settlement_criteria_ref: None,
        item_kind: ItemKind::Stage,
        sandbox_class: None,
        parent_stage: None,
        markers: ItemMarkers::default(), // no manual_activation
        max_instances: 1,
        depends_on: vec![],
        environment: None,
        proposed_by: None,
    };
    CasePlan {
        version: "0.2".into(),
        plan_id: "plan_mixed".into(),
        case_id: "case_placeholder".into(),
        run_id: "run_placeholder".into(),
        intent_id: "int_mixed".into(),
        items: vec![task, stage],
        template_ref: None,
        job_contract_ref: None,
    }
}

#[tokio::test]
async fn escalated_required_item_terminates_case_with_orphaned_approval() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("escalate.yaml");
    fs::write(
        &path,
        "version: \"0.1\"\nrules:\n  - name: escalate-command\n    verdict: escalate\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n",
    )
    .unwrap();
    let mut argv = vec![trusted_self_executable(root.path()).to_string_lossy().into_owned()];
    argv.extend(["--exact", "__no_test_matches_this_name__"].iter().map(|s| s.to_string()));
    let plan_path = root.path().join("plan.json");
    let mut plan = mixed_plan(argv);
    plan.items.truncate(1); // single required SandboxedTask under an escalate policy
    fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();

    let state = Arc::new(
        ServerState::new(ServerConfig {
            socket_path: root.path().join("unused.sock"),
            root: root.path().to_path_buf(),
            ..ServerConfig::default()
        })
        .unwrap(),
    );
    let request: Request = serde_json::from_value(serde_json::json!({
        "verb": "submit",
        "plan": plan_path,
        "policy": path,
        "entity": "operator_local",
        "process": "test",
        "timeout": 60,
    }))
    .unwrap();
    let response = handle_request(request, &state).await;
    println!("submit response: {response}");
    let case_id = response["case_id"].as_str().unwrap_or("").to_owned();

    let ledger = sea_forge_ledger::LedgerStream::open(
        root.path(),
        format!("case-{case_id}"),
        "audit",
    )
    .unwrap();
    let event_kinds: Vec<String> = ledger
        .read_entries()
        .unwrap()
        .iter()
        .filter(|e| e.record_kind == "case_event")
        .map(|e| e.payload["kind"].as_str().unwrap_or_default().to_string())
        .collect();
    println!("case event kinds: {event_kinds:?}");
    let approvals: Vec<serde_json::Value> = ledger
        .read_entries()
        .unwrap()
        .into_iter()
        .filter(|e| e.record_kind == "approval_request")
        .map(|e| e.payload.clone())
        .collect();
    println!("approval requests: {}", approvals.len());
    if let Some(a) = approvals.first() {
        println!("approval status: {}", a["status"]);
    }
    let case_dir = root.path().join("cases").join(&case_id);
    let case_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(case_dir.join("case.json")).unwrap()).unwrap();
    println!("case.json state: {}", case_json["state"]);

    assert_eq!(event_kinds.last().map(String::as_str), Some("case_terminated"),
        "case must have terminated");
    assert!(!approvals.is_empty(), "escalation opened an approval");
    assert_eq!(approvals[0]["status"], "pending", "approval still pending while case terminated");
}

#[tokio::test]
async fn mid_dispatch_error_aborts_spawned_episode_and_loses_settlement() {
    let root = tempfile::tempdir().unwrap();
    let policy_path = policy(root.path());
    let mut argv = vec![trusted_self_executable(root.path()).to_string_lossy().into_owned()];
    argv.extend(["--exact", "__no_test_matches_this_name__"].iter().map(|s| s.to_string()));

    let plan_path = root.path().join("plan.json");
    fs::write(&plan_path, serde_json::to_vec(&mixed_plan(argv)).unwrap()).unwrap();

    let state = Arc::new(
        ServerState::new(ServerConfig {
            socket_path: root.path().join("unused.sock"),
            root: root.path().to_path_buf(),
            ..ServerConfig::default()
        })
        .unwrap(),
    );
    let request: Request = serde_json::from_value(serde_json::json!({
        "verb": "submit",
        "plan": plan_path,
        "policy": policy_path,
        "entity": "operator_local",
        "process": "test",
        "timeout": 60,
    }))
    .unwrap();
    let response = handle_request(request, &state).await;
    println!("submit response: {response}");

    // Locate the case directory (submit may or may not have returned a case_id).
    let cases_dir = root.path().join("cases");
    let case_dir = fs::read_dir(&cases_dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .find(|p| p.is_dir())
        .expect("case dir must exist after submit");
    println!("case dir: {}", case_dir.display());

    let ledger = sea_forge_ledger::LedgerStream::open(
        root.path(),
        format!("case-{}", case_dir.file_name().unwrap().to_string_lossy()),
        "audit",
    )
    .unwrap();
    let entries = ledger.read_entries().unwrap();
    let event_kinds: Vec<String> = entries
        .iter()
        .filter(|e| e.record_kind == "case_event")
        .map(|e| e.payload["kind"].as_str().unwrap_or_default().to_string())
        .collect();
    println!("case event kinds: {event_kinds:?}");
    let activated = event_kinds.iter().filter(|k| k.as_str() == "item_activated").count();
    let settled = event_kinds.iter().filter(|k| k.as_str() == "settlement_recorded").count();
    println!("item_activated={activated} settlement_recorded={settled}");

    // trace events from run dirs
    let runs = case_dir.join("runs");
    if let Ok(rd) = fs::read_dir(&runs) {
        for run_dir in rd.flatten().map(|e| e.path()) {
            if let Ok(text) = fs::read_to_string(run_dir.join("trace.jsonl")) {
                let kinds: Vec<String> = text
                    .lines()
                    .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
                    .map(|v| v["kind"].as_str().unwrap_or_default().to_owned())
                    .collect();
                println!("run {} trace kinds: {:?}", run_dir.display(), kinds);
            }
        }
    }

    assert!(
        activated >= 1,
        "task_a must have been activated (episode spawned) before the error"
    );
    let runs_dir = case_dir.join("runs");
    let run_count = fs::read_dir(&runs_dir).map(|d| d.count()).unwrap_or(0);
    println!("run dirs created: {run_count}");
    let case_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(case_dir.join("case.json")).unwrap()).unwrap();
    println!("case.json state: {}", case_json["state"]);

    assert!(
        activated >= 1,
        "task_a must have been activated (episode spawned) before the error"
    );
    assert_eq!(
        settled, 0,
        "VERIFYING DEFECT: spawned episode's settlement was never recorded (JoinSet dropped mid-dispatch)"
    );
    println!("(episode aborted before creating its run dir — activation record already durable)");
}
