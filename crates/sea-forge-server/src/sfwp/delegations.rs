//! `delegation.list` — the SFWP **inspect** roster of agent delegations, live
//! and finished (Task 11, ADR-003 additive; epic 9.7).
//!
//! # The gap this closes
//!
//! `cancel_delegation` has been a verb since M12, and the Tauri host has
//! carried `SfwpCommand::CancelDelegation` since Task 3 — but nothing could
//! *enumerate* what there was to cancel. That is the same shape the
//! `approval.decide` gap had before Task 7: a reachable capability whose
//! targets were undiscoverable, which makes it unusable rather than merely
//! inconvenient. Epic 9.7 asks for exactly the missing half — see the roster,
//! cancel *one* delegation without touching its siblings.
//!
//! # Two sources, and why the seam between them is reported rather than hidden
//!
//! A delegation's state lives in two places that cannot be merged:
//!
//! - **In the server's memory**, while it runs: `ServerState::delegations` holds
//!   a cancel flag per live run. Nothing about this is durable.
//! - **On disk**, once it terminates: `runs/<run_id>/` gains `settlement.json`
//!   and `transcript-evidence.json`.
//!
//! A run that has neither — a delegation directory with no settlement and no
//! live handle — is the interesting case. It means the server restarted (or
//! died) while the episode was in flight. The kernel cannot say whether that
//! episode finished, so neither does this projection: it reports
//! [`DelegationStanding::Unresolved`] and stops. Calling it `cancelled` would
//! invent a cancellation nobody requested; calling it `failed` would invent a
//! settlement nobody recorded. Absence is reported, never inferred.
//!
//! # Why live turn and token counts are absent
//!
//! Epic 9.7 also asks for bounded turn, token, and streaming state *during* the
//! dialogue. The kernel does not currently expose it: `DelegationHandle` carries
//! only the case id and two atomics, and the per-turn loop publishes no event.
//! `turns_used`/`tool_calls` therefore appear only once
//! `transcript-evidence.json` is written — i.e. after termination. The fields
//! are `Option`, and a running delegation leaves them absent rather than
//! reporting `0`, which would be a false claim that no turn has been taken. The
//! kernel-side gap is logged in `.agents/OBSERVED_DEBT.md`.
//!
//! # Standing is not settlement
//!
//! [`DelegationStanding`] answers "where in its lifecycle is this episode?".
//! Whether the work was *accepted* is a separate field carrying the settlement's
//! own word. A delegation can be `settled` and rejected; it can be `settled`
//! after having been cancelled. Folding the two would force one of the facts to
//! be dropped — the same rule `sfwp::assets` follows for standing vs blocking,
//! and the same disjointness `run_views` keeps between execution and settlement.

use std::collections::BTreeMap;
use std::path::Path;

use schemars::JsonSchema;
use sea_forge_core::types::{CasePlan, Operation, SettlementEvent, TranscriptEvidence};
use serde::{Deserialize, Serialize};

use crate::sfwp::run_views::{case_index, read_json, run_dirs};

/// What the server knows about a delegation right now, live-only.
///
/// Deliberately a plain snapshot rather than a borrow of `ServerState`: this
/// module projects data, and taking the server's lock inside a projection would
/// make the read order part of the contract.
#[derive(Clone, Debug)]
pub struct LiveDelegation {
    pub case_id: String,
    /// A cancellation has been requested and recorded; the episode has not yet
    /// observed it or has not yet terminated.
    pub cancellation_requested: bool,
}

/// Where a delegation sits in its lifecycle.
///
/// Closed on purpose, and *not* an execution or settlement vocabulary — those
/// already exist in `run_views` and mean different things. A fifth variant here
/// would mean the kernel grew a fifth genuinely distinguishable state, not that
/// the UI wanted a new label.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DelegationStanding {
    /// The server holds a live handle and no cancellation has been requested.
    Active,
    /// A cancellation has been recorded; the episode has not yet settled. This
    /// is a *request*, not an outcome — the episode terminates on its own terms
    /// and its settlement records what actually happened.
    CancellationRequested,
    /// A settlement record exists on disk. Whether it was accepted is a
    /// separate question, answered by `settlement`.
    Settled,
    /// A delegation run exists with no settlement and no live handle. The
    /// server cannot say what happened to it — typically it was in flight when
    /// the server stopped. Never to be rendered as cancelled or failed.
    Unresolved,
}

/// One delegation in the roster.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DelegationRow {
    pub run_id: String,
    /// The case that claims this run, from the same index `run.list` uses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,
    /// The endpoint the plan named. Absent only when `plan.json` is unreadable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_ref: Option<String>,
    pub standing: DelegationStanding,
    /// True only when this exact run can be cancelled *now*: the server holds a
    /// live handle and no cancellation has already been recorded. Per-run by
    /// construction — cancelling one delegation cannot reach a sibling, because
    /// the cancel flag lives on that run's own handle (epic 9.7).
    pub cancellable: bool,
    /// The settlement's own word (`accepted`/`rejected`/…), present only once a
    /// settlement exists. Absent means unsettled, never "not accepted".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settlement: Option<String>,
    /// How the dialogue ended, from the transcript evidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub termination: Option<String>,
    /// Turns actually taken. Absent while the delegation is still running — the
    /// kernel publishes no per-turn state, and `0` would be a false claim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turns_used: Option<u32>,
    /// The turn cap the plan bound this episode to, so a turn count can be read
    /// against its bound rather than as a bare number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u32>,
    /// Token budget from the plan, if one was set. Absent means no token cap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_budget: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settled_at: Option<String>,
}

/// `delegation.list` result body.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct DelegationListResult {
    /// Live delegations first, then the rest newest-first — an operator looking
    /// for something to intervene in should not have to scroll past history.
    pub delegations: Vec<DelegationRow>,
    /// Run directories that name an agent task but whose plan could not be read.
    /// Reported rather than dropped: an unreadable delegation is an integrity
    /// signal, not an absent one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unreadable: Vec<String>,
}

/// The plan-side facts about an agent task, read once from `plan.json`.
struct PlannedTask {
    endpoint_ref: String,
    max_turns: u32,
    token_budget: Option<u64>,
}

/// Project the delegation roster. Creates nothing and mutates nothing.
///
/// `live` is a snapshot of the server's in-memory handles, keyed by run id.
pub fn list(root: &Path, live: &BTreeMap<String, LiveDelegation>) -> DelegationListResult {
    let index = case_index(root);
    let mut delegations = Vec::new();
    let mut unreadable = Vec::new();
    let mut seen: Vec<String> = Vec::new();

    for (run_id, dir) in run_dirs(root) {
        let Some(plan) = read_json::<CasePlan>(&dir.join("plan.json")) else {
            // A run directory with no readable plan is only *this* view's
            // problem when it is a delegation, and an unreadable plan is
            // precisely what makes that undecidable. `run.list` already reports
            // unreadable runs generally, so staying quiet here would not lose
            // the signal — but a live handle for this id proves it *is* a
            // delegation, and then the silence would.
            if live.contains_key(&run_id) {
                unreadable.push(run_id);
            }
            continue;
        };
        let Some(task) = planned_task(&plan) else {
            continue;
        };

        seen.push(run_id.clone());
        let settlement: Option<SettlementEvent> = read_json(&dir.join("settlement.json"));
        let evidence: Option<TranscriptEvidence> = read_json(&dir.join("transcript-evidence.json"));
        let handle = live.get(&run_id);

        delegations.push(DelegationRow {
            standing: standing_of(handle, settlement.is_some()),
            cancellable: handle.is_some_and(|held| !held.cancellation_requested),
            case_id: index
                .get(&run_id)
                .cloned()
                .or_else(|| handle.map(|held| held.case_id.clone())),
            endpoint_ref: Some(task.endpoint_ref),
            settlement: settlement.as_ref().map(|event| enum_word(&event.status)),
            settled_at: settlement.as_ref().map(|event| event.settled_at.clone()),
            termination: evidence
                .as_ref()
                .map(|record| enum_word(&record.termination)),
            turns_used: evidence.as_ref().map(|record| record.turns_used),
            tool_calls: evidence.as_ref().map(|record| record.summary.tool_calls),
            transcript_sha256: evidence
                .as_ref()
                .map(|record| record.transcript_sha256.clone()),
            max_turns: Some(task.max_turns),
            token_budget: task.token_budget,
            run_id,
        });
    }

    // A live handle whose run directory has not appeared yet is still a real,
    // cancellable delegation: `delegate` registers the handle before it commits
    // the plan. Dropping it would hide exactly the youngest episode — the one an
    // operator is most likely to want to stop.
    for (run_id, held) in live {
        if seen.contains(run_id) {
            continue;
        }
        delegations.push(DelegationRow {
            run_id: run_id.clone(),
            case_id: Some(held.case_id.clone()),
            endpoint_ref: None,
            standing: standing_of(Some(held), false),
            cancellable: !held.cancellation_requested,
            settlement: None,
            termination: None,
            turns_used: None,
            max_turns: None,
            token_budget: None,
            tool_calls: None,
            transcript_sha256: None,
            settled_at: None,
        });
    }

    // Live first, then newest-first. Run ids are ULID-prefixed and monotonic, so
    // a reverse id sort is a stable recency order without needing a timestamp
    // every row is guaranteed to have.
    delegations.sort_by(|a, b| {
        let live_first = b.cancellable.cmp(&a.cancellable);
        let unsettled_first = a.settled_at.is_some().cmp(&b.settled_at.is_some());
        live_first
            .then(unsettled_first)
            .then_with(|| b.run_id.cmp(&a.run_id))
    });
    unreadable.sort();

    DelegationListResult {
        delegations,
        unreadable,
    }
}

/// The first agent task in a plan, with the bounds it was committed under.
fn planned_task(plan: &CasePlan) -> Option<PlannedTask> {
    plan.items
        .iter()
        .flat_map(|item| &item.operations)
        .find_map(|operation| match operation {
            Operation::AgentTask {
                endpoint_ref,
                max_turns,
                token_budget,
                ..
            } => Some(PlannedTask {
                endpoint_ref: endpoint_ref.clone(),
                max_turns: *max_turns,
                token_budget: *token_budget,
            }),
            _ => None,
        })
}

/// The lifecycle fold. A settlement on disk always wins: it is durable truth,
/// whereas a live handle is only this process's memory of an episode it may
/// already have finished.
fn standing_of(handle: Option<&LiveDelegation>, settled: bool) -> DelegationStanding {
    if settled {
        return DelegationStanding::Settled;
    }
    match handle {
        Some(held) if held.cancellation_requested => DelegationStanding::CancellationRequested,
        Some(_) => DelegationStanding::Active,
        None => DelegationStanding::Unresolved,
    }
}

/// A record enum's own wire word, taken from serde rather than re-spelled so
/// this projection cannot drift from the record it is reporting.
fn enum_word<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unknown".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_forge_core::types::{DelegationTermination, SettlementStatus};

    fn held(cancellation_requested: bool) -> LiveDelegation {
        LiveDelegation {
            case_id: "case-1".into(),
            cancellation_requested,
        }
    }

    /// The fold that the whole view rests on. In particular an in-flight run the
    /// server has forgotten must not collapse into a terminal-looking state.
    #[test]
    fn standing_separates_forgotten_runs_from_cancelled_and_settled_ones() {
        assert_eq!(
            standing_of(Some(&held(false)), false),
            DelegationStanding::Active
        );
        assert_eq!(
            standing_of(Some(&held(true)), false),
            DelegationStanding::CancellationRequested
        );
        // Durable truth outranks this process's memory, cancellation or not.
        assert_eq!(
            standing_of(Some(&held(true)), true),
            DelegationStanding::Settled
        );
        assert_eq!(
            standing_of(Some(&held(false)), true),
            DelegationStanding::Settled
        );
        // No handle, no settlement: the server does not know. It must not guess.
        assert_eq!(standing_of(None, false), DelegationStanding::Unresolved);
        assert_eq!(standing_of(None, true), DelegationStanding::Settled);
    }

    /// The words must be the records' own, or the roster would report a
    /// termination the transcript never used.
    #[test]
    fn enum_words_match_the_records_they_report() {
        assert_eq!(enum_word(&SettlementStatus::Accepted), "accepted");
        assert_eq!(enum_word(&SettlementStatus::Rejected), "rejected");
        assert_eq!(enum_word(&DelegationTermination::Cancelled), "cancelled");
        assert_eq!(
            enum_word(&DelegationTermination::TurnCapExceeded),
            "turn_cap_exceeded"
        );
    }
}
