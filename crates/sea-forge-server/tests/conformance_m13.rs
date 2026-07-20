use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind};
use sea_forge_core::types::SettlementCriteria;
use sea_forge_server::{agent_probe, delegation, Request, ServerConfig, ServerState};
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
    let state = Arc::new(ServerState::new(config));

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
