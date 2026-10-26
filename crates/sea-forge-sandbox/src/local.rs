use crate::{ExecutionSandbox, RelPath, SandboxClass, SandboxError, SandboxHandle, SandboxSpec};
use sea_forge_core::types::{ArtifactRef, ExecutionRequest, ExecutionResult, ExecutionStatus};
use std::{
    fs,
    process::{Command, Stdio},
    time::Duration,
};
use wait_timeout::ChildExt;

pub struct LocalSandbox;

/// The execution record for a run that reached the workload or failed to
/// start, pointing at the standard artifact files.
pub(crate) fn execution_result(
    status: ExecutionStatus,
    exit_code: Option<i64>,
    started_at: String,
) -> ExecutionResult {
    ExecutionResult {
        status,
        exit_code,
        stdout_path: "artifacts/stdout.txt".into(),
        stderr_path: "artifacts/stderr.txt".into(),
        started_at,
        finished_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// Kills the child's process group (Unix). The group id is the child pid.
/// Called on timeout, and after normal completion so that background
/// descendants left in the group cannot outlive the execution.
#[cfg(unix)]
pub(crate) fn kill_timed_out_group(child: &std::process::Child) -> Result<(), SandboxError> {
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
    Ok(())
}

/// Bounded wait for teardown kills to take effect (mirrors the Windows
/// `terminate_and_wait` deadline). Linux-only: it is consumed by the
/// cgroup-leaf teardown; the other-Unix teardown is a bare group kill.
#[cfg(target_os = "linux")]
const TEARDOWN_DEADLINE: Duration = Duration::from_secs(5);

/// F08 teardown (Linux): kill through the run's cgroup leaf first — atomic,
/// and it reaches `setsid` descendants regardless of their session — then
/// the process-group kill as the secondary, then remove the leaf, which the
/// kernel only allows once it is empty. Every failure propagates: a survivor
/// can never masquerade as a settled run.
#[cfg(target_os = "linux")]
pub(crate) fn teardown_run(
    child: &std::process::Child,
    lease: Option<crate::cgroup::CgroupLease>,
) -> Result<(), SandboxError> {
    if let Some(lease) = &lease {
        lease.kill_verified(TEARDOWN_DEADLINE).map_err(|err| {
            let leaf = lease.leaf().display();
            match err {
                crate::cgroup::KillError::Survivors(pids) => SandboxError::new(
                    "internal_error",
                    format!("cgroup leaf {leaf} retained processes after kill: {pids:?}"),
                ),
                crate::cgroup::KillError::Io(e) => SandboxError::new(
                    "internal_error",
                    format!("cgroup leaf {leaf} kill failed: {e}"),
                ),
            }
        })?;
    }
    // Secondary: the group kill composes with containment — anything that
    // somehow sits outside the leaf but inside the group is still reached.
    kill_timed_out_group(child)?;
    if let Some(lease) = lease {
        lease.remove().map_err(|e| {
            SandboxError::new("internal_error", format!("cgroup leaf cleanup failed: {e}"))
        })?;
    }
    Ok(())
}

/// F08 teardown (other Unix): process-group kill only; failures propagate
/// instead of being discarded on the completion path.
#[cfg(all(unix, not(target_os = "linux")))]
pub(crate) fn teardown_run(child: &std::process::Child) -> Result<(), SandboxError> {
    kill_timed_out_group(child)
}

/// Terminates the timed-out child's job (Windows) and waits until it is empty.
/// Termination is asynchronous; survivors are reported only after the wait.
#[cfg(windows)]
fn kill_timed_out_job(job: &sea_forge_os::Job) -> Result<(), SandboxError> {
    job.terminate_and_wait(std::time::Duration::from_secs(5))
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::TimedOut => SandboxError::new(
                "internal_error",
                "timed out process group survived termination",
            ),
            _ => SandboxError::new("internal_error", e.to_string()),
        })
}

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
        let started_at = chrono::Utc::now().to_rfc3339();
        fs::create_dir_all(&h.spec.artifacts_root)
            .map_err(|e| SandboxError::new("io_error", e.to_string()))?;
        let stdout =
            crate::artifact_output::create_output_file(&h.spec.artifacts_root.join("stdout.txt"))
                .map_err(|e| SandboxError::new("io_error", e.to_string()))?;
        let stderr =
            crate::artifact_output::create_output_file(&h.spec.artifacts_root.join("stderr.txt"))
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
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        // F08: acquire this run's cgroup lifecycle leaf before spawning.
        // Under SEA_FORGE_REQUIRE_CGROUP=1 an unavailable cgroup refuses the
        // run (`cgroup_unavailable`, fail closed); otherwise the typed
        // degrade is surfaced and the process-group kill remains. The child
        // joins the leaf pre-exec (no uncontained window), composing with
        // the group setup above.
        #[cfg(target_os = "linux")]
        let lease = {
            let acquired = crate::cgroup::acquire_spawn_lease("local", std::process::id())?;
            crate::cgroup::attach_lease(&mut command, acquired, "local")?
        };
        for (k, v) in &req.env {
            command.env(k, v);
        }
        let spawn_failed =
            || execution_result(ExecutionStatus::SpawnFailed, None, started_at.clone());
        // Windows: the child starts suspended and is in its kill-on-close job
        // before it runs, so no grandchild can escape the timeout kill.
        // Any failure here (job creation, spawn, assignment) means nothing
        // ran; it is reported as a spawn failure.
        #[cfg(windows)]
        let (mut child, job) = match sea_forge_os::spawn_in_job(&mut command) {
            Ok(pair) => pair,
            Err(_) => return Ok(spawn_failed()),
        };
        #[cfg(unix)]
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(_) => {
                // A pre-exec join failure means nothing exec'd (fail
                // closed); the leaf is empty and is removed before the
                // spawn failure is reported. A removal failure is loud — a
                // leaked leaf must not pass silently.
                #[cfg(target_os = "linux")]
                if let Some(lease) = lease {
                    lease.remove().map_err(|e| {
                        SandboxError::new(
                            "internal_error",
                            format!("cgroup leaf cleanup after spawn failure failed: {e}"),
                        )
                    })?;
                }
                return Ok(spawn_failed());
            }
        };
        let status = child
            .wait_timeout(Duration::from_secs(req.timeout_secs))
            .map_err(|e| SandboxError::new("io_error", e.to_string()))?;
        let (state, code) = if let Some(status) = status {
            // F08: leftover descendants must not outlive the run. Teardown
            // failures propagate (previously the completion-path result was
            // discarded here), so a survivor can never read as Completed.
            #[cfg(target_os = "linux")]
            teardown_run(&child, lease)?;
            #[cfg(all(unix, not(target_os = "linux")))]
            teardown_run(&child)?;
            (ExecutionStatus::Completed, status.code().map(i64::from))
        } else {
            #[cfg(target_os = "linux")]
            teardown_run(&child, lease)?;
            #[cfg(all(unix, not(target_os = "linux")))]
            teardown_run(&child)?;
            #[cfg(windows)]
            kill_timed_out_job(&job)?;
            child
                .wait()
                .map_err(|e| SandboxError::new("io_error", e.to_string()))?;
            (ExecutionStatus::TimedOut, None)
        };
        Ok(execution_result(state, code, started_at))
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
        // F08: verified teardown of any lifecycle leaves this process
        // leaked (spawn-failure windows, earlier teardown errors). On hosts
        // without containment there is nothing to verify and no leaf can
        // exist; failures propagate — destroy is not a silent no-op.
        #[cfg(target_os = "linux")]
        crate::cgroup::destroy_process_leaves(std::process::id())?;
        Ok(())
    }
}
