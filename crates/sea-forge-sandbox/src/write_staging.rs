//! Protected staging directory for Unix safe writes (F32).
//!
//! The temp file for a safe write is created inside `.sf-staging-<pid>` (mode
//! 0700), a child of the destination directory, and published with
//! `renameat(staging, temp, destination_dir, name)`. Because the rename
//! SOURCE pathname resolves through the staging descriptor, a process that
//! can write the destination directory has no name it could unlink or
//! replant to redirect the rename: the published bytes always come from the
//! inode this process created. The temp file never appears in the
//! destination directory under any name.
//!
//! The staging directory is a child of the destination directory (not of its
//! parent) so that (a) it sits on the same filesystem as the destination —
//! `renameat` cannot cross mounts, and EXDEV is a loud refusal, never a
//! fallback — and (b) every byte this module writes stays inside the subtree
//! the caller authorized, never in a parent it did not open.
//!
//! Residual: mode bits cannot separate a same-uid attacker. A workload that
//! runs as the same host uid AND holds a write grant over the destination
//! tree (the Linux jail's Landlock grants, the macOS jail's app-sandbox
//! exceptions) can traverse the 0700 staging directory and race the temp
//! inside it; the dev/ino post-check in `write_entry_unix` stays
//! detection-only for that case. Workloads with a distinct identity (the
//! isolated backend's subordinate uid) cannot traverse the staging directory
//! at all, which turns the race into a prevention.
//!
//! The directory is created per call and removed after publish. The name is
//! per-call (`.sf-staging-<pid>-<sequence>`), so concurrent writes from this
//! process to the same destination directory never share one: a writer that
//! has opened its staging dir but not yet created its temp can never have
//! that dir removed under it (review pass 1, Medium-11). The `EEXIST` reuse
//! branch covers only a recycled pid colliding with a genuine leftover;
//! the identity check still proves the opened entry is the created one.

use std::ffi::{OsStr, OsString};
use std::io;
use std::os::fd::OwnedFd;
use std::sync::atomic::{AtomicU64, Ordering};

use rustix::fs::{AtFlags, Mode, OFlags};
use rustix::io::Errno;

/// Destination-directory name prefix of the staging directory (tests scan
/// for it).
pub(crate) const STAGING_PREFIX: &str = ".sf-staging-";

/// Why a staging directory could not be opened for use.
pub(crate) enum StagingCreateError {
    /// The entry at the staging name is not the directory this call opened —
    /// a symlink or directory rebind. Refused loudly, never retried or
    /// worked around.
    Rebind,
    /// Anything else (permissions, filesystem, ...).
    Io(io::Error),
}

/// A private 0700 directory used only as the rename SOURCE for one safe
/// write. It is a child of the destination directory; `dir` arguments below
/// always mean that destination directory.
pub(crate) struct StagingDir {
    fd: OwnedFd,
    name: OsString,
}

impl StagingDir {
    /// Creates (or reuses) the staging directory inside `dir` and returns a
    /// descriptor to it. The mode is re-applied on the descriptor so a umask
    /// cannot lower it.
    pub(crate) fn create(dir: &OwnedFd) -> Result<Self, StagingCreateError> {
        // Per-call name: two concurrent writers never share a staging dir,
        // so one writer's post-publish removal can never delete the other's
        // open staging directory (Medium-11). The sequence counter makes the
        // collision window a recycled pid AND the same sequence.
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let name = OsString::from(format!(
            "{STAGING_PREFIX}{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        match rustix::fs::mkdirat(dir, &name, Mode::RUSR | Mode::WUSR | Mode::XUSR) {
            Ok(()) => {}
            // A leftover under a recycled pid with the same sequence: the
            // identity check below proves that the entry at this name is the
            // directory being opened, and remove() after publish clears it.
            Err(Errno::EXIST) => {}
            Err(error) => return Err(StagingCreateError::Io(io::Error::from(error))),
        }
        let fd = rustix::fs::openat(
            dir,
            &name,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| match error {
            // A symlink (or non-directory) at the staging name.
            Errno::LOOP | Errno::NOTDIR => StagingCreateError::Rebind,
            error => StagingCreateError::Io(io::Error::from(error)),
        })?;
        rustix::fs::fchmod(&fd, Mode::RUSR | Mode::WUSR | Mode::XUSR)
            .map_err(|error| StagingCreateError::Io(io::Error::from(error)))?;
        // The directory this call uses must be the entry at `name`: a
        // concurrent rename-away-and-replant between the mkdir and this open
        // would otherwise hand the write an attacker-controlled directory.
        let opened = rustix::fs::fstat(&fd)
            .map_err(|error| StagingCreateError::Io(io::Error::from(error)))?;
        let entry = rustix::fs::statat(dir, &name, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|error| StagingCreateError::Io(io::Error::from(error)))?;
        if entry.st_dev != opened.st_dev || entry.st_ino != opened.st_ino {
            return Err(StagingCreateError::Rebind);
        }
        Ok(Self { fd, name })
    }

    /// Creates a fresh exclusive temp file for destination entry `name`
    /// inside the staging directory (never in the destination directory).
    /// The mode of an existing destination entry is kept; a new file gets
    /// 0666 minus the umask.
    pub(crate) fn new_temp(
        &self,
        name: &OsStr,
        // RawMode is u32 on Linux, u16 on macOS; callers hold platform values.
        mode: impl Into<u32>,
    ) -> io::Result<(std::fs::File, OsString)> {
        let mode: u32 = mode.into();
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let mut attempts = 0;
        loop {
            let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let mut candidate = OsString::from(".");
            candidate.push(name);
            candidate.push(format!(".sf-tmp-{}-{sequence}", std::process::id()));
            match rustix::fs::openat(
                &self.fd,
                candidate.as_os_str(),
                // RDWR, not WRONLY (review pass 2, R9): the content verifier
                // reads the published bytes back through this RETAINED
                // descriptor, so a destination whose preserved mode denies
                // read access (0200) still materializes, and the verifier
                // performs no pathname reopen at all (R1). Creation grants
                // the requested access regardless of the new file's mode; the
                // mode only governs later openers.
                OFlags::RDWR | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                // RawMode is u32 on Linux, u16 on macOS; callers hand us u32.
                Mode::from_raw_mode(mode as _),
            ) {
                Ok(fd) => return Ok((std::fs::File::from(fd), candidate)),
                Err(Errno::EXIST) if attempts < 16 => attempts += 1,
                Err(error) => return Err(io::Error::from(error)),
            }
        }
    }

    /// Publishes the temp over `dest_name` in `dest_dir`. The source pathname
    /// resolves through the staging descriptor, so a writer of the
    /// destination directory cannot substitute it.
    pub(crate) fn publish(
        &self,
        temp: &OsStr,
        dest_dir: &OwnedFd,
        dest_name: &OsStr,
    ) -> io::Result<()> {
        match rustix::fs::renameat(&self.fd, temp, dest_dir, dest_name) {
            Ok(()) => Ok(()),
            // Same-filesystem is a design invariant (the staging directory is
            // a child of the destination directory), so EXDEV means that
            // invariant was broken. Refuse loudly; never fall back to a
            // destination-directory temp.
            Err(Errno::XDEV) => Err(io::Error::other(
                "staged rename crossed filesystems; the staging directory must be on the same filesystem as the destination",
            )),
            Err(error) => Err(io::Error::from(error)),
        }
    }

    /// Best-effort cleanup of a failed temp file. A failure here only
    /// surfaces through [`StagingDir::remove`] (`ENOTEMPTY`).
    pub(crate) fn discard_temp(&self, temp: &OsStr) {
        let _ = rustix::fs::unlinkat(&self.fd, temp, AtFlags::empty());
    }

    /// Removes the staging directory once its temp is gone. Best-effort and
    /// logged, never fatal.
    pub(crate) fn remove(self, dir: &OwnedFd) {
        if let Err(error) = rustix::fs::unlinkat(dir, &self.name, AtFlags::REMOVEDIR) {
            eprintln!(
                "[write-staging] failed to remove staging directory {:?}: {error}",
                self.name
            );
        }
    }
}
