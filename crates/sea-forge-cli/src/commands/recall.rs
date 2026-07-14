use sea_forge_capability::{self as capability, RecallQuery};
use sea_forge_core::{
    errors::ForgeError,
    types::{AuthorityAction, MemoryKind, SettlementStatus},
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
    pub kind: Option<MemoryKind>,
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
        kind,
        limit,
    } = options;

    // Memory-item recall mode (M4b): search memory/items.jsonl when --kind is set.
    if let Some(memory_kind) = kind {
        return execute_memory_recall(root, policy, actor_id, query, entity, memory_kind, limit);
    }

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

fn execute_memory_recall(
    root: &Path,
    policy: Option<&Path>,
    actor_id: &str,
    query: &str,
    entity: Option<&str>,
    kind: MemoryKind,
    limit: usize,
) -> Result<u8, ForgeError> {
    let policy_path = super::mediated::policy_path(root, policy);
    super::mediated::authorize_read(
        root,
        &policy_path,
        actor_id,
        &AuthorityAction::Reserved {
            resource_type: "recall_memory".into(),
            resource_id: "memory".into(),
            parameters: serde_json::json!({"kind": kind, "limit": limit}),
        },
    )?;
    let items_path = root.join("memory/items.jsonl");
    let index_path = root.join("memory/index.json");
    let kind_slice = [kind];
    let kinds = Some(kind_slice.as_slice());
    let results = capability::memory::recall_with_fallback(
        &items_path,
        &index_path,
        capability::memory::MemoryRecallQuery {
            query,
            entity_id: entity,
            process_id: None,
            kinds,
            limit,
        },
    )?;
    for item in &results {
        println!("{}", serde_json::to_string(item)?);
    }
    Ok(if results.is_empty() { 3 } else { 0 })
}
