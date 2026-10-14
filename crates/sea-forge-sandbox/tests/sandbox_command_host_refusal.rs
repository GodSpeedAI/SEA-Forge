//! A sandbox command is authorized only for OpenShell. The host-side backends
//! (local and jail) must refuse it before anything is spawned.

use sea_forge_core::types::{ExecutionRequest, Operation};
#[cfg(target_os = "linux")]
use sea_forge_sandbox::JailSandbox;
use sea_forge_sandbox::{ExecutionSandbox, LocalSandbox, NetworkPosture, SandboxSpec};
use std::collections::BTreeMap;
use std::path::Path;

/// A sandbox-command request whose argv would create `marker` if it ran.
fn touch_request(marker: &Path) -> ExecutionRequest {
    ExecutionRequest {
        plan_item_id: "item_sandbox".into(),
        operation: Operation::ExecuteSandboxCommand {
            argv: vec![
                "/bin/sh".into(),
                "-c".into(),
                format!("touch {}", marker.display()),
            ],
            cwd: ".".into(),
            cell_id: "cell_test".into(),
            model_hash: "sha256:model".into(),
            policy_digest: "sha256:policy".into(),
            image: "registry.example/cell:1".into(),
        },
        timeout_secs: 10,
        env: BTreeMap::new(),
        compensating_controls: vec![],
    }
}

fn spec(root: &Path) -> SandboxSpec {
    SandboxSpec {
        workspace_root: root.join("workspace"),
        artifacts_root: root.join("artifacts"),
        network: NetworkPosture::default(),
    }
}

#[test]
fn local_backend_refuses_sandbox_commands() {
    let dir = tempfile::tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    let artifacts = dir.path().join("artifacts");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&artifacts).unwrap();
    let marker = dir.path().join("local-marker");
    let sandbox = LocalSandbox;
    let handle = sandbox.prepare(&spec(dir.path())).unwrap();

    let result = sandbox.execute(&handle, &touch_request(&marker));

    assert!(
        result.is_err(),
        "local backend must refuse a sandbox command"
    );
    assert!(!marker.exists(), "refused command must not have run");
}

#[cfg(target_os = "linux")]
#[test]
fn jail_backend_refuses_sandbox_commands() {
    let jail = match JailSandbox::new() {
        Ok(jail) => jail,
        Err(e) => {
            eprintln!("Skipping jail refusal test: {e}");
            return;
        }
    };
    let dir = tempfile::tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    let artifacts = dir.path().join("artifacts");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&artifacts).unwrap();
    let marker = dir.path().join("jail-marker");
    let handle = jail.prepare(&spec(dir.path())).unwrap();

    let result = jail.execute(&handle, &touch_request(&marker));
    let _ = jail.destroy(handle);

    assert!(
        result.is_err(),
        "jail backend must refuse a sandbox command"
    );
    assert!(!marker.exists(), "refused command must not have run");
}
