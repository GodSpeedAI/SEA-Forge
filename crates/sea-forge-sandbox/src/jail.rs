use crate::{
    ExecutionSandbox, NetworkPosture, RelPath, SandboxClass, SandboxError, SandboxHandle,
    SandboxSpec,
};
use sea_forge_core::types::{ArtifactRef, ExecutionRequest, ExecutionResult, ExecutionStatus};

/// Landlock ABI that first introduced network (TCP bind/connect) restriction.
///
/// The jail requests this ABI for the network dimension under a *hard* posture
/// requirement (see [`apply_landlock`]): on a kernel that predates it the setup
/// fails closed with `jail_unavailable` rather than silently degrading to an
/// unenforced (effectively `local`) posture. The filesystem dimension keeps the
/// crate's default best-effort negotiation so filesystem confinement still
/// works on kernels between the FS-only (v1) and network (v4) ABIs.
#[cfg(target_os = "linux")]
const JAIL_NET_ABI: landlock::ABI = landlock::ABI::V4;

/// Apply the jail's Landlock posture to the current thread: read-write on
/// `workspace_root`/`artifacts_root`, read-only on the rest of the filesystem,
/// and TCP network restriction per `network`.
///
/// ABI negotiation: filesystem access is handled best-effort at [`JAIL_NET_ABI`]
/// (degrades gracefully on older kernels, matching the pre-existing v1 posture),
/// while the network access right is handled under
/// [`CompatLevel::HardRequirement`](landlock::CompatLevel::HardRequirement) so a
/// kernel without network support makes setup fail rather than leak network. We
/// do not pin a second hardcoded ABI constant for degradation — the crate's own
/// `Compatible`/best-effort machinery handles older filesystem kernels.
///
/// Network semantics (TCP only; UDP/raw sockets are an out-of-scope documented
/// gap — Landlock has no coverage through ABI v6):
/// - [`NetworkPosture::Denied`]: handle `BindTcp`/`ConnectTcp` with **no** port
///   rules, denying every outbound connect and every bind/listen.
/// - [`NetworkPosture::AllowTcpPorts`]: add one [`NetPort`](landlock::NetPort)
///   rule per granted port for both bind and connect; all other ports stay
///   denied.
#[cfg(target_os = "linux")]
fn apply_landlock(
    workspace_root: &std::path::Path,
    artifacts_root: &std::path::Path,
    network: &NetworkPosture,
) -> Result<(), SandboxError> {
    use landlock::{
        path_beneath_rules, Access, AccessFs, AccessNet, CompatLevel, Compatible, NetPort, Ruleset,
        RulesetAttr, RulesetCreatedAttr, RulesetStatus,
    };

    fn map_err<E: std::fmt::Display>(error: E) -> SandboxError {
        SandboxError::new("jail_unavailable", error.to_string())
    }

    // Filesystem rights at the network-capable ABI, degrading best-effort on
    // older kernels. Network rights are a hard requirement so we never silently
    // run without the requested network confinement.
    let fs_all = AccessFs::from_all(JAIL_NET_ABI);
    let fs_read = AccessFs::from_read(JAIL_NET_ABI);
    let net_all = AccessNet::from_all(JAIL_NET_ABI);

    let created = Ruleset::default()
        // Network first, under a hard requirement: on a pre-v4 kernel this
        // errors instead of degrading, giving fail-closed setup.
        .set_compatibility(CompatLevel::HardRequirement)
        .handle_access(net_all)
        .map_err(map_err)?
        // Filesystem best-effort so FS confinement still works on kernels
        // between the FS-only and network ABIs.
        .set_compatibility(CompatLevel::BestEffort)
        .handle_access(fs_all)
        .map_err(map_err)?
        .create()
        .map_err(map_err)?
        // Read-write for workspace and artifacts.
        .add_rules(path_beneath_rules([workspace_root, artifacts_root], fs_all))
        .map_err(map_err)?
        // Read-only for the rest of the filesystem so the child can load its
        // interpreter and libraries.
        .add_rules(path_beneath_rules([std::path::Path::new("/")], fs_read))
        .map_err(map_err)?;

    // Add per-port TCP grants for an explicit network posture. Default denial
    // adds no port rules, so handling BindTcp/ConnectTcp with an empty rule set
    // denies all bind/listen and all outbound connects.
    let created = match network {
        NetworkPosture::Denied => created,
        NetworkPosture::AllowTcpPorts(ports) => {
            let mut created = created;
            for port in ports {
                created = created
                    .add_rule(NetPort::new(
                        *port,
                        AccessNet::BindTcp | AccessNet::ConnectTcp,
                    ))
                    .map_err(map_err)?;
            }
            created
        }
    };

    let status = created.restrict_self().map_err(map_err)?;

    if status.ruleset == RulesetStatus::NotEnforced {
        return Err(SandboxError::new(
            "jail_unavailable",
            "Landlock restrictions are not enforced by this kernel",
        ));
    }
    // A partially-enforced ruleset means at least one requested access right was
    // dropped by the running kernel. Because the network right is the only one
    // requested under a hard requirement (the FS rights are best-effort and
    // predate the network ABI, so they never degrade on a kernel new enough to
    // support any Landlock at all), any partial enforcement here would mean the
    // network confinement is not fully in force — fail closed rather than run a
    // jail whose network posture cannot be guaranteed.
    if status.ruleset == RulesetStatus::PartiallyEnforced {
        return Err(SandboxError::new(
            "jail_unavailable",
            "Landlock network restriction is not fully enforced by this kernel",
        ));
    }
    Ok(())
}

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
    network: &NetworkPosture,
) -> Result<InteractiveJailChild, SandboxError> {
    #[cfg(target_os = "linux")]
    {
        use std::{os::unix::process::CommandExt, process::Stdio};

        fn map_err<E: std::fmt::Display>(error: E) -> SandboxError {
            SandboxError::new("jail_unavailable", error.to_string())
        }
        if argv.is_empty() {
            return Err(SandboxError::new("input_error", "ACP argv is empty"));
        }
        std::fs::create_dir_all(workspace_root).map_err(map_err)?;
        std::fs::create_dir_all(artifacts_root).map_err(map_err)?;
        apply_landlock(workspace_root, artifacts_root, network)?;
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
        let _ = (workspace_root, artifacts_root, argv, cwd, env, network);
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

/// Cap on the child-written stderr scanned by the sandbox-violation heuristic.
///
/// Child-written stderr is untrusted: without a cap, a jailed child could
/// force an unbounded parent-side read. 64 KiB + 1 mirrors settlement's
/// stderr cap idiom (the +1 keeps an over-cap stream distinguishable from an
/// exactly-at-cap one).
#[cfg(target_os = "linux")]
const MAX_STDERR_SCAN_BYTES: u64 = 65_537;

/// Read at most [`MAX_STDERR_SCAN_BYTES`] bytes of the child's stderr file,
/// lossily decoded. Open/read failure yields an empty string, matching the
/// pre-cap `read_to_string(..).unwrap_or_default()` behavior.
#[cfg(target_os = "linux")]
fn read_capped_stderr(path: &std::path::Path) -> String {
    use std::io::Read;
    let mut bytes = Vec::new();
    if let Ok(file) = std::fs::File::open(path) {
        let _ = file.take(MAX_STDERR_SCAN_BYTES).read_to_end(&mut bytes);
    }
    String::from_utf8_lossy(&bytes).into_owned()
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
    let network = h.spec.network.clone();
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
            &network,
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

    // F-20: the jail does not *observe* this violation — a nonzero exit plus
    // a child-controlled stderr substring is only evidence of one. Recording
    // it as a definite `SandboxViolation` asserted more than ran; the
    // suspected classification keeps settlement rejected while leaving the
    // durable basis honest about what the heuristic actually saw.
    let mut result = result;
    if result.status == ExecutionStatus::Completed && result.exit_code != Some(0) {
        let stderr_text = read_capped_stderr(&stderr_path);
        if stderr_text.to_lowercase().contains("permission denied") {
            result.status = ExecutionStatus::SuspectedSandboxViolation;
        }
    }
    Ok(result)
}

#[cfg(target_os = "linux")]
#[allow(clippy::too_many_arguments)]
fn run_jailed(
    workspace_root: &std::path::Path,
    artifacts_root: &std::path::Path,
    network: &NetworkPosture,
    argv: &[String],
    cwd: &std::path::Path,
    env: &[(String, String)],
    timeout_secs: u64,
    stdout: std::fs::File,
    stderr: std::fs::File,
) -> Result<ExecutionResult, SandboxError> {
    use std::{
        os::unix::process::CommandExt,
        process::{Command, Stdio},
        time::Duration,
    };
    use wait_timeout::ChildExt;

    fn map_err<E: std::fmt::Display>(e: E) -> SandboxError {
        SandboxError::new("jail_unavailable", e.to_string())
    }

    // Apply filesystem + network Landlock confinement to this thread (inherited
    // by the child). Fails closed with `jail_unavailable` when the requested
    // posture cannot be enforced — never a silent fallback to `local`.
    apply_landlock(workspace_root, artifacts_root, network)?;

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
