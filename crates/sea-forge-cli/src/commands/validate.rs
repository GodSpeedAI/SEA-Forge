use sea_forge_core::errors::ForgeError;
use std::{fs, path::Path};

/// Config-free shape validator for the stub `.sea` contract (minimum §11.1).
///
/// Reads the file, checks domain/entities shape, prints exact success/failure
/// lines, and exits 0/1. Creates no state under `.sea-forge` and requires no
/// policy or root.
pub fn execute(path: &Path) -> Result<u8, ForgeError> {
    match fs::read(path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| {
            serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|e| e.to_string())
        })
        .and_then(validate)
    {
        Ok(()) => {
            println!("sea-forge: model valid");
            Ok(0)
        }
        Err(reason) => {
            eprintln!("sea-forge: model invalid: {reason}");
            Ok(1)
        }
    }
}

/// Config-free M5 stage acceptance gate (spec-audit-remediation Task 10B):
/// reads `path`, verifies its SHA-256 equals `expected_sha256`, and exits
/// 0/1. Creates no `.sea-forge` state, requires no policy or root — the
/// same minimum-kernel shape as `execute` above.
pub fn execute_stage_check(path: &Path, expected_sha256: &str) -> Result<u8, ForgeError> {
    match fs::read(path) {
        Ok(bytes) => {
            let actual = sea_forge_evidence::sha256_bytes(&bytes);
            let expected = expected_sha256
                .strip_prefix("sha256:")
                .unwrap_or(expected_sha256);
            if actual == expected {
                println!("sea-forge: stage valid");
                Ok(0)
            } else {
                eprintln!("sea-forge: stage invalid: sha256 mismatch");
                Ok(1)
            }
        }
        Err(error) => {
            eprintln!("sea-forge: stage invalid: {error}");
            Ok(1)
        }
    }
}

fn validate(value: serde_json::Value) -> Result<(), String> {
    let domain = value
        .get("domain")
        .and_then(|v| v.as_str())
        .filter(|v| !v.is_empty())
        .ok_or("domain must be a non-empty string")?;
    let _ = domain;
    let entities = value
        .get("entities")
        .and_then(|v| v.as_array())
        .filter(|v| !v.is_empty())
        .ok_or("entities must be a non-empty array")?;
    if entities.iter().all(|e| {
        e.get("name")
            .and_then(|v| v.as_str())
            .is_some_and(|v| !v.is_empty())
    }) {
        Ok(())
    } else {
        Err("every entity must have a non-empty name".into())
    }
}
