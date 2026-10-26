//! NVIDIA OpenShell execution backend.
//!
//! Drives the pinned `openshell` CLI and `openshell-prover` as subprocesses.
//! The CLI is the integration surface because OpenShell's Rust SDK is async
//! and this crate is part of the synchronous kernel (see ADR-009).
//!
//! The sandbox is created from a DomainForge OpenShell projection
//! (`policy.yaml`, `sandbox.json`, `openshell.lock`). Before any command runs:
//!
//! 1. the projection is checked against its own lock (a policy edited after
//!    projection is refused),
//! 2. the CLI version must be the one the projection targets,
//! 3. the gateway must answer,
//! 4. the policy the gateway actually composed, including provider rules, must
//!    be within the projected policy according to `openshell-prover`, with
//!    every required domain covered.
//!
//! Any failure is an error. Nothing here falls back to another sandbox class.
//! The prover checks policy containment; it is not evidence of runtime
//! isolation.

use crate::{ExecutionSandbox, RelPath, SandboxClass, SandboxError, SandboxHandle, SandboxSpec};
use sea_forge_core::types::{ArtifactRef, ExecutionRequest, ExecutionResult, ExecutionStatus};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};
use wait_timeout::ChildExt;

/// Prover domains that must all be covered for a containment result to count.
const REQUIRED_COVERAGE: [&str; 5] = [
    "filesystem",
    "landlock",
    "network_l4",
    "network_rest",
    "process",
];

/// The files a DomainForge OpenShell projection consists of besides its lock,
/// sorted. The lock must cover exactly these.
const PROJECTION_ARTIFACTS: [&str; 3] = ["mapping-report.json", "policy.yaml", "sandbox.json"];

/// OpenShell reports this exit code when a command exceeds its timeout.
const EXIT_TIMED_OUT: i64 = 124;

const CONTROL_TIMEOUT: Duration = Duration::from_secs(60);
const CREATE_TIMEOUT: Duration = Duration::from_secs(1800);
const TRANSFER_TIMEOUT: Duration = Duration::from_secs(600);

/// Where the projection lives and how to reach OpenShell. Supplied by the
/// trusted caller; never read from the workspace the sandbox can write.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenShellBinding {
    /// Directory holding `policy.yaml`, `sandbox.json` and `openshell.lock`.
    pub projection_dir: PathBuf,
    /// Registered gateway name; `None` uses the CLI's active gateway.
    pub gateway: Option<String>,
    /// OCI image for the sandbox; `None` uses the gateway's default image.
    pub image: Option<String>,
    pub cli: PathBuf,
    pub prover: PathBuf,
    /// The gateway runs MicroVMs on this host, so this host must offer the
    /// virtualization the driver needs.
    pub local_microvm_gateway: bool,
}

impl OpenShellBinding {
    pub fn new(projection_dir: impl Into<PathBuf>) -> Self {
        Self {
            projection_dir: projection_dir.into(),
            gateway: None,
            image: None,
            cli: PathBuf::from("openshell"),
            prover: PathBuf::from("openshell-prover"),
            local_microvm_gateway: true,
        }
    }
}

/// The identities a projection binds, as recorded in `openshell.lock`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionIdentity {
    pub cell_id: String,
    pub model_hash: String,
    pub policy_digest: String,
    pub openshell_version: String,
}

fn err(class: &'static str, message: impl Into<String>) -> SandboxError {
    SandboxError::new(class, message)
}

fn sha256_file(path: &Path) -> Result<String, SandboxError> {
    let bytes = fs::read(path).map_err(|e| {
        err(
            "openshell_projection_error",
            format!("{}: {e}", path.display()),
        )
    })?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(&bytes))))
}

fn read_json(path: &Path) -> Result<Value, SandboxError> {
    let text = fs::read_to_string(path).map_err(|e| {
        err(
            "openshell_projection_error",
            format!("{}: {e}", path.display()),
        )
    })?;
    serde_json::from_str(&text).map_err(|e| {
        err(
            "openshell_projection_error",
            format!("{}: {e}", path.display()),
        )
    })
}

fn text_field(value: &Value, pointer: &str, file: &str) -> Result<String, SandboxError> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            err(
                "openshell_projection_error",
                format!("{file} lacks {pointer}"),
            )
        })
}

/// Read a projection's identity and verify the policy against the lock.
pub fn projection_identity(projection_dir: &Path) -> Result<ProjectionIdentity, SandboxError> {
    let lock = read_json(&projection_dir.join("openshell.lock"))?;
    let identity = ProjectionIdentity {
        cell_id: text_field(&lock, "/cell_id", "openshell.lock")?,
        model_hash: text_field(&lock, "/model/hash", "openshell.lock")?,
        policy_digest: text_field(&lock, "/policy_digest", "openshell.lock")?,
        openshell_version: text_field(&lock, "/openshell_version", "openshell.lock")?,
    };
    // The lock must name exactly the projection's artifacts, and every one of
    // them must be present and unchanged. A missing, extra or edited file
    // means this is not the projection that was authorized.
    let files = lock
        .pointer("/files")
        .and_then(Value::as_object)
        .ok_or_else(|| err("openshell_projection_error", "openshell.lock lacks /files"))?;
    let mut locked: Vec<&str> = files.keys().map(String::as_str).collect();
    locked.sort_unstable();
    if locked != PROJECTION_ARTIFACTS {
        return Err(err(
            "openshell_projection_tampered",
            format!("openshell.lock covers {locked:?}, expected {PROJECTION_ARTIFACTS:?}"),
        ));
    }
    for (file, recorded) in files {
        if recorded.as_str() != Some(sha256_file(&projection_dir.join(file))?.as_str()) {
            return Err(err(
                "openshell_projection_tampered",
                format!("{file} does not match openshell.lock"),
            ));
        }
    }
    let present = fs::read_dir(projection_dir)
        .map_err(|e| err("openshell_projection_error", e.to_string()))?
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name != "openshell.lock")
        .collect::<std::collections::BTreeSet<_>>();
    if present.iter().map(String::as_str).ne(PROJECTION_ARTIFACTS) {
        return Err(err(
            "openshell_projection_tampered",
            format!(
                "the projection directory holds {present:?}, expected {PROJECTION_ARTIFACTS:?}"
            ),
        ));
    }
    if text_field(&lock, "/files/policy.yaml", "openshell.lock")? != identity.policy_digest {
        return Err(err(
            "openshell_projection_tampered",
            "openshell.lock is internally inconsistent",
        ));
    }
    Ok(identity)
}

/// Whether this host can run the MicroVM driver. Linux needs KVM.
pub fn probe_host_virtualization() -> Result<(), SandboxError> {
    // Windows has no KVM path here: refuse rather than report a host as capable.
    if cfg!(windows) {
        return Err(SandboxError::new(
            "workload_isolation_unavailable",
            "MicroVM virtualization is not available on Windows hosts",
        ));
    }
    if !cfg!(target_os = "linux") {
        return Ok(());
    }
    match File::open("/dev/kvm") {
        Ok(_) => Ok(()),
        Err(e) => {
            let wsl = fs::read_to_string("/proc/version")
                .map(|v| v.to_ascii_lowercase().contains("microsoft"))
                .unwrap_or(false);
            let hint = if wsl {
                " This is WSL2: enable nested virtualization (`nestedVirtualization=true` under [wsl2] in .wslconfig) and restart WSL."
            } else {
                " Enable KVM and grant this user access to /dev/kvm."
            };
            Err(err(
                "openshell_virtualization_unavailable",
                format!("cannot open /dev/kvm: {e}. OpenShell MicroVMs need KVM on Linux.{hint}"),
            ))
        }
    }
}

struct Output {
    code: Option<i64>,
    stdout: String,
    stderr: String,
}

/// Run a control command with a clean environment and a deadline.
fn run(program: &Path, args: &[String], timeout: Duration) -> Result<Output, SandboxError> {
    let mut command = Command::new(program);
    command
        .args(args)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // The CLI needs PATH (for ssh) and its own config and state directories.
    for key in [
        "PATH",
        "HOME",
        "XDG_CONFIG_HOME",
        "XDG_STATE_HOME",
        "XDG_RUNTIME_DIR",
    ] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    // ETXTBSY: another thread forked while the executable was still open for
    // writing (a just-installed binary). It clears as soon as that fork execs.
    let mut attempts = 0;
    let mut child = loop {
        match command.spawn() {
            Ok(child) => break child,
            Err(e) if e.kind() == std::io::ErrorKind::ExecutableFileBusy && attempts < 20 => {
                attempts += 1;
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(e) => {
                return Err(err(
                    "openshell_unavailable",
                    format!("cannot run {}: {e}", program.display()),
                ))
            }
        }
    };
    // Drain the pipes on threads so a chatty command cannot block on a full pipe.
    let drain = |pipe: Option<Box<dyn std::io::Read + Send>>| {
        std::thread::spawn(move || {
            let mut text = String::new();
            if let Some(mut pipe) = pipe {
                let _ = std::io::Read::read_to_string(&mut pipe, &mut text);
            }
            text
        })
    };
    let stdout = drain(
        child
            .stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn std::io::Read + Send>),
    );
    let stderr = drain(
        child
            .stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn std::io::Read + Send>),
    );
    let status = child
        .wait_timeout(timeout)
        .map_err(|e| err("io_error", e.to_string()))?;
    let code = match status {
        Some(status) => status.code().map(i64::from),
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(err(
                "openshell_timeout",
                format!(
                    "{} {} did not finish in {}s",
                    program.display(),
                    args.join(" "),
                    timeout.as_secs()
                ),
            ));
        }
    };
    Ok(Output {
        code,
        stdout: stdout.join().unwrap_or_default(),
        stderr: stderr.join().unwrap_or_default(),
    })
}

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| s.to_string()).collect()
}

pub struct OpenShellSandbox {
    binding: OpenShellBinding,
}

impl OpenShellSandbox {
    pub fn new(binding: OpenShellBinding) -> Self {
        Self { binding }
    }

    fn cli(&self, args: &[String], timeout: Duration) -> Result<Output, SandboxError> {
        let mut full = Vec::new();
        if let Some(gateway) = &self.binding.gateway {
            full.push("--gateway".to_string());
            full.push(gateway.clone());
        }
        full.extend_from_slice(args);
        run(&self.binding.cli, &full, timeout)
    }

    fn cli_ok(
        &self,
        class: &'static str,
        args: &[String],
        timeout: Duration,
    ) -> Result<Output, SandboxError> {
        let out = self.cli(args, timeout)?;
        if out.code != Some(0) {
            return Err(err(
                class,
                format!("openshell {} failed: {}", args.join(" "), out.stderr.trim()),
            ));
        }
        Ok(out)
    }

    /// One sandbox per artifacts directory, which is unique per run.
    fn name(spec: &SandboxSpec) -> String {
        let digest = Sha256::digest(spec.artifacts_root.to_string_lossy().as_bytes());
        format!("sf-{}", &hex::encode(digest)[..16])
    }

    fn evidence_dir(spec: &SandboxSpec) -> PathBuf {
        spec.artifacts_root.join("openshell")
    }

    fn workspace_path(&self) -> Result<String, SandboxError> {
        let sandbox = read_json(&self.binding.projection_dir.join("sandbox.json"))?;
        let workspaces = sandbox
            .pointer("/workspaces")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        match workspaces.as_slice() {
            [only] => text_field(only, "/sandbox_path", "sandbox.json"),
            _ => Err(err(
                "openshell_projection_error",
                "the projection must declare exactly one workspace mount",
            )),
        }
    }

    /// Run the prover and require containment with full coverage. Returns the
    /// prover's JSON envelope for the evidence record.
    fn prove(&self, candidate: &Path, class: &'static str) -> Result<Value, SandboxError> {
        let boundary = self.binding.projection_dir.join("policy.yaml");
        let out = run(
            &self.binding.prover,
            &[
                "check".to_string(),
                candidate.display().to_string(),
                "--boundary".to_string(),
                boundary.display().to_string(),
                "--output".to_string(),
                "json".to_string(),
            ],
            CONTROL_TIMEOUT,
        )?;
        let envelope: Value = serde_json::from_str(&out.stdout).map_err(|e| {
            err(
                class,
                format!(
                    "openshell-prover printed no result: {e}: {}",
                    out.stderr.trim()
                ),
            )
        })?;
        let result = envelope["result"].as_str().unwrap_or("missing");
        if result != "within_boundary" {
            return Err(err(
                class,
                format!(
                    "policy is not within the Cell boundary: {result} {} {}",
                    envelope["reason"].as_str().unwrap_or(""),
                    envelope["counterexample"]
                ),
            ));
        }
        let covered: Vec<&str> = envelope["coverage"]["domains"]
            .as_array()
            .map(|domains| domains.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        if let Some(missing) = REQUIRED_COVERAGE.iter().find(|d| !covered.contains(d)) {
            return Err(err(
                class,
                format!("the prover did not cover the {missing} domain"),
            ));
        }
        Ok(envelope)
    }

    fn delete(&self, name: &str) -> Result<(), SandboxError> {
        let out = self.cli(&strings(&["sandbox", "delete", name]), CONTROL_TIMEOUT)?;
        if out.code == Some(0) {
            return Ok(());
        }
        // Deleting twice must succeed: a sandbox that is already gone is the goal.
        let gone = self
            .cli(&strings(&["sandbox", "get", name]), CONTROL_TIMEOUT)
            .map(|probe| probe.code != Some(0))
            .unwrap_or(false);
        if gone {
            Ok(())
        } else {
            Err(err(
                "openshell_cleanup_failed",
                format!("could not delete sandbox {name}: {}", out.stderr.trim()),
            ))
        }
    }

    /// Copy the sandbox's workspace back to `dest` on the host.
    pub fn download_workspace(&self, h: &SandboxHandle, dest: &Path) -> Result<(), SandboxError> {
        let name = Self::name(&h.spec);
        fs::create_dir_all(dest).map_err(|e| err("io_error", e.to_string()))?;
        self.cli_ok(
            "openshell_transfer_failed",
            &[
                "sandbox".to_string(),
                "download".to_string(),
                name,
                self.workspace_path()?,
                dest.display().to_string(),
            ],
            TRANSFER_TIMEOUT,
        )?;
        Ok(())
    }

    /// Network and filesystem denials the sandbox logged, one per line.
    pub fn denials(&self, h: &SandboxHandle) -> Result<Vec<String>, SandboxError> {
        let out = self.cli_ok(
            "openshell_logs_unavailable",
            &[
                "logs".to_string(),
                Self::name(&h.spec),
                "--source".to_string(),
                "sandbox".to_string(),
                "-n".to_string(),
                "2000".to_string(),
            ],
            CONTROL_TIMEOUT,
        )?;
        Ok(out
            .stdout
            .lines()
            .filter(|line| line.contains(" DENIED "))
            .map(str::to_string)
            .collect())
    }

    fn create(
        &self,
        spec: &SandboxSpec,
        name: &str,
        identity: &ProjectionIdentity,
    ) -> Result<(), SandboxError> {
        let evidence = Self::evidence_dir(spec);
        fs::create_dir_all(&evidence).map_err(|e| err("io_error", e.to_string()))?;
        let policy = self.binding.projection_dir.join("policy.yaml");

        let projected = self.prove(&policy, "openshell_policy_unverified")?;

        let mut args = strings(&[
            "sandbox",
            "create",
            "--name",
            name,
            "--policy",
            &policy.display().to_string(),
            "--detach",
            "--no-auto-providers",
            "--no-tty",
        ]);
        if let Some(image) = &self.binding.image {
            args.push("--from".to_string());
            args.push(image.clone());
        }
        self.cli_ok("openshell_create_failed", &args, CREATE_TIMEOUT)?;

        // What the gateway composed, not what was asked for.
        let effective = self.cli_ok(
            "openshell_policy_unavailable",
            &strings(&["sandbox", "get", name, "--policy-only"]),
            CONTROL_TIMEOUT,
        )?;
        let effective_path = evidence.join("effective-policy.yaml");
        fs::write(&effective_path, &effective.stdout)
            .map_err(|e| err("io_error", e.to_string()))?;
        let contained = self.prove(
            &effective_path,
            "openshell_effective_policy_exceeds_boundary",
        )?;

        let status = self.cli_ok(
            "openshell_policy_unavailable",
            &strings(&["policy", "get", name, "--output", "json"]),
            CONTROL_TIMEOUT,
        )?;
        let status: Value = serde_json::from_str(&status.stdout).map_err(|e| {
            err(
                "openshell_policy_unavailable",
                format!("policy status is not JSON: {e}"),
            )
        })?;

        self.cli_ok(
            "openshell_transfer_failed",
            &[
                "sandbox".to_string(),
                "upload".to_string(),
                name.to_string(),
                spec.workspace_root.display().to_string(),
                self.workspace_path()?,
                "--no-git-ignore".to_string(),
            ],
            TRANSFER_TIMEOUT,
        )?;

        let record = json!({
            "schema": "sea-forge.openshell.execution/v1",
            "sandbox": name,
            "gateway": self.binding.gateway,
            "image": self.binding.image,
            "cell_id": identity.cell_id,
            "model_hash": identity.model_hash,
            "projected_policy_digest": identity.policy_digest,
            "openshell_version": identity.openshell_version,
            "effective_policy": {
                "sha256": sha256_file(&effective_path)?,
                "hash": status["hash"],
                "version": status["version"],
                "config_revision": status["config_revision"],
                "source": status["policy_source"],
            },
            "prover": { "projected": projected, "effective": contained },
        });
        fs::write(
            evidence.join("execution.json"),
            serde_json::to_string_pretty(&record).unwrap_or_default(),
        )
        .map_err(|e| err("io_error", e.to_string()))
    }
}

impl ExecutionSandbox for OpenShellSandbox {
    fn class(&self) -> SandboxClass {
        SandboxClass::OpenShell
    }

    fn prepare(&self, spec: &SandboxSpec) -> Result<SandboxHandle, SandboxError> {
        // The sandbox image is always explicit: an unspecified image would let
        // the gateway pick one that the authority decision never saw.
        if self.binding.image.as_deref().is_none_or(str::is_empty) {
            return Err(err(
                "openshell_image_unspecified",
                "the OpenShell binding names no sandbox image",
            ));
        }
        let identity = projection_identity(&self.binding.projection_dir)?;

        let version = run(&self.binding.cli, &strings(&["--version"]), CONTROL_TIMEOUT)?;
        let expected = format!("openshell {}", identity.openshell_version);
        if version.stdout.trim() != expected {
            return Err(err(
                "openshell_version_mismatch",
                format!(
                    "the projection targets `{expected}`, the CLI reports `{}`",
                    version.stdout.trim()
                ),
            ));
        }
        if self.binding.local_microvm_gateway {
            probe_host_virtualization()?;
        }
        let status = self.cli(&strings(&["status"]), CONTROL_TIMEOUT)?;
        if status.code != Some(0) {
            return Err(err(
                "openshell_gateway_unavailable",
                format!(
                    "the OpenShell gateway did not answer: {}",
                    status.stderr.trim()
                ),
            ));
        }

        let name = Self::name(spec);
        if let Err(error) = self.create(spec, &name, &identity) {
            // Never leave a sandbox behind that failed verification.
            let _ = self.delete(&name);
            return Err(error);
        }
        Ok(SandboxHandle {
            class: SandboxClass::OpenShell,
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
        // Only the typed sandbox action reaches this backend. A host
        // `execute_command` is never run through OpenShell.
        let (argv, cwd) = match &req.operation {
            Operation::ExecuteSandboxCommand { argv, cwd, .. } if !argv.is_empty() => (argv, cwd),
            _ => {
                return Err(err(
                    "input_error",
                    "the openshell backend runs only execute_sandbox_command",
                ))
            }
        };
        crate::validate_relative_path(cwd).map_err(|e| err("unsafe_path_error", e.to_string()))?;
        let workspace = self.workspace_path()?;
        let workdir = if cwd == "." {
            workspace
        } else {
            format!("{workspace}/{cwd}")
        };

        fs::create_dir_all(&h.spec.artifacts_root).map_err(|e| err("io_error", e.to_string()))?;
        let stdout = File::create(h.spec.artifacts_root.join("stdout.txt"))
            .map_err(|e| err("io_error", e.to_string()))?;
        let stderr = File::create(h.spec.artifacts_root.join("stderr.txt"))
            .map_err(|e| err("io_error", e.to_string()))?;

        let mut command = Command::new(&self.binding.cli);
        command.env_clear();
        for key in [
            "PATH",
            "HOME",
            "XDG_CONFIG_HOME",
            "XDG_STATE_HOME",
            "XDG_RUNTIME_DIR",
        ] {
            if let Some(value) = std::env::var_os(key) {
                command.env(key, value);
            }
        }
        if let Some(gateway) = &self.binding.gateway {
            command.args(["--gateway", gateway]);
        }
        command
            .args(["sandbox", "exec", "--name", &Self::name(&h.spec)])
            .args(["--workdir", &workdir])
            .args(["--timeout", &req.timeout_secs.to_string()])
            .arg("--no-tty");
        // Only the request's own variables reach the sandbox, never the host's.
        for (key, value) in &req.env {
            command.args(["--env", &format!("{key}={value}")]);
        }
        command
            .arg("--")
            .args(argv)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr));

        let result = |status, exit_code| ExecutionResult {
            status,
            exit_code,
            stdout_path: "artifacts/stdout.txt".into(),
            stderr_path: "artifacts/stderr.txt".into(),
            started_at: started_at.clone(),
            finished_at: chrono::Utc::now().to_rfc3339(),
        };
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(_) => return Ok(result(ExecutionStatus::SpawnFailed, None)),
        };
        // OpenShell enforces the timeout in the sandbox; the grace period only
        // covers a CLI that stops responding.
        let deadline = Duration::from_secs(req.timeout_secs.saturating_add(30));
        let status = child
            .wait_timeout(deadline)
            .map_err(|e| err("io_error", e.to_string()))?;
        let Some(status) = status else {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(result(ExecutionStatus::TimedOut, None));
        };
        let code = status.code().map(i64::from);
        if code == Some(EXIT_TIMED_OUT) {
            return Ok(result(ExecutionStatus::TimedOut, None));
        }

        let denials = self.denials(h)?;
        fs::write(
            Self::evidence_dir(&h.spec).join("denials.log"),
            denials.join("\n"),
        )
        .map_err(|e| err("io_error", e.to_string()))?;
        if code != Some(0) && !denials.is_empty() {
            return Ok(result(ExecutionStatus::SandboxViolation, code));
        }
        Ok(result(ExecutionStatus::Completed, code))
    }

    fn collect_artifacts(
        &self,
        _h: &SandboxHandle,
        _paths: &[RelPath],
    ) -> Result<Vec<ArtifactRef>, SandboxError> {
        Err(err(
            "artifact_collection_unsupported",
            "collect_artifacts is not implemented for this sandbox backend; use download_workspace and the changeset flow",
        ))
    }

    fn destroy(&self, h: SandboxHandle) -> Result<(), SandboxError> {
        self.delete(&Self::name(&h.spec))
    }
}
