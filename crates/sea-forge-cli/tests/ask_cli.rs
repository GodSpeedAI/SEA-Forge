//! Binary-spawn tests for `sea-forge ask` (spec-adlc-thoth §8.6, §11.1).
//! Tests exit codes: 0 answered, 4 denied, 2 usage, 1 error.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn bin_path() -> String {
    env!("CARGO_BIN_EXE_sea-forge").to_owned()
}

fn temp_root(name: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("sea-forge-ask-{name}-{nonce}"));
    std::fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn ask_unknown_kind_exits_nonzero() {
    let root = temp_root("unknown_kind");
    let bin = bin_path();
    let _ = Command::new(&bin)
        .args(["self-model", "--root", root.to_str().unwrap(), "rebuild"])
        .output();
    let output = Command::new(&bin)
        .args(["ask", "bogus_kind", "foo", "--root", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success(), "unknown kind must not succeed");
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn ask_without_snapshot_exits_1() {
    let root = temp_root("no_snapshot");
    let bin = bin_path();
    let output = Command::new(&bin)
        .args([
            "ask",
            "ask_capability",
            "domain-rust",
            "--root",
            root.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn ask_with_snapshot_but_no_policy_denies_exits_4() {
    let root = temp_root("denied");
    let bin = bin_path();
    let _ = Command::new(&bin)
        .args(["self-model", "--root", root.to_str().unwrap(), "rebuild"])
        .output();
    let output = Command::new(&bin)
        .args([
            "ask",
            "ask_capability",
            "domain-rust",
            "--root",
            root.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(4),
        "absent surface must deny: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("denied"));
    std::fs::remove_dir_all(root).ok();
}
