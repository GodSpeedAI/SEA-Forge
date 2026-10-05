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
/// Evidence and settlement packets may carry many items; still bounded.
pub const MAX_PACKET_BYTES: usize = 256 * 1024;

/// Settlement criteria, declared in cell configuration and bound into every allow decision when it is made,
/// so settlement cannot later be judged against different criteria than the ones in force at decision time.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SettlementCriteria {
    /// Supporting evidence items needed (at or above `min_reliability`) for a question to settle.
    pub min_supporting: u32,
    /// `low` < `medium` < `high`.
    pub min_reliability: String,
}

impl Default for SettlementCriteria {
    fn default() -> Self {
        Self {
            min_supporting: 1,
            min_reliability: "medium".into(),
        }
    }
}

fn reliability_rank(value: &str) -> Option<u8> {
    match value {
        "low" => Some(1),
        "medium" => Some(2),
        "high" => Some(3),
        _ => None,
    }
}

impl SettlementCriteria {
    pub fn sha256(&self) -> String {
        use sha2::{Digest, Sha256};
        let bytes = serde_json::to_vec(&json!({
            "min_supporting": self.min_supporting,
            "min_reliability": self.min_reliability,
        }))
        .unwrap_or_default();
        format!("sha256:{:x}", Sha256::digest(bytes))
    }
    /// Valid criteria: at least one supporting item and a known reliability level.
    pub fn validate(&self) -> Result<(), String> {
        if self.min_supporting == 0 {
            return Err("min_supporting must be at least 1".into());
        }
        if reliability_rank(&self.min_reliability).is_none() {
            return Err("min_reliability must be low, medium or high".into());
        }
        Ok(())
    }
}

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
    /// Evidence or settlement refused: broken chain, wrong world, no allow decision, criteria changed, policy.
    Evidence(String),
    /// A world transition whose facts do not hold: unregistered or identical worlds, or claims that
    /// contradict what SEA-Forge recomputes from the registered identities.
    Transition(String),
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
            Self::Evidence(m) => write!(f, "evidence or settlement refused: {m}"),
            Self::Transition(m) => write!(f, "world transition refused: {m}"),
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
            Self::Evidence(_) => "cep_evidence_refused",
            Self::Transition(_) => "cep_transition_refused",
            Self::Internal(_) => "cep_authority_internal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationKind {
    Action,
    Capability,
    /// Moves work from one immutable semantic world to another (Stage 10).
    WorldTransition,
}

impl OperationKind {
    fn resource_type(&self) -> &'static str {
        match self {
            Self::Action => "cognate_action",
            Self::Capability => "cognate_capability",
            Self::WorldTransition => "world_transition",
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
    /// The ledger entry of the allowed world transition that licenses a lineage reaching across worlds. It is the
    /// entry (a unique ULID), not the `decision_id`, which is only unique within one operation.
    pub transition_ref: Option<String>,
    /// Set for `OperationKind::WorldTransition`: the facts SEA-Forge recomputed, not the sender's claims.
    pub transition: Option<VerifiedTransition>,
}

pub const TRANSITION_KINDS: [&str; 4] = [
    "semantic_change",
    "source_edit_only",
    "compiler_upgrade",
    "migration",
];
pub const COMPATIBILITIES: [&str; 4] = ["compatible", "backward_compatible", "breaking", "unknown"];
const TRANSITION_NS: &str = "/extensions/godspeed.world_transition";

/// A requested move between two registered worlds, with the facts SEA-Forge verified itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedTransition {
    pub source_world_ref: String,
    pub target_world_ref: String,
    pub transition_kind: String,
    pub reason: String,
    /// Recorded as `compatible` when the closures are equal (provable). Otherwise the sender's claim, which
    /// SEA-Forge cannot check (DomainForge has no world-level diff); `unknown` when none was given.
    pub compatibility: String,
    /// Recomputed from the two registered identities, never taken from the request.
    pub semantic_closure_equal: bool,
    pub semantic_diff_ref: Option<String>,
    pub migration_ref: Option<String>,
    pub evidence_refs: Vec<String>,
}

impl VerifiedTransition {
    /// A move across a semantic change is never allowed without a human approval, whatever policy says.
    pub fn needs_approval(&self) -> bool {
        !self.semantic_closure_equal
    }

    /// The record, shaped exactly as cep's `world-transition.v1` (the schema forbids extra fields).
    pub fn record(&self, authority_decision_ref: Option<&str>) -> Value {
        let mut rec = json!({
            "source_world_ref": self.source_world_ref,
            "target_world_ref": self.target_world_ref,
            "transition_kind": self.transition_kind,
            "reason": self.reason,
            "compatibility": self.compatibility,
            "semantic_closure_equal": self.semantic_closure_equal,
        });
        if let Some(r) = &self.semantic_diff_ref {
            rec["semantic_diff_ref"] = json!(r);
        }
        if let Some(r) = &self.migration_ref {
            rec["migration_ref"] = json!(r);
        }
        if let Some(r) = authority_decision_ref {
            rec["authority_decision_ref"] = json!(r);
        }
        if !self.evidence_refs.is_empty() {
            rec["evidence_refs"] = json!(self.evidence_refs);
        }
        rec
    }
}

fn optional_text(
    envelope: &Value,
    pointer: &str,
    what: &'static str,
) -> Result<Option<String>, CepAuthorityError> {
    match envelope.pointer(pointer) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => Ok(Some(text(envelope, pointer, what)?.to_string())),
    }
}

/// Parse and verify the transition a `world_transition` request asks for. Every refusal happens before any
/// ledger write.
fn verify_transition(
    envelope: &Value,
    source_world_ref: &str,
    worlds: &WorldRegistry,
) -> Result<VerifiedTransition, CepAuthorityError> {
    let target = text(
        envelope,
        &format!("{TRANSITION_NS}/target_world_ref"),
        "world_transition.target_world_ref",
    )?;
    let (source_world, source_identity) = worlds
        .require(source_world_ref)
        .map_err(|e| CepAuthorityError::Transition(format!("source world: {e}")))?;
    let (target_world, target_identity) = worlds
        .require(target)
        .map_err(|e| CepAuthorityError::Transition(format!("target world: {e}")))?;
    if source_world.to_string() == target_world.to_string() {
        return Err(CepAuthorityError::Transition(
            "source and target are the same world; a transition must move somewhere".into(),
        ));
    }
    let closure_equal =
        source_identity.semantic_closure_hash == target_identity.semantic_closure_hash;
    match envelope.pointer(&format!("{TRANSITION_NS}/semantic_closure_equal")) {
        None | Some(Value::Null) => {}
        Some(Value::Bool(claimed)) if *claimed == closure_equal => {}
        Some(_) => {
            return Err(CepAuthorityError::Transition(
                "the claimed semantic_closure_equal contradicts the registered worlds".into(),
            ))
        }
    }
    let kind = text(
        envelope,
        &format!("{TRANSITION_NS}/transition_kind"),
        "world_transition.transition_kind",
    )?;
    if !TRANSITION_KINDS.contains(&kind) {
        return Err(CepAuthorityError::Malformed(
            "world_transition.transition_kind is not a known kind",
        ));
    }
    let misdeclared = match kind {
        "source_edit_only" | "compiler_upgrade" => !closure_equal,
        "semantic_change" => closure_equal,
        _ => false,
    };
    if misdeclared {
        return Err(CepAuthorityError::Transition(format!(
            "transition_kind {kind} contradicts the registered worlds (semantic closure {})",
            if closure_equal { "equal" } else { "differs" }
        )));
    }
    let claimed = optional_text(
        envelope,
        &format!("{TRANSITION_NS}/compatibility"),
        "world_transition.compatibility",
    )?;
    if let Some(c) = &claimed {
        if !COMPATIBILITIES.contains(&c.as_str()) {
            return Err(CepAuthorityError::Malformed(
                "world_transition.compatibility is not a known value",
            ));
        }
    }
    let compatibility = if closure_equal {
        "compatible".to_string()
    } else {
        claimed.unwrap_or_else(|| "unknown".into())
    };
    let evidence_refs = match envelope.pointer(&format!("{TRANSITION_NS}/evidence_refs")) {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(items)) if items.len() <= 32 => items
            .iter()
            .map(|v| {
                v.as_str()
                    .filter(|s| !s.is_empty() && s.len() <= FIELD_MAX)
                    .map(str::to_string)
                    .ok_or(CepAuthorityError::Malformed(
                        "world_transition.evidence_refs must be short non-empty strings",
                    ))
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => {
            return Err(CepAuthorityError::Malformed(
                "world_transition.evidence_refs must be an array of at most 32",
            ))
        }
    };
    Ok(VerifiedTransition {
        source_world_ref: source_world.to_string(),
        target_world_ref: target_world.to_string(),
        transition_kind: kind.to_string(),
        reason: text(
            envelope,
            &format!("{TRANSITION_NS}/reason"),
            "world_transition.reason",
        )?
        .to_string(),
        compatibility,
        semantic_closure_equal: closure_equal,
        semantic_diff_ref: optional_text(
            envelope,
            &format!("{TRANSITION_NS}/semantic_diff_ref"),
            "world_transition.semantic_diff_ref",
        )?,
        migration_ref: optional_text(
            envelope,
            &format!("{TRANSITION_NS}/migration_ref"),
            "world_transition.migration_ref",
        )?,
        evidence_refs,
    })
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
        "world_transition" => OperationKind::WorldTransition,
        _ => {
            return Err(CepAuthorityError::Malformed(
                "operation_kind must be action, capability or world_transition",
            ))
        }
    };
    let transition = if operation_kind == OperationKind::WorldTransition {
        let t = verify_transition(envelope, &world.to_string(), worlds)?;
        // An approval is bound to resource_id, so for a transition it must be the target world: an
        // approval for one move can never be spent on another.
        if text(envelope, &format!("{ns}/operation_name"), "operation_name")? != "transition" {
            return Err(CepAuthorityError::Malformed(
                "operation_name must be transition for a world_transition",
            ));
        }
        if text(envelope, &format!("{ns}/resource_id"), "resource_id")? != t.target_world_ref {
            return Err(CepAuthorityError::Malformed(
                "resource_id must be the target world_ref for a world_transition",
            ));
        }
        Some(t)
    } else {
        None
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
        transition_ref: match envelope.pointer(&format!("{ns}/transition_ref")) {
            None | Some(Value::Null) => None,
            Some(_) => {
                Some(text(envelope, &format!("{ns}/transition_ref"), "transition_ref")?.to_string())
            }
        },
        transition,
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
    pub criteria: SettlementCriteria,
}

/// One allowed world transition. `record` validates against cep's `world-transition.v1`.
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionFact {
    pub operation_id: String,
    pub decision_id: String,
    pub requester: String,
    /// Set when a human approval was spent on this move.
    pub approval_id: Option<String>,
    pub ledger_entry: String,
    pub record: Value,
}

impl TransitionFact {
    pub fn to_json(&self) -> Value {
        json!({
            "operation_id": self.operation_id,
            "decision_id": self.decision_id,
            "requester": self.requester,
            "approval_id": self.approval_id,
            "ledger_entry": self.ledger_entry,
            "record": self.record,
        })
    }
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

    /// Every world transition that was allowed, oldest first, read back from the ledger and nothing else.
    /// A transition exists exactly when its allow decision does: there is no second fact to drift from it.
    pub fn transitions(&self) -> Result<Vec<TransitionFact>, CepAuthorityError> {
        let entries = self.ledger()?.read_entries()?;
        let mut out = Vec::new();
        for entry in &entries {
            let Some(key) = entry.idempotency_key.as_deref() else {
                continue;
            };
            if !(key.starts_with("cep:") || key.starts_with("cep-approval-use:")) {
                continue;
            }
            let Ok(decision) = serde_json::from_value::<AuthorityDecision>(entry.payload.clone())
            else {
                continue;
            };
            if decision.verdict != Verdict::Allow
                || decision.normalized_disposition != NormalizedDisposition::Allow
            {
                continue;
            }
            let Ok(Value::Object(action)) = serde_json::to_value(&decision.action_request.action)
            else {
                continue;
            };
            let action = Value::Object(action);
            if text_at(&action, "/resource_type") != "world_transition" {
                continue;
            }
            let Some(mut record) = action.pointer("/parameters/transition").cloned() else {
                continue;
            };
            // The ledger entry, not `decision_id`: that is only unique within one operation ("auth_01" recurs).
            record["authority_decision_ref"] = json!(entry.entry_ulid);
            out.push(TransitionFact {
                operation_id: text_at(&action, "/parameters/operation_id"),
                decision_id: decision.decision_id.clone(),
                requester: text_at(&entry.payload, "/action_request/actor/actor_id"),
                approval_id: decision
                    .policy_refs
                    .iter()
                    .find_map(|r| r.strip_prefix("approval:"))
                    .map(str::to_string),
                ledger_entry: entry.entry_ulid.clone(),
                record,
            });
        }
        Ok(out)
    }

    /// A lineage stays in one world. If a request cites an envelope this authority decided in another world, the
    /// only thing that licenses that is an allowed transition from that world to this one, cited as
    /// `transition_ref`. Work already decided stays pinned to its world and is never re-pinned; a request in the
    /// new world carries it forward explicitly. Cited ids this ledger never decided are not judged here.
    fn check_lineage_worlds(&self, request: &ParsedRequest) -> Result<(), CepAuthorityError> {
        if request.lineage_refs.is_empty() {
            return Ok(());
        }
        let entries = self.ledger()?.read_entries()?;
        let mut crossed: Vec<String> = Vec::new();
        for entry in &entries {
            let Ok(decision) = serde_json::from_value::<AuthorityDecision>(entry.payload.clone())
            else {
                continue;
            };
            let cited = decision.audit_record.evidence_refs.iter().any(|r| {
                r.strip_prefix("cep:")
                    .is_some_and(|id| request.lineage_refs.iter().any(|l| l == id))
            });
            if !cited {
                continue;
            }
            let Ok(action) = serde_json::to_value(&decision.action_request.action) else {
                continue;
            };
            let world = text_at(&action, "/parameters/world_ref");
            if !world.is_empty() && world != request.world_ref && !crossed.contains(&world) {
                crossed.push(world);
            }
        }
        if crossed.is_empty() {
            return Ok(());
        }
        let Some(transition_ref) = &request.transition_ref else {
            return Err(CepAuthorityError::Transition(format!(
                "lineage reaches into {}; moving work between worlds needs an allowed world transition cited as transition_ref",
                crossed.join(", ")
            )));
        };
        let transitions = self.transitions()?;
        let licensed = transitions
            .iter()
            .find(|t| &t.ledger_entry == transition_ref)
            .ok_or_else(|| {
                CepAuthorityError::Transition(format!(
                    "{transition_ref} is not an allowed world transition"
                ))
            })?;
        let target = text_at(&licensed.record, "/target_world_ref");
        let source = text_at(&licensed.record, "/source_world_ref");
        if target != request.world_ref || crossed.iter().any(|w| *w != source) {
            return Err(CepAuthorityError::Transition(format!(
                "{transition_ref} moves {source} to {target}, which does not cover a lineage from {} into {}",
                crossed.join(", "),
                request.world_ref
            )));
        }
        Ok(())
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

    /// An allow carries the settlement criteria in force now; settlement is later judged against exactly these.
    fn bind_criteria(&self, decision: &mut AuthorityDecision) {
        if decision.verdict == Verdict::Allow {
            decision
                .policy_refs
                .push(format!("settlement-criteria:{}", self.criteria.sha256()));
        }
    }

    fn evaluate(
        &self,
        request: &ParsedRequest,
        caller: &Actor,
    ) -> Result<AuthorityDecision, CepAuthorityError> {
        let mut parameters = json!({
            "operation_name": request.operation_name,
            "operation_id": request.operation_id,
            "world_ref": request.world_ref,
            "subject_actor_ref": request.subject_actor_ref,
            "request_envelope_id": request.envelope_id,
        });
        if let Some(t) = &request.transition {
            // The verified facts travel in the committed decision, so the ledger alone can answer what moved.
            parameters["transition"] = t.record(None);
        }
        let action = AuthorityAction::Reserved {
            resource_type: request.kind.resource_type().to_string(),
            resource_id: request.resource_id.clone(),
            parameters,
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
        self.check_lineage_worlds(&request)?;
        let mut decision = self.evaluate(&request, caller)?;
        let ledger = self.ledger()?;
        let mut note = ApprovalNote::default();

        // Floor: a move across a semantic change needs a human whatever policy says. A policy allow becomes an
        // approvable escalation; a deny stays a deny.
        let floor = request
            .transition
            .as_ref()
            .is_some_and(VerifiedTransition::needs_approval);
        let floor_allow = floor && decision.verdict == Verdict::Allow;
        if floor_allow && request.approval_ref.is_none() {
            decision.verdict = Verdict::Escalate;
            decision.outcome = Verdict::Escalate;
            decision.normalized_disposition = NormalizedDisposition::Escalate;
            decision.reason_codes = vec!["policy_escalate".into()];
            decision.reason = "world_transition_requires_approval".into();
            decision.required_next_steps = vec!["approve_world_transition".into()];
            decision.approval_request_id = Some(approval_id_for(&request.operation_id));
            decision.boundary_constraints.clear();
            decision.compensating_controls.clear();
            decision
                .policy_refs
                .push("floor:semantic_closure_differs".into());
        }

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
            if approvable(&decision) || floor_allow {
                decision.verdict = Verdict::Allow;
                decision.outcome = Verdict::Allow;
                decision.normalized_disposition = NormalizedDisposition::Allow;
                decision.reason_codes = vec!["approved_escalation".into()];
                decision.reason = "approved_escalation".into();
                decision.required_next_steps.clear();
                decision.approval_request_id = None;
                decision.policy_refs.push(format!("approval:{approval_id}"));
                self.bind_criteria(&mut decision);
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

        self.bind_criteria(&mut decision);
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

/// What an accepted evidence packet came to.
#[derive(Debug)]
pub struct EvidenceAck {
    pub operation_id: String,
    pub envelope_id: String,
    pub items: usize,
    pub committed: CommittedRecordRef,
}

/// A settlement, with the packet that expresses it.
#[derive(Debug)]
pub struct SettlementOutcome {
    pub status: &'static str,
    pub final_status: bool,
    pub envelope: Value,
    pub committed: CommittedRecordRef,
    /// True when this call returned an earlier final settlement rather than making a new one.
    pub replayed: bool,
}

const SETTLE_SUFFIX_FINAL: &str = "cep-settle-final:";

fn id_text(value: &Value, pointer: &str, what: &'static str) -> Result<String, CepAuthorityError> {
    let s = value
        .pointer(pointer)
        .and_then(Value::as_str)
        .ok_or(CepAuthorityError::Malformed(what))?;
    if s.is_empty() || s.len() > FIELD_MAX || s.chars().any(char::is_control) {
        return Err(CepAuthorityError::Malformed(what));
    }
    Ok(s.to_string())
}

/// An item of a packet's `evidence` array, reduced to what settlement reads.
#[derive(Debug, Clone)]
struct EvidenceItem {
    raw: Value,
    id: String,
    question: String,
    direction: String,
    reliability: String,
}

fn parse_items(packet: &Value) -> Result<Vec<EvidenceItem>, CepAuthorityError> {
    let items = packet
        .get("evidence")
        .and_then(Value::as_array)
        .filter(|a| !a.is_empty())
        .ok_or(CepAuthorityError::Malformed(
            "evidence must be a non-empty array",
        ))?;
    items
        .iter()
        .map(|raw| {
            let direction = raw
                .get("direction")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            if !matches!(direction, "supports" | "contradicts" | "unknown") {
                return Err(CepAuthorityError::Malformed(
                    "evidence direction must be supports, contradicts or unknown",
                ));
            }
            Ok(EvidenceItem {
                raw: raw.clone(),
                id: id_text(raw, "/evidence_id", "evidence_id")?,
                question: id_text(raw, "/question_ref", "evidence question_ref")?,
                direction: direction.to_string(),
                reliability: raw
                    .get("reliability")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            })
        })
        .collect()
}

/// A committed ALLOW decision for `operation_id`: the only thing evidence may hang from.
struct AllowedOperation {
    world_ref: String,
    criteria_sha256: Option<String>,
    decision_envelope_id: String,
}

fn allowed_operation(entries: &[LedgerEntry], operation_id: &str) -> Option<AllowedOperation> {
    entries.iter().find_map(|entry| {
        let key = entry.idempotency_key.as_deref()?;
        if entry.record_kind != "authority_decision"
            || !(key.starts_with("cep:") || key.starts_with("cep-approval-use:"))
            || !entry.subject_refs.iter().any(|s| s == operation_id)
        {
            return None;
        }
        let decision: AuthorityDecision = serde_json::from_value(entry.payload.clone()).ok()?;
        if decision.verdict != Verdict::Allow {
            return None;
        }
        let action = serde_json::to_value(&decision.action_request.action).ok()?;
        Some(AllowedOperation {
            world_ref: text_at(&action, "/parameters/world_ref"),
            criteria_sha256: decision
                .policy_refs
                .iter()
                .find_map(|r| r.strip_prefix("settlement-criteria:"))
                .map(str::to_string),
            decision_envelope_id: format!("env-authority_decision-{operation_id}"),
        })
    })
}

impl CepAuthorityService<'_> {
    /// Evaluate a governed surface (`evidence_mutation`, `settlement_declaration`) for `caller`. A refusal is
    /// committed (it is governance-relevant) and returned as an error.
    fn gate(
        &self,
        ledger: &LedgerStream,
        surface: &str,
        operation_id: &str,
        caller: &Actor,
    ) -> Result<(), CepAuthorityError> {
        let action = AuthorityAction::Reserved {
            resource_type: surface.to_string(),
            resource_id: operation_id.to_string(),
            parameters: json!({ "operation_id": operation_id, "surface": surface }),
        };
        let binding = self
            .bundle
            .resolve_identity(&caller.actor_id, caller.role.clone());
        let decision = self.engine.evaluate(AuthorityEvaluation {
            actor: caller,
            binding,
            run_id: operation_id,
            case_id: "cognate",
            plan_item_id: operation_id,
            sequence: 1,
            action: &action,
            workspace_root: self.root,
            evidence_refs: vec![format!("operation:{operation_id}")],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            environment: None,
        })?;
        // Every attempt is its own governance event: a re-evaluated decision is never byte-identical, so it
        // is committed plainly rather than under an idempotency key.
        ledger.commit_typed(
            "authority_decision",
            vec!["cognate".into(), operation_id.to_string()],
            &decision,
            vec![],
        )?;
        if decision.verdict != Verdict::Allow {
            return Err(CepAuthorityError::Evidence(format!(
                "{} may not use {surface} for {operation_id}: {}",
                caller.actor_id, decision.reason
            )));
        }
        Ok(())
    }

    /// Accept a RealityTrace-derived `evidence_packet` for an operation SEA-Forge allowed. Evidence is recorded;
    /// it settles nothing.
    pub fn submit_evidence(
        &self,
        packet: &Value,
        caller: &Actor,
    ) -> Result<EvidenceAck, CepAuthorityError> {
        let size = serde_json::to_vec(packet)
            .map(|v| v.len())
            .unwrap_or(usize::MAX);
        if size > MAX_PACKET_BYTES {
            return Err(CepAuthorityError::Oversize(size));
        }
        let kind = id_text(packet, "/envelope_kind", "envelope_kind")?;
        if kind != "evidence_packet" {
            return Err(CepAuthorityError::WrongKind(kind));
        }
        let profile = format!(
            "{}@{}",
            id_text(
                packet,
                "/extensions/cep.profile/profile_id",
                "extensions.cep.profile.profile_id"
            )?,
            id_text(
                packet,
                "/extensions/cep.profile/profile_version",
                "extensions.cep.profile.profile_version"
            )?
        );
        if profile != format!("godspeed.evidence_packet@{PROFILE_VERSION}") {
            return Err(CepAuthorityError::WrongProfile(profile));
        }
        let world_ref = id_text(packet, "/scope/world_ref", "scope.world_ref")?;
        let (world, _) = self
            .worlds
            .require(&world_ref)
            .map_err(|e| CepAuthorityError::World(e.to_string()))?;
        let envelope_id = id_text(packet, "/envelope_id", "envelope_id")?;
        let ns = "/extensions/godspeed.evidence_packet";
        let operation_id = id_text(packet, &format!("{ns}/operation_id"), "operation_id")?;
        let decision_ref = id_text(
            packet,
            &format!("{ns}/authority_decision_ref"),
            "authority_decision_ref",
        )?;
        let trace_ref = id_text(
            packet,
            &format!("{ns}/execution_trace_ref"),
            "execution_trace_ref",
        )?;
        let lineage: Vec<&str> = packet
            .get("lineage_refs")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        if !lineage.contains(&trace_ref.as_str()) {
            return Err(CepAuthorityError::Evidence(
                "lineage does not include the execution trace".into(),
            ));
        }
        if packet
            .get("questions")
            .and_then(Value::as_array)
            .is_none_or(|q| q.is_empty())
        {
            return Err(CepAuthorityError::Malformed(
                "questions must be a non-empty array",
            ));
        }
        let items = parse_items(packet)?;

        let ledger = self.ledger()?;
        let entries = ledger.read_entries()?;
        let allowed = allowed_operation(&entries, &operation_id).ok_or_else(|| {
            CepAuthorityError::Evidence(format!("{operation_id} has no committed allow decision"))
        })?;
        if allowed.decision_envelope_id != decision_ref {
            return Err(CepAuthorityError::Evidence(
                "authority_decision_ref does not name this operation's decision".into(),
            ));
        }
        if allowed.world_ref != world.to_string() {
            return Err(CepAuthorityError::Evidence(
                "evidence is for a different world than the decision".into(),
            ));
        }
        self.gate(&ledger, "evidence_mutation", &operation_id, caller)?;
        let committed = ledger.commit_typed_once(
            "evidence_packet",
            format!("cep-evidence:{envelope_id}"),
            vec!["cognate".into(), operation_id.clone()],
            packet,
            vec![],
        )?;
        Ok(EvidenceAck {
            operation_id,
            envelope_id,
            items: items.len(),
            committed,
        })
    }

    /// Settle an operation from the evidence committed for it, against the criteria bound into its allow
    /// decision. `settled` and `rejected` are final; `unsettled` can be re-evaluated when evidence changes.
    pub fn settle(
        &self,
        operation_id: &str,
        caller: &Actor,
    ) -> Result<SettlementOutcome, CepAuthorityError> {
        let ledger = self.ledger()?;
        let entries = ledger.read_entries()?;
        let allowed = allowed_operation(&entries, operation_id).ok_or_else(|| {
            CepAuthorityError::Evidence(format!("{operation_id} has no committed allow decision"))
        })?;
        let bound = allowed.criteria_sha256.clone().ok_or_else(|| {
            CepAuthorityError::Evidence("the allow decision carries no settlement criteria".into())
        })?;
        if bound != self.criteria.sha256() {
            return Err(CepAuthorityError::Evidence(
                "settlement criteria changed since this operation was allowed; refusing to judge it against different criteria".into(),
            ));
        }
        // A final settlement is not made twice.
        if let Some(done) = entries.iter().find(|e| {
            e.idempotency_key.as_deref()
                == Some(format!("{SETTLE_SUFFIX_FINAL}{operation_id}").as_str())
        }) {
            let status = settlement_status_of(&done.payload);
            return Ok(SettlementOutcome {
                status,
                final_status: true,
                envelope: done.payload.clone(),
                committed: committed_ref(done),
                replayed: true,
            });
        }
        let packets: Vec<&LedgerEntry> = entries
            .iter()
            .filter(|e| {
                e.record_kind == "evidence_packet"
                    && e.subject_refs.iter().any(|s| s == operation_id)
            })
            .collect();
        if packets.is_empty() {
            return Err(CepAuthorityError::Evidence(format!(
                "no evidence has been submitted for {operation_id}; an execution alone cannot be settled"
            )));
        }
        self.gate(&ledger, "settlement_declaration", operation_id, caller)?;

        let mut items: Vec<EvidenceItem> = Vec::new();
        let mut questions: Vec<Value> = Vec::new();
        let mut lineage: Vec<String> = Vec::new();
        for entry in &packets {
            for item in parse_items(&entry.payload)? {
                if !items.iter().any(|i| i.id == item.id) {
                    items.push(item);
                }
            }
            for q in entry
                .payload
                .get("questions")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let id = q
                    .get("question_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if !questions
                    .iter()
                    .any(|e| e.get("question_id").and_then(Value::as_str) == Some(id))
                {
                    questions.push(q.clone());
                }
            }
            lineage.push(text_at(&entry.payload, "/envelope_id"));
        }

        let min_rank = reliability_rank(&self.criteria.min_reliability).unwrap_or(2);
        let mut settlements = Vec::new();
        let mut any_rejected = false;
        let mut all_settled = true;
        let mut asked: Vec<String> = items.iter().map(|i| i.question.clone()).collect();
        asked.sort();
        asked.dedup();
        let mut set_hash = sha_short(
            &items
                .iter()
                .map(|i| i.id.as_str())
                .collect::<Vec<_>>()
                .join("|"),
        );
        for question in &asked {
            let of: Vec<&EvidenceItem> = items.iter().filter(|i| &i.question == question).collect();
            let contradicting: Vec<&&EvidenceItem> =
                of.iter().filter(|i| i.direction == "contradicts").collect();
            let supporting = of
                .iter()
                .filter(|i| {
                    i.direction == "supports"
                        && reliability_rank(&i.reliability).is_some_and(|r| r >= min_rank)
                })
                .count() as u32;
            let (status, missing): (&str, Vec<String>) = if !contradicting.is_empty() {
                any_rejected = true;
                all_settled = false;
                ("rejected", vec![])
            } else if supporting >= self.criteria.min_supporting {
                ("settled", vec![])
            } else {
                all_settled = false;
                (
                    "unsettled",
                    vec![format!(
                        "{} supporting evidence item(s) at reliability {} or better; {} required",
                        supporting, self.criteria.min_reliability, self.criteria.min_supporting
                    )],
                )
            };
            let mut settlement = json!({
                "settlement_id": format!("set-{operation_id}-{set_hash}-{}", sha_short(question)),
                "settlement_status": status,
                "transformation_ref": format!("tr-{operation_id}"),
                "question_ref": question,
                "evidence_refs": of.iter().map(|i| i.id.clone()).collect::<Vec<_>>(),
                "threshold_basis": format!(
                    "min_supporting={}; min_reliability={}; criteria={}",
                    self.criteria.min_supporting, self.criteria.min_reliability, bound
                ),
            });
            if !missing.is_empty() {
                settlement["missing_evidence"] = json!(missing);
            }
            if !contradicting.is_empty() {
                settlement["unresolved_contradictions"] = json!(contradicting
                    .iter()
                    .map(|i| i.id.clone())
                    .collect::<Vec<_>>());
            }
            settlements.push(settlement);
        }
        let status: &'static str = if any_rejected {
            "rejected"
        } else if all_settled {
            "settled"
        } else {
            "unsettled"
        };
        let final_status = status != "unsettled";
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let world_ref = text_at(&packets[0].payload, "/scope/world_ref");
        set_hash.truncate(12);
        let envelope = json!({
            "envelope_id": format!("env-settlement_packet-{operation_id}-{set_hash}"),
            "cep_version": "1.0.0",
            "envelope_version": "1.0",
            "envelope_kind": "settlement_packet",
            "created_at": now,
            "created_by": "sea-forge-authority",
            "scope": {"evaluation_context": "godspeed-settlement_packet", "world_ref": world_ref},
            "boundary_record": {
                "scope": "settlement of one Cognate operation from submitted evidence",
                "included_sections": ["evidence", "questions", "settlements"],
                "excluded_sections": [],
                "known_omissions": ["evidence not submitted to SEA-Forge is not considered"],
                "unknowns": [],
                "redactions": [],
                "compression_notes": [],
                "out_of_scope_entities": [],
                "classification": "internal",
                "limitations": [],
            },
            "completeness_status": "partial",
            "omission_status": "marked",
            "provenance_refs": [format!("prov-settle-{operation_id}-{set_hash}")],
            "validation_status": "valid",
            "lineage_refs": lineage,
            "provenance": [{
                "provenance_id": format!("prov-settle-{operation_id}-{set_hash}"),
                "source_system_refs": ["sea-forge"],
                "producer_refs": ["sea-forge-authority"],
                "production_method": "settlement_evaluation",
                "created_at": now,
            }],
            "questions": questions,
            "evidence": items.iter().map(|i| i.raw.clone()).collect::<Vec<_>>(),
            "settlements": settlements,
            "extensions": {
                "cep.profile": {"profile_id": "godspeed.settlement_packet", "profile_version": PROFILE_VERSION},
                "godspeed.settlement_packet": {
                    "operation_id": operation_id,
                    "authority_decision_ref": allowed.decision_envelope_id,
                    "criteria_sha256": bound,
                    "overall_status": status,
                    "final": final_status,
                    "declared_by": caller.actor_id,
                },
            },
        });
        let subjects = vec!["cognate".to_string(), operation_id.to_string()];
        let committed = if final_status {
            match ledger.commit_typed_new(
                "settlement_packet",
                format!("{SETTLE_SUFFIX_FINAL}{operation_id}"),
                subjects,
                &envelope,
                vec![],
            ) {
                Ok(c) => c,
                Err(_) => {
                    // Another settlement became final first: return that one.
                    let entries = ledger.read_entries()?;
                    let done = entries
                        .iter()
                        .find(|e| {
                            e.idempotency_key.as_deref()
                                == Some(format!("{SETTLE_SUFFIX_FINAL}{operation_id}").as_str())
                        })
                        .ok_or_else(|| {
                            CepAuthorityError::Internal("final settlement vanished".into())
                        })?;
                    return Ok(SettlementOutcome {
                        status: settlement_status_of(&done.payload),
                        final_status: true,
                        envelope: done.payload.clone(),
                        committed: committed_ref(done),
                        replayed: true,
                    });
                }
            }
        } else {
            // The same evidence set gives the same unsettled answer: return the recorded one.
            let key = format!("cep-settle:{operation_id}:{set_hash}");
            if let Some(done) = entries
                .iter()
                .find(|e| e.idempotency_key.as_deref() == Some(key.as_str()))
            {
                return Ok(SettlementOutcome {
                    status,
                    final_status: false,
                    envelope: done.payload.clone(),
                    committed: committed_ref(done),
                    replayed: true,
                });
            }
            ledger.commit_typed_once("settlement_packet", key, subjects, &envelope, vec![])?
        };
        Ok(SettlementOutcome {
            status,
            final_status,
            envelope,
            committed,
            replayed: false,
        })
    }
}

fn sha_short(text: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(text.as_bytes()))[..16].to_string()
}

fn settlement_status_of(envelope: &Value) -> &'static str {
    match envelope
        .pointer("/extensions/godspeed.settlement_packet/overall_status")
        .and_then(Value::as_str)
    {
        Some("settled") => "settled",
        Some("rejected") => "rejected",
        _ => "unsettled",
    }
}

fn committed_ref(entry: &LedgerEntry) -> CommittedRecordRef {
    entry.committed_ref()
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
