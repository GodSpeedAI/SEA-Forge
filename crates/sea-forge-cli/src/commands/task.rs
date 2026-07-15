use sea_forge_core::{errors::ForgeError, types::*, RECORD_VERSION};
use sea_forge_planner::case_engine::{next_case_actions, CaseAction};
use serde_json::json;
use std::path::Path;

pub fn complete(
    root: &Path,
    policy: &Path,
    actor: &str,
    case_id: &str,
    item_id: &str,
    note: Option<&str>,
) -> Result<u8, ForgeError> {
    let (mut case, plan, mut events) = super::case::load_case_plan(root, case_id)?;
    let item = plan
        .items
        .iter()
        .find(|item| item.plan_item_id == item_id)
        .ok_or_else(|| ForgeError::Input("human task not found".into()))?;
    if item.item_kind != ItemKind::HumanTask {
        return Err(ForgeError::Input("item is not a human task".into()));
    }
    super::mediated::authorize_read(
        root,
        policy,
        actor,
        &AuthorityAction::Reserved {
            resource_type: "human_task_completion".into(),
            resource_id: format!("{case_id}:{item_id}"),
            parameters: json!({"note": note}),
        },
    )?;
    super::case::append_case_event(
        root,
        case_id,
        actor,
        TraceKind::HumanTaskCompleted,
        Some(item_id),
        json!({"note": note}),
    )?;
    events.push(TraceEvent {
        version: RECORD_VERSION.into(),
        event_id: "projection".into(),
        run_id: "case".into(),
        plan_item_id: Some(item_id.into()),
        kind: TraceKind::HumanTaskCompleted,
        actor_id: actor.into(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        payload: json!({"note": note}),
        cell_id: None,
    });
    loop {
        match next_case_actions(&plan.items, &events).as_slice() {
            [CaseAction::AchieveMilestone(id)] => {
                super::case::append_case_event(
                    root,
                    case_id,
                    actor,
                    TraceKind::MilestoneAchieved,
                    Some(id),
                    json!({}),
                )?;
                events.push(TraceEvent {
                    version: RECORD_VERSION.into(),
                    event_id: "projection".into(),
                    run_id: "case".into(),
                    plan_item_id: Some(id.clone()),
                    kind: TraceKind::MilestoneAchieved,
                    actor_id: actor.into(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    payload: json!({}),
                    cell_id: None,
                });
            }
            [CaseAction::CompleteCase] => {
                super::case::append_case_event(
                    root,
                    case_id,
                    actor,
                    TraceKind::CaseClosed,
                    None,
                    json!({}),
                )?;
                case.state = CaseState::Completed;
                case.closed_at = Some(chrono::Utc::now().to_rfc3339());
                super::case::save_case(root, case_id, &case)?;
                return Ok(0);
            }
            _ => return Ok(5),
        }
    }
}
