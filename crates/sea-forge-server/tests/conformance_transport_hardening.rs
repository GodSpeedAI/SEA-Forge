//! Transport hardening conformance (§11.1 socket ownership and permissions).
//!
//! These talk over a real Unix socket with a real `ServerConfig`/`ServerState`,
//! mirroring `conformance_sfwp.rs`, because every property here is about the
//! socket as an operating-system object: who may own it, who may connect to
//! it, and how much a connected client may make the server allocate.
//!
//! Each test fails if its hardening is reverted.

use sea_forge_server::{run, ServerConfig};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

/// Must match `MAX_REQUEST_BYTES` in `handle_connection`.
const MAX_REQUEST_BYTES: usize = 1024 * 1024;

/// A liveness probe that needs no case or policy fixture.
const PROBE: &str = "{\"verb\":\"system_hello\",\"protocol_version\":\"1\",\"client\":\"test\"}\n";

fn config_for(root: &tempfile::TempDir, socket: &Path) -> ServerConfig {
    ServerConfig {
        socket_path: socket.to_path_buf(),
        root: root.path().to_path_buf(),
        ..ServerConfig::default()
    }
}

/// Boot a server and wait for its socket to be published.
async fn boot() -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let socket = root.path().join("hardening.sock");
    let config = config_for(&root, &socket);
    tokio::spawn(async move {
        let _ = run(config).await;
    });
    for _ in 0..200 {
        if socket.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(socket.exists(), "server did not publish its socket");
    (root, socket)
}

/// Probe the server and assert it answered as a live SFWP endpoint.
async fn assert_still_serving(socket: &Path, context: &str) {
    let response = round_trip(socket, PROBE).await;
    assert_eq!(
        response["server_protocol_version"], "1",
        "{context}; got: {response}"
    );
}

/// Send one NDJSON request and read one response line.
async fn round_trip(socket: &Path, request: &str) -> Value {
    let stream = UnixStream::connect(socket).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    writer.write_all(request.as_bytes()).await.unwrap();
    writer.flush().await.unwrap();
    let mut line = String::new();
    tokio::time::timeout(Duration::from_secs(10), reader.read_line(&mut line))
        .await
        .expect("server did not answer in time")
        .unwrap();
    serde_json::from_str(&line).unwrap_or(Value::Null)
}

/// A second server on a live socket must fail closed rather than unlink it.
///
/// Two servers sharing one socket path would both append to the same JSONL
/// ledger and MMR, and the append-only invariant has no cross-process
/// reconciliation for interleaved writers.
#[tokio::test]
async fn a_second_server_on_a_live_socket_fails_closed() {
    let (root, socket) = boot().await;

    // Bounded: without the lock the second `run` does not fail, it *succeeds*
    // and loops accepting forever, so an unbounded await would hang the suite
    // instead of reporting the regression.
    let second = tokio::time::timeout(Duration::from_secs(10), run(config_for(&root, &socket)))
        .await
        .expect("second server kept running — it stole the live socket instead of failing closed");

    assert!(
        second.is_err(),
        "a second server on a live socket must refuse to start"
    );
    let message = second.unwrap_err().to_string();
    assert!(
        message.contains("already owns cell"),
        "the refusal should name the cell ownership cause, got: {message}"
    );

    // The decisive property: the original server is untouched and still serving.
    assert_still_serving(
        &socket,
        "the live server must keep serving after the second one is refused",
    )
    .await;
}

/// Socket overrides cannot bypass ownership of the underlying cell root.
#[tokio::test]
async fn a_second_server_with_same_root_and_different_socket_fails_closed() {
    let (root, first_socket) = boot().await;
    let second_socket = root.path().join("other.sock");

    let second = tokio::time::timeout(
        Duration::from_secs(10),
        run(config_for(&root, &second_socket)),
    )
    .await
    .expect("second server kept running despite the cell lock");

    let message = second.unwrap_err().to_string();
    assert!(message.contains("already owns cell"), "{message}");
    assert!(
        !second_socket.exists(),
        "refused server must not publish a socket"
    );
    assert_still_serving(&first_socket, "the original cell owner must remain live").await;
}

/// A crashed server's leftover socket inode must not block a restart.
///
/// The lock is advisory and process-scoped, so it is released on exit however
/// the process died; the stale socket inode is safe for the staging rename to
/// replace. A regular file is not equivalent and must be preserved.
#[tokio::test]
async fn a_stale_socket_does_not_block_a_restart() {
    let root = tempfile::tempdir().unwrap();
    let socket = root.path().join("stale.sock");
    let stale_listener = UnixListener::bind(&socket).unwrap();
    drop(stale_listener);

    let config = config_for(&root, &socket);
    let socket_for_server = socket.clone();
    tokio::spawn(async move {
        let _ = run(config).await;
    });
    for _ in 0..200 {
        if UnixStream::connect(&socket_for_server).await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    assert_still_serving(
        &socket,
        "a stale socket inode must be replaced, not treated as fatal",
    )
    .await;
}

#[tokio::test]
async fn regular_file_socket_destination_is_refused_without_data_loss() {
    let root = tempfile::tempdir().unwrap();
    let socket = root.path().join("important-file");
    let original = b"do not replace";
    std::fs::write(&socket, original).unwrap();

    let failure = run(config_for(&root, &socket))
        .await
        .unwrap_err()
        .to_string();

    assert!(failure.contains("non-socket"), "{failure}");
    assert_eq!(std::fs::read(&socket).unwrap(), original);
}

#[cfg(unix)]
#[tokio::test]
async fn symlink_socket_destination_is_refused_without_touching_its_target() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("target");
    let socket = root.path().join("link.sock");
    std::fs::write(&target, b"protected target").unwrap();
    symlink(&target, &socket).unwrap();

    let failure = run(config_for(&root, &socket))
        .await
        .unwrap_err()
        .to_string();

    assert!(failure.contains("symlink"), "{failure}");
    assert_eq!(std::fs::read(&target).unwrap(), b"protected target");
}

/// The socket must never be reachable by other local users, including during
/// the window between `bind()` and the permission change.
#[cfg(unix)]
#[tokio::test]
async fn the_published_socket_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;

    let (_root, socket) = boot().await;
    let mode = std::fs::metadata(&socket).unwrap().permissions().mode();

    assert_eq!(
        mode & 0o077,
        0,
        "socket must not be group- or world-accessible, got mode {:o}",
        mode & 0o777
    );
}

/// A client that never sends a newline must not be able to make the server
/// allocate without bound.
#[tokio::test]
async fn an_oversized_request_line_is_rejected_and_the_server_survives() {
    let (_root, socket) = boot().await;

    let stream = UnixStream::connect(&socket).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Deliberately no newline: unbounded `read_line` would grow forever.
    let flood = "a".repeat(MAX_REQUEST_BYTES + 4096);
    // The server stops reading at the cap and closes, so the write may fail
    // partway — that is the hardening working, not a test failure.
    let _ = writer.write_all(flood.as_bytes()).await;
    let _ = writer.flush().await;

    let mut line = String::new();
    tokio::time::timeout(Duration::from_secs(10), reader.read_line(&mut line))
        .await
        .expect("server must answer an oversized line rather than hang")
        .expect("reading the refusal must not fail");

    assert!(
        line.contains("exceeds"),
        "oversized line should be refused with a stated limit, got: {line}"
    );

    // The bound must cost only the offending connection, not the daemon.
    assert_still_serving(
        &socket,
        "the server must keep serving after refusing an oversized line",
    )
    .await;
}

/// A partial NDJSON request consumes one bounded connection permit only until
/// the §11.1 request deadline; it cannot keep a task and buffer forever.
#[tokio::test]
async fn partial_request_line_times_out_and_releases_the_connection() {
    let (_root, socket) = boot().await;
    let stream = UnixStream::connect(&socket).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    writer.write_all(b"{").await.unwrap();
    writer.flush().await.unwrap();

    let mut line = String::new();
    let read = tokio::time::timeout(Duration::from_secs(12), reader.read_line(&mut line))
        .await
        .expect("partial request was not timed out");
    assert_eq!(read.unwrap(), 0, "timed-out connection must close");
    assert_still_serving(&socket, "partial request timeout must not stop the server").await;
}

/// The accept loop must treat a failed accept as per-connection, not fatal.
///
/// This does not induce `EMFILE`/`ECONNABORTED` — neither is portably
/// reproducible from a test — so it is a regression guard for the weaker,
/// testable property: connection churn, including clients that vanish
/// immediately after connecting, never terminates the server.
#[tokio::test]
async fn connection_churn_does_not_terminate_the_server() {
    let (_root, socket) = boot().await;

    for _ in 0..200 {
        if let Ok(stream) = UnixStream::connect(&socket).await {
            drop(stream);
        }
    }

    assert_still_serving(&socket, "the server must survive connection churn").await;
}
