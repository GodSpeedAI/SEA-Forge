//! Task 14B: server `ask` adapter over the shared Thoth service, and
//! version-skew protection for the additive `Request::Ask` variant
//! (ADR-003 shape (3)).

use sea_forge_server::{handle_request, Request, ServerConfig, ServerState};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn temp_root(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("sea-forge-server-ask-{name}-{nonce}"));
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn init_state(root: &Path) -> Arc<ServerState> {
    Arc::new(
        ServerState::new(ServerConfig {
            root: root.to_path_buf(),
            ..Default::default()
        })
        .unwrap(),
    )
}

fn init_self_model(root: &Path) {
    let inputs = sea_forge_self_model::store::RebuildInputs {
        cell_id: "cell_test",
        active_extensions: vec![],
        environments_present: vec![],
        probes: vec![],
        sandbox_classes_available: vec!["local".into()],
        created_at: "2026-07-23T00:00:00Z",
        capability_projection_sha256: "sha256:test",
        actor_id: "operator_local",
    };
    sea_forge_self_model::store::ensure_init(root, &inputs).unwrap();
}

fn write_full_grant_policy(root: &Path, actor_role: &str) {
    let dir = root.join("authority");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("active-policy.json"),
        format!(
            r#"
version: "0.1"
rules: []
policy_surfaces:
  self_disclosure:
    mode: deny-by-default
    grants:
      - name: full-grant
        actor_role: {actor_role}
        claim_classes:
          - declared_capability
          - installed_capability
          - demonstrated_capability
"#
        ),
    )
    .unwrap();
}

#[tokio::test]
async fn server_ask_allowed_returns_answer() {
    let root = temp_root("allowed");
    init_self_model(&root);
    write_full_grant_policy(&root, "operator_local");
    let composed = sea_forge_self_model::load_composed(&sea_forge_self_model::bundled()).unwrap();
    let name = composed.concepts().into_iter().next().unwrap();
    let state = init_state(&root);

    let response = handle_request(
        Request::Ask {
            kind: "ask_capability".into(),
            subject: name,
            purpose: "test".into(),
            case: None,
            actor_id: "operator_local".into(),
        },
        &state,
    )
    .await;

    assert_eq!(response["disposition"], "answered");
    std::fs::remove_dir_all(root).ok();
}

#[tokio::test]
async fn server_ask_denied_leaks_no_restricted_facts() {
    let root = temp_root("denied");
    init_self_model(&root);
    // No policy file: absent surface denies all (§8.2).
    let state = init_state(&root);

    let response = handle_request(
        Request::Ask {
            kind: "ask_capability".into(),
            subject: "some_subject".into(),
            purpose: "test".into(),
            case: None,
            actor_id: "operator_local".into(),
        },
        &state,
    )
    .await;

    assert_eq!(response["disposition"], "denied");
    assert!(response["claims"]
        .as_array()
        .is_none_or(|claims| claims.is_empty()));
    std::fs::remove_dir_all(root).ok();
}

#[tokio::test]
async fn server_ask_unknown_kind_errors_without_panicking() {
    let root = temp_root("unknown_kind");
    init_self_model(&root);
    let state = init_state(&root);

    let response = handle_request(
        Request::Ask {
            kind: "bogus_kind".into(),
            subject: "x".into(),
            purpose: "test".into(),
            case: None,
            actor_id: "operator_local".into(),
        },
        &state,
    )
    .await;

    assert!(response["error"].is_string());
    std::fs::remove_dir_all(root).ok();
}

/// CLI and server both call `sea_forge_thoth::service::ask` directly (T14A);
/// this proves they leave the same ledgered lineage shape (record_kind
/// sequence: question → plan → decision → answer) for equivalent inputs,
/// rather than each ingress building its own governance path.
#[tokio::test]
async fn server_ask_ledgers_the_full_question_to_answer_chain() {
    let root = temp_root("lineage");
    init_self_model(&root);
    write_full_grant_policy(&root, "operator_local");
    let composed = sea_forge_self_model::load_composed(&sea_forge_self_model::bundled()).unwrap();
    let name = composed.concepts().into_iter().next().unwrap();
    let state = init_state(&root);

    handle_request(
        Request::Ask {
            kind: "ask_capability".into(),
            subject: name,
            purpose: "test".into(),
            case: None,
            actor_id: "operator_local".into(),
        },
        &state,
    )
    .await;

    let stream = sea_forge_ledger::LedgerStream::open(&root, "thoth-asks", "test_reader").unwrap();
    let entries = stream.read_entries().unwrap();
    let kinds: Vec<&str> = entries.iter().map(|e| e.record_kind.as_str()).collect();
    assert_eq!(
        kinds,
        vec![
            "self_disclosure_question",
            "self_disclosure_plan",
            "self_disclosure_decision",
            "self_disclosure_answer",
        ]
    );
    std::fs::remove_dir_all(root).ok();
}

/// ADR-003 shape (3): an old reader that does not know the `ask` verb tag
/// must fail deserialization cleanly (typed `Err`), never panic and never
/// silently misroute to a different variant. Since the workspace binary
/// always carries the current `Request` enum, this exercises the general
/// unknown-tag mechanism the additive `Ask` variant relies on for
/// version-skew safety against a genuinely older server build.
#[test]
fn unknown_request_verb_fails_clean_not_panic() {
    let result: Result<Request, _> =
        serde_json::from_str(r#"{"verb":"totally_unrecognized_future_verb"}"#);
    assert!(result.is_err());
}

/// The `ask` verb deserializes through the wire tag exactly as named in
/// ADR-003 (`enum Request { ..., Ask }`, `#[serde(rename_all = "snake_case")]`).
#[test]
fn ask_request_wire_tag_is_snake_case_ask() {
    let request: Request = serde_json::from_str(
        r#"{"verb":"ask","kind":"ask_capability","subject":"x","purpose":"test","actor_id":"operator_local"}"#,
    )
    .unwrap();
    assert!(matches!(request, Request::Ask { .. }));
}
