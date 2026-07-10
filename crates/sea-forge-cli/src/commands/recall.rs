use sea_forge_core::{
    capability::{self, RecallQuery},
    errors::ForgeError,
    types::SettlementStatus,
};
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
        eprintln!("sea-forge: recall skipped {malformed} malformed line(s)");
    }
    for envelope in &matches {
        println!("{}", serde_json::to_string(envelope)?);
    }
    Ok(if matches.is_empty() { 3 } else { 0 })
}
