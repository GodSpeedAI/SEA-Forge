//! Governed actor identity over the wire (SF-005, decision U-07).
//!
//! The unit tests in `server::identity` prove the resolver's logic. These prove
//! the *gate is installed*: that a protected verb arriving on a real socket
//! without a verified actor is refused before it reaches a handler, and that an
//! inspect verb on the same connection is not.
//!
//! Those are different claims. A resolver that works perfectly and is never
//! called would satisfy the first set and none of these.

use sea_forge_server::identity::IdentityBindings;
use sea_forge_server::{run, ServerConfig};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, UnixStream};

/// The actor block a well-behaved client sends.
fn actor() -> Value {
    json!({"actor_id": "operator_local", "role": "operator"})
}

async fn boot(identity: IdentityBindings) -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    // Short, because a deep temp path overflows `sun_path` (see the cell
    // contract) and the failure would look like an identity problem.
    let socket = std::env::temp_dir().join(format!(
        "sf-id-{}-{}.sock",
        std::process::id(),
        root.path().file_name().unwrap().to_string_lossy()
    ));
    let config = ServerConfig {
        socket_path: socket.clone(),
        root: root.path().to_path_buf(),
        identity,
        ..ServerConfig::default()
    };
    tokio::spawn(async move {
        let _ = run(config).await;
    });
    for _ in 0..500 {
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
        let (reader, writer) = UnixStream::connect(socket).await.unwrap().into_split();
        Self {
            writer,
            reader: BufReader::new(reader),
        }
    }

    async fn call(&mut self, value: Value) -> Value {
        self.writer
            .write_all(format!("{value}\n").as_bytes())
            .await
            .unwrap();
        self.writer.flush().await.unwrap();
        let mut line = String::new();
        tokio::time::timeout(Duration::from_secs(30), self.reader.read_line(&mut line))
            .await
            .expect("read timed out")
            .unwrap();
        serde_json::from_str(line.trim()).unwrap()
    }
}

/// A protected verb with no actor block. The refusal has to be typed, and it
/// has to say plainly that nothing happened — an operator should not have to
/// infer "no side effect" from the absence of other fields.
#[tokio::test]
async fn a_protected_verb_without_an_actor_is_refused() {
    let (root, socket) = boot(IdentityBindings::local_operator("operator_local")).await;
    let mut client = Client::connect(&socket).await;

    let response = client
        .call(json!({"verb": "submit", "plan": "/nonexistent/plan.json",
                     "policy": "/nonexistent/policy.yaml",
                     "entity": "operator_local", "process": "test", "timeout": 30}))
        .await;

    assert_eq!(response["error_class"], "identity_required", "{response}");
    assert_eq!(response["no_side_effect"], true);
    // Nothing ran, so the cell has no cases at all — not an empty case, none.
    assert!(
        !root.path().join("cases").exists(),
        "a refused submit created {}/cases",
        root.path().display()
    );
}

/// The compatibility half of U-07 answer 4. An old client that never learned
/// about the actor block keeps working for everything that only reads.
#[tokio::test]
async fn inspect_verbs_still_work_without_an_actor() {
    let (_root, socket) = boot(IdentityBindings::local_operator("operator_local")).await;
    let mut client = Client::connect(&socket).await;

    for verb in ["run_list", "case_list", "agent_list", "asset_list"] {
        let response = client.call(json!({"verb": verb})).await;
        assert_ne!(
            response["error_class"], "identity_required",
            "inspect verb {verb} demanded an actor: {response}"
        );
    }
}

/// An unconfigured cell must not fall back to deriving an actor from whoever
/// connects. That fallback would be the same fabricated identity SF-005 exists
/// to remove, relocated into the server.
#[tokio::test]
async fn an_unconfigured_cell_refuses_protected_work_even_with_a_valid_actor() {
    let (root, socket) = boot(IdentityBindings::default()).await;
    let mut client = Client::connect(&socket).await;

    let response = client
        .call(json!({"verb": "submit", "actor": actor(),
                     "plan": "/nonexistent/plan.json", "policy": "/nonexistent/policy.yaml",
                     "entity": "operator_local", "process": "test", "timeout": 30}))
        .await;

    assert_eq!(
        response["error_class"], "identity_unconfigured",
        "{response}"
    );
    assert!(!root.path().join("cases").exists());
}

/// Claiming an actor this cell has never heard of must fail, even though the
/// uid is the one the cell is configured for. The binding is (uid, actor_id),
/// not either half alone.
#[tokio::test]
async fn an_unbound_actor_id_is_refused_on_a_configured_cell() {
    let (_root, socket) = boot(IdentityBindings::local_operator("operator_local")).await;
    let mut client = Client::connect(&socket).await;

    let response = client
        .call(json!({"verb": "submit",
                     "actor": {"actor_id": "someone_else", "role": "operator"},
                     "plan": "/nonexistent/plan.json", "policy": "/nonexistent/policy.yaml",
                     "entity": "operator_local", "process": "test", "timeout": 30}))
        .await;

    assert_eq!(response["error_class"], "identity_not_bound", "{response}");
}

/// A role the actor does not hold is refused. `local_operator` grants only
/// `operator`, so a claim to act as a security officer must not pass.
#[tokio::test]
async fn a_role_the_actor_does_not_hold_is_refused() {
    let (_root, socket) = boot(IdentityBindings::local_operator("operator_local")).await;
    let mut client = Client::connect(&socket).await;

    let response = client
        .call(json!({"verb": "submit",
                     "actor": {"actor_id": "operator_local", "role": "R-SO"},
                     "plan": "/nonexistent/plan.json", "policy": "/nonexistent/policy.yaml",
                     "entity": "operator_local", "process": "test", "timeout": 30}))
        .await;

    assert_eq!(
        response["error_class"], "identity_role_not_held",
        "{response}"
    );
}

/// A bound actor gets past the gate. The submit then fails on its own merits
/// (the plan path does not exist), which is the point: the *identity* check no
/// longer refused it.
#[tokio::test]
async fn a_bound_actor_passes_the_gate_and_is_judged_on_the_request_itself() {
    let (_root, socket) = boot(IdentityBindings::local_operator("operator_local")).await;
    let mut client = Client::connect(&socket).await;

    let response = client
        .call(json!({"verb": "submit", "actor": actor(),
                     "plan": "/nonexistent/plan.json", "policy": "/nonexistent/policy.yaml",
                     "entity": "operator_local", "process": "test", "timeout": 30}))
        .await;

    let class = response["error_class"].as_str().unwrap_or_default();
    assert!(
        !class.starts_with("identity_"),
        "a bound actor was still refused on identity: {response}"
    );
}
