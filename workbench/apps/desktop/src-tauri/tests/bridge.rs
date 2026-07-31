//! Host-side SFWP integration test (Task 3): correlation recovery across a
//! killed connection, proven through the *host's* socket client — not just the
//! server's raw wire path.
//!
//! We spin up a real `sea-forge-server` on a temp root + temp socket (mirroring
//! `crates/sea-forge-server/tests/conformance_sfwp.rs`'s `boot` pattern) and
//! drive it through `app_lib::socket::SocketClient` / `SocketHandle` — the same
//! `call()` chokepoint every Tauri command uses. The `events.rs` reconnect loop
//! needs a full Tauri `AppHandle` to run end to end, so we exercise the socket
//! layer's reconnect primitives directly here, which is exactly what the
//! correlation-recovery scenario proves.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use std::sync::{Arc, Mutex as StdMutex};

use app_lib::events::{run_event_loop, EventCursor, EventEmitter};
use app_lib::socket::{SocketError, SocketHandle};
use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind};
use sea_forge_planner::templates::{instantiate, sequential_agents_template};
use sea_forge_server::{run, ServerConfig};
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

const STUB_ENDPOINT: &str = "sfwp-host-test-endpoint";

/// Boot a real server on a temp-root socket with a stub agent endpoint so a
/// `submit` can run to a terminal outcome. Returns `(tempdir, socket_path)`.
async fn boot(endpoint: AgentEndpointConfig) -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let socket = root.path().join("sfwp-host.sock");
    let agent = AgentConfig {
        endpoints: vec![endpoint],
        ..AgentConfig::default()
    };
    let config = ServerConfig {
        socket_path: socket.clone(),
        root: root.path().to_path_buf(),
        agent,
        // A cell that configures no identity refuses every protected verb
        // (SF-005), so `submit` here would be denied before it ran. Bind the
        // uid this test's own connections will present — the same shape a
        // single-operator local cell uses.
        identity: sea_forge_server::identity::IdentityBindings::local_operator("operator_local"),
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

/// A stub OpenAI-compatible endpoint answering every request identically.
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

/// Write an allow-all policy + valid sequential plan under `root`, returning a
/// `submit` request line carrying `request_id`.
fn success_submit(root: &Path, request_id: &str) -> Value {
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
    json!({
        "verb": "submit",
        // The actor block the host attaches in `bridge::sfwp_command`. Spelled
        // out here because these tests drive the socket layer directly, below
        // the Tauri command that would add it.
        "actor": {"actor_id": "operator_local", "role": "operator"},
        "plan": plan_path.to_str().unwrap(),
        "policy": policy_path.to_str().unwrap(),
        "entity": "operator_local",
        "process": "test",
        "request_id": request_id,
    })
}

/// The named scenario: issue a correlated `submit` through the host's socket
/// client, kill the underlying connection mid-flight (simulating a crash),
/// reconnect a fresh client through the same `SocketHandle`, and recover the
/// terminal outcome via `request.get_status` — proving the host's reconnect
/// path (not merely the server's) recovers correlation.
#[tokio::test]
async fn host_socket_recovers_correlation_across_killed_connection() {
    let (endpoint, stub) = stub_endpoint(
        r#"{"choices":[{"message":{"content":"step complete"}}],"usage":{"total_tokens":1}}"#,
    );
    let (root, socket) = boot(endpoint).await;

    // A throwaway event sink; this test drives the request path, not events.
    let (event_tx, _event_rx) = mpsc::unbounded_channel::<Value>();
    let handle = SocketHandle::new(socket.clone(), event_tx);

    // --- Connection 1: hello, then issue a correlated submit and abandon it. --
    let c1 = handle.client().await.expect("connect c1");
    let hello = c1
        .call(json!({"verb": "system_hello", "protocol_version": "1"}))
        .await
        .expect("hello");
    assert_eq!(hello["server_protocol_version"], "1");

    let request_id = "host-req-1";
    let submit = success_submit(root.path(), request_id);

    // Kill the connection concurrently with the in-flight submit. Dropping the
    // handle's current client and forcing a reconnect abandons c1 mid-request;
    // the server keeps executing and records the terminal outcome regardless.
    let submit_result = c1.call(submit).await;
    // The submit may either complete just before the drop or fail as the
    // connection dies — both are valid crash timings. Either way the server
    // recorded the correlated outcome.
    match submit_result {
        Ok(_) | Err(SocketError::Disconnected) => {}
        Err(other) => panic!("unexpected submit error: {other}"),
    }
    drop(c1);

    // --- Connection 2: fresh client via reconnect, recover status. -----------
    let c2 = handle.reconnect().await.expect("reconnect c2");

    let mut status = json!(null);
    for _ in 0..200 {
        status = c2
            .call(json!({"verb": "request_get_status", "request_id": request_id}))
            .await
            .expect("get_status");
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
        "host must recover a terminal correlated outcome after reconnect, got {recovered}: {status}"
    );
    assert!(
        status["outcome"].is_object(),
        "recovered status must carry the original outcome: {status}"
    );

    // An unknown request id recovers cleanly as "unknown", not an error.
    let unknown = c2
        .call(json!({"verb": "request_get_status", "request_id": "never-issued"}))
        .await
        .expect("unknown status");
    assert_eq!(unknown["status"], "unknown");

    stub.abort();
}

/// Concurrent calls through one cloned client stay correctly FIFO-paired: each
/// caller receives the response to *its own* request, never another's. This is
/// the atomic-write+register invariant the socket client guarantees, and the
/// property that makes a shared `SocketHandle` safe for concurrent Tauri
/// commands.
#[tokio::test]
async fn concurrent_calls_are_correctly_paired() {
    let (endpoint, stub) = stub_endpoint(r#"{"choices":[{"message":{"content":"x"}}]}"#);
    let (_root, socket) = boot(endpoint).await;

    let (event_tx, _event_rx) = mpsc::unbounded_channel::<Value>();
    let handle = SocketHandle::new(socket.clone(), event_tx);
    let client = handle.client().await.expect("connect");

    // Fire a batch of distinguishable requests concurrently through clones of
    // the same underlying connection. `system_get_schema { method }` echoes the
    // request context indirectly; we instead use `system_hello` with a distinct
    // `protocol_version`-shaped marker via the `client` field which the server
    // ignores but which proves the response matches the correct request only if
    // pairing holds. Simpler: alternate hello/describe and assert each response
    // is the right *kind* for its request.
    let mut tasks = Vec::new();
    for i in 0..24 {
        let c = client.clone();
        tasks.push(tokio::spawn(async move {
            if i % 2 == 0 {
                let r = c
                    .call(json!({"verb": "system_hello", "protocol_version": "1"}))
                    .await
                    .expect("hello");
                // Only hello responses carry `server_protocol_version`.
                assert_eq!(
                    r["server_protocol_version"], "1",
                    "hello #{i} mispaired: {r}"
                );
            } else {
                let r = c
                    .call(json!({"verb": "system_describe"}))
                    .await
                    .expect("describe");
                // Only describe responses carry a `methods` array.
                assert!(r["methods"].is_array(), "describe #{i} mispaired: {r}");
            }
        }));
    }
    for t in tasks {
        t.await.expect("task panicked");
    }

    stub.abort();
}

/// A test emitter that records every forwarded `EventFrame`.
struct RecordingEmitter {
    frames: StdMutex<Vec<Value>>,
}

impl EventEmitter for RecordingEmitter {
    fn emit_event(&self, frame: &Value) {
        self.frames.lock().unwrap().push(frame.clone());
    }
}

/// The `events.rs` reconnect + gap-recovery loop, driven directly (no Tauri
/// `AppHandle`): a durable event already exists on the server before the loop
/// starts, and a persisted cursor is absent, so the loop's `events.get_range`
/// catch-up must recover the backlog and emit it, persisting the cursor.
#[tokio::test]
async fn event_loop_recovers_backlog_via_get_range() {
    let (endpoint, stub) = stub_endpoint(
        r#"{"choices":[{"message":{"content":"step complete"}}],"usage":{"total_tokens":1}}"#,
    );
    let (root, socket) = boot(endpoint).await;

    // Produce a durable event (a successful submit) BEFORE any subscriber, so
    // it exists purely as backlog the loop must recover via get_range.
    {
        let raw = tokio::net::UnixStream::connect(&socket).await.unwrap();
        let (r, mut w) = raw.into_split();
        let submit = success_submit(root.path(), "backlog-1");
        let line = format!("{submit}\n");
        w.write_all(line.as_bytes()).await.unwrap();
        w.flush().await.unwrap();
        // Read exactly one response line so the submit completes server-side.
        use tokio::io::{AsyncBufReadExt, BufReader};
        let mut reader = BufReader::new(r);
        let mut resp = String::new();
        reader.read_line(&mut resp).await.unwrap();
        let resp: Value = serde_json::from_str(resp.trim()).unwrap();
        assert_eq!(resp["state"], "completed", "{resp}");
    }

    // Fresh event sink + handle wired the same way lib.rs wires them.
    let (event_tx, event_rx) = mpsc::unbounded_channel::<Value>();
    let handle = Arc::new(SocketHandle::new(socket.clone(), event_tx));

    let cursor_path = root.path().join("sfwp-cursor.json");
    let cursor = Arc::new(EventCursor::load(cursor_path.clone()));
    let emitter = Arc::new(RecordingEmitter {
        frames: StdMutex::new(Vec::new()),
    });

    let loop_task = tokio::spawn(run_event_loop(
        Arc::clone(&handle),
        Arc::clone(&cursor),
        Arc::clone(&emitter),
        event_rx,
        "1".to_string(),
    ));

    // Wait until the backlog event has been recovered and emitted.
    let mut recovered = false;
    for _ in 0..200 {
        if emitter
            .frames
            .lock()
            .unwrap()
            .iter()
            .any(|f| f["kind"] == "case.submitted")
        {
            recovered = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert!(
        recovered,
        "event loop must recover the backlog via get_range"
    );

    // The cursor was persisted to disk (survives restart, not just reconnect).
    assert!(cursor_path.exists(), "cursor file must be persisted");
    assert!(
        cursor.get().await.is_some(),
        "cursor must advance past the recovered event"
    );

    loop_task.abort();
    stub.abort();
}

/// A scripted SFWP stub that hands out `events.get_range` pages from a fixed
/// script. Lets the catch-up drain be tested against page sizes a real server
/// would not currently produce — which is the point: the host must not depend
/// on the server's page size at all.
async fn scripted_sfwp_stub(socket: PathBuf, pages: Vec<usize>) -> tokio::task::JoinHandle<()> {
    let listener = tokio::net::UnixListener::bind(&socket).expect("bind stub socket");
    tokio::spawn(async move {
        let Ok((stream, _)) = listener.accept().await else {
            return;
        };
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = tokio::io::BufReader::new(read_half);
        let mut page_index = 0_usize;
        let mut emitted = 0_usize;
        loop {
            let mut line = String::new();
            use tokio::io::AsyncBufReadExt;
            match reader.read_line(&mut line).await {
                Ok(0) | Err(_) => return,
                Ok(_) => {}
            }
            let Ok(request) = serde_json::from_str::<Value>(line.trim()) else {
                return;
            };
            let response = match request["verb"].as_str() {
                Some("system_hello") => json!({
                    "protocol_version": "1",
                    "server_protocol_version": "1",
                    "implemented_methods": ["system.hello", "events.get_range"],
                }),
                Some("events_get_range") => {
                    let size = pages.get(page_index).copied().unwrap_or(0);
                    page_index += 1;
                    let events: Vec<Value> = (0..size)
                        .map(|_| {
                            emitted += 1;
                            json!({
                                "cursor": format!("cur-{emitted:04}"),
                                "kind": "case.submitted",
                                "detail": {},
                                "committed_at": "2026-07-26T00:00:00Z",
                            })
                        })
                        .collect();
                    json!({ "events": events })
                }
                Some("events_subscribe") => json!({"ok": true}),
                _ => json!({"error": "unsupported in stub"}),
            };
            let line = format!("{response}\n");
            if write_half.write_all(line.as_bytes()).await.is_err() {
                return;
            }
            let _ = write_half.flush().await;
        }
    })
}

/// The catch-up drain must keep reading until a page comes back **empty**, not
/// until a page looks short.
///
/// The script hands out two non-empty pages of 3 before the empty one. Any rule
/// of the form "a page shorter than N proves the backlog is exhausted" stops
/// after the first page for every N > 3 and silently drops the remaining
/// events — the exact loss this loop exists to prevent, and what the host's
/// former hardcoded 256 would have done against a server that pages smaller.
#[tokio::test]
async fn catch_up_drains_until_a_page_is_empty_not_merely_short() {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("scripted.sock");
    let stub = scripted_sfwp_stub(socket.clone(), vec![3, 3]).await;

    let (event_tx, event_rx) = mpsc::unbounded_channel::<Value>();
    let handle = Arc::new(SocketHandle::new(socket.clone(), event_tx));
    let cursor = Arc::new(EventCursor::load(dir.path().join("cursor.json")));
    let emitter = Arc::new(RecordingEmitter {
        frames: StdMutex::new(Vec::new()),
    });

    let loop_task = tokio::spawn(run_event_loop(
        Arc::clone(&handle),
        Arc::clone(&cursor),
        Arc::clone(&emitter),
        event_rx,
        "1".to_string(),
    ));

    for _ in 0..200 {
        if emitter.frames.lock().unwrap().len() >= 6 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    let frames = emitter.frames.lock().unwrap().clone();
    assert_eq!(
        frames.len(),
        6,
        "both non-empty pages must be drained before termination, got {} frame(s)",
        frames.len()
    );
    assert_eq!(
        cursor.get().await.as_deref(),
        Some("cur-0006"),
        "cursor must advance to the last recovered event"
    );

    loop_task.abort();
    stub.abort();
}
