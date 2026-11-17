//! The execution gate (WP-21 part A, `contracts/fde/README.md` §7): I-1 "freeze
//! before move" enforced where work starts, on the verbs `item_execute`,
//! `case_advance`, `delegate` (`agent_run.delegate`) and `human_task_complete`.
//!
//! The case is resolved HERE, from `case_id` and the case store, never from a
//! tag the caller sends: the template comes from `cases/<case_id>/plan.json`
//! and the episode's freeze, lifecycle, settlement and latest `n` come from the
//! ledger. A case whose template is not `fde.field_case@1` is not touched.
//!
//! Order of evaluation (README §7): R9, R13, R6, R11 and R14 (the
//! transition-only path ends there), R1/R2, R3, R4, R5, R12, request
//! consistency, R7 (the program-execution decision, step 5), R10. A refusal is
//! decided before any run, spend or record: it carries `no_side_effect: true`.
//!
//! The caller holds the case's keyed lock around [`admit`]: the check and the
//! ledger writes it makes (the assignment record) are one critical section.
//! The execution itself starts after the lock is released; a record written
//! between the two (a suspension, say) is not seen by that run: a stated limit.

use super::case_records::{self, Lifecycle};
use super::run_records;
use sea_forge_core::types::{CasePlan, ItemKind, TraceEvent, TraceKind};
use serde_json::{json, Value};
use std::path::Path;

/// The template of a Field Case.
pub const FIELD_CASE_TEMPLATE: &str = "fde.field_case@1";
const ASSIGNMENTS: &str = "field-assignments.jsonl";
/// Runs started under each program-execution decision (README §10 `runs_used`).
const EXEC_RUNS: &str = "field-exec-decisions.jsonl";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    ItemExecute,
    CaseAdvance,
    Delegate,
    HumanTaskComplete,
}

/// One gated request, as it arrived (nothing here is trusted until checked).
pub struct GateInput<'a> {
    pub verb: Verb,
    pub case_id: &'a str,
    /// `item_execute` and `human_task_complete`.
    pub item_id: Option<&'a str>,
    pub episode_ref: &'a Value,
    /// `case_advance`.
    pub transition_items: &'a Value,
    /// `delegate`.
    pub assignment_id: Option<&'a str>,
    /// The `delegate` endpoint (names the delegated agent).
    pub endpoint: Option<&'a str>,
    /// The authenticated principal of the request, when one was resolved.
    pub actor: Option<&'a str>,
    /// The gateway that spoke for the actor, when the request was delegated.
    pub gateway_actor: Option<&'a str>,
    /// The execution's time bound: the request's `timeout`, or `max_turns` for a delegation.
    pub timeout_s: u64,
}

/// An episode the gate resolved on the server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Episode {
    pub case_id: String,
    pub plan_item_id: String,
    pub n: u64,
    pub kind: String,
    /// `episode:<case>:<item>:<n>@sha256:<digest>`, from the freeze record.
    pub episode_ref: String,
}

/// What an admitted request may do.
#[derive(Debug, Default)]
pub struct Admission {
    /// The case is a Field Case (the gate applied).
    pub field_case: bool,
    /// The resolved episode (absent on a plain case and on a transition-only advance).
    pub episode: Option<Episode>,
    /// A transition-only advance: `(item_id, "skip" | "complete")`, nothing is executed.
    pub transitions: Vec<(String, String)>,
}

/// A gate item completes only on a passing assessment, and a hard gate is not
/// skipped either: the rule lives in ONE function, `case_ops::field_guard`, which
/// every writer (this admission, the transition writer, the engine, the human
/// completion) calls. Refused with the contract's code and no side effect.
fn gate_item_check(root: &Path, case_id: &str, item: &str, how: &str) -> Result<(), Value> {
    sea_forge_case_runner::case_ops::field_guard::gate_item_check(
        &root.join("cases").join(case_id),
        item,
        how,
    )
    .map_err(|r| r.to_value())
}

/// The check adjacent to an execution (item 2 of the WP-21b review, item 4 of
/// round 2): under the SAME lock acquisition as the execution, the case is read
/// again. A suspension is refused. A closed case runs only the action that ends
/// the plan (R13, D42): the execution on `item` must leave nothing to do; any
/// other pass (another item, the supervisor) is `CASE_CLOSED`. Not a Field
/// Case: nothing to check.
pub fn recheck_lifecycle(root: &Path, case_id: &str, item: Option<&str>) -> Result<(), Value> {
    if !matches!(template_of(root, case_id), Ok(Some(t)) if t == FIELD_CASE_TEMPLATE) {
        return Ok(());
    }
    let facts = case_records::episode_facts(root, case_id, "-", 0).map_err(|why| {
        refuse(
            "FREEZE_INCOMPLETE",
            &format!("store integrity: {why}"),
            Some("I-1"),
        )
    })?;
    match facts.lifecycle {
        Lifecycle::Suspended => Err(refuse("CASE_SUSPENDED", "the case is suspended", None)),
        Lifecycle::Closed => {
            let ends_plan = match item {
                Some(item) => reaches_terminal(root, case_id, &[(item.to_string(), "complete")])
                    .map_err(|why| {
                        refuse(
                            "FREEZE_INCOMPLETE",
                            &format!("the plan cannot be read: {why}"),
                            Some("I-1"),
                        )
                    })?,
                None => false,
            };
            if ends_plan {
                Ok(())
            } else {
                Err(refuse(
                    "CASE_CLOSED",
                    "the case has a close disposition; closed is final",
                    None,
                ))
            }
        }
        Lifecycle::Open => Ok(()),
    }
}

/// [`admit`] again for a human completion, at the write: the lifecycle, the
/// close rule and the gate rule are decided on the state the write sees.
pub fn recheck_human_completion(root: &Path, case_id: &str, item: &str) -> Result<(), Value> {
    admit(
        root,
        &GateInput {
            verb: Verb::HumanTaskComplete,
            case_id,
            item_id: Some(item),
            episode_ref: &Value::Null,
            transition_items: &Value::Null,
            assignment_id: None,
            endpoint: None,
            actor: None,
            gateway_actor: None,
            timeout_s: 0,
        },
    )
    .map(|_| ())
}

/// [`admit`] again for a transition-only advance, at the write: the lifecycle,
/// the close rule and the gate rule are decided on the state the write sees.
pub fn recheck_transitions(
    root: &Path,
    case_id: &str,
    transitions: &[(String, String)],
) -> Result<(), Value> {
    let items: Vec<Value> = transitions
        .iter()
        .map(|(i, how)| json!({"item_id": i, "transition": how, "transition_only": true}))
        .collect();
    admit(
        root,
        &GateInput {
            verb: Verb::CaseAdvance,
            case_id,
            item_id: None,
            episode_ref: &Value::Null,
            transition_items: &Value::Array(items),
            assignment_id: None,
            endpoint: None,
            actor: None,
            gateway_actor: None,
            timeout_s: 0,
        },
    )
    .map(|_| ())
}

fn refuse(code: &str, message: &str, invariant: Option<&str>) -> Value {
    let mut out = json!({"outcome": "refused", "code": code, "message": message,
                         "no_side_effect": true});
    if let Some(invariant) = invariant {
        out["invariant"] = json!(invariant);
    }
    out
}

fn template_of(root: &Path, case_id: &str) -> Result<Option<String>, String> {
    let dir = root.join("cases").join(case_id);
    let path = dir.join("plan.json");
    match std::fs::read(&path) {
        Ok(bytes) => {
            let doc: Value = serde_json::from_slice(&bytes)
                .map_err(|e| format!("{} is not valid JSON: {e}", path.display()))?;
            Ok(doc["template_ref"].as_str().map(String::from))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("read {}: {e}", path.display())),
    }
}

/// Is every item finished, or does the engine have only the case close or
/// terminate left?
fn plan_is_finished(plan: &CasePlan, events: &[TraceEvent]) -> bool {
    use sea_forge_planner::case_engine::{next_case_actions, replay_case, CaseAction, ItemStatus};
    replay_case(&plan.items, events).items.iter().all(|s| {
        matches!(
            s.status,
            ItemStatus::Completed | ItemStatus::Terminated | ItemStatus::Failed
        )
    }) || next_case_actions(&plan.items, events).iter().any(|a| {
        matches!(
            a,
            CaseAction::CompleteCase | CaseAction::TerminateCase { .. }
        )
    })
}

/// Would the action(s) leave the plan with nothing left to do (the engine would
/// complete or terminate the case)? Evaluated on the plan and the event ledger
/// with the scope's items hypothetically finished.
fn reaches_terminal(root: &Path, case_id: &str, scope: &[(String, &str)]) -> Result<bool, String> {
    if scope.is_empty() {
        return Ok(false);
    }
    let dir = root.join("cases").join(case_id);
    let plan: CasePlan = serde_json::from_slice(
        &std::fs::read(dir.join("plan.json")).map_err(|e| format!("read plan.json: {e}"))?,
    )
    .map_err(|e| format!("plan.json: {e}"))?;
    let mut events: Vec<TraceEvent> = Vec::new();
    for (i, line) in std::fs::read_to_string(dir.join("case-events.jsonl"))
        .map_err(|e| format!("read case-events.jsonl: {e}"))?
        .lines()
        .filter(|l| !l.trim().is_empty())
        .enumerate()
    {
        events.push(
            serde_json::from_str(line)
                .map_err(|e| format!("case-events.jsonl line {}: {e}", i + 1))?,
        );
    }
    // A plan that is already finished has nothing left for an action to end,
    // except the case close the engine still owes: once that is written, a
    // repeat of the action that ended the plan is not "the action ending it".
    if plan_is_finished(&plan, &events) {
        return Ok(!events
            .iter()
            .any(|e| matches!(e.kind, TraceKind::CaseClosed | TraceKind::CaseTerminated)));
    }
    for (item, how) in scope {
        events.push(TraceEvent {
            version: "0.2".into(),
            event_id: format!("gate-hypothetical-{item}"),
            run_id: plan.run_id.clone(),
            plan_item_id: Some(item.clone()),
            kind: if *how == "skip" {
                TraceKind::ItemTerminated
            } else {
                TraceKind::ItemCompleted
            },
            actor_id: "execution-gate".into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            payload: json!({}),
            cell_id: None,
        });
    }
    use sea_forge_planner::case_engine::{next_case_actions, replay_case, CaseAction, ItemStatus};
    // The engine's automatic milestones cascade: a milestone that would be
    // achieved next (its gate passing, read now) is part of what this action
    // leads to, so a close behind it is seen here and refused before a write.
    let case_dir = root.join("cases").join(case_id);
    for _ in 0..plan.items.len() {
        let Some(id) = next_case_actions(&plan.items, &events)
            .iter()
            .find_map(|a| match a {
                CaseAction::AchieveMilestone(id) => Some(id.clone()),
                _ => None,
            })
        else {
            break;
        };
        if sea_forge_case_runner::case_ops::field_guard::gate_item_check(&case_dir, &id, "complete")
            .is_err()
        {
            break;
        }
        events.push(TraceEvent {
            version: "0.2".into(),
            event_id: format!("gate-hypothetical-{id}"),
            run_id: plan.run_id.clone(),
            plan_item_id: Some(id),
            kind: TraceKind::MilestoneAchieved,
            actor_id: "execution-gate".into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            payload: json!({}),
            cell_id: None,
        });
    }
    let projection = replay_case(&plan.items, &events);
    let finished = projection.items.iter().all(|s| {
        matches!(
            s.status,
            ItemStatus::Completed | ItemStatus::Terminated | ItemStatus::Failed
        )
    });
    Ok(finished
        || next_case_actions(&plan.items, &events).iter().any(|a| {
            matches!(
                a,
                CaseAction::CompleteCase | CaseAction::TerminateCase { .. }
            )
        }))
}

/// The plan items that have an execution step (sandboxed or agent work).
fn execution_items(root: &Path, case_id: &str) -> Result<Vec<String>, String> {
    let dir = root.join("cases").join(case_id);
    let plan: CasePlan = serde_json::from_slice(
        &std::fs::read(dir.join("plan.json")).map_err(|e| format!("read plan.json: {e}"))?,
    )
    .map_err(|e| format!("plan.json: {e}"))?;
    Ok(plan
        .items
        .iter()
        .filter(|i| matches!(i.item_kind, ItemKind::SandboxedTask | ItemKind::AgentTask))
        .map(|i| i.plan_item_id.clone())
        .collect())
}

fn parse_assignment(id: &str) -> Option<(String, String, u64, u64)> {
    let mut parts = id.strip_prefix("assign:")?.split(':');
    let (case, item, n, k) = (parts.next()?, parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() {
        return None;
    }
    Some((case.into(), item.into(), n.parse().ok()?, k.parse().ok()?))
}

/// Check one request. `Ok` means the gate admits it (and the admission's own
/// records are written); `Err` is the refusal to return, with no side effect.
/// The caller holds the case lock.
pub fn admit(root: &Path, input: &GateInput) -> Result<Admission, Value> {
    let delegate = input.verb == Verb::Delegate;
    let unresolved = |why: &str| refuse("CASE_UNRESOLVED", why, None);
    // R9: the case is resolved from the store. A `delegate` whose case does not
    // resolve fails closed (never assumed plain); the other verbs keep their
    // own unknown-case answers.
    let known = sea_forge_core::path::valid_id_segment(input.case_id, 128)
        && root
            .join("cases")
            .join(input.case_id)
            .join("case.json")
            .exists();
    if !known {
        return if delegate {
            Err(unresolved("case_id does not resolve to a case"))
        } else {
            Ok(Admission::default())
        };
    }
    let template = match template_of(root, input.case_id) {
        Ok(t) => t,
        Err(why) if delegate => return Err(unresolved(&why)),
        Err(_) => return Ok(Admission::default()),
    };
    if template.as_deref() != Some(FIELD_CASE_TEMPLATE) {
        return Ok(Admission::default());
    }
    let schema = |why: &str| refuse("SCHEMA_INVALID", why, None);

    // The request's own episode_ref, validated for shape.
    let er = input.episode_ref;
    let key = if er.is_object() {
        let (Some(case), Some(item), Some(n), Some(kind)) = (
            er["case_id"].as_str(),
            er["plan_item_id"].as_str(),
            er["n"].as_u64().filter(|n| *n >= 1),
            er["kind"]
                .as_str()
                .filter(|k| matches!(*k, "evaluated" | "exploratory")),
        ) else {
            return Err(schema(
                "episode_ref is not {case_id, plan_item_id, n, kind}",
            ));
        };
        if case != input.case_id {
            return Err(schema("episode_ref.case_id differs from case_id"));
        }
        Some((item.to_string(), n, kind.to_string()))
    } else if er.is_null() {
        None
    } else {
        return Err(schema("episode_ref is not an object"));
    };

    let scope_item = key
        .as_ref()
        .map(|(i, ..)| i.as_str())
        .or(input.item_id)
        .unwrap_or("-");
    let facts = case_records::episode_facts(
        root,
        input.case_id,
        scope_item,
        key.as_ref().map_or(0, |(_, n, _)| *n),
    )
    .map_err(|why| {
        refuse(
            "FREEZE_INCOMPLETE",
            &format!("store integrity: {why}"),
            Some("I-1"),
        )
    })?;

    // The transition-only request, parsed: (item, transition) pairs.
    let mut transitions: Vec<(String, String)> = Vec::new();
    let mut transitions_shape_ok = true;
    if let Some(items) = input.transition_items.as_array() {
        for t in items {
            match (
                t["item_id"].as_str(),
                t["transition"].as_str(),
                t["transition_only"].as_bool(),
            ) {
                (Some(i), Some(how @ ("skip" | "complete")), Some(true)) => {
                    transitions.push((i.to_string(), how.to_string()))
                }
                _ => transitions_shape_ok = false,
            }
        }
    } else if !input.transition_items.is_null() {
        transitions_shape_ok = false;
    }

    // R11: would this action finish the plan?
    let advance_alone = input.verb == Verb::CaseAdvance && key.is_none();
    let scope: Vec<(String, &str)> = if advance_alone {
        transitions
            .iter()
            .map(|(i, how)| (i.clone(), how.as_str()))
            .collect()
    } else {
        scope_item_scope(input, key.as_ref())
    };
    let terminal = reaches_terminal(root, input.case_id, &scope).map_err(|why| {
        refuse(
            "FREEZE_INCOMPLETE",
            &format!("the plan cannot be read: {why}"),
            Some("I-1"),
        )
    })?;

    // R13: a closed case runs nothing more, except the action that ends the plan.
    if facts.lifecycle == Lifecycle::Closed && !terminal {
        return Err(refuse(
            "CASE_CLOSED",
            "the case has a close disposition; closed is final",
            None,
        ));
    }
    // R6
    if facts.lifecycle == Lifecycle::Suspended {
        return Err(refuse("CASE_SUSPENDED", "the case is suspended", None));
    }
    // R11: any path to a terminal state needs a case-level close disposition.
    if terminal
        && !facts
            .latest_disposition
            .as_deref()
            .is_some_and(|d| d.starts_with("close-"))
    {
        return Err(refuse(
            "CLOSE_DISPOSITION_REQUIRED",
            "this action ends the case and no case-level close disposition exists",
            None,
        ));
    }
    if input.verb == Verb::HumanTaskComplete {
        if let Some(item) = input.item_id {
            gate_item_check(root, input.case_id, item, "complete")?;
        }
        return Ok(Admission {
            field_case: true,
            ..Admission::default()
        });
    }
    // R14: an advance without an episode_ref must say, per item, that it only
    // transitions, and never for an item that has an execution step.
    if advance_alone {
        if transitions.is_empty() && transitions_shape_ok {
            return Err(refuse(
                "FREEZE_MISSING",
                "no episode_ref and no transition_items",
                Some("I-1"),
            ));
        }
        // A gate item moves only on its assessment (SOP 8.1), never by a bare
        // transition, whatever the request says about transition_only.
        for item in input.transition_items.as_array().into_iter().flatten() {
            if let (Some(i), Some(how)) = (item["item_id"].as_str(), item["transition"].as_str()) {
                gate_item_check(root, input.case_id, i, how)?;
            }
        }
        let exec = execution_items(root, input.case_id).map_err(|why| {
            refuse(
                "FREEZE_INCOMPLETE",
                &format!("the plan cannot be read: {why}"),
                Some("I-1"),
            )
        })?;
        if !transitions_shape_ok || transitions.iter().any(|(i, _)| exec.contains(i)) {
            return Err(refuse(
                "TRANSITION_ONLY_INVALID",
                "transition_items must list non-execution items with transition_only: true",
                None,
            ));
        }
        return Ok(Admission {
            field_case: true,
            transitions,
            ..Admission::default()
        });
    }
    // R1/R2/R3: the freeze.
    let Some((item, n, kind)) = key else {
        return Err(refuse(
            "FREEZE_MISSING",
            "a Field Case request needs an episode_ref",
            Some("I-1"),
        ));
    };
    let exists = [facts.realitytrace, facts.case_record, facts.anchor];
    if exists.iter().all(|p| !p) {
        return Err(refuse(
            "FREEZE_MISSING",
            "the episode has no freeze",
            Some("I-1"),
        ));
    }
    let required = [true, true, facts.anchor_required];
    if exists.iter().zip(required).any(|(e, r)| r && !e) {
        return Err(refuse(
            "FREEZE_INCOMPLETE",
            "the episode's freeze is incomplete",
            Some("I-1"),
        ));
    }
    // R4
    if facts.anchor && !facts.anchor_publisher_ok {
        return Err(refuse(
            "ANCHOR_PUBLISHER_INVALID",
            "the anchor was not published by the case keeper",
            None,
        ));
    }
    // R5
    if facts.frozen_kind.as_deref() != Some(kind.as_str()) {
        return Err(refuse(
            "KIND_IMMUTABLE",
            "episode_ref.kind differs from the frozen kind",
            Some("I-3"),
        ));
    }
    // R12
    if n < facts.latest_n {
        return Err(refuse(
            "EPISODE_SUPERSEDED",
            "a later episode was allocated for this plan item",
            None,
        ));
    }
    if facts.settled {
        return Err(refuse(
            "EPISODE_SETTLED",
            "the episode is already settled",
            None,
        ));
    }
    // Request consistency.
    if input.verb == Verb::ItemExecute && input.item_id != Some(item.as_str()) {
        return Err(schema("item_id differs from episode_ref.plan_item_id"));
    }
    if delegate && input.assignment_id.is_none() {
        return Err(schema("a Field Case delegate needs an assignment_id"));
    }
    // R7: the program-execution decision (README §10), D39: the CEP decision of
    // the existing authority path whose resource_id is `execution:sha256:<digest>`
    // of the nine-member execution object built HERE from the ledger and the plan,
    // never from the request.
    let config = case_records::fde_config(root).map_err(|why| {
        refuse(
            "FREEZE_INCOMPLETE",
            &format!("store integrity: {why}"),
            Some("I-1"),
        )
    })?;
    let workspace = workspace_of(input, &item);
    let frozen_ref = facts.episode_ref.clone().unwrap_or_default();
    let object = execution_object(
        root,
        input,
        &item,
        &frozen_ref,
        facts.world_ref.as_deref().unwrap_or(""),
        &workspace,
    )
    .map_err(|why| {
        refuse(
            "AUTHORITY_DENIED",
            &format!("the execution cannot be bound: {why}"),
            None,
        )
    })?;
    let decision = find_decision(root, input.case_id, &object, &config)?;
    // R10: the assignment.
    let mut assignment: Option<(String, &str)> = None;
    if let Some(id) = input.assignment_id.filter(|_| delegate) {
        let invalid = |why: &str| refuse("ASSIGNMENT_ID_INVALID", why, None);
        let Some((c, i, an, k)) = parse_assignment(id) else {
            return Err(invalid("assignment_id is not assign:<case>:<item>:<n>:<k>"));
        };
        if (c.as_str(), i.as_str(), an) != (input.case_id, item.as_str(), n) || k == 0 {
            return Err(invalid("assignment_id names another episode"));
        }
        let path = root.join("cases").join(input.case_id).join(ASSIGNMENTS);
        let lines = case_records::read_lines_strict(&path).map_err(|why| {
            refuse(
                "FREEZE_INCOMPLETE",
                &format!("store integrity: {why}"),
                Some("I-1"),
            )
        })?;
        let mine: Vec<&Value> = lines.iter().filter(|l| l["assignment_id"] == id).collect();
        match mine.last() {
            None => assignment = Some((id.to_string(), "issued")),
            Some(last) if last["state"] == "interrupted" => {
                let expired = last["link_expires_at"]
                    .as_str()
                    .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                    .is_none_or(|t| t <= chrono::Utc::now());
                if expired {
                    return Err(refuse(
                        "ASSIGNMENT_LINK_EXPIRED",
                        "the link of the interrupted assignment expired; a new link and a new k are needed",
                        None,
                    ));
                }
                assignment = Some((id.to_string(), "resumed"));
            }
            Some(_) => {
                return Err(invalid(
                    "k is already issued and that assignment is not interrupted",
                ))
            }
        }
    }
    // Admitted: the gate's own record (still under the caller's case lock).
    if let Some((id, state)) = assignment {
        case_records::append_line(
            &root.join("cases").join(input.case_id).join(ASSIGNMENTS),
            &json!({"assignment_id": id, "state": state,
                    "episode_key": {"case_id": input.case_id, "plan_item_id": item, "n": n},
                    "at": case_records::now_rfc3339()}),
        )
        .map_err(|why| {
            refuse(
                "FREEZE_INCOMPLETE",
                &format!("store integrity: {why}"),
                Some("I-1"),
            )
        })?;
    }
    // The run record, and for a delegation the link before it (README §8),
    // still under the caller's case lock and before anything starts.
    let episode_ref = facts.episode_ref.unwrap_or_default();
    if input.verb != Verb::HumanTaskComplete {
        let config = case_records::fde_config(root).map_err(|why| {
            refuse(
                "FREEZE_INCOMPLETE",
                &format!("store integrity: {why}"),
                Some("I-1"),
            )
        })?;
        let agent = input.endpoint.map(|e| format!("agent:{e}"));
        let _ = &workspace;
        let verifier = delegate
            && input.endpoint.is_some_and(|e| {
                config["verifier_endpoints"]
                    .as_array()
                    .is_some_and(|l| l.iter().any(|v| v == e))
            });
        let link = match (delegate, input.actor, &agent, input.assignment_id) {
            (true, Some(actor), Some(agent), Some(assignment)) => Some(run_records::LinkInput {
                assignment_id: assignment,
                delegator: actor,
                gateway_actor: input.gateway_actor,
                delegate: agent,
                ttl_secs: 8 * 3600,
            }),
            _ => None,
        };
        let run_actor = if delegate {
            agent.as_deref()
        } else {
            input.actor
        };
        let on_behalf_of = if delegate { input.actor } else { None };
        let (run_id, run_record) = run_records::record_run(
            root,
            &run_records::RunInput {
                case_id: input.case_id,
                episode_ref: &episode_ref,
                verb: match input.verb {
                    Verb::ItemExecute => "item_execute",
                    Verb::CaseAdvance => "case_advance",
                    _ => "delegate",
                },
                role: if verifier { "verifier" } else { "executor" },
                actor: run_actor,
                on_behalf_of,
                workspace_ref: &workspace,
                link,
                decision_id: Some(decision.decision_id.as_str()),
            },
        )
        .map_err(|why| {
            refuse(
                "FREEZE_INCOMPLETE",
                &format!("store integrity: {why}"),
                Some("I-1"),
            )
        })?;
        // `runs_used` + 1, in the same critical section as the check above.
        case_records::append_line(
            &root.join("cases").join(input.case_id).join(EXEC_RUNS),
            &json!({"decision_key": decision.key, "decision_id": decision.decision_id,
                    "run_id": run_id, "at": case_records::now_rfc3339()}),
        )
        .map_err(|why| {
            refuse(
                "FREEZE_INCOMPLETE",
                &format!("store integrity: {why}"),
                Some("I-1"),
            )
        })?;
        // The interaction event carries `authority.decision_id` for RealityTrace.
        case_records::enqueue_run_event(root, input.case_id, &run_record);
    }
    Ok(Admission {
        field_case: true,
        episode: Some(Episode {
            case_id: input.case_id.to_string(),
            plan_item_id: item,
            n,
            kind,
            episode_ref,
        }),
        transitions: Vec::new(),
    })
}

fn scope_item_scope(
    input: &GateInput,
    key: Option<&(String, u64, String)>,
) -> Vec<(String, &'static str)> {
    match (input.verb, key, input.item_id) {
        (Verb::Delegate, Some((i, ..)), _) | (Verb::CaseAdvance, Some((i, ..)), _) => {
            vec![(i.clone(), "complete")]
        }
        (Verb::ItemExecute | Verb::HumanTaskComplete, _, Some(i)) => {
            vec![(i.to_string(), "complete")]
        }
        _ => Vec::new(),
    }
}

/// The workspace a run uses (also the sixth member of the bound execution).
fn workspace_of(input: &GateInput, item: &str) -> String {
    match (input.verb, input.endpoint) {
        (Verb::Delegate, Some(endpoint)) => format!("ws:agent:{endpoint}"),
        _ => format!("ws:case:{}:{item}", input.case_id),
    }
}

/// The nine-member execution object of `reserved-operation.v1` `BoundExecution`,
/// built from what the server knows: the frozen episode and world, the plan
/// item's command, the request's time bound. A delegation has no command: its
/// argv is the endpoint and its sandbox profile `agent`.
fn execution_object(
    root: &Path,
    input: &GateInput,
    item: &str,
    episode_ref: &str,
    world_ref: &str,
    workspace: &str,
) -> Result<Value, String> {
    use sea_forge_core::types::Operation;
    let plan: CasePlan = serde_json::from_slice(
        &std::fs::read(root.join("cases").join(input.case_id).join("plan.json"))
            .map_err(|e| format!("read plan.json: {e}"))?,
    )
    .map_err(|e| format!("plan.json: {e}"))?;
    let plan_item = plan.items.iter().find(|i| i.plan_item_id == item);
    let (argv, cwd, profile) = if input.verb == Verb::Delegate {
        (
            vec![input.endpoint.unwrap_or("").to_string()],
            ".".to_string(),
            "agent".to_string(),
        )
    } else {
        let command = plan_item.and_then(|i| {
            i.operations.iter().find_map(|o| match o {
                Operation::ExecuteCommand { argv, cwd } => Some((argv.clone(), cwd.clone())),
                _ => None,
            })
        });
        let (argv, cwd) = command.unwrap_or_else(|| (vec![item.to_string()], ".".to_string()));
        let profile = plan_item
            .and_then(|i| i.sandbox_class.clone())
            .unwrap_or_else(|| "default".to_string());
        (argv, cwd, profile)
    };
    let empty_env = case_records::digest_of(&json!({})).ok_or("env digest")?;
    let patch = hex::encode(<sha2::Sha256 as sha2::Digest>::digest(b""));
    Ok(json!({
        "episode_ref": episode_ref, "patch_sha256": patch, "argv": argv, "cwd": cwd,
        "env_digest": empty_env, "workspace_ref": workspace, "sandbox_profile": profile,
        "world_ref": world_ref, "timeout_s": input.timeout_s.max(1),
    }))
}

struct Decision {
    /// The ledger entry of the decision (unique; `decision_id` is only unique per operation).
    key: String,
    decision_id: String,
}

/// Find an unexpired `allow` decision of the CEP authority path for exactly this
/// execution and take the first one with runs left. Missing, denied, expired,
/// exhausted, or any unreadable record is `AUTHORITY_DENIED` (fail closed).
fn find_decision(
    root: &Path,
    case_id: &str,
    object: &Value,
    config: &Value,
) -> Result<Decision, Value> {
    let denied = |why: &str| refuse("AUTHORITY_DENIED", why, None);
    let digest = case_records::digest_of(object)
        .ok_or_else(|| denied("the execution object cannot be canonicalised"))?;
    let resource = format!("execution:sha256:{digest}");
    let ledger = sea_forge_ledger::LedgerStream::open(
        root,
        sea_forge_authority::cep::LEDGER_ID,
        "execution-gate",
    )
    .map_err(|e| denied(&format!("the authority ledger cannot be opened: {e}")))?;
    let entries = ledger
        .read_entries()
        .map_err(|e| denied(&format!("the authority ledger cannot be read: {e}")))?;
    let runs = case_records::read_lines_strict(&root.join("cases").join(case_id).join(EXEC_RUNS))
        .map_err(|e| denied(&format!("the run counter cannot be read: {e}")))?;
    let max_runs = config["execution_max_runs"].as_u64().unwrap_or(1).max(1);
    let ttl = config["execution_decision_ttl_secs"]
        .as_i64()
        .unwrap_or(86_400);
    let now = chrono::Utc::now();
    let mut seen_allow = false;
    for e in entries
        .iter()
        .filter(|e| e.record_kind == "authority_decision")
    {
        let p = &e.payload;
        if p["operation"]["resource_id"] != resource.as_str() || p["outcome"] != "allow" {
            continue;
        }
        let fresh = p["decided_at"]
            .as_str()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
            .is_some_and(|t| (now - t.with_timezone(&chrono::Utc)).num_seconds() <= ttl);
        if !fresh {
            continue;
        }
        seen_allow = true;
        let used = runs
            .iter()
            .filter(|r| r["decision_key"] == e.entry_ulid.as_str())
            .count() as u64;
        if used < max_runs {
            return Ok(Decision {
                key: e.entry_ulid.clone(),
                decision_id: p["decision_id"].as_str().unwrap_or("").to_string(),
            });
        }
    }
    Err(denied(if seen_allow {
        "the program-execution decision is exhausted (runs_used reached max_runs)"
    } else {
        "no unexpired allow decision binds this execution (denied, absent, expired or another execution)"
    }))
}
