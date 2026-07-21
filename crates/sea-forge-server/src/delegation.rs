//! Governed `agent_task` delegation service (spec-agent-orchestration §10.2, §16.1).
//!
//! Executes a multi-turn agent delegation as an ordinary governed run:
//! intent → plan → exact authority → credential → delegation loop →
//! transcript evidence → settlement.

use crate::agent_probe::{commit_view, resolve_policy_path, CredentialResolver};
use crate::ServerConfig;
use chrono::Utc;
use sea_forge_agent::{
    run_delegation, AnthropicProvider, DelegationConfig, OpenAiCompatibleProvider, ProviderKind,
};
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::{
    errors::ForgeError,
    ids::{case_id, random_id, run_id},
    types::{
        Actor, ActorRole, AuthorityAction, CasePlan, DelegationTermination, Intent, ItemKind,
        Operation, PlanItem, SettlementCriteria, SettlementEvent, SettlementStatus,
        TranscriptEvidence, Verdict,
    },
};
use sea_forge_ledger::LedgerStream;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::Path;
use zeroize::Zeroizing;

/// Governed delegation request.
#[derive(Clone, Debug)]
pub struct DelegationRequest<'a> {
    pub endpoint_id: &'a str,
    pub instruction: &'a str,
    pub model: Option<&'a str>,
    pub max_turns: u32,
    pub token_budget: Option<u64>,
    pub criteria: SettlementCriteria,
    pub policy_path: &'a str,
    pub entity: &'a str,
    pub process: &'a str,
}

/// Serializable outcome for the socket response and tests.
#[derive(Clone, Debug, Serialize)]
pub struct DelegationResult {
    pub endpoint: String,
    pub run_id: String,
    pub settlement: SettlementStatus,
    pub termination: Option<String>,
    pub transcript_sha256: Option<String>,
    pub turns_used: u32,
    pub error_class: Option<String>,
}

#[derive(Serialize)]
struct DelegationSettlement<'a> {
    #[serde(flatten)]
    settlement: &'a SettlementEvent,
    case_id: &'a str,
    item_id: &'a str,
}

#[derive(Serialize)]
struct DelegationTranscriptEvidence<'a> {
    #[serde(flatten)]
    evidence: &'a TranscriptEvidence,
    case_id: &'a str,
    item_id: &'a str,
}

#[derive(Serialize)]
struct DelegationRejectionEvidence<'a> {
    endpoint_ref: &'a str,
    termination: &'a str,
    case_id: &'a str,
    item_id: &'a str,
    run_id: &'a str,
}

/// Identity allocated for one delegation episode.
pub struct DelegationEpisodeContext<'a> {
    case_id: &'a str,
    item_id: &'a str,
    run_id: &'a str,
}

impl<'a> DelegationEpisodeContext<'a> {
    pub fn standalone(case_id: &'a str, run_id: &'a str) -> Self {
        Self {
            case_id,
            item_id: "item_agent_task",
            run_id,
        }
    }

    pub fn planned(case_id: &'a str, item_id: &'a str, run_id: &'a str) -> Self {
        Self {
            case_id,
            item_id,
            run_id,
        }
    }
}

pub async fn execute(
    config: &ServerConfig,
    request: DelegationRequest<'_>,
    resolver: &dyn CredentialResolver,
) -> Result<DelegationResult, ForgeError> {
    let case = case_id()?;
    let run = run_id()?;
    execute_with_control(
        config,
        request,
        resolver,
        DelegationEpisodeContext::standalone(&case, &run),
        || false,
    )
    .await
}

/// Execute a delegation with a caller-owned run identity and cancellation projection.
pub async fn execute_with_control(
    config: &ServerConfig,
    request: DelegationRequest<'_>,
    resolver: &dyn CredentialResolver,
    episode: DelegationEpisodeContext<'_>,
    cancel: impl Fn() -> bool + Send + Sync + 'static,
) -> Result<DelegationResult, ForgeError> {
    let endpoint_id = request.endpoint_id;
    let instruction = request.instruction;

    if instruction.is_empty() || instruction.len() > 1_048_576 {
        return Err(ForgeError::Input(
            "agent_task instruction must be 1..1048576 bytes".into(),
        ));
    }
    if request.max_turns == 0 {
        return Err(ForgeError::Input(
            "agent_task max_turns must be >= 1".into(),
        ));
    }

    let endpoint = config
        .agent
        .endpoint(endpoint_id)
        .ok_or_else(|| ForgeError::Config {
            class: "missing_config_error",
            path: config.root.join("server.yaml"),
            message: format!("agent endpoint '{endpoint_id}' not found"),
        })?;
    let snapshot = endpoint.snapshot().map_err(|message| ForgeError::Config {
        class: "schema_error",
        path: config.root.join("server.yaml"),
        message,
    })?;
    let model = request.model.unwrap_or(&snapshot.model).to_owned();
    if model.is_empty() {
        return Err(ForgeError::Input(
            "agent_task model must not be empty".into(),
        ));
    }

    let run = episode.run_id;
    let case = episode.case_id;
    let item = episode.item_id;
    let ledger = LedgerStream::open(&config.root, format!("case-{case}"), "sea-forge-agent")?;

    // Intent → plan → authority chain.
    let intent = Intent {
        intent_id: random_id("int")?,
        summary: format!("Delegate to agent endpoint {endpoint_id}"),
        actor_id: request.entity.into(),
        process_id: request.process.into(),
        created_at: Utc::now().to_rfc3339(),
    };
    let intent_ref = commit_view(
        &ledger,
        "intent",
        vec![case.into(), run.into()],
        &intent,
        &config.root.join("runs").join(&run).join("intent.json"),
        vec![],
    )?;

    let instruction_hash = format!("sha256:{:x}", Sha256::digest(instruction.as_bytes()));
    let operation = Operation::AgentTask {
        endpoint_ref: endpoint_id.into(),
        instruction: instruction.into(),
        max_turns: request.max_turns,
        token_budget: request.token_budget,
        response_schema: None,
        transcript_retention: None,
    };
    let plan = CasePlan {
        version: "0.2".into(),
        plan_id: random_id("plan")?,
        case_id: case.into(),
        run_id: run.into(),
        intent_id: intent.intent_id.clone(),
        items: vec![PlanItem {
            plan_item_id: item.into(),
            name: "agent_task".into(),
            operations: vec![operation],
            entry_criteria: vec![],
            exit_criteria: vec![],
            settlement_criteria: request.criteria.clone(),
            settlement_criteria_ref: None,
            item_kind: ItemKind::AgentTask,
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
    let plan_ref = commit_view(
        &ledger,
        "plan",
        vec![case.into(), run.into()],
        &plan,
        &config.root.join("runs").join(&run).join("plan.json"),
        vec![intent_ref.entry_ulid().into()],
    )?;

    let action = action_for_delegation(
        &snapshot,
        endpoint_id,
        &model,
        &instruction_hash,
        request.max_turns,
        request.token_budget,
    );
    let policy = resolve_policy_path(&config.root, request.policy_path);
    let bundle = AuthorityPolicyBundle::load(&policy)?;
    let engine = PolicyAuthorityEngine::new(bundle.clone())?;
    let actor = Actor {
        actor_id: request.entity.into(),
        role: ActorRole::Operator,
    };
    let binding = bundle.resolve_identity(&actor.actor_id, actor.role.clone());
    let decision = engine.evaluate(AuthorityEvaluation {
        actor: &actor,
        binding,
        run_id: run,
        case_id: case,
        plan_item_id: item,
        sequence: 1,
        action: &action,
        workspace_root: &config.root,
        evidence_refs: vec![intent_ref.entry_ulid().into(), plan_ref.entry_ulid().into()],
        artifacts_root: None,
        timeout_secs: Some(snapshot.timeout.as_secs()),
        env_keys: Default::default(),
        domainforge_candidate: None,
        environment: None,
    })?;
    let decision_ref = commit_view(
        &ledger,
        "authority_decision",
        vec![run.into(), item.into()],
        &decision,
        &config.root.join("runs").join(&run).join("authority.json"),
        vec![intent_ref.entry_ulid().into(), plan_ref.entry_ulid().into()],
    )?;

    if decision.verdict != Verdict::Allow {
        return finish_rejected(
            &ledger,
            &config.root,
            case,
            item,
            run,
            endpoint_id,
            decision_ref.entry_ulid(),
            "authority_denied",
        );
    }
    let grant = engine.grant(&decision, &decision_ref, &action, None)?;
    grant.authorize(&action, run, item, &config.root)?;

    // Credential resolution via secret_access authority.
    let credential = match snapshot.credential_ref.as_deref() {
        Some(reference) => {
            let secret_action = AuthorityAction::Reserved {
                resource_type: "secret_access".into(),
                resource_id: reference.into(),
                parameters: json!({"credential_ref": reference}),
            };
            let secret_binding = bundle.resolve_identity(&actor.actor_id, actor.role.clone());
            let secret_decision = engine.evaluate(AuthorityEvaluation {
                actor: &actor,
                binding: secret_binding,
                run_id: run,
                case_id: case,
                plan_item_id: item,
                sequence: 2,
                action: &secret_action,
                workspace_root: &config.root,
                evidence_refs: vec![decision_ref.entry_ulid().into()],
                artifacts_root: None,
                timeout_secs: Some(snapshot.timeout.as_secs()),
                env_keys: Default::default(),
                domainforge_candidate: None,
                environment: None,
            })?;
            let secret_ref = commit_view(
                &ledger,
                "authority_decision",
                vec![run.into(), item.into(), "secret_access".into()],
                &secret_decision,
                &config
                    .root
                    .join("runs")
                    .join(&run)
                    .join("secret-authority.json"),
                vec![decision_ref.entry_ulid().into()],
            )?;
            if secret_decision.verdict != Verdict::Allow {
                return finish_rejected(
                    &ledger,
                    &config.root,
                    case,
                    item,
                    run,
                    endpoint_id,
                    secret_ref.entry_ulid(),
                    "secret_access_denied",
                );
            }
            let secret_grant = engine.grant(&secret_decision, &secret_ref, &secret_action, None)?;
            secret_grant.authorize(&secret_action, run, item, &config.root)?;
            resolver.resolve(reference)?
        }
        None => Zeroizing::new(String::new()),
    };

    // Build provider and run the delegation loop.
    let delegation_config = DelegationConfig {
        model: model.clone(),
        instruction: instruction.into(),
        max_turns: request.max_turns,
        token_budget: request.token_budget,
        max_output_tokens: Some(snapshot.max_response_bytes.min(4096) as u32),
    };

    let outcome = match &snapshot.kind {
        ProviderKind::OpenAiCompatible => {
            let provider = OpenAiCompatibleProvider::new(snapshot.clone()).map_err(|message| {
                ForgeError::Config {
                    class: "agent_endpoint_error",
                    path: config.root.join("server.yaml"),
                    message,
                }
            })?;
            run_delegation(&provider, credential, &delegation_config, &cancel)
                .await
                .map_err(|e| ForgeError::Internal(e.to_string()))?
        }
        ProviderKind::Anthropic => {
            let provider =
                AnthropicProvider::new(snapshot.clone()).map_err(|message| ForgeError::Config {
                    class: "agent_endpoint_error",
                    path: config.root.join("server.yaml"),
                    message,
                })?;
            run_delegation(&provider, credential, &delegation_config, &cancel)
                .await
                .map_err(|e| ForgeError::Internal(e.to_string()))?
        }
        ProviderKind::Acp => {
            return finish_rejected(
                &ledger,
                &config.root,
                case,
                item,
                run,
                endpoint_id,
                decision_ref.entry_ulid(),
                "unsupported_kind_error",
            );
        }
    };

    // Persist the redacted canonical transcript artifact in full mode.
    // ponytail: only full retention is supported; summarized mode is
    // intentionally not implemented because digest recomputation is not
    // independently verifiable (spec §7.4).
    let artifact_ref = if outcome.transcript.is_empty() {
        None
    } else {
        let hex = outcome
            .transcript_sha256
            .strip_prefix("sha256:")
            .unwrap_or(&outcome.transcript_sha256);
        let artifact_path = config
            .root
            .join("runs")
            .join(run)
            .join(format!("transcript-{hex}.jsonl"));
        let jsonl = outcome
            .transcript
            .iter()
            .filter_map(|entry| serde_json::to_string(entry).ok())
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::create_dir_all(artifact_path.parent().unwrap())
            .map_err(|e| ForgeError::io("create transcript artifact parent", e))?;
        std::fs::write(&artifact_path, jsonl)
            .map_err(|e| ForgeError::io("write transcript artifact", e))?;
        Some(
            artifact_path
                .strip_prefix(&config.root)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| format!("runs/{run}/transcript-{hex}.jsonl")),
        )
    };

    // Record transcript evidence and settle.
    let evidence = TranscriptEvidence {
        run_id: run.into(),
        endpoint_ref: endpoint_id.into(),
        turns_used: outcome.turns_used,
        termination: outcome.termination.clone(),
        transcript_sha256: outcome.transcript_sha256.clone(),
        summary: outcome.summary.clone(),
        artifact_ref: artifact_ref.clone(),
        harvested_refs: vec![],
    };
    let evidence_ref = commit_view(
        &ledger,
        "agent_task_evidence",
        vec![case.into(), run.into(), item.into()],
        &DelegationTranscriptEvidence {
            evidence: &evidence,
            case_id: case,
            item_id: item,
        },
        &config
            .root
            .join("runs")
            .join(&run)
            .join("transcript-evidence.json"),
        vec![decision_ref.entry_ulid().into()],
    )?;

    let (status, basis, criteria_error) = match outcome.termination {
        DelegationTermination::Completed => match sea_forge_settlement::evaluate_agent_output(
            outcome.final_output.as_deref().unwrap_or_default(),
            request.criteria.agent_output_must_contain.as_deref(),
        ) {
            Some(false) => (
                SettlementStatus::Rejected,
                vec!["authority_allow".into(), "agent_output_mismatch".into()],
                Some("agent_output_mismatch".into()),
            ),
            _ => (
                SettlementStatus::Accepted,
                vec!["authority_allow".into(), "delegation_completed".into()],
                None,
            ),
        },
        DelegationTermination::TurnCapExceeded => (
            SettlementStatus::Rejected,
            vec!["turn_cap_exceeded".into()],
            None,
        ),
        DelegationTermination::Cancelled => {
            (SettlementStatus::Rejected, vec!["cancelled".into()], None)
        }
        DelegationTermination::EndpointError => (
            SettlementStatus::Rejected,
            vec!["agent_endpoint_error".into()],
            None,
        ),
        DelegationTermination::AcpDisconnect => (
            SettlementStatus::Rejected,
            vec!["acp_disconnect".into()],
            None,
        ),
    };

    let settlement = SettlementEvent {
        version: "0.2".into(),
        settlement_id: random_id("set")?,
        run_id: run.into(),
        status: status.clone(),
        basis,
        review_required: false,
        settled_at: Utc::now().to_rfc3339(),
        criteria_ref: None,
    };
    commit_view(
        &ledger,
        "settlement",
        vec![run.into(), item.into()],
        &DelegationSettlement {
            settlement: &settlement,
            case_id: case,
            item_id: item,
        },
        &config.root.join("runs").join(&run).join("settlement.json"),
        vec![evidence_ref.entry_ulid().into()],
    )?;

    Ok(DelegationResult {
        endpoint: endpoint_id.into(),
        run_id: run.into(),
        settlement: status,
        termination: Some(termination_str(&outcome.termination)),
        transcript_sha256: Some(outcome.transcript_sha256),
        turns_used: outcome.turns_used,
        error_class: criteria_error.or(outcome.error_subcode),
    })
}

fn termination_str(t: &DelegationTermination) -> String {
    match t {
        DelegationTermination::Completed => "completed",
        DelegationTermination::TurnCapExceeded => "turn_cap_exceeded",
        DelegationTermination::Cancelled => "cancelled",
        DelegationTermination::EndpointError => "endpoint_error",
        DelegationTermination::AcpDisconnect => "acp_disconnect",
    }
    .into()
}

fn action_for_delegation(
    endpoint: &sea_forge_agent::EndpointSnapshot,
    endpoint_ref: &str,
    model: &str,
    instruction_hash: &str,
    max_turns: u32,
    token_budget: Option<u64>,
) -> AuthorityAction {
    AuthorityAction::AgentTask {
        endpoint_ref: endpoint_ref.into(),
        descriptor_config_sha256: endpoint.descriptor_config_sha256.clone(),
        provider_kind: match &endpoint.kind {
            ProviderKind::OpenAiCompatible => "openai_compatible",
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::Acp => "acp",
        }
        .into(),
        scheme: endpoint.base_url.scheme().into(),
        host: endpoint.base_url.host_str().unwrap_or_default().into(),
        port: endpoint
            .base_url
            .port_or_known_default()
            .unwrap_or_default(),
        path: endpoint.base_url.path().into(),
        model: model.into(),
        max_request_bytes: endpoint.max_request_bytes as u64,
        max_response_bytes: endpoint.max_response_bytes as u64,
        timeout_secs: endpoint.timeout.as_secs(),
        credential_ref: endpoint.credential_ref.clone(),
        instruction_sha256: instruction_hash.into(),
        max_turns,
        token_budget,
    }
}

fn finish_rejected(
    ledger: &LedgerStream,
    root: &Path,
    case: &str,
    item: &str,
    run: &str,
    endpoint: &str,
    authority_ref: &str,
    error_class: impl Into<String>,
) -> Result<DelegationResult, ForgeError> {
    let error_class = error_class.into();
    let evidence = DelegationRejectionEvidence {
        endpoint_ref: endpoint,
        termination: &error_class,
        case_id: case,
        item_id: item,
        run_id: run,
    };
    let evidence_ref = commit_view(
        ledger,
        "agent_task_evidence",
        vec![case.into(), run.into(), item.into()],
        &evidence,
        &root.join("runs").join(run).join("transcript-evidence.json"),
        vec![authority_ref.into()],
    )?;
    let settlement = SettlementEvent {
        version: "0.2".into(),
        settlement_id: random_id("set")?,
        run_id: run.into(),
        status: SettlementStatus::Rejected,
        basis: vec![error_class.clone()],
        review_required: false,
        settled_at: Utc::now().to_rfc3339(),
        criteria_ref: None,
    };
    commit_view(
        ledger,
        "settlement",
        vec![run.into(), item.into()],
        &DelegationSettlement {
            settlement: &settlement,
            case_id: case,
            item_id: item,
        },
        &root.join("runs").join(run).join("settlement.json"),
        vec![evidence_ref.entry_ulid().into()],
    )?;
    Ok(DelegationResult {
        endpoint: endpoint.into(),
        run_id: run.into(),
        settlement: SettlementStatus::Rejected,
        termination: None,
        transcript_sha256: None,
        turns_used: 0,
        error_class: Some(error_class),
    })
}
