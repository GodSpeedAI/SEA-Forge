use crate::{errors::ForgeError, sandbox::safe_join, types::*};
use chrono::Utc;
use std::{
    fs::{self, File},
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};
use wait_timeout::ChildExt;
pub fn execute(
    request: &ExecutionRequest,
    workspace: &Path,
    artifacts: &Path,
) -> Result<ExecutionResult, ForgeError> {
    let started_at = Utc::now().to_rfc3339();
    fs::create_dir_all(artifacts).map_err(|e| ForgeError::io("create artifacts", e))?;
    let stdout = File::create(artifacts.join("stdout.txt"))
        .map_err(|e| ForgeError::io("create stdout", e))?;
    let stderr = File::create(artifacts.join("stderr.txt"))
        .map_err(|e| ForgeError::io("create stderr", e))?;
    let (argv, cwd) = match &request.operation {
        Operation::ExecuteCommand { argv, cwd } if !argv.is_empty() => (argv, cwd),
        _ => {
            return Err(ForgeError::Internal(
                "execution request requires non-empty execute_command".into(),
            ))
        }
    };
    let cwd = safe_join(workspace, cwd)?;
    let mut command = Command::new(&argv[0]);
    command
        .args(&argv[1..])
        .current_dir(cwd)
        .env_clear()
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    for (k, v) in &request.env {
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
                finished_at: Utc::now().to_rfc3339(),
            })
        }
    };
    let status = child
        .wait_timeout(Duration::from_secs(request.timeout_secs))
        .map_err(|e| ForgeError::io("wait for child", e))?;
    let (state, code) = if let Some(status) = status {
        (ExecutionStatus::Completed, status.code().map(i64::from))
    } else {
        child
            .kill()
            .map_err(|e| ForgeError::io("kill timed out child", e))?;
        let _ = child.wait();
        (ExecutionStatus::TimedOut, None)
    };
    Ok(ExecutionResult {
        status: state,
        exit_code: code,
        stdout_path: "artifacts/stdout.txt".into(),
        stderr_path: "artifacts/stderr.txt".into(),
        started_at,
        finished_at: Utc::now().to_rfc3339(),
    })
}
