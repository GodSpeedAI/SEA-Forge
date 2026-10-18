//! Resource observation relative to an already opened, trusted workspace.
use sea_forge_core::errors::ForgeError;
use std::fs::File;
use std::path::Path;

#[cfg(target_os = "linux")]
fn open_child(
    parent: &File,
    name: &std::ffi::OsStr,
    directory: bool,
    follow: bool,
) -> Result<File, ForgeError> {
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::OpenOptionsExt;
    let path = Path::new("/proc/self/fd")
        .join(parent.as_raw_fd().to_string())
        .join(name);
    let mut flags = libc::O_NONBLOCK;
    if !follow {
        flags |= libc::O_NOFOLLOW;
    }
    if directory {
        flags |= libc::O_DIRECTORY;
    }
    std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(flags)
        .open(path)
        .map_err(|e| ForgeError::io("open scoped observation component", e))
}

/// Other Unix: open `name` relative to the pinned parent descriptor
/// (`openat`) with `O_NOFOLLOW`, the same guarantee as the procfs walk above:
/// each untrusted name is the final component of a no-follow open.
#[cfg(all(unix, not(target_os = "linux")))]
fn open_child(
    parent: &File,
    name: &std::ffi::OsStr,
    directory: bool,
    follow: bool,
) -> Result<File, ForgeError> {
    use rustix::fs::{openat, Mode, OFlags};
    let mut flags = OFlags::RDONLY | OFlags::NONBLOCK | OFlags::CLOEXEC;
    if !follow {
        flags |= OFlags::NOFOLLOW;
    }
    if directory {
        flags |= OFlags::DIRECTORY;
    }
    openat(parent, name, flags, Mode::empty())
        .map(File::from)
        .map_err(|e| ForgeError::io("open scoped observation component", e.into()))
}

/// A symlink the host itself configured: owned by root, in a directory only
/// root can change (macOS `/var` -> `/private/var`, Linux `/var/run`). It is
/// part of the trusted host namespace; every other symlink is refused.
#[cfg(unix)]
fn host_symlink(parent: &File, name: &std::ffi::OsStr) -> bool {
    use rustix::fs::{fstat, statat, AtFlags, FileType};
    let (Ok(link), Ok(dir)) = (
        statat(parent, name, AtFlags::SYMLINK_NOFOLLOW),
        fstat(parent),
    ) else {
        return false;
    };
    FileType::from_raw_mode(link.st_mode as _) == FileType::Symlink
        && link.st_uid == 0
        && dir.st_uid == 0
        && dir.st_mode & 0o022 == 0
}

/// Pin an absolute workspace without following any symlink the host did
/// not configure itself. The host mount namespace and its root are trusted.
/// Replacement of a name after opening it cannot redirect the remainder of
/// this descriptor walk.
#[cfg(unix)]
pub fn open_observation_root(path: &Path) -> Result<File, ForgeError> {
    use std::path::Component;
    if !path.is_absolute() {
        return Err(ForgeError::UnsafePath(
            "observation root must be absolute".into(),
        ));
    }
    let mut directory = File::open("/").map_err(|e| ForgeError::io("pin host root", e))?;
    for component in path.components() {
        match component {
            Component::RootDir => {}
            Component::Normal(name) => {
                let follow = host_symlink(&directory, name);
                directory = open_child(&directory, name, true, follow)?
            }
            _ => {
                return Err(ForgeError::UnsafePath(
                    "observation root is not normalized".into(),
                ))
            }
        }
    }
    Ok(directory)
}

/// Open a regular file without following any workload-controlled symlink.
/// Each pinned parent descriptor resolves the next name (procfs on Linux,
/// openat elsewhere); each untrusted name
/// is the final component of an O_NOFOLLOW open. Renaming an ancestor cannot
/// redirect subsequent opens. This reads effects; it grants no workload access.
#[cfg(unix)]
pub fn open_observation_file(root: &File, relative: &str) -> Result<File, ForgeError> {
    use std::path::Component;

    crate::validate_relative_path(relative)?;
    let components: Vec<_> = Path::new(relative).components().collect();
    if components.is_empty() || !root.metadata().is_ok_and(|m| m.is_dir()) {
        return Err(ForgeError::UnsafePath(
            "observation requires a directory root and file".into(),
        ));
    }
    let mut parent = root
        .try_clone()
        .map_err(|e| ForgeError::io("pin observation root", e))?;
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(name) = component else {
            return Err(ForgeError::UnsafePath(
                "observation path is not normalized".into(),
            ));
        };
        let final_component = index + 1 == components.len();
        // Inside the workspace nothing is followed, host-owned or not.
        let file = open_child(&parent, name, !final_component, false)?;
        if final_component {
            if !file.metadata().is_ok_and(|m| m.is_file()) {
                return Err(ForgeError::UnsafePath(
                    "observation target is not a regular file".into(),
                ));
            }
            return Ok(file);
        }
        parent = file;
    }
    Err(ForgeError::UnsafePath("observation requires a file".into()))
}

#[cfg(not(unix))]
pub fn open_observation_root(_path: &Path) -> Result<File, ForgeError> {
    Err(ForgeError::Input(
        "scoped file observation requires a Unix descriptor walk".into(),
    ))
}

#[cfg(not(unix))]
pub fn open_observation_file(_root: &File, _relative: &str) -> Result<File, ForgeError> {
    Err(ForgeError::Input(
        "scoped file observation requires a Unix descriptor walk".into(),
    ))
}
