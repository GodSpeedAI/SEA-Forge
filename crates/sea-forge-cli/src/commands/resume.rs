use chrono::Utc;
use sea_forge_core::{errors::ForgeError, ids, types::*};
use sea_forge_ledger::LedgerStream;
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;

use crate::approvals;
use crate::plan_pipeline::{append_event, write_json};

struct AuthorizedOperation {
    decision: AuthorityDecision,
    committed: sea_forge_ledger::CommittedRecordRef,
}

pub struct ResumeOptions {
    pub root: PathBuf,
    pub case_id: String,
    pub policy: PathBuf,
    pub timeout_secs: u64,
    pub entity: String,
    #[allow(dead_code)]
    pub process: String,
}

pub struct ResumeOutcome {
    pub case_id: String,
    pub state: &'static str,
    pub exit_code: u8,
}

pub fn resume(options: ResumeOptions) -> Result<ResumeOutcome, ForgeError> {
    let root = options
        .root
        .canonicalize()
        .map_err(|e| ForgeError::io("canonicalize root", e))?;
    let case_dir = root.join("cases").join(&options.case_id);
    if !case_dir.exists() {
        return Err(ForgeError::Input(format!(
            "case {} not found",
            options.case_id
        )));
    }

    // Load case state.
    let case: Case = serde_json::from_slice(
        &fs::read(case_dir.join("case.json")).map_err(|e| ForgeError::io("read case.json", e))?,
    )
    .map_err(|e| ForgeError::Serialization(format!("parse case.json: {e}")))?;

    if case.state != CaseState::AwaitingApproval {
        return Err(ForgeError::Input(format!(
            "case {} is {} — only awaiting_approval can resume",
            options.case_id,
            serde_json::to_string(&case.state)
                .unwrap_or_default()
                .trim_matches('"')
        )));
    }

    // Load plan.
    let plan: CasePlan = serde_json::from_slice(
        &fs::read(case_dir.join("plan.json")).map_err(|e| ForgeError::io("read plan.json", e))?,
    )
    .map_err(|e| ForgeError::Serialization(format!("parse plan.json: {e}")))?;

    // Load events.
    let case_events = case_dir.join("case-events.jsonl");
    let mut events: Vec<TraceEvent> = Vec::new();
    if case_events.exists() {
        for line in fs::read_to_string(&case_events)
            .map_err(|e| ForgeError::io("read case-events.jsonl", e))?
            .lines()
        {
            if line.trim().is_empty() {
                continue;
            }
            let event: TraceEvent = serde_json::from_str(line)
                .map_err(|e| ForgeError::Serialization(format!("parse event: {e}")))?;
            events.push(event);
        }
    }

    // Check approval status.
    let pending = approvals::pending_for_case(&root, &options.case_id)?;
    if !pending.is_empty() {
        return Err(ForgeError::Input(format!(
            "case {} still has {} pending approval(s)",
            options.case_id,
            pending.len()
        )));
    }

    // Determine the approval outcome for this case.
    let all_approvals = approvals::load_all(&root)?;
    let case_approvals: Vec<_> = all_approvals
        .iter()
        .filter(|a| a.case_id == options.case_id)
        .collect();
    let has_approved = case_approvals
        .iter()
        .any(|a| a.status == ApprovalStatus::Approved);
    let _has_expired = case_approvals
        .iter()
        .any(|a| a.status == ApprovalStatus::Expired || a.status == ApprovalStatus::Rejected);

    if !has_approved {
        // If rejected or expired, terminate the case.
        let mut case = case;
        case.state = CaseState::Terminated;
        case.close_reason = Some("authority_escalate_expired".into());
        case.closed_at = Some(Utc::now().to_rfc3339());
        write_json(&case_dir.join("case.json"), &case)?;
        let stream =
            LedgerStream::open(&root, format!("case-{}", options.case_id), &options.entity)?;
        append_event(
            &case_events,
            &stream,
            &mut events,
            TraceKind::CaseTerminated,
            None,
            json!({"basis": "authority_escalate_expired"}),
        )?;
        return Ok(ResumeOutcome {
            case_id: options.case_id,
            state: "terminated",
            exit_code: 4,
        });
    }

    // Approved: re-enter the case loop.
    let stream = LedgerStream::open(&root, format!("case-{}", options.case_id), &options.entity)?;
    let bundle = sea_forge_authority::AuthorityPolicyBundle::load(&options.policy)?;
    let engine = sea_forge_authority::PolicyAuthorityEngine::new(bundle.clone())?;
    let actor = Actor {
        actor_id: options.entity.clone(),
        role: ActorRole::Operator,
    };
    let binding = bundle.resolve_identity(&actor.actor_id, actor.role.clone());
    let runs_dir = case_dir.join("runs");

    // Reconstruct authority for items that need to execute.
    // ponytail: on resume, re-evaluate authority for items that haven't completed.
    let mut run_ids = HashMap::<(String, u32), String>::new();
    let mut auth = HashMap::<(String, u32), Vec<AuthorizedOperation>>::new();
    for item in &plan.items {
        if item.item_kind != ItemKind::SandboxedTask {
            continue;
        }
        let projection = sea_forge_planner::case_engine::replay_case(&plan.items, &events);
        let existing = projection
            .items
            .iter()
            .find(|s| s.item_id == item.plan_item_id);
        let completed = existing
            .is_some_and(|s| s.status == sea_forge_planner::case_engine::ItemStatus::Completed);
        if completed {
            continue;
        }
        for instance in 1..=item.max_instances {
            let run_id = ids::run_id()?;
            let workspace = runs_dir.join(&run_id).join("workspace");
            let artifacts = runs_dir.join(&run_id).join("artifacts");
            let mut operations = Vec::new();
            for (sequence, operation) in item.operations.iter().enumerate() {
                let action = AuthorityAction::from(operation);
                let env_keys = if matches!(operation, Operation::ExecuteCommand { .. }) {
                    ["PATH", "HOME"].into_iter().map(str::to_owned).collect()
                } else {
                    Default::default()
                };
                let decision = engine.evaluate(sea_forge_authority::AuthorityEvaluation {
                    actor: &actor,
                    binding: binding.clone(),
                    run_id: &run_id,
                    case_id: &options.case_id,
                    plan_item_id: &item.plan_item_id,
                    sequence: sequence + 1,
                    action: &action,
                    workspace_root: &workspace,
                    evidence_refs: vec![],
                    artifacts_root: matches!(operation, Operation::ExecuteCommand { .. })
                        .then_some(artifacts.as_path()),
                    timeout_secs: matches!(operation, Operation::ExecuteCommand { .. })
                        .then_some(options.timeout_secs),
                    env_keys,
                    domainforge_candidate: None,
                    environment: None,
                })?;
                let request = stream.commit_typed(
                    "authority_request",
                    vec![options.case_id.clone(), item.plan_item_id.clone()],
                    &decision.action_request,
                    vec![],
                )?;
                let committed = stream.commit_typed(
                    "authority_decision",
                    vec![options.case_id.clone(), item.plan_item_id.clone()],
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

    // Continue the case loop.
    let mut case = case;
    case.state = CaseState::Active;
    write_json(&case_dir.join("case.json"), &case)?;
    loop {
        let actions = sea_forge_planner::case_engine::next_case_actions(&plan.items, &events);
        if actions.is_empty() {
            write_json(&case_dir.join("case.json"), &case)?;
            return Ok(ResumeOutcome {
                case_id: options.case_id,
                state: "active",
                exit_code: 5,
            });
        }
        for action in actions {
            match action {
                sea_forge_planner::case_engine::CaseAction::Enable(item_id) => {
                    append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::ItemEnabled,
                        Some(&item_id),
                        json!({}),
                    )?;
                }
                sea_forge_planner::case_engine::CaseAction::ParkHumanTask(item_id) => {
                    append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::ItemActivated,
                        Some(&item_id),
                        json!({"human_task": true}),
                    )?;
                    write_json(&case_dir.join("case.json"), &case)?;
                    return Ok(ResumeOutcome {
                        case_id: options.case_id,
                        state: "active",
                        exit_code: 5,
                    });
                }
                sea_forge_planner::case_engine::CaseAction::AchieveMilestone(item_id) => {
                    append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::MilestoneAchieved,
                        Some(&item_id),
                        json!({}),
                    )?;
                }
                sea_forge_planner::case_engine::CaseAction::Activate(item_id) => {
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
                            evaluator_scores: BTreeMap::new(),
                            batch: None,
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
                    case.run_ids.push(run_id.clone());
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
                sea_forge_planner::case_engine::CaseAction::CompleteCase => {
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
                    return Ok(ResumeOutcome {
                        case_id: options.case_id,
                        state: "completed",
                        exit_code: 0,
                    });
                }
                sea_forge_planner::case_engine::CaseAction::TerminateCase { blocking_item } => {
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
                    return Ok(ResumeOutcome {
                        case_id: options.case_id,
                        state: "terminated",
                        exit_code: 3,
                    });
                }
            }
        }
    }
}
