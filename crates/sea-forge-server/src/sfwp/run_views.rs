//! `run.list` / `run.get` — SFWP **inspect** methods for the run record, the
//! join point every other governed view already references but none could
//! navigate to (Task 8, ADR-003 additive).
//!
//! # Why this is the linchpin view
//!
//! [`super::case_views::CaseOverview::run_ids`], [`super::case_views::HorizonItem::run_ids`],
//! [`super::case_views::RunSettlement::run_id`], and every [`TraceEvent::run_id`]
//! on the event stream are run ids. Before this module they were opaque strings:
//! the UI could display them but could not resolve them to anything, which left
//! epic invariant 4 — *every important status resolves to committed source
//! records* — unsatisfiable by construction. `run.get` is that resolution.
//!
//! # Contract
//!
//! Both methods are *read-only projections* over records the kernel already
//! committed. They introduce **no new truth**, mutate nothing, and create no
//! ledger entry:
//!
//! | View | Source of truth |
//! |---|---|
//! | `run.list` | `<root>/runs/*` cross-indexed against `<root>/cases/*/case.json` |
//! | `run.get` | `<root>/runs/<run_id>/{plan,criteria,authority,settlement}.json`, `trace.jsonl`, `evidence.jsonl`, `declarations.jsonl` |
//!
//! # Execution termination is never settlement (epic 12.4)
//!
//! [`RunRecord::termination`] is read from the run's own trace — the
//! [`TraceKind::CommandFinished`] payload's `execution` object, or the halt/
//! failure event that ended the episode. [`RunRecord::settlement`] is read from
//! `settlement.json`. Neither is derived from the other, and they carry disjoint
//! vocabularies. A command that exited zero and a settlement that rejected the
//! work is a representable, correctly-rendered state; collapsing the two would
//! let an exit code define a governed outcome.
//!
//! # The criteria pairing reports the settlement's judgment, it does not re-judge
//!
//! Epic 12.3 asks for each immutable criterion paired with the evidence that
//! passed, failed, or remained unavailable. The temptation is to re-evaluate the
//! criteria here against the workspace. That would be a *second settlement
//! engine* — a competing authority over the same fact, guaranteed to drift from
//! `sea_forge_settlement::settle` the moment either changes.
//!
//! Instead the pairing is navigational: [`sea_forge_settlement::settle`] already
//! writes one basis token per criterion it evaluated (`exit_zero`/`exit_nonzero`,
//! `required_artifact_present:<path>`/`required_artifact_missing:<path>`,
//! `stdout_match`/`stdout_mismatch`, …). This module reads the declared criteria
//! from the plan, reads those tokens from the settlement record, and joins them.
//! A criterion whose deciding token is absent is [`CriterionStanding::Unavailable`]
//! — never assumed satisfied.
//!
//! # Evaluator scores are evidence, never standing
//!
//! Per spec §10.6 an evaluator score is recorded *as evidence* and never decides
//! acceptance on its own. Its criterion therefore reports
//! [`CriterionStanding::Recorded`] with the observed score rather than borrowing
//! the pass/fail vocabulary — the distinction the spec exists to protect.
//!
//! # Absence is reported, never inferred
//!
//! [`RunRecord::records`] lists every canonical run-record file with whether it
//! was actually present. A run missing `authority.json` renders as *that record
//! is not here*, which is an operator-actionable integrity signal, and is
//! distinct from a run whose authority decision denied the work. `unknown ≠
//! unavailable` holds throughout.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use sea_forge_core::types::{
    AuthorityDecision, Case, CasePlan, EvidenceRecord, SettlementCriteria,
    SettlementCriteriaRecord, SettlementDeclaration, SettlementEvent, TraceEvent, TraceKind,
};
use serde::{Deserialize, Serialize};

use super::case_views::{ExecutionStanding, SettlementStanding};

/// Canonical per-run record files, in the order an operator reads them:
/// what was planned, what it had to satisfy, what authorized it, what it did,
/// what it produced, and what judged it.
///
/// Shared with [`crate::sfwp::run_views::RunRecord::records`] so a newly-added
/// record file becomes visible in the UI by extending exactly one list.
const RUN_RECORD_FILES: &[&str] = &[
    "plan.json",
    "criteria.json",
    "authority.json",
    "trace.jsonl",
    "evidence.jsonl",
    "settlement.json",
    "declarations.jsonl",
    "semantic-envelope.json",
    "transcript-evidence.json",
];

/// How the *execution* episode ended. Deliberately disjoint from
/// [`SettlementStanding`]: this is the terminal fact about the process, not a
/// governed outcome (epic 12.4).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RunTermination {
    /// The trace event kind that ended the episode — `command_finished`,
    /// `run_halted`, `item_failed`, `run_finished`, …
    pub trace_kind: String,
    /// Process exit code, when the terminating event carried an execution
    /// result. Absent for episodes that never spawned a process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i64>,
    /// The kernel's `ExecutionStatus` for the episode (`completed`,
    /// `timed_out`, `spawn_failed`, `sandbox_violation`), when recorded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_status: Option<String>,
    pub at: String,
}

/// One row of the run's trace, projected for display. The payload is carried
/// through verbatim rather than summarized: the trace *is* the explanation of
/// every state transition (epic 7.4), and a lossy summary would be a second
/// account of it.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TraceRow {
    pub event_id: String,
    /// Snake-case `TraceKind`.
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_item_id: Option<String>,
    pub actor_id: String,
    pub timestamp: String,
    pub payload: serde_json::Value,
}

/// One row of the run's evidence journal.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceRow {
    pub evidence_id: String,
    /// `artifact`, `authority_decision`, `execution_result`, `recall`.
    pub kind: String,
    pub uri: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// The trace event this evidence was captured from — the link back that
    /// makes evidence attributable rather than free-floating.
    pub source_event_id: String,
}

/// The authority decision that gated this run, projected from `authority.json`.
///
/// Reason codes, matched rule, and required next steps are carried through so a
/// denial renders as a governed outcome with a next lawful path (epic 2.6, 11.8)
/// rather than as a generic error.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct AuthorityProjection {
    pub decision_id: String,
    /// `allow`, `deny`, `escalate`.
    pub verdict: String,
    /// `allow`, `deny`, `escalate`, `boundary`, `degraded`.
    pub normalized_disposition: String,
    pub reason: String,
    #[serde(default)]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_rule: Option<String>,
    #[serde(default)]
    pub policy_refs: Vec<String>,
    #[serde(default)]
    pub required_next_steps: Vec<String>,
    pub decided_at: String,
}

/// Whether the settlement record judged a declared criterion satisfied.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CriterionStanding {
    /// The settlement record's basis contains this criterion's passing token.
    Satisfied,
    /// The settlement record's basis contains this criterion's failing token.
    Unsatisfied,
    /// Observed and recorded, but not a pass/fail input to acceptance —
    /// evaluator scores (§10.6). Kept out of the pass/fail vocabulary on
    /// purpose: an evaluator score is evidence, never standing.
    Recorded,
    /// No settlement record yet, or it recorded no token deciding this
    /// criterion. Never assumed satisfied.
    Unavailable,
}

/// One declared criterion paired with what the settlement record said about it.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CriterionCheck {
    /// Stable machine key: `require_exit_zero`, `required_artifact`,
    /// `stdout_must_contain`, `agent_output_must_contain`, `require_approval`,
    /// `evaluator`, `batch`.
    pub criterion: String,
    /// The criterion's declared value, as committed in the plan.
    pub expected: String,
    pub standing: CriterionStanding,
    /// The exact basis token(s) from `settlement.json` that decided this row.
    /// Empty when `standing` is `unavailable` — the operator can then see the
    /// settlement said nothing about it, rather than inferring a verdict.
    #[serde(default)]
    pub basis: Vec<String>,
}

/// The settlement record, projected. Separate from [`RunRecord::termination`]
/// by construction (epic 12.4).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SettlementDetail {
    pub settlement_id: String,
    pub status: SettlementStanding,
    #[serde(default)]
    pub basis: Vec<String>,
    pub review_required: bool,
    pub settled_at: String,
    /// The immutable criteria record this settlement judged against. Absent on
    /// legacy runs, which the kernel marks with a `legacy_unattributed_criteria`
    /// basis token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criteria_ref: Option<String>,
}

/// One settlement declaration, projected for standing/reliability inspection
/// (epic 12.5). Weak and strong settlement must not look alike, so
/// `qualifies_for_capability` and the reliability weight travel with the row.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeclarationRow {
    pub declaration_id: String,
    /// `accepted`, `rejected`, … as recorded.
    pub status: String,
    pub strength: String,
    pub qualifies_for_capability: bool,
    pub declarer_actor_id: String,
    pub declarer_role: String,
    /// Whether the declarer is independent of the acting entity. Self-
    /// certification is visible here rather than silently weighted (epic 12.6).
    pub independent: bool,
    pub reliability_weight: String,
    pub issued_at: String,
}

/// Whether a canonical run-record file was present on disk.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RecordPresence {
    /// Filename relative to `<root>/runs/<run_id>/`.
    pub record: String,
    pub present: bool,
    /// Size in bytes when present — enough for an operator to tell an empty
    /// journal from a populated one without opening it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
}

/// One row of `run.list`.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RunSummary {
    pub run_id: String,
    /// The case whose `run_ids` contains this run. Absent for a run directory
    /// no case claims — an orphan, which epic 11.6 requires be *visible* rather
    /// than dropped from the list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_item_id: Option<String>,
    pub execution: ExecutionStanding,
    pub settlement: SettlementStanding,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    pub evidence_count: usize,
}

/// `run.list` result body.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct RunListResult {
    /// Newest first.
    pub runs: Vec<RunSummary>,
    /// Run directories that exist but whose trace could not be read at all.
    /// Reported rather than dropped: an unreadable run is an integrity signal.
    #[serde(default)]
    pub unreadable: Vec<String>,
}

/// `run.get` result body — the single linked view of one run episode
/// (epic 12.1).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RunRecord {
    pub run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_item_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_item_name: Option<String>,
    /// `sandboxed_task`, `human_task`, `agent_task`, …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_kind: Option<String>,
    pub execution: ExecutionStanding,
    pub settlement: SettlementStanding,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    /// How execution ended. Never conflated with `settlement` (epic 12.4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub termination: Option<RunTermination>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<AuthorityProjection>,
    /// The immutable criteria record id this run was committed against.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criteria_ref: Option<String>,
    /// Declared criteria paired with the settlement's own basis (epic 12.3).
    #[serde(default)]
    pub criteria: Vec<CriterionCheck>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settlement_record: Option<SettlementDetail>,
    #[serde(default)]
    pub declarations: Vec<DeclarationRow>,
    #[serde(default)]
    pub evidence: Vec<EvidenceRow>,
    #[serde(default)]
    pub trace: Vec<TraceRow>,
    /// Which canonical records exist for this run (epic 12.10). Absence is
    /// reported here rather than inferred from an empty panel.
    #[serde(default)]
    pub records: Vec<RecordPresence>,
}

/// Why a run view could not be produced. Mirrors
/// [`super::case_views::CaseViewError`] so both families map to the same
/// `error_class` wire vocabulary on the frontend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunViewError {
    NotFound(String),
    Unreadable(String),
}

impl RunViewError {
    pub fn class(&self) -> &'static str {
        match self {
            RunViewError::NotFound(_) => "not_found",
            RunViewError::Unreadable(_) => "record_unreadable",
        }
    }

    pub fn message(&self) -> String {
        match self {
            RunViewError::NotFound(id) => format!("no run record for {id}"),
            RunViewError::Unreadable(detail) => detail.clone(),
        }
    }
}

/// `run_id` values address a directory under `<root>/runs`, so a caller-supplied
/// id must never escape it. Ids are kernel-minted and alphanumeric-with-dashes;
/// anything else is refused before it reaches the filesystem. Same rule as
/// [`super::case_views`] — deliberately duplicated rather than shared, so
/// loosening one id's validation can never silently loosen the other's.
fn valid_run_id(run_id: &str) -> bool {
    !run_id.is_empty()
        && run_id.len() <= 128
        && run_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn run_dir(root: &Path, run_id: &str) -> Option<PathBuf> {
    valid_run_id(run_id).then(|| root.join("runs").join(run_id))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Option<T> {
    serde_json::from_slice(&std::fs::read(path).ok()?).ok()
}

/// Read a JSONL journal, folding the well-formed prefix.
///
/// A truncated tail is the expected shape after a crash mid-append. Rendering
/// everything up to the crash serves an operator diagnosing that crash better
/// than failing the whole view — the same choice `case_views` makes for
/// `case-events.jsonl`, for the same reason.
fn read_jsonl<T: serde::de::DeserializeOwned>(path: &Path) -> Vec<T> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map_while(|line| serde_json::from_str::<T>(line).ok())
        .collect()
}

fn enum_str<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".into())
}

/// Map every run id a case claims back to that case. Built by one pass over
/// `<root>/cases/*/case.json`, which is also the only place that ownership is
/// recorded — a run does not name its own case.
fn case_index(root: &Path) -> BTreeMap<String, String> {
    let mut index = BTreeMap::new();
    let Ok(entries) = std::fs::read_dir(root.join("cases")) else {
        return index;
    };
    for entry in entries.flatten() {
        let path = entry.path().join("case.json");
        let Some(case) = read_json::<Case>(&path) else {
            continue;
        };
        for run_id in &case.run_ids {
            index.insert(run_id.clone(), case.case_id.clone());
        }
    }
    index
}

/// Fold a run's trace into execution standing, timestamps, and termination.
///
/// Only execution-domain events move execution standing; the settlement arm is
/// unreachable from here by construction, mirroring `case_views::get_horizon`.
struct TraceFold {
    execution: ExecutionStanding,
    plan_item_id: Option<String>,
    started_at: Option<String>,
    finished_at: Option<String>,
    termination: Option<RunTermination>,
}

fn fold_trace(events: &[TraceEvent]) -> TraceFold {
    let mut fold = TraceFold {
        execution: ExecutionStanding::Pending,
        plan_item_id: None,
        started_at: None,
        finished_at: None,
        termination: None,
    };

    for event in events {
        if fold.plan_item_id.is_none() {
            fold.plan_item_id.clone_from(&event.plan_item_id);
        }
        if fold.started_at.is_none() {
            fold.started_at = Some(event.timestamp.clone());
        }

        match event.kind {
            TraceKind::ItemEnabled => fold.execution = ExecutionStanding::Enabled,
            TraceKind::RunStarted | TraceKind::ItemActivated | TraceKind::CommandStarted => {
                fold.execution = ExecutionStanding::Active;
            }
            TraceKind::ItemCompleted | TraceKind::HumanTaskCompleted => {
                fold.execution = ExecutionStanding::Completed;
            }
            TraceKind::ItemFailed | TraceKind::InternalError => {
                fold.execution = ExecutionStanding::Failed;
            }
            TraceKind::ItemTerminated => fold.execution = ExecutionStanding::Terminated,
            _ => {}
        }

        // A terminating event also fixes how the episode ended. `CommandFinished`
        // carries the kernel's own `ExecutionResult` in its payload — read it
        // rather than re-deriving an exit status from anything else.
        match event.kind {
            TraceKind::CommandFinished => {
                let execution = event.payload.get("execution");
                fold.termination = Some(RunTermination {
                    trace_kind: enum_str(&event.kind),
                    exit_code: execution
                        .and_then(|e| e.get("exit_code"))
                        .and_then(serde_json::Value::as_i64),
                    execution_status: execution
                        .and_then(|e| e.get("status"))
                        .and_then(|s| s.as_str().map(str::to_string)),
                    at: event.timestamp.clone(),
                });
                // `CommandFinished` says the process ended, not that the item
                // completed — the runner appends `ItemCompleted`/`ItemFailed`
                // separately, and only that decides execution standing.
                if fold.execution == ExecutionStanding::Pending {
                    fold.execution = ExecutionStanding::Active;
                }
                fold.finished_at = Some(event.timestamp.clone());
            }
            TraceKind::RunHalted | TraceKind::ItemFailed | TraceKind::ItemTerminated => {
                fold.termination = Some(RunTermination {
                    trace_kind: enum_str(&event.kind),
                    exit_code: None,
                    execution_status: None,
                    at: event.timestamp.clone(),
                });
                fold.finished_at = Some(event.timestamp.clone());
            }
            TraceKind::RunFinished => fold.finished_at = Some(event.timestamp.clone()),
            _ => {}
        }
    }

    fold
}

fn settlement_standing(event: &SettlementEvent) -> SettlementStanding {
    use sea_forge_core::types::SettlementStatus;
    match event.status {
        SettlementStatus::Accepted => SettlementStanding::Accepted,
        SettlementStatus::Rejected => SettlementStanding::Rejected,
        SettlementStatus::Escalated => SettlementStanding::Escalated,
    }
}

/// Join one declared criterion to the settlement basis token that decided it.
///
/// `pass` and `fail` name the exact tokens `sea_forge_settlement::settle`
/// appends. Neither present means the settlement recorded nothing about this
/// criterion, which is [`CriterionStanding::Unavailable`] — the honest answer,
/// and never an assumed pass.
fn join_basis(basis: &[String], pass: &str, fail: &str) -> (CriterionStanding, Vec<String>) {
    if basis.iter().any(|b| b == fail) {
        (CriterionStanding::Unsatisfied, vec![fail.to_string()])
    } else if basis.iter().any(|b| b == pass) {
        (CriterionStanding::Satisfied, vec![pass.to_string()])
    } else {
        (CriterionStanding::Unavailable, Vec::new())
    }
}

/// Pair the plan's declared criteria with the settlement record's basis.
///
/// See the module docs: this joins, it does not evaluate. The settlement engine
/// remains the only thing that decides whether a criterion was met.
fn pair_criteria(
    criteria: &SettlementCriteria,
    settlement: Option<&SettlementEvent>,
) -> Vec<CriterionCheck> {
    let basis: &[String] = settlement.map(|s| s.basis.as_slice()).unwrap_or(&[]);
    let mut checks = Vec::new();

    if criteria.require_exit_zero {
        let (standing, matched) = join_basis(basis, "exit_zero", "exit_nonzero");
        checks.push(CriterionCheck {
            criterion: "require_exit_zero".into(),
            expected: "process exits 0".into(),
            standing,
            basis: matched,
        });
    }

    for path in &criteria.required_artifacts {
        let (standing, matched) = join_basis(
            basis,
            &format!("required_artifact_present:{path}"),
            &format!("required_artifact_missing:{path}"),
        );
        checks.push(CriterionCheck {
            criterion: "required_artifact".into(),
            expected: path.clone(),
            standing,
            basis: matched,
        });
    }

    if let Some(needle) = &criteria.stdout_must_contain {
        let (standing, matched) = join_basis(basis, "stdout_match", "stdout_mismatch");
        checks.push(CriterionCheck {
            criterion: "stdout_must_contain".into(),
            expected: needle.clone(),
            standing,
            basis: matched,
        });
    }

    if let Some(needle) = &criteria.agent_output_must_contain {
        // The delegation path records a token only on *mismatch* — a matching
        // response is accepted silently. So a mismatch is decisive, an accepted
        // settlement implies the literal was found, and anything else is
        // genuinely unknown.
        let (standing, matched) = if basis.iter().any(|b| b == "agent_output_mismatch") {
            (
                CriterionStanding::Unsatisfied,
                vec!["agent_output_mismatch".to_string()],
            )
        } else if settlement.map(settlement_standing) == Some(SettlementStanding::Accepted) {
            (CriterionStanding::Satisfied, Vec::new())
        } else {
            (CriterionStanding::Unavailable, Vec::new())
        };
        checks.push(CriterionCheck {
            criterion: "agent_output_must_contain".into(),
            expected: needle.clone(),
            standing,
            basis: matched,
        });
    }

    if criteria.require_approval {
        // Approval is enforced upstream by the authority engine, so the
        // settlement's authority token is what reports it.
        let (standing, matched) = if basis.iter().any(|b| b == "authority_deny") {
            (
                CriterionStanding::Unsatisfied,
                vec!["authority_deny".to_string()],
            )
        } else if basis.iter().any(|b| b == "authority_escalate") {
            (
                CriterionStanding::Unavailable,
                vec!["authority_escalate".to_string()],
            )
        } else if basis.iter().any(|b| b == "authority_allow") {
            (
                CriterionStanding::Satisfied,
                vec!["authority_allow".to_string()],
            )
        } else {
            (CriterionStanding::Unavailable, Vec::new())
        };
        checks.push(CriterionCheck {
            criterion: "require_approval".into(),
            expected: "approval recorded before execution".into(),
            standing,
            basis: matched,
        });
    }

    if let Some(evaluator) = &criteria.evaluator {
        // §10.6: a score is evidence, never standing. It gets `Recorded`, not
        // a pass/fail — the whole point of the distinction.
        let matched: Vec<String> = basis
            .iter()
            .filter(|b| b.starts_with(&format!("evaluator_score:{evaluator}=")))
            .cloned()
            .collect();
        checks.push(CriterionCheck {
            criterion: "evaluator".into(),
            expected: evaluator.clone(),
            standing: if matched.is_empty() {
                CriterionStanding::Unavailable
            } else {
                CriterionStanding::Recorded
            },
            basis: matched,
        });
    }

    if let Some(min_ratio) = criteria.min_pass_ratio {
        let matched: Vec<String> = basis
            .iter()
            .filter(|b| b.starts_with("batch_"))
            .cloned()
            .collect();
        let standing = if basis.iter().any(|b| b == "batch_below_threshold") {
            CriterionStanding::Unsatisfied
        } else if matched.is_empty() {
            CriterionStanding::Unavailable
        } else {
            CriterionStanding::Satisfied
        };
        checks.push(CriterionCheck {
            criterion: "batch".into(),
            expected: format!("pass ratio >= {min_ratio}"),
            standing,
            basis: matched,
        });
    }

    checks
}

/// Locate this run's plan item in the run-local `plan.json`.
fn plan_item<'a>(
    plan: &'a CasePlan,
    plan_item_id: Option<&str>,
) -> Option<&'a sea_forge_core::types::PlanItem> {
    match plan_item_id {
        // A run directory holds the plan it executed; when the trace names an
        // item, that is the authoritative selector.
        Some(id) => plan.items.iter().find(|i| i.plan_item_id == id),
        // A single-item plan is unambiguous. More than one item with no trace
        // to disambiguate is genuinely unknown — return nothing rather than
        // guessing which item's criteria to show.
        None if plan.items.len() == 1 => plan.items.first(),
        None => None,
    }
}

/// List every readable run directory, newest first.
pub fn list(root: &Path, case_id: Option<&str>) -> RunListResult {
    let index = case_index(root);
    let mut runs = Vec::new();
    let mut unreadable = Vec::new();

    let Ok(entries) = std::fs::read_dir(root.join("runs")) else {
        // No `runs/` directory yet is the normal state of a fresh cell.
        return RunListResult::default();
    };

    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Some(run_id) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        let owner = index.get(&run_id).cloned();

        // Scoping to a case means "runs this case claims" — a run the case does
        // not list is not this case's, even if it sits in the same cell.
        if let Some(wanted) = case_id {
            if owner.as_deref() != Some(wanted) {
                continue;
            }
        }

        let dir = entry.path();
        let events: Vec<TraceEvent> = read_jsonl(&dir.join("trace.jsonl"));
        let settlement: Option<SettlementEvent> = read_json(&dir.join("settlement.json"));

        // A run directory with no readable trace *and* no settlement has
        // nothing to project. Report it rather than rendering an empty row that
        // looks like a run that simply has not started.
        if events.is_empty() && settlement.is_none() {
            unreadable.push(run_id);
            continue;
        }

        let fold = fold_trace(&events);
        let evidence_count = read_jsonl::<EvidenceRecord>(&dir.join("evidence.jsonl")).len();

        runs.push(RunSummary {
            run_id,
            case_id: owner,
            plan_item_id: fold.plan_item_id,
            execution: fold.execution,
            settlement: settlement
                .as_ref()
                .map(settlement_standing)
                .unwrap_or(SettlementStanding::Unsettled),
            started_at: fold.started_at,
            finished_at: fold.finished_at,
            evidence_count,
        });
    }

    // Runs are ULID-prefixed and monotonic, but `started_at` is what an
    // operator reasons about; break ties by id for a total order so rows do not
    // reshuffle between polls.
    runs.sort_by(|a, b| {
        b.started_at
            .cmp(&a.started_at)
            .then_with(|| b.run_id.cmp(&a.run_id))
    });
    unreadable.sort();

    RunListResult { runs, unreadable }
}

/// Project one run's linked record from its committed files.
pub fn get(root: &Path, run_id: &str) -> Result<RunRecord, RunViewError> {
    let dir = run_dir(root, run_id).ok_or_else(|| RunViewError::NotFound(run_id.to_string()))?;
    if !dir.is_dir() {
        return Err(RunViewError::NotFound(run_id.to_string()));
    }

    let events: Vec<TraceEvent> = read_jsonl(&dir.join("trace.jsonl"));
    let settlement: Option<SettlementEvent> = read_json(&dir.join("settlement.json"));
    let plan: Option<CasePlan> = read_json(&dir.join("plan.json"));
    let criteria_record: Option<SettlementCriteriaRecord> = read_json(&dir.join("criteria.json"));
    let authority: Option<AuthorityDecision> = read_json(&dir.join("authority.json"));

    // A directory that exists but holds none of the canonical records is a
    // half-created run, not a run we can project. That is an integrity signal,
    // distinct from `not_found`.
    if events.is_empty() && settlement.is_none() && plan.is_none() {
        return Err(RunViewError::Unreadable(format!(
            "run {run_id} directory exists but holds no readable trace, plan, or settlement record"
        )));
    }

    let fold = fold_trace(&events);
    let item = plan
        .as_ref()
        .and_then(|p| plan_item(p, fold.plan_item_id.as_deref()));

    // The criteria record is the immutable, attributable form; the plan item's
    // inline criteria are the fallback for runs committed before criteria
    // records existed. Prefer the record — it is the one with provenance.
    let criteria = criteria_record
        .as_ref()
        .map(|record| record.criteria.clone())
        .or_else(|| item.map(|i| i.settlement_criteria.clone()));

    let records = RUN_RECORD_FILES
        .iter()
        .map(|name| {
            let meta = std::fs::metadata(dir.join(name)).ok();
            RecordPresence {
                record: (*name).to_string(),
                present: meta.is_some(),
                bytes: meta.map(|m| m.len()),
            }
        })
        .collect();

    Ok(RunRecord {
        run_id: run_id.to_string(),
        case_id: case_index(root).get(run_id).cloned(),
        plan_item_id: fold.plan_item_id.clone(),
        plan_item_name: item.map(|i| i.name.clone()),
        item_kind: item.map(|i| enum_str(&i.item_kind)),
        execution: fold.execution,
        settlement: settlement
            .as_ref()
            .map(settlement_standing)
            .unwrap_or(SettlementStanding::Unsettled),
        started_at: fold.started_at,
        finished_at: fold.finished_at,
        termination: fold.termination,
        authority: authority.as_ref().map(|decision| AuthorityProjection {
            decision_id: decision.decision_id.clone(),
            verdict: enum_str(&decision.verdict),
            normalized_disposition: enum_str(&decision.normalized_disposition),
            reason: decision.reason.clone(),
            reason_codes: decision.reason_codes.clone(),
            matched_rule: decision.matched_rule.clone(),
            policy_refs: decision.policy_refs.clone(),
            required_next_steps: decision.required_next_steps.clone(),
            decided_at: decision.decided_at.clone(),
        }),
        criteria_ref: criteria_record
            .as_ref()
            .map(|r| r.criteria_id.clone())
            .or_else(|| settlement.as_ref().and_then(|s| s.criteria_ref.clone())),
        criteria: criteria
            .as_ref()
            .map(|c| pair_criteria(c, settlement.as_ref()))
            .unwrap_or_default(),
        settlement_record: settlement.as_ref().map(|event| SettlementDetail {
            settlement_id: event.settlement_id.clone(),
            status: settlement_standing(event),
            basis: event.basis.clone(),
            review_required: event.review_required,
            settled_at: event.settled_at.clone(),
            criteria_ref: event.criteria_ref.clone(),
        }),
        declarations: read_jsonl::<SettlementDeclaration>(&dir.join("declarations.jsonl"))
            .iter()
            .map(|d| DeclarationRow {
                declaration_id: d.declaration_id.clone(),
                status: enum_str(&d.status),
                strength: enum_str(&d.strength),
                qualifies_for_capability: d.qualifies_for_capability,
                declarer_actor_id: d.declarer.actor_id.clone(),
                declarer_role: d.declarer.role.clone(),
                independent: d.independence.independent,
                reliability_weight: d.reliability.weight.clone(),
                issued_at: d.issued_at.clone(),
            })
            .collect(),
        evidence: read_jsonl::<EvidenceRecord>(&dir.join("evidence.jsonl"))
            .iter()
            .map(|e| EvidenceRow {
                evidence_id: e.evidence_id.clone(),
                kind: enum_str(&e.kind),
                uri: e.uri.clone(),
                sha256: e.sha256.clone(),
                source_event_id: e.source_event_id.clone(),
            })
            .collect(),
        trace: events
            .iter()
            .map(|e| TraceRow {
                event_id: e.event_id.clone(),
                kind: enum_str(&e.kind),
                plan_item_id: e.plan_item_id.clone(),
                actor_id: e.actor_id.clone(),
                timestamp: e.timestamp.clone(),
                payload: e.payload.clone(),
            })
            .collect(),
        records,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_id_cannot_traverse_out_of_the_runs_directory() {
        assert!(!valid_run_id("../../etc/passwd"));
        assert!(!valid_run_id("a/b"));
        assert!(!valid_run_id(""));
        assert!(valid_run_id("run-01JABCDEF"));
    }

    #[test]
    fn listing_a_cell_with_no_runs_directory_is_an_empty_list_not_an_error() {
        let root = tempfile::tempdir().unwrap();
        let result = list(root.path(), None);
        assert!(result.runs.is_empty());
        assert!(result.unreadable.is_empty());
    }

    #[test]
    fn a_missing_run_is_not_found_rather_than_unreadable() {
        let root = tempfile::tempdir().unwrap();
        let error = get(root.path(), "run-nope").unwrap_err();
        assert_eq!(error.class(), "not_found");
    }

    /// The join reports what the settlement said — and says nothing when the
    /// settlement did not. An unsettled run must never render as passing.
    #[test]
    fn a_criterion_with_no_settlement_is_unavailable_not_satisfied() {
        let criteria = SettlementCriteria {
            require_exit_zero: true,
            ..Default::default()
        };
        let checks = pair_criteria(&criteria, None);
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].standing, CriterionStanding::Unavailable);
        assert!(checks[0].basis.is_empty());
    }

    #[test]
    fn a_criterion_is_decided_by_the_settlements_own_basis_token() {
        let criteria = SettlementCriteria {
            require_exit_zero: true,
            required_artifacts: vec!["out/report.md".into()],
            ..Default::default()
        };
        let settled = SettlementEvent {
            version: "0.1".into(),
            settlement_id: "set_01".into(),
            run_id: "run-1".into(),
            status: sea_forge_core::types::SettlementStatus::Rejected,
            basis: vec![
                "authority_allow".into(),
                "exit_zero".into(),
                "required_artifact_missing:out/report.md".into(),
            ],
            review_required: false,
            settled_at: "2026-01-01T00:00:00Z".into(),
            criteria_ref: None,
        };
        let checks = pair_criteria(&criteria, Some(&settled));
        assert_eq!(checks[0].standing, CriterionStanding::Satisfied);
        assert_eq!(checks[1].standing, CriterionStanding::Unsatisfied);
        assert_eq!(
            checks[1].basis,
            vec!["required_artifact_missing:out/report.md"]
        );
    }

    /// §10.6 — an evaluator score is evidence, never standing. It must not
    /// borrow the pass/fail vocabulary even when the score is present.
    #[test]
    fn an_evaluator_score_is_recorded_never_satisfied() {
        let criteria = SettlementCriteria {
            evaluator: Some("py.pytest".into()),
            ..Default::default()
        };
        let settled = SettlementEvent {
            version: "0.1".into(),
            settlement_id: "set_01".into(),
            run_id: "run-1".into(),
            status: sea_forge_core::types::SettlementStatus::Accepted,
            basis: vec!["evaluator_score:py.pytest=1".into()],
            review_required: false,
            settled_at: "2026-01-01T00:00:00Z".into(),
            criteria_ref: None,
        };
        let checks = pair_criteria(&criteria, Some(&settled));
        assert_eq!(checks[0].standing, CriterionStanding::Recorded);
    }
}
