use chrono::Utc;
use sea_forge_agent::{acp::KNOWN_TOOL_KINDS, AgentConfig, AgentEndpointConfig, ProviderKind};
use sea_forge_core::{
    ids::{case_id, run_id},
    types::{ApprovalRequest, ApprovalStatus, SettlementCriteria, SettlementStatus},
};
use sea_forge_ledger::LedgerStream;
use sea_forge_server::{
    agent_probe::EnvironmentCredentialResolver,
    delegation::{
        self, AcpApprovalBroker, DelegationEpisodeContext, DelegationRequest, DelegationResult,
    },
    ServerConfig, ServerState,
};
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::Path;

const ENDPOINT: &str = "acp-fixture";

fn write_rpc(value: Value) {
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "\n{value}").unwrap();
    stdout.flush().unwrap();
}

fn finish_prompt(prompt_id: u64, session_id: &str, output: &str) {
    write_rpc(json!({
        "jsonrpc": "2.0",
        "method": "session/update",
            "params": {
                "sessionId": session_id,
                "update": {
                    "sessionUpdate": "agent_message_chunk",
                    "content": {"type": "text", "text": output}
                }
            }
    }));
    write_rpc(json!({
        "jsonrpc": "2.0",
        "id": prompt_id,
        "result": {"stopReason": "end_turn"}
    }));
}

/// Scripted ACP child used by the portable M16 tests. The parent process
/// launches this same test binary with `--exact acp_fixture_child --nocapture`.
#[test]
fn acp_fixture_child() {
    if std::env::var("SEA_ACP_FIXTURE").as_deref() != Ok("1") {
        return;
    }
    assert!(std::env::var_os("HOME").is_none(), "parent HOME leaked");
    assert!(
        std::env::current_dir()
            .unwrap()
            .ends_with(Path::new("workspace")),
        "ACP child cwd is not the run workspace"
    );

    let mode = std::env::var("FIXTURE_MODE").unwrap_or_else(|_| "normal".into());
    let tool_kind = std::env::var("FIXTURE_TOOL_KIND").unwrap_or_else(|_| "read".into());
    let session_id = "sess-portable-m16";
    let mut prompt_id = None;
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let Ok(message) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let id = message.get("id").and_then(Value::as_u64);
        match message.get("method").and_then(Value::as_str) {
            Some("initialize") => write_rpc(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": if mode == "protocol_bad" { 999 } else { 1 },
                    "agentCapabilities": {"loadSession": true}
                }
            })),
            Some("session/new") => {
                let declared = message["params"]["cwd"].as_str().unwrap_or_default();
                assert!(declared.ends_with("workspace"), "session/new cwd drifted");
                assert!(message["params"]["mcpServers"].is_array());
                write_rpc(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {"sessionId": session_id}
                }));
            }
            Some("session/load") => {
                assert_eq!(message["params"]["sessionId"], session_id);
                let declared = message["params"]["cwd"].as_str().unwrap_or_default();
                assert!(declared.ends_with("workspace"), "session/load cwd drifted");
                assert!(message["params"]["mcpServers"].is_array());
                write_rpc(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {}
                }));
            }
            Some("session/prompt") => {
                prompt_id = id;
                assert!(message["params"]["prompt"].is_array());
                if mode.contains("jail") {
                    let escaped = std::env::temp_dir()
                        .join(format!("sea-forge-acp-escape-{}", std::process::id()));
                    assert!(
                        std::fs::write(&escaped, "forbidden").is_err(),
                        "jail permitted ACP child write outside run workspace"
                    );
                    std::fs::write("jail-proof.txt", "inside-workspace").unwrap();
                }
                if mode == "disconnect" {
                    write_rpc(json!({
                        "jsonrpc": "2.0",
                        "method": "session/update",
                        "params": {
                            "sessionId": session_id,
                            "update": {
                                "sessionUpdate": "agent_message_chunk",
                                "content": {"type": "text", "text": "partial"}
                            }
                        }
                    }));
                    return;
                }
                if mode == "secret" {
                    // Emit a chunk carrying full plaintext sentinels so the ACP
                    // transcript path must redact them before hashing/storage.
                    finish_prompt(
                        prompt_id.unwrap(),
                        session_id,
                        "leaking password=hunter2 and API_KEY=sk-live-abc123 done", // gitleaks:allow
                    );
                    continue;
                }
                write_rpc(json!({
                    "jsonrpc": "2.0",
                    "method": "session/update",
                    "params": {
                        "sessionId": session_id,
                        "update": {
                            "sessionUpdate": "tool_call",
                            "toolCallId": "tool-1",
                            "title": "portable tool",
                            "kind": tool_kind,
                            "status": "pending",
                            "rawInput": {"path": "/tmp/portable"}
                        }
                    }
                }));
                write_rpc(json!({
                    "jsonrpc": "2.0",
                    "id": 900,
                    "method": "session/request_permission",
                    "params": {
                        "sessionId": session_id,
                        "toolCall": {"toolCallId": "tool-1"},
                        "options": [
                            {"optionId": "allow-1", "kind": "allow_once"},
                            {"optionId": "deny-1", "kind": "reject_once"}
                        ]
                    }
                }));
            }
            None if id == Some(900) => {
                if mode.contains("escalate") {
                    write_rpc(json!({
                        "jsonrpc": "2.0",
                        "id": 901,
                        "method": "session/set_mode",
                        "params": {"sessionId": session_id, "modeId": "unrestricted"}
                    }));
                } else {
                    if mode.starts_with("swe_seed") {
                        std::fs::create_dir_all(".agent-harness/proofs").unwrap();
                        std::fs::write(".agent-harness/route.json", br#"{"route":"portable"}"#)
                            .unwrap();
                        std::fs::write(
                            ".agent-harness/proofs/completed.json",
                            br#"{"event":"ProofCompleted"}"#,
                        )
                        .unwrap();
                    }
                    finish_prompt(prompt_id.unwrap(), session_id, "done");
                }
            }
            None if id == Some(901) => {
                assert!(message.get("error").is_some(), "escalation was not refused");
                finish_prompt(prompt_id.unwrap(), session_id, "done");
            }
            _ => {}
        }
    }
}

fn write_policy(root: &Path, allow_permission: bool) {
    let permission = if allow_permission {
        "  - name: allow-acp-permission\n    verdict: allow\n    actor_role: operator\n    operation_kind: sandbox_execution\n"
    } else {
        ""
    };
    std::fs::write(
        root.join("policy.yaml"),
        format!(
            "version: \"0.1\"\npolicy_surfaces:\n  external_api:\n    mode: deny-by-default\n    allow_hosts: [argv]\nrules:\n  - name: allow-agent-task\n    verdict: allow\n    actor_role: operator\n    operation_kind: agent_task\n{permission}"
        ),
    )
    .unwrap();
}

fn write_escalating_policy(root: &Path) {
    std::fs::write(
        root.join("policy.yaml"),
        "version: \"0.1\"\npolicy_surfaces:\n  external_api:\n    mode: deny-by-default\n    allow_hosts: [argv]\nrules:\n  - name: allow-agent-task\n    verdict: allow\n    actor_role: operator\n    operation_kind: agent_task\n  - name: escalate-acp-permission\n    verdict: escalate\n    actor_role: operator\n    operation_kind: sandbox_execution\n    requires_approval: true\n",
    )
    .unwrap();
}

fn server_config(root: &Path, tool_kind: &str, mode: &str) -> ServerConfig {
    let mut agent = AgentConfig::default();
    let mut env = vec![
        "SEA_ACP_FIXTURE=1".into(),
        format!("FIXTURE_TOOL_KIND={tool_kind}"),
        format!("FIXTURE_MODE={mode}"),
    ];
    if mode.starts_with("swe_seed") {
        let repo = std::env::current_dir().unwrap();
        let commit = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&repo)
            .output()
            .unwrap();
        assert!(commit.status.success());
        env.push(format!("SEA_FORGE_SWE_SEED_REPO={}", repo.display()));
        let commit = if mode == "swe_seed_bad_commit" {
            "0000000000000000000000000000000000000000".into()
        } else {
            String::from_utf8_lossy(&commit.stdout).trim().to_string()
        };
        env.push(format!("SEA_FORGE_SWE_SEED_COMMIT={commit}"));
    }
    agent.endpoints.push(AgentEndpointConfig {
        id: ENDPOINT.into(),
        kind: ProviderKind::Acp,
        base_url: None,
        argv: vec![
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            "--exact".into(),
            "acp_fixture_child".into(),
            "--nocapture".into(),
            "--test-threads=1".into(),
        ],
        env,
        credential_ref: None,
        default_model: Some("fixture".into()),
        allow_loopback_test: false,
        max_request_bytes: 16_384,
        max_response_bytes: 16_384,
        timeout_secs: 3,
        status: None,
    });
    ServerConfig {
        root: root.to_path_buf(),
        agent,
        ..ServerConfig::default()
    }
}

async fn execute_fixture(config: &ServerConfig, case: &str) -> (String, DelegationResult) {
    let run = run_id().unwrap();
    let result = delegation::execute_with_control(
        config,
        DelegationRequest {
            endpoint_id: ENDPOINT,
            instruction: "portable ACP task",
            model: None,
            max_turns: 8,
            token_budget: None,
            criteria: SettlementCriteria::default(),
            policy_path: "policy.yaml",
            entity: "operator_local",
            process: "conformance_m16",
        },
        &EnvironmentCredentialResolver,
        DelegationEpisodeContext::planned(case, "item_acp", &run),
        || false,
    )
    .await
    .unwrap();
    (run, result)
}

async fn run_fixture(
    tool_kind: &str,
    mode: &str,
    allow_permission: bool,
) -> (tempfile::TempDir, String, String, DelegationResult) {
    let root = tempfile::tempdir().unwrap();
    write_policy(root.path(), allow_permission);
    let config = server_config(root.path(), tool_kind, mode);
    let case = case_id().unwrap();
    let (run, result) = execute_fixture(&config, &case).await;
    (root, case, run, result)
}

fn entries(root: &Path, case: &str) -> Vec<sea_forge_ledger::LedgerEntry> {
    let ledger = LedgerStream::open(root, format!("case-{case}"), "m16-test").unwrap();
    ledger.verify().unwrap();
    ledger.read_entries().unwrap()
}

#[tokio::test]
async fn t16_1_portable_acp_session_commits_transcript_evidence() {
    let (root, case, _, result) = run_fixture("read", "normal", true).await;
    assert_eq!(result.settlement, SettlementStatus::Accepted, "{result:?}");
    assert_eq!(result.termination.as_deref(), Some("completed"));
    assert_eq!(
        result.continuation_key.as_deref(),
        Some("sess-portable-m16")
    );
    let records = entries(root.path(), &case);
    assert!(records
        .iter()
        .any(|e| e.record_kind == "agent_task_evidence"));
    assert!(records.iter().any(|e| e.record_kind == "acp_session"));
}

#[tokio::test]
async fn t16_2_permission_allow_and_deny_are_recorded_and_session_survives() {
    for (allow, expected) in [(true, "allow"), (false, "deny")] {
        let (root, case, _, result) = run_fixture("read", "normal", allow).await;
        assert_eq!(result.settlement, SettlementStatus::Accepted);
        let records = entries(root.path(), &case);
        let permission = records
            .iter()
            .find(|e| e.record_kind == "permission_request")
            .unwrap();
        assert_eq!(permission.payload["verdict"], expected);
    }
}

#[tokio::test]
async fn t16_2_escalated_permission_suspends_and_resolves_same_session() {
    for granted in [true, false] {
        let root = tempfile::tempdir().unwrap();
        write_escalating_policy(root.path());
        let config = server_config(root.path(), "read", "normal");
        let case = case_id().unwrap();
        let run = run_id().unwrap();
        let broker = AcpApprovalBroker::default();
        let broker_for_task = broker.clone();
        let case_for_task = case.clone();
        let run_for_task = run.clone();
        let task = tokio::spawn(async move {
            delegation::execute_with_permission_broker(
                &config,
                DelegationRequest {
                    endpoint_id: ENDPOINT,
                    instruction: "portable ACP approval task",
                    model: None,
                    max_turns: 8,
                    token_budget: None,
                    criteria: SettlementCriteria::default(),
                    policy_path: "policy.yaml",
                    entity: "operator_local",
                    process: "conformance_m16",
                },
                &EnvironmentCredentialResolver,
                DelegationEpisodeContext::planned(&case_for_task, "item_acp", &run_for_task),
                || false,
                Some(broker_for_task),
            )
            .await
            .unwrap()
        });

        let approval = loop {
            if let Ok(text) = std::fs::read_to_string(root.path().join("approvals.jsonl")) {
                if let Some(line) = text.lines().next() {
                    break serde_json::from_str::<ApprovalRequest>(line).unwrap();
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        };
        assert_eq!(approval.status, ApprovalStatus::Pending);
        let mut resolution = approval.clone();
        resolution.status = if granted {
            ApprovalStatus::Approved
        } else {
            ApprovalStatus::Rejected
        };
        resolution.resolved_by = Some("security_officer".into());
        resolution.resolved_at = Some(Utc::now().to_rfc3339());
        let ledger = LedgerStream::open(root.path(), format!("case-{case}"), "m16-test").unwrap();
        ledger
            .commit_typed(
                "approval_resolution",
                vec![case.clone(), resolution.approval_id.clone()],
                &resolution,
                vec![],
            )
            .unwrap();
        assert!(broker.resolve(&approval.approval_id).await);
        assert!(!broker.resolve(&approval.approval_id).await);
        let result = task.await.unwrap();
        assert_eq!(result.settlement, SettlementStatus::Accepted);
        let records = entries(root.path(), &case);
        assert!(records
            .iter()
            .any(|entry| entry.record_kind == "permission_resolution"
                && entry.payload["granted"] == granted));
        assert!(records
            .iter()
            .any(|entry| entry.record_kind == "approval_resolution"));
    }
}

#[tokio::test]
async fn t16_2_escalated_permission_timeout_denies_and_session_continues() {
    let root = tempfile::tempdir().unwrap();
    write_escalating_policy(root.path());
    let mut config = server_config(root.path(), "read", "normal");
    config.agent.endpoints[0].timeout_secs = 1;
    let case = case_id().unwrap();
    let run = run_id().unwrap();
    let started = std::time::Instant::now();
    let result = delegation::execute_with_permission_broker(
        &config,
        DelegationRequest {
            endpoint_id: ENDPOINT,
            instruction: "portable ACP timeout task",
            model: None,
            max_turns: 8,
            token_budget: None,
            criteria: SettlementCriteria::default(),
            policy_path: "policy.yaml",
            entity: "operator_local",
            process: "conformance_m16",
        },
        &EnvironmentCredentialResolver,
        DelegationEpisodeContext::planned(&case, "item_acp", &run),
        || false,
        Some(AcpApprovalBroker::default()),
    )
    .await
    .unwrap();
    assert!(started.elapsed() >= std::time::Duration::from_millis(900));
    assert_eq!(result.settlement, SettlementStatus::Accepted);
    assert!(entries(root.path(), &case)
        .iter()
        .any(|entry| entry.record_kind == "permission_resolution"
            && entry.payload["granted"] == false));
}

#[tokio::test]
async fn t16_2_server_restart_rejects_orphaned_approval_episode_once() {
    let root = tempfile::tempdir().unwrap();
    write_escalating_policy(root.path());
    let mut config = server_config(root.path(), "read", "normal");
    config.agent.endpoints[0].timeout_secs = 30;
    let case = case_id().unwrap();
    let run = run_id().unwrap();
    let config_for_task = config.clone();
    let case_for_task = case.clone();
    let run_for_task = run.clone();
    let task = tokio::spawn(async move {
        delegation::execute_with_permission_broker(
            &config_for_task,
            DelegationRequest {
                endpoint_id: ENDPOINT,
                instruction: "restart during ACP approval",
                model: None,
                max_turns: 8,
                token_budget: None,
                criteria: SettlementCriteria::default(),
                policy_path: "policy.yaml",
                entity: "operator_local",
                process: "conformance_m16",
            },
            &EnvironmentCredentialResolver,
            DelegationEpisodeContext::planned(&case_for_task, "item_acp", &run_for_task),
            || false,
            Some(AcpApprovalBroker::default()),
        )
        .await
    });
    loop {
        if std::fs::read_to_string(root.path().join("approvals.jsonl"))
            .is_ok_and(|text| !text.is_empty())
        {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    task.abort();
    let _ = task.await;

    ServerState::new(config.clone()).unwrap();
    let first = entries(root.path(), &case);
    assert_eq!(
        first
            .iter()
            .filter(|entry| entry.record_kind == "settlement" && entry.payload["run_id"] == run)
            .count(),
        1
    );
    assert!(first.iter().any(|entry| entry.record_kind == "acp_session"
        && entry.payload["continuation_key"] == "sess-portable-m16"));
    ServerState::new(config).unwrap();
    let second = entries(root.path(), &case);
    assert_eq!(
        second
            .iter()
            .filter(|entry| entry.record_kind == "settlement" && entry.payload["run_id"] == run)
            .count(),
        1,
        "restart recovery must be idempotent"
    );
}

#[tokio::test]
async fn t16_3_every_v1_tool_kind_maps_and_unknown_kind_denies() {
    for kind in KNOWN_TOOL_KINDS {
        let (root, case, _, result) = run_fixture(kind, "normal", true).await;
        assert_eq!(result.settlement, SettlementStatus::Accepted, "{kind}");
        let records = entries(root.path(), &case);
        let permission = records
            .iter()
            .find(|e| e.record_kind == "permission_request")
            .unwrap();
        assert_eq!(permission.payload["tool_kind"], *kind);
        assert_eq!(permission.payload["verdict"], "allow");
    }

    let (root, case, _, result) = run_fixture("unmapped_kind", "normal", true).await;
    assert_eq!(result.settlement, SettlementStatus::Accepted);
    let records = entries(root.path(), &case);
    assert!(records
        .iter()
        .any(|entry| entry.record_kind == "permission_request"
            && entry.payload["verdict"] == "deny_unmapped"));
}

#[tokio::test]
async fn t16_4_minimal_env_workspace_cwd_and_escalation_refused() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("policy.yaml"),
        "version: \"0.1\"\npolicy_surfaces:\n  external_api:\n    mode: deny-by-default\n    allow_hosts: [argv]\nrules:\n  - name: allow-jailed-agent-task\n    verdict: allow\n    actor_role: operator\n    operation_kind: agent_task\n    sandbox_class: jail\n  - name: allow-acp-permission\n    verdict: allow\n    actor_role: operator\n    operation_kind: sandbox_execution\n",
    )
    .unwrap();
    let config = server_config(root.path(), "read", "escalate_jail");
    let case = case_id().unwrap();
    let (run, result) = execute_fixture(&config, &case).await;
    assert_eq!(result.settlement, SettlementStatus::Accepted);
    let artifact = std::fs::read_to_string(root.path().join("runs").join(&run).join(format!(
                "transcript-{}.jsonl",
                result
                    .transcript_sha256
                    .as_deref()
                    .unwrap()
                    .trim_start_matches("sha256:")
            )))
    .unwrap();
    assert!(artifact.contains("acp_method_unknown:session/set_mode"));
    assert!(root
        .path()
        .join("runs")
        .join(&run)
        .join("workspace/jail-proof.txt")
        .exists());
    assert!(entries(root.path(), &case)
        .iter()
        .any(|e| e.record_kind == "permission_request"));
}

/// Sentinel-aware transcript redaction on the ACP path: plaintext secrets in
/// agent output must be absent from the stored transcript artifact, its hash's
/// source bytes, the committed evidence records, and the settlement result.
/// This is the ACP analogue of the HTTP redaction guarantee — both flow through
/// the same `produce_transcript` choke point.
#[tokio::test]
async fn t16_7_acp_transcript_redaction_scrubs_sentinels_before_hash_and_storage() {
    let root = tempfile::tempdir().unwrap();
    write_policy(root.path(), true);
    let config = server_config(root.path(), "read", "secret");
    let case = case_id().unwrap();
    let (run, result) = execute_fixture(&config, &case).await;
    assert_eq!(result.settlement, SettlementStatus::Accepted, "{result:?}");

    // The stored artifact (the exact bytes the digest commits to) is clean.
    let sha = result.transcript_sha256.as_deref().unwrap();
    let artifact = std::fs::read_to_string(root.path().join("runs").join(&run).join(format!(
        "transcript-{}.jsonl",
        sha.trim_start_matches("sha256:")
    )))
    .unwrap();
    let lower = artifact.to_lowercase();
    assert!(
        !lower.contains("password") && !lower.contains("api_key"),
        "sentinel leaked in transcript artifact: {artifact}"
    );
    // Re-hashing the stored bytes reproduces the committed digest exactly.
    let rehash = format!(
        "sha256:{:x}",
        <sha2::Sha256 as sha2::Digest>::digest(artifact.as_bytes())
    );
    assert_eq!(rehash, sha, "stored artifact bytes diverge from digest");

    // Committed evidence/summary records carry no sentinel either.
    let records = entries(root.path(), &case);
    for entry in &records {
        let payload = entry.payload.to_string().to_lowercase();
        assert!(
            !payload.contains("password") && !payload.contains("api_key"),
            "sentinel leaked in {} payload",
            entry.record_kind
        );
    }
    assert!(records
        .iter()
        .any(|e| e.record_kind == "agent_task_evidence"));
}

#[tokio::test]
async fn t16_5_disconnect_preserves_partial_transcript_and_resumes_linked_episode() {
    let root = tempfile::tempdir().unwrap();
    write_policy(root.path(), true);
    let case = case_id().unwrap();
    let disconnected = server_config(root.path(), "read", "disconnect");
    let (run, result) = execute_fixture(&disconnected, &case).await;
    assert_eq!(result.settlement, SettlementStatus::Rejected);
    assert_eq!(result.termination.as_deref(), Some("acp_disconnect"));
    assert_eq!(
        result.continuation_key.as_deref(),
        Some("sess-portable-m16")
    );
    let records = entries(root.path(), &case);
    let session = records
        .iter()
        .find(|e| e.record_kind == "acp_session")
        .unwrap();
    assert_eq!(session.payload["continuation_key"], "sess-portable-m16");
    let evidence = records
        .iter()
        .find(|e| e.record_kind == "agent_task_evidence")
        .unwrap();
    let artifact = evidence.payload["artifact_ref"].as_str().unwrap();
    assert!(std::fs::read_to_string(root.path().join(artifact))
        .unwrap()
        .contains("partial"));
    assert!(root
        .path()
        .join("runs")
        .join(run)
        .join("settlement.json")
        .exists());

    let resumed = server_config(root.path(), "read", "normal");
    let (_, resumed_result) = execute_fixture(&resumed, &case).await;
    assert_eq!(resumed_result.settlement, SettlementStatus::Accepted);
    assert_eq!(
        resumed_result.continuation_key, result.continuation_key,
        "successor episode must load and ledger the same ACP session"
    );
    let records = entries(root.path(), &case);
    let linked_sessions: Vec<_> = records
        .iter()
        .filter(|entry| entry.record_kind == "acp_session")
        .collect();
    assert_eq!(linked_sessions.len(), 2);
    assert!(linked_sessions
        .iter()
        .all(|entry| entry.payload["continuation_key"] == "sess-portable-m16"));
}

#[tokio::test]
async fn t16_6_portable_swe_seed_artifacts_are_hashed_and_run_correlated() {
    let (root, case, run, result) = run_fixture("read", "swe_seed", true).await;
    assert_eq!(result.settlement, SettlementStatus::Accepted);
    let records = entries(root.path(), &case);
    let evidence = records
        .iter()
        .find(|entry| entry.record_kind == "agent_task_evidence")
        .unwrap();
    let refs = evidence.payload["harvested_refs"].as_array().unwrap();
    assert_eq!(refs.len(), 2);
    assert!(refs.iter().all(|value| {
        let value = value.as_str().unwrap();
        value.starts_with(&format!("{run}:.agent-harness/")) && value.contains(":sha256:")
    }));
    let correlation = records
        .iter()
        .find(|entry| entry.record_kind == "swe_seed_correlation")
        .unwrap();
    assert_eq!(correlation.payload["run_id"], run);
    assert_eq!(correlation.payload["harvested_refs"], json!(refs));
}

#[tokio::test]
async fn t16_6_swe_seed_commit_mismatch_rejects_but_preserves_transcript() {
    let (root, case, _, result) = run_fixture("read", "swe_seed_bad_commit", true).await;
    assert_eq!(result.settlement, SettlementStatus::Rejected);
    assert_eq!(
        result.error_class.as_deref(),
        Some("swe_seed_harvest_error")
    );
    let records = entries(root.path(), &case);
    let evidence = records
        .iter()
        .find(|entry| entry.record_kind == "agent_task_evidence")
        .unwrap();
    assert!(evidence.payload["artifact_ref"].as_str().is_some());
    assert!(evidence
        .payload
        .get("harvested_refs")
        .and_then(Value::as_array)
        .is_none_or(Vec::is_empty));
}

#[tokio::test]
async fn t16_7_unsupported_protocol_version_rejects_before_prompt() {
    let (root, case, _, result) = run_fixture("read", "protocol_bad", true).await;
    assert_eq!(result.settlement, SettlementStatus::Rejected);
    assert_eq!(result.termination.as_deref(), Some("endpoint_error"));
    assert!(entries(root.path(), &case)
        .iter()
        .all(|e| e.record_kind != "permission_request"));
}

fn real_config(root: &Path, extra_env: Vec<String>) -> ServerConfig {
    let argv: Vec<String> = serde_json::from_str(
        &std::env::var("SEA_FORGE_REAL_ACP_ARGV")
            .expect("SEA_FORGE_REAL_ACP_ARGV must be a JSON argv array"),
    )
    .unwrap();
    let mut env: Vec<String> = std::env::var("SEA_FORGE_REAL_ACP_ENV")
        .ok()
        .map(|value| serde_json::from_str(&value).unwrap())
        .unwrap_or_default();
    env.extend(extra_env);
    let mut agent = AgentConfig::default();
    agent.endpoints.push(AgentEndpointConfig {
        id: ENDPOINT.into(),
        kind: ProviderKind::Acp,
        base_url: None,
        argv,
        env,
        credential_ref: None,
        default_model: Some("configured-host".into()),
        allow_loopback_test: false,
        max_request_bytes: 1_048_576,
        max_response_bytes: 4_194_304,
        timeout_secs: 60,
        status: None,
    });
    ServerConfig {
        root: root.to_path_buf(),
        agent,
        ..ServerConfig::default()
    }
}

#[tokio::test]
#[ignore = "release gate: set SEA_FORGE_REAL_ACP_ARGV/SEA_FORGE_REAL_ACP_ENV"]
async fn t16_1_real_acp_host_release_gate() {
    let root = tempfile::tempdir().unwrap();
    write_policy(root.path(), true);
    let config = real_config(root.path(), Vec::new());
    config.agent.validate().unwrap();
    let case = case_id().unwrap();
    let (_, result) = execute_fixture(&config, &case).await;
    assert_eq!(result.settlement, SettlementStatus::Accepted, "{result:?}");
    assert!(entries(root.path(), &case)
        .iter()
        .any(|entry| entry.record_kind == "agent_task_evidence"));
}

#[tokio::test]
#[ignore = "release gate: configured SWE_SEED-projected ACP host required"]
async fn t16_6_real_swe_seed_release_gate() {
    let root = tempfile::tempdir().unwrap();
    write_policy(root.path(), true);
    let repo = std::env::var("SEA_FORGE_REAL_SWE_SEED_REPO")
        .expect("SEA_FORGE_REAL_SWE_SEED_REPO required");
    let commit = std::env::var("SEA_FORGE_REAL_SWE_SEED_COMMIT")
        .expect("SEA_FORGE_REAL_SWE_SEED_COMMIT required");
    let config = real_config(
        root.path(),
        vec![
            format!("SEA_FORGE_SWE_SEED_REPO={repo}"),
            format!("SEA_FORGE_SWE_SEED_COMMIT={commit}"),
        ],
    );
    config.agent.validate().unwrap();
    let case = case_id().unwrap();
    let (_, result) = execute_fixture(&config, &case).await;
    assert_eq!(result.settlement, SettlementStatus::Accepted, "{result:?}");
    let records = entries(root.path(), &case);
    let refs = records
        .iter()
        .find(|entry| entry.record_kind == "agent_task_evidence")
        .unwrap()
        .payload["harvested_refs"]
        .as_array()
        .unwrap()
        .clone();
    assert!(!refs.is_empty());
}
