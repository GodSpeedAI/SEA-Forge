use crate::{agent_probe, delegation, DispatchOutcome, ServerState, SubmitPayload};
use chrono::Utc;
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_case_runner::{write_json, CaseRunner};
use sea_forge_core::{
    errors::ForgeError,
    ids,
    types::{
        Actor, ActorRole, ApprovalRequest, ApprovalStatus, Case, CasePlan, CaseState,
        CriteriaDerivation, DerivationMethod, EvidenceKind, ExecutionRequest, ItemKind, Operation,
        OriginRef, OriginRefKind, OriginRole, SettlementCriteriaRecord, SettlementEvent,
        SettlementStatus, TraceKind, Verdict,
    },
    RECORD_VERSION,
};
use sea_forge_evidence::JsonlEvidenceWriter;
use sea_forge_ledger::LedgerStream;
use sea_forge_planner::case_engine::{replay_case, validate_proposal, CaseAction};
use sea_forge_trace::JsonlTraceRecorder;
use std::{collections::BTreeMap, fs, path::PathBuf, sync::Arc};
use tokio::task::JoinSet;

struct Completion {
    item_id: String,
    instance: u32,
    run_id: String,
    settlement: SettlementEvent,
    required: bool,
}

/// Server-owned, case-local dispatch. Active tasks and their permits live only
/// in this function; all scheduling decisions are re-derived from case events.
/// Upper bound for client-controlled episode/approval timeouts, clamped at the
/// protocol boundary (F-19/SUP-09a). Keeps a pathological `u64` from flipping
/// negative through `as i64`, pinning a semaphore permit, or creating an
/// already-expired approval.
pub(crate) const MAX_TIMEOUT_SECS: u64 = 7 * 24 * 60 * 60;

pub(crate) async fn submit(
    mut payload: SubmitPayload,
    state: &Arc<ServerState>,
) -> Result<DispatchOutcome, ForgeError> {
    let _ = &payload.intent;
    payload.timeout = payload.timeout.min(MAX_TIMEOUT_SECS);
    let plan_path = payload
        .plan
        .ok_or_else(|| ForgeError::Input("submit requires a plan".into()))?;
    let mut plan: CasePlan = serde_json::from_slice(
        &fs::read(&plan_path).map_err(|e| ForgeError::io("read plan proposal", e))?,
    )?;
    validate_proposal(&mut plan)?;
    fs::create_dir_all(&state.root).map_err(|e| ForgeError::io("create state root", e))?;
    let root = state
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
        // F-03: a required escalated item has parked the case; return the
        // parked state (approval resolution re-drives via the CLI resume flow).
        if case.state == CaseState::AwaitingApproval {
            write_json(&case_dir.join("case.json"), &case)?;
            return Ok(DispatchOutcome {
                case_id,
                state: "awaiting_approval",
                exit_code: 5,
            });
        }
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
        // F-02: pre-validate dispatchability of the whole batch *before* any
        // spawn. A batch mixing an executable item with a non-executable one
        // (e.g. `[SandboxedTask, Stage]`) must reject with a typed error while
        // `active` is still empty, so no already-activated episode is aborted
        // mid-dispatch and left without a terminal settlement.
        for action in &actions {
            if let CaseAction::Activate(item_id) = action {
                let item = plan
                    .items
                    .iter()
                    .find(|item| item.plan_item_id == *item_id)
                    .ok_or_else(|| ForgeError::Internal("missing plan item".into()))?;
                if !matches!(
                    item.item_kind,
                    ItemKind::SandboxedTask | ItemKind::AgentTask
                ) {
                    return Err(ForgeError::Input(format!(
                        "non_executable item kind cannot be dispatched: {:?}",
                        item.item_kind
                    )));
                }
            }
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
                    let config = state.config();
                    let policy = payload.policy.clone();
                    let entity = payload.entity.clone();
                    let process = payload.process.clone();
                    let timeout = payload.timeout;
                    let root = root.clone();
                    let permission_broker = state.permission_broker.clone();
                    let state_for_task = Arc::clone(state);
                    let case_for_task = case_id.clone();
                    let run_for_task = run_id.clone();
                    let required = item.markers.required;
                    active.spawn(async move {
                        let _permit = permit;
                        let criteria_ref = item.settlement_criteria_ref.clone();
                        let result = match item.item_kind {
                            ItemKind::AgentTask => {
                                let cancel = state_for_task
                                    .begin_planned_delegation(case_for_task.clone(), run_id.clone())
                                    .await;
                                let result = match cancel {
                                    Ok(cancel) => {
                                        execute_agent(
                                            &config,
                                            &item,
                                            &policy,
                                            &entity,
                                            &process,
                                            &case_for_task,
                                            &run_id,
                                            permission_broker,
                                            cancel,
                                        )
                                        .await
                                    }
                                    Err(error) => Err(error),
                                };
                                state_for_task.end_delegation(&run_id).await;
                                result
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
                        let settlement = result.unwrap_or_else(|error| {
                            // The basis carries the error *class* because it is a
                            // durable record with a stable vocabulary. The message
                            // is the only thing that says which policy file or
                            // which field was wrong, so it must not die here.
                            tracing::error!(
                                run_id = %run_id,
                                error_class = error.class(),
                                "episode dispatch failed: {error}"
                            );
                            SettlementEvent {
                                version: RECORD_VERSION.into(),
                                settlement_id: ids::random_id("set")
                                    .unwrap_or_else(|_| "set_dispatch_error".into()),
                                run_id: run_id.clone(),
                                status: SettlementStatus::Rejected,
                                basis: vec!["episode_dispatch_error".into(), error.class().into()],
                                review_required: false,
                                settled_at: Utc::now().to_rfc3339(),
                                criteria_ref,
                            }
                        });
                        Completion {
                            item_id,
                            instance,
                            run_id,
                            settlement,
                            required,
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
    // F-03: a required item that escalated parks the case as awaiting approval
    // instead of being folded into `ItemFailed` → `TerminateCase`. The approval
    // stays resolvable and the CLI `resume` flow re-drives the case.
    if completion.settlement.status == SettlementStatus::Escalated && completion.required {
        case.state = CaseState::AwaitingApproval;
        case.close_reason = Some(format!("awaiting_approval:{}", completion.item_id));
    }
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
        serde_json::json!({
            "instance": completion.instance,
            "run_id": completion.run_id,
            "status": completion.settlement.status,
        }),
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

#[allow(clippy::too_many_arguments)]
async fn execute_agent(
    config: &crate::ServerConfig,
    item: &sea_forge_core::types::PlanItem,
    policy: &str,
    entity: &str,
    process: &str,
    case_id: &str,
    run_id: &str,
    permission_broker: delegation::AcpApprovalBroker,
    cancel: Arc<std::sync::atomic::AtomicBool>,
) -> Result<SettlementEvent, ForgeError> {
    let Operation::AgentTask {
        endpoint_ref,
        instruction,
        max_turns,
        token_budget,
        response_schema,
        transcript_retention,
    } = &item.operations[0]
    else {
        return Err(ForgeError::Input(
            "agent task missing agent operation".into(),
        ));
    };
    // Resolve retention once, here, through the full precedence chain (spec
    // §8.1: plan item → endpoint → global → summarized default) and pass the
    // typed mode into delegation (M13 T16 step 3).
    let resolved_retention = sea_forge_agent::TranscriptRetentionMode::resolve(
        transcript_retention.as_deref(),
        config.agent.endpoint(endpoint_ref),
        &config.agent,
    )
    .map_err(ForgeError::Input)?;
    let outcome = delegation::execute_with_permission_broker(
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
            response_schema: response_schema.as_ref(),
            transcript_retention: resolved_retention,
        },
        &agent_probe::EnvironmentCredentialResolver,
        delegation::DelegationEpisodeContext::planned(case_id, &item.plan_item_id, run_id),
        move || cancel.load(std::sync::atomic::Ordering::SeqCst),
        Some(permission_broker),
    )
    .await?;
    Ok(SettlementEvent {
        version: RECORD_VERSION.into(),
        settlement_id: ids::random_id("set")?,
        run_id: run_id.into(),
        status: outcome.settlement,
        // Reuse the exact basis delegation already committed (M13 T15) —
        // never reconstruct a synthetic one that hides how the episode
        // actually terminated (cancelled/turn-capped/endpoint error/
        // criteria mismatch).
        basis: outcome.basis,
        review_required: false,
        settled_at: Utc::now().to_rfc3339(),
        criteria_ref: item.settlement_criteria_ref.clone(),
    })
}

/// Commit the settlement criteria an escalated item's approval will be bound to,
/// returning the record.
///
/// A plan submitted to the server carries its criteria inline, with
/// `settlement_criteria_ref` usually absent — the CLI's plan pipeline mints
/// criteria records at plan time, but a plan arriving over SFWP has been
/// through no such pass. Reusing an already-committed record when the ref does
/// resolve keeps a re-escalation of the same item pointing at one immutable
/// set of criteria rather than minting a second copy per attempt.
fn commit_item_criteria(
    ledger: &LedgerStream,
    case_id: &str,
    item: &sea_forge_core::types::PlanItem,
) -> Result<SettlementCriteriaRecord, ForgeError> {
    if let Some(existing) = item.settlement_criteria_ref.as_deref() {
        let committed = ledger.read_entries()?.into_iter().find(|entry| {
            entry.record_kind == "settlement_criteria"
                && entry.payload.get("criteria_id").and_then(|id| id.as_str()) == Some(existing)
        });
        if let Some(entry) = committed {
            return serde_json::from_value(entry.payload).map_err(Into::into);
        }
    }

    let criteria = item.settlement_criteria.clone();
    let mut record = SettlementCriteriaRecord {
        version: RECORD_VERSION.into(),
        criteria_id: ids::random_id("crit")?,
        criteria_sha256: sea_forge_planner::compute_criteria_sha256(&criteria)?,
        criteria,
        // The origin is the plan item as submitted. `ImplementationDefined`
        // rather than `Intent` or `PlanTemplate` because neither is true here:
        // these criteria were declared inline by whoever wrote the plan, and
        // naming an intent that was never hashed would be a provenance claim
        // this path cannot support.
        origin_refs: vec![OriginRef {
            kind: OriginRefKind::ImplementationDefined,
            reference: format!("case:{case_id}#{}", item.plan_item_id),
            sha256: sea_forge_ledger::types::hash_canonical(item)?,
            role: OriginRole::AcceptanceSource,
            evidence_refs: vec![],
            domain_model_ref: None,
        }],
        derivation: CriteriaDerivation {
            method: DerivationMethod::Manual,
            actor_ref: "sea-forge-server".into(),
            producer_ref: None,
            rationale: format!(
                "criteria declared inline on plan item '{}' of the submitted case plan",
                item.plan_item_id
            ),
        },
        declared_at: Utc::now().to_rfc3339(),
        criteria_record_hash: String::new(),
    };
    record.criteria_record_hash = sea_forge_planner::compute_record_hash(&record)?;
    ledger.commit_typed(
        "settlement_criteria",
        vec![case_id.into(), record.criteria_id.clone()],
        &record,
        vec![],
    )?;
    Ok(record)
}

#[allow(clippy::too_many_arguments)]
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
    let run_dir = workspace.parent().unwrap().to_path_buf();
    let artifacts = run_dir.join("artifacts");
    // The run directory is where this episode's *governance* records live, so
    // it exists before authority is evaluated: a denial that wrote its trace
    // and evidence nowhere would be indistinguishable from an episode that
    // never happened. `workspace/` and `artifacts/` — the only directories the
    // operation itself would touch — stay unborn until a committed Allow
    // (AUTH-01), and are created inside the allow branch below.
    fs::create_dir_all(&run_dir).map_err(|e| ForgeError::io("create run dir", e))?;
    let mut trace = JsonlTraceRecorder::create(&run_dir.join("trace.jsonl"), run_id, entity)?;
    let mut evidence = JsonlEvidenceWriter::create(&run_dir.join("evidence.jsonl"), run_id)?;

    let policy = agent_probe::resolve_policy_path(&config.root, policy_path);
    let bundle = AuthorityPolicyBundle::load(&policy)?;
    let engine = PolicyAuthorityEngine::new(bundle.clone())?;
    let actor = Actor {
        actor_id: entity.into(),
        role: ActorRole::Operator,
    };
    let action = sea_forge_core::types::AuthorityAction::from(&operation);
    // DOM-03: the domain model judges the action *before* policy authority
    // does, exactly as `cli/src/pipeline.rs:486-498` does it. A policy with no
    // `domainforge` engine yields `None` and the evaluation is unchanged; a
    // policy that declares one but whose model has drifted fails here, with a
    // typed error, before any grant exists.
    let domainforge_candidate = bundle
        .load_domainforge_model()?
        .map(|model| {
            sea_forge_authority::DomainForgeCandidate::evaluate(
                &model,
                &action,
                vec![format!(
                    "domain-model:{}",
                    model.model_ref.semantic_model_sha256
                )],
            )
        })
        .transpose()?;
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
        domainforge_candidate: domainforge_candidate.as_ref(),
        environment: None,
    })?;
    let ledger = LedgerStream::open(&root, format!("case-{case_id}"), entity)?;
    let decision_event = trace.append(
        TraceKind::AuthorityEvaluated,
        Some(item.plan_item_id.clone()),
        serde_json::json!({
            "decision_id": decision.decision_id,
            "verdict": decision.verdict,
        }),
    )?;
    evidence.append(
        EvidenceKind::AuthorityDecision,
        decision.decision_id.clone(),
        None,
        decision_event,
        BTreeMap::new(),
    )?;
    let decision_ref = ledger.commit_typed(
        "authority_decision",
        vec![run_id.into(), item.plan_item_id.clone()],
        &decision,
        vec![],
    )?;
    if decision.verdict == Verdict::Escalate {
        // An escalation asks a human a question. Recording only "rejected"
        // would throw that question away and make the episode look decided.
        // The request is committed and mirrored into the approvals view the
        // same way delegation does it (`delegation.rs:1671-1722`); nothing
        // runs, here or later, until someone resolves it.
        let sequence = ledger
            .read_entries()?
            .iter()
            .filter(|entry| entry.record_kind == "approval_request")
            .count()
            + 1;
        // The approval must name the criteria it gates, and those criteria must
        // be committed. `approve` reads the `settlement_criteria` record, then
        // cross-checks the approval's `criteria_sha256` and
        // `criteria_record_hash` against it — so an approval carrying `None`
        // for all three is not merely thin, it is *unresolvable*: every
        // escalated episode opened an approval that parked the case forever
        // with `approval criteria reference is missing`.
        let criteria = commit_item_criteria(&ledger, case_id, item)?;
        // `approve` authorizes the resolution against the cell's active policy
        // at `<root>/authority/active-policy.json`, which only the CLI's plan
        // pipeline used to materialize (`cli/src/pipeline.rs:345-355`). A case
        // submitted over SFWP therefore escalated into an approval that could
        // never be resolved: `missing_config_error` on a file the server had
        // never written. Recorded through the ledger rather than written
        // loose, so the snapshot the approver is judged against is the one this
        // decision was actually made under.
        let committed_policy =
            ledger.commit_typed("authority_policy", vec![case_id.into()], &bundle, vec![])?;
        ledger.materialize_view(
            &committed_policy,
            &root.join("authority/active-policy.json"),
            &serde_json::to_vec_pretty(&bundle)?,
        )?;
        let requested_at = Utc::now();
        let approval = ApprovalRequest {
            version: RECORD_VERSION.into(),
            approval_id: ids::seq_id("apr", 4, sequence),
            run_id: run_id.into(),
            case_id: case_id.into(),
            decision_id: decision.decision_id.clone(),
            plan_item_id: item.plan_item_id.clone(),
            criteria_ref: Some(criteria.criteria_id.clone()),
            criteria_sha256: Some(criteria.criteria_sha256.clone()),
            criteria_record_hash: Some(criteria.criteria_record_hash.clone()),
            job_contract_ref: None,
            requested_at: requested_at.to_rfc3339(),
            expires_at: (requested_at + chrono::Duration::seconds(timeout as i64)).to_rfc3339(),
            status: ApprovalStatus::Pending,
            resolved_by: None,
            resolved_at: None,
            note: Some(format!(
                "sandbox episode {} awaits approval",
                item.plan_item_id
            )),
        };
        ledger.commit_typed(
            "approval_request",
            vec![case_id.into(), approval.approval_id.clone()],
            &approval,
            vec![decision_ref.entry_ulid().into()],
        )?;
        delegation::append_approval_view(&root, &approval)?;
    }
    let execution = if decision.verdict == Verdict::Allow {
        // The Allow is committed (`decision_ref`) before anything is written.
        fs::create_dir_all(&workspace).map_err(|e| ForgeError::io("create workspace", e))?;
        fs::create_dir_all(&artifacts).map_err(|e| ForgeError::io("create artifacts", e))?;
        trace.append(
            TraceKind::WorkspaceCreated,
            Some(item.plan_item_id.clone()),
            serde_json::json!({"workspace": workspace.display().to_string()}),
        )?;
        let grant = engine.grant(&decision, &decision_ref, &action, None)?;
        trace.append(
            TraceKind::CommandStarted,
            Some(item.plan_item_id.clone()),
            // `ActionGrant`'s fields are private, so the trace names the
            // committed decision the grant was minted from — which is the
            // cross-reference an auditor follows anyway.
            serde_json::json!({"decision_id": decision.decision_id}),
        )?;
        let result = sea_forge_runtime::execute(
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
        )?;
        let finished = trace.append(
            TraceKind::CommandFinished,
            Some(item.plan_item_id.clone()),
            serde_json::json!({
                "status": result.status,
                "exit_code": result.exit_code,
            }),
        )?;
        evidence.append(
            EvidenceKind::ExecutionResult,
            result.stdout_path.clone(),
            None,
            finished,
            BTreeMap::new(),
        )?;
        Some(result)
    } else {
        // Nothing ran. Reporting this as a `SpawnFailed` execution — as this
        // used to — describes a spawn that was attempted and failed, which is
        // not what happened and is not what an operator reading the record
        // needs to know. `None` says the truth: no execution took place.
        trace.append(
            TraceKind::RunHalted,
            Some(item.plan_item_id.clone()),
            serde_json::json!({"verdict": decision.verdict}),
        )?;
        None
    };

    // Settlement evaluates the declared criteria against the evidence the run
    // actually produced. The previous `exit_code == Some(0)` shortcut accepted
    // a command that exited zero while writing none of its required artifacts,
    // and it had no representation for `Escalate` at all — an escalation was
    // silently settled as a rejection, losing the review the verdict asked for.
    let settlement = sea_forge_settlement::settle(
        &sea_forge_core::types::SettlementClaim {
            run_id: run_id.into(),
            plan_item_id: item.plan_item_id.clone(),
            criteria_ref: item.settlement_criteria_ref.clone(),
            criteria: item.settlement_criteria.clone(),
            execution,
            authority_verdicts: vec![decision.verdict.clone()],
            evaluator_scores: BTreeMap::new(),
            batch: None,
            write_only: false,
        },
        &workspace,
        &run_dir,
    )?;

    // The ledger is the truth, but `run.get` and `case.get_overview` read
    // `settlement.json` — so a settlement that lived only in the ledger made
    // every case-dispatched run render as *unsettled*, including runs that had
    // been denied and settled. That is the one thing these views must never
    // say: `unknown != unavailable`, and "unsettled" is a claim about the run,
    // not about a missing record.
    //
    // Written after the decision it reports, and only ever as a projection of
    // the value returned above — never a second place a settlement is decided.
    write_json(&run_dir.join("settlement.json"), &settlement)?;

    // Same reason for the authority decision: `RunRecord::records` lists
    // `authority.json` as canonical, and a run whose decision is unreadable is
    // an integrity signal an operator is meant to be able to see.
    write_json(&run_dir.join("authority.json"), &vec![&decision])?;

    Ok(settlement)
}
