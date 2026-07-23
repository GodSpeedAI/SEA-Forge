use sea_forge_capability::{self as capability, RecallQuery};
use sea_forge_core::{
    errors::ForgeError,
    types::{AuthorityAction, MemoryKind, SettlementStatus},
};
use sea_forge_ledger::LedgerStream;
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
        return execute_memory_recall(
            root,
            policy,
            actor_id,
            query,
            entity,
            process,
            memory_kind,
            limit,
        );
    }

    // Compatibility capability-envelope recall (minimum spec): a read-only
    // scan of capabilities.jsonl. Output must equal the matched stored
    // envelopes exactly — no injected fields — so v0.1 callers see byte-for-
    // structure identical output. Assurance is a governed-path concern (see
    // execute_memory_recall), not a compatibility-envelope mutation.
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
        println!("{}", serde_json::to_string(envelope)?);
    }
    Ok(if matches.is_empty() { 3 } else { 0 })
}

/// Governed memory recall (M4b): the exact requester/target/kinds/limit are
/// bound into the authority action so the decision (and its hash) changes
/// whenever the requested scope changes (Task 6). A denial short-circuits
/// before `memory/items.jsonl` is ever read. The granted `memory_scope` is
/// re-applied at this executor, independent of the query filter, so a query-
/// layer bug can never surface results outside what authority actually
/// granted. A recall-evidence record names the request scope and every
/// returned memory ID before any result reaches stdout.
#[allow(clippy::too_many_arguments)]
fn execute_memory_recall(
    root: &Path,
    policy: Option<&Path>,
    actor_id: &str,
    query: &str,
    entity: Option<&str>,
    process: Option<&str>,
    kind: MemoryKind,
    limit: usize,
) -> Result<u8, ForgeError> {
    let policy_path = super::mediated::policy_path(root, policy);
    // An omitted --entity means "recall my own memory": the target defaults
    // to the requester's own identity, not an empty string, so "own"-scoped
    // grants authorize the implicit self-recall case. An explicit --entity
    // always names the real target, including cross-entity requests that a
    // narrower grant must deny.
    let target_entity = entity.unwrap_or(actor_id);
    let action = AuthorityAction::Reserved {
        resource_type: "recall_memory".into(),
        resource_id: "memory".into(),
        parameters: serde_json::json!({
            "entity_id": target_entity,
            "requester_entity": actor_id,
            "process_id": process,
            "kinds": [&kind],
            "limit": limit,
        }),
    };
    let memory_scope = super::mediated::with_authorized_action(
        root,
        &policy_path,
        actor_id,
        &action,
        false,
        |grant| {
            // v0.1 policies with no recall_memory rule fall back to a
            // scope-free legacy allow (same permissiveness as inspect_run /
            // validate_model); treat that as "any", matching the unscoped
            // legacy capability-envelope path's behavior.
            let scope = grant.memory_scope().unwrap_or("any").to_string();
            grant.authorize(&action, "read_ingress", "read_ingress", root)?;
            Ok(scope)
        },
    )?;

    // Push the authorized target down into the query filter whenever it's
    // meaningful, so the common case never depends solely on the defense-in-
    // depth post-filter below. An "any" grant with no explicit --entity stays
    // unfiltered at the query layer (a genuinely cross-entity recall).
    let query_entity_filter = if memory_scope == "any" && entity.is_none() {
        None
    } else {
        Some(target_entity)
    };
    let items_path = root.join("memory/items.jsonl");
    let index_path = root.join("memory/index.sqlite");
    let kind_slice = [kind.clone()];
    let kinds = Some(kind_slice.as_slice());
    let results = capability::memory::recall_with_fallback(
        &items_path,
        &index_path,
        capability::memory::MemoryRecallQuery {
            query,
            entity_id: query_entity_filter,
            process_id: process,
            kinds,
            limit,
        },
    )?;
    let scoped_results: Vec<_> = results
        .into_iter()
        .filter(|item| {
            sea_forge_authority::scope_allows(&memory_scope, actor_id, &item.attribution.entity_id)
        })
        .collect();

    let memory_ids: Vec<&str> = scoped_results
        .iter()
        .map(|i| i.memory_id.as_str())
        .collect();
    let assurance = super::mediated::assurance(root, &policy_path, actor_id)?;
    commit_recall_evidence(
        root,
        actor_id,
        &memory_scope,
        &assurance,
        entity,
        process,
        kind,
        limit,
        &memory_ids,
    )?;
    tracing::info!(
        event = "recall_evidence_committed",
        component = "sea-forge-cli::recall",
        requester_entity = actor_id,
        memory_scope = %memory_scope,
        assurance = %assurance,
        returned = memory_ids.len(),
    );

    for item in &scoped_results {
        println!("{}", serde_json::to_string(item)?);
    }
    Ok(if scoped_results.is_empty() { 3 } else { 0 })
}

#[allow(clippy::too_many_arguments)]
fn commit_recall_evidence(
    root: &Path,
    actor_id: &str,
    memory_scope: &str,
    assurance: &str,
    entity: Option<&str>,
    process: Option<&str>,
    kind: MemoryKind,
    limit: usize,
    memory_ids: &[&str],
) -> Result<(), ForgeError> {
    let stream = LedgerStream::open(root, "memory-recalls", actor_id)?;
    stream.commit_typed(
        "recall_evidence",
        memory_ids.iter().map(|id| id.to_string()).collect(),
        &serde_json::json!({
            "requester_entity": actor_id,
            "entity_id": entity,
            "process_id": process,
            "memory_scope": memory_scope,
            "assurance": assurance,
            "kind": kind,
            "limit": limit,
            "returned_memory_ids": memory_ids,
        }),
        vec![],
    )?;
    Ok(())
}
