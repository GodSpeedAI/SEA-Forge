use sea_forge_core::{errors::ForgeError, types::*};
use std::{
    fmt, fs,
    path::{Component, Path, PathBuf},
};

mod artifact_output;
pub mod environment;
#[cfg(target_os = "linux")]
pub mod isolated;
pub mod jail;
pub mod local;
mod observation;
pub mod openshell;
#[cfg(target_os = "linux")]
pub mod socket_policy;
#[cfg(target_os = "linux")]
pub mod syscall_filter;
#[cfg(target_os = "linux")]
pub mod workload;

pub use observation::{open_observation_file, open_observation_root};

pub use environment::{
    demo_environment, load_pinned_environment, materialize_base, parse_env_ref,
    parse_evaluator_score, store_builtin_environment, BaseFile, EnvironmentSpec, Evaluator,
    Provides, ScoreFrom,
};
#[cfg(target_os = "linux")]
pub use isolated::IsolatedSandbox;
pub use jail::JailSandbox;
pub use local::LocalSandbox;
pub use openshell::{
    projection_identity, ApprovedProjection, OpenShellBinding, OpenShellSandbox, ProjectionIdentity,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxClass {
    Local,
    Jail,
    /// One exclusive subordinate host identity per execution (SEC-01).
    Isolated,
    Microvm,
    OpenShell,
}

impl fmt::Display for SandboxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SandboxClass::Local => write!(f, "local"),
            SandboxClass::Jail => write!(f, "jail"),
            SandboxClass::Isolated => write!(f, "isolated"),
            SandboxClass::Microvm => write!(f, "microvm"),
            SandboxClass::OpenShell => write!(f, "openshell"),
        }
    }
}

impl std::str::FromStr for SandboxClass {
    type Err = ForgeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(SandboxClass::Local),
            "jail" => Ok(SandboxClass::Jail),
            "isolated" => Ok(SandboxClass::Isolated),
            "microvm" => Ok(SandboxClass::Microvm),
            "openshell" => Ok(SandboxClass::OpenShell),
            other => Err(ForgeError::Input(format!("unknown sandbox class: {other}"))),
        }
    }
}

/// Outbound/listening network posture a jail-class run is permitted.
///
/// Derived from authority/policy (the grant's `network` boundary), never from
/// child-controlled input. The default is [`NetworkPosture::Denied`]: a jail
/// run may not open outbound TCP connections or bind/listen on TCP sockets.
///
/// Scope note: enforcement is TCP-only (Landlock v4+ `AccessNet::{BindTcp,
/// ConnectTcp}`). UDP and raw sockets are an explicitly documented, out-of-scope
/// gap — Landlock has no coverage for them through ABI v6 (see
/// `docs/decisions/ADR-002-audit-remediation-dependencies.md` §2).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum NetworkPosture {
    /// No outbound TCP connect and no TCP bind/listen (default, fail-closed).
    #[default]
    Denied,
    /// Explicit grant: only the listed TCP ports may be bound and connected to.
    /// An empty port list is equivalent to [`NetworkPosture::Denied`] and is
    /// normalized to it at construction time.
    AllowTcpPorts(Vec<u16>),
}

impl NetworkPosture {
    /// Build a posture from an authority-derived set of granted TCP ports.
    /// An empty grant collapses to [`NetworkPosture::Denied`] so that "no ports
    /// granted" and "network denied" are the same fail-closed state.
    pub fn from_granted_ports(ports: impl IntoIterator<Item = u16>) -> Self {
        let mut ports: Vec<u16> = ports.into_iter().collect();
        ports.sort_unstable();
        ports.dedup();
        if ports.is_empty() {
            NetworkPosture::Denied
        } else {
            NetworkPosture::AllowTcpPorts(ports)
        }
    }

    /// TCP ports this posture explicitly grants (empty when denied).
    pub fn granted_ports(&self) -> &[u16] {
        match self {
            NetworkPosture::Denied => &[],
            NetworkPosture::AllowTcpPorts(ports) => ports,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SandboxSpec {
    pub workspace_root: PathBuf,
    pub artifacts_root: PathBuf,
    /// Authority-derived network posture. Defaults to [`NetworkPosture::Denied`].
    pub network: NetworkPosture,
}

#[derive(Clone, Debug)]
pub struct SandboxHandle {
    pub class: SandboxClass,
    pub spec: SandboxSpec,
}

#[derive(Clone, Debug)]
pub struct RelPath(pub String);

#[derive(Debug)]
pub struct SandboxError {
    pub class: &'static str,
    pub message: String,
}

impl SandboxError {
    pub fn new(class: &'static str, message: impl Into<String>) -> Self {
        Self {
            class,
            message: message.into(),
        }
    }
}

impl fmt::Display for SandboxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.class, self.message)
    }
}

impl std::error::Error for SandboxError {}

impl From<SandboxError> for ForgeError {
    fn from(e: SandboxError) -> Self {
        ForgeError::Config {
            class: e.class,
            path: PathBuf::new(),
            message: e.message,
        }
    }
}

pub trait ExecutionSandbox {
    fn class(&self) -> SandboxClass;
    fn prepare(&self, spec: &SandboxSpec) -> Result<SandboxHandle, SandboxError>;
    fn execute(
        &self,
        h: &SandboxHandle,
        req: &ExecutionRequest,
    ) -> Result<ExecutionResult, SandboxError>;
    fn collect_artifacts(
        &self,
        h: &SandboxHandle,
        paths: &[RelPath],
    ) -> Result<Vec<ArtifactRef>, SandboxError>;
    fn destroy(&self, h: SandboxHandle) -> Result<(), SandboxError>;
}

/// Select a sandbox backend for the given class.
/// Returns an error if the class is unavailable on this host.
pub fn select_sandbox(class: SandboxClass) -> Result<Box<dyn ExecutionSandbox>, SandboxError> {
    match class {
        SandboxClass::Local => Ok(Box::new(LocalSandbox)),
        SandboxClass::Jail => Ok(Box::new(JailSandbox::new()?)),
        #[cfg(target_os = "linux")]
        SandboxClass::Isolated => Ok(Box::new(IsolatedSandbox::new(Vec::new())?)),
        #[cfg(not(target_os = "linux"))]
        SandboxClass::Isolated => Err(SandboxError::new(
            "workload_isolation_unavailable",
            "isolated workloads require Linux",
        )),
        SandboxClass::Microvm => Err(SandboxError::new(
            "unsupported_sandbox_class_error",
            "microvm backend is not available",
        )),
        SandboxClass::OpenShell => Err(SandboxError::new(
            "openshell_binding_missing",
            "the openshell backend needs a projection binding; use select_openshell",
        )),
    }
}

/// The OpenShell backend cannot be selected by class alone: it must be bound
/// to the DomainForge projection that defines the sandbox.
pub fn select_openshell(binding: OpenShellBinding) -> Box<dyn ExecutionSandbox> {
    Box::new(OpenShellSandbox::new(binding))
}

pub fn validate_relative_path(path: &str) -> Result<(), ForgeError> {
    // Delegates to the shared canonical-path primitive (sea_forge_core::path).
    // In addition to the permissive charset/component checks, it rejects the
    // ambiguous spellings `a//b`, `a/./b`, and `a/` so a non-canonical path
    // can never alias a canonical one through materialization.
    sea_forge_core::path::validate_relative_path(path)
}

/// Purely lexical safe join for paths that may not exist yet.
///
/// Unlike [`safe_join`], this performs no filesystem access: it does not
/// canonicalize `root`, create parents, or inspect symlinks. It is the correct
/// primitive for validating an untrusted, manifest-controlled path *before* the
/// first filesystem mutation (e.g. before a staging subtree is created).
///
/// It rejects any path that [`validate_relative_path`] rejects (absolute paths,
/// `..`, root/prefix components, non `[A-Za-z0-9._/-]` characters). In addition
/// it rejects ambiguous spellings — empty segments (from `a//b` or a trailing
/// `/`) and `.` segments (from `a/./b`) — so every accepted input has exactly
/// one canonical, unambiguous rendering. Callers relying on this for
/// duplicate-detection get a single normalized form per logical destination.
///
/// The returned path is `root` joined with the validated, normalized relative
/// path. Symlink-parent escapes are NOT covered here (they require filesystem
/// inspection); use [`safe_join`] for the final write once parents exist.
pub fn safe_lexical_join(root: &Path, rel: &str) -> Result<PathBuf, ForgeError> {
    validate_relative_path(rel)?;
    let relative = Path::new(rel);
    let mut normalized = PathBuf::new();
    for component in relative.components() {
        match component {
            Component::Normal(name) => normalized.push(name),
            Component::CurDir => {
                return Err(ForgeError::UnsafePath(format!(
                    "ambiguous relative path (`.` segment): {rel}"
                )))
            }
            // `validate_relative_path` already rejects these, but stay total.
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(ForgeError::UnsafePath(format!(
                    "unsafe relative path: {rel}"
                )))
            }
        }
    }
    // An empty segment (e.g. `a//b`, `a/`, or `/`) collapses under `components()`
    // above; detect it via the raw spelling so `a//b` and `a/b` cannot alias.
    if rel != "." && (rel.contains("//") || rel.ends_with('/') || rel.starts_with('/')) {
        return Err(ForgeError::UnsafePath(format!(
            "ambiguous relative path (empty segment): {rel}"
        )));
    }
    if normalized.as_os_str().is_empty() {
        return Err(ForgeError::UnsafePath(format!(
            "empty relative path: {rel}"
        )));
    }
    Ok(root.join(normalized))
}

pub fn safe_join(root: &Path, rel: &str) -> Result<PathBuf, ForgeError> {
    validate_relative_path(rel)?;
    fs::create_dir_all(root).map_err(|e| ForgeError::io("create workspace", e))?;
    let canonical_root = root
        .canonicalize()
        .map_err(|e| ForgeError::io("canonicalize workspace", e))?;
    if rel == "." {
        return Ok(canonical_root);
    }
    let relative = Path::new(rel);
    checked_parent(&canonical_root, relative, rel, true)?;
    let candidate = canonical_root.join(relative);
    if fs::symlink_metadata(&candidate).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(ForgeError::UnsafePath(format!(
            "target is a symlink: {rel}"
        )));
    }
    Ok(candidate)
}
pub fn safe_existing(root: &Path, rel: &str) -> Result<PathBuf, ForgeError> {
    validate_relative_path(rel)?;
    let canonical_root = root
        .canonicalize()
        .map_err(|e| ForgeError::io("canonicalize workspace", e))?;
    let relative = Path::new(rel);
    checked_parent(&canonical_root, relative, rel, false)?;
    let candidate = canonical_root.join(relative);
    if fs::symlink_metadata(&candidate)
        .map_err(|e| ForgeError::io("inspect existing workspace path", e))?
        .file_type()
        .is_symlink()
    {
        return Err(ForgeError::UnsafePath(format!(
            "existing path is a symlink: {rel}"
        )));
    }
    let canonical = candidate
        .canonicalize()
        .map_err(|e| ForgeError::io("canonicalize existing workspace path", e))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(ForgeError::UnsafePath(format!(
            "existing path escapes workspace: {rel}"
        )));
    }
    Ok(canonical)
}
/// Windows: every parent component is walked through pinned directory
/// handles (`sea_forge_os::PinnedDir`). Reparse points are refused, and each
/// step is checked to stay inside `canonical_root`. With `create_missing`, a
/// missing component is created under its pinned parent, one at a time. The
/// previous pathname-based `create_dir` could follow a junction swapped in
/// after validation, so it is not used on Windows.
#[cfg(windows)]
fn checked_parent(
    canonical_root: &Path,
    relative: &Path,
    original: &str,
    create_missing: bool,
) -> Result<PathBuf, ForgeError> {
    let parent = relative
        .parent()
        .ok_or_else(|| ForgeError::UnsafePath(original.into()))?;
    let names: Vec<&std::ffi::OsStr> = parent
        .components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name),
            _ => None,
        })
        .collect();
    pinned_walk(canonical_root, &names, create_missing)?;
    Ok(canonical_root.join(parent))
}

#[cfg(not(windows))]
fn checked_parent(
    canonical_root: &Path,
    relative: &Path,
    original: &str,
    create_missing: bool,
) -> Result<PathBuf, ForgeError> {
    let parent = relative
        .parent()
        .ok_or_else(|| ForgeError::UnsafePath(original.into()))?;
    let mut safe_parent = canonical_root.to_path_buf();
    for component in parent.components() {
        let Component::Normal(name) = component else {
            continue;
        };
        let next = safe_parent.join(name);
        match fs::symlink_metadata(&next) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(ForgeError::UnsafePath(format!(
                    "parent is a symlink: {original}"
                )))
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(ForgeError::UnsafePath(format!(
                    "parent is not a directory: {original}"
                )))
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && create_missing => {
                fs::create_dir(&next)
                    .map_err(|error| ForgeError::io("create workspace parent", error))?
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(ForgeError::UnsafePath(format!(
                    "parent does not exist: {original}"
                )))
            }
            Err(error) => return Err(ForgeError::io("inspect workspace parent", error)),
        }
        safe_parent = next;
    }
    Ok(safe_parent)
}
/// F-25.i / F04: replaces the destination entry, never writing an existing
/// inode. The bytes go to a fresh temp sibling created with
/// `O_CREAT|O_EXCL|O_NOFOLLOW` in the destination's parent, which is opened
/// with `O_DIRECTORY|O_NOFOLLOW`. The temp file is fsynced and then renamed
/// over the destination with `renameat`, relative to that directory
/// descriptor. An existing hard link to the old entry therefore keeps its
/// bytes, because only the directory entry changes. A symlink at the
/// destination is refused before anything is written, and a directory fails
/// with `EISDIR`.
///
/// The parent is opened by path, and its containment is not checked here.
/// Callers that need containment use `safe_write_within`. A concurrent hard
/// link created after the check is harmless: the rename replaces the entry,
/// and the link keeps the old inode, so no race can make this write reach a
/// shared inode.
#[cfg(unix)]
pub fn safe_write(destination: &Path, bytes: &[u8]) -> Result<(), ForgeError> {
    use rustix::fs::{Mode, OFlags};

    let parent = destination.parent().ok_or_else(|| {
        ForgeError::UnsafePath(format!(
            "destination has no parent: {}",
            destination.display()
        ))
    })?;
    let name = destination.file_name().ok_or_else(|| {
        ForgeError::UnsafePath(format!(
            "destination has no file name: {}",
            destination.display()
        ))
    })?;
    let dir = rustix::fs::open(
        parent,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| {
        ForgeError::io(
            format!("open planned file {}", destination.display()),
            std::io::Error::from(error),
        )
    })?;
    write_entry_unix(&dir, destination, name, bytes)
}

/// Unix: replaces the entry `name` inside the already-open directory `dir`.
/// Shared by `safe_write` and `safe_write_within`. The bytes go to a fresh
/// `O_CREAT|O_EXCL` temp sibling, which is fsynced and renamed over `name`.
/// Afterwards the entry is checked against the written file (see below).
#[cfg(unix)]
fn write_entry_unix(
    dir: &std::os::fd::OwnedFd,
    destination: &Path,
    name: &std::ffi::OsStr,
    bytes: &[u8],
) -> Result<(), ForgeError> {
    use rustix::fs::{AtFlags, Mode, OFlags};
    use rustix::io::Errno;
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);

    let open_error = |error: Errno| {
        ForgeError::io(
            format!("open planned file {}", destination.display()),
            std::io::Error::from(error),
        )
    };
    // The mode of an existing entry is kept; a new file gets 0666 minus the umask.
    let mode = match rustix::fs::statat(dir, name, AtFlags::SYMLINK_NOFOLLOW) {
        Ok(stat) => match stat.st_mode & libc::S_IFMT {
            libc::S_IFLNK => {
                return Err(ForgeError::UnsafePath(format!(
                    "destination {} was swapped to a symlink during materialization",
                    destination.display()
                )))
            }
            libc::S_IFDIR => {
                return Err(open_error(Errno::ISDIR));
            }
            _ => stat.st_mode & 0o7777,
        },
        Err(Errno::NOENT) => 0o666,
        Err(error) => return Err(open_error(error)),
    };
    let mut attempts = 0;
    let (temp_name, fd) = loop {
        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let mut candidate = std::ffi::OsString::from(".");
        candidate.push(name);
        candidate.push(format!(".sf-tmp-{}-{sequence}", std::process::id()));
        match rustix::fs::openat(
            dir,
            candidate.as_os_str(),
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_raw_mode(mode),
        ) {
            Ok(fd) => break (candidate, fd),
            Err(Errno::EXIST) if attempts < 16 => attempts += 1,
            Err(error) => return Err(open_error(error)),
        }
    };
    let mut file = std::fs::File::from(fd);
    let outcome = file
        .write_all(bytes)
        .and_then(|()| file.sync_all())
        .and_then(|()| {
            rustix::fs::renameat(dir, temp_name.as_os_str(), dir, name)
                .map_err(std::io::Error::from)
        });
    if let Err(error) = outcome {
        let _ = rustix::fs::unlinkat(dir, temp_name.as_os_str(), AtFlags::empty());
        return Err(ForgeError::io("write planned file", error));
    }
    // F32: after the rename, the entry must be the regular file just written.
    // This detects substitution of the entry; it does not prevent it. A
    // mismatch is reported as an error, never as success.
    let substituted = || {
        ForgeError::UnsafePath(format!(
            "destination {} was substituted during materialization",
            destination.display()
        ))
    };
    let written = rustix::fs::fstat(&file).map_err(|error| {
        ForgeError::io(
            format!("stat planned file {}", destination.display()),
            std::io::Error::from(error),
        )
    })?;
    let entry =
        rustix::fs::statat(dir, name, AtFlags::SYMLINK_NOFOLLOW).map_err(|_| substituted())?;
    if entry.st_dev != written.st_dev
        || entry.st_ino != written.st_ino
        || (entry.st_mode & libc::S_IFMT) != libc::S_IFREG
    {
        return Err(substituted());
    }
    let _ = rustix::fs::fsync(dir);
    Ok(())
}

/// Windows: `safe_write` without an authorized root. The destination's parent
/// is pinned (see `sea_forge_os::PinnedDir`), and the entry is replaced
/// atomically through that pinned directory. No containment check runs,
/// because no root is given.
#[cfg(windows)]
pub fn safe_write(destination: &Path, bytes: &[u8]) -> Result<(), ForgeError> {
    let parent = destination.parent().ok_or_else(|| {
        ForgeError::UnsafePath(format!(
            "destination has no parent: {}",
            destination.display()
        ))
    })?;
    let name = destination.file_name().ok_or_else(|| {
        ForgeError::UnsafePath(format!(
            "destination has no file name: {}",
            destination.display()
        ))
    })?;
    let dir = pin_path(parent)?;
    write_entry_windows(&dir, destination, name, bytes)
}

/// Writes `bytes` to `destination`, which must resolve inside `root`.
///
/// Windows: the destination's parent chain is walked from the canonical root
/// with every directory pinned (no `FILE_SHARE_DELETE`), reparse points
/// refused, and each resolved path checked to stay inside `root`. The entry
/// is then replaced atomically inside the pinned parent. A hard link to the
/// old entry is never written. A destination that is a symlink or junction,
/// or a directory, is refused before anything is written.
#[cfg(windows)]
pub fn safe_write_within(root: &Path, destination: &Path, bytes: &[u8]) -> Result<(), ForgeError> {
    let canonical_root = root
        .canonicalize()
        .map_err(|e| ForgeError::io("canonicalize workspace", e))?;
    let relative = destination
        .strip_prefix(&canonical_root)
        .or_else(|_| destination.strip_prefix(root))
        .map_err(|_| {
            ForgeError::UnsafePath(format!(
                "destination {} resolves outside the workspace",
                destination.display()
            ))
        })?;
    let names = normal_components(relative, destination)?;
    let (file_name, parents) = names.split_last().ok_or_else(|| {
        ForgeError::UnsafePath(format!(
            "destination is not a file: {}",
            destination.display()
        ))
    })?;
    let dir = pinned_walk(&canonical_root, parents, false)?;
    write_entry_windows(&dir, destination, file_name, bytes)
}

/// Unix: writes `bytes` to `destination`, which must resolve inside `root`.
///
/// `root` is canonicalized, and the destination must sit under it (the raw
/// `root` prefix is also accepted). Only plain path components are allowed.
/// The root is opened with `O_DIRECTORY`, and each parent component is opened
/// relative to the previous directory with `O_DIRECTORY|O_NOFOLLOW`, so a
/// symlink ancestor fails as `UnsafePath`. The entry is then replaced through
/// the final directory descriptor (`write_entry_unix`).
///
/// Residual: a directory that is already open can still be renamed elsewhere
/// by a concurrent writer. The descriptor stays bound to the inode, so the
/// write lands in the moved directory. That relocation is not excluded here.
#[cfg(unix)]
pub fn safe_write_within(root: &Path, destination: &Path, bytes: &[u8]) -> Result<(), ForgeError> {
    use rustix::fs::{Mode, OFlags};
    use rustix::io::Errno;

    let canonical_root = root
        .canonicalize()
        .map_err(|e| ForgeError::io("canonicalize workspace", e))?;
    let relative = destination
        .strip_prefix(&canonical_root)
        .or_else(|_| destination.strip_prefix(root))
        .map_err(|_| {
            ForgeError::UnsafePath(format!(
                "destination {} resolves outside the workspace",
                destination.display()
            ))
        })?;
    let names = normal_components(relative, destination)?;
    let (file_name, parents) = names.split_last().ok_or_else(|| {
        ForgeError::UnsafePath(format!(
            "destination is not a file: {}",
            destination.display()
        ))
    })?;
    let mut dir = rustix::fs::open(
        canonical_root.as_path(),
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| ForgeError::io("open workspace root", std::io::Error::from(error)))?;
    for parent in parents {
        dir = rustix::fs::openat(
            &dir,
            *parent,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| match error {
            Errno::LOOP | Errno::NOTDIR => ForgeError::UnsafePath(format!(
                "parent is a symlink or not a directory: {}",
                destination.display()
            )),
            other => ForgeError::io(
                format!("open parent of {}", destination.display()),
                std::io::Error::from(other),
            ),
        })?;
    }
    write_entry_unix(&dir, destination, file_name, bytes)
}

/// Neither Unix nor Windows: no containment support is available here.
#[cfg(not(any(unix, windows)))]
pub fn safe_write_within(_root: &Path, destination: &Path, bytes: &[u8]) -> Result<(), ForgeError> {
    safe_write(destination, bytes)
}

#[cfg(any(unix, windows))]
fn normal_components<'a>(
    relative: &'a Path,
    destination: &Path,
) -> Result<Vec<&'a std::ffi::OsStr>, ForgeError> {
    relative
        .components()
        .map(|component| match component {
            Component::Normal(name) => Ok(name),
            _ => Err(ForgeError::UnsafePath(format!(
                "unsafe destination {}",
                destination.display()
            ))),
        })
        .collect()
}

#[cfg(windows)]
fn pin_error(error: sea_forge_os::PinError, path: &Path) -> ForgeError {
    use sea_forge_os::PinError;
    match error {
        PinError::ReparsePoint(_) => ForgeError::UnsafePath(format!(
            "parent is a symlink or junction: {}",
            path.display()
        )),
        PinError::NotDirectory(_) => {
            ForgeError::UnsafePath(format!("parent is not a directory: {}", path.display()))
        }
        PinError::Missing(_) => {
            ForgeError::UnsafePath(format!("parent does not exist: {}", path.display()))
        }
        PinError::Escaped(_) => ForgeError::UnsafePath(format!(
            "parent resolves outside the workspace: {}",
            path.display()
        )),
        PinError::Io(error) => ForgeError::io(format!("pin directory {}", path.display()), error),
    }
}

/// Opens `path` as a pinned directory with no containment check.
#[cfg(windows)]
fn pin_path(path: &Path) -> Result<sea_forge_os::PinnedDir, ForgeError> {
    sea_forge_os::PinnedDir::open(path).map_err(|error| pin_error(error, path))
}

/// Pins `canonical_root`, then each of `names` below it, one component at a
/// time. Each pinned parent is checked to stay inside `canonical_root`
/// before any child is opened. With `create`, a missing component is created
/// under its pinned parent. Nothing is created outside the pinned chain.
#[cfg(windows)]
fn pinned_walk(
    canonical_root: &Path,
    names: &[&std::ffi::OsStr],
    create: bool,
) -> Result<sea_forge_os::PinnedDir, ForgeError> {
    let mut dir = pin_path(canonical_root)?;
    if !dir.path().starts_with(canonical_root) {
        return Err(ForgeError::UnsafePath(format!(
            "workspace root resolves outside the workspace: {}",
            canonical_root.display()
        )));
    }
    for name in names {
        let child_path = dir.path().join(name);
        dir = dir
            .child_dir(name, create)
            .map_err(|error| pin_error(error, &child_path))?;
        if !dir.path().starts_with(canonical_root) {
            return Err(ForgeError::UnsafePath(format!(
                "parent resolves outside the workspace: {}",
                child_path.display()
            )));
        }
    }
    Ok(dir)
}

/// Replaces the entry `name` inside the pinned `dir`, after refusing a
/// destination that is a reparse point or a directory.
#[cfg(windows)]
fn write_entry_windows(
    dir: &sea_forge_os::PinnedDir,
    destination: &Path,
    name: &std::ffi::OsStr,
    bytes: &[u8],
) -> Result<(), ForgeError> {
    use sea_forge_os::EntryState;
    match dir.entry_state(name) {
        Ok(EntryState::Missing | EntryState::File) => {}
        Ok(EntryState::ReparsePoint) => {
            return Err(ForgeError::UnsafePath(format!(
                "destination {} is a reparse point (symlink or junction)",
                destination.display()
            )))
        }
        Ok(EntryState::Directory) => {
            return Err(ForgeError::UnsafePath(format!(
                "destination {} is a directory",
                destination.display()
            )))
        }
        Err(error) => return Err(pin_error(error, destination)),
    }
    dir.write_replacing(name, bytes)
        .map_err(|e| ForgeError::io(format!("write planned file {}", destination.display()), e))
}

#[cfg(not(any(unix, windows)))]
pub fn safe_write(destination: &Path, bytes: &[u8]) -> Result<(), ForgeError> {
    fs::write(destination, bytes).map_err(|e| ForgeError::io("write planned file", e))
}

pub fn materialize(
    grant: sea_forge_authority::ActionGrant,
    root: &Path,
    run_id: &str,
    plan_item_id: &str,
    operation: &sea_forge_core::types::Operation,
    compensating_controls: &[String],
) -> Result<(), ForgeError> {
    grant.authorize_with_controls(
        &sea_forge_core::types::AuthorityAction::from(operation),
        run_id,
        plan_item_id,
        root,
        compensating_controls,
    )?;
    if let sea_forge_core::types::Operation::WriteFile { path, content_hint } = operation {
        let destination = safe_join(root, path)?;
        safe_write_within(root, &destination, content_hint.as_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::time::{SystemTime, UNIX_EPOCH};
    #[test]
    fn lexical_escape_is_rejected() {
        assert!(validate_relative_path("../x").is_err());
        assert!(validate_relative_path("/x").is_err());
        assert!(validate_relative_path("bad path").is_err());
        assert!(validate_relative_path("ok/model.sea").is_ok());
    }

    #[test]
    fn ambiguous_spellings_are_rejected() {
        // F-01 regression: `//`, `.` segments, and trailing `/` must not be
        // materialized as aliases of a canonical path.
        for bad in [
            "src//gen//model.rs",
            "src/./gen/model.rs",
            "src/gen/",
            "a/./b",
        ] {
            assert!(
                validate_relative_path(bad).is_err(),
                "{bad:?} must be rejected"
            );
        }
        // A bare `.` remains valid (workspace-root cwd marker).
        assert!(validate_relative_path(".").is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_parent_escape_is_rejected() {
        use std::os::unix::fs::symlink;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let parent = std::env::temp_dir().join(format!("sea-forge-safe-join-{nonce}"));
        let root = parent.join("workspace");
        let outside = parent.join("outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        symlink(&outside, root.join("linked")).unwrap();
        assert!(safe_join(&root, "linked/escape.txt").is_err());
        assert!(!outside.join("escape.txt").exists());
        symlink(outside.join("file.txt"), root.join("file.txt")).unwrap();
        assert!(safe_join(&root, "file.txt").is_err());
        fs::remove_dir_all(parent).unwrap();
    }

    /// Creates `<tmp>/sea-forge-<tag>-<nonce>/{workspace, outside.txt}`.
    #[cfg(unix)]
    fn unix_write_fixture(tag: &str) -> (PathBuf, PathBuf, PathBuf) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let parent = std::env::temp_dir().join(format!("sea-forge-{tag}-{nonce}"));
        let root = parent.join("workspace");
        let outside = parent.join("outside.txt");
        fs::create_dir_all(&root).unwrap();
        fs::write(&outside, b"outside original").unwrap();
        (parent, root, outside)
    }

    /// F04 regression: a pre-planted hard link to an outside file must keep
    /// its bytes. The old `O_TRUNC` open rewrote the shared inode.
    #[cfg(unix)]
    #[test]
    fn hard_linked_destination_keeps_outside_bytes() {
        let (parent, root, outside) = unix_write_fixture("hardlink");
        let destination = root.join("planned.txt");
        fs::hard_link(&outside, &destination).unwrap();

        safe_write(&destination, b"overwritten").unwrap();
        assert_eq!(fs::read(&outside).unwrap(), b"outside original");
        assert_eq!(fs::read(&destination).unwrap(), b"overwritten");

        safe_write_within(&root, &destination, b"again").unwrap();
        assert_eq!(fs::read(&outside).unwrap(), b"outside original");
        assert_eq!(fs::read(&destination).unwrap(), b"again");
        fs::remove_dir_all(parent).unwrap();
    }

    /// A symlink at the destination is refused and its target is untouched.
    #[cfg(unix)]
    #[test]
    fn symlink_destination_is_refused_and_its_target_untouched() {
        use std::os::unix::fs::symlink;
        let (parent, root, outside) = unix_write_fixture("symlink-dest");
        let destination = root.join("planned.txt");
        symlink(&outside, &destination).unwrap();

        let result = safe_write(&destination, b"captured");
        assert!(matches!(result, Err(ForgeError::UnsafePath(_))));
        assert_eq!(fs::read(&outside).unwrap(), b"outside original");
        fs::remove_dir_all(parent).unwrap();
    }

    /// F31 regression: a parent swapped to a symlink after the caller planned
    /// `a/b/file.txt` must not let the write reach the outside directory.
    #[cfg(unix)]
    #[test]
    fn ancestor_symlink_substitution_is_refused() {
        use std::os::unix::fs::symlink;
        let (parent, root, _) = unix_write_fixture("ancestor-swap");
        let outside = parent.join("outside");
        fs::create_dir_all(root.join("a/b")).unwrap();
        fs::create_dir_all(outside.join("b")).unwrap();
        let destination = root.join("a/b/file.txt");

        fs::rename(root.join("a"), root.join("a_aside")).unwrap();
        symlink(&outside, root.join("a")).unwrap();

        let result = safe_write_within(&root, &destination, b"x");
        assert!(matches!(result, Err(ForgeError::UnsafePath(_))));
        assert_eq!(fs::read_dir(outside.join("b")).unwrap().count(), 0);
        assert!(!root.join("a_aside/b/file.txt").exists());
        fs::remove_dir_all(parent).unwrap();
    }

    /// A plain nested write inside the workspace still succeeds.
    #[cfg(unix)]
    #[test]
    fn nested_write_inside_root_succeeds() {
        let (parent, root, _) = unix_write_fixture("nested-write");
        fs::create_dir_all(root.join("a/b")).unwrap();
        let destination = root.join("a/b/file.txt");

        safe_write_within(&root, &destination, b"nested bytes").unwrap();
        assert_eq!(fs::read(&destination).unwrap(), b"nested bytes");
        fs::remove_dir_all(parent).unwrap();
    }

    /// A destination outside the root is refused and nothing is written.
    #[cfg(unix)]
    #[test]
    fn destination_outside_root_is_refused() {
        let (parent, root, outside) = unix_write_fixture("outside-root");
        let escaped = root.join("../outside.txt");

        assert!(matches!(
            safe_write_within(&root, &outside, b"captured"),
            Err(ForgeError::UnsafePath(_))
        ));
        assert!(matches!(
            safe_write_within(&root, &escaped, b"captured"),
            Err(ForgeError::UnsafePath(_))
        ));
        assert_eq!(fs::read(&outside).unwrap(), b"outside original");
        fs::remove_dir_all(parent).unwrap();
    }
}
