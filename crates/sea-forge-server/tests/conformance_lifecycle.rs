//! Fail-closed service lifecycle conformance (spec §8.3, §8.4, §8.5, §11.1).
//!
//! Three seams, each of which fails *open* if it regresses — which is why they
//! are held down here rather than left to review:
//!
//! 1. **Startup.** A `server.yaml` that exists and does not validate must block
//!    the process. The failure mode it replaces is the worst kind: the server
//!    starts, logs a warning nobody reads, and serves a cell under defaults the
//!    operator never wrote.
//! 2. **Reload.** An invalid reload keeps the last-known-good snapshot *and*
//!    tells the operator. A reload can never relocate the cell.
//! 3. **Timeout.** A hung peer bounds one request; it does not consume the
//!    server, and it does not cancel work that may already be durable.
//!
//! The startup cases drive the real binary, because the fallback they replace
//! lived in `main`, not in a library function.

use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind};
use sea_forge_server::{run, ServerConfig, ServerState};
use serde_json::{json, Value};
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, UnixStream};

const SERVER_BIN: &str = env!("CARGO_BIN_EXE_sea-forge-server");

// ---------------------------------------------------------------------------
// 1. Startup fails closed (§8.3 `server_config_error`)
// ---------------------------------------------------------------------------

/// Start the real binary against `root` and wait for it to either publish a
/// socket or exit. Returns the child so the caller can inspect or kill it.
fn spawn_server(root: &Path) -> std::process::Child {
    std::process::Command::new(SERVER_BIN)
        .env("SEA_FORGE_ROOT", root)
        .env("RUST_LOG", "info")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn sea-forge-server")
}

#[test]
fn a_malformed_server_yaml_blocks_startup() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path()).unwrap();
    fs::write(
        root.path().join("server.yaml"),
        "max_concurrent_runs: [not\n",
    )
    .unwrap();

    let output = spawn_server(root.path()).wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "a server.yaml that does not parse must block startup; exited {:?}\n{stderr}",
        output.status.code()
    );
    assert!(
        stderr.contains("server_config_error"),
        "the failure must be classified for the operator, not just printed: {stderr}"
    );
    assert!(
        !root.path().join("server.sock").exists(),
        "a server that refused to start must not have published a socket"
    );
}

#[test]
fn an_invalid_agent_endpoint_blocks_startup() {
    let root = tempfile::tempdir().unwrap();
    // Parses as YAML; rejected by `AgentConfig::validate`. This is the case the
    // old `unwrap_or_else` swallowed most quietly — the file is well-formed, so
    // nothing looks wrong at a glance.
    fs::write(
        root.path().join("server.yaml"),
        "agent:\n  endpoints:\n    - id: broken\n      kind: openai_compatible\n      base_url: \"http://198.51.100.7/v1\"\n",
    )
    .unwrap();

    let output = spawn_server(root.path()).wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "an invalid agent endpoint must block startup; exited {:?}\n{stderr}",
        output.status.code()
    );
    assert!(
        stderr.contains("server_config_error"),
        "expected a classified config error: {stderr}"
    );
}

#[test]
fn an_absent_server_yaml_is_a_first_run_not_a_failure() {
    let root = tempfile::tempdir().unwrap();
    let mut child = spawn_server(root.path());

    let socket = root.path().join("server.sock");
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        if socket.exists() {
            break;
        }
        if let Some(status) = child.try_wait().unwrap() {
            let output = child.wait_with_output().unwrap();
            panic!(
                "first run exited {status:?} instead of starting on defaults:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let published = socket.exists();
    let _ = child.kill();
    let _ = child.wait();
    assert!(
        published,
        "a cell with no server.yaml must start on defaults — that is a first run"
    );
}

#[test]
fn a_notify_command_that_cannot_run_blocks_startup() {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("server.yaml"),
        "notify_command: [\"/nonexistent/sea-forge-notify\"]\n",
    )
    .unwrap();

    let output = spawn_server(root.path()).wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "a notify hook that can never run must block startup; exited {:?}\n{stderr}",
        output.status.code()
    );
    assert!(
        stderr.contains("/nonexistent/sea-forge-notify"),
        "the operator must be told which program is missing: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// 2. Reload keeps last-known-good and cannot move the cell (§8.4)
// ---------------------------------------------------------------------------

fn state_for(root: &Path) -> ServerState {
    ServerState::new(ServerConfig {
        socket_path: root.join("unused.sock"),
        root: root.to_path_buf(),
        max_concurrent_runs: 4,
        ..ServerConfig::default()
    })
    .expect("failed to build server state")
}

#[test]
fn a_valid_reload_replaces_the_live_snapshot() {
    let root = tempfile::tempdir().unwrap();
    let state = state_for(root.path());
    assert_eq!(state.config().max_concurrent_runs, 4);

    fs::write(
        root.path().join("server.yaml"),
        "max_concurrent_runs: 9\napproval_ttl_hours: 48\n",
    )
    .unwrap();
    state.reload_config().expect("valid reload must succeed");

    assert_eq!(state.config().max_concurrent_runs, 9);
    assert_eq!(state.config().approval_ttl_hours, 48);
}

#[test]
fn an_invalid_reload_keeps_the_last_known_good() {
    let root = tempfile::tempdir().unwrap();
    let state = state_for(root.path());

    fs::write(root.path().join("server.yaml"), "max_concurrent_runs: 9\n").unwrap();
    state.reload_config().unwrap();
    assert_eq!(state.config().max_concurrent_runs, 9);

    fs::write(
        root.path().join("server.yaml"),
        "max_concurrent_runs: [oops\n",
    )
    .unwrap();
    let error = state
        .reload_config()
        .expect_err("a malformed reload must be rejected");

    assert!(
        error.contains("parse"),
        "expected a typed parse error: {error}"
    );
    assert_eq!(
        state.config().max_concurrent_runs,
        9,
        "an invalid reload must leave the previous snapshot untouched, not fall back to defaults"
    );
}

/// A reload with no `server.yaml` present must change nothing.
///
/// `ServerConfig::load` answers `Ok(default)` for a missing path, which is the
/// right answer at startup and the wrong one on reload: applying it would mean
/// a cell configured in-process — or one whose `server.yaml` was deleted while
/// running — silently reverted to defaults on its next dispatch, losing its
/// agent endpoints along with everything else. That is a fail-open, and it is
/// invisible: the reload "succeeds".
#[test]
fn a_reload_with_no_file_present_changes_nothing() {
    let root = tempfile::tempdir().unwrap();
    let state = ServerState::new(ServerConfig {
        socket_path: root.path().join("unused.sock"),
        root: root.path().to_path_buf(),
        max_concurrent_runs: 6,
        notify_command: Some(vec!["sh".into(), "-c".into(), "true".into()]),
        agent: AgentConfig {
            endpoints: vec![AgentEndpointConfig {
                id: "configured-in-process".into(),
                kind: ProviderKind::OpenAiCompatible,
                base_url: Some("http://127.0.0.1:1/".into()),
                argv: Vec::new(),
                env: Vec::new(),
                credential_ref: None,
                default_model: None,
                allow_loopback_test: true,
                max_request_bytes: 4096,
                max_response_bytes: 4096,
                timeout_secs: 30,
                status: None,
                transcript_retention: None,
            }],
            ..AgentConfig::default()
        },
        ..ServerConfig::default()
    })
    .unwrap();

    assert!(!root.path().join("server.yaml").exists());
    state
        .reload_config()
        .expect("an absent file is not a failure");

    assert_eq!(state.config().max_concurrent_runs, 6);
    assert!(state.config().notify_command.is_some());
    assert_eq!(
        state.config().agent.endpoints.len(),
        1,
        "a reload with no file wiped the configured agent endpoints"
    );
    assert_eq!(
        state.config().agent.endpoints[0].id,
        "configured-in-process"
    );
}

/// `server.yaml` lives inside the cell it configures, so it cannot name a
/// different one. Without this, a `root:` key drifting into the file would
/// silently repoint a running server's records away from the socket its
/// clients are already connected to.
#[test]
fn a_reload_cannot_relocate_the_cell() {
    let root = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let state = state_for(root.path());
    let socket_before = state.config().resolved_socket_path();

    fs::write(
        root.path().join("server.yaml"),
        format!(
            "root: {}\nsocket_path: /tmp/somewhere-else.sock\nmax_concurrent_runs: 7\n",
            elsewhere.path().display()
        ),
    )
    .unwrap();
    state.reload_config().unwrap();

    assert_eq!(state.root, root.path(), "the cell root must be immovable");
    assert_eq!(state.config().root, root.path());
    assert_eq!(
        state.config().resolved_socket_path(),
        socket_before,
        "a reload must not move the socket out from under live connections"
    );
    // The rest of the file still applied — pinning the cell is not a veto on
    // everything else in it.
    assert_eq!(state.config().max_concurrent_runs, 7);
}

// ---------------------------------------------------------------------------
// 3. Timeout bounds a hung peer without cancelling durable work (§11.1)
// ---------------------------------------------------------------------------

/// Accept a connection and never answer, holding the socket open. This is the
/// real condition the §11.1 bound exists for: a reachable agent endpoint that
/// stops responding mid-request.
async fn hung_endpoint() -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        let mut held = Vec::new();
        while let Ok((stream, _)) = listener.accept().await {
            held.push(stream);
        }
    });
    (address, task)
}

fn hung_policy(root: &Path) -> PathBuf {
    let path = root.join("policy.yaml");
    fs::write(
        &path,
        "version: \"0.1\"\npolicy_surfaces:\n  external_api:\n    mode: deny-by-default\n    allow_hosts: [127.0.0.1]\nrules:\n  - name: allow-agent-probe\n    verdict: allow\n    actor_role: operator\n    operation_kind: agent_probe\n",
    )
    .unwrap();
    path
}

async fn boot_with_hung_endpoint(port: u16) -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let socket = root.path().join("lifecycle.sock");
    let config = ServerConfig {
        socket_path: socket.clone(),
        root: root.path().to_path_buf(),
        agent: AgentConfig {
            endpoints: vec![AgentEndpointConfig {
                id: "hung".into(),
                kind: ProviderKind::OpenAiCompatible,
                base_url: Some(format!("http://127.0.0.1:{port}/")),
                argv: Vec::new(),
                env: Vec::new(),
                credential_ref: None,
                default_model: Some("m1".into()),
                allow_loopback_test: true,
                max_request_bytes: 16_384,
                max_response_bytes: 16_384,
                // Deliberately far beyond the server's own bound: the point is
                // that the *server* gives up first. An endpoint timeout below
                // 10s would test the agent client instead.
                timeout_secs: 120,
                status: None,
                transcript_retention: None,
            }],
            ..AgentConfig::default()
        },
        ..ServerConfig::default()
    };
    tokio::spawn(async move {
        let _ = run(config).await;
    });
    for _ in 0..400 {
        if socket.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(socket.exists(), "server did not publish its socket");
    (root, socket)
}

async fn round_trip(socket: &Path, request: Value, wait: Duration) -> Value {
    let stream = UnixStream::connect(socket).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    writer
        .write_all(format!("{request}\n").as_bytes())
        .await
        .unwrap();
    writer.flush().await.unwrap();
    let mut line = String::new();
    tokio::time::timeout(wait, reader.read_line(&mut line))
        .await
        .expect("server never answered")
        .unwrap();
    serde_json::from_str(&line).unwrap_or(Value::Null)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_hung_endpoint_is_bounded_and_the_server_keeps_serving() {
    let (address, listener) = hung_endpoint().await;
    let (root, socket) = boot_with_hung_endpoint(address.port()).await;
    let policy = hung_policy(root.path());

    let started = Instant::now();
    let response = round_trip(
        &socket,
        json!({
            "verb": "agent_probe",
            "endpoint": "hung",
            "prompt": "health check",
            "policy": policy.to_str().unwrap(),
            "entity": "operator_local",
            "process": "test",
        }),
        // Generous: the assertion below is what pins the actual bound.
        Duration::from_secs(60),
    )
    .await;
    let elapsed = started.elapsed();

    assert_eq!(
        response["error_class"], "request_timeout",
        "a hung endpoint must produce a typed timeout, not a hang: {response}"
    );
    assert_eq!(response["timeout_seconds"], 10);
    assert_eq!(response["recover_with"], "request.get_status");
    assert!(
        elapsed < Duration::from_secs(25),
        "the bound did not fire: waited {elapsed:?}"
    );

    // The whole point of bounding one request is that the server survives it.
    let hello = round_trip(
        &socket,
        json!({"verb": "system_hello", "protocol_version": "1", "client": "test"}),
        Duration::from_secs(10),
    )
    .await;
    assert_eq!(
        hello["server_protocol_version"], "1",
        "the server stopped serving after a timed-out request: {hello}"
    );

    listener.abort();
}

/// An invalid `server.yaml` discovered on the reload path must reach the
/// operator through the event ledger, not only through a log line. `commit_plan`
/// reloads before it dispatches, so the event lands whether or not the submit
/// itself succeeds.
#[tokio::test(flavor = "multi_thread")]
async fn an_invalid_reload_publishes_an_operator_visible_event() {
    let (address, listener) = hung_endpoint().await;
    let (root, socket) = boot_with_hung_endpoint(address.port()).await;

    fs::write(
        root.path().join("server.yaml"),
        "max_concurrent_runs: [oops\n",
    )
    .unwrap();

    let _ = round_trip(
        &socket,
        json!({"verb": "submit", "intent": "anything", "policy": "policy.yaml"}),
        Duration::from_secs(30),
    )
    .await;

    let events = round_trip(
        &socket,
        json!({"verb": "events_get_range"}),
        Duration::from_secs(10),
    )
    .await;

    let kinds: Vec<&str> = events["events"]
        .as_array()
        .map(|frames| {
            frames
                .iter()
                .filter_map(|frame| frame["kind"].as_str())
                .collect()
        })
        .unwrap_or_default();
    assert!(
        kinds.contains(&"invalid_reload_error"),
        "an invalid reload must be operator-visible in the event ledger; saw {kinds:?} in {events}"
    );

    listener.abort();
}
