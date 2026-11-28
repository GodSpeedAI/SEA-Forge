#![cfg(unix)]
//! The approval inbox flags `expired` by comparing INSTANTS: `expires_at` at or before now, whatever spelling
//! the stored stamp uses, and an `expires_at` that does not parse is expired (fail closed).
use sea_forge_core::types::{ApprovalRequest, ApprovalStatus};
use sea_forge_core::RECORD_VERSION;
use sea_forge_server::{run, ServerConfig};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, UnixStream};

async fn boot() -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let socket = root.path().join("sfwp.sock");
    let config = ServerConfig {
        identity: sea_forge_server::identity::IdentityBindings::local_operator("operator_local"),
        socket_path: socket.clone(),
        root: root.path().to_path_buf(),
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

fn approval(id: &str, expires_at: &str) -> ApprovalRequest {
    ApprovalRequest {
        version: RECORD_VERSION.into(),
        approval_id: id.into(),
        run_id: "run-1".into(),
        case_id: "case-1".into(),
        decision_id: "dec-1".into(),
        plan_item_id: "item-1".into(),
        criteria_ref: None,
        criteria_sha256: None,
        criteria_record_hash: None,
        job_contract_ref: None,
        requested_at: "2026-07-26T00:00:00Z".into(),
        expires_at: expires_at.into(),
        status: ApprovalStatus::Pending,
        resolved_by: None,
        resolved_at: None,
        note: None,
    }
}

#[tokio::test]
async fn approval_inbox_expiry_compares_instants_and_fails_closed() {
    let (root, socket) = boot().await;
    // Spellings of instants far from now cannot be told apart by the clock; the comparison itself is
    // what changed, so each row is decided by what the stamp MEANS.
    let cases: [(&str, bool); 12] = [
        ("2000-01-01T00:00:00Z", true),
        ("2000-01-01T00:00:00+00:00", true),
        ("2000-01-01T05:30:00+05:30", true),
        ("2000-01-01T00:00:00.123456789Z", true),
        ("1999-12-31T16:00:00-08:00", true),
        ("2099-01-01T00:00:00Z", false),
        ("2099-01-01T00:00:00+00:00", false),
        ("2099-01-01T05:30:00+05:30", false),
        ("2098-12-31T16:00:00-08:00", false),
        // Unparsable: expired, whatever the text would have sorted as.
        ("zzz", true),
        ("ABC", true),
        ("", true),
    ];
    for (i, (expires, _)) in cases.iter().enumerate() {
        sea_forge_core::approvals::append(root.path(), &approval(&format!("ap-{i}"), expires))
            .unwrap();
    }
    let mut c = Client::connect(&socket).await;
    let listed = c.call(json!({"verb": "approval_list"})).await;
    let approvals = listed["approvals"].as_array().unwrap();
    assert_eq!(approvals.len(), cases.len(), "{listed}");
    for (i, (expires, expired)) in cases.iter().enumerate() {
        let row = approvals
            .iter()
            .find(|a| a["approval_id"] == format!("ap-{i}"))
            .unwrap();
        assert_eq!(
            row["expired"],
            json!(expired),
            "expires_at {expires:?}: {row}"
        );
        assert_eq!(
            row["expires_at"],
            json!(expires),
            "the stored text is returned verbatim"
        );
    }
}
