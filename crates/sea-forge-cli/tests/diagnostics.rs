use std::process::Command;

#[test]
fn startup_failure_is_structured_and_correlated() {
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .env_remove("RUST_LOG")
        .output()
        .expect("sea-forge binary should run");

    assert_eq!(output.status.code(), Some(64));

    let stderr = String::from_utf8(output.stderr).expect("diagnostic should be UTF-8");
    let diagnostic: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("diagnostic should be one JSON object");

    assert_eq!(diagnostic["event"], "startup_failed");
    assert_eq!(diagnostic["run_id"], "none");
    assert_eq!(diagnostic["component"], "sea-forge-cli");
    assert_eq!(diagnostic["error_class"], "not_implemented_error");
    assert!(diagnostic["message"]
        .as_str()
        .expect("message should be a string")
        .contains("minimum slice not implemented"));
}
