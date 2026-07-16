use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use sea_forge_core::types::{
    ApprovalRequest, ApprovalStatus, AuthorityDecision, SettlementCriteriaRecord,
};
use sea_forge_ledger::LedgerStream;

fn temp_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("sea-forge-cli-m3-{nonce}"));
    fs::create_dir_all(&root).unwrap();
    root
}

fn write_escalate_policy(parent: &Path) -> PathBuf {
    let path = parent.join("policy.yaml");
    fs::write(
        &path,
        r#"version: "0.1"
identity:
  source: test
  allow_unresolved: false
identity_bindings:
  - principal: operator_local
    actor_type: human
    role: operator
  - principal: security_officer
    actor_type: human
    role: R-SO
  - principal: unprivileged_operator
    actor_type: human
    role: operator
sod_rules:
  - name: write_requires_security_officer
    requester_role: operator
    approver_role: R-SO
    operation_kind: write_file
rules:
  - name: escalate-write
    verdict: escalate
    actor_role: operator
    operation_kind: write_file
    path_prefix: ""
  - name: allow-execute
    verdict: allow
    actor_role: operator
    operation_kind: execute_command
    argv0: sh
    sandbox_class: jail
  - name: resolve-approval
    verdict: allow
    actor_role: R-SO
    operation_kind: approval_resolution
"#,
    )
    .unwrap();
    path
}

fn write_allow_policy(parent: &Path) -> PathBuf {
    let path = parent.join("policy.yaml");
    fs::write(
        &path,
        r#"version: "0.1"
identity:
  source: test
  allow_unresolved: false
rules:
  - name: allow-write
    verdict: allow
    actor_role: operator
    operation_kind: write_file
    path_prefix: ""
  - name: allow-execute
    verdict: allow
    actor_role: operator
    operation_kind: execute_command
    argv0: sh
    sandbox_class: jail
"#,
    )
    .unwrap();
    path
}

fn write_simple_plan(parent: &Path) -> PathBuf {
    let plan = parent.join("plan.json");
    fs::write(
        &plan,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": "0.2",
            "plan_id": "plan_01",
            "case_id": "case_proposal",
            "run_id": "run_proposal",
            "intent_id": "int_proposal",
            "template_ref": null,
            "items": [{
                "plan_item_id": "task",
                "name": "task",
                "operations": [{"kind":"write_file","path":"out.txt","content_hint":"ok"}],
                "entry_criteria": [], "exit_criteria": [],
                "settlement_criteria": {"require_exit_zero":false,"required_artifacts":["out.txt"],"stdout_must_contain":null,"require_approval":false},
                "item_kind":"sandboxed_task","sandbox_class":"local","parent_stage":null,
                "markers":{"required":true,"repetition":false,"manual_activation":false},
                "max_instances":1,"depends_on":[]
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    plan
}

fn sea_forge_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sea-forge"))
}

fn run_sea_forge(args: &[&str]) -> std::process::Output {
    Command::new(sea_forge_bin()).args(args).output().unwrap()
}

#[test]
fn conformance_m3_escalate_creates_approval_and_exits_5() {
    let parent = temp_root();
    let root = parent.join("state");
    let policy = write_escalate_policy(&parent);
    let plan = write_simple_plan(&parent);

    let output = run_sea_forge(&[
        "run",
        "--plan",
        plan.to_str().unwrap(),
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
    ]);

    assert_eq!(
        output.status.code(),
        Some(5),
        "escalated run should exit 5\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let case_id = stdout
        .lines()
        .find_map(|line| line.strip_prefix("case_id="))
        .unwrap();
    let case_dir = root.join("cases").join(case_id);
    let case: serde_json::Value =
        serde_json::from_slice(&fs::read(case_dir.join("case.json")).unwrap()).unwrap();
    assert_eq!(case["state"], "awaiting_approval");

    // Verify approval was created.
    let approvals = fs::read_to_string(root.join("approvals.jsonl")).unwrap();
    assert!(approvals.contains("\"status\":\"pending\""));
    let approval_id = approvals
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .find(|v| v["status"] == "pending")
        .unwrap()["approval_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Approve.
    let approve_output = run_sea_forge(&[
        "approve",
        case_id,
        &approval_id,
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
        "--actor",
        "security_officer",
    ]);
    assert!(
        approve_output.status.success(),
        "{}",
        String::from_utf8_lossy(&approve_output.stderr)
    );

    // Resume with allow policy.
    let allow_policy = write_allow_policy(&parent);
    let resume_output = run_sea_forge(&[
        "resume",
        case_id,
        "--root",
        root.to_str().unwrap(),
        "--policy",
        allow_policy.to_str().unwrap(),
    ]);
    assert_eq!(
        resume_output.status.code(),
        Some(0),
        "resume should complete the case\n{}",
        String::from_utf8_lossy(&resume_output.stderr)
    );

    let resume_stdout = String::from_utf8(resume_output.stdout).unwrap();
    assert!(resume_stdout.contains("case_state=completed"));

    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn conformance_m3_double_approve_is_refused() {
    let parent = temp_root();
    let root = parent.join("state");
    let policy = write_escalate_policy(&parent);
    let plan = write_simple_plan(&parent);

    let output = run_sea_forge(&[
        "run",
        "--plan",
        plan.to_str().unwrap(),
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(5));

    let stdout = String::from_utf8(output.stdout).unwrap();
    let case_id = stdout
        .lines()
        .find_map(|line| line.strip_prefix("case_id="))
        .unwrap();

    let approvals = fs::read_to_string(root.join("approvals.jsonl")).unwrap();
    let approval_id = approvals
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .find(|v| v["status"] == "pending")
        .unwrap()["approval_id"]
        .as_str()
        .unwrap()
        .to_string();

    // First approve succeeds.
    let first = run_sea_forge(&[
        "approve",
        case_id,
        &approval_id,
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
        "--actor",
        "security_officer",
    ]);
    assert!(first.status.success());

    // Second approve is refused (no re-resolution).
    let second = run_sea_forge(&[
        "approve",
        case_id,
        &approval_id,
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
        "--actor",
        "security_officer",
    ]);
    assert!(!second.status.success());

    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn conformance_m3_unauthorized_approver_cannot_append_resolution() {
    let parent = temp_root();
    let root = parent.join("state");
    let policy = write_escalate_policy(&parent);
    let output = run_sea_forge(&[
        "run",
        "--plan",
        write_simple_plan(&parent).to_str().unwrap(),
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(5));
    let case_id = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("case_id="))
        .unwrap()
        .to_owned();
    let approval_id = fs::read_to_string(root.join("approvals.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .find(|approval| approval["status"] == "pending")
        .unwrap()["approval_id"]
        .as_str()
        .unwrap()
        .to_owned();

    let unauthorized = run_sea_forge(&[
        "approve",
        &case_id,
        &approval_id,
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
        "--actor",
        "unprivileged_operator",
    ]);
    assert!(!unauthorized.status.success());
    let stream = LedgerStream::open(&root, format!("case-{case_id}"), "test").unwrap();
    assert!(!stream.read_entries().unwrap().iter().any(|entry| {
        entry.record_kind == "approval_resolution"
            && entry
                .payload
                .get("approval_id")
                .and_then(|value| value.as_str())
                == Some(approval_id.as_str())
    }));

    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn conformance_m3_security_officer_can_append_authorized_resolution() {
    let parent = temp_root();
    let root = parent.join("state");
    let policy = write_escalate_policy(&parent);
    let output = run_sea_forge(&[
        "run",
        "--plan",
        write_simple_plan(&parent).to_str().unwrap(),
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(5));
    let case_id = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("case_id="))
        .unwrap()
        .to_owned();
    let approval_id = fs::read_to_string(root.join("approvals.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .find(|approval| approval["status"] == "pending")
        .unwrap()["approval_id"]
        .as_str()
        .unwrap()
        .to_owned();

    let authorized = run_sea_forge(&[
        "approve",
        &case_id,
        &approval_id,
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
        "--actor",
        "security_officer",
    ]);
    assert!(
        authorized.status.success(),
        "{}",
        String::from_utf8_lossy(&authorized.stderr)
    );
    let stream = LedgerStream::open(&root, format!("case-{case_id}"), "test").unwrap();
    assert!(stream.read_entries().unwrap().iter().any(|entry| {
        entry.record_kind == "approval_resolution"
            && entry
                .payload
                .get("approval_id")
                .and_then(|value| value.as_str())
                == Some(approval_id.as_str())
    }));

    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn conformance_m3_approve_uses_ledger_truth_not_tampered_compatibility_view() {
    let parent = temp_root();
    let root = parent.join("state");
    let policy = write_escalate_policy(&parent);
    let output = run_sea_forge(&[
        "run",
        "--plan",
        write_simple_plan(&parent).to_str().unwrap(),
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(5));
    let case_id = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("case_id="))
        .unwrap()
        .to_owned();
    let approvals_path = root.join("approvals.jsonl");
    let mut request: serde_json::Value = serde_json::from_str(
        fs::read_to_string(&approvals_path)
            .unwrap()
            .lines()
            .next()
            .unwrap(),
    )
    .unwrap();
    let approval_id = request["approval_id"].as_str().unwrap().to_owned();
    request["decision_id"] = serde_json::json!("auth_tampered");
    fs::write(&approvals_path, format!("{request}\n")).unwrap();

    let stream = LedgerStream::open(&root, format!("case-{case_id}"), "test").unwrap();
    let mut wrong: ApprovalRequest = serde_json::from_value(
        stream
            .read_entries()
            .unwrap()
            .into_iter()
            .find(|entry| entry.record_kind == "approval_request")
            .unwrap()
            .payload,
    )
    .unwrap();
    wrong.approval_id = "apr_wrong_criteria".into();
    wrong.criteria_record_hash = Some("sha256:wrong".into());
    stream
        .commit_typed("approval_request", vec![], &wrong, vec![])
        .unwrap();
    let wrong_output = run_sea_forge(&[
        "approve",
        &case_id,
        &wrong.approval_id,
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
        "--actor",
        "security_officer",
    ]);
    assert!(!wrong_output.status.success());

    let self_approval = run_sea_forge(&[
        "approve",
        &case_id,
        &approval_id,
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
    ]);
    assert!(!self_approval.status.success());

    let approved = run_sea_forge(&[
        "approve",
        &case_id,
        &approval_id,
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
        "--actor",
        "security_officer",
    ]);
    assert!(
        approved.status.success(),
        "{}",
        String::from_utf8_lossy(&approved.stderr)
    );
    let resolution: ApprovalRequest = serde_json::from_value(
        stream
            .read_entries()
            .unwrap()
            .into_iter()
            .rfind(|entry| entry.record_kind == "approval_resolution")
            .unwrap()
            .payload,
    )
    .unwrap();
    assert_ne!(resolution.decision_id, "auth_tampered");

    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn conformance_m3_reject_terminates_case() {
    let parent = temp_root();
    let root = parent.join("state");
    let policy = write_escalate_policy(&parent);
    let plan = write_simple_plan(&parent);

    let output = run_sea_forge(&[
        "run",
        "--plan",
        plan.to_str().unwrap(),
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(5));

    let stdout = String::from_utf8(output.stdout).unwrap();
    let case_id = stdout
        .lines()
        .find_map(|line| line.strip_prefix("case_id="))
        .unwrap();

    let approvals = fs::read_to_string(root.join("approvals.jsonl")).unwrap();
    let approval_id = approvals
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .find(|v| v["status"] == "pending")
        .unwrap()["approval_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Reject.
    let reject_output = run_sea_forge(&[
        "reject",
        case_id,
        &approval_id,
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
        "--actor",
        "security_officer",
    ]);
    assert!(
        reject_output.status.success(),
        "{}",
        String::from_utf8_lossy(&reject_output.stderr)
    );

    // Resume with rejected approval → terminated.
    let resume_output = run_sea_forge(&[
        "resume",
        case_id,
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
    ]);
    assert_eq!(
        resume_output.status.code(),
        Some(4),
        "rejected resume should exit 4\n{}",
        String::from_utf8_lossy(&resume_output.stderr)
    );

    let resume_stdout = String::from_utf8(resume_output.stdout).unwrap();
    assert!(resume_stdout.contains("case_state=terminated"));

    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn conformance_m3_approval_basis_is_authority_escalate_expired_on_ttl() {
    let parent = temp_root();
    let root = parent.join("state");
    let policy = write_escalate_policy(&parent);
    let output = run_sea_forge(&[
        "run",
        "--plan",
        write_simple_plan(&parent).to_str().unwrap(),
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(5));
    let case_id = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("case_id="))
        .unwrap()
        .to_owned();
    let stream = LedgerStream::open(&root, format!("case-{case_id}"), "test").unwrap();
    let entries = stream.read_entries().unwrap();
    let decision: AuthorityDecision = serde_json::from_value(
        entries
            .iter()
            .find(|entry| entry.record_kind == "authority_decision")
            .unwrap()
            .payload
            .clone(),
    )
    .unwrap();
    let criteria: SettlementCriteriaRecord = serde_json::from_value(
        entries
            .iter()
            .find(|entry| entry.record_kind == "settlement_criteria")
            .unwrap()
            .payload
            .clone(),
    )
    .unwrap();
    let approval = ApprovalRequest {
        version: "0.2".into(),
        approval_id: "apr_expired".into(),
        run_id: decision.run_id.clone(),
        case_id: case_id.clone(),
        decision_id: decision.decision_id.clone(),
        plan_item_id: decision.plan_item_id.clone(),
        criteria_ref: Some(criteria.criteria_id.clone()),
        criteria_sha256: Some(criteria.criteria_sha256.clone()),
        criteria_record_hash: Some(criteria.criteria_record_hash.clone()),
        job_contract_ref: None,
        requested_at: "2026-01-01T00:00:00Z".into(),
        expires_at: "2026-01-01T00:01:00Z".into(),
        status: ApprovalStatus::Pending,
        resolved_by: None,
        resolved_at: None,
        note: None,
    };
    stream
        .commit_typed("approval_request", vec![], &approval, vec![])
        .unwrap();
    fs::write(
        root.join("approvals.jsonl"),
        format!("{}\n", serde_json::to_string(&approval).unwrap()),
    )
    .unwrap();

    // Try to approve an already-expired approval.
    let output = run_sea_forge(&[
        "approve",
        &case_id,
        "apr_expired",
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy.to_str().unwrap(),
        "--actor",
        "security_officer",
    ]);
    assert!(
        !output.status.success(),
        "expired approval should not be resolvable"
    );

    // Verify it was marked expired.
    let approvals = fs::read_to_string(root.join("approvals.jsonl")).unwrap();
    assert!(approvals.contains("\"status\":\"expired\""));

    fs::remove_dir_all(parent).unwrap();
}
