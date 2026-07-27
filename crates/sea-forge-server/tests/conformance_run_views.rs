//! Run-record conformance tests (Task 8, ADR-003 additive): `run.list`, `run.get`.
//!
//! Fixtures are written as raw JSON rather than constructed from the typed
//! structs on purpose: that exercises the same deserialization path a real
//! kernel-written record takes, so a field this module reads by the wrong name
//! fails here instead of silently projecting an empty view. Several assertions
//! below exist specifically to catch a fixture that failed to parse (a view
//! whose `plan_item_name` is absent means `plan.json` did not deserialize, not
//! that the plan had no name).

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
        let stream = UnixStream::connect(socket).await.unwrap();
        let (reader, writer) = stream.into_split();
        Self {
            writer,
            reader: BufReader::new(reader),
        }
    }

    async fn call(&mut self, value: Value) -> Value {
        let line = format!("{value}\n");
        self.writer.write_all(line.as_bytes()).await.unwrap();
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
    let body: String = values
        .iter()
        .map(|v| format!("{v}\n"))
        .collect::<Vec<_>>()
        .concat();
    std::fs::write(path, body).unwrap();
}

fn trace_event(run_id: &str, event_id: &str, kind: &str, timestamp: &str, payload: Value) -> Value {
    json!({
        "version": "0.1",
        "event_id": event_id,
        "run_id": run_id,
        "plan_item_id": "item-1",
        "kind": kind,
        "actor_id": "operator_local",
        "timestamp": timestamp,
        "payload": payload,
    })
}

/// A run whose command exited 0 and whose settlement nevertheless rejected the
/// work — the exact state epic 12.4 exists to keep representable.
fn seed_rejected_despite_exit_zero(root: &Path, case_id: &str, run_id: &str) {
    let run_dir = root.join("runs").join(run_id);

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
            "run_ids": [run_id],
            "stages": [],
            "close_reason": null,
            "created_at": "2026-01-01T00:00:00Z",
            "closed_at": null,
        }),
    );

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
                "name": "Build the report",
                "item_kind": "sandboxed_task",
                "settlement_criteria": {
                    "require_exit_zero": true,
                    "required_artifacts": ["out/report.md"],
                },
            }],
        }),
    );

    write_jsonl(
        &run_dir.join("trace.jsonl"),
        &[
            trace_event(
                run_id,
                "evt-1",
                "run_started",
                "2026-01-01T00:00:01Z",
                json!({}),
            ),
            trace_event(
                run_id,
                "evt-2",
                "command_finished",
                "2026-01-01T00:00:02Z",
                json!({"execution": {"status": "completed", "exit_code": 0}}),
            ),
            trace_event(
                run_id,
                "evt-3",
                "item_completed",
                "2026-01-01T00:00:03Z",
                json!({}),
            ),
        ],
    );

    write_jsonl(
        &run_dir.join("evidence.jsonl"),
        &[json!({
            "version": "0.1",
            "evidence_id": "ev-1",
            "run_id": run_id,
            "kind": "execution_result",
            "uri": "runs/evidence/stdout.txt",
            "sha256": "abc123",
            "source_event_id": "evt-2",
        })],
    );

    // Exit zero, but the required artifact was missing — so the settlement
    // rejected. The two facts must both survive into the view.
    write_json(
        &run_dir.join("settlement.json"),
        &json!({
            "version": "0.1",
            "settlement_id": "set-1",
            "run_id": run_id,
            "status": "rejected",
            "basis": [
                "authority_allow",
                "exit_zero",
                "required_artifact_missing:out/report.md",
            ],
            "review_required": false,
            "settled_at": "2026-01-01T00:00:04Z",
        }),
    );
}

#[tokio::test]
async fn run_list_is_empty_for_a_fresh_cell_rather_than_an_error() {
    // A cell with no `runs/` directory has *no runs*, which is a fact, not a
    // failure. `unknown != unavailable`.
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;

    let result = client.call(json!({"verb": "run_list"})).await;

    assert!(result.get("error").is_none(), "unexpected error: {result}");
    assert_eq!(result["runs"].as_array().unwrap().len(), 0);
    assert_eq!(result["unreadable"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn a_missing_run_is_a_typed_not_found_never_an_empty_view() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;

    let result = client
        .call(json!({"verb": "run_get", "run_id": "run-does-not-exist"}))
        .await;

    assert_eq!(result["error_class"], "not_found");
}

#[tokio::test]
async fn a_run_id_cannot_traverse_out_of_the_runs_directory() {
    let (root, socket) = boot().await;
    std::fs::write(root.path().join("secret.json"), "{}").unwrap();
    let mut client = Client::connect(&socket).await;

    let result = client
        .call(json!({"verb": "run_get", "run_id": "../../etc"}))
        .await;

    assert_eq!(result["error_class"], "not_found");
}

/// The load-bearing test for epic 12.4: an exit code must never be able to
/// define, or be overwritten by, the governed outcome.
#[tokio::test]
async fn execution_termination_and_settlement_are_reported_as_separate_facts() {
    let (root, socket) = boot().await;
    seed_rejected_despite_exit_zero(root.path(), "case-1", "run-1");
    let mut client = Client::connect(&socket).await;

    let record = client
        .call(json!({"verb": "run_get", "run_id": "run-1"}))
        .await;
    assert!(record.get("error").is_none(), "unexpected error: {record}");

    // Proves `plan.json` actually deserialized — otherwise the criteria
    // assertions below would pass vacuously against an empty list.
    assert_eq!(record["plan_item_name"], "Build the report");

    // Execution completed. Settlement rejected. Both true, neither derived from
    // the other.
    assert_eq!(record["execution"], "completed");
    assert_eq!(record["settlement"], "rejected");
    assert_eq!(record["termination"]["exit_code"], 0);
    assert_eq!(record["termination"]["execution_status"], "completed");
    assert_eq!(record["settlement_record"]["status"], "rejected");
}

/// Epic 12.3: each criterion pairs with the settlement's own basis token, and
/// the join never invents a verdict the settlement did not record.
#[tokio::test]
async fn each_criterion_pairs_with_the_settlements_own_basis_token() {
    let (root, socket) = boot().await;
    seed_rejected_despite_exit_zero(root.path(), "case-1", "run-1");
    let mut client = Client::connect(&socket).await;

    let record = client
        .call(json!({"verb": "run_get", "run_id": "run-1"}))
        .await;
    let criteria = record["criteria"].as_array().unwrap();
    assert_eq!(criteria.len(), 2, "criteria: {criteria:?}");

    assert_eq!(criteria[0]["criterion"], "require_exit_zero");
    assert_eq!(criteria[0]["standing"], "satisfied");
    assert_eq!(criteria[0]["basis"][0], "exit_zero");

    assert_eq!(criteria[1]["criterion"], "required_artifact");
    assert_eq!(criteria[1]["expected"], "out/report.md");
    assert_eq!(criteria[1]["standing"], "unsatisfied");
    assert_eq!(
        criteria[1]["basis"][0],
        "required_artifact_missing:out/report.md"
    );
}

/// An unsettled run must never render as passing. Absence of a verdict is
/// `unavailable`, which is a different claim from "satisfied".
#[tokio::test]
async fn a_criterion_with_no_settlement_record_is_unavailable_not_satisfied() {
    let (root, socket) = boot().await;
    let run_dir = root.path().join("runs").join("run-unsettled");
    write_json(
        &run_dir.join("plan.json"),
        &json!({
            "version": "0.2",
            "plan_id": "plan-2",
            "case_id": "case-2",
            "run_id": "run-unsettled",
            "intent_id": "int-2",
            "items": [{
                "plan_item_id": "item-1",
                "name": "Unsettled work",
                "settlement_criteria": {"require_exit_zero": true},
            }],
        }),
    );
    write_jsonl(
        &run_dir.join("trace.jsonl"),
        &[trace_event(
            "run-unsettled",
            "evt-1",
            "item_activated",
            "2026-01-01T00:00:01Z",
            json!({}),
        )],
    );

    let mut client = Client::connect(&socket).await;
    let record = client
        .call(json!({"verb": "run_get", "run_id": "run-unsettled"}))
        .await;

    assert_eq!(record["settlement"], "unsettled");
    assert_eq!(record["criteria"][0]["standing"], "unavailable");
    assert_eq!(record["criteria"][0]["basis"].as_array().unwrap().len(), 0);
    // No settlement record must not be reported as a rejecting one.
    assert!(record.get("settlement_record").is_none());
}

/// Epic 12.10 / invariant "absence is reported, never inferred": a record that
/// is not on disk is listed as absent rather than rendered as an empty panel.
#[tokio::test]
async fn the_record_inventory_reports_absent_files_as_absent() {
    let (root, socket) = boot().await;
    seed_rejected_despite_exit_zero(root.path(), "case-1", "run-1");
    let mut client = Client::connect(&socket).await;

    let record = client
        .call(json!({"verb": "run_get", "run_id": "run-1"}))
        .await;
    let records = record["records"].as_array().unwrap();

    let presence = |name: &str| -> bool {
        records
            .iter()
            .find(|entry| entry["record"] == name)
            .unwrap_or_else(|| panic!("{name} missing from inventory"))["present"]
            .as_bool()
            .unwrap()
    };

    assert!(presence("plan.json"));
    assert!(presence("trace.jsonl"));
    assert!(presence("settlement.json"));
    // Never written by this fixture — and reported as such rather than omitted.
    assert!(!presence("authority.json"));
    assert!(!presence("declarations.jsonl"));
}

#[tokio::test]
async fn run_list_scoped_to_a_case_returns_only_the_runs_that_case_claims() {
    let (root, socket) = boot().await;
    seed_rejected_despite_exit_zero(root.path(), "case-1", "run-1");
    seed_rejected_despite_exit_zero(root.path(), "case-2", "run-2");
    let mut client = Client::connect(&socket).await;

    let all = client.call(json!({"verb": "run_list"})).await;
    assert_eq!(all["runs"].as_array().unwrap().len(), 2);

    let scoped = client
        .call(json!({"verb": "run_list", "case_id": "case-2"}))
        .await;
    let runs = scoped["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0]["run_id"], "run-2");
    assert_eq!(runs[0]["case_id"], "case-2");
    assert_eq!(runs[0]["evidence_count"], 1);
}

/// `run.get` and `run.list` are inspect methods: they must appear in the
/// negotiated catalog, or a client has no lawful way to discover them.
#[tokio::test]
async fn the_run_methods_are_advertised_in_the_negotiated_catalog() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;

    let hello = client
        .call(json!({"verb": "system_hello", "protocol_version": "1"}))
        .await;
    let methods: Vec<&str> = hello["implemented_methods"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m.as_str().unwrap())
        .collect();

    assert!(methods.contains(&"run.list"), "methods: {methods:?}");
    assert!(methods.contains(&"run.get"), "methods: {methods:?}");
}
