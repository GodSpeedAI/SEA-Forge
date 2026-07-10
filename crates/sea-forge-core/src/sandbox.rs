use crate::errors::ForgeError;
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
    if rel == "." {
        return root
            .canonicalize()
            .map_err(|e| ForgeError::io("canonicalize workspace", e));
    }
    let candidate = root.join(rel);
    let parent = candidate
        .parent()
        .ok_or_else(|| ForgeError::UnsafePath(rel.into()))?;
    fs::create_dir_all(parent).map_err(|e| ForgeError::io("create workspace parent", e))?;
    let canonical_root = root
        .canonicalize()
        .map_err(|e| ForgeError::io("canonicalize workspace", e))?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|e| ForgeError::io("canonicalize target parent", e))?;
    if !canonical_parent.starts_with(canonical_root) {
        return Err(ForgeError::UnsafePath(format!(
            "path escapes workspace: {rel}"
        )));
    }
    Ok(candidate)
}
pub fn materialize(root: &Path, operation: &crate::types::Operation) -> Result<(), ForgeError> {
    if let crate::types::Operation::WriteFile { path, content_hint } = operation {
        let destination = safe_join(root, path)?;
        fs::write(destination, content_hint)
            .map_err(|e| ForgeError::io("write planned file", e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lexical_escape_is_rejected() {
        assert!(validate_relative_path("../x").is_err());
        assert!(validate_relative_path("/x").is_err());
        assert!(validate_relative_path("ok/model.sea").is_ok());
    }
}
