use ed25519_dalek::{Signer, Verifier};
use sea_forge_core::errors::ForgeError;
use std::{fs, path::Path};

pub use ed25519_dalek::{SigningKey, VerifyingKey};

/// Load or create an Ed25519 signing key referenced by `key_id`.
/// Keys are stored outside `.sea-forge/` by passing an absolute key directory.
pub fn load_or_create_signing_key(key_dir: &Path, key_id: &str) -> Result<SigningKey, ForgeError> {
    let path = key_dir.join(format!("{key_id}.key"));
    if path.exists() {
        let bytes = fs::read(&path).map_err(|e| ForgeError::io("read signing key", e))?;
        let key_bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| ForgeError::Internal(format!("signing key {key_id} is not 32 bytes")))?;
        Ok(SigningKey::from_bytes(&key_bytes))
    } else {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        fs::create_dir_all(key_dir).map_err(|e| ForgeError::io("create key directory", e))?;
        fs::write(&path, signing_key.to_bytes())
            .map_err(|e| ForgeError::io("write signing key", e))?;
        // Restrict permissions on the private key file.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o600);
            let _ = fs::set_permissions(&path, perms);
        }
        Ok(signing_key)
    }
}

pub fn load_verifying_key(key_dir: &Path, key_id: &str) -> Result<VerifyingKey, ForgeError> {
    let signing_key = load_or_create_signing_key(key_dir, key_id)?;
    Ok(signing_key.verifying_key())
}

pub fn sign_bytes(signing_key: &SigningKey, bytes: &[u8]) -> String {
    let signature = signing_key.sign(bytes);
    format!("ed25519:{}", base64_encode(&signature.to_bytes()))
}

pub fn verify_signature(
    verifying_key: &VerifyingKey,
    bytes: &[u8],
    signature_str: &str,
) -> Result<(), ForgeError> {
    let signature_str = signature_str
        .strip_prefix("ed25519:")
        .ok_or_else(|| ForgeError::Internal("signature algorithm must be ed25519".into()))?;
    let signature_bytes = base64_decode(signature_str)
        .map_err(|e| ForgeError::Internal(format!("invalid signature base64: {e}")))?;
    let signature = ed25519_dalek::Signature::from_slice(&signature_bytes)
        .map_err(|e| ForgeError::Internal(format!("invalid signature length: {e}")))?;
    verifying_key
        .verify(bytes, &signature)
        .map_err(|e| ForgeError::Internal(format!("signature verification failed: {e}")))
}

fn base64_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() * 4).div_ceil(3));
    let mut i = 0;
    while i < bytes.len() {
        let b1 = bytes[i];
        let b2 = bytes.get(i + 1).copied().unwrap_or(0);
        let b3 = bytes.get(i + 2).copied().unwrap_or(0);
        out.push(ALPHABET[((b1 >> 2) & 0x3f) as usize] as char);
        out.push(ALPHABET[(((b1 & 0x03) << 4) | ((b2 >> 4) & 0x0f)) as usize] as char);
        if i + 1 < bytes.len() {
            out.push(ALPHABET[(((b2 & 0x0f) << 2) | ((b3 >> 6) & 0x03)) as usize] as char);
        } else {
            out.push('=');
        }
        if i + 2 < bytes.len() {
            out.push(ALPHABET[(b3 & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        i += 3;
    }
    out
}

fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    const ALPHABET: [i8; 256] = {
        let mut table = [-1i8; 256];
        let bytes = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < bytes.len() {
            table[bytes[i] as usize] = i as i8;
            i += 1;
        }
        table
    };

    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let mut buf = [0u8; 4];
    let mut buf_len = 0usize;
    for c in s.chars() {
        if c == '=' {
            break;
        }
        let idx = ALPHABET[c as usize];
        if idx < 0 {
            return Err(format!("invalid base64 character: {c}"));
        }
        buf[buf_len] = idx as u8;
        buf_len += 1;
        if buf_len == 4 {
            out.push((buf[0] << 2) | (buf[1] >> 4));
            if buf[2] != 64 {
                out.push((buf[1] << 4) | (buf[2] >> 2));
            }
            if buf[3] != 64 {
                out.push((buf[2] << 6) | buf[3]);
            }
            buf_len = 0;
        }
    }
    if buf_len == 2 {
        out.push((buf[0] << 2) | (buf[1] >> 4));
    } else if buf_len == 3 {
        out.push((buf[0] << 2) | (buf[1] >> 4));
        out.push((buf[1] << 4) | (buf[2] >> 2));
    } else if buf_len == 1 {
        return Err("invalid base64 length".into());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn round_trip_sign_and_verify() {
        let dir = tempdir().unwrap();
        let key = load_or_create_signing_key(dir.path(), "test").unwrap();
        let msg = b"hello ledger";
        let sig = sign_bytes(&key, msg);
        let vk = key.verifying_key();
        verify_signature(&vk, msg, &sig).unwrap();
    }

    #[test]
    fn base64_round_trip() {
        let data = b"hello world";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}
