//! `case.list` / `case.get_overview` / `case.get_horizon` — SFWP **inspect**
//! methods for navigating committed cases (Task 7, ADR-003 additive).
//!
//! # Contract
//!
//! All three are *read-only projections* over records the kernel already
//! committed. They introduce **no new truth** and mutate nothing:
//!
//! | View | Source of truth |
//! |---|---|
//! | `case.list` | `<root>/cases/<case_id>/case.json` |
//! | `case.get_overview` | `case.json` + `plan.json` + `<root>/runs/<run_id>/settlement.json` |
//! | `case.get_horizon` | `plan.json` items folded over `case-events.jsonl` |
//!
//! # Why the horizon is folded from trace events, not read off a status field
//!
//! There is no per-item status field anywhere in the kernel to read: item
//! standing exists only as the sequence of [`TraceKind`] events the case runner
//! appended. Folding those events *is* reading kernel truth — the same
//! derivation `next_case_actions` performs when it decides what may run next.
//! Any per-item status cached elsewhere would be a second authority over the
//! same fact, and would drift.
//!
//! # Execution is never settlement
//!
//! [`HorizonItem::execution`] and [`HorizonItem::settlement`] are separate
//! fields with disjoint vocabularies, and neither is derived from the other.
//! An item whose command exited zero is `execution: "completed"` with
//! `settlement: "unsettled"` until a settlement record says otherwise; a
//! rejected settlement leaves execution `completed` and settlement `rejected`.
//! Collapsing them would let a process exit code masquerade as governed
//! completion — the failure the epic's invariant 2 exists to prevent.
//!
//! # Absence is reported, never inferred
//!
//! A case directory with an unreadable `case.json` is skipped from `case.list`
//! rather than rendered as a broken row, and a requested case that does not
//! exist returns a typed `not_found` — distinct from a case that exists with no
//! items. `unknown ≠ unavailable` holds throughout.

use std::collections::BTreeMap;
use std::path::Path;

use schemars::JsonSchema;
use sea_forge_core::types::{Case, CasePlan, ItemKind, SettlementEvent, TraceEvent, TraceKind};
use serde::{Deserialize, Serialize};

/// How far an item has progressed *as execution* — derived only from the trace
/// events the case runner appended. Deliberately disjoint from
/// [`SettlementStanding`] so no consumer can conflate the two.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStanding {
    /// In the plan, but no event has been observed for it yet.
    Pending,
    /// Entry criteria met; eligible to be dispatched.
    Enabled,
    /// Dispatched and running.
    Active,
    /// Execution finished without error. Says nothing about settlement.
    Completed,
    /// Execution ended in error.
    Failed,
    /// Deliberately stopped before completing.
    Terminated,
}

/// Whether a *governed settlement* has been recorded for the item. Distinct
/// from execution: work can execute perfectly and settle as rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SettlementStanding {
    /// No settlement record exists for this item yet.
    Unsettled,
    /// A settlement record accepted the work.
    Accepted,
    /// A settlement record rejected the work.
    Rejected,
    /// A settlement record escalated the decision to a human.
    Escalated,
}

/// One row of the case horizon: a plan item with its separately-derived
/// execution and settlement standing.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct HorizonItem {
    pub plan_item_id: String,
    pub name: String,
    /// `sandboxed_task`, `human_task`, `agent_task`, `stage`, `milestone`, …
    pub item_kind: String,
    pub execution: ExecutionStanding,
    pub settlement: SettlementStanding,
    /// Stage this item belongs to, when the plan nests it under one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_stage: Option<String>,
    /// Plan items that must settle before this one is eligible.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Timestamp of the most recent trace event observed for this item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event_at: Option<String>,
    /// Run ids that carried episodes of this item, in first-seen order. An item
    /// retried as a new episode has more than one.
    #[serde(default)]
    pub run_ids: Vec<String>,
}

/// `case.get_horizon` result body.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CaseHorizon {
    pub case_id: String,
    /// `active`, `awaiting_approval`, `completed`, `terminated`.
    pub case_state: String,
    pub items: Vec<HorizonItem>,
    /// `event_id` of the last trace event folded into this view. A client that
    /// has seen a later event than this knows its view is behind and must
    /// refetch rather than patch — the projection is rebuildable, never a
    /// second source of truth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event_id: Option<String>,
    /// Number of trace events folded, so a client can tell "no events yet" from
    /// "events exist but none matched an item".
    pub events_folded: usize,
}

/// One row of `case.list`.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CaseSummary {
    pub case_id: String,
    pub case_state: String,
    /// The operator-facing intent summary committed with the case.
    pub summary: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
    pub run_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_reason: Option<String>,
}

/// `case.list` result body.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct CaseListResult {
    /// Newest first. Empty is an honest answer for a cell with no cases.
    pub cases: Vec<CaseSummary>,
    /// Case directories that exist but could not be read as a `Case` record.
    /// Reported rather than silently dropped: a case whose record is
    /// unreadable is an integrity signal, not an absence.
    #[serde(default)]
    pub unreadable: Vec<String>,
}

/// A settlement record linked to a run, projected for the overview.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RunSettlement {
    pub run_id: String,
    pub status: SettlementStanding,
    /// The evidence basis the settlement cited.
    #[serde(default)]
    pub basis: Vec<String>,
    pub review_required: bool,
    pub settled_at: String,
}

/// `case.get_overview` result body.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CaseOverview {
    pub case_id: String,
    pub case_state: String,
    pub summary: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_reason: Option<String>,
    /// The plan this case was committed from, when `plan.json` is readable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_ref: Option<String>,
    /// `name@version` of the originating template, when the plan recorded one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_ref: Option<String>,
    pub item_count: usize,
    /// Stage item ids, in plan order.
    #[serde(default)]
    pub stages: Vec<String>,
    /// Settlement records found for this case's runs. Empty means nothing has
    /// settled yet — never that the work failed.
    #[serde(default)]
    pub settlements: Vec<RunSettlement>,
    pub run_ids: Vec<String>,
}

/// Why a case view could not be produced. Mapped to a machine-readable
/// `error_class` on the wire so the frontend can tell "no such case" from
/// "the record is there but unreadable" — the two need different operator
/// responses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaseViewError {
    /// No case directory with a readable `case.json` for this id.
    NotFound(String),
    /// The record exists but could not be parsed.
    Unreadable(String),
}

impl CaseViewError {
    pub fn class(&self) -> &'static str {
        match self {
            CaseViewError::NotFound(_) => "not_found",
            CaseViewError::Unreadable(_) => "record_unreadable",
        }
    }

    pub fn message(&self) -> String {
        match self {
            CaseViewError::NotFound(id) => format!("no case record for {id}"),
            CaseViewError::Unreadable(detail) => detail.clone(),
        }
    }
}

fn case_state_str(case: &Case) -> String {
    serde_json::to_value(&case.state)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".into())
}

fn item_kind_str(kind: &ItemKind) -> String {
    serde_json::to_value(kind)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".into())
}

/// `case_id` values address a directory under `<root>/cases`, so a caller-
/// supplied id must never be able to escape it. Ids are kernel-minted
/// (`ids::case_id`) and alphanumeric-with-dashes; anything else is refused
/// before it reaches the filesystem.
fn valid_case_id(case_id: &str) -> bool {
    !case_id.is_empty()
        && case_id.len() <= 128
        && case_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn read_case(root: &Path, case_id: &str) -> Result<Case, CaseViewError> {
    if !valid_case_id(case_id) {
        return Err(CaseViewError::NotFound(case_id.to_string()));
    }
    let path = root.join("cases").join(case_id).join("case.json");
    let bytes = std::fs::read(&path).map_err(|_| CaseViewError::NotFound(case_id.to_string()))?;
    serde_json::from_slice(&bytes).map_err(|error| {
        CaseViewError::Unreadable(format!("case {case_id} record is unreadable: {error}"))
    })
}

fn read_plan(root: &Path, case_id: &str) -> Option<CasePlan> {
    let path = root.join("cases").join(case_id).join("plan.json");
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Read `case-events.jsonl`, skipping any trailing malformed line.
///
/// A truncated tail is the expected shape after a crash mid-append, and the
/// ledger's own quarantine path handles that durably. Here the honest response
/// is to fold the well-formed prefix rather than fail the whole view: a
/// horizon that renders everything up to the crash is more useful to an
/// operator diagnosing that crash than an error page.
fn read_case_events(root: &Path, case_id: &str) -> Vec<TraceEvent> {
    let path = root.join("cases").join(case_id).join("case-events.jsonl");
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map_while(|line| serde_json::from_str::<TraceEvent>(line).ok())
        .collect()
}

/// List every readable case record under `<root>/cases`, newest first.
pub fn list(root: &Path) -> CaseListResult {
    let mut cases = Vec::new();
    let mut unreadable = Vec::new();

    let Ok(entries) = std::fs::read_dir(root.join("cases")) else {
        // No `cases/` directory yet is the normal state of a fresh cell.
        return CaseListResult::default();
    };

    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Some(case_id) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        match read_case(root, &case_id) {
            Ok(case) => cases.push(CaseSummary {
                case_id: case.case_id.clone(),
                case_state: case_state_str(&case),
                summary: case.intent.summary.clone(),
                created_at: case.created_at.clone(),
                closed_at: case.closed_at.clone(),
                run_count: case.run_ids.len(),
                close_reason: case.close_reason.clone(),
            }),
            // A directory with no `case.json` at all is not a case (it may be
            // a partially-created one); only a *present but broken* record is
            // worth reporting as an integrity signal.
            Err(CaseViewError::NotFound(_)) => {}
            Err(CaseViewError::Unreadable(_)) => unreadable.push(case_id),
        }
    }

    // Case ids are ULID-prefixed and monotonic, but `created_at` is the field
    // an operator actually reasons about, so sort on it and break ties by id
    // for a total order (a stable list is a usability requirement — rows must
    // not reshuffle between polls).
    cases.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| b.case_id.cmp(&a.case_id))
    });
    unreadable.sort();

    CaseListResult { cases, unreadable }
}

/// Project one case's overview from its committed records.
pub fn get_overview(root: &Path, case_id: &str) -> Result<CaseOverview, CaseViewError> {
    let case = read_case(root, case_id)?;
    let plan = read_plan(root, case_id);

    let settlements = case
        .run_ids
        .iter()
        .filter_map(|run_id| read_settlement(root, run_id))
        .collect();

    Ok(CaseOverview {
        case_id: case.case_id.clone(),
        case_state: case_state_str(&case),
        summary: case.intent.summary.clone(),
        created_at: case.created_at.clone(),
        closed_at: case.closed_at.clone(),
        close_reason: case.close_reason.clone(),
        plan_ref: Some(case.plan_ref.clone()),
        template_ref: plan.as_ref().and_then(|p| p.template_ref.clone()),
        item_count: plan.as_ref().map(|p| p.items.len()).unwrap_or(0),
        stages: case.stages.clone(),
        settlements,
        run_ids: case.run_ids.clone(),
    })
}

fn read_settlement(root: &Path, run_id: &str) -> Option<RunSettlement> {
    // Through the one locator (K-04/DATA-02): a case's own episodes write the
    // case-owned layout, so reading `<root>/runs/` alone made a case view
    // unable to see the settlements of the runs that case created.
    let path = crate::sfwp::run_views::run_dir(root, run_id)?.join("settlement.json");
    let bytes = std::fs::read(path).ok()?;
    let event: SettlementEvent = serde_json::from_slice(&bytes).ok()?;
    Some(RunSettlement {
        run_id: run_id.to_string(),
        status: settlement_standing(&event),
        basis: event.basis.clone(),
        review_required: event.review_required,
        settled_at: event.settled_at.clone(),
    })
}

fn settlement_standing(event: &SettlementEvent) -> SettlementStanding {
    use sea_forge_core::types::SettlementStatus;
    match event.status {
        SettlementStatus::Accepted => SettlementStanding::Accepted,
        SettlementStatus::Rejected => SettlementStanding::Rejected,
        SettlementStatus::Escalated => SettlementStanding::Escalated,
    }
}

/// Fold this case's trace events over its plan items to produce the horizon.
pub fn get_horizon(root: &Path, case_id: &str) -> Result<CaseHorizon, CaseViewError> {
    let case = read_case(root, case_id)?;
    let plan = read_plan(root, case_id);
    let events = read_case_events(root, case_id);

    // Seed one row per plan item so an item with no events yet is `pending`
    // rather than missing. An absent plan yields an empty item list, not a
    // fabricated one.
    let mut rows: BTreeMap<String, HorizonItem> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    if let Some(plan) = plan.as_ref() {
        for item in &plan.items {
            order.push(item.plan_item_id.clone());
            rows.insert(
                item.plan_item_id.clone(),
                HorizonItem {
                    plan_item_id: item.plan_item_id.clone(),
                    name: item.name.clone(),
                    item_kind: item_kind_str(&item.item_kind),
                    execution: ExecutionStanding::Pending,
                    settlement: SettlementStanding::Unsettled,
                    parent_stage: item.parent_stage.clone(),
                    depends_on: item.depends_on.clone(),
                    last_event_at: None,
                    run_ids: Vec::new(),
                },
            );
        }
    }

    let mut last_event_id = None;
    for event in &events {
        last_event_id = Some(event.event_id.clone());
        let Some(item_id) = event.plan_item_id.as_ref() else {
            continue;
        };
        let Some(row) = rows.get_mut(item_id) else {
            // An event for an item the plan does not contain means the plan was
            // mutated after the event, or the records disagree. Skipping keeps
            // the view a projection of the *plan*; the discrepancy stays
            // visible via `events_folded` exceeding what the rows explain.
            continue;
        };

        // Execution standing advances only on execution-domain events; the
        // settlement arm is deliberately unreachable from them.
        match event.kind {
            TraceKind::ItemEnabled => row.execution = ExecutionStanding::Enabled,
            TraceKind::ItemActivated => row.execution = ExecutionStanding::Active,
            TraceKind::ItemCompleted | TraceKind::HumanTaskCompleted => {
                row.execution = ExecutionStanding::Completed;
            }
            TraceKind::ItemFailed => row.execution = ExecutionStanding::Failed,
            TraceKind::ItemTerminated => row.execution = ExecutionStanding::Terminated,
            TraceKind::SettlementRecorded => {
                // Settlement standing comes from the settlement record's own
                // status field, never from the fact that execution finished.
                if let Some(status) = event.payload.get("status").and_then(|v| v.as_str()) {
                    row.settlement = match status {
                        "accepted" => SettlementStanding::Accepted,
                        "rejected" => SettlementStanding::Rejected,
                        "escalated" => SettlementStanding::Escalated,
                        _ => SettlementStanding::Unsettled,
                    };
                }
            }
            _ => {}
        }

        row.last_event_at = Some(event.timestamp.clone());
        if !event.run_id.is_empty() && !row.run_ids.contains(&event.run_id) {
            row.run_ids.push(event.run_id.clone());
        }
    }

    let items = order
        .iter()
        .filter_map(|id| rows.get(id).cloned())
        .collect();

    Ok(CaseHorizon {
        case_id: case.case_id.clone(),
        case_state: case_state_str(&case),
        items,
        last_event_id,
        events_folded: events.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_case_id_cannot_traverse_out_of_the_cases_directory() {
        assert!(!valid_case_id("../../etc/passwd"));
        assert!(!valid_case_id("a/b"));
        assert!(!valid_case_id(""));
        assert!(valid_case_id("case-01JABCDEF"));
    }

    #[test]
    fn a_missing_case_is_not_found_rather_than_unreadable() {
        let root = tempfile::tempdir().unwrap();
        let error = get_overview(root.path(), "case-nope").unwrap_err();
        assert_eq!(error.class(), "not_found");
    }

    #[test]
    fn listing_a_cell_with_no_cases_directory_is_an_empty_list_not_an_error() {
        let root = tempfile::tempdir().unwrap();
        let result = list(root.path());
        assert!(result.cases.is_empty());
        assert!(result.unreadable.is_empty());
    }
}
