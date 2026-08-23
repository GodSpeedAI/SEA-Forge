use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
};

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
    let mut reader = BufReader::new(file);
    let mut raw = Vec::new();
    loop {
        // F-25.r: one non-UTF-8 byte must not abort a whole recall scan;
        // undecodable lines are skipped exactly like malformed-JSON lines.
        raw.clear();
        let read = reader
            .read_until(b'\n', &mut raw)
            .map_err(|e| ForgeError::io("read memory items", e))?;
        if read == 0 {
            break;
        }
        match std::str::from_utf8(&raw)
            .ok()
            .and_then(|line| serde_json::from_str::<MemoryItem>(line.trim()).ok())
        {
            Some(item) => items.push(item),
            None => { /* skip malformed */ }
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
    // Return most-recently-confirmed first; tie-break on memory_id ascending
    // so the fallback path matches `query_index` byte-for-byte (§10.5).
    results.sort_by(|a, b| {
        b.last_confirmed_at
            .cmp(&a.last_confirmed_at)
            .then_with(|| a.memory_id.cmp(&b.memory_id))
    });
    Ok(results)
}

// ── Index (rebuildable SQLite FTS5 projection) ──
// §6.3/§10.5 select SQLite FTS explicitly. `items.jsonl` stays authoritative;
// this index is a disposable acceleration structure proven fresh by a source
// digest, never trusted merely because it deserializes.

fn source_commitment(items_path: &Path) -> Result<String, ForgeError> {
    let bytes = std::fs::read(items_path).map_err(|e| ForgeError::io("read memory items", e))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

fn sqlite_error(context: &str, error: rusqlite::Error) -> ForgeError {
    ForgeError::Internal(format!("memory index {context}: {error}"))
}

/// Rebuild `index_path` from `items_path` into a fresh SQLite FTS5 database,
/// writing to a temporary file and renaming into place only after an integrity
/// check passes, so a crash mid-rebuild never leaves a half-written index.
pub fn rebuild_index(items_path: &Path, index_path: &Path, now: &str) -> Result<(), ForgeError> {
    let items = load_memory_items(items_path)?;
    let source_sha256 = source_commitment(items_path)?;
    if let Some(parent) = index_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ForgeError::io("create index dir", e))?;
    }
    let tmp_path = index_path.with_extension("sqlite.tmp");
    let _ = std::fs::remove_file(&tmp_path);
    {
        let mut conn =
            rusqlite::Connection::open(&tmp_path).map_err(|e| sqlite_error("open", e))?;
        conn.execute_batch(
            "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             CREATE VIRTUAL TABLE memory_fts USING fts5(
                 memory_id UNINDEXED,
                 statement,
                 entity_id UNINDEXED,
                 process_id UNINDEXED,
                 kind UNINDEXED,
                 last_confirmed_at UNINDEXED
             );",
        )
        .map_err(|e| sqlite_error("schema", e))?;
        let tx = conn.transaction().map_err(|e| sqlite_error("begin", e))?;
        {
            let mut stmt = tx
                .prepare(
                    "INSERT INTO memory_fts \
                     (memory_id, statement, entity_id, process_id, kind, last_confirmed_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                )
                .map_err(|e| sqlite_error("prepare insert", e))?;
            for item in &items {
                let kind = serde_json::to_string(&item.kind).unwrap_or_default();
                stmt.execute(rusqlite::params![
                    item.memory_id,
                    item.statement,
                    item.attribution.entity_id,
                    item.attribution.process_id,
                    kind,
                    item.last_confirmed_at,
                ])
                .map_err(|e| sqlite_error("insert row", e))?;
            }
            tx.execute(
                "INSERT INTO meta (key, value) VALUES \
                 ('version', ?1), ('rebuilt_at', ?2), ('source_sha256', ?3), ('count', ?4)",
                rusqlite::params![RECORD_VERSION, now, source_sha256, items.len().to_string()],
            )
            .map_err(|e| sqlite_error("write meta", e))?;
        }
        tx.commit().map_err(|e| sqlite_error("commit", e))?;
        let check: String = conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(|e| sqlite_error("integrity_check", e))?;
        if check != "ok" {
            return Err(ForgeError::Internal(format!(
                "memory index integrity_check failed: {check}"
            )));
        }
    }
    std::fs::rename(&tmp_path, index_path).map_err(|e| ForgeError::io("rename memory index", e))
}

/// Query the index. Returns `None` if the index is missing, corrupt, or stale
/// relative to `items_path`'s current source digest, causing the caller to
/// fall back to the linear scan. When present and fresh, applies the exact
/// same substring/scope filter as the linear scan (rather than FTS5 `MATCH`
/// token semantics) so indexed and fallback results are always identical.
pub fn query_index(
    index_path: &Path,
    items_path: &Path,
    query: &MemoryRecallQuery<'_>,
) -> Option<Vec<MemoryItem>> {
    let conn = rusqlite::Connection::open_with_flags(
        index_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .ok()?;
    let check: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .ok()?;
    if check != "ok" {
        return None;
    }
    let stored_sha256: String = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'source_sha256'",
            [],
            |row| row.get(0),
        )
        .ok()?;
    let current_sha256 = source_commitment(items_path).ok()?;
    if stored_sha256 != current_sha256 {
        return None;
    }
    let mut stmt = conn.prepare("SELECT memory_id FROM memory_fts").ok()?;
    let indexed_ids: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .ok()?
        .collect::<Result<_, _>>()
        .ok()?;
    let items = load_memory_items(items_path).ok()?;
    let by_id: HashMap<String, MemoryItem> = items
        .into_iter()
        .map(|item| (item.memory_id.clone(), item))
        .collect();
    let needle = query.query.to_lowercase();
    let limit = query.limit.min(MAX_RECALL_LIMIT);
    let mut results: Vec<MemoryItem> = indexed_ids
        .iter()
        .filter_map(|id| by_id.get(id).cloned())
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
    results.sort_by(|a, b| {
        b.last_confirmed_at
            .cmp(&a.last_confirmed_at)
            .then_with(|| a.memory_id.cmp(&b.memory_id))
    });
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

#[cfg(test)]
mod f25r_tests {
    use super::*;
    use sea_forge_core::types::{
        Attribution, CapabilityDelta, Intent, SemanticEnvelope, SettlementStatus,
    };

    // F-25.r: one non-UTF-8 byte degrades to a skipped line, never a failed
    // recall scan — the same tolerance malformed-JSON lines already had.
    #[test]
    fn non_utf8_line_degrades_to_skipped_not_scan_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("items.jsonl");

        let make = |statement: &str, capability: &str| {
            let envelope = SemanticEnvelope {
                version: "0.1".into(),
                run_id: format!("run_{statement}"),
                case_ref: "case_r".into(),
                intent: Intent {
                    intent_id: "int_r".into(),
                    summary: statement.into(),
                    actor_id: "entity_a".into(),
                    process_id: "proc".into(),
                    created_at: "2026-08-23T00:00:00Z".into(),
                },
                plan_ref: "plan_r".into(),
                template_ref: None,
                authority_decisions: vec![],
                evidence_refs: vec!["evd_1".into()],
                settlement_ref: "set_1".into(),
                capability_delta: CapabilityDelta {
                    attempted_capability: capability.into(),
                    result: SettlementStatus::Accepted,
                },
                attribution: Attribution {
                    entity_id: "entity_a".into(),
                    process_id: "proc".into(),
                    session_id: "sess".into(),
                },
                artifact_refs: vec![],
                extension_refs: vec![],
                projection_refs: vec![],
                cell_id: None,
            };
            extract_from_envelope(&envelope, "2026-08-23T00:00:00Z").remove(0)
        };

        append_memory_items(&path, &[make("first statement", "cap-one")]).unwrap();
        use std::io::Write as _;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        file.write_all(b"\xff\xfe not utf8\n").unwrap();
        drop(file);
        append_memory_items(&path, &[make("second statement", "cap-two")]).unwrap();

        // Deduplication unions same-key items; distinct capabilities keep the
        // two records apart, so exactly the two valid lines survive.
        let items = load_memory_items(&path).expect("scan must not fail on bad bytes");
        assert_eq!(items.len(), 2, "{items:?}");
    }
}
