//! The Windows named-pipe name for a control-plane socket path.
//!
//! Server and client derive the same name from the same socket path, so the
//! path configured for the cell is the only input an operator needs. Unix
//! keeps using the socket file itself and never calls this.

use sha2::{Digest, Sha256};
use std::io;
use std::path::Path;

/// `\\.\pipe\sea-forge-<32 hex chars>`, keyed by the canonical socket path.
///
/// The parent directory is canonicalized (the socket file may not exist yet),
/// so `./x/server.sock` and its absolute form map to one pipe.
pub fn pipe_name_for(socket_path: &Path) -> io::Result<String> {
    let parent = socket_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = socket_path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "socket path has no file name")
    })?;
    let canonical = parent.canonicalize()?.join(file_name);
    let key = canonical.to_str().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "socket path is not valid UTF-8")
    })?;
    Ok(pipe_name_for_canonical(key))
}

/// The pipe name for an already-canonical socket path. Split out so the
/// derivation is pinned by a test that needs no file system.
pub(crate) fn pipe_name_for_canonical(key: &str) -> String {
    let digest = hex::encode(Sha256::digest(key.as_bytes()));
    format!(r"\\.\pipe\sea-forge-{}", &digest[..32])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_stable_and_well_formed() {
        let dir = std::env::temp_dir();
        let path = dir.join("sea-forge-pipe-name-test.sock");
        let first = pipe_name_for(&path).expect("name");
        let second = pipe_name_for(&path).expect("name");
        assert_eq!(first, second);
        let suffix = first.strip_prefix(r"\\.\pipe\sea-forge-").expect("prefix");
        assert_eq!(suffix.len(), 32);
        assert!(suffix.bytes().all(|b| b.is_ascii_hexdigit()));
    }

    /// Pins the derivation the identity E2E script recomputes
    /// (`scripts/windows/identity-e2e.ps1`): SHA-256 of the verbatim canonical
    /// path, first 32 hex digits. The digest was computed independently.
    #[test]
    fn derivation_is_pinned_for_a_verbatim_path() {
        assert_eq!(
            pipe_name_for_canonical(r"\\?\C:\sfe2e\cell\server.sock"),
            r"\\.\pipe\sea-forge-6b544457d9e2211fbc18db682120eec7"
        );
    }

    #[test]
    fn distinct_paths_get_distinct_names() {
        let dir = std::env::temp_dir();
        let a = pipe_name_for(&dir.join("a.sock")).expect("name");
        let b = pipe_name_for(&dir.join("b.sock")).expect("name");
        assert_ne!(a, b);
    }
}
