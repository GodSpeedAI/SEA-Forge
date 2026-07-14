use sea_forge_core::{
    errors::ForgeError,
    types::{
        CasePlan, ItemKind, PlanItem, Sentry, SentryPredicate, SentryTrigger, TraceEvent, TraceKind,
    },
};
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path};

/// Check whether a sentry's `on` trigger has fired in the given events.
/// Returns the matching event if found.
fn trigger_fired<'a>(
    sentry: &sea_forge_core::types::Sentry,
    events: &'a [TraceEvent],
) -> Option<&'a TraceEvent> {
    events.iter().find(|event| {
        let source_match = sentry.on.source == "case"
            || event.plan_item_id.as_deref() == Some(sentry.on.source.as_str());
        let event_match = match &event.kind {
            TraceKind::MilestoneAchieved => sentry.on.event == "milestone_achieved",
            TraceKind::SettlementRecorded => sentry.on.event == "settlement_status",
            TraceKind::PlanMutated => sentry.on.event == "plan_mutated",
            TraceKind::CaseFileItemAdded => sentry.on.event == "case_file_item_added",
            kind => {
                let kind_str = format!("{:?}", kind).to_lowercase();
                sentry.on.event == kind_str
            }
        };
        source_match && event_match
    })
}

/// Check whether a sentry's `if` predicate holds given the current case state.
fn predicate_holds(
    predicate: &SentryPredicate,
    events: &[TraceEvent],
    workspace_files: &HashSet<String>,
) -> bool {
    match predicate {
        SentryPredicate::ArtifactExists { path } => workspace_files.contains(path),
        SentryPredicate::SettlementStatus { status } => {
            // Check if any settlement event has this status.
            events.iter().any(|event| {
                event.kind == TraceKind::SettlementRecorded
                    && event.payload.get("status").and_then(|v| v.as_str()) == Some(status)
            })
        }
    }
}

/// Check whether a single sentry is satisfied.
fn sentry_satisfied(
    sentry: &Sentry,
    events: &[TraceEvent],
    workspace_files: &HashSet<String>,
) -> bool {
    let Some(_trigger_event) = trigger_fired(sentry, events) else {
        return false;
    };
    match &sentry.if_predicate {
        None => true,
        Some(pred) => predicate_holds(pred, events, workspace_files),
    }
}

/// Check whether any of an item's entry criteria are satisfied (OR-of-sentries).
/// An empty entry_criteria list means "available at stage activation" (always true).
fn entry_criteria_satisfied(
    item: &PlanItem,
    events: &[TraceEvent],
    workspace_files: &HashSet<String>,
) -> bool {
    if item.entry_criteria.is_empty() {
        return true;
    }
    item.entry_criteria
        .iter()
        .any(|s| sentry_satisfied(s, events, workspace_files))
}

/// Evaluate sentries against the trace events and return the set of item IDs
/// whose entry criteria are currently satisfied.
///
/// This is a pure function of the trace events and workspace state.
/// It does not depend on any hidden engine state.
pub fn evaluate_sentries(
    items: &[PlanItem],
    events: &[TraceEvent],
    workspace_files: &HashSet<String>,
) -> Vec<String> {
    items
        .iter()
        .filter(|item| entry_criteria_satisfied(item, events, workspace_files))
        .map(|item| item.plan_item_id.clone())
        .collect()
}

/// Track per-item instance state for the case engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemState {
    pub item_id: String,
    pub status: ItemStatus,
    pub instances: u32,
    pub failed_instances: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ItemStatus {
    Available,
    Enabled,
    Active,
    Completed,
    Failed,
    Terminated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaseAction {
    Enable(String),
    Activate(String),
    AchieveMilestone(String),
    ParkHumanTask(String),
    CompleteCase,
    TerminateCase { blocking_item: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseProjection {
    pub items: Vec<ItemState>,
    pub activation_sequence: Vec<String>,
    pub completed: bool,
    pub terminated_by: Option<String>,
}

impl ItemState {
    pub fn new(item_id: String) -> Self {
        Self {
            item_id,
            status: ItemStatus::Available,
            instances: 0,
            failed_instances: 0,
        }
    }
}

/// Static satisfiability check on the sentry "on" graph.
/// Detects cycles in entry-criteria dependencies.
pub fn check_satisfiability(items: &[PlanItem]) -> Result<(), ForgeError> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for item in items {
        graph.entry(item.plan_item_id.clone()).or_default();
        for sentry in &item.entry_criteria {
            if sentry.on.source != "case" {
                graph
                    .entry(sentry.on.source.clone())
                    .or_default()
                    .push(item.plan_item_id.clone());
            }
        }
    }
    // DFS cycle detection
    let mut visited = HashSet::new();
    let mut stack = HashSet::new();
    for node in graph.keys() {
        if has_cycle(node, &graph, &mut visited, &mut stack) {
            return Err(ForgeError::Plan {
                class: "plan_cycle_error",
                message: "sentry dependency cycle detected".into(),
            });
        }
    }
    Ok(())
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
}

fn valid_relative_path(value: &str) -> bool {
    let path = Path::new(value);
    !value.is_empty()
        && !path.is_absolute()
        && !path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
}

/// Validate and normalize a plan proposal before any case state is created.
pub fn validate_proposal(plan: &mut CasePlan) -> Result<(), ForgeError> {
    if plan.items.is_empty() {
        return Err(ForgeError::Plan {
            class: "plan_schema_error",
            message: "plan must contain at least one item".into(),
        });
    }
    let ids = plan
        .items
        .iter()
        .map(|item| item.plan_item_id.clone())
        .collect::<HashSet<_>>();
    if ids.len() != plan.items.len() || ids.iter().any(|id| !valid_id(id)) {
        return Err(ForgeError::Plan {
            class: "plan_schema_error",
            message: "plan item IDs must be unique and canonical".into(),
        });
    }
    let stages = plan
        .items
        .iter()
        .filter(|item| item.item_kind == ItemKind::Stage)
        .map(|item| item.plan_item_id.clone())
        .collect::<HashSet<_>>();
    for item in &mut plan.items {
        if item.max_instances == 0
            || (item.markers.repetition && item.max_instances <= 1)
            || (!item.markers.repetition && item.max_instances != 1)
        {
            return Err(ForgeError::Plan {
                class: "plan_schema_error",
                message: format!("invalid repetition settings for {}", item.plan_item_id),
            });
        }
        if item.item_kind == ItemKind::SandboxedTask && item.sandbox_class.is_none() {
            item.sandbox_class = Some("local".into());
        }
        if item.item_kind != ItemKind::SandboxedTask && !item.operations.is_empty() {
            return Err(ForgeError::Plan {
                class: "plan_schema_error",
                message: format!(
                    "non-sandboxed item {} cannot have operations",
                    item.plan_item_id
                ),
            });
        }
        if item.item_kind == ItemKind::Milestone && item.markers.repetition {
            return Err(ForgeError::Plan {
                class: "plan_schema_error",
                message: "milestones cannot repeat".into(),
            });
        }
        if item
            .parent_stage
            .as_ref()
            .is_some_and(|parent| !stages.contains(parent))
        {
            return Err(ForgeError::Plan {
                class: "plan_schema_error",
                message: format!("unknown parent stage for {}", item.plan_item_id),
            });
        }
        for dependency in std::mem::take(&mut item.depends_on) {
            if !ids.contains(&dependency) {
                return Err(ForgeError::Plan {
                    class: "plan_schema_error",
                    message: format!("unknown dependency {dependency}"),
                });
            }
            item.entry_criteria.push(Sentry {
                on: SentryTrigger {
                    source: dependency,
                    event: "milestone_achieved".into(),
                },
                if_predicate: None,
            });
        }
        for sentry in item.entry_criteria.iter().chain(&item.exit_criteria) {
            if sentry.on.source != "case" && !ids.contains(&sentry.on.source) {
                return Err(ForgeError::Plan {
                    class: "plan_schema_error",
                    message: format!("unknown sentry source {}", sentry.on.source),
                });
            }
        }
        for operation in &item.operations {
            match operation {
                sea_forge_core::types::Operation::WriteFile { path, .. }
                | sea_forge_core::types::Operation::ExecuteCommand { cwd: path, .. } => {
                    if !valid_relative_path(path) {
                        return Err(ForgeError::Plan {
                            class: "plan_schema_error",
                            message: format!("unsafe operation path {path}"),
                        });
                    }
                }
            }
            if let sea_forge_core::types::Operation::ExecuteCommand { argv, .. } = operation {
                if argv.is_empty() || argv.iter().any(|arg| arg.contains('\0')) {
                    return Err(ForgeError::Plan {
                        class: "plan_schema_error",
                        message: "execute_command requires safe non-empty argv".into(),
                    });
                }
            }
        }
    }
    check_satisfiability(&plan.items)
}

fn has_cycle(
    node: &str,
    graph: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    stack: &mut HashSet<String>,
) -> bool {
    if stack.contains(node) {
        return true;
    }
    if visited.contains(node) {
        return false;
    }
    visited.insert(node.into());
    stack.insert(node.into());
    if let Some(neighbors) = graph.get(node) {
        for neighbor in neighbors {
            if has_cycle(neighbor, graph, visited, stack) {
                return true;
            }
        }
    }
    stack.remove(node);
    false
}

/// Check whether a case can auto-complete.
/// Auto-complete: all required items completed, no active/enabled items.
pub fn can_auto_complete(item_states: &[ItemState], items: &[PlanItem]) -> bool {
    let item_map: HashMap<&str, &PlanItem> =
        items.iter().map(|i| (i.plan_item_id.as_str(), i)).collect();
    for state in item_states {
        if let Some(item) = item_map.get(state.item_id.as_str()) {
            if item.markers.required && state.status != ItemStatus::Completed {
                return false;
            }
        }
    }
    // No active or enabled items
    !item_states
        .iter()
        .any(|s| s.status == ItemStatus::Active || s.status == ItemStatus::Enabled)
}

/// Check whether a required item has failed, making auto-complete impossible.
/// Returns the item ID if so.
pub fn required_item_failed(item_states: &[ItemState], items: &[PlanItem]) -> Option<String> {
    let item_map: HashMap<&str, &PlanItem> =
        items.iter().map(|i| (i.plan_item_id.as_str(), i)).collect();
    for state in item_states {
        if let Some(item) = item_map.get(state.item_id.as_str()) {
            if item.markers.required && state.status == ItemStatus::Failed {
                return Some(state.item_id.clone());
            }
        }
    }
    None
}

fn derived_workspace_files(events: &[TraceEvent]) -> HashSet<String> {
    events
        .iter()
        .filter(|event| event.kind == TraceKind::ArtifactCaptured)
        .filter_map(|event| event.payload.get("artifact")?.as_str().map(str::to_owned))
        .collect()
}

/// Rebuild the complete case projection from ordered case events.
pub fn replay_case(items: &[PlanItem], events: &[TraceEvent]) -> CaseProjection {
    let mut states = items
        .iter()
        .map(|item| ItemState::new(item.plan_item_id.clone()))
        .collect::<Vec<_>>();
    let item_map = items
        .iter()
        .map(|item| (item.plan_item_id.as_str(), item))
        .collect::<HashMap<_, _>>();
    let mut activation_sequence = Vec::new();
    let mut completed = false;
    let mut terminated_by = None;
    for event in events {
        let Some(item_id) = event.plan_item_id.as_deref() else {
            match event.kind {
                TraceKind::CaseClosed => completed = true,
                TraceKind::CaseTerminated => {
                    terminated_by = event
                        .payload
                        .get("blocking_item")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                }
                _ => {}
            }
            continue;
        };
        let Some(state) = states.iter_mut().find(|state| state.item_id == item_id) else {
            continue;
        };
        match event.kind {
            TraceKind::ItemEnabled => state.status = ItemStatus::Enabled,
            TraceKind::ItemActivated => {
                state.status = ItemStatus::Active;
                state.instances += 1;
                activation_sequence.push(item_id.to_owned());
            }
            TraceKind::ItemCompleted | TraceKind::HumanTaskCompleted => {
                state.status = ItemStatus::Completed
            }
            TraceKind::ItemFailed => {
                state.failed_instances += 1;
                let repeats = item_map.get(item_id).is_some_and(|item| {
                    item.markers.repetition && state.instances < item.max_instances
                });
                state.status = if repeats {
                    ItemStatus::Available
                } else {
                    ItemStatus::Failed
                };
            }
            TraceKind::ItemTerminated => state.status = ItemStatus::Terminated,
            TraceKind::MilestoneAchieved => state.status = ItemStatus::Completed,
            _ => {}
        }
    }
    CaseProjection {
        items: states,
        activation_sequence,
        completed,
        terminated_by,
    }
}

/// Determine the next deterministic case actions from persisted events alone.
pub fn next_case_actions(items: &[PlanItem], events: &[TraceEvent]) -> Vec<CaseAction> {
    let projection = replay_case(items, events);
    if projection.completed || projection.terminated_by.is_some() {
        return vec![];
    }
    if let Some(blocking_item) = required_item_failed(&projection.items, items) {
        return vec![CaseAction::TerminateCase { blocking_item }];
    }
    if can_auto_complete(&projection.items, items) {
        return vec![CaseAction::CompleteCase];
    }
    let workspace_files = derived_workspace_files(events);
    let mut actions = Vec::new();
    for item in items {
        let Some(state) = projection
            .items
            .iter()
            .find(|state| state.item_id == item.plan_item_id)
        else {
            continue;
        };
        if state.status != ItemStatus::Available
            || state.instances >= item.max_instances
            || !entry_criteria_satisfied(item, events, &workspace_files)
        {
            continue;
        }
        actions.push(if item.markers.manual_activation {
            CaseAction::Enable(item.plan_item_id.clone())
        } else {
            match item.item_kind {
                ItemKind::Milestone => CaseAction::AchieveMilestone(item.plan_item_id.clone()),
                ItemKind::HumanTask => CaseAction::ParkHumanTask(item.plan_item_id.clone()),
                _ => CaseAction::Activate(item.plan_item_id.clone()),
            }
        });
    }
    actions
}

/// Replay sentries over a sequence of trace events and reproduce the
/// activation sequence. Returns a list of (event_index, activated_item_ids).
pub fn replay_activations(items: &[PlanItem], events: &[TraceEvent]) -> Vec<(usize, Vec<String>)> {
    let mut result = Vec::new();
    let mut workspace_files: HashSet<String> = HashSet::new();
    for (i, event) in events.iter().enumerate() {
        // Update workspace state from artifact_captured events
        if event.kind == TraceKind::ArtifactCaptured {
            if let Some(name) = event.payload.get("artifact").and_then(|v| v.as_str()) {
                workspace_files.insert(name.into());
            }
        }
        let activated = evaluate_sentries(items, &events[..=i], &workspace_files);
        result.push((i, activated));
    }
    result
}
