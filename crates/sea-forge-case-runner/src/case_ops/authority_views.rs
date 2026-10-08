//! Rebuildable authority projections from the append-only ledger.
//!
//! Extracted verbatim from `sea-forge-cli`'s `pipeline.rs` (they were private
//! helpers of the CLI's mediation path) so the mediated authority check in
//! [`super::mediation`] keeps writing the same `authority/decisions.jsonl`,
//! `authority/opaque-constraints.json` and `authority/audit.jsonl` mirror
//! views no matter which binary invoked it.

use sea_forge_core::errors::ForgeError;
use sea_forge_ledger::{CommittedRecordRef, LedgerManager, LedgerStream};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::Path;

pub fn rebuild_authority_mirrors(
    root: &Path,
    materializer: &LedgerStream,
    committed: &CommittedRecordRef,
) -> Result<(), ForgeError> {
    let manager = LedgerManager::new(root)?;
    let mut decisions = Vec::new();
    for stream_id in manager.list_stream_ids()? {
        let stream = manager.open_stream(&stream_id, "authority-rebuild")?;
        stream.verify()?;
        for entry in stream.read_entries()? {
            if entry.record_kind == "authority_decision" {
                decisions.push(entry);
            }
        }
    }
    decisions.sort_by(|left, right| {
        (&left.committed_at, &left.ledger_id, left.append_ordinal).cmp(&(
            &right.committed_at,
            &right.ledger_id,
            right.append_ordinal,
        ))
    });
    let mut decision_bytes = Vec::new();
    let mut audit_bytes = Vec::new();
    let mut opaque_constraints = Vec::new();
    let source_entry_ulids = decisions
        .iter()
        .map(|entry| entry.entry_ulid.clone())
        .collect::<Vec<_>>();
    for entry in decisions {
        serde_json::to_writer(&mut decision_bytes, &entry.payload)?;
        decision_bytes.push(b'\n');
        if let Some(audit) = entry.payload.get("audit_record") {
            serde_json::to_writer(&mut audit_bytes, audit)?;
            audit_bytes.push(b'\n');
        }
        if let Some(constraint_id) = entry
            .payload
            .get("opaque_constraint_id")
            .and_then(serde_json::Value::as_str)
        {
            opaque_constraints.push(json!({
                "constraint_id": constraint_id,
                "created_by_decision_id": entry.payload["decision_id"],
                "target": {
                    "resource_type": entry.payload["action_request"]["action"]["resource_type"],
                    "resource_id": entry.payload["action_request"]["action"]["resource_id"]
                },
                "reason": "all configured evaluators escalated",
                "created_at": entry.payload["decided_at"],
                "expires_at": serde_json::Value::Null
            }));
        }
    }
    materializer.materialize_aggregate_view(
        committed,
        source_entry_ulids.clone(),
        &root.join("authority/decisions.jsonl"),
        &decision_bytes,
    )?;
    materializer.materialize_aggregate_view(
        committed,
        source_entry_ulids.clone(),
        &root.join("authority/opaque-constraints.json"),
        &serde_json::to_vec_pretty(&opaque_constraints)?,
    )?;
    materializer.materialize_aggregate_view(
        committed,
        source_entry_ulids,
        &root.join("authority/audit.jsonl"),
        &audit_bytes,
    )?;
    Ok(())
}

pub fn load_opaque_constraints(
    root: &Path,
) -> Result<Vec<sea_forge_authority::OpaqueConstraint>, ForgeError> {
    let manager = LedgerManager::new(root)?;
    let mut constraints = BTreeMap::new();
    for stream_id in manager.list_stream_ids()? {
        let stream = manager.open_stream(&stream_id, "opaque-constraint-load")?;
        stream.verify()?;
        for entry in stream.read_entries()? {
            let Some(constraint_id) = entry
                .payload
                .get("opaque_constraint_id")
                .and_then(serde_json::Value::as_str)
            else {
                continue;
            };
            let action = &entry.payload["action_request"]["action"];
            let resource_type = action["resource_type"].as_str().unwrap_or_default();
            let resource_id = action["resource_id"].as_str().unwrap_or_default();
            if resource_type.is_empty() || resource_id.is_empty() {
                continue;
            }
            constraints.insert(
                format!("{resource_type}:{resource_id}"),
                sea_forge_authority::OpaqueConstraint {
                    constraint_id: constraint_id.into(),
                    resource_type: resource_type.into(),
                    resource_id: resource_id.into(),
                    created_by_decision_id: entry.payload["decision_id"]
                        .as_str()
                        .unwrap_or_default()
                        .into(),
                    reason: "all configured evaluators escalated".into(),
                    created_at: entry.payload["decided_at"]
                        .as_str()
                        .unwrap_or_default()
                        .into(),
                    expires_at: None,
                },
            );
        }
    }
    Ok(constraints.into_values().collect())
}
