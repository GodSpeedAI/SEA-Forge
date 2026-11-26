//! Real process death between the durable admission claim and its projection.
use super::*;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

const ROOT_ENV: &str = "SEA_FORGE_TEST_ADMISSION_CRASH_ROOT";
const MARKER_ENV: &str = "SEA_FORGE_TEST_ADMISSION_CRASH_MARKER";
const REQUEST_ID: &str = "crash-boundary-request";

/// Test builds only: notify the parent after the real ledger commit returns,
/// then stay at the exact boundary until the parent kills this process.
pub(super) fn pause_after_claim() {
    let Some(marker) = std::env::var_os(MARKER_ENV) else {
        return;
    };
    std::fs::write(marker, b"claim committed").unwrap();
    loop {
        std::thread::park();
    }
}

const ENDPOINT: &str = "crash-test-endpoint";
const TEMPLATE_REF: &str = "sequential_agents@0.1.0";

fn config(root: &Path) -> ServerConfig {
    let endpoint = sea_forge_agent::AgentEndpointConfig {
        id: ENDPOINT.into(),
        kind: sea_forge_agent::ProviderKind::OpenAiCompatible,
        base_url: Some("http://127.0.0.1:9/".into()),
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
    ServerConfig {
        root: root.to_path_buf(),
        socket_path: root.join("server.sock"),
        identity: identity::IdentityBindings::local_operator("operator_local"),
        agent: sea_forge_agent::AgentConfig {
            endpoints: vec![endpoint],
            ..sea_forge_agent::AgentConfig::default()
        },
        ..ServerConfig::default()
    }
}

/// Materialize what a committable `case_commit` needs: a template and a policy.
fn prepare(root: &Path) {
    let templates = root.join("templates");
    std::fs::create_dir_all(&templates).unwrap();
    let template = sea_forge_planner::templates::sequential_agents_template(ENDPOINT).unwrap();
    std::fs::write(
        templates.join(format!("{TEMPLATE_REF}.yaml")),
        sea_forge_core::yaml::to_string(&template).unwrap(),
    )
    .unwrap();
    std::fs::write(
        root.join("policy.yaml"),
        "version: \"0.1\"\npolicy_surfaces:\n  external_api:\n    mode: deny-by-default\n    allow_hosts: [127.0.0.1]\nrules:\n  - name: allow-agent-task\n    verdict: allow\n    actor_role: operator\n    operation_kind: agent_task\n  - name: allow-propose\n    verdict: allow\n    actor_role: operator\n    operation_kind: discretionary_task_add\n",
    )
    .unwrap();
}

/// A request that WOULD create a case if admission let it run.
fn request_with_id(request_id: &str) -> serde_json::Value {
    serde_json::json!({
        "verb": "case_commit", "actor": {"actor_id":"operator_local", "role":"operator"},
        "template_ref": TEMPLATE_REF, "params": {}, "policy": "policy.yaml",
        "entity": "operator_local", "process": "test", "request_id": request_id
    })
}

fn request() -> serde_json::Value {
    request_with_id(REQUEST_ID)
}

fn entries_in(dir: PathBuf) -> usize {
    std::fs::read_dir(dir).map(|d| d.count()).unwrap_or(0)
}

fn claim_ledger_len(root: &Path, request_id: &str) -> usize {
    let id = request_admission_claim::ledger_id(request_id);
    sea_forge_ledger::LedgerStream::open(root, id, "test")
        .unwrap()
        .read_entries()
        .unwrap()
        .len()
}

struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

async fn connect(socket: &Path) -> UnixStream {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(stream) = UnixStream::connect(socket).await {
            return stream;
        }
        assert!(
            Instant::now() < deadline,
            "server did not publish its socket"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

async fn call(socket: &Path, value: &serde_json::Value) -> serde_json::Value {
    let mut stream = connect(socket).await;
    stream
        .write_all(format!("{value}\n").as_bytes())
        .await
        .unwrap();
    let mut line = String::new();
    tokio::time::timeout(
        Duration::from_secs(30),
        BufReader::new(stream).read_line(&mut line),
    )
    .await
    .unwrap()
    .unwrap();
    serde_json::from_str(&line).unwrap()
}

/// The parent invokes only this exact test in the child process.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn server_child() {
    let Some(root) = std::env::var_os(ROOT_ENV) else {
        return;
    };
    run(config(Path::new(&root))).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn killed_after_claim_before_projection_refuses_retry_after_restart() {
    use std::os::unix::process::ExitStatusExt;
    let root = tempfile::tempdir().unwrap();
    prepare(root.path());
    let marker = root.path().join("claim-committed.marker");
    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "admission_crash_tests::server_child",
            "--nocapture",
        ])
        .env(ROOT_ENV, root.path())
        .env(MARKER_ENV, &marker)
        .stdin(Stdio::null())
        .spawn()
        .unwrap();
    let mut child = ChildGuard(child);
    let socket = root.path().join("server.sock");
    let mut connection = connect(&socket).await;
    let original = request();
    connection
        .write_all(format!("{original}\n").as_bytes())
        .await
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !marker.exists() {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "server exited before the boundary"
        );
        assert!(
            Instant::now() < deadline,
            "server did not reach the claim boundary"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let claim = request_admission_claim::lookup(root.path(), REQUEST_ID)
        .unwrap()
        .unwrap();
    let parsed: Request = serde_json::from_value(original.clone()).unwrap();
    let (_, _, expected_hash) = dedupe_key(&parsed, &original.to_string()).unwrap();
    assert_eq!(claim.payload_hash, expected_hash);
    let correlation = RequestCorrelationStore::open(root.path()).unwrap();
    assert!(correlation.get(REQUEST_ID).unwrap().is_none());
    assert_eq!(entries_in(root.path().join("cases")), 0);
    assert_eq!(entries_in(root.path().join("runs")), 0);
    assert_eq!(claim_ledger_len(root.path(), REQUEST_ID), 1);
    child.0.kill().unwrap();
    let status = child.0.wait().unwrap();
    assert_eq!(
        status.signal(),
        Some(9),
        "child was not killed at the boundary"
    );
    drop(connection);

    let restarted_config = config(root.path());
    let restarted = tokio::spawn(async move {
        let _ = run(restarted_config).await;
    });
    let response = call(&socket, &original).await;
    assert_eq!(
        response["error_class"], "idempotency_unverifiable",
        "{response}"
    );
    assert_eq!(
        request_admission_claim::lookup(root.path(), REQUEST_ID).unwrap(),
        Some(claim)
    );
    assert!(correlation.get(REQUEST_ID).unwrap().is_none());
    assert_eq!(
        entries_in(root.path().join("cases")),
        0,
        "retry must not create a case"
    );
    assert_eq!(
        entries_in(root.path().join("runs")),
        0,
        "retry must not create a run"
    );
    assert_eq!(
        claim_ledger_len(root.path(), REQUEST_ID),
        1,
        "retry must not append a second claim"
    );
    assert_eq!(response["reason_code"], "orphan_claim", "{response}");
    let status = call(&socket, &serde_json::json!({
        "verb":"request_get_status", "actor":{"actor_id":"operator_local", "role":"operator"}, "request_id":REQUEST_ID
    })).await;
    assert_eq!(status["error_class"], "request_status_denied", "{status}");
    // Control: the very same request under a fresh id is committable, so the
    // zero-case assertions above are not vacuous.
    let control = call(&socket, &request_with_id("crash-control-request")).await;
    assert!(control.get("error").is_none(), "{control}");
    assert_eq!(entries_in(root.path().join("cases")), 1, "{control}");
    restarted.abort();
}

/// A transient failure to bind the correlation record, after the claim has
/// committed and before any handler ran, must tell the client the id is spent.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unwritable_correlation_after_claim_says_request_id_is_spent() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    prepare(root.path());
    let cfg = config(root.path());
    let socket = cfg.socket_path.clone();
    let server = tokio::spawn(async move {
        let _ = run(cfg).await;
    });
    drop(connect(&socket).await);
    let requests = root.path().join("requests");
    std::fs::create_dir_all(&requests).unwrap();
    std::fs::set_permissions(&requests, std::fs::Permissions::from_mode(0o555)).unwrap();
    if std::fs::write(requests.join("probe"), b"x").is_ok() {
        // Running with privileges that ignore directory modes: cannot inject.
        std::fs::set_permissions(&requests, std::fs::Permissions::from_mode(0o755)).unwrap();
        server.abort();
        return;
    }
    let response = call(&socket, &request_with_id("spent-request")).await;
    std::fs::set_permissions(&requests, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(
        response["error_class"], "idempotency_unverifiable",
        "{response}"
    );
    assert_eq!(response["no_side_effect"], true, "{response}");
    assert_eq!(response["reason_code"], "request_id_spent", "{response}");
    assert_eq!(response["new_request_id_required"], true, "{response}");
    assert_eq!(entries_in(root.path().join("cases")), 0);
    assert_eq!(claim_ledger_len(root.path(), "spent-request"), 1);
    server.abort();
}
