//! CEP authority loop, SEA-Forge side (CEP/world_ref migration, Stage 6).
//!
//! A Cognate `authority_request` envelope is parsed fail-closed, its pinned
//! world must be registered (Stage 4), and the operation is evaluated by the
//! real [`PolicyAuthorityEngine`] as a reserved `cognate_action` /
//! `cognate_capability` surface. The resulting [`AuthorityDecision`] is
//! committed append-only before it is returned as a CEP `authority_decision`
//! envelope. Dispositions are never collapsed to a boolean, and boundary /
//! degraded decisions keep their constraints, policy basis and controls.
//!
//! The evaluated actor is the *verified caller* supplied by the server's
//! identity gate. The envelope's subject is recorded, never trusted for role.

use crate::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use chrono::{DateTime, Duration, Utc};
use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::{
    Actor, AuthorityAction, AuthorityDecision, NormalizedDisposition, Verdict,
};
use sea_forge_domainforge::WorldRegistry;
use sea_forge_ledger::{CommittedRecordRef, LedgerEntry, LedgerStream};
use serde_json::{json, Value};
use std::fmt;
use std::path::Path;

pub const REQUEST_PROFILE: &str = "godspeed.authority_request";
pub const DECISION_PROFILE: &str = "godspeed.authority_decision";
pub const PROFILE_VERSION: &str = "1.0.0";
pub const MAX_ENVELOPE_BYTES: usize = 64 * 1024;
pub const LEDGER_ID: &str = "cognate-authority";

const FIELD_MAX: usize = 256;

/// Why a request was refused before any decision existed. Never an allow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CepAuthorityError {
    Oversize(usize),
    Malformed(&'static str),
    WrongKind(String),
    WrongProfile(String),
    World(String),
    /// An approval that does not exist, is not approved, expired, was used, or names another operation.
    Approval(String),
    Internal(String),
}

impl fmt::Display for CepAuthorityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Oversize(n) => write!(
                f,
                "authority request is {n} bytes; limit {MAX_ENVELOPE_BYTES}"
            ),
            Self::Malformed(what) => write!(f, "malformed authority request: {what}"),
            Self::WrongKind(k) => write!(f, "expected envelope_kind authority_request, got {k}"),
            Self::WrongProfile(p) => write!(
                f,
                "expected profile {REQUEST_PROFILE}@{PROFILE_VERSION}, got {p}"
            ),
            Self::World(m) => write!(f, "world refused: {m}"),
            Self::Approval(m) => write!(f, "approval refused: {m}"),
            Self::Internal(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for CepAuthorityError {}

impl From<ForgeError> for CepAuthorityError {
    fn from(e: ForgeError) -> Self {
        Self::Internal(e.to_string())
    }
}

impl CepAuthorityError {
    pub fn class(&self) -> &'static str {
        match self {
            Self::Oversize(_) => "cep_request_oversize",
            Self::Malformed(_) | Self::WrongKind(_) | Self::WrongProfile(_) => {
                "cep_request_invalid"
            }
            Self::World(_) => "cep_world_refused",
            Self::Approval(_) => "cep_approval_refused",
            Self::Internal(_) => "cep_authority_internal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationKind {
    Action,
    Capability,
}

impl OperationKind {
    fn resource_type(&self) -> &'static str {
        match self {
            Self::Action => "cognate_action",
            Self::Capability => "cognate_capability",
        }
    }
}

/// The validated, typed content of an `authority_request` envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRequest {
    pub envelope_id: String,
    pub world_ref: String,
    pub operation_id: String,
    pub correlation_id: String,
    pub subject_actor_ref: String,
    pub kind: OperationKind,
    pub operation_name: String,
    pub resource_id: String,
    /// Set when this request revalidates an escalation that has since been approved.
    pub approval_ref: Option<String>,
    pub lineage_refs: Vec<String>,
}

fn text<'a>(
    value: &'a Value,
    pointer: &str,
    what: &'static str,
) -> Result<&'a str, CepAuthorityError> {
    let s = value
        .pointer(pointer)
        .and_then(Value::as_str)
        .ok_or(CepAuthorityError::Malformed(what))?;
    if s.is_empty() || s.len() > FIELD_MAX || s.chars().any(char::is_control) {
        return Err(CepAuthorityError::Malformed(what));
    }
    Ok(s)
}

/// Parse an `authority_request`. World binding is checked by the caller's registry.
pub fn parse_request(
    envelope: &Value,
    worlds: &WorldRegistry,
) -> Result<ParsedRequest, CepAuthorityError> {
    let size = serde_json::to_vec(envelope)
        .map(|v| v.len())
        .unwrap_or(usize::MAX);
    if size > MAX_ENVELOPE_BYTES {
        return Err(CepAuthorityError::Oversize(size));
    }
    let kind = text(envelope, "/envelope_kind", "envelope_kind")?;
    if kind != "authority_request" {
        return Err(CepAuthorityError::WrongKind(kind.to_string()));
    }
    let profile = format!(
        "{}@{}",
        text(
            envelope,
            "/extensions/cep.profile/profile_id",
            "extensions.cep.profile.profile_id"
        )?,
        text(
            envelope,
            "/extensions/cep.profile/profile_version",
            "extensions.cep.profile.profile_version"
        )?
    );
    if profile != format!("{REQUEST_PROFILE}@{PROFILE_VERSION}") {
        return Err(CepAuthorityError::WrongProfile(profile));
    }
    let world_ref = text(envelope, "/scope/world_ref", "scope.world_ref")?;
    let (world, _) = worlds
        .require(world_ref)
        .map_err(|e| CepAuthorityError::World(e.to_string()))?;
    let ns = "/extensions/godspeed.authority_request";
    let operation_kind = match text(envelope, &format!("{ns}/operation_kind"), "operation_kind")? {
        "action" => OperationKind::Action,
        "capability" => OperationKind::Capability,
        _ => {
            return Err(CepAuthorityError::Malformed(
                "operation_kind must be action or capability",
            ))
        }
    };
    Ok(ParsedRequest {
        envelope_id: text(envelope, "/envelope_id", "envelope_id")?.to_string(),
        world_ref: world.to_string(),
        operation_id: text(envelope, &format!("{ns}/operation_id"), "operation_id")?.to_string(),
        correlation_id: text(envelope, &format!("{ns}/correlation_id"), "correlation_id")?
            .to_string(),
        subject_actor_ref: text(
            envelope,
            "/authority/0/subject_actor_ref",
            "authority[0].subject_actor_ref",
        )?
        .to_string(),
        kind: operation_kind,
        operation_name: text(envelope, &format!("{ns}/operation_name"), "operation_name")?
            .to_string(),
        resource_id: text(envelope, &format!("{ns}/resource_id"), "resource_id")?.to_string(),
        approval_ref: match envelope.pointer(&format!("{ns}/approval_id")) {
            None | Some(Value::Null) => None,
            Some(_) => {
                Some(text(envelope, &format!("{ns}/approval_id"), "approval_id")?.to_string())
            }
        },
        lineage_refs: envelope
            .get("lineage_refs")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
    })
}

#[derive(Debug)]
pub struct CepDecision {
    pub envelope: Value,
    pub decision: AuthorityDecision,
    pub committed: CommittedRecordRef,
}

pub struct CepAuthorityService<'a> {
    pub engine: &'a PolicyAuthorityEngine,
    pub bundle: &'a AuthorityPolicyBundle,
    pub worlds: &'a WorldRegistry,
    pub root: &'a Path,
    /// How long an escalation stays approvable.
    pub approval_ttl: Duration,
}

/// Where an escalation stands, derived from the append-only ledger and nothing else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalState {
    Pending,
    Approved,
    Rejected,
    Expired,
    Used,
}

impl ApprovalState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
            Self::Used => "used",
        }
    }
}

/// One escalation and what has happened to it since.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalStanding {
    pub approval_id: String,
    pub state: ApprovalState,
    pub requester: String,
    pub world_ref: String,
    pub kind: String,
    pub operation_name: String,
    pub resource_id: String,
    pub subject_actor_ref: String,
    pub original_request_envelope_id: String,
    pub escalation_entry: String,
    pub expires_at: DateTime<Utc>,
    pub resolved_by: Option<String>,
}

impl ApprovalStanding {
    pub fn to_json(&self) -> Value {
        json!({
            "approval_id": self.approval_id,
            "state": self.state.as_str(),
            "requester": self.requester,
            "world_ref": self.world_ref,
            "operation_kind": self.kind,
            "operation_name": self.operation_name,
            "resource_id": self.resource_id,
            "subject_actor_ref": self.subject_actor_ref,
            "request_ref": self.original_request_envelope_id,
            "expires_at": self.expires_at.to_rfc3339(),
            "resolved_by": self.resolved_by,
        })
    }
}

fn approval_id_for(operation_id: &str) -> String {
    format!("apr-{operation_id}")
}

fn escalation_key(operation_id: &str) -> String {
    format!("cep:{operation_id}")
}

fn resolve_key(approval_id: &str) -> String {
    format!("cep-approval-resolve:{approval_id}")
}

fn use_key(approval_id: &str) -> String {
    format!("cep-approval-use:{approval_id}")
}

/// Only a policy-driven escalation (a rule that requires approval) can be satisfied by one. Escalations
/// caused by unresolved identity, an active opaque constraint, or an unavailable engine cannot.
fn approvable(decision: &AuthorityDecision) -> bool {
    decision.verdict == Verdict::Escalate
        && decision.reason_codes.iter().any(|c| c == "policy_escalate")
        && decision
            .reason_codes
            .iter()
            .all(|c| matches!(c.as_str(), "policy_escalate" | "candidate_resolution"))
}

fn text_at(value: &Value, pointer: &str) -> String {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn find_entry<'e>(entries: &'e [LedgerEntry], key: &str) -> Option<&'e LedgerEntry> {
    entries
        .iter()
        .find(|e| e.idempotency_key.as_deref() == Some(key))
}

impl CepAuthorityService<'_> {
    fn ledger(&self) -> Result<LedgerStream, CepAuthorityError> {
        Ok(LedgerStream::open(
            self.root,
            LEDGER_ID,
            "sea-forge-cep-authority",
        )?)
    }

    /// The standing of one approval, from the ledger. `None` if no such approvable escalation exists.
    fn standing_in(
        &self,
        entries: &[LedgerEntry],
        approval_id: &str,
        now: DateTime<Utc>,
    ) -> Option<ApprovalStanding> {
        let operation_id = approval_id.strip_prefix("apr-")?;
        let entry = find_entry(entries, &escalation_key(operation_id))?;
        let decision: AuthorityDecision = serde_json::from_value(entry.payload.clone()).ok()?;
        if !approvable(&decision) {
            return None;
        }
        let action = serde_json::to_value(&decision.action_request.action).ok()?;
        let committed_at = DateTime::parse_from_rfc3339(&entry.committed_at)
            .ok()?
            .with_timezone(&Utc);
        let expires_at = committed_at + self.approval_ttl;
        let resolution = find_entry(entries, &resolve_key(approval_id));
        let state = if find_entry(entries, &use_key(approval_id)).is_some() {
            ApprovalState::Used
        } else if let Some(resolved) = resolution {
            match resolved
                .payload
                .pointer("/action_request/action/parameters/resolution")
                .and_then(Value::as_str)
            {
                Some("approved") => ApprovalState::Approved,
                _ => ApprovalState::Rejected,
            }
        } else {
            ApprovalState::Pending
        };
        // An approval nobody acted on in time lapses; one that was granted stays granted until used or its
        // own window closes, so an approval cannot outlive its window either.
        let state = match state {
            ApprovalState::Pending | ApprovalState::Approved if now >= expires_at => {
                ApprovalState::Expired
            }
            other => other,
        };
        Some(ApprovalStanding {
            approval_id: approval_id.to_string(),
            state,
            requester: text_at(&entry.payload, "/action_request/actor/actor_id"),
            world_ref: text_at(&action, "/parameters/world_ref"),
            kind: text_at(&action, "/resource_type"),
            operation_name: text_at(&action, "/parameters/operation_name"),
            resource_id: text_at(&action, "/resource_id"),
            subject_actor_ref: text_at(&action, "/parameters/subject_actor_ref"),
            original_request_envelope_id: decision
                .audit_record
                .evidence_refs
                .iter()
                .find_map(|r| r.strip_prefix("cep:"))
                .unwrap_or_default()
                .to_string(),
            escalation_entry: entry.entry_ulid.clone(),
            expires_at,
            resolved_by: resolution.map(|r| text_at(&r.payload, "/action_request/actor/actor_id")),
        })
    }

    /// Every escalation that can be approved, with its current standing.
    pub fn list_approvals(&self) -> Result<Vec<ApprovalStanding>, CepAuthorityError> {
        let entries = self.ledger()?.read_entries()?;
        let now = Utc::now();
        Ok(entries
            .iter()
            .filter_map(|e| {
                e.idempotency_key
                    .as_deref()?
                    .strip_prefix("cep:")
                    .map(approval_id_for)
            })
            .filter_map(|id| self.standing_in(&entries, &id, now))
            .collect())
    }

    fn evaluate(
        &self,
        request: &ParsedRequest,
        caller: &Actor,
    ) -> Result<AuthorityDecision, CepAuthorityError> {
        let action = AuthorityAction::Reserved {
            resource_type: request.kind.resource_type().to_string(),
            resource_id: request.resource_id.clone(),
            parameters: json!({
                "operation_name": request.operation_name,
                "operation_id": request.operation_id,
                "world_ref": request.world_ref,
                "subject_actor_ref": request.subject_actor_ref,
                "request_envelope_id": request.envelope_id,
            }),
        };
        let binding = self
            .bundle
            .resolve_identity(&caller.actor_id, caller.role.clone());
        Ok(self.engine.evaluate(AuthorityEvaluation {
            actor: caller,
            binding,
            run_id: &request.correlation_id,
            case_id: "cognate",
            plan_item_id: &request.operation_id,
            sequence: 1,
            action: &action,
            workspace_root: self.root,
            evidence_refs: vec![format!("cep:{}", request.envelope_id)],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            environment: None,
        })?)
    }

    /// Decide one request as the verified `caller`. The decision is committed
    /// before it is returned. A request that carries an `approval_id` revalidates
    /// an earlier escalation: the approval must exist, be approved, unexpired,
    /// unused and for exactly this operation, and policy is evaluated again now.
    pub fn decide(
        &self,
        envelope: &Value,
        caller: &Actor,
    ) -> Result<CepDecision, CepAuthorityError> {
        let request = parse_request(envelope, self.worlds)?;
        let mut decision = self.evaluate(&request, caller)?;
        let ledger = self.ledger()?;
        let mut note = ApprovalNote::default();

        if let Some(approval_id) = &request.approval_ref {
            let entries = ledger.read_entries()?;
            let standing = self
                .standing_in(&entries, approval_id, Utc::now())
                .ok_or_else(|| {
                    CepAuthorityError::Approval(format!(
                        "{approval_id} is not an approvable escalation"
                    ))
                })?;
            if standing.state != ApprovalState::Approved {
                return Err(CepAuthorityError::Approval(format!(
                    "{approval_id} is {}, not approved",
                    standing.state.as_str()
                )));
            }
            let same = standing.world_ref == request.world_ref
                && standing.kind == request.kind.resource_type()
                && standing.operation_name == request.operation_name
                && standing.resource_id == request.resource_id
                && standing.subject_actor_ref == request.subject_actor_ref
                && standing.requester == caller.actor_id;
            if !same {
                return Err(CepAuthorityError::Approval(format!(
                    "{approval_id} was granted for a different operation, subject, world or requester"
                )));
            }
            if !request
                .lineage_refs
                .contains(&standing.original_request_envelope_id)
            {
                return Err(CepAuthorityError::Approval(
                    "request lineage does not include the escalated request".into(),
                ));
            }
            note.approval_id = Some(approval_id.clone());
            note.approved_by = standing.resolved_by.clone();
            note.lineage
                .push(standing.original_request_envelope_id.clone());
            // An approval satisfies a policy-driven escalation. It never overrides a deny, and it adds
            // nothing when policy now allows outright.
            if approvable(&decision) {
                decision.verdict = Verdict::Allow;
                decision.outcome = Verdict::Allow;
                decision.normalized_disposition = NormalizedDisposition::Allow;
                decision.reason_codes = vec!["approved_escalation".into()];
                decision.reason = "approved_escalation".into();
                decision.required_next_steps.clear();
                decision.approval_request_id = None;
                decision.policy_refs.push(format!("approval:{approval_id}"));
                let committed = ledger
                    .commit_typed_new(
                        "authority_decision",
                        use_key(approval_id),
                        vec!["cognate".into(), request.operation_id.clone()],
                        &decision,
                        vec![standing.escalation_entry.clone()],
                    )
                    .map_err(|_| {
                        CepAuthorityError::Approval(format!("{approval_id} was already used"))
                    })?;
                note.consumed = true;
                let out = decision_envelope(envelope, &request, &decision, &committed, &note)?;
                return Ok(CepDecision {
                    envelope: out,
                    decision,
                    committed,
                });
            }
        }

        if approvable(&decision) {
            let approval_id = approval_id_for(&request.operation_id);
            note.approval_id = Some(approval_id);
            note.issued_expires_at = Some(
                (Utc::now() + self.approval_ttl)
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string(),
            );
        }
        let committed = ledger.commit_typed_once(
            "authority_decision",
            escalation_key(&request.operation_id),
            vec!["cognate".into(), request.operation_id.clone()],
            &decision,
            vec![],
        )?;
        let out = decision_envelope(envelope, &request, &decision, &committed, &note)?;
        Ok(CepDecision {
            envelope: out,
            decision,
            committed,
        })
    }

    /// Resolve an escalation as the verified `resolver`. The policy engine decides whether this actor may
    /// resolve it (a rule for `approval_resolution`; the requester can never resolve their own request).
    pub fn resolve_approval(
        &self,
        approval_id: &str,
        approve: bool,
        resolver: &Actor,
    ) -> Result<Value, CepAuthorityError> {
        let ledger = self.ledger()?;
        let entries = ledger.read_entries()?;
        let standing = self
            .standing_in(&entries, approval_id, Utc::now())
            .ok_or_else(|| {
                CepAuthorityError::Approval(format!(
                    "{approval_id} is not an approvable escalation"
                ))
            })?;
        if standing.state != ApprovalState::Pending {
            return Err(CepAuthorityError::Approval(format!(
                "{approval_id} is {}, not pending",
                standing.state.as_str()
            )));
        }
        let resolution = if approve { "approved" } else { "rejected" };
        let action = AuthorityAction::Reserved {
            resource_type: "approval_resolution".into(),
            resource_id: approval_id.to_string(),
            parameters: json!({
                "approval_id": approval_id,
                "original_decision_id": standing.escalation_entry,
                "requester_id": standing.requester,
                "resolution": resolution,
                "required_approver_roles": [],
            }),
        };
        let binding = self
            .bundle
            .resolve_identity(&resolver.actor_id, resolver.role.clone());
        let decision = self.engine.evaluate(AuthorityEvaluation {
            actor: resolver,
            binding,
            run_id: approval_id,
            case_id: "cognate",
            plan_item_id: approval_id,
            sequence: 1,
            action: &action,
            workspace_root: self.root,
            evidence_refs: vec![format!("approval:{approval_id}")],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            environment: None,
        })?;
        if decision.verdict != Verdict::Allow {
            // The attempt is governance-relevant: record the refusal, then refuse.
            ledger.commit_typed(
                "authority_decision",
                vec!["cognate".into(), approval_id.to_string()],
                &decision,
                vec![standing.escalation_entry.clone()],
            )?;
            return Err(CepAuthorityError::Approval(format!(
                "{} may not resolve {approval_id}: {}",
                resolver.actor_id, decision.reason
            )));
        }
        let committed = ledger
            .commit_typed_new(
                "authority_decision",
                resolve_key(approval_id),
                vec!["cognate".into(), approval_id.to_string()],
                &decision,
                vec![standing.escalation_entry.clone()],
            )
            .map_err(|_| {
                CepAuthorityError::Approval(format!("{approval_id} was already resolved"))
            })?;
        Ok(json!({
            "ok": true,
            "approval_id": approval_id,
            "state": resolution,
            "resolved_by": resolver.actor_id,
            "ledger_entry": committed.entry_ulid(),
        }))
    }
}

/// What the decision envelope must say about an approval.
#[derive(Default)]
struct ApprovalNote {
    approval_id: Option<String>,
    /// Escalation: when the approval window closes.
    issued_expires_at: Option<String>,
    approved_by: Option<String>,
    consumed: bool,
    lineage: Vec<String>,
}

/// The CEP disposition a committed decision expresses.
pub fn cep_disposition(decision: &AuthorityDecision) -> &'static str {
    match (&decision.verdict, &decision.normalized_disposition) {
        (Verdict::Deny, _) => "deny",
        (Verdict::Escalate, _) => "escalate",
        (Verdict::Allow, NormalizedDisposition::Boundary) => "boundary",
        (Verdict::Allow, NormalizedDisposition::Degraded) => "degraded",
        (Verdict::Allow, NormalizedDisposition::Allow) => "allow",
        // An Allow verdict carrying Deny/Escalate normalization is incoherent: refuse.
        (Verdict::Allow, _) => "unknown",
    }
}

fn decision_envelope(
    request_envelope: &Value,
    request: &ParsedRequest,
    decision: &AuthorityDecision,
    committed: &CommittedRecordRef,
    note: &ApprovalNote,
) -> Result<Value, CepAuthorityError> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let disposition = cep_disposition(decision);
    let mut ext = serde_json::Map::new();
    ext.insert("decision".into(), json!(disposition));
    ext.insert("operation_id".into(), json!(request.operation_id));
    ext.insert("request_ref".into(), json!(request.envelope_id));
    ext.insert("decision_id".into(), json!(decision.decision_id));
    ext.insert("ledger_entry".into(), json!(committed.entry_ulid()));
    ext.insert(
        "policy_bundle_hash".into(),
        json!(decision.determinism.policy_bundle_hash),
    );
    if let Some(approval_id) = &note.approval_id {
        ext.insert("approval_id".into(), json!(approval_id));
        if let Some(expires) = &note.issued_expires_at {
            ext.insert("approval_expires_at".into(), json!(expires));
        }
        if note.consumed {
            ext.insert("approval_used".into(), json!(true));
            ext.insert("approved_by".into(), json!(note.approved_by));
        }
    }
    let basis: Vec<&String> = decision.policy_refs.iter().collect();
    let mut constraints: Vec<Value> = Vec::new();
    if matches!(disposition, "boundary" | "degraded") {
        for (n, (key, values)) in decision.boundary_constraints.iter().enumerate() {
            constraints.push(json!({
                "constraint_id": format!("con-{}-{}", decision.decision_id, n + 1),
                "constraint_type": key,
                "description": values.join(","),
            }));
        }
        if basis.is_empty() || constraints.is_empty() {
            // A constrained disposition without its constraints or basis is not expressible: refuse.
            ext.insert("decision".into(), json!("unknown"));
            constraints.clear();
        } else {
            ext.insert("policy_basis".into(), json!(basis));
        }
        if disposition == "degraded" && ext["decision"] == json!("degraded") {
            if decision.compensating_controls.is_empty() {
                ext.insert("decision".into(), json!("unknown"));
                ext.remove("policy_basis");
                constraints.clear();
            } else {
                ext.insert(
                    "compensating_controls".into(),
                    json!(decision.compensating_controls),
                );
            }
        }
    }
    let mut scope = request_envelope
        .get("scope")
        .cloned()
        .unwrap_or_else(|| json!({}));
    if let Some(o) = scope.as_object_mut() {
        o.insert(
            "evaluation_context".into(),
            json!("godspeed-authority_decision"),
        );
    }
    let mut out = json!({
        "envelope_id": format!("env-authority_decision-{}", request.operation_id),
        "cep_version": "1.0.0",
        "envelope_version": "1.0",
        "envelope_kind": "authority_decision",
        "created_at": now,
        "created_by": "sea-forge-authority",
        "scope": scope,
        "boundary_record": {
            "scope": "authority decision for one Cognate operation",
            "included_sections": ["authority", "transformations"],
            "excluded_sections": [],
            "known_omissions": ["policy evaluation detail is in the committed ledger record"],
            "unknowns": [],
            "redactions": [],
            "compression_notes": [],
            "out_of_scope_entities": [],
            "classification": "internal",
            "limitations": [],
        },
        "completeness_status": "partial",
        "omission_status": "marked",
        "provenance_refs": [format!("prov-{}", request.operation_id)],
        "validation_status": "valid",
        "lineage_refs": std::iter::once(request.envelope_id.clone()).chain(note.lineage.iter().cloned()).collect::<Vec<_>>(),
        "provenance": [{
            "provenance_id": format!("prov-{}", request.operation_id),
            "source_system_refs": ["sea-forge"],
            "producer_refs": ["sea-forge-authority"],
            "production_method": "authority_evaluation",
            "created_at": now,
        }],
        "transformations": request_envelope.get("transformations").cloned().unwrap_or_else(|| json!([])),
        "authority": [{
            "authority_id": decision.decision_id,
            "subject_actor_ref": request.subject_actor_ref,
        }],
        "extensions": {
            "cep.profile": {"profile_id": DECISION_PROFILE, "profile_version": PROFILE_VERSION},
            "godspeed.authority_decision": Value::Object(ext),
        },
    });
    if !constraints.is_empty() {
        out["constraints"] = Value::Array(constraints);
    }
    Ok(out)
}
