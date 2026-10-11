use sea_forge_core::{
    errors::ForgeError,
    types::{
        CasePlan, ItemKind, PlanItem, Sentry, SentryPredicate, SentryTrigger, TraceEvent, TraceKind,
    },
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

/// Check whether a sentry's `if` predicate holds for the trigger source.
/// `source` is the sentry's named source item; `"case"` is a wildcard.
fn predicate_holds(
    predicate: &SentryPredicate,
    source: &str,
    events: &[TraceEvent],
    workspace_files: &HashSet<String>,
) -> bool {
    match predicate {
        SentryPredicate::ArtifactExists { path } => workspace_files.contains(path),
        // Source-bound: evaluate only settlement events from the named source,
        // not any matching event in the case (spec-adlc-thoth §7.5/§10.4).
        SentryPredicate::SettlementStatus { status } => events.iter().any(|event| {
            if event.kind != TraceKind::SettlementRecorded {
                return false;
            }
            let source_ok = source == "case" || event.plan_item_id.as_deref() == Some(source);
            source_ok
                && event.payload.get("status").and_then(|v| v.as_str()) == Some(status.as_str())
        }),
    }
}

/// Check whether a single sentry is satisfied.
fn sentry_satisfied(
    sentry: &Sentry,
    events: &[TraceEvent],
    workspace_files: &HashSet<String>,
) -> bool {
    if trigger_fired(sentry, events).is_none() {
        return false;
    };
    match &sentry.if_predicate {
        None => true,
        Some(pred) => predicate_holds(pred, &sentry.on.source, events, workspace_files),
    }
}

/// Check whether an item's entry criteria are satisfied. `Any` (default)
/// is OR-of-sentries; `All` requires every listed sentry satisfied — used
/// for concurrent-branch success rollups (§7.5 E16a). An empty
/// entry_criteria list means "available at stage activation" (always true)
/// regardless of mode.
fn entry_criteria_satisfied(
    item: &PlanItem,
    events: &[TraceEvent],
    workspace_files: &HashSet<String>,
) -> bool {
    if item.entry_criteria.is_empty() {
        return true;
    }
    match item.entry_criteria_mode {
        sea_forge_core::types::EntryCriteriaMode::Any => item
            .entry_criteria
            .iter()
            .any(|s| sentry_satisfied(s, events, workspace_files)),
        sea_forge_core::types::EntryCriteriaMode::All => item
            .entry_criteria
            .iter()
            .all(|s| sentry_satisfied(s, events, workspace_files)),
    }
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
    // The shared canonical-path primitive: rejects `..`, absolute paths, and
    // the ambiguous spellings `//`/`.` segments/trailing `/` (bare `.` is
    // accepted as the workspace-root marker used by `ExecuteCommand.cwd`).
    sea_forge_core::path::validate_relative_path(value).is_ok()
}

/// F-25.m: maximum number of items a single submitted plan may carry.
pub const MAX_PLAN_ITEMS: usize = 256;

/// Validate and normalize a plan proposal before any case state is created.
pub fn validate_proposal(plan: &mut CasePlan) -> Result<(), ForgeError> {
    if plan.items.is_empty() {
        return Err(ForgeError::Plan {
            class: "plan_schema_error",
            message: "plan must contain at least one item".into(),
        });
    }
    // F-25.m: an upper bound on plan size. The template path is already
    // bounded (MAX_REPEATED_ENTRIES), but a directly submitted adversarial
    // plan is not — and every downstream derivation (replay, satisfiability,
    // cycle check) is at least linear in item count.
    if plan.items.len() > MAX_PLAN_ITEMS {
        return Err(ForgeError::Plan {
            class: "plan_schema_error",
            message: format!(
                "plan exceeds maximum item count: {} > {MAX_PLAN_ITEMS}",
                plan.items.len()
            ),
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
        // F-25.m: a SandboxedTask with no operations would dispatch as a
        // silent no-op episode (activate → settle nothing) while consuming a
        // permit and a settlement record. Exception: an item whose settlement
        // criteria declare an evaluator executes *through* that evaluator
        // (e.g. governed artifact transitions) rather than through item
        // operations, so it is not a no-op.
        if item.item_kind == ItemKind::SandboxedTask
            && item.operations.is_empty()
            && item.settlement_criteria.evaluator.is_none()
        {
            return Err(ForgeError::Plan {
                class: "plan_schema_error",
                message: format!(
                    "sandboxed task {} must declare at least one operation",
                    item.plan_item_id
                ),
            });
        }
        // AgentTask items carry exactly one Operation::AgentTask (validated below).
        if !matches!(
            item.item_kind,
            ItemKind::SandboxedTask | ItemKind::AgentTask
        ) && !item.operations.is_empty()
        {
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
                sea_forge_core::types::Operation::AgentProbe {
                    endpoint_ref,
                    model,
                    prompt_sha256,
                } => {
                    if endpoint_ref.is_empty() || model.is_empty() || prompt_sha256.is_empty() {
                        return Err(ForgeError::Plan {
                            class: "plan_schema_error",
                            message: "agent_probe requires endpoint, model, and prompt hash".into(),
                        });
                    }
                }
                sea_forge_core::types::Operation::AgentTask {
                    endpoint_ref,
                    instruction,
                    max_turns,
                    ..
                } => {
                    if endpoint_ref.is_empty() || instruction.is_empty() || *max_turns == 0 {
                        return Err(ForgeError::Plan {
                            class: "plan_schema_error",
                            message:
                                "agent_task requires endpoint_ref, instruction, and max_turns > 0"
                                    .into(),
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
        // AgentTask items carry exactly one Operation::AgentTask — nothing else.
        if item.item_kind == ItemKind::AgentTask
            && (item.operations.len() != 1
                || !matches!(
                    item.operations[0],
                    sea_forge_core::types::Operation::AgentTask { .. }
                ))
        {
            return Err(ForgeError::Plan {
                class: "plan_schema_error",
                message: format!(
                    "agent_task item {} must carry exactly one Operation::AgentTask",
                    item.plan_item_id
                ),
            });
        }
    }
    check_satisfiability(&plan.items)
}

/// F-25.m: iterative DFS. The recursive form could overflow the stack on an
/// adversarial ~10^5-node dependency chain submitted straight to `submit`
/// (an abort-class DoS the server cannot contain); the explicit stack makes
/// depth a heap concern instead.
fn has_cycle(
    root: &str,
    graph: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    stack_set: &mut HashSet<String>,
) -> bool {
    // Each frame: (node, next neighbor index). `on_path` marks the current
    // DFS path; `visited` marks fully-explored nodes.
    let mut frames: Vec<(String, usize)> = vec![(root.to_string(), 0)];
    stack_set.insert(root.to_string());
    while let Some((node, ref mut next)) = frames.last_mut() {
        let neighbors = graph.get(node.as_str()).map(Vec::as_slice).unwrap_or(&[]);
        if *next < neighbors.len() {
            let neighbor = &neighbors[*next];
            *next += 1;
            if stack_set.contains(neighbor) {
                return true;
            }
            if !visited.contains(neighbor) {
                stack_set.insert(neighbor.clone());
                frames.push((neighbor.clone(), 0));
            }
        } else {
            let done = frames.pop().expect("frame stack cannot be empty here");
            stack_set.remove(&done.0);
            visited.insert(done.0);
        }
    }
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
