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

fn config(root: &Path) -> ServerConfig {
    ServerConfig {
        root: root.to_path_buf(),
        socket_path: root.join("server.sock"),
        identity: identity::IdentityBindings::local_operator("operator_local"),
        ..ServerConfig::default()
    }
}

fn request() -> serde_json::Value {
    serde_json::json!({
        "verb": "case_commit", "actor": {"actor_id":"operator_local", "role":"operator"},
        "template_ref": "missing@1", "entity": "operator_local", "request_id": REQUEST_ID
    })
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
        Duration::from_secs(5),
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
    assert!(!root.path().join("cases").exists());
    assert!(!root.path().join("runs").exists());
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
    assert!(
        !root.path().join("cases").exists(),
        "retry must not create a case"
    );
    assert!(
        !root.path().join("runs").exists(),
        "retry must not create a run"
    );
    let status = call(&socket, &serde_json::json!({
        "verb":"request_get_status", "actor":{"actor_id":"operator_local", "role":"operator"}, "request_id":REQUEST_ID
    })).await;
    assert_eq!(status["error_class"], "request_status_denied", "{status}");
    restarted.abort();
}
