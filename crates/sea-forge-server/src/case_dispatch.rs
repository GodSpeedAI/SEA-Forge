use crate::{agent_probe, delegation, DispatchOutcome, ServerState, SubmitPayload};
use chrono::Utc;
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_case_runner::{write_json, CaseRunner};
use sea_forge_core::{
    errors::ForgeError,
    ids,
    types::{
        Actor, ActorRole, Case, CasePlan, CaseState, ExecutionRequest, ItemKind, Operation,
        SettlementEvent, SettlementStatus, TraceKind, Verdict,
    },
    RECORD_VERSION,
};
use sea_forge_ledger::LedgerStream;
use sea_forge_planner::case_engine::{replay_case, validate_proposal, CaseAction};
use std::{collections::BTreeMap, fs, path::PathBuf, sync::Arc};
use tokio::task::JoinSet;

struct Completion {
    item_id: String,
    instance: u32,
    run_id: String,
    settlement: SettlementEvent,
}

/// Server-owned, case-local dispatch. Active tasks and their permits live only
/// in this function; all scheduling decisions are re-derived from case events.
pub(crate) async fn submit(
    payload: SubmitPayload,
    state: &Arc<ServerState>,
) -> Result<DispatchOutcome, ForgeError> {
    let _ = &payload.intent;
    let plan_path = payload
        .plan
        .ok_or_else(|| ForgeError::Input("submit requires a plan".into()))?;
    let mut plan: CasePlan = serde_json::from_slice(
        &fs::read(&plan_path).map_err(|e| ForgeError::io("read plan proposal", e))?,
    )?;
    validate_proposal(&mut plan)?;
    fs::create_dir_all(&state.config.root).map_err(|e| ForgeError::io("create state root", e))?;
    let root = state
        .config
        .root
        .canonicalize()
        .map_err(|e| ForgeError::io("canonicalize state root", e))?;
    let case_id = ids::case_id()?;
    plan.case_id.clone_from(&case_id);
    let case_dir = root.join("cases").join(&case_id);
    let (_runs_dir, case_events) = CaseRunner::initialize_case(&case_dir)?;
    let stream = LedgerStream::open(&root, format!("case-{case_id}"), &payload.entity)?;
    stream.commit_typed("case_plan", vec![case_id.clone()], &plan, vec![])?;
    let intent = sea_forge_core::types::Intent {
        intent_id: ids::random_id("int")?,
        summary: format!("Execute plan {plan_path}"),
        actor_id: payload.entity.clone(),
        process_id: payload.process.clone(),
        created_at: Utc::now().to_rfc3339(),
    };
    let mut case = Case {
        version: RECORD_VERSION.into(),
        case_id: case_id.clone(),
        intent,
        state: CaseState::Active,
        plan_ref: plan.plan_id.clone(),
        run_ids: vec![],
        stages: plan
            .items
            .iter()
            .filter(|item| item.item_kind == ItemKind::Stage)
            .map(|item| item.plan_item_id.clone())
            .collect(),
        close_reason: None,
        created_at: Utc::now().to_rfc3339(),
        closed_at: None,
    };
    write_json(&case_dir.join("case.json"), &case)?;
    write_json(&case_dir.join("plan.json"), &plan)?;
    let mut events = vec![];
    CaseRunner::append_event(
        &case_events,
        &stream,
        &mut events,
        TraceKind::CaseCreated,
        None,
        serde_json::json!({"case_id": case_id}),
    )?;
    let mut active = JoinSet::new();

    'dispatch: loop {
        let actions = CaseRunner::next_ready_actions(&plan.items, &events);
        if actions.is_empty() {
            if active.is_empty() {
                return Ok(DispatchOutcome {
                    case_id,
                    state: "active",
                    exit_code: 5,
                });
            }
            let completion = active
                .join_next()
                .await
                .ok_or_else(|| ForgeError::Internal("missing active episode".into()))?
                .map_err(|error| ForgeError::Internal(format!("episode task panic: {error}")))?;
            record_completion(
                &stream,
                &case_id,
                &case_events,
                &mut events,
                &mut case,
                completion,
            )?;
            write_json(&case_dir.join("case.json"), &case)?;
            continue;
        }
        let mut dispatched = false;
        let mut park_human = false;
        for action in actions {
            match action {
                CaseAction::Enable(item) => CaseRunner::append_event(
                    &case_events,
                    &stream,
                    &mut events,
                    TraceKind::ItemEnabled,
                    Some(&item),
                    serde_json::json!({}),
                )?,
                CaseAction::AchieveMilestone(item) => CaseRunner::append_event(
                    &case_events,
                    &stream,
                    &mut events,
                    TraceKind::MilestoneAchieved,
                    Some(&item),
                    serde_json::json!({}),
                )?,
                CaseAction::CompleteCase => {
                    case.state = CaseState::Completed;
                    case.closed_at = Some(Utc::now().to_rfc3339());
                    CaseRunner::append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::CaseClosed,
                        None,
                        serde_json::json!({}),
                    )?;
                    write_json(&case_dir.join("case.json"), &case)?;
                    return Ok(DispatchOutcome {
                        case_id,
                        state: "completed",
                        exit_code: 0,
                    });
                }
                CaseAction::TerminateCase { blocking_item } => {
                    while let Some(completion) = active.join_next().await {
                        let completion = completion.map_err(|error| {
                            ForgeError::Internal(format!("episode task panic: {error}"))
                        })?;
                        record_completion(
                            &stream,
                            &case_id,
                            &case_events,
                            &mut events,
                            &mut case,
                            completion,
                        )?;
                    }
                    case.state = CaseState::Terminated;
                    case.close_reason = Some(format!("required_item_failed:{blocking_item}"));
                    case.closed_at = Some(Utc::now().to_rfc3339());
                    CaseRunner::append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::CaseTerminated,
                        None,
                        serde_json::json!({"blocking_item": blocking_item}),
                    )?;
                    write_json(&case_dir.join("case.json"), &case)?;
                    return Ok(DispatchOutcome {
                        case_id,
                        state: "terminated",
                        exit_code: 3,
                    });
                }
                CaseAction::ParkHumanTask(item_id) => {
                    CaseRunner::append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::ItemActivated,
                        Some(&item_id),
                        serde_json::json!({"human_task": true}),
                    )?;
                    park_human = true;
                }
                CaseAction::Activate(item_id) => {
                    let item = plan
                        .items
                        .iter()
                        .find(|item| item.plan_item_id == item_id)
                        .ok_or_else(|| ForgeError::Internal("missing plan item".into()))?
                        .clone();
                    // The case engine emits Activate for any non-Milestone,
                    // non-HumanTask item without manual_activation, including
                    // Stage/TimerListener/UserEventListener. Only SandboxedTask
                    // and AgentTask have dispatch semantics; reject the rest
                    // with a typed error before any permit or side effect.
                    if !matches!(
                        item.item_kind,
                        ItemKind::SandboxedTask | ItemKind::AgentTask
                    ) {
                        return Err(ForgeError::Input(format!(
                            "non_executable item kind cannot be dispatched: {:?}",
                            item.item_kind
                        )));
                    }
                    let permit = if active.is_empty() {
                        state.semaphore.clone().acquire_owned().await.map_err(|_| {
                            ForgeError::Internal("server semaphore unavailable".into())
                        })?
                    } else {
                        tokio::select! {
                            biased;
                            completion = active.join_next() => {
                                let completion = completion
                                    .ok_or_else(|| ForgeError::Internal("missing active episode".into()))?
                                    .map_err(|error| ForgeError::Internal(format!("episode task panic: {error}")))?;
                                record_completion(&stream, &case_id, &case_events, &mut events, &mut case, completion)?;
                                write_json(&case_dir.join("case.json"), &case)?;
                                continue 'dispatch;
                            }
                            permit = state.semaphore.clone().acquire_owned() => permit
                                .map_err(|_| ForgeError::Internal("server semaphore unavailable".into()))?,
                        }
                    };
                    let projection = replay_case(&plan.items, &events);
                    let instance = projection
                        .items
                        .iter()
                        .find(|state| state.item_id == item_id)
                        .map_or(1, |state| state.instances + 1);
                    let run_id = ids::run_id()?;
                    CaseRunner::append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::ItemActivated,
                        Some(&item_id),
                        serde_json::json!({
                            "instance": instance,
                            "run_id": run_id,
                            "episode_kind": match item.item_kind {
                                ItemKind::SandboxedTask => "sandboxed_task",
                                ItemKind::AgentTask => "agent_task",
                                // Validated above; the wildcard is exhaustive
                                // for the remaining variants and never selects.
                                _ => "non_executable",
                            },
                        }),
                    )?;
                    let config = state.config.clone();
                    let policy = payload.policy.clone();
                    let entity = payload.entity.clone();
                    let process = payload.process.clone();
                    let timeout = payload.timeout;
                    let root = root.clone();
                    let case_for_task = case_id.clone();
                    let run_for_task = run_id.clone();
                    active.spawn(async move {
                        let _permit = permit;
                        let criteria_ref = item.settlement_criteria_ref.clone();
                        let result = match item.item_kind {
                            ItemKind::AgentTask => {
                                execute_agent(
                                    &config,
                                    &item,
                                    &policy,
                                    &entity,
                                    &process,
                                    &case_for_task,
                                    &run_id,
                                )
                                .await
                            }
                            ItemKind::SandboxedTask => tokio::task::spawn_blocking(move || {
                                execute_sandbox(
                                    &config,
                                    &item,
                                    &policy,
                                    &entity,
                                    &process,
                                    &case_for_task,
                                    &run_for_task,
                                    root,
                                    timeout,
                                )
                            })
                            .await
                            .map_err(|e| {
                                ForgeError::Internal(format!("sandbox episode panic: {e}"))
                            })
                            .and_then(|result| result),
                            _ => Err(ForgeError::Input("unsupported executable plan item".into())),
                        };
                        let settlement = result.unwrap_or_else(|error| SettlementEvent {
                            version: RECORD_VERSION.into(),
                            settlement_id: ids::random_id("set")
                                .unwrap_or_else(|_| "set_dispatch_error".into()),
                            run_id: run_id.clone(),
                            status: SettlementStatus::Rejected,
                            basis: vec!["episode_dispatch_error".into(), error.class().into()],
                            review_required: false,
                            settled_at: Utc::now().to_rfc3339(),
                            criteria_ref,
                        });
                        Completion {
                            item_id,
                            instance,
                            run_id,
                            settlement,
                        }
                    });
                    dispatched = true;
                }
            }
        }
        if park_human {
            while let Some(completion) = active.join_next().await {
                let completion = completion.map_err(|error| {
                    ForgeError::Internal(format!("episode task panic: {error}"))
                })?;
                record_completion(
                    &stream,
                    &case_id,
                    &case_events,
                    &mut events,
                    &mut case,
                    completion,
                )?;
            }
            write_json(&case_dir.join("case.json"), &case)?;
            return Ok(DispatchOutcome {
                case_id,
                state: "active",
                exit_code: 5,
            });
        }
        if dispatched {
            // Apply exactly one returned completion before looking at persisted state again.
            let completion = active
                .join_next()
                .await
                .ok_or_else(|| ForgeError::Internal("missing active episode".into()))?
                .map_err(|error| ForgeError::Internal(format!("episode task panic: {error}")))?;
            record_completion(
                &stream,
                &case_id,
                &case_events,
                &mut events,
                &mut case,
                completion,
            )?;
            write_json(&case_dir.join("case.json"), &case)?;
        }
    }
}

fn record_completion(
    stream: &LedgerStream,
    case_id: &str,
    case_events: &std::path::Path,
    events: &mut Vec<sea_forge_core::types::TraceEvent>,
    case: &mut Case,
    completion: Completion,
) -> Result<(), ForgeError> {
    stream.commit_typed(
        "settlement_event",
        vec![
            case_id.into(),
            completion.run_id.clone(),
            completion.settlement.settlement_id.clone(),
        ],
        &completion.settlement,
        vec![],
    )?;
    CaseRunner::append_event(
        case_events,
        stream,
        events,
        TraceKind::SettlementRecorded,
        Some(&completion.item_id),
        serde_json::json!({"instance": completion.instance, "status": completion.settlement.status}),
    )?;
    CaseRunner::apply_episode_completion(
        case,
        &completion.run_id,
        &completion.item_id,
        completion.instance,
        &completion.settlement,
        case_events,
        stream,
        events,
    )
    .map(|_| ())
}

async fn execute_agent(
    config: &crate::ServerConfig,
    item: &sea_forge_core::types::PlanItem,
    policy: &str,
    entity: &str,
    process: &str,
    case_id: &str,
    run_id: &str,
) -> Result<SettlementEvent, ForgeError> {
    let Operation::AgentTask {
        endpoint_ref,
        instruction,
        max_turns,
        token_budget,
        ..
    } = &item.operations[0]
    else {
        return Err(ForgeError::Input(
            "agent task missing agent operation".into(),
        ));
    };
    let outcome = delegation::execute_with_control(
        config,
        delegation::DelegationRequest {
            endpoint_id: endpoint_ref,
            instruction,
            model: None,
            max_turns: *max_turns,
            token_budget: *token_budget,
            criteria: item.settlement_criteria.clone(),
            policy_path: policy,
            entity,
            process,
        },
        &agent_probe::EnvironmentCredentialResolver,
        delegation::DelegationEpisodeContext::planned(case_id, &item.plan_item_id, run_id),
        || false,
    )
    .await?;
    Ok(SettlementEvent {
        version: RECORD_VERSION.into(),
        settlement_id: ids::random_id("set")?,
        run_id: run_id.into(),
        status: outcome.settlement,
        basis: vec!["delegation_completed".into()],
        review_required: false,
        settled_at: Utc::now().to_rfc3339(),
        criteria_ref: item.settlement_criteria_ref.clone(),
    })
}

fn execute_sandbox(
    config: &crate::ServerConfig,
    item: &sea_forge_core::types::PlanItem,
    policy_path: &str,
    entity: &str,
    _process: &str,
    case_id: &str,
    run_id: &str,
    root: PathBuf,
    timeout: u64,
) -> Result<SettlementEvent, ForgeError> {
    let operation = item
        .operations
        .first()
        .ok_or_else(|| ForgeError::Input("sandbox task missing operation".into()))?
        .clone();
    let workspace = root
        .join("cases")
        .join(case_id)
        .join("runs")
        .join(run_id)
        .join("workspace");
    let artifacts = workspace.parent().unwrap().join("artifacts");
    fs::create_dir_all(&workspace).map_err(|e| ForgeError::io("create workspace", e))?;
    fs::create_dir_all(&artifacts).map_err(|e| ForgeError::io("create artifacts", e))?;
    let policy = agent_probe::resolve_policy_path(&config.root, policy_path);
    let bundle = AuthorityPolicyBundle::load(&policy)?;
    let engine = PolicyAuthorityEngine::new(bundle.clone())?;
    let actor = Actor {
        actor_id: entity.into(),
        role: ActorRole::Operator,
    };
    let action = sea_forge_core::types::AuthorityAction::from(&operation);
    let decision = engine.evaluate(AuthorityEvaluation {
        actor: &actor,
        binding: bundle.resolve_identity(entity, ActorRole::Operator),
        run_id,
        case_id,
        plan_item_id: &item.plan_item_id,
        sequence: 1,
        action: &action,
        workspace_root: &workspace,
        evidence_refs: vec![],
        artifacts_root: Some(&artifacts),
        timeout_secs: Some(timeout),
        env_keys: ["PATH", "HOME"].into_iter().map(str::to_owned).collect(),
        domainforge_candidate: None,
        environment: None,
    })?;
    let ledger = LedgerStream::open(&root, format!("case-{case_id}"), entity)?;
    let decision_ref = ledger.commit_typed(
        "authority_decision",
        vec![run_id.into(), item.plan_item_id.clone()],
        &decision,
        vec![],
    )?;
    let execution = if decision.verdict == Verdict::Allow {
        let grant = engine.grant(&decision, &decision_ref, &action, None)?;
        sea_forge_runtime::execute(
            grant,
            &ExecutionRequest {
                plan_item_id: item.plan_item_id.clone(),
                operation,
                timeout_secs: timeout,
                env: BTreeMap::from([
                    (
                        String::from("PATH"),
                        std::env::var("PATH").unwrap_or_default(),
                    ),
                    (
                        String::from("HOME"),
                        std::env::var("HOME").unwrap_or_default(),
                    ),
                ]),
                compensating_controls: decision.compensating_controls.clone(),
            },
            run_id,
            &workspace,
            &artifacts,
        )?
    } else {
        sea_forge_core::types::ExecutionResult {
            status: sea_forge_core::types::ExecutionStatus::SpawnFailed,
            exit_code: None,
            stdout_path: String::new(),
            stderr_path: String::new(),
            started_at: Utc::now().to_rfc3339(),
            finished_at: Utc::now().to_rfc3339(),
        }
    };
    let status = if decision.verdict == Verdict::Allow
        && execution.status == sea_forge_core::types::ExecutionStatus::Completed
        && execution.exit_code == Some(0)
    {
        SettlementStatus::Accepted
    } else {
        SettlementStatus::Rejected
    };
    Ok(SettlementEvent {
        version: RECORD_VERSION.into(),
        settlement_id: ids::random_id("set")?,
        run_id: run_id.into(),
        status,
        basis: vec![if decision.verdict == Verdict::Allow {
            "execution_completed".into()
        } else {
            "authority_denied".into()
        }],
        review_required: false,
        settled_at: Utc::now().to_rfc3339(),
        criteria_ref: item.settlement_criteria_ref.clone(),
    })
}
