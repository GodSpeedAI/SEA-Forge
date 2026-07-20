use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind};
use sea_forge_server::{agent_probe, delegation, ServerConfig};
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
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
