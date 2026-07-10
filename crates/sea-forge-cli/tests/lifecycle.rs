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
