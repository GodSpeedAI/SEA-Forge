//! Federation bundle export/import (spec-full §7.4, §14.8 atomic reject).

use crate::cell::{schema_version, CellRecord};
use sea_forge_core::{
    errors::ForgeError,
    ids,
    types::{BundleFile, BundleManifest},
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Files in a run directory that are evidence-relevant and bundled.
/// `workspace/` is transient scratch and is excluded.
const RUN_EVIDENCE_FILES: &[&str] = &[
    "plan.json",
    "trace.jsonl",
    "evidence.jsonl",
    "authority.json",
    "settlement.json",
    "semantic-envelope.json",
];

/// Export selected runs (and optionally templates) to a tar bundle at `out_path`.
/// Returns the manifest. Caller is responsible for authority approval.
pub fn export(
    root: &Path,
    run_ids: &[String],
    templates: &[String],
    out_path: &Path,
) -> Result<BundleManifest, ForgeError> {
    let cell = crate::cell::read(root)?;
    let cell_id = match cell {
        Some(CellRecord { cell_id, .. }) => cell_id,
        None => crate::cell::ensure(root)?,
    };
    let bundle_id = ids::bundle_id()?;
    let created_at = chrono::Utc::now().to_rfc3339();
    let mut files = Vec::new();
    let mut tar_buf: Vec<u8> = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_buf);
        builder.mode(tar::HeaderMode::Deterministic);
        for run_id in run_ids {
            for evidence_file in RUN_EVIDENCE_FILES {
                let src = root
                    .join(".sea-forge/runs")
                    .join(run_id)
                    .join(evidence_file);
                let arc = format!("runs/{run_id}/{evidence_file}");
                if src.exists() {
                    push_file(&mut builder, &src, &arc, &mut files)?;
                }
            }
            // Recurse artifacts/ (real evidence). Skip workspace/ (scratch).
            let artifacts_dir = root.join(".sea-forge/runs").join(run_id).join("artifacts");
            if artifacts_dir.is_dir() {
                push_tree(
                    &mut builder,
                    &artifacts_dir,
                    &format!("runs/{run_id}/artifacts"),
                    &mut files,
                )?;
            }
        }
        for reference in templates {
            let (name, version) = parse_template_ref(reference)?;
            let src = root
                .join(".sea-forge/templates")
                .join(format!("{name}@{version}.yaml"));
            if !src.exists() {
                return Err(ForgeError::Input(format!("template {reference} not found")));
            }
            let arc = format!("templates/{name}@{version}.yaml");
            push_file(&mut builder, &src, &arc, &mut files)?;
        }
        builder
            .finish()
            .map_err(|e| ForgeError::io("finalize tar bundle", e))?;
    }
    let manifest = BundleManifest {
        schema_version: schema_version().into(),
        bundle_id: bundle_id.clone(),
        cell_id: cell_id.clone(),
        created_at,
        run_ids: run_ids.to_vec(),
        templates: templates.to_vec(),
        files,
    };
    // Prepend manifest.json into a final tar by rebuilding.
    let mut final_buf: Vec<u8> = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut final_buf);
        builder.mode(tar::HeaderMode::Deterministic);
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
        append_bytes(&mut builder, "manifest.json", &manifest_bytes)?;
        // Append original archive contents.
        let mut input = tar::Archive::new(&tar_buf[..]);
        for entry in input
            .entries()
            .map_err(|e| ForgeError::io("read intermediate archive", e))?
        {
            let mut entry = entry.map_err(|e| ForgeError::io("read archive entry", e))?;
            let path = entry
                .path()
                .map_err(|e| ForgeError::io("entry path", e))?
                .into_owned();
            let mut data = Vec::new();
            entry
                .read_to_end(&mut data)
                .map_err(|e| ForgeError::io("read entry bytes", e))?;
            append_bytes(&mut builder, &path.to_string_lossy(), &data)?;
        }
        builder
            .finish()
            .map_err(|e| ForgeError::io("finalize final tar", e))?;
    }
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(|e| ForgeError::io("create bundle parent", e))?;
    }
    let mut out = File::create(out_path)
        .map_err(|e| ForgeError::io(format!("create {}", out_path.display()), e))?;
    out.write_all(&final_buf)
        .map_err(|e| ForgeError::io("write bundle", e))?;
    Ok(manifest)
}

/// Import a bundle into `<root>/.sea-forge/imported/<exporter_cell_id>/`.
/// Atomic per §14.8: any hash mismatch or structural error rejects the whole
/// bundle — no partial state remains. Imported runs NEVER merge into local
/// `capabilities.jsonl` (§7.4).
pub fn import(root: &Path, bundle: &Path) -> Result<BundleManifest, ForgeError> {
    let bytes =
        fs::read(bundle).map_err(|e| ForgeError::io(format!("read {}", bundle.display()), e))?;
    // First pass: read manifest, then verify each entry by recomputing sha256.
    let mut archive = tar::Archive::new(&bytes[..]);
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    let mut manifest: Option<BundleManifest> = None;
    for entry in archive
        .entries()
        .map_err(|e| ForgeError::io("read bundle entries", e))?
    {
        let mut entry = entry.map_err(|e| ForgeError::io("read bundle entry", e))?;
        let path = entry
            .path()
            .map_err(|e| ForgeError::io("entry path", e))?
            .to_string_lossy()
            .into_owned();
        let mut data = Vec::new();
        entry
            .read_to_end(&mut data)
            .map_err(|e| ForgeError::io("read entry bytes", e))?;
        if path == "manifest.json" {
            manifest = Some(
                serde_json::from_slice(&data).map_err(|e| ForgeError::Config {
                    class: "bundle_integrity_error",
                    path: bundle.to_path_buf(),
                    message: format!("invalid manifest.json: {e}"),
                })?,
            );
        } else {
            entries.push((path, data));
        }
    }
    let manifest = manifest.ok_or_else(|| ForgeError::Config {
        class: "bundle_integrity_error",
        path: bundle.to_path_buf(),
        message: "bundle missing manifest.json".into(),
    })?;
    if !crate::cell::is_known_schema(&manifest.schema_version) {
        return Err(ForgeError::Config {
            class: "bundle_integrity_error",
            path: bundle.to_path_buf(),
            message: format!("unknown schema_version: {}", manifest.schema_version),
        });
    }
    // Build path→bytes map and verify each manifest file.
    let mut by_path: BTreeMap<String, Vec<u8>> = entries.into_iter().collect();
    let mut verified: Vec<(String, Vec<u8>)> = Vec::with_capacity(manifest.files.len());
    for expected in &manifest.files {
        let data = match by_path.remove(&expected.path) {
            Some(d) => d,
            None => {
                return Err(tamper_error(
                    bundle,
                    format!("missing bundle entry: {}", expected.path),
                ));
            }
        };
        if data.len() as u64 != expected.size {
            return Err(tamper_error(
                bundle,
                format!(
                    "size mismatch for {}: expected {}, got {}",
                    expected.path,
                    expected.size,
                    data.len()
                ),
            ));
        }
        let hash = format!("{:x}", Sha256::digest(&data));
        if hash != expected.sha256 {
            return Err(tamper_error(
                bundle,
                format!("sha256 mismatch for {}", expected.path),
            ));
        }
        verified.push((expected.path.clone(), data));
    }
    if !by_path.is_empty() {
        let extra: Vec<String> = by_path.into_keys().collect();
        return Err(tamper_error(
            bundle,
            format!("bundle contains entries not in manifest: {extra:?}"),
        ));
    }
    // All verified — stage to a temp dir, then rename into place atomically.
    let staging = root
        .join(".sea-forge/imported")
        .join(format!(".staging-{}", manifest.bundle_id));
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|e| ForgeError::io("clear stale staging", e))?;
    }
    fs::create_dir_all(&staging).map_err(|e| ForgeError::io("create staging", e))?;
    for (arc, data) in &verified {
        let dest = staging.join(arc);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| ForgeError::io("create arc parent", e))?;
        }
        let mut f = File::create(&dest)
            .map_err(|e| ForgeError::io(format!("create {}", dest.display()), e))?;
        f.write_all(data)
            .map_err(|e| ForgeError::io("write staged file", e))?;
    }
    let final_dir = root.join(".sea-forge/imported").join(&manifest.cell_id);
    // If the final dir already exists (re-import), keep it; atomic guarantee is
    // about not leaving partial NEW imports behind.
    if final_dir.exists() {
        // ponytail: re-import replaces prior — the cell_id names the trust root.
        fs::remove_dir_all(&final_dir).map_err(|e| ForgeError::io("clear prior import", e))?;
    }
    fs::rename(&staging, &final_dir).map_err(|e| {
        // If rename fails, clean up staging so the import is still atomic.
        let _ = fs::remove_dir_all(&staging);
        ForgeError::io("rename staging into place", e)
    })?;
    Ok(manifest)
}

fn push_file(
    builder: &mut tar::Builder<&mut Vec<u8>>,
    src: &Path,
    arc: &str,
    files: &mut Vec<BundleFile>,
) -> Result<(), ForgeError> {
    let bytes = fs::read(src).map_err(|e| ForgeError::io(format!("read {}", src.display()), e))?;
    let hash = format!("{:x}", Sha256::digest(&bytes));
    files.push(BundleFile {
        path: arc.into(),
        sha256: hash,
        size: bytes.len() as u64,
    });
    append_bytes(builder, arc, &bytes)?;
    Ok(())
}

fn push_tree(
    builder: &mut tar::Builder<&mut Vec<u8>>,
    src_dir: &Path,
    arc_prefix: &str,
    files: &mut Vec<BundleFile>,
) -> Result<(), ForgeError> {
    if !src_dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(src_dir).map_err(|e| ForgeError::io("read artifacts dir", e))? {
        let entry = entry.map_err(|e| ForgeError::io("dir entry", e))?;
        let path = entry.path();
        let arc = format!("{}/{}", arc_prefix, entry.file_name().to_string_lossy());
        if path.is_dir() {
            push_tree(builder, &path, &arc, files)?;
        } else {
            push_file(builder, &path, &arc, files)?;
        }
    }
    Ok(())
}

fn append_bytes(
    builder: &mut tar::Builder<&mut Vec<u8>>,
    arc: &str,
    bytes: &[u8],
) -> Result<(), ForgeError> {
    let mut header = tar::Header::new_gnu();
    header
        .set_path(arc)
        .map_err(|e| ForgeError::io("set path", e))?;
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append(&header, bytes)
        .map_err(|e| ForgeError::io(format!("append {arc}"), e))?;
    Ok(())
}

fn tamper_error(bundle: &Path, message: String) -> ForgeError {
    ForgeError::Config {
        class: "bundle_integrity_error",
        path: bundle.to_path_buf(),
        message,
    }
}

/// Parse a `name@version` template reference.
pub(crate) fn parse_template_ref(reference: &str) -> Result<(String, String), ForgeError> {
    let (name, version) = reference
        .split_once('@')
        .ok_or_else(|| ForgeError::Input(format!("invalid template reference {reference}")))?;
    if name.is_empty() || version.is_empty() {
        return Err(ForgeError::Input(format!(
            "invalid template reference {reference}"
        )));
    }
    Ok((name.into(), version.into()))
}

/// Path where an imported template lives before adoption.
/// `<root>/.sea-forge/imported/<cell_id>/templates/<name>@<version>.yaml`
pub fn imported_template_path(root: &Path, cell_id: &str, name: &str, version: &str) -> PathBuf {
    root.join(".sea-forge/imported")
        .join(cell_id)
        .join("templates")
        .join(format!("{name}@{version}.yaml"))
}

/// Read a manifest from a bundle tar (read-only).
pub fn read_manifest(bundle: &Path) -> Result<BundleManifest, ForgeError> {
    let bytes =
        fs::read(bundle).map_err(|e| ForgeError::io(format!("read {}", bundle.display()), e))?;
    let mut archive = tar::Archive::new(&bytes[..]);
    for entry in archive
        .entries()
        .map_err(|e| ForgeError::io("read bundle entries", e))?
    {
        let mut entry = entry.map_err(|e| ForgeError::io("read bundle entry", e))?;
        let path = entry
            .path()
            .map_err(|e| ForgeError::io("entry path", e))?
            .to_string_lossy()
            .into_owned();
        if path == "manifest.json" {
            let mut data = Vec::new();
            entry
                .read_to_end(&mut data)
                .map_err(|e| ForgeError::io("read manifest bytes", e))?;
            let manifest: BundleManifest =
                serde_json::from_slice(&data).map_err(|e| ForgeError::Config {
                    class: "bundle_integrity_error",
                    path: bundle.to_path_buf(),
                    message: format!("invalid manifest.json: {e}"),
                })?;
            return Ok(manifest);
        }
    }
    Err(ForgeError::Config {
        class: "bundle_integrity_error",
        path: bundle.to_path_buf(),
        message: "bundle missing manifest.json".into(),
    })
}
