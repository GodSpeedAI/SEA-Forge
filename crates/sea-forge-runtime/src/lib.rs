use sea_forge_core::{errors::ForgeError, types::*};
#[cfg(test)]
use sea_forge_sandbox::ExecutionSandbox;
use sea_forge_sandbox::{select_sandbox, NetworkPosture, SandboxClass, SandboxSpec};
use std::path::Path;

pub fn execute(
    grant: sea_forge_authority::ActionGrant,
    request: &ExecutionRequest,
    run_id: &str,
    workspace: &Path,
    artifacts: &Path,
) -> Result<ExecutionResult, ForgeError> {
    let sandbox_class: SandboxClass =
        grant
            .sandbox_class()
            .parse()
            .map_err(|e: ForgeError| ForgeError::Config {
                class: "unsupported_sandbox_class_error",
                path: workspace.into(),
                message: e.to_string(),
            })?;
    // Derive the network posture from authority (the grant's `network`
    // boundary) before the grant is consumed by `authorize_execution`. Never
    // from child-controlled request input.
    let network = NetworkPosture::from_granted_ports(grant.network_tcp_ports());

    // F-25.h: capture the decision-time executable identity, then re-resolve
    // argv[0] immediately before the spawn. A binary swapped in between the
    // authority decision and this point no longer matches and is refused —
    // shrinking the old decision→spawn window to a same-instant race.
    let resolved_executable = grant.resolved_executable().cloned();
    grant.authorize_execution(
        &sea_forge_core::types::AuthorityAction::from(&request.operation),
        sea_forge_authority::ExecutionGrantContext {
            run_id,
            plan_item_id: &request.plan_item_id,
            workspace_root: workspace,
            artifacts_root: artifacts,
            timeout_secs: request.timeout_secs,
            env_keys: request.env.keys().cloned().collect(),
            compensating_controls: &request.compensating_controls,
        },
    )?;
    if let sea_forge_core::types::Operation::ExecuteCommand { argv, .. } = &request.operation {
        match (&resolved_executable, argv.first()) {
            (Some(bound), Some(argv0)) => {
                let current = std::fs::canonicalize(argv0).map_err(|e| {
                    ForgeError::Input(format!(
                        "authority grant executable is unresolvable at spawn: {e}"
                    ))
                })?;
                if current != *bound {
                    return Err(ForgeError::Input(
                        "authority grant executable identity changed since the decision".into(),
                    ));
                }
            }
            _ => {
                return Err(ForgeError::Input(
                    "authority grant carries no executable identity for an execute_command".into(),
                ));
            }
        }
    }

    let sandbox = select_sandbox(sandbox_class).map_err(|e: sea_forge_sandbox::SandboxError| {
        ForgeError::Config {
            class: e.class,
            path: workspace.into(),
            message: e.message,
        }
    })?;
    let spec = SandboxSpec {
        workspace_root: workspace.to_path_buf(),
        artifacts_root: artifacts.to_path_buf(),
        network,
    };
    let handle = sandbox.prepare(&spec).map_err(|e| ForgeError::Config {
        class: e.class,
        path: workspace.into(),
        message: e.message,
    })?;
    let result = sandbox
        .execute(&handle, request)
        .map_err(|e| ForgeError::Config {
            class: e.class,
            path: workspace.into(),
            message: e.message,
        })?;
    let _ = sandbox.destroy(handle);
    Ok(result)
}

/// Test and local-shortcut helper: execute with the local sandbox.
#[cfg(test)]
fn execute_authorized(
    request: &ExecutionRequest,
    workspace: &Path,
    artifacts: &Path,
) -> Result<ExecutionResult, ForgeError> {
    let sandbox = sea_forge_sandbox::LocalSandbox;
    let spec = SandboxSpec {
        workspace_root: workspace.to_path_buf(),
        artifacts_root: artifacts.to_path_buf(),
        network: NetworkPosture::default(),
    };
    let handle = sandbox.prepare(&spec)?;
    let result = sandbox.execute(&handle, request)?;
    let _ = sandbox.destroy(handle);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::fs;
    use std::process::Command;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    fn directories(name: &str) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let parent = std::env::temp_dir().join(format!("sea-forge-runtime-{name}-{nonce}"));
        let workspace = parent.join("workspace");
        let artifacts = parent.join("artifacts");
        fs::create_dir_all(&workspace).unwrap();
        fs::create_dir_all(&artifacts).unwrap();
        (parent, workspace, artifacts)
    }
    fn request(argv: Vec<String>, timeout_secs: u64) -> ExecutionRequest {
        let mut env = BTreeMap::new();
        env.insert("PATH".into(), std::env::var("PATH").unwrap_or_default());
        env.insert("HOME".into(), "/tmp/sea-forge-home".into());
        ExecutionRequest {
            plan_item_id: "item_01".into(),
            operation: Operation::ExecuteCommand {
                argv,
                cwd: ".".into(),
            },
            timeout_secs,
            env,
            compensating_controls: vec![],
        }
    }

    #[test]
    fn child_environment_is_minimal() {
        let (parent, workspace, artifacts) = directories("env");
        let result =
            execute_authorized(&request(vec!["env".into()], 5), &workspace, &artifacts).unwrap();
        assert_eq!(result.status, ExecutionStatus::Completed);
        let output = fs::read_to_string(artifacts.join("stdout.txt")).unwrap();
        let mut names: Vec<_> = output
            .lines()
            .filter_map(|line| line.split_once('=').map(|v| v.0))
            .collect();
        names.sort_unstable();
        assert_eq!(names, ["HOME", "PATH"]);
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn timeout_kills_and_reaps_child() {
        let (parent, workspace, artifacts) = directories("timeout");
        let started = Instant::now();
        let result = execute_authorized(
            &request(vec!["sleep".into(), "30".into()], 0),
            &workspace,
            &artifacts,
        )
        .unwrap();
        assert_eq!(result.status, ExecutionStatus::TimedOut);
        assert!(started.elapsed().as_secs() < 10);
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn spawn_failure_is_an_execution_result() {
        let (parent, workspace, artifacts) = directories("spawn");
        let result = execute_authorized(
            &request(vec!["sea-forge-command-does-not-exist".into()], 1),
            &workspace,
            &artifacts,
        )
        .unwrap();
        assert_eq!(result.status, ExecutionStatus::SpawnFailed);
        fs::remove_dir_all(parent).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn timed_out_child_pid_is_no_longer_alive() {
        let (parent, workspace, artifacts) = directories("reaped");
        let pid_file = parent.join("pid");
        let mut request = request(
            vec![
                std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                "--exact".into(),
                "tests::timeout_child_helper".into(),
            ],
            1,
        );
        request
            .env
            .insert("SEA_FORGE_TIMEOUT_HELPER".into(), "1".into());
        request.env.insert(
            "SEA_FORGE_TIMEOUT_PID_FILE".into(),
            pid_file.to_string_lossy().into_owned(),
        );
        let result = execute_authorized(&request, &workspace, &artifacts).unwrap();
        assert_eq!(result.status, ExecutionStatus::TimedOut);
        let pids = fs::read_to_string(&pid_file).unwrap();
        for pid in pids.split_whitespace() {
            let mut alive = true;
            for _ in 0..40 {
                alive = Command::new("kill")
                    .args(["-0", pid])
                    .output()
                    .unwrap()
                    .status
                    .success();
                if !alive {
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            assert!(!alive, "timed-out process {pid} survived");
        }
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn timeout_child_helper() {
        if std::env::var("SEA_FORGE_TIMEOUT_HELPER").as_deref() != Ok("1") {
            return;
        }
        let path = std::env::var("SEA_FORGE_TIMEOUT_PID_FILE").unwrap();
        let mut child = Command::new("sleep").arg("30").spawn().unwrap();
        fs::write(path, format!("{} {}", std::process::id(), child.id())).unwrap();
        let _ = child.wait();
    }
}
