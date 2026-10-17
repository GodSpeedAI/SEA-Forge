//! SEC-09/SEC-10/SEC-11 at the real MCP forwarding boundary.
//!
//! A context that genuinely received classified plaintext (observed by the
//! broker as it forwarded a classified tool result) can no longer send to
//! destinations not authorized for that data, whatever the agent was told by
//! injected content, however it transforms the data, across reconnection,
//! and in delegated children. The upstream's own log is the observation.
#![cfg(target_os = "linux")]

use sea_forge_authority::AuthorityPolicyBundle;
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_server::exposure::ExposureRegistry;
use sea_forge_server::mcp::{serve_workload, McpListenerGuard, McpServerConfig};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

const POLICY: &str = "version: \"0.1\"\nrules:\n  - name: reads\n    verdict: allow\n    actor_role: operator\n    operation_kind: mcp_tool_call\n    boundary_constraints:\n      mcp_server: [exposure-upstream]\n      mcp_tool: [ticket.read, metadata.read]\n      permitted_exposure: [confidential]\n  - name: external-sinks-for-unexposed-work\n    verdict: allow\n    actor_role: operator\n    operation_kind: mcp_tool_call\n    boundary_constraints:\n      mcp_server: [exposure-upstream]\n      mcp_tool: [notify.send, report.publish]\n  - name: internal-audit-sink\n    verdict: allow\n    actor_role: operator\n    operation_kind: mcp_tool_call\n    boundary_constraints:\n      mcp_server: [exposure-upstream]\n      mcp_tool: [audit.file]\n      permitted_exposure: [confidential]\n";

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    observer: PathBuf,
    injection: String,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let observer = root.join("observer");
    std::fs::create_dir_all(&observer).unwrap();
    std::fs::create_dir_all(root.join("workspace")).unwrap();
    let corpus: Value =
        serde_json::from_str(include_str!("../../../tests/security-v1/attacks.json")).unwrap();
    Fixture {
        injection: corpus["prompt_injection"]["lower_trust_document"]
            .as_str()
            .unwrap()
            .into(),
        observer,
        root,
        _dir: dir,
    }
}

fn broker(fx: &Fixture, context: &str) -> (McpListenerGuard, PathBuf) {
    let socket = fx.root.join(format!("{context}.sock"));
    let upstream = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/security-v2/exposure_upstream.py")
        .canonicalize()
        .unwrap();
    let guard = serve_workload(
        UnixListener::bind(&socket).unwrap(),
        std::fs::metadata("/proc/self").unwrap().uid(),
        fx.root.clone(),
        fx.root.join("workspace"),
        sea_forge_core::yaml::from_str::<AuthorityPolicyBundle>(POLICY).unwrap(),
        Actor {
            actor_id: "support-agent".into(),
            role: ActorRole::Operator,
        },
        McpServerConfig {
            name: "exposure-upstream".into(),
            argv: vec!["/usr/bin/python3".into(), upstream.display().to_string()],
            env: [
                (
                    "SEA_FORGE_SECURITY_OBSERVER_DIR".to_string(),
                    fx.observer.display().to_string(),
                ),
                ("SEA_FORGE_INJECTION_TEXT".to_string(), fx.injection.clone()),
            ]
            .into(),
            classify: [("ticket.read".to_string(), "confidential".to_string())].into(),
        },
        Some(context.into()),
    )
    .unwrap();
    (guard, socket)
}

struct Client(BufReader<UnixStream>, u64);

impl Client {
    fn connect(socket: &Path) -> Self {
        let stream = UnixStream::connect(socket).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut client = Client(BufReader::new(stream), 1);
        client.rpc(
            "initialize",
            json!({"protocolVersion":"2024-11-05","capabilities":{},
            "clientInfo":{"name":"t","version":"1"}}),
        );
        writeln!(
            client.0.get_mut(),
            "{}",
            json!({"jsonrpc":"2.0","method":"notifications/initialized"})
        )
        .unwrap();
        client
    }
    fn rpc(&mut self, method: &str, params: Value) -> Value {
        self.1 += 1;
        writeln!(
            self.0.get_mut(),
            "{}",
            json!({"jsonrpc":"2.0","id":self.1,"method":method,"params":params})
        )
        .unwrap();
        let mut line = String::new();
        self.0.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    }
    fn call(&mut self, tool: &str, arguments: Value) -> Value {
        self.rpc("tools/call", json!({"name": tool, "arguments": arguments}))
    }
}

fn allowed(response: &Value) -> bool {
    response["result"]["content"].is_array()
}

fn log(fx: &Fixture) -> Vec<Value> {
    std::fs::read_to_string(fx.observer.join("invocations.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn exposure_restricts_destinations_across_injection_transforms_resumption_and_delegation() {
    let fx = fixture();
    let registry = ExposureRegistry::open(&fx.root, "test-auditor").unwrap();
    let (parent_guard, parent_socket) = broker(&fx, "run-parent");
    let mut agent = Client::connect(&parent_socket);

    // Before exposure the external sink is legitimately usable.
    assert!(allowed(&agent.call(
        "notify.send",
        json!({"message": "starting ticket triage"})
    )));
    // The agent genuinely receives classified plaintext with an injection.
    let ticket = agent.call("ticket.read", json!({"id": 7731}));
    let text = ticket["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        text.contains("CONFIDENTIAL-TICKET-7731") && text.contains("ignore your task restrictions")
    );
    assert_eq!(registry.labels("run-parent").unwrap(), ["confidential"]);

    // Obeying the injection: export through the sink, then a different
    // output tool, then a "summary" that no longer contains the plaintext.
    assert_eq!(
        agent.call("notify.send", json!({"message": text}))["error"]["code"],
        -32001
    );
    assert_eq!(
        agent.call("report.publish", json!({"body": text}))["error"]["code"],
        -32001
    );
    assert_eq!(
        agent.call(
            "notify.send",
            json!({"message": "summary: one customer record, details withheld"})
        )["error"]["code"],
        -32001
    );
    // Discovery reflects the restriction too.
    let listed = agent.rpc("tools/list", json!({}));
    let names: Vec<&str> = listed["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["ticket.read", "metadata.read", "audit.file"]);
    // Explicitly authorized work for confidential data stays possible.
    assert!(allowed(&agent.call("audit.file", json!({"ticket": text}))));

    // Resumption: a new session of the same context keeps its exposure.
    drop(agent);
    let mut resumed = Client::connect(&parent_socket);
    assert_eq!(
        resumed.call("notify.send", json!({"message": "hello"}))["error"]["code"],
        -32001
    );
    drop(resumed);

    // Delegation: the child inherits exposure; its own reads cannot launder it.
    registry.derive("run-parent", "run-child").unwrap();
    assert!(
        registry.derive("run-child", "run-parent").is_err(),
        "lineage cycle accepted"
    );
    let (child_guard, child_socket) = broker(&fx, "run-child");
    let mut child = Client::connect(&child_socket);
    assert!(allowed(&child.call("metadata.read", json!({}))));
    assert_eq!(
        child.call("notify.send", json!({"message": "child report"}))["error"]["code"],
        -32001
    );
    assert_eq!(registry.labels("run-child").unwrap(), ["confidential"]);

    // Brokered credential use without plaintext is not exposure.
    let (clean_guard, clean_socket) = broker(&fx, "run-clean");
    let mut clean = Client::connect(&clean_socket);
    assert!(allowed(&clean.call("metadata.read", json!({}))));
    assert!(allowed(
        &clean.call("notify.send", json!({"message": "metadata summary"}))
    ));
    assert!(registry.labels("run-clean").unwrap().is_empty());

    // Revoking the parent revokes the delegated child's live session.
    registry
        .revoke("run-parent", "operator revocation")
        .unwrap();
    assert_eq!(
        child.call("metadata.read", json!({}))["error"]["code"],
        -32003
    );
    assert!(
        allowed(&clean.call("metadata.read", json!({}))),
        "unrelated context affected"
    );
    drop((child, clean, parent_guard, child_guard, clean_guard));

    // Every brokered attempt is attributed to the kernel-verified workload
    // and its context, next to the decision and the request digest.
    let uid = std::fs::metadata("/proc/self").unwrap().uid();
    let attempts: Vec<Value> = std::fs::read_dir(fx.root.join("ledgers"))
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("mcp-"))
        .flat_map(|e| {
            let lines = std::fs::read_to_string(e.path().join("entries.jsonl")).unwrap_or_default();
            lines
                .lines()
                .map(|l| serde_json::from_str::<Value>(l).unwrap())
                .collect::<Vec<_>>()
        })
        .filter(|e| e["record_kind"] == "mcp_attempt")
        .map(|e| e["payload"].clone())
        .collect();
    assert!(!attempts.is_empty());
    assert!(
        attempts.iter().all(|a| a["workload_uid"] == uid
            && a["context"].is_string()
            && a["decision_id"].is_string()
            && a["request_sha256"].is_string()),
        "{attempts:?}"
    );
    // Upstream observation: no sink ever received exposed data or calls
    // made from exposed contexts; only the two unexposed notifications ran.
    let calls = log(&fx);
    let count = |tool: &str| calls.iter().filter(|c| c["tool"] == tool).count();
    assert_eq!(count("notify.send"), 2, "{calls:?}");
    assert_eq!(count("report.publish"), 0);
    assert_eq!(count("audit.file"), 1);
    assert!(calls
        .iter()
        .filter(|c| c["tool"] != "audit.file")
        .all(|c| c["carries_secret"] == false));
}
