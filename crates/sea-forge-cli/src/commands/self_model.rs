//! Self-model commands (spec-adlc-thoth §11.1): validate, rebuild, show.
//!
//! All three go through `sea_forge_self_model::store`. `rebuild [--probe]` is
//! the lifecycle entry: it ensures init, then rebuilds a snapshot. Probe
//! *execution* (authority-checked sandbox runs of environment-contract commands)
//! is the CLI/server's job; here it is a no-op stub until the probe runner is
//! wired (the store consumes evidenced probe results, never running commands).

use sea_forge_core::errors::ForgeError;
use sea_forge_self_model::{
    store,
    store::{read_cell_realization, RebuildInputs},
};
use std::path::Path;

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn default_cell_id(root: &Path) -> Result<String, ForgeError> {
    // Reuse the cell id if a cell.json realization exists; else a stable local default.
    if let Some(cell) = read_cell_realization(root)? {
        return Ok(cell.cell_id);
    }
    Ok("cell_local".into())
}

/// `sea-forge self-model validate` — verify bundled models, validate the
/// composed model, verify the current snapshot + persisted projections. Exit 0/1.
pub fn validate(root: &Path) -> Result<u8, ForgeError> {
    store::validate(root)?;
    println!("self-model: valid (bundled models verified; snapshot + projections verified)");
    Ok(0)
}

/// `sea-forge self-model rebuild [--probe]` — init-or-upgrade, then build a new
/// snapshot from the release realization + cell realization. `--probe` is
/// accepted but probe execution is deferred to the sandbox layer (no-op here).
pub fn rebuild(root: &Path, _probe: bool, capability_hash: &str) -> Result<u8, ForgeError> {
    let cell_id = default_cell_id(root)?;
    let inputs = RebuildInputs {
        cell_id: &cell_id,
        active_extensions: vec![],
        environments_present: vec![],
        probes: vec![],
        sandbox_classes_available: vec!["local".into()],
        created_at: &now_iso(),
        capability_projection_sha256: capability_hash,
    };
    let snapshot = store::rebuild(root, &inputs)?;
    println!("snapshot_id={}", snapshot.snapshot_id);
    println!("release_id={}", snapshot.release_id);
    println!("snapshot_hash={}", snapshot.snapshot_hash);
    println!("freshness={:?}", snapshot.freshness);
    Ok(0)
}

/// `sea-forge self-model show [--json]` — composed-view summary from the newest
/// snapshot; `--json` emits the full snapshot record.
pub fn show(root: &Path, json: bool) -> Result<u8, ForgeError> {
    let Some(snapshot) = store::current_snapshot(root)? else {
        if json {
            println!("null");
        } else {
            println!("(no self-model snapshot; run `sea-forge self-model rebuild`)");
        }
        return Ok(0);
    };
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&snapshot)
                .map_err(|e| ForgeError::Serialization(e.to_string()))?
        );
        return Ok(0);
    }
    println!("snapshot_id={}", snapshot.snapshot_id);
    println!("release_id={}", snapshot.release_id);
    println!("snapshot_hash={}", snapshot.snapshot_hash);
    println!("freshness={:?}", snapshot.freshness);
    println!(
        "system_model_sha256={}",
        snapshot.system_model_ref.semantic_model_sha256
    );
    println!(
        "release_realization_sha256={}",
        snapshot.release_realization_sha256
    );
    println!(
        "cell_realization_sha256={}",
        snapshot.cell_realization_sha256
    );
    println!(
        "capability_projection_sha256={}",
        snapshot.capability_projection_sha256
    );
    if let Some(cell) = read_cell_realization(root)? {
        println!("degraded_components={:?}", cell.degraded_components);
        println!(
            "sandbox_classes_available={:?}",
            cell.sandbox_classes_available
        );
    }
    Ok(0)
}
