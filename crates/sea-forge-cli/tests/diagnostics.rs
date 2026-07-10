use std::{
    env, fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn config_failure_is_structured_and_correlated() {
    let missing = env::temp_dir().join(format!("sea-forge-missing-policy-{}", std::process::id()));
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--policy",
            missing.to_str().unwrap(),
            "--intent",
            "Generate and validate a simple DomainForge .sea model",
        ])
        .env_remove("RUST_LOG")
        .output()
        .expect("sea-forge binary should run");
    assert_eq!(output.status.code(), Some(1));
    let diagnostic: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("diagnostic should be JSON");
    assert_eq!(diagnostic["event"], "command_failed");
    assert_eq!(diagnostic["run_id"], "none");
    assert_eq!(diagnostic["component"], "sea-forge-cli");
    assert_eq!(diagnostic["error_class"], "missing_config_error");
}

#[test]
fn malformed_policy_classes_fail_before_run_creation() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let parent = env::temp_dir().join(format!("sea-forge-config-errors-{nonce}"));
    fs::create_dir_all(&parent).unwrap();
    let cases = [
        ("parse.yaml", "rules: [", "parse_error"),
        ("schema.yaml", "version: \"9\"\nrules: []\n", "schema_error"),
        ("kind.yaml", "version: \"0.1\"\nrules:\n  - name: bad\n    verdict: allow\n    actor_role: operator\n    operation_kind: teleport\n", "unsupported_kind_error"),
        ("secret.yaml", "version: \"0.1\"\nidentity:\n  allow_unresolved: super-secret-token\nrules: []\n", "schema_error"),
    ];
    for (name, content, class) in cases {
        let policy = parent.join(name);
        let root = parent.join(format!("state-{name}"));
        fs::write(&policy, content).unwrap();
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
        assert_eq!(output.status.code(), Some(1));
        let diagnostic: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(diagnostic["error_class"], class);
        assert!(!String::from_utf8_lossy(&output.stderr).contains("super-secret-token"));
        assert!(!root.exists());
    }
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn recall_missing_memory_is_an_io_error() {
    let root = env::temp_dir().join(format!("sea-forge-missing-memory-{}", std::process::id()));
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args(["recall", "anything", "--root", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let diagnostic: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(diagnostic["error_class"], "io_error");
}

#[test]
fn inspect_rejects_noncanonical_run_ids() {
    for run_id in ["../../outside", "run_99999999T999999Z_abcdef"] {
        let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
            .args(["inspect", run_id, "--root", "."])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        let diagnostic: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(diagnostic["error_class"], "input_error");
    }
}

#[cfg(unix)]
#[test]
fn state_child_symlink_is_rejected_before_outside_write() {
    use std::os::unix::fs::symlink;
    let parent = env::temp_dir().join(format!("sea-forge-state-symlink-{}", std::process::id()));
    let root = parent.join("state");
    let outside = parent.join("outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    symlink(&outside, root.join("runs")).unwrap();
    let policy = parent.join("policy.yaml");
    fs::write(&policy, "version: \"0.1\"\nrules: []\n").unwrap();
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
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn invalid_intents_exit_two_without_creating_a_run_root() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let parent = env::temp_dir().join(format!("sea-forge-input-errors-{nonce}"));
    fs::create_dir_all(&parent).unwrap();
    let policy = parent.join("policy.yaml");
    fs::write(&policy, "version: \"0.1\"\nrules: []\n").unwrap();
    for (index, intent) in [
        "   ".to_owned(),
        "x".repeat(501),
        "do something unknown".to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        let root = parent.join(format!("state-{index}"));
        let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
            .args([
                "run",
                "--root",
                root.to_str().unwrap(),
                "--policy",
                policy.to_str().unwrap(),
                "--intent",
                &intent,
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(!root.exists());
    }
    fs::remove_dir_all(parent).unwrap();
}
