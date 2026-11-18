//! Read and verify one immutable, published Field Case generation.

use sea_forge_core::path::{valid_id_segment, validate_relative_path};
use serde::de::{Error as _, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

// Exercise the writer under the production verifier and guard before enabling
// it in normal entry points. Runtime readers and platform gates must be wired first.
#[cfg(test)]
mod publisher;
#[cfg(test)]
mod stage;
#[cfg(test)]
pub(super) mod transaction;

#[derive(Debug)]
pub(super) enum StoreError {
    Io(io::Error),
    Invalid(&'static str),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Invalid(reason) => f.write_str(reason),
        }
    }
}
impl From<io::Error> for StoreError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

type Result<T> = std::result::Result<T, StoreError>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Selector {
    version: u8,
    case_id: String,
    generation: NonZeroU64,
    manifest_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u64,
    case_id: String,
    generation: u64,
    #[serde(default)]
    predecessor: Option<Predecessor>,
    #[serde(deserialize_with = "unique_files")]
    files: BTreeMap<String, String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Predecessor {
    generation: NonZeroU64,
    manifest_sha256: String,
}

fn unique_files<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, String>, D::Error> {
    struct Files;
    impl<'de> Visitor<'de> for Files {
        type Value = BTreeMap<String, String>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a map of unique file paths to hashes")
        }
        fn visit_map<M: MapAccess<'de>>(
            self,
            mut map: M,
        ) -> std::result::Result<Self::Value, M::Error> {
            let mut files = BTreeMap::new();
            while let Some((path, hash)) = map.next_entry::<String, String>()? {
                if files.insert(path, hash).is_some() {
                    return Err(M::Error::custom("duplicate file path"));
                }
            }
            Ok(files)
        }
    }
    d.deserialize_map(Files)
}

#[cfg(windows)]
fn is_link_or_reparse(meta: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_link_or_reparse(meta: &fs::Metadata) -> bool {
    meta.file_type().is_symlink()
}

fn metadata(path: &Path) -> Result<fs::Metadata> {
    let meta = fs::symlink_metadata(path)?;
    if is_link_or_reparse(&meta) {
        return Err(StoreError::Invalid("link or reparse point"));
    }
    Ok(meta)
}
fn directory(path: &Path) -> Result<()> {
    if !metadata(path)?.is_dir() {
        return Err(StoreError::Invalid("expected directory"));
    }
    Ok(())
}
fn read_file(path: &Path) -> Result<Vec<u8>> {
    if !metadata(path)?.is_file() {
        return Err(StoreError::Invalid("expected regular file"));
    }
    Ok(fs::read(path)?)
}
fn hash_ok(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn hash_file(path: &Path) -> Result<String> {
    if !metadata(path)?.is_file() {
        return Err(StoreError::Invalid("expected regular file"));
    }
    let mut f = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 16384];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}
fn walk(base: &Path, dir: &Path, found: &mut BTreeMap<String, String>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let meta = metadata(&path)?;
        if meta.is_dir() {
            walk(base, &path, found)?;
        } else if meta.is_file() {
            let relative = path
                .strip_prefix(base)
                .map_err(|_| StoreError::Invalid("invalid path"))?;
            let mut parts = Vec::new();
            for part in relative.components() {
                let std::path::Component::Normal(name) = part else {
                    return Err(StoreError::Invalid("invalid path"));
                };
                parts.push(name.to_str().ok_or(StoreError::Invalid("non-UTF-8 path"))?);
            }
            let name = parts.join("/");
            if name == "." || validate_relative_path(&name).is_err() {
                return Err(StoreError::Invalid("invalid file path"));
            }
            found.insert(name, hash_file(&path)?);
        } else {
            return Err(StoreError::Invalid("non-regular entry"));
        }
    }
    Ok(())
}

fn optional_directory(path: &Path) -> Result<bool> {
    match metadata(path) {
        Ok(meta) if meta.is_dir() => Ok(true),
        Ok(_) => Err(StoreError::Invalid("expected directory")),
        Err(StoreError::Io(error)) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

pub(super) fn storage_dir(root: &Path) -> Result<Option<PathBuf>> {
    let storage = root.join(".case-storage");
    Ok(optional_directory(&storage)?.then_some(storage))
}

/// An existing versioned namespace takes precedence over legacy files.
pub(super) fn namespace(root: &Path, case_id: &str) -> Result<Option<PathBuf>> {
    if !valid_id_segment(case_id, 128) {
        return Err(StoreError::Invalid("invalid case id"));
    }
    let Some(storage) = storage_dir(root)? else {
        return Ok(None);
    };
    let case = storage.join(case_id);
    Ok(optional_directory(&case)?.then_some(case))
}

/// Recognize the physical data root passed to a path-based mutation.
pub(super) fn is_generation_data(root: &Path) -> bool {
    let mut names = root.ancestors().take(5).map(Path::file_name);
    matches!(
        (names.next(), names.next(), names.next(), names.next(), names.next()),
        (Some(Some(data)), Some(Some(generation)), Some(Some(generations)), Some(Some(_)), Some(Some(storage)))
            if data == "data" && generations == "generations" && storage == ".case-storage"
                && generation.to_str().is_some_and(|value| value.parse::<NonZeroU64>().is_ok())
    )
}

/// Resolve a generation once per logical read. The store must retain selected
/// generations for readers and committed references.
pub(super) fn open(root: &Path, case_id: &str) -> Result<Option<PathBuf>> {
    Ok(open_verified(root, case_id)?.map(|snapshot| snapshot.case_dir))
}

pub(super) struct Snapshot {
    pub(super) case_dir: PathBuf,
    pub(super) generation: NonZeroU64,
    #[cfg(test)]
    manifest_sha256: String,
    predecessor: Option<Predecessor>,
    format_version: u64,
    #[cfg(test)]
    files: BTreeMap<String, String>,
}

fn open_verified(root: &Path, case_id: &str) -> Result<Option<Snapshot>> {
    let Some(case) = namespace(root, case_id)? else {
        return Ok(None);
    };
    let selector: Selector = serde_json::from_slice(&read_file(&case.join("current.json"))?)
        .map_err(|_| StoreError::Invalid("invalid selector"))?;
    if selector.version != 1 || selector.case_id != case_id || !hash_ok(&selector.manifest_sha256) {
        return Err(StoreError::Invalid("selector identity or digest mismatch"));
    }
    Ok(Some(verify_generation(
        &case,
        case_id,
        selector.generation,
        &selector.manifest_sha256,
    )?))
}

fn verify_generation(
    case: &Path,
    case_id: &str,
    number: NonZeroU64,
    digest: &str,
) -> Result<Snapshot> {
    if !hash_ok(digest) {
        return Err(StoreError::Invalid("invalid manifest digest"));
    }
    let generations = case.join("generations");
    directory(&generations)?;
    let generation = generations.join(number.to_string());
    directory(&generation)?;
    let bytes = read_file(&generation.join("manifest.json"))?;
    if hex::encode(Sha256::digest(&bytes)) != digest {
        return Err(StoreError::Invalid("manifest digest mismatch"));
    }
    let manifest: Manifest =
        serde_json::from_slice(&bytes).map_err(|_| StoreError::Invalid("invalid manifest"))?;
    if !(1..=2).contains(&manifest.version)
        || (manifest.version == 1 && manifest.predecessor.is_some())
        || (manifest.version == 2 && manifest.predecessor.is_none() && number.get() != 1)
        || manifest.case_id != case_id
        || manifest.generation != number.get()
    {
        return Err(StoreError::Invalid("manifest identity mismatch"));
    }
    if let Some(previous) = &manifest.predecessor {
        if previous.generation >= number || !hash_ok(&previous.manifest_sha256) {
            return Err(StoreError::Invalid("invalid predecessor"));
        }
    }
    for (name, hash) in &manifest.files {
        if name == "." || validate_relative_path(name).is_err() || !hash_ok(hash) {
            return Err(StoreError::Invalid("invalid manifest entry"));
        }
    }
    let data = generation.join("data");
    directory(&data)?;
    let mut actual = BTreeMap::new();
    walk(&data, &data, &mut actual)?;
    if actual != manifest.files {
        return Err(StoreError::Invalid("file set or digest mismatch"));
    }
    let case_dir = data.join("cases").join(case_id);
    let index = format!(".case-index/{case_id}.json");
    let case_prefix = format!("cases/{case_id}/");
    directory(&case_dir)?;
    if !actual.contains_key(&index)
        || actual
            .keys()
            .any(|name| name != &index && !name.starts_with(&case_prefix))
    {
        return Err(StoreError::Invalid("invalid case or index layout"));
    }
    Ok(Snapshot {
        case_dir,
        generation: number,
        #[cfg(test)]
        manifest_sha256: digest.to_owned(),
        predecessor: manifest.predecessor,
        format_version: manifest.version,
        #[cfg(test)]
        files: manifest.files,
    })
}

/// A cursor rooted in the selector read when the cursor was opened.
pub(super) struct History {
    case: PathBuf,
    case_id: String,
    current: Snapshot,
}

impl History {
    pub(super) fn open(root: &Path, case_id: &str) -> Result<Option<Self>> {
        let Some(current) = open_verified(root, case_id)? else {
            return Ok(None);
        };
        let case = namespace(root, case_id)?.ok_or(StoreError::Invalid("missing namespace"))?;
        Ok(Some(Self {
            case,
            case_id: case_id.to_owned(),
            current,
        }))
    }

    pub(super) fn current(&self) -> &Path {
        &self.current.case_dir
    }

    pub(super) fn previous(&mut self) -> Result<Option<&Path>> {
        let Some(previous) = &self.current.predecessor else {
            return Ok(None);
        };
        let parent = verify_generation(
            &self.case,
            &self.case_id,
            previous.generation,
            &previous.manifest_sha256,
        )?;
        if parent.generation >= self.current.generation
            || parent.format_version > self.current.format_version
        {
            return Err(StoreError::Invalid("invalid predecessor ordering"));
        }
        self.current = parent;
        Ok(Some(&self.current.case_dir))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    fn fixture() -> (TempDir, PathBuf) {
        let root = tempfile::tempdir().unwrap();
        let case = root.path().join(".case-storage/case-1");
        let generation = case.join("generations/1");
        fs::create_dir_all(generation.join("data/cases/case-1/sub")).unwrap();
        fs::create_dir_all(generation.join("data/.case-index")).unwrap();
        fs::write(generation.join("data/cases/case-1/sub/a.txt"), b"hello").unwrap();
        fs::write(generation.join("data/.case-index/case-1.json"), b"{}").unwrap();
        let manifest = serde_json::to_vec(&json!({"version":1,"case_id":"case-1","generation":1,
        "files":{
            "cases/case-1/sub/a.txt":hex::encode(Sha256::digest(b"hello")),
            ".case-index/case-1.json":hex::encode(Sha256::digest(b"{}")),
        }}))
        .unwrap();
        fs::write(generation.join("manifest.json"), &manifest).unwrap();
        write_selector(root.path(), &manifest);
        (root, generation)
    }
    fn write_selector(root: &Path, manifest: &[u8]) {
        let selector = json!({
            "version": 1,
            "case_id": "case-1",
            "generation": 1,
            "manifest_sha256": hex::encode(Sha256::digest(manifest)),
        });
        fs::write(
            root.join(".case-storage/case-1/current.json"),
            serde_json::to_vec(&selector).unwrap(),
        )
        .unwrap();
    }

    fn replace_manifest(root: &Path, generation: &Path, bytes: &[u8]) {
        fs::write(generation.join("manifest.json"), bytes).unwrap();
        write_selector(root, bytes);
    }
    #[test]
    fn history_follows_selected_links_and_rejects_damage() {
        let (root, first) = fixture();
        let parent_bytes = fs::read(first.join("manifest.json")).unwrap();
        let case = root.path().join(".case-storage/case-1");
        // Generation 2 was installed but never selected.
        fs::create_dir(case.join("generations/2")).unwrap();
        let third = case.join("generations/3");
        fs::create_dir_all(third.join("data/cases/case-1/sub")).unwrap();
        fs::create_dir_all(third.join("data/.case-index")).unwrap();
        fs::copy(
            first.join("data/cases/case-1/sub/a.txt"),
            third.join("data/cases/case-1/sub/a.txt"),
        )
        .unwrap();
        fs::copy(
            first.join("data/.case-index/case-1.json"),
            third.join("data/.case-index/case-1.json"),
        )
        .unwrap();
        let mut manifest: serde_json::Value = serde_json::from_slice(&parent_bytes).unwrap();
        manifest["version"] = json!(2);
        manifest["generation"] = json!(3);
        manifest["predecessor"] =
            json!({"generation": 1, "manifest_sha256": hex::encode(Sha256::digest(&parent_bytes))});
        let bytes = serde_json::to_vec(&manifest).unwrap();
        fs::write(third.join("manifest.json"), &bytes).unwrap();
        fs::write(
            case.join("current.json"),
            serde_json::to_vec(&json!({
                "version":1, "case_id":"case-1", "generation":3,
                "manifest_sha256":hex::encode(Sha256::digest(&bytes))
            }))
            .unwrap(),
        )
        .unwrap();
        let mut history = History::open(root.path(), "case-1").unwrap().unwrap();
        assert_eq!(history.current.generation.get(), 3);
        let selector = fs::read(case.join("current.json")).unwrap();
        fs::write(case.join("current.json"), b"changed after pin").unwrap();
        assert!(history.previous().unwrap().is_some());
        assert_eq!(history.current.generation.get(), 1);
        assert!(history.previous().unwrap().is_none());
        fs::write(case.join("current.json"), selector).unwrap();
        let mut history = History::open(root.path(), "case-1").unwrap().unwrap();
        fs::write(first.join("manifest.json"), b"damaged").unwrap();
        assert!(history.previous().is_err());
        assert!(open(root.path(), "case-1").is_ok());
    }

    #[test]
    fn selected_tree_and_absence() {
        let (root, generation) = fixture();
        assert_eq!(
            open(root.path(), "case-1").unwrap(),
            Some(generation.join("data/cases/case-1"))
        );
        assert_eq!(open(root.path(), "other").unwrap(), None);
        fs::create_dir_all(root.path().join("cases/case-1")).unwrap();
        fs::create_dir_all(root.path().join(".case-storage/.stage-123")).unwrap();
        assert_eq!(super::super::case_ids(root.path()).unwrap(), ["case-1"]);
        fs::create_dir_all(root.path().join(".case-storage/case-2")).unwrap();
        assert_eq!(
            super::super::case_ids(root.path()).unwrap(),
            ["case-1", "case-2"]
        );
    }
    #[test]
    fn published_case_and_index_are_bound_together() {
        let (root, generation) = fixture();
        let selected = open(root.path(), "case-1").unwrap().unwrap();
        assert_eq!(
            super::super::case_guard::index_path(&selected),
            generation
                .canonicalize()
                .unwrap()
                .join("data/.case-index/case-1.json")
        );
        assert!(super::super::case_guard::CaseLock::acquire(&selected).is_err());
        let logical = root.path().join("cases/case-1");
        assert!(super::super::case_guard::CaseLock::acquire(&logical).is_err());
        assert!(!generation.join("data/.case-locks").exists());
        fs::write(generation.join("data/.case-index/case-1.json"), b"changed").unwrap();
        assert!(open(root.path(), "case-1").is_err());
    }

    #[test]
    fn rejects_corruption_and_incomplete_namespace() {
        let (root, generation) = fixture();
        fs::write(generation.join("data/cases/case-1/sub/a.txt"), b"changed").unwrap();
        assert!(open(root.path(), "case-1").is_err());
        let (root, generation) = fixture();
        fs::write(generation.join("data/extra"), b"extra").unwrap();
        assert!(open(root.path(), "case-1").is_err());
        let (root, generation) = fixture();
        fs::remove_file(generation.join("data/cases/case-1/sub/a.txt")).unwrap();
        assert!(open(root.path(), "case-1").is_err());
        let (root, generation) = fixture();
        fs::write(generation.join("manifest.json"), b"{}").unwrap();
        assert!(open(root.path(), "case-1").is_err());
        let (root, generation) = fixture();
        replace_manifest(
            root.path(),
            &generation,
            br#"{"version":1,"case_id":"case-1","generation":1,"files":{"a":"00","a":"00"}}"#,
        );
        assert!(open(root.path(), "case-1").is_err());
        let (root, generation) = fixture();
        replace_manifest(
            root.path(),
            &generation,
            br#"{"version":1,"case_id":"other","generation":1,"files":{}}"#,
        );
        assert!(open(root.path(), "case-1").is_err());
        let (root, _) = fixture();
        fs::remove_file(root.path().join(".case-storage/case-1/current.json")).unwrap();
        assert!(open(root.path(), "case-1").is_err());
    }
}
