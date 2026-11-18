//! Prepare and seal an editable stage before publication.
//!
//! The caller holds the logical case lock and supplies verified owned paths
//! and flush operations qualified for the platform and volume.

use super::{
    directory, hash_ok, metadata, walk, Manifest, Predecessor, Result, Selector, StoreError,
};
use sea_forge_core::path::{valid_id_segment, validate_relative_path};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// First: `parent` is the namespace destination's parent.
/// Next: `case` is an existing namespace with an existing generations directory.
pub(super) enum Layout<'a> {
    First { parent: &'a Path },
    Next { case: &'a Path },
}

/// Callback contract includes file data and metadata, and directory entries.
pub(super) struct Flush<'a> {
    pub file: &'a dyn Fn(&File) -> io::Result<()>,
    pub dir: &'a dyn Fn(&Path) -> io::Result<()>,
}

/// A generation that is writable only before sealing.
pub(super) struct Stage {
    temporary: SealedStage,
    gen: PathBuf,
    parent: PathBuf,
    data: PathBuf,
    case_id: String,
    generation: NonZeroU64,
    next_namespace: Option<PathBuf>,
    predecessor: Option<Predecessor>,
}

/// Access paths by reference and retain this guard through publication.
pub(super) struct SealedStage {
    dir: PathBuf,
    selector: Option<PathBuf>,
}

impl SealedStage {
    pub(super) fn directory(&self) -> &Path {
        &self.dir
    }

    pub(super) fn selector(&self) -> Option<&Path> {
        self.selector.as_deref()
    }
}

impl Drop for SealedStage {
    fn drop(&mut self) {
        if let Some(path) = &self.selector {
            let _ = fs::remove_file(path);
        }
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn unique_name() -> String {
    format!(
        ".wp21b-stage-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}
fn private_dir(parent: &Path) -> Result<PathBuf> {
    for _ in 0..1024 {
        let path = parent.join(unique_name());
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(&path) {
            Ok(()) => return Ok(path),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Err(StoreError::Invalid(
        "cannot allocate unique staging directory",
    ))
}
fn private_file(parent: &Path) -> Result<(PathBuf, File)> {
    for _ in 0..1024 {
        let path = parent.join(unique_name());
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Err(StoreError::Invalid(
        "cannot allocate unique staging selector",
    ))
}
fn write_new(path: &Path, bytes: &[u8], flush: &Flush<'_>) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    (flush.file)(&file)?;
    Ok(())
}
fn flush_tree(path: &Path, flush: &Flush<'_>) -> Result<()> {
    // Flush files and directories bottom-up after the batch has been applied.
    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        let meta = metadata(&path)?;
        if meta.is_dir() {
            flush_tree(&path, flush)?;
        } else if meta.is_file() {
            (flush.file)(&OpenOptions::new().write(true).open(&path)?)?;
        } else {
            return Err(StoreError::Invalid("non-regular stage entry"));
        }
    }
    (flush.dir)(path)?;
    Ok(())
}

impl Stage {
    /// Allocate a private generation, optionally seeding it from the caller's
    /// complete, verified set of owned source paths. The caller holds the
    /// logical lock and keeps the source and stage exclusively controlled.
    pub(super) fn prepare(
        layout: Layout<'_>,
        case_id: &str,
        generation: NonZeroU64,
        source: Option<(&Path, &BTreeMap<String, String>)>,
    ) -> Result<Self> {
        if !valid_id_segment(case_id, 128) {
            return Err(StoreError::Invalid("invalid case id"));
        }
        let (parent, next_namespace) = match layout {
            Layout::First { parent } => (parent.to_path_buf(), None),
            Layout::Next { case } => (case.join("generations"), Some(case.to_path_buf())),
        };
        directory(&parent)?;
        let temporary = SealedStage {
            dir: private_dir(&parent)?,
            selector: None,
        };
        let gen = match next_namespace {
            Some(_) => temporary.dir.clone(),
            None => {
                let gen = temporary
                    .dir
                    .join("generations")
                    .join(generation.to_string());
                fs::create_dir_all(&gen)?;
                gen
            }
        };
        let data = gen.join("data");
        fs::create_dir_all(data.join("cases").join(case_id))?;

        let stage = Self {
            temporary,
            gen,
            parent,
            data,
            case_id: case_id.into(),
            generation,
            next_namespace,
            predecessor: None,
        };
        if let Some((source, files)) = source {
            stage.seed(source, files)?;
        }
        Ok(stage)
    }

    pub(super) fn set_predecessor(&mut self, predecessor: Predecessor) -> Result<()> {
        if predecessor.generation >= self.generation || !hash_ok(&predecessor.manifest_sha256) {
            return Err(StoreError::Invalid("invalid predecessor"));
        }
        self.predecessor = Some(predecessor);
        Ok(())
    }

    pub(super) fn case_dir(&self) -> PathBuf {
        self.data.join("cases").join(&self.case_id)
    }

    fn validate_paths(&self, files: &BTreeMap<String, String>) -> Result<()> {
        let index = format!(".case-index/{}.json", self.case_id);
        let prefix = format!("cases/{}/", self.case_id);
        if files.iter().any(|(name, digest)| {
            name == "."
                || validate_relative_path(name).is_err()
                || !hash_ok(digest)
                || (name != &index && !name.starts_with(&prefix))
        }) {
            return Err(StoreError::Invalid("invalid qualified file set"));
        }
        Ok(())
    }

    fn seed(&self, source_data: &Path, files: &BTreeMap<String, String>) -> Result<()> {
        self.validate_paths(files)?;
        directory(source_data)?;
        for (name, expected) in files {
            let source = source_data.join(name);
            for ancestor in source
                .ancestors()
                .skip(1)
                .take_while(|path| *path != source_data)
            {
                directory(ancestor)?;
            }
            let destination = self.data.join(name);
            fs::create_dir_all(
                destination
                    .parent()
                    .ok_or(StoreError::Invalid("invalid destination"))?,
            )?;
            copy_verified(&source, &destination, expected)?;
        }
        Ok(())
    }

    /// Seal only after the case writer has applied the batch and verified the
    /// complete resulting state. The caller supplies a qualified flush backend.
    pub(super) fn seal(mut self, flush: &Flush<'_>) -> Result<SealedStage> {
        let lock_dir = self.data.join(".case-locks");
        match fs::remove_dir(lock_dir) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        let mut files = BTreeMap::new();
        walk(&self.data, &self.data, &mut files)?;
        self.validate_paths(&files)?;
        let index = format!(".case-index/{}.json", self.case_id);
        if !files.contains_key(&index) {
            return Err(StoreError::Invalid("staged case index is missing"));
        }
        let manifest = serde_json::to_vec(&serde_json::json!({
            "version": 2,
            "case_id": self.case_id,
            "generation": self.generation.get(),
            "predecessor": self.predecessor,
            "files": files,
        }))
        .map_err(|_| StoreError::Invalid("manifest serialization"))?;
        let parsed: Manifest = serde_json::from_slice(&manifest)
            .map_err(|_| StoreError::Invalid("manifest verification"))?;
        if parsed.files != files {
            return Err(StoreError::Invalid("manifest digest inputs changed"));
        }
        write_new(&self.gen.join("manifest.json"), &manifest, flush)?;

        let selector = serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "case_id": self.case_id,
            "generation": self.generation.get(),
            "manifest_sha256": hex::encode(Sha256::digest(&manifest)),
        }))
        .map_err(|_| StoreError::Invalid("selector serialization"))?;
        let _: Selector = serde_json::from_slice(&selector)
            .map_err(|_| StoreError::Invalid("selector verification"))?;

        if let Some(case) = &self.next_namespace {
            let (path, mut file) = private_file(case)?;
            self.temporary.selector = Some(path);
            file.write_all(&selector)?;
            (flush.file)(&file)?;
            (flush.dir)(case)?;
        } else {
            write_new(&self.temporary.dir.join("current.json"), &selector, flush)?;
        }
        flush_tree(&self.temporary.dir, flush)?;
        (flush.dir)(&self.parent)?;
        Ok(self.temporary)
    }
}

fn copy_verified(source: &Path, destination: &Path, expected: &str) -> Result<()> {
    if !metadata(source)?.is_file() {
        return Err(StoreError::Invalid("expected regular source file"));
    }
    let mut input = File::open(source)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 16384];
    loop {
        let n = input.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        output.write_all(&buffer[..n])?;
        digest.update(&buffer[..n]);
    }
    if hex::encode(digest.finalize()) != expected {
        return Err(StoreError::Invalid("source changed during staged copy"));
    }
    Ok(())
}
