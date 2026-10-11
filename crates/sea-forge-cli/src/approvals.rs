//! Approvals journal access for the CLI.
//!
//! The journal's format and its "latest record per `approval_id` wins" fold are
//! owned by [`sea_forge_core::approvals`], because the SFWP server reads the
//! same file to list pending approvals for the workbench inbox. Two independent
//! folds could disagree about whether an approval is still open, and the one
//! that said "open" would offer an operator an action on an already-decided
//! request. This module is the CLI's view onto that single owner.

#[allow(unused_imports)]
pub use sea_forge_core::approvals::{
    append, check_expiry, latest_status, load_all, pending_for_case,
};
