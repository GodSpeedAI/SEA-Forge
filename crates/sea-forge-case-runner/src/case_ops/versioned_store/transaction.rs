//! Staged publication of an already-admitted Field Case batch.
//!
//! The caller holds the logical case lock, establishes the set of owned source
//! paths, and supplies directory sync for the qualified platform and volume.

use super::{
    directory, hash_file, namespace, open_verified, optional_directory,
    publisher::{self, Publication, PublishError},
    stage, walk, StoreError,
};
use crate::case_ops::case_guard::CommitError;
use crate::case_ops::field_guard::Refusal;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::num::NonZeroU64;
use std::path::Path;

fn storage_refusal(error: impl std::fmt::Display) -> CommitError {
    Refusal {
        code: "STORE_INTEGRITY",
        message: format!("stage Field Case snapshot: {error}"),
    }
    .into()
}

/// Choose a free generation without overwriting prior or unselected data.
fn next_generation(case: &Path) -> io::Result<NonZeroU64> {
    let mut maximum = 0;
    for entry in fs::read_dir(case.join("generations"))? {
        if let Some(number) = entry?
            .file_name()
            .to_str()
            .and_then(|name| name.parse().ok())
        {
            maximum = maximum.max(number);
        }
    }
    u64::checked_add(maximum, 1)
        .and_then(NonZeroU64::new)
        .ok_or_else(|| io::Error::other("Field Case generation sequence exhausted"))
}

fn verify_legacy_files(
    root: &Path,
    case_id: &str,
    supplied: &BTreeMap<String, String>,
) -> super::Result<()> {
    let cases = root.join("cases");
    let case = cases.join(case_id);
    directory(&cases)?;
    directory(&case)?;
    let mut actual = BTreeMap::new();
    walk(root, &case, &mut actual)?;
    let indexes = root.join(".case-index");
    if optional_directory(&indexes)? {
        let relative = format!(".case-index/{case_id}.json");
        match hash_file(&root.join(&relative)) {
            Ok(hash) => {
                actual.insert(relative, hash);
            }
            Err(StoreError::Io(error)) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    if &actual != supplied {
        return Err(StoreError::Invalid("legacy file set or digest mismatch"));
    }
    Ok(())
}

/// First publication can use a verified legacy base if the caller has already
/// classified its owned paths. A new case supplies no base.
pub(in crate::case_ops) fn replace<T>(
    root: &Path,
    case_id: &str,
    legacy_files: Option<&BTreeMap<String, String>>,
    flush_dir: &dyn Fn(&Path) -> io::Result<()>,
    apply_and_verify: impl FnOnce(&Path) -> Result<T, Refusal>,
) -> Result<T, CommitError> {
    let storage = root.join(".case-storage");
    let selected = open_verified(root, case_id).map_err(storage_refusal)?;
    let case = namespace(root, case_id).map_err(storage_refusal)?;
    let (layout, generation, source) = if let (Some(case), Some(selected)) = (&case, &selected) {
        let data =
            selected.case_dir.ancestors().nth(2).ok_or_else(|| {
                storage_refusal(StoreError::Invalid("invalid selected case path"))
            })?;
        (
            stage::Layout::Next { case },
            next_generation(case).map_err(storage_refusal)?,
            Some((data, &selected.files)),
        )
    } else {
        if selected.is_some() || case.is_some() {
            return Err(storage_refusal("case namespace changed while locked"));
        }
        if let Some(files) = legacy_files {
            verify_legacy_files(root, case_id, files).map_err(storage_refusal)?;
        }
        match fs::create_dir(&storage) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(storage_refusal(error)),
        }
        // Check the qualification and persist the storage parent before any
        // attempt to publish a new case namespace.
        flush_dir(root).map_err(storage_refusal)?;
        (
            stage::Layout::First { parent: &storage },
            NonZeroU64::MIN,
            legacy_files.map(|files| (root, files)),
        )
    };
    let stage =
        stage::Stage::prepare(layout, case_id, generation, source).map_err(storage_refusal)?;
    let applied = apply_and_verify(&stage.case_dir())?;
    let sealed = stage
        .seal(&stage::Flush {
            file: &fs::File::sync_all,
            dir: flush_dir,
        })
        .map_err(storage_refusal)?;
    let target = storage.join(case_id);
    let publication = match sealed.selector() {
        Some(selector) => Publication::Next {
            case: &target,
            staged_generation: sealed.directory(),
            generation,
            staged_selector: selector,
        },
        None => Publication::First {
            staged_namespace: sealed.directory(),
            case: &target,
        },
    };
    publisher::publish(publication, flush_dir).map_err(|error| match error {
        PublishError::PreCommit(error) => storage_refusal(error),
        PublishError::PublishedUncertain(error) => CommitError::PublicationUncertain(error),
    })?;
    Ok(applied)
}
