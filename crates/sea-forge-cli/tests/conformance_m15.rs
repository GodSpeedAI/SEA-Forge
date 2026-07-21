//! M15 (E16b) Thoth manager loop conformance — T15.1-T15.5.
//! See `.agents/specs/spec-agent-orchestration.md` §17.4.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "sea-forge-m15-{name}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn write_policy(root: &Path, rules: &str) -> PathBuf {
    let path = root.join("policy.yaml");
    fs::write(&path, format!("version: \"0.1\"\nrules:\n{rules}")).unwrap();
    path
}

fn cli() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sea-forge"))
}

/// A single item gated on a sentry that never fires: zero ready actions,
/// not completed, not blocked — the deterministic definition of `stalled`.
/// `source: "case"` is a wildcard (matches any settled item) but since
/// nothing in this plan ever dispatches, no `settlement_status` event is
/// ever emitted and the sentry never fires.
fn stalled_plan_json() -> serde_json::Value {
    serde_json::json!({
        "version": "0.2",
        "plan_id": "plan_01",
        "case_id": "ignored_run_mints_its_own",
        "run_id": "run_01",
        "intent_id": "int_01",
        "items": [{
            "plan_item_id": "blocked_task",
            "name": "blocked_task",
            "operations": [{"kind": "write_file", "path": "out.txt", "content_hint": "done"}],
            "entry_criteria": [{
                "on": {"source": "case", "event": "settlement_status"},
                "if": {"kind": "settlement_status", "status": "accepted"}
            }],
            "exit_criteria": [],
            "settlement_criteria": {},
            "item_kind": "sandboxed_task",
            "sandbox_class": "local",
            "parent_stage": null,
            "markers": {"required": true, "repetition": false, "manual_activation": false},
            "max_instances": 1,
            "depends_on": []
        }],
        "template_ref": null,
        "job_contract_ref": null
    })
}

fn write_plan(root: &Path, plan: &serde_json::Value) -> PathBuf {
    let path = root.join("plan.json");
    fs::write(&path, serde_json::to_vec_pretty(plan).unwrap()).unwrap();
    path
}

/// Instantiate a plan via `run --plan`. `run` mints its own case_id
/// (ignoring the plan JSON's `case_id` field) and prints it to stdout as
/// `case_id=...`. A plan with zero ready actions exits 5 ("active") — not a
/// process failure.
fn bootstrap_case(root: &Path, policy: &Path, plan: &Path, entity: &str) -> String {
    let output = Command::new(cli())
        .args([
            "run",
            "--plan",
            plan.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--entity",
            entity,
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert_eq!(
        output.status.code(),
        Some(5),
        "bootstrap run --plan must leave the case active with zero ready items:\nstdout: {}\nstderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("case_id="))
        .expect("run --plan must print case_id=...")
        .to_string()
}

fn manager_iterate(
    root: &Path,
    policy: &Path,
    case_id: &str,
    actor: &str,
    max_iterations: u32,
) -> std::process::Output {
    Command::new(cli())
        .args([
            "case",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--actor",
            actor,
            "manager-iterate",
            case_id,
            "--max-iterations",
            &max_iterations.to_string(),
        ])
        .output()
        .unwrap()
}

fn ledger_entries(root: &Path, case_id: &str) -> Vec<serde_json::Value> {
    let entries_path = root
        .join("ledgers")
        .join(format!("case-{case_id}"))
        .join("entries.jsonl");
    fs::read_to_string(entries_path)
        .unwrap()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect()
}

fn manager_iteration_records(root: &Path, case_id: &str) -> Vec<serde_json::Value> {
    ledger_entries(root, case_id)
        .into_iter()
        .filter(|entry| entry["record_kind"] == "manager_iteration")
        .map(|entry| entry["payload"].clone())
        .collect()
}

fn approval_records(root: &Path, case_id: &str) -> Vec<serde_json::Value> {
    ledger_entries(root, case_id)
        .into_iter()
        .filter(|entry| entry["record_kind"] == "approval_request")
        .map(|entry| entry["payload"].clone())
        .collect()
}

fn read_case(root: &Path, case_id: &str) -> serde_json::Value {
    serde_json::from_slice(&fs::read(root.join("cases").join(case_id).join("case.json")).unwrap())
        .unwrap()
}

fn read_plan(root: &Path, case_id: &str) -> serde_json::Value {
    serde_json::from_slice(&fs::read(root.join("cases").join(case_id).join("plan.json")).unwrap())
        .unwrap()
}

const MANAGER_POLICY: &str = "  - name: allow-manager-read\n    verdict: allow\n    actor_role: operator\n    operation_kind: manager_iteration\n";
const PROPOSE_ALLOW: &str = "  - name: allow-propose\n    verdict: allow\n    actor_role: operator\n    operation_kind: discretionary_task_add\n";
const PROPOSE_DENY: &str = "  - name: deny-propose\n    verdict: deny\n    actor_role: operator\n    operation_kind: discretionary_task_add\n";
const WRITE_ALLOW: &str = "  - name: allow-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n";

#[test]
fn t15_1_stalled_case_one_iteration_proposes_and_records() {
    let root = temp_root("t15-1");
    let policy = write_policy(
        &root,
        &format!("{WRITE_ALLOW}{MANAGER_POLICY}{PROPOSE_ALLOW}"),
    );
    let plan = write_plan(&root, &stalled_plan_json());
    let case_id = bootstrap_case(&root, &policy, &plan, "operator_runner");

    let output = manager_iterate(&root, &policy, &case_id, "thoth", 8);
    assert!(
        output.status.success(),
        "manager-iterate failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let records = manager_iteration_records(&root, &case_id);
    assert_eq!(
        records.len(),
        1,
        "expected exactly one ManagerIteration record"
    );
    let record = &records[0];
    assert_eq!(record["iteration"], 1);
    assert_eq!(record["judgment"], "stalled");
    assert_eq!(record["action"], "propose_item");
    assert_eq!(record["granted"], true);
    assert!(!record["rationale_claim_refs"]
        .as_array()
        .unwrap()
        .is_empty());
    let proposed_ref = record["proposed_item_ref"].as_str().unwrap().to_string();

    let plan = read_plan(&root, &case_id);
    let items = plan["items"].as_array().unwrap();
    let proposed = items
        .iter()
        .find(|item| item["plan_item_id"] == proposed_ref)
        .expect("proposed item must be present in the case plan");
    assert_eq!(proposed["proposed_by"], "thoth");
    assert_eq!(proposed["item_kind"], "agent_task");
}

#[test]
fn t15_2_proposal_without_authority_is_denied_and_recorded() {
    let root = temp_root("t15-2");
    let policy = write_policy(
        &root,
        &format!("{WRITE_ALLOW}{MANAGER_POLICY}{PROPOSE_DENY}"),
    );
    let plan = write_plan(&root, &stalled_plan_json());
    let case_id = bootstrap_case(&root, &policy, &plan, "operator_runner");

    let output = manager_iterate(&root, &policy, &case_id, "thoth", 8);
    assert!(
        output.status.success(),
        "manager-iterate must not hard-fail on a denied proposal:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let records = manager_iteration_records(&root, &case_id);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["judgment"], "stalled");
    assert_eq!(records[0]["action"], "propose_item");
    assert_eq!(
        records[0]["granted"], false,
        "denial must be recorded as the outcome, not swallowed"
    );

    let plan = read_plan(&root, &case_id);
    assert_eq!(
        plan["items"].as_array().unwrap().len(),
        1,
        "a denied proposal must not mutate the plan"
    );
}

#[test]
fn t15_3_iteration_cap_reached_parks_and_escalates_without_further_proposals() {
    let root = temp_root("t15-3");
    let policy = write_policy(
        &root,
        &format!("{WRITE_ALLOW}{MANAGER_POLICY}{PROPOSE_ALLOW}"),
    );
    let plan = write_plan(&root, &stalled_plan_json());
    let case_id = bootstrap_case(&root, &policy, &plan, "operator_runner");

    let first = manager_iterate(&root, &policy, &case_id, "thoth", 1);
    assert!(first.status.success());
    assert_eq!(manager_iteration_records(&root, &case_id).len(), 1);

    let second = manager_iterate(&root, &policy, &case_id, "thoth", 1);
    assert_eq!(
        second.status.code(),
        Some(5),
        "cap-reached escalation must report the awaiting-approval exit code:\nstderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );

    // The cap-exceeded call is a guard before judgment — no new
    // ManagerIteration record, no further proposal (§16.2).
    assert_eq!(
        manager_iteration_records(&root, &case_id).len(),
        1,
        "iteration-cap escalation must not itself add a ManagerIteration record"
    );
    let approvals = approval_records(&root, &case_id);
    assert_eq!(approvals.len(), 1);
    assert_eq!(approvals[0]["status"], "pending");
    assert!(approvals[0]["note"]
        .as_str()
        .unwrap()
        .contains("iteration cap reached"));

    let case = read_case(&root, &case_id);
    assert_eq!(case["state"], "awaiting_approval");
}

#[test]
fn t15_4_proposer_cannot_resolve_its_own_proposed_items_approval() {
    let root = temp_root("t15-4");
    let policy = write_policy(
        &root,
        "  - name: escalate-write\n    verdict: escalate\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n  - name: allow-approval-resolution\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n",
    );
    let plan = serde_json::json!({
        "version": "0.2",
        "plan_id": "plan_01",
        "case_id": "ignored_run_mints_its_own",
        "run_id": "run_01",
        "intent_id": "int_01",
        "items": [{
            "plan_item_id": "proposed_task",
            "name": "proposed_task",
            "operations": [{"kind": "write_file", "path": "out.txt", "content_hint": "done"}],
            "entry_criteria": [],
            "exit_criteria": [],
            "settlement_criteria": {},
            "item_kind": "sandboxed_task",
            "sandbox_class": "local",
            "parent_stage": null,
            "markers": {"required": true, "repetition": false, "manual_activation": false},
            "max_instances": 1,
            "depends_on": [],
            "proposed_by": "thoth"
        }],
        "template_ref": null,
        "job_contract_ref": null
    });
    let plan_path = write_plan(&root, &plan);

    let output = Command::new(cli())
        .args([
            "run",
            "--plan",
            plan_path.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--entity",
            "operator_runner",
        ])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(5),
        "pre-dispatch authority escalation must park the case awaiting approval:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let case_id = String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("case_id="))
        .expect("run --plan must print case_id=...")
        .to_string();

    let approvals = approval_records(&root, &case_id);
    assert_eq!(approvals.len(), 1);
    let approval_id = approvals[0]["approval_id"].as_str().unwrap();

    // The proposer cannot resolve its own item's approval — structural SoD.
    let denied = Command::new(cli())
        .args([
            "approve",
            &case_id,
            approval_id,
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--actor",
            "thoth",
        ])
        .output()
        .unwrap();
    assert!(
        !denied.status.success(),
        "thoth must not be able to resolve its own proposal's approval"
    );
    assert!(String::from_utf8_lossy(&denied.stderr).contains("sod_violation"));

    // A distinct, uninvolved actor can still resolve it normally.
    let granted = Command::new(cli())
        .args([
            "approve",
            &case_id,
            approval_id,
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--actor",
            "operator_reviewer",
        ])
        .output()
        .unwrap();
    assert!(
        granted.status.success(),
        "an uninvolved reviewer must still be able to resolve the approval:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&granted.stdout),
        String::from_utf8_lossy(&granted.stderr)
    );
}

#[test]
fn t15_5_every_judgment_cites_nonempty_resolvable_claim_refs() {
    let root = temp_root("t15-5");
    let policy = write_policy(
        &root,
        &format!("{WRITE_ALLOW}{MANAGER_POLICY}{PROPOSE_ALLOW}"),
    );
    // An item with satisfied entry criteria and manual_activation is
    // "enabled" — has_ready_work true, judgment progressing (§9.5).
    let plan = serde_json::json!({
        "version": "0.2",
        "plan_id": "plan_01",
        "case_id": "ignored_run_mints_its_own",
        "run_id": "run_01",
        "intent_id": "int_01",
        "items": [{
            "plan_item_id": "enabled_task",
            "name": "enabled_task",
            "operations": [{"kind": "write_file", "path": "out.txt", "content_hint": "done"}],
            "entry_criteria": [],
            "exit_criteria": [],
            "settlement_criteria": {},
            "item_kind": "sandboxed_task",
            "sandbox_class": "local",
            "parent_stage": null,
            "markers": {"required": true, "repetition": false, "manual_activation": true},
            "max_instances": 1,
            "depends_on": []
        }],
        "template_ref": null,
        "job_contract_ref": null
    });
    let plan_path = write_plan(&root, &plan);
    let case_id = bootstrap_case(&root, &policy, &plan_path, "operator_runner");

    let output = manager_iterate(&root, &policy, &case_id, "thoth", 8);
    assert!(output.status.success());

    let records = manager_iteration_records(&root, &case_id);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["judgment"], "progressing");
    let refs = records[0]["rationale_claim_refs"].as_array().unwrap();
    assert!(
        !refs.is_empty(),
        "a judgment without evidence refs must be rejected at record time"
    );
    assert!(
        refs.iter()
            .any(|r| r.as_str().unwrap().contains("enabled_task")),
        "rationale refs must resolve to the real item that grounds the judgment, refs={refs:?}"
    );
}
