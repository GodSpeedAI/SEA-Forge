use chrono::Utc;
use sea_forge_core::{errors::ForgeError, ids, types::*};
use sea_forge_ledger::{LedgerEntry, LedgerStream};
use serde_json::{json, Value};
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

fn load_ledgered_case_approvals(
    entries: &[LedgerEntry],
    plan: &CasePlan,
    case_id: &str,
) -> Result<Vec<ApprovalRequest>, ForgeError> {
    let integrity_error =
        |message: &str| ForgeError::Internal(format!("ledger_integrity_error: {message}"));
    let mut requests = BTreeMap::<String, ApprovalRequest>::new();
    let mut resolutions = BTreeMap::<String, ApprovalRequest>::new();
    for entry in entries {
        let target = match entry.record_kind.as_str() {
            "approval_request" => &mut requests,
            "approval_resolution" => &mut resolutions,
            _ => continue,
        };
        let approval: ApprovalRequest = serde_json::from_value(entry.payload.clone())?;
        if target
            .insert(approval.approval_id.clone(), approval)
            .is_some()
        {
            return Err(integrity_error("duplicate approval record"));
        }
    }
    if resolutions.keys().any(|id| !requests.contains_key(id)) {
        return Err(integrity_error("approval resolution has no request"));
    }

    requests
        .into_values()
        .map(|request| {
            if request.case_id != case_id {
                return Err(integrity_error("approval request case mismatch"));
            }
            let item = plan
                .items
                .iter()
                .find(|item| item.plan_item_id == request.plan_item_id)
                .ok_or_else(|| integrity_error("approval plan item is missing"))?;
            if item.settlement_criteria_ref.as_deref() != request.criteria_ref.as_deref() {
                return Err(integrity_error(
                    "approval criteria does not match plan item",
                ));
            }
            let criteria_ref = request
                .criteria_ref
                .as_deref()
                .ok_or_else(|| integrity_error("approval criteria reference is missing"))?;
            let criteria_entries = entries
                .iter()
                .filter(|entry| {
                    entry.record_kind == "settlement_criteria"
                        && entry.payload.get("criteria_id").and_then(Value::as_str)
                            == Some(criteria_ref)
                })
                .collect::<Vec<_>>();
            if criteria_entries.len() != 1 {
                return Err(integrity_error("approval criteria record is not unique"));
            }
            let criteria: SettlementCriteriaRecord =
                serde_json::from_value(criteria_entries[0].payload.clone())?;
            if request.criteria_sha256.as_deref() != Some(criteria.criteria_sha256.as_str())
                || request.criteria_record_hash.as_deref()
                    != Some(criteria.criteria_record_hash.as_str())
            {
                return Err(integrity_error("approval criteria hash mismatch"));
            }
            let Some(resolution) = resolutions.remove(&request.approval_id) else {
                return Ok(request);
            };
            if resolution.status == ApprovalStatus::Pending
                || resolution.run_id != request.run_id
                || resolution.case_id != request.case_id
                || resolution.decision_id != request.decision_id
                || resolution.plan_item_id != request.plan_item_id
                || resolution.criteria_ref != request.criteria_ref
                || resolution.criteria_sha256 != request.criteria_sha256
                || resolution.criteria_record_hash != request.criteria_record_hash
            {
                return Err(integrity_error("approval resolution binding mismatch"));
            }
            Ok(resolution)
        })
        .collect()
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

    if let Some(completed) =
        sea_forge_artifact_ip::load_completed_transition(&root, &options.entity, &options.case_id)?
    {
        let mut case = case;
        case.state = CaseState::Completed;
        case.closed_at
            .get_or_insert_with(|| Utc::now().to_rfc3339());
        write_json(&case_dir.join("case.json"), &case)?;
        println!("declaration_id={}", completed.declaration.declaration_id);
        println!(
            "transition_token_id={}",
            completed.token.transition_token_id
        );
        return Ok(ResumeOutcome {
            case_id: options.case_id,
            state: "completed",
            exit_code: 0,
        });
    }
    if sea_forge_artifact_ip::load_transition_terminal(&root, &options.entity, &options.case_id)?
        .is_some()
    {
        let mut case = case;
        case.state = CaseState::Terminated;
        case.close_reason = Some("artifact_transition_terminal".into());
        case.closed_at
            .get_or_insert_with(|| Utc::now().to_rfc3339());
        write_json(&case_dir.join("case.json"), &case)?;
        return Ok(ResumeOutcome {
            case_id: options.case_id,
            state: "terminated",
            exit_code: 4,
        });
    }
    let case_stream =
        LedgerStream::open(&root, format!("case-{}", options.case_id), &options.entity)?;
    case_stream.verify()?;
    let case_entries = case_stream.read_entries()?;
    if case_entries
        .iter()
        .any(|entry| entry.record_kind == "pending_artifact_transition")
    {
        let pending = sea_forge_artifact_ip::load_pending_transition(
            &root,
            &options.entity,
            &options.case_id,
        )?;
        return resume_artifact_transition(options, root, case_dir, case, pending);
    }

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
    let case_approvals = load_ledgered_case_approvals(&case_entries, &plan, &options.case_id)?;
    let pending = case_approvals
        .iter()
        .filter(|approval| approval.status == ApprovalStatus::Pending)
        .count();
    if pending > 0 {
        return Err(ForgeError::Input(format!(
            "case {} still has {} pending approval(s)",
            options.case_id, pending
        )));
    }

    // Determine the approval outcome from authoritative case-ledger records.
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
            let run_id = if instance == 1 {
                case_approvals
                    .iter()
                    .find(|approval| {
                        approval.status == ApprovalStatus::Approved
                            && approval.plan_item_id == item.plan_item_id
                    })
                    .map_or_else(ids::run_id, |approval| Ok(approval.run_id.clone()))?
            } else {
                ids::run_id()?
            };
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
                                Operation::AgentProbe { .. } => {
                                    unreachable!("agent_probe is rejected before execution")
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
                    if !case.run_ids.contains(&run_id) {
                        case.run_ids.push(run_id.clone());
                    }
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

fn resume_artifact_transition(
    options: ResumeOptions,
    root: PathBuf,
    case_dir: PathBuf,
    mut case: Case,
    pending: sea_forge_artifact_ip::PendingArtifactTransition,
) -> Result<ResumeOutcome, ForgeError> {
    let stream = LedgerStream::open(&root, format!("case-{}", options.case_id), &options.entity)?;
    stream.verify()?;
    let mut entries = stream.read_entries()?;
    let mut resolutions = entries
        .iter()
        .filter(|entry| {
            entry.record_kind == "approval_resolution"
                && entry.payload.get("approval_id").and_then(Value::as_str)
                    == Some(pending.approval_ref.as_str())
        })
        .collect::<Vec<_>>();
    if resolutions.is_empty() {
        let request_entry = unique_entry(
            &entries,
            "approval_request",
            "approval_id",
            &pending.approval_ref,
        )?;
        let request: ApprovalRequest = serde_json::from_value(request_entry.payload.clone())?;
        approval_binds_pending(&request, &pending)?;
        let expires_at = chrono::DateTime::parse_from_rfc3339(&request.expires_at)
            .map_err(|_| ForgeError::Input("approval expiry is invalid".into()))?
            .with_timezone(&Utc);
        if request.status != ApprovalStatus::Pending || Utc::now() < expires_at {
            return Ok(ResumeOutcome {
                case_id: options.case_id,
                state: "awaiting_approval",
                exit_code: 5,
            });
        }
        let expired = ApprovalRequest {
            status: ApprovalStatus::Expired,
            resolved_by: None,
            resolved_at: Some(Utc::now().to_rfc3339()),
            note: Some("ttl_expired".into()),
            ..request
        };
        stream.commit_typed_once(
            "approval_resolution",
            &expired.approval_id,
            vec![pending.case_id.clone(), expired.approval_id.clone()],
            &expired,
            vec![expired.decision_id.clone()],
        )?;
        approvals::append(&root, &expired)?;
        entries = stream.read_entries()?;
        resolutions = entries
            .iter()
            .filter(|entry| {
                entry.record_kind == "approval_resolution"
                    && entry.payload.get("approval_id").and_then(Value::as_str)
                        == Some(pending.approval_ref.as_str())
            })
            .collect();
    }
    if resolutions.len() != 1 {
        return Err(ForgeError::Internal(
            "ledger_integrity_error: conflicting approval resolutions".into(),
        ));
    }
    let resolution_entry = resolutions[0];
    let approval: ApprovalRequest = serde_json::from_value(resolution_entry.payload.clone())?;
    // Resume validates exact approval binding for ALL statuses before
    // terminalizing: a detached rejected/expired resolution from another
    // transition cannot affect this pending one.
    let approval_expires = chrono::DateTime::parse_from_rfc3339(&approval.expires_at)
        .map_err(|_| ForgeError::Input("approval expiry is invalid".into()))?
        .with_timezone(&Utc);
    let approval_resolved_after_expiry = approval
        .resolved_at
        .as_deref()
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .map(|resolved| resolved.with_timezone(&Utc) > approval_expires)
        .unwrap_or(false);
    approval_binds_pending(&approval, &pending)?;
    // An approval resolved Approved before expires_at remains valid after wall
    // clock TTL; expiry only applies to unresolved pending or resolved_at
    // after expiry. Rejected/expired resolutions are terminal regardless.
    let terminal_for_expiry = approval.status == ApprovalStatus::Expired
        || (approval.status != ApprovalStatus::Approved && Utc::now() > approval_expires)
        || approval_resolved_after_expiry;
    if approval.status == ApprovalStatus::Rejected || terminal_for_expiry {
        let (status, reason, close_reason) = if approval.status == ApprovalStatus::Rejected {
            (
                sea_forge_artifact_ip::ArtifactTransitionTerminalStatus::Rejected,
                sea_forge_artifact_ip::ArtifactTransitionTerminalReason::ApprovalRejected,
                "artifact_transition_approval_rejected",
            )
        } else {
            (
                sea_forge_artifact_ip::ArtifactTransitionTerminalStatus::Expired,
                sea_forge_artifact_ip::ArtifactTransitionTerminalReason::ApprovalExpired,
                "artifact_transition_approval_expired",
            )
        };
        sea_forge_artifact_ip::commit_transition_terminal(
            &root,
            &options.entity,
            &sea_forge_artifact_ip::ArtifactTransitionTerminal {
                version: sea_forge_artifact_ip::M8_RECORD_VERSION.into(),
                pending_key: pending.pending_key,
                case_id: options.case_id.clone(),
                run_id: pending.run_id,
                status,
                reason,
                transition_token_ref: None,
                declaration_ref: None,
                terminal_at: Utc::now().to_rfc3339(),
            },
        )?;
        case.state = CaseState::Terminated;
        case.close_reason = Some(close_reason.into());
        case.closed_at = Some(Utc::now().to_rfc3339());
        write_json(&case_dir.join("case.json"), &case)?;
        return Ok(ResumeOutcome {
            case_id: options.case_id,
            state: "terminated",
            exit_code: 4,
        });
    }
    if approval.status != ApprovalStatus::Approved {
        return Ok(ResumeOutcome {
            case_id: options.case_id,
            state: "awaiting_approval",
            exit_code: 5,
        });
    }

    let decision_entry = unique_entry(
        &entries,
        "authority_decision",
        "decision_id",
        &pending.authority_decision_ref,
    )?;
    let criteria_entry = unique_entry(
        &entries,
        "settlement_criteria",
        "criteria_id",
        &pending.criteria_ref,
    )?;
    let decision: AuthorityDecision = serde_json::from_value(decision_entry.payload.clone())?;
    let criteria: SettlementCriteriaRecord =
        serde_json::from_value(criteria_entry.payload.clone())?;
    let plan: CasePlan = serde_json::from_slice(
        &fs::read(case_dir.join("plan.json"))
            .map_err(|error| ForgeError::io("read plan", error))?,
    )?;
    let item = plan
        .items
        .iter()
        .find(|item| item.plan_item_id == pending.plan_item_id)
        .ok_or_else(|| ForgeError::Internal("pending transition plan item is missing".into()))?;
    let bundle = sea_forge_authority::AuthorityPolicyBundle::load(&options.policy)?;
    let descriptor = bundle.strong_settlement_authority()?.clone();
    let engine = sea_forge_authority::PolicyAuthorityEngine::new(bundle.clone())?;
    let pre_action_assurance = if bundle.integrity_ledger.required_for_side_effects {
        let integrity = &bundle.integrity_ledger;
        let key_dir = integrity.signing_key_dir.as_deref().ok_or_else(|| {
            ForgeError::Internal("ledger_integrity_error: signing_key_dir is required".into())
        })?;
        Some(
            sea_forge_ledger::LedgerManager::new(&root)?.create_pre_action_assurance(
                key_dir,
                &integrity.signing_key_id,
                &options.entity,
                &integrity
                    .witnesses
                    .iter()
                    .map(|witness| {
                        (
                            witness.witness_id.clone(),
                            witness.key_dir.clone(),
                            witness.key_id.clone(),
                        )
                    })
                    .collect::<Vec<_>>(),
                integrity.min_witnesses,
                &[&decision_entry.committed_ref()],
            )?,
        )
    } else {
        None
    };
    let transition_grant = engine.grant_after_approval_idempotent(
        &decision,
        &decision_entry.committed_ref(),
        &decision.operation,
        &approval,
        &resolution_entry.committed_ref(),
        &criteria,
        &criteria_entry.committed_ref(),
        pre_action_assurance.as_ref(),
        &stream,
    )?;
    let actor = Actor {
        actor_id: options.entity.clone(),
        role: ActorRole::Operator,
    };
    let run_dir = case_dir.join("runs").join(&pending.run_id);
    let workspace = run_dir.join("workspace");
    let artifacts = run_dir.join("artifacts");
    fs::create_dir_all(&workspace)
        .and_then(|_| fs::create_dir_all(&artifacts))
        .map_err(|error| ForgeError::io("create transition run directories", error))?;
    let environment = crate::pipeline::load_item_environment(&root, item)?
        .ok_or_else(|| ForgeError::Input("transition evaluator environment is missing".into()))?;
    sea_forge_sandbox::environment::materialize_base(&environment, &workspace)?;
    let (verifier_ref, evaluator) =
        crate::pipeline::resolve_item_evaluator(item, Some(&environment))?
            .ok_or_else(|| ForgeError::Input("transition evaluator is missing".into()))?;
    let sea_forge_sandbox::Evaluator::Command { argv, score_from } = evaluator;
    let operation = Operation::ExecuteCommand {
        argv,
        cwd: ".".into(),
    };
    let execution_key = format!("{}:artifact-transition-execution", pending.pending_key);
    let settlement_key = format!("{}:artifact-transition-settlement", pending.pending_key);
    let mut settlement_authority_refs = Vec::new();
    let execution = if let Some(entry) = entries.iter().find(|entry| {
        entry.record_kind == "artifact_transition_execution"
            && entry.idempotency_key.as_deref() == Some(execution_key.as_str())
    }) {
        // Recover evaluator decision refs from the committed execution linkage.
        settlement_authority_refs.extend(entry.authority_refs.iter().cloned());
        serde_json::from_value(entry.payload.clone())?
    } else {
        let action = AuthorityAction::from(&operation);
        let evaluator_decision = engine.evaluate(sea_forge_authority::AuthorityEvaluation {
            actor: &actor,
            binding: bundle.resolve_identity(&actor.actor_id, actor.role.clone()),
            run_id: &pending.run_id,
            case_id: &pending.case_id,
            plan_item_id: &pending.plan_item_id,
            sequence: 2,
            action: &action,
            workspace_root: &workspace,
            evidence_refs: vec![pending.authority_decision_ref.clone()],
            artifacts_root: Some(&artifacts),
            timeout_secs: Some(options.timeout_secs),
            env_keys: ["PATH", "HOME"].into_iter().map(str::to_owned).collect(),
            domainforge_candidate: None,
            environment: Some((
                item.environment.as_deref().unwrap_or(""),
                environment.provides.commands.as_slice(),
            )),
        })?;
        let evaluator_committed = stream.commit_typed(
            "authority_decision",
            vec![pending.case_id.clone(), pending.run_id.clone()],
            &evaluator_decision,
            vec![pending.authority_decision_ref.clone()],
        )?;
        let evaluator_assurance = if bundle.integrity_ledger.required_for_side_effects {
            let integrity = &bundle.integrity_ledger;
            let key_dir = integrity.signing_key_dir.as_deref().ok_or_else(|| {
                ForgeError::Internal("ledger_integrity_error: signing_key_dir is required".into())
            })?;
            Some(
                sea_forge_ledger::LedgerManager::new(&root)?.create_pre_action_assurance(
                    key_dir,
                    &integrity.signing_key_id,
                    &options.entity,
                    &integrity
                        .witnesses
                        .iter()
                        .map(|witness| {
                            (
                                witness.witness_id.clone(),
                                witness.key_dir.clone(),
                                witness.key_id.clone(),
                            )
                        })
                        .collect::<Vec<_>>(),
                    integrity.min_witnesses,
                    &[&evaluator_committed],
                )?,
            )
        } else {
            None
        };
        let evaluator_grant = engine.grant(
            &evaluator_decision,
            &evaluator_committed,
            &action,
            evaluator_assurance.as_ref(),
        )?;
        let execution = sea_forge_runtime::execute(
            evaluator_grant,
            &ExecutionRequest {
                plan_item_id: pending.plan_item_id.clone(),
                operation,
                timeout_secs: options.timeout_secs,
                env: ["PATH", "HOME"]
                    .into_iter()
                    .filter_map(|key| std::env::var(key).ok().map(|value| (key.into(), value)))
                    .collect(),
                compensating_controls: evaluator_decision.compensating_controls.clone(),
            },
            &pending.run_id,
            &workspace,
            &artifacts,
        )?;
        stream.commit_typed_once(
            "artifact_transition_execution",
            &execution_key,
            vec![pending.case_id.clone(), pending.run_id.clone()],
            &execution,
            vec![evaluator_decision.decision_id.clone()],
        )?;
        settlement_authority_refs.push(evaluator_decision.decision_id);
        execution
    };
    let score =
        sea_forge_sandbox::environment::parse_evaluator_score(&score_from, &execution, &run_dir)?;
    let settlement = if let Some(entry) = entries.iter().find(|entry| {
        entry.record_kind == "settlement_event"
            && entry.idempotency_key.as_deref() == Some(settlement_key.as_str())
    }) {
        serde_json::from_value(entry.payload.clone())?
    } else {
        let settlement = sea_forge_settlement::settle(
            &SettlementClaim {
                run_id: pending.run_id.clone(),
                plan_item_id: pending.plan_item_id.clone(),
                criteria_ref: Some(criteria.criteria_id.clone()),
                criteria: criteria.criteria.clone(),
                execution: Some(execution.clone()),
                authority_verdicts: vec![Verdict::Allow],
                evaluator_scores: BTreeMap::from([(verifier_ref.clone(), score)]),
                batch: None,
            },
            &workspace,
            &run_dir,
        )?;
        stream.commit_typed_once(
            "settlement_event",
            &settlement_key,
            vec![pending.case_id.clone(), pending.run_id.clone()],
            &settlement,
            settlement_authority_refs,
        )?;
        settlement
    };
    write_json(&run_dir.join("settlement.json"), &settlement)?;
    if settlement.status != SettlementStatus::Accepted {
        sea_forge_artifact_ip::commit_transition_terminal(
            &root,
            &options.entity,
            &sea_forge_artifact_ip::ArtifactTransitionTerminal {
                version: sea_forge_artifact_ip::M8_RECORD_VERSION.into(),
                pending_key: pending.pending_key,
                case_id: pending.case_id,
                run_id: pending.run_id,
                status: sea_forge_artifact_ip::ArtifactTransitionTerminalStatus::Rejected,
                reason: sea_forge_artifact_ip::ArtifactTransitionTerminalReason::SettlementRejected,
                transition_token_ref: None,
                declaration_ref: None,
                terminal_at: Utc::now().to_rfc3339(),
            },
        )?;
        case.state = CaseState::Terminated;
        case.close_reason = Some("artifact_transition_settlement_rejected".into());
        case.closed_at = Some(Utc::now().to_rfc3339());
        write_json(&case_dir.join("case.json"), &case)?;
        return Ok(ResumeOutcome {
            case_id: options.case_id,
            state: "terminated",
            exit_code: 4,
        });
    }

    let transport = sea_forge_settlement::CommandSweSeedTransport::new(
        descriptor.command.clone(),
        descriptor.timeout_secs,
    )?;
    let authority =
        sea_forge_settlement::SweSeedSettlementAuthority::new(transport, &options.entity)
            .with_adapter_ref(&descriptor.authority_ref);
    let declarer_role = descriptor
        .permitted_declarer_roles
        .first()
        .cloned()
        .ok_or_else(|| ForgeError::Internal("strong authority role is missing".into()))?;
    let result = sea_forge_artifact_ip::resume_pending_transition(
        &root,
        &options.entity,
        &pending,
        &approval,
        &criteria,
        &settlement,
        &execution.started_at,
        &verifier_ref,
        &sea_forge_ledger::types::hash_canonical(&environment)?,
        &workspace,
        Declarer {
            actor_id: descriptor.declarer_actor_id,
            authority_ref: descriptor.authority_ref,
            role: declarer_role,
            standing_basis: descriptor.standing_basis,
        },
        &authority,
        transition_grant,
    );
    let resumed = match result {
        Err(ForgeError::Plan {
            class: "settlement_authority_unavailable",
            ..
        }) => {
            return Ok(ResumeOutcome {
                case_id: options.case_id,
                state: "awaiting_approval",
                exit_code: 5,
            });
        }
        other => other?,
    };
    sea_forge_artifact_ip::commit_transition_terminal(
        &root,
        &options.entity,
        &sea_forge_artifact_ip::ArtifactTransitionTerminal {
            version: sea_forge_artifact_ip::M8_RECORD_VERSION.into(),
            pending_key: pending.pending_key,
            case_id: pending.case_id,
            run_id: pending.run_id.clone(),
            status: sea_forge_artifact_ip::ArtifactTransitionTerminalStatus::Completed,
            reason: sea_forge_artifact_ip::ArtifactTransitionTerminalReason::TransitionCommitted,
            transition_token_ref: Some(resumed.token.transition_token_id.clone()),
            declaration_ref: Some(resumed.declaration.declaration_id.clone()),
            terminal_at: Utc::now().to_rfc3339(),
        },
    )?;
    case.state = CaseState::Completed;
    case.run_ids = vec![pending.run_id];
    case.closed_at = Some(Utc::now().to_rfc3339());
    write_json(&case_dir.join("case.json"), &case)?;
    println!("declaration_id={}", resumed.declaration.declaration_id);
    println!("transition_token_id={}", resumed.token.transition_token_id);
    Ok(ResumeOutcome {
        case_id: options.case_id,
        state: "completed",
        exit_code: 0,
    })
}

fn approval_binds_pending(
    approval: &ApprovalRequest,
    pending: &sea_forge_artifact_ip::PendingArtifactTransition,
) -> Result<(), ForgeError> {
    if approval.case_id != pending.case_id
        || approval.run_id != pending.run_id
        || approval.decision_id != pending.authority_decision_ref
        || approval.plan_item_id != pending.plan_item_id
        || approval.approval_id != pending.approval_ref
        || approval.criteria_ref.as_deref() != Some(pending.criteria_ref.as_str())
        || approval.criteria_sha256.as_deref() != Some(pending.criteria_sha256.as_str())
        || approval.criteria_record_hash.as_deref() != Some(pending.criteria_record_hash.as_str())
    {
        return Err(ForgeError::Input(format!(
            "approval resolution {} does not bind to pending transition {}",
            approval.approval_id, pending.pending_key
        )));
    }
    Ok(())
}

fn unique_entry<'a>(
    entries: &'a [sea_forge_ledger::types::LedgerEntry],
    kind: &str,
    id_field: &str,
    id: &str,
) -> Result<&'a sea_forge_ledger::types::LedgerEntry, ForgeError> {
    let matching = entries
        .iter()
        .filter(|entry| {
            entry.record_kind == kind
                && entry.payload.get(id_field).and_then(Value::as_str) == Some(id)
        })
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return Err(ForgeError::Internal(format!(
            "ledger_integrity_error: expected one {kind} record for {id}"
        )));
    }
    Ok(matching[0])
}
