//! Live plan-template wiring (plan casework-live-wiring-production T03,
//! resolves GAP-D): real `PlanTemplate` YAMLs, loaded through the real
//! loader, proven over the real Unix-socket server on a temp cell seeded the
//! way `just casework-cell-init` seeds the live cell.
//!
//! The seed mirrors the justfile recipe exactly (templates copied from
//! `fixtures/cells/e2e/templates/`, the operator identity binding written for
//! the invoking uid, `fixtures/cells/e2e/policy.yaml` installed at
//! `authority/active-policy.json`) and — importantly — the server config is
//! then built by LOADING that seeded `server.yaml` through
//! `ServerConfig::load`, so the identity the protected verbs rely on is the
//! one the recipe actually writes, not a forked test fixture.
//!
//! Honesty note on `PlanCreated`: the SFWP `case.commit` path records the
//! plan as a `case_plan` ledger record plus `cases/<id>/plan.json`; the
//! `PlanCreated` TraceKind belongs to the CLI plan pipeline and is never
//! appended by `case_dispatch::submit`. These tests assert what the kernel
//! actually writes — `CaseCreated` in `case-events.jsonl`, the plan in the
//! ledger and on disk — rather than an event the live path does not emit.

use sea_forge_core::types::{TraceEvent, TraceKind};
use sea_forge_server::{identity, run, ServerConfig};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, UnixStream};

const SENTRY_CHAIN_REF: &str = "e2e-sentry-chain@0.1.0";
const SIGNOFF_GATE_REF: &str = "e2e-signoff-gate@0.1.0";
const OPERATOR_ACTOR: &str = "operator_local";

/// The checked-in E2E fixtures root (`fixtures/cells/e2e` in the repo).
fn e2e_fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cells/e2e")
}

/// Seed a temp cell exactly the way `just casework-cell-init` seeds the live
/// cell (T03): the templates, the operator identity binding for the invoking
/// uid, and the E2E authority policy. The template list is enumerated from
/// the same fixtures directory the recipe globs — one source of truth, no
/// forked list.
fn seed_cell(root: &Path) {
    let fixtures = e2e_fixtures();

    let templates = root.join("templates");
    std::fs::create_dir_all(&templates).expect("create templates dir");
    let mut installed = 0;
    for entry in std::fs::read_dir(fixtures.join("templates"))
        .expect("fixtures templates dir must exist")
        .flatten()
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("yaml") {
            let target = templates.join(entry.file_name());
            std::fs::copy(&path, &target)
                .unwrap_or_else(|e| panic!("copy template {}: {e}", target.display()));
            installed += 1;
        }
    }
    assert!(
        installed >= 2,
        "expected both E2E templates, found {installed}"
    );

    let uid = identity::current_uid().expect("invoking uid must be readable");
    let server_yaml = format!(
        "# Live-stack cell seed (plan casework-live-wiring-production T03).\n\
         # The invoking OS user (uid {uid}) is bound to the operator actor.\n\
         identity:\n  bindings:\n    - uid: {uid}\n      actor_id: {OPERATOR_ACTOR}\n      roles: [\"operator\"]\n"
    );
    std::fs::write(root.join("server.yaml"), server_yaml).expect("write server.yaml");

    let authority = root.join("authority");
    std::fs::create_dir_all(&authority).expect("create authority dir");
    std::fs::copy(
        fixtures.join("policy.yaml"),
        authority.join("active-policy.json"),
    )
    .expect("install E2E policy");
}

/// Boot a server on a temp cell seeded like `casework-cell-init`. The config
/// is loaded from the seeded `server.yaml` (then pointed at this temp root
/// and socket), so the identity bindings in play are the recipe's own.
async fn boot() -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    seed_cell(root.path());
    let socket = root.path().join("sfwp.sock");
    let mut config = ServerConfig::load(&root.path().join("server.yaml"))
        .expect("seeded server.yaml must load as a real ServerConfig");
    config.root = root.path().to_path_buf();
    config.socket_path = socket.clone();

    tokio::spawn(async move {
        let _ = run(config).await;
    });
    for _ in 0..200 {
        if socket.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(socket.exists(), "server socket must appear");
    (root, socket)
}

/// A minimal line-delimited JSON client over the Unix socket (the shared
/// idiom of `conformance_sfwp.rs` / `conformance_case_authoring.rs`).
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

/// Valid parameters for the sentry-chain template.
fn sentry_chain_params() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("dataset_name".to_string(), "orders-q3".to_string()),
        ("dataset_label".to_string(), "cj04".to_string()),
        ("max_rows".to_string(), "25".to_string()),
        ("out_dir".to_string(), "work".to_string()),
    ])
}

fn actor_block() -> Value {
    json!({"actor_id": OPERATOR_ACTOR, "role": "operator"})
}

/// The case's trace events, read off `cases/<id>/case-events.jsonl`.
fn case_events(root: &Path, case_id: &str) -> Vec<TraceEvent> {
    let text = std::fs::read_to_string(root.join("cases").join(case_id).join("case-events.jsonl"))
        .expect("case-events.jsonl must exist");
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

/// Every entry kind in the case's ledger (`ledgers/case-<id>`).
fn ledger_entry_kinds(root: &Path, case_id: &str) -> Vec<String> {
    let ledger = sea_forge_ledger::LedgerStream::open(
        root,
        format!("case-{case_id}"),
        "case_templates_live",
    )
    .expect("open case ledger");
    ledger
        .read_entries()
        .expect("read case ledger")
        .into_iter()
        .map(|entry| entry.record_kind)
        .collect()
}

/// Preflight then commit `template_ref` over the socket, carrying the
/// preflight precondition into the commit exactly as the contract requires.
async fn preflight_then_commit(
    client: &mut Client,
    template_ref: &str,
    params: &BTreeMap<String, String>,
) -> (Value, Value) {
    let preflight = client
        .call(json!({
            "verb": "case_preflight",
            "template_ref": template_ref,
            "params": params,
        }))
        .await;
    assert_eq!(preflight["ok"], true, "preflight failed: {preflight}");

    let mut commit = json!({
        "verb": "case_commit", "actor": actor_block(),
        "template_ref": template_ref,
        "params": params,
        "policy": "authority/active-policy.json",
        "entity": OPERATOR_ACTOR,
        "process": "case_templates_live",
        // Socket-path durable mutations are admitted only under a correlated
        // request id (`durable_locator_required`).
        "request_id": format!("req-t03-{template_ref}-{}", params.get("dataset_name")
            .or_else(|| params.get("change_summary"))
            .map(|v| v.replace('/', "_"))
            .unwrap_or_else(|| "x".into())),
    });
    if let Some(expected) = preflight["precondition"]["expected_digest"].as_str() {
        commit["preconditions"] = json!({
            "records": [
                {"ref": format!("template:{template_ref}"), "expected_digest": expected}
            ]
        });
    }
    let committed = client.call(commit).await;
    (preflight, committed)
}

#[tokio::test]
async fn entry_options_lists_both_e2e_templates_with_parameters() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let response = client.call(json!({"verb": "case_entry_options"})).await;

    let templates = response["templates"].as_array().unwrap();
    let find = |reference: &str| {
        templates
            .iter()
            .find(|t| t["template_ref"] == reference)
            .unwrap_or_else(|| panic!("{reference} missing from entry_options: {response}"))
            .clone()
    };
    let chain = find(SENTRY_CHAIN_REF);
    let gate = find(SIGNOFF_GATE_REF);

    // Template A's parameters arrive with their declared type, requiredness,
    // and defaults — the real ParameterDefs, not a projection of them.
    let dataset_name = &chain["parameters"]["dataset_name"];
    assert_eq!(dataset_name["param_type"], "string");
    assert_eq!(dataset_name["required"], true);
    assert!(dataset_name["default"].is_null());
    let max_rows = &chain["parameters"]["max_rows"];
    assert_eq!(max_rows["param_type"], "int");
    assert_eq!(max_rows["default"], "10");
    assert_eq!(chain["parameters"]["out_dir"]["param_type"], "path");

    // Template B carries its own required parameter.
    let summary = &gate["parameters"]["change_summary"];
    assert_eq!(summary["param_type"], "string");
    assert_eq!(summary["required"], true);
}

#[tokio::test]
async fn preflight_passes_with_items_and_a_template_digest() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let response = client
        .call(json!({
            "verb": "case_preflight",
            "template_ref": SENTRY_CHAIN_REF,
            "params": sentry_chain_params(),
        }))
        .await;

    assert_eq!(response["ok"], true, "expected ok preflight: {response}");
    // `errors` is skip-serialized when empty — absence IS the success shape.
    assert!(
        response
            .get("errors")
            .and_then(|e| e.as_array())
            .is_none_or(|e| e.is_empty()),
        "unexpected preflight errors: {response}"
    );
    assert_eq!(
        response["precondition"]["ref"],
        format!("template:{SENTRY_CHAIN_REF}")
    );
    assert!(response["precondition"]["expected_digest"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));

    // The instantiated draft carries the two sandboxed tasks and the rollup
    // milestone, in template order.
    let items = response["items"].as_array().unwrap();
    let ids: Vec<&str> = items
        .iter()
        .map(|item| item["plan_item_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["task_prepare", "task_publish", "ms_chain_accepted"]);
    assert_eq!(items[0]["item_kind"], "SandboxedTask");
    assert_eq!(items[1]["item_kind"], "SandboxedTask");
    assert_eq!(items[2]["item_kind"], "Milestone");
}

#[tokio::test]
async fn preflight_reports_the_designed_failure_reason_for_invalid_params() {
    let (_root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;

    // An int parameter fed a non-int value: the declared type constraint is
    // the designed failure.
    let mut bad_type = sentry_chain_params();
    bad_type.insert("max_rows".to_string(), "many".to_string());
    let response = client
        .call(json!({
            "verb": "case_preflight",
            "template_ref": SENTRY_CHAIN_REF,
            "params": bad_type,
        }))
        .await;
    assert_eq!(response["ok"], false, "invalid int must fail: {response}");
    let errors: Vec<&str> = response["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e.as_str().unwrap())
        .collect();
    assert!(
        errors
            .iter()
            .any(|e| e.contains("parameter max_rows must be int")),
        "designed type failure missing: {errors:?}"
    );

    // A missing required parameter fails with its own reason.
    let mut missing = sentry_chain_params();
    missing.remove("dataset_name");
    let response = client
        .call(json!({
            "verb": "case_preflight",
            "template_ref": SENTRY_CHAIN_REF,
            "params": missing,
        }))
        .await;
    assert_eq!(response["ok"], false);
    let errors: Vec<&str> = response["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e.as_str().unwrap())
        .collect();
    assert!(
        errors
            .iter()
            .any(|e| e.contains("missing required template parameter: dataset_name")),
        "missing-parameter failure missing: {errors:?}"
    );
}

#[tokio::test]
async fn commit_creates_the_case_with_case_created_and_the_plan_record() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let (_preflight, commit) =
        preflight_then_commit(&mut client, SENTRY_CHAIN_REF, &sentry_chain_params()).await;

    assert!(commit.get("error").is_none(), "commit failed: {commit}");
    let case_id = commit["case_id"]
        .as_str()
        .expect("case_id present")
        .to_string();
    // `ids::case_id` mints `case_<timestamp>_<rand>` (underscore-separated).
    assert!(
        case_id.starts_with("case_"),
        "kernel-minted case id: {case_id}"
    );
    assert_eq!(commit["state"], "active");

    // Trace truth: CaseCreated is the first event in the case's own journal.
    let events = case_events(root.path(), &case_id);
    assert_eq!(events[0].kind, TraceKind::CaseCreated);
    assert_eq!(events[0].payload["case_id"], case_id.as_str());

    // Plan truth: the SFWP commit path commits a `case_plan` ledger record
    // and writes `plan.json` naming the originating template. (It does not
    // append a `PlanCreated` trace event — that TraceKind belongs to the CLI
    // plan pipeline; see this file's header.)
    let kinds = ledger_entry_kinds(root.path(), &case_id);
    assert!(
        kinds.iter().any(|kind| kind == "case_plan"),
        "case_plan ledger record missing: {kinds:?}"
    );
    let plan: Value = serde_json::from_str(
        &std::fs::read_to_string(root.path().join("cases").join(&case_id).join("plan.json"))
            .expect("plan.json must exist"),
    )
    .unwrap();
    assert_eq!(plan["template_ref"], SENTRY_CHAIN_REF);
    let ids: Vec<&str> = plan["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["plan_item_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["task_prepare", "task_publish", "ms_chain_accepted"]);
}

#[tokio::test]
async fn overview_and_horizon_report_the_sentry_chain_standing() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let (_preflight, commit) =
        preflight_then_commit(&mut client, SENTRY_CHAIN_REF, &sentry_chain_params()).await;
    let case_id = commit["case_id"].as_str().unwrap().to_string();

    let overview = client
        .call(json!({"verb": "case_get_overview", "case_id": case_id}))
        .await;
    assert_eq!(overview["case_state"], "active");
    assert_eq!(overview["template_ref"], SENTRY_CHAIN_REF);
    assert_eq!(overview["item_count"], 3);
    assert_eq!(overview["plan_ref"], "plan_01");
    assert!(overview["run_ids"].as_array().unwrap().is_empty());

    let horizon = client
        .call(json!({"verb": "case_get_horizon", "case_id": case_id}))
        .await;
    let rows: Vec<Value> = horizon["items"].as_array().unwrap().clone();
    assert_eq!(rows.len(), 3, "one row per plan item: {horizon}");

    let standing = |id: &str| {
        rows.iter()
            .find(|row| row["plan_item_id"] == id)
            .unwrap_or_else(|| panic!("{id} missing from horizon: {horizon}"))
            .clone()
    };

    // The real standing vocabulary (`sfwp::case_views`): the manual-activation
    // head of the chain is `enabled` (entry criteria met, eligible for
    // dispatch); the sentry-gated item and the rollup milestone are `pending`
    // — blocked by their entry criteria, not failed, with nothing settled.
    let prepare = standing("task_prepare");
    assert_eq!(prepare["execution"], "enabled");
    assert_eq!(prepare["settlement"], "unsettled");

    let publish = standing("task_publish");
    assert_eq!(publish["execution"], "pending");
    assert_eq!(publish["settlement"], "unsettled");

    let milestone = standing("ms_chain_accepted");
    assert_eq!(milestone["execution"], "pending");
    assert_eq!(milestone["settlement"], "unsettled");

    // The events the standing was folded from are the case's own trace truth.
    assert!(horizon["events_folded"].as_u64().unwrap() >= 1);
    assert_eq!(
        horizon["last_event_id"],
        case_events(root.path(), &case_id).last().unwrap().event_id
    );
}

#[tokio::test]
async fn signoff_gate_template_parks_the_human_task_for_a_decision() {
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;
    let params = BTreeMap::from([(
        "change_summary".to_string(),
        "rotate the e2e signing key".to_string(),
    )]);
    let (_preflight, commit) = preflight_then_commit(&mut client, SIGNOFF_GATE_REF, &params).await;
    assert!(commit.get("error").is_none(), "commit failed: {commit}");
    let case_id = commit["case_id"].as_str().unwrap().to_string();

    let horizon = client
        .call(json!({"verb": "case_get_horizon", "case_id": case_id}))
        .await;
    let rows: Vec<Value> = horizon["items"].as_array().unwrap().clone();
    let standing = |id: &str| {
        rows.iter()
            .find(|row| row["plan_item_id"] == id)
            .unwrap_or_else(|| panic!("{id} missing from horizon: {horizon}"))
            .clone()
    };

    // The draft is enabled (manual activation); the sign-off is `active` with
    // nothing settled — the case engine parked the HumanTask
    // (ItemActivated{human_task: true}) and the case now waits for the
    // operator decision that resolves it.
    assert_eq!(standing("task_draft")["execution"], "enabled");
    let signoff = standing("signoff_release");
    assert_eq!(signoff["execution"], "active");
    assert_eq!(signoff["settlement"], "unsettled");

    // The parking is visible in the trace the horizon was folded from.
    let events = case_events(root.path(), &case_id);
    assert!(
        events.iter().any(|event| {
            event.plan_item_id.as_deref() == Some("signoff_release")
                && event.kind == TraceKind::ItemActivated
                && event.payload["human_task"] == true
        }),
        "signoff_release must be parked as a human task: {events:?}"
    );

    // And the case itself stays open awaiting that decision.
    let overview = client
        .call(json!({"verb": "case_get_overview", "case_id": case_id}))
        .await;
    assert_eq!(overview["case_state"], "active");
}

#[tokio::test]
async fn commit_without_the_operator_identity_is_refused_with_no_case() {
    // The seed's identity binding is load-bearing: a protected verb whose
    // actor block does not match it must be refused before any side effect.
    let (root, socket) = boot().await;
    let mut client = Client::connect(&socket).await;

    let cases_before = std::fs::read_dir(root.path().join("cases"))
        .map(|d| d.count())
        .unwrap_or(0);

    let response = client
        .call(json!({
            "verb": "case_commit",
            "actor": {"actor_id": "somebody_else", "role": "operator"},
            "template_ref": SENTRY_CHAIN_REF,
            "params": sentry_chain_params(),
            "policy": "authority/active-policy.json",
            "entity": "somebody_else",
            "process": "case_templates_live",
        }))
        .await;

    assert_eq!(response["error_class"], "identity_not_bound");
    assert_eq!(response["no_side_effect"], true);

    let cases_after = std::fs::read_dir(root.path().join("cases"))
        .map(|d| d.count())
        .unwrap_or(0);
    assert_eq!(cases_before, cases_after, "refusal must create no case");
}
