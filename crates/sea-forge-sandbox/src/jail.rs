use crate::{
    ExecutionSandbox, NetworkPosture, RelPath, SandboxClass, SandboxError, SandboxHandle,
    SandboxSpec,
};
use sea_forge_core::types::{ArtifactRef, ExecutionRequest, ExecutionResult};
// Only the Linux jail classifies an execution outcome here; the macOS module imports its own.
#[cfg(target_os = "linux")]
use sea_forge_core::types::{ExecutionStatus, LifecycleContainment};

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
    workload: Option<&[std::path::PathBuf]>,
) -> Result<(), SandboxError> {
    use landlock::{
        path_beneath_rules, Access, AccessFs, AccessNet, CompatLevel, Compatible, NetPort, Ruleset,
        RulesetAttr, RulesetCreatedAttr, RulesetStatus, Scope, ABI,
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

    let ruleset = Ruleset::default()
        // Network first, under a hard requirement: on a pre-v4 kernel this
        // errors instead of degrading, giving fail-closed setup.
        .set_compatibility(CompatLevel::HardRequirement)
        .handle_access(net_all)
        .map_err(map_err)?;
    let created = match workload {
        // Trusted jail class: best-effort filesystem rights, read on `/`.
        None => ruleset
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
            // Read-only for the rest of the filesystem so the child can load
            // its interpreter and libraries.
            .add_rules(path_beneath_rules([std::path::Path::new("/")], fs_read))
            .map_err(map_err)?,
        // Isolated workloads (SEC-06): every right of the scoping ABI is a
        // hard requirement, reads are limited to the system trees plus the
        // operator-declared runtime paths, and abstract UNIX sockets and
        // signals outside the workload's Landlock domain are unreachable.
        Some(read_paths) => {
            let fs_all = AccessFs::from_all(ABI::V6);
            let fs_read = AccessFs::from_read(ABI::V6);
            let mut readable: Vec<std::path::PathBuf> = WORKLOAD_SYSTEM_READ
                .iter()
                .map(std::path::PathBuf::from)
                .filter(|path| path.exists())
                .collect();
            readable.extend(read_paths.iter().cloned());
            ruleset
                .handle_access(fs_all)
                .map_err(map_err)?
                .scope(Scope::from_all(ABI::V6))
                .map_err(map_err)?
                .create()
                .map_err(map_err)?
                .add_rules(path_beneath_rules([workspace_root, artifacts_root], fs_all))
                .map_err(map_err)?
                .add_rules(path_beneath_rules(&readable, fs_read))
                .map_err(map_err)?
                // Ordinary shells redirect to /dev/null; the device discards
                // writes, so granting it widens no data path.
                .add_rules(path_beneath_rules(
                    [std::path::Path::new("/dev/null")],
                    fs_read | AccessFs::WriteFile,
                ))
                .map_err(map_err)?
        }
    };

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

/// System trees an isolated workload may read and execute: binaries,
/// libraries, configuration, its own procfs and devices. Home directories,
/// `/tmp`, `/run`, `/var`, `/sys`, `/mnt` and `/srv` are not readable.
#[cfg(target_os = "linux")]
pub const WORKLOAD_SYSTEM_READ: [&str; 10] = [
    "/usr", "/bin", "/sbin", "/lib", "/lib32", "/lib64", "/libx32", "/etc", "/proc", "/dev",
];

/// Apply the isolated-workload posture to the calling thread: read-write on
/// `workspace_root`/`artifacts_root`, read on [`WORKLOAD_SYSTEM_READ`] and
/// `read_paths` only, scoped abstract sockets and signals, and TCP per
/// `network`. Used by the trusted `sea-forge-workload-exec` helper inside a
/// workload namespace.
#[cfg(target_os = "linux")]
pub fn restrict_current_thread(
    workspace_root: &std::path::Path,
    artifacts_root: &std::path::Path,
    read_paths: &[std::path::PathBuf],
    network: &NetworkPosture,
) -> Result<(), SandboxError> {
    apply_landlock(workspace_root, artifacts_root, network, Some(read_paths))
}

#[derive(Debug)]
pub struct JailSandbox;

/// Interactive child spawned after Landlock restriction. The caller owns the
/// stdio pipes and child lifecycle; no shell is involved.
pub struct InteractiveJailChild {
    pub child: std::process::Child,
    pub stdin: std::process::ChildStdin,
    pub stdout: std::process::ChildStdout,
    pub stderr: std::process::ChildStderr,
    /// Isolated workloads only: dropping it ends the workload's namespace.
    #[cfg(target_os = "linux")]
    pub lifeline: Option<crate::workload::Lifeline>,
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
        apply_landlock(workspace_root, artifacts_root, network, None)?;
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
            lifeline: None,
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
        #[cfg(target_os = "macos")]
        {
            macos::preflight()?;
            Ok(JailSandbox)
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
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
        #[cfg(target_os = "macos")]
        {
            macos::execute(h, req)
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
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
        // F08: verified teardown of any lifecycle leaves this process
        // leaked. On hosts without containment there is nothing to verify
        // and no leaf can exist; failures propagate — destroy is not a
        // silent no-op.
        #[cfg(target_os = "linux")]
        crate::cgroup::destroy_process_leaves()?;
        // F08 (macOS): verified sweep of runs this process leaked mid-flight
        // (execute aborted between spawn and verified teardown), mirroring
        // the Linux leaf sweep; CagePersists survivors are reported loudly.
        #[cfg(target_os = "macos")]
        macos::destroy_sweep()?;
        Ok(())
    }
}

/// Cap on the child-written stderr scanned by the sandbox-violation heuristic.
///
/// Child-written stderr is untrusted: without a cap, a jailed child could
/// force an unbounded parent-side read. 64 KiB + 1 mirrors settlement's
/// stderr cap idiom (the +1 keeps an over-cap stream distinguishable from an
/// exactly-at-cap one).
#[cfg(any(target_os = "linux", target_os = "macos"))]
const MAX_STDERR_SCAN_BYTES: u64 = 65_537;

/// Read at most [`MAX_STDERR_SCAN_BYTES`] bytes of the child's stderr file,
/// lossily decoded. Open/read failure yields an empty string, matching the
/// pre-cap `read_to_string(..).unwrap_or_default()` behavior.
#[cfg(any(target_os = "linux", target_os = "macos"))]
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
    use std::{fs, sync::mpsc, thread};

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
    let stdout = crate::artifact_output::create_output_file(&stdout_path).map_err(map_err)?;
    let stderr = crate::artifact_output::create_output_file(&stderr_path).map_err(map_err)?;

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
    // spawning the child, keeping the main thread unrestricted. ONLY the
    // spawn happens on that thread: the wait and the verified teardown run
    // back on this unrestricted thread, because teardown acts on the run's
    // cgroup leaf (kill write, rmdir) — writes outside the workspace and
    // artifacts trees are exactly what the jail's own Landlock domain denies,
    // so a teardown on the restricted thread degrades `cgroup.kill` to the
    // pidfd sweep and strands the drained leaf (the round-6/7 runner
    // observation). The unrestricted caller teardown keeps the leaf kill at
    // full strength, identical to the local backend.
    let (tx, rx) = mpsc::channel::<Result<Result<JailedSpawn, JailedSpawnFailure>, SandboxError>>();

    // F08: acquire the run's cgroup lifecycle leaf here, on the unrestricted
    // main thread — the Landlock-restricted runner thread could not create
    // the leaf under the delegated cgroup root. Same required-lane contract
    // as the local backend: under SEA_FORGE_REQUIRE_CGROUP=1 an unavailable
    // cgroup refuses the run; otherwise the typed degrade is surfaced and
    // the process-group kill remains.
    let lease = crate::cgroup::acquire_spawn_lease("jail", std::process::id())?;

    thread::spawn(move || {
        let _ = tx.send(spawn_jailed(
            &workspace_root,
            &artifacts_root,
            &network,
            &argv,
            &cwd,
            &env,
            stdout,
            stderr,
            lease,
        ));
    });

    // The outer Err propagates setup failures with their typed class (the
    // fail-closed `jail_unavailable` / `cgroup_unavailable` contract — never
    // a fallback); only the inner Err is a spawn failure, which reads as the
    // SpawnFailed outcome.
    let JailedSpawn {
        mut child,
        lease,
        started_at,
    } = match rx
        .recv()
        .map_err(|e| SandboxError::new("internal_error", e.to_string()))??
    {
        Ok(spawned) => spawned,
        Err(failure) => {
            // A spawn failure means nothing exec'd (fail closed). The
            // PRIMARY spawn error is never masked by cleanup: it leads the
            // surfaced report, and the leaf is removed HERE on the
            // unrestricted thread (a failed spawn can never run uncontained;
            // a leaf the restricted thread could not remove is not left
            // behind either).
            let mut report = format!(
                "jail spawn failed before exec (nothing ran uncontained): {}",
                failure.error.message
            );
            if let Some(lease) = failure.lease {
                if let Err(cleanup) = lease.remove() {
                    report.push_str(&format!(
                        "; cgroup leaf {} could not be removed after the failed spawn and \
                         is left for the destroy-time leaf sweep: {cleanup}",
                        lease.leaf().display()
                    ));
                }
            }
            eprintln!("sea-forge-sandbox: {report}");
            return Ok(crate::local::execution_result(
                ExecutionStatus::SpawnFailed,
                None,
                failure.started_at,
                // Nothing exec'd: there is no process tree whose lifecycle
                // could have been contained.
                LifecycleContainment::NoProcess,
            ));
        }
    };

    let result = finish_jailed(&mut child, lease, started_at, timeout_secs)?;

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

/// A jail run that reached the spawn point: the child exists and the run's
/// leaf (if any) travels with it for the caller's verified teardown.
#[cfg(target_os = "linux")]
struct JailedSpawn {
    child: std::process::Child,
    lease: Option<crate::cgroup::CgroupLease>,
    started_at: String,
}

/// A spawn that failed before exec: nothing ran, and the leaf — if one was
/// acquired — travels back so the unrestricted caller can remove it (the
/// restricted thread's Landlock domain denies the rmdir).
#[cfg(target_os = "linux")]
struct JailedSpawnFailure {
    error: SandboxError,
    lease: Option<crate::cgroup::CgroupLease>,
    started_at: String,
}

/// Apply the jail's Landlock posture and spawn the jailed child (Linux).
///
/// Runs on the dedicated restricted thread: the thread restricts itself with
/// the run's Landlock posture and the child inherits that domain at exec.
/// Registration of the pre-exec cgroup join rides along here so the child is
/// contained before any workload-controlled code runs; a failed join fails
/// the spawn (nothing execs uncontained). Waiting and teardown stay on the
/// unrestricted caller ([`execute_linux`]) — see the note there.
///
/// The outer `Err` carries setup failures with their typed classes
/// (`apply_landlock`'s `jail_unavailable`, the join registration's
/// `cgroup_unavailable` under a required lane) and propagates — the jail
/// fails closed, never falling back to a weaker posture. The inner `Err` is
/// a spawn failure of an already-configured command: nothing exec'd, the
/// leaf travels back for the caller to remove.
#[cfg(target_os = "linux")]
#[allow(clippy::too_many_arguments)]
fn spawn_jailed(
    workspace_root: &std::path::Path,
    artifacts_root: &std::path::Path,
    network: &NetworkPosture,
    argv: &[String],
    cwd: &std::path::Path,
    env: &[(String, String)],
    stdout: std::fs::File,
    stderr: std::fs::File,
    lease: Option<crate::cgroup::CgroupLease>,
) -> Result<Result<JailedSpawn, JailedSpawnFailure>, SandboxError> {
    use std::{os::unix::process::CommandExt, process::Command, process::Stdio};

    // Apply filesystem + network Landlock confinement to this thread (inherited
    // by the child). Fails closed with `jail_unavailable` when the requested
    // posture cannot be enforced — never a silent fallback to `local`.
    apply_landlock(workspace_root, artifacts_root, network, None)?;

    let started_at = chrono::Utc::now().to_rfc3339();
    let mut command = Command::new(&argv[0]);
    command
        .args(&argv[1..])
        .current_dir(cwd)
        .env_clear()
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    command.process_group(0);
    // F08: attach the pre-exec join so the child moves into its cgroup leaf
    // before exec — a failed join fails the spawn (nothing runs uncontained).
    // Writing to the pre-opened descriptor is not re-mediated by the thread's
    // Landlock domain (which mediates opens, not writes to inherited fds).
    let lease = crate::cgroup::attach_lease(&mut command, lease, "jail")?;
    for (k, v) in env {
        command.env(k, v);
    }

    match command.spawn() {
        Ok(child) => Ok(Ok(JailedSpawn {
            child,
            lease,
            started_at,
        })),
        Err(error) => Ok(Err(JailedSpawnFailure {
            error: SandboxError::new("jail_unavailable", error.to_string()),
            lease,
            started_at,
        })),
    }
}

/// Wait for the jailed child, run the verified teardown, and assemble the
/// execution result. Runs on the unrestricted caller thread so the leaf
/// kill write and rmdir are not denied by the jail's own Landlock domain.
#[cfg(target_os = "linux")]
fn finish_jailed(
    child: &mut std::process::Child,
    lease: Option<crate::cgroup::CgroupLease>,
    started_at: String,
    timeout_secs: u64,
) -> Result<ExecutionResult, SandboxError> {
    use std::time::Duration;
    use wait_timeout::ChildExt;

    let status = child
        .wait_timeout(Duration::from_secs(timeout_secs))
        .map_err(|e| SandboxError::new("jail_unavailable", e.to_string()))?;

    // C1: the containment claim matches the mechanism that actually held. A
    // cgroup-leaf teardown that failed already returned Err above, so a
    // result produced while a lease existed always carried a verified kill.
    let (state, code, containment) = if let Some(status) = status {
        // F08: leftover descendants must not outlive the run. Teardown
        // failures propagate (previously the result was discarded here), so
        // a survivor can never read as Completed.
        let containment = if lease.is_some() {
            LifecycleContainment::CgroupLeaf {
                kill_verified: true,
            }
        } else {
            LifecycleContainment::ProcessGroup
        };
        crate::local::teardown_run(child, lease)?;
        if status.code().is_none() {
            crate::local::report_signal_death("jail", status);
        }
        (
            ExecutionStatus::Completed,
            status.code().map(i64::from),
            containment,
        )
    } else {
        let containment = if lease.is_some() {
            LifecycleContainment::CgroupLeaf {
                kill_verified: true,
            }
        } else {
            LifecycleContainment::ProcessGroup
        };
        crate::local::teardown_run(child, lease)?;
        child
            .wait()
            .map_err(|e| SandboxError::new("jail_unavailable", e.to_string()))?;
        (ExecutionStatus::TimedOut, None, containment)
    };

    Ok(crate::local::execution_result(
        state,
        code,
        started_at,
        containment,
    ))
}

/// macOS App Sandbox backend.
///
/// Each execution gets its own copy of `sea-forge-jail-launcher`, ad-hoc signed
/// with `com.apple.security.app-sandbox` and temporary exceptions that grant
/// read-write access to exactly the workspace and artifacts trees. The copy then
/// execs the workload, and the kernel applies the sandbox at exec time, so every
/// descendant inherits it. No network entitlement is requested, which denies
/// all TCP, UDP and DNS. Per-port grants cannot be expressed, so
/// `AllowTcpPorts` is refused.
///
/// Reads outside the sandbox's default system set are denied, which is stricter
/// than the Linux jail's read-only `/`. Environment variables are not filtered by
/// the App Sandbox, so the caller's `env_clear` remains the only env boundary.
///
/// F08 lifecycle boundary (documented, not solved): teardown is POSIX process
/// groups only. The workload's group receives SIGKILL on completion, on
/// timeout, and in `destroy`'s sweep, but a descendant that calls `setsid(2)`
/// into its own session is outside every kill sweep macOS offers — there is no
/// cgroup-v2 leaf and no job object, and macOS has no equivalent of the
/// FreeBSD reaper that closes this gap for other BSDs (ProcessKit, the
/// strongest reference implementation, documents the same boundary). What
/// macOS does guarantee: the App Sandbox attaches at exec from the signed
/// launcher and is inherited by every descendant regardless of session, so a
/// surviving `setsid` descendant cannot escape the cage — writes outside the
/// granted trees and all network stay denied even while it outlives the run.
/// Teardown therefore verifies exactly what holds: after the group kill it
/// classifies survivors from a descendant census sampled while the workload
/// ran, reports `CagePersists` survivors loudly on stderr (not an error — the
/// documented residual), and propagates real cleanup errors.
#[cfg(target_os = "macos")]
mod macos {
    use crate::{local::kill_timed_out_group, NetworkPosture, SandboxError, SandboxHandle};
    use sea_forge_core::types::{
        ExecutionRequest, ExecutionResult, ExecutionStatus, LifecycleContainment, Operation,
    };
    use std::{
        collections::{HashMap, HashSet},
        fs,
        os::unix::process::CommandExt,
        path::{Path, PathBuf},
        process::{Command, Stdio},
        sync::{Mutex, MutexGuard},
        time::{Duration, Instant},
    };
    use wait_timeout::ChildExt;

    const CODESIGN: &str = "/usr/bin/codesign";
    const LAUNCHER: &str = "sea-forge-jail-launcher";

    fn unavailable(message: impl std::fmt::Display) -> SandboxError {
        SandboxError::new("jail_unavailable", message.to_string())
    }

    /// Finds the launcher next to the running executable. Cargo puts integration
    /// test binaries in `target/<profile>/deps/`, where the bin sits one level up.
    fn launcher_path() -> Result<PathBuf, SandboxError> {
        let exe = std::env::current_exe()
            .map_err(|e| unavailable(format!("locate current executable: {e}")))?;
        let dir = exe
            .parent()
            .ok_or_else(|| unavailable("current executable has no parent directory"))?;
        let mut candidates = vec![dir.join(LAUNCHER)];
        if dir.file_name().is_some_and(|name| name == "deps") {
            if let Some(parent) = dir.parent() {
                candidates.push(parent.join(LAUNCHER));
            }
        }
        candidates
            .into_iter()
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| unavailable(format!("{LAUNCHER} was not found next to the executable")))
    }

    /// Succeeds only when the signing tool and the launcher are present.
    pub(super) fn preflight() -> Result<(), SandboxError> {
        if !Path::new(CODESIGN).is_file() {
            return Err(unavailable(format!("{CODESIGN} is missing")));
        }
        launcher_path().map(|_| ())
    }

    /// Interval between descendant-census samples while the workload runs.
    /// The census race window — a descendant that forks and `setsid`s in the
    /// final interval before the workload exits — is bounded by this value.
    const CENSUS_INTERVAL: Duration = Duration::from_millis(100);

    /// Bounded wait for killed members to die, mirroring the Linux
    /// teardown's verify deadline.
    const VERIFY_DEADLINE: Duration = Duration::from_secs(5);

    /// Poll interval while waiting for kills to take effect.
    const POLL_INTERVAL: Duration = Duration::from_millis(10);

    /// Crate-private F08 teardown classification for the macOS backend.
    /// Lifted into `ExecutionResult.lifecycle_containment`:
    /// [`MacTeardown::FullyTerminated`] maps to
    /// `LifecycleContainment::ProcessGroup`, [`MacTeardown::CagePersists`]
    /// maps to `LifecycleContainment::SandboxCagePersists`.
    #[derive(Debug)]
    pub(crate) enum MacTeardown {
        /// The group kill reached every censused descendant; nothing
        /// survived the run's cancellation.
        FullyTerminated,
        /// A `setsid` descendant escaped the kill sweep but remains inside
        /// the App Sandbox cage — the documented F08 macOS residual, never
        /// an error but always reported.
        CagePersists {
            /// Pids of the survivors, identity-verified against the census.
            survivor_pids: Vec<i32>,
        },
    }

    /// One censused process identity: pid plus kernel start time, so a pid
    /// recycled between sampling and verification is never mistaken for the
    /// recorded process (macOS has no pidfd to pin a pid).
    #[derive(Clone, Copy, Debug)]
    struct RecordedPid {
        pid: i32,
        start: (u64, u64),
    }

    /// A run this process spawned into the jail whose verified teardown has
    /// not completed: the group id (the workload's pid — the launcher execs
    /// in place) plus the process census. `destroy` sweeps whatever remains
    /// here (an execute that errored or panicked between spawn and
    /// verified teardown).
    struct LiveRun {
        group: i32,
        census: Vec<RecordedPid>,
    }

    static LIVE_RUNS: Mutex<Vec<LiveRun>> = Mutex::new(Vec::new());

    fn live_runs() -> MutexGuard<'static, Vec<LiveRun>> {
        LIVE_RUNS
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    /// Reads `proc_pidinfo(PROC_PIDTBSDINFO)` for `pid`; `None` when the
    /// process has vanished or the kernel returns a short read. The struct
    /// is value-initialized field by field — no `mem::zeroed` — so the only
    /// unsafety here is the libc call itself.
    #[allow(unsafe_code)]
    fn proc_bsd(pid: i32) -> Option<libc::proc_bsdinfo> {
        let mut info = libc::proc_bsdinfo {
            pbi_flags: 0,
            pbi_status: 0,
            pbi_xstatus: 0,
            pbi_pid: 0,
            pbi_ppid: 0,
            pbi_uid: 0,
            pbi_gid: 0,
            pbi_ruid: 0,
            pbi_rgid: 0,
            pbi_svuid: 0,
            pbi_svgid: 0,
            rfu_1: 0,
            pbi_comm: [0; libc::MAXCOMLEN],
            pbi_name: [0; 32],
            pbi_nfiles: 0,
            pbi_pgid: 0,
            pbi_pjobc: 0,
            e_tdev: 0,
            e_tpgid: 0,
            pbi_nice: 0,
            pbi_start_tvsec: 0,
            pbi_start_tvusec: 0,
        };
        let size = std::mem::size_of::<libc::proc_bsdinfo>() as libc::c_int;
        // SAFETY: libproc writes kernel process metadata into our own
        // value-initialized buffer through a pointer taken for the call
        // alone — no aliasing, no retained reference, pure metadata read.
        let written = unsafe {
            libc::proc_pidinfo(
                pid,
                libc::PROC_PIDTBSDINFO,
                0,
                &mut info as *mut libc::proc_bsdinfo as *mut libc::c_void,
                size,
            )
        };
        if written == size {
            Some(info)
        } else {
            None
        }
    }

    /// Every pid on the system, or `None` when the libproc listing fails.
    #[allow(unsafe_code)]
    fn all_pids() -> Option<Vec<i32>> {
        // SAFETY: the null-probe form only returns the required pid count;
        // the fill form writes into an owned `Vec` sized exactly as passed.
        let count = unsafe { libc::proc_listallpids(std::ptr::null_mut(), 0) };
        if count <= 0 {
            return None;
        }
        let mut pids = vec![0_i32; count as usize];
        let written = unsafe {
            libc::proc_listallpids(
                pids.as_mut_ptr() as *mut libc::c_void,
                (pids.len() * std::mem::size_of::<i32>()) as libc::c_int,
            )
        };
        if written <= 0 {
            return None;
        }
        pids.truncate(written as usize);
        Some(pids)
    }

    /// Child adjacency from one libproc pass: parent pid -> (child pid,
    /// kernel start time).
    type ChildMap = HashMap<i32, Vec<(i32, (u64, u64))>>;

    /// Samples the descendant closure of `root` by parent-pid enumeration.
    /// A `setsid` descendant keeps its parent link, so this is how its
    /// identity gets recorded before cancellation — after the workload
    /// exits, its descendants are reparented to launchd and can no longer
    /// be enumerated. `None` means the system-wide enumeration failed this
    /// round.
    fn sample_descendants(root: i32) -> Option<Vec<RecordedPid>> {
        let mut children: ChildMap = HashMap::new();
        for pid in all_pids()? {
            if let Some(info) = proc_bsd(pid) {
                children
                    .entry(info.pbi_ppid as i32)
                    .or_default()
                    .push((pid, (info.pbi_start_tvsec, info.pbi_start_tvusec)));
            }
        }
        let mut descendants = Vec::new();
        let mut seen = HashSet::new();
        seen.insert(root);
        let mut frontier = vec![root];
        while let Some(parent) = frontier.pop() {
            for (pid, start) in children.remove(&parent).unwrap_or_default() {
                if seen.insert(pid) {
                    descendants.push(RecordedPid { pid, start });
                    frontier.push(pid);
                }
            }
        }
        Some(descendants)
    }

    /// Merges a sample into the census; the first observation of a pid wins,
    /// so a recycled pid keeps its recorded (now mismatching) start time and
    /// can never pass the identity check.
    fn merge_census(census: &mut Vec<RecordedPid>, sampled: Vec<RecordedPid>) {
        for entry in sampled {
            if !census.iter().any(|recorded| recorded.pid == entry.pid) {
                census.push(entry);
            }
        }
    }

    fn register_run(group: i32) {
        // A stale entry under the same pgid (a leaked run whose pid the
        // kernel later recycled into this child) must not absorb this
        // run's census publishes.
        live_runs().retain(|run| run.group != group);
        live_runs().push(LiveRun {
            group,
            census: Vec::new(),
        });
    }

    /// Publishes the census so an aborted run still sweeps with identity.
    fn publish_census(group: i32, census: &[RecordedPid]) {
        if let Some(run) = live_runs().iter_mut().find(|run| run.group == group) {
            run.census.clear();
            run.census.extend_from_slice(census);
        }
    }

    fn unregister_run(group: i32) {
        live_runs().retain(|run| run.group != group);
    }

    /// Whether the recorded pid still runs. `kill(pid, 0)` succeeds while
    /// the process entry exists — including for a zombie, which executes
    /// nothing — so the kernel status is read back and `SZOMB` counts as
    /// gone: the killed workload child is exactly such an unreaped zombie on
    /// the timeout path until `child.wait()` reaps it.
    #[allow(unsafe_code)]
    fn alive_recorded(recorded: &RecordedPid) -> bool {
        // SAFETY: signal 0 performs no delivery; the call only probes
        // whether the process entry exists.
        match unsafe { libc::kill(recorded.pid, 0) } {
            0 => match proc_bsd(recorded.pid) {
                Some(info) => info.pbi_status != libc::SZOMB,
                // Existed at the kill, vanished before the read: still alive
                // for this round (conservative — never under-report).
                None => true,
            },
            // -1/ESRCH: gone. -1/EPERM: exists but not signalable by this
            // user — alive. kill(2) has no other return convention.
            -1 => std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH),
            _ => false,
        }
    }

    /// Whether the recorded identity still matches the live process: a
    /// recycled pid reads a different kernel start time and is dropped.
    fn identity_unchanged(recorded: &RecordedPid) -> bool {
        proc_bsd(recorded.pid)
            .is_some_and(|info| recorded.start == (info.pbi_start_tvsec, info.pbi_start_tvusec))
    }

    /// Bounded poll until every recorded pid is gone (or a zombie). Returns
    /// the pids still alive at the deadline, filtered by the start-time
    /// identity check.
    fn survivors_after(recorded: &[RecordedPid]) -> Result<Vec<RecordedPid>, SandboxError> {
        let deadline = Instant::now() + VERIFY_DEADLINE;
        loop {
            let alive: Vec<RecordedPid> = recorded.iter().copied().filter(alive_recorded).collect();
            if alive.is_empty() {
                return Ok(Vec::new());
            }
            if Instant::now() >= deadline {
                return Ok(alive.into_iter().filter(identity_unchanged).collect());
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    }

    /// F08 teardown for one macOS jail run: the verified process-group kill
    /// (every failure propagates — the pre-F08 discarded completion-path
    /// kill is gone), then the census survivor check. Survivors are `setsid`
    /// escapees that remain sandbox-caged: the documented macOS residual,
    /// classified — not an error — in [`MacTeardown`].
    fn teardown_verified(
        child: &mut std::process::Child,
        census: &[RecordedPid],
    ) -> Result<MacTeardown, SandboxError> {
        kill_timed_out_group(child)?;
        let survivors = survivors_after(census)?;
        if survivors.is_empty() {
            Ok(MacTeardown::FullyTerminated)
        } else {
            Ok(MacTeardown::CagePersists {
                survivor_pids: survivors.iter().map(|recorded| recorded.pid).collect(),
            })
        }
    }

    /// Surface a `CagePersists` outcome on stderr (the crate's observability
    /// channel, mirroring `cgroup::report_degrade`). A survivor is the
    /// documented F08 macOS residual — observable, never silent, never an
    /// error.
    fn report_teardown(outcome: &MacTeardown) {
        if let MacTeardown::CagePersists { survivor_pids } = outcome {
            eprintln!(
                "sea-forge-sandbox: macOS App Sandbox jail teardown: {} descendant(s) escaped \
                 the kill sweep (setsid) but remain inside the App Sandbox cage \
                 {survivor_pids:?} — the documented F08 macOS residual (containment holds, \
                 lifecycle does not)",
                survivor_pids.len()
            );
        }
    }

    /// Whether at least one live member of `run.group` matches a censused
    /// (pid, start) identity — the recycle guard that makes `destroy`'s
    /// group kill safe against a stale pgid handed to an unrelated new group.
    fn group_identity_verified(run: &LiveRun) -> bool {
        all_pids().is_some_and(|pids| {
            pids.iter().any(|pid| {
                proc_bsd(*pid).is_some_and(|info| {
                    info.pbi_pgid == run.group as u32
                        && run.census.iter().any(|recorded| {
                            recorded.pid == *pid
                                && recorded.start == (info.pbi_start_tvsec, info.pbi_start_tvusec)
                        })
                })
            })
        })
    }

    /// Verified sweep of one registered run. The group kill is guarded by
    /// the recycle check; census survivors are classified and reported
    /// loudly, never silently dropped, and are not errors.
    #[allow(unsafe_code)]
    fn sweep_run(run: &LiveRun) -> Result<(), SandboxError> {
        // Reap our own zombie group leader first: the killed-but-unreaped
        // direct child of an aborted run keeps both its process entry and
        // its group alive. waitpid(2) on a pid that is not our child fails
        // with ECHILD, which just means there is nothing to reap.
        loop {
            let mut status: libc::c_int = 0;
            // SAFETY: `status` is our own stack slot and WNOHANG cannot block.
            if unsafe { libc::waitpid(run.group, &mut status, libc::WNOHANG) } <= 0 {
                break;
            }
        }
        // SAFETY: signal-only syscalls — no pointers. Signal 0 probes group
        // existence; SIGKILL targets the process group named by the
        // negative pid, gated on the recycle guard below.
        let group_alive = unsafe { libc::kill(-run.group, 0) } == 0;
        let mut killed = false;
        if group_alive {
            if group_identity_verified(run) {
                if unsafe { libc::kill(-run.group, libc::SIGKILL) } == -1 {
                    return Err(SandboxError::new(
                        "internal_error",
                        format!(
                            "macOS jail destroy: kill of process group -{} failed: {}",
                            run.group,
                            std::io::Error::last_os_error()
                        ),
                    ));
                }
                killed = true;
            } else {
                // Fail-safe: a stale pgid may have been recycled to an
                // unrelated process group, and a blind killpg could hit it.
                // Any leftover member of THIS run stays sandbox-caged either
                // way — report, never guess.
                eprintln!(
                    "sea-forge-sandbox: macOS App Sandbox jail destroy: process group -{} \
                     could not be verified as this run's (pid-recycle guard); not killing a \
                     possibly-recycled group — any leftover member remains sandbox-caged",
                    run.group
                );
            }
        }
        // Census survivor check: `setsid` escapees sit outside every group
        // kill. Classified and reported loudly — never an error.
        let survivors = survivors_after(&run.census)?;
        if !survivors.is_empty() {
            report_teardown(&MacTeardown::CagePersists {
                survivor_pids: survivors.iter().map(|recorded| recorded.pid).collect(),
            });
        }
        if killed {
            // Verified-empty: the group must drain. Zombie members whose
            // parent is gone are reaped by launchd within this window; our
            // own were reaped above.
            let deadline = Instant::now() + VERIFY_DEADLINE;
            loop {
                let gone = unsafe { libc::kill(-run.group, 0) } == -1
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH);
                if gone {
                    break;
                }
                if Instant::now() >= deadline {
                    return Err(SandboxError::new(
                        "internal_error",
                        format!(
                            "macOS jail destroy: process group -{} retained members after kill",
                            run.group
                        ),
                    ));
                }
                std::thread::sleep(POLL_INTERVAL);
            }
        }
        Ok(())
    }

    /// F08 `destroy` sweep (macOS): verified teardown of every run this
    /// process registered and never tore down, mirroring the Linux leaf
    /// sweep — group kill, census survivor check, loud `CagePersists`
    /// report, real errors propagate. Runs whose sweep errored stay
    /// registered so a later `destroy` can retry.
    pub(super) fn destroy_sweep() -> Result<(), SandboxError> {
        let mut pending = std::mem::take(&mut *live_runs());
        let mut first_error = None;
        let mut index = 0;
        while index < pending.len() {
            match sweep_run(&pending[index]) {
                Ok(()) => {
                    pending.swap_remove(index);
                }
                Err(err) => {
                    first_error.get_or_insert(err);
                    index += 1;
                }
            }
        }
        if !pending.is_empty() {
            live_runs().extend(pending);
        }
        match first_error {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    fn canonical_dir(path: &Path) -> Result<PathBuf, SandboxError> {
        fs::create_dir_all(path).map_err(unavailable)?;
        path.canonicalize().map_err(unavailable)
    }

    /// Removes the per-execution launcher directory when the execution ends.
    struct PrivateDir(PathBuf);

    impl Drop for PrivateDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn xml_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    /// App Sandbox entitlements: sandbox on, read-write temporary exceptions for
    /// the given directories (trailing slash, as the exception syntax requires),
    /// and no network entitlement.
    fn entitlements_plist(read_write: &[&Path]) -> String {
        let paths: String = read_write
            .iter()
            .map(|path| {
                format!(
                    "\t\t<string>{}/</string>\n",
                    xml_escape(&path.to_string_lossy())
                )
            })
            .collect();
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n<dict>\n\
\t<key>com.apple.security.app-sandbox</key>\n\t<true/>\n\
\t<key>com.apple.security.temporary-exception.files.absolute-path.read-write</key>\n\
\t<array>\n{paths}\t</array>\n\
</dict>\n</plist>\n"
        )
    }

    /// Copies the launcher into `dir`, signs the copy with `entitlements`, and
    /// verifies that the signature carries the App Sandbox key.
    fn sign_launcher(dir: &Path, entitlements: &Path) -> Result<PathBuf, SandboxError> {
        let copy = dir.join(LAUNCHER);
        fs::copy(launcher_path()?, &copy).map_err(unavailable)?;
        let signed = Command::new(CODESIGN)
            .args(["-s", "-", "--force", "--entitlements"])
            .arg(entitlements)
            .arg(&copy)
            .env_clear()
            .output()
            .map_err(unavailable)?;
        if !signed.status.success() {
            return Err(unavailable(format!(
                "codesign failed: {}",
                String::from_utf8_lossy(&signed.stderr).trim()
            )));
        }
        let verified = Command::new(CODESIGN)
            .args(["-d", "--entitlements", ":-"])
            .arg(&copy)
            .env_clear()
            .output()
            .map_err(unavailable)?;
        let stdout = String::from_utf8_lossy(&verified.stdout);
        if !verified.status.success() || !stdout.contains("com.apple.security.app-sandbox") {
            return Err(unavailable(
                "signed launcher does not carry the App Sandbox entitlement",
            ));
        }
        Ok(copy)
    }

    pub(super) fn execute(
        h: &SandboxHandle,
        req: &ExecutionRequest,
    ) -> Result<ExecutionResult, SandboxError> {
        let started_at = chrono::Utc::now().to_rfc3339();
        if let NetworkPosture::AllowTcpPorts(_) = &h.spec.network {
            return Err(unavailable(
                "macOS App Sandbox cannot restrict network by port",
            ));
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

        let workspace = canonical_dir(&h.spec.workspace_root)?;
        let artifacts = canonical_dir(&h.spec.artifacts_root)?;
        let run_parent = artifacts
            .parent()
            .ok_or_else(|| unavailable("artifacts root has no parent for the launcher copy"))?
            .to_path_buf();
        if run_parent.starts_with(&workspace) {
            return Err(unavailable(
                "launcher copy must not be placed inside the workspace",
            ));
        }
        let cwd = crate::safe_join(&h.spec.workspace_root, cwd)
            .map_err(|e| SandboxError::new("unsafe_path_error", e.to_string()))?;

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let private = run_parent.join(format!("sea-forge-jail-{}-{nanos}", std::process::id()));
        fs::create_dir(&private).map_err(unavailable)?;
        let private = PrivateDir(private);

        let entitlements = private.0.join("entitlements.plist");
        fs::write(
            &entitlements,
            entitlements_plist(&[workspace.as_path(), artifacts.as_path()]),
        )
        .map_err(unavailable)?;
        let copy = sign_launcher(&private.0, &entitlements)?;

        let stdout = crate::artifact_output::create_output_file(&artifacts.join("stdout.txt"))
            .map_err(unavailable)?;
        let stderr = crate::artifact_output::create_output_file(&artifacts.join("stderr.txt"))
            .map_err(unavailable)?;
        let mut command = Command::new(&copy);
        command
            .args(argv)
            .current_dir(&cwd)
            .env_clear()
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .process_group(0);
        for (k, v) in &req.env {
            command.env(k, v);
        }
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(_) => {
                return Ok(crate::local::execution_result(
                    ExecutionStatus::SpawnFailed,
                    None,
                    started_at,
                    // Nothing exec'd: there is no process tree whose
                    // lifecycle could have been contained.
                    LifecycleContainment::NoProcess,
                ));
            }
        };
        // F08: register the run so `destroy` can sweep it if this execute
        // aborts between spawn and verified teardown (an error return or a
        // panic leaves the sandboxed group alive).
        let group = child.id() as i32;
        register_run(group);
        // The group leader's identity anchors `destroy`'s recycle guard even
        // when no descendant sample ever runs.
        let mut census = Vec::new();
        if let Some(info) = proc_bsd(group) {
            census.push(RecordedPid {
                pid: group,
                start: (info.pbi_start_tvsec, info.pbi_start_tvusec),
            });
        }
        publish_census(group, &census);

        // Wait for the workload, sampling the descendant census while it is
        // alive: once the workload exits, its descendants are reparented to
        // launchd and can no longer be enumerated, so the census must be
        // recorded BEFORE cancellation.
        let deadline = Instant::now() + Duration::from_secs(req.timeout_secs);
        let mut census_attempts = 0u32;
        let mut census_failures = 0u32;
        let status = loop {
            let Some(slice) = deadline.checked_duration_since(Instant::now()) else {
                break None;
            };
            match child
                .wait_timeout(slice.min(CENSUS_INTERVAL))
                .map_err(unavailable)?
            {
                Some(status) => break Some(status),
                None => {
                    census_attempts += 1;
                    match sample_descendants(group) {
                        Some(descendants) => {
                            merge_census(&mut census, descendants);
                            publish_census(group, &census);
                        }
                        None => census_failures += 1,
                    }
                }
            }
        };
        if census_attempts > 0 && census_failures == census_attempts {
            // Sampling never succeeded: the survivor check below is vacuous
            // beyond the group kill. Surface the typed degrade instead of
            // claiming a verified sweep.
            eprintln!(
                "sea-forge-sandbox: macOS App Sandbox jail: descendant census unavailable, \
                 the survivor check covers the process group only"
            );
        }

        // F08 teardown on BOTH paths: verified group kill + census survivor
        // classification (the completion-path kill result is no longer
        // discarded, so a cleanup failure can never read as Completed).
        let (state, code, teardown) = match status {
            Some(status) => {
                let teardown = teardown_verified(&mut child, &census)?;
                (
                    ExecutionStatus::Completed,
                    status.code().map(i64::from),
                    teardown,
                )
            }
            None => {
                let teardown = teardown_verified(&mut child, &census)?;
                // Reap the killed workload; it presents as a zombie (not a
                // survivor) in the census check until this wait.
                child.wait().map_err(unavailable)?;
                (ExecutionStatus::TimedOut, None, teardown)
            }
        };
        unregister_run(group);
        report_teardown(&teardown);

        // C1: the containment claim is the verified teardown classification.
        // A fully-reached group kill claims ProcessGroup; census survivors
        // are reported as the documented cage-persists residual, never as a
        // full lifecycle containment.
        let containment = match teardown {
            MacTeardown::FullyTerminated => LifecycleContainment::ProcessGroup,
            MacTeardown::CagePersists { survivor_pids } => {
                LifecycleContainment::SandboxCagePersists {
                    survivor_count: survivor_pids.len() as u32,
                }
            }
        };

        let mut result = crate::local::execution_result(state, code, started_at, containment);
        // Same suspicion heuristic as the Linux jail, extended to the errno text
        // the App Sandbox produces. Child-written stderr is only evidence.
        if result.status == ExecutionStatus::Completed && result.exit_code != Some(0) {
            let stderr_text =
                super::read_capped_stderr(&artifacts.join("stderr.txt")).to_lowercase();
            if stderr_text.contains("permission denied")
                || stderr_text.contains("operation not permitted")
            {
                result.status = ExecutionStatus::SuspectedSandboxViolation;
            }
        }
        Ok(result)
    }
}
