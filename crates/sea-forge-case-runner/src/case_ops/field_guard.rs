//! The Field Case write rules that live where the events are written
//! (WP-21 part B, `contracts/fde/README.md` §7 R11 and the hard-gate rule).
//!
//! The gate and close rules as functions. They are applied by exactly one
//! caller, `case_guard::admit_write`, at the write, under the case lock; the
//! pre-checks elsewhere only decide earlier whether to try. A case whose plan
//! is not `fde.field_case@1` is not touched.

use sea_forge_core::errors::ForgeError;
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

/// Is the case at `case_dir` a Field Case? Detection does not depend on a
/// parseable plan: the plan's template says so, and so does any Field Case
/// marker (the document, a case record). A case with a marker and a plan that
/// does not parse is a Field Case whose store is corrupt, and fails closed in
/// the guard's integrity check.
pub fn is_field_case(case_dir: &Path) -> bool {
    let plan = std::fs::read(case_dir.join("plan.json")).ok();
    let by_plan = plan.as_ref().is_some_and(|bytes| {
        match serde_json::from_slice::<Value>(bytes) {
            Ok(plan) => plan["template_ref"] == FIELD_CASE_TEMPLATE,
            // A plan that does not parse but names the template is a Field
            // Case with a corrupt plan (it fails closed in the load).
            Err(_) => String::from_utf8_lossy(bytes).contains(FIELD_CASE_TEMPLATE),
        }
    });
    by_plan || has_marker(case_dir)
}

/// ANY `field*` artifact of the case directory (the document, the ledger, an
/// auxiliary store), whatever its size: deleting the plan and the document
/// does not make a Field Case look plain.
fn has_marker(case_dir: &Path) -> bool {
    std::fs::read_dir(case_dir).is_ok_and(|entries| {
        entries
            .flatten()
            .any(|e| e.file_name().to_string_lossy().starts_with("field"))
    })
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
    gate_check_records(&records(case_dir), item, how)
}

/// [`gate_item_check`] on records already loaded and verified (the pure form
/// `case_guard::decide` uses).
pub fn gate_check_records(records: &[Value], item: &str, how: &str) -> Result<(), Refusal> {
    let Some(gate) = gate_of_item(item) else {
        return Ok(());
    };
    let hard = HARD_GATES.contains(&gate);
    if how == "skip" && !hard {
        return Ok(());
    }
    let state = records
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
