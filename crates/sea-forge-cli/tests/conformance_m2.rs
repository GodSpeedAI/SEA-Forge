use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("sea-forge-cli-m2-{nonce}"));
    fs::create_dir_all(&root).unwrap();
    root
}

fn write_policy(parent: &Path) -> PathBuf {
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
  - name: allow-sh
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

#[test]
fn conformance_m2_plan_runs_repeated_item_and_replays_activation_order() {
    let parent = temp_root();
    let root = parent.join("state");
    let policy = write_policy(&parent);
    let plan = parent.join("plan.json");
    fs::write(
        &plan,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": "0.2",
            "plan_id": "plan_01",
            "case_id": "case_proposal",
            "run_id": "run_proposal",
            "intent_id": "int_proposal",
            "items": [
                {
                    "plan_item_id": "A",
                    "name": "A",
                    "operations": [{"kind":"write_file","path":"a.txt","content_hint":"ok"}],
                    "entry_criteria": [], "exit_criteria": [],
                    "settlement_criteria": {"require_exit_zero":false,"required_artifacts":["a.txt"],"stdout_must_contain":null,"require_approval":false},
                    "item_kind":"sandboxed_task","sandbox_class":"local","parent_stage":null,
                    "markers":{"required":true,"repetition":false,"manual_activation":false},
                    "max_instances":1,"depends_on":[]
                },
                {
                    "plan_item_id": "B",
                    "name": "B",
                    "operations": [{"kind":"execute_command","argv":["sh","-c","exit 1"],"cwd":"."}],
                    "entry_criteria": [], "exit_criteria": [],
                    "settlement_criteria": {"require_exit_zero":true,"required_artifacts":[],"stdout_must_contain":null,"require_approval":false},
                    "item_kind":"sandboxed_task","sandbox_class":"jail","parent_stage":null,
                    "markers":{"required":true,"repetition":true,"manual_activation":false},
                    "max_instances":2,"depends_on":["A"]
                },
                {
                    "plan_item_id": "C", "name": "C", "operations": [],
                    "entry_criteria": [], "exit_criteria": [],
                    "settlement_criteria": {"require_exit_zero":false,"required_artifacts":[],"stdout_must_contain":null,"require_approval":false},
                    "item_kind":"milestone","sandbox_class":null,"parent_stage":null,
                    "markers":{"required":false,"repetition":false,"manual_activation":false},
                    "max_instances":1,"depends_on":["B"]
                }
            ],
            "template_ref": null
        }))
        .unwrap(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--plan",
            plan.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(3),
        "{}",
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
    assert_eq!(case["state"], "terminated");
    assert_eq!(case["close_reason"], "required_item_failed:B");
    assert_eq!(case["run_ids"].as_array().unwrap().len(), 3);
    let events = fs::read_to_string(case_dir.join("case-events.jsonl")).unwrap();
    let activations = events
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .filter(|event| event["kind"] == "item_activated")
        .map(|event| event["plan_item_id"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(activations, ["A", "B", "B"]);
    assert!(!events.contains("\"plan_item_id\":\"C\",\"kind\":\"milestone_achieved\""));
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn conformance_m2_human_task_parks_without_failure() {
    let parent = temp_root();
    let root = parent.join("state");
    let policy = write_policy(&parent);
    let plan = parent.join("human.json");
    fs::write(
        &plan,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version":"0.2","plan_id":"plan_human","case_id":"case_proposal",
            "run_id":"run_proposal","intent_id":"int_proposal","template_ref":null,
            "items":[{
                "plan_item_id":"review","name":"review","operations":[],
                "entry_criteria":[],"exit_criteria":[],
                "settlement_criteria":{"require_exit_zero":false,"required_artifacts":[],"stdout_must_contain":null,"require_approval":false},
                "item_kind":"human_task","sandbox_class":null,"parent_stage":null,
                "markers":{"required":true,"repetition":false,"manual_activation":false},
                "max_instances":1,"depends_on":[]
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--plan",
            plan.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(5),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("case_state=active"));
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn conformance_m2_invalid_plan_has_no_case_side_effect() {
    let parent = temp_root();
    let root = parent.join("state");
    let policy = write_policy(&parent);
    let plan = parent.join("bad.json");
    fs::write(&plan, b"{\"version\":\"0.2\",\"items\":[]}").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--plan",
            plan.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(!root.exists());
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn conformance_m2_plan_produces_committed_settlement_criteria_record() {
    let parent = temp_root();
    let root = parent.join("state");
    let policy = write_policy(&parent);
    let plan = parent.join("criteria_plan.json");
    fs::write(
        &plan,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": "0.2",
            "plan_id": "plan_criteria",
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
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--plan",
            plan.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let case_id = stdout
        .lines()
        .find_map(|line| line.strip_prefix("case_id="))
        .unwrap();
    let case_dir = root.join("cases").join(case_id);
    let plan_file: serde_json::Value =
        serde_json::from_slice(&fs::read(case_dir.join("plan.json")).unwrap()).unwrap();
    let criteria_ref = plan_file["items"][0]["settlement_criteria_ref"]
        .as_str()
        .unwrap();
    assert!(criteria_ref.starts_with("crit_"));

    let mut found = None;
    for stream_id in fs::read_dir(root.join("ledgers")).unwrap().flatten() {
        let path = stream_id.path().join("entries.jsonl");
        if !path.exists() {
            continue;
        }
        for line in fs::read_to_string(path).unwrap().lines() {
            let entry: serde_json::Value = serde_json::from_str(line).unwrap();
            if entry["record_kind"] == "settlement_criteria" {
                let record: serde_json::Value =
                    serde_json::from_value(entry["payload"].clone()).unwrap();
                if record["criteria_id"] == criteria_ref {
                    found = Some(record);
                }
            }
        }
    }
    let record = found.expect("criteria record should be committed to ledger");
    let criteria = record["criteria"].clone();
    let expected = plan_file["items"][0]["settlement_criteria"].clone();
    assert_eq!(criteria, expected);
    assert!(!record["origin_refs"].as_array().unwrap().is_empty());
    assert!(!record["criteria_sha256"].as_str().unwrap().is_empty());
    assert!(!record["criteria_record_hash"].as_str().unwrap().is_empty());
    fs::remove_dir_all(parent).unwrap();
}
