use sea_forge_core::{errors::ForgeError, ids, types::*, RECORD_VERSION};
use sea_forge_ledger::LedgerStream;
use sea_forge_planner::case_engine::validate_proposal;
use serde::Serialize;
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

fn paths(
    root: &Path,
    case_id: &str,
) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let dir = root.join("cases").join(case_id);
    (
        dir.join("case.json"),
        dir.join("plan.json"),
        dir.join("case-events.jsonl"),
    )
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ForgeError> {
    serde_json::from_slice(
        &fs::read(path).map_err(|error| ForgeError::io("read case state", error))?,
    )
    .map_err(Into::into)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), ForgeError> {
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(value)?)
        .map_err(|error| ForgeError::io("write case state", error))?;
    fs::rename(temporary, path).map_err(|error| ForgeError::io("replace case state", error))
}

pub(crate) fn append_case_event(
    root: &Path,
    case_id: &str,
    actor: &str,
    kind: TraceKind,
    item_id: Option<&str>,
    payload: serde_json::Value,
) -> Result<(), ForgeError> {
    let (_, _, events_path) = paths(root, case_id);
    let count = fs::read_to_string(&events_path)
        .map(|value| value.lines().count())
        .unwrap_or(0);
    let event = TraceEvent {
        version: RECORD_VERSION.into(),
        event_id: ids::seq_id("cev", 6, count + 1),
        run_id: "case".into(),
        plan_item_id: item_id.map(str::to_owned),
        kind,
        actor_id: actor.into(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        payload,
        cell_id: None,
    };
    LedgerStream::open(root, format!("case-{case_id}"), actor)?.commit_typed(
        "case_event",
        vec![case_id.into()],
        &event,
        vec![],
    )?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(events_path)
        .map_err(|error| ForgeError::io("open case events", error))?;
    serde_json::to_writer(&mut file, &event)?;
    file.write_all(b"\n")
        .map_err(|error| ForgeError::io("append case event", error))
}

pub fn reopen(root: &Path, policy: &Path, actor: &str, case_id: &str) -> Result<u8, ForgeError> {
    let (case_path, _, _) = paths(root, case_id);
    let mut case: Case = read_json(&case_path)?;
    if !matches!(case.state, CaseState::Completed | CaseState::Terminated) {
        return Err(ForgeError::Input(
            "only closed cases can be reopened".into(),
        ));
    }
    super::mediated::authorize_read(
        root,
        policy,
        actor,
        &AuthorityAction::Reserved {
            resource_type: "case_reopen".into(),
            resource_id: case_id.into(),
            parameters: json!({}),
        },
    )?;
    append_case_event(
        root,
        case_id,
        actor,
        TraceKind::CaseReopened,
        None,
        json!({}),
    )?;
    case.state = CaseState::Active;
    case.close_reason = None;
    case.closed_at = None;
    write_json(&case_path, &case)?;
    Ok(0)
}

pub fn add_task(
    root: &Path,
    policy: &Path,
    actor: &str,
    case_id: &str,
    item_path: &Path,
) -> Result<u8, ForgeError> {
    let item: PlanItem = read_json(item_path)?;
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
    let (_, plan_path, _) = paths(root, case_id);
    let mut plan: CasePlan = read_json(&plan_path)?;
    plan.items.push(item.clone());
    validate_proposal(&mut plan)?;
    super::mediated::authorize_read(
        root,
        policy,
        actor,
        &AuthorityAction::Reserved {
            resource_type: "discretionary_task_add".into(),
            resource_id: format!("{case_id}:{}", item.plan_item_id),
            parameters: json!({"item": item.plan_item_id}),
        },
    )?;
    let committed = LedgerStream::open(root, format!("case-{case_id}"), actor)?.commit_typed(
        "case_plan_mutation",
        vec![case_id.into()],
        &plan,
        vec![],
    )?;
    LedgerStream::open(root, format!("case-{case_id}"), actor)?.materialize_view(
        &committed,
        &plan_path,
        &serde_json::to_vec_pretty(&plan)?,
    )?;
    append_case_event(
        root,
        case_id,
        actor,
        TraceKind::PlanMutated,
        Some(&item.plan_item_id),
        json!({"operation": "add_task"}),
    )?;
    Ok(0)
}

pub(crate) fn load_case_plan(
    root: &Path,
    case_id: &str,
) -> Result<(Case, CasePlan, Vec<TraceEvent>), ForgeError> {
    let (case_path, plan_path, events_path) = paths(root, case_id);
    let events = fs::read_to_string(events_path)
        .map_err(|error| ForgeError::io("read case events", error))?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(serde_json::from_str)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| ForgeError::Serialization(error.to_string()))?;
    Ok((read_json(&case_path)?, read_json(&plan_path)?, events))
}

pub(crate) fn save_case(root: &Path, case_id: &str, case: &Case) -> Result<(), ForgeError> {
    let (path, _, _) = paths(root, case_id);
    write_json(&path, case)
}
