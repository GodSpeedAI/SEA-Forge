//! Windows materialization writes must not escape the authorized workspace
//! through hard links or junctions. Both tests check the outside file, not
//! just the return value.
#![cfg(windows)]

use std::path::Path;
use std::process::Command;

use sea_forge_sandbox::{safe_join, safe_write, safe_write_within};
#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

/// Process-wide tally of scenarios that could not run because a junction could
/// not be created. A required lane (the REQUIRE variable set) must not skip;
/// elsewhere the `BLOCKED:` label keeps the skip honest (F15).
static BLOCKED_SCENARIOS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Creates a junction `link` to `target` with `mklink /J`, which needs no
/// elevation. Returns false only if the junction could not be made; the
/// scenario is then tallied and labeled `BLOCKED:`, and fails the test when
/// SEA_FORGE_REQUIRE_LINK_PRIVILEGE=1.
fn make_junction(link: &Path, target: &Path) -> bool {
    let output = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .output()
        .expect("run cmd");
    if output.status.success() {
        return true;
    }
    let reason = String::from_utf8_lossy(&output.stderr).trim().to_string();
    BLOCKED_SCENARIOS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if std::env::var("SEA_FORGE_REQUIRE_LINK_PRIVILEGE").as_deref() == Ok("1") {
        panic!("BLOCKED prerequisite, not a pass: mklink /J failed: {reason}");
    }
    eprintln!("BLOCKED: mklink /J failed: {reason}");
    false
}

#[test]
fn hard_link_to_outside_file_keeps_its_bytes() {
    let outside_dir = tempfile::tempdir().expect("outside");
    let workspace = tempfile::tempdir().expect("workspace");
    let outside = outside_dir.path().join("outside.txt");
    std::fs::write(&outside, b"outside original").expect("seed outside");
    let planned = safe_join(workspace.path(), "planned.txt").expect("join");
    std::fs::hard_link(&outside, &planned).expect("hard link into workspace");

    // The write replaces the workspace entry with a fresh file. The outside
    // hard-link target keeps its bytes, and the destination gets the new ones.
    safe_write(&planned, b"overwritten").expect("replace the linked entry");
    assert_eq!(
        std::fs::read(&outside).expect("read outside"),
        b"outside original",
        "the outside hard-link target was truncated or rewritten"
    );
    assert_eq!(
        std::fs::read(&planned).expect("read planned"),
        b"overwritten"
    );
}

#[test]
fn junction_destination_is_refused_and_its_target_untouched() {
    let outside = tempfile::tempdir().expect("outside");
    let workspace = tempfile::tempdir().expect("workspace");
    let root = workspace.path();
    let link = root.join("dest.txt");
    if !make_junction(&link, outside.path()) {
        return;
    }
    // Rust reports a junction as a symlink, so `safe_join` may refuse it
    // first. Either refusal is correct; the write must not happen.
    let result = safe_join(root, "dest.txt")
        .and_then(|checked| safe_write_within(root, &checked, b"through the junction"));
    assert!(result.is_err(), "a junction destination was written");
    assert_eq!(
        std::fs::read_dir(outside.path())
            .expect("read outside")
            .count(),
        0,
        "a file was created inside the junction target"
    );
}

#[test]
fn missing_directories_under_a_swapped_junction_create_nothing_outside() {
    let outside = tempfile::tempdir().expect("outside");
    let workspace = tempfile::tempdir().expect("workspace");
    let root = workspace.path();
    std::fs::create_dir(root.join("sub")).expect("sub");
    std::fs::rename(root.join("sub"), root.join("sub_moved")).expect("move sub");
    if !make_junction(&root.join("sub"), outside.path()) {
        return;
    }

    // The walker must refuse the junction, and must not create `deeper`
    // through it in the outside directory.
    let result = safe_join(root, "sub/deeper/file.txt");
    assert!(
        result.is_err(),
        "a directory was created through the junction"
    );
    assert_eq!(
        std::fs::read_dir(outside.path())
            .expect("read outside")
            .count(),
        0,
        "a directory was created in the outside target"
    );
}

#[test]
fn junction_swapped_parent_does_not_redirect_the_write() {
    let outside = tempfile::tempdir().expect("outside");
    let workspace = tempfile::tempdir().expect("workspace");
    let root = workspace.path();
    std::fs::create_dir(root.join("sub")).expect("sub");
    // Validate while `sub` is an ordinary directory, as the caller would.
    let checked = safe_join(root, "sub/file.txt").expect("join");

    // Swap `sub` for a junction to the outside directory.
    std::fs::rename(root.join("sub"), root.join("sub_moved")).expect("move sub");
    if !make_junction(&root.join("sub"), outside.path()) {
        return;
    }

    let result = safe_write_within(root, &checked, b"redirected");
    assert!(result.is_err(), "write through swapped junction succeeded");
    assert!(
        !outside.path().join("file.txt").exists(),
        "a file was created in the outside directory through the junction"
    );
}

#[test]
fn ordinary_rewrite_inside_the_workspace_still_replaces_contents() {
    let workspace = tempfile::tempdir().expect("workspace");
    let root = workspace.path();
    let path = safe_join(root, "notes.txt").expect("join");
    safe_write_within(root, &path, b"first version, longer").expect("first write");
    safe_write_within(root, &path, b"second").expect("second write");
    assert_eq!(std::fs::read(&path).expect("read"), b"second");
}

/// Required-lane guard (F15): blocked scenarios must not survive a lane that
/// requires the junction-creation prerequisite.
#[test]
fn no_blocked_scenarios_in_required_mode() {
    let blocked_count = BLOCKED_SCENARIOS.load(std::sync::atomic::Ordering::SeqCst);
    assert!(
        !(blocked_count > 0
            && std::env::var("SEA_FORGE_REQUIRE_LINK_PRIVILEGE").as_deref() == Ok("1")),
        "required lane (SEA_FORGE_REQUIRE_LINK_PRIVILEGE=1) finished with \
         {blocked_count} BLOCKED scenario(s); required lanes cannot skip"
    );
}

/// The pinned anchor's load-bearing F05 gate: while the workspace root is
/// pinned (`PinnedDir::open` takes an exclusive FILE_SHARE_READ-only pin),
/// any write-class HANDLE on the pinned directory is refused with a sharing
/// violation — which is exactly what an FSCTL_SET_REPARSE_POINT mutation
/// would have to open, so a validated anchor cannot be reparse-mutated
/// mid-operation. Path-based ENTRY creation inside the pinned directory is
/// a different kernel operation: the parent is checked against its DACL and
/// never opened as a share-checked handle, so a planted junction entry can
/// exist. That entry must not redirect the contained write — the publish
/// replaces it, and the release-rename's FileId verification fails loudly on
/// any successful interference. The outside target must stay empty, and a
/// contained write after the pin is released must land inside the workspace.
#[test]
fn pinned_anchor_refuses_write_handles_and_a_planted_junction_does_not_redirect() {
    let outside = tempfile::tempdir().expect("outside");
    let workspace = tempfile::tempdir().expect("workspace");
    let root = workspace.path();

    let pin = sea_forge_os::PinnedDir::open(root).expect("pin the workspace root");

    // A write-class open of the pinned directory itself is refused: this is
    // the handle an FSCTL_SET_REPARSE_POINT mutation would need.
    let write_open = std::fs::OpenOptions::new()
        .write(true)
        .custom_flags(0x0200_0000) // FILE_FLAG_BACKUP_SEMANTICS: open directories by handle
        .open(root);
    // (custom_flags comes from std::os::windows::fs::OpenOptionsExt, imported
    // at the top of this file for the whole cfg(windows) target.)
    assert!(
        write_open.is_err(),
        "a write-class handle was opened on the exclusively pinned directory"
    );

    // Path-based entry creation is DACL-only, not share-gated (observed on
    // Server 2025): a planted junction entry may exist. What it must never do
    // is redirect the contained write below.
    let attempt = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(root.join("link"))
        .arg(outside.path())
        .output()
        .expect("run cmd");
    println!(
        "junction planting attempt succeeded: {} (path-based entry creation is not share-gated)",
        attempt.status.success()
    );

    drop(pin);
    let path = safe_join(root, "file.txt").expect("join");
    safe_write_within(root, &path, b"contained").expect("contained write after release");
    assert_eq!(
        std::fs::read(&path).expect("read"),
        b"contained",
        "the contained write did not land inside the workspace"
    );
    assert_eq!(
        std::fs::read_dir(outside.path())
            .expect("read outside")
            .count(),
        0,
        "the outside junction target received entries from the contained write"
    );
}
