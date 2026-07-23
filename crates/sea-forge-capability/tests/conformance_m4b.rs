use sea_forge_capability::memory::{
    append_memory_items, compute_dedup_key, extract_from_envelope, query_index, rebuild_index,
    recall_memory, recall_with_fallback, MemoryRecallQuery,
};
use sea_forge_core::types::*;
use tempfile::TempDir;

fn make_envelope(
    run_id: &str,
    entity: &str,
    capability: &str,
    result: SettlementStatus,
) -> SemanticEnvelope {
    SemanticEnvelope {
        version: "0.1".into(),
        run_id: run_id.into(),
        case_ref: "case_test".into(),
        intent: Intent {
            intent_id: "int_test".into(),
            summary: format!("test {capability}"),
            actor_id: entity.into(),
            process_id: "proc_test".into(),
            created_at: "2026-07-14T00:00:00Z".into(),
        },
        plan_ref: "plan_test".into(),
        template_ref: None,
        authority_decisions: vec![],
        evidence_refs: vec!["evd_0001".into()],
        settlement_ref: "set_0001".into(),
        capability_delta: CapabilityDelta {
            attempted_capability: capability.into(),
            result,
        },
        attribution: Attribution {
            entity_id: entity.into(),
            process_id: "proc_test".into(),
            session_id: "sess_test".into(),
        },
        artifact_refs: vec![],
        extension_refs: vec![],
        projection_refs: vec![],
        cell_id: None,
    }
}

fn write_items(dir: &TempDir, items: &[MemoryItem]) {
    append_memory_items(&dir.path().join("memory/items.jsonl"), items).unwrap();
}

// ── 1. Two-entity extraction produces deduplicated, provenance-linked items ──

#[test]
fn two_entity_extraction_deduplicates_and_links_provenance() {
    let dir = TempDir::new().unwrap();

    // Entity A runs the same capability twice with the same result.
    let env1 = make_envelope("run_001", "entity_a", "cap_x", SettlementStatus::Accepted);
    let env2 = make_envelope("run_002", "entity_a", "cap_x", SettlementStatus::Accepted);
    // Entity B runs the same capability with the same result.
    let env3 = make_envelope("run_003", "entity_b", "cap_x", SettlementStatus::Accepted);

    let items1 = extract_from_envelope(&env1, "2026-07-14T00:00:00Z");
    let items2 = extract_from_envelope(&env2, "2026-07-14T01:00:00Z");
    let items3 = extract_from_envelope(&env3, "2026-07-14T02:00:00Z");

    write_items(&dir, &items1);
    write_items(&dir, &items2);
    write_items(&dir, &items3);

    let loaded =
        sea_forge_capability::memory::load_memory_items(&dir.path().join("memory/items.jsonl"))
            .unwrap();

    // Entity A items dedup to one item (same dedup_key), entity B is a separate item.
    assert_eq!(
        loaded.len(),
        2,
        "expected 2 items after dedup (one per entity)"
    );

    let entity_a_item = loaded
        .iter()
        .find(|i| i.attribution.entity_id == "entity_a")
        .expect("entity_a item exists");
    assert_eq!(entity_a_item.kind, MemoryKind::Outcome);
    assert_eq!(
        entity_a_item.provenance.run_ids,
        vec!["run_001".to_string(), "run_002".to_string()],
        "provenance should merge run_ids from both extractions"
    );
    assert_eq!(
        entity_a_item.last_confirmed_at, "2026-07-14T01:00:00Z",
        "last_confirmed_at should be the latest"
    );
    assert_eq!(
        entity_a_item.created_at, "2026-07-14T00:00:00Z",
        "created_at should be the earliest"
    );

    let entity_b_item = loaded
        .iter()
        .find(|i| i.attribution.entity_id == "entity_b")
        .expect("entity_b item exists");
    assert_eq!(
        entity_b_item.provenance.run_ids,
        vec!["run_003".to_string()]
    );
}

// ── 2. Own-scope recall returns only requester's items ──

#[test]
fn own_scope_recall_returns_only_requester_items() {
    let dir = TempDir::new().unwrap();

    let env1 = make_envelope("run_001", "entity_a", "cap_x", SettlementStatus::Accepted);
    let env2 = make_envelope("run_002", "entity_b", "cap_y", SettlementStatus::Accepted);

    let items1 = extract_from_envelope(&env1, "2026-07-14T00:00:00Z");
    let items2 = extract_from_envelope(&env2, "2026-07-14T01:00:00Z");
    write_items(&dir, &items1);
    write_items(&dir, &items2);

    let path = dir.path().join("memory/items.jsonl");
    let own_results = recall_memory(
        &path,
        MemoryRecallQuery {
            query: "attempt",
            entity_id: Some("entity_a"),
            process_id: None,
            kinds: None,
            limit: 50,
        },
    )
    .unwrap();

    assert_eq!(
        own_results.len(),
        1,
        "own scope returns only requester's items"
    );
    assert_eq!(own_results[0].attribution.entity_id, "entity_a");
    // The memory_ids would be cited in recall evidence metadata.
    assert!(own_results[0].memory_id.starts_with("mem_"));
}

// ── 3. Cross-entity scope enforcement moved to authority (Task 6) ──
// scope_allows now lives in sea-forge-authority and is tested in
// conformance_m0_authority.rs::memory_scope_*.

// ── 4. Index-delete ⇒ identical recall results via fallback scan ──

#[test]
fn index_delete_yields_identical_recall_via_fallback() {
    let dir = TempDir::new().unwrap();

    for i in 0..5 {
        let entity = if i % 2 == 0 { "entity_a" } else { "entity_b" };
        let cap = format!("cap_{i}");
        let result = if i % 3 == 0 {
            SettlementStatus::Rejected
        } else {
            SettlementStatus::Accepted
        };
        let env = make_envelope(&format!("run_{i:03}"), entity, &cap, result);
        let items = extract_from_envelope(&env, &format!("2026-07-14T0{i}:00:00Z"));
        write_items(&dir, &items);
    }

    let items_path = dir.path().join("memory/items.jsonl");
    let index_path = dir.path().join("memory/index.sqlite");

    // Build the index.
    rebuild_index(&items_path, &index_path, "2026-07-14T00:00:00Z").unwrap();
    assert!(index_path.exists(), "index should exist after rebuild");

    // Query via index.
    let indexed_results = recall_with_fallback(
        &items_path,
        &index_path,
        MemoryRecallQuery {
            query: "attempt",
            entity_id: None,
            process_id: None,
            kinds: None,
            limit: 50,
        },
    )
    .unwrap();

    // Delete the index → fallback scan.
    std::fs::remove_file(&index_path).unwrap();
    assert!(!index_path.exists(), "index deleted");

    let fallback_results = recall_with_fallback(
        &items_path,
        &index_path,
        MemoryRecallQuery {
            query: "attempt",
            entity_id: None,
            process_id: None,
            kinds: None,
            limit: 50,
        },
    )
    .unwrap();

    assert_eq!(
        indexed_results.len(),
        fallback_results.len(),
        "index and fallback return same count"
    );
    for (indexed, fallback) in indexed_results.iter().zip(fallback_results.iter()) {
        assert_eq!(
            indexed.memory_id, fallback.memory_id,
            "same memory_id ordering"
        );
        assert_eq!(indexed.statement, fallback.statement, "same statement");
        assert_eq!(indexed.dedup_key, fallback.dedup_key, "same dedup_key");
    }
}

// ── 5. Extraction-error-never-fails-run: extraction never returns errors ──

#[test]
fn extraction_never_panics_or_errors() {
    // Extraction on a minimal but valid envelope returns one outcome item.
    let env = make_envelope("run_001", "entity_a", "cap_x", SettlementStatus::Accepted);
    let items = extract_from_envelope(&env, "2026-07-14T00:00:00Z");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].kind, MemoryKind::Outcome);
    assert!(!items[0].statement.is_empty());
    assert!(!items[0].dedup_key.is_empty());
    assert!(!items[0].provenance.run_ids.is_empty());
    assert!(!items[0].provenance.evidence_refs.is_empty());

    // Statement is capped at 1000 chars.
    let long_env = SemanticEnvelope {
        capability_delta: CapabilityDelta {
            attempted_capability: "x".repeat(2000),
            result: SettlementStatus::Accepted,
        },
        ..env
    };
    let long_items = extract_from_envelope(&long_env, "2026-07-14T00:00:00Z");
    assert_eq!(long_items.len(), 1);
    assert!(long_items[0].statement.len() <= 1000);
}

// ── 6. Dedup key is deterministic for same kind/statement/entity ──

#[test]
fn dedup_key_is_deterministic() {
    let key1 = compute_dedup_key(
        &MemoryKind::Outcome,
        "attempt cap_x was accepted",
        "entity_a",
    );
    let key2 = compute_dedup_key(
        &MemoryKind::Outcome,
        "attempt cap_x was accepted",
        "entity_a",
    );
    assert_eq!(key1, key2, "same inputs → same key");

    let key3 = compute_dedup_key(
        &MemoryKind::Outcome,
        "attempt cap_x was accepted",
        "entity_b",
    );
    assert_ne!(key1, key3, "different entity → different key");

    let key4 = compute_dedup_key(
        &MemoryKind::Decision,
        "attempt cap_x was accepted",
        "entity_a",
    );
    assert_ne!(key1, key4, "different kind → different key");

    // Whitespace normalization.
    let key5 = compute_dedup_key(
        &MemoryKind::Outcome,
        "  attempt cap_x was accepted  ",
        "entity_a",
    );
    assert_eq!(key1, key5, "whitespace trimmed");
}

// ── 7. Recall respects limit cap of 50 ──

#[test]
fn recall_limit_capped_at_50() {
    let dir = TempDir::new().unwrap();

    for i in 0..60 {
        let env = make_envelope(
            &format!("run_{i:03}"),
            "entity_a",
            &format!("cap_{i}"),
            SettlementStatus::Accepted,
        );
        let items = extract_from_envelope(&env, &format!("2026-07-14T{i:02}:00:00Z"));
        write_items(&dir, &items);
    }

    let path = dir.path().join("memory/items.jsonl");
    let results = recall_memory(
        &path,
        MemoryRecallQuery {
            query: "attempt",
            entity_id: Some("entity_a"),
            process_id: None,
            kinds: None,
            limit: 100, // request 100, should be capped
        },
    )
    .unwrap();

    assert_eq!(results.len(), 50, "recall capped at 50 items");
}

// ── 8. Kind filter works ──

#[test]
fn kind_filter_returns_only_matching_kind() {
    let dir = TempDir::new().unwrap();

    let env = make_envelope("run_001", "entity_a", "cap_x", SettlementStatus::Accepted);
    let items = extract_from_envelope(&env, "2026-07-14T00:00:00Z");
    write_items(&dir, &items);

    let path = dir.path().join("memory/items.jsonl");
    let outcomes = recall_memory(
        &path,
        MemoryRecallQuery {
            query: "attempt",
            entity_id: Some("entity_a"),
            process_id: None,
            kinds: Some(&[MemoryKind::Outcome]),
            limit: 50,
        },
    )
    .unwrap();
    assert_eq!(outcomes.len(), 1);

    let decisions = recall_memory(
        &path,
        MemoryRecallQuery {
            query: "attempt",
            entity_id: Some("entity_a"),
            process_id: None,
            kinds: Some(&[MemoryKind::Decision]),
            limit: 50,
        },
    )
    .unwrap();
    assert_eq!(decisions.len(), 0, "no decision items exists");
}

// ── 9. Task 7: rebuild produces a real SQLite FTS5 projection ──

fn seed_items(dir: &TempDir, count: usize) -> (std::path::PathBuf, std::path::PathBuf) {
    for i in 0..count {
        let entity = if i % 2 == 0 { "entity_a" } else { "entity_b" };
        let env = make_envelope(
            &format!("run_{i:03}"),
            entity,
            &format!("cap_{i}"),
            SettlementStatus::Accepted,
        );
        let items = extract_from_envelope(&env, &format!("2026-07-14T{i:02}:00:00Z"));
        write_items(dir, &items);
    }
    (
        dir.path().join("memory/items.jsonl"),
        dir.path().join("memory/index.sqlite"),
    )
}

#[test]
fn rebuild_writes_queryable_fts5_schema_and_source_commitment() {
    let dir = TempDir::new().unwrap();
    let (items_path, index_path) = seed_items(&dir, 4);

    rebuild_index(&items_path, &index_path, "2026-07-14T09:00:00Z").unwrap();
    assert!(index_path.is_file(), "sqlite index file created");

    let conn = rusqlite::Connection::open(&index_path).unwrap();
    let table_kind: String = conn
        .query_row(
            "SELECT type FROM sqlite_master WHERE name = 'memory_fts'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(table_kind, "table", "memory_fts must be a real FTS5 table");
    let sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE name = 'memory_fts'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(
        sql.contains("fts5"),
        "index schema must use the fts5 module, got: {sql}"
    );
    let row_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM memory_fts", [], |row| row.get(0))
        .unwrap();
    assert_eq!(row_count, 4, "one indexed row per deduplicated item");
    let stored_sha256: String = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'source_sha256'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let source_bytes = std::fs::read(&items_path).unwrap();
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    hasher.update(&source_bytes);
    assert_eq!(
        stored_sha256,
        format!("{:x}", hasher.finalize()),
        "meta must commit to the exact items.jsonl source digest"
    );
}

// ── 10. Task 7: FTS index and linear scan return identical results ──

#[test]
fn fts_index_query_is_result_equivalent_to_linear_scan_including_mid_word_substrings() {
    let dir = TempDir::new().unwrap();
    let (items_path, index_path) = seed_items(&dir, 6);
    rebuild_index(&items_path, &index_path, "2026-07-14T09:00:00Z").unwrap();

    // "ap_2" is a mid-token substring (not a word boundary for an FTS5
    // tokenizer), which is exactly the case where naive MATCH-based
    // filtering would diverge from linear `contains`.
    for query in ["attempt", "ap_2", "cap_"] {
        let indexed = query_index(
            &index_path,
            &items_path,
            &MemoryRecallQuery {
                query,
                entity_id: None,
                process_id: None,
                kinds: None,
                limit: 50,
            },
        )
        .expect("fresh index must serve the query");
        let scanned = recall_memory(
            &items_path,
            MemoryRecallQuery {
                query,
                entity_id: None,
                process_id: None,
                kinds: None,
                limit: 50,
            },
        )
        .unwrap();
        assert_eq!(
            indexed.iter().map(|i| &i.memory_id).collect::<Vec<_>>(),
            scanned.iter().map(|i| &i.memory_id).collect::<Vec<_>>(),
            "index and linear scan must return identical ordered IDs for query {query:?}"
        );
    }
}

/// Timestamp tie-break: when multiple matching items share the same
/// `last_confirmed_at`, both the FTS index path and the linear-scan fallback
/// must agree on order by ascending `memory_id`. Guards against a divergent
/// sort (e.g. one path stable, the other not) producing different IDs at the
/// same timestamp. Does not change the distinct-timestamp cases above.
#[test]
fn fts_index_and_linear_scan_agree_on_timestamp_tie_break() {
    let dir = TempDir::new().unwrap();
    // Build three items that all (a) match the query "tie_cap" and (b) share
    // the same last_confirmed_at, so memory_id ascending is the only
    // remaining ordering signal.
    let shared_ts = "2026-07-14T12:00:00Z";
    let items: Vec<MemoryItem> = ["tie_cap_one", "tie_cap_two", "tie_cap_three"]
        .into_iter()
        .flat_map(|cap| {
            let env = make_envelope(
                &format!("run_tie_{}", cap),
                "entity_a",
                cap,
                SettlementStatus::Accepted,
            );
            extract_from_envelope(&env, shared_ts)
        })
        .collect();
    write_items(&dir, &items);

    let items_path = dir.path().join("memory/items.jsonl");
    let index_path = dir.path().join("memory/index.sqlite");
    rebuild_index(&items_path, &index_path, "2026-07-14T13:00:00Z").unwrap();

    let query = MemoryRecallQuery {
        query: "tie_cap",
        entity_id: None,
        process_id: None,
        kinds: None,
        limit: 50,
    };
    let indexed = query_index(&index_path, &items_path, &query)
        .expect("fresh index must serve the tie-break query");
    let scanned = recall_memory(&items_path, query).unwrap();

    assert!(
        indexed.len() == scanned.len() && !indexed.is_empty(),
        "both paths must return the same non-empty set of tie-break items"
    );
    assert_eq!(
        indexed.iter().map(|i| &i.memory_id).collect::<Vec<_>>(),
        scanned.iter().map(|i| &i.memory_id).collect::<Vec<_>>(),
        "ordered memory IDs must match between indexed and fallback paths on timestamp ties"
    );
    // Sanity: the shared tie-break is memory_id ascending.
    let ids: Vec<&String> = indexed.iter().map(|i| &i.memory_id).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "tie-break must be memory_id ascending");
}

// ── 11. Task 7: appending an item makes the index stale ⇒ fallback ──

#[test]
fn appended_item_forces_stale_index_to_fall_back() {
    let dir = TempDir::new().unwrap();
    let (items_path, index_path) = seed_items(&dir, 2);
    rebuild_index(&items_path, &index_path, "2026-07-14T09:00:00Z").unwrap();

    assert!(
        query_index(
            &index_path,
            &items_path,
            &MemoryRecallQuery {
                query: "attempt",
                entity_id: None,
                process_id: None,
                kinds: None,
                limit: 50,
            },
        )
        .is_some(),
        "index is fresh immediately after rebuild"
    );

    // Append a new item without rebuilding the index.
    let env = make_envelope("run_999", "entity_c", "cap_new", SettlementStatus::Accepted);
    let items = extract_from_envelope(&env, "2026-07-14T23:00:00Z");
    write_items(&dir, &items);

    assert!(
        query_index(
            &index_path,
            &items_path,
            &MemoryRecallQuery {
                query: "attempt",
                entity_id: None,
                process_id: None,
                kinds: None,
                limit: 50,
            },
        )
        .is_none(),
        "stale index (source digest mismatch) must signal fallback, not silently succeed"
    );

    // recall_with_fallback must still surface the new item via linear scan.
    let results = recall_with_fallback(
        &items_path,
        &index_path,
        MemoryRecallQuery {
            query: "attempt",
            entity_id: Some("entity_c"),
            process_id: None,
            kinds: None,
            limit: 50,
        },
    )
    .unwrap();
    assert_eq!(results.len(), 1, "fallback scan sees the appended item");
}

// ── 12. Task 7: missing index falls back with no side effects ──

#[test]
fn missing_index_falls_back_without_creating_a_file() {
    let dir = TempDir::new().unwrap();
    let (items_path, index_path) = seed_items(&dir, 2);
    assert!(!index_path.exists());

    let result = query_index(
        &index_path,
        &items_path,
        &MemoryRecallQuery {
            query: "attempt",
            entity_id: None,
            process_id: None,
            kinds: None,
            limit: 50,
        },
    );
    assert!(result.is_none(), "missing index falls back");
    assert!(
        !index_path.exists(),
        "querying a missing index must not create one as a side effect"
    );
}

// ── 13. Task 7: truncated/corrupt DB falls back, never panics ──

#[test]
fn corrupt_index_falls_back_without_panicking() {
    let dir = TempDir::new().unwrap();
    let (items_path, index_path) = seed_items(&dir, 2);
    rebuild_index(&items_path, &index_path, "2026-07-14T09:00:00Z").unwrap();

    // Truncate the valid database to a handful of header bytes.
    let bytes = std::fs::read(&index_path).unwrap();
    std::fs::write(&index_path, &bytes[..16.min(bytes.len())]).unwrap();

    let result = query_index(
        &index_path,
        &items_path,
        &MemoryRecallQuery {
            query: "attempt",
            entity_id: None,
            process_id: None,
            kinds: None,
            limit: 50,
        },
    );
    assert!(result.is_none(), "truncated database must fall back");

    // Also cover non-SQLite garbage bytes.
    std::fs::write(&index_path, b"not a sqlite database at all").unwrap();
    let result = query_index(
        &index_path,
        &items_path,
        &MemoryRecallQuery {
            query: "attempt",
            entity_id: None,
            process_id: None,
            kinds: None,
            limit: 50,
        },
    );
    assert!(result.is_none(), "garbage bytes must fall back, not panic");
}

// ── 14. Task 7: tampered source-digest commitment is detected as stale ──

#[test]
fn tampered_source_digest_is_detected_as_stale() {
    let dir = TempDir::new().unwrap();
    let (items_path, index_path) = seed_items(&dir, 2);
    rebuild_index(&items_path, &index_path, "2026-07-14T09:00:00Z").unwrap();

    {
        let conn = rusqlite::Connection::open(&index_path).unwrap();
        conn.execute(
            "UPDATE meta SET value = 'deadbeef' WHERE key = 'source_sha256'",
            [],
        )
        .unwrap();
    }

    let result = query_index(
        &index_path,
        &items_path,
        &MemoryRecallQuery {
            query: "attempt",
            entity_id: None,
            process_id: None,
            kinds: None,
            limit: 50,
        },
    );
    assert!(
        result.is_none(),
        "a source digest that no longer matches items.jsonl must never be trusted"
    );
}

// ── 15. Task 7: index results are deterministically ordered, including ties ──

#[test]
fn indexed_results_are_deterministically_ordered_on_timestamp_ties() {
    let dir = TempDir::new().unwrap();
    // Same last_confirmed_at for every item forces the ordering to depend on
    // the documented tie-break (memory_id), not on SQLite/HashMap iteration
    // order.
    for i in 0..5 {
        let env = make_envelope(
            &format!("run_{i:03}"),
            "entity_a",
            &format!("cap_{i}"),
            SettlementStatus::Accepted,
        );
        let items = extract_from_envelope(&env, "2026-07-14T00:00:00Z");
        write_items(&dir, &items);
    }
    let items_path = dir.path().join("memory/items.jsonl");
    let index_path = dir.path().join("memory/index.sqlite");
    rebuild_index(&items_path, &index_path, "2026-07-14T09:00:00Z").unwrap();

    let query = MemoryRecallQuery {
        query: "attempt",
        entity_id: None,
        process_id: None,
        kinds: None,
        limit: 50,
    };
    let first = query_index(&index_path, &items_path, &query).unwrap();
    for _ in 0..3 {
        let repeat = query_index(&index_path, &items_path, &query).unwrap();
        assert_eq!(
            first.iter().map(|i| &i.memory_id).collect::<Vec<_>>(),
            repeat.iter().map(|i| &i.memory_id).collect::<Vec<_>>(),
            "repeated queries against the same index must return the same order"
        );
    }
    let mut sorted_ids: Vec<&String> = first.iter().map(|i| &i.memory_id).collect();
    sorted_ids.sort();
    assert_eq!(
        first.iter().map(|i| &i.memory_id).collect::<Vec<_>>(),
        sorted_ids,
        "ties break deterministically by memory_id"
    );
}
