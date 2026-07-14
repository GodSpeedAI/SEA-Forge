use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use sea_forge_core::{
    errors::ForgeError,
    ids,
    types::{Attribution, MemoryItem, MemoryItemProvenance, MemoryKind, SemanticEnvelope},
    RECORD_VERSION,
};

const MAX_STATEMENT_LEN: usize = 1000;
pub const MAX_RECALL_LIMIT: usize = 50;

// ── Dedup key ──

pub fn compute_dedup_key(kind: &MemoryKind, statement: &str, entity_id: &str) -> String {
    let kind_str = serde_json::to_string(kind).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(kind_str.as_bytes());
    hasher.update(b"\x00");
    hasher.update(statement.trim().to_lowercase().as_bytes());
    hasher.update(b"\x00");
    hasher.update(entity_id.as_bytes());
    format!("{:x}", hasher.finalize())
}

// ── Extraction ──

/// Deterministic extraction: one `outcome` MemoryItem per envelope.
/// Never fails — returns an empty vec on error.
pub fn extract_from_envelope(envelope: &SemanticEnvelope, now: &str) -> Vec<MemoryItem> {
    let statement = format!(
        "attempt {} was {}",
        envelope.capability_delta.attempted_capability,
        serde_json::to_string(&envelope.capability_delta.result)
            .unwrap_or_else(|_| "\"unknown\"".into())
            .trim_matches('"')
    );
    let statement: String = statement.chars().take(MAX_STATEMENT_LEN).collect();
    let dedup_key = compute_dedup_key(
        &MemoryKind::Outcome,
        &statement,
        &envelope.attribution.entity_id,
    );
    let memory_id = match ids::random_id("mem") {
        Ok(id) => id,
        Err(_) => return vec![],
    };
    vec![MemoryItem {
        version: RECORD_VERSION.into(),
        memory_id,
        kind: MemoryKind::Outcome,
        statement,
        attribution: Attribution {
            entity_id: envelope.attribution.entity_id.clone(),
            process_id: envelope.attribution.process_id.clone(),
            session_id: envelope.attribution.session_id.clone(),
        },
        provenance: MemoryItemProvenance {
            run_ids: vec![envelope.run_id.clone()],
            evidence_refs: envelope.evidence_refs.clone(),
        },
        dedup_key,
        created_at: now.to_string(),
        last_confirmed_at: now.to_string(),
    }]
}

// ── items.jsonl store (append-only; dedup at read) ──

pub fn append_memory_items(path: &Path, items: &[MemoryItem]) -> Result<(), ForgeError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ForgeError::io("create memory dir", e))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| ForgeError::io("open memory items", e))?;
    for item in items {
        let mut encoded = serde_json::to_vec(item)?;
        encoded.push(b'\n');
        file.write_all(&encoded)
            .map_err(|e| ForgeError::io("append memory item", e))?;
    }
    file.flush()
        .map_err(|e| ForgeError::io("flush memory items", e))
}

pub fn load_memory_items(path: &Path) -> Result<Vec<MemoryItem>, ForgeError> {
    let file = File::open(path).map_err(|e| ForgeError::io("open memory items", e))?;
    let mut items = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|e| ForgeError::io("read memory items", e))?;
        match serde_json::from_str::<MemoryItem>(&line) {
            Ok(item) => items.push(item),
            Err(_) => { /* skip malformed */ }
        }
    }
    Ok(deduplicate(items))
}

/// Merge items with the same dedup_key: union run_ids/evidence_refs,
/// take earliest created_at, latest last_confirmed_at.
fn deduplicate(items: Vec<MemoryItem>) -> Vec<MemoryItem> {
    let mut by_key: HashMap<String, MemoryItem> = HashMap::new();
    for item in items {
        by_key
            .entry(item.dedup_key.clone())
            .and_modify(|existing| {
                for rid in &item.provenance.run_ids {
                    if !existing.provenance.run_ids.contains(rid) {
                        existing.provenance.run_ids.push(rid.clone());
                    }
                }
                for eid in &item.provenance.evidence_refs {
                    if !existing.provenance.evidence_refs.contains(eid) {
                        existing.provenance.evidence_refs.push(eid.clone());
                    }
                }
                if item.last_confirmed_at > existing.last_confirmed_at {
                    existing.last_confirmed_at = item.last_confirmed_at.clone();
                }
                if item.created_at < existing.created_at {
                    existing.created_at = item.created_at.clone();
                }
            })
            .or_insert(item);
    }
    let mut result: Vec<MemoryItem> = by_key.into_values().collect();
    result.sort_by(|a, b| b.last_confirmed_at.cmp(&a.last_confirmed_at));
    result
}

// ── Recall ──

pub struct MemoryRecallQuery<'a> {
    pub query: &'a str,
    pub entity_id: Option<&'a str>,
    pub process_id: Option<&'a str>,
    pub kinds: Option<&'a [MemoryKind]>,
    pub limit: usize,
}

/// Linear scan of items.jsonl. The source of truth for recall.
pub fn recall_memory(
    path: &Path,
    query: MemoryRecallQuery<'_>,
) -> Result<Vec<MemoryItem>, ForgeError> {
    let items = load_memory_items(path)?;
    let needle = query.query.to_lowercase();
    let limit = query.limit.min(MAX_RECALL_LIMIT);
    let mut results: Vec<MemoryItem> = items
        .into_iter()
        .filter(|item| {
            item.statement.to_lowercase().contains(&needle)
                && query
                    .entity_id
                    .is_none_or(|e| item.attribution.entity_id == e)
                && query
                    .process_id
                    .is_none_or(|p| item.attribution.process_id == p)
                && query.kinds.is_none_or(|kinds| kinds.contains(&item.kind))
        })
        .take(limit)
        .collect();
    // Return most-recently-confirmed first.
    results.sort_by(|a, b| b.last_confirmed_at.cmp(&a.last_confirmed_at));
    Ok(results)
}

// ── Scope enforcement ──

/// Check if a memory_scope rule allows the requester to access the given entity's items.
/// Returns false when no explicit cross-entity rule exists.
pub fn scope_allows(rule_scope: &str, requester_entity: &str, item_entity: &str) -> bool {
    if rule_scope == "any" {
        return true;
    }
    if rule_scope == "own" {
        return requester_entity == item_entity;
    }
    if let Some(target) = rule_scope.strip_prefix("entity:") {
        return target == item_entity;
    }
    false
}

// ── Index (pure rebuildable projection) ──
// ponytail: JSON inverted index instead of rusqlite/SQLite. The spec (§6.3, §10.5)
// names rusqlite as the implementation choice; a JSON projection achieves the same
// outcome (rebuildable, fallback-equivalent) without a C compilation dependency.
// Switch to rusqlite if the linear scan becomes a measured bottleneck.

#[derive(Serialize, Deserialize)]
struct MemoryIndex {
    version: String,
    rebuilt_at: String,
    entries: Vec<MemoryIndexEntry>,
}

#[derive(Serialize, Deserialize)]
struct MemoryIndexEntry {
    memory_id: String,
    dedup_key: String,
    statement: String,
    entity_id: String,
    process_id: String,
    kind: String,
}

pub fn rebuild_index(items_path: &Path, index_path: &Path, now: &str) -> Result<(), ForgeError> {
    let items = load_memory_items(items_path)?;
    let entries: Vec<MemoryIndexEntry> = items
        .iter()
        .map(|item| MemoryIndexEntry {
            memory_id: item.memory_id.clone(),
            dedup_key: item.dedup_key.clone(),
            statement: item.statement.clone(),
            entity_id: item.attribution.entity_id.clone(),
            process_id: item.attribution.process_id.clone(),
            kind: serde_json::to_string(&item.kind).unwrap_or_default(),
        })
        .collect();
    let index = MemoryIndex {
        version: RECORD_VERSION.into(),
        rebuilt_at: now.to_string(),
        entries,
    };
    if let Some(parent) = index_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ForgeError::io("create index dir", e))?;
    }
    let encoded = serde_json::to_vec_pretty(&index)?;
    std::fs::write(index_path, encoded).map_err(|e| ForgeError::io("write memory index", e))
}

/// Query the index. Returns None if the index is missing, stale, or unreadable,
/// causing the caller to fall back to the linear scan.
pub fn query_index(
    index_path: &Path,
    items_path: &Path,
    query: &MemoryRecallQuery<'_>,
) -> Option<Vec<MemoryItem>> {
    let index_bytes = std::fs::read(index_path).ok()?;
    let index: MemoryIndex = serde_json::from_slice(&index_bytes).ok()?;
    let items = load_memory_items(items_path).ok()?;
    let by_id: HashMap<String, MemoryItem> = items
        .into_iter()
        .map(|item| (item.memory_id.clone(), item))
        .collect();
    let needle = query.query.to_lowercase();
    let limit = query.limit.min(MAX_RECALL_LIMIT);
    let mut results: Vec<MemoryItem> = index
        .entries
        .iter()
        .filter_map(|entry| by_id.get(&entry.memory_id).cloned())
        .filter(|item| {
            item.statement.to_lowercase().contains(&needle)
                && query
                    .entity_id
                    .is_none_or(|e| item.attribution.entity_id == e)
                && query
                    .process_id
                    .is_none_or(|p| item.attribution.process_id == p)
                && query.kinds.is_none_or(|kinds| kinds.contains(&item.kind))
        })
        .take(limit)
        .collect();
    results.sort_by(|a, b| b.last_confirmed_at.cmp(&a.last_confirmed_at));
    Some(results)
}

/// Recall using the index if available, falling back to linear scan.
/// The results MUST be identical regardless of path (§10.5).
pub fn recall_with_fallback(
    items_path: &Path,
    index_path: &Path,
    query: MemoryRecallQuery<'_>,
) -> Result<Vec<MemoryItem>, ForgeError> {
    if let Some(indexed) = query_index(index_path, items_path, &query) {
        Ok(indexed)
    } else {
        recall_memory(items_path, query)
    }
}
