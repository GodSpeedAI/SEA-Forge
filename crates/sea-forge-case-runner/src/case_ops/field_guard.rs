//! The Field Case write rules that live where the events are written
//! (WP-21 part B, `contracts/fde/README.md` §7 R11 and the hard-gate rule).
//!
//! Every append of a case trace event goes through [`guard_event`], so no path
//! (a transition, the advance engine, a human completion, the dispatcher) can
//! complete a gate item or close a Field Case on a stale read: the check runs
//! at the write, on the Field Case records as they are then. A case whose plan
//! is not `fde.field_case@1` is not touched.

use sea_forge_core::{errors::ForgeError, types::TraceKind};
use serde_json::{json, Value};
use std::path::Path;

/// The template of a Field Case.
pub const FIELD_CASE_TEMPLATE: &str = "fde.field_case@1";
/// The hard gates (D12, SOP 8.1b): never waived without a passing assessment.
pub const HARD_GATES: [&str; 2] = ["allocate_authority", "integrate_govern"];
const RECORDS: &str = "field-records.jsonl";

/// A refusal with the contract's code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub code: &'static str,
    pub message: String,
}

impl Refusal {
    /// The contract's `Error` object.
    pub fn to_value(&self) -> Value {
        json!({"outcome": "refused", "code": self.code, "message": self.message,
               "no_side_effect": true})
    }

    /// As a [`ForgeError`] the server turns back into the contract's refusal.
    pub fn into_error(self) -> ForgeError {
        ForgeError::Plan {
            class: "episode_gate_refused",
            message: self.to_value().to_string(),
        }
    }
}

/// The gate a plan item stands for (`gate_<id>` of the template), if any.
pub fn gate_of_item(item: &str) -> Option<&str> {
    let gate = item.strip_prefix("gate_")?;
    sea_forge_planner::templates::FDE_GATES
        .contains(&gate)
        .then_some(gate)
}

/// Is the case at `case_dir` a Field Case (its plan carries the template)?
pub fn is_field_case(case_dir: &Path) -> bool {
    std::fs::read(case_dir.join("plan.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .is_some_and(|plan| plan["template_ref"] == FIELD_CASE_TEMPLATE)
}

fn records(case_dir: &Path) -> Vec<Value> {
    let bytes = std::fs::read(case_dir.join(RECORDS)).unwrap_or_default();
    let end = bytes.iter().rposition(|b| *b == b'\n').map_or(0, |i| i + 1);
    bytes[..end]
        .split(|b| *b == b'\n')
        .filter_map(|line| serde_json::from_slice(line).ok())
        .collect()
}

/// THE hard-gate rule, in one place: a gate item completes (or is achieved)
/// only when the latest `gate.assessed` for its gate is `met` or
/// `not_applicable` (which already rests on its approval); a hard gate is not
/// skipped either unless it passes. `how` is `"complete"` or `"skip"`.
/// Refused with the contract's code; the caller writes nothing.
pub fn gate_item_check(case_dir: &Path, item: &str, how: &str) -> Result<(), Refusal> {
    let Some(gate) = gate_of_item(item) else {
        return Ok(());
    };
    let hard = HARD_GATES.contains(&gate);
    if how == "skip" && !hard {
        return Ok(());
    }
    let state = records(case_dir)
        .iter()
        .rfind(|r| r["kind"] == "gate.assessed" && r["gate"] == gate)
        .and_then(|r| r["state"].as_str().map(String::from));
    if matches!(state.as_deref(), Some("met" | "not_applicable")) {
        return Ok(());
    }
    let seen = state.as_deref().unwrap_or("unassessed");
    Err(if hard {
        Refusal {
            code: "HARD_GATE_NOT_WAIVABLE",
            message: format!(
                "hard gate {gate} is {seen}: its item moves only on a passing assessment"
            ),
        }
    } else {
        Refusal {
            code: "AUTHORITY_DENIED",
            message: format!("gate {gate} is {seen}: its item moves only on a passing assessment"),
        }
    })
}

/// R11: no close without a case-level `close-*` disposition.
pub fn close_check(case_dir: &Path) -> Result<(), Refusal> {
    let closed = records(case_dir).iter().any(|r| {
        r["kind"] == "disposition.recorded"
            && r["level"] == "case"
            && r["disposition"]
                .as_str()
                .is_some_and(|d| d.starts_with("close-"))
    });
    if closed {
        Ok(())
    } else {
        Err(Refusal {
            code: "CLOSE_DISPOSITION_REQUIRED",
            message: "this action ends the case and no case-level close disposition exists".into(),
        })
    }
}

/// Would an event of `kind` on `item` be refused? Called by every appender
/// before anything is written.
pub fn event_check(case_dir: &Path, kind: &TraceKind, item: Option<&str>) -> Result<(), Refusal> {
    match kind {
        TraceKind::ItemCompleted | TraceKind::MilestoneAchieved => {
            if let Some(item) = item.filter(|i| gate_of_item(i).is_some()) {
                if is_field_case(case_dir) {
                    gate_item_check(case_dir, item, "complete")?;
                }
            }
            Ok(())
        }
        TraceKind::ItemTerminated => {
            if let Some(item) = item.filter(|i| gate_of_item(i).is_some()) {
                if is_field_case(case_dir) {
                    gate_item_check(case_dir, item, "skip")?;
                }
            }
            Ok(())
        }
        TraceKind::CaseClosed if is_field_case(case_dir) => close_check(case_dir),
        _ => Ok(()),
    }
}

/// [`event_check`] as the appenders use it.
pub fn guard_event(
    case_dir: &Path,
    kind: &TraceKind,
    item: Option<&str>,
) -> Result<(), ForgeError> {
    event_check(case_dir, kind, item).map_err(Refusal::into_error)
}
