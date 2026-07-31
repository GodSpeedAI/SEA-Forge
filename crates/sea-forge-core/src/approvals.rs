//! The approvals journal: `<root>/approvals.jsonl`.
//!
//! # Why this lives in core
//!
//! The journal is append-only and **latest line wins per `approval_id`** — a
//! decision never edits the request it resolves, it appends a new record with
//! the same id (epic invariant 8: history is appended, never rewritten). That
//! "latest wins" fold is the single rule that turns the raw journal into
//! current approval standing.
//!
//! Two callers need that standing: the CLI (which writes decisions) and the
//! SFWP server (which lists pending approvals for the workbench inbox). If each
//! implemented the fold, the two could disagree about whether an approval is
//! still open — and the one that says "open" would let an operator act on an
//! already-decided request. One rule, one owner, so both read the same answer.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::errors::ForgeError;
use crate::types::{ApprovalRequest, ApprovalStatus};

/// The journal path for a cell root.
pub fn approvals_path(root: &Path) -> PathBuf {
    root.join("approvals.jsonl")
}

/// Append an approval record. Appending is the only write: a decision adds a
/// record carrying the same `approval_id`, it does not rewrite the request.
pub fn append(root: &Path, request: &ApprovalRequest) -> Result<(), ForgeError> {
    let path = approvals_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| ForgeError::io("create approvals parent", e))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| ForgeError::io("open approvals.jsonl", e))?;
    serde_json::to_writer(&mut file, request)?;
    file.write_all(b"\n")
        .map_err(|e| ForgeError::io("flush approval", e))?;
    Ok(())
}

/// Read every record in journal order. A malformed line is a hard error here:
/// silently skipping one could drop a *decision*, leaving a resolved approval
/// looking open — the one misread this file must never produce.
pub fn load_all(root: &Path) -> Result<Vec<ApprovalRequest>, ForgeError> {
    let path = approvals_path(root);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&path).map_err(|e| ForgeError::io("read approvals.jsonl", e))?;
    let mut all: Vec<ApprovalRequest> = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let request: ApprovalRequest = serde_json::from_str(line)
            .map_err(|e| ForgeError::Serialization(format!("parse approval line: {e}")))?;
        all.push(request);
    }
    Ok(all)
}

/// Current standing of every approval: the last record written for each one.
/// This is the fold every other query in this module is built from.
///
/// The key is `(case_id, approval_id)`, **not `approval_id` alone**. Approval
/// ids are per-case ordinals — every case's first escalation is `apr_0001` — so
/// folding on the id by itself makes unrelated cases collide. The observed
/// consequence, on a cell holding two cases: resolving `apr_0001` in one case
/// appended an `approved` record that superseded the still-pending `apr_0001`
/// of the other, which then vanished from the inbox with no lawful way to
/// reach it again. The work was stranded, and nothing reported a problem.
///
/// Every test here used a single case, which is why the collision was invisible
/// until a cell held two.
pub fn latest_by_id(root: &Path) -> Result<Vec<ApprovalRequest>, ForgeError> {
    let all = load_all(root)?;
    let mut seen: std::collections::BTreeMap<(String, String), ApprovalRequest> =
        std::collections::BTreeMap::new();
    for request in all {
        seen.insert(
            (request.case_id.clone(), request.approval_id.clone()),
            request,
        );
    }
    Ok(seen.into_values().collect())
}

/// The latest record for one approval, if the journal knows it.
///
/// Scoped by case for the same reason as [`latest_by_id`]: an `approval_id`
/// alone does not identify an approval.
pub fn latest_status(
    root: &Path,
    case_id: &str,
    approval_id: &str,
) -> Result<Option<ApprovalRequest>, ForgeError> {
    let all = load_all(root)?;
    Ok(all
        .into_iter()
        .rfind(|a| a.case_id == case_id && a.approval_id == approval_id))
}

/// Every approval still awaiting a decision, across all cases.
pub fn pending(root: &Path) -> Result<Vec<ApprovalRequest>, ForgeError> {
    Ok(latest_by_id(root)?
        .into_iter()
        .filter(|r| r.status == ApprovalStatus::Pending)
        .collect())
}

/// Every approval still awaiting a decision for one case.
pub fn pending_for_case(root: &Path, case_id: &str) -> Result<Vec<ApprovalRequest>, ForgeError> {
    Ok(pending(root)?
        .into_iter()
        .filter(|r| r.case_id == case_id)
        .collect())
}

/// Whether a pending approval's window has closed as of `now` (RFC3339).
/// `None` means the approval is not pending, so expiry does not apply.
pub fn check_expiry(
    root: &Path,
    case_id: &str,
    approval_id: &str,
    now: &str,
) -> Result<Option<bool>, ForgeError> {
    match latest_status(root, case_id, approval_id)? {
        Some(request) if request.status == ApprovalStatus::Pending => {
            Ok(Some(request.expires_at.as_str() <= now))
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RECORD_VERSION;

    fn request(id: &str, status: ApprovalStatus) -> ApprovalRequest {
        in_case("case-1", id, status)
    }

    fn in_case(case_id: &str, id: &str, status: ApprovalStatus) -> ApprovalRequest {
        ApprovalRequest {
            version: RECORD_VERSION.into(),
            approval_id: id.into(),
            run_id: "run-1".into(),
            case_id: case_id.into(),
            decision_id: "dec-1".into(),
            plan_item_id: "item-1".into(),
            criteria_ref: None,
            criteria_sha256: None,
            criteria_record_hash: None,
            job_contract_ref: None,
            requested_at: "2026-07-26T00:00:00Z".into(),
            expires_at: "2026-07-27T00:00:00Z".into(),
            status,
            resolved_by: None,
            resolved_at: None,
            note: None,
        }
    }

    /// The whole point of the fold: an approval that was decided must not still
    /// read as pending, or an operator would be offered an action on it twice.
    #[test]
    fn a_later_decision_supersedes_the_original_request() {
        let root = tempfile::tempdir().unwrap();
        append(root.path(), &request("ap-1", ApprovalStatus::Pending)).unwrap();
        append(root.path(), &request("ap-2", ApprovalStatus::Pending)).unwrap();
        append(root.path(), &request("ap-1", ApprovalStatus::Approved)).unwrap();

        let pending = pending(root.path()).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].approval_id, "ap-2");
    }

    #[test]
    fn a_cell_with_no_journal_has_no_approvals_rather_than_an_error() {
        let root = tempfile::tempdir().unwrap();
        assert!(pending(root.path()).unwrap().is_empty());
    }

    /// Approval ids are per-case ordinals: every case's first escalation is
    /// `apr_0001`. Folding on the id alone made resolving one case's `apr_0001`
    /// supersede every other case's, stranding work that no longer appeared in
    /// any inbox.
    ///
    /// Found by driving a packaged cell holding two cases. Every other test in
    /// this module uses one case, which is exactly why it survived so long.
    #[test]
    fn resolving_one_case_does_not_clear_the_same_ordinal_in_another() {
        let root = tempfile::tempdir().unwrap();
        append(
            root.path(),
            &in_case("case-a", "apr_0001", ApprovalStatus::Pending),
        )
        .unwrap();
        append(
            root.path(),
            &in_case("case-b", "apr_0001", ApprovalStatus::Pending),
        )
        .unwrap();
        append(
            root.path(),
            &in_case("case-b", "apr_0001", ApprovalStatus::Approved),
        )
        .unwrap();

        let pending = pending(root.path()).unwrap();
        assert_eq!(
            pending.len(),
            1,
            "case-a's approval was lost when case-b's was resolved: {pending:?}"
        );
        assert_eq!(pending[0].case_id, "case-a");

        // And scoping to the case must agree with the unscoped list.
        assert_eq!(pending_for_case(root.path(), "case-a").unwrap().len(), 1);
        assert!(pending_for_case(root.path(), "case-b").unwrap().is_empty());
    }

    /// The same collision in the single-approval lookup: a status read for one
    /// case must not answer from another case's record.
    #[test]
    fn a_status_lookup_does_not_answer_from_another_case() {
        let root = tempfile::tempdir().unwrap();
        append(
            root.path(),
            &in_case("case-a", "apr_0001", ApprovalStatus::Pending),
        )
        .unwrap();
        append(
            root.path(),
            &in_case("case-b", "apr_0001", ApprovalStatus::Approved),
        )
        .unwrap();

        let a = latest_status(root.path(), "case-a", "apr_0001")
            .unwrap()
            .expect("case-a's approval is in the journal");
        assert_eq!(a.status, ApprovalStatus::Pending);

        let b = latest_status(root.path(), "case-b", "apr_0001")
            .unwrap()
            .expect("case-b's approval is in the journal");
        assert_eq!(b.status, ApprovalStatus::Approved);

        // Expiry follows the same scoping, or a resolved approval elsewhere
        // would answer for a pending one here.
        assert_eq!(
            check_expiry(root.path(), "case-a", "apr_0001", "2026-07-26T12:00:00Z").unwrap(),
            Some(false),
        );
        assert_eq!(
            check_expiry(root.path(), "case-b", "apr_0001", "2026-07-26T12:00:00Z").unwrap(),
            None,
            "a resolved approval has no expiry standing",
        );
    }
}
