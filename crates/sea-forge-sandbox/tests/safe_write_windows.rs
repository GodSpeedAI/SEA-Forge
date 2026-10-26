//! Windows materialization writes must not escape the authorized workspace
//! through hard links or junctions. Both tests check the outside file, not
//! just the return value.
#![cfg(windows)]

use std::path::Path;
use std::process::Command;

use sea_forge_sandbox::{safe_join, safe_write, safe_write_within};

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

/// While the workspace root is pinned (`PinnedDir::open` now takes an
/// exclusive FILE_SHARE_READ-only pin), every write-class open of the pinned
/// directory fails with a sharing violation. That is the same gate an
/// FSCTL_SET_REPARSE_POINT junction planting would have to pass: the second
/// handle cannot even be opened, so no reparse point can be set on the pinned
/// directory mid-operation. The outside target must stay empty, and a
/// contained write after the pin is released must land inside the workspace.
#[test]
fn junction_planting_in_a_pinned_directory_is_refused_and_the_target_stays_empty() {
    let outside = tempfile::tempdir().expect("outside");
    let workspace = tempfile::tempdir().expect("workspace");
    let root = workspace.path();

    let pin = sea_forge_os::PinnedDir::open(root).expect("pin the workspace root");

    // A direct entry creation inside the pinned directory (the kernel opens
    // the parent for FILE_ADD_SUBDIRECTORY, a write-class access).
    let planted = root.join("planted");
    assert!(
        std::fs::create_dir(&planted).is_err(),
        "a directory was created inside the exclusively pinned directory"
    );
    // The junction flavor: `mklink /J` creates a reparse-point entry in the
    // pinned directory. The refusal here is the asserted enforcement, not a
    // blocked prerequisite, so the make_junction BLOCKED/REQUIRE tally does
    // not apply; a junction that did get created fails the empty-outside
    // check below.
    let attempt = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(root.join("link"))
        .arg(outside.path())
        .output()
        .expect("run cmd");
    assert!(
        !attempt.status.success(),
        "a junction was planted inside the exclusively pinned directory"
    );
    assert_eq!(
        std::fs::read_dir(outside.path())
            .expect("read outside")
            .count(),
        0,
        "the outside junction target received an entry from the refused planting"
    );

    drop(pin);
    let path = safe_join(root, "file.txt").expect("join");
    safe_write_within(root, &path, b"contained").expect("contained write after release");
    assert_eq!(
        std::fs::read(&path).expect("read"),
        b"contained",
        "the contained write did not land inside the workspace"
    );
}
