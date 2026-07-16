use chrono::Utc;
use sea_forge_core::{
    errors::ForgeError,
    types::{
        Actor, ActorRole, ApprovalRequest, ApprovalStatus, AuthorityAction, AuthorityDecision,
        NormalizedDisposition, SettlementCriteriaRecord, Verdict,
    },
};
use sea_forge_ledger::LedgerStream;
use serde_json::{json, Value};
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
        opts.policy,
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
        opts.policy,
        false,
    )
}

fn resolve(
    root: &Path,
    case_id: &str,
    approval_id: &str,
    actor: &str,
    note: Option<&str>,
    policy: Option<&Path>,
    approved: bool,
) -> Result<u8, ForgeError> {
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
        &stream,
        root,
        policy.ok_or_else(|| ForgeError::Input("approval policy is missing".into()))?,
        actor,
        case_id,
        &latest,
        &decision,
        &criteria,
        approved,
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
        approvals::append(root, &expired)?;
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
    approvals::append(root, &resolved)?;
    println!("approval_id={approval_id}");
    println!("status={:?}", resolved.status);
    println!("resolved_by={actor}");
    Ok(0)
}

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
    let decision = engine.evaluate(sea_forge_authority::AuthorityEvaluation {
        actor: &actor,
        binding: bundle.resolve_identity(actor_id, role),
        run_id: &approval.run_id,
        case_id,
        plan_item_id: &approval.plan_item_id,
        sequence: 1,
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
