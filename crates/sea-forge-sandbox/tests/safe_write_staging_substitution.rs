//! F32 regression (black box): temp-name entries planted in the destination
//! directory can neither block nor capture a safe write.
//!
//! The pre-staging scheme created and renamed the temp file in the
//! destination directory itself, so names pre-planted there collided with
//! the write (it was refused outright) and a mid-write swap could publish
//! attacker bytes. The staged scheme resolves the rename source inside its
//! own protected directory, so destination-directory names are inert.

#![cfg(unix)]

use std::fs;
use std::process::id;
use std::time::{SystemTime, UNIX_EPOCH};

fn staging_fixture(tag: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let parent = std::env::temp_dir().join(format!("sea-forge-{tag}-{}-{nonce}", id()));
    let root = parent.join("workspace");
    fs::create_dir_all(&root).unwrap();
    (parent, root)
}

/// Pre-plants the whole candidate range of the old in-destination temp name
/// (same process id, sequences 0..64) and requires the write to succeed with
/// exactly the caller's bytes.
#[test]
fn pre_planted_temp_names_cannot_block_or_capture_the_write() {
    let (parent, root) = staging_fixture("staged-preplant");
    let pid = id();
    for sequence in 0..64u64 {
        fs::write(
            root.join(format!(".planned.txt.sf-tmp-{pid}-{sequence}")),
            b"attacker bytes",
        )
        .unwrap();
    }
    let destination = root.join("planned.txt");

    sea_forge_sandbox::safe_write(&destination, b"validated bytes")
        .expect("a planted temp name must not block or capture the write");

    let meta = fs::symlink_metadata(&destination).unwrap();
    assert!(meta.is_file(), "published entry must be a regular file");
    assert_eq!(fs::read(&destination).unwrap(), b"validated bytes");
    fs::remove_dir_all(parent).unwrap();
}

/// After a successful write the destination directory keeps no staging
/// leftovers: the staging directory is removed and no temp name is exposed.
#[test]
fn destination_directory_keeps_no_staging_leftovers() {
    let (parent, root) = staging_fixture("staged-leftovers");
    let destination = root.join("planned.txt");

    sea_forge_sandbox::safe_write(&destination, b"payload").unwrap();
    sea_forge_sandbox::safe_write(&destination, b"payload again").unwrap();

    let names: Vec<String> = fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["planned.txt".to_string()]);
    assert_eq!(fs::read(&destination).unwrap(), b"payload again");
    fs::remove_dir_all(parent).unwrap();
}
