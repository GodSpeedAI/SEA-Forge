//! Bundle reads are bounded (§14.8 untrusted bundle bytes).
//!
//! `import`/`read_manifest` hold the whole bundle in memory and every hash
//! check runs *after* the read, so an unbounded read exhausts memory before
//! the integrity checks that would reject the file can run. These tests pin
//! the ceiling against the real production constant — they fail if the bound
//! is removed or raised without a deliberate edit here.

use sea_forge_cell as cell;
use sea_forge_core::errors::ForgeError;
use std::fs::File;

/// Must match `MAX_BUNDLE_BYTES` in `bundle.rs`.
const MAX_BUNDLE_BYTES: u64 = 256 * 1024 * 1024;

/// A sparse file of `len` bytes: `set_len` allocates no blocks, so this is
/// instant and costs no disk, while still presenting `len` readable bytes.
fn sparse_file(path: &std::path::Path, len: u64) -> std::io::Result<()> {
    let file = File::create(path)?;
    file.set_len(len)?;
    Ok(())
}

fn assert_integrity_rejection(error: ForgeError) {
    assert_eq!(
        error.class(),
        "bundle_integrity_error",
        "oversized bundle must be rejected as an integrity error, got: {error}"
    );
    let message = error.to_string();
    assert!(
        message.contains("limit"),
        "rejection should name the limit, got: {message}"
    );
}

#[test]
fn import_rejects_a_bundle_over_the_size_limit() {
    let dir = tempfile::tempdir().unwrap();
    let bundle = dir.path().join("oversized.tar");
    sparse_file(&bundle, MAX_BUNDLE_BYTES + 1).unwrap();

    let root = dir.path().join("root");
    std::fs::create_dir_all(&root).unwrap();

    let error = cell::import(&root, &bundle)
        .expect_err("import must reject a bundle larger than the size limit");
    assert_integrity_rejection(error);

    // Fail closed: nothing was staged or imported before the rejection.
    assert!(
        !root.join(".sea-forge/imported").exists(),
        "a rejected bundle must leave no imported state behind"
    );
}

#[test]
fn read_manifest_rejects_a_bundle_over_the_size_limit() {
    let dir = tempfile::tempdir().unwrap();
    let bundle = dir.path().join("oversized.tar");
    sparse_file(&bundle, MAX_BUNDLE_BYTES + 1).unwrap();

    let error = cell::read_manifest(&bundle)
        .expect_err("read_manifest must reject a bundle larger than the size limit");
    assert_integrity_rejection(error);
}

#[test]
fn a_bundle_at_the_limit_is_still_read() {
    // The bound must reject only what exceeds it. A file exactly at the ceiling
    // is read (and then fails later, on its own merits, as a malformed tar) —
    // proving the limit is not off by one in the rejecting direction.
    let dir = tempfile::tempdir().unwrap();
    // The name must not contain "limit" — the assertion below inspects a
    // message that embeds this path.
    let bundle = dir.path().join("boundary.tar");
    sparse_file(&bundle, MAX_BUNDLE_BYTES).unwrap();

    let error = cell::read_manifest(&bundle)
        .expect_err("an all-zero tar has no manifest.json and must still fail");
    // Reaching the tar-parsing stage is the proof: the size gate let it past.
    assert!(
        error.to_string().contains("missing manifest.json"),
        "a bundle exactly at the limit must be read, then fail on its own \
         merits as a malformed tar, got: {error}"
    );
}
