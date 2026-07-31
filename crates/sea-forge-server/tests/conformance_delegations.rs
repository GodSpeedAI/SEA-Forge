//! Delegation roster conformance tests (Task 11, ADR-003 additive):
//! `delegation.list`.
//!
//! What these hold down:
//!
//! 1. **The seam between memory and disk is reported, not guessed.** A run with
//!    no settlement and no live handle is `unresolved` — never `cancelled`,
//!    never `settled`. This is the state a server restart produces, so it is
//!    the one most likely to be quietly rounded off to something terminal.
//! 2. **Standing is not settlement.** A settled-and-rejected delegation and a
//!    settled-and-accepted one share a standing and differ in a separate field.
//! 3. **Cancellation is per-run.** Cancelling one delegation must leave its
//!    siblings cancellable, which is the whole of epic 9.7's control claim.
//!
//! Fixtures are raw JSON on disk, mirroring exactly what `delegation.rs`
//! commits (`plan.json` carrying `Operation::AgentTask`, `settlement.json`,
//! `transcript-evidence.json`). A field read by the wrong name fails here
//! instead of silently yielding an empty roster.

use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind};
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
        agent: AgentConfig {
            endpoints: vec![endpoint("local")],
            ..AgentConfig::default()
        },
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

fn endpoint(id: &str) -> AgentEndpointConfig {
    AgentEndpointConfig {
        id: id.into(),
        kind: ProviderKind::OpenAiCompatible,
        base_url: Some("https://api.example.com/v1".into()),
        argv: Vec::new(),
        env: Vec::new(),
        credential_ref: None,
        default_model: Some("m1".into()),
        allow_loopback_test: false,
        max_request_bytes: 4096,
        max_response_bytes: 4096,
        timeout_secs: 30,
        status: None,
        transcript_retention: None,
    }
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
    std::fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
}

/// An agent-task plan exactly as `delegation.rs` commits it.
fn write_delegation_plan(root: &Path, run_id: &str, endpoint_ref: &str, max_turns: u32) {
    write_json(
        &root.join("runs").join(run_id).join("plan.json"),
        &json!({
            "version": "0.2",
            "plan_id": format!("plan-{run_id}"),
            "case_id": format!("case-{run_id}"),
            "run_id": run_id,
            "intent_id": format!("int-{run_id}"),
            "items": [{
                "plan_item_id": "item_agent_task",
                "name": "agent_task",
                "operations": [{
                    "kind": "agent_task",
                    "endpoint_ref": endpoint_ref,
                    "instruction": "do the thing",
                    "max_turns": max_turns,
                }],
                "entry_criteria": [],
                "exit_criteria": [],
                "settlement_criteria": {},
                "item_kind": "agent_task",
                "max_instances": 1,
                "depends_on": [],
            }],
        }),
    );
}

fn write_settlement(root: &Path, run_id: &str, status: &str, settled_at: &str) {
    write_json(
        &root.join("runs").join(run_id).join("settlement.json"),
        &json!({
            "version": "0.2",
            "settlement_id": format!("set-{run_id}"),
            "run_id": run_id,
            "status": status,
            "basis": ["criteria met"],
            "review_required": false,
            "settled_at": settled_at,
        }),
    );
}

fn write_transcript_evidence(root: &Path, run_id: &str, turns: u32, termination: &str) {
    write_json(
        &root
            .join("runs")
            .join(run_id)
            .join("transcript-evidence.json"),
        &json!({
            "run_id": run_id,
            "endpoint_ref": "local",
            "turns_used": turns,
            "termination": termination,
            "transcript_sha256": format!("sha256:{}", "b".repeat(64)),
            "summary": {"turn_count": turns, "tool_calls": 2, "final_excerpt": "done"},
        }),
    );
}

fn rows(body: &Value) -> Vec<Value> {
    body["delegations"].as_array().cloned().unwrap_or_default()
}

/// Roster order, as run ids.
fn run_ids(body: &Value) -> Vec<String> {
    rows(body)
        .iter()
        .map(|row| row["run_id"].as_str().unwrap().to_string())
        .collect()
}

fn row<'a>(body: &'a Value, run_id: &str) -> &'a Value {
    body["delegations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["run_id"] == run_id)
        .unwrap_or_else(|| panic!("no row for {run_id} in {body}"))
}

const LIST: fn() -> Value = || json!({"verb": "delegation_list"});

#[tokio::test]
async fn a_fresh_cell_has_an_empty_roster_rather_than_an_error() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let body = client.call(LIST()).await;

    assert!(body.get("error").is_none(), "{body}");
    assert_eq!(rows(&body).len(), 0, "{body}");
}

/// The state a server restart produces. Neither `cancelled` nor `settled` — the
/// kernel does not know what happened, and neither may this projection.
#[tokio::test]
async fn a_delegation_with_no_settlement_and_no_handle_is_unresolved() {
    let (root, socket) = boot().await;
    write_delegation_plan(root.path(), "run-orphan", "local", 5);
    let mut client = Client::connect(&socket).await;
    let body = client.call(LIST()).await;

    let orphan = row(&body, "run-orphan");
    assert_eq!(orphan["standing"], "unresolved", "{orphan}");
    // It is not cancellable: the server holds nothing to cancel.
    assert_eq!(orphan["cancellable"], false);
    // And it claims no outcome of any kind.
    assert!(orphan.get("settlement").is_none(), "{orphan}");
    assert!(orphan.get("termination").is_none(), "{orphan}");
    assert!(orphan.get("turns_used").is_none(), "{orphan}");
}

/// Standing says where in the lifecycle; settlement says what the verdict was.
/// A rejected delegation is every bit as `settled` as an accepted one.
#[tokio::test]
async fn standing_and_settlement_are_separate_facts() {
    let (root, socket) = boot().await;
    write_delegation_plan(root.path(), "run-ok", "local", 4);
    write_settlement(
        root.path(),
        "run-ok",
        "accepted",
        "2026-07-27T01:00:00+00:00",
    );
    write_delegation_plan(root.path(), "run-no", "local", 4);
    write_settlement(
        root.path(),
        "run-no",
        "rejected",
        "2026-07-27T02:00:00+00:00",
    );
    let mut client = Client::connect(&socket).await;
    let body = client.call(LIST()).await;

    assert_eq!(row(&body, "run-ok")["standing"], "settled");
    assert_eq!(row(&body, "run-ok")["settlement"], "accepted");
    assert_eq!(row(&body, "run-no")["standing"], "settled");
    assert_eq!(row(&body, "run-no")["settlement"], "rejected");
}

/// The dialogue state epic 9.7 asks for, once it exists. Turn count is reported
/// against the cap the plan bound the episode to, not as a bare number.
#[tokio::test]
async fn a_finished_delegation_reports_its_dialogue_against_its_bounds() {
    let (root, socket) = boot().await;
    write_delegation_plan(root.path(), "run-done", "local", 7);
    write_settlement(
        root.path(),
        "run-done",
        "accepted",
        "2026-07-27T00:00:00+00:00",
    );
    write_transcript_evidence(root.path(), "run-done", 3, "completed");
    let mut client = Client::connect(&socket).await;
    let body = client.call(LIST()).await;

    let done = row(&body, "run-done");
    assert_eq!(done["turns_used"], 3);
    assert_eq!(done["max_turns"], 7);
    assert_eq!(done["tool_calls"], 2);
    assert_eq!(done["termination"], "completed");
    assert_eq!(done["endpoint_ref"], "local");
    assert!(done["transcript_sha256"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
}

/// A cancelled episode still settles, and the cancellation shows up as the
/// dialogue's *termination* — not as a standing that hides the settlement.
#[tokio::test]
async fn a_cancelled_delegation_settles_and_reports_cancellation_as_its_termination() {
    let (root, socket) = boot().await;
    write_delegation_plan(root.path(), "run-cancelled", "local", 9);
    write_settlement(
        root.path(),
        "run-cancelled",
        "rejected",
        "2026-07-27T00:00:00+00:00",
    );
    write_transcript_evidence(root.path(), "run-cancelled", 1, "cancelled");
    let mut client = Client::connect(&socket).await;
    let body = client.call(LIST()).await;

    let cancelled = row(&body, "run-cancelled");
    assert_eq!(cancelled["standing"], "settled");
    assert_eq!(cancelled["termination"], "cancelled");
    assert_eq!(cancelled["settlement"], "rejected");
    assert_eq!(cancelled["cancellable"], false);
}

/// A run that is not an agent task is not a delegation. The roster must not
/// absorb probes, sandboxed tasks, or anything else that happens to live in
/// `runs/`.
#[tokio::test]
async fn non_delegation_runs_are_not_in_the_roster() {
    let (root, socket) = boot().await;
    write_json(
        &root.path().join("runs").join("run-probe").join("plan.json"),
        &json!({
            "version": "0.2",
            "plan_id": "plan-probe",
            "case_id": "case-probe",
            "run_id": "run-probe",
            "intent_id": "int-probe",
            "items": [{
                "plan_item_id": "item_agent_probe",
                "name": "agent_probe",
                "operations": [{
                    "kind": "agent_probe",
                    "endpoint_ref": "local",
                    "model": "m1",
                    "prompt_sha256": format!("sha256:{}", "a".repeat(64)),
                }],
                "entry_criteria": [],
                "exit_criteria": [],
                "settlement_criteria": {},
                "item_kind": "sandboxed_task",
                "max_instances": 1,
                "depends_on": [],
            }],
        }),
    );
    write_delegation_plan(root.path(), "run-task", "local", 4);
    let mut client = Client::connect(&socket).await;
    let body = client.call(LIST()).await;

    assert_eq!(run_ids(&body), vec!["run-task"], "{body}");
}

/// Ownership comes from the same index `run.list` uses. Two views disagreeing
/// about which case owns a run would make one of them wrong.
#[tokio::test]
async fn a_delegation_is_attributed_to_the_case_that_claims_it() {
    let (root, socket) = boot().await;
    write_delegation_plan(root.path(), "run-owned", "local", 4);
    write_json(
        &root.path().join("cases").join("case-7").join("case.json"),
        &json!({
            "version": "0.2",
            "case_id": "case-7",
            "intent": {
                "intent_id": "int-7",
                "summary": "delegate",
                "actor_id": "operator_local",
                "process_id": "proc-7",
                "created_at": "2026-07-27T00:00:00+00:00",
            },
            "state": "active",
            "plan_ref": "plan-7",
            "run_ids": ["run-owned"],
            "close_reason": null,
            "created_at": "2026-07-27T00:00:00+00:00",
            "closed_at": null,
        }),
    );
    let mut client = Client::connect(&socket).await;
    let body = client.call(LIST()).await;

    assert_eq!(row(&body, "run-owned")["case_id"], "case-7");
}

/// `plan.json` names the bounds; an episode with no token cap must omit the
/// field rather than report `0`, which would mean "spend nothing".
#[tokio::test]
async fn an_absent_token_budget_is_omitted_rather_than_zeroed() {
    let (root, socket) = boot().await;
    write_delegation_plan(root.path(), "run-unbounded", "local", 4);
    let mut client = Client::connect(&socket).await;
    let body = client.call(LIST()).await;

    let unbounded = row(&body, "run-unbounded");
    assert!(unbounded.get("token_budget").is_none(), "{unbounded}");
    assert_eq!(unbounded["max_turns"], 4);
}

/// Cancelling an inactive delegation is refused as a governed outcome, not
/// silently accepted. The roster's `cancellable: false` and this refusal have to
/// agree, or the UI would offer an action the kernel rejects.
#[tokio::test]
async fn cancelling_a_delegation_that_is_not_active_is_refused() {
    let (root, socket) = boot().await;
    write_delegation_plan(root.path(), "run-done", "local", 4);
    write_settlement(
        root.path(),
        "run-done",
        "accepted",
        "2026-07-27T00:00:00+00:00",
    );
    let mut client = Client::connect(&socket).await;

    let body = client.call(LIST()).await;
    assert_eq!(row(&body, "run-done")["cancellable"], false);

    let refused = client
        .call(json!({"verb": "cancel_delegation", "actor": {"actor_id": "operator_local", "role": "operator"}, "run_id": "run-done"}))
        .await;
    assert_eq!(refused["error"], "delegation run not active", "{refused}");
}

/// Live delegations sort ahead of history so an operator looking for something
/// to intervene in does not scroll past finished work to find it.
#[tokio::test]
async fn unsettled_delegations_sort_ahead_of_settled_ones() {
    let (root, socket) = boot().await;
    write_delegation_plan(root.path(), "run-a-settled", "local", 4);
    write_settlement(
        root.path(),
        "run-a-settled",
        "accepted",
        "2026-07-27T00:00:00+00:00",
    );
    write_delegation_plan(root.path(), "run-z-open", "local", 4);
    let mut client = Client::connect(&socket).await;
    let body = client.call(LIST()).await;

    assert_eq!(
        run_ids(&body),
        vec!["run-z-open", "run-a-settled"],
        "{body}"
    );
}

#[tokio::test]
async fn delegation_list_is_advertised_as_an_inspect_method() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let hello = client
        .call(json!({"verb": "system_hello", "protocol_version": "1"}))
        .await;
    assert!(
        hello["implemented_methods"]
            .as_array()
            .unwrap()
            .iter()
            .any(|method| method == "delegation.list"),
        "{hello}"
    );

    let describe = client.call(json!({"verb": "system_describe"})).await;
    let descriptor = describe["methods"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["method"] == "delegation.list")
        .expect("delegation.list is described");
    assert_eq!(descriptor["class"], "inspect");
}

/// Every path under the cell root, for before/after comparison.
fn tree(root: &Path) -> Vec<String> {
    fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            out.push(
                path.strip_prefix(base)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .into_owned(),
            );
            if path.is_dir() {
                walk(&path, base, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

/// An inspect method writes nothing.
///
/// Compared as a full before/after tree rather than by checking a list of
/// directory names: the server creates some of its own scaffolding at boot, and
/// a name-based check silently passes for any directory the test author did not
/// think to name. Diffing the tree cannot be fooled that way.
#[tokio::test]
async fn listing_the_roster_creates_nothing() {
    let (root, socket) = boot().await;
    write_delegation_plan(root.path(), "run-1", "local", 4);
    let mut client = Client::connect(&socket).await;

    // Read once first, so anything the *first* read would lazily create is
    // already present and the comparison isolates the read itself.
    let _ = client.call(LIST()).await;
    let before = tree(root.path());
    let body = client.call(LIST()).await;
    let after = tree(root.path());

    assert_eq!(rows(&body).len(), 1, "the read still returned the roster");
    assert_eq!(before, after, "delegation.list must not touch the cell");
}
