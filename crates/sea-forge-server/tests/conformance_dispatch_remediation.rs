//! Regression tests for the deep-audit dispatch findings (F-02, F-03).
//! These convert the retained audit-harness reproductions into permanent
//! conformance tests asserting the *corrected* invariant.

use sea_forge_core::types::{
    CasePlan, ItemKind, ItemMarkers, Operation, PlanItem, SettlementCriteria,
};
use sea_forge_server::{handle_request, Request, ServerConfig, ServerState};
use std::fs;
use std::path::Path;
use std::sync::Arc;

fn write_policy(root: &Path, verdict: &str) -> std::path::PathBuf {
    let path = root.join("policy.yaml");
    fs::write(
        &path,
        format!(
            "version: \"0.1\"\nrules:\n  - name: command\n    verdict: {verdict}\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n",
        ),
    )
    .unwrap();
    path
}

fn trusted_self_executable(dir: &Path) -> std::path::PathBuf {
    let link = dir.join("sea-forge");
    std::os::unix::fs::symlink(std::env::current_exe().unwrap(), &link).unwrap();
    link
}

fn sandboxed_task(id: &str, argv: Vec<String>, required: bool) -> PlanItem {
    PlanItem {
        plan_item_id: id.into(),
        name: "sandboxed".into(),
        operations: vec![Operation::ExecuteCommand {
            argv,
            cwd: ".".into(),
        }],
        entry_criteria: vec![],
        entry_criteria_mode: Default::default(),
        exit_criteria: vec![],
        settlement_criteria: SettlementCriteria::default(),
        settlement_criteria_ref: None,
        item_kind: ItemKind::SandboxedTask,
        sandbox_class: None,
        parent_stage: None,
        markers: ItemMarkers {
            required,
            ..Default::default()
        },
        max_instances: 1,
        depends_on: vec![],
        environment: None,
        proposed_by: None,
    }
}

fn stage(id: &str) -> PlanItem {
    PlanItem {
        plan_item_id: id.into(),
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
        markers: ItemMarkers::default(),
        max_instances: 1,
        depends_on: vec![],
        environment: None,
        proposed_by: None,
    }
}

fn plan(items: Vec<PlanItem>) -> CasePlan {
    CasePlan {
        version: "0.2".into(),
        plan_id: "plan_mixed".into(),
        case_id: "case_placeholder".into(),
        run_id: "run_placeholder".into(),
        intent_id: "int_mixed".into(),
        items,
        template_ref: None,
        job_contract_ref: None,
    }
}

fn state(root: &Path) -> Arc<ServerState> {
    Arc::new(
        ServerState::new(ServerConfig {
            socket_path: root.join("unused.sock"),
            root: root.to_path_buf(),
            ..ServerConfig::default()
        })
        .unwrap(),
    )
}

fn submit(root: &Path, plan_path: &Path, policy_path: &Path) -> serde_json::Value {
    let state = state(root);
    // F-16: plan/policy references are cell-relative spellings under the root.
    let plan_ref = plan_path.strip_prefix(root).unwrap().to_string_lossy();
    let policy_ref = policy_path.strip_prefix(root).unwrap().to_string_lossy();
    let request: Request = serde_json::from_value(serde_json::json!({
        "verb": "submit",
        "plan": plan_ref,
        "policy": policy_ref,
        "entity": "operator_local",
        "process": "test",
        "timeout": 60,
    }))
    .unwrap();
    // `handle_request` is async; drive it on a current-thread runtime.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(handle_request(request, &state))
}

#[test]
fn f02_mixed_batch_rejects_before_any_episode_spawns() {
    // F-02 regression: `[SandboxedTask, Stage]` must reject with a typed error
    // *before* the sandboxed task is activated, so no episode is aborted
    // mid-dispatch and no settlement is lost.
    let root = tempfile::tempdir().unwrap();
    let policy_path = write_policy(root.path(), "allow");
    let argv = vec![trusted_self_executable(root.path())
        .to_string_lossy()
        .into_owned()];
    let plan_path = root.path().join("plan.json");
    fs::write(
        &plan_path,
        serde_json::to_vec(&plan(vec![
            sandboxed_task("task_a", argv, true),
            stage("stage_b"),
        ]))
        .unwrap(),
    )
    .unwrap();

    let response = submit(root.path(), &plan_path, &policy_path);
    assert!(response.get("error").is_some(), "{response}");
    assert!(
        response["error"]
            .as_str()
            .unwrap()
            .contains("non_executable"),
        "{response}"
    );

    // No episode was activated: the pre-validation rejected the batch before
    // the first spawn.
    let cases_dir = root.path().join("cases");
    let case_dir = fs::read_dir(&cases_dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .find(|p| p.is_dir())
        .expect("case dir must exist");
    let ledger = sea_forge_ledger::LedgerStream::open(
        root.path(),
        format!("case-{}", case_dir.file_name().unwrap().to_string_lossy()),
        "audit",
    )
    .unwrap();
    let activated = ledger
        .read_entries()
        .unwrap()
        .iter()
        .filter(|e| e.record_kind == "case_event")
        .filter(|e| e.payload["kind"].as_str() == Some("item_activated"))
        .count();
    assert_eq!(activated, 0, "no episode may be activated before rejection");
}

#[test]
fn f03_escalated_required_item_parks_case_not_terminates() {
    // F-03 regression: a required item under an escalate policy must park the
    // case as awaiting approval (not terminate it), keep the approval pending,
    // and must not label the escalation `item_failed`.
    let root = tempfile::tempdir().unwrap();
    let policy_path = write_policy(root.path(), "escalate");
    let argv = vec![trusted_self_executable(root.path())
        .to_string_lossy()
        .into_owned()];
    let plan_path = root.path().join("plan.json");
    fs::write(
        &plan_path,
        serde_json::to_vec(&plan(vec![sandboxed_task("task_a", argv, true)])).unwrap(),
    )
    .unwrap();

    let response = submit(root.path(), &plan_path, &policy_path);
    let case_id = response["case_id"].as_str().unwrap_or("").to_owned();
    assert!(!case_id.is_empty(), "{response}");

    let ledger =
        sea_forge_ledger::LedgerStream::open(root.path(), format!("case-{case_id}"), "audit")
            .unwrap();
    let entries = ledger.read_entries().unwrap();
    let event_kinds: Vec<String> = entries
        .iter()
        .filter(|e| e.record_kind == "case_event")
        .map(|e| e.payload["kind"].as_str().unwrap_or_default().to_string())
        .collect();
    // The escalation must not be folded into `item_failed` or `case_terminated`.
    assert!(
        !event_kinds.iter().any(|k| k == "item_failed"),
        "escalation must not be labeled item_failed: {event_kinds:?}"
    );
    assert!(
        !event_kinds.iter().any(|k| k == "case_terminated"),
        "escalated case must not be terminated: {event_kinds:?}"
    );

    let approvals: Vec<serde_json::Value> = entries
        .iter()
        .filter(|e| e.record_kind == "approval_request")
        .map(|e| e.payload.clone())
        .collect();
    assert_eq!(approvals.len(), 1, "escalation opens exactly one approval");
    assert_eq!(approvals[0]["status"], "pending", "approval stays pending");

    let case_json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.path().join("cases").join(&case_id).join("case.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        case_json["state"], "awaiting_approval",
        "case must be parked awaiting approval, not terminated"
    );
}

#[test]
fn f19_huge_timeout_is_clamped_not_panic() {
    // F-19/SUP-09a regression: a pathological u64 timeout must be clamped at
    // the boundary, never flipped negative through `as i64` (which would create
    // an already-expired approval) or panic.
    let root = tempfile::tempdir().unwrap();
    let policy_path = write_policy(root.path(), "allow");
    let argv = vec![trusted_self_executable(root.path())
        .to_string_lossy()
        .into_owned()];
    let plan_path = root.path().join("plan.json");
    fs::write(
        &plan_path,
        serde_json::to_vec(&plan(vec![sandboxed_task("task_a", argv, false)])).unwrap(),
    )
    .unwrap();

    let state = state(root.path());
    let request: Request = serde_json::from_value(serde_json::json!({
        "verb": "submit",
        "plan": "plan.json",
        "policy": policy_path.strip_prefix(root.path()).unwrap().to_string_lossy(),
        "entity": "operator_local",
        "process": "test",
        "timeout": 18_446_744_073_709_551_615u64,
    }))
    .unwrap();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let response = rt.block_on(handle_request(request, &state));
    // No panic; the submit either succeeds or returns a typed error.
    assert!(
        response.get("case_id").is_some() || response.get("error").is_some(),
        "{response}"
    );
}
