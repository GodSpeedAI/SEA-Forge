use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind};
use sea_forge_core::types::{
    CasePlan, ItemKind, ItemMarkers, Operation, PlanItem, SettlementCriteria, TraceEvent, TraceKind,
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            transcript_retention: sea_forge_agent::TranscriptRetentionMode::Full,
            ..Default::default()
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

/// M13 T15 step 2/5: a schema-valid final output settles accepted and
/// becomes a named evidence field (schema hash + validity, never the raw
/// payload).
#[tokio::test]
async fn t15_schema_valid_response_settles_accepted_with_named_evidence() {
    let (address, _connections, task) = stub(
        r#"{"choices":[{"message":{"content":"{\"status\":\"accepted\"}"}}],"usage":{"total_tokens":1}}"#,
    )
    .await;
    let root = tempfile::tempdir().unwrap();
    let policy = policy(root.path(), true, true);
    let (res, _reads) = resolver();
    let schema = serde_json::json!({
        "type": "object",
        "required": ["status"],
        "properties": {"status": {"type": "string", "enum": ["accepted"]}}
    });
    let outcome = delegation::execute(
        &config(root.path(), endpoint(address.port())),
        delegation::DelegationRequest {
            endpoint_id: "local-test",
            instruction: "report status",
            max_turns: 1,
            response_schema: Some(&schema),
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
            ..Default::default()
        },
        &res,
    )
    .await
    .unwrap();

    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Accepted
    );
    assert!(outcome.basis.iter().any(|b| b == "response_schema_valid"));
    let run = root.path().join("runs").join(&outcome.run_id);
    let evidence_path = run.join("response-schema-evidence.json");
    assert!(
        evidence_path.is_file(),
        "named response_schema evidence must be persisted"
    );
    let evidence: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&evidence_path).unwrap()).unwrap();
    assert_eq!(evidence["valid"], true);
    let _ = task.await;
}

/// M13 T15 step 5: a schema-invalid final output settles rejected with a
/// typed `schema_invalid` basis, and the named evidence never carries the
/// raw non-conforming payload.
#[tokio::test]
async fn t15_schema_invalid_response_settles_rejected_without_payload_leak() {
    let (address, _connections, task) = stub(
        r#"{"choices":[{"message":{"content":"{\"status\":\"unleaked-marker-12345\"}"}}],"usage":{"total_tokens":1}}"#,
    )
    .await;
    let root = tempfile::tempdir().unwrap();
    let policy = policy(root.path(), true, true);
    let (res, _reads) = resolver();
    let schema = serde_json::json!({
        "type": "object",
        "required": ["status"],
        "properties": {"status": {"type": "string", "enum": ["accepted"]}}
    });
    let outcome = delegation::execute(
        &config(root.path(), endpoint(address.port())),
        delegation::DelegationRequest {
            endpoint_id: "local-test",
            instruction: "report status",
            max_turns: 1,
            response_schema: Some(&schema),
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
            ..Default::default()
        },
        &res,
    )
    .await
    .unwrap();

    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Rejected
    );
    assert_eq!(outcome.error_class.as_deref(), Some("schema_invalid"));
    assert!(outcome.basis.iter().any(|b| b == "schema_invalid"));
    let run = root.path().join("runs").join(&outcome.run_id);
    let evidence_path = run.join("response-schema-evidence.json");
    let evidence_text = fs::read_to_string(&evidence_path).unwrap();
    let evidence: serde_json::Value = serde_json::from_str(&evidence_text).unwrap();
    assert_eq!(evidence["valid"], false);
    assert!(
        !evidence_text.contains("unleaked-marker-12345"),
        "schema evidence must not leak the raw non-conforming payload"
    );
    let _ = task.await;
}

/// M13 T15 step 3: cap termination accepts or rejects solely by criteria —
/// a turn-cap-exhausted episode whose available final output satisfies the
/// declared criteria settles accepted, but `turn_cap_exceeded` always stays
/// in the basis so how it terminated is never hidden.
#[tokio::test]
async fn t15_turn_cap_exceeded_with_satisfied_criteria_accepts_and_retains_basis() {
    let (address, _connections, task) = stub(
        r#"{"choices":[{"message":{"content":"proof artifact committed"}}],"usage":{"total_tokens":42}}"#,
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
            max_turns: 5,
            token_budget: Some(25),
            criteria: SettlementCriteria {
                agent_output_must_contain: Some("proof artifact committed".into()),
                ..SettlementCriteria::default()
            },
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
            ..Default::default()
        },
        &res,
    )
    .await
    .unwrap();

    assert_eq!(outcome.termination.as_deref(), Some("turn_cap_exceeded"));
    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Accepted
    );
    assert!(
        outcome.basis.iter().any(|b| b == "turn_cap_exceeded"),
        "must retain turn_cap_exceeded in basis even when accepted: {:?}",
        outcome.basis
    );
    let _ = task.await;
}

/// M13 T15 step 4: case dispatch must reuse the exact basis delegation
/// already committed instead of constructing a synthetic
/// `["delegation_completed"]` basis that hides the real termination/
/// criteria outcome from the case-level completion record.
#[tokio::test]
async fn t15_case_dispatch_settlement_reuses_real_basis_not_synthetic() {
    let (address, _connections, task) = stub(
        r#"{"choices":[{"message":{"content":"done, but no proof artifact"}}],"usage":{"total_tokens":1}}"#,
    )
    .await;
    let root = tempfile::tempdir().unwrap();
    let policy_path = policy(root.path(), true, true);
    // Case dispatch resolves credentials via the real environment resolver
    // (unlike the fixture `CountingResolver` used by direct-delegation
    // tests); an endpoint with no credential_ref skips that resolution
    // entirely so this test exercises settlement basis reuse, not secret
    // handling.
    let mut ep = endpoint(address.port());
    ep.credential_ref = None;
    let plan = CasePlan {
        version: "0.2".into(),
        plan_id: "plan_t15".into(),
        case_id: "case_t15".into(),
        run_id: "run_t15".into(),
        intent_id: "int_t15".into(),
        items: vec![PlanItem {
            plan_item_id: "agent".into(),
            name: "agent_task".into(),
            operations: vec![Operation::AgentTask {
                endpoint_ref: "local-test".into(),
                instruction: "publish proof".into(),
                max_turns: 1,
                token_budget: None,
                response_schema: None,
                transcript_retention: None,
            }],
            entry_criteria: vec![],
            entry_criteria_mode: Default::default(),
            exit_criteria: vec![],
            settlement_criteria: SettlementCriteria {
                agent_output_must_contain: Some("proof artifact committed".into()),
                ..SettlementCriteria::default()
            },
            settlement_criteria_ref: None,
            item_kind: ItemKind::AgentTask,
            sandbox_class: None,
            parent_stage: None,
            markers: ItemMarkers {
                required: true,
                ..ItemMarkers::default()
            },
            max_instances: 1,
            depends_on: vec![],
            environment: None,
            proposed_by: None,
        }],
        template_ref: None,
        job_contract_ref: None,
    };
    let plan_path = root.path().join("plan.json");
    fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();

    let state = std::sync::Arc::new(ServerState::new(config(root.path(), ep)).unwrap());
    let request: Request = serde_json::from_str(&format!(
        r#"{{"verb":"submit","plan":{:?},"policy":{:?},"entity":"operator_local","process":"test"}}"#,
        plan_path.to_str().unwrap(),
        policy_path.to_str().unwrap(),
    ))
    .unwrap();
    let response = handle_request(request, &state).await;
    let case_id = response["case_id"].as_str().unwrap().to_string();

    let ledger =
        sea_forge_ledger::LedgerStream::open(root.path(), format!("case-{case_id}"), "test")
            .unwrap();
    let entries = ledger.read_entries().unwrap();
    let completion = entries
        .iter()
        .find(|entry| entry.record_kind == "settlement_event")
        .expect("case dispatch must record a settlement_event completion");
    assert_eq!(completion.payload["status"], "rejected");
    assert_ne!(
        completion.payload["basis"],
        serde_json::json!(["delegation_completed"]),
        "case-level completion must reuse the real delegation basis, not a synthetic one"
    );
    assert_eq!(
        completion.payload["basis"],
        serde_json::json!([
            "authority_allow",
            "delegation_completed",
            "agent_output_mismatch"
        ])
    );
    let _ = task.await;
}

/// M13 T16 step 2: one transcript, both retention modes — the committed
/// redacted digest must be identical regardless of mode, full mode must
/// store a public plaintext artifact (still redacted), and summarized mode
/// must store a sealed ciphertext artifact that never exposes the plaintext.
#[tokio::test]
async fn t16_redaction_digest_identical_across_full_and_summarized_modes() {
    let response = r#"{"choices":[{"message":{"content":"secret test-secret leaked? no: redacted-check"}}],"usage":{"total_tokens":1}}"#;
    let root = tempfile::tempdir().unwrap();
    let policy = policy(root.path(), true, true);

    let (address, _connections, task) = stub(response).await;
    let (res, _reads) = resolver();
    let full_outcome = delegation::execute(
        &config(root.path(), endpoint(address.port())),
        delegation::DelegationRequest {
            endpoint_id: "local-test",
            instruction: "report",
            max_turns: 1,
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
            transcript_retention: sea_forge_agent::TranscriptRetentionMode::Full,
            ..Default::default()
        },
        &res,
    )
    .await
    .unwrap();
    let _ = task.await;

    let (address2, _connections2, task2) = stub(response).await;
    let (res2, _reads2) = resolver();
    let summarized_outcome = delegation::execute(
        &config(root.path(), endpoint(address2.port())),
        delegation::DelegationRequest {
            endpoint_id: "local-test",
            instruction: "report",
            max_turns: 1,
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
            transcript_retention: sea_forge_agent::TranscriptRetentionMode::Summarized,
            ..Default::default()
        },
        &res2,
    )
    .await
    .unwrap();
    let _ = task2.await;

    assert_eq!(
        full_outcome.transcript_sha256, summarized_outcome.transcript_sha256,
        "the same redacted transcript must hash identically regardless of retention mode"
    );
    assert_eq!(
        summarized_outcome.settlement,
        sea_forge_core::types::SettlementStatus::Accepted,
        "a normal seal must verify and never block settlement"
    );

    let full_hex = full_outcome
        .transcript_sha256
        .as_deref()
        .unwrap()
        .trim_start_matches("sha256:")
        .to_string();
    let full_artifact = root
        .path()
        .join("runs")
        .join(&full_outcome.run_id)
        .join(format!("transcript-{full_hex}.jsonl"));
    assert!(
        full_artifact.is_file(),
        "full mode stores a public plaintext artifact"
    );
    let full_text = fs::read_to_string(&full_artifact).unwrap();
    assert!(
        !full_text.contains("test-secret"),
        "the credential must be redacted even in full (public) mode"
    );

    let summarized_hex = summarized_outcome
        .transcript_sha256
        .as_deref()
        .unwrap()
        .trim_start_matches("sha256:")
        .to_string();
    let sealed_artifact = root
        .path()
        .join("runs")
        .join(&summarized_outcome.run_id)
        .join(format!("transcript-{summarized_hex}.sealed"));
    assert!(
        sealed_artifact.is_file(),
        "summarized mode stores a sealed ciphertext artifact, not plaintext"
    );
    let raw = fs::read(&sealed_artifact).unwrap();
    assert!(
        std::str::from_utf8(&raw).is_err()
            || !std::str::from_utf8(&raw)
                .unwrap()
                .contains("redacted-check"),
        "sealed artifact must never expose the plaintext transcript content"
    );
}

/// M13 T16 step 3: case dispatch resolves retention through the full
/// precedence chain — plan item override outranks endpoint config, which
/// outranks the global `[agent]` default, which outranks the built-in
/// summarized default.
#[tokio::test]
async fn t16_case_dispatch_retention_precedence_item_endpoint_global_default() {
    async fn dispatch_and_get_artifact_extension(
        item_override: Option<&str>,
        endpoint_override: Option<sea_forge_agent::TranscriptRetentionMode>,
        global_default: sea_forge_agent::TranscriptRetentionMode,
    ) -> String {
        let (address, _connections, task) = stub(
            r#"{"choices":[{"message":{"content":"proof artifact committed"}}],"usage":{"total_tokens":1}}"#,
        )
        .await;
        let root = tempfile::tempdir().unwrap();
        let policy_path = policy(root.path(), true, true);
        let mut ep = endpoint(address.port());
        ep.credential_ref = None;
        ep.transcript_retention = endpoint_override;
        let plan = CasePlan {
            version: "0.2".into(),
            plan_id: "plan_t16".into(),
            case_id: "case_t16".into(),
            run_id: "run_t16".into(),
            intent_id: "int_t16".into(),
            items: vec![PlanItem {
                plan_item_id: "agent".into(),
                name: "agent_task".into(),
                operations: vec![Operation::AgentTask {
                    endpoint_ref: "local-test".into(),
                    instruction: "publish proof".into(),
                    max_turns: 1,
                    token_budget: None,
                    response_schema: None,
                    transcript_retention: item_override.map(str::to_string),
                }],
                entry_criteria: vec![],
                entry_criteria_mode: Default::default(),
                exit_criteria: vec![],
                settlement_criteria: SettlementCriteria {
                    agent_output_must_contain: Some("proof artifact committed".into()),
                    ..SettlementCriteria::default()
                },
                settlement_criteria_ref: None,
                item_kind: ItemKind::AgentTask,
                sandbox_class: None,
                parent_stage: None,
                markers: ItemMarkers {
                    required: true,
                    ..ItemMarkers::default()
                },
                max_instances: 1,
                depends_on: vec![],
                environment: None,
                proposed_by: None,
            }],
            template_ref: None,
            job_contract_ref: None,
        };
        let plan_path = root.path().join("plan.json");
        fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();

        let mut config = config(root.path(), ep);
        config.agent.transcript_retention = global_default;
        let state = std::sync::Arc::new(ServerState::new(config).unwrap());
        let request: Request = serde_json::from_str(&format!(
            r#"{{"verb":"submit","plan":{:?},"policy":{:?},"entity":"operator_local","process":"test"}}"#,
            plan_path.to_str().unwrap(),
            policy_path.to_str().unwrap(),
        ))
        .unwrap();
        let response = handle_request(request, &state).await;
        let case_id = response["case_id"].as_str().unwrap().to_string();

        let ledger =
            sea_forge_ledger::LedgerStream::open(root.path(), format!("case-{case_id}"), "test")
                .unwrap();
        let entries = ledger.read_entries().unwrap();
        let evidence = entries
            .iter()
            .find(|entry| entry.record_kind == "agent_task_evidence")
            .expect("agent_task_evidence must be committed");
        let artifact_ref = evidence.payload["artifact_ref"]
            .as_str()
            .expect("artifact_ref must be present")
            .to_string();
        let _ = task.await;
        artifact_ref
            .rsplit('.')
            .next()
            .expect("artifact_ref must have an extension")
            .to_string()
    }

    use sea_forge_agent::TranscriptRetentionMode::{Full, Summarized};

    // No override anywhere: summarized default.
    assert_eq!(
        dispatch_and_get_artifact_extension(None, None, Summarized).await,
        "sealed"
    );
    // Global default promotes to full with no endpoint/item override.
    assert_eq!(
        dispatch_and_get_artifact_extension(None, None, Full).await,
        "jsonl"
    );
    // Endpoint config outranks the (summarized) global default.
    assert_eq!(
        dispatch_and_get_artifact_extension(None, Some(Full), Summarized).await,
        "jsonl"
    );
    // Plan-item override outranks both endpoint (full) and global (full).
    assert_eq!(
        dispatch_and_get_artifact_extension(Some("summarized"), Some(Full), Full).await,
        "sealed"
    );
}

/// M13 T16 step 5: a sealed-transcript verification failure settles rejected
/// with a typed basis — it never degrades to a summary-only success. The
/// pure crypto tamper/wrong-key/missing-key/missing-ciphertext/restart/
/// no-plaintext-marker cases are unit-tested directly in
/// `transcript_seal.rs`; this proves the integration point (the settlement
/// result) itself fails closed.
#[tokio::test]
async fn t16_sealed_verification_failure_settles_rejected_never_summary_only_success() {
    let (address, _connections, task) = stub(
        r#"{"choices":[{"message":{"content":"proof artifact committed"}}],"usage":{"total_tokens":1}}"#,
    )
    .await;
    let root = tempfile::tempdir().unwrap();
    let policy = policy(root.path(), true, true);
    let (res, _reads) = resolver();
    // Force `seal_transcript` to fail: `.sea-forge/sealed` exists as a
    // *file*, so its `create_dir_all` for the key directory cannot succeed.
    fs::create_dir_all(root.path().join(".sea-forge")).unwrap();
    fs::write(root.path().join(".sea-forge").join("sealed"), b"not a dir").unwrap();

    let outcome = delegation::execute(
        &config(root.path(), endpoint(address.port())),
        delegation::DelegationRequest {
            endpoint_id: "local-test",
            instruction: "publish proof",
            max_turns: 1,
            policy_path: policy.to_str().unwrap(),
            entity: "operator_local",
            process: "test",
            transcript_retention: sea_forge_agent::TranscriptRetentionMode::Summarized,
            ..Default::default()
        },
        &res,
    )
    .await
    .unwrap();

    assert_eq!(
        outcome.settlement,
        sea_forge_core::types::SettlementStatus::Rejected,
        "a sealed-verification failure must settle rejected, never summary-only success"
    );
    assert_eq!(
        outcome.error_class.as_deref(),
        Some("sealed_verification_failed")
    );
    assert!(outcome
        .basis
        .iter()
        .any(|b| b == "sealed_verification_failed"));
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
            ..Default::default()
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
            ..Default::default()
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
                    request_id: None,
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
                    request_id: None,
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
            request_id: None,
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
                    entry_criteria_mode: Default::default(),
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
                    proposed_by: None,
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
    // T13.2 Task 4: additive persisted ordinals. Each dispatch carries a
    // 1-based dispatch_ordinal and each settlement a settlement_ordinal;
    // both sequences are strictly monotonic over the persisted order.
    let dispatch_ordinals: Vec<u64> = dispatches
        .iter()
        .map(|event| event.payload["dispatch_ordinal"].as_u64().unwrap())
        .collect();
    let settlement_ordinals: Vec<u64> = settlements
        .iter()
        .map(|event| event.payload["settlement_ordinal"].as_u64().unwrap())
        .collect();
    assert_eq!(
        dispatch_ordinals,
        vec![1, 2, 3, 4, 5],
        "dispatch ordinals must be 1..=5 in persisted order: {dispatch_ordinals:?}"
    );
    assert_eq!(
        settlement_ordinals,
        vec![1, 2, 3, 4, 5],
        "settlement ordinals must be 1..=5 in persisted order: {settlement_ordinals:?}"
    );
    // Each settlement carries the run_id of the dispatch it settles, so
    // `ledger replay --case` can correlate dispatch and settlement by run_id.
    // Every persisted run_id appears in both an ItemActivated and a
    // SettlementRecorded payload, with one match per dispatch.
    let dispatch_run_ids: std::collections::HashMap<&str, &str> = dispatches
        .iter()
        .filter_map(|event| {
            let run = event.payload["run_id"].as_str()?;
            let item = event.plan_item_id.as_deref()?;
            Some((run, item))
        })
        .collect();
    assert_eq!(
        dispatch_run_ids.len(),
        dispatches.len(),
        "each dispatch must carry a distinct run_id: {:?}",
        dispatch_run_ids.keys().collect::<Vec<_>>()
    );
    for event in &settlements {
        let run_id = event.payload["run_id"]
            .as_str()
            .expect("settlement payload carries run_id");
        let item_id = event.plan_item_id.as_deref().unwrap();
        assert_eq!(
            dispatch_run_ids.get(run_id).copied(),
            Some(item_id),
            "settlement run_id {run_id} must correlate to a dispatch of the same item"
        );
    }
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
                request_id: None,
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
                entry_criteria_mode: Default::default(),
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
                proposed_by: None,
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
                    entry_criteria_mode: Default::default(),
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
                    proposed_by: None,
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
                    entry_criteria_mode: Default::default(),
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
                    proposed_by: None,
                },
                PlanItem {
                    plan_item_id: "human".into(),
                    name: "human".into(),
                    operations: vec![],
                    entry_criteria: vec![],
                    entry_criteria_mode: Default::default(),
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
                    proposed_by: None,
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
                    entry_criteria_mode: Default::default(),
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
                    proposed_by: None,
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

#[tokio::test]
async fn t13_2_non_executable_activation_returns_typed_error() {
    // The case engine emits CaseAction::Activate for any non-Milestone,
    // non-HumanTask item without manual_activation, including ItemKind::Stage.
    // The dispatcher must not panic; it returns a typed Input error instead.
    let root = tempfile::tempdir().unwrap();
    let policy_path = policy(root.path(), false, false);
    let state = Arc::new(
        ServerState::new(ServerConfig {
            root: root.path().to_path_buf(),
            max_concurrent_runs: 2,
            ..ServerConfig::default()
        })
        .unwrap(),
    );
    let plan_path = root.path().join("stage-item.json");
    fs::write(
        &plan_path,
        serde_json::to_vec(&CasePlan {
            version: "0.2".into(),
            plan_id: "plan_stage".into(),
            case_id: "case_placeholder".into(),
            run_id: "run_placeholder".into(),
            intent_id: "int_stage".into(),
            items: vec![PlanItem {
                plan_item_id: "stage".into(),
                name: "stage".into(),
                operations: vec![],
                entry_criteria: vec![],
                entry_criteria_mode: Default::default(),
                exit_criteria: vec![],
                settlement_criteria: SettlementCriteria::default(),
                settlement_criteria_ref: None,
                item_kind: ItemKind::Stage,
                sandbox_class: None,
                parent_stage: None,
                markers: sea_forge_core::types::ItemMarkers {
                    required: true,
                    ..Default::default()
                },
                max_instances: 1,
                depends_on: vec![],
                environment: None,
                proposed_by: None,
            }],
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
        "timeout": 5
    }))
    .unwrap();
    let response = handle_request(request, &state).await;
    assert!(response.get("error").is_some(), "{response}");
    assert!(
        response["error"]
            .as_str()
            .unwrap()
            .contains("non_executable"),
        "{response}"
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
            entry_criteria_mode: Default::default(),
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
            proposed_by: None,
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
