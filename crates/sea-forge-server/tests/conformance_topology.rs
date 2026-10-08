//! Task 17 (M14/M15 audit remediation) — built-in topology templates must
//! dispatch against real, registered endpoints and settle from real episode
//! results end to end through the server, not just via sentry-only unit
//! tests. See `.agents/plans/2026-07-22-spec-audit-remediation.md` Task 17.

use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind};
use sea_forge_core::types::{CasePlan, ItemKind, SettlementCriteria};
use sea_forge_planner::templates::{
    concurrent_agents_template, instantiate, sequential_agents_template,
};
use sea_forge_server::{handle_request, Request, ServerConfig, ServerState};
use std::{
    collections::BTreeMap,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

fn endpoint(port: u16) -> AgentEndpointConfig {
    AgentEndpointConfig {
        id: "topology-test".into(),
        kind: ProviderKind::OpenAiCompatible,
        base_url: Some(format!("http://127.0.0.1:{port}/")),
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

fn policy(root: &Path) -> PathBuf {
    let path = root.join("policy.yaml");
    fs::write(
        &path,
        "version: \"0.1\"\npolicy_surfaces:\n  external_api:\n    mode: deny-by-default\n    allow_hosts: [127.0.0.1]\nrules:\n  - name: allow-agent-task\n    verdict: allow\n    actor_role: operator\n    operation_kind: agent_task\n",
    )
    .unwrap();
    path
}

fn config(root: &Path, endpoint: AgentEndpointConfig) -> ServerConfig {
    ServerConfig {
        root: root.to_path_buf(),
        agent: AgentConfig {
            endpoints: vec![endpoint],
            ..AgentConfig::default()
        },
        ..ServerConfig::default()
    }
}

/// Accepts and answers every connection with the same canned response,
/// until aborted — the sequential/concurrent topologies dispatch multiple
/// agent-task episodes against the one stub endpoint.
fn stub_always(response: &'static str) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = listener.local_addr().unwrap();
    let listener = TcpListener::from_std(listener).unwrap();
    let task = tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let mut request = vec![0_u8; 32_768];
                let _ = stream.read(&mut request).await;
                let body = response.as_bytes();
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes()).await;
                let _ = stream.write_all(body).await;
            });
        }
    });
    (address, task)
}

fn submit_plan(root: &Path, policy_path: &Path, plan: &CasePlan) -> Request {
    let plan_path = root.join("plan.json");
    fs::write(&plan_path, serde_json::to_vec(plan).unwrap()).unwrap();
    serde_json::from_str(&format!(
        // F-16: cell-relative spellings.
        r#"{{"verb":"submit","plan":"plan.json","policy":{:?},"entity":"operator_local","process":"test"}}"#,
        policy_path.strip_prefix(root).unwrap().to_str().unwrap(),
    ))
    .unwrap()
}

fn settlement_events(root: &Path, case_id: &str) -> Vec<serde_json::Value> {
    let ledger =
        sea_forge_ledger::LedgerStream::open(root, format!("case-{case_id}"), "test").unwrap();
    ledger
        .read_entries()
        .unwrap()
        .into_iter()
        .filter(|entry| entry.record_kind == "settlement_event")
        .map(|entry| entry.payload)
        .collect()
}

fn case_events(root: &Path, case_id: &str) -> Vec<serde_json::Value> {
    let ledger =
        sea_forge_ledger::LedgerStream::open(root, format!("case-{case_id}"), "test").unwrap();
    ledger
        .read_entries()
        .unwrap()
        .into_iter()
        .filter(|entry| entry.record_kind == "case_event")
        .map(|entry| entry.payload)
        .collect()
}

#[tokio::test]
async fn topology_sequential_dispatches_through_the_server_in_order() {
    let (address, task) = stub_always(
        r#"{"choices":[{"message":{"content":"step complete"}}],"usage":{"total_tokens":1}}"#,
    );
    let root = tempfile::tempdir().unwrap();
    let policy_path = policy(root.path());
    let ep = endpoint(address.port());

    let tmpl = sequential_agents_template("topology-test").unwrap();
    let plan = instantiate(&tmpl, &BTreeMap::new(), "case_x", "run_x", "intent_x").unwrap();
    assert!(
        plan.items.iter().all(|item| item.markers.required),
        "every generated topology step must be required so it actually dispatches"
    );

    let state = Arc::new(ServerState::new(config(root.path(), ep)).unwrap());
    let request = submit_plan(root.path(), &policy_path, &plan);
    let response = handle_request(request, &state).await;
    assert_eq!(response["state"], "completed", "response was: {response:?}");
    assert_eq!(response["exit_code"], 0);
    let case_id = response["case_id"].as_str().unwrap().to_string();

    let settlements = settlement_events(root.path(), &case_id);
    assert_eq!(
        settlements.len(),
        3,
        "all three sequential steps must dispatch and settle"
    );
    assert!(
        settlements.iter().all(|s| s["status"] == "accepted"),
        "settlements: {settlements:?}"
    );

    // The step order in the case_events activation log must be step_1 → step_2 → step_3.
    let activation_order: Vec<String> = case_events(root.path(), &case_id)
        .iter()
        .filter(|event| event["kind"] == "item_activated")
        .filter_map(|event| event["plan_item_id"].as_str().map(str::to_owned))
        .collect();
    assert_eq!(activation_order, vec!["step_1", "step_2", "step_3"]);

    task.abort();
}

#[tokio::test]
async fn topology_concurrent_all_branches_accept_and_rollup_fires() {
    let (address, task) = stub_always(
        r#"{"choices":[{"message":{"content":"branch complete"}}],"usage":{"total_tokens":1}}"#,
    );
    let root = tempfile::tempdir().unwrap();
    let policy_path = policy(root.path());
    let ep = endpoint(address.port());

    let tmpl = concurrent_agents_template("topology-test").unwrap();
    let plan = instantiate(&tmpl, &BTreeMap::new(), "case_x", "run_x", "intent_x").unwrap();
    let branch_count = plan
        .items
        .iter()
        .filter(|item| item.item_kind == ItemKind::AgentTask)
        .count();
    assert!(branch_count >= 2);

    let state = Arc::new(ServerState::new(config(root.path(), ep)).unwrap());
    let request = submit_plan(root.path(), &policy_path, &plan);
    let response = handle_request(request, &state).await;
    assert_eq!(response["state"], "completed", "response was: {response:?}");
    let case_id = response["case_id"].as_str().unwrap().to_string();

    let settlements = settlement_events(root.path(), &case_id);
    assert_eq!(settlements.len(), branch_count);
    assert!(settlements.iter().all(|s| s["status"] == "accepted"));

    let rollup_achieved = case_events(root.path(), &case_id).into_iter().any(|event| {
        event["kind"] == "milestone_achieved" && event["plan_item_id"] == "ms_all_branches_accepted"
    });
    assert!(
        rollup_achieved,
        "rollup milestone must fire once every real branch settles accepted"
    );

    task.abort();
}

#[tokio::test]
async fn topology_concurrent_one_branch_rejected_rollup_never_fires_and_case_terminates() {
    let (address, task) = stub_always(
        r#"{"choices":[{"message":{"content":"branch complete"}}],"usage":{"total_tokens":1}}"#,
    );
    let root = tempfile::tempdir().unwrap();
    let policy_path = policy(root.path());
    let ep = endpoint(address.port());

    let tmpl = concurrent_agents_template("topology-test").unwrap();
    let mut plan = instantiate(&tmpl, &BTreeMap::new(), "case_x", "run_x", "intent_x").unwrap();
    // Make branch_a's settlement criterion unsatisfiable by the stub's
    // canned response — its real episode output is accepted by the
    // endpoint but rejected at settlement, exercising the "rejected
    // branch" path with a real dispatched episode, not a synthetic one.
    let branch_a = plan
        .items
        .iter_mut()
        .find(|item| item.plan_item_id == "branch_a")
        .expect("concurrent_agents must produce branch_a");
    branch_a.settlement_criteria = SettlementCriteria {
        agent_output_must_contain: Some("this string never appears in the stub response".into()),
        ..SettlementCriteria::default()
    };

    let state = Arc::new(ServerState::new(config(root.path(), ep)).unwrap());
    let request = submit_plan(root.path(), &policy_path, &plan);
    let response = handle_request(request, &state).await;
    assert_eq!(
        response["state"], "terminated",
        "a required branch's rejection must terminate the case, response was: {response:?}"
    );
    assert_eq!(response["exit_code"], 3);
    let case_id = response["case_id"].as_str().unwrap().to_string();

    let settlements = settlement_events(root.path(), &case_id);
    assert!(
        settlements.iter().any(|s| s["status"] == "rejected"),
        "branch_a's real episode must settle rejected: {settlements:?}"
    );

    let rollup_achieved = case_events(root.path(), &case_id).into_iter().any(|event| {
        event["kind"] == "milestone_achieved" && event["plan_item_id"] == "ms_all_branches_accepted"
    });
    assert!(
        !rollup_achieved,
        "the rollup must never fire when a required branch is rejected"
    );

    let terminated = case_events(root.path(), &case_id)
        .into_iter()
        .find(|event| event["kind"] == "case_terminated")
        .expect("a case_terminated event must be recorded");
    assert_eq!(terminated["payload"]["blocking_item"], "branch_a");

    task.abort();
}
