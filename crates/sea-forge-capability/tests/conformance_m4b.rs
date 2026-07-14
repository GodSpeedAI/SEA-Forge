use sea_forge_capability::memory::{
    append_memory_items, compute_dedup_key, extract_from_envelope, rebuild_index, recall_memory,
    recall_with_fallback, scope_allows, MemoryRecallQuery,
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

// ── 3. Cross-entity denial: scope_allows rejects without explicit rule ──

#[test]
fn cross_entity_recall_denied_without_explicit_rule() {
    // scope_allows("own", ...) denies cross-entity access.
    assert!(!scope_allows("own", "entity_a", "entity_b"));

    // scope_allows("entity:<id>", ...) allows access to the named entity.
    assert!(scope_allows("entity:entity_b", "entity_a", "entity_b"));
    assert!(!scope_allows("entity:entity_b", "entity_a", "entity_c"));

    // scope_allows("any", ...) allows all.
    assert!(scope_allows("any", "entity_a", "entity_b"));

    // own scope allows same-entity.
    assert!(scope_allows("own", "entity_a", "entity_a"));

    // Unknown scope denies.
    assert!(!scope_allows("garbage", "entity_a", "entity_b"));
}

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
    let index_path = dir.path().join("memory/index.json");

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
