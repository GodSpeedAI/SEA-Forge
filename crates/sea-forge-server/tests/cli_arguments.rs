use std::process::Command;

#[test]
fn unknown_argument_exits_two_without_creating_cell_state() {
    let temp = tempfile::tempdir().expect("temporary parent directory");
    let root = temp.path().join("cell");

    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge-server"))
        .arg("--version")
        .env("SEA_FORGE_ROOT", &root)
        .output()
        .expect("server process starts");

    assert_eq!(output.status.code(), Some(2));
    assert!(
        !root.exists(),
        "argument rejection must precede configuration and socket side effects"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("takes no arguments"),
        "rejection must explain the supported configuration surface"
    );
}
