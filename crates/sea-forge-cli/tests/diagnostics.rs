use std::{env, process::Command};

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
