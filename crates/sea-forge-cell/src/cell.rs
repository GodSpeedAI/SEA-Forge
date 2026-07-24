//! Cell identity: load or generate `.sea-forge/cell.json` (spec-full §7.4).

use chrono::Utc;
use sea_forge_core::{errors::ForgeError, ids};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct CellRecord {
    pub cell_id: String,
    pub created_at: String,
}

const SCHEMA_VERSION: &str = "cell.v1";

/// Load the cell id from `<root>/.sea-forge/cell.json`, returning `None` if it
/// does not exist (legacy root). Stamps a fresh `cell_<8hex>` on first call.
pub fn ensure(root: &Path) -> Result<String, ForgeError> {
    let path = root.join(".sea-forge/cell.json");
    if let Some(record) = read(root)? {
        return Ok(record.cell_id);
    }
    let cell_id = ids::cell_id()?;
    let record = CellRecord {
        cell_id: cell_id.clone(),
        created_at: Utc::now().to_rfc3339(),
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| ForgeError::io("create .sea-forge directory", e))?;
    }
    let bytes = serde_json::to_vec_pretty(&record)?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .map_err(|e| ForgeError::io(format!("create {}", path.display()), e))?;
    use std::io::Write;
    file.write_all(&bytes)
        .and_then(|_| file.flush())
        .map_err(|e| ForgeError::io("write cell.json", e))?;
    Ok(cell_id)
}

/// Read the existing `cell.json`. Returns `Ok(None)` if absent (legacy).
pub fn read(root: &Path) -> Result<Option<CellRecord>, ForgeError> {
    let path = root.join(".sea-forge/cell.json");
    let mut file = match OpenOptions::new().read(true).open(&path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(ForgeError::io(format!("read {}", path.display()), e));
        }
    };
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|e| ForgeError::io("read cell.json", e))?;
    let record: CellRecord = serde_json::from_str(&buf)?;
    if !record.cell_id.starts_with("cell_") || record.cell_id.len() != 13 {
        return Err(ForgeError::Config {
            class: "cell_id_error",
            path,
            message: "cell.json cell_id must be `cell_<8 hex>`".into(),
        });
    }
    Ok(Some(record))
}

pub(crate) fn schema_version() -> &'static str {
    SCHEMA_VERSION
}

// Re-export so callers that read raw bytes can validate the version field.
pub fn is_known_schema(version: &str) -> bool {
    version == SCHEMA_VERSION
}
