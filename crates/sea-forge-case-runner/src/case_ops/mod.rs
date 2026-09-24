//! Governed case mutation operations shared by the CLI and the SFWP server.
//!
//! These functions were extracted verbatim from the CLI (`sea-forge-cli`
//! `commands/case.rs` and `commands/approve.rs`) so that case mutations have
//! exactly one implementation: the CLI renders their results, the server calls
//! them in-process, and neither carries a private copy of a governed write.
//! The library performs no printing — callers render typed results.

pub mod authority_views;
pub mod mediation;

use chrono::Utc;
use sea_forge_core::{
    errors::ForgeError,
    ids,
    types::{
        Actor, ActorRole, ApprovalRequest, ApprovalStatus, AuthorityAction, AuthorityDecision,
        Case, CasePlan, CaseState, NormalizedDisposition, PlanItem, SettlementCriteriaRecord,
        TraceEvent, TraceKind, Verdict,
    },
    RECORD_VERSION,
};
use sea_forge_ledger::LedgerStream;
use serde::Serialize;
use serde_json::{json, Value};
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

pub fn append_case_event(
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
    if !sea_forge_core::path::valid_id_segment(case_id, 128) {
        return Err(ForgeError::Input(format!("unsafe case id: {case_id}")));
    }
    let (case_path, _, _) = paths(root, case_id);
    let mut case: Case = read_json(&case_path)?;
    if !matches!(case.state, CaseState::Completed | CaseState::Terminated) {
        return Err(ForgeError::Input(
            "only closed cases can be reopened".into(),
        ));
    }
    mediation::authorize_read(
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
    sea_forge_planner::case_engine::validate_proposal(&mut plan)?;
    mediation::authorize_read(
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

pub fn load_case_plan(
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

pub fn save_case(root: &Path, case_id: &str, case: &Case) -> Result<(), ForgeError> {
    let (path, _, _) = paths(root, case_id);
    write_json(&path, case)
}

/// Resolve an approval request (`approve`/`reject`), enforcing the same
/// ledger, separation-of-duty, criteria-binding and authority checks the CLI
/// enforces. Returns the resolved [`ApprovalRequest`] so the caller can render
/// the outcome; it performs no printing of its own.
pub fn resolve_approval(
    root: &Path,
    policy: &Path,
    case_id: &str,
    approval_id: &str,
    actor: &str,
    note: Option<&str>,
    approved: bool,
) -> Result<ApprovalRequest, ForgeError> {
    let stream = LedgerStream::open(root, format!("case-{case_id}"), actor)?;
    stream.verify()?;
    let entries = stream.read_entries()?;
    if entries.iter().any(|entry| {
        entry.record_kind == "approval_resolution"
            && entry.payload.get("approval_id").and_then(Value::as_str) == Some(approval_id)
    }) {
        return Err(ForgeError::Input(format!(
            "approval {approval_id} is already resolved — MUST NOT be re-resolved"
        )));
    }
    let latest: ApprovalRequest = entries
        .iter()
        .find(|entry| {
            entry.record_kind == "approval_request"
                && entry.payload.get("approval_id").and_then(Value::as_str) == Some(approval_id)
        })
        .ok_or_else(|| ForgeError::Input(format!("approval {approval_id} not found in ledger")))
        .and_then(|entry| serde_json::from_value(entry.payload.clone()).map_err(Into::into))?;
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
    // SoD (§10.3, T15.4): the actor who proposed a discretionary item
    // cannot resolve settlement approval over that same item. `proposed_by`
    // is part of the plan's canonical hash — relabeling, copying, or
    // replaying the record cannot remove the binding.
    let (_, plan, _) = load_case_plan(root, case_id)?;
    if let Some(item) = plan
        .items
        .iter()
        .find(|item| item.plan_item_id == latest.plan_item_id)
    {
        if item.proposed_by.as_deref() == Some(actor) {
            return Err(ForgeError::Plan {
                class: "sod_violation",
                message: format!(
                    "actor '{actor}' cannot resolve an approval for item '{}' it proposed",
                    item.plan_item_id
                ),
            });
        }
    }
    let decision: AuthorityDecision = entries
        .iter()
        .find(|entry| {
            entry.record_kind == "authority_decision"
                && entry.payload.get("decision_id").and_then(Value::as_str)
                    == Some(latest.decision_id.as_str())
                && entry.payload.get("run_id").and_then(Value::as_str)
                    == Some(latest.run_id.as_str())
                && entry.payload.get("plan_item_id").and_then(Value::as_str)
                    == Some(latest.plan_item_id.as_str())
                && entry
                    .payload
                    .get("audit_record")
                    .and_then(|audit| audit.get("case_id"))
                    .and_then(Value::as_str)
                    == Some(case_id)
        })
        .ok_or_else(|| ForgeError::Input("approval authority decision is missing".into()))
        .and_then(|entry| serde_json::from_value(entry.payload.clone()).map_err(Into::into))?;
    let criteria_id = latest
        .criteria_ref
        .as_deref()
        .ok_or_else(|| ForgeError::Input("approval criteria reference is missing".into()))?;
    let criteria: SettlementCriteriaRecord = entries
        .iter()
        .find(|entry| {
            entry.record_kind == "settlement_criteria"
                && entry.payload.get("criteria_id").and_then(Value::as_str) == Some(criteria_id)
        })
        .ok_or_else(|| ForgeError::Input("approval criteria record is missing".into()))
        .and_then(|entry| serde_json::from_value(entry.payload.clone()).map_err(Into::into))?;
    let context = decision
        .action_request
        .context
        .as_object()
        .ok_or_else(|| ForgeError::Input("approval authority context is malformed".into()))?;
    let requester = decision
        .action_request
        .actor
        .get("actor_id")
        .and_then(Value::as_str)
        .ok_or_else(|| ForgeError::Input("approval requester is malformed".into()))?;
    if actor == requester {
        return Err(ForgeError::Input(
            "approval resolver must differ from requester".into(),
        ));
    }
    if decision.verdict != Verdict::Escalate
        || decision.normalized_disposition != NormalizedDisposition::Escalate
        || decision.run_id != latest.run_id
        || decision.plan_item_id != latest.plan_item_id
        || decision.audit_record.case_id.as_deref() != Some(case_id)
        || decision.audit_record.run_id.as_deref() != Some(latest.run_id.as_str())
        || context.get("run_id").and_then(Value::as_str) != Some(latest.run_id.as_str())
        || context.get("plan_item_id").and_then(Value::as_str) != Some(latest.plan_item_id.as_str())
        || decision.determinism.action_request_hash
            != sea_forge_ledger::types::hash_canonical(&decision.action_request)?
        || latest.criteria_sha256.as_deref() != Some(criteria.criteria_sha256.as_str())
        || latest.criteria_record_hash.as_deref() != Some(criteria.criteria_record_hash.as_str())
        || criteria.criteria_sha256
            != sea_forge_planner::compute_criteria_sha256(&criteria.criteria)?
        || criteria.criteria_record_hash != sea_forge_planner::compute_record_hash(&criteria)?
    {
        return Err(ForgeError::Input(
            "approval does not match its authority decision and criteria".into(),
        ));
    }
    authorize_resolution(
        &stream, root, policy, actor, case_id, &latest, &decision, &criteria, approved,
    )?;
    // Check TTL expiry before resolving.
    let now = Utc::now();
    let expires_at = chrono::DateTime::parse_from_rfc3339(&latest.expires_at)
        .map_err(|_| ForgeError::Input("approval expiry is invalid".into()))?
        .with_timezone(&Utc);
    let now_rfc3339 = now.to_rfc3339();
    if expires_at <= now {
        let expired = ApprovalRequest {
            status: ApprovalStatus::Expired,
            resolved_by: None,
            resolved_at: Some(now_rfc3339.clone()),
            note: Some("ttl_expired".into()),
            ..latest
        };
        stream.commit_typed_once(
            "approval_resolution",
            approval_id,
            vec![case_id.into(), approval_id.into()],
            &expired,
            vec![expired.decision_id.clone()],
        )?;
        sea_forge_core::approvals::append(root, &expired)?;
        return Err(ForgeError::Input(format!(
            "approval {approval_id} expired before resolution"
        )));
    }
    let resolved = ApprovalRequest {
        status: if approved {
            ApprovalStatus::Approved
        } else {
            ApprovalStatus::Rejected
        },
        resolved_by: Some(actor.into()),
        resolved_at: Some(now_rfc3339),
        note: note.map(str::to_owned),
        ..latest
    };
    stream.commit_typed_once(
        "approval_resolution",
        approval_id,
        vec![case_id.into(), approval_id.into()],
        &resolved,
        vec![resolved.decision_id.clone()],
    )?;
    sea_forge_core::approvals::append(root, &resolved)?;
    Ok(resolved)
}

#[allow(clippy::too_many_arguments)]
fn authorize_resolution(
    stream: &LedgerStream,
    root: &Path,
    policy: &Path,
    actor_id: &str,
    case_id: &str,
    approval: &ApprovalRequest,
    original: &AuthorityDecision,
    criteria: &SettlementCriteriaRecord,
    approved: bool,
) -> Result<(), ForgeError> {
    let bundle = sea_forge_authority::AuthorityPolicyBundle::load(policy)?;
    let required_roles = original
        .required_next_steps
        .iter()
        .filter_map(|step| step.strip_prefix("approval-role:"))
        .map(|role| {
            serde_json::from_value::<ActorRole>(Value::String(role.into())).map_err(|_| {
                ForgeError::Input("approval authority decision has an invalid approver role".into())
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let role = if required_roles.is_empty() {
        bundle
            .identity_bindings
            .iter()
            .find(|binding| binding.principal == actor_id)
            .map(|binding| binding.role.clone())
            .unwrap_or(ActorRole::Operator)
    } else {
        bundle
            .identity_bindings
            .iter()
            .filter(|binding| binding.principal == actor_id)
            .find(|binding| required_roles.contains(&binding.role))
            .map(|binding| binding.role.clone())
            .ok_or_else(|| {
                ForgeError::Input("approval resolver lacks a required approver role".into())
            })?
    };
    let actor = Actor {
        actor_id: actor_id.into(),
        role: role.clone(),
    };
    let action = AuthorityAction::Reserved {
        resource_type: "approval_resolution".into(),
        resource_id: approval.approval_id.clone(),
        parameters: json!({
            "approval_id": approval.approval_id,
            "original_decision_id": original.decision_id,
            "original_operation": original.operation,
            "requester_id": original.action_request.actor.get("actor_id"),
            "required_approver_roles": required_roles,
            "resolution": if approved { "approved" } else { "rejected" },
            "criteria_ref": approval.criteria_ref,
            "criteria_sha256": approval.criteria_sha256,
            "criteria_record_hash": approval.criteria_record_hash,
            "committed_criteria_id": criteria.criteria_id,
            "committed_criteria_sha256": criteria.criteria_sha256,
            "committed_criteria_record_hash": criteria.criteria_record_hash,
        }),
    };
    let engine = sea_forge_authority::PolicyAuthorityEngine::new(bundle.clone())?;
    let policy_record = stream.commit_typed("authority_policy", vec![], &bundle, vec![])?;
    let evidence = stream.commit_typed(
        "authority_evidence",
        vec![approval.approval_id.clone()],
        &json!({"kind":"approval_resolution_authority_request", "action": action}),
        vec![policy_record.entry_ulid().into()],
    )?;
    // Derive the sequence from existing decisions so the approval-resolution
    // decision_id (auth_{sequence:02}) never collides with an earlier escalate
    // decision in the same case ledger.
    let sequence = stream
        .read_entries()?
        .iter()
        .filter(|entry| entry.record_kind == "authority_decision")
        .count()
        + 1;
    let decision = engine.evaluate(sea_forge_authority::AuthorityEvaluation {
        actor: &actor,
        binding: bundle.resolve_identity(actor_id, role),
        run_id: &approval.run_id,
        case_id,
        plan_item_id: &approval.plan_item_id,
        sequence,
        action: &action,
        workspace_root: root,
        evidence_refs: vec![evidence.entry_ulid().into(), original.decision_id.clone()],
        artifacts_root: None,
        timeout_secs: None,
        env_keys: Default::default(),
        domainforge_candidate: None,
        environment: None,
    })?;
    stream.commit_typed(
        "authority_decision",
        vec![case_id.into(), approval.approval_id.clone()],
        &decision,
        vec![evidence.entry_ulid().into()],
    )?;
    if decision.verdict != Verdict::Allow {
        return Err(ForgeError::Input(
            "approval resolution is not authorized by policy".into(),
        ));
    }
    Ok(())
}
