use chrono::Utc;
use sea_forge_core::{
    errors::ForgeError,
    ids,
    types::{
        ApprovalRequest, ApprovalStatus, AuthorityAction, CaseState, ItemKind, ItemMarkers,
        ManagerAction, ManagerIteration, ManagerJudgment, Operation, PlanItem, SettlementCriteria,
        TraceKind,
    },
    RECORD_VERSION,
};
use sea_forge_ledger::LedgerStream;
use sea_forge_planner::case_engine::{next_case_actions, CaseAction};
use sea_forge_thoth::manager::{judge, ManagerView};
use serde_json::json;
use std::path::Path;

use super::case::{load_case_plan, propose_item, save_case};

/// Fixed built-in proposal-catalog entry (§7.6): the manager loop never
/// invents free-form task content, only this pinned description. The hash
/// is `sha256(BUILTIN_PROPOSAL_SOURCE_REF)`, precomputed — adding a real
/// catalog file loader is unwarranted until more than one entry exists.
const BUILTIN_PROPOSAL_SOURCE_REF: &str = "builtin:manager-agent-task-proposal@0.1.0";
const BUILTIN_PROPOSAL_SOURCE_SHA256: &str =
    "c990a16ddb9bd1e2e0ba8f2f609bda601f3e76a9e35d33c6418c2bb84c6be376";

pub struct ManagerIterateOptions<'a> {
    pub root: &'a Path,
    pub policy: &'a Path,
    pub actor: &'a str,
    pub case_id: &'a str,
    pub max_iterations: u32,
}

/// Run one auditable step of the Thoth manager loop (§7.6, §16.2). Never a
/// background daemon — always an invoked, granted operation.
pub fn iterate(opts: ManagerIterateOptions) -> Result<u8, ForgeError> {
    let ManagerIterateOptions {
        root,
        policy,
        actor,
        case_id,
        max_iterations,
    } = opts;

    super::mediated::authorize_read(
        root,
        policy,
        actor,
        &AuthorityAction::Reserved {
            resource_type: "manager_iteration".into(),
            resource_id: case_id.into(),
            parameters: json!({}),
        },
    )?;

    let (mut case, plan, events) = load_case_plan(root, case_id)?;
    let stream = LedgerStream::open(root, format!("case-{case_id}"), actor)?;
    let entries = stream.read_entries()?;

    let prior_iterations: Vec<&sea_forge_ledger::LedgerEntry> = entries
        .iter()
        .filter(|e| e.record_kind == "manager_iteration")
        .collect();
    let iteration = prior_iterations.len() as u32 + 1;

    if iteration > max_iterations {
        return escalate(
            root,
            case_id,
            &mut case,
            &stream,
            &entries,
            iteration,
            "manager loop iteration cap reached",
        );
    }

    let actions = next_case_actions(&plan.items, &events);
    let blocked_reason = actions.iter().find_map(|a| match a {
        CaseAction::TerminateCase { blocking_item } => {
            Some(format!("terminal dependency block on {blocking_item}"))
        }
        CaseAction::ParkHumanTask(id) => Some(format!("unresolved human task {id}")),
        _ => None,
    });
    // "active or enabled work" (§9.5) includes both fresh transitions this
    // tick (next_case_actions) and items that already transitioned to
    // Enabled/Active on a prior tick and are still awaiting their next step
    // (e.g. manual_activation items awaiting an explicit `task complete`).
    let mut ready_ids: Vec<String> = actions
        .iter()
        .filter_map(|a| match a {
            CaseAction::Enable(id)
            | CaseAction::Activate(id)
            | CaseAction::AchieveMilestone(id) => Some(id.clone()),
            _ => None,
        })
        .collect();
    for state in &sea_forge_planner::case_engine::replay_case(&plan.items, &events).items {
        if matches!(
            state.status,
            sea_forge_planner::case_engine::ItemStatus::Enabled
                | sea_forge_planner::case_engine::ItemStatus::Active
        ) && !ready_ids.contains(&state.item_id)
        {
            ready_ids.push(state.item_id.clone());
        }
    }
    let case_completed = case.state == CaseState::Completed
        || actions
            .iter()
            .any(|a| matches!(a, CaseAction::CompleteCase));
    let blocked = case.state == CaseState::AwaitingApproval || blocked_reason.is_some();

    let settlement_events_observed = events
        .iter()
        .filter(|e| e.kind == TraceKind::SettlementRecorded)
        .count() as u32;
    let prior_settlement_count = prior_iterations
        .last()
        .and_then(|e| e.payload.get("settlement_events_observed"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let new_settlement_progress = settlement_events_observed > prior_settlement_count;

    let view = ManagerView {
        case_completed,
        blocked,
        has_ready_work: !ready_ids.is_empty(),
        new_settlement_progress,
    };
    let judgment = judge(&view);

    let rationale_claim_refs: Vec<String> = if case_completed {
        vec![format!("case:{case_id}")]
    } else if let Some(reason) = &blocked_reason {
        vec![format!("case:{case_id}:{reason}")]
    } else if !ready_ids.is_empty() {
        ready_ids
            .iter()
            .map(|id| format!("plan_item:{id}"))
            .collect()
    } else if new_settlement_progress {
        vec![format!("case:{case_id}:settlement_progress")]
    } else {
        vec![format!("case:{case_id}:no_ready_work")]
    };

    let mut iteration_record = ManagerIteration {
        version: RECORD_VERSION.into(),
        case_id: case_id.into(),
        iteration,
        snapshot_case_version: case.version.clone(),
        snapshot_ledger_head: entries.len() as u64,
        settlement_events_observed,
        proposal_source_ref: BUILTIN_PROPOSAL_SOURCE_REF.into(),
        proposal_source_sha256: BUILTIN_PROPOSAL_SOURCE_SHA256.into(),
        judgment,
        rationale_claim_refs,
        action: ManagerAction::Noop,
        proposed_item_ref: None,
        granted: None,
        recorded_at: Utc::now().to_rfc3339(),
    };

    match judgment {
        ManagerJudgment::Satisfied | ManagerJudgment::Progressing => {
            commit_iteration(&stream, case_id, &iteration_record)?;
            Ok(0)
        }
        ManagerJudgment::Blocked => {
            iteration_record.action = ManagerAction::Escalate;
            commit_iteration(&stream, case_id, &iteration_record)?;
            escalate(
                root,
                case_id,
                &mut case,
                &stream,
                &entries,
                iteration,
                blocked_reason.as_deref().unwrap_or("blocked"),
            )
        }
        ManagerJudgment::Stalled => {
            let proposal = synthesized_proposal(case_id, iteration, actor);
            iteration_record.action = ManagerAction::ProposeItem;
            iteration_record.proposed_item_ref = Some(proposal.plan_item_id.clone());
            let grant_result = propose_item(root, policy, actor, case_id, proposal);
            iteration_record.granted = Some(grant_result.is_ok());
            commit_iteration(&stream, case_id, &iteration_record)?;
            // A denied proposal is an outcome, not an error (T15.2, §7.6):
            // the iteration is still recorded and the loop continues.
            Ok(0)
        }
    }
}

fn commit_iteration(
    stream: &LedgerStream,
    case_id: &str,
    record: &ManagerIteration,
) -> Result<(), ForgeError> {
    stream.commit_typed("manager_iteration", vec![case_id.into()], record, vec![])?;
    Ok(())
}

/// Park the case and escalate through the existing approval mechanism
/// (§9.3, §16.2) — never a new escalation channel, never a background
/// daemon retrying on its own.
fn escalate(
    root: &Path,
    case_id: &str,
    case: &mut sea_forge_core::types::Case,
    stream: &LedgerStream,
    entries: &[sea_forge_ledger::LedgerEntry],
    iteration: u32,
    reason: &str,
) -> Result<u8, ForgeError> {
    let n_existing_approvals = entries
        .iter()
        .filter(|e| e.record_kind == "approval_request")
        .count();
    let now = Utc::now().to_rfc3339();
    let approval = ApprovalRequest {
        version: RECORD_VERSION.into(),
        approval_id: ids::seq_id("apr", 4, n_existing_approvals + 1),
        run_id: case_id.into(),
        case_id: case_id.into(),
        decision_id: ids::seq_id("mgrdec", 4, iteration as usize),
        plan_item_id: format!("manager:{case_id}"),
        criteria_ref: None,
        criteria_sha256: None,
        criteria_record_hash: None,
        job_contract_ref: None,
        requested_at: now.clone(),
        expires_at: now,
        status: ApprovalStatus::Pending,
        resolved_by: None,
        resolved_at: None,
        note: Some(format!(
            "thoth manager loop escalation (iteration {iteration}): {reason}"
        )),
    };
    stream.commit_typed(
        "approval_request",
        vec![case_id.into(), approval.approval_id.clone()],
        &approval,
        vec![],
    )?;
    crate::approvals::append(root, &approval)?;
    case.state = CaseState::AwaitingApproval;
    save_case(root, case_id, case)?;
    Ok(5)
}

fn synthesized_proposal(case_id: &str, iteration: u32, actor: &str) -> PlanItem {
    PlanItem {
        plan_item_id: format!("mgr_{case_id}_{iteration}"),
        name: "thoth_proposed_agent_task".into(),
        operations: vec![Operation::AgentTask {
            endpoint_ref: "agent:default".into(),
            instruction: format!("Advance stalled case {case_id} (manager iteration {iteration})."),
            max_turns: 1,
            token_budget: None,
            response_schema: None,
            transcript_retention: None,
        }],
        entry_criteria: vec![],
        entry_criteria_mode: Default::default(),
        exit_criteria: vec![],
        settlement_criteria: SettlementCriteria::default(),
        settlement_criteria_ref: None,
        item_kind: ItemKind::AgentTask,
        sandbox_class: None,
        parent_stage: None,
        markers: ItemMarkers::default(),
        max_instances: 1,
        depends_on: vec![],
        environment: None,
        proposed_by: Some(actor.into()),
    }
}
