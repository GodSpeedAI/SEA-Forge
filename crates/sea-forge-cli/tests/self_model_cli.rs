//! CLI integration for `sea-forge self-model validate|rebuild|show`
//! (spec-adlc-thoth §11.1, M9 slice 1.5b). Drives the built binary end-to-end
//! against a temporary root.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn sea_forge_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sea-forge"))
}

fn run(root: &std::path::Path, action: &str, extra: &[&str]) -> (u8, String) {
    let mut args = vec!["self-model", "--root", root.to_str().unwrap(), action];
    args.extend_from_slice(extra);
    let output = Command::new(sea_forge_bin()).args(&args).output().unwrap();
    let code = output.status.code().unwrap_or(1) as u8;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    (code, stdout)
}

fn temp_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("sea-forge-self-model-{nonce}"));
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn rebuild_validate_show_round_trip() {
    let root = temp_root();
    // Rebuild builds the initial snapshot (exit 0).
    let (rc, out) = run(&root, "rebuild", &[]);
    assert_eq!(rc, 0, "rebuild stdout: {out}");
    assert!(out.contains("snapshot_id=smsnap_"));

    // Validate verifies bundled models + snapshot + projections (exit 0).
    let (rc, out) = run(&root, "validate", &[]);
    assert_eq!(rc, 0, "validate stdout: {out}");

    // Show --json emits the full snapshot record.
    let (rc, out) = run(&root, "show", &["--json"]);
    assert_eq!(rc, 0);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["release_id"], "sea-forge@0.1.0");
    assert!(v["snapshot_hash"].as_str().unwrap().starts_with("sha256:"));

    // A second rebuild creates a distinct snapshot; validate stays green.
    let first = v["snapshot_id"].as_str().unwrap().to_string();
    let (rc, _) = run(&root, "rebuild", &[]);
    assert_eq!(rc, 0);
    let (_, out) = run(&root, "show", &[]);
    assert!(
        !out.contains(&first),
        "second rebuild must mint a new snapshot id"
    );
    let (rc, _) = run(&root, "validate", &[]);
    assert_eq!(rc, 0);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn tampered_snapshot_is_self_model_error_exit1() {
    let root = temp_root();
    run(&root, "rebuild", &[]);
    // Corrupt the snapshot file on disk.
    let snap_dir = root.join(".sea-forge/self-model/snapshots");
    let snap_file = fs::read_dir(&snap_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::write(&snap_file, b"{not valid json").unwrap();
    let (rc, _out) = run(&root, "validate", &[]);
    assert_eq!(rc, 1, "corrupt snapshot must fail validation with exit 1");
    let _ = fs::remove_dir_all(&root);
}
