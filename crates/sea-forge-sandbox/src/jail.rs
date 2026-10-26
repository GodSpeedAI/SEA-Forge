use crate::{
    ExecutionSandbox, NetworkPosture, RelPath, SandboxClass, SandboxError, SandboxHandle,
    SandboxSpec,
};
use sea_forge_core::types::{ArtifactRef, ExecutionRequest, ExecutionResult};
// Only the Linux jail classifies an execution outcome here; the macOS module imports its own.
#[cfg(target_os = "linux")]
use sea_forge_core::types::ExecutionStatus;

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
    apply_landlock(workspace_root, artifacts_root, network, None)?;

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
            return Ok(crate::local::execution_result(
                ExecutionStatus::SpawnFailed,
                None,
                started_at,
            ));
        }
    };

    let status = child
        .wait_timeout(Duration::from_secs(timeout_secs))
        .map_err(map_err)?;

    let (state, code) = if let Some(status) = status {
        // Best effort: leftover descendants must not outlive the run.
        let group = format!("-{}", child.id());
        let _ = Command::new("/bin/kill")
            .args(["-KILL", "--", &group])
            .env_clear()
            .output();
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

    Ok(crate::local::execution_result(state, code, started_at))
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
#[cfg(target_os = "macos")]
mod macos {
    use crate::{local::kill_timed_out_group, NetworkPosture, SandboxError, SandboxHandle};
    use sea_forge_core::types::{ExecutionRequest, ExecutionResult, ExecutionStatus, Operation};
    use std::{
        fs,
        os::unix::process::CommandExt,
        path::{Path, PathBuf},
        process::{Command, Stdio},
        time::Duration,
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
                ))
            }
        };
        let status = child
            .wait_timeout(Duration::from_secs(req.timeout_secs))
            .map_err(unavailable)?;
        let (state, code) = if let Some(status) = status {
            // Best effort: leftover descendants must not outlive the run.
            let _ = kill_timed_out_group(&child);
            (ExecutionStatus::Completed, status.code().map(i64::from))
        } else {
            kill_timed_out_group(&child)?;
            child.wait().map_err(unavailable)?;
            (ExecutionStatus::TimedOut, None)
        };

        let mut result = crate::local::execution_result(state, code, started_at);
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
