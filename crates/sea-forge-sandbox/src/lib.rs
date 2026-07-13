use sea_forge_core::{errors::ForgeError, types::*};
use std::{
    fmt, fs,
    path::{Component, Path, PathBuf},
};

pub mod jail;
pub mod local;

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

#[derive(Clone, Debug)]
pub struct SandboxSpec {
    pub workspace_root: PathBuf,
    pub artifacts_root: PathBuf,
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
    let value = Path::new(path);
    if value.is_absolute()
        || path.is_empty()
        || !path
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._/-".contains(c))
        || value.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(ForgeError::UnsafePath(format!(
            "unsafe workspace-relative path: {path}"
        )));
    }
    Ok(())
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
        fs::write(destination, content_hint)
            .map_err(|e| ForgeError::io("write planned file", e))?;
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
