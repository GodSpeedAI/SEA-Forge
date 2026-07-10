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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ids_follow_the_persisted_grammar() {
        let run = run_id().unwrap();
        assert!(run.starts_with("run_") && run.len() == 27);
        assert_eq!(seq_id("tev", 4, 1), "tev_0001");
    }
}
