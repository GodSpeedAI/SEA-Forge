#![cfg(unix)]
// Any Unix: Linux walks procfs, other Unix walks openat; both must hold.
#![cfg(unix)]

use sea_forge_sandbox::{open_observation_file, open_observation_root};
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::symlink;

#[test]
fn observation_uses_the_opened_workspace_and_refuses_links_and_escape() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    fs::create_dir_all(workspace.join("nested")).unwrap();
    fs::write(workspace.join("nested/result"), b"approved effect").unwrap();
    fs::write(temp.path().join("private"), b"outside workspace").unwrap();
    let root = File::open(&workspace).unwrap();
    symlink(temp.path().join("private"), workspace.join("link")).unwrap();
    symlink(temp.path(), workspace.join("parent-link")).unwrap();
    for path in [
        "../private",
        "/etc/passwd",
        "link",
        "parent-link/private",
        "nested",
    ] {
        assert!(open_observation_file(&root, path).is_err(), "{path}");
    }

    // Renaming/replacing the pathname must not redirect the pinned root.
    fs::rename(&workspace, temp.path().join("original-workspace")).unwrap();
    fs::create_dir_all(workspace.join("nested")).unwrap();
    fs::write(workspace.join("nested/result"), b"replacement").unwrap();
    let mut result = String::new();
    open_observation_file(&root, "nested/result")
        .unwrap()
        .read_to_string(&mut result)
        .unwrap();
    assert_eq!(result, "approved effect");
}

#[test]
fn observation_root_refuses_symlinks_in_every_component() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    fs::create_dir_all(workspace.join("nested")).unwrap();
    assert!(open_observation_root(&workspace).is_ok());
    let alias = temp.path().join("alias");
    symlink(&workspace, &alias).unwrap();
    assert!(open_observation_root(&alias).is_err());
    assert!(open_observation_root(&alias.join("nested")).is_err());
    assert!(open_observation_root(std::path::Path::new("workspace")).is_err());
    assert!(open_observation_root(&workspace.join("../workspace")).is_err());
}

/// A symlink the host configured (root-owned, in a root-only directory) is
/// part of the trusted namespace, like macOS `/var` or Linux `/var/run`.
#[test]
fn host_configured_symlinks_resolve_in_the_root_prefix() {
    use std::os::unix::fs::MetadataExt;
    let Some(link) = ["/var/run", "/var", "/tmp", "/lib", "/bin"]
        .into_iter()
        .map(std::path::Path::new)
        .find(|p| {
            fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink() && m.uid() == 0)
                && fs::metadata(p).is_ok_and(|m| m.is_dir())
        })
    else {
        eprintln!("NOT-EXECUTABLE: no root-owned directory symlink on this host");
        return;
    };
    assert!(open_observation_root(link).is_ok(), "{}", link.display());
}
