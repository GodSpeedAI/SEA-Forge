use getrandom::fill;
use time::{macros::format_description, OffsetDateTime, PrimitiveDateTime};

use crate::errors::ForgeError;

fn random_hex() -> Result<String, ForgeError> {
    let mut bytes = [0_u8; 3];
    fill(&mut bytes).map_err(|error| ForgeError::Internal(format!("OS RNG failed: {error}")))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

/// 4 random bytes as 8 lowercase hex chars.
fn random_hex_4() -> Result<String, ForgeError> {
    let mut bytes = [0_u8; 4];
    fill(&mut bytes).map_err(|error| ForgeError::Internal(format!("OS RNG failed: {error}")))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

/// `cell_<8 hex>` — generated once per `.sea-forge/cell.json` (spec-full §7.4).
pub fn cell_id() -> Result<String, ForgeError> {
    Ok(format!("cell_{}", random_hex_4()?))
}

/// `bundle_<8 hex>` — federation bundle identifier.
pub fn bundle_id() -> Result<String, ForgeError> {
    Ok(format!("bundle_{}", random_hex_4()?))
}

pub fn run_id() -> Result<String, ForgeError> {
    timestamped("run")
}
pub fn case_id() -> Result<String, ForgeError> {
    timestamped("case")
}
/// `smsnap_<timestamp>_<6hex>` — self-model snapshot identifier (spec-adlc-thoth §7.6).
pub fn snapshot_id() -> Result<String, ForgeError> {
    timestamped("smsnap")
}
pub fn random_id(prefix: &str) -> Result<String, ForgeError> {
    Ok(format!("{prefix}_{}", random_hex()?))
}
fn timestamped(prefix: &str) -> Result<String, ForgeError> {
    let timestamp = OffsetDateTime::now_utc()
        .format(&format_description!(
            "[year][month][day]T[hour][minute][second]Z"
        ))
        .map_err(|error| ForgeError::Internal(format!("timestamp formatting failed: {error}")))?;
    Ok(format!("{prefix}_{timestamp}_{}", random_hex()?))
}
pub fn seq_id(prefix: &str, width: usize, sequence: usize) -> String {
    format!("{prefix}_{sequence:0width$}")
}
pub fn valid_run_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 27
        && &bytes[..4] == b"run_"
        && bytes[4..12].iter().all(u8::is_ascii_digit)
        && bytes[12] == b'T'
        && bytes[13..19].iter().all(u8::is_ascii_digit)
        && bytes[19] == b'Z'
        && bytes[20] == b'_'
        && bytes[21..]
            .iter()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        && {
            // Chrono accepted leap-second spellings at any minute. Validate the
            // same calendar and clock fields with second 59 in that case.
            let timestamp = if &value[17..19] == "60" {
                format!("{}59Z", &value[4..17])
            } else {
                value[4..20].to_owned()
            };
            PrimitiveDateTime::parse(
                &timestamp,
                &format_description!("[year][month][day]T[hour][minute][second]Z"),
            )
            .is_ok()
        }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn historical_run_ids_keep_calendar_and_leap_second_semantics() {
        for value in [
            "run_00000101T000000Z_abcdef",
            "run_20240229T235960Z_123abc",
            "run_20170101T123460Z_abcdef",
            "run_99991231T235959Z_ffffff",
        ] {
            assert!(valid_run_id(value), "{value}");
        }
        for value in [
            "run_20230229T235960Z_abcdef",
            "run_20240101T246060Z_abcdef",
            "run_20240101T235961Z_abcdef",
            "run_20240101T235959Z_ABCDEF",
        ] {
            assert!(!valid_run_id(value), "{value}");
        }
    }
    #[test]
    fn ids_follow_the_persisted_grammar() {
        let run = run_id().unwrap();
        assert!(run.starts_with("run_") && run.len() == 27);
        assert_eq!(seq_id("tev", 4, 1), "tev_0001");
        assert!(valid_run_id(&run));
        assert!(!valid_run_id("run_99999999T999999Z_abcdef"));
    }
}
