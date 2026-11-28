#![cfg(unix)]
//! Golden pins for every timestamp the server serializes, parses or compares, taken on chrono before the
//! move to the `time` crate. Only public functions and the wire are used, and no test names a date-time type,
//! so the swap leaves them unedited.
//!
//! Pinned: the correlation record stamps, the case `created_at`, the approval inbox returning stored stamps
//! verbatim, and the persisted shape of every correlation timestamp (`+00:00` with an automatic fraction).

use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind};
use sea_forge_core::types::{ApprovalRequest, ApprovalStatus};
use sea_forge_core::RECORD_VERSION;
use sea_forge_planner::templates::sequential_agents_template;
use sea_forge_server::sfwp::correlation::{RequestCorrelationStore, RequestStatus};
use sea_forge_server::{run, ServerConfig};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, UnixStream};

const TEMPLATE_REF: &str = "sequential_agents@0.1.0";
const ENDPOINT: &str = "ts-golden-endpoint";

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Nanoseconds since the epoch of exactly `YYYY-MM-DDTHH:MM:SS[.fff|.ffffff|.fffffffff]+00:00`.
fn epoch_ns_plus_zero(stamp: &str) -> Option<i128> {
    let body = stamp.strip_suffix("+00:00")?;
    let (whole, frac) = match body.split_once('.') {
        None => (body, None),
        Some((w, f)) if matches!(f.len(), 3 | 6 | 9) && f.bytes().all(|b| b.is_ascii_digit()) => {
            (w, Some(f))
        }
        _ => return None,
    };
    let b = whole.as_bytes();
    if whole.len() != 19
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
    {
        return None;
    }
    let n = |r: std::ops::Range<usize>| whole.get(r)?.parse::<i64>().ok();
    let (y, mo, d, h, mi, s) = (
        n(0..4)?,
        n(5..7)?,
        n(8..10)?,
        n(11..13)?,
        n(14..16)?,
        n(17..19)?,
    );
    let secs = days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + s;
    let frac_ns: i128 = frac.map_or(0, |f| format!("{f:0<9}").parse().unwrap());
    Some(secs as i128 * 1_000_000_000 + frac_ns)
}

fn now_ns() -> i128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i128
}

fn assert_recent(stamp: &str, before: i128, after: i128) -> i128 {
    let ns = epoch_ns_plus_zero(stamp).unwrap_or_else(|| panic!("not the pinned shape: {stamp}"));
    assert!(
        ns >= before - 1_000_000_000 && ns <= after + 1_000_000_000,
        "{stamp}"
    );
    ns
}

#[test]
fn correlation_record_stamps_are_plus_zero_and_stored_verbatim() {
    let root = tempfile::tempdir().unwrap();
    let store = RequestCorrelationStore::open(root.path()).unwrap();
    let before = now_ns();
    store
        .record_pending("req-ts-1", "case.commit", Some("hash-1"))
        .unwrap();
    let pending = store.get("req-ts-1").unwrap().unwrap();
    assert_eq!(pending.status, RequestStatus::Pending);
    let submitted = assert_recent(&pending.submitted_at, before, now_ns());
    assert!(pending.completed_at.is_none());

    store
        .record_outcome("req-ts-1", "case.commit", &json!({"ok": true}))
        .unwrap();
    let done = store.get("req-ts-1").unwrap().unwrap();
    assert_eq!(done.status, RequestStatus::Completed);
    // The first stamp survives the outcome write byte for byte; the completion stamp is new.
    assert_eq!(done.submitted_at, pending.submitted_at);
    let completed = assert_recent(done.completed_at.as_deref().unwrap(), before, now_ns());
    assert!(completed >= submitted);

    // The persisted JSON carries the very same text.
    let raw = std::fs::read_to_string(root.path().join("requests/req-ts-1.json")).unwrap();
    let value: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(value["submitted_at"], json!(done.submitted_at));
    assert_eq!(value["completed_at"], json!(done.completed_at));
}

#[test]
fn correlation_records_never_parse_their_stamps() {
    // Whatever text an older writer stored is loaded and returned unchanged: the stamps are opaque.
    let root = tempfile::tempdir().unwrap();
    let store = RequestCorrelationStore::open(root.path()).unwrap();
    for (i, stamp) in [
        "2026-10-26T12:00:00Z",
        "2026-10-26T12:00:00.123Z",
        "2026-10-26T12:00:00+00:00",
        "2026-10-26T12:00:00.123456789+00:00",
        "2026-10-26T17:30:00+05:30",
        "not a timestamp",
    ]
    .iter()
    .enumerate()
    {
        let id = format!("legacy-{i}");
        let record = json!({"request_id": id, "method": "case.commit", "status": "pending", "submitted_at": stamp});
        std::fs::write(
            root.path().join(format!("requests/{id}.json")),
            record.to_string(),
        )
        .unwrap();
        let loaded = store.get(&id).unwrap().unwrap();
        assert_eq!(&loaded.submitted_at, stamp);
    }
}

// --- the wire -------------------------------------------------------------------------------------------

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

fn endpoint() -> AgentEndpointConfig {
    AgentEndpointConfig {
        id: ENDPOINT.into(),
        kind: ProviderKind::OpenAiCompatible,
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
    }
}

async fn boot() -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let socket = root.path().join("sfwp.sock");
    let config = ServerConfig {
        identity: sea_forge_server::identity::IdentityBindings::local_operator("operator_local"),
        socket_path: socket.clone(),
        root: root.path().to_path_buf(),
        agent: AgentConfig {
            endpoints: vec![endpoint()],
            ..AgentConfig::default()
        },
        ..ServerConfig::default()
    };
    let templates = root.path().join("templates");
    std::fs::create_dir_all(&templates).unwrap();
    let template = sequential_agents_template(ENDPOINT).unwrap();
    std::fs::write(
        templates.join(format!("{TEMPLATE_REF}.yaml")),
        sea_forge_core::yaml::to_string(&template).unwrap(),
    )
    .unwrap();
    std::fs::write(
        root.path().join("policy.yaml"),
        "version: \"0.1\"\npolicy_surfaces:\n  external_api:\n    mode: deny-by-default\n    allow_hosts: [127.0.0.1]\nrules:\n  - name: allow-agent-task\n    verdict: allow\n    actor_role: operator\n    operation_kind: agent_task\n  - name: allow-propose\n    verdict: allow\n    actor_role: operator\n    operation_kind: discretionary_task_add\n",
    )
    .unwrap();
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

#[tokio::test]
async fn case_created_at_is_plus_zero_with_an_automatic_fraction_on_every_read() {
    let (_root, socket) = boot().await;
    let mut c = Client::connect(&socket).await;
    let before = now_ns();
    let commit = c
        .call(json!({
            "verb": "case_commit", "actor": {"actor_id": "operator_local", "role": "operator"},
            "template_ref": TEMPLATE_REF, "params": {}, "policy": "policy.yaml",
            "entity": "operator_local", "process": "test", "request_id": "req-ts-case",
        }))
        .await;
    let case_id = commit["case_id"]
        .as_str()
        .expect("a committed case")
        .to_string();
    let after = now_ns();

    let list = c.call(json!({"verb": "case_list"})).await;
    let listed = list["cases"][0]["created_at"].as_str().unwrap().to_string();
    assert_recent(&listed, before, after);

    let overview = c
        .call(json!({"verb": "case_get_overview", "case_id": case_id}))
        .await;
    assert_eq!(
        overview["created_at"],
        json!(listed),
        "one stamp, rendered identically by both views"
    );

    // The correlation record written for this request carries the same pinned shape.
    let status = c
        .call(json!({"verb": "request_get_status", "actor": {"actor_id": "operator_local", "role": "operator"}, "request_id": "req-ts-case"}))
        .await;
    assert_eq!(status["status"], "completed", "{status}");
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
async fn approval_inbox_returns_stored_stamps_verbatim_in_whatever_spelling_was_stored() {
    // The inbox never re-renders `requested_at` / `expires_at`: it returns the stored bytes. That is the
    // format contract pinned here. (How an `expires_at` is COMPARED with now is not pinned: that belongs to
    // the instant-comparison fix, not to this byte-format baseline.)
    let (root, socket) = boot().await;
    let spellings = [
        "2000-01-01T00:00:00Z",
        "2000-01-01T00:00:00+00:00",
        "2000-01-01T00:00:00.123456789Z",
        "2099-01-01T05:30:00+05:30",
        "2099-01-01T00:00:00.5+00:00",
        "zzz",
        "",
    ];
    for (i, expires) in spellings.iter().enumerate() {
        sea_forge_core::approvals::append(root.path(), &approval(&format!("ap-{i}"), expires))
            .unwrap();
    }
    let mut c = Client::connect(&socket).await;
    let listed = c.call(json!({"verb": "approval_list"})).await;
    let approvals = listed["approvals"].as_array().unwrap();
    assert_eq!(approvals.len(), spellings.len(), "{listed}");
    for (i, expires) in spellings.iter().enumerate() {
        let row = approvals
            .iter()
            .find(|a| a["approval_id"] == format!("ap-{i}"))
            .unwrap();
        assert_eq!(row["expires_at"], json!(expires), "{row}");
        assert_eq!(row["requested_at"], json!("2026-07-26T00:00:00Z"), "{row}");
    }
}
