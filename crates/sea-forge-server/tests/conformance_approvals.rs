//! Approval conformance tests (Task 7, ADR-003 additive): `approval.list` and
//! the `approval.decide` envelope.
//!
//! The property under test is discoverability: before these methods, an
//! approver could *resolve* an approval but could not *find* one, because both
//! `approve` and `reject` require identifiers no method enumerated.

use sea_forge_core::types::{ApprovalRequest, ApprovalStatus};
use sea_forge_core::RECORD_VERSION;
use sea_forge_server::{run, ServerConfig};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, UnixStream};

async fn boot() -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let socket = root.path().join("sfwp.sock");
    let config = ServerConfig {
        identity: sea_forge_server::identity::IdentityBindings::local_operator("operator_local"),
        socket_path: socket.clone(),
        root: root.path().to_path_buf(),
        ..ServerConfig::default()
    };
    tokio::spawn(async move {
        let _ = run(config).await;
    });
    for _ in 0..200 {
        if socket.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    (root, socket)
}

struct Client {
    writer: OwnedWriteHalf,
    reader: BufReader<OwnedReadHalf>,
}

impl Client {
    async fn connect(socket: &Path) -> Self {
        let stream = UnixStream::connect(socket).await.unwrap();
        let (reader, writer) = stream.into_split();
        Self {
            writer,
            reader: BufReader::new(reader),
        }
    }

    async fn call(&mut self, value: Value) -> Value {
        let line = format!("{value}\n");
        self.writer.write_all(line.as_bytes()).await.unwrap();
        self.writer.flush().await.unwrap();
        let mut line = String::new();
        let n = tokio::time::timeout(Duration::from_secs(30), self.reader.read_line(&mut line))
            .await
            .expect("read timed out")
            .unwrap();
        assert!(n > 0, "connection closed unexpectedly");
        serde_json::from_str(line.trim()).unwrap()
    }
}

fn approval(id: &str, case_id: &str, status: ApprovalStatus, expires_at: &str) -> ApprovalRequest {
    ApprovalRequest {
        version: RECORD_VERSION.into(),
        approval_id: id.into(),
        run_id: "run-1".into(),
        case_id: case_id.into(),
        decision_id: "dec-1".into(),
        plan_item_id: "item-1".into(),
        criteria_ref: Some("criteria/settle.yaml".into()),
        criteria_sha256: Some("sha256:abc".into()),
        criteria_record_hash: None,
        job_contract_ref: None,
        requested_at: "2026-07-26T00:00:00Z".into(),
        expires_at: expires_at.into(),
        status,
        resolved_by: None,
        resolved_at: None,
        note: None,
    }
}

#[tokio::test]
async fn approval_list_is_empty_for_a_cell_with_no_journal() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let response = client.call(json!({"verb": "approval_list"})).await;
    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(response["approvals"].as_array().unwrap().len(), 0);
    assert!(
        response.get("unreadable").is_none(),
        "an absent journal is an empty inbox, not an unreadable one: {response}"
    );
}

/// The whole point of the method: the ids `approve`/`reject` demand must be
/// obtainable from the protocol itself.
#[tokio::test]
async fn approval_list_yields_the_identifiers_a_decision_requires() {
    let (root, socket) = boot().await;
    sea_forge_core::approvals::append(
        root.path(),
        &approval(
            "ap-1",
            "case-1",
            ApprovalStatus::Pending,
            "2099-01-01T00:00:00Z",
        ),
    )
    .unwrap();

    let mut client = Client::connect(&socket).await;
    let response = client.call(json!({"verb": "approval_list"})).await;
    let approvals = response["approvals"].as_array().unwrap();
    assert_eq!(approvals.len(), 1, "{response}");
    assert_eq!(approvals[0]["approval_id"], "ap-1");
    assert_eq!(approvals[0]["case_id"], "case-1");
    assert_eq!(
        approvals[0]["expired"], false,
        "an approval inside its window is not expired: {response}"
    );
    assert_eq!(
        approvals[0]["criteria_sha256"], "sha256:abc",
        "the criteria hash must travel with the approval so the UI can prove \
         which criteria the decision is judged against: {response}"
    );
}

/// A decided approval must leave the queue, or an operator would be offered the
/// same decision twice.
#[tokio::test]
async fn a_decided_approval_no_longer_appears_as_pending() {
    let (root, socket) = boot().await;
    sea_forge_core::approvals::append(
        root.path(),
        &approval(
            "ap-1",
            "case-1",
            ApprovalStatus::Pending,
            "2099-01-01T00:00:00Z",
        ),
    )
    .unwrap();
    sea_forge_core::approvals::append(
        root.path(),
        &approval(
            "ap-2",
            "case-1",
            ApprovalStatus::Pending,
            "2099-01-01T00:00:00Z",
        ),
    )
    .unwrap();
    // The decision appends a new record; it never rewrites the request.
    sea_forge_core::approvals::append(
        root.path(),
        &approval(
            "ap-1",
            "case-1",
            ApprovalStatus::Approved,
            "2099-01-01T00:00:00Z",
        ),
    )
    .unwrap();

    let mut client = Client::connect(&socket).await;
    let response = client.call(json!({"verb": "approval_list"})).await;
    let approvals = response["approvals"].as_array().unwrap();
    assert_eq!(approvals.len(), 1, "{response}");
    assert_eq!(approvals[0]["approval_id"], "ap-2");
}

/// Expiry is reported, not enforced by omission: a stuck item must still be
/// explainable from the inbox.
#[tokio::test]
async fn an_expired_approval_is_flagged_rather_than_hidden() {
    let (root, socket) = boot().await;
    sea_forge_core::approvals::append(
        root.path(),
        &approval(
            "ap-old",
            "case-1",
            ApprovalStatus::Pending,
            "2000-01-01T00:00:00Z",
        ),
    )
    .unwrap();

    let mut client = Client::connect(&socket).await;
    let response = client.call(json!({"verb": "approval_list"})).await;
    let approvals = response["approvals"].as_array().unwrap();
    assert_eq!(
        approvals.len(),
        1,
        "an expired approval must remain visible so the operator can see why \
         work is blocked: {response}"
    );
    assert_eq!(approvals[0]["expired"], true, "{response}");
}

#[tokio::test]
async fn approval_list_scopes_to_one_case_when_asked() {
    let (root, socket) = boot().await;
    for (id, case) in [("ap-a", "case-a"), ("ap-b", "case-b")] {
        sea_forge_core::approvals::append(
            root.path(),
            &approval(id, case, ApprovalStatus::Pending, "2099-01-01T00:00:00Z"),
        )
        .unwrap();
    }

    let mut client = Client::connect(&socket).await;
    let response = client
        .call(json!({"verb": "approval_list", "case_id": "case-b"}))
        .await;
    let approvals = response["approvals"].as_array().unwrap();
    assert_eq!(approvals.len(), 1, "{response}");
    assert_eq!(approvals[0]["case_id"], "case-b");
}

/// A governed decision must never be resolved on the server's guess.
#[tokio::test]
async fn an_unrecognized_decision_is_refused_rather_than_defaulted() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;

    let response = client
        .call(json!({
            "verb": "approval_decide", "actor": {"actor_id": "operator_local", "role": "operator"},
            "case_id": "case-1",
            "approval_id": "ap-1",
            "decision": "maybe",
        }))
        .await;
    assert_eq!(
        response["error_class"], "invalid_decision",
        "an unknown verdict must be refused, never defaulted to approve or reject: {response}"
    );
}

#[tokio::test]
async fn approval_methods_are_advertised_by_hello_and_describe() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;

    let hello = client
        .call(json!({"verb": "system_hello", "protocol_version": "1"}))
        .await;
    let implemented: Vec<&str> = hello["implemented_methods"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(implemented.contains(&"approval.list"), "{hello}");
    assert!(implemented.contains(&"approval.decide"), "{hello}");

    let describe = client.call(json!({"verb": "system_describe"})).await;
    let methods = describe["methods"].as_array().unwrap();
    let list_entry = methods.iter().find(|m| m["method"] == "approval.list");
    let decide_entry = methods.iter().find(|m| m["method"] == "approval.decide");
    assert_eq!(list_entry.unwrap()["class"], "inspect");
    assert_eq!(
        decide_entry.unwrap()["class"],
        "command",
        "resolving an approval has effects and must be classed as a command"
    );
}
