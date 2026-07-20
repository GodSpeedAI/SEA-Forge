use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind};
use sea_forge_server::{agent_probe, ServerConfig};
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

fn policy(root: &Path, allow_external: bool, allow_secret: bool) -> PathBuf {
    let mut rules = String::new();
    if allow_external {
        rules.push_str(
            "  - name: allow-agent-probe\n    verdict: allow\n    actor_role: operator\n    operation_kind: agent_probe\n",
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

#[tokio::test]
async fn t12_1_probe_records_intent_plan_authority_evidence_and_settlement() {
    let (address, _connections, task) = stub(
        r#"{"choices":[{"message":{"content":"API_KEY=test-secret"}}],"usage":{"total_tokens":1}}"#,
    )
    .await;
    let root = tempfile::tempdir().unwrap();
    let endpoint = endpoint(address.port());
    let policy = policy(root.path(), true, true);
    let reads = Arc::new(AtomicUsize::new(0));
    let outcome = agent_probe::probe(
        &config(root.path(), endpoint),
        agent_probe::ProbeRequest {
            endpoint_id: "local-test",
            prompt: "health check",
            model: None,
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
        },
        &CountingResolver {
            reads: Arc::clone(&reads),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Accepted
    );
    assert_eq!(reads.load(Ordering::SeqCst), 1);
    let run = root.path().join("runs").join(&outcome.run_id);
    for file in [
        "intent.json",
        "plan.json",
        "authority.json",
        "evidence.json",
        "settlement.json",
    ] {
        assert!(run.join(file).is_file(), "missing {file}");
    }
    let _ = task.await;
}

#[tokio::test]
async fn t12_2_denied_external_api_has_no_connection_or_secret_read() {
    let (address, connections, task) =
        stub(r#"{"choices":[{"message":{"content":"should not be received"}}]}"#).await;
    let root = tempfile::tempdir().unwrap();
    let policy = policy(root.path(), false, true);
    let reads = Arc::new(AtomicUsize::new(0));
    let outcome = agent_probe::probe(
        &config(root.path(), endpoint(address.port())),
        agent_probe::ProbeRequest {
            endpoint_id: "local-test",
            prompt: "health check",
            model: None,
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
        },
        &CountingResolver {
            reads: Arc::clone(&reads),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Rejected
    );
    assert_eq!(connections.load(Ordering::SeqCst), 0);
    assert_eq!(reads.load(Ordering::SeqCst), 0);
    task.abort();
}

#[tokio::test]
async fn t12_2_denied_secret_access_has_no_connection_or_secret_read() {
    let (address, connections, task) =
        stub(r#"{"choices":[{"message":{"content":"should not be received"}}]}"#).await;
    let root = tempfile::tempdir().unwrap();
    let policy = policy(root.path(), true, false);
    let reads = Arc::new(AtomicUsize::new(0));
    let outcome = agent_probe::probe(
        &config(root.path(), endpoint(address.port())),
        agent_probe::ProbeRequest {
            endpoint_id: "local-test",
            prompt: "health check",
            model: None,
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
        },
        &CountingResolver {
            reads: Arc::clone(&reads),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Rejected
    );
    assert_eq!(connections.load(Ordering::SeqCst), 0);
    assert_eq!(reads.load(Ordering::SeqCst), 0);
    task.abort();
}

#[tokio::test]
async fn t12_3_credential_sentinel_never_enters_persisted_probe_records() {
    let (address, _connections, task) =
        stub(r#"{"choices":[{"message":{"content":"API_KEY=test-secret"}}]}"#).await;
    let root = tempfile::tempdir().unwrap();
    let policy = policy(root.path(), true, true);
    let reads = Arc::new(AtomicUsize::new(0));
    let outcome = agent_probe::probe(
        &config(root.path(), endpoint(address.port())),
        agent_probe::ProbeRequest {
            endpoint_id: "local-test",
            prompt: "health check",
            model: None,
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
        },
        &CountingResolver { reads },
    )
    .await
    .unwrap();
    let mut contents = Vec::new();
    collect_files(root.path(), &mut contents);
    let joined = String::from_utf8_lossy(&contents.concat()).to_ascii_lowercase();
    assert!(!joined.contains("test-secret"));
    assert!(!joined.contains("api_key"));
    task.abort();
    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Accepted
    );
}

fn collect_files(path: &Path, output: &mut Vec<Vec<u8>>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, output);
        } else if let Ok(bytes) = fs::read(path) {
            output.push(bytes);
        }
    }
}
