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
//! 3. the projection's compute driver must be one whose enforcement SEA-Forge
//!    knows, and it must enforce what the policy requires (Landlock hard
//!    requirement, Linux process identity, egress proxy). This is the
//!    enforcement capability; it is separate from the policy proof below,
//! 4. the workload image must match the driver's kind (OCI for `vm`, native
//!    executable for `mxc`), and a native executable must hash to its pin,
//! 5. the gateway must answer,
//! 6. the policy the gateway actually composed, including provider rules, must
//!    be within the projected policy according to `openshell-prover`, with
//!    every required domain covered.
//!
//! Any failure is an error. Nothing here falls back to another sandbox class.
//! The prover checks policy containment; it is not evidence of runtime
//! isolation, and neither is the enforcement table.

use crate::{ExecutionSandbox, RelPath, SandboxClass, SandboxError, SandboxHandle, SandboxSpec};
use sea_forge_core::types::{ArtifactRef, ExecutionRequest, ExecutionResult, ExecutionStatus};
use sea_forge_core::workload_image::WorkloadImage;
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
    /// The gateway runs its compute driver on this host, so this host must
    /// provide what that driver needs (see `probe_local_driver`).
    pub local_microvm_gateway: bool,
    /// The native executable a `native:sha256:…` image pins. Hashed before
    /// creation and required to be the `argv[0]` of every command.
    pub native_executable: Option<PathBuf>,
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
            native_executable: None,
        }
    }
}

/// The OpenShell compute drivers, by the name `sandbox.json` gives them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputeDriver {
    Vm,
    Mxc,
    Docker,
    Podman,
    Kubernetes,
}

/// What a driver enforces, or what a policy requires of its driver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Enforcement {
    pub landlock_hard: bool,
    pub linux_process_identity: bool,
    pub egress_proxy: bool,
}

impl Enforcement {
    fn json(self) -> Value {
        json!({
            "landlock_hard": self.landlock_hard,
            "linux_process_identity": self.linux_process_identity,
            "egress_proxy": self.egress_proxy,
        })
    }
}

impl ComputeDriver {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "vm" => Some(Self::Vm),
            "mxc" => Some(Self::Mxc),
            "docker" => Some(Self::Docker),
            "podman" => Some(Self::Podman),
            "kubernetes" => Some(Self::Kubernetes),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Vm => "vm",
            Self::Mxc => "mxc",
            Self::Docker => "docker",
            Self::Podman => "podman",
            Self::Kubernetes => "kubernetes",
        }
    }

    /// What the driver enforces, from OpenShell 0.1.3. `None` means SEA-Forge
    /// has no table for the driver, so the projection is refused.
    ///
    /// * `vm`: `openshell-driver-vm` runs the workload in a libkrun MicroVM
    ///   whose guest init and supervisor apply the policy. Upstream's design
    ///   claims all three guarantees. This is not live-proven here (see the
    ///   subsystem document, section 9).
    /// * `mxc`: `crates/openshell-driver-mxc/README.md`. The workload runs in
    ///   an AppContainer through `wxc-exec`, which cannot enforce Landlock or
    ///   Linux process identity. Network goes only through the egress proxy,
    ///   and the driver fails closed when egress is disabled.
    fn enforces(self) -> Option<Enforcement> {
        match self {
            Self::Vm => Some(Enforcement {
                landlock_hard: true,
                linux_process_identity: true,
                egress_proxy: true,
            }),
            Self::Mxc => Some(Enforcement {
                landlock_hard: false,
                linux_process_identity: false,
                egress_proxy: true,
            }),
            Self::Docker | Self::Podman | Self::Kubernetes => None,
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
    pub compute_driver: ComputeDriver,
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
    let cell_id = text_field(&lock, "/cell_id", "openshell.lock")?;
    let model_hash = text_field(&lock, "/model/hash", "openshell.lock")?;
    let policy_digest = text_field(&lock, "/policy_digest", "openshell.lock")?;
    let openshell_version = text_field(&lock, "/openshell_version", "openshell.lock")?;
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
    if text_field(&lock, "/files/policy.yaml", "openshell.lock")? != policy_digest {
        return Err(err(
            "openshell_projection_tampered",
            "openshell.lock is internally inconsistent",
        ));
    }
    // Read only after the lock has verified sandbox.json.
    let sandbox = read_json(&projection_dir.join("sandbox.json"))?;
    let driver = text_field(&sandbox, "/compute_driver", "sandbox.json")?;
    let compute_driver = ComputeDriver::parse(&driver).ok_or_else(|| {
        err(
            "openshell_projection_error",
            format!("sandbox.json declares an unknown compute_driver `{driver}`"),
        )
    })?;
    Ok(ProjectionIdentity {
        cell_id,
        model_hash,
        policy_digest,
        openshell_version,
        compute_driver,
    })
}

/// Copies the projection's files into `snapshot`, reading each source file
/// once, and verifies the copy. Every later read goes to the snapshot, so a
/// file swapped in the source directory after this point is never used.
fn snapshot_projection(source: &Path, snapshot: &Path) -> Result<ProjectionIdentity, SandboxError> {
    use std::io::Write;
    match fs::symlink_metadata(snapshot) {
        Ok(_) => fs::remove_dir_all(snapshot).map_err(|e| err("io_error", e.to_string()))?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(err("io_error", e.to_string())),
    }
    fs::create_dir_all(snapshot).map_err(|e| err("io_error", e.to_string()))?;
    for name in PROJECTION_ARTIFACTS
        .iter()
        .copied()
        .chain(["openshell.lock"])
    {
        let from = source.join(name);
        let bytes = fs::read(&from).map_err(|e| {
            err(
                "openshell_projection_error",
                format!("{}: {e}", from.display()),
            )
        })?;
        let mut copy = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(snapshot.join(name))
            .map_err(|e| err("io_error", e.to_string()))?;
        copy.write_all(&bytes)
            .map_err(|e| err("io_error", e.to_string()))?;
    }
    // The source directory must hold exactly the artifacts, as the lock says;
    // an extra file is not part of the authorized projection.
    let present = fs::read_dir(source)
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
    projection_identity(snapshot)
}

/// The host facts the local driver check needs. Gathered once, then judged by
/// `host_verdict`, which is pure and unit-tested for every platform.
struct HostFacts {
    os: &'static str,
    arch: &'static str,
    /// `/dev/kvm` opened, or the open error. Linux only.
    kvm: Result<(), String>,
    /// Whether `/proc/version` names a Microsoft kernel (WSL). Linux only.
    wsl: bool,
    /// `kern.hv_support` as printed by `sysctl`. macOS only.
    hv_support: String,
}

fn host_facts() -> HostFacts {
    let os = std::env::consts::OS;
    let linux = os == "linux";
    let macos = os == "macos";
    HostFacts {
        os,
        arch: std::env::consts::ARCH,
        kvm: if linux {
            File::open("/dev/kvm").map(drop).map_err(|e| e.to_string())
        } else {
            Ok(())
        },
        wsl: linux
            && fs::read_to_string("/proc/version")
                .map(|v| v.to_ascii_lowercase().contains("microsoft"))
                .unwrap_or(false),
        hv_support: if macos {
            // Run with a cleared environment: the value does not depend on the caller.
            Command::new("/usr/sbin/sysctl")
                .args(["-n", "kern.hv_support"])
                .env_clear()
                .output()
                .ok()
                .filter(|out| out.status.success())
                .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
                .unwrap_or_else(|| "unavailable".to_string())
        } else {
            String::new()
        },
    }
}

fn host_verdict(driver: ComputeDriver, host: &HostFacts) -> Result<(), SandboxError> {
    match driver {
        ComputeDriver::Vm => match host.os {
            "linux" => match &host.kvm {
                Ok(()) => Ok(()),
                Err(e) => {
                    let hint = if host.wsl {
                        " This is WSL2: enable nested virtualization (`nestedVirtualization=true` under [wsl2] in .wslconfig) and restart WSL."
                    } else {
                        " Enable KVM and grant this user access to /dev/kvm."
                    };
                    Err(err(
                        "openshell_virtualization_unavailable",
                        format!("cannot open /dev/kvm: {e}. OpenShell MicroVMs need KVM on Linux.{hint}"),
                    ))
                }
            },
            "macos" if host.arch == "aarch64" && host.hv_support.trim() == "1" => Ok(()),
            "macos" => Err(err(
                "openshell_virtualization_unavailable",
                format!(
                    "the vm driver on macOS needs Apple Silicon and kern.hv_support=1; this host has arch {} and kern.hv_support `{}`",
                    host.arch, host.hv_support
                ),
            )),
            os => Err(err(
                "openshell_driver_unsupported_on_host",
                format!("the vm driver runs on Linux (KVM) and on macOS (Apple Silicon); this host is {os}"),
            )),
        },
        ComputeDriver::Mxc if host.os == "windows" => Ok(()),
        ComputeDriver::Mxc => Err(err(
            "openshell_driver_unsupported_on_host",
            format!("the mxc driver runs only on Windows; this host is {}", host.os),
        )),
        other => Err(err(
            "openshell_driver_unsupported",
            format!("the {} driver is not supported by SEA-Forge", other.name()),
        )),
    }
}

/// Whether this host provides what `driver` needs. Called when the gateway
/// runs its compute driver on this host (`local_microvm_gateway`).
fn probe_local_driver(driver: ComputeDriver) -> Result<(), SandboxError> {
    host_verdict(driver, &host_facts())
}

fn present(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => false,
        Some(Value::String(text)) => !text.is_empty(),
        Some(Value::Array(items)) => !items.is_empty(),
        Some(Value::Object(map)) => !map.is_empty(),
        Some(_) => true,
    }
}

/// The guarantees `policy.yaml` demands. Only the three fields the enforcement
/// check needs are read; the policy's meaning is OpenShell's and the prover's.
fn policy_requires(policy: &Path) -> Result<Enforcement, SandboxError> {
    let refuse = |e: String| {
        err(
            "openshell_projection_error",
            format!("{}: {e}", policy.display()),
        )
    };
    let bytes = fs::read(policy).map_err(|e| refuse(e.to_string()))?;
    let doc: Value = sea_forge_core::yaml::from_slice(&bytes).map_err(|e| refuse(e.to_string()))?;
    Ok(Enforcement {
        landlock_hard: doc
            .pointer("/landlock/compatibility")
            .and_then(Value::as_str)
            == Some("hard_requirement"),
        linux_process_identity: present(doc.pointer("/process/run_as_user"))
            || present(doc.pointer("/process/run_as_group")),
        egress_proxy: present(doc.get("network_policies")),
    })
}

/// The enforcement capability check. It runs before anything reaches the
/// gateway, and returns the record that `execution.json` keeps.
fn check_enforcement(driver: ComputeDriver, policy: &Path) -> Result<Value, SandboxError> {
    let provides = driver.enforces().ok_or_else(|| {
        err(
            "openshell_driver_unsupported",
            format!(
                "the {} driver has no enforcement table in SEA-Forge",
                driver.name()
            ),
        )
    })?;
    let requires = policy_requires(policy)?;
    let gaps: Vec<&str> = [
        (
            requires.landlock_hard && !provides.landlock_hard,
            "a Landlock hard_requirement",
        ),
        (
            requires.linux_process_identity && !provides.linux_process_identity,
            "Linux process identity (process.run_as_user or run_as_group)",
        ),
        (
            requires.egress_proxy && !provides.egress_proxy,
            "an egress proxy (network_policies)",
        ),
    ]
    .into_iter()
    .filter(|(gap, _)| *gap)
    .map(|(_, name)| name)
    .collect();
    if !gaps.is_empty() {
        return Err(err(
            "openshell_enforcement_unavailable",
            format!(
                "the {} driver cannot enforce {}",
                driver.name(),
                gaps.join("; ")
            ),
        ));
    }
    Ok(json!({
        "driver": driver.name(),
        "provides": provides.json(),
        "requires": requires.json(),
    }))
}

/// `vm` runs OCI images; `mxc` runs native executables.
fn check_workload_kind(
    driver: ComputeDriver,
    workload: &WorkloadImage,
) -> Result<(), SandboxError> {
    let refused = |message: &str| {
        Err(err(
            "openshell_workload_kind_mismatch",
            format!("{message}; the binding's image is the wrong kind for the projection's driver"),
        ))
    };
    match (driver, workload) {
        (ComputeDriver::Vm, WorkloadImage::Native { .. }) => {
            refused("the vm driver runs OCI images, not native executables")
        }
        (ComputeDriver::Mxc, WorkloadImage::Oci { .. }) => {
            refused("the mxc driver runs native executables, not OCI images")
        }
        _ => Ok(()),
    }
}

struct Output {
    code: Option<i64>,
    stdout: String,
    stderr: String,
}

/// Run a control command with a clean environment and a deadline.
fn run(
    program: &Path,
    args: &[String],
    cwd: Option<&Path>,
    timeout: Duration,
) -> Result<Output, SandboxError> {
    let mut command = Command::new(program);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
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

/// The identity the runtime authorized for a projection. When set, `prepare`
/// refuses a projection snapshot that does not carry exactly these values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApprovedProjection {
    pub cell_id: String,
    pub model_hash: String,
    pub policy_digest: String,
}

pub struct OpenShellSandbox {
    binding: OpenShellBinding,
    approved: Option<ApprovedProjection>,
}

impl OpenShellSandbox {
    pub fn new(binding: OpenShellBinding) -> Self {
        Self {
            binding,
            approved: None,
        }
    }

    /// Require the projection snapshot to carry the identity the runtime
    /// authorized, rather than whatever the projection directory holds now.
    pub fn approving(mut self, approved: ApprovedProjection) -> Self {
        self.approved = Some(approved);
        self
    }

    /// The binding's image, parsed. Refused when it is neither kind.
    fn workload(&self) -> Result<WorkloadImage, SandboxError> {
        let image = self.binding.image.as_deref().unwrap_or_default();
        WorkloadImage::parse(image).map_err(|e| err("openshell_image_invalid", e.to_string()))
    }

    /// A native image pins the executable's sha256 and the platform it runs
    /// on. The file is hashed here, before creation. The gateway later opens
    /// the same path itself, so this check is not a snapshot of what runs.
    fn verify_native_identity(&self, workload: &WorkloadImage) -> Result<(), SandboxError> {
        let WorkloadImage::Native { sha256, platform } = workload else {
            return Ok(());
        };
        let mismatch = |message: String| err("openshell_executable_identity_mismatch", message);
        let host = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
        if *platform != host {
            return Err(mismatch(format!(
                "the native image is for {platform}, and this host is {host}"
            )));
        }
        let executable = self
            .binding
            .native_executable
            .as_ref()
            .ok_or_else(|| mismatch("the native image declares no native_executable".into()))?;
        let bytes =
            fs::read(executable).map_err(|e| mismatch(format!("{}: {e}", executable.display())))?;
        let actual = hex::encode(Sha256::digest(&bytes));
        if actual != *sha256 {
            return Err(mismatch(format!(
                "{} hashes to sha256:{actual}, but the image pins sha256:{sha256}",
                executable.display()
            )));
        }
        Ok(())
    }

    fn cli(&self, args: &[String], timeout: Duration) -> Result<Output, SandboxError> {
        self.cli_in(args, None, timeout)
    }

    fn cli_in(
        &self,
        args: &[String],
        cwd: Option<&Path>,
        timeout: Duration,
    ) -> Result<Output, SandboxError> {
        let mut full = Vec::new();
        if let Some(gateway) = &self.binding.gateway {
            full.push("--gateway".to_string());
            full.push(gateway.clone());
        }
        full.extend_from_slice(args);
        run(&self.binding.cli, &full, cwd, timeout)
    }

    fn cli_ok(
        &self,
        class: &'static str,
        args: &[String],
        timeout: Duration,
    ) -> Result<Output, SandboxError> {
        self.cli_ok_in(class, args, None, timeout)
    }

    fn cli_ok_in(
        &self,
        class: &'static str,
        args: &[String],
        cwd: Option<&Path>,
        timeout: Duration,
    ) -> Result<Output, SandboxError> {
        let out = self.cli_in(args, cwd, timeout)?;
        if out.code != Some(0) {
            return Err(err(
                class,
                format!("openshell {} failed: {}", args.join(" "), out.stderr.trim()),
            ));
        }
        Ok(out)
    }

    /// The compute driver the gateway reports, which must be the projection's.
    /// `gateway info` is the only 0.1.3 surface that names it, and it needs
    /// admin rights, so a gateway that cannot answer is refused.
    fn gateway_driver(&self, expected: ComputeDriver) -> Result<Value, SandboxError> {
        let info = self.cli_ok(
            "openshell_driver_unverified",
            &strings(&["gateway", "info", "-o", "json"]),
            CONTROL_TIMEOUT,
        )?;
        let info: Value = serde_json::from_str(&info.stdout).map_err(|e| {
            err(
                "openshell_driver_unverified",
                format!("gateway info is not JSON: {e}"),
            )
        })?;
        let reported: Vec<String> = info
            .pointer("/compute_drivers")
            .and_then(Value::as_array)
            .map(|drivers| {
                drivers
                    .iter()
                    .filter_map(|d| d.pointer("/capabilities/driver_name"))
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        if reported.is_empty() {
            return Err(err(
                "openshell_driver_unverified",
                "gateway info names no compute driver",
            ));
        }
        if reported != [expected.name().to_string()] {
            return Err(err(
                "openshell_driver_mismatch",
                format!(
                    "the projection requires the {} driver, the gateway reports {reported:?}",
                    expected.name()
                ),
            ));
        }
        Ok(json!({
            "driver": expected.name(),
            "source": "gateway_info",
            "gateway_reports": reported,
        }))
    }

    /// One sandbox per artifacts directory, which is unique per run.
    fn name(spec: &SandboxSpec) -> String {
        let digest = Sha256::digest(spec.artifacts_root.to_string_lossy().as_bytes());
        format!("sf-{}", &hex::encode(digest)[..16])
    }

    fn evidence_dir(spec: &SandboxSpec) -> PathBuf {
        spec.artifacts_root.join("openshell")
    }

    /// The snapshot of the projection taken by `prepare`. Every read after
    /// preparation goes here, never to the caller's projection directory.
    fn projection_dir(spec: &SandboxSpec) -> PathBuf {
        Self::evidence_dir(spec).join("projection")
    }

    fn workspace_path(&self, spec: &SandboxSpec) -> Result<String, SandboxError> {
        let sandbox = read_json(&Self::projection_dir(spec).join("sandbox.json"))?;
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
    fn prove(
        &self,
        spec: &SandboxSpec,
        candidate: &Path,
        class: &'static str,
    ) -> Result<Value, SandboxError> {
        let boundary = Self::projection_dir(spec).join("policy.yaml");
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
            None,
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
                self.workspace_path(&h.spec)?,
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
        enforcement: &Value,
        driver: &Value,
        workload: &WorkloadImage,
    ) -> Result<(), SandboxError> {
        let evidence = Self::evidence_dir(spec);
        fs::create_dir_all(&evidence).map_err(|e| err("io_error", e.to_string()))?;
        let policy = Self::projection_dir(spec).join("policy.yaml");

        let projected = self.prove(spec, &policy, "openshell_policy_unverified")?;

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
        // A native executable is not an image: the gateway's default applies.
        if let WorkloadImage::Oci { reference } = workload {
            args.push("--from".to_string());
            args.push(reference.clone());
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
            spec,
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

        // A named directory uploads as `<dest>/<dirname>/...` (like `scp -r`);
        // only `.` extracts flat. So the CLI runs inside the workspace and
        // uploads `.`, matching the flat `tar -C <path> .` of the download.
        self.cli_ok_in(
            "openshell_transfer_failed",
            &[
                "sandbox".to_string(),
                "upload".to_string(),
                name.to_string(),
                ".".to_string(),
                self.workspace_path(spec)?,
                "--no-git-ignore".to_string(),
            ],
            Some(&spec.workspace_root),
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
            "enforcement": enforcement,
            "compute_driver": driver,
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
        let workload = self.workload()?;
        // Snapshot first: the identity checked, the policy proved and the
        // policy created are all read from this copy, never the source.
        let snapshot = Self::projection_dir(spec);
        let identity = snapshot_projection(&self.binding.projection_dir, &snapshot)?;
        if let Some(approved) = &self.approved {
            let snapshot_identity = (
                &identity.cell_id,
                &identity.model_hash,
                &identity.policy_digest,
            );
            if snapshot_identity
                != (
                    &approved.cell_id,
                    &approved.model_hash,
                    &approved.policy_digest,
                )
            {
                return Err(err(
                    "openshell_projection_tampered",
                    "the projection differs from the identity the runtime authorized",
                ));
            }
        }
        let enforcement =
            check_enforcement(identity.compute_driver, &snapshot.join("policy.yaml"))?;
        check_workload_kind(identity.compute_driver, &workload)?;

        let version = run(
            &self.binding.cli,
            &strings(&["--version"]),
            None,
            CONTROL_TIMEOUT,
        )?;
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
            probe_local_driver(identity.compute_driver)?;
        }
        self.verify_native_identity(&workload)?;
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

        let driver = self.gateway_driver(identity.compute_driver)?;
        let name = Self::name(spec);
        if let Err(error) = self.create(spec, &name, &identity, &enforcement, &driver, &workload) {
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
        // A native image runs exactly the executable it pins, never another.
        if let WorkloadImage::Native { .. } = self.workload()? {
            let declared = self
                .binding
                .native_executable
                .as_ref()
                .map(|p| p.display().to_string());
            if declared.as_deref() != argv.first().map(String::as_str) {
                return Err(err(
                    "openshell_executable_identity_mismatch",
                    "argv[0] is not the native executable the image pins",
                ));
            }
        }
        let workspace = self.workspace_path(&h.spec)?;
        let workdir = if cwd == "." {
            workspace
        } else {
            format!("{workspace}/{cwd}")
        };

        fs::create_dir_all(&h.spec.artifacts_root).map_err(|e| err("io_error", e.to_string()))?;
        let stdout =
            crate::artifact_output::create_output_file(&h.spec.artifacts_root.join("stdout.txt"))
                .map_err(|e| err("io_error", e.to_string()))?;
        let stderr =
            crate::artifact_output::create_output_file(&h.spec.artifacts_root.join("stderr.txt"))
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

        let result = |status, exit_code| {
            crate::local::execution_result(status, exit_code, started_at.clone())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn host(os: &'static str, arch: &'static str) -> HostFacts {
        HostFacts {
            os,
            arch,
            kvm: Ok(()),
            wsl: false,
            hv_support: String::new(),
        }
    }

    fn class(result: Result<(), SandboxError>) -> &'static str {
        match result {
            Ok(()) => "ok",
            Err(error) => error.class,
        }
    }

    #[test]
    fn vm_on_windows_is_refused_as_unsupported_on_host() {
        assert_eq!(
            class(host_verdict(ComputeDriver::Vm, &host("windows", "x86_64"))),
            "openshell_driver_unsupported_on_host"
        );
    }

    #[test]
    fn vm_on_linux_needs_kvm() {
        let mut facts = host("linux", "x86_64");
        assert_eq!(class(host_verdict(ComputeDriver::Vm, &facts)), "ok");
        facts.kvm = Err("Permission denied".into());
        assert_eq!(
            class(host_verdict(ComputeDriver::Vm, &facts)),
            "openshell_virtualization_unavailable"
        );
    }

    #[test]
    fn vm_on_macos_needs_apple_silicon_and_hypervisor_support() {
        let mut facts = host("macos", "aarch64");
        facts.hv_support = "1\n".into();
        assert_eq!(class(host_verdict(ComputeDriver::Vm, &facts)), "ok");
        facts.hv_support = "0".into();
        assert_eq!(
            class(host_verdict(ComputeDriver::Vm, &facts)),
            "openshell_virtualization_unavailable"
        );
        facts.hv_support = "1".into();
        facts.arch = "x86_64";
        assert_eq!(
            class(host_verdict(ComputeDriver::Vm, &facts)),
            "openshell_virtualization_unavailable"
        );
    }

    #[test]
    fn mxc_runs_only_on_windows() {
        assert_eq!(
            class(host_verdict(ComputeDriver::Mxc, &host("windows", "x86_64"))),
            "ok"
        );
        assert_eq!(
            class(host_verdict(ComputeDriver::Mxc, &host("linux", "x86_64"))),
            "openshell_driver_unsupported_on_host"
        );
    }

    #[test]
    fn container_drivers_have_no_enforcement_table() {
        for driver in [
            ComputeDriver::Docker,
            ComputeDriver::Podman,
            ComputeDriver::Kubernetes,
        ] {
            assert!(driver.enforces().is_none());
            assert_eq!(
                class(host_verdict(driver, &host("linux", "x86_64"))),
                "openshell_driver_unsupported"
            );
        }
    }

    #[test]
    fn unknown_compute_driver_names_do_not_parse() {
        assert_eq!(ComputeDriver::parse("firecracker"), None);
        assert_eq!(ComputeDriver::parse("vm"), Some(ComputeDriver::Vm));
    }
}
