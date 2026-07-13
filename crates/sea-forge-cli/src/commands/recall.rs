use sea_forge_capability::{self as capability, RecallQuery};
use sea_forge_core::{
    errors::ForgeError,
    types::{AuthorityAction, SettlementStatus},
};
use std::path::Path;
pub struct RecallOptions<'a> {
    pub root: &'a Path,
    pub policy: Option<&'a Path>,
    pub actor_id: &'a str,
    pub query: &'a str,
    pub entity: Option<&'a str>,
    pub process: Option<&'a str>,
    pub result: Option<SettlementStatus>,
    pub limit: usize,
}

pub fn execute(options: RecallOptions<'_>) -> Result<u8, ForgeError> {
    let RecallOptions {
        root,
        policy,
        actor_id,
        query,
        entity,
        process,
        result,
        limit,
    } = options;
    let capability_path = root.join("capabilities.jsonl");
    let policy = super::mediated::policy_path(root, policy);
    super::mediated::authorize_read(
        root,
        &policy,
        actor_id,
        &AuthorityAction::Reserved {
            resource_type: "recall_memory".into(),
            resource_id: "capabilities".into(),
            parameters: serde_json::json!({"limit": limit}),
        },
    )?;
    if !capability_path.exists() {
        std::fs::File::open(&capability_path)
            .map_err(|error| ForgeError::io("open capability memory", error))?;
    }
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
        let mut value = serde_json::to_value(envelope)?;
        if let Some(object) = value.as_object_mut() {
            let assurance = super::mediated::record_assurance(
                root,
                &policy,
                actor_id,
                "capability_envelope",
                envelope,
            )?;
            object.insert("assurance".into(), assurance.into());
        }
        println!("{}", serde_json::to_string(&value)?);
    }
    Ok(if matches.is_empty() { 3 } else { 0 })
}
