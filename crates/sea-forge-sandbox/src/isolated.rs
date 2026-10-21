//! `isolated` execution backend for planned commands (SEC-01/SEC-06).
//!
//! Each execution acquires one exclusive subordinate host identity, grants it
//! the run's workspace and artifacts trees, runs the command through the same
//! launch chain as isolated ACP agents (`setpriv` parent-death signal,
//! `unshare` single-ID user namespace with pid/mount/ipc/net namespaces,
//! `sea-forge-workload-exec` Landlock posture), then withdraws the grant
//! before the identity is released. Any missing prerequisite fails closed;
//! there is no fallback to a shared-UID class.
use crate::workload::{
    grant_exclusive_access, isolated_command, revoke_exclusive_access, sibling_helper,
    WorkloadConfinement, WorkloadIdentityPool,
};
use crate::{ExecutionSandbox, RelPath, SandboxClass, SandboxError, SandboxHandle, SandboxSpec};
use sea_forge_core::types::{ArtifactRef, ExecutionRequest, ExecutionResult, ExecutionStatus};
use std::path::PathBuf;

#[derive(Debug)]
pub struct IsolatedSandbox {
    pool: WorkloadIdentityPool,
    helper: PathBuf,
    read_paths: Vec<PathBuf>,
}

impl IsolatedSandbox {
    /// Probe the host prerequisites. `read_paths` are runtime trees the
    /// command may read besides the system set (already validated by the
    /// caller with [`crate::workload::validate_read_paths`]).
    pub fn new(read_paths: Vec<PathBuf>) -> Result<Self, SandboxError> {
        Ok(Self {
            pool: WorkloadIdentityPool::from_host()?,
            helper: sibling_helper()?,
            read_paths,
        })
    }
}

impl ExecutionSandbox for IsolatedSandbox {
    fn class(&self) -> SandboxClass {
        SandboxClass::Isolated
    }

    fn prepare(&self, spec: &SandboxSpec) -> Result<SandboxHandle, SandboxError> {
        Ok(SandboxHandle {
            class: SandboxClass::Isolated,
            spec: spec.clone(),
        })
    }

    fn execute(
        &self,
        h: &SandboxHandle,
        req: &ExecutionRequest,
    ) -> Result<ExecutionResult, SandboxError> {
        use sea_forge_core::types::Operation;
        use std::fs::{self, File};
        use std::process::Stdio;
        use std::time::Duration;
        use wait_timeout::ChildExt;

        fn map_err<E: std::fmt::Display>(e: E) -> SandboxError {
            SandboxError::new("workload_isolation_unavailable", e.to_string())
        }
        let (argv, cwd) = match &req.operation {
            Operation::ExecuteCommand { argv, cwd } if !argv.is_empty() => (argv, cwd),
            _ => {
                return Err(SandboxError::new(
                    "input_error",
                    "execution request requires non-empty execute_command",
                ));
            }
        };
        let cwd = crate::safe_join(&h.spec.workspace_root, cwd)
            .map_err(|e| SandboxError::new("unsafe_path_error", e.to_string()))?;
        fs::create_dir_all(&h.spec.workspace_root).map_err(map_err)?;
        fs::create_dir_all(&h.spec.artifacts_root).map_err(map_err)?;
        // Launcher-owned output files, opened before the workload exists.
        let stdout_path = h.spec.artifacts_root.join("stdout.txt");
        let stderr_path = h.spec.artifacts_root.join("stderr.txt");
        let stdout = File::create(&stdout_path).map_err(map_err)?;
        let stderr = File::create(&stderr_path).map_err(map_err)?;

        let identity = self.pool.acquire()?;
        let trees = [&h.spec.workspace_root, &h.spec.artifacts_root];
        for tree in trees {
            grant_exclusive_access(tree, &identity)?;
        }
        let confinement = WorkloadConfinement {
            workspace_root: h.spec.workspace_root.clone(),
            artifacts_root: h.spec.artifacts_root.clone(),
            read_paths: self.read_paths.clone(),
            // No pathname-socket grants on this path: default-deny (SEC-06).
            socket_paths: Vec::new(),
            network: h.spec.network.clone(),
        };
        let (mut command, lifeline) =
            isolated_command(&identity, &self.helper, &confinement, argv)?;
        command
            .current_dir(&cwd)
            .envs(req.env.iter())
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr));
        let started_at = chrono::Utc::now().to_rfc3339();
        let outcome = match command.spawn() {
            Err(_) => Ok((ExecutionStatus::SpawnFailed, None)),
            Ok(mut child) => match child
                .wait_timeout(Duration::from_secs(req.timeout_secs))
                .map_err(map_err)
            {
                Ok(Some(status)) => Ok((ExecutionStatus::Completed, status.code().map(i64::from))),
                Ok(None) => {
                    // Killing `unshare` kills the pid namespace's init and
                    // with it every workload process.
                    let _ = child.kill();
                    child
                        .wait()
                        .map(|_| (ExecutionStatus::TimedOut, None))
                        .map_err(map_err)
                }
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    Err(error)
                }
            },
        };
        // End anything the command left running, then withdraw the
        // identity's reach before its slot can be reissued.
        drop(lifeline);
        for tree in trees {
            revoke_exclusive_access(tree)?;
        }
        drop(identity);
        let (mut status, exit_code) = outcome?;
        // As for the jail: a nonzero exit plus a child-written denial is only
        // evidence of a confinement refusal, recorded as suspected.
        if status == ExecutionStatus::Completed && exit_code != Some(0) {
            use std::io::Read;
            let mut bytes = Vec::new();
            if let Ok(file) = File::open(&stderr_path) {
                let _ = file.take(65_537).read_to_end(&mut bytes);
            }
            if String::from_utf8_lossy(&bytes)
                .to_lowercase()
                .contains("permission denied")
            {
                status = ExecutionStatus::SuspectedSandboxViolation;
            }
        }
        Ok(ExecutionResult {
            status,
            exit_code,
            stdout_path: "artifacts/stdout.txt".into(),
            stderr_path: "artifacts/stderr.txt".into(),
            started_at,
            finished_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    fn collect_artifacts(
        &self,
        _h: &SandboxHandle,
        _paths: &[RelPath],
    ) -> Result<Vec<ArtifactRef>, SandboxError> {
        Err(SandboxError::new(
            "artifact_collection_unsupported",
            "collect_artifacts is not implemented for this sandbox backend",
        ))
    }

    fn destroy(&self, _h: SandboxHandle) -> Result<(), SandboxError> {
        Ok(())
    }
}
