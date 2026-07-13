use sea_forge_capability::{self as capability, RecallQuery};
use sea_forge_core::{
    errors::ForgeError,
    types::{AuthorityAction, SettlementStatus},
};
use std::path::Path;
pub fn execute(
    root: &Path,
    policy: Option<&Path>,
    query: &str,
    entity: Option<&str>,
    process: Option<&str>,
    result: Option<SettlementStatus>,
    limit: usize,
) -> Result<u8, ForgeError> {
    let capability_path = root.join("capabilities.jsonl");
    if !capability_path.exists() {
        capability::recall(
            &capability_path,
            RecallQuery {
                query,
                entity,
                process,
                result: result.clone(),
                limit,
            },
        )?;
    }
    let policy = super::mediated::policy_path(root, policy);
    super::mediated::authorize_read(
        root,
        &policy,
        entity.unwrap_or("operator_local"),
        &AuthorityAction::Reserved {
            resource_type: "recall_memory".into(),
            resource_id: "capabilities".into(),
            parameters: serde_json::json!({"limit": limit}),
        },
    )?;
    let (matches, malformed) = capability::recall(
        &capability_path,
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
