use sea_forge_core::{
    errors::ForgeError,
    types::{PlanItem, Sentry, SentryPredicate, TraceEvent, TraceKind},
};
use std::collections::{HashMap, HashSet};

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
#[derive(Clone, Debug)]
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
            return Err(ForgeError::Internal(
                "plan_cycle_error: sentry dependency cycle detected".into(),
            ));
        }
    }
    Ok(())
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
