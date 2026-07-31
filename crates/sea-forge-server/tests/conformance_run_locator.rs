//! One run locator across both layouts (SF-004, K-04, DATA-02).
//!
//! Two run layouts exist in a cell and both are permanent:
//!
//! - `<root>/runs/<id>` — written by the minimum CLI, and the layout
//!   `just proof` pins.
//! - `<root>/cases/<case>/runs/<id>` — written by a case episode
//!   (spec-full.md:659).
//!
//! `run_views` resolved only the first. Every run a case dispatched was
//! therefore invisible to `run_get` and absent from `run_list`, so a Workbench
//! run link opened `not_found` for exactly the runs the server itself had just
//! created. Nothing about the record was wrong — the reader was looking in one
//! of the two places records live.
//!
//! These drive the real socket, and the restart case re-boots a second server
//! against the same root, because resolution that only works while the process
//! that wrote the run is still up is not resolution.

use sea_forge_server::{run, ServerConfig};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, UnixStream};

async fn serve(root: &Path) -> PathBuf {
    // A fresh socket name per boot: a restart in these tests is a second
    // server over the same records, not a reuse of the first one's endpoint.
    let socket = root.join(format!("sfwp-{}.sock", std::process::id()));
    let socket = (0..)
        .map(|n| socket.with_extension(format!("{n}.sock")))
        .find(|candidate| !candidate.exists())
        .unwrap();
    let config = ServerConfig {
        socket_path: socket.clone(),
        root: root.to_path_buf(),
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
    socket
}

struct Client {
    writer: OwnedWriteHalf,
    reader: BufReader<OwnedReadHalf>,
}

impl Client {
    async fn connect(socket: &Path) -> Self {
        let stream = UnixStream::connect(socket).await.unwrap();
        let (reader, writer) = stream.into_split();
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
        let n = tokio::time::timeout(Duration::from_secs(30), self.reader.read_line(&mut line))
            .await
            .expect("read timed out")
            .unwrap();
        assert!(n > 0, "connection closed unexpectedly");
        serde_json::from_str(line.trim()).unwrap()
    }
}

fn write_json(path: &Path, value: &Value) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn write_jsonl(path: &Path, values: &[Value]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let body: String = values.iter().map(|v| format!("{v}\n")).collect();
    std::fs::write(path, body).unwrap();
}

/// One complete, settled run record at `run_dir`. `item_name` is the probe the
/// assertions read back: it only survives if `plan.json` actually deserialized,
/// so a view that renders empty because the fixture failed to parse fails here
/// rather than passing vacuously.
fn seed_run(run_dir: &Path, run_id: &str, case_id: &str, item_name: &str) {
    write_json(
        &run_dir.join("plan.json"),
        &json!({
            "version": "0.2",
            "plan_id": "plan-1",
            "case_id": case_id,
            "run_id": run_id,
            "intent_id": "int-1",
            "items": [{
                "plan_item_id": "item-1",
                "name": item_name,
                "item_kind": "sandboxed_task",
                "settlement_criteria": {"require_exit_zero": true},
            }],
        }),
    );
    write_jsonl(
        &run_dir.join("trace.jsonl"),
        &[json!({
            "version": "0.1",
            "event_id": "evt-1",
            "run_id": run_id,
            "plan_item_id": "item-1",
            "kind": "command_finished",
            "actor_id": "operator_local",
            "timestamp": "2026-01-01T00:00:02Z",
            "payload": {"execution": {"status": "completed", "exit_code": 0}},
        })],
    );
    write_jsonl(
        &run_dir.join("evidence.jsonl"),
        &[json!({
            "version": "0.1",
            "evidence_id": "evi_0001",
            "run_id": run_id,
            "kind": "execution_result",
            "uri": "artifacts/stdout.txt",
            "source_event_id": "evt-1",
        })],
    );
    write_json(
        &run_dir.join("settlement.json"),
        &json!({
            "version": "0.1",
            "settlement_id": "set_probe",
            "run_id": run_id,
            "status": "accepted",
            "basis": ["authority_allow", "exit_zero"],
            "review_required": false,
            "settled_at": "2026-01-01T00:00:04Z",
        }),
    );
}

fn seed_case(root: &Path, case_id: &str, run_ids: &[&str]) {
    write_json(
        &root.join("cases").join(case_id).join("case.json"),
        &json!({
            "version": "0.1",
            "case_id": case_id,
            "intent": {
                "intent_id": "int-1",
                "summary": "seeded case",
                "actor_id": "operator_local",
                "process_id": "test",
                "created_at": "2026-01-01T00:00:00Z",
            },
            "state": "active",
            "plan_ref": "plan-1",
            "run_ids": run_ids,
            "stages": [],
            "close_reason": null,
            "created_at": "2026-01-01T00:00:00Z",
            "closed_at": null,
        }),
    );
}

/// A cell holding one run of each layout.
fn seed_both_layouts(root: &Path) {
    seed_case(root, "case-owned", &["run-cased"]);
    seed_run(
        &root
            .join("cases")
            .join("case-owned")
            .join("runs")
            .join("run-cased"),
        "run-cased",
        "case-owned",
        "Case episode",
    );
    seed_case(root, "case-flat", &["run-flat"]);
    seed_run(
        &root.join("runs").join("run-flat"),
        "run-flat",
        "case-flat",
        "CLI run",
    );
}

#[tokio::test]
async fn both_layouts_resolve_through_one_run_get() {
    let root = tempfile::tempdir().unwrap();
    seed_both_layouts(root.path());
    let socket = serve(root.path()).await;
    let mut client = Client::connect(&socket).await;

    let cased = client
        .call(json!({"verb": "run_get", "run_id": "run-cased"}))
        .await;
    assert!(
        cased.get("error").is_none(),
        "a case-owned run must resolve: {cased}"
    );
    assert_eq!(cased["plan_item_name"], "Case episode");

    let flat = client
        .call(json!({"verb": "run_get", "run_id": "run-flat"}))
        .await;
    assert!(
        flat.get("error").is_none(),
        "a flat CLI run must still resolve: {flat}"
    );
    assert_eq!(flat["plan_item_name"], "CLI run");
}

/// Resolution has to survive the process that wrote the run. Nothing is held
/// in memory — but that is a claim about the implementation, and this is the
/// test that makes it a property.
#[tokio::test]
async fn both_layouts_still_resolve_after_a_restart() {
    let root = tempfile::tempdir().unwrap();
    seed_both_layouts(root.path());

    let first = serve(root.path()).await;
    let mut client = Client::connect(&first).await;
    assert!(client
        .call(json!({"verb": "run_get", "run_id": "run-cased"}))
        .await
        .get("error")
        .is_none());
    drop(client);

    let second = serve(root.path()).await;
    assert_ne!(first, second, "the restart must be a second server");
    let mut client = Client::connect(&second).await;
    for run_id in ["run-cased", "run-flat"] {
        let record = client
            .call(json!({"verb": "run_get", "run_id": run_id}))
            .await;
        assert!(
            record.get("error").is_none(),
            "{run_id} stopped resolving after restart: {record}"
        );
    }
}

#[tokio::test]
async fn run_list_reports_both_layouts_once_each() {
    let root = tempfile::tempdir().unwrap();
    seed_both_layouts(root.path());
    let socket = serve(root.path()).await;
    let mut client = Client::connect(&socket).await;

    let result = client.call(json!({"verb": "run_list"})).await;
    assert!(result.get("error").is_none(), "unexpected error: {result}");
    let mut ids: Vec<&str> = result["runs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["run_id"].as_str().unwrap())
        .collect();
    ids.sort_unstable();
    assert_eq!(ids, ["run-cased", "run-flat"], "runs {result}");
}

/// If the same id somehow exists in both layouts, one of them has to win
/// deterministically, or `run_get` and `run_list` can disagree about which
/// record a link opens. Flat wins: it is the older layout and the one the
/// existing minimum-CLI proofs read.
#[tokio::test]
async fn a_duplicated_id_resolves_to_the_flat_layout() {
    let root = tempfile::tempdir().unwrap();
    seed_case(root.path(), "case-dup", &["run-dup"]);
    seed_run(
        &root
            .path()
            .join("cases")
            .join("case-dup")
            .join("runs")
            .join("run-dup"),
        "run-dup",
        "case-dup",
        "Case copy",
    );
    seed_run(
        &root.path().join("runs").join("run-dup"),
        "run-dup",
        "case-dup",
        "Flat copy",
    );
    let socket = serve(root.path()).await;
    let mut client = Client::connect(&socket).await;

    let record = client
        .call(json!({"verb": "run_get", "run_id": "run-dup"}))
        .await;
    assert_eq!(record["plan_item_name"], "Flat copy");

    let listed = client.call(json!({"verb": "run_list"})).await;
    let runs = listed["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 1, "one id is one run: {listed}");
    assert_eq!(runs[0]["run_id"], "run-dup");
}

/// The case view reads settlements through the same locator. Before it did, a
/// case could not see the settlement of a run it had itself dispatched.
#[tokio::test]
async fn a_case_sees_the_settlement_of_its_own_case_owned_run() {
    let root = tempfile::tempdir().unwrap();
    seed_both_layouts(root.path());
    let socket = serve(root.path()).await;
    let mut client = Client::connect(&socket).await;

    let overview = client
        .call(json!({"verb": "case_get_overview", "case_id": "case-owned"}))
        .await;
    assert!(
        overview.get("error").is_none(),
        "unexpected error: {overview}"
    );
    let settlements = overview["settlements"].as_array().unwrap();
    assert_eq!(settlements.len(), 1, "settlements {overview}");
    assert_eq!(settlements[0]["run_id"], "run-cased");
    assert_eq!(settlements[0]["status"], "accepted");
}

/// A traversal attempt must not become reachable because the locator gained a
/// second place to look.
#[tokio::test]
async fn the_case_owned_probe_does_not_widen_the_traversal_surface() {
    let root = tempfile::tempdir().unwrap();
    seed_both_layouts(root.path());
    std::fs::write(root.path().join("secret.json"), "{}").unwrap();
    let socket = serve(root.path()).await;
    let mut client = Client::connect(&socket).await;

    for probe in ["../../etc", "..", "case-owned/runs/run-cased", "run cased"] {
        let result = client
            .call(json!({"verb": "run_get", "run_id": probe}))
            .await;
        assert_eq!(
            result["error_class"], "not_found",
            "run_id {probe:?} was not refused: {result}"
        );
    }
}
