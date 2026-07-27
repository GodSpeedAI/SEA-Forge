//! Governed `agent_task` delegation service (spec-agent-orchestration §10.2, §16.1).
//!
//! Executes a multi-turn agent delegation as an ordinary governed run:
//! intent → plan → exact authority → credential → delegation loop →
//! transcript evidence → settlement.

use crate::agent_probe::{commit_view, resolve_policy_path, CredentialResolver};
use crate::ServerConfig;
use chrono::Utc;
use sea_forge_agent::{
    deny_once, run_delegation, AcpPermissionMediator, AcpSession, AcpSpawn, AnthropicProvider,
    DelegationConfig, OpenAiCompatibleProvider, PermissionDecision, ProviderKind,
};
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::{
    errors::ForgeError,
    ids::{self, case_id, random_id, run_id},
    types::{
        Actor, ActorRole, ApprovalRequest, ApprovalStatus, AuthorityAction, CasePlan, Declarer,
        DelegationTermination, Intent, ItemKind, Operation, PlanItem, SettlementCriteria,
        SettlementCriteriaRecord, SettlementDeclarationRequest, SettlementEvent, SettlementStatus,
        SettlementStrength, TranscriptEvidence, TranscriptSummary, Verdict,
    },
    RECORD_VERSION,
};
use sea_forge_ledger::LedgerStream;
use sea_forge_settlement::SettlementAuthority as _;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex as AsyncMutex};
use zeroize::Zeroizing;

/// Governed delegation request.
#[derive(Clone, Debug, Default)]
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
    /// Optional JSON Schema (M13 T15, spec §7.3): a schema-valid final
    /// output becomes a named evidence field and can accept an otherwise
    /// cap-terminated episode; a schema-invalid one never accepts.
    pub response_schema: Option<&'a serde_json::Value>,
    /// Effective retention mode, already resolved through the full
    /// precedence chain by the caller (M13 T16, spec §8.1).
    pub transcript_retention: sea_forge_agent::TranscriptRetentionMode,
}

#[derive(Clone, Default)]
pub struct AcpApprovalBroker {
    pending: Arc<AsyncMutex<std::collections::HashMap<String, oneshot::Sender<()>>>>,
}

impl AcpApprovalBroker {
    async fn register(&self, approval_id: String) -> oneshot::Receiver<()> {
        let (sender, receiver) = oneshot::channel();
        self.pending.lock().await.insert(approval_id, sender);
        receiver
    }

    /// Resolve a suspended ACP permission exactly once. Returns false for an
    /// unknown or already-resolved approval id.
    pub async fn resolve(&self, approval_id: &str) -> bool {
        self.pending
            .lock()
            .await
            .remove(approval_id)
            .is_some_and(|sender| sender.send(()).is_ok())
    }
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
    /// ACP session id linking successor episodes after a disconnect (E17).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_key: Option<String>,
    /// The exact basis of the committed `SettlementEvent` (M13 T15). Callers
    /// that record their own completion (e.g. case dispatch) must reuse this
    /// rather than construct a synthetic basis.
    #[serde(default)]
    pub basis: Vec<String>,
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

/// Named evidence for `response_schema` validation (M13 T15, spec §7.3).
/// Records only the schema's own hash and a validity flag, never the raw
/// agent output, so a schema-invalid response never leaks its
/// non-conforming payload through this record.
#[derive(Serialize)]
struct ResponseSchemaEvidence<'a> {
    version: &'static str,
    case_id: &'a str,
    run_id: &'a str,
    item_id: &'a str,
    schema_sha256: &'a str,
    valid: bool,
}

#[derive(Serialize)]
struct AcpSessionRecord<'a> {
    version: &'static str,
    case_id: &'a str,
    run_id: &'a str,
    plan_item_id: &'a str,
    endpoint_ref: &'a str,
    continuation_key: &'a str,
    protocol_version: u32,
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
    execute_with_permission_broker(config, request, resolver, episode, cancel, None).await
}

pub async fn execute_with_permission_broker(
    config: &ServerConfig,
    request: DelegationRequest<'_>,
    resolver: &dyn CredentialResolver,
    episode: DelegationEpisodeContext<'_>,
    cancel: impl Fn() -> bool + Send + Sync + 'static,
    permission_broker: Option<AcpApprovalBroker>,
) -> Result<DelegationResult, ForgeError> {
    let endpoint_id = request.endpoint_id;
    let instruction = request.instruction;

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
    let model = check_preconditions(&snapshot, instruction, request.model, request.max_turns)
        .map_err(|unmet| ForgeError::Input(unmet.join("; ")))?;

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
        &config.root.join("runs").join(run).join("intent.json"),
        vec![],
    )?;

    let instruction_hash = format!("sha256:{:x}", Sha256::digest(instruction.as_bytes()));
    let operation = Operation::AgentTask {
        endpoint_ref: endpoint_id.into(),
        instruction: instruction.into(),
        max_turns: request.max_turns,
        token_budget: request.token_budget,
        response_schema: request.response_schema.cloned(),
        transcript_retention: Some(
            match request.transcript_retention {
                sea_forge_agent::TranscriptRetentionMode::Full => "full",
                sea_forge_agent::TranscriptRetentionMode::Summarized => "summarized",
            }
            .into(),
        ),
    };
    let mut plan_item = PlanItem {
        plan_item_id: item.into(),
        name: "agent_task".into(),
        operations: vec![operation],
        entry_criteria: vec![],
        entry_criteria_mode: Default::default(),
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
        proposed_by: None,
    };
    let criteria = sea_forge_planner::derive_from_intent(
        &intent,
        &plan_item,
        request.entity,
        &intent.created_at,
    )?;
    let criteria_ref = commit_view(
        &ledger,
        "settlement_criteria",
        vec![case.into(), run.into(), item.into()],
        &criteria,
        &config.root.join("runs").join(run).join("criteria.json"),
        vec![intent_ref.entry_ulid().into()],
    )?;
    plan_item.settlement_criteria_ref = Some(criteria.criteria_id.clone());
    let plan = CasePlan {
        version: "0.2".into(),
        plan_id: random_id("plan")?,
        case_id: case.into(),
        run_id: run.into(),
        intent_id: intent.intent_id.clone(),
        items: vec![plan_item],
        template_ref: None,
        job_contract_ref: None,
    };
    let plan_ref = commit_view(
        &ledger,
        "plan",
        vec![case.into(), run.into()],
        &plan,
        &config.root.join("runs").join(run).join("plan.json"),
        vec![
            intent_ref.entry_ulid().into(),
            criteria_ref.entry_ulid().into(),
        ],
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
        evidence_refs: vec![
            intent_ref.entry_ulid().into(),
            criteria_ref.entry_ulid().into(),
            plan_ref.entry_ulid().into(),
        ],
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
        &config.root.join("runs").join(run).join("authority.json"),
        vec![
            intent_ref.entry_ulid().into(),
            criteria_ref.entry_ulid().into(),
            plan_ref.entry_ulid().into(),
        ],
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
    let sandbox_class = grant.sandbox_class().to_string();
    grant.authorize(&action, run, item, &config.root)?;

    // Provider dispatch. ACP is a child-process executor with no ambient
    // credential; HTTP kinds resolve a credential via secret_access first.
    // Captured before dispatch for the SWE_SEED declaration request (M16
    // T18): `check_integrity` requires `criteria_declared_at <=
    // execution_started_at`, and `criteria.declared_at` was set above,
    // strictly before this point.
    let execution_started_at = Utc::now().to_rfc3339();
    let mut continuation_key: Option<String> = None;
    let mut harvested_refs: Vec<String> = Vec::new();
    let outcome: sea_forge_agent::DelegationOutcome = if snapshot.kind == ProviderKind::Acp {
        let (out, cont, harvested) = run_acp_episode(
            &config.root,
            &snapshot,
            &bundle,
            request.entity,
            request.process,
            run,
            case,
            item,
            instruction,
            request.max_turns,
            cancel,
            permission_broker,
            criteria.clone(),
            criteria_ref.clone(),
            &sandbox_class,
        )
        .await?;
        continuation_key = cont;
        harvested_refs = harvested;
        out
    } else {
        // Credential resolution via secret_access (HTTP only). A denial
        // commits rejection evidence + settlement and short-circuits, exactly
        // as in M13 — no provider call, no secret read.
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
                let secret_ledger =
                    LedgerStream::open(&config.root, format!("case-{case}"), "sea-forge-agent")?;
                let secret_ref = commit_view(
                    &secret_ledger,
                    "authority_decision",
                    vec![run.into(), item.into(), "secret_access".into()],
                    &secret_decision,
                    &config
                        .root
                        .join("runs")
                        .join(run)
                        .join("secret-authority.json"),
                    vec![decision_ref.entry_ulid().into()],
                )?;
                if secret_decision.verdict != Verdict::Allow {
                    return finish_rejected(
                        &secret_ledger,
                        &config.root,
                        case,
                        item,
                        run,
                        endpoint_id,
                        secret_ref.entry_ulid(),
                        "secret_access_denied",
                    );
                }
                let secret_grant =
                    engine.grant(&secret_decision, &secret_ref, &secret_action, None)?;
                secret_grant.authorize(&secret_action, run, item, &config.root)?;
                resolver.resolve(reference)?
            }
            None => Zeroizing::new(String::new()),
        };
        run_http_episode(
            &config.root,
            &snapshot,
            credential,
            run,
            case,
            item,
            instruction,
            request.max_turns,
            request.token_budget,
            cancel,
            model.clone(),
        )
        .await?
    };

    // Persist the redacted canonical transcript. Full mode stores it as a
    // public plaintext artifact; summarized mode seals it with
    // XChaCha20Poly1305 (ADR-002) and verifies the seal by decrypting it
    // immediately — a verification failure is captured here and forces the
    // settlement rejected below rather than degrading to a summary-only
    // success (M13 T16 step 4/5, spec §7.4).
    let (artifact_ref, sealed_verification_error) = if outcome.transcript.is_empty() {
        (None, None)
    } else {
        // Single producer: re-run the shared redactor/canonicalizer over the
        // (already-redacted) entries so the artifact bytes are the EXACT bytes
        // the digest commits to. `produce_transcript` is idempotent on
        // pre-redacted input; the sentinel corpus is applied once more as
        // defense in depth. This replaces the old divergent per-entry
        // `serde_json::to_string` serialization for both HTTP and ACP.
        let produced = sea_forge_agent::produce_transcript(&outcome.transcript, &[]);
        debug_assert_eq!(
            produced.sha256, outcome.transcript_sha256,
            "artifact bytes diverge from committed transcript digest"
        );
        let hex = outcome
            .transcript_sha256
            .strip_prefix("sha256:")
            .unwrap_or(&outcome.transcript_sha256);
        let relative = |path: &Path| -> String {
            path.strip_prefix(&config.root)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| path.to_string_lossy().into_owned())
        };
        match request.transcript_retention {
            sea_forge_agent::TranscriptRetentionMode::Full => {
                let artifact_path = config
                    .root
                    .join("runs")
                    .join(run)
                    .join(format!("transcript-{hex}.jsonl"));
                std::fs::create_dir_all(artifact_path.parent().unwrap())
                    .map_err(|e| ForgeError::io("create transcript artifact parent", e))?;
                std::fs::write(&artifact_path, &produced.canonical_bytes)
                    .map_err(|e| ForgeError::io("write transcript artifact", e))?;
                (Some(relative(&artifact_path)), None)
            }
            sea_forge_agent::TranscriptRetentionMode::Summarized => {
                let ciphertext_path = config
                    .root
                    .join("runs")
                    .join(run)
                    .join(format!("transcript-{hex}.sealed"));
                match crate::transcript_seal::seal_transcript(
                    &config.root,
                    run,
                    &ciphertext_path,
                    &produced.canonical_bytes,
                )
                .and_then(|sealed| {
                    crate::transcript_seal::verify_sealed_transcript(
                        &sealed,
                        &outcome.transcript_sha256,
                    )
                    .map(|()| sealed)
                }) {
                    Ok(sealed) => (Some(relative(&sealed.ciphertext_path)), None),
                    Err(_) => (
                        ciphertext_path
                            .try_exists()
                            .unwrap_or(false)
                            .then(|| relative(&ciphertext_path)),
                        Some("sealed_verification_failed".to_string()),
                    ),
                }
            }
        }
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
        harvested_refs,
    };
    let mut evidence_parents = vec![decision_ref.entry_ulid().into()];
    if let Some(key) = continuation_key.as_deref() {
        let session_ref = commit_view(
            &ledger,
            "acp_session",
            vec![case.into(), run.into(), item.into()],
            &AcpSessionRecord {
                version: "0.2",
                case_id: case,
                run_id: run,
                plan_item_id: item,
                endpoint_ref: endpoint_id,
                continuation_key: key,
                protocol_version: sea_forge_agent::ACP_PROTOCOL_VERSION,
            },
            &config.root.join("runs").join(run).join("acp-session.json"),
            vec![decision_ref.entry_ulid().into()],
        )?;
        evidence_parents.push(session_ref.entry_ulid().into());
    }
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
            .join(run)
            .join("transcript-evidence.json"),
        evidence_parents,
    )?;

    // Schema-valid final output becomes a named evidence field (spec §7.3).
    // The evidence records only the schema's own hash and a validity flag —
    // never the raw agent output — so a schema-invalid response never leaks
    // its non-conforming payload through this record (M13 T15 step 5).
    let schema_result = request.response_schema.map(|schema| {
        sea_forge_settlement::validate_response_schema(
            schema,
            outcome.final_output.as_deref().unwrap_or_default(),
        )
    });
    let schema_evidence_ref = match request.response_schema {
        Some(schema) => {
            let schema_sha256 = format!(
                "sha256:{:x}",
                Sha256::digest(serde_json::to_vec(schema).unwrap_or_default())
            );
            Some(commit_view(
                &ledger,
                "response_schema_evidence",
                vec![case.into(), run.into(), item.into()],
                &ResponseSchemaEvidence {
                    version: "0.2",
                    case_id: case,
                    run_id: run,
                    item_id: item,
                    schema_sha256: &schema_sha256,
                    valid: schema_result == Some(true),
                },
                &config
                    .root
                    .join("runs")
                    .join(run)
                    .join("response-schema-evidence.json"),
                vec![evidence_ref.entry_ulid().into()],
            )?)
        }
        None => None,
    };

    // Correlation is owned by the shared, idempotent reconciler (M16 T18):
    // it re-reads the case ledger this `agent_task_evidence` was just
    // committed to and joins it against whatever declarations already
    // exist (usually none yet — the production declaration ingress below
    // runs after settlement). The same reconciler runs again at server
    // startup and at read time, so a declaration that arrives later still
    // correlates without mutating this settlement.
    let mut settlement_parents = if evidence.harvested_refs.is_empty() {
        vec![evidence_ref.entry_ulid().into()]
    } else {
        let outcomes =
            crate::swe_seed_reconciliation::reconcile_swe_seed_declarations(&config.root, case)?;
        let correlation_ref = outcomes
            .into_iter()
            .find(|outcome| outcome.run_id == run)
            .map(|outcome| outcome.committed)
            .ok_or_else(|| {
                ForgeError::Internal("swe_seed correlation was not produced for this run".into())
            })?;
        vec![correlation_ref.entry_ulid().into()]
    };
    if let Some(schema_ref) = &schema_evidence_ref {
        settlement_parents.push(schema_ref.entry_ulid().into());
    }

    // Completion and cap-exhaustion share one evaluation: a schema-invalid
    // response never accepts; otherwise the declared agent-output literal
    // (when present) decides. `turn_cap_exceeded` always stays in the basis
    // so an accepted cap-terminated episode is still honestly labeled as one
    // (M13 T15 step 3 — cap termination accepts or rejects solely by
    // criteria, it never forces rejection nor hides how it terminated).
    let evaluate_output_criteria =
        |mut basis: Vec<String>| -> (SettlementStatus, Vec<String>, Option<String>) {
            if schema_result == Some(false) {
                basis.push("schema_invalid".into());
                return (
                    SettlementStatus::Rejected,
                    basis,
                    Some("schema_invalid".into()),
                );
            }
            match sea_forge_settlement::evaluate_agent_output(
                outcome.final_output.as_deref().unwrap_or_default(),
                request.criteria.agent_output_must_contain.as_deref(),
            ) {
                Some(false) => {
                    basis.push("agent_output_mismatch".into());
                    (
                        SettlementStatus::Rejected,
                        basis,
                        Some("agent_output_mismatch".into()),
                    )
                }
                _ => {
                    if schema_result == Some(true) {
                        basis.push("response_schema_valid".into());
                    }
                    (SettlementStatus::Accepted, basis, None)
                }
            }
        };

    let (status, basis, criteria_error) = if let Some(error_class) = &sealed_verification_error {
        // A failed seal-verification is a settlement-blocking failure on its
        // own, independent of termination/criteria — never degrade to a
        // summary-only success (M13 T16 step 5).
        (
            SettlementStatus::Rejected,
            vec![error_class.clone()],
            Some(error_class.clone()),
        )
    } else {
        match outcome.termination {
            DelegationTermination::Completed => evaluate_output_criteria(vec![
                "authority_allow".into(),
                "delegation_completed".into(),
            ]),
            DelegationTermination::TurnCapExceeded => {
                let has_criteria = request.criteria.agent_output_must_contain.is_some()
                    || request.response_schema.is_some();
                if has_criteria {
                    evaluate_output_criteria(vec!["turn_cap_exceeded".into()])
                } else {
                    (
                        SettlementStatus::Rejected,
                        vec!["turn_cap_exceeded".into()],
                        None,
                    )
                }
            }
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
        }
    };

    let settlement = SettlementEvent {
        version: "0.2".into(),
        settlement_id: random_id("set")?,
        run_id: run.into(),
        status: status.clone(),
        basis: basis.clone(),
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
        &config.root.join("runs").join(run).join("settlement.json"),
        settlement_parents,
    )?;

    // Production SWE_SEED declaration ingress (M16 T18). Best-effort and
    // strictly after settlement: a settlement authority outage must never
    // mutate the settlement already committed above. An out-of-process
    // declaration (or a retried submission) still reconciles later via
    // `swe_seed_reconciliation::reconcile_all_cases` at server startup or
    // `verify_swe_seed_completion` at read time.
    if !evidence.harvested_refs.is_empty() {
        if let Err(error) = submit_swe_seed_declaration(
            &config.root,
            request.policy_path,
            request.entity,
            case,
            run,
            item,
            &criteria,
            &evidence,
            evidence_ref.entry_ulid(),
            &settlement.settlement_id,
            &execution_started_at,
        )
        .await
        {
            tracing::warn!(
                event = "swe_seed_declaration_submit_failed",
                case_id = case,
                run_id = run,
                error = %error,
                "SWE_SEED declaration submission did not complete; a later out-of-process declaration will still reconcile"
            );
        }
    }

    Ok(DelegationResult {
        endpoint: endpoint_id.into(),
        run_id: run.into(),
        settlement: status,
        termination: Some(termination_str(&outcome.termination)),
        transcript_sha256: Some(outcome.transcript_sha256),
        turns_used: outcome.turns_used,
        error_class: criteria_error.or(outcome.error_subcode),
        continuation_key,
        basis,
    })
}

/// Production SWE_SEED declaration ingress (M16 T18, spec-agent-orchestration
/// §12.3). Constructs the declaration request only from what is already
/// persisted — the committed settlement criteria and transcript evidence —
/// and calls the configured strong settlement authority through
/// `spawn_blocking` (the transport is a real, timeout-bounded subprocess
/// call). No local fallback: an unavailable authority surfaces as
/// `settlement_authority_unavailable` and this function returns without
/// declaring anything.
#[allow(clippy::too_many_arguments)]
async fn submit_swe_seed_declaration(
    root: &Path,
    policy_path: &str,
    entity: &str,
    case: &str,
    run: &str,
    item: &str,
    criteria: &SettlementCriteriaRecord,
    evidence: &TranscriptEvidence,
    evidence_entry_ulid: &str,
    settlement_id: &str,
    execution_started_at: &str,
) -> Result<(), ForgeError> {
    let policy = resolve_policy_path(root, policy_path);
    let bundle = AuthorityPolicyBundle::load(&policy)?;
    let descriptor = bundle.strong_settlement_authority()?.clone();
    let declarer_role = descriptor
        .permitted_declarer_roles
        .first()
        .cloned()
        .ok_or_else(|| ForgeError::Config {
            class: "settlement_authority_config_error",
            path: policy.clone(),
            message: "strong settlement authority declares no permitted role".into(),
        })?;

    let claim_manifest_sha256 = crate::swe_seed_reconciliation::swe_seed_claim_manifest_sha256(
        case,
        run,
        item,
        settlement_id,
        &evidence.transcript_sha256,
        &evidence.harvested_refs,
    )?;
    let verifier_sha256 = format!(
        "sha256:{:x}",
        Sha256::digest(evidence.harvested_refs.join(",").as_bytes())
    );
    let mut source_evidence_refs = evidence.harvested_refs.clone();
    source_evidence_refs.push(evidence_entry_ulid.into());

    let request = SettlementDeclarationRequest {
        settlement_ref: settlement_id.into(),
        run_id: run.into(),
        case_id: case.into(),
        plan_item_id: item.into(),
        claim_manifest_sha256,
        criteria_ref: criteria.criteria_id.clone(),
        criteria_sha256: criteria.criteria_sha256.clone(),
        criteria_record_hash: criteria.criteria_record_hash.clone(),
        criteria_declared_at: criteria.declared_at.clone(),
        execution_started_at: execution_started_at.into(),
        job_contract_ref: None,
        origin_refs: criteria.origin_refs.clone(),
        verifier_ref: "swe_seed_harness".into(),
        verifier_sha256,
        acting_entity_id: entity.into(),
        requested_strength: SettlementStrength::Strong,
        declarer: Declarer {
            actor_id: descriptor.declarer_actor_id.clone(),
            authority_ref: descriptor.authority_ref.clone(),
            role: declarer_role,
            standing_basis: descriptor.standing_basis.clone(),
        },
        variation_tags: BTreeMap::new(),
        disruption_tags: vec![],
        orchestration_burden: None,
        source_evidence_refs,
        authored_by: None,
    };

    let declaration = tokio::task::spawn_blocking(
        move || -> Result<sea_forge_core::types::SettlementDeclaration, ForgeError> {
            let transport = sea_forge_settlement::CommandSweSeedTransport::new(
                descriptor.command.clone(),
                descriptor.timeout_secs,
            )?;
            let authority = sea_forge_settlement::SweSeedSettlementAuthority::new(
                transport,
                &request.acting_entity_id,
            )
            .with_adapter_ref(&descriptor.authority_ref);
            authority.declare(&request)
        },
    )
    .await
    .map_err(|error| ForgeError::Internal(format!("swe_seed declare task panic: {error}")))??;

    crate::swe_seed_reconciliation::append_and_reconcile_swe_seed_declaration(
        root,
        entity,
        &declaration,
    )?;
    Ok(())
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

/// The request-shape preconditions a delegation must meet before any record is
/// written, evaluated in one place.
///
/// Shared with [`crate::sfwp::delegation_preview`] rather than copied. The whole
/// value of a preview is that it agrees with execution about what is runnable,
/// and two copies of these rules would drift the first time one of them moved —
/// leaving the workbench confidently offering a delegation the kernel refuses,
/// or refusing one it would have run.
///
/// The two callers differ only in how much they report: execution takes the
/// first unmet rule and turns it into a `ForgeError::Input`, while the preview
/// shows every one, because an operator repairing a request wants the whole
/// list rather than one round trip per problem.
///
/// Returns the resolved model on success — resolution and validation are the
/// same step, so there is no window in which a caller holds an unchecked model.
pub(crate) fn check_preconditions(
    snapshot: &sea_forge_agent::EndpointSnapshot,
    instruction: &str,
    model: Option<&str>,
    max_turns: u32,
) -> Result<String, Vec<String>> {
    let mut unmet = Vec::new();
    if instruction.is_empty() {
        unmet.push("agent_task instruction must be non-empty".to_string());
    } else if instruction.len() > snapshot.max_request_bytes {
        unmet.push(format!(
            "agent_task instruction exceeds endpoint max_request_bytes ({})",
            snapshot.max_request_bytes
        ));
    }
    if max_turns == 0 {
        unmet.push("agent_task max_turns must be >= 1".to_string());
    }
    let model = model.unwrap_or(&snapshot.model);
    if model.is_empty() {
        unmet.push("agent_task model must not be empty".to_string());
    }
    if unmet.is_empty() {
        Ok(model.to_owned())
    } else {
        Err(unmet)
    }
}

pub(crate) fn action_for_delegation(
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

#[allow(clippy::too_many_arguments)]
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
        continuation_key: None,
        basis: settlement.basis,
    })
}

/// HTTP-provider episode: run the delegation loop against a resolved
/// credential. Credential resolution + `secret_access` authority stays in
/// `execute_with_control` so the denial short-circuit (`finish_rejected`)
/// preserves M13 behavior exactly.
#[allow(clippy::too_many_arguments)]
async fn run_http_episode(
    root: &Path,
    snapshot: &sea_forge_agent::EndpointSnapshot,
    credential: Zeroizing<String>,
    run: &str,
    case: &str,
    item: &str,
    instruction: &str,
    max_turns: u32,
    token_budget: Option<u64>,
    cancel: impl Fn() -> bool + Send + Sync + 'static,
    model: String,
) -> Result<sea_forge_agent::DelegationOutcome, ForgeError> {
    let _ = (root, run, case, item);
    let delegation_config = DelegationConfig {
        model,
        instruction: instruction.into(),
        max_turns,
        token_budget,
        max_output_tokens: Some(snapshot.max_response_bytes.min(4096) as u32),
    };
    let outcome = match &snapshot.kind {
        ProviderKind::OpenAiCompatible => {
            let provider = OpenAiCompatibleProvider::new(snapshot.clone()).map_err(|message| {
                ForgeError::Config {
                    class: "agent_endpoint_error",
                    path: root.join("server.yaml"),
                    message,
                }
            })?;
            run_delegation(&provider, credential, &delegation_config, cancel)
                .await
                .map_err(|e| ForgeError::Internal(e.to_string()))?
        }
        ProviderKind::Anthropic => {
            let provider =
                AnthropicProvider::new(snapshot.clone()).map_err(|message| ForgeError::Config {
                    class: "agent_endpoint_error",
                    path: root.join("server.yaml"),
                    message,
                })?;
            run_delegation(&provider, credential, &delegation_config, cancel)
                .await
                .map_err(|e| ForgeError::Internal(e.to_string()))?
        }
        ProviderKind::Acp => {
            return Err(ForgeError::Internal(
                "ACP dispatched through HTTP episode path".into(),
            ));
        }
    };
    Ok(outcome)
}

/// ACP episode: spawn the CLI agent under the run grant and drive one
/// delegation through the ACP driver. No credential, no ambient environment
/// (spec §10.4, §15). Every `session/request_permission` is mediated into a
/// durable `permission_request` record + authority decision.
#[allow(clippy::too_many_arguments)]
async fn run_acp_episode(
    root: &Path,
    snapshot: &sea_forge_agent::EndpointSnapshot,
    bundle: &AuthorityPolicyBundle,
    entity: &str,
    process: &str,
    run: &str,
    case: &str,
    item: &str,
    instruction: &str,
    max_turns: u32,
    cancel: impl Fn() -> bool + Send + Sync + 'static,
    permission_broker: Option<AcpApprovalBroker>,
    criteria: sea_forge_core::types::SettlementCriteriaRecord,
    criteria_ref: sea_forge_ledger::CommittedRecordRef,
    sandbox_class: &str,
) -> Result<
    (
        sea_forge_agent::DelegationOutcome,
        Option<String>,
        Vec<String>,
    ),
    ForgeError,
> {
    let argv: Vec<String> = snapshot.argv.clone();
    if argv.is_empty() {
        return Err(ForgeError::Config {
            class: "schema_error",
            path: root.join("server.yaml"),
            message: "acp endpoint argv is empty".into(),
        });
    }
    let env_pairs: Vec<(String, String)> = snapshot
        .env
        .iter()
        .filter_map(|e| {
            e.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
        })
        .filter(|(key, _)| !key.starts_with("SEA_FORGE_SWE_SEED_"))
        .collect();
    let workspace = root.join("runs").join(run).join("workspace");
    std::fs::create_dir_all(&workspace).map_err(|e| ForgeError::io("create acp workspace", e))?;
    let spawn = AcpSpawn {
        argv,
        env: env_pairs,
        cwd: Some(workspace.clone()),
        max_response_bytes: snapshot.max_response_bytes,
        max_transcript_bytes: snapshot
            .max_request_bytes
            .saturating_add(snapshot.max_response_bytes),
    };
    let session = match sandbox_class {
        "local" => AcpSession::spawn(spawn)
            .map(|(session, _)| session)
            .map_err(|message| ForgeError::Config {
                class: "agent_endpoint_error",
                path: root.join("server.yaml"),
                message,
            })?,
        "jail" => {
            let artifacts = root.join("runs").join(run).join("artifacts");
            std::fs::create_dir_all(&artifacts)
                .map_err(|error| ForgeError::io("create ACP artifacts", error))?;
            let workspace_for_jail = workspace.clone();
            let argv = spawn.argv.clone();
            let env = spawn.env.clone();
            let (sender, receiver) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                // Jail-class ACP agents default to network denial (fail-closed).
                // No scoped egress grant is plumbed to this path yet; the jail
                // must not open ungranted outbound/listening TCP sockets.
                let result = sea_forge_sandbox::jail::spawn_interactive(
                    &workspace_for_jail,
                    &artifacts,
                    &argv,
                    &workspace_for_jail,
                    &env,
                    &sea_forge_sandbox::NetworkPosture::Denied,
                );
                let _ = sender.send(result);
            });
            let child = receiver
                .recv()
                .map_err(|error| ForgeError::Internal(error.to_string()))??;
            AcpSession::from_std_child(
                child.child,
                child.stdin,
                child.stdout,
                child.stderr,
                workspace.clone(),
                snapshot.max_response_bytes,
                snapshot
                    .max_request_bytes
                    .saturating_add(snapshot.max_response_bytes),
            )
            .map_err(ForgeError::Internal)?
        }
        other => {
            return Err(ForgeError::Config {
                class: "unsupported_sandbox_class_error",
                path: root.join("server.yaml"),
                message: format!("ACP sandbox class '{other}' is unavailable"),
            });
        }
    };
    let mediator = AcpAuthorityMediator {
        root: root.to_path_buf(),
        case_id: case.into(),
        run_id: run.into(),
        item_id: item.into(),
        entity: entity.into(),
        process: process.into(),
        bundle: Arc::new(bundle.clone()),
        timeout_secs: snapshot.timeout.as_secs(),
        decision_sequence: Arc::new(std::sync::atomic::AtomicUsize::new(3)),
        permission_broker,
        criteria,
        criteria_ref,
    };
    let per_turn = snapshot.timeout;
    let prior_continuation = latest_disconnected_continuation(root, case, item, &snapshot.id)?;
    let acp_outcome = session
        .run_episode_with_continuation(
            instruction,
            max_turns,
            per_turn,
            cancel,
            &mediator,
            prior_continuation.as_deref(),
        )
        .await;
    let continuation_key = acp_outcome.continuation_key.clone();
    let mut outcome = acp_outcome_to_delegation(acp_outcome);
    let harvested = match harvest_swe_seed(snapshot, &workspace, run) {
        Ok(refs) => refs,
        Err(_) => {
            outcome.termination = DelegationTermination::EndpointError;
            outcome.error_subcode = Some("swe_seed_harvest_error".into());
            Vec::new()
        }
    };
    Ok((outcome, continuation_key, harvested))
}

fn harvest_swe_seed(
    snapshot: &sea_forge_agent::EndpointSnapshot,
    workspace: &Path,
    run: &str,
) -> Result<Vec<String>, ForgeError> {
    let configured = |key: &str| {
        snapshot.env.iter().find_map(|entry| {
            entry
                .split_once('=')
                .filter(|(name, _)| *name == key)
                .map(|(_, value)| value.to_string())
        })
    };
    let repo = configured("SEA_FORGE_SWE_SEED_REPO");
    let expected_commit = configured("SEA_FORGE_SWE_SEED_COMMIT");
    match (&repo, &expected_commit) {
        (None, None) => return Ok(Vec::new()),
        (Some(_), Some(_)) => {}
        _ => {
            return Err(ForgeError::Input(
                "SWE_SEED repo and commit must be configured together".into(),
            ));
        }
    }
    let repo = std::fs::canonicalize(repo.unwrap())
        .map_err(|error| ForgeError::io("canonicalize SWE_SEED repo", error))?;
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&repo)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|error| ForgeError::io("verify SWE_SEED commit", error))?;
    let actual = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !output.status.success() || actual != expected_commit.unwrap() {
        return Err(ForgeError::Input("SWE_SEED commit mismatch".into()));
    }

    let harness = workspace.join(".agent-harness");
    if !harness.is_dir() {
        return Err(ForgeError::Input(
            "configured SWE_SEED run produced no .agent-harness evidence".into(),
        ));
    }
    let canonical_harness = std::fs::canonicalize(&harness)
        .map_err(|error| ForgeError::io("canonicalize SWE_SEED harness", error))?;
    let canonical_workspace = std::fs::canonicalize(workspace)
        .map_err(|error| ForgeError::io("canonicalize ACP workspace", error))?;
    let mut pending = vec![harness];
    let mut files = Vec::new();
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(&path)
            .map_err(|error| ForgeError::io("read SWE_SEED harness", error))?
        {
            let entry = entry.map_err(|error| ForgeError::io("read SWE_SEED entry", error))?;
            let metadata = entry
                .file_type()
                .map_err(|error| ForgeError::io("read SWE_SEED entry type", error))?;
            if metadata.is_symlink() {
                return Err(ForgeError::UnsafePath(
                    "SWE_SEED evidence may not contain symlinks".into(),
                ));
            }
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if metadata.is_file() {
                files.push(entry.path());
                if files.len() > 64 {
                    return Err(ForgeError::Input(
                        "SWE_SEED evidence exceeds 64 files".into(),
                    ));
                }
            }
        }
    }
    files.sort();
    let mut refs = Vec::with_capacity(files.len());
    for path in files {
        let canonical = std::fs::canonicalize(&path)
            .map_err(|error| ForgeError::io("canonicalize SWE_SEED evidence", error))?;
        if !canonical.starts_with(&canonical_harness) {
            return Err(ForgeError::UnsafePath(
                "SWE_SEED evidence escaped .agent-harness".into(),
            ));
        }
        let bytes = std::fs::read(&canonical)
            .map_err(|error| ForgeError::io("read SWE_SEED evidence", error))?;
        if bytes.len() > 1_048_576 {
            return Err(ForgeError::Input(
                "SWE_SEED evidence file exceeds 1 MiB".into(),
            ));
        }
        let relative = canonical
            .strip_prefix(&canonical_workspace)
            .map_err(|_| ForgeError::UnsafePath("invalid SWE_SEED evidence path".into()))?;
        refs.push(format!(
            "{run}:{}:sha256:{:x}",
            relative.to_string_lossy(),
            Sha256::digest(&bytes)
        ));
    }
    Ok(refs)
}

fn latest_disconnected_continuation(
    root: &Path,
    case: &str,
    item: &str,
    endpoint: &str,
) -> Result<Option<String>, ForgeError> {
    let ledger = LedgerStream::open(root, format!("case-{case}"), "sea-forge-agent")?;
    let entries = ledger.read_entries()?;
    for session in entries
        .iter()
        .rev()
        .filter(|entry| entry.record_kind == "acp_session")
    {
        if session.payload["plan_item_id"].as_str() != Some(item)
            || session.payload["endpoint_ref"].as_str() != Some(endpoint)
        {
            continue;
        }
        let Some(session_run) = session.payload["run_id"].as_str() else {
            continue;
        };
        let disconnected = entries.iter().any(|entry| {
            entry.record_kind == "settlement"
                && entry.payload["run_id"].as_str() == Some(session_run)
                && entry.payload["basis"]
                    .as_array()
                    .is_some_and(|basis| basis.iter().any(|value| value == "acp_disconnect"))
        });
        if disconnected {
            return Ok(session.payload["continuation_key"]
                .as_str()
                .map(str::to_owned));
        }
    }
    Ok(None)
}
/// Map an ACP driver outcome to the shared delegation outcome shape so the
/// common evidence/settlement postamble treats HTTP and ACP uniformly.
fn acp_outcome_to_delegation(
    outcome: sea_forge_agent::AcpOutcome,
) -> sea_forge_agent::DelegationOutcome {
    let raw_entries: Vec<sea_forge_agent::TranscriptEntry> = outcome
        .transcript
        .iter()
        .map(|(role, content)| sea_forge_agent::TranscriptEntry {
            role: role.clone(),
            content: content.clone(),
        })
        .collect();
    // Route ACP through the SAME shared producer as HTTP: redact against the
    // sentinel corpus, canonicalize, and hash the exact canonical bytes BEFORE
    // any summary or artifact is derived. Previously ACP transcripts reached
    // hashing/storage with no redaction at all.
    let produced = sea_forge_agent::produce_transcript(&raw_entries, &[]);
    let entries = produced.entries;
    let sha = produced.sha256;
    let tool_calls = entries.iter().filter(|e| e.role == "tool").count() as u32;
    let final_excerpt = entries
        .iter()
        .rev()
        .find(|e| e.role == "assistant")
        .map(|e| {
            if e.content.len() <= 200 {
                e.content.clone()
            } else {
                // Respect char boundaries when truncating redacted UTF-8.
                let mut end = 200;
                while !e.content.is_char_boundary(end) {
                    end -= 1;
                }
                format!("{}...", &e.content[..end])
            }
        })
        .unwrap_or_default();
    let termination = match outcome.termination {
        sea_forge_agent::AcpTermination::Completed => DelegationTermination::Completed,
        sea_forge_agent::AcpTermination::Cancelled => DelegationTermination::Cancelled,
        sea_forge_agent::AcpTermination::Disconnect => DelegationTermination::AcpDisconnect,
        sea_forge_agent::AcpTermination::EndpointError => DelegationTermination::EndpointError,
        sea_forge_agent::AcpTermination::TurnCapExceeded => DelegationTermination::TurnCapExceeded,
    };
    let error_subcode = match outcome.termination {
        sea_forge_agent::AcpTermination::Disconnect => Some("acp_disconnect".into()),
        sea_forge_agent::AcpTermination::EndpointError => Some("acp_spawn_error".into()),
        _ => None,
    };
    // The final output flows into the settlement result; redact it through the
    // same corpus so a sentinel cannot leak via the result surface either.
    let final_output = outcome.final_output.map(|text| {
        sea_forge_agent::produce_transcript(
            &[sea_forge_agent::TranscriptEntry {
                role: "assistant".into(),
                content: text,
            }],
            &[],
        )
        .entries
        .swap_remove(0)
        .content
    });
    sea_forge_agent::DelegationOutcome {
        termination,
        turns_used: outcome.turns_used,
        final_output,
        transcript_sha256: sha,
        summary: TranscriptSummary {
            turn_count: outcome.turns_used,
            tool_calls,
            final_excerpt,
        },
        transcript: entries,
        error_subcode,
    }
}

/// Durable record of one ACP permission request and its SEA decision
/// (spec §10.4: bound to session episode, run, item, exact requested action,
/// and the authority decision). Appended to the case ledger whether the
/// request is granted or denied.
#[derive(Serialize)]
struct AcpPermissionRecord {
    version: &'static str,
    permission_id: String,
    case_id: String,
    run_id: String,
    plan_item_id: String,
    session_id: String,
    tool_call_id: String,
    tool_kind: String,
    options: Vec<(String, String)>,
    authority_decision_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    approval_id: Option<String>,
    verdict: String,
    decided_at: String,
    raw_sha256: String,
}

/// Server-side ACP permission mediator: every `session/request_permission` is
/// turned into a canonical `sandbox_execution` authority decision and a durable
/// `permission_request` record before the driver responds to the agent.
///
/// Allow ⇒ select the first `allow_*` option; Deny/Escalate ⇒ `reject_once`
/// (portable path resolves immediately; a full approval-suspend channel plugs
/// in here without changing the driver contract).
struct AcpAuthorityMediator {
    root: std::path::PathBuf,
    case_id: String,
    run_id: String,
    item_id: String,
    entity: String,
    #[allow(dead_code)]
    process: String,
    bundle: Arc<AuthorityPolicyBundle>,
    timeout_secs: u64,
    decision_sequence: Arc<std::sync::atomic::AtomicUsize>,
    permission_broker: Option<AcpApprovalBroker>,
    criteria: sea_forge_core::types::SettlementCriteriaRecord,
    criteria_ref: sea_forge_ledger::CommittedRecordRef,
}

fn append_approval_view(root: &Path, approval: &ApprovalRequest) -> Result<(), ForgeError> {
    let path = root.join("approvals.jsonl");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| ForgeError::io("create approvals parent", error))?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| ForgeError::io("open approvals.jsonl", error))?;
    serde_json::to_writer(&mut file, approval)?;
    file.write_all(b"\n")
        .map_err(|error| ForgeError::io("flush approval", error))?;
    Ok(())
}

impl AcpPermissionMediator for AcpAuthorityMediator {
    fn mediate<'a>(
        &'a self,
        request: sea_forge_agent::AcpPermissionRequest,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = PermissionDecision> + Send + 'a>> {
        Box::pin(async move {
            let mapped =
                sea_forge_agent::acp::KNOWN_TOOL_KINDS.contains(&request.tool_kind.as_str());
            let raw_sha256 = format!(
                "sha256:{:x}",
                Sha256::digest(serde_json::to_vec(&request.raw).unwrap_or_default())
            );
            let action = AuthorityAction::Reserved {
                resource_type: "sandbox_execution".into(),
                resource_id: request.tool_kind.clone(),
                parameters: json!({
                    "tool_call_id": request.tool_call_id,
                    "tool_kind": request.tool_kind,
                    "title": request.title,
                    "options": request.options,
                    "raw_sha256": raw_sha256,
                }),
            };
            let engine = match PolicyAuthorityEngine::new((*self.bundle).clone()) {
                Ok(e) => e,
                Err(_) => return deny_once(&request),
            };
            let actor = Actor {
                actor_id: self.entity.clone(),
                role: ActorRole::Operator,
            };
            let binding = self
                .bundle
                .resolve_identity(&actor.actor_id, actor.role.clone());
            let ledger = match LedgerStream::open(
                &self.root,
                format!("case-{}", self.case_id),
                "sea-forge-agent",
            ) {
                Ok(l) => l,
                Err(_) => return deny_once(&request),
            };
            let sequence = self
                .decision_sequence
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let decision = match engine.evaluate(AuthorityEvaluation {
                actor: &actor,
                binding,
                run_id: &self.run_id,
                case_id: &self.case_id,
                plan_item_id: &self.item_id,
                sequence,
                action: &action,
                workspace_root: &self.root,
                evidence_refs: vec![],
                artifacts_root: None,
                timeout_secs: Some(self.timeout_secs),
                env_keys: Default::default(),
                domainforge_candidate: None,
                environment: None,
            }) {
                Ok(d) => d,
                Err(_) => return deny_once(&request),
            };
            let decision_ref = match commit_view(
                &ledger,
                "authority_decision",
                vec![
                    self.case_id.clone(),
                    self.run_id.clone(),
                    self.item_id.clone(),
                    "sandbox_execution".into(),
                ],
                &decision,
                &self
                    .root
                    .join("runs")
                    .join(&self.run_id)
                    .join(format!("acp-permission-{}.json", request.tool_call_id)),
                vec![],
            ) {
                Ok(r) => r,
                Err(_) => return deny_once(&request),
            };
            let permission_id = match random_id("acp") {
                Ok(id) => id,
                Err(_) => return deny_once(&request),
            };
            let mut approval = None;
            let mut approval_receiver = None;
            if decision.verdict == Verdict::Escalate {
                let count = match ledger.read_entries() {
                    Ok(entries) => entries
                        .iter()
                        .filter(|entry| entry.record_kind == "approval_request")
                        .count(),
                    Err(_) => return deny_once(&request),
                };
                let requested_at = Utc::now();
                let request_record = ApprovalRequest {
                    version: RECORD_VERSION.into(),
                    approval_id: ids::seq_id("apr", 4, count + 1),
                    run_id: self.run_id.clone(),
                    case_id: self.case_id.clone(),
                    decision_id: decision.decision_id.clone(),
                    plan_item_id: self.item_id.clone(),
                    criteria_ref: Some(self.criteria.criteria_id.clone()),
                    criteria_sha256: Some(self.criteria.criteria_sha256.clone()),
                    criteria_record_hash: Some(self.criteria.criteria_record_hash.clone()),
                    job_contract_ref: None,
                    requested_at: requested_at.to_rfc3339(),
                    expires_at: (requested_at
                        + chrono::Duration::seconds(self.timeout_secs as i64))
                    .to_rfc3339(),
                    status: ApprovalStatus::Pending,
                    resolved_by: None,
                    resolved_at: None,
                    note: Some(format!(
                        "ACP permission request: {} ({})",
                        request.tool_kind, request.tool_call_id
                    )),
                };
                if let Some(broker) = &self.permission_broker {
                    approval_receiver =
                        Some(broker.register(request_record.approval_id.clone()).await);
                }
                if ledger
                    .commit_typed(
                        "approval_request",
                        vec![self.case_id.clone(), request_record.approval_id.clone()],
                        &request_record,
                        vec![decision_ref.entry_ulid().into()],
                    )
                    .is_err()
                    || append_approval_view(&self.root, &request_record).is_err()
                {
                    if let Some(broker) = &self.permission_broker {
                        let _ = broker.resolve(&request_record.approval_id).await;
                    }
                    return deny_once(&request);
                }
                approval = Some(request_record);
            }
            let record = AcpPermissionRecord {
                version: "0.2",
                permission_id: permission_id.clone(),
                case_id: self.case_id.clone(),
                run_id: self.run_id.clone(),
                plan_item_id: self.item_id.clone(),
                session_id: request.session_id.clone(),
                tool_call_id: request.tool_call_id.clone(),
                tool_kind: request.tool_kind.clone(),
                options: request.options.clone(),
                authority_decision_ref: decision_ref.entry_ulid().into(),
                approval_id: approval.as_ref().map(|value| value.approval_id.clone()),
                verdict: if mapped {
                    format!("{:?}", decision.verdict).to_lowercase()
                } else {
                    "deny_unmapped".into()
                },
                decided_at: Utc::now().to_rfc3339(),
                raw_sha256,
            };
            if commit_view(
                &ledger,
                "permission_request",
                vec![
                    self.case_id.clone(),
                    self.run_id.clone(),
                    self.item_id.clone(),
                    permission_id.clone(),
                ],
                &record,
                &self
                    .root
                    .join("runs")
                    .join(&self.run_id)
                    .join(format!("permission-{}.json", request.tool_call_id)),
                vec![decision_ref.entry_ulid().into()],
            )
            .is_err()
            {
                return deny_once(&request);
            }
            if !mapped {
                return deny_once(&request);
            }
            match decision.verdict {
                Verdict::Allow => {
                    if let Some((id, _)) =
                        request.options.iter().find(|(_, k)| k.starts_with("allow"))
                    {
                        PermissionDecision::Allow {
                            option_id: id.clone(),
                        }
                    } else {
                        deny_once(&request)
                    }
                }
                Verdict::Escalate => {
                    let signalled = match approval_receiver {
                        Some(receiver) => tokio::time::timeout(
                            std::time::Duration::from_secs(self.timeout_secs),
                            receiver,
                        )
                        .await
                        .ok()
                        .is_some_and(|result| result.is_ok()),
                        None => false,
                    };
                    let mut effective_granted = false;
                    if let Some(mut approval) = approval {
                        let resolved = signalled.then(|| {
                            ledger.read_entries().ok().and_then(|entries| {
                                entries
                                    .into_iter()
                                    .rev()
                                    .find(|entry| {
                                        entry.record_kind == "approval_resolution"
                                            && entry.payload["approval_id"].as_str()
                                                == Some(approval.approval_id.as_str())
                                    })
                                    .and_then(|entry| {
                                        serde_json::from_value::<ApprovalRequest>(
                                            entry.payload.clone(),
                                        )
                                        .ok()
                                        .map(|record| (record, entry.committed_ref()))
                                    })
                            })
                        });
                        if let Some(Some((resolved, resolution_ref))) = resolved {
                            if resolved.status == ApprovalStatus::Approved {
                                effective_granted = engine
                                    .grant_after_approval(
                                        &decision,
                                        &decision_ref,
                                        &action,
                                        &resolved,
                                        &resolution_ref,
                                        &self.criteria,
                                        &self.criteria_ref,
                                        None,
                                        &ledger,
                                    )
                                    .and_then(|grant| {
                                        grant.authorize(
                                            &action,
                                            &self.run_id,
                                            &self.item_id,
                                            &self.root,
                                        )
                                    })
                                    .is_ok();
                            }
                            approval = resolved;
                        }
                        let resolution = json!({
                            "permission_id": permission_id,
                            "approval_id": approval.approval_id,
                            "granted": effective_granted,
                            "resolved_at": approval.resolved_at,
                        });
                        if ledger
                            .commit_typed(
                                "permission_resolution",
                                vec![self.case_id.clone(), self.run_id.clone()],
                                &resolution,
                                vec![decision_ref.entry_ulid().into()],
                            )
                            .is_err()
                        {
                            return deny_once(&request);
                        }
                    }
                    if effective_granted {
                        request
                            .options
                            .iter()
                            .find(|(_, kind)| kind.starts_with("allow"))
                            .map_or_else(
                                || deny_once(&request),
                                |(id, _)| PermissionDecision::Allow {
                                    option_id: id.clone(),
                                },
                            )
                    } else {
                        deny_once(&request)
                    }
                }
                Verdict::Deny => deny_once(&request),
            }
        })
    }
}
