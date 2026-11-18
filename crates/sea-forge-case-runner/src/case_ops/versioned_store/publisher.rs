//! Install a verified generation and select it with one atomic rename.
//!
//! The caller holds the logical lock, qualifies same-volume atomic rename and
//! directory durability on the selected volume, and passes preflushed stages.

use std::fs;
use std::io;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

pub enum Publication<'a> {
    /// `staged_namespace` is a sibling of the absent `case` destination.
    First {
        staged_namespace: &'a Path,
        case: &'a Path,
    },
    /// `staged_generation` is a sibling of `generations/<number>`;
    /// `staged_selector` is a sibling of `case/current.json`.
    Next {
        case: &'a Path,
        staged_generation: &'a Path,
        generation: NonZeroU64,
        staged_selector: &'a Path,
    },
}

#[derive(Debug)]
pub enum PublishError {
    PreCommit(io::Error),
    PublishedUncertain(io::Error),
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn parent(path: &Path) -> io::Result<&Path> {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| invalid("path must have a nonempty parent"))
}

fn require_siblings(source: &Path, target: &Path) -> io::Result<()> {
    if source == target || parent(source)? != parent(target)? {
        return Err(invalid("stage and target must be distinct siblings"));
    }
    Ok(())
}

fn require_absent(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "publication slot occupied",
        )),
    }
}

/// `flush_dir` must satisfy the platform qualification contract above.
pub fn publish(
    publication: Publication<'_>,
    flush_dir: &dyn Fn(&Path) -> io::Result<()>,
) -> Result<(), PublishError> {
    use PublishError::{PreCommit, PublishedUncertain};
    match publication {
        Publication::First {
            staged_namespace,
            case,
        } => {
            require_siblings(staged_namespace, case).map_err(PreCommit)?;
            require_absent(case).map_err(PreCommit)?;
            fs::rename(staged_namespace, case).map_err(PreCommit)?; // commit point
            flush_dir(parent(case).map_err(PublishedUncertain)?).map_err(PublishedUncertain)
        }
        Publication::Next {
            case,
            staged_generation,
            generation,
            staged_selector,
        } => {
            let generations: PathBuf = case.join("generations");
            let slot = generations.join(generation.get().to_string());
            let current = case.join("current.json");
            require_siblings(staged_generation, &slot).map_err(PreCommit)?;
            require_siblings(staged_selector, &current).map_err(PreCommit)?;
            require_absent(&slot).map_err(PreCommit)?;
            // Under the caller's exclusion contract, these are regular files.
            if !fs::symlink_metadata(&current)
                .map_err(PreCommit)?
                .file_type()
                .is_file()
                || !fs::symlink_metadata(staged_selector)
                    .map_err(PreCommit)?
                    .file_type()
                    .is_file()
            {
                return Err(PreCommit(invalid("selectors must be regular files")));
            }
            fs::rename(staged_generation, &slot).map_err(PreCommit)?;
            // Ensure the generation binding precedes publication of its selector.
            flush_dir(&generations).map_err(PreCommit)?;
            fs::rename(staged_selector, &current).map_err(PreCommit)?; // commit point
            flush_dir(case).map_err(PublishedUncertain)
        }
    }
}
