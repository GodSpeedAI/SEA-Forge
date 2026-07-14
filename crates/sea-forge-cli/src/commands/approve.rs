use chrono::Utc;
use sea_forge_core::{
    errors::ForgeError,
    types::{ApprovalRequest, ApprovalStatus},
};
use std::path::Path;

use crate::approvals;

pub struct ApproveOptions<'a> {
    pub root: &'a Path,
    pub case_id: &'a str,
    pub approval_id: &'a str,
    pub actor: &'a str,
    pub note: Option<&'a str>,
    #[allow(dead_code)]
    pub policy: Option<&'a Path>,
}

pub fn approve(opts: ApproveOptions) -> Result<u8, ForgeError> {
    resolve(
        opts.root,
        opts.case_id,
        opts.approval_id,
        opts.actor,
        opts.note,
        true,
    )
}

pub fn reject(opts: ApproveOptions) -> Result<u8, ForgeError> {
    resolve(
        opts.root,
        opts.case_id,
        opts.approval_id,
        opts.actor,
        opts.note,
        false,
    )
}

fn resolve(
    root: &Path,
    case_id: &str,
    approval_id: &str,
    actor: &str,
    note: Option<&str>,
    approved: bool,
) -> Result<u8, ForgeError> {
    let latest = approvals::latest_status(root, approval_id)?
        .ok_or_else(|| ForgeError::Input(format!("approval {approval_id} not found")))?;
    if latest.case_id != case_id {
        return Err(ForgeError::Input(format!(
            "approval {approval_id} does not belong to case {case_id}"
        )));
    }
    if latest.status != ApprovalStatus::Pending {
        return Err(ForgeError::Input(format!(
            "approval {approval_id} is already {} — MUST NOT be re-resolved",
            serde_json::to_string(&latest.status)
                .unwrap_or_default()
                .trim_matches('"')
        )));
    }
    // Check TTL expiry before resolving.
    let now = Utc::now().to_rfc3339();
    if latest.expires_at.as_str() <= now.as_str() {
        let expired = ApprovalRequest {
            status: ApprovalStatus::Expired,
            resolved_by: None,
            resolved_at: Some(now.clone()),
            note: Some("ttl_expired".into()),
            ..latest
        };
        approvals::append(root, &expired)?;
        return Err(ForgeError::Input(format!(
            "approval {approval_id} expired before resolution"
        )));
    }
    // ponytail: unauthorized-approver check lands when policy schema gains
    // approve rules in Task 11; the authority-checked gate already exists
    // for case operations and is reused when the server dispatches.
    let resolved = ApprovalRequest {
        status: if approved {
            ApprovalStatus::Approved
        } else {
            ApprovalStatus::Rejected
        },
        resolved_by: Some(actor.into()),
        resolved_at: Some(now),
        note: note.map(str::to_owned),
        ..latest
    };
    approvals::append(root, &resolved)?;
    println!("approval_id={approval_id}");
    println!("status={:?}", resolved.status);
    println!("resolved_by={actor}");
    Ok(0)
}
