//! Sealed summarized-mode transcript storage (M13 T16, spec §7.4, ADR-002).
//!
//! `spec-agent-orchestration.md` §7.4 requires summarized-mode retention to
//! be independently verifiable rather than a bare digest+summary that can
//! never be recomputed. ADR-002 selects `XChaCha20Poly1305` (RustCrypto
//! `chacha20poly1305`, already workspace-approved) for this: seal the exact
//! canonical redacted transcript bytes with a fresh per-run key, store the
//! key at `<root>/sealed/<run_id>.key` (outside the public `full`
//! artifact surface), and verify by decrypting immediately after sealing.
//! Deleting the key file is the crypto-shred operation — the ciphertext
//! alone is permanently unrecoverable without it.

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    Key, XChaCha20Poly1305, XNonce,
};
use sea_forge_core::errors::ForgeError;
use sha2::{Digest, Sha256};
use std::io::Write as _;
use std::path::{Path, PathBuf};

const NONCE_LEN: usize = 24;
const KEY_LEN: usize = 32;

/// `getrandom` directly (the same OS-RNG source already used by
/// `sea_forge_core::ids`), rather than `chacha20poly1305`'s own `aead`
/// re-export chain — that chain's `rand_core`/`OsRng` surface has churned
/// across versions and is not the approved dependency here (ADR-002 approves
/// only `chacha20poly1305` itself for this crate).
fn random_bytes<const N: usize>() -> Result<[u8; N], ForgeError> {
    let mut bytes = [0u8; N];
    getrandom::fill(&mut bytes)
        .map_err(|error| ForgeError::Internal(format!("OS RNG failed: {error}")))?;
    Ok(bytes)
}

/// Paths of one sealed transcript's two artifacts.
pub struct SealedTranscript {
    pub key_path: PathBuf,
    pub ciphertext_path: PathBuf,
}

/// Seal `plaintext` for `run_id` under `root`. Writes a fresh random key to
/// `<root>/sealed/<run_id>.key` (mode 0600 on unix) and `nonce ||
/// ciphertext` to `ciphertext_path`. Returns before verification — callers
/// must call `verify_sealed_transcript` before treating the seal as durable
/// evidence (M13 T16 step 4: "verify before completion").
pub fn seal_transcript(
    root: &Path,
    run_id: &str,
    ciphertext_path: &Path,
    plaintext: &[u8],
) -> Result<SealedTranscript, ForgeError> {
    let key_bytes = random_bytes::<KEY_LEN>()?;
    let key = Key::try_from(key_bytes.as_slice())
        .map_err(|_| ForgeError::Internal("transcript seal key generation failed".into()))?;
    let cipher = XChaCha20Poly1305::new(&key);
    let nonce_bytes = random_bytes::<NONCE_LEN>()?;
    let nonce = XNonce::try_from(nonce_bytes.as_slice())
        .map_err(|_| ForgeError::Internal("transcript seal nonce generation failed".into()))?;
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| ForgeError::Internal("transcript seal failed".into()))?;

    // F-12: the server root is the state root; sealed keys join directly.
    let sealed_dir = root.join("sealed");
    std::fs::create_dir_all(&sealed_dir)
        .map_err(|e| ForgeError::io("create sealed transcript dir", e))?;
    let key_path = sealed_dir.join(format!("{run_id}.key"));
    write_private_file(&key_path, key.as_slice())?;

    if let Some(parent) = ciphertext_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| ForgeError::io("create sealed transcript parent", e))?;
    }
    let mut payload = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    payload.extend_from_slice(&nonce);
    payload.extend_from_slice(&ciphertext);
    std::fs::write(ciphertext_path, &payload)
        .map_err(|e| ForgeError::io("write sealed transcript", e))?;

    Ok(SealedTranscript {
        key_path,
        ciphertext_path: ciphertext_path.to_path_buf(),
    })
}

/// Decrypt and verify a previously sealed transcript against the expected
/// `sha256:<hex>` digest of its plaintext. Fails typed and closed on a
/// missing key (crypto-shredded), missing/tampered ciphertext, wrong key, or
/// a digest mismatch — never degrades to summary-only success (M13 T16
/// step 5).
pub fn verify_sealed_transcript(
    sealed: &SealedTranscript,
    expected_sha256: &str,
) -> Result<(), ForgeError> {
    let key_bytes = std::fs::read(&sealed.key_path).map_err(|_| {
        ForgeError::Input("sealed_verification_failed: key unavailable (crypto-shredded?)".into())
    })?;
    let key = Key::try_from(key_bytes.as_slice())
        .map_err(|_| ForgeError::Input("sealed_verification_failed: malformed key".into()))?;
    let cipher = XChaCha20Poly1305::new(&key);

    let payload = std::fs::read(&sealed.ciphertext_path).map_err(|_| {
        ForgeError::Input("sealed_verification_failed: ciphertext unavailable".into())
    })?;
    if payload.len() < NONCE_LEN {
        return Err(ForgeError::Input(
            "sealed_verification_failed: truncated payload".into(),
        ));
    }
    let (nonce_bytes, ciphertext) = payload.split_at(NONCE_LEN);
    let nonce = XNonce::try_from(nonce_bytes)
        .map_err(|_| ForgeError::Input("sealed_verification_failed: malformed nonce".into()))?;
    let plaintext = cipher.decrypt(&nonce, ciphertext).map_err(|_| {
        ForgeError::Input(
            "sealed_verification_failed: decryption failed (tampered or wrong key)".into(),
        )
    })?;

    let actual = format!("sha256:{:x}", Sha256::digest(&plaintext));
    if actual != expected_sha256 {
        return Err(ForgeError::Input(
            "sealed_verification_failed: digest mismatch".into(),
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), ForgeError> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .map_err(|e| ForgeError::io("write sealed transcript key", e))?;
    file.write_all(bytes)
        .map_err(|e| ForgeError::io("write sealed transcript key", e))
}

#[cfg(not(unix))]
fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), ForgeError> {
    std::fs::write(path, bytes).map_err(|e| ForgeError::io("write sealed transcript key", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(bytes: &[u8]) -> String {
        format!("sha256:{:x}", Sha256::digest(bytes))
    }

    #[test]
    fn round_trip_verifies() {
        let root = tempfile::tempdir().unwrap();
        let plaintext = b"transcript with a secret marker unleaked-xyz";
        let ciphertext_path = root
            .path()
            .join("runs")
            .join("r1")
            .join("transcript.sealed");
        let sealed = seal_transcript(root.path(), "r1", &ciphertext_path, plaintext).unwrap();
        verify_sealed_transcript(&sealed, &digest(plaintext)).unwrap();
    }

    #[test]
    fn ciphertext_never_contains_the_plaintext_marker() {
        let root = tempfile::tempdir().unwrap();
        let plaintext = b"transcript with a secret marker unleaked-xyz";
        let ciphertext_path = root
            .path()
            .join("runs")
            .join("r1")
            .join("transcript.sealed");
        seal_transcript(root.path(), "r1", &ciphertext_path, plaintext).unwrap();
        let raw = std::fs::read(&ciphertext_path).unwrap();
        assert!(!raw
            .windows(b"unleaked-xyz".len())
            .any(|w| w == b"unleaked-xyz"));
    }

    #[test]
    fn tampered_ciphertext_fails_verification() {
        let root = tempfile::tempdir().unwrap();
        let plaintext = b"transcript body";
        let ciphertext_path = root
            .path()
            .join("runs")
            .join("r1")
            .join("transcript.sealed");
        let sealed = seal_transcript(root.path(), "r1", &ciphertext_path, plaintext).unwrap();
        let mut bytes = std::fs::read(&ciphertext_path).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        std::fs::write(&ciphertext_path, &bytes).unwrap();
        assert!(verify_sealed_transcript(&sealed, &digest(plaintext)).is_err());
    }

    #[test]
    fn wrong_key_fails_verification() {
        let root = tempfile::tempdir().unwrap();
        let plaintext = b"transcript body";
        let ciphertext_path = root
            .path()
            .join("runs")
            .join("r1")
            .join("transcript.sealed");
        let sealed = seal_transcript(root.path(), "r1", &ciphertext_path, plaintext).unwrap();
        let wrong_key = random_bytes::<KEY_LEN>().unwrap();
        std::fs::write(&sealed.key_path, wrong_key).unwrap();
        assert!(verify_sealed_transcript(&sealed, &digest(plaintext)).is_err());
    }

    #[test]
    fn missing_key_fails_verification() {
        let root = tempfile::tempdir().unwrap();
        let plaintext = b"transcript body";
        let ciphertext_path = root
            .path()
            .join("runs")
            .join("r1")
            .join("transcript.sealed");
        let sealed = seal_transcript(root.path(), "r1", &ciphertext_path, plaintext).unwrap();
        std::fs::remove_file(&sealed.key_path).unwrap();
        assert!(verify_sealed_transcript(&sealed, &digest(plaintext)).is_err());
    }

    #[test]
    fn missing_ciphertext_fails_verification() {
        let root = tempfile::tempdir().unwrap();
        let plaintext = b"transcript body";
        let ciphertext_path = root
            .path()
            .join("runs")
            .join("r1")
            .join("transcript.sealed");
        let sealed = seal_transcript(root.path(), "r1", &ciphertext_path, plaintext).unwrap();
        std::fs::remove_file(&sealed.ciphertext_path).unwrap();
        assert!(verify_sealed_transcript(&sealed, &digest(plaintext)).is_err());
    }

    #[test]
    fn crypto_shred_makes_ciphertext_permanently_unrecoverable() {
        let root = tempfile::tempdir().unwrap();
        let plaintext = b"transcript to be shredded";
        let ciphertext_path = root
            .path()
            .join("runs")
            .join("r1")
            .join("transcript.sealed");
        let sealed = seal_transcript(root.path(), "r1", &ciphertext_path, plaintext).unwrap();
        verify_sealed_transcript(&sealed, &digest(plaintext)).unwrap();
        // Crypto-shred: delete only the key.
        std::fs::remove_file(&sealed.key_path).unwrap();
        assert!(
            sealed.ciphertext_path.is_file(),
            "ciphertext alone survives"
        );
        assert!(
            verify_sealed_transcript(&sealed, &digest(plaintext)).is_err(),
            "shredded transcript must never verify again"
        );
    }

    #[test]
    fn restart_reads_durable_state_from_disk_not_memory() {
        let root = tempfile::tempdir().unwrap();
        let plaintext = b"transcript surviving a restart";
        let ciphertext_path = root
            .path()
            .join("runs")
            .join("r1")
            .join("transcript.sealed");
        let sealed = seal_transcript(root.path(), "r1", &ciphertext_path, plaintext).unwrap();
        // Simulate a fresh process: rebuild the handle purely from paths, no
        // in-memory cipher/key state carried over.
        let reopened = SealedTranscript {
            key_path: sealed.key_path.clone(),
            ciphertext_path: sealed.ciphertext_path.clone(),
        };
        verify_sealed_transcript(&reopened, &digest(plaintext)).unwrap();
    }
}
