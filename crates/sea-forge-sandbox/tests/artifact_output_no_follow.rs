//! Artifact output files are launcher-owned: a workload that plants entries
//! under `artifacts/` must not redirect the host's writes, and a background
//! child must not outlive a completed execution. Checked on the local backend,
//! whose output-file creation is shared with the Unix jails.
#![cfg(unix)]

use sea_forge_core::types::{ExecutionRequest, ExecutionStatus, Operation};
use sea_forge_sandbox::{ExecutionSandbox, LocalSandbox, NetworkPosture, SandboxSpec};
use std::{
    collections::BTreeMap, fs, os::unix::fs::symlink, path::PathBuf, thread, time::Duration,
};

const CANARY: &str = "host canary: must not change\n";

struct Fixture {
    _root: tempfile::TempDir,
    workspace: PathBuf,
    artifacts: PathBuf,
    canary: PathBuf,
}

fn fixture() -> Fixture {
    let root = tempfile::tempdir().expect("temp root");
    let base = root.path().to_path_buf();
    let fx = Fixture {
        workspace: base.join("workspace"),
        artifacts: base.join("artifacts"),
        canary: base.join("outside-canary.txt"),
        _root: root,
    };
    fs::create_dir_all(&fx.workspace).expect("workspace");
    fs::create_dir_all(&fx.artifacts).expect("artifacts");
    fs::write(&fx.canary, CANARY).expect("canary");
    fx
}

fn run(fx: &Fixture, script: &str, timeout_secs: u64) -> sea_forge_core::types::ExecutionResult {
    let spec = SandboxSpec {
        workspace_root: fx.workspace.clone(),
        artifacts_root: fx.artifacts.clone(),
        network: NetworkPosture::Denied,
    };
    let sandbox = LocalSandbox;
    let handle = sandbox.prepare(&spec).expect("prepare");
    let request = ExecutionRequest {
        plan_item_id: "artifact-output-no-follow".into(),
        operation: Operation::ExecuteCommand {
            argv: vec!["/bin/sh".into(), "-c".into(), script.into()],
            cwd: ".".into(),
        },
        timeout_secs,
        env: BTreeMap::from([("PATH".to_string(), "/usr/bin:/bin".to_string())]),
        compensating_controls: vec![],
    };
    sandbox.execute(&handle, &request).expect("execute")
}

#[test]
fn planted_symlink_does_not_redirect_stdout_write() {
    let fx = fixture();
    symlink(&fx.canary, fx.artifacts.join("stdout.txt")).expect("plant symlink");

    let result = run(&fx, "echo workload-output", 20);
    assert_eq!(result.status, ExecutionStatus::Completed);
    assert_eq!(fs::read_to_string(&fx.canary).unwrap(), CANARY);
    let stdout = fs::read_to_string(fx.artifacts.join("stdout.txt")).unwrap();
    assert_eq!(stdout, "workload-output\n");
}

#[test]
fn planted_hard_link_does_not_truncate_canary() {
    let fx = fixture();
    fs::hard_link(&fx.canary, fx.artifacts.join("stderr.txt")).expect("plant hard link");

    let result = run(&fx, "echo workload-error 1>&2", 20);
    assert_eq!(result.status, ExecutionStatus::Completed);
    assert_eq!(fs::read_to_string(&fx.canary).unwrap(), CANARY);
    let stderr = fs::read_to_string(fx.artifacts.join("stderr.txt")).unwrap();
    assert_eq!(stderr, "workload-error\n");
}

#[test]
fn background_child_does_not_outlive_completed_execution() {
    let fx = fixture();
    let result = run(&fx, "(sleep 3; echo late > late.txt) & exit 0", 20);
    assert_eq!(result.status, ExecutionStatus::Completed);

    thread::sleep(Duration::from_secs(5));
    assert!(
        !fx.workspace.join("late.txt").exists(),
        "background child wrote after the execution completed"
    );
}
