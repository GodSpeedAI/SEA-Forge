use crate::{ExecutionSandbox, RelPath, SandboxClass, SandboxError, SandboxHandle, SandboxSpec};
use sea_forge_core::types::{ArtifactRef, ExecutionRequest, ExecutionResult, ExecutionStatus};

pub struct JailSandbox;

/// Interactive child spawned after Landlock restriction. The caller owns the
/// stdio pipes and child lifecycle; no shell is involved.
pub struct InteractiveJailChild {
    pub child: std::process::Child,
    pub stdin: std::process::ChildStdin,
    pub stdout: std::process::ChildStdout,
    pub stderr: std::process::ChildStderr,
}

/// Restrict the current thread with the same Landlock posture as `JailSandbox`
/// and spawn an interactive child whose stdio remains connected to the caller.
/// Call this from a dedicated thread: Landlock restriction is irreversible for
/// that thread and inherited by the spawned child.
pub fn spawn_interactive(
    workspace_root: &std::path::Path,
    artifacts_root: &std::path::Path,
    argv: &[String],
    cwd: &std::path::Path,
    env: &[(String, String)],
) -> Result<InteractiveJailChild, SandboxError> {
    #[cfg(target_os = "linux")]
    {
        use landlock::{
            path_beneath_rules, Access, AccessFs, Ruleset, RulesetAttr, RulesetCreatedAttr,
            RulesetStatus, ABI,
        };
        use std::{os::unix::process::CommandExt, process::Stdio};

        fn map_err<E: std::fmt::Display>(error: E) -> SandboxError {
            SandboxError::new("jail_unavailable", error.to_string())
        }
        if argv.is_empty() {
            return Err(SandboxError::new("input_error", "ACP argv is empty"));
        }
        std::fs::create_dir_all(workspace_root).map_err(map_err)?;
        std::fs::create_dir_all(artifacts_root).map_err(map_err)?;
        let abi = ABI::V1;
        let status = Ruleset::default()
            .handle_access(AccessFs::from_all(abi))
            .map_err(map_err)?
            .create()
            .map_err(map_err)?
            .add_rules(path_beneath_rules(
                [workspace_root, artifacts_root],
                AccessFs::from_all(abi),
            ))
            .map_err(map_err)?
            .add_rules(path_beneath_rules(
                [std::path::Path::new("/")],
                AccessFs::from_read(abi),
            ))
            .map_err(map_err)?
            .restrict_self()
            .map_err(map_err)?;
        if status.ruleset == RulesetStatus::NotEnforced {
            return Err(SandboxError::new(
                "jail_unavailable",
                "Landlock restrictions are not enforced by this kernel",
            ));
        }
        let mut command = std::process::Command::new(&argv[0]);
        command
            .args(&argv[1..])
            .current_dir(cwd)
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        for (key, value) in env {
            command.env(key, value);
        }
        let mut child = command.spawn().map_err(map_err)?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| SandboxError::new("jail_unavailable", "child stdin unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| SandboxError::new("jail_unavailable", "child stdout unavailable"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| SandboxError::new("jail_unavailable", "child stderr unavailable"))?;
        Ok(InteractiveJailChild {
            child,
            stdin,
            stdout,
            stderr,
        })
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (workspace_root, artifacts_root, argv, cwd, env);
        Err(SandboxError::new(
            "unsupported_sandbox_class_error",
            "interactive jail is unavailable on this platform",
        ))
    }
}

impl JailSandbox {
    pub fn new() -> Result<Self, SandboxError> {
        #[cfg(target_os = "linux")]
        {
            Ok(JailSandbox)
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(SandboxError::new(
                "unsupported_sandbox_class_error",
                "jail backend is unavailable on this platform",
            ))
        }
    }
}

impl ExecutionSandbox for JailSandbox {
    fn class(&self) -> SandboxClass {
        SandboxClass::Jail
    }

    fn prepare(&self, spec: &SandboxSpec) -> Result<SandboxHandle, SandboxError> {
        Ok(SandboxHandle {
            class: SandboxClass::Jail,
            spec: spec.clone(),
        })
    }

    fn execute(
        &self,
        h: &SandboxHandle,
        req: &ExecutionRequest,
    ) -> Result<ExecutionResult, SandboxError> {
        #[cfg(target_os = "linux")]
        {
            execute_linux(h, req)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (h, req);
            Err(SandboxError::new(
                "unsupported_sandbox_class_error",
                "jail backend is unavailable on this platform",
            ))
        }
    }

    fn collect_artifacts(
        &self,
        _h: &SandboxHandle,
        _paths: &[RelPath],
    ) -> Result<Vec<ArtifactRef>, SandboxError> {
        Ok(Vec::new())
    }

    fn destroy(&self, _h: SandboxHandle) -> Result<(), SandboxError> {
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn execute_linux(
    h: &SandboxHandle,
    req: &ExecutionRequest,
) -> Result<ExecutionResult, SandboxError> {
    use sea_forge_core::types::Operation;
    use std::{
        fs::{self, File},
        sync::mpsc,
        thread,
    };

    fn map_err<E: std::fmt::Display>(e: E) -> SandboxError {
        SandboxError::new("jail_unavailable", e.to_string())
    }

    fs::create_dir_all(&h.spec.artifacts_root).map_err(map_err)?;

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

    let stdout_path = h.spec.artifacts_root.join("stdout.txt");
    let stderr_path = h.spec.artifacts_root.join("stderr.txt");
    let stdout = File::create(&stdout_path).map_err(map_err)?;
    let stderr = File::create(&stderr_path).map_err(map_err)?;

    let workspace_root = h.spec.workspace_root.clone();
    let artifacts_root = h.spec.artifacts_root.clone();
    let argv: Vec<String> = argv.clone();
    let env: Vec<(String, String)> = req
        .env
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let timeout_secs = req.timeout_secs;

    // Landlock restrictions are per-thread and inherited by child processes.
    // We run the command on a dedicated thread that restricts itself before
    // spawning the child, keeping the main thread unrestricted.
    let (tx, rx) = mpsc::channel::<Result<ExecutionResult, SandboxError>>();

    thread::spawn(move || {
        let result = run_jailed(
            &workspace_root,
            &artifacts_root,
            &argv,
            &cwd,
            &env,
            timeout_secs,
            stdout,
            stderr,
        );
        let _ = tx.send(result);
    });

    let result = rx
        .recv()
        .map_err(|e| SandboxError::new("internal_error", e.to_string()))??;

    // Heuristic: detect jail violation from non-zero exit + "Permission denied".
    let mut result = result;
    if result.status == ExecutionStatus::Completed && result.exit_code != Some(0) {
        let stderr_text = fs::read_to_string(&stderr_path).unwrap_or_default();
        if stderr_text.to_lowercase().contains("permission denied") {
            result.status = ExecutionStatus::SandboxViolation;
        }
    }
    Ok(result)
}

#[cfg(target_os = "linux")]
#[allow(clippy::too_many_arguments)]
fn run_jailed(
    workspace_root: &std::path::Path,
    artifacts_root: &std::path::Path,
    argv: &[String],
    cwd: &std::path::Path,
    env: &[(String, String)],
    timeout_secs: u64,
    stdout: std::fs::File,
    stderr: std::fs::File,
) -> Result<ExecutionResult, SandboxError> {
    use landlock::{
        path_beneath_rules, Access, AccessFs, Ruleset, RulesetAttr, RulesetCreatedAttr,
        RulesetStatus, ABI,
    };
    use std::{
        os::unix::process::CommandExt,
        process::{Command, Stdio},
        time::Duration,
    };
    use wait_timeout::ChildExt;

    fn map_err<E: std::fmt::Display>(e: E) -> SandboxError {
        SandboxError::new("jail_unavailable", e.to_string())
    }

    let abi = ABI::V1;
    let status = Ruleset::default()
        .handle_access(AccessFs::from_all(abi))
        .map_err(map_err)?
        .create()
        .map_err(map_err)?
        // Read-write for workspace and artifacts.
        .add_rules(path_beneath_rules(
            [workspace_root, artifacts_root],
            AccessFs::from_all(abi),
        ))
        .map_err(map_err)?
        // Read-only for the rest of the filesystem so the child can load
        // its interpreter and libraries.
        .add_rules(path_beneath_rules(
            [std::path::Path::new("/")],
            AccessFs::from_read(abi),
        ))
        .map_err(map_err)?
        .restrict_self()
        .map_err(map_err)?;

    if status.ruleset == RulesetStatus::NotEnforced {
        return Err(SandboxError::new(
            "jail_unavailable",
            "Landlock restrictions are not enforced by this kernel",
        ));
    }

    let mut command = Command::new(&argv[0]);
    command
        .args(&argv[1..])
        .current_dir(cwd)
        .env_clear()
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    command.process_group(0);
    for (k, v) in env {
        command.env(k, v);
    }

    let started_at = chrono::Utc::now().to_rfc3339();
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
            });
        }
    };

    let status = child
        .wait_timeout(Duration::from_secs(timeout_secs))
        .map_err(map_err)?;

    let (state, code) = if let Some(status) = status {
        (ExecutionStatus::Completed, status.code().map(i64::from))
    } else {
        let group = format!("-{}", child.id());
        let _ = Command::new("/bin/kill")
            .args(["-KILL", "--", &group])
            .env_clear()
            .output();
        child.wait().map_err(map_err)?;
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
