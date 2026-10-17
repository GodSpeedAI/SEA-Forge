//! Lifecycle of the per-workload MCP broker socket (SEC-01/SEC-03/SEC-08).
//!
//! Observations come from the upstream itself: a wrapper records every
//! upstream process start, and the fixture upstream logs every tool call.
#![cfg(target_os = "linux")]

use sea_forge_authority::AuthorityPolicyBundle;
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_ledger::LedgerStream;
use sea_forge_server::mcp::{serve_workload, McpServerConfig};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    observer: PathBuf,
    socket: PathBuf,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let observer = root.join("observer");
    std::fs::create_dir_all(&observer).unwrap();
    std::fs::create_dir_all(root.join("workspace")).unwrap();
    Fixture {
        socket: root.join("broker.sock"),
        observer,
        root,
        _dir: dir,
    }
}

fn start(fx: &Fixture, uid: u32) -> sea_forge_server::mcp::McpListenerGuard {
    let upstream = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/security-v1/mcp_upstream.py")
        .canonicalize()
        .unwrap();
    // Record each upstream process start (pid) before handing over.
    let wrapper = format!(
        "echo $$ >> {}/spawned; exec /usr/bin/python3 {}",
        fx.observer.display(),
        upstream.display()
    );
    let bundle: AuthorityPolicyBundle =
        sea_forge_core::yaml::from_str(include_str!("../../../tests/security-v2/mcp-policy.yaml"))
            .unwrap();
    serve_workload(
        UnixListener::bind(&fx.socket).unwrap(),
        uid,
        fx.root.clone(),
        fx.root.join("workspace"),
        bundle,
        Actor {
            actor_id: "metadata-workload".into(),
            role: ActorRole::Operator,
        },
        McpServerConfig {
            name: "security-upstream".into(),
            argv: vec!["/bin/sh".into(), "-c".into(), wrapper],
            env: [(
                "SEA_FORGE_SECURITY_OBSERVER_DIR".to_string(),
                fx.observer.display().to_string(),
            )]
            .into(),
        },
    )
    .unwrap()
}

struct Client(BufReader<UnixStream>);

impl Client {
    fn connect(socket: &Path) -> Self {
        let stream = UnixStream::connect(socket).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        Client(BufReader::new(stream))
    }
    fn send(&mut self, frame: Value) {
        // A closed broker surfaces as a missing response, not a panic.
        let _ = writeln!(self.0.get_mut(), "{frame}");
    }
    fn call(&mut self, frame: Value) -> Option<Value> {
        self.send(frame);
        let mut line = String::new();
        match self.0.read_line(&mut line) {
            Ok(0) | Err(_) => None,
            Ok(_) => Some(serde_json::from_str(&line).unwrap()),
        }
    }
    fn handshake(&mut self) {
        let init = self.call(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}));
        assert!(init.unwrap()["result"].is_object());
        self.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    }
}

fn call_tool(id: u64, name: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":{}}})
}

fn spawned(fx: &Fixture) -> Vec<u32> {
    std::fs::read_to_string(fx.observer.join("spawned"))
        .unwrap_or_default()
        .lines()
        .map(|l| l.parse().unwrap())
        .collect()
}

fn own_uid() -> u32 {
    std::fs::metadata("/proc/self").unwrap().uid()
}

#[test]
fn foreign_peer_is_refused_before_any_upstream_starts() {
    let fx = fixture();
    let guard = start(&fx, own_uid() + 1);
    let mut client = Client::connect(&fx.socket);
    let response = client.call(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}));
    assert_eq!(response, None, "foreign peer got a broker response");
    drop(guard);
    assert!(
        spawned(&fx).is_empty(),
        "upstream started for a foreign peer"
    );
    let ledger = LedgerStream::open(
        &fx.root,
        "mcp-listener-security-upstream",
        "metadata-workload",
    )
    .unwrap();
    let refused = ledger.read_entries().unwrap();
    assert_eq!(refused.len(), 1);
    assert_eq!(refused[0].record_kind, "mcp_peer_refused");
    assert_eq!(refused[0].payload["peer_uid"], own_uid());
}

#[test]
fn reconnection_is_a_fresh_session_and_shutdown_kills_the_upstream() {
    let fx = fixture();
    let guard = start(&fx, own_uid());
    let mut first = Client::connect(&fx.socket);
    first.handshake();
    let read = first.call(call_tool(2, "metadata.read")).unwrap();
    assert!(read["result"]["content"].is_array(), "{read}");
    // Unsupported protocol surface is refused locally, never forwarded.
    let resources = first
        .call(json!({"jsonrpc":"2.0","id":3,"method":"resources/list"}))
        .unwrap();
    assert_eq!(resources["error"]["code"], -32601);
    drop(first);

    // A reconnect inherits nothing: tools are unavailable until it initializes.
    let mut second = Client::connect(&fx.socket);
    let early = second.call(call_tool(4, "metadata.read")).unwrap();
    assert_eq!(early["error"]["code"], -32601, "{early}");
    second.handshake();
    let denied = second.call(call_tool(5, "audit.delete")).unwrap();
    assert_eq!(denied["error"]["code"], -32001);
    let pids = spawned(&fx);
    assert_eq!(
        pids.len(),
        2,
        "each session needs its own upstream: {pids:?}"
    );
    assert!(Path::new(&format!("/proc/{}", pids[1])).exists());

    // Stopping the broker while a session is open ends it and its upstream.
    drop(guard);
    assert_eq!(second.call(call_tool(6, "metadata.read")), None);
    for pid in pids {
        let status = std::fs::read_to_string(format!("/proc/{pid}/status")).unwrap_or_default();
        assert!(
            status.is_empty() || status.contains("State:\tZ"),
            "upstream {pid} survived"
        );
    }
    let log = std::fs::read_to_string(fx.observer.join("invocations.jsonl")).unwrap();
    let tools: Vec<String> = log
        .lines()
        .map(|l| {
            serde_json::from_str::<Value>(l).unwrap()["tool"]
                .as_str()
                .unwrap()
                .into()
        })
        .collect();
    assert_eq!(
        tools,
        ["metadata.read"],
        "only the first session's permitted call ran"
    );
}

#[test]
fn revocation_barrier_ends_the_live_session_before_acknowledging() {
    let fx = fixture();
    let guard = start(&fx, own_uid());
    let mut client = Client::connect(&fx.socket);
    client.handshake();
    // Pre-revocation control: the permitted call reaches the upstream.
    assert!(client.call(call_tool(2, "metadata.read")).unwrap()["result"].is_object());
    let pids = spawned(&fx);
    let ack = guard.revoke("operator revocation").unwrap();
    // The upstream is gone before the acknowledgement exists.
    for pid in &pids {
        let status = std::fs::read_to_string(format!("/proc/{pid}/status")).unwrap_or_default();
        assert!(
            status.is_empty() || status.contains("State:\tZ"),
            "upstream {pid} survived"
        );
    }
    assert_eq!(client.call(call_tool(3, "metadata.read")), None);
    assert!(
        UnixStream::connect(&fx.socket).is_err() || {
            let mut late = Client::connect(&fx.socket);
            late.call(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}))
                .is_none()
        }
    );
    let log = std::fs::read_to_string(fx.observer.join("invocations.jsonl")).unwrap();
    assert_eq!(
        log.lines().count(),
        1,
        "an effect was forwarded after the barrier"
    );
    let ledger = LedgerStream::open(
        &fx.root,
        "mcp-listener-security-upstream",
        "metadata-workload",
    )
    .unwrap();
    let records = ledger.read_entries().unwrap();
    assert_eq!(records.last().unwrap().record_kind, "mcp_session_revoked");
    assert_eq!(
        records.last().unwrap().payload["acknowledged_at"],
        ack["acknowledged_at"]
    );
}
