//! Launcher-owned artifact output files.
//!
//! The workload controls the entries under `artifacts/` between executions,
//! so a planted symlink or hard link there must never redirect a write to a
//! file outside the sandbox. Every backend creates `stdout.txt`/`stderr.txt`
//! through [`create_output_file`], which removes whatever entry is present
//! without following it and then creates a fresh file exclusively.

use std::{
    fs::{self, File, OpenOptions},
    io,
    path::Path,
};

/// Creates a fresh, empty, writable output file at `path`.
///
/// Any existing entry is unlinked rather than opened: a symlink is removed
/// without touching its target, and a hard link only drops this name, so the
/// other links keep their bytes. The file is then created with `create_new`
/// (and `O_NOFOLLOW` on Unix), so a racing entry cannot be followed either.
pub(crate) fn create_output_file(path: &Path) -> io::Result<File> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "artifact output path is a directory",
            ));
        }
        Ok(_) => fs::remove_file(path)?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    options.open(path)
}
