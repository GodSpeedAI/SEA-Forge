//! Federation bundle export/import (spec-full §7.4, §14.8 atomic reject).

use crate::cell::{schema_version, CellRecord};
use sea_forge_core::{
    errors::ForgeError,
    ids,
    types::{BundleFile, BundleManifest},
};
use sea_forge_sandbox::{safe_join, safe_lexical_join};
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

/// Maximum accepted size of a bundle file.
///
/// Import holds the whole bundle in memory at once — the raw bytes, then a
/// per-entry copy, then the verified copy — and every hash check happens
/// *after* the read. An unbounded read therefore exhausts memory before the
/// integrity checks that would have rejected the file get a chance to run,
/// and bundle bytes are untrusted by construction (§14.8).
// ponytail: fixed ceiling, no config knob; raise only when a real exported
// cell is measured near it.
const MAX_BUNDLE_BYTES: u64 = 256 * 1024 * 1024;

/// Read a bundle file, rejecting anything over [`MAX_BUNDLE_BYTES`].
///
/// Reads one byte past the ceiling rather than consulting `metadata` first, so
/// the bound is enforced by the read itself; a `metadata` check could be
/// invalidated by a concurrent writer between the check and the read.
fn read_bundle_bytes(bundle: &Path) -> Result<Vec<u8>, ForgeError> {
    let file =
        File::open(bundle).map_err(|e| ForgeError::io(format!("read {}", bundle.display()), e))?;
    let mut bytes = Vec::new();
    file.take(MAX_BUNDLE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| ForgeError::io(format!("read {}", bundle.display()), e))?;
    if bytes.len() as u64 > MAX_BUNDLE_BYTES {
        return Err(tamper_error(
            bundle,
            format!("bundle exceeds the {MAX_BUNDLE_BYTES}-byte limit"),
        ));
    }
    Ok(bytes)
}

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
    let bytes = read_bundle_bytes(bundle)?;
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
    // All content is hash-valid but the manifest-controlled *paths* are still
    // untrusted input. Fail closed: validate every path that will be joined
    // into the filesystem (staging dir name from `bundle_id`, final dir name
    // from `cell_id`, and each archived file path) BEFORE the first mutation.
    // A single hostile path aborts the whole import with `bundle_integrity_error`
    // and leaves the import root and anything outside it byte-identical.
    //
    // Establish the trusted import-root hierarchy beneath the canonical root:
    // `.sea-forge` and `imported` must each be a real directory (created if
    // absent), never a symlink whose target escapes root. A pre-existing
    // symlink at either level is an import-time escape vector and fails
    // closed with `bundle_integrity_error` before any per-entry write. All
    // later staging/final paths derive from the canonical `imported_root`,
    // so they inherit the trusted root rather than trusting lexical joins
    // over untrusted parent segments.
    let canonical_root = root
        .canonicalize()
        .map_err(|e| ForgeError::io("canonicalize import root", e))?;
    let sea_forge_dir = ensure_trusted_dir(&canonical_root, ".sea-forge")
        .map_err(|e| tamper_error(bundle, format!("unsafe .sea-forge: {e}")))?;
    let imported_root = ensure_trusted_dir(&sea_forge_dir, "imported")
        .map_err(|e| tamper_error(bundle, format!("unsafe imported root: {e}")))?;
    // The `bundle_id`/`cell_id` are single directory-name segments joined under
    // `imported/`; reuse the canonical lexical validator (the staging/final
    // subtree may not exist yet). `safe_lexical_join` rejects absolute paths,
    // `..`, root/prefix components, and ambiguous spellings.
    let staging_name = format!(".staging-{}", manifest.bundle_id);
    let staging = safe_lexical_join(&imported_root, &staging_name)
        .map_err(|e| tamper_error(bundle, format!("unsafe bundle_id path: {e}")))?;
    let final_dir = safe_lexical_join(&imported_root, &manifest.cell_id)
        .map_err(|e| tamper_error(bundle, format!("unsafe cell_id path: {e}")))?;
    // Each manifest file path is lexically validated relative to staging, and —
    // once the final dir's parents exist on disk — checked with `safe_join`
    // against the final dir so a pre-existing symlink parent (a re-import trap)
    // cannot let a write escape the trust root.
    for (arc, _data) in &verified {
        safe_lexical_join(&staging, arc)
            .map_err(|e| tamper_error(bundle, format!("unsafe manifest path: {e}")))?;
        // Filesystem-level symlink-parent escape check against the final dir,
        // which may already exist from a prior import. `safe_join` canonicalizes
        // and inspects symlinks; it only creates parents under the final dir,
        // never outside it, and never touches the sentinel.
        if final_dir.exists() {
            safe_join(&final_dir, arc)
                .map_err(|e| tamper_error(bundle, format!("unsafe manifest path: {e}")))?;
        }
    }

    // Paths are proven safe — stage to a temp dir, then rename into place
    // atomically.
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|e| ForgeError::io("clear stale staging", e))?;
    }
    fs::create_dir_all(&staging).map_err(|e| ForgeError::io("create staging", e))?;
    for (arc, data) in &verified {
        let dest = safe_lexical_join(&staging, arc)
            .map_err(|e| tamper_error(bundle, format!("unsafe manifest path: {e}")))?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| ForgeError::io("create arc parent", e))?;
        }
        let mut f = File::create(&dest)
            .map_err(|e| ForgeError::io(format!("create {}", dest.display()), e))?;
        f.write_all(data)
            .map_err(|e| ForgeError::io("write staged file", e))?;
    }
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

/// Validate one directory segment beneath a trusted canonical parent: if the
/// segment exists it must be a real directory whose canonical path stays
/// beneath `parent` (never a symlink). If absent, the lexical path is
/// returned unchanged — creation is deferred to the staging step so a
/// rejected import leaves zero filesystem side effects. Used to harden
/// `.sea-forge` and `imported` against pre-existing symlink escapes at
/// import time.
fn ensure_trusted_dir(parent: &Path, name: &str) -> Result<PathBuf, ForgeError> {
    let candidate = parent.join(name);
    match fs::symlink_metadata(&candidate) {
        Ok(md) if md.file_type().is_symlink() => {
            Err(ForgeError::UnsafePath(format!("{name} is a symlink")))
        }
        Ok(md) if !md.is_dir() => Err(ForgeError::UnsafePath(format!("{name} is not a directory"))),
        Ok(_) => {
            let canonical = candidate
                .canonicalize()
                .map_err(|e| ForgeError::io(format!("canonicalize {name}"), e))?;
            if !canonical.starts_with(parent) {
                return Err(ForgeError::UnsafePath(format!("{name} escapes trust root")));
            }
            Ok(canonical)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(candidate),
        Err(e) => Err(ForgeError::io(format!("inspect {name}"), e)),
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
    let bytes = read_bundle_bytes(bundle)?;
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
