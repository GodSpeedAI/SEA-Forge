//! Resource observation relative to an already opened, trusted workspace.
use sea_forge_core::errors::ForgeError;
use std::fs::File;
use std::path::Path;

#[cfg(target_os = "linux")]
fn open_child(parent: &File, name: &std::ffi::OsStr, directory: bool) -> Result<File, ForgeError> {
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::OpenOptionsExt;
    let path = Path::new("/proc/self/fd")
        .join(parent.as_raw_fd().to_string())
        .join(name);
    let mut flags = libc::O_NOFOLLOW | libc::O_NONBLOCK;
    if directory {
        flags |= libc::O_DIRECTORY;
    }
    std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(flags)
        .open(path)
        .map_err(|e| ForgeError::io("open scoped observation component", e))
}

/// Pin an absolute workspace without following symlinks in any component.
/// The host mount namespace and its root are trusted. Replacement of a name
/// after opening it cannot redirect the remainder of this descriptor walk.
#[cfg(target_os = "linux")]
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
            Component::Normal(name) => directory = open_child(&directory, name, true)?,
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
/// Linux procfs resolves each pinned parent descriptor; each untrusted name
/// is the final component of an O_NOFOLLOW open. Renaming an ancestor cannot
/// redirect subsequent opens. This reads effects; it grants no workload access.
#[cfg(target_os = "linux")]
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
        let file = open_child(&parent, name, !final_component)?;
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

#[cfg(not(target_os = "linux"))]
pub fn open_observation_root(_path: &Path) -> Result<File, ForgeError> {
    Err(ForgeError::Input(
        "scoped file observation requires Linux procfs".into(),
    ))
}

#[cfg(not(target_os = "linux"))]
pub fn open_observation_file(_root: &File, _relative: &str) -> Result<File, ForgeError> {
    Err(ForgeError::Input(
        "scoped file observation requires Linux procfs".into(),
    ))
}
