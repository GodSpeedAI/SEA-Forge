//! Case-authoring conformance tests (Task 6, ADR-003 additive):
//! `case.entry_options`, `case.preflight`, `case.commit`.
//!
//! Mirrors `conformance_sfwp.rs`'s self-contained `Client`/`boot` idiom (no
//! shared test-helper crate exists yet).

use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind};
use sea_forge_planner::templates::sequential_agents_template;
use sea_forge_server::{run, ServerConfig};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, TcpListener, UnixStream};

const STUB_ENDPOINT: &str = "case-authoring-test-endpoint";
const TEMPLATE_REF: &str = "sequential_agents@0.1.0";

/// A stub OpenAI-compatible endpoint answering every request identically
/// (copied from `conformance_sfwp.rs::stub_endpoint` — no shared helper crate).
fn stub_endpoint() -> AgentEndpointConfig {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    let listener = TcpListener::from_std(listener).unwrap();
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let mut request = vec![0_u8; 32_768];
                let _ = stream.read(&mut request).await;
                let body =
                    br#"{"choices":[{"message":{"content":"step complete"}}],"usage":{"total_tokens":1}}"#;
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes()).await;
                let _ = stream.write_all(body).await;
            });
        }
    });
    AgentEndpointConfig {
        id: STUB_ENDPOINT.into(),
        kind: ProviderKind::OpenAiCompatible,
        base_url: Some(format!("http://127.0.0.1:{}/", address.port())),
        argv: vec![],
        env: vec![],
        credential_ref: None,
        default_model: Some("test-model".into()),
        allow_loopback_test: true,
        max_request_bytes: 16_384,
        max_response_bytes: 16_384,
        timeout_secs: 5,
        status: None,
        transcript_retention: None,
    }
}

/// Boot a server on a temp-root socket with the stub endpoint registered and
/// `sequential_agents@0.1.0` materialized under `templates/` (as
/// `case.entry_options` would find it after a CLI `store_builtin` run — here
/// written directly since this test only needs the file to exist, not the
/// CLI's materialization path). Returns `(tempdir, socket_path)`.
async fn boot() -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let socket = root.path().join("sfwp.sock");
    let agent = AgentConfig {
        endpoints: vec![stub_endpoint()],
        ..AgentConfig::default()
    };
    let config = ServerConfig {
        identity: sea_forge_server::identity::IdentityBindings::local_operator("operator_local"),
        socket_path: socket.clone(),
        root: root.path().to_path_buf(),
        agent,
        ..ServerConfig::default()
    };

    let templates_dir = root.path().join("templates");
    std::fs::create_dir_all(&templates_dir).unwrap();
    let template = sequential_agents_template(STUB_ENDPOINT).unwrap();
    std::fs::write(
        templates_dir.join(format!("{TEMPLATE_REF}.yaml")),
        serde_yaml::to_string(&template).unwrap(),
    )
    .unwrap();

    let policy_path = root.path().join("policy.yaml");
    std::fs::write(
        &policy_path,
        "version: \"0.1\"\npolicy_surfaces:\n  external_api:\n    mode: deny-by-default\n    allow_hosts: [127.0.0.1]\nrules:\n  - name: allow-agent-task\n    verdict: allow\n    actor_role: operator\n    operation_kind: agent_task\n",
    )
    .unwrap();

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

fn policy_path(_root: &Path) -> String {
    // F-16: the policy reference is a cell-relative spelling.
    "policy.yaml".to_string()
}

/// A minimal line-delimited JSON client over the Unix socket (copied from
/// `conformance_sfwp.rs::Client` — no shared helper crate).
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
        let n = tokio::time::timeout(Duration::from_secs(10), self.reader.read_line(&mut line))
            .await
            .expect("read timed out")
            .unwrap();
        assert!(n > 0, "connection closed unexpectedly");
        serde_json::from_str(line.trim()).unwrap()
    }
}

#[tokio::test]
async fn entry_options_lists_materialized_template_honestly() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let response = client.call(json!({"verb": "case_entry_options"})).await;
    let templates = response["templates"].as_array().unwrap();
    assert!(
        templates.iter().any(|t| t["template_ref"] == TEMPLATE_REF),
        "expected materialized template in entry_options: {response}"
    );
}

#[tokio::test]
async fn entry_options_is_empty_not_fabricated_when_no_templates_exist() {
    // A fresh root with no `templates/` dir at all — `unknown != unavailable`:
    // an honest empty list, never a fabricated built-in catalog.
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
    let mut client = Client::connect(&socket).await;
    let response = client.call(json!({"verb": "case_entry_options"})).await;
    assert_eq!(response["templates"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn preflight_ok_returns_items_and_a_reusable_precondition() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let response = client
        .call(json!({
            "verb": "case_preflight",
            "template_ref": TEMPLATE_REF,
            "params": {},
        }))
        .await;
    assert_eq!(response["ok"], true, "expected ok preflight: {response}");
    assert!(!response["items"].as_array().unwrap().is_empty());
    assert_eq!(
        response["precondition"]["ref"],
        format!("template:{TEMPLATE_REF}")
    );
    assert!(response["precondition"]["expected_digest"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
}

#[tokio::test]
async fn preflight_unknown_template_reports_errors_not_a_transport_error() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let response = client
        .call(json!({
            "verb": "case_preflight",
            "template_ref": "does_not_exist@9.9.9",
            "params": {},
        }))
        .await;
    assert_eq!(response["ok"], false);
    assert!(!response["errors"].as_array().unwrap().is_empty());
    assert!(response["precondition"].is_null());
}

#[tokio::test]
async fn commit_with_matching_precondition_creates_a_real_case() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;

    let preflight = client
        .call(json!({
            "verb": "case_preflight",
            "template_ref": TEMPLATE_REF,
            "params": {},
        }))
        .await;
    let expected_digest = preflight["precondition"]["expected_digest"]
        .as_str()
        .unwrap()
        .to_string();

    let commit = client
        .call(json!({
            "verb": "case_commit", "actor": {"actor_id": "operator_local", "role": "operator"},
            "template_ref": TEMPLATE_REF,
            "params": {},
            "policy": policy_path(root.path()),
            "entity": "operator_local",
            "process": "test",
            "preconditions": {
                "records": [
                    {"ref": format!("template:{TEMPLATE_REF}"), "expected_digest": expected_digest}
                ]
            }
        }))
        .await;
    assert!(commit.get("error").is_none(), "commit failed: {commit}");
    let case_id = commit["case_id"].as_str().expect("case_id present");

    let status = client
        .call(json!({"verb": "status", "case_id": case_id}))
        .await;
    assert_eq!(status["case_id"], case_id);
}

#[tokio::test]
async fn commit_with_stale_precondition_is_rejected_with_no_case_created() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;

    let cases_before = std::fs::read_dir(root.path().join("cases"))
        .map(|d| d.count())
        .unwrap_or(0);

    let commit = client
        .call(json!({
            "verb": "case_commit", "actor": {"actor_id": "operator_local", "role": "operator"},
            "template_ref": TEMPLATE_REF,
            "params": {},
            "policy": policy_path(root.path()),
            "entity": "operator_local",
            "process": "test",
            "preconditions": {
                "records": [
                    {"ref": format!("template:{TEMPLATE_REF}"), "expected_digest": "sha256:deadbeef"}
                ]
            }
        }))
        .await;

    assert_eq!(commit["outcome"], "rejected_as_stale");
    assert_eq!(commit["code"], "precondition_failed");
    assert!(!commit["changed_records"].as_array().unwrap().is_empty());

    let cases_after = std::fs::read_dir(root.path().join("cases"))
        .map(|d| d.count())
        .unwrap_or(0);
    assert_eq!(
        cases_before, cases_after,
        "stale-rejected commit must not create a case"
    );
}

#[tokio::test]
async fn commit_outcome_is_recoverable_via_request_get_status() {
    // Proof scenario 6 (no duplicate commit reachable): a client that lost the
    // response to a correlated `case.commit` recovers the terminal outcome via
    // `request.get_status` instead of resubmitting — reusing the same
    // correlation store `Submit` already participates in.
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;

    let commit = client
        .call(json!({
            "verb": "case_commit", "actor": {"actor_id": "operator_local", "role": "operator"},
            "template_ref": TEMPLATE_REF,
            "params": {},
            "policy": policy_path(root.path()),
            "entity": "operator_local",
            "process": "test",
            "request_id": "req-case-commit-1",
        }))
        .await;
    let case_id = commit["case_id"]
        .as_str()
        .expect("case_id present")
        .to_string();

    let recovered = client
        .call(json!({"verb": "request_get_status", "request_id": "req-case-commit-1"}))
        .await;
    assert_eq!(recovered["status"], "completed");
    assert_eq!(recovered["method"], "case.commit");
    assert_eq!(recovered["outcome"]["case_id"], case_id);
}
