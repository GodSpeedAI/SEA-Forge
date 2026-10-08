use chrono::Utc;
use sea_forge_core::errors::ForgeError;
use sea_forge_sandbox as sandbox;
use std::path::Path;

pub fn rebuild(root: &Path) -> Result<(), ForgeError> {
    let items_path = sandbox::safe_join(root, "memory/items.jsonl")?;
    let index_path = sandbox::safe_join(root, "memory/index.sqlite")?;
    let now = Utc::now().to_rfc3339();
    sea_forge_capability::memory::rebuild_index(&items_path, &index_path, &now)?;
    println!("memory index rebuilt");
    Ok(())
}
