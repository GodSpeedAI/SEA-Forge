use chrono::Utc;
use getrandom::fill;

use crate::errors::ForgeError;

fn random_hex() -> Result<String, ForgeError> {
    let mut bytes = [0_u8; 3];
    fill(&mut bytes).map_err(|error| ForgeError::Internal(format!("OS RNG failed: {error}")))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

pub fn run_id() -> Result<String, ForgeError> {
    timestamped("run")
}
pub fn case_id() -> Result<String, ForgeError> {
    timestamped("case")
}
pub fn random_id(prefix: &str) -> Result<String, ForgeError> {
    Ok(format!("{prefix}_{}", random_hex()?))
}
fn timestamped(prefix: &str) -> Result<String, ForgeError> {
    Ok(format!(
        "{prefix}_{}_{}",
        Utc::now().format("%Y%m%dT%H%M%SZ"),
        random_hex()?
    ))
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
        && chrono::NaiveDateTime::parse_from_str(&value[4..20], "%Y%m%dT%H%M%SZ").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ids_follow_the_persisted_grammar() {
        let run = run_id().unwrap();
        assert!(run.starts_with("run_") && run.len() == 27);
        assert_eq!(seq_id("tev", 4, 1), "tev_0001");
        assert!(valid_run_id(&run));
        assert!(!valid_run_id("run_99999999T999999Z_abcdef"));
    }
}
