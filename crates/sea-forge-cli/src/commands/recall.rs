use sea_forge_capability::{self as capability, RecallQuery};
use sea_forge_core::{errors::ForgeError, types::SettlementStatus};
use std::path::Path;
pub fn execute(
    root: &Path,
    query: &str,
    entity: Option<&str>,
    process: Option<&str>,
    result: Option<SettlementStatus>,
    limit: usize,
) -> Result<u8, ForgeError> {
    let (matches, malformed) = capability::recall(
        &root.join("capabilities.jsonl"),
        RecallQuery {
            query,
            entity,
            process,
            result,
            limit,
        },
    )?;
    if malformed > 0 {
        tracing::warn!(
            event = "recall_malformed_lines",
            run_id = "none",
            component = "sea-forge-cli::recall",
            error_class = "capability_parse_error",
            malformed_lines = malformed
        );
    }
    for envelope in &matches {
        println!("{}", serde_json::to_string(envelope)?);
    }
    Ok(if matches.is_empty() { 3 } else { 0 })
}
