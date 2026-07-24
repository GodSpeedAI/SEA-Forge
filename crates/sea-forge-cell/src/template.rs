//! Template adoption — imported templates are inert until adopted (§10.6).

use sea_forge_core::errors::ForgeError;
use std::fs;
use std::path::{Path, PathBuf};

use crate::bundle::parse_template_ref;

/// Adopt an imported template by copying it into the active templates dir.
/// After adoption, `sea_forge_planner::templates::load_pinned` finds it.
/// Authority approval is evaluated at the CLI layer before calling this.
pub fn adopt(root: &Path, cell_id: &str, reference: &str) -> Result<PathBuf, ForgeError> {
    let (name, version) = parse_template_ref(reference)?;
    let src = crate::bundle::imported_template_path(root, cell_id, &name, &version);
    if !src.exists() {
        return Err(ForgeError::Input(format!(
            "imported template {reference} not found under cell {cell_id}"
        )));
    }
    let templates_dir = root.join(".sea-forge/templates");
    fs::create_dir_all(&templates_dir).map_err(|e| ForgeError::io("create templates dir", e))?;
    let dest = templates_dir.join(format!("{name}@{version}.yaml"));
    // ponytail: copy, not move — preserves the imported provenance trail under
    // imported/<cell_id>/ so a later auditor can see where it came from.
    fs::copy(&src, &dest).map_err(|e| ForgeError::io("adopt template", e))?;
    Ok(dest)
}
