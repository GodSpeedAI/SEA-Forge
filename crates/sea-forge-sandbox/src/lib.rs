use sea_forge_core::{errors::ForgeError, types::*};
use std::{
    fmt, fs,
    path::{Component, Path, PathBuf},
};

pub mod environment;
pub mod jail;
pub mod local;

pub use environment::{
    demo_environment, load_pinned_environment, materialize_base, parse_env_ref,
    parse_evaluator_score, store_builtin_environment, BaseFile, EnvironmentSpec, Evaluator,
    Provides, ScoreFrom,
};
pub use jail::JailSandbox;
pub use local::LocalSandbox;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxClass {
    Local,
    Jail,
    Microvm,
}

impl fmt::Display for SandboxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SandboxClass::Local => write!(f, "local"),
            SandboxClass::Jail => write!(f, "jail"),
            SandboxClass::Microvm => write!(f, "microvm"),
        }
    }
}

impl std::str::FromStr for SandboxClass {
    type Err = ForgeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(SandboxClass::Local),
            "jail" => Ok(SandboxClass::Jail),
            "microvm" => Ok(SandboxClass::Microvm),
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
        SandboxClass::Microvm => Err(SandboxError::new(
            "unsupported_sandbox_class_error",
            "microvm backend is not available",
        )),
    }
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
/// F-25.i: write-through after `safe_join` without reopening the
/// check-then-use window. The destination is opened with `O_NOFOLLOW`, so a
/// symlink swapped in between validation and open fails with `ELOOP` (mapped
/// to a typed unsafe-path error) instead of writing through the link.
/// Residual parent-component races remain until `openat2(RESOLVE_BENEATH)`
/// stabilizes; per-component re-validation in `safe_join` keeps them narrow.
#[cfg(unix)]
pub fn safe_write(destination: &Path, bytes: &[u8]) -> Result<(), ForgeError> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    // 0o400000 is the Linux `O_NOFOLLOW` (glibc) / `O_NOFOLLOW` (FreeBSD) value.
    const O_NOFOLLOW: i32 = 0o400000;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .custom_flags(O_NOFOLLOW)
        .open(destination)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::Other && e.raw_os_error() == Some(40) {
                ForgeError::UnsafePath(format!(
                    "destination {} was swapped to a symlink during materialization",
                    destination.display()
                ))
            } else {
                ForgeError::io(format!("open planned file {}", destination.display()), e)
            }
        })?;
    file.write_all(bytes)
        .map_err(|e| ForgeError::io("write planned file", e))?;
    Ok(())
}

#[cfg(not(unix))]
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
        safe_write(&destination, content_hint.as_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
