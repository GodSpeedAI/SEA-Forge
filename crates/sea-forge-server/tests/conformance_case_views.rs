//! Case-navigation conformance tests (Task 7, ADR-003 additive):
//! `case.list`, `case.get_overview`, `case.get_horizon`.
//!
//! Mirrors `conformance_case_authoring.rs`'s self-contained `Client`/`boot`
//! idiom (no shared test-helper crate exists yet).

use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind};
use sea_forge_planner::templates::sequential_agents_template;
use sea_forge_server::{run, ServerConfig};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, TcpListener, UnixStream};

const STUB_ENDPOINT: &str = "case-views-test-endpoint";
const TEMPLATE_REF: &str = "sequential_agents@0.1.0";

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

    std::fs::write(
        root.path().join("policy.yaml"),
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

fn policy_path(root: &Path) -> String {
    root.join("policy.yaml").to_str().unwrap().to_string()
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

/// Commit a real case through `case.commit` and return its id.
async fn commit_case(client: &mut Client, root: &Path) -> String {
    let preflight = client
        .call(json!({
            "verb": "case_preflight",
            "template_ref": TEMPLATE_REF,
            "params": {},
        }))
        .await;
    let digest = preflight["precondition"]["expected_digest"]
        .as_str()
        .expect("preflight digest")
        .to_string();

    let commit = client
        .call(json!({
            "verb": "case_commit", "actor": {"actor_id": "operator_local", "role": "operator"},
            "template_ref": TEMPLATE_REF,
            "params": {},
            "policy": policy_path(root),
            "entity": "operator_local",
            "process": "test",
            "preconditions": {
                "records": [
                    {"ref": format!("template:{TEMPLATE_REF}"), "expected_digest": digest}
                ]
            }
        }))
        .await;
    assert!(commit.get("error").is_none(), "commit failed: {commit}");
    commit["case_id"]
        .as_str()
        .expect("case_id present")
        .to_string()
}

#[tokio::test]
async fn case_list_is_empty_for_a_fresh_cell_rather_than_an_error() {
    // A cell with no `cases/` directory has *no cases*, which is a fact, not a
    // failure. `unknown != unavailable`.
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
    let response = client.call(json!({"verb": "case_list"})).await;
    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(response["cases"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn a_committed_case_appears_in_list_overview_and_horizon() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let case_id = commit_case(&mut client, root.path()).await;

    let list = client.call(json!({"verb": "case_list"})).await;
    let cases = list["cases"].as_array().unwrap();
    assert!(
        cases.iter().any(|c| c["case_id"] == case_id.as_str()),
        "committed case missing from case.list: {list}"
    );

    let overview = client
        .call(json!({"verb": "case_get_overview", "case_id": case_id}))
        .await;
    assert!(overview.get("error").is_none(), "{overview}");
    assert_eq!(overview["case_id"], case_id.as_str());
    assert_eq!(
        overview["template_ref"], TEMPLATE_REF,
        "overview must carry the originating template ref: {overview}"
    );
    assert!(
        overview["item_count"].as_u64().unwrap() > 0,
        "overview must report the plan's real item count: {overview}"
    );

    let horizon = client
        .call(json!({"verb": "case_get_horizon", "case_id": case_id}))
        .await;
    assert!(horizon.get("error").is_none(), "{horizon}");
    let items = horizon["items"].as_array().unwrap();
    assert!(
        !items.is_empty(),
        "horizon must seed a row per plan item: {horizon}"
    );
    for item in items {
        assert!(
            item.get("execution").is_some() && item.get("settlement").is_some(),
            "every horizon row must carry execution AND settlement separately: {item}"
        );
    }
}

/// Execution standing and settlement standing must never be derived from each
/// other. This is the structural guarantee behind the epic's "execution is
/// never settlement" invariant: a row whose execution completed carries a
/// settlement value drawn only from a settlement record.
#[tokio::test]
async fn execution_and_settlement_are_separate_vocabularies() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let case_id = commit_case(&mut client, root.path()).await;

    let horizon = client
        .call(json!({"verb": "case_get_horizon", "case_id": case_id}))
        .await;

    let execution_values = [
        "pending",
        "enabled",
        "active",
        "completed",
        "failed",
        "terminated",
    ];
    let settlement_values = ["unsettled", "accepted", "rejected", "escalated"];

    for item in horizon["items"].as_array().unwrap() {
        let execution = item["execution"].as_str().unwrap();
        let settlement = item["settlement"].as_str().unwrap();
        assert!(
            execution_values.contains(&execution),
            "execution used a non-execution value {execution:?}: {item}"
        );
        assert!(
            settlement_values.contains(&settlement),
            "settlement used a non-settlement value {settlement:?}: {item}"
        );
        // The vocabularies are disjoint, so a value can never cross domains —
        // "completed" is not a settlement outcome and "accepted" is not an
        // execution outcome.
        assert!(
            !settlement_values.contains(&execution),
            "execution borrowed a settlement word: {item}"
        );
        assert!(
            !execution_values.contains(&settlement),
            "settlement borrowed an execution word: {item}"
        );
    }
}

#[tokio::test]
async fn an_unknown_case_is_typed_not_found_not_a_generic_error() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;

    let overview = client
        .call(json!({"verb": "case_get_overview", "case_id": "case-does-not-exist"}))
        .await;
    assert_eq!(
        overview["error_class"], "not_found",
        "a missing case must be typed so the UI can distinguish it from a broken record: {overview}"
    );

    let horizon = client
        .call(json!({"verb": "case_get_horizon", "case_id": "case-does-not-exist"}))
        .await;
    assert_eq!(horizon["error_class"], "not_found", "{horizon}");
}

/// A case id is used to build a filesystem path, so a traversal attempt must
/// be refused before it reaches the disk — not resolved and then found empty.
#[tokio::test]
async fn a_traversing_case_id_is_refused_rather_than_resolved() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;

    for hostile in ["../../etc", "..", "a/b", "case/../../.."] {
        let response = client
            .call(json!({"verb": "case_get_overview", "case_id": hostile}))
            .await;
        assert_eq!(
            response["error_class"], "not_found",
            "traversal id {hostile:?} must be refused: {response}"
        );
    }
}

/// The new methods must be discoverable through negotiation, or the workbench
/// has no lawful way to learn they exist.
#[tokio::test]
async fn case_view_methods_are_advertised_by_hello_and_describe() {
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
    for method in ["case.list", "case.get_overview", "case.get_horizon"] {
        assert!(
            implemented.contains(&method),
            "{method} missing from hello's catalog: {hello}"
        );
    }

    let describe = client.call(json!({"verb": "system_describe"})).await;
    let methods = describe["methods"].as_array().unwrap();
    for method in ["case.list", "case.get_overview", "case.get_horizon"] {
        let entry = methods
            .iter()
            .find(|m| m["method"] == method)
            .unwrap_or_else(|| panic!("{method} missing from describe: {describe}"));
        assert_eq!(
            entry["class"], "inspect",
            "case views are read-only projections and must be classed inspect: {entry}"
        );
    }
}
