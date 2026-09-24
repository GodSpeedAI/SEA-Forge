//! In-process case advancement (T04 step 2, decision D-3).
//!
//! One engine drives both the operator verbs (`case.advance`, `item.execute`)
//! and any future supervisor: it re-derives the next actions from persisted
//! case events alone (the same `next_case_actions` the dispatcher uses),
//! appends the case trace events the engine semantics prescribe, and runs
//! SandboxedTask episodes through a host-supplied executor callback so the
//! kernel crate stays synchronous and sandbox-agnostic.
//!
//! Enabled items are activated here on purpose. `next_case_actions` returns
//! `Enable` for a manual-activation item and then never returns `Activate`
//! for it — enabling is the sentry's half, activating is the operator's
//! (D-3). This loop is the "both, behind one engine call" half.

use super::{load_case_plan, paths, write_json, CaseEventNotifier};
use chrono::Utc;
use sea_forge_core::{
    errors::ForgeError,
    ids,
    types::{
        CaseState, ItemKind, PlanItem, SettlementEvent, SettlementStatus, TraceEvent, TraceKind,
    },
};
use sea_forge_ledger::LedgerStream;
use sea_forge_planner::case_engine::{next_case_actions, replay_case, CaseAction};
use serde_json::json;
use std::path::Path;

/// Which items one advance pass may activate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdvanceScope {
    /// `case.advance`: every item the engine is ready to run.
    All,
    /// `item.execute`: exactly this item, which must itself be ready.
    Item(String),
    /// The opt-in case-advance supervisor (server-internal, D-3): a strict
    /// subset of `All` for unattended passes. Only `SandboxedTask` items may
    /// be activated/run, `ParkHumanTask` actions are filtered out before the
    /// idle check — the supervisor never activates or completes human work,
    /// never writes `approvals.jsonl`, and a case whose only ready work is
    /// human therefore reads `idle` with zero writes. Everything else
    /// (`Enable`, sentries, milestones, case close/terminate) is engine
    /// semantics re-derived from events and stays identical to the verb, so
    /// event publishing and episode execution remain literally one code path.
    Supervisor,
}

/// One executed episode, as recorded by the engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpisodeOutcome {
    pub item_id: String,
    pub run_id: String,
    pub status: SettlementStatus,
}

/// The terminal standing of one advance pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdvanceOutcome {
    /// One of: `completed`, `terminated`, `awaiting_approval`,
    /// `parked_human_task`, `idle` (no ready actions), `blocked` (ready
    /// actions exist but none are advanceable sandboxed tasks), or
    /// `episode_budget_reached`.
    pub state: &'static str,
    pub episodes: Vec<EpisodeOutcome>,
}

/// Upper bound on episodes a single advance pass will run. A case that needs
/// more passes needs more calls — a bounded verb must not become an
/// unbounded dispatch loop.
pub const MAX_EPISODES_PER_ADVANCE: usize = 32;

/// Synchronous host hook that executes one sandboxed episode for `item` under
/// `run_id` and returns its settlement. The server passes the same
/// governed-execution path `case_dispatch` uses (authority first, then
/// runtime, then settlement); the engine commits the settlement event and the
/// completion trace events itself so their ordering cannot drift from the
/// dispatcher's.
pub type EpisodeExecutor<'a> =
    &'a mut dyn FnMut(&PlanItem, &str) -> Result<SettlementEvent, ForgeError>;

/// Append a case trace event through the runner's own recorder (the same
/// append the dispatch loop uses, ordinals included) and notify the host sink.
fn append_notify(
    case_events: &Path,
    stream: &LedgerStream,
    events: &mut Vec<TraceEvent>,
    kind: TraceKind,
    item_id: Option<&str>,
    payload: serde_json::Value,
    notify: CaseEventNotifier<'_>,
) -> Result<(), ForgeError> {
    crate::CaseRunner::append_event(case_events, stream, events, kind, item_id, payload)?;
    if let Some(event) = events.last() {
        notify(event);
    }
    Ok(())
}

/// Advance a committed case in-process. All state is re-derived from
/// `case-events.jsonl`; nothing in this function depends on live handles.
pub fn advance_case(
    root: &Path,
    actor: &str,
    case_id: &str,
    scope: AdvanceScope,
    notify: CaseEventNotifier<'_>,
    run_episode: EpisodeExecutor<'_>,
) -> Result<AdvanceOutcome, ForgeError> {
    if !sea_forge_core::path::valid_id_segment(case_id, 128) {
        return Err(ForgeError::Input(format!("unsafe case id: {case_id}")));
    }
    let (mut case, plan, mut events) = load_case_plan(root, case_id)?;
    if let AdvanceScope::Item(target) = &scope {
        if !plan.items.iter().any(|item| item.plan_item_id == *target) {
            return Err(ForgeError::Input(format!(
                "item {target} is not in the plan for case {case_id}"
            )));
        }
    }
    let (_, _, case_events) = paths(root, case_id);
    let stream = LedgerStream::open(root, format!("case-{case_id}"), actor)?;
    let mut episodes: Vec<EpisodeOutcome> = Vec::new();

    let state = loop {
        // F-03: a required escalated item parks the case pending approval;
        // the approval resolution flow re-drives it.
        if case.state == CaseState::AwaitingApproval {
            break "awaiting_approval";
        }
        let mut actions = next_case_actions(&plan.items, &events);
        // Supervisor scope (T04C): an unattended pass must never activate
        // human/agent work, so `ParkHumanTask` — which appends
        // `ItemActivated {human_task: true}` — and `Activate` of any
        // non-SandboxedTask target are dropped before the idle check (the
        // `Activate` arm never writes on its own; it only feeds the
        // activation set below, which is filtered by kind anyway). A case
        // whose only ready work is non-sandboxed reads `idle` for the
        // supervisor: zero writes, and the verb drives that work when an
        // operator does. The verb's scope is untouched.
        if matches!(scope, AdvanceScope::Supervisor) {
            actions.retain(|action| match action {
                CaseAction::ParkHumanTask(_) => false,
                CaseAction::Activate(item_id) => !plan.items.iter().any(|item| {
                    item.plan_item_id == *item_id && item.item_kind != ItemKind::SandboxedTask
                }),
                _ => true,
            });
        }
        // The operator activation set: items the engine would activate now
        // (Available, criteria satisfied) plus items a sentry already enabled
        // — the manual-activation items `next_case_actions` never returns
        // `Activate` for (D-3: enabling is the sentry's half, activating is
        // the operator's). Computed *before* the idle check so a case whose
        // only remaining work is enabled items still advances. Deterministic
        // plan order.
        let projection = replay_case(&plan.items, &events);
        let mut activations: Vec<String> = plan
            .items
            .iter()
            .filter(|item| {
                actions.iter().any(|action| match action {
                    CaseAction::Activate(id) => id == &item.plan_item_id,
                    _ => false,
                }) || projection.items.iter().any(|state| {
                    state.item_id == item.plan_item_id
                        && state.status == sea_forge_planner::case_engine::ItemStatus::Enabled
                        && state.instances < item.max_instances
                })
            })
            .map(|item| item.plan_item_id.clone())
            .collect();
        if let AdvanceScope::Item(target) = &scope {
            activations.retain(|item| item == target);
        }
        // Supervisor scope (T04C): only SandboxedTask items may be
        // activated/run by an unattended pass. Filtering here (not only in
        // the episode loop below) keeps the idle check honest: a case whose
        // Enabled items are all human/agent/milestone work reads `idle`
        // instead of looping to `blocked`, so the supervisor's poll appends
        // nothing at all. The verb's scopes are untouched.
        if matches!(scope, AdvanceScope::Supervisor) {
            activations.retain(|item_id| {
                plan.items.iter().any(|item| {
                    item.plan_item_id == *item_id && item.item_kind == ItemKind::SandboxedTask
                })
            });
        }
        if actions.is_empty() && activations.is_empty() {
            break "idle";
        }
        let mut progressed = false;
        let mut terminal: Option<&'static str> = None;
        let mut parked_human = false;
        for action in &actions {
            match action {
                CaseAction::Enable(item) => {
                    append_notify(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::ItemEnabled,
                        Some(item),
                        json!({}),
                        notify,
                    )?;
                    progressed = true;
                }
                CaseAction::AchieveMilestone(item) => {
                    append_notify(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::MilestoneAchieved,
                        Some(item),
                        json!({}),
                        notify,
                    )?;
                    progressed = true;
                }
                CaseAction::CompleteCase => {
                    case.state = CaseState::Completed;
                    case.closed_at = Some(Utc::now().to_rfc3339());
                    append_notify(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::CaseClosed,
                        None,
                        json!({}),
                        notify,
                    )?;
                    terminal = Some("completed");
                }
                CaseAction::TerminateCase { blocking_item } => {
                    case.state = CaseState::Terminated;
                    case.close_reason = Some(format!("required_item_failed:{blocking_item}"));
                    case.closed_at = Some(Utc::now().to_rfc3339());
                    append_notify(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::CaseTerminated,
                        None,
                        json!({"blocking_item": blocking_item}),
                        notify,
                    )?;
                    terminal = Some("terminated");
                }
                CaseAction::ParkHumanTask(item_id) => {
                    append_notify(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::ItemActivated,
                        Some(item_id),
                        json!({"human_task": true}),
                        notify,
                    )?;
                    parked_human = true;
                    progressed = true;
                }
                // Activate is handled with the operator activation set below
                // so Enabled items join the same batch deterministically.
                CaseAction::Activate(_) => {}
            }
        }
        for item_id in &activations {
            if terminal.is_some() || parked_human {
                break;
            }
            let Some(item) = plan.items.iter().find(|item| &item.plan_item_id == item_id) else {
                continue;
            };
            // This engine executes sandboxed tasks only. Agent tasks advance
            // through the delegation path; anything else is not executable.
            if item.item_kind != ItemKind::SandboxedTask {
                continue;
            }
            if episodes.len() >= MAX_EPISODES_PER_ADVANCE {
                terminal = Some("episode_budget_reached");
                break;
            }
            let instance = projection
                .items
                .iter()
                .find(|state| &state.item_id == item_id)
                .map_or(1, |state| state.instances + 1);
            let episode_run_id = ids::run_id()?;
            append_notify(
                &case_events,
                &stream,
                &mut events,
                TraceKind::ItemActivated,
                Some(item_id),
                json!({
                    "instance": instance,
                    "run_id": episode_run_id,
                    "episode_kind": "sandboxed_task",
                }),
                notify,
            )?;
            // The dispatcher's stance: an episode executor error is settled,
            // not propagated, so an `ItemActivated` can never dangle without a
            // settlement. The basis names the error class durably.
            let settlement = run_episode(item, &episode_run_id).unwrap_or_else(|error| {
                let class = error.class();
                SettlementEvent {
                    version: sea_forge_core::RECORD_VERSION.into(),
                    settlement_id: ids::random_id("set")
                        .unwrap_or_else(|_| "set_dispatch_error".into()),
                    run_id: episode_run_id.clone(),
                    status: SettlementStatus::Rejected,
                    basis: vec!["episode_dispatch_error".into(), class.into()],
                    review_required: false,
                    settled_at: Utc::now().to_rfc3339(),
                    criteria_ref: item.settlement_criteria_ref.clone(),
                }
            });
            stream.commit_typed(
                "settlement_event",
                vec![
                    case_id.into(),
                    episode_run_id.clone(),
                    settlement.settlement_id.clone(),
                ],
                &settlement,
                vec![],
            )?;
            append_notify(
                &case_events,
                &stream,
                &mut events,
                TraceKind::SettlementRecorded,
                Some(item_id),
                json!({
                    "instance": instance,
                    "run_id": episode_run_id,
                    "status": settlement.status,
                }),
                notify,
            )?;
            episodes.push(EpisodeOutcome {
                item_id: item_id.clone(),
                run_id: episode_run_id.clone(),
                status: settlement.status.clone(),
            });
            // F-03: a required escalated item parks the case; the completion
            // trace must still be recorded (as `ItemTerminated`) before the
            // park, exactly as the dispatcher's completion recorder does.
            let before_completion = events.len();
            crate::CaseRunner::apply_episode_completion(
                &mut case,
                &episode_run_id,
                item_id,
                instance,
                &settlement,
                &case_events,
                &stream,
                &mut events,
            )?;
            for event in &events[before_completion..] {
                notify(event);
            }
            if settlement.status == SettlementStatus::Escalated && item.markers.required {
                case.state = CaseState::AwaitingApproval;
                case.close_reason = Some(format!("awaiting_approval:{item_id}"));
            }
            progressed = true;
        }
        write_json(&root.join("cases").join(case_id).join("case.json"), &case)?;
        if let Some(state) = terminal {
            break state;
        }
        if parked_human {
            break "parked_human_task";
        }
        if !progressed {
            break "blocked";
        }
    };

    if let AdvanceScope::Item(target) = &scope {
        if episodes.is_empty() {
            return Err(ForgeError::Input(format!(
                "item {target} is not ready for execution in case {case_id}"
            )));
        }
    }
    Ok(AdvanceOutcome { state, episodes })
}
