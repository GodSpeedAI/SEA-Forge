use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind};
use sea_forge_core::types::{
    CasePlan, ItemKind, Operation, PlanItem, SettlementCriteria, TraceEvent, TraceKind,
};
use sea_forge_server::{
    agent_probe, delegation, handle_request, Request, ServerConfig, ServerState,
};
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use zeroize::Zeroizing;

struct CountingResolver {
    reads: Arc<AtomicUsize>,
}

impl agent_probe::CredentialResolver for CountingResolver {
    fn resolve(&self, _reference: &str) -> Result<Zeroizing<String>, sea_forge_core::ForgeError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        Ok(Zeroizing::new("test-secret".into()))
    }
}

fn endpoint(port: u16) -> AgentEndpointConfig {
    AgentEndpointConfig {
        id: "local-test".into(),
        kind: ProviderKind::OpenAiCompatible,
        base_url: Some(format!("http://127.0.0.1:{port}/")),
        credential_ref: Some("TEST_CREDENTIAL".into()),
        default_model: Some("test-model".into()),
        allow_loopback_test: true,
        max_request_bytes: 16_384,
        max_response_bytes: 16_384,
        timeout_secs: 5,
        status: None,
    }
}

fn policy(root: &Path, allow_task: bool, allow_secret: bool) -> PathBuf {
    let mut rules = String::new();
    if allow_task {
        rules.push_str(
            "  - name: allow-agent-task\n    verdict: allow\n    actor_role: operator\n    operation_kind: agent_task\n",
        );
    }
    if allow_secret {
        rules.push_str(
            "  - name: allow-secret-access\n    verdict: allow\n    actor_role: operator\n    operation_kind: secret_access\n",
        );
    }
    let path = root.join("policy.yaml");
    fs::write(
        &path,
        format!(
            "version: \"0.1\"\npolicy_surfaces:\n  external_api:\n    mode: deny-by-default\n    allow_hosts: [127.0.0.1]\nrules:\n{rules}"
        ),
    )
    .unwrap();
    path
}

async fn stub(
    response: &'static str,
) -> (SocketAddr, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let connections = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&connections);
    let task = tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            count.fetch_add(1, Ordering::SeqCst);
            let mut request = vec![0_u8; 32_768];
            let _ = stream.read(&mut request).await;
            let body = response.as_bytes();
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(header.as_bytes()).await.unwrap();
            stream.write_all(body).await.unwrap();
        }
    });
    (address, connections, task)
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

fn resolver() -> (CountingResolver, Arc<AtomicUsize>) {
    let reads = Arc::new(AtomicUsize::new(0));
    (
        CountingResolver {
            reads: Arc::clone(&reads),
        },
        reads,
    )
}

#[tokio::test]
async fn t13_delegation_completes_with_transcript_evidence() {
    let (address, _connections, task) = stub(
        r#"{"choices":[{"message":{"content":"Task completed successfully"}}],"usage":{"total_tokens":42}}"#,
    )
    .await;
    let root = tempfile::tempdir().unwrap();
    let endpoint = endpoint(address.port());
    let policy = policy(root.path(), true, true);
    let (res, reads) = resolver();
    let outcome = delegation::execute(
        &config(root.path(), endpoint),
        delegation::DelegationRequest {
            endpoint_id: "local-test",
            instruction: "Summarize the project status",
            model: None,
            max_turns: 3,
            token_budget: None,
            criteria: SettlementCriteria::default(),
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
        },
        &res,
    )
    .await
    .unwrap();

    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Accepted
    );
    assert_eq!(outcome.termination.as_deref(), Some("completed"));
    assert_eq!(outcome.turns_used, 1);
    assert!(outcome.transcript_sha256.is_some());
    assert_eq!(reads.load(Ordering::SeqCst), 1);

    let run = root.path().join("runs").join(&outcome.run_id);
    for file in [
        "intent.json",
        "plan.json",
        "authority.json",
        "transcript-evidence.json",
        "settlement.json",
    ] {
        assert!(run.join(file).is_file(), "missing {file}");
    }

    let evidence = fs::read_to_string(run.join("transcript-evidence.json")).unwrap();
    assert!(
        !evidence.contains("test-secret"),
        "credential leaked in transcript evidence"
    );

    let _ = task.await;
}

#[tokio::test]
async fn t13_delegation_denied_without_agent_task_policy() {
    // ponytail: no stub needed — denial happens before any network I/O.
    let root = tempfile::tempdir().unwrap();
    let policy = policy(root.path(), false, true);
    let (res, reads) = resolver();
    let outcome = delegation::execute(
        &config(root.path(), endpoint(1)),
        delegation::DelegationRequest {
            endpoint_id: "local-test",
            instruction: "test",
            model: None,
            max_turns: 1,
            token_budget: None,
            criteria: SettlementCriteria::default(),
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
        },
        &res,
    )
    .await
    .unwrap();

    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Rejected
    );
    assert_eq!(outcome.error_class.as_deref(), Some("authority_denied"));
    assert_eq!(reads.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn t13_delegation_denied_secret_access_no_credential_read() {
    let root = tempfile::tempdir().unwrap();
    let policy = policy(root.path(), true, false);
    let (res, reads) = resolver();
    let outcome = delegation::execute(
        &config(root.path(), endpoint(1)),
        delegation::DelegationRequest {
            endpoint_id: "local-test",
            instruction: "test",
            model: None,
            max_turns: 1,
            token_budget: None,
            criteria: SettlementCriteria::default(),
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
        },
        &res,
    )
    .await
    .unwrap();

    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Rejected
    );
    assert_eq!(outcome.error_class.as_deref(), Some("secret_access_denied"));
    assert_eq!(reads.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn t13_endpoint_error_settles_rejected_with_transcript() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let body = b"Service Unavailable";
            let header = format!(
                "HTTP/1.1 503 Service Unavailable\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(header.as_bytes()).await.unwrap();
            stream.write_all(body).await.unwrap();
        }
    });

    let root = tempfile::tempdir().unwrap();
    let policy = policy(root.path(), true, true);
    let (res, _reads) = resolver();
    let outcome = delegation::execute(
        &config(root.path(), endpoint(address.port())),
        delegation::DelegationRequest {
            endpoint_id: "local-test",
            instruction: "test",
            model: None,
            max_turns: 1,
            token_budget: None,
            criteria: SettlementCriteria::default(),
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
        },
        &res,
    )
    .await
    .unwrap();

    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Rejected
    );
    assert_eq!(outcome.termination.as_deref(), Some("endpoint_error"));
    assert_eq!(outcome.error_class.as_deref(), Some("http_5xx"));
    let run = root.path().join("runs").join(&outcome.run_id);
    assert!(run.join("transcript-evidence.json").is_file());
    let _ = task.await;
}

#[tokio::test]
async fn t13_token_budget_breach_settles_turn_cap_exceeded() {
    // The stub reports 42 tokens per call. With a budget of 25, the
    // delegation must terminate immediately as turn_cap_exceeded.
    let (address, _connections, task) =
        stub(r#"{"choices":[{"message":{"content":"response"}}],"usage":{"total_tokens":42}}"#)
            .await;
    let root = tempfile::tempdir().unwrap();
    let policy = policy(root.path(), true, true);
    let (res, _reads) = resolver();
    let outcome = delegation::execute(
        &config(root.path(), endpoint(address.port())),
        delegation::DelegationRequest {
            endpoint_id: "local-test",
            instruction: "test",
            model: None,
            max_turns: 5,
            token_budget: Some(25),
            criteria: SettlementCriteria::default(),
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
        },
        &res,
    )
    .await
    .unwrap();

    assert_eq!(outcome.termination.as_deref(), Some("turn_cap_exceeded"));
    assert_eq!(
        outcome.error_class.as_deref(),
        Some("token_budget_exceeded")
    );
    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Rejected
    );
    // Transcript evidence still committed despite budget breach.
    let run = root.path().join("runs").join(&outcome.run_id);
    assert!(run.join("transcript-evidence.json").is_file());
    let _ = task.await;
}

#[tokio::test]
async fn t13_agent_success_with_failed_output_criteria_settles_rejected() {
    let (address, _connections, task) = stub(
        r#"{"choices":[{"message":{"content":"Task completed successfully"}}],"usage":{"total_tokens":42}}"#,
    )
    .await;
    let root = tempfile::tempdir().unwrap();
    let policy = policy(root.path(), true, true);
    let (res, _reads) = resolver();
    let outcome = delegation::execute(
        &config(root.path(), endpoint(address.port())),
        delegation::DelegationRequest {
            endpoint_id: "local-test",
            instruction: "publish proof",
            model: None,
            max_turns: 1,
            token_budget: None,
            criteria: SettlementCriteria {
                agent_output_must_contain: Some("proof artifact committed".into()),
                ..SettlementCriteria::default()
            },
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
        },
        &res,
    )
    .await
    .unwrap();

    assert_eq!(outcome.termination.as_deref(), Some("completed"));
    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Rejected
    );
    assert_eq!(
        outcome.error_class.as_deref(),
        Some("agent_output_mismatch")
    );
    assert!(root
        .path()
        .join("runs")
        .join(&outcome.run_id)
        .join("transcript-evidence.json")
        .is_file());
    let _ = task.await;
}

#[tokio::test]
async fn t13_transcript_artifact_hash_verifies() {
    let (address, _connections, task) = stub(
        r#"{"choices":[{"message":{"content":"artifact: accepted"}}],"usage":{"total_tokens":42}}"#,
    )
    .await;
    let root = tempfile::tempdir().unwrap();
    let policy = policy(root.path(), true, true);
    let (res, _reads) = resolver();
    let outcome = delegation::execute(
        &config(root.path(), endpoint(address.port())),
        delegation::DelegationRequest {
            endpoint_id: "local-test",
            instruction: "publish proof",
            model: None,
            max_turns: 1,
            token_budget: None,
            criteria: SettlementCriteria::default(),
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
        },
        &res,
    )
    .await
    .unwrap();

    let evidence_path = root
        .path()
        .join("runs")
        .join(&outcome.run_id)
        .join("transcript-evidence.json");
    let evidence: sea_forge_core::types::TranscriptEvidence =
        serde_json::from_str(&fs::read_to_string(&evidence_path).unwrap()).unwrap();
    let artifact_ref = evidence
        .artifact_ref
        .as_deref()
        .expect("full-mode artifact_ref present");
    let artifact_path = root.path().join(artifact_ref);
    assert!(artifact_path.is_file());

    let content = fs::read_to_string(&artifact_path).unwrap();
    let entries: Vec<sea_forge_agent::TranscriptEntry> = content
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        sea_forge_agent::transcript_sha256(&entries),
        evidence.transcript_sha256
    );
    let _ = task.await;
}

#[tokio::test]
async fn planned_agent_episode_uses_case_context() {
    let (address, _connections, task) = stub(
        r#"{"choices":[{"message":{"content":"planned episode complete"}}],"usage":{"total_tokens":1}}"#,
    )
    .await;
    let root = tempfile::tempdir().unwrap();
    let policy = policy(root.path(), true, true);
    let (res, _reads) = resolver();
    let submitted_case_id = sea_forge_core::ids::case_id().unwrap();
    let dispatched_run_id = sea_forge_core::ids::run_id().unwrap();

    delegation::execute_with_control(
        &config(root.path(), endpoint(address.port())),
        delegation::DelegationRequest {
            endpoint_id: "local-test",
            instruction: "complete the planned episode",
            model: None,
            max_turns: 1,
            token_budget: None,
            criteria: SettlementCriteria::default(),
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
        },
        &res,
        delegation::DelegationEpisodeContext::planned(
            &submitted_case_id,
            "agent",
            &dispatched_run_id,
        ),
        || false,
    )
    .await
    .unwrap();

    let ledger = sea_forge_ledger::LedgerStream::open(
        root.path(),
        format!("case-{submitted_case_id}"),
        "test",
    )
    .unwrap();
    let entries = ledger.read_entries().unwrap();
    let settlement_payload = &entries
        .iter()
        .find(|entry| entry.record_kind == "settlement")
        .unwrap()
        .payload;
    assert_eq!(settlement_payload["case_id"], submitted_case_id);
    assert_eq!(settlement_payload["item_id"], "agent");
    assert_eq!(settlement_payload["run_id"], dispatched_run_id);
    let evidence_payload = &entries
        .iter()
        .find(|entry| {
            entry.record_kind == "agent_task_evidence"
                && entry.payload["run_id"] == dispatched_run_id
        })
        .unwrap()
        .payload;
    assert_eq!(evidence_payload["case_id"], submitted_case_id);
    assert_eq!(evidence_payload["item_id"], "agent");
    assert_eq!(evidence_payload["run_id"], dispatched_run_id);

    let _ = task.await;
}

#[tokio::test]
async fn planned_agent_rejection_uses_case_context_once() {
    let root = tempfile::tempdir().unwrap();
    let policy = policy(root.path(), false, true);
    let (res, _reads) = resolver();
    let submitted_case_id = sea_forge_core::ids::case_id().unwrap();
    let dispatched_run_id = sea_forge_core::ids::run_id().unwrap();

    delegation::execute_with_control(
        &config(root.path(), endpoint(1)),
        delegation::DelegationRequest {
            endpoint_id: "local-test",
            instruction: "reject the planned episode",
            model: None,
            max_turns: 1,
            token_budget: None,
            criteria: SettlementCriteria::default(),
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
        },
        &res,
        delegation::DelegationEpisodeContext::planned(
            &submitted_case_id,
            "agent",
            &dispatched_run_id,
        ),
        || false,
    )
    .await
    .unwrap();

    let ledger = sea_forge_ledger::LedgerStream::open(
        root.path(),
        format!("case-{submitted_case_id}"),
        "test",
    )
    .unwrap();
    let entries = ledger.read_entries().unwrap();
    let settlements: Vec<_> = entries
        .iter()
        .filter(|entry| {
            entry.record_kind == "settlement" && entry.payload["run_id"] == dispatched_run_id
        })
        .collect();
    assert_eq!(
        settlements.len(),
        1,
        "one terminal settlement for the episode"
    );
    let [settlement] = settlements.as_slice() else {
        panic!("expected one terminal settlement");
    };
    assert_eq!(settlement.payload["case_id"], submitted_case_id);
    assert_eq!(settlement.payload["item_id"], "agent");
    assert_eq!(settlement.payload["run_id"], dispatched_run_id);
    assert_eq!(settlement.payload["status"], "rejected");
    let evidence_payload = &entries
        .iter()
        .find(|entry| {
            entry.record_kind == "agent_task_evidence"
                && entry.payload["run_id"] == dispatched_run_id
        })
        .unwrap()
        .payload;
    assert_eq!(evidence_payload["case_id"], submitted_case_id);
    assert_eq!(evidence_payload["item_id"], "agent");
    assert_eq!(evidence_payload["run_id"], dispatched_run_id);
}

/// T13.2: the server semaphore respects one shared cap. With
/// `max_concurrent_runs=2` and 4 concurrent delegation requests, the stub
/// must never observe more than 2 in-flight connections.
#[tokio::test]
async fn t13_2_server_semaphore_caps_concurrent_delegations() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let current = Arc::new(AtomicUsize::new(0));
    let max_seen = Arc::new(AtomicUsize::new(0));
    let cur = Arc::clone(&current);
    let max = Arc::clone(&max_seen);
    let stub_task = tokio::spawn(async move {
        let mut handlers = vec![];
        for _ in 0..4 {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            let cur = Arc::clone(&cur);
            let max = Arc::clone(&max);
            handlers.push(tokio::spawn(async move {
                let c = cur.fetch_add(1, Ordering::SeqCst) + 1;
                max.fetch_max(c, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(80)).await;
                let mut req = vec![0_u8; 4096];
                let _ = stream.read(&mut req).await;
                let body =
                    br#"{"choices":[{"message":{"content":"ok"}}],"usage":{"total_tokens":1}}"#;
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes()).await;
                let _ = stream.write_all(body).await;
                cur.fetch_sub(1, Ordering::SeqCst);
            }));
        }
        for h in handlers {
            let _ = h.await;
        }
    });

    let root = tempfile::tempdir().unwrap();
    let policy = policy(root.path(), true, false);
    let mut ep = endpoint(address.port());
    ep.credential_ref = None;
    let config = ServerConfig {
        root: root.path().to_path_buf(),
        max_concurrent_runs: 2,
        agent: AgentConfig {
            endpoints: vec![ep],
            ..AgentConfig::default()
        },
        ..ServerConfig::default()
    };
    let state = Arc::new(ServerState::new(config).unwrap());

    let mut handles = vec![];
    for _ in 0..4 {
        let state = Arc::clone(&state);
        let policy = policy.to_string_lossy().into_owned();
        handles.push(tokio::spawn(async move {
            sea_forge_server::handle_request(
                Request::Delegate {
                    endpoint: "local-test".into(),
                    instruction: "test".into(),
                    run_id: None,
                    model: None,
                    max_turns: 1,
                    token_budget: None,
                    criteria: SettlementCriteria::default(),
                    policy,
                    entity: "operator_local".into(),
                    process: "test".into(),
                },
                &state,
            )
            .await
        }));
    }
    for handle in handles {
        let _ = handle.await;
    }
    let _ = stub_task.await;

    let observed = max_seen.load(Ordering::SeqCst);
    assert!(
        observed <= 2,
        "semaphore exceeded cap: observed {observed} concurrent connections"
    );
    assert!(
        observed >= 2,
        "semaphore never reached cap: observed {observed} (test may need more delay)"
    );
}

/// T13.3: cancel one of three concurrent delegations; siblings settle
/// normally. The cancelled run settles rejected with termination
/// "cancelled"; the other two complete.
#[tokio::test]
async fn t13_3_cancel_one_of_three_siblings_settle_normally() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let stub_task = tokio::spawn(async move {
        let mut handlers = vec![];
        for _ in 0..3 {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            handlers.push(tokio::spawn(async move {
                // Delay so cancellation can arrive mid-flight.
                tokio::time::sleep(Duration::from_millis(200)).await;
                let mut req = vec![0_u8; 4096];
                let _ = stream.read(&mut req).await;
                let body =
                    br#"{"choices":[{"message":{"content":"ok"}}],"usage":{"total_tokens":1}}"#;
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes()).await;
                let _ = stream.write_all(body).await;
            }));
        }
        for h in handlers {
            let _ = h.await;
        }
    });

    let root = tempfile::tempdir().unwrap();
    // Policy includes run_cancel so the cancellation authority succeeds.
    let policy_path = root.path().join("policy.yaml");
    fs::write(
        &policy_path,
        "version: \"0.1\"\n\
         policy_surfaces:\n\
         \x20 external_api:\n\
         \x20   mode: deny-by-default\n\
         \x20   allow_hosts: [127.0.0.1]\n\
         rules:\n\
         \x20 - name: allow-agent-task\n\
         \x20   verdict: allow\n\
         \x20   actor_role: operator\n\
         \x20   operation_kind: agent_task\n\
         \x20 - name: allow-cancel\n\
         \x20   verdict: allow\n\
         \x20   actor_role: operator\n\
         \x20   operation_kind: run_cancel\n",
    )
    .unwrap();

    let mut ep = endpoint(address.port());
    ep.credential_ref = None;
    let config = ServerConfig {
        root: root.path().to_path_buf(),
        max_concurrent_runs: 3,
        agent: AgentConfig {
            endpoints: vec![ep],
            ..AgentConfig::default()
        },
        ..ServerConfig::default()
    };
    let state = Arc::new(ServerState::new(config).unwrap());

    let run_a = sea_forge_core::ids::run_id().unwrap();
    let run_b = sea_forge_core::ids::run_id().unwrap();
    let run_c = sea_forge_core::ids::run_id().unwrap();
    let policy_str = policy_path.to_string_lossy().into_owned();

    let mk_delegate = |rid: String| {
        let state = Arc::clone(&state);
        let policy = policy_str.clone();
        tokio::spawn(async move {
            sea_forge_server::handle_request(
                Request::Delegate {
                    endpoint: "local-test".into(),
                    instruction: "test".into(),
                    run_id: Some(rid),
                    model: None,
                    max_turns: 1,
                    token_budget: None,
                    criteria: SettlementCriteria::default(),
                    policy,
                    entity: "operator_local".into(),
                    process: "test".into(),
                },
                &state,
            )
            .await
        })
    };

    let task_a = mk_delegate(run_a.clone());
    let task_b = mk_delegate(run_b.clone());
    let task_c = mk_delegate(run_c.clone());

    // Give the delegations time to register before cancelling.
    tokio::time::sleep(Duration::from_millis(50)).await;

    let cancel_result = sea_forge_server::handle_request(
        Request::CancelDelegation {
            run_id: run_b.clone(),
            policy: policy_str.clone(),
            entity: "operator_local".into(),
            process: "test".into(),
        },
        &state,
    )
    .await;
    assert_eq!(cancel_result["state"], "cancellation_requested");

    let res_a = task_a.await.unwrap();
    let res_b = task_b.await.unwrap();
    let res_c = task_c.await.unwrap();
    let _ = stub_task.await;

    assert_eq!(res_a["termination"], "completed");
    assert_eq!(res_a["settlement"], "accepted");
    assert_eq!(res_c["termination"], "completed");
    assert_eq!(res_c["settlement"], "accepted");
    assert_eq!(res_b["termination"], "cancelled");
    assert_eq!(res_b["settlement"], "rejected");
}

/// T13.2: `submit` dispatches each ready episode under the server's one shared
/// concurrency cap, then derives further work only from completed episodes.
#[tokio::test]
async fn t13_2_mixed_episodes_share_server_cap() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let current = Arc::new(AtomicUsize::new(0));
    let max_seen = Arc::new(AtomicUsize::new(0));
    let current_for_stub = Arc::clone(&current);
    let max_for_stub = Arc::clone(&max_seen);
    let stub_task = tokio::spawn(async move {
        let mut handlers = vec![];
        for _ in 0..3 {
            let (mut stream, _) = listener.accept().await.unwrap();
            let current = Arc::clone(&current_for_stub);
            let max_seen = Arc::clone(&max_for_stub);
            handlers.push(tokio::spawn(async move {
                let in_flight = current.fetch_add(1, Ordering::SeqCst) + 1;
                max_seen.fetch_max(in_flight, Ordering::SeqCst);
                let mut request = vec![0_u8; 4096];
                let _ = stream.read(&mut request).await;
                tokio::time::sleep(Duration::from_millis(75)).await;
                let body =
                    br#"{"choices":[{"message":{"content":"ok"}}],"usage":{"total_tokens":1}}"#;
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                stream.write_all(header.as_bytes()).await.unwrap();
                stream.write_all(body).await.unwrap();
                current.fetch_sub(1, Ordering::SeqCst);
            }));
        }
        for handler in handlers {
            handler.await.unwrap();
        }
    });

    let root = tempfile::tempdir().unwrap();
    let policy_path = root.path().join("policy.yaml");
    fs::write(
        &policy_path,
        "version: \"0.1\"\n\
         policy_surfaces:\n\
         \x20 external_api:\n\
         \x20   mode: deny-by-default\n\
         \x20   allow_hosts: [127.0.0.1]\n\
         rules:\n\
         \x20 - name: allow-agent-task\n\
         \x20   verdict: allow\n\
         \x20   actor_role: operator\n\
         \x20   operation_kind: agent_task\n\
         \x20 - name: allow-command\n\
         \x20   verdict: allow\n\
         \x20   actor_role: operator\n\
         \x20   operation_kind: execute_command\n\
         \x20   argv0: sh\n\
         \x20   sandbox_class: jail\n",
    )
    .unwrap();
    let mut agent_endpoint = endpoint(address.port());
    agent_endpoint.credential_ref = None;
    let state = Arc::new(
        ServerState::new(ServerConfig {
            root: root.path().to_path_buf(),
            max_concurrent_runs: 2,
            agent: AgentConfig {
                endpoints: vec![agent_endpoint],
                ..AgentConfig::default()
            },
            ..ServerConfig::default()
        })
        .unwrap(),
    );
    let plan_path = root.path().join("mixed-plan.json");
    fs::write(
        &plan_path,
        serde_json::to_vec_pretty(&CasePlan {
            version: "0.2".into(),
            plan_id: "plan_mixed".into(),
            case_id: "case_placeholder".into(),
            run_id: "run_placeholder".into(),
            intent_id: "int_mixed".into(),
            items: (0..5)
                .map(|index| PlanItem {
                    plan_item_id: format!("item_{index}"),
                    name: format!("item_{index}"),
                    operations: if index % 2 == 0 {
                        vec![Operation::AgentTask {
                            endpoint_ref: "local-test".into(),
                            instruction: "complete".into(),
                            max_turns: 1,
                            token_budget: None,
                            response_schema: None,
                            transcript_retention: None,
                        }]
                    } else {
                        vec![Operation::ExecuteCommand {
                            argv: vec!["sh".into(), "-c".into(), "sleep 0.075".into()],
                            cwd: ".".into(),
                        }]
                    },
                    entry_criteria: vec![],
                    exit_criteria: vec![],
                    settlement_criteria: SettlementCriteria::default(),
                    settlement_criteria_ref: None,
                    item_kind: if index % 2 == 0 {
                        ItemKind::AgentTask
                    } else {
                        ItemKind::SandboxedTask
                    },
                    sandbox_class: None,
                    parent_stage: None,
                    markers: sea_forge_core::types::ItemMarkers {
                        required: index % 2 == 0,
                        ..Default::default()
                    },
                    max_instances: 1,
                    depends_on: vec![],
                    environment: None,
                })
                .collect(),
            template_ref: None,
            job_contract_ref: None,
        })
        .unwrap(),
    )
    .unwrap();

    let request: Request = serde_json::from_value(serde_json::json!({
        "verb": "submit",
        "plan": plan_path,
        "policy": policy_path,
        "entity": "operator_local",
        "process": "test",
        "timeout": 5,
    }))
    .unwrap();
    let response = handle_request(request, &state).await;

    assert_eq!(response["state"], "completed", "{response}");
    let case_id = response["case_id"].as_str().unwrap();
    let events: Vec<TraceEvent> = fs::read_to_string(
        root.path()
            .join("cases")
            .join(case_id)
            .join("case-events.jsonl"),
    )
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
    let dispatches: Vec<_> = events
        .iter()
        .filter(|event| event.kind == TraceKind::ItemActivated)
        .collect();
    let settlements: Vec<_> = events
        .iter()
        .filter(|event| event.kind == TraceKind::SettlementRecorded)
        .collect();
    assert_eq!(dispatches.len(), 5);
    assert_eq!(settlements.len(), 5);
    let mut unsettled = 0_u32;
    let mut max_unsettled = 0_u32;
    for event in &events {
        match event.kind {
            TraceKind::ItemActivated => {
                unsettled += 1;
                max_unsettled = max_unsettled.max(unsettled);
            }
            TraceKind::SettlementRecorded => unsettled -= 1,
            _ => {}
        }
    }
    assert!(
        max_unsettled <= 2,
        "observed {max_unsettled} unsettled dispatches"
    );
    assert!(dispatches
        .iter()
        .any(|event| event.payload["episode_kind"] == "sandboxed_task"));
    assert!(dispatches
        .iter()
        .any(|event| event.payload["episode_kind"] == "agent_task"));
    assert!(max_seen.load(Ordering::SeqCst) <= 2);
    stub_task.await.unwrap();
}

#[tokio::test]
async fn t13_2_submit_waits_for_direct_delegate_permit() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let connected = Arc::new(AtomicUsize::new(0));
    let connected_for_stub = Arc::clone(&connected);
    let stub = tokio::spawn(async move {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().await.unwrap();
            connected_for_stub.fetch_add(1, Ordering::SeqCst);
            let mut request = vec![0_u8; 4096];
            let _ = stream.read(&mut request).await;
            tokio::time::sleep(Duration::from_millis(100)).await;
            let body = br#"{"choices":[{"message":{"content":"ok"}}],"usage":{"total_tokens":1}}"#;
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(header.as_bytes()).await.unwrap();
            stream.write_all(body).await.unwrap();
        }
    });
    let root = tempfile::tempdir().unwrap();
    let policy_path = policy(root.path(), true, false);
    let mut agent_endpoint = endpoint(address.port());
    agent_endpoint.credential_ref = None;
    let state = Arc::new(
        ServerState::new(ServerConfig {
            root: root.path().to_path_buf(),
            max_concurrent_runs: 1,
            agent: AgentConfig {
                endpoints: vec![agent_endpoint],
                ..AgentConfig::default()
            },
            ..ServerConfig::default()
        })
        .unwrap(),
    );
    let direct_state = Arc::clone(&state);
    let direct_policy = policy_path.to_string_lossy().into_owned();
    let direct = tokio::spawn(async move {
        handle_request(
            Request::Delegate {
                endpoint: "local-test".into(),
                instruction: "hold permit".into(),
                run_id: None,
                model: None,
                max_turns: 1,
                token_budget: None,
                criteria: SettlementCriteria::default(),
                policy: direct_policy,
                entity: "operator_local".into(),
                process: "test".into(),
            },
            &direct_state,
        )
        .await
    });
    tokio::time::timeout(Duration::from_secs(1), async {
        while connected.load(Ordering::SeqCst) == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let plan_path = root.path().join("one-agent.json");
    fs::write(
        &plan_path,
        serde_json::to_vec(&CasePlan {
            version: "0.2".into(),
            plan_id: "plan_wait".into(),
            case_id: "case_placeholder".into(),
            run_id: "run_placeholder".into(),
            intent_id: "int_wait".into(),
            items: vec![PlanItem {
                plan_item_id: "agent".into(),
                name: "agent".into(),
                operations: vec![Operation::AgentTask {
                    endpoint_ref: "local-test".into(),
                    instruction: "wait".into(),
                    max_turns: 1,
                    token_budget: None,
                    response_schema: None,
                    transcript_retention: None,
                }],
                entry_criteria: vec![],
                exit_criteria: vec![],
                settlement_criteria: SettlementCriteria::default(),
                settlement_criteria_ref: None,
                item_kind: ItemKind::AgentTask,
                sandbox_class: None,
                parent_stage: None,
                markers: sea_forge_core::types::ItemMarkers {
                    required: true,
                    ..Default::default()
                },
                max_instances: 1,
                depends_on: vec![],
                environment: None,
            }],
            template_ref: None,
            job_contract_ref: None,
        })
        .unwrap(),
    )
    .unwrap();
    let request: Request = serde_json::from_value(serde_json::json!({"verb":"submit", "plan":plan_path, "policy":policy_path, "entity":"operator_local", "process":"test", "timeout":5})).unwrap();
    let mut submit = tokio::spawn({
        let state = Arc::clone(&state);
        async move { handle_request(request, &state).await }
    });
    let waited = tokio::time::timeout(Duration::from_millis(50), &mut submit).await;
    if waited.is_ok() {
        stub.abort();
    }
    assert!(waited.is_err());
    assert_eq!(direct.await.unwrap()["settlement"], "accepted");
    assert_eq!(submit.await.unwrap()["state"], "completed");
    stub.await.unwrap();
}

#[tokio::test]
async fn t13_2_post_dispatch_failure_settles_and_drains_siblings() {
    let root = tempfile::tempdir().unwrap();
    let policy_path = policy(root.path(), true, false);
    let state = Arc::new(
        ServerState::new(ServerConfig {
            root: root.path().to_path_buf(),
            max_concurrent_runs: 2,
            ..ServerConfig::default()
        })
        .unwrap(),
    );
    let plan_path = root.path().join("failure-plan.json");
    fs::write(
        &plan_path,
        serde_json::to_vec(&CasePlan {
            version: "0.2".into(),
            plan_id: "plan_failure".into(),
            case_id: "case_placeholder".into(),
            run_id: "run_placeholder".into(),
            intent_id: "int_failure".into(),
            items: (0..2)
                .map(|index| PlanItem {
                    plan_item_id: format!("agent_{index}"),
                    name: "agent".into(),
                    operations: vec![Operation::AgentTask {
                        endpoint_ref: "missing".into(),
                        instruction: "fail after dispatch".into(),
                        max_turns: 1,
                        token_budget: None,
                        response_schema: None,
                        transcript_retention: None,
                    }],
                    entry_criteria: vec![],
                    exit_criteria: vec![],
                    settlement_criteria: SettlementCriteria::default(),
                    settlement_criteria_ref: None,
                    item_kind: ItemKind::AgentTask,
                    sandbox_class: None,
                    parent_stage: None,
                    markers: sea_forge_core::types::ItemMarkers {
                        required: index == 0,
                        ..Default::default()
                    },
                    max_instances: 1,
                    depends_on: vec![],
                    environment: None,
                })
                .collect(),
            template_ref: None,
            job_contract_ref: None,
        })
        .unwrap(),
    )
    .unwrap();
    let request: Request = serde_json::from_value(serde_json::json!({"verb":"submit", "plan":plan_path, "policy":policy_path, "entity":"operator_local", "process":"test", "timeout":5})).unwrap();
    let response = handle_request(request, &state).await;
    assert_eq!(response["state"], "terminated", "{response}");
    let events: Vec<TraceEvent> = fs::read_to_string(
        root.path()
            .join("cases")
            .join(response["case_id"].as_str().unwrap())
            .join("case-events.jsonl"),
    )
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
    assert_eq!(
        events
            .iter()
            .filter(|event| event.kind == TraceKind::ItemActivated)
            .count(),
        2
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event.kind == TraceKind::SettlementRecorded)
            .count(),
        2
    );
    assert!(events
        .iter()
        .filter(|event| event.kind == TraceKind::SettlementRecorded)
        .all(|event| event.payload["status"] == "rejected"));
}

#[tokio::test]
async fn t13_2_human_task_waits_for_dispatched_episode_settlement() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let stub = tokio::spawn(async move {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = vec![0_u8; 4096];
            let _ = stream.read(&mut request).await;
            let body = br#"{"choices":[{"message":{"content":"ok"}}],"usage":{"total_tokens":1}}"#;
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(header.as_bytes()).await.unwrap();
            stream.write_all(body).await.unwrap();
        }
    });
    let root = tempfile::tempdir().unwrap();
    let policy_path = policy(root.path(), true, false);
    let mut agent_endpoint = endpoint(address.port());
    agent_endpoint.credential_ref = None;
    let state = Arc::new(
        ServerState::new(ServerConfig {
            root: root.path().to_path_buf(),
            max_concurrent_runs: 2,
            agent: AgentConfig {
                endpoints: vec![agent_endpoint],
                ..AgentConfig::default()
            },
            ..ServerConfig::default()
        })
        .unwrap(),
    );
    let plan_path = root.path().join("agent-and-human.json");
    fs::write(
        &plan_path,
        serde_json::to_vec(&CasePlan {
            version: "0.2".into(),
            plan_id: "plan_human".into(),
            case_id: "case_placeholder".into(),
            run_id: "run_placeholder".into(),
            intent_id: "int_human".into(),
            items: vec![
                PlanItem {
                    plan_item_id: "agent".into(),
                    name: "agent".into(),
                    operations: vec![Operation::AgentTask {
                        endpoint_ref: "local-test".into(),
                        instruction: "complete".into(),
                        max_turns: 1,
                        token_budget: None,
                        response_schema: None,
                        transcript_retention: None,
                    }],
                    entry_criteria: vec![],
                    exit_criteria: vec![],
                    settlement_criteria: SettlementCriteria::default(),
                    settlement_criteria_ref: None,
                    item_kind: ItemKind::AgentTask,
                    sandbox_class: None,
                    parent_stage: None,
                    markers: sea_forge_core::types::ItemMarkers {
                        required: true,
                        ..Default::default()
                    },
                    max_instances: 1,
                    depends_on: vec![],
                    environment: None,
                },
                PlanItem {
                    plan_item_id: "human".into(),
                    name: "human".into(),
                    operations: vec![],
                    entry_criteria: vec![],
                    exit_criteria: vec![],
                    settlement_criteria: SettlementCriteria::default(),
                    settlement_criteria_ref: None,
                    item_kind: ItemKind::HumanTask,
                    sandbox_class: None,
                    parent_stage: None,
                    markers: Default::default(),
                    max_instances: 1,
                    depends_on: vec![],
                    environment: None,
                },
            ],
            template_ref: None,
            job_contract_ref: None,
        })
        .unwrap(),
    )
    .unwrap();
    let request: Request = serde_json::from_value(serde_json::json!({"verb":"submit", "plan":plan_path, "policy":policy_path, "entity":"operator_local", "process":"test", "timeout":5})).unwrap();
    let response = handle_request(request, &state).await;
    assert_eq!(response["state"], "active");
    let events: Vec<TraceEvent> = fs::read_to_string(
        root.path()
            .join("cases")
            .join(response["case_id"].as_str().unwrap())
            .join("case-events.jsonl"),
    )
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
    assert!(events
        .iter()
        .any(|event| event.plan_item_id.as_deref() == Some("human")
            && event.kind == TraceKind::ItemActivated
            && event.payload["human_task"] == true));
    assert_eq!(
        events
            .iter()
            .filter(|event| event.plan_item_id.as_deref() == Some("agent")
                && event.kind == TraceKind::SettlementRecorded)
            .count(),
        1
    );
    let mut reversed: CasePlan = serde_json::from_slice(&fs::read(&plan_path).unwrap()).unwrap();
    reversed.items.reverse();
    let reversed_path = root.path().join("human-and-agent.json");
    fs::write(&reversed_path, serde_json::to_vec(&reversed).unwrap()).unwrap();
    let request: Request = serde_json::from_value(serde_json::json!({"verb":"submit", "plan":reversed_path, "policy":policy_path, "entity":"operator_local", "process":"test", "timeout":5})).unwrap();
    let response = handle_request(request, &state).await;
    assert_eq!(response["state"], "active");
    let events: Vec<TraceEvent> = fs::read_to_string(
        root.path()
            .join("cases")
            .join(response["case_id"].as_str().unwrap())
            .join("case-events.jsonl"),
    )
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
    assert!(events
        .iter()
        .any(|event| event.plan_item_id.as_deref() == Some("human")
            && event.kind == TraceKind::ItemActivated
            && event.payload["human_task"] == true));
    assert_eq!(
        events
            .iter()
            .filter(|event| event.plan_item_id.as_deref() == Some("agent")
                && event.kind == TraceKind::SettlementRecorded)
            .count(),
        1
    );
    stub.await.unwrap();
}

#[tokio::test]
async fn t13_2_permit_completion_rederives_before_stale_action() {
    let root = tempfile::tempdir().unwrap();
    let policy_path = policy(root.path(), true, false);
    let state = Arc::new(
        ServerState::new(ServerConfig {
            root: root.path().to_path_buf(),
            max_concurrent_runs: 1,
            ..ServerConfig::default()
        })
        .unwrap(),
    );
    let plan_path = root.path().join("stale-actions.json");
    fs::write(
        &plan_path,
        serde_json::to_vec(&CasePlan {
            version: "0.2".into(),
            plan_id: "plan_stale".into(),
            case_id: "case_placeholder".into(),
            run_id: "run_placeholder".into(),
            intent_id: "int_stale".into(),
            items: (0..2)
                .map(|index| PlanItem {
                    plan_item_id: format!("agent_{index}"),
                    name: "agent".into(),
                    operations: vec![Operation::AgentTask {
                        endpoint_ref: "missing".into(),
                        instruction: "reject".into(),
                        max_turns: 1,
                        token_budget: None,
                        response_schema: None,
                        transcript_retention: None,
                    }],
                    entry_criteria: vec![],
                    exit_criteria: vec![],
                    settlement_criteria: SettlementCriteria::default(),
                    settlement_criteria_ref: None,
                    item_kind: ItemKind::AgentTask,
                    sandbox_class: None,
                    parent_stage: None,
                    markers: sea_forge_core::types::ItemMarkers {
                        required: true,
                        ..Default::default()
                    },
                    max_instances: 1,
                    depends_on: vec![],
                    environment: None,
                })
                .collect(),
            template_ref: None,
            job_contract_ref: None,
        })
        .unwrap(),
    )
    .unwrap();
    let request: Request = serde_json::from_value(serde_json::json!({"verb":"submit", "plan":plan_path, "policy":policy_path, "entity":"operator_local", "process":"test", "timeout":5})).unwrap();
    let response = handle_request(request, &state).await;
    assert_eq!(response["state"], "terminated", "{response}");
    let events: Vec<TraceEvent> = fs::read_to_string(
        root.path()
            .join("cases")
            .join(response["case_id"].as_str().unwrap())
            .join("case-events.jsonl"),
    )
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
    assert_eq!(
        events
            .iter()
            .filter(|event| event.kind == TraceKind::ItemActivated)
            .count(),
        1
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event.kind == TraceKind::SettlementRecorded)
            .count(),
        1
    );
}

/// T13.3: after restart, a durable cancellation control settles the
/// interrupted HTTP delegation exactly once without resuming it.
#[test]
fn t13_3_restart_after_durable_cancellation_settles_once() {
    let root = tempfile::tempdir().unwrap();
    let run = sea_forge_core::ids::run_id().unwrap();
    let case = sea_forge_core::ids::case_id().unwrap();
    let plan = sea_forge_core::types::CasePlan {
        version: "0.2".into(),
        plan_id: "plan_test".into(),
        case_id: case.clone(),
        run_id: run.clone(),
        intent_id: "int_test".into(),
        items: vec![sea_forge_core::types::PlanItem {
            plan_item_id: "agent".into(),
            name: "agent_task".into(),
            operations: vec![sea_forge_core::types::Operation::AgentTask {
                endpoint_ref: "local-test".into(),
                instruction: "test".into(),
                max_turns: 1,
                token_budget: None,
                response_schema: None,
                transcript_retention: None,
            }],
            entry_criteria: vec![],
            exit_criteria: vec![],
            settlement_criteria: SettlementCriteria::default(),
            settlement_criteria_ref: None,
            item_kind: sea_forge_core::types::ItemKind::AgentTask,
            sandbox_class: None,
            parent_stage: None,
            markers: Default::default(),
            max_instances: 1,
            depends_on: vec![],
            environment: None,
        }],
        template_ref: None,
        job_contract_ref: None,
    };
    let run_dir = root.path().join("runs").join(&run);
    fs::create_dir_all(&run_dir).unwrap();
    fs::write(
        run_dir.join("plan.json"),
        serde_json::to_vec_pretty(&plan).unwrap(),
    )
    .unwrap();
    let ledger =
        sea_forge_ledger::LedgerStream::open(root.path(), format!("case-{case}"), "test").unwrap();
    ledger
        .commit_typed(
            "settlement",
            vec!["unrelated".into()],
            &serde_json::json!({"run_id": "run_unrelated", "status": "accepted"}),
            vec![],
        )
        .unwrap();
    ledger
        .commit_typed(
            "control_request",
            vec![case.clone(), run.clone(), "agent".into()],
            &serde_json::json!({
                "version": "0.2",
                "control_id": "cancel_test",
                "case_id": case,
                "run_id": run,
                "plan_item_id": "agent",
                "requester": "operator_local",
                "authority_decision_ref": "dec_test",
                "requested_at": "2026-07-20T00:00:00Z",
                "ordinal": 1
            }),
            vec!["dec_test".into()],
        )
        .unwrap();

    let state = ServerState::new(config(root.path(), endpoint(1))).unwrap();
    drop(state);

    let entries = ledger.read_entries().unwrap();
    let settlements: Vec<_> = entries
        .iter()
        .filter(|entry| entry.record_kind == "settlement" && entry.payload["run_id"] == run)
        .collect();
    assert_eq!(settlements.len(), 1, "exactly one terminal settlement");
    let [settlement] = settlements.as_slice() else {
        panic!("expected one terminal settlement");
    };
    assert_eq!(settlement.payload["case_id"], case);
    assert_eq!(settlement.payload["item_id"], "agent");
    assert_eq!(settlement.payload["run_id"], run);
    assert_eq!(settlement.payload["status"], "rejected");
    assert_eq!(
        settlement.payload["basis"],
        serde_json::json!(["cancelled"])
    );
    let evidence: Vec<_> = entries
        .iter()
        .filter(|entry| {
            entry.record_kind == "agent_task_evidence" && entry.payload["run_id"] == run
        })
        .collect();
    assert_eq!(evidence.len(), 1, "one recovery evidence record");
    let [evidence] = evidence.as_slice() else {
        panic!("expected one recovery evidence record");
    };
    assert_eq!(evidence.payload["case_id"], case);
    assert_eq!(evidence.payload["item_id"], "agent");
    assert_eq!(evidence.payload["run_id"], run);
    assert!(run_dir.join("transcript-evidence.json").is_file());

    // A second restart sees the terminal settlement and does not append one.
    let state = ServerState::new(config(root.path(), endpoint(1))).unwrap();
    drop(state);
    let count = ledger
        .read_entries()
        .unwrap()
        .iter()
        .filter(|entry| entry.record_kind == "settlement" && entry.payload["run_id"] == run)
        .count();
    assert_eq!(count, 1);
}
