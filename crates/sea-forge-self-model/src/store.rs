//! Self-model persistence and lifecycle (spec-adlc-thoth §3.1, §7.1, §8.5,
//! slice 1.0 + 1.5).
//!
//! Layout under `<root>/.sea-forge/self-model/`:
//!   manifest.json                     — installation manifest (release, current
//!                                        snapshot id, staleness); the lifecycle
//!                                        discriminator, never an immutable record.
//!   realization/release.json          — release realization (verified, not regenerated).
//!   realization/cell.json             — cell realization (rebuildable).
//!   snapshots/<snapshot_id>.json      — immutable SelfModelSnapshot records.
//!   projections/<kind>/...            — rebuildable KG/CALM/JSON projections.
//!
//! Init/upgrade is detected by the manifest's release_id/schema_version. Upgrade
//! never rewrites prior snapshots. Extension mutations mark the current snapshot
//! stale via the manifest and request a rebuild; immutable snapshot files are
//! never mutated. A failed rebuild preserves the prior verifiable snapshot.

use crate::projections::{project_self, verify_projection, SelfModelProjection};
use crate::{
    build_cell_realization, build_snapshot, bundled, load_composed, release_realization,
    verify_bundled, verify_snapshot, CellRealization, ExtensionState, ReleaseRealization,
    SelfModelSnapshot, SnapshotFreshness, ToolchainProbe, KERNEL_VERSION, RELEASE_ID,
};
use sea_forge_core::errors::ForgeError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const SCHEMA_VERSION: &str = "self_model.v1";

fn dir(root: &Path) -> PathBuf {
    root.join(".sea-forge").join("self-model")
}
fn snapshots_dir(root: &Path) -> PathBuf {
    dir(root).join("snapshots")
}
fn projections_dir(root: &Path) -> PathBuf {
    dir(root).join("projections")
}
fn realization_dir(root: &Path) -> PathBuf {
    dir(root).join("realization")
}
fn manifest_path(root: &Path) -> PathBuf {
    dir(root).join("manifest.json")
}
fn snapshot_path(root: &Path, snapshot_id: &str) -> PathBuf {
    snapshots_dir(root).join(format!("{snapshot_id}.json"))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelfModelManifest {
    pub schema_version: String,
    pub release_id: String,
    pub kernel_version: String,
    pub current_snapshot_id: Option<String>,
    #[serde(default)]
    pub stale: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stale_reason: Option<String>,
    pub initialized_at: String,
}

/// Inputs for a snapshot rebuild. The caller (CLI/server) supplies extension
/// state, environment contracts, and EVIDENCED probe results; the store never
/// executes probes itself (spec-adlc-thoth §7.2, slice 1.4).
#[derive(Clone, Debug, Default)]
pub struct RebuildInputs<'a> {
    pub cell_id: &'a str,
    pub active_extensions: Vec<ExtensionState>,
    pub environments_present: Vec<String>,
    pub probes: Vec<ToolchainProbe>,
    pub sandbox_classes_available: Vec<String>,
    pub created_at: &'a str,
    pub capability_projection_sha256: &'a str,
}

fn read_manifest(root: &Path) -> Result<Option<SelfModelManifest>, ForgeError> {
    let path = manifest_path(root);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(|e| ForgeError::io("read self-model manifest", e))?;
    let manifest: SelfModelManifest = serde_json::from_slice(&bytes)
        .map_err(|e| ForgeError::SelfModel(format!("self_model_error: bad manifest: {e}")))?;
    Ok(Some(manifest))
}

fn write_manifest(root: &Path, manifest: &SelfModelManifest) -> Result<(), ForgeError> {
    let path = manifest_path(root);
    fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))
        .map_err(|e| ForgeError::io("create self-model dir", e))?;
    let bytes = serde_json::to_vec_pretty(manifest)?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &bytes).map_err(|e| ForgeError::io("write manifest tmp", e))?;
    fs::rename(&tmp, &path).map_err(|e| ForgeError::io("rename manifest", e))?;
    Ok(())
}

fn write_immutable<T: Serialize>(path: &Path, value: &T) -> Result<(), ForgeError> {
    if path.exists() {
        // Immutable records are never overwritten; a same-id collision is integrity error.
        return Err(ForgeError::SelfModel(format!(
            "self_model_error: immutable record already exists: {}",
            path.display()
        )));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| ForgeError::io("create record parent", e))?;
    }
    let bytes = serde_json::to_vec_pretty(value)?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &bytes).map_err(|e| ForgeError::io("write record tmp", e))?;
    fs::rename(&tmp, path).map_err(|e| ForgeError::io("rename record", e))?;
    Ok(())
}

fn write_realization(
    root: &Path,
    release: &ReleaseRealization,
    cell: &CellRealization,
) -> Result<(), ForgeError> {
    let rdir = realization_dir(root);
    fs::create_dir_all(&rdir).map_err(|e| ForgeError::io("create realization dir", e))?;
    let rpath = rdir.join("release.json");
    let bytes = serde_json::to_vec_pretty(release)?;
    let tmp = rpath.with_extension("tmp");
    fs::write(&tmp, &bytes).map_err(|e| ForgeError::io("write release realization tmp", e))?;
    fs::rename(&tmp, &rpath).map_err(|e| ForgeError::io("rename release realization", e))?;
    let cpath = rdir.join("cell.json");
    let bytes = serde_json::to_vec_pretty(cell)?;
    let tmp = cpath.with_extension("tmp");
    fs::write(&tmp, &bytes).map_err(|e| ForgeError::io("write cell realization tmp", e))?;
    fs::rename(&tmp, &cpath).map_err(|e| ForgeError::io("rename cell realization", e))?;
    Ok(())
}

/// Read the persisted cell realization (rebuildable projection of registry+probes).
pub fn read_cell_realization(root: &Path) -> Result<Option<CellRealization>, ForgeError> {
    let path = realization_dir(root).join("cell.json");
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(|e| ForgeError::io("read cell realization", e))?;
    let cell: CellRealization = serde_json::from_slice(&bytes).map_err(|e| {
        ForgeError::SelfModel(format!("self_model_error: bad cell realization: {e}"))
    })?;
    Ok(Some(cell))
}

pub fn read_release_realization(root: &Path) -> Result<Option<ReleaseRealization>, ForgeError> {
    let path = realization_dir(root).join("release.json");
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(|e| ForgeError::io("read release realization", e))?;
    let r: ReleaseRealization = serde_json::from_slice(&bytes).map_err(|e| {
        ForgeError::SelfModel(format!("self_model_error: bad release realization: {e}"))
    })?;
    Ok(Some(r))
}

fn materialize_projections(
    root: &Path,
    projections: &[SelfModelProjection],
) -> Result<(), ForgeError> {
    let pdir = projections_dir(root);
    // Clear prior projection outputs so a wrong-hash pre-generated file is
    // regenerated, not trusted (spec §10.1, T9.3).
    if pdir.exists() {
        fs::remove_dir_all(&pdir).map_err(|e| ForgeError::io("clear projections dir", e))?;
    }
    fs::create_dir_all(&pdir).map_err(|e| ForgeError::io("create projections dir", e))?;
    for proj in projections {
        // Write the ProjectionRecord alongside its outputs.
        let kind_label = match proj.record.projection_kind {
            sea_forge_core::types::ProjectionKind::Kg => "kg",
            sea_forge_core::types::ProjectionKind::Calm => "calm",
            sea_forge_core::types::ProjectionKind::SelfModelSnapshot => "self_model_snapshot",
            _ => continue,
        };
        let kind_dir = pdir.join(kind_label);
        fs::create_dir_all(&kind_dir)
            .map_err(|e| ForgeError::io("create projection kind dir", e))?;
        for (path, content) in &proj.outputs {
            let full = pdir.join(path);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| ForgeError::io("create projection output parent", e))?;
            }
            fs::write(&full, content).map_err(|e| ForgeError::io("write projection output", e))?;
        }
        let record_path = kind_dir.join("projection-record.json");
        let bytes = serde_json::to_vec_pretty(&proj.record)?;
        fs::write(&record_path, bytes).map_err(|e| ForgeError::io("write projection record", e))?;
    }
    Ok(())
}

fn assemble(
    root: &Path,
    inputs: &RebuildInputs<'_>,
) -> Result<(SelfModelSnapshot, Vec<SelfModelProjection>, CellRealization), ForgeError> {
    let _ = root;
    let models = bundled();
    verify_bundled(&models)?;
    let composed = load_composed(&models)?;
    let release = release_realization();
    let cell = build_cell_realization(
        inputs.cell_id,
        inputs.active_extensions.clone(),
        inputs.environments_present.clone(),
        inputs.probes.clone(),
        inputs.sandbox_classes_available.clone(),
        inputs.created_at,
    );
    let snapshot = build_snapshot(
        &composed,
        &release,
        &cell,
        inputs.capability_projection_sha256,
        inputs.created_at,
    )?;
    let projections = project_self(&composed, &snapshot, inputs.created_at)?;
    Ok((snapshot, projections, cell))
}

/// Build a fresh snapshot and commit it: writes projections, realization, the
/// immutable snapshot record, and updates the manifest (current=new, stale=false).
/// On any failure the prior snapshot remains verifiable (failure atomicity).
pub fn rebuild(root: &Path, inputs: &RebuildInputs<'_>) -> Result<SelfModelSnapshot, ForgeError> {
    fs::create_dir_all(snapshots_dir(root))
        .map_err(|e| ForgeError::io("create snapshots dir", e))?;
    let (snapshot, projections, cell) = assemble(root, inputs)?;

    // Commit order: projections + realization first, then the immutable snapshot,
    // then the manifest pointer. A failure before the manifest write leaves the
    // prior current snapshot authoritative.
    materialize_projections(root, &projections)?;
    let release = release_realization();
    write_realization(root, &release, &cell)?;
    write_immutable(&snapshot_path(root, &snapshot.snapshot_id), &snapshot)?;

    let manifest = SelfModelManifest {
        schema_version: SCHEMA_VERSION.into(),
        release_id: RELEASE_ID.into(),
        kernel_version: KERNEL_VERSION.into(),
        current_snapshot_id: Some(snapshot.snapshot_id.clone()),
        stale: false,
        stale_reason: None,
        initialized_at: inputs.created_at.to_string(),
    };
    write_manifest(root, &manifest)?;
    Ok(snapshot)
}

/// First-root initialization: if no manifest exists, build the initial snapshot.
/// If a manifest exists for the same release, return the current snapshot. A
/// release-id change is an upgrade (builds a new snapshot; prior ones remain).
pub fn ensure_init(
    root: &Path,
    inputs: &RebuildInputs<'_>,
) -> Result<SelfModelSnapshot, ForgeError> {
    if let Some(manifest) = read_manifest(root)? {
        if manifest.release_id == RELEASE_ID
            && manifest.schema_version == SCHEMA_VERSION
            && manifest.current_snapshot_id.is_some()
        {
            return current_snapshot(root)?.ok_or_else(|| {
                ForgeError::SelfModel(
                    "self_model_error: manifest names a missing current snapshot".into(),
                )
            });
        }
        // Release/schema change ⇒ upgrade: rebuild (never rewrite prior snapshots).
    }
    rebuild(root, inputs)
}

/// The current (newest committed) snapshot, or None if uninitialized.
pub fn current_snapshot(root: &Path) -> Result<Option<SelfModelSnapshot>, ForgeError> {
    let Some(manifest) = read_manifest(root)? else {
        return Ok(None);
    };
    let Some(sid) = manifest.current_snapshot_id else {
        return Ok(None);
    };
    let path = snapshot_path(root, &sid);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(|e| ForgeError::io("read snapshot", e))?;
    let snap: SelfModelSnapshot = serde_json::from_slice(&bytes).map_err(|e| {
        ForgeError::SelfModel(format!("self_model_error: bad snapshot record: {e}"))
    })?;
    Ok(Some(snap))
}

/// Mark the current snapshot stale (e.g. after an extension mutation) without
/// mutating any immutable snapshot file. Records the reason in the manifest.
pub fn mark_current_stale(root: &Path, reason: &str) -> Result<(), ForgeError> {
    let Some(mut manifest) = read_manifest(root)? else {
        return Err(ForgeError::SelfModel(
            "self_model_error: cannot mark stale before init".into(),
        ));
    };
    manifest.stale = true;
    manifest.stale_reason = Some(reason.to_string());
    write_manifest(root, &manifest)?;
    Ok(())
}

/// Whether the current snapshot is marked stale.
pub fn is_stale(root: &Path) -> Result<bool, ForgeError> {
    Ok(read_manifest(root)?.map(|m| m.stale).unwrap_or(false))
}

/// Verify bundled models, validate the composed model, verify the current
/// snapshot's hash, and verify each persisted projection's rebuild_hash.
pub fn validate(root: &Path) -> Result<(), ForgeError> {
    let models = bundled();
    verify_bundled(&models)?;
    let _composed = load_composed(&models)?;
    if let Some(snap) = current_snapshot(root)? {
        verify_snapshot(&snap)?;
        if snap.freshness == SnapshotFreshness::Stale {
            // Stale is allowed but disclosed; not a validation failure.
        }
    }
    // Verify persisted projections.
    let pdir = projections_dir(root);
    if pdir.exists() {
        for entry in fs::read_dir(&pdir).map_err(|e| ForgeError::io("read projections dir", e))? {
            let entry = entry.map_err(|e| ForgeError::io("projection dir entry", e))?;
            let record_path = entry.path().join("projection-record.json");
            if record_path.exists() {
                let bytes = fs::read(&record_path)
                    .map_err(|e| ForgeError::io("read projection record", e))?;
                let record: sea_forge_core::types::ProjectionRecord =
                    serde_json::from_slice(&bytes).map_err(|e| {
                        ForgeError::SelfModel(format!(
                            "self_model_error: bad projection record: {e}"
                        ))
                    })?;
                verify_projection(&record)?;
            }
        }
    }
    Ok(())
}
