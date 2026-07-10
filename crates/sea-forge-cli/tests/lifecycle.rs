use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_root(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("sea-forge-{name}-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}
fn policy(root: &std::path::Path, rules: &str) -> PathBuf {
    let path = root.join("policy.yaml");
    fs::write(&path, format!("version: \"0.1\"\nrules:\n{rules}")).unwrap();
    path
}

#[test]
fn intent_to_settlement_produces_complete_accepted_run() {
    let parent = temp_root("accepted");
    let root = parent.join("state");
    let policy=policy(&parent,"  - name: allow-model-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n  - name: allow-self-validate\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n");
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--intent",
            "Generate and validate a simple DomainForge .sea model",
        ])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let run_id = stdout
        .lines()
        .find_map(|l| l.strip_prefix("run_id="))
        .unwrap();
    let run = root.join("runs").join(run_id);
    for name in [
        "plan.json",
        "authority.json",
        "trace.jsonl",
        "evidence.jsonl",
        "settlement.json",
        "semantic-envelope.json",
    ] {
        assert!(run.join(name).is_file(), "missing {name}");
    }
    let settlement: serde_json::Value =
        serde_json::from_slice(&fs::read(run.join("settlement.json")).unwrap()).unwrap();
    assert_eq!(settlement["status"], "accepted");
    let envelope: serde_json::Value =
        serde_json::from_slice(&fs::read(run.join("semantic-envelope.json")).unwrap()).unwrap();
    assert!(envelope["artifact_refs"]
        .as_array()
        .is_some_and(|v| v.len() == 1));
    assert_eq!(
        fs::read_to_string(root.join("capabilities.jsonl"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn authority_denial_settles_rejected_without_command_start() {
    let parent = temp_root("denied");
    let root = parent.join("state");
    let policy = policy(&parent, "  []\n");
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--intent",
            "Generate and validate a simple DomainForge .sea model",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let run_id = stdout
        .lines()
        .find_map(|l| l.strip_prefix("run_id="))
        .unwrap();
    let trace = fs::read_to_string(root.join("runs").join(run_id).join("trace.jsonl")).unwrap();
    assert!(!trace.contains("command_started"));
    assert!(trace.contains("run_halted"));
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn escalation_halts_and_requires_review() {
    let parent = temp_root("escalated");
    let root = parent.join("state");
    let policy = policy(&parent, "  - name: review-write\n    verdict: escalate\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n  - name: review-command\n    verdict: escalate\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n");
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--intent",
            "Generate and validate a simple DomainForge .sea model",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let run_id = stdout
        .lines()
        .find_map(|line| line.strip_prefix("run_id="))
        .unwrap();
    let run = root.join("runs").join(run_id);
    let settlement: serde_json::Value =
        serde_json::from_slice(&fs::read(run.join("settlement.json")).unwrap()).unwrap();
    assert_eq!(settlement["status"], "escalated");
    assert_eq!(settlement["review_required"], true);
    assert!(!fs::read_to_string(run.join("trace.jsonl"))
        .unwrap()
        .contains("command_started"));
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn repeated_runs_have_stable_artifact_identity_and_recall_is_read_only() {
    let parent = temp_root("repeat");
    let root = parent.join("state");
    let policy = policy(&parent, "  - name: allow-model-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n  - name: allow-self-validate\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n");
    let mut identities = Vec::new();
    for _ in 0..2 {
        let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
            .args([
                "run",
                "--root",
                root.to_str().unwrap(),
                "--policy",
                policy.to_str().unwrap(),
                "--entity",
                "team_a",
                "--process",
                "agent_1",
                "--intent",
                "Generate and validate a simple DomainForge .sea model",
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = String::from_utf8(output.stdout).unwrap();
        let run_id = stdout
            .lines()
            .find_map(|line| line.strip_prefix("run_id="))
            .unwrap();
        let envelope: serde_json::Value = serde_json::from_slice(
            &fs::read(
                root.join("runs")
                    .join(run_id)
                    .join("semantic-envelope.json"),
            )
            .unwrap(),
        )
        .unwrap();
        identities.push(
            envelope["artifact_refs"][0]["pre_mint_identity"]
                .as_str()
                .unwrap()
                .to_owned(),
        );
    }
    assert_eq!(identities[0], identities[1]);
    let before = fs::read(root.join("capabilities.jsonl")).unwrap();
    let recall = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "recall",
            "generate",
            "--root",
            root.to_str().unwrap(),
            "--entity",
            "team_a",
            "--process",
            "agent_1",
        ])
        .output()
        .unwrap();
    assert_eq!(recall.status.code(), Some(0));
    assert_eq!(String::from_utf8(recall.stdout).unwrap().lines().count(), 2);
    assert_eq!(before, fs::read(root.join("capabilities.jsonl")).unwrap());
    assert_eq!(fs::read_dir(root.join("runs")).unwrap().count(), 2);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn validator_accepts_only_the_stub_contract() {
    let parent = temp_root("validate");
    let valid = parent.join("valid.sea");
    fs::write(
        &valid,
        r#"{"domain":"demo","entities":[{"name":"Sample"}]}"#,
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args(["validate", valid.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"sea-forge: model valid\n");
    let invalid = parent.join("invalid.sea");
    fs::write(&invalid, r#"{"domain":"demo","entities":[]}"#).unwrap();
    assert_eq!(
        Command::new(env!("CARGO_BIN_EXE_sea-forge"))
            .args(["validate", invalid.to_str().unwrap()])
            .status()
            .unwrap()
            .code(),
        Some(1)
    );
    fs::remove_dir_all(parent).unwrap();
}
