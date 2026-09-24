//! Thin CLI surface over the shared case-mutation library.
//!
//! The mutation logic (event appends, reopen, discretionary proposals, plan
//! state IO) lives in `sea_forge_case_runner::case_ops` so the SFWP server can
//! call the same governed implementation in-process; these wrappers keep the
//! CLI's command surface unchanged.

use sea_forge_core::{errors::ForgeError, types::*};
use std::path::Path;

pub(crate) fn append_case_event(
    root: &Path,
    case_id: &str,
    actor: &str,
    kind: TraceKind,
    item_id: Option<&str>,
    payload: serde_json::Value,
) -> Result<(), ForgeError> {
    sea_forge_case_runner::case_ops::append_case_event(root, case_id, actor, kind, item_id, payload)
}

pub fn reopen(root: &Path, policy: &Path, actor: &str, case_id: &str) -> Result<u8, ForgeError> {
    sea_forge_case_runner::case_ops::reopen(root, policy, actor, case_id)
}

pub fn add_task(
    root: &Path,
    policy: &Path,
    actor: &str,
    case_id: &str,
    item_path: &Path,
) -> Result<u8, ForgeError> {
    if !sea_forge_core::path::valid_id_segment(case_id, 128) {
        return Err(ForgeError::Input(format!("unsafe case id: {case_id}")));
    }
    let item: PlanItem = serde_json::from_slice(
        &std::fs::read(item_path).map_err(|error| ForgeError::io("read case state", error))?,
    )
    .map_err(ForgeError::from)?;
    propose_item(root, policy, actor, case_id, item)
}

/// Append a discretionary item to a case's plan through the standard
/// authorized mutation path (§7.6, M15 slice 7.3) — the same path `add_task`
/// uses for a file-supplied item, reusable with an in-memory item (e.g. one
/// synthesized by the Thoth manager loop with `proposed_by` set).
pub fn propose_item(
    root: &Path,
    policy: &Path,
    actor: &str,
    case_id: &str,
    item: PlanItem,
) -> Result<u8, ForgeError> {
    sea_forge_case_runner::case_ops::propose_item(root, policy, actor, case_id, item)
}

pub(crate) fn load_case_plan(
    root: &Path,
    case_id: &str,
) -> Result<(Case, CasePlan, Vec<TraceEvent>), ForgeError> {
    sea_forge_case_runner::case_ops::load_case_plan(root, case_id)
}

pub(crate) fn save_case(root: &Path, case_id: &str, case: &Case) -> Result<(), ForgeError> {
    sea_forge_case_runner::case_ops::save_case(root, case_id, case)
}
