//! SUP-09b: `migrate` must refuse symlinked legacy trees and over-deep
//! nesting before creating key material or any migration side effect.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_root(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "sea-forge-migrate-safety-{name}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn run_migrate(root: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args(["migrate", "--root", root.to_str().unwrap()])
        .output()
        .unwrap()
}

fn assert_no_migration_side_effects(parent: &Path, root: &Path) {
    assert!(
        !root.join("ledgers").exists(),
        "ledgers/ must not exist after a rejected migration"
    );
    assert!(
        !root.join("migration.json").exists(),
        "migration marker must not exist after a rejected migration"
    );
    assert!(
        !root.join("quarantine").exists(),
        "quarantine/ must not exist after a rejected migration"
    );
    assert!(
        !parent.join(".sea-forge-migration-keys").exists(),
        "default key dir must not be created before traversal validation"
    );
}

#[cfg(unix)]
#[test]
fn migrate_rejects_symlink_directory_cycle_before_side_effects() {
    let parent = temp_root("symlink-cycle");
    let root = parent.join("state");
    let runs = root.join("runs");
    fs::create_dir_all(runs.join("run-1")).unwrap();
    fs::write(runs.join("run-1").join("plan.json"), b"{}").unwrap();
    std::os::unix::fs::symlink("../runs", runs.join("loop")).unwrap();

    let output = run_migrate(&root);
    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("refusing to migrate through symlink"),
        "stderr: {stderr}"
    );
    assert_no_migration_side_effects(&parent, &root);

    fs::remove_dir_all(parent).unwrap();
}

#[cfg(unix)]
#[test]
fn migrate_rejects_symlinked_file_before_side_effects() {
    let parent = temp_root("symlink-file");
    let root = parent.join("state");
    let runs = root.join("runs").join("run-1");
    fs::create_dir_all(&runs).unwrap();
    fs::write(runs.join("plan.json"), b"{}").unwrap();
    let outside = parent.join("outside-root.json");
    fs::write(&outside, b"{}").unwrap();
    std::os::unix::fs::symlink(&outside, runs.join("linked.json")).unwrap();

    let output = run_migrate(&root);
    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("refusing to migrate through symlink"),
        "stderr: {stderr}"
    );
    assert_no_migration_side_effects(&parent, &root);

    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn migrate_rejects_nesting_deeper_than_the_depth_cap() {
    let parent = temp_root("depth-cap");
    let root = parent.join("state");
    let mut dir = root.clone();
    for i in 0..=40 {
        dir = dir.join(format!("d{i}"));
    }
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("bottom.txt"), b"deep").unwrap();

    let output = run_migrate(&root);
    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("exceeds maximum depth"), "stderr: {stderr}");
    assert_no_migration_side_effects(&parent, &root);

    fs::remove_dir_all(parent).unwrap();
}
