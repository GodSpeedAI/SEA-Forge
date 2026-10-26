#![cfg(unix)]
//! SEC-06 owner-only control-plane state, independent of Landlock.
//!
//! ADV-A07 found the state tree world-readable (0755/0644), protected only
//! by the isolated Landlock read posture. `harden_state_tree` remediates an
//! existing tree and `prepare_traversal` opens only the traverse-only chain
//! to a run. The attacker here is a different subordinate UID with no
//! Landlock and no mount namespace, so only DAC stands between it and the
//! files. A mutation control runs the same attack on the unhardened tree.
#![cfg(target_os = "linux")]

use sea_forge_sandbox::workload::{
    grant_exclusive_access, harden_state_tree, prepare_traversal, WorkloadIdentity,
    WorkloadIdentityPool, WORKLOAD_INNER_ID,
};
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;

fn pool() -> Option<WorkloadIdentityPool> {
    match WorkloadIdentityPool::from_host() {
        Ok(pool) => Some(pool),
        Err(error) => {
            assert!(
                std::env::var_os("SEA_FORGE_REQUIRE_WORKLOAD_ISOLATION").is_none(),
                "workload isolation required but unavailable: {error}"
            );
            eprintln!("NOT-EXECUTABLE: {error}");
            None
        }
    }
}

/// Run `script` as `identity` with DAC only (user namespace, no Landlock).
fn as_identity(identity: &WorkloadIdentity, script: &str) -> String {
    let out = Command::new("/usr/bin/unshare")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .arg("--user")
        .arg(format!(
            "--map-users={WORKLOAD_INNER_ID}:{}:1",
            identity.host_uid
        ))
        .arg(format!(
            "--map-groups={WORKLOAD_INNER_ID}:{}:1",
            identity.host_gid
        ))
        .arg(format!("--setuid={WORKLOAD_INNER_ID}"))
        .arg(format!("--setgid={WORKLOAD_INNER_ID}"))
        .args(["--", "/bin/sh", "-c", script])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// A permissive tree shaped like a control-plane root (0755 directories,
/// 0644 files), as ADV-A07 found it.
fn permissive_tree(base: &Path) -> PathBuf {
    let root = base.join("cell");
    let run = root.join("runs").join("r1");
    for dir in [&root, &root.join("runs"), &run] {
        fs::create_dir_all(dir).unwrap();
    }
    for tree in ["workspace", "artifacts", "mcp"] {
        fs::create_dir_all(run.join(tree)).unwrap();
    }
    fs::write(root.join("ledger.jsonl"), "control-plane-record\n").unwrap();
    fs::write(run.join("transcript.jsonl"), "run-record\n").unwrap();
    for path in [
        root.clone(),
        root.join("runs"),
        run.clone(),
        run.join("workspace"),
        run.join("artifacts"),
        run.join("mcp"),
    ] {
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    for file in [root.join("ledger.jsonl"), run.join("transcript.jsonl")] {
        fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
    }
    // The base must be traversable so only the tree's own modes decide.
    fs::set_permissions(base, fs::Permissions::from_mode(0o711)).unwrap();
    root
}

fn probe(root: &Path) -> String {
    format!(
        "cat {root}/ledger.jsonl 2>/dev/null; cat {root}/runs/r1/transcript.jsonl 2>/dev/null; \
         ls {root}/runs 2>/dev/null",
        root = root.display()
    )
}

fn mode(path: &Path) -> u32 {
    fs::symlink_metadata(path).unwrap().mode() & 0o7777
}

#[test]
fn unhardened_tree_is_readable_by_another_workload_mutation_control() {
    let Some(pool) = pool() else { return };
    let base = tempfile::tempdir().unwrap();
    let root = permissive_tree(base.path());
    let other = pool.acquire().unwrap();
    let seen = as_identity(&other, &probe(&root));
    // Proves the attack below is live: without hardening, DAC alone leaks.
    assert!(seen.contains("control-plane-record"), "{seen}");
    assert!(seen.contains("run-record"), "{seen}");
}

#[test]
fn hardened_tree_denies_another_workload_by_dac_alone() {
    let Some(pool) = pool() else { return };
    let base = tempfile::tempdir().unwrap();
    let root = permissive_tree(base.path());
    let traverse = [root.clone(), root.join("runs")];
    let report = harden_state_tree(&root, &traverse).unwrap();
    assert!(report.skipped.is_empty(), "{report:?}");
    assert_eq!(mode(&root), 0o711);
    assert_eq!(mode(&root.join("runs")), 0o711);
    assert_eq!(mode(&root.join("runs/r1")), 0o700);
    assert_eq!(mode(&root.join("ledger.jsonl")), 0o600);
    assert_eq!(mode(&root.join("runs/r1/transcript.jsonl")), 0o600);
    for tree in ["workspace", "artifacts", "mcp"] {
        assert_eq!(mode(&root.join("runs/r1").join(tree)), 0o700, "{tree}");
    }
    let other = pool.acquire().unwrap();
    let seen = as_identity(&other, &probe(&root));
    assert!(!seen.contains("control-plane-record"), "{seen}");
    assert!(!seen.contains("run-record"), "{seen}");
    assert!(
        !seen.contains("r1"),
        "traverse-only root must not list: {seen}"
    );
    // The launcher keeps its own access.
    assert_eq!(
        fs::read_to_string(root.join("ledger.jsonl")).unwrap(),
        "control-plane-record\n"
    );
}

#[test]
fn granted_workload_reaches_only_its_own_tree() {
    let Some(pool) = pool() else { return };
    let base = tempfile::tempdir().unwrap();
    let root = permissive_tree(base.path());
    let run = root.join("runs/r1");
    harden_state_tree(&root, &[root.clone(), root.join("runs")]).unwrap();
    prepare_traversal(&[root.clone(), root.join("runs"), run.clone()]).unwrap();
    let owner = pool.acquire().unwrap();
    let other = pool.acquire().unwrap();
    grant_exclusive_access(&run.join("workspace"), &owner).unwrap();

    let workspace = run.join("workspace");
    let wrote = as_identity(
        &owner,
        &format!(
            "echo owner-data > {w}/out && cat {w}/out; cat {r}/transcript.jsonl 2>/dev/null; \
             ls {r} 2>/dev/null",
            w = workspace.display(),
            r = run.display()
        ),
    );
    // Positive control: legitimate access through the traverse-only chain.
    assert!(wrote.contains("owner-data"), "{wrote}");
    assert!(!wrote.contains("run-record"), "{wrote}");
    assert!(
        !wrote.contains("transcript"),
        "run dir must not list: {wrote}"
    );

    let stolen = as_identity(
        &other,
        &format!("cat {}/out 2>/dev/null", workspace.display()),
    );
    assert!(!stolen.contains("owner-data"), "{stolen}");
}

#[test]
fn hardening_never_follows_or_changes_symlinks() {
    let Some(_pool) = pool() else { return };
    let base = tempfile::tempdir().unwrap();
    let root = permissive_tree(base.path());
    let outside = base.path().join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("host-file"), "x").unwrap();
    fs::set_permissions(&outside, fs::Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(outside.join("host-file"), fs::Permissions::from_mode(0o644)).unwrap();
    std::os::unix::fs::symlink(outside.join("host-file"), root.join("file-link")).unwrap();
    std::os::unix::fs::symlink(&outside, root.join("runs/dir-link")).unwrap();

    let report = harden_state_tree(&root, &[]).unwrap();
    assert_eq!(mode(&outside), 0o755, "symlinked directory was changed");
    assert_eq!(
        mode(&outside.join("host-file")),
        0o644,
        "symlinked file was changed"
    );
    let skipped: Vec<_> = report
        .skipped
        .iter()
        .map(|(p, why)| (p.clone(), why.as_str()))
        .collect();
    assert!(
        skipped.contains(&(root.join("file-link"), "symlink")),
        "{skipped:?}"
    );
    assert!(
        skipped.contains(&(root.join("runs/dir-link"), "symlink")),
        "{skipped:?}"
    );
}

#[test]
fn symlinked_state_root_is_refused() {
    let base = tempfile::tempdir().unwrap();
    let real = base.path().join("real");
    fs::create_dir(&real).unwrap();
    fs::set_permissions(&real, fs::Permissions::from_mode(0o755)).unwrap();
    let link = base.path().join("link");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    assert!(harden_state_tree(&link, &[]).is_err());
    assert!(prepare_traversal(&[link]).is_err());
    assert_eq!(mode(&real), 0o755);
}

#[test]
fn creation_mask_makes_new_state_owner_only() {
    // A child process, so the process-wide umask does not leak into other tests.
    if std::env::var_os("STATE_TREE_UMASK_CHILD").is_some() {
        sea_forge_sandbox::workload::restrict_creation_mask();
        let dir = PathBuf::from(std::env::var_os("STATE_TREE_UMASK_DIR").unwrap());
        fs::create_dir(dir.join("d")).unwrap();
        fs::write(dir.join("f"), "x").unwrap();
        return;
    }
    let base = tempfile::tempdir().unwrap();
    let status = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "creation_mask_makes_new_state_owner_only",
            "--nocapture",
        ])
        .env("STATE_TREE_UMASK_CHILD", "1")
        .env("STATE_TREE_UMASK_DIR", base.path())
        .status()
        .unwrap();
    assert!(status.success());
    assert_eq!(mode(&base.path().join("d")), 0o700);
    assert_eq!(mode(&base.path().join("f")), 0o600);
}
