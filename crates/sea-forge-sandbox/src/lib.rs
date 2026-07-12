use sea_forge_core::errors::ForgeError;
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

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
    root: &Path,
    operation: &sea_forge_core::types::Operation,
) -> Result<(), ForgeError> {
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
