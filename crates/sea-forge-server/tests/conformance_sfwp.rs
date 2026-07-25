//! SFWP transport conformance tests (Task 3).
//!
//! These talk over a real Unix socket with real `ServerConfig`/`ServerState`,
//! because the whole point of Task 3 is connection-level correlation, reconnect
//! and event-cursor behavior — not in-process `handle_request` calls. There was
//! no reusable socket-client helper in the existing conformance suite
//! (`conformance_m16.rs` etc. all call into the crate directly), so this file
//! defines its own minimal one (`Client`).

use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind};
use sea_forge_planner::templates::{instantiate, sequential_agents_template};
use sea_forge_server::{run, ServerConfig};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, TcpListener, UnixStream};

const STUB_ENDPOINT: &str = "sfwp-test-endpoint";

/// Boot a server on a temp-root socket and return `(tempdir, socket_path)`.
async fn boot() -> (tempfile::TempDir, PathBuf) {
    boot_with_endpoint(None).await
}

/// Boot a server, optionally registering a stub agent endpoint so that
/// `submit` can succeed end to end (and thus emit a durable `case.submitted`
/// event). Returns `(tempdir, socket_path)`.
async fn boot_with_endpoint(endpoint: Option<AgentEndpointConfig>) -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let socket = root.path().join("sfwp.sock");
    let agent = AgentConfig {
        endpoints: endpoint.into_iter().collect(),
        ..AgentConfig::default()
    };
    let config = ServerConfig {
        socket_path: socket.clone(),
        root: root.path().to_path_buf(),
        agent,
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

/// A stub OpenAI-compatible endpoint answering every request identically,
/// mirroring `conformance_topology.rs::stub_always`.
fn stub_endpoint(response: &'static str) -> (AgentEndpointConfig, tokio::task::JoinHandle<()>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    let listener = TcpListener::from_std(listener).unwrap();
    let task = tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let mut request = vec![0_u8; 32_768];
                let _ = stream.read(&mut request).await;
                let body = response.as_bytes();
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes()).await;
                let _ = stream.write_all(body).await;
            });
        }
    });
    let endpoint = AgentEndpointConfig {
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
    };
    (endpoint, task)
}

/// Write an allow-all policy file and a valid sequential-topology plan under
/// `root`, returning the JSON `submit` request that dispatches successfully.
fn success_submit(root: &Path, request_id: Option<&str>) -> Value {
    let policy_path = root.join("policy.yaml");
    std::fs::write(
        &policy_path,
        "version: \"0.1\"\npolicy_surfaces:\n  external_api:\n    mode: deny-by-default\n    allow_hosts: [127.0.0.1]\nrules:\n  - name: allow-agent-task\n    verdict: allow\n    actor_role: operator\n    operation_kind: agent_task\n",
    )
    .unwrap();
    let tmpl = sequential_agents_template(STUB_ENDPOINT).unwrap();
    let plan = instantiate(&tmpl, &BTreeMap::new(), "case_x", "run_x", "intent_x").unwrap();
    let plan_path = root.join("plan.json");
    std::fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let mut request = json!({
        "verb": "submit",
        "plan": plan_path.to_str().unwrap(),
        "policy": policy_path.to_str().unwrap(),
        "entity": "operator_local",
        "process": "test",
    });
    if let Some(id) = request_id {
        request["request_id"] = json!(id);
    }
    request
}

/// A minimal line-delimited JSON client over the Unix socket.
struct Client {
    writer: tokio::net::unix::OwnedWriteHalf,
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

    async fn send(&mut self, value: Value) {
        let line = format!("{value}\n");
        self.writer.write_all(line.as_bytes()).await.unwrap();
        self.writer.flush().await.unwrap();
    }

    /// Send a request and read exactly one response line.
    async fn call(&mut self, value: Value) -> Value {
        self.send(value).await;
        self.read_line().await
    }

    async fn read_line(&mut self) -> Value {
        let mut line = String::new();
        let n = tokio::time::timeout(Duration::from_secs(10), self.reader.read_line(&mut line))
            .await
            .expect("read timed out")
            .unwrap();
        assert!(n > 0, "connection closed unexpectedly");
        serde_json::from_str(line.trim()).unwrap()
    }

    /// Read the next line that is an event frame, skipping any non-event lines.
    async fn read_event(&mut self) -> Value {
        loop {
            let value = self.read_line().await;
            if value.get("type").and_then(Value::as_str) == Some("event") {
                return value["event"].clone();
            }
        }
    }
}

#[tokio::test]
async fn system_hello_returns_protocol_version_and_implemented_methods() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let response = client
        .call(json!({"verb": "system_hello", "protocol_version": "1", "client": "test"}))
        .await;
    assert_eq!(response["protocol_version"], "1");
    assert_eq!(response["server_protocol_version"], "1");
    let methods = response["implemented_methods"].as_array().unwrap();
    assert!(methods.iter().any(|m| m == "system.hello"));
    assert!(methods.iter().any(|m| m == "events.subscribe"));
    assert!(methods.iter().any(|m| m == "request.get_status"));

    // Unsupported major is a structured error, not a panic/drop.
    let bad = client
        .call(json!({"verb": "system_hello", "protocol_version": "99"}))
        .await;
    assert_eq!(bad["error_class"], "unsupported_version");
    assert_eq!(bad["supported"], "1");
}

#[tokio::test]
async fn unknown_verb_still_rejected_cleanly() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let response = client
        .call(json!({"verb": "totally_bogus_verb", "x": 1}))
        .await;
    // ADR-003: unknown verbs fail cleanly as a structured error, no panic and
    // the connection survives for the next request.
    assert!(
        response.get("error").is_some(),
        "expected clean error: {response}"
    );
    // Connection still usable.
    let ok = client.call(json!({"verb": "system_describe"})).await;
    assert_eq!(ok["protocol_version"], "1");
}

#[tokio::test]
async fn system_describe_lists_classes() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let response = client.call(json!({"verb": "system_describe"})).await;
    let methods = response["methods"].as_array().unwrap();
    let subscribe = methods
        .iter()
        .find(|m| m["method"] == "events.subscribe")
        .unwrap();
    assert_eq!(subscribe["class"], "subscribe");
    let hello = methods
        .iter()
        .find(|m| m["method"] == "system.hello")
        .unwrap();
    assert_eq!(hello["class"], "inspect");
}

/// Compute the sha256 digest the same way `sfwp::precondition::digest_of`
/// does, over the current `case:<id>` record view, so we can pin a *stale*
/// expectation deliberately.
#[tokio::test]
async fn stale_precondition_on_approve_is_rejected_with_no_side_effect() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;

    // A precondition against a case ref that will never match (the case ledger
    // does not exist / differs), with a deliberately bogus expected digest.
    let response = client
        .call(json!({
            "verb": "approve",
            "case_id": "case_does_not_exist",
            "approval_id": "appr_1",
            "preconditions": {
                "records": [
                    {"ref": "case:case_does_not_exist", "expected_digest": "sha256:deadbeef"}
                ]
            }
        }))
        .await;

    assert_eq!(response["outcome"], "rejected_as_stale");
    assert_eq!(response["code"], "precondition_failed");
    assert!(!response["changed_records"].as_array().unwrap().is_empty());

    // No side effect: no case ledger was created for this case as a result of
    // the rejected approval (the approval CLI was never invoked).
    let ledger_dir = root.path().join("ledgers").join("case-case_does_not_exist");
    assert!(
        !ledger_dir.exists(),
        "stale-rejected approval must not create/mutate the case ledger"
    );
}

/// A `case:<id>` precondition ref whose id differs from the request's
/// `case_id` must NOT resolve to a fingerprint of the request case's ledger
/// mislabeled with the foreign id. The resolver treats a mismatched id as an
/// unresolvable reference (`current_digest: null`), so the precondition
/// stale-rejects without leaking a real-case fingerprint under a foreign id.
#[tokio::test]
async fn precondition_case_ref_mismatch_does_not_leak_other_case_fingerprint() {
    let (root, socket) = boot().await;

    // Materialize a real case ledger with one entry so the resolver's
    // `ledger_dir.exists()` guard passes; without the mismatch guard this is
    // exactly the state under which the bug would compute and leak the
    // fingerprint.
    let ledger = sea_forge_ledger::LedgerStream::open(root.path(), "case-real_case", "test")
        .expect("open case ledger");
    ledger
        .append(
            "case_created",
            vec![],
            serde_json::json!({"case_id": "real_case"}),
            vec![],
        )
        .expect("append case entry");

    let mut client = Client::connect(&socket).await;
    let response = client
        .call(json!({
            "verb": "approve",
            "case_id": "real_case",
            "approval_id": "appr_1",
            "preconditions": {
                "records": [
                    {"ref": "case:other_case", "expected_digest": "sha256:deadbeef"}
                ]
            }
        }))
        .await;

    assert_eq!(response["outcome"], "rejected_as_stale");
    assert_eq!(response["code"], "precondition_failed");
    let changed = response["changed_records"]
        .as_array()
        .expect("changed_records present");
    assert!(
        !changed.is_empty(),
        "mismatched ref must be reported changed"
    );
    // The crux: a mismatched id resolves to no record (null current_digest),
    // never to this case's fingerprint mislabeled with the foreign id.
    assert!(
        changed[0]["current_digest"].is_null(),
        "mismatched case ref must not leak a fingerprint: {}",
        changed[0]
    );
}

/// The single most important scenario: hello -> subscribe -> command ->
/// kill connection mid-flight -> reconnect -> request.get_status recovers ->
/// resubscribe from cursor -> induce a gap while disconnected ->
/// events.get_range recovers exactly the missed event.
#[tokio::test]
async fn end_to_end_correlation_reconnect_and_gap_recovery() {
    let (endpoint, stub) = stub_endpoint(
        r#"{"choices":[{"message":{"content":"step complete"}}],"usage":{"total_tokens":1}}"#,
    );
    let (root, socket) = boot_with_endpoint(Some(endpoint)).await;

    // --- Connection 1: hello, subscribe, issue a correlated command. -------
    let mut c1 = Client::connect(&socket).await;
    let hello = c1
        .call(json!({"verb": "system_hello", "protocol_version": "1"}))
        .await;
    assert_eq!(hello["server_protocol_version"], "1");

    let sub = c1.call(json!({"verb": "events_subscribe"})).await;
    assert_eq!(sub["subscribed"], true);

    // Issue a real (successful) submit carrying a request_id.
    let request_id = "req-e2e-1";
    let submit = success_submit(root.path(), Some(request_id));
    c1.send(submit).await;

    // Kill the connection mid-flight (drop it) before draining the response.
    // The server future keeps running to completion and records the terminal
    // outcome even though this client has vanished.
    drop(c1);

    // --- Connection 2: recover status, then resume events. -----------------
    // Poll status until the (still-executing) request reaches a terminal
    // state, proving the outcome survives the reconnect.
    let mut c2 = Client::connect(&socket).await;
    let mut status = json!(null);
    for _ in 0..200 {
        status = c2
            .call(json!({"verb": "request_get_status", "request_id": request_id}))
            .await;
        let s = status["status"].as_str().unwrap_or("");
        if s == "completed" || s == "failed" {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(status["request_id"], request_id);
    let recovered = status["status"].as_str().unwrap();
    assert!(
        recovered == "completed" || recovered == "failed",
        "status must be terminal after reconnect, got {recovered}"
    );
    assert!(status["outcome"].is_object());
    assert_eq!(
        status["outcome"]["state"], "completed",
        "recovered outcome must carry the original response: {status}"
    );

    // Unknown request id recovers as "unknown", not an error.
    let unknown = c2
        .call(json!({"verb": "request_get_status", "request_id": "never-seen"}))
        .await;
    assert_eq!(unknown["status"], "unknown");

    // Establish a known cursor: read the full durable range so far. The
    // successful submit emitted a durable `case.submitted` event.
    let range0 = c2.call(json!({"verb": "events_get_range"})).await;
    let events0 = range0["events"].as_array().unwrap().clone();
    assert!(
        events0.iter().any(|e| e["kind"] == "case.submitted"),
        "first submit must have produced a durable event: {events0:?}"
    );
    let last_cursor = events0
        .last()
        .map(|e| e["cursor"].as_str().unwrap().to_string());

    // Subscribe from the last-known cursor: no duplicate replay of prior events.
    let resume = c2
        .call(json!({"verb": "events_subscribe", "from_cursor": last_cursor}))
        .await;
    assert_eq!(resume["subscribed"], true);
    assert_eq!(resume["replayed"], 0, "resuming from tip replays nothing");

    // --- Induce a gap: perform an action with NO live subscriber. ----------
    drop(c2);
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut c3 = Client::connect(&socket).await;
    // A second successful submit produces a new durable event while nobody is
    // subscribed — the deliberately induced gap.
    let gap_submit = success_submit(root.path(), Some("req-e2e-gap"));
    let gap_response = c3.call(gap_submit).await;
    assert_eq!(gap_response["state"], "completed", "{gap_response}");

    // events.get_range from the last-known cursor recovers exactly the missed
    // event(s) — deterministic gap recovery over the durable ledger.
    let range1 = c3
        .call(json!({"verb": "events_get_range", "from_cursor": last_cursor}))
        .await;
    let recovered_events = range1["events"].as_array().unwrap();
    assert!(
        !recovered_events.is_empty(),
        "gap recovery must return the missed event(s)"
    );
    // Every recovered event has a strictly greater cursor than the pin.
    if let Some(pin) = &last_cursor {
        for event in recovered_events {
            assert!(
                event["cursor"].as_str().unwrap() > pin.as_str(),
                "get_range must be exclusive of from_cursor"
            );
        }
    }
    // The recovered gap event is the second case submission.
    assert!(recovered_events
        .iter()
        .any(|e| e["kind"] == "case.submitted"));

    stub.abort();
}

/// A live subscriber receives an event frame pushed by another connection's
/// mutation — proving unsolicited event lines interleave with the request path.
#[tokio::test]
async fn live_subscriber_receives_pushed_event() {
    let (endpoint, stub) = stub_endpoint(
        r#"{"choices":[{"message":{"content":"step complete"}}],"usage":{"total_tokens":1}}"#,
    );
    let (root, socket) = boot_with_endpoint(Some(endpoint)).await;

    let mut sub = Client::connect(&socket).await;
    let ack = sub.call(json!({"verb": "events_subscribe"})).await;
    assert_eq!(ack["subscribed"], true);

    let mut actor = Client::connect(&socket).await;
    let submit = success_submit(root.path(), None);
    let response = actor.call(submit).await;
    assert_eq!(response["state"], "completed", "{response}");

    let event = sub.read_event().await;
    assert_eq!(event["kind"], "case.submitted");
    assert!(!event["cursor"].as_str().unwrap().is_empty());

    stub.abort();
}

/// Drift guard: regenerating the SFWP schemas must produce byte-identical
/// output to the committed `workbench/packages/contracts/schema/*.schema.json`.
/// Shells out to the same generator binary the developer runs, into a temp
/// dir, then diffs. Changing a schemars-derived type without regenerating
/// fails here (the Rust-side half of the end-to-end drift check).
#[test]
fn generated_schemas_are_committed_and_current() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest.parent().and_then(Path::parent).unwrap();
    let committed_dir = workspace.join("workbench/packages/contracts/schema");
    assert!(
        committed_dir.exists(),
        "committed schema dir missing; run `cargo run -p sea-forge-server --bin gen_sfwp_schema`"
    );

    // Regenerate into a temp dir (never mutating the committed tree) and diff.
    let out = tempfile::tempdir().unwrap();
    let status = std::process::Command::new(env!("CARGO"))
        .args([
            "run",
            "--quiet",
            "-p",
            "sea-forge-server",
            "--bin",
            "gen_sfwp_schema",
        ])
        .env("SFWP_SCHEMA_OUT_DIR", out.path())
        .current_dir(workspace)
        .status()
        .expect("run gen_sfwp_schema");
    assert!(status.success(), "schema generator failed");

    let mut fresh: Vec<String> = std::fs::read_dir(out.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".schema.json"))
        .collect();
    fresh.sort();
    assert!(
        fresh.len() >= 11,
        "expected >= 11 schemas, got {}",
        fresh.len()
    );

    for name in &fresh {
        let generated = std::fs::read(out.path().join(name)).unwrap();
        let committed = std::fs::read(committed_dir.join(name)).unwrap_or_else(|_| {
            panic!("missing committed schema {name}; regenerate with gen_sfwp_schema")
        });
        assert_eq!(
            generated, committed,
            "schema drift in {name}: committed file differs from freshly generated output; \
             run `cargo run -p sea-forge-server --bin gen_sfwp_schema`"
        );
        // Sanity: valid JSON.
        let _: Value = serde_json::from_slice(&committed).unwrap();
    }

    // No committed schema is stale (present in committed but not regenerated).
    for entry in std::fs::read_dir(&committed_dir).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        if name.ends_with(".schema.json") {
            assert!(
                fresh.contains(&name),
                "committed schema {name} is no longer generated; delete it or restore the type"
            );
        }
    }
}
