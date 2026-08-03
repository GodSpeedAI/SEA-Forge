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
    let (socket, _server) = boot_on(root.path(), endpoint).await;
    (root, socket)
}

/// Boot a server over an *already existing* root, returning `(socket, task)`.
///
/// The task handle is what separates this from `boot_with_endpoint`: a restart
/// test has to be able to *stop* the first server. Booting a second one beside
/// a first that is still running and still executing the request under test
/// would recover an outcome from the original task's memory and call it
/// durability. Callers with nothing to restart may drop the handle.
async fn boot_on(
    root: &Path,
    endpoint: Option<AgentEndpointConfig>,
) -> (PathBuf, tokio::task::JoinHandle<()>) {
    let socket = root.join("sfwp.sock");
    let agent = AgentConfig {
        endpoints: endpoint.into_iter().collect(),
        ..AgentConfig::default()
    };
    let config = ServerConfig {
        identity: sea_forge_server::identity::IdentityBindings::local_operator("operator_local"),
        socket_path: socket.clone(),
        root: root.to_path_buf(),
        agent,
        ..ServerConfig::default()
    };
    let server = tokio::spawn(async move {
        let _ = run(config).await;
    });
    // Readiness is a successful connect, not the socket file existing. A
    // restarted server binds over the path its predecessor left behind, so the
    // file is already there — and answering nothing — before the replacement
    // has taken the lock and published its own socket.
    for _ in 0..200 {
        if UnixStream::connect(&socket).await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    (socket, server)
}

/// How many cases exist under `root`.
///
/// The idempotency tests below assert on this rather than on response equality
/// alone: two identical responses are also exactly what a double mutation
/// produces, so only the case count can tell the two apart.
fn cases_committed(root: &Path) -> usize {
    std::fs::read_dir(root.join("cases"))
        .map(|dir| dir.count())
        .unwrap_or(0)
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
        "verb": "submit", "actor": {"actor_id": "operator_local", "role": "operator"},
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
            "verb": "approve", "actor": {"actor_id": "operator_local", "role": "operator"},
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
            "verb": "approve", "actor": {"actor_id": "operator_local", "role": "operator"},
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

// --- readiness.get (Task 5) ------------------------------------------------
//
// `readiness.get` is a read-only inspect projection. These tests prove it
// always answers with a shaped view (infallible), that it reflects endpoint
// configuration honestly, and that switching the intended operation changes
// which capability is foregrounded (operation-sensitivity).

/// (a) A fresh cell has no committed self-model snapshot and no endpoint probe,
/// so readiness must be explicitly `unknown`; implementation configuration and
/// source locations cannot masquerade as evidence.
#[tokio::test]
async fn readiness_get_no_committed_sources_is_unknown() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;

    let view = client.call(json!({"verb": "readiness_get"})).await;

    let foundations = view["foundations"].as_array().unwrap();
    let self_model = foundations
        .iter()
        .find(|i| i["id"] == "self_model_integrity")
        .expect("self_model_integrity foundation present");
    assert_eq!(self_model["status"], "unknown", "{view}");
    assert!(
        self_model.get("source").is_none(),
        "an uninitialized cell must not substitute a code citation for a committed record: {view}"
    );

    let caps = view["operational_capabilities"].as_array().unwrap();
    let local = caps
        .iter()
        .find(|c| c["id"] == "local_governed_execution")
        .expect("local capability present");
    assert_eq!(local["status"], "blocked", "{view}");

    let external = caps
        .iter()
        .find(|c| c["id"] == "external_delegation")
        .expect("external capability present");
    assert_eq!(external["status"], "unknown", "{view}");
    assert_eq!(
        external["reason"], "No committed endpoint verification record is available",
        "configuration cannot be reported as endpoint evidence: {view}"
    );

    assert_eq!(view["overall"], "unknown", "{view}");

    // Partial scope is honest, not fabricated.
    assert!(
        view["recent_invalidations"].as_array().unwrap().is_empty(),
        "recent_invalidations must be an honest empty array in this slice"
    );
}

/// (b) Registering an endpoint remains intent rather than a verified probe.
/// It cannot upgrade the readiness verdict until a committed probe/settlement
/// record exists.
#[tokio::test]
async fn readiness_get_endpoint_configuration_is_not_readiness_evidence() {
    let (endpoint, stub) =
        stub_endpoint(r#"{"choices":[{"message":{"content":"ok"}}],"usage":{"total_tokens":1}}"#);
    let (_root, socket) = boot_with_endpoint(Some(endpoint)).await;
    let mut client = Client::connect(&socket).await;

    let view = client.call(json!({"verb": "readiness_get"})).await;

    let caps = view["operational_capabilities"].as_array().unwrap();
    let external = caps
        .iter()
        .find(|c| c["id"] == "external_delegation")
        .expect("external capability present");
    assert_eq!(external["status"], "unknown", "{view}");
    assert_eq!(view["overall"], "unknown", "{view}");

    stub.abort();
}

/// (c) Operation-sensitivity: passing `intended_operation.method = "delegate"`
/// foregrounds the external-delegation capability (orders it first), whereas
/// omitting the intended operation foregrounds local execution. This is the
/// "switching intended operation changes readiness" proof — a real assertion on
/// *which* capability leads, not merely that the responses differ.
#[tokio::test]
async fn readiness_get_intended_operation_foregrounds_relevant_capability() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;

    // No intended operation: local governed execution leads.
    let default_view = client.call(json!({"verb": "readiness_get"})).await;
    let default_caps = default_view["operational_capabilities"].as_array().unwrap();
    assert_eq!(
        default_caps[0]["id"], "local_governed_execution",
        "without an intended operation, local execution is foregrounded: {default_view}"
    );

    // Intended operation that requires delegation: external capability leads.
    let delegate_view = client
        .call(json!({
            "verb": "readiness_get",
            "intended_operation": {"method": "delegate"}
        }))
        .await;
    let delegate_caps = delegate_view["operational_capabilities"]
        .as_array()
        .unwrap();
    assert_eq!(
        delegate_caps[0]["id"], "external_delegation",
        "a delegation operation must foreground external delegation: {delegate_view}"
    );

    // The view echoes the intended operation it was shaped for.
    assert_eq!(
        delegate_view["intended_operation"]["method"], "delegate",
        "{delegate_view}"
    );

    // Concrete proof the foregrounding actually changed order.
    assert_ne!(
        default_caps[0]["id"], delegate_caps[0]["id"],
        "switching intended operation must change which capability leads"
    );
}

/// `readiness.get` is discoverable via the catalog with the `inspect` class.
#[tokio::test]
async fn readiness_get_is_listed_as_inspect() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;

    let hello = client
        .call(json!({"verb": "system_hello", "protocol_version": "1"}))
        .await;
    let methods = hello["implemented_methods"].as_array().unwrap();
    assert!(
        methods.iter().any(|m| m == "readiness.get"),
        "readiness.get must be advertised: {hello}"
    );

    let describe = client.call(json!({"verb": "system_describe"})).await;
    let entry = describe["methods"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["method"] == "readiness.get")
        .expect("readiness.get in describe");
    assert_eq!(entry["class"], "inspect");
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
        "committed schema dir missing; run `just workbench-contracts-generate`"
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
             run `just workbench-contracts-generate`"
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

    // `sfwp::SCHEMA_TYPES` claims to share its list with the generator "so they
    // never drift". Until this assertion existed the claim was only a comment,
    // and it had already been broken once: eight Task 7 types reached the
    // generator but never `system.get_schema`, which under-reported the
    // contracts a client could fetch. Now the claim is enforced.
    let mut advertised: Vec<String> = sea_forge_server::sfwp::SCHEMA_TYPES
        .iter()
        .map(|name| format!("{name}.schema.json"))
        .collect();
    advertised.sort();
    assert_eq!(
        advertised, fresh,
        "sfwp::SCHEMA_TYPES and gen_sfwp_schema disagree; system.get_schema would \
         under- or over-report the available contracts"
    );
}

/// SF-006: the response is lost after the mutation lands, and the client
/// retries the same id with the same payload.
///
/// The client cannot tell "the commit never happened" from "the commit
/// happened and the answer was lost". Only the server can, and only by
/// recognising the id it already bound to this payload.
#[tokio::test]
async fn a_retried_request_replays_its_outcome_instead_of_mutating_twice() {
    let (endpoint, stub) = stub_endpoint(
        r#"{"choices":[{"message":{"content":"step complete"}}],"usage":{"total_tokens":1}}"#,
    );
    let (root, socket) = boot_with_endpoint(Some(endpoint)).await;
    let mut client = Client::connect(&socket).await;

    let request = success_submit(root.path(), Some("req-lost-response"));
    let first = client.call(request.clone()).await;
    assert_eq!(first["state"], "completed", "{first}");
    assert_eq!(
        cases_committed(root.path()),
        1,
        "the first submit must have created exactly one case"
    );

    // The client never saw `first`, so it sends the identical line again.
    let second = client.call(request).await;
    // Checked before the responses are compared, because it is the assertion
    // that decides the question: two identical responses are also what running
    // the work twice would produce.
    assert_eq!(
        cases_committed(root.path()),
        1,
        "the retry must not have committed a second case"
    );
    assert_eq!(
        second, first,
        "the retry must return the stored outcome verbatim, not a fresh one"
    );

    stub.abort();
}

/// SF-006: the same id pointed at a changed payload is refused outright.
///
/// Replaying the first outcome here would answer a question nobody asked —
/// the operator edited the request. Executing it would spend one id on two
/// different decisions, which is what makes recovery-by-id unsound.
#[tokio::test]
async fn a_reused_request_id_with_a_changed_payload_is_refused_with_no_mutation() {
    let (endpoint, stub) = stub_endpoint(
        r#"{"choices":[{"message":{"content":"step complete"}}],"usage":{"total_tokens":1}}"#,
    );
    let (root, socket) = boot_with_endpoint(Some(endpoint)).await;
    let mut client = Client::connect(&socket).await;

    let mut request = success_submit(root.path(), Some("req-reused-id"));
    let first = client.call(request.clone()).await;
    assert_eq!(first["state"], "completed", "{first}");
    let committed = cases_committed(root.path());
    assert_eq!(committed, 1);

    // A changed field that would otherwise commit perfectly well, so the
    // refusal below can only be the id check and not a validation failure.
    request["process"] = json!("a-different-process");
    let second = client.call(request).await;
    assert_eq!(
        second["error_class"], "request_id_reused",
        "expected the id to be refused: {second}"
    );
    assert_eq!(second["no_side_effect"], true);
    assert_eq!(
        cases_committed(root.path()),
        committed,
        "a refused reuse must leave the case count where it was"
    );

    stub.abort();
}

/// SF-006: a terminal outcome survives a restart, and the id it belongs to
/// still dedupes afterwards.
///
/// The client drops the connection before reading the response, so from its
/// side the request is left in flight. The first server is then stopped and a
/// second one booted over the same root, so every assertion below is answered
/// by a process that shares nothing with the first but the files.
///
/// What this covers is a restart *after* the outcome was recorded. It does not
/// cover a restart between the dispatch `record_pending` and the handler's
/// `record_outcome`: nothing scans `<root>/requests` at boot, so a record left
/// `Pending` by a kill in that window stays `Pending` and `request.get_status`
/// answers `pending` forever. Boot reconciliation is new boot behaviour and
/// therefore an ask-first decision (`docs/execution/DECISION_REGISTER.md`).
#[tokio::test]
async fn a_terminal_outcome_and_its_dedupe_key_survive_a_restart() {
    let (endpoint, stub) = stub_endpoint(
        r#"{"choices":[{"message":{"content":"step complete"}}],"usage":{"total_tokens":1}}"#,
    );
    let root = tempfile::tempdir().unwrap();
    let (socket, first) = boot_on(root.path(), Some(endpoint.clone())).await;

    let request_id = "req-restart-recovery";
    let request = success_submit(root.path(), Some(request_id));
    let mut before = Client::connect(&socket).await;
    before.send(request.clone()).await;
    drop(before);

    // Wait on the *first* server for the work to settle. Stopping it earlier
    // would prove nothing about durability, because there would be nothing
    // durable yet for the restart to carry across.
    let mut watcher = Client::connect(&socket).await;
    let mut settled = false;
    for _ in 0..200 {
        let status = watcher
            .call(json!({"verb": "request_get_status", "request_id": request_id}))
            .await;
        if matches!(
            status["status"].as_str(),
            Some("completed") | Some("failed")
        ) {
            settled = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    drop(watcher);
    assert!(
        settled,
        "the first server must reach a terminal outcome before it is stopped"
    );

    // The restart. Awaiting the aborted task is not tidiness: `run` holds the
    // socket lock inside the future being cancelled, and the replacement cannot
    // bind until that future has actually been dropped.
    first.abort();
    let _ = first.await;
    let (socket, second) = boot_on(root.path(), Some(endpoint)).await;
    let mut after = Client::connect(&socket).await;

    let status = after
        .call(json!({"verb": "request_get_status", "request_id": request_id}))
        .await;
    assert_eq!(
        status["status"], "completed",
        "the outcome must be recoverable after the restart: {status}"
    );
    assert_eq!(status["outcome"]["state"], "completed");
    assert_eq!(cases_committed(root.path()), 1);

    // And the recovered id is still bound to its payload: a client that
    // retried instead of asking gets the same answer, not a second case.
    let replayed = after.call(request).await;
    assert_eq!(
        cases_committed(root.path()),
        1,
        "the retry after restart must not have committed a second case"
    );
    assert_eq!(
        replayed, status["outcome"],
        "the retry after restart must replay the recovered outcome"
    );

    second.abort();
    stub.abort();
}
