use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind};
use sea_forge_core::types::{ContractRef, ExtensionDescriptor, ExtensionKind};
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
        argv: vec![],
        env: vec![],
        credential_ref: Some("TEST_CREDENTIAL".into()),
        default_model: Some("test-model".into()),
        allow_loopback_test: true,
        max_request_bytes: 16_384,
        max_response_bytes: 16_384,
        timeout_secs: 5,
        status: None,
        transcript_retention: None,
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
    policy(root.path(), true, true);
    let reads = Arc::new(AtomicUsize::new(0));
    let outcome = agent_probe::probe(
        &config(root.path(), endpoint),
        agent_probe::ProbeRequest {
            endpoint_id: "local-test",
            prompt: "health check",
            model: None,
            // F-16: cell-relative policy spelling.
            policy_path: "policy.yaml",
            entity: "operator_local",
            process: "test",
            // F-08: in-process fixture has no socket identity; local-operator shape.
            actor_role: sea_forge_core::types::ActorRole::Operator,
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
    policy(root.path(), false, true);
    let reads = Arc::new(AtomicUsize::new(0));
    let outcome = agent_probe::probe(
        &config(root.path(), endpoint(address.port())),
        agent_probe::ProbeRequest {
            endpoint_id: "local-test",
            prompt: "health check",
            model: None,
            // F-16: cell-relative policy spelling.
            policy_path: "policy.yaml",
            entity: "operator_local",
            process: "test",
            // F-08: in-process fixture has no socket identity; local-operator shape.
            actor_role: sea_forge_core::types::ActorRole::Operator,
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
    policy(root.path(), true, false);
    let reads = Arc::new(AtomicUsize::new(0));
    let outcome = agent_probe::probe(
        &config(root.path(), endpoint(address.port())),
        agent_probe::ProbeRequest {
            endpoint_id: "local-test",
            prompt: "health check",
            model: None,
            // F-16: cell-relative policy spelling.
            policy_path: "policy.yaml",
            entity: "operator_local",
            process: "test",
            // F-08: in-process fixture has no socket identity; local-operator shape.
            actor_role: sea_forge_core::types::ActorRole::Operator,
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
    policy(root.path(), true, true);
    let reads = Arc::new(AtomicUsize::new(0));
    let outcome = agent_probe::probe(
        &config(root.path(), endpoint(address.port())),
        agent_probe::ProbeRequest {
            endpoint_id: "local-test",
            prompt: "health check",
            model: None,
            // F-16: cell-relative policy spelling.
            policy_path: "policy.yaml",
            entity: "operator_local",
            process: "test",
            // F-08: in-process fixture has no socket identity; local-operator shape.
            actor_role: sea_forge_core::types::ActorRole::Operator,
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

/// Slice 4.2: a newly registered agent endpoint is an extension mutation that
/// marks the current self-model snapshot stale so the next rebuild reflects it,
/// without mutating any immutable snapshot file.
#[tokio::test]
async fn t12_endpoint_registration_marks_self_model_snapshot_stale() {
    let (address, _connections, task) =
        stub(r#"{"choices":[{"message":{"content":"ok"}}],"usage":{"total_tokens":1}}"#).await;
    let root = tempfile::tempdir().unwrap();
    // Initialize a self-model snapshot so staleness is observable.
    sea_forge_self_model::store::ensure_init(
        root.path(),
        &sea_forge_self_model::store::RebuildInputs {
            cell_id: "cell_t12stale",
            active_extensions: vec![],
            environments_present: vec![],
            probes: vec![],
            sandbox_classes_available: vec!["local".into()],
            created_at: "2026-07-20T00:00:00Z",
            capability_projection_sha256: "sha256:cap0",
            actor_id: "operator_test",
        },
    )
    .unwrap();
    assert!(!sea_forge_self_model::store::is_stale(root.path()).unwrap());

    policy(root.path(), true, true);
    let outcome = agent_probe::probe(
        &config(root.path(), endpoint(address.port())),
        agent_probe::ProbeRequest {
            endpoint_id: "local-test",
            prompt: "health check",
            model: None,
            // F-16: cell-relative policy spelling.
            policy_path: "policy.yaml",
            entity: "operator_local",
            process: "test",
            // F-08: in-process fixture has no socket identity; local-operator shape.
            actor_role: sea_forge_core::types::ActorRole::Operator,
        },
        &CountingResolver {
            reads: Arc::new(AtomicUsize::new(0)),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Accepted
    );
    // A brand-new endpoint registration must mark the snapshot stale.
    assert!(
        sea_forge_self_model::store::is_stale(root.path()).unwrap(),
        "endpoint registration must mark the current snapshot stale"
    );
    let _ = task.await;
}

// Slice 4.2: idempotent re-registration of an unchanged endpoint is a no-op
// on the registry (covered by the extension crate's
// `immutable_runtime_adapter_is_idempotent_for_same_version_and_hash` unit
// test). This case exists only at the registry layer, so it is not duplicated
// here against the probe service.

/// T12.6: endpoint error taxonomy. Each failure mode settles rejected with a
/// typed `error_class` subcode; no provider fallback is attempted (the single
/// configured endpoint receives exactly one connection attempt).
async fn error_taxonomy_case(
    stub_status: &'static str,
    stub_body: &'static str,
    expected_class: &str,
) {
    let (address, connections, task) = server_status_body(stub_status, stub_body).await;
    let root = tempfile::tempdir().unwrap();
    policy(root.path(), true, true);
    let outcome = agent_probe::probe(
        &config(root.path(), endpoint(address.port())),
        agent_probe::ProbeRequest {
            endpoint_id: "local-test",
            prompt: "health check",
            model: None,
            // F-16: cell-relative policy spelling.
            policy_path: "policy.yaml",
            entity: "operator_local",
            process: "test",
            // F-08: in-process fixture has no socket identity; local-operator shape.
            actor_role: sea_forge_core::types::ActorRole::Operator,
        },
        &CountingResolver {
            reads: Arc::new(AtomicUsize::new(0)),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Rejected,
        "expected rejected for {stub_status}"
    );
    assert_eq!(
        outcome.error_class.as_deref(),
        Some(expected_class),
        "expected class {expected_class} for {stub_status}, got {:?}",
        outcome.error_class
    );
    // No fallback: exactly one connection to the single configured endpoint.
    assert_eq!(
        connections.load(Ordering::SeqCst),
        1,
        "no provider fallback permitted"
    );
    task.abort();
}

#[tokio::test]
async fn t12_6_http_4xx_settles_rejected_with_typed_subcode() {
    error_taxonomy_case("400 Bad Request", "bad request", "agent_endpoint_http_4xx").await;
}

#[tokio::test]
async fn t12_6_http_5xx_settles_rejected_with_typed_subcode() {
    error_taxonomy_case(
        "500 Internal Server Error",
        "oops",
        "agent_endpoint_http_5xx",
    )
    .await;
}

#[tokio::test]
async fn t12_6_redirect_settles_rejected_with_typed_subcode() {
    // The provider disables redirects; a 302 surfaces as AgentError::Redirect.
    error_taxonomy_case("302 Found", "", "agent_endpoint_redirect").await;
}

#[tokio::test]
async fn t12_6_schema_invalid_settles_rejected_with_typed_subcode() {
    error_taxonomy_case("200 OK", "not-json-at-all", "agent_endpoint_schema_invalid").await;
}

#[tokio::test]
async fn t12_6_oversize_response_settles_rejected_with_typed_subcode() {
    // Build a stub whose declared Content-Length exceeds the endpoint limit
    // (16_384 bytes) so the provider rejects before reading the body.
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let connections = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&connections);
    let task = tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            count.fetch_add(1, Ordering::SeqCst);
            let header =
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 999999\r\nConnection: close\r\n\r\n"
                    .to_string();
            stream.write_all(header.as_bytes()).await.unwrap();
            // Send a few bytes then close; the Content-Length check fires first.
            stream.write_all(b"{").await.unwrap();
        }
    });
    let root = tempfile::tempdir().unwrap();
    policy(root.path(), true, true);
    let outcome = agent_probe::probe(
        &config(root.path(), endpoint(address.port())),
        agent_probe::ProbeRequest {
            endpoint_id: "local-test",
            prompt: "health check",
            model: None,
            // F-16: cell-relative policy spelling.
            policy_path: "policy.yaml",
            entity: "operator_local",
            process: "test",
            // F-08: in-process fixture has no socket identity; local-operator shape.
            actor_role: sea_forge_core::types::ActorRole::Operator,
        },
        &CountingResolver {
            reads: Arc::new(AtomicUsize::new(0)),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Rejected
    );
    assert_eq!(
        outcome.error_class.as_deref(),
        Some("agent_endpoint_oversize")
    );
    assert_eq!(connections.load(Ordering::SeqCst), 1);
    task.abort();
}

#[tokio::test]
async fn t12_6_unreachable_settles_rejected_with_typed_subcode() {
    // Bind and immediately drop to obtain a free but closed port.
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let root = tempfile::tempdir().unwrap();
    policy(root.path(), true, true);
    let outcome = agent_probe::probe(
        &config(root.path(), endpoint(port)),
        agent_probe::ProbeRequest {
            endpoint_id: "local-test",
            prompt: "health check",
            model: None,
            // F-16: cell-relative policy spelling.
            policy_path: "policy.yaml",
            entity: "operator_local",
            process: "test",
            // F-08: in-process fixture has no socket identity; local-operator shape.
            actor_role: sea_forge_core::types::ActorRole::Operator,
        },
        &CountingResolver {
            reads: Arc::new(AtomicUsize::new(0)),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Rejected
    );
    assert_eq!(
        outcome.error_class.as_deref(),
        Some("agent_endpoint_unreachable")
    );
}

/// A stub server that replies with an arbitrary status line and body, and
/// counts connections so the no-fallback invariant is observable.
async fn server_status_body(
    status: &'static str,
    body: &'static str,
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
            let body_bytes = body.as_bytes();
            let header = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body_bytes.len()
            );
            let _ = stream.write_all(header.as_bytes()).await;
            let _ = stream.write_all(body_bytes).await;
        }
    });
    (address, connections, task)
}

// ── SUP-06: config-derived adapter version + failed registration settles ──

fn seed_descriptor(version: &str) -> ExtensionDescriptor {
    ExtensionDescriptor {
        extension_id: "agent_endpoint_local-test".into(),
        kind: ExtensionKind::RuntimeAdapter,
        name: "agent endpoint local-test".into(),
        version: version.into(),
        provider: "sea-forge-agent".into(),
        capabilities: vec!["agent_probe".into()],
        authority_surface: "external_api".into(),
        input_contract: ContractRef {
            schema: "sea-forge-agent.endpoint.v1".into(),
            sha256: format!("sha256:{}", "aa".repeat(32)),
        },
        output_contract: ContractRef {
            schema: "sea-forge-agent.probe.v1".into(),
            sha256: format!("sha256:{}", "bb".repeat(32)),
        },
        deterministic: false,
        installed_at: None,
    }
}

async fn run_probe(root: &Path, port: u16) -> agent_probe::ProbeOutcome {
    let reads = Arc::new(AtomicUsize::new(0));
    agent_probe::probe(
        &config(root, endpoint(port)),
        agent_probe::ProbeRequest {
            endpoint_id: "local-test",
            prompt: "health check",
            model: None,
            policy_path: "policy.yaml",
            entity: "operator_local",
            process: "test",
            actor_role: sea_forge_core::types::ActorRole::Operator,
        },
        &CountingResolver {
            reads: Arc::clone(&reads),
        },
    )
    .await
    .unwrap()
}

/// A stub that accepts connections in a loop — for tests that probe the
/// same endpoint more than once or never connect at all.
async fn stub_multi(
    response: &'static str,
) -> (SocketAddr, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let connections = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&connections);
    let body = response.to_owned();
    let task = tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                count.fetch_add(1, Ordering::SeqCst);
                let mut request = vec![0_u8; 32_768];
                let _ = stream.read(&mut request).await;
                let body_bytes = body.as_bytes();
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body_bytes.len()
                );
                let _ = stream.write_all(header.as_bytes()).await;
                let _ = stream.write_all(body_bytes).await;
            }
        }
    });
    (address, connections, task)
}

#[tokio::test]
async fn t12_7_probe_registers_adapter_with_config_derived_version() {
    let (address, connections, task) =
        stub(r#"{"choices":[{"message":{"content":"ok"}}],"usage":{"total_tokens":1}}"#).await;
    let root = tempfile::tempdir().unwrap();
    policy(root.path(), true, true);
    let outcome = run_probe(root.path(), address.port()).await;
    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Accepted
    );

    let registry = sea_forge_extension::ExtensionRegistry::load(root.path()).unwrap();
    let entry = registry
        .extensions
        .iter()
        .find(|e| e.extension_id == "agent_endpoint_local-test")
        .expect("probe must register its endpoint");
    // The version is derived from the config identity, not a hardcoded
    // literal — and the full config hash stays in the input contract.
    let snapshot = config(root.path(), endpoint(address.port()))
        .agent
        .endpoints[0]
        .snapshot()
        .unwrap();
    assert_eq!(
        entry.version,
        agent_probe::endpoint_adapter_version(&snapshot)
    );
    assert_ne!(entry.version, "0.1.0");
    assert!(
        snapshot.descriptor_config_sha256.starts_with("sha256:") && entry.version.len() < 40,
        "version must be grammar-safe and short"
    );
    assert_eq!(connections.load(Ordering::SeqCst), 1);
    let _ = task.await;
}

#[tokio::test]
async fn t12_7_identical_config_reprobe_is_idempotent_and_stable() {
    // One loop-accepting stub so both probes share the identical endpoint
    // configuration — the exact idempotent-re-registration scenario.
    let (address, _connections, task) =
        stub_multi(r#"{"choices":[{"message":{"content":"ok"}}],"usage":{"total_tokens":1}}"#)
            .await;
    let root = tempfile::tempdir().unwrap();
    policy(root.path(), true, true);
    let first = run_probe(root.path(), address.port()).await;
    let second = run_probe(root.path(), address.port()).await;
    assert_eq!(
        first.settlement,
        sea_forge_core::types::SettlementStatus::Accepted
    );
    assert_eq!(
        second.settlement,
        sea_forge_core::types::SettlementStatus::Accepted
    );

    let registry = sea_forge_extension::ExtensionRegistry::load(root.path()).unwrap();
    let entries: Vec<_> = registry
        .extensions
        .iter()
        .filter(|e| e.extension_id == "agent_endpoint_local-test")
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "identical configs must not pile up entries"
    );
    task.abort();
}

#[tokio::test]
async fn t12_7_config_edit_then_reprobe_replaces_in_place_and_marks_stale() {
    let (address, _connections, task) =
        stub(r#"{"choices":[{"message":{"content":"ok"}}],"usage":{"total_tokens":1}}"#).await;
    let (address2, _connections2, task2) =
        stub_multi(r#"{"choices":[{"message":{"content":"ok"}}],"usage":{"total_tokens":1}}"#)
            .await;
    let root = tempfile::tempdir().unwrap();
    // Seed a self-model snapshot so staleness is observable across the edit.
    sea_forge_self_model::store::ensure_init(
        root.path(),
        &sea_forge_self_model::store::RebuildInputs {
            cell_id: "cell_t12_7",
            active_extensions: vec![],
            environments_present: vec![],
            probes: vec![],
            sandbox_classes_available: vec!["local".into()],
            created_at: "2026-08-23T00:00:00Z",
            capability_projection_sha256: "sha256:cap",
            actor_id: "operator_test",
        },
    )
    .unwrap();

    policy(root.path(), true, true);
    let first = run_probe(root.path(), address.port()).await;
    assert_eq!(
        first.settlement,
        sea_forge_core::types::SettlementStatus::Accepted
    );
    sea_forge_self_model::store::rebuild(
        root.path(),
        &sea_forge_self_model::store::RebuildInputs {
            cell_id: "cell_t12_7",
            active_extensions: vec![],
            environments_present: vec![],
            probes: vec![],
            sandbox_classes_available: vec!["local".into()],
            created_at: "2026-08-23T00:00:01Z",
            capability_projection_sha256: "sha256:cap",
            actor_id: "operator_test",
        },
    )
    .unwrap();
    assert!(!sea_forge_self_model::store::is_stale(root.path()).unwrap());

    // Edit the endpoint's config: a different default_model changes the
    // descriptor identity while the old hardcoded world would have kept
    // version "0.1.0" and bricked every future probe.
    let mut edited = endpoint(address2.port());
    edited.default_model = Some("edited-model".into());
    let edited_for_snapshot = edited.clone();
    let reads = Arc::new(AtomicUsize::new(0));
    let outcome = agent_probe::probe(
        &ServerConfig {
            root: root.path().to_path_buf(),
            agent: AgentConfig {
                endpoints: vec![edited],
                ..AgentConfig::default()
            },
            ..ServerConfig::default()
        },
        agent_probe::ProbeRequest {
            endpoint_id: "local-test",
            prompt: "health check",
            model: None,
            policy_path: "policy.yaml",
            entity: "operator_local",
            process: "test",
            actor_role: sea_forge_core::types::ActorRole::Operator,
        },
        &CountingResolver {
            reads: Arc::clone(&reads),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Accepted,
        "a config edit must not brick future probes"
    );

    let registry = sea_forge_extension::ExtensionRegistry::load(root.path()).unwrap();
    let entry = registry
        .extensions
        .iter()
        .find(|e| e.extension_id == "agent_endpoint_local-test")
        .unwrap();
    let edited_snapshot = edited_for_snapshot.snapshot().unwrap();
    assert_eq!(
        entry.version,
        agent_probe::endpoint_adapter_version(&edited_snapshot),
        "the registry must reflect the edited config's derived version"
    );
    // An install/change after a clean rebuild marks the snapshot stale again.
    assert!(
        sea_forge_self_model::store::is_stale(root.path()).unwrap(),
        "changed-config re-registration must mark the self-model stale"
    );
    task.abort();
    task2.abort();
}

#[tokio::test]
async fn t12_7_registration_failure_settles_the_run_fail_closed() {
    let (address, connections, task) =
        stub(r#"{"choices":[{"message":{"content":"should never be reached"}}]}"#).await;
    let root = tempfile::tempdir().unwrap();
    policy(root.path(), true, true);

    // Seed an honest ledger-committed registry whose runtime-adapter entry is
    // quarantined: registration must refuse it (SUP-08 guard), which is the
    // failure SUP-06 settles instead of stranding.
    // SUP-06: seed through the dedicated registry ledger — the one
    // `register_endpoint` attests and verifies against.
    let stream =
        sea_forge_ledger::LedgerStream::open(root.path(), "extension-registry", "tester").unwrap();
    let mut reg = sea_forge_extension::ExtensionRegistry {
        version: "0.2".into(),
        updated_at: "now".into(),
        extensions: Vec::new(),
    };
    reg.register_immutable_runtime_adapter(&seed_descriptor("seed-0"))
        .unwrap();
    let authority = stream
        .commit_typed("authority_decision", vec![], &serde_json::json!({}), vec![])
        .unwrap();
    reg.save(root.path(), &stream, &authority).unwrap();
    reg.extensions[0].status = sea_forge_extension::ExtensionStatus::Quarantined;
    reg.save(root.path(), &stream, &authority).unwrap();

    let outcome = run_probe(root.path(), address.port()).await;
    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Rejected,
        "registration failure must settle the run"
    );
    assert_eq!(
        outcome.error_class.as_deref(),
        Some("agent_endpoint_registration_failed")
    );

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
    let settlement: serde_json::Value =
        serde_json::from_slice(&fs::read(run.join("settlement.json")).unwrap()).unwrap();
    assert_eq!(settlement["status"], "rejected");
    assert!(
        settlement["basis"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b == "agent_endpoint_registration_failed"),
        "{settlement}"
    );
    // Fail-closed: the provider is never contacted on a failed registration.
    assert_eq!(connections.load(Ordering::SeqCst), 0);
    // The quarantine survives untouched.
    let registry = sea_forge_extension::ExtensionRegistry::load(root.path()).unwrap();
    assert_eq!(
        registry.extensions[0].status,
        sea_forge_extension::ExtensionStatus::Quarantined
    );
    // The stub never accepted a connection; do not await its task — abort it.
    task.abort();
}
