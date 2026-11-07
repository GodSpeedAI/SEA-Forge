#![cfg(unix)]
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
        sea_forge_core::yaml::to_string(&template).unwrap(),
    )
    .unwrap();

    let policy_path = root.path().join("policy.yaml");
    std::fs::write(
        &policy_path,
        "version: \"0.1\"\npolicy_surfaces:\n  external_api:\n    mode: deny-by-default\n    allow_hosts: [127.0.0.1]\nrules:\n  - name: allow-agent-task\n    verdict: allow\n    actor_role: operator\n    operation_kind: agent_task\n  - name: allow-propose\n    verdict: allow\n    actor_role: operator\n    operation_kind: discretionary_task_add\n",
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
            "request_id": "req-matching-precondition",
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
            "request_id": "req-stale-precondition",
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

// ---------------------------------------------------------------------------
// plan: digest on preflight, case.add_item (external_ref, review marker,
// request_id, dry_run, plan preconditions), digests on case reads
// ---------------------------------------------------------------------------

#[tokio::test]
async fn preflight_returns_plan_precondition() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let call = json!({"verb": "case_preflight", "template_ref": TEMPLATE_REF, "params": {}});
    let first = client.call(call.clone()).await;
    assert_eq!(first["ok"], true, "{first}");
    assert_eq!(
        first["plan_precondition"]["ref"],
        format!("plan:{TEMPLATE_REF}")
    );
    let digest = first["plan_precondition"]["expected_digest"]
        .as_str()
        .unwrap();
    assert!(digest.starts_with("sha256:"));
    // The plan digest is distinct from the template-bytes digest, and stable.
    assert_ne!(digest, first["precondition"]["expected_digest"]);
    let second = client.call(call).await;
    assert_eq!(second["plan_precondition"], first["plan_precondition"]);
    // No plan to pin when the template does not exist.
    let missing = client
        .call(json!({"verb": "case_preflight", "template_ref": "nope@1.0.0", "params": {}}))
        .await;
    assert!(missing["plan_precondition"].is_null());
}

fn extra_item(id: &str) -> Value {
    json!({
        "plan_item_id": id,
        "name": "extra",
        "operations": [],
        "entry_criteria": [], "exit_criteria": [],
        "settlement_criteria": {"required_artifacts": [], "required_declarations": []},
        "item_kind": "human_task",
        "markers": {"required": false},
        "max_instances": 1,
        "depends_on": []
    })
}

async fn commit_case(client: &mut Client, root: &Path, request_id: &str) -> String {
    let commit = client
        .call(json!({
            "verb": "case_commit", "actor": {"actor_id": "operator_local", "role": "operator"},
            "template_ref": TEMPLATE_REF,
            "params": {},
            "policy": policy_path(root),
            "entity": "operator_local",
            "process": "test",
            "request_id": request_id,
        }))
        .await;
    commit["case_id"]
        .as_str()
        .unwrap_or_else(|| panic!("commit failed: {commit}"))
        .to_string()
}

fn add_item_request(root: &Path, case_id: &str, item: Value, extra: Value) -> Value {
    let mut request = json!({
        "verb": "case_add_item",
        "actor": {"actor_id": "operator_local", "role": "operator"},
        "case_id": case_id,
        "item": item,
        "policy": policy_path(root),
    });
    for (key, value) in extra.as_object().unwrap() {
        request[key] = value.clone();
    }
    request
}

fn plan_json(root: &Path, case_id: &str) -> Value {
    serde_json::from_slice(
        &std::fs::read(root.join("cases").join(case_id).join("plan.json")).unwrap(),
    )
    .unwrap()
}

fn plan_item<'a>(plan: &'a Value, id: &str) -> Option<&'a Value> {
    plan["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["plan_item_id"] == id)
}

#[tokio::test]
async fn add_item_carries_external_ref() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let case_id = commit_case(&mut client, root.path(), "req-commit-ext").await;
    let mut item = extra_item("from_issue");
    item["external_ref"] = json!({
        "kind": "github_issue",
        "id": "GodSpeedAI/horizon-sandbox#7",
        "url": "https://github.com/GodSpeedAI/horizon-sandbox/issues/7",
    });
    let reply = client
        .call(add_item_request(
            root.path(),
            &case_id,
            item.clone(),
            json!({"request_id": "github:GodSpeedAI/horizon-sandbox#7"}),
        ))
        .await;
    assert_eq!(reply["ok"], true, "{reply}");
    let plan = plan_json(root.path(), &case_id);
    assert_eq!(
        plan_item(&plan, "from_issue").unwrap()["external_ref"],
        item["external_ref"]
    );
    // Items without one stay free of the key.
    assert!(plan_item(&plan, "from_issue").is_some());
    assert!(plan["items"][0].get("external_ref").is_none());
}

#[tokio::test]
async fn add_item_same_request_id_replays() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let case_id = commit_case(&mut client, root.path(), "req-commit-replay").await;
    let request = add_item_request(
        root.path(),
        &case_id,
        extra_item("once"),
        json!({"request_id": "handoff:draft-1"}),
    );
    let first = client.call(request.clone()).await;
    assert_eq!(first["ok"], true, "{first}");
    let before = plan_json(root.path(), &case_id)["items"]
        .as_array()
        .unwrap()
        .len();
    let second = client.call(request.clone()).await;
    let third = client.call(request).await;
    assert_eq!(second, first);
    assert_eq!(third, first);
    let plan = plan_json(root.path(), &case_id);
    assert_eq!(plan["items"].as_array().unwrap().len(), before);
    assert_eq!(
        plan["items"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| item["plan_item_id"] == "once")
            .count(),
        1
    );
}

#[tokio::test]
async fn add_item_changed_payload_is_request_id_reused() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let case_id = commit_case(&mut client, root.path(), "req-commit-reuse").await;
    let first = client
        .call(add_item_request(
            root.path(),
            &case_id,
            extra_item("first"),
            json!({"request_id": "handoff:draft-2"}),
        ))
        .await;
    assert_eq!(first["ok"], true, "{first}");
    let reused = client
        .call(add_item_request(
            root.path(),
            &case_id,
            extra_item("second"),
            json!({"request_id": "handoff:draft-2"}),
        ))
        .await;
    assert_eq!(reused["error_class"], "request_id_reused", "{reused}");
    assert!(plan_item(&plan_json(root.path(), &case_id), "second").is_none());
}

#[tokio::test]
async fn add_item_keeps_review_marker() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let case_id = commit_case(&mut client, root.path(), "req-commit-review").await;
    let mut item = extra_item("review_it");
    item["markers"] = json!({"required": false, "review": true});
    let reply = client
        .call(add_item_request(
            root.path(),
            &case_id,
            item,
            json!({"request_id": "handoff:review-1"}),
        ))
        .await;
    assert_eq!(reply["ok"], true, "{reply}");
    let plan = plan_json(root.path(), &case_id);
    assert_eq!(
        plan_item(&plan, "review_it").unwrap()["markers"]["review"],
        true
    );
}

#[tokio::test]
async fn add_item_dry_run_commits_nothing() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let case_id = commit_case(&mut client, root.path(), "req-commit-dry").await;
    let case_dir = root.path().join("cases").join(&case_id);
    let plan_before = std::fs::read(case_dir.join("plan.json")).unwrap();
    let events_before = std::fs::read(case_dir.join("case-events.jsonl")).unwrap();

    // A dry run needs no request_id; with one, it must not consume it.
    let request = add_item_request(
        root.path(),
        &case_id,
        extra_item("preview"),
        json!({"dry_run": true, "request_id": "handoff:dry-1"}),
    );
    let dry = client.call(request.clone()).await;
    assert_eq!(dry["ok"], true, "{dry}");
    assert_eq!(dry["dry_run"], true);
    assert_eq!(dry["outcome"], "would_add");
    assert_eq!(dry["item"]["plan_item_id"], "preview");
    assert_eq!(dry["item"]["proposed_by"], "operator_local");
    assert_eq!(dry["plan_precondition"]["ref"], format!("plan:{case_id}"));
    assert!(dry["plan_precondition"]["expected_digest"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    let again = client.call(request).await;
    assert_eq!(again["plan_precondition"], dry["plan_precondition"]);
    let mut without_id = add_item_request(
        root.path(),
        &case_id,
        extra_item("preview"),
        json!({"dry_run": true}),
    );
    without_id.as_object_mut().unwrap().remove("request_id");
    let anonymous = client.call(without_id).await;
    assert_eq!(anonymous["plan_precondition"], dry["plan_precondition"]);

    assert_eq!(
        std::fs::read(case_dir.join("plan.json")).unwrap(),
        plan_before
    );
    assert_eq!(
        std::fs::read(case_dir.join("case-events.jsonl")).unwrap(),
        events_before
    );

    // The request_id is still free: the real call is a first attempt, not a
    // replay of the preview.
    let real = client
        .call(add_item_request(
            root.path(),
            &case_id,
            extra_item("preview"),
            json!({"request_id": "handoff:dry-1"}),
        ))
        .await;
    assert_eq!(real["ok"], true, "{real}");
    assert!(real.get("dry_run").is_none(), "{real}");
    assert!(plan_item(&plan_json(root.path(), &case_id), "preview").is_some());

    // A dry run is refused like the real call: duplicate id fails validation.
    let refused = client
        .call(add_item_request(
            root.path(),
            &case_id,
            extra_item("preview"),
            json!({"dry_run": true}),
        ))
        .await;
    assert!(refused.get("error").is_some(), "{refused}");
}

#[tokio::test]
async fn add_item_with_stale_plan_digest_is_refused() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let case_id = commit_case(&mut client, root.path(), "req-commit-stale").await;

    // Fresh digest: the real call goes through.
    let dry = client
        .call(add_item_request(
            root.path(),
            &case_id,
            extra_item("a"),
            json!({"dry_run": true}),
        ))
        .await;
    let digest = dry["plan_precondition"].clone();
    let ok = client
        .call(add_item_request(
            root.path(),
            &case_id,
            extra_item("a"),
            json!({"request_id": "add-a", "preconditions": {"records": [digest]}}),
        ))
        .await;
    assert_eq!(ok["ok"], true, "{ok}");

    // The plan changes between preview and confirm.
    let dry_b = client
        .call(add_item_request(
            root.path(),
            &case_id,
            extra_item("b"),
            json!({"dry_run": true}),
        ))
        .await;
    let digest_b = dry_b["plan_precondition"].clone();
    let other = client
        .call(add_item_request(
            root.path(),
            &case_id,
            extra_item("c"),
            json!({"request_id": "add-c"}),
        ))
        .await;
    assert_eq!(other["ok"], true, "{other}");
    let events_before = std::fs::read(
        root.path()
            .join("cases")
            .join(&case_id)
            .join("case-events.jsonl"),
    )
    .unwrap();

    let stale = client
        .call(add_item_request(
            root.path(),
            &case_id,
            extra_item("b"),
            json!({"request_id": "add-b", "preconditions": {"records": [digest_b]}}),
        ))
        .await;
    assert_eq!(stale["outcome"], "rejected_as_stale", "{stale}");
    assert_eq!(stale["code"], "precondition_failed");
    assert_eq!(
        stale["changed_records"][0]["ref"],
        format!("plan:{case_id}")
    );
    assert!(plan_item(&plan_json(root.path(), &case_id), "b").is_none());
    assert_eq!(
        std::fs::read(
            root.path()
                .join("cases")
                .join(&case_id)
                .join("case-events.jsonl")
        )
        .unwrap(),
        events_before
    );
}

#[tokio::test]
async fn case_overview_returns_a_digest_that_tracks_the_case() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let case_id = commit_case(&mut client, root.path(), "req-commit-digest").await;
    let read = || json!({"verb": "case_get_overview", "case_id": case_id});
    let first = client.call(read()).await;
    assert_eq!(first["digest"]["ref"], format!("case:{case_id}"), "{first}");
    assert!(first["digest"]["expected_digest"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert_eq!(client.call(read()).await["digest"], first["digest"]);
    let added = client
        .call(add_item_request(
            root.path(),
            &case_id,
            extra_item("moves_digest"),
            json!({"request_id": "add-digest"}),
        ))
        .await;
    assert_eq!(added["ok"], true, "{added}");
    let after = client.call(read()).await;
    assert_ne!(after["digest"], first["digest"]);
}

// ---------------------------------------------------------------------------
// Blind-critic fixes: replay protection, locking, durable-write-free previews,
// consistent digests, normalized preview item
// ---------------------------------------------------------------------------

/// Every file under `root` whose path mentions `authority`, with its bytes.
fn authority_snapshot(root: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    fn walk(dir: &Path, root: &Path, out: &mut std::collections::BTreeMap<String, Vec<u8>>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, root, out);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                if relative.contains("authority") {
                    out.insert(relative, std::fs::read(&path).unwrap());
                }
            }
        }
    }
    let mut out = std::collections::BTreeMap::new();
    walk(root, root, &mut out);
    out
}

#[tokio::test]
async fn dry_run_field_on_another_verb_does_not_skip_replay_protection() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let request = |process: &str| {
        json!({
            "verb": "case_commit", "actor": {"actor_id": "operator_local", "role": "operator"},
            "template_ref": TEMPLATE_REF, "params": {},
            "policy": policy_path(root.path()), "entity": "operator_local",
            "process": process, "request_id": "req-dry-run-smuggle", "dry_run": true,
        })
    };
    let first = client.call(request("test")).await;
    assert!(first["case_id"].is_string(), "{first}");
    let replay = client.call(request("test")).await;
    assert_eq!(replay, first, "same id and payload must replay");
    let reused = client.call(request("other")).await;
    assert_eq!(reused["error_class"], "request_id_reused", "{reused}");
    let cases = std::fs::read_dir(root.path().join("cases"))
        .unwrap()
        .count();
    assert_eq!(cases, 1, "exactly one case");
}

#[tokio::test]
async fn add_item_concurrent_confirmations_of_one_preview_commit_once() {
    let (root, socket) = boot().await;
    let mut setup = Client::connect(&socket).await;
    let case_id = commit_case(&mut setup, root.path(), "req-commit-race").await;
    let mut digests = Vec::new();
    for id in ["race_a", "race_b"] {
        let dry = setup
            .call(add_item_request(
                root.path(),
                &case_id,
                extra_item(id),
                json!({"dry_run": true}),
            ))
            .await;
        digests.push(dry["plan_precondition"].clone());
    }
    let (mut a, mut b) = (
        Client::connect(&socket).await,
        Client::connect(&socket).await,
    );
    let req_a = add_item_request(
        root.path(),
        &case_id,
        extra_item("race_a"),
        json!({"request_id": "race-a", "preconditions": {"records": [digests[0].clone()]}}),
    );
    let req_b = add_item_request(
        root.path(),
        &case_id,
        extra_item("race_b"),
        json!({"request_id": "race-b", "preconditions": {"records": [digests[1].clone()]}}),
    );
    let (ra, rb) = tokio::join!(a.call(req_a), b.call(req_b));
    let ok = [&ra, &rb].iter().filter(|r| r["ok"] == true).count();
    let stale = [&ra, &rb]
        .iter()
        .filter(|r| r["outcome"] == "rejected_as_stale")
        .count();
    assert_eq!((ok, stale), (1, 1), "a: {ra} b: {rb}");
    let plan = plan_json(root.path(), &case_id);
    let added = ["race_a", "race_b"]
        .iter()
        .filter(|id| plan_item(&plan, id).is_some())
        .count();
    assert_eq!(added, 1);
}

#[tokio::test]
async fn add_item_duplicate_after_preview_is_stale_not_a_validation_error() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let case_id = commit_case(&mut client, root.path(), "req-commit-dup").await;
    let dry = client
        .call(add_item_request(
            root.path(),
            &case_id,
            extra_item("dup"),
            json!({"dry_run": true}),
        ))
        .await;
    let digest = dry["plan_precondition"].clone();
    let other = client
        .call(add_item_request(
            root.path(),
            &case_id,
            extra_item("dup"),
            json!({"request_id": "dup-first"}),
        ))
        .await;
    assert_eq!(other["ok"], true, "{other}");
    let stale = client
        .call(add_item_request(
            root.path(),
            &case_id,
            extra_item("dup"),
            json!({"request_id": "dup-second", "preconditions": {"records": [digest]}}),
        ))
        .await;
    assert_eq!(stale["outcome"], "rejected_as_stale", "{stale}");
}

#[tokio::test]
async fn add_item_dry_run_and_stale_rejection_write_no_authority_records() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let case_id = commit_case(&mut client, root.path(), "req-commit-authority").await;
    let before = authority_snapshot(root.path());
    assert!(!before.is_empty(), "the snapshot must see authority files");

    let dry = client
        .call(add_item_request(
            root.path(),
            &case_id,
            extra_item("quiet"),
            json!({"dry_run": true}),
        ))
        .await;
    assert_eq!(dry["ok"], true, "{dry}");
    assert_eq!(
        authority_snapshot(root.path()),
        before,
        "dry run wrote authority state"
    );

    let stale = client
        .call(add_item_request(
            root.path(),
            &case_id,
            extra_item("quiet"),
            json!({"request_id": "quiet-stale", "preconditions": {"records": [
                {"ref": format!("plan:{case_id}"), "expected_digest": "sha256:00"}
            ]}}),
        ))
        .await;
    assert_eq!(stale["outcome"], "rejected_as_stale", "{stale}");
    assert_eq!(
        authority_snapshot(root.path()),
        before,
        "stale rejection wrote authority state"
    );

    // Sanity: the real call does record its decision, so the detector works.
    let real = client
        .call(add_item_request(
            root.path(),
            &case_id,
            extra_item("quiet"),
            json!({"request_id": "quiet-real"}),
        ))
        .await;
    assert_eq!(real["ok"], true, "{real}");
    assert_ne!(authority_snapshot(root.path()), before);
}

#[tokio::test]
async fn case_overview_digest_matches_the_state_it_returns() {
    let (root, socket) = boot().await;
    let mut setup = Client::connect(&socket).await;
    let case_id = commit_case(&mut setup, root.path(), "req-commit-consistent").await;
    let writer_socket = socket.clone();
    let writer_case = case_id.clone();
    let writer_root = root.path().to_path_buf();
    let writer = tokio::spawn(async move {
        let mut client = Client::connect(&writer_socket).await;
        for n in 0..6 {
            let reply = client
                .call(add_item_request(
                    &writer_root,
                    &writer_case,
                    extra_item(&format!("w{n}")),
                    json!({"request_id": format!("consistent-{n}")}),
                ))
                .await;
            assert_eq!(reply["ok"], true, "{reply}");
        }
    });
    let mut seen: std::collections::BTreeMap<String, std::collections::BTreeSet<u64>> =
        Default::default();
    let mut reader = Client::connect(&socket).await;
    while !writer.is_finished() {
        let overview = reader
            .call(json!({"verb": "case_get_overview", "case_id": case_id}))
            .await;
        if let Some(digest) = overview["digest"]["expected_digest"].as_str() {
            seen.entry(digest.to_string())
                .or_default()
                .insert(overview["item_count"].as_u64().unwrap());
        }
    }
    writer.await.unwrap();
    for (digest, counts) in &seen {
        assert_eq!(
            counts.len(),
            1,
            "digest {digest} paired with item counts {counts:?}"
        );
    }
}

#[tokio::test]
async fn add_item_dry_run_returns_the_item_as_it_will_be_stored() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let case_id = commit_case(&mut client, root.path(), "req-commit-normalized").await;
    let first_id = plan_json(root.path(), &case_id)["items"][0]["plan_item_id"]
        .as_str()
        .unwrap()
        .to_string();
    let mut item = extra_item("normalized");
    item["item_kind"] = json!("sandboxed_task");
    item["operations"] = json!([{"kind": "execute_command", "argv": ["true"], "cwd": "."}]);
    item["depends_on"] = json!([first_id]);
    let dry = client
        .call(add_item_request(
            root.path(),
            &case_id,
            item.clone(),
            json!({"dry_run": true}),
        ))
        .await;
    assert_eq!(dry["ok"], true, "{dry}");
    let real = client
        .call(add_item_request(
            root.path(),
            &case_id,
            item,
            json!({"request_id": "normalized-real"}),
        ))
        .await;
    assert_eq!(real["ok"], true, "{real}");
    let stored = plan_item(&plan_json(root.path(), &case_id), "normalized")
        .unwrap()
        .clone();
    assert_eq!(dry["item"], stored, "preview must equal what is saved");
    assert_ne!(
        stored["sandbox_class"],
        json!(null),
        "normalization should have set a sandbox class"
    );
}
