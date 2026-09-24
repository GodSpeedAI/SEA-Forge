//! Thin CLI surface over the shared human-task completion logic.
//!
//! The completion semantics (governed mutation, milestone/closure follow-on
//! events) live in `sea_forge_case_runner::case_ops::complete_human_task` so
//! the SFWP server's `human_task.complete` verb calls the same governed
//! implementation in-process; this wrapper keeps the CLI's command surface
//! unchanged.

use sea_forge_core::errors::ForgeError;
use std::path::Path;

pub fn complete(
    root: &Path,
    policy: &Path,
    actor: &str,
    case_id: &str,
    item_id: &str,
    note: Option<&str>,
) -> Result<u8, ForgeError> {
    sea_forge_case_runner::case_ops::complete_human_task(
        root, policy, actor, case_id, item_id, note,
    )
}
