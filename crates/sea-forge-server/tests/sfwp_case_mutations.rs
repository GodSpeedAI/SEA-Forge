//! SFWP case mutation and execution verbs over a real socket (T04, P2).
//!
//! Every test here drives the real server over its Unix socket on a temp
//! cell and asserts the *durable* effect: the exact `TraceKind` deltas in
//! `case-events.jsonl` (and the run `trace.jsonl` where the episode writes
//! its own events), plus ledger/plan/approval effects where applicable.
//! Refusals assert the absence of the write, not just the error text.
//!
//! The teeth named in the plan live here too:
//! - a `case.add_item` that would create a cycle is refused with `plan.json`
//!   byte-identical and no `PlanMutated`;
//! - the proposer of a discretionary item cannot resolve the approval gating
//!   it (`sod_violation`, `approvals.jsonl` unchanged);
//! - a subscriber killed mid-stream and resumed from its last cursor sees no
//!   gaps and no duplicates, with cursors monotonic.

use sea_forge_server::identity::{IdentityBinding, IdentityBindings};
use sea_forge_server::{run, ServerConfig};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, UnixStream};

// ---------------------------------------------------------------------------
// Harness (mirrors conformance_approvals / conformance_identity)
// ---------------------------------------------------------------------------

fn boot_config(root: &Path, identity: IdentityBindings) -> ServerConfig {
    // Short, because a deep temp path overflows `sun_path` (cell contract).
    let socket = std::env::temp_dir().join(format!(
        "sf-t04b-{}-{}.sock",
        std::process::id(),
        root.file_name().unwrap().to_string_lossy()
    ));
    ServerConfig {
        socket_path: socket,
        root: root.to_path_buf(),
        identity,
        ..ServerConfig::default()
    }
}

async fn boot(identity: IdentityBindings) -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let config = boot_config(root.path(), identity);
    let socket = config.socket_path.clone();
    tokio::spawn(async move {
        let _ = run(config).await;
    });
    for _ in 0..500 {
        if socket.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(socket.exists(), "server socket never appeared");
    (root, socket)
}

fn local_operator() -> IdentityBindings {
    IdentityBindings::local_operator("operator_local")
}

/// Two actors on this machine's uid: one proposer, one executor. The kernel
/// fills in `SO_PEERCRED`, so both are legitimately claimable by this test —
/// separation of duty must hold between the *actors*, not the uid.
fn two_actors() -> IdentityBindings {
    let uid = sea_forge_server::identity::current_uid().expect("uid must be readable");
    IdentityBindings {
        bindings: ["operator_a", "operator_b"]
            .into_iter()
            .map(|actor_id| IdentityBinding {
                uid,
                actor_id: actor_id.into(),
                roles: vec![sea_forge_core::types::ActorRole::Operator],
            })
            .collect(),
    }
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

    /// `call`, but tolerating (and returning) unsolicited event frames that
    /// arrive before the response — `events.subscribe` writes its replay
    /// burst onto the connection before the ack, so a resubscribe response
    /// is never the first line after a non-empty replay.
    async fn call_among_events(&mut self, value: Value) -> (Value, Vec<Value>) {
        let line = format!("{value}\n");
        self.writer.write_all(line.as_bytes()).await.unwrap();
        self.writer.flush().await.unwrap();
        let mut events = Vec::new();
        loop {
            let mut line = String::new();
            let n = tokio::time::timeout(Duration::from_secs(30), self.reader.read_line(&mut line))
                .await
                .expect("read timed out")
                .unwrap();
            assert!(n > 0, "connection closed unexpectedly");
            let parsed: Value = serde_json::from_str(line.trim()).unwrap();
            if parsed["type"] == "event" {
                events.push(parsed["event"].clone());
                continue;
            }
            return (parsed, events);
        }
    }

    /// Read the next unsolicited `{"type":"event",...}` line, polling until
    /// the deadline. Frames are published concurrently with responses, so a
    /// subscriber must tolerate waiting for them. A quiet read window sends
    /// a cheap inspect ping: the connection's request-line timeout would
    /// otherwise close a listener that only ever listens.
    async fn next_event(&mut self, want_kind: &str) -> Value {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        loop {
            let mut line = String::new();
            let read = tokio::time::timeout(
                Duration::from_millis(1500),
                self.reader.read_line(&mut line),
            )
            .await;
            let n = match read {
                Ok(Ok(n)) => n,
                Ok(Err(error)) => panic!("read error while waiting for {want_kind}: {error}"),
                Err(_) => {
                    assert!(
                        tokio::time::Instant::now() < deadline,
                        "timed out waiting for event {want_kind}"
                    );
                    self.writer
                        .write_all(b"{\"verb\":\"system_describe\"}\n")
                        .await
                        .unwrap();
                    self.writer.flush().await.unwrap();
                    continue;
                }
            };
            assert!(n > 0, "connection closed while waiting for {want_kind}");
            let value: Value = serde_json::from_str(line.trim()).unwrap();
            if value["type"] == "event" && value["event"]["kind"] == want_kind {
                return value["event"].clone();
            }
        }
    }
}

/// The policy spelling the conformance suites use: a minimal bundle allowing
/// exactly the reserved actions and command verdicts under test.
fn write_policy(root: &Path, name: &str, rules: &str) -> String {
    let path = root.join(name);
    fs::write(&path, format!("version: \"0.1\"\nrules:\n{rules}")).unwrap();
    name.to_string()
}

const ALLOW_RULES: &str = "  - name: allow-command\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n";
const ESCALATE_RULES: &str = "  - name: escalate-command\n    verdict: escalate\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n";
const REOPEN_RULE: &str = "  - name: allow-reopen\n    verdict: allow\n    actor_role: operator\n    operation_kind: case_reopen\n";
const ADD_ITEM_RULE: &str = "  - name: allow-propose\n    verdict: allow\n    actor_role: operator\n    operation_kind: discretionary_task_add\n";
const TERMINATE_RULE: &str = "  - name: allow-terminate\n    verdict: allow\n    actor_role: operator\n    operation_kind: case_terminate\n";
const HUMAN_TASK_RULE: &str = "  - name: allow-human-task\n    verdict: allow\n    actor_role: operator\n    operation_kind: human_task_completion\n";

fn full_policy(root: &Path) -> String {
    write_policy(
        root,
        "policy.yaml",
        &format!("{ALLOW_RULES}{REOPEN_RULE}{ADD_ITEM_RULE}{TERMINATE_RULE}{HUMAN_TASK_RULE}"),
    )
}

/// An absolute path named `sea-forge` that resolves to *this* test binary —
/// the intersection of the two argv0 constraints (policy match on the file
/// name, `untrusted_executable` on the canonical path) that makes a real
/// sandboxed episode executable in a test.
fn trusted_self_executable(dir: &Path) -> PathBuf {
    let link = dir.join("sea-forge");
    // Idempotent: several fixtures in one test ask for the same link.
    let _ = std::fs::remove_file(&link);
    std::os::unix::fs::symlink(std::env::current_exe().expect("current_exe"), &link)
        .expect("link the test binary as sea-forge");
    link
}

fn actor_block(actor_id: &str) -> Value {
    json!({"actor_id": actor_id, "role": "operator"})
}

/// Arguments that make the libtest harness exit 0 without running anything.
const EXIT_ZERO: &[&str] = &["--exact", "__no_test_matches_this_name__"];

fn execute_operation(dir: &Path) -> Value {
    let mut argv = vec![trusted_self_executable(dir).to_string_lossy().into_owned()];
    argv.extend(EXIT_ZERO.iter().map(|arg| (*arg).to_owned()));
    json!({"kind": "execute_command", "argv": argv, "cwd": "."})
}

/// A plan whose single SandboxedTask carries `manual_activation`: submit
/// enables it and parks, which is exactly the state `case.advance` and
/// `item.execute` exist to drive (D-3).
fn manual_sandbox_plan(dir: &Path) -> Value {
    json!({
        "version": "0.2",
        "plan_id": "plan_t04b",
        "case_id": "case_placeholder",
        "run_id": "run_placeholder",
        "intent_id": "int_t04b",
        "items": [{
            "plan_item_id": "task",
            "name": "sandboxed",
            "operations": [execute_operation(dir)],
            "entry_criteria": [], "exit_criteria": [],
            "settlement_criteria": {"required_artifacts": [], "required_declarations": []},
            "item_kind": "sandboxed_task",
            "markers": {"required": true, "manual_activation": true},
            "max_instances": 1,
            "depends_on": []
        }],
        "template_ref": null,
        "job_contract_ref": null
    })
}

/// A plan with one human task: submit parks it with `ItemActivated
/// {"human_task": true}` awaiting `human_task.complete`.
fn human_task_plan() -> Value {
    json!({
        "version": "0.2",
        "plan_id": "plan_t04b_judge",
        "case_id": "case_placeholder",
        "run_id": "run_placeholder",
        "intent_id": "int_t04b_judge",
        "items": [{
            "plan_item_id": "judge",
            "name": "sign off",
            "operations": [],
            "entry_criteria": [], "exit_criteria": [],
            "settlement_criteria": {"required_artifacts": [], "required_declarations": []},
            "item_kind": "human_task",
            "markers": {"required": true},
            "max_instances": 1,
            "depends_on": []
        }],
        "template_ref": null,
        "job_contract_ref": null
    })
}

/// Submit a plan over the socket and return the parked case id.
async fn submit_plan(client: &mut Client, root: &Path, plan: Value, policy: &str) -> String {
    let plan_path = root.join("plan-t04b.json");
    fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let response = client
        .call(json!({
            "verb": "submit",
            "plan": "plan-t04b.json",
            "policy": policy,
            "entity": "operator_local",
            "process": "test",
            "timeout": 60,
            "request_id": format!("req-submit-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;
    response["case_id"]
        .as_str()
        .unwrap_or_else(|| panic!("submit did not return a case_id: {response}"))
        .to_owned()
}

fn line_nonce() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

fn case_events(root: &Path, case_id: &str) -> Vec<Value> {
    read_jsonl(&root.join("cases").join(case_id).join("case-events.jsonl"))
}

fn read_jsonl(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn kinds_of(events: &[Value]) -> Vec<String> {
    events
        .iter()
        .map(|event| event["kind"].as_str().unwrap_or_default().to_owned())
        .collect()
}

/// The kinds of one run directory's `trace.jsonl` (the newest run).
fn latest_run_trace_kinds(root: &Path, case_id: &str) -> Vec<String> {
    let latest = latest_run_dir(root, case_id).expect("at least one run directory");
    kinds_of(&read_jsonl(&latest.join("trace.jsonl")))
}

/// One JSON record from the newest run directory, for assertion context.
fn latest_run_json(root: &Path, case_id: &str, file: &str) -> String {
    match latest_run_dir(root, case_id) {
        Some(dir) => fs::read_to_string(dir.join(file)).unwrap_or_else(|_| "<missing>".into()),
        None => "<no run dir>".into(),
    }
}

fn latest_run_dir(root: &Path, case_id: &str) -> Option<PathBuf> {
    let runs = root.join("cases").join(case_id).join("runs");
    let mut dirs: Vec<PathBuf> = fs::read_dir(&runs)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .collect();
    dirs.sort();
    dirs.pop()
}

fn case_state(root: &Path, case_id: &str) -> String {
    let case: Value = serde_json::from_slice(
        &fs::read(root.join("cases").join(case_id).join("case.json")).unwrap(),
    )
    .unwrap();
    case["state"].as_str().unwrap().to_owned()
}

// ---------------------------------------------------------------------------
// case.add_item
// ---------------------------------------------------------------------------

/// Happy path: exactly one `PlanMutated`, with the verified actor as the
/// proposer of record, and the item durably in `plan.json`.
#[tokio::test]
async fn case_add_item_appends_exactly_one_plan_mutated_with_the_verified_proposer() {
    let (root, socket) = boot(local_operator()).await;
    let policy = full_policy(root.path());
    let mut client = Client::connect(&socket).await;
    let case_id = submit_plan(
        client_ref(&mut client),
        root.path(),
        human_task_plan(),
        &policy,
    )
    .await;

    let response = client
        .call(json!({
            "verb": "case_add_item",
            "case_id": case_id,
            "item": {
                "plan_item_id": "extra",
                "name": "discretionary",
                "operations": [],
                "entry_criteria": [], "exit_criteria": [],
                "settlement_criteria": {"required_artifacts": [], "required_declarations": []},
                "item_kind": "human_task",
                "markers": {"required": false},
                "max_instances": 1,
                "depends_on": []
            },
            "policy": policy,
            "request_id": format!("req-add-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;

    assert_eq!(response["ok"], true, "{response}");
    assert_eq!(response["proposed_by"], "operator_local", "{response}");

    // The durable delta: one PlanMutated, nothing else.
    let events = case_events(root.path(), &case_id);
    let delta = &events[events.len() - 1..];
    assert_eq!(kinds_of(delta), vec!["plan_mutated"], "delta {delta:?}");
    assert_eq!(delta[0]["plan_item_id"], "extra");
    assert_eq!(delta[0]["actor_id"], "operator_local");

    // The plan carries the item with the proposer binding.
    let plan: Value = serde_json::from_slice(
        &fs::read(root.path().join("cases").join(&case_id).join("plan.json")).unwrap(),
    )
    .unwrap();
    let item = plan["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["plan_item_id"] == "extra")
        .expect("proposed item is in the durable plan");
    assert_eq!(item["proposed_by"], "operator_local");
}

/// Tooth: a proposal that would close a sentry cycle is refused with
/// `plan.json` byte-identical and no `PlanMutated` anywhere.
#[tokio::test]
async fn case_add_item_cycle_refusal_leaves_the_plan_byte_identical() {
    let (root, socket) = boot(local_operator()).await;
    let policy = full_policy(root.path());
    let mut client = Client::connect(&socket).await;
    let case_id = submit_plan(
        client_ref(&mut client),
        root.path(),
        human_task_plan(),
        &policy,
    )
    .await;

    let plan_path = root.path().join("cases").join(&case_id).join("plan.json");
    let before = fs::read(&plan_path).unwrap();
    let events_before = case_events(root.path(), &case_id).len();

    // A self-dependency closes a cycle in the sentry graph.
    let response = client
        .call(json!({
            "verb": "case_add_item",
            "case_id": case_id,
            "item": {
                "plan_item_id": "cyclic",
                "name": "cyclic",
                "operations": [],
                "entry_criteria": [], "exit_criteria": [],
                "settlement_criteria": {"required_artifacts": [], "required_declarations": []},
                "item_kind": "human_task",
                "markers": {"required": false},
                "max_instances": 1,
                "depends_on": ["cyclic"]
            },
            "policy": policy,
            "request_id": format!("req-add-cycle-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;

    assert_eq!(response["error_class"], "plan_cycle_error", "{response}");
    assert_eq!(fs::read(&plan_path).unwrap(), before, "plan.json changed");
    let events_after = case_events(root.path(), &case_id);
    assert_eq!(events_after.len(), events_before, "an event was appended");
    assert!(
        !kinds_of(&events_after).contains(&"plan_mutated".to_owned()),
        "a refused proposal emitted PlanMutated"
    );
}

// ---------------------------------------------------------------------------
// case.reopen / case.terminate
// ---------------------------------------------------------------------------

/// Reopen after a real closure appends exactly one `CaseReopened` and
/// reactivates the case; reopening an active case is refused with no write.
#[tokio::test]
async fn case_reopen_appends_case_reopened_only_after_closure() {
    let (root, socket) = boot(local_operator()).await;
    let policy = full_policy(root.path());
    let mut client = Client::connect(&socket).await;
    let case_id = submit_plan(
        client_ref(&mut client),
        root.path(),
        human_task_plan(),
        &policy,
    )
    .await;

    // Close the case for real: complete its human task.
    let complete = client
        .call(json!({
            "verb": "human_task_complete",
            "case_id": case_id,
            "item_id": "judge",
            "justification": "reviewed and accepted",
            "policy": policy,
            "request_id": format!("req-htc-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;
    assert_eq!(complete["ok"], true, "{complete}");
    assert_eq!(case_state(root.path(), &case_id), "completed");

    // An active... no: a *completed* case reopens. An attempt before closure
    // is exercised below on a second case.
    let reopen = client
        .call(json!({
            "verb": "case_reopen",
            "case_id": case_id,
            "policy": policy,
            "request_id": format!("req-reopen-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;
    assert_eq!(reopen["ok"], true, "{reopen}");
    assert_eq!(reopen["case_state"], "active");

    let events = case_events(root.path(), &case_id);
    assert_eq!(
        kinds_of(&events),
        vec![
            "case_created",
            "item_activated",
            "human_task_completed",
            "case_closed",
            "case_reopened"
        ],
        "full case history must be exactly the documented kinds"
    );
    assert_eq!(case_state(root.path(), &case_id), "active");

    // Reopening an already-active case is refused with no new event.
    let refused = client
        .call(json!({
            "verb": "case_reopen",
            "case_id": case_id,
            "policy": policy,
            "request_id": format!("req-reopen-2-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;
    assert!(
        refused["error"]
            .as_str()
            .unwrap_or_default()
            .contains("only closed cases can be reopened"),
        "{refused}"
    );
    assert_eq!(case_events(root.path(), &case_id).len(), events.len());
}

/// Terminate on an active case appends exactly one `CaseTerminated`, marks
/// the case Terminated with the operator close reason, and refuses a
/// completed case.
#[tokio::test]
async fn case_terminate_appends_case_terminated_on_an_active_case() {
    let (root, socket) = boot(local_operator()).await;
    let policy = full_policy(root.path());
    let mut client = Client::connect(&socket).await;
    let case_id = submit_plan(
        client_ref(&mut client),
        root.path(),
        manual_sandbox_plan(root.path()),
        &policy,
    )
    .await;
    assert_eq!(case_state(root.path(), &case_id), "active");

    let events_before = case_events(root.path(), &case_id).len();
    let response = client
        .call(json!({
            "verb": "case_terminate",
            "case_id": case_id,
            "reason": "superseded by case-2",
            "policy": policy,
            "request_id": format!("req-terminate-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;
    assert_eq!(response["ok"], true, "{response}");
    assert_eq!(response["case_state"], "terminated");

    let events = case_events(root.path(), &case_id);
    assert_eq!(
        events.len(),
        events_before + 1,
        "exactly one event appended"
    );
    assert_eq!(
        kinds_of(&events)[events.len() - 1..],
        vec!["case_terminated"]
    );
    assert_eq!(case_state(root.path(), &case_id), "terminated");

    let case: Value = serde_json::from_slice(
        &fs::read(root.path().join("cases").join(&case_id).join("case.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        case["close_reason"],
        "operator_terminated:superseded by case-2"
    );

    // A terminated case is not terminable again.
    let refused = client
        .call(json!({
            "verb": "case_terminate",
            "case_id": case_id,
            "reason": "again",
            "policy": policy,
            "request_id": format!("req-terminate-2-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;
    assert!(
        refused["error"]
            .as_str()
            .unwrap_or_default()
            .contains("only active cases can be terminated"),
        "{refused}"
    );
    assert_eq!(case_events(root.path(), &case_id).len(), events.len());
}

// ---------------------------------------------------------------------------
// case.advance / item.execute (a real sandboxed episode)
// ---------------------------------------------------------------------------

/// The parked manual-activation item runs for real: the case delta is
/// exactly the engine's documented sequence and the run trace carries the
/// episode's own events, including the captured artifacts.
#[tokio::test]
async fn case_advance_runs_a_parked_sandboxed_item_end_to_end() {
    let (root, socket) = boot(local_operator()).await;
    let policy = full_policy(root.path());
    let mut client = Client::connect(&socket).await;
    let case_id = submit_plan(
        client_ref(&mut client),
        root.path(),
        manual_sandbox_plan(root.path()),
        &policy,
    )
    .await;

    // Submit parked the case at the enabled manual item.
    assert_eq!(
        kinds_of(&case_events(root.path(), &case_id)),
        vec!["case_created", "item_enabled"]
    );

    let response = client
        .call(json!({
            "verb": "case_advance",
            "case_id": case_id,
            "policy": policy,
            "timeout": 60,
            "request_id": format!("req-advance-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;
    assert_eq!(response["ok"], true, "{response}");
    assert_eq!(
        response["state"],
        "completed",
        "advance response {response}; latest settlement {}",
        latest_run_json(root.path(), &case_id, "settlement.json")
    );
    assert_eq!(response["episodes"].as_array().unwrap().len(), 1);
    assert_eq!(response["episodes"][0]["item_id"], "task");
    assert_eq!(
        response["episodes"][0]["settlement_status"],
        "accepted",
        "settlement {}",
        latest_run_json(root.path(), &case_id, "settlement.json")
    );

    // The case-events delta: activate → settle → complete (+ milestone) →
    // close. SettlementRecorded and MilestoneAchieved are engine semantics
    // (`apply_episode_completion`), not inventions of this verb.
    assert_eq!(
        kinds_of(&case_events(root.path(), &case_id)),
        vec![
            "case_created",
            "item_enabled",
            "item_activated",
            "settlement_recorded",
            "item_completed",
            "milestone_achieved",
            "case_closed"
        ]
    );
    assert_eq!(case_state(root.path(), &case_id), "completed");

    // The run's own trace: authority → workspace → command → artifacts.
    assert_eq!(
        latest_run_trace_kinds(root.path(), &case_id),
        vec![
            "authority_evaluated",
            "workspace_created",
            "command_started",
            "command_finished",
            "artifact_captured",
            "artifact_captured"
        ]
    );
}

/// `item.execute` scopes the same engine to one item, and refuses an item
/// that is not ready (unknown, or already terminal) with no writes.
#[tokio::test]
async fn item_execute_scopes_to_one_item_and_refuses_non_ready_targets() {
    let (root, socket) = boot(local_operator()).await;
    let policy = full_policy(root.path());
    let mut client = Client::connect(&socket).await;
    let case_id = submit_plan(
        client_ref(&mut client),
        root.path(),
        manual_sandbox_plan(root.path()),
        &policy,
    )
    .await;

    // Unknown item: refused before anything is appended.
    let unknown = client
        .call(json!({
            "verb": "item_execute",
            "case_id": case_id,
            "item_id": "does-not-exist",
            "policy": policy,
            "timeout": 60,
            "request_id": format!("req-exec-1-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;
    assert!(
        unknown["error"]
            .as_str()
            .unwrap_or_default()
            .contains("not in the plan"),
        "{unknown}"
    );
    assert_eq!(
        kinds_of(&case_events(root.path(), &case_id)),
        vec!["case_created", "item_enabled"]
    );

    let response = client
        .call(json!({
            "verb": "item_execute",
            "case_id": case_id,
            "item_id": "task",
            "policy": policy,
            "timeout": 60,
            "request_id": format!("req-exec-2-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;
    assert_eq!(response["ok"], true, "{response}");
    assert_eq!(response["state"], "completed", "{response}");
    assert_eq!(
        kinds_of(&case_events(root.path(), &case_id)),
        vec![
            "case_created",
            "item_enabled",
            "item_activated",
            "settlement_recorded",
            "item_completed",
            "milestone_achieved",
            "case_closed"
        ]
    );

    // The item is terminal now: executing it again is refused with no writes.
    let spent = client
        .call(json!({
            "verb": "item_execute",
            "case_id": case_id,
            "item_id": "task",
            "policy": policy,
            "timeout": 60,
            "request_id": format!("req-exec-3-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;
    assert!(
        spent["error"]
            .as_str()
            .unwrap_or_default()
            .contains("not ready"),
        "{spent}"
    );
    assert_eq!(case_events(root.path(), &case_id).len(), 7);
}

// ---------------------------------------------------------------------------
// human_task.complete
// ---------------------------------------------------------------------------

#[tokio::test]
async fn human_task_complete_closes_the_case_and_mandates_a_justification() {
    let (root, socket) = boot(local_operator()).await;
    let policy = full_policy(root.path());
    let mut client = Client::connect(&socket).await;
    let case_id = submit_plan(
        client_ref(&mut client),
        root.path(),
        human_task_plan(),
        &policy,
    )
    .await;

    // Missing justification: refused, no event, case unchanged.
    let missing = client
        .call(json!({
            "verb": "human_task_complete",
            "case_id": case_id,
            "item_id": "judge",
            "justification": "",
            "policy": policy,
            "request_id": format!("req-htc-1-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;
    assert!(
        missing["error"]
            .as_str()
            .unwrap_or_default()
            .contains("non-empty justification"),
        "{missing}"
    );
    assert_eq!(
        kinds_of(&case_events(root.path(), &case_id)),
        vec!["case_created", "item_activated"]
    );

    let response = client
        .call(json!({
            "verb": "human_task_complete",
            "case_id": case_id,
            "item_id": "judge",
            "justification": "reviewed and accepted",
            "policy": policy,
            "request_id": format!("req-htc-2-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;
    assert_eq!(response["ok"], true, "{response}");
    assert_eq!(response["exit_code"], 0);

    assert_eq!(
        kinds_of(&case_events(root.path(), &case_id)),
        vec![
            "case_created",
            "item_activated",
            "human_task_completed",
            "case_closed"
        ]
    );
    assert_eq!(case_state(root.path(), &case_id), "completed");

    let completion = &case_events(root.path(), &case_id)[2];
    assert_eq!(completion["payload"]["note"], "reviewed and accepted");
}

// ---------------------------------------------------------------------------
// artifact.get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn artifact_get_returns_content_addressed_bytes_and_refuses_unknown_digests() {
    let (root, socket) = boot(local_operator()).await;
    let policy = full_policy(root.path());
    let mut client = Client::connect(&socket).await;
    let case_id = submit_plan(
        client_ref(&mut client),
        root.path(),
        manual_sandbox_plan(root.path()),
        &policy,
    )
    .await;
    let advance = client
        .call(json!({
            "verb": "case_advance",
            "case_id": case_id,
            "policy": policy,
            "timeout": 60,
            "request_id": format!("req-adv-art-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;
    assert_eq!(advance["ok"], true, "{advance}");

    // The digest the kernel's own artifact store committed for stdout.
    let runs = root.path().join("cases").join(&case_id).join("runs");
    let mut dirs: Vec<PathBuf> = fs::read_dir(&runs)
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .collect();
    dirs.sort();
    let evidence = read_jsonl(&dirs.last().unwrap().join("evidence.jsonl"));
    let artifact_record = evidence
        .iter()
        .find(|record| {
            record["kind"] == "artifact"
                && record["uri"]
                    .as_str()
                    .unwrap_or_default()
                    .ends_with("stdout.txt")
        })
        .expect("the episode captured stdout as an evidence artifact");
    let digest = artifact_record["sha256"].as_str().unwrap().to_owned();
    let expected_bytes =
        fs::read(dirs.last().unwrap().join("artifacts").join("stdout.txt")).unwrap();

    // Inspect verb: no actor block required.
    let fetched = client
        .call(json!({"verb": "artifact_get", "digest": digest}))
        .await;
    assert_eq!(fetched["digest"], digest, "{fetched}");
    assert_eq!(fetched["uri"], "artifacts/stdout.txt");
    assert_eq!(fetched["run_id"], artifact_record["run_id"]);
    assert_eq!(fetched["evidence_id"], artifact_record["evidence_id"]);
    let content = fetched["content"].as_str().unwrap().as_bytes().to_vec();
    assert_eq!(
        content, expected_bytes,
        "content-addressed bytes must match"
    );

    // Wrong digest: a typed refusal, not a wildcard read.
    let wrong = client
        .call(json!({"verb": "artifact_get", "digest": "b".repeat(64)}))
        .await;
    assert!(
        wrong["error"]
            .as_str()
            .unwrap_or_default()
            .contains("artifact_not_found"),
        "{wrong}"
    );

    // Malformed digest: refused before touching disk.
    let malformed = client
        .call(json!({"verb": "artifact_get", "digest": "not-a-digest"}))
        .await;
    assert!(
        malformed["error"]
            .as_str()
            .unwrap_or_default()
            .contains("malformed artifact digest"),
        "{malformed}"
    );
}

// ---------------------------------------------------------------------------
// Tooth: the proposer cannot resolve its own discretionary item's approval
// ---------------------------------------------------------------------------

/// operator_a proposes a discretionary sandboxed item; operator_b executes
/// it under an escalating policy (so the episode opens an approval whose
/// requester is operator_b); operator_a then tries to resolve the approval
/// and is refused as an SoD violation with `approvals.jsonl` unchanged.
#[tokio::test]
async fn the_proposer_cannot_resolve_its_own_discretionary_items_approval() {
    let (root, socket) = boot(two_actors()).await;
    let escalate = write_policy(
        root.path(),
        "escalate.yaml",
        &format!("{ESCALATE_RULES}{ADD_ITEM_RULE}"),
    );
    let mut proposer = Client::connect(&socket).await;

    // Submit the parked case as operator_a.
    let plan_path = root.path().join("plan-sod.json");
    fs::write(
        &plan_path,
        serde_json::to_vec(&manual_sandbox_plan(root.path())).unwrap(),
    )
    .unwrap();
    let submitted = proposer
        .call(json!({
            "verb": "submit",
            "plan": "plan-sod.json",
            "policy": escalate,
            "entity": "operator_a",
            "process": "test",
            "timeout": 60,
            "request_id": format!("req-sod-submit-{}", line_nonce()),
            "actor": actor_block("operator_a"),
        }))
        .await;
    let case_id = submitted["case_id"].as_str().expect("case_id").to_owned();

    // operator_a proposes a discretionary sandboxed item (auto-activatable).
    let proposed = proposer
        .call(json!({
            "verb": "case_add_item",
            "case_id": case_id,
            "item": {
                "plan_item_id": "disc",
                "name": "discretionary",
                "operations": [execute_operation(root.path())],
                "entry_criteria": [], "exit_criteria": [],
                "settlement_criteria": {"required_artifacts": [], "required_declarations": []},
                "item_kind": "sandboxed_task",
                "markers": {"required": false},
                "max_instances": 1,
                "depends_on": []
            },
            "policy": escalate,
            "request_id": format!("req-sod-add-{}", line_nonce()),
            "actor": actor_block("operator_a"),
        }))
        .await;
    assert_eq!(proposed["ok"], true, "{proposed}");
    assert_eq!(proposed["proposed_by"], "operator_a");

    // operator_b executes it; the escalate verdict opens an approval whose
    // requester is operator_b (the episode's entity).
    let mut executor = Client::connect(&socket).await;
    let executed = executor
        .call(json!({
            "verb": "item_execute",
            "case_id": case_id,
            "item_id": "disc",
            "policy": escalate,
            "timeout": 60,
            "request_id": format!("req-sod-exec-{}", line_nonce()),
            "actor": actor_block("operator_b"),
        }))
        .await;
    assert_eq!(executed["ok"], true, "{executed}");
    assert_eq!(executed["episodes"][0]["settlement_status"], "escalated");

    let listed = executor
        .call(json!({"verb": "approval_list", "case_id": case_id}))
        .await;
    let approval_id = listed["approvals"][0]["approval_id"]
        .as_str()
        .unwrap_or_else(|| panic!("no pending approval: {listed}"))
        .to_owned();

    let approvals_path = root.path().join("approvals.jsonl");
    let before = fs::read(&approvals_path).unwrap();

    // The proposer tries to resolve its own item's approval.
    let refused = proposer
        .call(json!({
            "verb": "approval_decide",
            "case_id": case_id,
            "approval_id": approval_id,
            "decision": "approve",
            "note": "trust me",
            "request_id": format!("req-sod-decide-{}", line_nonce()),
            "actor": actor_block("operator_a"),
        }))
        .await;
    assert_eq!(refused["error_class"], "sod_violation", "{refused}");
    assert!(
        refused["error"]
            .as_str()
            .unwrap_or_default()
            .contains("cannot resolve an approval"),
        "{refused}"
    );
    assert_eq!(
        fs::read(&approvals_path).unwrap(),
        before,
        "a refused resolution must leave approvals.jsonl unchanged"
    );
}

// ---------------------------------------------------------------------------
// Tooth: subscriber kill + resume from cursor
// ---------------------------------------------------------------------------

/// A subscriber sees frames in cursor order as mutations happen; after being
/// killed and resubscribed from its last seen cursor, replay has no gaps and
/// no duplicates, and every cursor is monotonic — cross-checked against the
/// durable ledger order via `events.get_range`.
#[tokio::test]
async fn subscriber_kill_and_resume_replays_without_gaps_or_duplicates() {
    let (root, socket) = boot(local_operator()).await;
    let policy = full_policy(root.path());

    let mut subscriber = Client::connect(&socket).await;
    let subscribe = subscriber.call(json!({"verb": "events_subscribe"})).await;
    assert_eq!(subscribe["ok"], true, "{subscribe}");
    let replayed_before: usize = subscribe["replayed"].as_u64().unwrap() as usize;

    // A mutation happens; the subscriber receives its trace frame live.
    let mut operator = Client::connect(&socket).await;
    let case_id = submit_plan(
        client_ref(&mut operator),
        root.path(),
        manual_sandbox_plan(root.path()),
        &policy,
    )
    .await;
    let add = operator
        .call(json!({
            "verb": "case_add_item",
            "case_id": case_id,
            "item": {
                "plan_item_id": "extra",
                "name": "discretionary",
                "operations": [],
                "entry_criteria": [], "exit_criteria": [],
                "settlement_criteria": {"required_artifacts": [], "required_declarations": []},
                "item_kind": "human_task",
                "markers": {"required": false},
                "max_instances": 1,
                "depends_on": []
            },
            "policy": policy,
            "request_id": format!("req-sub-add-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;
    assert_eq!(add["ok"], true, "{add}");

    let frame = subscriber.next_event("case.trace.plan_mutated").await;
    assert_eq!(frame["case_id"], case_id.as_str());
    assert_eq!(frame["detail"]["kind"], "plan_mutated");
    assert_eq!(frame["detail"]["plan_item_id"], "extra");
    let last_seen = frame["cursor"].as_str().unwrap().to_owned();
    let first_batch_cursors = vec![last_seen.clone()];

    // Kill the subscriber mid-stream (drop the connection entirely).
    drop(subscriber);

    // More durable mutations happen while nobody is subscribed.
    let terminate = operator
        .call(json!({
            "verb": "case_terminate",
            "case_id": case_id,
            "reason": "operator chose to stop here",
            "policy": policy,
            "request_id": format!("req-sub-term-{}", line_nonce()),
            "actor": actor_block("operator_local"),
        }))
        .await;
    assert_eq!(terminate["ok"], true, "{terminate}");

    // Resubscribe from the last seen cursor. The replay burst lands on the
    // wire before the subscribe ack, so drain the replayed frames from the
    // stream and pair them with the response.
    let mut resumed = Client::connect(&socket).await;
    let (resubscribe, replayed_frames) = resumed
        .call_among_events(json!({"verb": "events_subscribe", "from_cursor": last_seen}))
        .await;
    assert_eq!(resubscribe["ok"], true, "{resubscribe}");
    let replayed = resubscribe["replayed"].as_u64().unwrap() as usize;
    assert!(
        replayed >= 1,
        "the terminated frame must be replayed: {resubscribe}"
    );
    assert_eq!(
        replayed_frames.len(),
        replayed,
        "replayed count must equal the frames delivered before the ack"
    );
    assert_eq!(
        replayed_frames[0]["kind"], "case.trace.case_terminated",
        "replay must lead with the frame the killed subscriber missed"
    );

    // No gaps and no duplicates: combine what the first subscriber saw live
    // with what the resubscription replayed, and compare against the durable
    // ledger order from `events.get_range` starting at the same cursor.
    let range = resumed
        .call(json!({"verb": "events_get_range", "from_cursor": last_seen}))
        .await;
    let ledger_order: Vec<String> = range["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|frame| frame["cursor"].as_str().unwrap().to_owned())
        .collect();
    assert!(
        !ledger_order.is_empty(),
        "ledger must hold the missed frames"
    );
    assert_eq!(
        ledger_order.len(),
        ledger_order
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        "durable ledger itself must not repeat cursors"
    );

    // The replayed burst is a prefix of the durable order and contains the
    // terminated trace frame.
    let replayed = resubscribe["replayed"].as_u64().unwrap() as usize;
    assert!(replayed <= ledger_order.len());
    let replayed_cursors = &ledger_order[..replayed];
    let combined: Vec<String> = first_batch_cursors
        .into_iter()
        .chain(replayed_cursors.iter().cloned())
        .collect();
    let unique: std::collections::HashSet<&String> = combined.iter().collect();
    assert_eq!(
        unique.len(),
        combined.len(),
        "duplicate cursors after resume: {combined:?}"
    );
    for pair in combined.windows(2) {
        assert!(
            pair[0] < pair[1],
            "cursors must be strictly monotonic across the resume: {combined:?}"
        );
    }
    assert_eq!(
        ledger_order[0], replayed_cursors[0],
        "replay must start at the first durable frame after the cursor"
    );

    // The missed mutation is in the replay, in durable order.
    let resumed_frame = &replayed_frames[0];
    assert_eq!(resumed_frame["case_id"], case_id.as_str());
    assert_eq!(resumed_frame["detail"]["kind"], "case_terminated");
    assert!(resumed_frame["cursor"].as_str().unwrap() > last_seen.as_str());

    // Sanity: the subscriber attached to a fresh cell, so its initial
    // subscribe had nothing to replay — every frame it saw was live.
    assert_eq!(replayed_before, 0, "{subscribe}");
}

/// A tiny helper so `submit_plan` can borrow the same client the test holds.
fn client_ref(client: &mut Client) -> &mut Client {
    client
}
