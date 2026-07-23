//! Task 8: governed memory recall (M4b) — own/cross-entity/any scope
//! enforcement, exact action binding, resolvable recall evidence, limit, and
//! SQLite-index/fallback equivalence, all driven through the real `sea-forge`
//! binary.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_root(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "sea-forge-m4b-{name}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

const BASE_RULES: &str = "  - name: allow-model-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n  - name: allow-self-validate\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n";

fn policy_with_scope(root: &Path, memory_scope: &str) -> PathBuf {
    let path = root.join("policy.yaml");
    let rules = format!(
        "{BASE_RULES}  - name: allow-recall\n    verdict: allow\n    actor_role: operator\n    operation_kind: recall_memory\n    memory_scope: {memory_scope}\n"
    );
    fs::write(&path, format!("version: \"0.1\"\nrules:\n{rules}")).unwrap();
    path
}

fn run_intent(root: &Path, policy: &Path, entity: &str, process: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--entity",
            entity,
            "--process",
            process,
            "--intent",
            "Generate and validate a simple DomainForge .sea model",
        ])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "run stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[allow(clippy::too_many_arguments)]
fn recall_memory(
    root: &Path,
    policy: &Path,
    actor: &str,
    entity: Option<&str>,
    limit: Option<usize>,
) -> Output {
    let mut args = vec![
        "recall".to_string(),
        "attempt".to_string(),
        "--root".to_string(),
        root.to_str().unwrap().to_string(),
        "--policy".to_string(),
        policy.to_str().unwrap().to_string(),
        "--kind".to_string(),
        "outcome".to_string(),
        "--actor".to_string(),
        actor.to_string(),
    ];
    if let Some(entity) = entity {
        args.push("--entity".into());
        args.push(entity.into());
    }
    if let Some(limit) = limit {
        args.push("--limit".into());
        args.push(limit.to_string());
    }
    Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args(args)
        .output()
        .unwrap()
}

fn stdout_lines(output: &Output) -> Vec<serde_json::Value> {
    String::from_utf8(output.stdout.clone())
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn latest_recall_evidence(root: &Path) -> serde_json::Value {
    let dir = root.join("ledgers").join("memory-recalls");
    let mut records: Vec<serde_json::Value> = Vec::new();
    for entry in fs::read_dir(&dir).unwrap() {
        let entry = entry.unwrap();
        if entry.path().extension().is_some_and(|ext| ext == "jsonl") {
            for line in fs::read_to_string(entry.path()).unwrap().lines() {
                records.push(serde_json::from_str(line).unwrap());
            }
        }
    }
    records
        .into_iter()
        .rfind(|r| r["record_kind"] == "recall_evidence")
        .expect("a recall_evidence record must be committed")
}

// ── 1. own scope: recall with no --entity returns only the requester's items ──

#[test]
fn own_scope_implicit_self_recall_returns_only_own_items() {
    let parent = temp_root("own-implicit");
    let root = parent.join("state");
    let policy = policy_with_scope(&parent, "own");
    run_intent(&root, &policy, "team_a", "agent_1");
    run_intent(&root, &policy, "team_b", "agent_2");

    let recall = recall_memory(&root, &policy, "team_a", None, None);
    assert_eq!(
        recall.status.code(),
        Some(0),
        "recall stderr: {}",
        String::from_utf8_lossy(&recall.stderr)
    );
    let items = stdout_lines(&recall);
    assert!(!items.is_empty(), "team_a must recall its own item");
    for item in &items {
        assert_eq!(item["attribution"]["entity_id"], "team_a");
    }
    fs::remove_dir_all(parent).unwrap();
}

// ── 2. own scope: explicit cross-entity target is denied and reads nothing ──

#[test]
fn own_scope_cross_entity_target_is_denied_before_any_read() {
    let parent = temp_root("own-cross");
    let root = parent.join("state");
    let policy = policy_with_scope(&parent, "own");
    run_intent(&root, &policy, "team_a", "agent_1");
    run_intent(&root, &policy, "team_b", "agent_2");

    let recall = recall_memory(&root, &policy, "team_a", Some("team_b"), None);
    assert_ne!(
        recall.status.code(),
        Some(0),
        "cross-entity recall under own scope must not succeed"
    );
    assert!(
        recall.stdout.is_empty(),
        "a denied recall must print no memory data: {}",
        String::from_utf8_lossy(&recall.stdout)
    );
    // No recall_evidence record naming team_b's items may exist: the whole
    // ledgers/memory-recalls directory must be absent (denial happens before
    // any query, so nothing is ever committed).
    assert!(
        !root.join("ledgers").join("memory-recalls").exists(),
        "denial must not commit any recall evidence"
    );
    fs::remove_dir_all(parent).unwrap();
}

// ── 3. entity:<id> scope allows exactly the named entity, denies others ──

#[test]
fn entity_scope_allows_named_entity_denies_others() {
    let parent = temp_root("entity-scope");
    let root = parent.join("state");
    let policy = policy_with_scope(&parent, "entity:team_a");
    run_intent(&root, &policy, "team_a", "agent_1");
    run_intent(&root, &policy, "team_b", "agent_2");

    let allowed = recall_memory(&root, &policy, "operator_local", Some("team_a"), None);
    assert_eq!(allowed.status.code(), Some(0));
    let items = stdout_lines(&allowed);
    assert!(!items.is_empty());
    for item in &items {
        assert_eq!(item["attribution"]["entity_id"], "team_a");
    }

    let denied = recall_memory(&root, &policy, "operator_local", Some("team_b"), None);
    assert_ne!(denied.status.code(), Some(0));
    assert!(denied.stdout.is_empty());
    fs::remove_dir_all(parent).unwrap();
}

// ── 4. any scope recalls across entities with no --entity filter ──

#[test]
fn any_scope_recalls_across_entities() {
    let parent = temp_root("any-scope");
    let root = parent.join("state");
    let policy = policy_with_scope(&parent, "any");
    run_intent(&root, &policy, "team_a", "agent_1");
    run_intent(&root, &policy, "team_b", "agent_2");

    let recall = recall_memory(&root, &policy, "operator_local", None, None);
    assert_eq!(
        recall.status.code(),
        Some(0),
        "recall stderr: {}",
        String::from_utf8_lossy(&recall.stderr)
    );
    let items = stdout_lines(&recall);
    let entities: std::collections::BTreeSet<&str> = items
        .iter()
        .map(|item| item["attribution"]["entity_id"].as_str().unwrap())
        .collect();
    assert!(
        entities.contains("team_a") && entities.contains("team_b"),
        "any scope must recall across every entity, got: {entities:?}"
    );
    fs::remove_dir_all(parent).unwrap();
}

// ── 5. recall evidence names the exact request scope and every returned ID ──

#[test]
fn recall_evidence_names_request_scope_and_returned_ids() {
    let parent = temp_root("evidence");
    let root = parent.join("state");
    let policy = policy_with_scope(&parent, "own");
    run_intent(&root, &policy, "team_a", "agent_1");

    let recall = recall_memory(&root, &policy, "team_a", None, None);
    assert_eq!(recall.status.code(), Some(0));
    let items = stdout_lines(&recall);
    assert!(!items.is_empty());
    let printed_ids: std::collections::BTreeSet<String> = items
        .iter()
        .map(|i| i["memory_id"].as_str().unwrap().to_string())
        .collect();

    let evidence = latest_recall_evidence(&root);
    let payload = &evidence["payload"];
    assert_eq!(payload["requester_entity"], "team_a");
    assert_eq!(payload["memory_scope"], "own");
    let evidenced_ids: std::collections::BTreeSet<String> = payload["returned_memory_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        printed_ids, evidenced_ids,
        "evidence must name exactly the IDs that were printed, no more, no fewer"
    );
    fs::remove_dir_all(parent).unwrap();
}

// ── 6. limit caps the returned/evidenced set ──

#[test]
fn limit_caps_returned_and_evidenced_items() {
    let parent = temp_root("limit");
    let root = parent.join("state");
    let policy = policy_with_scope(&parent, "any");
    for i in 0..3 {
        run_intent(&root, &policy, "team_a", &format!("agent_{i}"));
    }

    let recall = recall_memory(&root, &policy, "operator_local", None, Some(1));
    assert_eq!(recall.status.code(), Some(0));
    let items = stdout_lines(&recall);
    assert_eq!(items.len(), 1, "limit=1 must cap output to one item");
    fs::remove_dir_all(parent).unwrap();
}

// ── 7. SQLite index and linear-scan fallback agree end-to-end ──

#[test]
fn cli_recall_is_identical_with_and_without_the_sqlite_index() {
    let parent = temp_root("index-equiv");
    let root = parent.join("state");
    let policy = policy_with_scope(&parent, "any");
    run_intent(&root, &policy, "team_a", "agent_1");
    run_intent(&root, &policy, "team_b", "agent_2");

    let without_index = stdout_lines(&recall_memory(&root, &policy, "operator_local", None, None));
    assert!(!without_index.is_empty());

    let rebuild = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args(["memory", "--root", root.to_str().unwrap(), "rebuild"])
        .output()
        .unwrap();
    assert_eq!(
        rebuild.status.code(),
        Some(0),
        "memory rebuild stderr: {}",
        String::from_utf8_lossy(&rebuild.stderr)
    );
    assert!(root.join("memory/index.sqlite").is_file());

    let with_index = stdout_lines(&recall_memory(&root, &policy, "operator_local", None, None));

    let ids = |items: &[serde_json::Value]| -> Vec<String> {
        items
            .iter()
            .map(|i| i["memory_id"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(
        ids(&without_index),
        ids(&with_index),
        "indexed and fallback CLI recall must return identical ordered IDs"
    );
    fs::remove_dir_all(parent).unwrap();
}
