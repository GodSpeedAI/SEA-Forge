use sea_forge_authority::AuthorityPolicyBundle;
use sea_forge_core::errors::ForgeError;
use sea_forge_ledger::{signing::load_or_create_signing_key, LedgerManager};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const MIGRATION_STREAM: &str = "legacy-import";
const LEGACY_RECORD_VERSION: &str = "0.1";

/// Bounded traversal: a legacy tree nested deeper than this is rejected
/// rather than recursing, so a symlink cycle can never exhaust the stack.
const MAX_MIGRATION_DEPTH: usize = 32;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LegacyImportPayload {
    path: String,
    sha256: String,
    size_bytes: u64,
    legacy_record_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MigrationMarker {
    version: String,
    migrated_at: String,
    stream_id: String,
    entry_count: u64,
    global_checkpoint_hash: String,
}

pub struct MigrateOptions<'a> {
    pub root: &'a Path,
    pub policy: Option<&'a Path>,
    pub key_dir: Option<&'a Path>,
    pub key_id: &'a str,
}

fn default_key_dir(root: &Path) -> PathBuf {
    root.parent()
        .unwrap_or(root)
        .join(".sea-forge-migration-keys")
}

fn resolve_key_config(options: &MigrateOptions) -> Result<(PathBuf, String), ForgeError> {
    if let Some(policy_path) = options.policy {
        let bundle = AuthorityPolicyBundle::load(policy_path)?;
        let key_dir = bundle
            .integrity_ledger
            .signing_key_dir
            .clone()
            .unwrap_or_else(|| default_key_dir(options.root));
        let key_id = bundle.integrity_ledger.signing_key_id.clone();
        return Ok((key_dir, key_id));
    }
    let key_dir = options
        .key_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_key_dir(options.root));
    Ok((key_dir, options.key_id.to_string()))
}

fn migration_marker_path(root: &Path) -> PathBuf {
    root.join("migration.json")
}

fn ensure_not_migrated(root: &Path) -> Result<(), ForgeError> {
    if migration_marker_path(root).exists() {
        return Err(ForgeError::Input(
            "migration already completed; refusing to re-migrate".into(),
        ));
    }
    Ok(())
}

fn read_case_mapping(root: &Path) -> Result<BTreeMap<String, String>, ForgeError> {
    let cases_dir = root.join("cases");
    let mut run_to_case: BTreeMap<String, String> = BTreeMap::new();
    if !cases_dir.is_dir() {
        return Ok(run_to_case);
    }
    for entry in fs::read_dir(&cases_dir)
        .map_err(|e| ForgeError::io("read cases directory", e))?
        .flatten()
    {
        let path = entry.path();
        let is_regular_file = entry
            .file_type()
            .map_err(|e| ForgeError::io("stat case entry", e))?
            .is_file();
        if !is_regular_file || path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|e| ForgeError::io("read case file", e))?;
        let case: sea_forge_core::types::Case =
            serde_json::from_str(&content).map_err(|e| ForgeError::Serialization(e.to_string()))?;
        for run_id in &case.run_ids {
            run_to_case.insert(run_id.clone(), case.case_id.clone());
        }
    }
    Ok(run_to_case)
}

fn target_path_for(
    original: &str,
    run_to_case: &BTreeMap<String, String>,
) -> Result<String, ForgeError> {
    let original_path = Path::new(original);
    let mut components = original_path.components().peekable();
    let first = components.next();
    let first = first.ok_or_else(|| ForgeError::Input("empty relative path".into()))?;
    let first = first
        .as_os_str()
        .to_str()
        .ok_or_else(|| ForgeError::Input("non-utf8 path component".into()))?;

    if first == "cases" {
        let second = components
            .next()
            .ok_or_else(|| ForgeError::Input("case path too short".into()))?;
        let file_name = second
            .as_os_str()
            .to_str()
            .ok_or_else(|| ForgeError::Input("non-utf8 case file name".into()))?;
        if file_name.ends_with(".json") && components.peek().is_none() {
            let case_id = file_name
                .strip_suffix(".json")
                .ok_or_else(|| ForgeError::Input("case file missing .json suffix".into()))?;
            return Ok(format!("cases/{case_id}/case.json"));
        }
    }

    if first == "runs" {
        let run_component = components
            .next()
            .ok_or_else(|| ForgeError::Input("run path missing run id".into()))?;
        let run_id = run_component
            .as_os_str()
            .to_str()
            .ok_or_else(|| ForgeError::Input("non-utf8 run id".into()))?;
        let case_id = run_to_case
            .get(run_id)
            .ok_or_else(|| ForgeError::Input(format!("run {run_id} not referenced by a case")))?;
        let rest: PathBuf = components.collect();
        let rest = rest
            .to_str()
            .ok_or_else(|| ForgeError::Input("non-utf8 path inside run directory".into()))?;
        if rest.is_empty() {
            return Ok(format!("cases/{case_id}/runs/{run_id}"));
        }
        return Ok(format!("cases/{case_id}/runs/{run_id}/{rest}"));
    }

    Ok(original.to_string())
}

fn subject_refs_for(original: &str) -> Vec<String> {
    let path = Path::new(original);
    let mut refs = Vec::new();
    if let Some(first) = path.components().next() {
        let first = first.as_os_str().to_str().unwrap_or("");
        if first == "runs" {
            if let Some(run_id) = path
                .components()
                .nth(1)
                .and_then(|c| c.as_os_str().to_str())
            {
                refs.push(format!("run:{run_id}"));
            }
        } else if first == "cases" {
            if let Some(file) = path.file_name().and_then(|s| s.to_str()) {
                if file.ends_with(".json") {
                    let case_id = file.strip_suffix(".json").unwrap_or(file);
                    refs.push(format!("case:{case_id}"));
                } else if let Some(case_id) = path
                    .components()
                    .nth(1)
                    .and_then(|c| c.as_os_str().to_str())
                {
                    refs.push(format!("case:{case_id}"));
                }
            }
        }
    }
    if refs.is_empty() {
        refs.push("global:root".into());
    }
    refs
}

fn sha256_file_with_prefix(path: &Path) -> Result<String, ForgeError> {
    let hash = sea_forge_evidence::sha256_file(path)?;
    Ok(format!("sha256:{hash}"))
}

fn enumerate_files_recursive(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    depth: usize,
) -> Result<(), ForgeError> {
    if depth > MAX_MIGRATION_DEPTH {
        return Err(ForgeError::Input(format!(
            "migration tree exceeds maximum depth {}: {}",
            MAX_MIGRATION_DEPTH,
            dir.display()
        )));
    }
    for entry in
        fs::read_dir(dir).map_err(|e| ForgeError::io(format!("read dir {}", dir.display()), e))?
    {
        let entry = entry.map_err(|e| ForgeError::io(format!("dir entry {}", dir.display()), e))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| ForgeError::io(format!("stat {}", path.display()), e))?;
        if file_type.is_symlink() {
            return Err(ForgeError::Input(format!(
                "refusing to migrate through symlink: {}",
                path.display()
            )));
        }
        if file_type.is_dir() {
            enumerate_files_recursive(&path, files, depth + 1)?;
        } else if file_type.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn enumerate_files(root: &Path, prefix: &Path) -> Result<Vec<PathBuf>, ForgeError> {
    let mut files = Vec::new();
    let dir = root.join(prefix);
    if !dir.is_dir() {
        return Ok(files);
    }
    enumerate_files_recursive(&dir, &mut files, 0)?;
    files.sort();
    Ok(files)
}

fn move_file_atomicish(src: &Path, dst: &Path) -> Result<(), ForgeError> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| ForgeError::io(format!("create {}", parent.display()), e))?;
    }
    fs::rename(src, dst)
        .map_err(|e| ForgeError::io(format!("move {} -> {}", src.display(), dst.display()), e))?;
    Ok(())
}

fn is_empty_dir(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false)
        && fs::read_dir(path)
            .map(|mut d| d.next().is_none())
            .unwrap_or(false)
}

fn remove_empty_dirs_recursive(dir: &Path, depth: usize) -> Result<(), ForgeError> {
    if depth > MAX_MIGRATION_DEPTH {
        return Err(ForgeError::Input(format!(
            "migration tree exceeds maximum depth {}: {}",
            MAX_MIGRATION_DEPTH,
            dir.display()
        )));
    }
    let is_real_dir = fs::symlink_metadata(dir)
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false);
    if !is_real_dir {
        return Ok(());
    }
    for entry in fs::read_dir(dir)
        .map_err(|e| ForgeError::io(format!("read dir {}", dir.display()), e))?
        .flatten()
    {
        let path = entry.path();
        let is_dir = entry
            .file_type()
            .map_err(|e| ForgeError::io(format!("stat {}", path.display()), e))?
            .is_dir();
        if is_dir {
            remove_empty_dirs_recursive(&path, depth + 1)?;
            if is_empty_dir(&path) {
                fs::remove_dir(&path).map_err(|e| {
                    ForgeError::io(format!("remove empty dir {}", path.display()), e)
                })?;
            }
        }
    }
    Ok(())
}

fn remove_empty_run_dirs(root: &Path) -> Result<(), ForgeError> {
    let legacy_runs = root.join("runs");
    remove_empty_dirs_recursive(&legacy_runs, 0)?;
    if is_empty_dir(&legacy_runs) {
        fs::remove_dir(&legacy_runs)
            .map_err(|e| ForgeError::io("remove empty legacy runs dir", e))?;
    }
    Ok(())
}

pub fn execute(options: MigrateOptions<'_>) -> Result<u8, ForgeError> {
    if !options.root.is_dir() {
        return Err(ForgeError::Input(format!(
            "root does not exist: {}",
            options.root.display()
        )));
    }
    let root = options
        .root
        .canonicalize()
        .map_err(|e| ForgeError::io("canonicalize root", e))?;
    ensure_not_migrated(&root)?;

    // Traverse the legacy tree before touching key material so symlink and
    // depth rejections happen before any filesystem side effect.
    let run_to_case = read_case_mapping(&root)?;
    let files = enumerate_files(&root, Path::new("."))?;

    let (key_dir, key_id) = resolve_key_config(&options)?;
    let signing_key = load_or_create_signing_key(&key_dir, &key_id)?;

    let mut entries: Vec<(String, LegacyImportPayload)> = Vec::new();
    for file in &files {
        let rel = file
            .strip_prefix(&root)
            .map_err(|_| ForgeError::Internal("file escaped root".into()))?;
        let rel_str = rel
            .to_str()
            .ok_or_else(|| ForgeError::Input("non-utf8 relative path".into()))?;
        // Skip files that are part of the ledger itself or migration marker.
        if rel_str == "migration.json"
            || rel_str.starts_with("ledgers/")
            || rel_str.starts_with("quarantine/")
            || rel_str.starts_with("authority/")
            || rel_str == "capabilities.jsonl"
        {
            continue;
        }
        let target = target_path_for(rel_str, &run_to_case)?;
        let size_bytes = fs::metadata(file)
            .map_err(|e| ForgeError::io(format!("metadata {}", file.display()), e))?
            .len();
        let sha256 = sha256_file_with_prefix(file)?;
        entries.push((
            rel_str.to_string(),
            LegacyImportPayload {
                path: target,
                sha256,
                size_bytes,
                legacy_record_version: LEGACY_RECORD_VERSION.into(),
            },
        ));
    }

    let manager = LedgerManager::new(&root)?;
    let stream = manager.open_stream(MIGRATION_STREAM, "sea-forge-migrate")?;
    let mut committed = Vec::new();
    for (original, payload) in &entries {
        let refs = subject_refs_for(original);
        let record = stream.commit_typed("legacy_import", refs, payload, vec![])?;
        committed.push(record);
    }

    let global_checkpoint = manager.create_global_checkpoint(&signing_key, &key_id)?;
    manager.verify_global_checkpoints(&signing_key.verifying_key())?;

    // Move files to their v0.2 case layout destinations.
    for (original, payload) in &entries {
        let src = root.join(original);
        let dst = root.join(&payload.path);
        move_file_atomicish(&src, &dst)?;
    }

    // Create empty case-level event logs for each migrated case.
    for case_id in run_to_case
        .values()
        .collect::<std::collections::BTreeSet<_>>()
    {
        let events_path = root.join("cases").join(case_id).join("case-events.jsonl");
        if !events_path.exists() {
            fs::write(&events_path, b"")
                .map_err(|e| ForgeError::io(format!("write {}", events_path.display()), e))?;
        }
    }

    // Remove the now-empty v0.1 run directories.
    remove_empty_run_dirs(&root)?;

    // Write the migration marker only after all prior steps succeeded.
    let marker = MigrationMarker {
        version: sea_forge_core::RECORD_VERSION.into(),
        migrated_at: chrono::Utc::now().to_rfc3339(),
        stream_id: MIGRATION_STREAM.into(),
        entry_count: committed.len() as u64,
        global_checkpoint_hash: global_checkpoint.global_checkpoint_hash,
    };
    fs::write(
        migration_marker_path(&root),
        serde_json::to_vec_pretty(&marker).map_err(|e| ForgeError::Serialization(e.to_string()))?,
    )
    .map_err(|e| ForgeError::io("write migration marker", e))?;

    println!("migrated {} legacy files", committed.len());
    println!("stream={}", MIGRATION_STREAM);
    println!("checkpoint_hash={}", marker.global_checkpoint_hash);
    Ok(0)
}
