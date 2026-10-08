//! The opt-in case-advance supervisor over a real socket (plan T04 step 5,
//! operator decision D-3, unit C).
//!
//! Every test here boots the real server on a temp cell — the same pattern as
//! `sfwp_case_mutations.rs` — and asserts the *durable* effect (the exact
//! `TraceKind` sequence in `cases/<id>/case-events.jsonl`, byte-identity where
//! the claim is "nothing happened", and the run trace's own events). The
//! supervisor is a background task spawned by `run()`; its only observable
//! behavior is what it writes to the cell, which is why unit-level placement
//! could not prove requirements (b)–(e) — the config parse/validate teeth for
//! the same feature live as unit tests in `src/config.rs`.
//!
//! Coverage:
//! - (b) default config: the supervisor is never spawned, so a ready enabled
//!   SandboxedTask never auto-advances within a window longer than a default
//!   poll interval;
//! - (c) supervisor enabled: the ready SandboxedTask auto-advances with NO
//!   client `case.advance`, in the engine's documented event order, honestly
//!   attributed to the configured supervisor actor — and the completed case is
//!   then quiet across further polls;
//! - (d) sandbox-class selection: a case holding a human task/sign-off gate is
//!   never completed by the supervisor; `case-events.jsonl` stays
//!   byte-identical and `approvals.jsonl` is never created;
//! - (e) per-case serialization: a client `case.advance` racing a supervisor
//!   pass on the same case yields coherent results, exactly one episode, and
//!   no torn or duplicated jsonl lines.

use sea_forge_server::config::SupervisorConfig;
use sea_forge_server::identity::IdentityBindings;
use sea_forge_server::{run, ServerConfig};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, UnixStream};

// ---------------------------------------------------------------------------
// Harness (mirrors sfwp_case_mutations)
// ---------------------------------------------------------------------------

fn local_operator() -> IdentityBindings {
    IdentityBindings::local_operator("operator_local")
}

/// The supervisor under test: enabled, fast polls, two case passes per wave,
/// attributed to a dedicated actor (never `operator_local`).
fn enabled_supervisor(actor: &str) -> SupervisorConfig {
    SupervisorConfig {
        enabled: true,
        poll_interval_secs: 1,
        max_concurrent_cases: 2,
        actor: actor.into(),
    }
}

fn boot_config(root: &Path, supervisor: SupervisorConfig) -> ServerConfig {
    // Short, because a deep temp path overflows `sun_path` (cell contract).
    let socket = std::env::temp_dir().join(format!(
        "sf-t04c-{}-{}.sock",
        std::process::id(),
        root.file_name().unwrap().to_string_lossy()
    ));
    ServerConfig {
        socket_path: socket,
        root: root.to_path_buf(),
        identity: local_operator(),
        supervisor,
        ..ServerConfig::default()
    }
}

async fn boot(supervisor: SupervisorConfig) -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let config = boot_config(root.path(), supervisor);
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

/// Install the cell's active authority policy at the exact spelling the
/// supervisor authorizes against (`authority/active-policy.json`): one rule
/// for the socket's verified human operator, one for the supervisor's
/// `service` role. Same YAML-in-.json shape the E2E fixture installs.
fn install_active_policy(root: &Path) -> &'static str {
    let dir = root.join("authority");
    fs::create_dir_all(&dir).expect("create authority dir");
    fs::write(
        dir.join("active-policy.json"),
        "version: \"0.1\"\nrules:\n  \
         - name: allow-operator-command\n    verdict: allow\n    actor_role: operator\n    \
         operation_kind: execute_command\n    argv0: sea-forge\n  \
         - name: allow-supervisor-command\n    verdict: allow\n    actor_role: service\n    \
         operation_kind: execute_command\n    argv0: sea-forge\n",
    )
    .expect("install active policy");
    "authority/active-policy.json"
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
        let n = tokio::time::timeout(Duration::from_secs(10), self.reader.read_line(&mut line))
            .await
            .expect("read timed out")
            .unwrap();
        assert!(n > 0, "connection closed unexpectedly");
        serde_json::from_str(line.trim()).unwrap()
    }
}

fn actor_block(actor_id: &str) -> Value {
    json!({"actor_id": actor_id, "role": "operator"})
}

fn line_nonce() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Arguments that make the libtest harness exit 0 without running anything.
const EXIT_ZERO: &[&str] = &["--exact", "__no_test_matches_this_name__"];

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

fn execute_operation(dir: &Path) -> Value {
    let mut argv = vec![trusted_self_executable(dir).to_string_lossy().into_owned()];
    argv.extend(EXIT_ZERO.iter().map(|arg| (*arg).to_owned()));
    json!({"kind": "execute_command", "argv": argv, "cwd": "."})
}

/// A plan whose single SandboxedTask carries `manual_activation`: submit
/// enables it and parks, which is exactly the state the supervisor must
/// auto-advance when enabled — and must NOT touch when disabled.
fn manual_sandbox_plan(dir: &Path) -> Value {
    json!({
        "version": "0.2",
        "plan_id": "plan_t04c",
        "case_id": "case_placeholder",
        "run_id": "run_placeholder",
        "intent_id": "int_t04c",
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
/// {"human_task": true}` awaiting `human_task.complete`. The supervisor must
/// never complete it and never create `approvals.jsonl`.
fn human_task_plan() -> Value {
    json!({
        "version": "0.2",
        "plan_id": "plan_t04c_judge",
        "case_id": "case_placeholder",
        "run_id": "run_placeholder",
        "intent_id": "int_t04c_judge",
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

/// Submit a plan over the socket and return the case id. The policy spelling
/// is the cell's active policy — the same file the supervisor uses — so the
/// verb and the supervisor authorize against one bundle.
async fn submit_plan(client: &mut Client, root: &Path, plan: Value, policy: &str) -> String {
    let plan_path = root.join("plan-t04c.json");
    fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let response = client
        .call(json!({
            "verb": "submit",
            "plan": "plan-t04c.json",
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

fn case_events_path(root: &Path, case_id: &str) -> PathBuf {
    root.join("cases").join(case_id).join("case-events.jsonl")
}

fn read_jsonl(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn case_events(root: &Path, case_id: &str) -> Vec<Value> {
    read_jsonl(&case_events_path(root, case_id))
}

fn kinds_of(events: &[Value]) -> Vec<String> {
    events
        .iter()
        .map(|event| event["kind"].as_str().unwrap_or_default().to_owned())
        .collect()
}

fn case_state(root: &Path, case_id: &str) -> String {
    let case: Value = serde_json::from_slice(
        &fs::read(root.join("cases").join(case_id).join("case.json")).unwrap(),
    )
    .unwrap();
    case["state"].as_str().unwrap().to_owned()
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

/// The kinds of one run directory's `trace.jsonl` (the newest run).
fn latest_run_trace_kinds(root: &Path, case_id: &str) -> Vec<String> {
    let latest = latest_run_dir(root, case_id).expect("at least one run directory");
    let events = read_jsonl(&latest.join("trace.jsonl"));
    events
        .iter()
        .map(|event| event["kind"].as_str().unwrap_or_default().to_owned())
        .collect()
}

/// Poll until the case's events satisfy `done` or the deadline passes;
/// panics with the last observed kinds so a timeout reports real state.
async fn wait_for_case<F: FnMut(&[Value]) -> bool>(
    root: &Path,
    case_id: &str,
    mut done: F,
    timeout: Duration,
) -> Vec<Value> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let events = case_events(root, case_id);
        if done(&events) {
            return events;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "condition never held; last kinds: {:?}",
            kinds_of(&events)
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

// ---------------------------------------------------------------------------
// (b) Default config: no supervisor activity, ever
// ---------------------------------------------------------------------------

/// A server with no `supervisor:` section never spawns the task: a case
/// holding an enabled, ready SandboxedTask sits untouched. The wait window
/// spans more than one *default* poll interval (5s) plus the immediate
/// first pass a wrongly-spawned task would take at boot, so the absence is
/// an honest observation, not a race the supervisor could hide behind.
#[tokio::test]
async fn default_config_never_auto_advances_a_ready_sandboxed_task() {
    let (root, socket) = boot(SupervisorConfig::default()).await;
    let policy = install_active_policy(root.path());
    let mut client = Client::connect(&socket).await;
    let case_id = submit_plan(
        &mut client,
        root.path(),
        manual_sandbox_plan(root.path()),
        policy,
    )
    .await;

    // Submit parked the case at the enabled manual item.
    let events_path = case_events_path(root.path(), &case_id);
    let before = fs::read(&events_path).unwrap();
    assert_eq!(
        kinds_of(&case_events(root.path(), &case_id)),
        vec!["case_created", "item_enabled"]
    );

    tokio::time::sleep(Duration::from_millis(6500)).await;

    let after = fs::read(&events_path).unwrap();
    assert_eq!(
        before,
        after,
        "a disabled (default) supervisor must append nothing: {:?}",
        kinds_of(&case_events(root.path(), &case_id))
    );
    assert_eq!(case_state(root.path(), &case_id), "active");
    // No episode ran, so no run directory exists under the case.
    let runs = root.path().join("cases").join(&case_id).join("runs");
    let run_count = fs::read_dir(&runs)
        .map(|entries| entries.flatten().count())
        .unwrap_or(0);
    assert_eq!(run_count, 0, "no episode may run without the supervisor");
}

// ---------------------------------------------------------------------------
// (c) Enabled: auto-advance with no client call, then quiet when terminal
// ---------------------------------------------------------------------------

/// Without any client `case.advance`, the enabled supervisor advances the
/// ready SandboxedTask through the SAME engine the verb uses: the durable
/// `case-events.jsonl` sequence is exactly the engine's documented order,
/// the run trace carries the episode's own events, and every record is
/// attributed to the configured supervisor actor — never `operator_local`.
/// Once the case is terminal the supervisor is quiet across further polls.
#[tokio::test]
async fn supervisor_enabled_auto_advances_ready_sandboxed_task_without_a_client() {
    let (root, socket) = boot(enabled_supervisor("cell_supervisor")).await;
    let policy = install_active_policy(root.path());
    let mut client = Client::connect(&socket).await;
    let case_id = submit_plan(
        &mut client,
        root.path(),
        manual_sandbox_plan(root.path()),
        policy,
    )
    .await;

    // No case.advance / item.execute from any caller: wait for the
    // supervisor's poll loop to complete the case on its own.
    let events = wait_for_case(
        root.path(),
        &case_id,
        |events| kinds_of(events).iter().any(|kind| kind == "case_closed"),
        Duration::from_secs(15),
    )
    .await;

    // The engine's documented sequence for this fixture (identical to what
    // the verb produces — one code path).
    assert_eq!(
        kinds_of(&events),
        vec![
            "case_created",
            "item_enabled",
            "item_activated",
            "settlement_recorded",
            "item_completed",
            "milestone_achieved",
            "case_closed",
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
            "artifact_captured",
        ]
    );

    // Honest attribution: the episode's trace records name the configured
    // supervisor actor, not an end user.
    let trace = fs::read_to_string(
        latest_run_dir(root.path(), &case_id)
            .expect("run directory")
            .join("trace.jsonl"),
    )
    .unwrap();
    assert!(
        trace.contains("cell_supervisor"),
        "supervisor episodes must be attributed to supervisor.actor"
    );
    assert!(
        !trace.contains("operator_local"),
        "the supervisor must never impersonate the end-user actor"
    );

    // Quiet when done: two more polls append nothing to a terminal case.
    let settled = fs::read(case_events_path(root.path(), &case_id)).unwrap();
    tokio::time::sleep(Duration::from_millis(2500)).await;
    assert_eq!(
        settled,
        fs::read(case_events_path(root.path(), &case_id)).unwrap(),
        "a completed case must stay byte-identical across supervisor polls"
    );
}

// ---------------------------------------------------------------------------
// (d) Sandbox-class selection: human work is never the supervisor's to finish
// ---------------------------------------------------------------------------

/// A case whose only work is a human task/sign-off gate: the supervisor
/// never completes it, never activates new human work, and never writes an
/// approval. `case-events.jsonl` stays byte-identical across several polls
/// and `approvals.jsonl` is not even created.
#[tokio::test]
async fn supervisor_never_completes_human_tasks_or_creates_approvals() {
    let (root, socket) = boot(enabled_supervisor("cell_supervisor")).await;
    let policy = install_active_policy(root.path());
    let mut client = Client::connect(&socket).await;
    let case_id = submit_plan(&mut client, root.path(), human_task_plan(), policy).await;

    // Submit parked the human task; that is the only activation this case
    // will ever see from anyone but `human_task.complete`.
    let events_path = case_events_path(root.path(), &case_id);
    let events_before = fs::read(&events_path).unwrap();
    assert_eq!(
        kinds_of(&case_events(root.path(), &case_id)),
        vec!["case_created", "item_activated"]
    );
    let approvals_path = root.path().join("approvals.jsonl");
    let approvals_before = fs::read(&approvals_path).ok();

    // Three full poll intervals with the supervisor enabled.
    tokio::time::sleep(Duration::from_millis(3500)).await;

    assert_eq!(
        events_before,
        fs::read(&events_path).unwrap(),
        "the supervisor must leave a human-task case byte-identical: {:?}",
        kinds_of(&case_events(root.path(), &case_id))
    );
    assert_eq!(
        approvals_before,
        fs::read(&approvals_path).ok(),
        "the supervisor must never write approvals.jsonl"
    );
    assert!(
        !approvals_path.exists(),
        "no approval may exist for a case the supervisor merely polled"
    );
    let kinds = kinds_of(&case_events(root.path(), &case_id));
    assert!(
        !kinds.contains(&"human_task_completed".to_string()),
        "the supervisor must never complete a human task: {kinds:?}"
    );
    assert_eq!(case_state(root.path(), &case_id), "active");
}

// ---------------------------------------------------------------------------
// (e) Per-case serialization: verb and supervisor never interleave a case
// ---------------------------------------------------------------------------

/// Supervisor enabled; a client `case.advance` fires near the supervisor's
/// next 1s poll so the two passes race as often as this timing allows
/// (overlap is best-effort — the invariants below must hold whether or not
/// the passes actually interleave): both callers get coherent results, the
/// episode runs exactly once, and every jsonl line parses — no torn or
/// duplicated lines.
#[tokio::test]
async fn supervisor_and_client_advance_serialize_on_the_same_case() {
    let (root, socket) = boot(enabled_supervisor("cell_supervisor")).await;
    let policy = install_active_policy(root.path());
    let mut client = Client::connect(&socket).await;
    let case_id = submit_plan(
        &mut client,
        root.path(),
        manual_sandbox_plan(root.path()),
        policy,
    )
    .await;

    // The supervisor polls at ~1s intervals from boot; approach the next poll.
    tokio::time::sleep(Duration::from_millis(950)).await;
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
    assert!(
        response["state"] == "completed" || response["state"] == "idle",
        "the client must get a coherent outcome whether it or the \
         supervisor held the per-case lock: {response}"
    );
    if response["state"] == "idle" {
        // The supervisor ran the episode first; wait for its pass to settle
        // the case so both callers' effects are visible before asserting.
        wait_for_case(
            root.path(),
            &case_id,
            |events| kinds_of(events).iter().any(|kind| kind == "case_closed"),
            Duration::from_secs(10),
        )
        .await;
    }

    // Line integrity: the journal ends with a newline and every non-empty
    // line is a complete JSON record — a torn append would fail to parse.
    let text = fs::read_to_string(case_events_path(root.path(), &case_id)).unwrap();
    assert!(text.ends_with('\n'), "journal must end with a full line");
    for (index, line) in text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .enumerate()
    {
        serde_json::from_str::<Value>(line)
            .unwrap_or_else(|error| panic!("torn jsonl line {index}: {error}: {line}"));
    }

    // Exactly one episode: serialized passes cannot both see the item ready.
    let events = case_events(root.path(), &case_id);
    let kinds = kinds_of(&events);
    assert_eq!(
        kinds
            .iter()
            .filter(|kind| *kind == "item_activated")
            .count(),
        1,
        "the episode must activate exactly once: {kinds:?}"
    );
    assert_eq!(
        kinds
            .iter()
            .filter(|kind| *kind == "settlement_recorded")
            .count(),
        1,
        "exactly one settlement may be recorded: {kinds:?}"
    );
    assert_eq!(case_state(root.path(), &case_id), "completed");
}
