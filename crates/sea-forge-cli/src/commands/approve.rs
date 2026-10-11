//! Thin CLI surface over the shared approval-resolution library.
//!
//! The resolution logic (ledger replay, SoD, criteria-binding cross-checks,
//! authority evaluation and the approvals journal write) lives in
//! `sea_forge_case_runner::case_ops::resolve_approval` so the SFWP server
//! resolves approvals through the same governed path in-process; these
//! wrappers render the outcome exactly as the CLI always has.

use sea_forge_core::errors::ForgeError;
use std::path::Path;

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
    resolve(&opts, true)
}

pub fn reject(opts: ApproveOptions) -> Result<u8, ForgeError> {
    resolve(&opts, false)
}

fn resolve(opts: &ApproveOptions, approved: bool) -> Result<u8, ForgeError> {
    let policy = opts
        .policy
        .ok_or_else(|| ForgeError::Input("approval policy is missing".into()))?;
    let resolved = sea_forge_case_runner::case_ops::resolve_approval(
        opts.root,
        policy,
        opts.case_id,
        opts.approval_id,
        opts.actor,
        opts.note,
        approved,
    )?;
    println!("approval_id={}", opts.approval_id);
    println!("status={:?}", resolved.status);
    println!("resolved_by={}", opts.actor);
    Ok(0)
}
