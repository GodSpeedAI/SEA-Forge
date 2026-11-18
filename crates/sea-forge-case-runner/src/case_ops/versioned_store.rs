//! Read and verify one immutable, published Field Case generation.

use sea_forge_core::path::{valid_id_segment, validate_relative_path};
use serde::de::{Error as _, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

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
    #[serde(deserialize_with = "unique_files")]
    files: BTreeMap<String, String>,
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

/// An existing versioned namespace takes precedence over legacy files.
pub(super) fn namespace(root: &Path, case_id: &str) -> Result<Option<PathBuf>> {
    if !valid_id_segment(case_id, 128) {
        return Err(StoreError::Invalid("invalid case id"));
    }
    let storage = root.join(".case-storage");
    if !optional_directory(&storage)? {
        return Ok(None);
    }
    let case = storage.join(case_id);
    Ok(optional_directory(&case)?.then_some(case))
}

/// Resolve a generation once per logical read. The store must retain selected
/// generations for readers and committed references.
pub(super) fn open(root: &Path, case_id: &str) -> Result<Option<PathBuf>> {
    let Some(case) = namespace(root, case_id)? else {
        return Ok(None);
    };
    let selector: Selector = serde_json::from_slice(&read_file(&case.join("current.json"))?)
        .map_err(|_| StoreError::Invalid("invalid selector"))?;
    if selector.version != 1 || selector.case_id != case_id || !hash_ok(&selector.manifest_sha256) {
        return Err(StoreError::Invalid("selector identity or digest mismatch"));
    }
    let generations = case.join("generations");
    directory(&generations)?;
    let generation = generations.join(selector.generation.to_string());
    directory(&generation)?;
    let bytes = read_file(&generation.join("manifest.json"))?;
    if hex::encode(Sha256::digest(&bytes)) != selector.manifest_sha256 {
        return Err(StoreError::Invalid("manifest digest mismatch"));
    }
    let manifest: Manifest =
        serde_json::from_slice(&bytes).map_err(|_| StoreError::Invalid("invalid manifest"))?;
    if manifest.version != 1
        || manifest.case_id != case_id
        || manifest.generation != selector.generation.get()
    {
        return Err(StoreError::Invalid("manifest identity mismatch"));
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
    Ok(Some(data))
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
        fs::create_dir_all(generation.join("data/sub")).unwrap();
        fs::write(generation.join("data/sub/a.txt"), b"hello").unwrap();
        let manifest = serde_json::to_vec(&json!({"version":1,"case_id":"case-1","generation":1,
            "files":{"sub/a.txt":hex::encode(Sha256::digest(b"hello"))}}))
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
    fn selected_tree_and_absence() {
        let (root, generation) = fixture();
        assert_eq!(
            open(root.path(), "case-1").unwrap(),
            Some(generation.join("data"))
        );
        assert_eq!(open(root.path(), "other").unwrap(), None);
    }
    #[test]
    fn rejects_corruption_and_incomplete_namespace() {
        let (root, generation) = fixture();
        fs::write(generation.join("data/sub/a.txt"), b"changed").unwrap();
        assert!(open(root.path(), "case-1").is_err());
        let (root, generation) = fixture();
        fs::write(generation.join("data/extra"), b"extra").unwrap();
        assert!(open(root.path(), "case-1").is_err());
        let (root, generation) = fixture();
        fs::remove_file(generation.join("data/sub/a.txt")).unwrap();
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
