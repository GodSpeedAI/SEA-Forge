use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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
    ]);
    assert!(first.status.success());

    // Second approve is refused (no re-resolution).
    let second = run_sea_forge(&[
        "approve",
        case_id,
        &approval_id,
        "--root",
        root.to_str().unwrap(),
    ]);
    assert!(!second.status.success());

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
    // This test verifies that an expired approval prevents resolution.
    // Since TTL is 24h by default, we manually create an expired approval.
    let parent = temp_root();
    let root = parent.join("state");
    fs::create_dir_all(&root).unwrap();

    let approval = serde_json::json!({
        "version": "0.2",
        "approval_id": "apr_0001",
        "run_id": "run_test",
        "case_id": "case_test",
        "decision_id": "auth_01",
        "plan_item_id": "item_01",
        "criteria_ref": null,
        "criteria_sha256": null,
        "criteria_record_hash": null,
        "job_contract_ref": null,
        "requested_at": "2026-01-01T00:00:00Z",
        "expires_at": "2026-01-01T00:01:00Z",
        "status": "pending",
        "resolved_by": null,
        "resolved_at": null,
        "note": null
    });

    fs::write(
        root.join("approvals.jsonl"),
        format!("{}\n", serde_json::to_string(&approval).unwrap()),
    )
    .unwrap();

    // Try to approve an already-expired approval.
    let output = run_sea_forge(&[
        "approve",
        "case_test",
        "apr_0001",
        "--root",
        root.to_str().unwrap(),
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
