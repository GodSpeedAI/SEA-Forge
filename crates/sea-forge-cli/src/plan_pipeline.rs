use chrono::Utc;
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::{errors::ForgeError, ids, types::*, RECORD_VERSION};
use sea_forge_ledger::{CommittedRecordRef, LedgerStream};
use sea_forge_planner::case_engine::{next_case_actions, validate_proposal, CaseAction};
use serde::Serialize;
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct PlanRunOptions {
    pub plan: PathBuf,
    pub policy: PathBuf,
    pub root: PathBuf,
    pub timeout_secs: u64,
    pub entity: String,
    pub process: String,
}

pub struct PlanRunOutcome {
    pub case_id: String,
    pub state: &'static str,
    pub exit_code: u8,
}

struct AuthorizedOperation {
    decision: AuthorityDecision,
    committed: CommittedRecordRef,
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), ForgeError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| ForgeError::io("create JSON parent", error))?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .map_err(|error| ForgeError::io("write JSON", error))
}

fn append_event(
    path: &Path,
    stream: &LedgerStream,
    events: &mut Vec<TraceEvent>,
    kind: TraceKind,
    item_id: Option<&str>,
    payload: serde_json::Value,
) -> Result<(), ForgeError> {
    let event = TraceEvent {
        version: RECORD_VERSION.into(),
        event_id: ids::seq_id("cev", 6, events.len() + 1),
        run_id: "case".into(),
        plan_item_id: item_id.map(str::to_owned),
        kind,
        actor_id: "case_engine".into(),
        timestamp: Utc::now().to_rfc3339(),
        payload,
    };
    stream.commit_typed("case_event", vec![], &event, vec![])?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| ForgeError::io("open case events", error))?;
    serde_json::to_writer(&mut file, &event)?;
    file.write_all(b"\n")
        .and_then(|_| file.flush())
        .map_err(|error| ForgeError::io("append case event", error))?;
    events.push(event);
    Ok(())
}

fn load_plan(path: &Path) -> Result<CasePlan, ForgeError> {
    let bytes = fs::read(path).map_err(|error| ForgeError::io("read plan proposal", error))?;
    serde_json::from_slice(&bytes).map_err(|error| ForgeError::Plan {
        class: "plan_schema_error",
        message: error.to_string(),
    })
}

pub fn run_plan(options: PlanRunOptions) -> Result<PlanRunOutcome, ForgeError> {
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
    let runs_dir = case_dir.join("runs");
    fs::create_dir_all(&runs_dir).map_err(|error| ForgeError::io("create case runs", error))?;
    let case_events = case_dir.join("case-events.jsonl");
    let stream = LedgerStream::open(&root, format!("case-{case_id}"), &options.entity)?;
    stream.commit_typed("case_plan", vec![case_id.clone()], &plan, vec![])?;

    let intent = Intent {
        intent_id: ids::random_id("int")?,
        summary: format!("Execute plan {}", options.plan.display()),
        actor_id: options.entity.clone(),
        process_id: options.process.clone(),
        created_at: Utc::now().to_rfc3339(),
    };
    let actor = Actor {
        actor_id: options.entity.clone(),
        role: ActorRole::Operator,
    };
    let binding = bundle.resolve_identity(&actor.actor_id, actor.role.clone());

    // Derive and commit settlement criteria records for every settling item.
    let mut criteria_map: BTreeMap<String, SettlementCriteriaRecord> = BTreeMap::new();
    let declared_at = Utc::now().to_rfc3339();
    for item in &mut plan.items {
        let record =
            sea_forge_planner::derive_from_intent(&intent, item, &options.entity, &declared_at)?;
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
    sea_forge_planner::verify_plan_criteria(&plan, &criteria_map)?;

    // Allocate every potential episode and decide every operation before activation.
    let mut run_ids = HashMap::<(String, u32), String>::new();
    let mut auth = HashMap::<(String, u32), Vec<AuthorizedOperation>>::new();
    for item in &plan.items {
        if item.item_kind != ItemKind::SandboxedTask {
            continue;
        }
        for instance in 1..=item.max_instances {
            let run_id = ids::run_id()?;
            let workspace = runs_dir.join(&run_id).join("workspace");
            let artifacts = runs_dir.join(&run_id).join("artifacts");
            let mut operations = Vec::new();
            for (sequence, operation) in item.operations.iter().enumerate() {
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
                    sequence: sequence + 1,
                    action: &action,
                    workspace_root: &workspace,
                    evidence_refs: vec![evidence.entry_ulid().into()],
                    artifacts_root: matches!(operation, Operation::ExecuteCommand { .. })
                        .then_some(artifacts.as_path()),
                    timeout_secs: matches!(operation, Operation::ExecuteCommand { .. })
                        .then_some(options.timeout_secs),
                    env_keys,
                    domainforge_candidate: None,
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

    loop {
        let actions = next_case_actions(&plan.items, &events);
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
                    let run_id = run_ids
                        .get(&(item_id.clone(), instance))
                        .cloned()
                        .ok_or_else(|| ForgeError::Internal("missing allocated episode".into()))?;
                    let run_dir = runs_dir.join(&run_id);
                    let workspace = run_dir.join("workspace");
                    let artifacts = run_dir.join("artifacts");
                    fs::create_dir_all(&workspace)
                        .and_then(|_| fs::create_dir_all(&artifacts))
                        .map_err(|error| ForgeError::io("create episode directories", error))?;
                    let item = plan
                        .items
                        .iter()
                        .find(|item| item.plan_item_id == item_id)
                        .ok_or_else(|| ForgeError::Internal("missing plan item".into()))?;
                    let authorized = auth
                        .get(&(item_id.clone(), instance))
                        .ok_or_else(|| ForgeError::Internal("missing episode authority".into()))?;
                    let mut execution = None;
                    let allowed = authorized
                        .iter()
                        .all(|operation| operation.decision.verdict == Verdict::Allow);
                    if allowed {
                        for (index, operation) in item.operations.iter().enumerate() {
                            let granted = &authorized[index];
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
                                    execution = Some(sea_forge_runtime::execute(
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
                                    )?);
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
                        },
                        &workspace,
                        &run_dir,
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
                    case.run_ids.push(run_id);
                    let accepted = settlement.status == SettlementStatus::Accepted;
                    append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        if accepted {
                            TraceKind::ItemCompleted
                        } else {
                            TraceKind::ItemFailed
                        },
                        Some(&item_id),
                        json!({"instance": instance, "settlement": settlement.status}),
                    )?;
                    if accepted {
                        append_event(
                            &case_events,
                            &stream,
                            &mut events,
                            TraceKind::MilestoneAchieved,
                            Some(&item_id),
                            json!({"instance": instance}),
                        )?;
                    }
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
