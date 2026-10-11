//! Self-model commands (spec-adlc-thoth §11.1): validate, rebuild, show.
//!
//! All three go through `sea_forge_self_model::store`. `rebuild [--probe]` is
//! the lifecycle entry: it ensures init, then rebuilds a snapshot. Probe
//! *execution* (authority-checked sandbox runs of environment-contract commands)
//! is the CLI/server's job; here it is a no-op stub until the probe runner is
//! wired (the store consumes evidenced probe results, never running commands).

use sea_forge_core::errors::ForgeError;
use sea_forge_extension::{ExtensionRegistry, ExtensionStatus};
use sea_forge_self_model::{
    store,
    store::{read_cell_realization, RebuildInputs},
    ExtensionState,
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

/// Real active-extension state from the installation's own extension registry
/// (`<root>/extensions/registry.json`, F-12 state root), replacing the
/// caller-supplied empty vec (Task 11 audit remediation). Callers use
/// [`ExtensionRegistry::exists`] to disclose an *absent* registry as a stale
/// cell instead of silently asserting zero extensions (SUP-07).
fn active_extensions_from_registry(root: &Path) -> Result<Vec<ExtensionState>, ForgeError> {
    let registry = ExtensionRegistry::load(root)?;
    Ok(registry
        .extensions
        .into_iter()
        .map(|entry| ExtensionState {
            descriptor_ref: format!("{}@{}", entry.extension_id, entry.version),
            status: match entry.status {
                ExtensionStatus::Active => "active",
                ExtensionStatus::Disabled => "disabled",
                ExtensionStatus::Quarantined => "quarantined",
                ExtensionStatus::Superseded => "superseded",
            }
            .to_string(),
        })
        .collect())
}

/// Real sandbox classes this host can currently construct, replacing the
/// caller-supplied hardcoded `["local"]` (Task 11 audit remediation). `local`
/// is always available; other classes are probed by attempting construction.
fn real_sandbox_classes_available() -> Vec<String> {
    use sea_forge_sandbox::{select_sandbox, SandboxClass};
    [
        SandboxClass::Local,
        SandboxClass::Jail,
        SandboxClass::Microvm,
    ]
    .into_iter()
    .filter(|class| select_sandbox(*class).is_ok())
    .map(|class| class.to_string())
    .collect()
}

/// Real, non-placeholder commitment over the demonstrated-capability surface
/// (`.sea-forge/capabilities.jsonl`), replacing the fixed
/// `sha256:capability-projection` CLI default literal (Task 11 audit
/// remediation). An absent or empty file hashes to a well-defined, still-real
/// "no capability data yet" digest rather than a fabricated placeholder.
fn real_capability_projection_sha256(root: &Path) -> Result<String, ForgeError> {
    let path = root.join("capabilities.jsonl");
    let envelopes = if path.exists() {
        sea_forge_capability::load_envelopes(&path)?
    } else {
        vec![]
    };
    sea_forge_self_model::canonical_sha256(&envelopes)
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
/// Inputs come from verified installation state (extension registry, sandbox
/// availability, demonstrated-capability envelopes) rather than caller-supplied
/// placeholders (Task 11 audit remediation).
pub fn rebuild(root: &Path, _probe: bool, actor_id: &str) -> Result<u8, ForgeError> {
    let cell_id = default_cell_id(root)?;
    let active_extensions = active_extensions_from_registry(root)?;
    let capability_hash = real_capability_projection_sha256(root)?;
    let sandbox_classes_available = real_sandbox_classes_available();
    let created_at = now_iso();
    let inputs = RebuildInputs {
        cell_id: &cell_id,
        active_extensions,
        environments_present: vec![],
        probes: vec![],
        sandbox_classes_available,
        created_at: &created_at,
        capability_projection_sha256: &capability_hash,
        actor_id,
    };
    let snapshot = store::rebuild(root, &inputs)?;
    // SUP-07: a rebuild over an *absent* registry attested nothing about the
    // cell's extensions. The snapshot still lands (its model content is real)
    // but is disclosed stale — degraded, never a clean "zero extensions"
    // cell — until a rebuild runs against an existing registry.
    if !ExtensionRegistry::exists(root) {
        store::mark_current_stale(root, "extension_registry_absent")?;
        println!("snapshot_id={}", snapshot.snapshot_id);
        println!("release_id={}", snapshot.release_id);
        println!("snapshot_hash={}", snapshot.snapshot_hash);
        println!("freshness=Stale (extension_registry_absent)");
        return Ok(0);
    }
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
