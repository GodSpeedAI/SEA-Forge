//! `approval.list` — an SFWP **inspect** method (Task 7, ADR-003 additive).
//!
//! # Why this method had to exist
//!
//! `Approve`/`Reject` (and the `approval.decide` envelope over them) have
//! always been reachable, but nothing in the protocol could *enumerate* a
//! pending approval. Both verbs require a `case_id` and an `approval_id`, so an
//! approver working from the workbench had no lawful way to obtain the very
//! identifiers the command demands — the capability existed but the path to it
//! did not. An affordance is a reachable path, not merely a present feature.
//!
//! # Contract
//!
//! A read-only projection over `<root>/approvals.jsonl` via
//! [`sea_forge_core::approvals`], which owns the append-only journal's
//! "latest record per id wins" fold. This module shapes that standing for the
//! wire; it does not re-derive it.
//!
//! Expiry is **reported, not enforced**. An approval whose window has closed is
//! still listed, flagged `expired: true`, because the decision to treat it as
//! expired belongs to the governance path that resolves it — not to a view. A
//! list that silently dropped expired approvals would leave an operator unable
//! to see why an item is stuck (epic invariant 7: a blocked state must expose
//! the next lawful path).

use std::path::Path;

use schemars::JsonSchema;
use sea_forge_core::types::ApprovalRequest;
use serde::{Deserialize, Serialize};

/// One pending approval, shaped for the inbox.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PendingApproval {
    pub approval_id: String,
    pub case_id: String,
    pub run_id: String,
    pub plan_item_id: String,
    pub decision_id: String,
    pub requested_at: String,
    pub expires_at: String,
    /// True when `expires_at` has passed. Reported so the approver sees *why*
    /// a decision may be refused, rather than the row vanishing.
    pub expired: bool,
    /// Reference to the settlement criteria this decision is judged against,
    /// when the request recorded one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criteria_ref: Option<String>,
    /// Content hash of those criteria — lets the UI prove the criteria being
    /// shown are the ones the approval was raised against.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criteria_sha256: Option<String>,
}

/// `approval.list` result body.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalListResult {
    /// Oldest request first: an approval inbox is a queue, and the thing that
    /// has waited longest is the thing most likely to be blocking work.
    pub approvals: Vec<PendingApproval>,
    /// Set when the journal itself could not be read. The list is then empty
    /// *because nothing could be determined*, which is different from an empty
    /// queue — the UI must be able to tell those apart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unreadable: Option<String>,
}

fn shape(request: ApprovalRequest, now: &str) -> PendingApproval {
    PendingApproval {
        expired: request.expires_at.as_str() <= now,
        approval_id: request.approval_id,
        case_id: request.case_id,
        run_id: request.run_id,
        plan_item_id: request.plan_item_id,
        decision_id: request.decision_id,
        requested_at: request.requested_at,
        expires_at: request.expires_at,
        criteria_ref: request.criteria_ref,
        criteria_sha256: request.criteria_sha256,
    }
}

/// List approvals still awaiting a decision, optionally scoped to one case.
pub fn list(root: &Path, case_id: Option<&str>) -> ApprovalListResult {
    let now = chrono::Utc::now().to_rfc3339();

    let pending = match sea_forge_core::approvals::pending(root) {
        Ok(pending) => pending,
        // A journal that exists but cannot be parsed is an integrity signal,
        // not an empty queue. Reporting it keeps the two distinguishable.
        Err(error) => {
            return ApprovalListResult {
                approvals: Vec::new(),
                unreadable: Some(error.to_string()),
            }
        }
    };

    let mut approvals: Vec<PendingApproval> = pending
        .into_iter()
        .filter(|request| case_id.is_none_or(|id| request.case_id == id))
        .map(|request| shape(request, &now))
        .collect();

    approvals.sort_by(|a, b| {
        a.requested_at
            .cmp(&b.requested_at)
            .then_with(|| a.approval_id.cmp(&b.approval_id))
    });

    ApprovalListResult {
        approvals,
        unreadable: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cell_with_no_journal_lists_nothing_without_claiming_a_read_failure() {
        let root = tempfile::tempdir().unwrap();
        let result = list(root.path(), None);
        assert!(result.approvals.is_empty());
        assert!(
            result.unreadable.is_none(),
            "no journal is an empty queue, not an unreadable one"
        );
    }

    /// The operator-visible half of the `(case_id, approval_id)` fold key.
    ///
    /// Approval ids are per-case ordinals, so a cell holding two cases has two
    /// `apr_0001`s. Resolving one used to empty the other out of the inbox —
    /// the request stayed committed in its case ledger, but nothing could
    /// enumerate it, so the only lawful path to the identifiers `approval.decide`
    /// demands was gone and the work was stranded.
    ///
    /// Caught by driving a packaged cell that had been seeded with two cases.
    #[test]
    fn one_cases_resolved_approval_does_not_empty_another_cases_inbox() {
        use sea_forge_core::types::{ApprovalRequest, ApprovalStatus};

        let root = tempfile::tempdir().unwrap();
        let record = |case: &str, status| ApprovalRequest {
            version: sea_forge_core::RECORD_VERSION.into(),
            approval_id: "apr_0001".into(),
            run_id: "run-1".into(),
            case_id: case.into(),
            decision_id: "dec-1".into(),
            plan_item_id: "item-1".into(),
            criteria_ref: None,
            criteria_sha256: None,
            criteria_record_hash: None,
            job_contract_ref: None,
            requested_at: "2026-07-26T00:00:00Z".into(),
            expires_at: "2126-07-27T00:00:00Z".into(),
            status,
            resolved_by: None,
            resolved_at: None,
            note: None,
        };
        let append = |r| sea_forge_core::approvals::append(root.path(), &r).unwrap();
        append(record("case-a", ApprovalStatus::Pending));
        append(record("case-b", ApprovalStatus::Pending));
        append(record("case-b", ApprovalStatus::Approved));

        let all = list(root.path(), None);
        assert_eq!(
            all.approvals.len(),
            1,
            "case-a's approval vanished from the inbox: {:?}",
            all.approvals
        );
        assert_eq!(all.approvals[0].case_id, "case-a");

        assert_eq!(list(root.path(), Some("case-a")).approvals.len(), 1);
        assert!(list(root.path(), Some("case-b")).approvals.is_empty());
    }

    #[test]
    fn an_unparseable_journal_is_reported_rather_than_read_as_an_empty_queue() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("approvals.jsonl"), "{not json\n").unwrap();
        let result = list(root.path(), None);
        assert!(result.approvals.is_empty());
        assert!(
            result.unreadable.is_some(),
            "a broken journal must not look like an empty inbox"
        );
    }
}
