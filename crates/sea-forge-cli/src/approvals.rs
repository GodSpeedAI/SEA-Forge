use sea_forge_core::{
    errors::ForgeError,
    types::{ApprovalRequest, ApprovalStatus},
};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

fn approvals_path(root: &Path) -> Result<std::path::PathBuf, ForgeError> {
    let path = root.join("approvals.jsonl");
    Ok(path)
}

/// Append an approval request to `.sea-forge/approvals.jsonl`.
#[allow(dead_code)]
pub fn append(root: &Path, request: &ApprovalRequest) -> Result<(), ForgeError> {
    let path = approvals_path(root)?;
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

/// Read all approvals. Later lines with the same approval_id win.
#[allow(dead_code)]
pub fn load_all(root: &Path) -> Result<Vec<ApprovalRequest>, ForgeError> {
    let path = approvals_path(root)?;
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

/// Return the latest status for a given approval_id.
pub fn latest_status(
    root: &Path,
    approval_id: &str,
) -> Result<Option<ApprovalRequest>, ForgeError> {
    let all = load_all(root)?;
    Ok(all.into_iter().rfind(|a| a.approval_id == approval_id))
}

/// Find pending approvals for a given case.
#[allow(dead_code)]
pub fn pending_for_case(root: &Path, case_id: &str) -> Result<Vec<ApprovalRequest>, ForgeError> {
    let all = load_all(root)?;
    let mut seen = std::collections::BTreeMap::new();
    for req in all {
        if req.case_id == case_id {
            seen.insert(req.approval_id.clone(), req);
        }
    }
    Ok(seen
        .into_values()
        .filter(|r| r.status == ApprovalStatus::Pending)
        .collect())
}

/// Check if a pending approval has expired by comparing expires_at to now.
#[allow(dead_code)]
pub fn check_expiry(root: &Path, approval_id: &str, now: &str) -> Result<Option<bool>, ForgeError> {
    let latest = latest_status(root, approval_id)?;
    match latest {
        Some(req) if req.status == ApprovalStatus::Pending => {
            Ok(Some(req.expires_at.as_str() <= now))
        }
        _ => Ok(None),
    }
}
