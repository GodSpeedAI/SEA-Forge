use chrono::Utc;
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
pub use sea_forge_case_runner::write_json;
use sea_forge_case_runner::CaseRunner;
use sea_forge_core::{errors::ForgeError, ids, types::*, RECORD_VERSION};
use sea_forge_ledger::{CommittedRecordRef, LedgerStream};
use sea_forge_planner::case_engine::{validate_proposal, CaseAction};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

pub fn append_event(
    path: &Path,
    stream: &LedgerStream,
    events: &mut Vec<TraceEvent>,
    kind: TraceKind,
    item_id: Option<&str>,
    payload: serde_json::Value,
) -> Result<(), ForgeError> {
    CaseRunner::append_event(path, stream, events, kind, item_id, payload)
}

pub struct PlanRunOptions {
    pub plan: PathBuf,
    pub policy: PathBuf,
    pub root: PathBuf,
    pub timeout_secs: u64,
    pub entity: String,
    pub process: String,
    pub intent_summary: Option<String>,
    pub origin_evidence_refs: Vec<String>,
}

pub struct PlanRunOutcome {
    pub case_id: String,
    pub state: &'static str,
    pub exit_code: u8,
}

pub struct ApprovalParkingContext {
    pub case_id: String,
    pub run_id: String,
    pub plan_item_id: String,
    pub criteria: SettlementCriteriaRecord,
    pub decision: AuthorityDecision,
    pub committed_decision: CommittedRecordRef,
    pub approval: ApprovalRequest,
    pub committed_approval: CommittedRecordRef,
}

pub struct PlanApprovalExtension {
    pub plan_item_id: String,
    pub action: AuthorityAction,
    pub before_awaiting_approval: Box<dyn FnOnce(ApprovalParkingContext) -> Result<(), ForgeError>>,
}

struct AuthorizedOperation {
    decision: AuthorityDecision,
    committed: CommittedRecordRef,
}

fn load_plan(path: &Path) -> Result<CasePlan, ForgeError> {
    let bytes = fs::read(path).map_err(|error| ForgeError::io("read plan proposal", error))?;
    serde_json::from_slice(&bytes).map_err(|error| ForgeError::Plan {
        class: "plan_schema_error",
        message: error.to_string(),
    })
}

pub fn run_plan(options: PlanRunOptions) -> Result<PlanRunOutcome, ForgeError> {
    run_plan_inner(options, None)
}

pub fn run_plan_with_approval_extension(
    options: PlanRunOptions,
    extension: PlanApprovalExtension,
) -> Result<PlanRunOutcome, ForgeError> {
    run_plan_inner(options, Some(extension))
}

fn run_plan_inner(
    options: PlanRunOptions,
    mut approval_extension: Option<PlanApprovalExtension>,
) -> Result<PlanRunOutcome, ForgeError> {
    // Proposal and policy validation precede state creation.
    let mut plan = load_plan(&options.plan)?;
    validate_proposal(&mut plan)?;
    let bundle = AuthorityPolicyBundle::load(&options.policy)?;
    let engine = PolicyAuthorityEngine::new(bundle.clone())?;

    fs::create_dir_all(&options.root)
        .map_err(|error| ForgeError::io("create state root", error))?;
    let root = options
        .root
        .canonicalize()
        .map_err(|error| ForgeError::io("canonicalize state root", error))?;
    let case_id = ids::case_id()?;
    plan.case_id.clone_from(&case_id);
    let case_dir = root.join("cases").join(&case_id);
    let (runs_dir, case_events) = CaseRunner::initialize_case(&case_dir)?;
    let stream = LedgerStream::open(&root, format!("case-{case_id}"), &options.entity)?;
    stream.commit_typed("case_plan", vec![case_id.clone()], &plan, vec![])?;

    let intent = Intent {
        intent_id: ids::random_id("int")?,
        summary: options
            .intent_summary
            .clone()
            .unwrap_or_else(|| format!("Execute plan {}", options.plan.display())),
        actor_id: options.entity.clone(),
        process_id: options.process.clone(),
        created_at: Utc::now().to_rfc3339(),
    };
    let actor = Actor {
        actor_id: options.entity.clone(),
        role: ActorRole::Operator,
    };
    let binding = bundle.resolve_identity(&actor.actor_id, actor.role.clone());

    // Real desired-outcome resolver over Task 11's validated self-model seed
    // (audit remediation Task 12) — never the fail-closed NoModelResolver
    // wrapper for externally supplied plans.
    let (desired_outcome_resolver, seed_domain_model_ref, seed_model_sha256) =
        crate::pipeline::seed_desired_outcome_resolver()?;

    // A submitted plan naming a known built-in template derives its criteria
    // from that template (desired-outcome provenance included) rather than
    // intent-only provenance; materializing it through the source-owned
    // installer (`store_builtin`) and loading the pinned, tamper-evident copy
    // (`load_pinned`) is the same path any other consumer of that template
    // would take (Task 12 audit remediation step 3).
    let built_in_template = match plan.template_ref.as_deref() {
        Some(reference) if sea_forge_planner::is_built_in_template_ref(reference) => {
            sea_forge_planner::store_builtin(
                &root,
                &seed_domain_model_ref,
                &seed_model_sha256,
                sea_forge_planner::DEFAULT_TOPOLOGY_ENDPOINT_REF,
            )?;
            Some(sea_forge_planner::load_pinned(&root, reference)?)
        }
        _ => None,
    };

    // Derive and commit settlement criteria records for every settling item.
    let mut criteria_map: BTreeMap<String, SettlementCriteriaRecord> = BTreeMap::new();
    let declared_at = Utc::now().to_rfc3339();
    for item in &mut plan.items {
        let mut record = if let Some(template) = &built_in_template {
            sea_forge_planner::derive_from_template(
                template,
                item,
                &intent,
                &options.entity,
                &declared_at,
            )?
        } else {
            sea_forge_planner::derive_from_intent(&intent, item, &options.entity, &declared_at)?
        };
        if !options.origin_evidence_refs.is_empty() {
            // `derive_from_template` sorts `origin_refs` by `reference`, so the
            // Intent origin is not reliably at index 0 once a template is in
            // play; look it up by kind instead.
            let origin = record
                .origin_refs
                .iter_mut()
                .find(|o| o.kind == OriginRefKind::Intent)
                .ok_or_else(|| ForgeError::Plan {
                    class: "criteria_origin_missing",
                    message: format!(
                        "cannot attach origin_evidence_refs: no Intent origin found in criteria {}",
                        record.criteria_id
                    ),
                })?;
            origin
                .evidence_refs
                .clone_from(&options.origin_evidence_refs);
            record.criteria_record_hash = sea_forge_planner::compute_record_hash(&record)?;
        }
        let committed = stream.commit_typed(
            "settlement_criteria",
            vec![case_id.clone(), record.criteria_id.clone()],
            &record,
            vec![],
        )?;
        let _ = committed;
        item.settlement_criteria_ref = Some(record.criteria_id.clone());
        criteria_map.insert(record.criteria_id.clone(), record);
    }
    sea_forge_planner::verify_plan_criteria_with_resolver(
        &plan,
        &criteria_map,
        &desired_outcome_resolver,
    )?;

    // Allocate every potential episode and decide every operation before activation.
    let mut run_ids = HashMap::<(String, u32), String>::new();
    let mut auth = HashMap::<(String, u32), Vec<AuthorizedOperation>>::new();
    let mut approval_authority: Option<AuthorizedOperation> = None;
    for item in &plan.items {
        if item.item_kind != ItemKind::SandboxedTask {
            continue;
        }
        for instance in 1..=item.max_instances {
            let mut has_approval_gate = false;
            let run_id = ids::run_id()?;
            let workspace = runs_dir.join(&run_id).join("workspace");
            let artifacts = runs_dir.join(&run_id).join("artifacts");
            let env_spec = crate::pipeline::load_item_environment(&root, item)?;
            let evaluator = crate::pipeline::resolve_item_evaluator(item, env_spec.as_ref())?;
            let mut item_operations = item.operations.clone();
            if let Some((_, sea_forge_sandbox::Evaluator::Command { argv, .. })) = &evaluator {
                item_operations.push(Operation::ExecuteCommand {
                    argv: argv.clone(),
                    cwd: ".".into(),
                });
            }
            let environment = env_spec.as_ref().map(|spec| {
                (
                    item.environment.as_deref().unwrap_or(""),
                    spec.provides.commands.as_slice(),
                )
            });
            if instance == 1
                && approval_extension
                    .as_ref()
                    .is_some_and(|extension| extension.plan_item_id == item.plan_item_id)
            {
                let action = &approval_extension
                    .as_ref()
                    .ok_or_else(|| ForgeError::Internal("approval extension is missing".into()))?
                    .action;
                let evidence = stream.commit_typed(
                    "authority_evidence",
                    vec![run_id.clone(), item.plan_item_id.clone()],
                    &json!({"kind": "artifact_transition", "action": action}),
                    vec![],
                )?;
                let decision = engine.evaluate(AuthorityEvaluation {
                    actor: &actor,
                    binding: binding.clone(),
                    run_id: &run_id,
                    case_id: &case_id,
                    plan_item_id: &item.plan_item_id,
                    sequence: 1,
                    action,
                    workspace_root: &workspace,
                    evidence_refs: vec![evidence.entry_ulid().into()],
                    artifacts_root: None,
                    timeout_secs: None,
                    env_keys: Default::default(),
                    domainforge_candidate: None,
                    environment: None,
                })?;
                let request = stream.commit_typed(
                    "authority_request",
                    vec![run_id.clone(), item.plan_item_id.clone()],
                    &decision.action_request,
                    vec![],
                )?;
                let committed = stream.commit_typed(
                    "authority_decision",
                    vec![run_id.clone(), item.plan_item_id.clone()],
                    &decision,
                    vec![request.entry_ulid().into()],
                )?;
                approval_authority = Some(AuthorizedOperation {
                    decision,
                    committed,
                });
                has_approval_gate = true;
            }
            let mut operations = Vec::new();
            for (sequence, operation) in item_operations.iter().enumerate() {
                let action = AuthorityAction::from(operation);
                let evidence = stream.commit_typed(
                    "authority_evidence",
                    vec![run_id.clone(), item.plan_item_id.clone()],
                    &json!({"kind": "plan_operation", "action": action}),
                    vec![],
                )?;
                let env_keys = if matches!(operation, Operation::ExecuteCommand { .. }) {
                    ["PATH", "HOME"].into_iter().map(str::to_owned).collect()
                } else {
                    Default::default()
                };
                let decision = engine.evaluate(AuthorityEvaluation {
                    actor: &actor,
                    binding: binding.clone(),
                    run_id: &run_id,
                    case_id: &case_id,
                    plan_item_id: &item.plan_item_id,
                    sequence: sequence + 1 + usize::from(has_approval_gate),
                    action: &action,
                    workspace_root: &workspace,
                    evidence_refs: vec![evidence.entry_ulid().into()],
                    artifacts_root: matches!(operation, Operation::ExecuteCommand { .. })
                        .then_some(artifacts.as_path()),
                    timeout_secs: matches!(operation, Operation::ExecuteCommand { .. })
                        .then_some(options.timeout_secs),
                    env_keys,
                    domainforge_candidate: None,
                    environment,
                })?;
                let request = stream.commit_typed(
                    "authority_request",
                    vec![run_id.clone(), item.plan_item_id.clone()],
                    &decision.action_request,
                    vec![],
                )?;
                let committed = stream.commit_typed(
                    "authority_decision",
                    vec![run_id.clone(), item.plan_item_id.clone()],
                    &decision,
                    vec![request.entry_ulid().into()],
                )?;
                operations.push(AuthorizedOperation {
                    decision,
                    committed,
                });
            }
            run_ids.insert((item.plan_item_id.clone(), instance), run_id);
            auth.insert((item.plan_item_id.clone(), instance), operations);
        }
    }
    if approval_extension.is_some()
        && approval_authority
            .as_ref()
            .is_none_or(|operation| operation.decision.verdict != Verdict::Escalate)
    {
        return Err(ForgeError::Plan {
            class: "artifact_transition_authority_error",
            message: "external transition gate requires an exact escalated decision".into(),
        });
    }

    // Check for escalated items and create approval requests.
    let now = Utc::now();
    let now_rfc = now.to_rfc3339();
    // ponytail: TTL from policy or default 24h; policy parsing added when server lands
    let expires_at = (now + chrono::Duration::hours(24)).to_rfc3339();
    let mut escalated: Option<(String, String)> = approval_authority
        .as_ref()
        .filter(|operation| operation.decision.verdict == Verdict::Escalate)
        .map(|operation| {
            (
                operation.decision.plan_item_id.clone(),
                operation.decision.decision_id.clone(),
            )
        });
    'outer: for ((item_id, _instance), ops) in &auth {
        if escalated.is_some() {
            break;
        }
        for op in ops {
            if op.decision.verdict == Verdict::Escalate {
                escalated = Some((item_id.clone(), op.decision.decision_id.clone()));
                break 'outer;
            }
        }
    }

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
    append_event(
        &case_events,
        &stream,
        &mut events,
        TraceKind::CaseCreated,
        None,
        json!({"case_id": case_id}),
    )?;

    // If any item escalated, create an approval request and park the case.
    if let Some((item_id, decision_id)) = escalated {
        let run_id = run_ids
            .get(&(item_id.clone(), 1))
            .cloned()
            .unwrap_or_default();
        let criteria_ref = plan
            .items
            .iter()
            .find(|item| item.plan_item_id == item_id)
            .and_then(|item| item.settlement_criteria_ref.as_ref())
            .ok_or_else(|| ForgeError::Internal("approval item has no criteria ref".into()))?;
        let criteria = criteria_map
            .get(criteria_ref)
            .ok_or_else(|| ForgeError::Internal("approval criteria record is missing".into()))?;
        let approval = ApprovalRequest {
            version: RECORD_VERSION.into(),
            approval_id: ids::seq_id("apr", 4, 1),
            run_id: run_id.clone(),
            case_id: case_id.clone(),
            decision_id: decision_id.clone(),
            plan_item_id: item_id.clone(),
            criteria_ref: Some(criteria_ref.clone()),
            criteria_sha256: Some(criteria.criteria_sha256.clone()),
            criteria_record_hash: Some(criteria.criteria_record_hash.clone()),
            job_contract_ref: None,
            requested_at: now_rfc.clone(),
            expires_at: expires_at.clone(),
            status: ApprovalStatus::Pending,
            resolved_by: None,
            resolved_at: None,
            note: None,
        };
        let committed_approval = stream.commit_typed(
            "approval_request",
            vec![case_id.clone(), approval.approval_id.clone()],
            &approval,
            vec![],
        )?;
        let run_dir = runs_dir.join(&run_id);
        fs::create_dir_all(&run_dir)
            .map_err(|error| ForgeError::io("create parked run directory", error))?;
        write_json(&run_dir.join("plan.json"), &plan)?;
        let mut decisions: Vec<&AuthorityDecision> = auth
            .get(&(item_id.clone(), 1))
            .into_iter()
            .flatten()
            .map(|operation| &operation.decision)
            .collect();
        if let Some(operation) = approval_authority.as_ref() {
            decisions.push(&operation.decision);
        }
        write_json(&run_dir.join("authority.json"), &decisions)?;
        case.run_ids.push(run_id.clone());
        if approval_authority
            .as_ref()
            .is_some_and(|operation| operation.decision.decision_id == decision_id)
        {
            let extension = approval_extension.take().ok_or_else(|| {
                ForgeError::Internal("approval extension callback is missing".into())
            })?;
            let operation = approval_authority.take().ok_or_else(|| {
                ForgeError::Internal("approval extension decision is missing".into())
            })?;
            (extension.before_awaiting_approval)(ApprovalParkingContext {
                case_id: case_id.clone(),
                run_id: run_id.clone(),
                plan_item_id: item_id,
                criteria: criteria.clone(),
                decision: operation.decision,
                committed_decision: operation.committed,
                approval: approval.clone(),
                committed_approval,
            })?;
        }
        crate::approvals::append(&root, &approval)?;
        case.state = CaseState::AwaitingApproval;
        write_json(&case_dir.join("case.json"), &case)?;
        return Ok(PlanRunOutcome {
            case_id,
            state: "awaiting_approval",
            exit_code: 5,
        });
    }

    loop {
        let actions = CaseRunner::next_ready_actions(&plan.items, &events);
        if actions.is_empty() {
            write_json(&case_dir.join("case.json"), &case)?;
            return Ok(PlanRunOutcome {
                case_id,
                state: "active",
                exit_code: 5,
            });
        }
        for action in actions {
            match action {
                CaseAction::Enable(item_id) => append_event(
                    &case_events,
                    &stream,
                    &mut events,
                    TraceKind::ItemEnabled,
                    Some(&item_id),
                    json!({}),
                )?,
                CaseAction::ParkHumanTask(item_id) => {
                    append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::ItemActivated,
                        Some(&item_id),
                        json!({"human_task": true}),
                    )?;
                    write_json(&case_dir.join("case.json"), &case)?;
                    return Ok(PlanRunOutcome {
                        case_id,
                        state: "active",
                        exit_code: 5,
                    });
                }
                CaseAction::AchieveMilestone(item_id) => append_event(
                    &case_events,
                    &stream,
                    &mut events,
                    TraceKind::MilestoneAchieved,
                    Some(&item_id),
                    json!({}),
                )?,
                CaseAction::Activate(item_id) => {
                    let projection =
                        sea_forge_planner::case_engine::replay_case(&plan.items, &events);
                    let instance = projection
                        .items
                        .iter()
                        .find(|state| state.item_id == item_id)
                        .map_or(1, |state| state.instances + 1);
                    append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::ItemActivated,
                        Some(&item_id),
                        json!({"instance": instance}),
                    )?;
                    let item = plan
                        .items
                        .iter()
                        .find(|item| item.plan_item_id == item_id)
                        .ok_or_else(|| ForgeError::Internal("missing plan item".into()))?;
                    let run_id = if item.item_kind == ItemKind::AgentTask {
                        ids::run_id()?
                    } else {
                        run_ids
                            .get(&(item_id.clone(), instance))
                            .cloned()
                            .ok_or_else(|| {
                                ForgeError::Internal("missing allocated episode".into())
                            })?
                    };
                    let run_dir = runs_dir.join(&run_id);
                    let workspace = run_dir.join("workspace");
                    let artifacts = run_dir.join("artifacts");
                    fs::create_dir_all(&workspace)
                        .and_then(|_| fs::create_dir_all(&artifacts))
                        .map_err(|error| ForgeError::io("create episode directories", error))?;

                    // Agent tasks delegate to sea-forge-server; no local
                    // execution. Authority, transcript evidence, and
                    // settlement are handled by the server delegation
                    // service (spec §10.2, T13.1).
                    if item.item_kind == ItemKind::AgentTask {
                        let (endpoint_ref, instruction, max_turns, token_budget) =
                            match &item.operations[0] {
                                Operation::AgentTask {
                                    endpoint_ref,
                                    instruction,
                                    max_turns,
                                    token_budget,
                                    ..
                                } => (
                                    endpoint_ref.as_str(),
                                    instruction.as_str(),
                                    *max_turns,
                                    *token_budget,
                                ),
                                _ => unreachable!("validated by case_engine"),
                            };
                        let delegate_response = crate::commands::agent::request(
                            &root,
                            json!({
                                "verb": "delegate",
                                "endpoint": endpoint_ref,
                                "instruction": instruction,
                                "max_turns": max_turns,
                                "token_budget": token_budget,
                                "criteria": item.settlement_criteria,
                                "policy": options.policy.to_string_lossy(),
                                "entity": options.entity,
                                "process": options.process,
                            }),
                        )?;
                        let status = match delegate_response["settlement"].as_str() {
                            Some("accepted") => SettlementStatus::Accepted,
                            _ => SettlementStatus::Rejected,
                        };
                        let settlement = SettlementEvent {
                            version: RECORD_VERSION.into(),
                            settlement_id: ids::random_id("set")?,
                            run_id: run_id.clone(),
                            status: status.clone(),
                            basis: vec!["agent_task_delegated".into()],
                            review_required: false,
                            settled_at: Utc::now().to_rfc3339(),
                            criteria_ref: item.settlement_criteria_ref.clone(),
                        };
                        stream.commit_typed(
                            "settlement_event",
                            vec![
                                case_id.clone(),
                                run_id.clone(),
                                settlement.settlement_id.clone(),
                            ],
                            &settlement,
                            vec![],
                        )?;
                        write_json(&run_dir.join("settlement.json"), &settlement)?;
                        append_event(
                            &case_events,
                            &stream,
                            &mut events,
                            TraceKind::SettlementRecorded,
                            Some(&item_id),
                            json!({"instance": instance, "status": settlement.status}),
                        )?;
                        CaseRunner::apply_episode_completion(
                            &mut case,
                            &run_id,
                            &item_id,
                            instance,
                            &settlement,
                            &case_events,
                            &stream,
                            &mut events,
                        )?;
                        write_json(&case_dir.join("case.json"), &case)?;
                        continue;
                    }

                    let env_spec = crate::pipeline::load_item_environment(&root, item)?;
                    if let Some(spec) = &env_spec {
                        sea_forge_sandbox::environment::materialize_base(spec, &workspace)?;
                    }
                    let evaluator =
                        crate::pipeline::resolve_item_evaluator(item, env_spec.as_ref())?;
                    let mut operations = item.operations.clone();
                    let evaluator_index = evaluator.as_ref().map(|(_, evaluator)| {
                        let sea_forge_sandbox::Evaluator::Command { argv, .. } = evaluator;
                        operations.push(Operation::ExecuteCommand {
                            argv: argv.clone(),
                            cwd: ".".into(),
                        });
                        operations.len() - 1
                    });
                    let authorized = auth
                        .get(&(item_id.clone(), instance))
                        .ok_or_else(|| ForgeError::Internal("missing episode authority".into()))?;
                    let mut execution = None;
                    let mut evaluator_scores = BTreeMap::new();
                    let allowed = authorized
                        .iter()
                        .all(|operation| operation.decision.verdict == Verdict::Allow);
                    if allowed {
                        for (index, operation) in operations.iter().enumerate() {
                            let granted = &authorized[index];
                            if matches!(operation, Operation::AgentProbe { .. }) {
                                return Err(ForgeError::Input(
                                    "agent_probe must be dispatched by sea-forge-server".into(),
                                ));
                            }
                            let grant = engine.grant(
                                &granted.decision,
                                &granted.committed,
                                &AuthorityAction::from(operation),
                                None,
                            )?;
                            match operation {
                                Operation::WriteFile { .. } => sea_forge_sandbox::materialize(
                                    grant,
                                    &workspace,
                                    &run_id,
                                    &item_id,
                                    operation,
                                    &granted.decision.compensating_controls,
                                )?,
                                Operation::ExecuteCommand { .. } => {
                                    let mut env = BTreeMap::new();
                                    if let Ok(value) = std::env::var("PATH") {
                                        env.insert("PATH".into(), value);
                                    }
                                    if let Ok(value) = std::env::var("HOME") {
                                        env.insert("HOME".into(), value);
                                    }
                                    let result = CaseRunner::run_sandboxed_episode(|| {
                                        sea_forge_runtime::execute(
                                            grant,
                                            &ExecutionRequest {
                                                plan_item_id: item_id.clone(),
                                                operation: operation.clone(),
                                                timeout_secs: options.timeout_secs,
                                                env,
                                                compensating_controls: granted
                                                    .decision
                                                    .compensating_controls
                                                    .clone(),
                                            },
                                            &run_id,
                                            &workspace,
                                            &artifacts,
                                        )
                                    })?;
                                    if Some(index) == evaluator_index {
                                        let (reference, evaluator) =
                                            evaluator.as_ref().ok_or_else(|| {
                                                ForgeError::Internal(
                                                    "missing resolved evaluator".into(),
                                                )
                                            })?;
                                        let sea_forge_sandbox::Evaluator::Command {
                                            score_from,
                                            ..
                                        } = evaluator;
                                        let score =
                                            sea_forge_sandbox::environment::parse_evaluator_score(
                                                score_from, &result, &run_dir,
                                            )?;
                                        evaluator_scores.insert(reference.clone(), score);
                                        if item.operations.is_empty() {
                                            execution = Some(result);
                                        }
                                    } else {
                                        execution = Some(result);
                                    }
                                }
                                Operation::AgentProbe { .. } => {
                                    unreachable!("agent_probe is rejected before execution")
                                }
                                Operation::AgentTask { .. } => {
                                    unreachable!("agent_task is rejected before execution")
                                }
                            }
                        }
                    }
                    if execution.is_none() && allowed {
                        execution = Some(ExecutionResult {
                            status: ExecutionStatus::Completed,
                            exit_code: Some(0),
                            stdout_path: "artifacts/stdout.txt".into(),
                            stderr_path: "artifacts/stderr.txt".into(),
                            started_at: Utc::now().to_rfc3339(),
                            finished_at: Utc::now().to_rfc3339(),
                        });
                        fs::write(artifacts.join("stdout.txt"), b"")
                            .map_err(|error| ForgeError::io("write stdout", error))?;
                        fs::write(artifacts.join("stderr.txt"), b"")
                            .map_err(|error| ForgeError::io("write stderr", error))?;
                    }
                    let settlement = sea_forge_settlement::settle(
                        &SettlementClaim {
                            run_id: run_id.clone(),
                            plan_item_id: item_id.clone(),
                            criteria_ref: item.settlement_criteria_ref.clone(),
                            criteria: item.settlement_criteria.clone(),
                            execution,
                            authority_verdicts: authorized
                                .iter()
                                .map(|operation| operation.decision.verdict.clone())
                                .collect(),
                            evaluator_scores,
                            batch: None,
                        },
                        &workspace,
                        &run_dir,
                    )?;
                    stream.commit_typed(
                        "settlement_event",
                        vec![
                            case_id.clone(),
                            run_id.clone(),
                            settlement.settlement_id.clone(),
                        ],
                        &settlement,
                        authorized
                            .iter()
                            .map(|operation| operation.decision.decision_id.clone())
                            .collect(),
                    )?;
                    write_json(&run_dir.join("plan.json"), &plan)?;
                    write_json(
                        &run_dir.join("authority.json"),
                        &authorized
                            .iter()
                            .map(|operation| &operation.decision)
                            .collect::<Vec<_>>(),
                    )?;
                    write_json(&run_dir.join("settlement.json"), &settlement)?;
                    CaseRunner::apply_episode_completion(
                        &mut case,
                        &run_id,
                        &item_id,
                        instance,
                        &settlement,
                        &case_events,
                        &stream,
                        &mut events,
                    )?;
                    write_json(&case_dir.join("case.json"), &case)?;
                }
                CaseAction::CompleteCase => {
                    case.state = CaseState::Completed;
                    case.closed_at = Some(Utc::now().to_rfc3339());
                    append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::CaseClosed,
                        None,
                        json!({}),
                    )?;
                    write_json(&case_dir.join("case.json"), &case)?;
                    return Ok(PlanRunOutcome {
                        case_id,
                        state: "completed",
                        exit_code: 0,
                    });
                }
                CaseAction::TerminateCase { blocking_item } => {
                    case.state = CaseState::Terminated;
                    case.close_reason = Some(format!("required_item_failed:{blocking_item}"));
                    case.closed_at = Some(Utc::now().to_rfc3339());
                    append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::CaseTerminated,
                        None,
                        json!({"blocking_item": blocking_item}),
                    )?;
                    write_json(&case_dir.join("case.json"), &case)?;
                    return Ok(PlanRunOutcome {
                        case_id,
                        state: "terminated",
                        exit_code: 3,
                    });
                }
            }
        }
    }
}
