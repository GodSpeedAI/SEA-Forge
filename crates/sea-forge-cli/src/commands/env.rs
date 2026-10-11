//! Environment commands (§7.6): list and show EnvironmentSpecs.

use sea_forge_core::errors::ForgeError;
use sea_forge_sandbox::environment::{load_pinned_environment, store_builtin_environment};
use std::path::Path;

pub fn list(root: &Path) -> Result<u8, ForgeError> {
    let envs_dir = root.join("environments");
    if !envs_dir.exists() {
        println!("(no environments)");
        return Ok(0);
    }
    let mut entries: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&envs_dir).map_err(|e| ForgeError::io("read envs dir", e))? {
        let entry = entry.map_err(|e| ForgeError::io("dir entry", e))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".yaml") && !name.starts_with('.') {
            entries.push(name.trim_end_matches(".yaml").into());
        }
    }
    entries.sort();
    for e in &entries {
        println!("{e}");
    }
    Ok(0)
}

pub fn show(root: &Path, reference: &str) -> Result<u8, ForgeError> {
    // Ensure the demo is available so show works on a fresh root.
    let _ = store_builtin_environment(root);
    let spec = load_pinned_environment(root, reference)?;
    println!(
        "{}",
        serde_yaml::to_string(&spec)
            .map_err(|e| ForgeError::Internal(format!("serialize: {e}")))?
    );
    Ok(0)
}
