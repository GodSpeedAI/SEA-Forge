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

/// Current standing of every approval: the last record written for each id.
/// This is the fold every other query in this module is built from.
pub fn latest_by_id(root: &Path) -> Result<Vec<ApprovalRequest>, ForgeError> {
    let all = load_all(root)?;
    let mut seen: std::collections::BTreeMap<String, ApprovalRequest> =
        std::collections::BTreeMap::new();
    for request in all {
        seen.insert(request.approval_id.clone(), request);
    }
    Ok(seen.into_values().collect())
}

/// The latest record for one approval id, if the journal knows it.
pub fn latest_status(
    root: &Path,
    approval_id: &str,
) -> Result<Option<ApprovalRequest>, ForgeError> {
    let all = load_all(root)?;
    Ok(all.into_iter().rfind(|a| a.approval_id == approval_id))
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
pub fn check_expiry(root: &Path, approval_id: &str, now: &str) -> Result<Option<bool>, ForgeError> {
    match latest_status(root, approval_id)? {
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
        ApprovalRequest {
            version: RECORD_VERSION.into(),
            approval_id: id.into(),
            run_id: "run-1".into(),
            case_id: "case-1".into(),
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
}
