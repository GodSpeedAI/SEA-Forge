use chrono::Utc;
use sea_forge_agent::{
    AgentConfig, AgentError, AgentMessage, AgentProvider, AnthropicProvider, CompletionRequest,
    OpenAiCompatibleProvider, ProviderKind,
};
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{ContractRef, ExtensionDescriptor, ExtensionKind};
use sea_forge_core::{
    errors::ForgeError,
    ids::{case_id, random_id, run_id},
    types::{
        Actor, ActorRole, AuthorityAction, CasePlan, Intent, ItemKind, Operation, PlanItem,
        SettlementCriteria, SettlementEvent, SettlementStatus, Verdict,
    },
};
use sea_forge_extension::ExtensionRegistry;
use sea_forge_ledger::LedgerStream;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

pub trait CredentialResolver: Send + Sync {
    fn resolve(&self, reference: &str) -> Result<Zeroizing<String>, ForgeError>;
}

pub struct EnvironmentCredentialResolver;

impl CredentialResolver for EnvironmentCredentialResolver {
    fn resolve(&self, reference: &str) -> Result<Zeroizing<String>, ForgeError> {
        std::env::var(reference)
            .map(Zeroizing::new)
            .map_err(|_| ForgeError::Config {
                class: "missing_credential_error",
                path: PathBuf::from(reference),
                message: "credential reference is not resolvable".into(),
            })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ProbeOutcome {
    pub endpoint: String,
    pub run_id: String,
    pub settlement: SettlementStatus,
    pub response_sha256: Option<String>,
    pub error_class: Option<String>,
}

/// Governed probe request: endpoint identity + prompt + governance context.
/// `config` and `resolver` stay outside this struct because they are services,
/// not request payload.
#[derive(Clone, Debug)]
pub struct ProbeRequest<'a> {
    pub endpoint_id: &'a str,
    pub prompt: &'a str,
    pub model: Option<&'a str>,
    pub policy_path: &'a str,
    pub entity: &'a str,
    pub process: &'a str,
}

#[derive(Clone, Debug, Serialize)]
struct AgentListEntry {
    id: String,
    kind: ProviderKind,
    descriptor_config_sha256: String,
    status: &'static str,
}

pub fn list(config: &AgentConfig) -> serde_json::Value {
    let endpoints = config
        .endpoints
        .iter()
        .filter_map(|endpoint| endpoint.snapshot().ok())
        .map(|snapshot| AgentListEntry {
            id: snapshot.id,
            kind: snapshot.kind,
            descriptor_config_sha256: snapshot.descriptor_config_sha256,
            status: "declared",
        })
        .collect::<Vec<_>>();
    serde_json::to_value(endpoints).unwrap_or_else(|_| json!([]))
}

pub async fn probe(
    config: &crate::ServerConfig,
    request: ProbeRequest<'_>,
    resolver: &dyn CredentialResolver,
) -> Result<ProbeOutcome, ForgeError> {
    let endpoint_id = request.endpoint_id;
    let prompt = request.prompt;
    if prompt.is_empty() || prompt.len() > 65_536 {
        return Err(ForgeError::Input(
            "agent probe prompt must be 1..65536 bytes".into(),
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
        class: if message.contains("unsupported_kind_error") {
            "unsupported_kind_error"
        } else {
            "schema_error"
        },
        path: config.root.join("server.yaml"),
        message,
    })?;
    let model = request.model.unwrap_or(&snapshot.model).to_owned();
    if model.is_empty() {
        return Err(ForgeError::Input(
            "agent probe model must not be empty".into(),
        ));
    }
    let run = run_id()?;
    let case = case_id()?;
    let item = "item_agent_probe";
    let ledger = LedgerStream::open(&config.root, format!("case-{case}"), "sea-forge-agent")?;
    let intent = Intent {
        intent_id: random_id("int")?,
        summary: format!("Probe configured agent endpoint {endpoint_id}"),
        actor_id: request.entity.into(),
        process_id: request.process.into(),
        created_at: Utc::now().to_rfc3339(),
    };
    let intent_ref = commit_view(
        &ledger,
        "intent",
        vec![case.clone(), run.clone()],
        &intent,
        &config.root.join("runs").join(&run).join("intent.json"),
        vec![],
    )?;
    let prompt_hash = format!("sha256:{:x}", Sha256::digest(prompt.as_bytes()));
    let operation = Operation::AgentProbe {
        endpoint_ref: endpoint_id.into(),
        model: model.clone(),
        prompt_sha256: prompt_hash.clone(),
    };
    let plan = CasePlan {
        version: "0.2".into(),
        plan_id: random_id("plan")?,
        case_id: case.clone(),
        run_id: run.clone(),
        intent_id: intent.intent_id.clone(),
        items: vec![PlanItem {
            plan_item_id: item.into(),
            name: "agent_probe".into(),
            operations: vec![operation],
            entry_criteria: vec![],
            entry_criteria_mode: Default::default(),
            exit_criteria: vec![],
            settlement_criteria: SettlementCriteria::default(),
            settlement_criteria_ref: None,
            item_kind: ItemKind::SandboxedTask,
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
    let plan_ref = commit_view(
        &ledger,
        "plan",
        vec![case.clone(), run.clone()],
        &plan,
        &config.root.join("runs").join(&run).join("plan.json"),
        vec![intent_ref.entry_ulid().into()],
    )?;
    let action = action_for(&snapshot, endpoint_id, &model, &prompt_hash)?;
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
        run_id: &run,
        case_id: &case,
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
        vec![run.clone(), item.into()],
        &decision,
        &config.root.join("runs").join(&run).join("authority.json"),
        vec![intent_ref.entry_ulid().into(), plan_ref.entry_ulid().into()],
    )?;
    if decision.verdict != Verdict::Allow {
        return finish_rejected(
            &ledger,
            &config.root,
            &run,
            endpoint_id,
            decision_ref.entry_ulid(),
            "authority_denied",
        );
    }
    let grant = engine.grant(&decision, &decision_ref, &action, None)?;
    grant.authorize(&action, &run, item, &config.root)?;

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
                run_id: &run,
                case_id: &case,
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
                vec![run.clone(), item.into(), "secret_access".into()],
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
                    &run,
                    endpoint_id,
                    secret_ref.entry_ulid(),
                    "secret_access_denied",
                );
            }
            let secret_grant = engine.grant(&secret_decision, &secret_ref, &secret_action, None)?;
            secret_grant.authorize(&secret_action, &run, item, &config.root)?;
            resolver.resolve(reference)?
        }
        None => Zeroizing::new(String::new()),
    };

    register_endpoint(&config.root, &ledger, &snapshot, &decision_ref)?;

    let request = CompletionRequest {
        model,
        messages: vec![AgentMessage {
            role: sea_forge_agent::MessageRole::User,
            content: prompt.into(),
        }],
        max_tokens: Some(1024),
    };
    let completion = match &snapshot.kind {
        ProviderKind::OpenAiCompatible => {
            OpenAiCompatibleProvider::new(snapshot.clone())
                .map_err(|message| ForgeError::Config {
                    class: "agent_endpoint_error",
                    path: config.root.join("server.yaml"),
                    message,
                })?
                .complete(request, credential)
                .await
        }
        ProviderKind::Anthropic => {
            AnthropicProvider::new(snapshot.clone())
                .map_err(|message| ForgeError::Config {
                    class: "agent_endpoint_error",
                    path: config.root.join("server.yaml"),
                    message,
                })?
                .complete(request, credential)
                .await
        }
        ProviderKind::Acp => Err(AgentError::UnsupportedKind),
    };
    match completion {
        Ok(response) => {
            let response_hash = format!(
                "sha256:{:x}",
                Sha256::digest(response.message.content.as_bytes())
            );
            let evidence = json!({"endpoint_ref": endpoint_id, "provider_kind": snapshot.kind, "status":"schema_valid", "response_sha256": response_hash});
            let evidence_ref = commit_view(
                &ledger,
                "agent_probe_evidence",
                vec![run.clone(), item.into()],
                &evidence,
                &config.root.join("runs").join(&run).join("evidence.json"),
                vec![decision_ref.entry_ulid().into()],
            )?;
            let settlement = SettlementEvent {
                version: "0.2".into(),
                settlement_id: random_id("set")?,
                run_id: run.clone(),
                status: SettlementStatus::Accepted,
                basis: vec!["authority_allow".into(), "schema_valid_response".into()],
                review_required: false,
                settled_at: Utc::now().to_rfc3339(),
                criteria_ref: None,
            };
            commit_view(
                &ledger,
                "settlement",
                vec![run.clone()],
                &settlement,
                &config.root.join("runs").join(&run).join("settlement.json"),
                vec![evidence_ref.entry_ulid().into()],
            )?;
            Ok(ProbeOutcome {
                endpoint: endpoint_id.into(),
                run_id: run,
                settlement: SettlementStatus::Accepted,
                response_sha256: Some(response_hash),
                error_class: None,
            })
        }
        Err(error) => finish_rejected(
            &ledger,
            &config.root,
            &run,
            endpoint_id,
            decision_ref.entry_ulid(),
            error_class(&error),
        ),
    }
}

fn error_class(error: &AgentError) -> String {
    match error {
        AgentError::Unreachable => "agent_endpoint_unreachable",
        AgentError::Timeout => "agent_endpoint_timeout",
        AgentError::Http4xx(_) => "agent_endpoint_http_4xx",
        AgentError::Http5xx(_) => "agent_endpoint_http_5xx",
        AgentError::Oversize => "agent_endpoint_oversize",
        AgentError::SchemaInvalid => "agent_endpoint_schema_invalid",
        AgentError::Redirect => "agent_endpoint_redirect",
        AgentError::UnsupportedKind => "unsupported_kind_error",
        AgentError::InvalidRequest(_) => "agent_endpoint_invalid_request",
        AgentError::Transport => "agent_endpoint_transport",
    }
    .into()
}

fn finish_rejected(
    ledger: &LedgerStream,
    root: &Path,
    run: &str,
    endpoint: &str,
    authority_ref: &str,
    error_class: impl Into<String>,
) -> Result<ProbeOutcome, ForgeError> {
    let error_class = error_class.into();
    let evidence = json!({"endpoint_ref": endpoint, "termination": error_class});
    let evidence_ref = commit_view(
        ledger,
        "agent_probe_evidence",
        vec![run.into()],
        &evidence,
        &root.join("runs").join(run).join("evidence.json"),
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
        vec![run.into()],
        &settlement,
        &root.join("runs").join(run).join("settlement.json"),
        vec![evidence_ref.entry_ulid().into()],
    )?;
    Ok(ProbeOutcome {
        endpoint: endpoint.into(),
        run_id: run.into(),
        settlement: SettlementStatus::Rejected,
        response_sha256: None,
        error_class: Some(error_class),
    })
}

fn action_for(
    endpoint: &sea_forge_agent::EndpointSnapshot,
    endpoint_ref: &str,
    model: &str,
    prompt_hash: &str,
) -> Result<AuthorityAction, ForgeError> {
    Ok(AuthorityAction::AgentProbe {
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
        prompt_sha256: prompt_hash.into(),
    })
}

fn register_endpoint(
    root: &Path,
    ledger: &LedgerStream,
    endpoint: &sea_forge_agent::EndpointSnapshot,
    authority_ref: &sea_forge_ledger::CommittedRecordRef,
) -> Result<(), ForgeError> {
    let descriptor = ExtensionDescriptor {
        extension_id: format!("agent_endpoint_{}", endpoint.id),
        kind: ExtensionKind::RuntimeAdapter,
        name: format!("agent endpoint {}", endpoint.id),
        version: "0.1.0".into(),
        provider: "sea-forge-agent".into(),
        capabilities: vec![
            "agent_probe".into(),
            format!("provider:{:?}", endpoint.kind),
        ],
        authority_surface: "external_api".into(),
        input_contract: ContractRef {
            schema: "sea-forge-agent.endpoint.v1".into(),
            sha256: endpoint.descriptor_config_sha256.clone(),
        },
        output_contract: ContractRef {
            schema: "sea-forge-agent.probe.v1".into(),
            sha256: format!("sha256:{}", "b".repeat(64)),
        },
        deterministic: false,
        installed_at: None,
    };
    let mut registry = ExtensionRegistry::load(root)?;
    let was_new = !registry.extensions.iter().any(|entry| {
        entry.extension_id == descriptor.extension_id && entry.version == descriptor.version
    });
    registry.register_immutable_runtime_adapter(&descriptor)?;
    registry.save(root, ledger, authority_ref)?;
    // Slice 4.2: an endpoint install/change is an extension mutation that
    // invalidates the current self-model snapshot. Mark stale only when a
    // snapshot already exists (marking before init is an error); this never
    // mutates an immutable snapshot file.
    if was_new {
        if let Ok(Some(_)) = sea_forge_self_model::store::current_snapshot(root) {
            sea_forge_self_model::store::mark_current_stale(
                root,
                &format!("agent_endpoint_registered:{}", endpoint.id),
            )?;
        }
    }
    Ok(())
}

pub fn resolve_policy_path(root: &Path, configured: &str) -> PathBuf {
    let path = PathBuf::from(configured);
    if path.is_absolute() || path.exists() {
        path
    } else {
        root.join(path)
    }
}

pub fn commit_view<T: Serialize>(
    ledger: &LedgerStream,
    kind: &str,
    subjects: Vec<String>,
    record: &T,
    path: &Path,
    authority_refs: Vec<String>,
) -> Result<sea_forge_ledger::CommittedRecordRef, ForgeError> {
    let reference = ledger.commit_typed(kind, subjects, record, authority_refs)?;
    let bytes = serde_json::to_vec_pretty(record)?;
    ledger.materialize_view(&reference, path, &bytes)?;
    Ok(reference)
}
