//! Windows materialization writes must not escape the authorized workspace
//! through hard links or junctions. Both tests check the outside file, not
//! just the return value.
#![cfg(windows)]

use std::path::Path;
use std::process::Command;

use sea_forge_sandbox::{safe_join, safe_write, safe_write_within};

/// Creates a junction `link` to `target` with `mklink /J`, which needs no
/// elevation. Returns false only if the junction could not be made, and then
/// fails the test when SEA_FORGE_REQUIRE_LINK_PRIVILEGE=1.
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
    if std::env::var("SEA_FORGE_REQUIRE_LINK_PRIVILEGE").as_deref() == Ok("1") {
        panic!("BLOCKED prerequisite, not a pass: mklink /J failed: {reason}");
    }
    println!("SKIPPED: mklink /J failed: {reason}");
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

    let result = safe_write(&planned, b"overwritten");
    assert!(result.is_err(), "hard-linked destination was written");
    assert_eq!(
        std::fs::read(&outside).expect("read outside"),
        b"outside original",
        "the outside hard-link target was truncated or rewritten"
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
