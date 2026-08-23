use crate::{ExecutionSandbox, RelPath, SandboxClass, SandboxError, SandboxHandle, SandboxSpec};
use sea_forge_core::types::{ArtifactRef, ExecutionRequest, ExecutionResult, ExecutionStatus};
use std::{
    fs::{self, File},
    process::{Command, Stdio},
    time::Duration,
};
use wait_timeout::ChildExt;

pub struct LocalSandbox;

impl ExecutionSandbox for LocalSandbox {
    fn class(&self) -> SandboxClass {
        SandboxClass::Local
    }

    fn prepare(&self, spec: &SandboxSpec) -> Result<SandboxHandle, SandboxError> {
        Ok(SandboxHandle {
            class: SandboxClass::Local,
            spec: spec.clone(),
        })
    }

    fn execute(
        &self,
        h: &SandboxHandle,
        req: &ExecutionRequest,
    ) -> Result<ExecutionResult, SandboxError> {
        use sea_forge_core::types::Operation;
        use std::os::unix::process::CommandExt;

        let started_at = chrono::Utc::now().to_rfc3339();
        fs::create_dir_all(&h.spec.artifacts_root)
            .map_err(|e| SandboxError::new("io_error", e.to_string()))?;
        let stdout = File::create(h.spec.artifacts_root.join("stdout.txt"))
            .map_err(|e| SandboxError::new("io_error", e.to_string()))?;
        let stderr = File::create(h.spec.artifacts_root.join("stderr.txt"))
            .map_err(|e| SandboxError::new("io_error", e.to_string()))?;

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
        let mut command = Command::new(&argv[0]);
        command
            .args(&argv[1..])
            .current_dir(cwd)
            .env_clear()
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr));
        command.process_group(0);
        for (k, v) in &req.env {
            command.env(k, v);
        }
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(_) => {
                return Ok(ExecutionResult {
                    status: ExecutionStatus::SpawnFailed,
                    exit_code: None,
                    stdout_path: "artifacts/stdout.txt".into(),
                    stderr_path: "artifacts/stderr.txt".into(),
                    started_at,
                    finished_at: chrono::Utc::now().to_rfc3339(),
                })
            }
        };
        let status = child
            .wait_timeout(Duration::from_secs(req.timeout_secs))
            .map_err(|e| SandboxError::new("io_error", e.to_string()))?;
        let (state, code) = if let Some(status) = status {
            (ExecutionStatus::Completed, status.code().map(i64::from))
        } else {
            let group = format!("-{}", child.id());
            let group_kill = Command::new("/bin/kill")
                .args(["-KILL", "--", &group])
                .env_clear()
                .output()
                .map_err(|e| SandboxError::new("io_error", e.to_string()))?;
            if !group_kill.status.success() {
                let group_still_exists = Command::new("/bin/kill")
                    .args(["-0", "--", &group])
                    .env_clear()
                    .output()
                    .map_err(|e| SandboxError::new("io_error", e.to_string()))?
                    .status
                    .success();
                if group_still_exists {
                    return Err(SandboxError::new(
                        "internal_error",
                        "timed out process group survived termination",
                    ));
                }
            }
            child
                .wait()
                .map_err(|e| SandboxError::new("io_error", e.to_string()))?;
            (ExecutionStatus::TimedOut, None)
        };
        Ok(ExecutionResult {
            status: state,
            exit_code: code,
            stdout_path: "artifacts/stdout.txt".into(),
            stderr_path: "artifacts/stderr.txt".into(),
            started_at,
            finished_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    // F-25.k: this backend does not implement artifact collection. Returning
    // an empty vec silently would make a future caller mistake "unsupported"
    // for "nothing was requested", so the stub fails closed instead.
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
