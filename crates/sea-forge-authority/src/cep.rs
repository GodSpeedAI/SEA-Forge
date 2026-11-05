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

mod declaration_authority;
pub use declaration_authority::{
    declaration_content_hash, is_external_declaration, read_all_declarations,
    read_case_declarations, reconcile_operation_claims, settle_failed_resume_attempt,
    verify_case_declarations, AttemptSettlement, DeclarationAuthorization, DeclarationRefusal,
};
mod obligation;
pub use obligation::{
    argv_digest, judge, obligation_of, ClosureAck, Obligation, ObligationSource, RunResult,
    SourceRow, CLOSURE_KIND, DEFAULT_CLOSE_GRACE_S,
};

use crate::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use chrono::{DateTime, Duration, Utc};
use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::{
    Actor, AuthorityAction, AuthorityDecision, NormalizedDisposition, SettlementDeclaration,
    SettlementDeclarationRequest, Verdict,
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
        format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
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
    /// Settlement is not yet decidable: the obligation's window is still open, or its closure is not
    /// recorded yet. Nothing is written. A client may retry later; this is not a refusal of the evidence.
    SettlementPending(String),
    /// A world transition whose facts do not hold: unregistered or identical worlds, or claims that
    /// contradict what SEA-Forge recomputes from the registered identities.
    Transition(String),
    /// The request's `claims` are not a bounded, flat map of plain keys to strings or numbers.
    Claims(String),
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
            Self::SettlementPending(m) => write!(f, "settlement pending: {m}"),
            Self::Transition(m) => write!(f, "world transition refused: {m}"),
            Self::Claims(m) => write!(f, "malformed authority request: claims {m}"),
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
            Self::Malformed(_) | Self::WrongKind(_) | Self::WrongProfile(_) | Self::Claims(_) => {
                "cep_request_invalid"
            }
            Self::World(_) => "cep_world_refused",
            Self::Approval(_) => "cep_approval_refused",
            Self::Evidence(_) => "cep_evidence_refused",
            Self::SettlementPending(_) => "cep_settlement_pending",
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
    /// The requester's assertions about this operation (`claims` in the request profile), recorded verbatim
    /// with the decision. Never an input to the verdict.
    pub claims: Option<serde_json::Map<String, Value>>,
}

/// Most claims one request may carry, and the longest claim string (mirrors Cognate's bounds).
pub const MAX_CLAIMS: usize = 16;
pub const MAX_CLAIM_LENGTH: usize = 512;

fn parse_claims(
    envelope: &Value,
    ns: &str,
) -> Result<Option<serde_json::Map<String, Value>>, CepAuthorityError> {
    let Some(raw) = envelope.pointer(&format!("{ns}/claims")) else {
        return Ok(None);
    };
    if raw.is_null() {
        return Ok(None);
    }
    let invalid = CepAuthorityError::Claims;
    let map = raw
        .as_object()
        .ok_or_else(|| invalid("must be an object".into()))?;
    if map.is_empty() || map.len() > MAX_CLAIMS {
        return Err(invalid(format!("must hold 1 to {MAX_CLAIMS} entries")));
    }
    for (key, value) in map {
        let plain = key.len() <= 64
            && key.chars().next().is_some_and(|c| c.is_ascii_lowercase())
            && key
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !plain {
            return Err(invalid(format!(
                "key {key:?} must match [a-z][a-z0-9_]{{0,63}}"
            )));
        }
        match value {
            Value::String(text) if !text.is_empty() && text.chars().count() <= MAX_CLAIM_LENGTH => {}
            Value::Number(_) => {}
            _ => return Err(invalid(format!("{key} must be a non-empty string of at most {MAX_CLAIM_LENGTH} characters or a number"))),
        }
    }
    Ok(Some(map.clone()))
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
    if worlds.is_prior(&target_world.to_string()) {
        return Err(CepAuthorityError::Transition(format!(
            "target world {target_world} is a prior identity; work moves only into a current world"
        )));
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
    // A source_edit_only never changes the producer: a producer change is an upgrade, and must say so.
    if kind == "source_edit_only"
        && (source_identity.producer != target_identity.producer
            || source_identity.producer_version != target_identity.producer_version)
    {
        return Err(CepAuthorityError::Transition(format!(
            "transition_kind source_edit_only is misdeclared: the producer changed ({} {} to {} {}); a producer change is a compiler_upgrade",
            source_identity.producer,
            source_identity.producer_version,
            target_identity.producer,
            target_identity.producer_version
        )));
    }
    // A compiler_upgrade keeps every meaning-bearing identity field equal and changes only the producer version.
    // The first differing field is named, so the refusal says exactly what moved.
    if kind == "compiler_upgrade" {
        let equal = [
            (
                "semantic_closure_hash",
                source_identity.semantic_closure_hash == target_identity.semantic_closure_hash,
            ),
            (
                "source_set_hash",
                source_identity.source_set_hash == target_identity.source_set_hash,
            ),
            (
                "semantic_pack_set_hash",
                source_identity.semantic_pack_set_hash == target_identity.semantic_pack_set_hash,
            ),
            (
                "registry_content_hash",
                source_identity.registry_content_hash == target_identity.registry_content_hash,
            ),
            (
                "identity_scheme_version",
                source_identity.identity_scheme_version == target_identity.identity_scheme_version,
            ),
            (
                "language_schema_version",
                source_identity.language_schema_version == target_identity.language_schema_version,
            ),
            (
                "compiler_interpretation_version",
                source_identity.compiler_interpretation_version
                    == target_identity.compiler_interpretation_version,
            ),
            (
                "canonicalization_version",
                source_identity.canonicalization_version
                    == target_identity.canonicalization_version,
            ),
            (
                "producer",
                source_identity.producer == target_identity.producer,
            ),
        ];
        if let Some((field, _)) = equal.iter().find(|(_, same)| !same) {
            return Err(CepAuthorityError::Transition(format!(
                "transition_kind compiler_upgrade is misdeclared: {field} differs between the registered worlds; a compiler_upgrade may change only producer_version"
            )));
        }
        if source_identity.producer_version == target_identity.producer_version {
            return Err(CepAuthorityError::Transition(format!(
                "transition_kind compiler_upgrade is misdeclared: producer_version is {} on both sides, so this is not an upgrade",
                source_identity.producer_version
            )));
        }
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
    let mut evidence_refs = match envelope.pointer(&format!("{TRANSITION_NS}/evidence_refs")) {
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
    if kind == "compiler_upgrade" {
        // SEA-Forge's own recomputed facts, recorded where the CEP record allows free strings.
        evidence_refs.push(format!(
            "domainforge.producer_version.source:{}",
            source_identity.producer_version
        ));
        evidence_refs.push(format!(
            "domainforge.producer_version.target:{}",
            target_identity.producer_version
        ));
    }
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
    // A prior identity is a place to move work out of, never to act in: only a world_transition may name it.
    if operation_kind != OperationKind::WorldTransition && worlds.is_prior(&world.to_string()) {
        return Err(CepAuthorityError::World(format!(
            "{world} is a prior identity; governed actions run only in a current world, so move the work out with an allowed world_transition first"
        )));
    }
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
        claims: parse_claims(envelope, ns)?,
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
        if approval_id.starts_with("apr-")
            && approval_id[4..].starts_with(SETTLEMENT_PENDING_PREFIX)
        {
            return settlement_standing_in(entries, approval_id, now);
        }
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
        let ids: Vec<String> = entries
            .iter()
            .filter_map(|e| {
                e.idempotency_key
                    .as_deref()?
                    .strip_prefix("cep:")
                    .map(approval_id_for)
            })
            .chain(entries.iter().filter_map(|e| {
                (e.record_kind == SETTLEMENT_PENDING_KIND)
                    .then(|| {
                        e.payload
                            .get("pending_id")?
                            .as_str()
                            .map(settlement_approval_id)
                    })
                    .flatten()
            }))
            .collect();
        Ok(ids
            .iter()
            .filter_map(|id| self.standing_in(&entries, id, now))
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
        if let Some(claims) = &request.claims {
            // Recorded with the decision, so the ledger alone shows what the requester asserted when it asked.
            parameters["claims"] = Value::Object(claims.clone());
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
            opa_candidate: None,
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

    /// Resolve an escalation as the verified `resolver`. The requester can never resolve their own request:
    /// that is checked here, before and independent of policy. Otherwise the policy engine decides whether
    /// this actor may resolve it (a rule for `approval_resolution`).
    pub fn resolve_approval(
        &self,
        approval_id: &str,
        approve: bool,
        resolver: &Actor,
    ) -> Result<Value, CepAuthorityError> {
        let ledger = self.ledger()?;
        let entries = ledger.read_entries()?;
        let Some(standing) = self.standing_in(&entries, approval_id, Utc::now()) else {
            if approval_id.starts_with("apr-")
                && approval_id[4..].starts_with(SETTLEMENT_PENDING_PREFIX)
            {
                // A forged or unknown settlement approval is refused and audited.
                ledger.commit_typed(
                    SETTLEMENT_APPROVAL_REFUSED_KIND,
                    vec!["cognate".into(), approval_id.to_string()],
                    &json!({"approval_id": approval_id, "resolver_id": resolver.actor_id, "reason": "unknown_settlement_approval"}),
                    vec![],
                )?;
            }
            return Err(CepAuthorityError::Approval(format!(
                "{approval_id} is not an approvable escalation"
            )));
        };
        if resolver.actor_id == standing.requester {
            // Audit the attempt before refusing. Policy is not evaluated for self-resolution.
            #[derive(serde::Serialize)]
            struct SelfResolutionRefusal<'a> {
                approval_id: &'a str,
                requester_id: &'a str,
                resolver_id: &'a str,
                reason: &'static str,
            }
            ledger.commit_typed(
                "approval_self_resolution_refused",
                vec!["cognate".into(), approval_id.to_string()],
                &SelfResolutionRefusal {
                    approval_id,
                    requester_id: &standing.requester,
                    resolver_id: &resolver.actor_id,
                    reason: "self_approval_refused",
                },
                vec![standing.escalation_entry.clone()],
            )?;
            return Err(CepAuthorityError::Approval(format!(
                "self_approval_refused: {} requested {approval_id} and may not resolve it",
                standing.requester
            )));
        }
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
            opa_candidate: None,
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
    /// The evidence source every item was checked against, or `None` when any item was taken as stated.
    pub corroborated_by: Option<&'static str>,
}

/// Where an evidence item's original record can be read back, so a packet is checked against what the
/// evidence source itself recorded rather than believed as submitted.
pub trait EvidenceSource {
    /// The source's name, recorded with the evidence (`realitytrace`).
    fn name(&self) -> &'static str;
    /// The item exactly as the source recorded it, after checking that the source's own record is intact.
    /// `Err` says why the item cannot be corroborated.
    fn recorded_item(&self, item: &Value) -> Result<Value, String>;
}

/// How `authority_evidence` treats a packet's items.
pub enum Corroboration<'a> {
    /// Items are taken as stated (no evidence source configured).
    None,
    /// Every item whose `source` is the configured source's name must match its recorded original; with
    /// `required`, an item from any other source is refused.
    Source {
        source: &'a dyn EvidenceSource,
        required: bool,
    },
}

/// Fields a packet item must carry exactly as the evidence source recorded them. Direction and reliability
/// decide settlement, so they must be present in the original.
const CORROBORATED_FIELDS: [&str; 7] = [
    "evidence_id",
    "evidence_type",
    "question_ref",
    "representation_ref",
    "relevance",
    "reliability",
    "direction",
];

fn corroborate(
    items: &[EvidenceItem],
    how: &Corroboration<'_>,
) -> Result<Option<&'static str>, CepAuthorityError> {
    let Corroboration::Source { source, required } = how else {
        return Ok(None);
    };
    // The packet is marked corroborated only when every item was checked; a packet with any item taken
    // as stated must never count later as corroborated evidence.
    let mut every_item_checked = true;
    for item in items {
        let from = item.raw.get("source").and_then(Value::as_str).unwrap_or("");
        if from != source.name() {
            if *required {
                return Err(CepAuthorityError::Evidence(format!(
                    "evidence {} is not from {}; this cell settles only on corroborated evidence",
                    item.id,
                    source.name()
                )));
            }
            every_item_checked = false;
            continue;
        }
        let recorded = source.recorded_item(&item.raw).map_err(|why| {
            CepAuthorityError::Evidence(format!(
                "evidence {} cannot be corroborated against {}: {why}",
                item.id,
                source.name()
            ))
        })?;
        for field in ["direction", "reliability"] {
            if recorded.get(field).and_then(Value::as_str).is_none() {
                return Err(CepAuthorityError::Evidence(format!(
                    "{}'s record of evidence {} has no {field}",
                    source.name(),
                    item.id
                )));
            }
        }
        for field in CORROBORATED_FIELDS {
            if let Some(original) = recorded.get(field) {
                if item.raw.get(field) != Some(original) {
                    return Err(CepAuthorityError::Evidence(format!(
                        "evidence {}: {field} differs from what {} recorded",
                        item.id,
                        source.name()
                    )));
                }
            }
        }
    }
    Ok(every_item_checked.then(|| source.name()))
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
    claims: Option<serde_json::Map<String, Value>>,
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
            claims: action
                .pointer("/parameters/claims")
                .and_then(Value::as_object)
                .cloned(),
        })
    })
}

impl CepAuthorityService<'_> {
    /// Evaluate the `evidence_mutation` surface for `caller`. A refusal is committed (it is governance-relevant)
    /// and returned as an error.
    fn gate(
        &self,
        ledger: &LedgerStream,
        surface: &str,
        operation_id: &str,
        caller: &Actor,
    ) -> Result<(), CepAuthorityError> {
        let parameters = json!({ "operation_id": operation_id, "surface": surface });
        commit_gate_decision(
            self.engine,
            self.bundle,
            self.root,
            ledger,
            surface,
            parameters,
            operation_id,
            caller,
        )
    }

    /// The gate for a settlement declaration. The class is derived from the verified ingress, never passed in.
    fn gate_settlement_declaration(
        &self,
        ledger: &LedgerStream,
        operation_id: &str,
        ingress: &VerifiedSettlementIngress<'_>,
    ) -> Result<(), CepAuthorityError> {
        let class = derive_settlement_authority_class(ingress);
        commit_gate_decision(
            self.engine,
            self.bundle,
            self.root,
            ledger,
            "settlement_declaration",
            settlement_parameters(operation_id, class),
            operation_id,
            ingress.caller(),
        )
    }

    /// Accept a RealityTrace-derived `evidence_packet` for an operation SEA-Forge allowed. Evidence is recorded;
    /// it settles nothing.
    pub fn submit_evidence(
        &self,
        packet: &Value,
        caller: &Actor,
    ) -> Result<EvidenceAck, CepAuthorityError> {
        self.submit_evidence_with(packet, caller, &Corroboration::None)
    }

    /// `submit_evidence`, checking each item against its evidence source first. A mismatch or an item that
    /// cannot be found commits nothing.
    pub fn submit_evidence_with(
        &self,
        packet: &Value,
        caller: &Actor,
        corroboration: &Corroboration<'_>,
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
        let corroborated_by = corroborate(&items, corroboration)?;
        let mut subjects = vec!["cognate".to_string(), operation_id.clone()];
        if let Some(by) = corroborated_by {
            subjects.push(format!("corroborated:{by}"));
        }
        let committed = ledger.commit_typed_once(
            "evidence_packet",
            format!("cep-evidence:{envelope_id}"),
            subjects,
            packet,
            vec![],
        )?;
        Ok(EvidenceAck {
            operation_id,
            envelope_id,
            items: items.len(),
            committed,
            corroborated_by,
        })
    }

    /// Settle an operation from the evidence committed for it, against the criteria bound into its allow
    /// decision. `settled` and `rejected` are final; `unsettled` can be re-evaluated when evidence changes.
    pub fn settle(
        &self,
        operation_id: &str,
        caller: &Actor,
    ) -> Result<SettlementOutcome, CepAuthorityError> {
        self.settle_with(operation_id, caller, None)
    }

    /// `settle`, counting only evidence packets corroborated against `required_source` when it is set, so a
    /// packet committed before corroboration was required never decides a settlement.
    pub fn settle_with(
        &self,
        operation_id: &str,
        caller: &Actor,
        required_source: Option<&str>,
    ) -> Result<SettlementOutcome, CepAuthorityError> {
        let corroborated = required_source.map(|s| format!("corroborated:{s}"));
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
                    && corroborated
                        .as_ref()
                        .is_none_or(|c| e.subject_refs.iter().any(|s| s == c))
            })
            .collect();
        if packets.is_empty() {
            let what = if corroborated.is_some() {
                "corroborated evidence"
            } else {
                "evidence"
            };
            return Err(CepAuthorityError::Evidence(format!(
                "no {what} has been submitted for {operation_id}; an execution alone cannot be settled"
            )));
        }
        // Local: CEP settlement is SEA-Forge's own judgement over the evidence (no SettlementAuthority or
        // SWE_SEED transport is invoked on this path; the record is attributed to `caller`).
        let ingress = VerifiedSettlementIngress::verified(
            SettlementIngress::CepSettlement,
            SettlementTransport::None,
            None,
            caller,
            vec![format!("operation:{operation_id}")],
        )?;
        self.gate_settlement_declaration(&ledger, operation_id, &ingress)?;

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
    hex::encode(Sha256::digest(text.as_bytes()))[..16].to_string()
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

/// Who is attesting a settlement declaration. Each caller passes this from its own code path; it is never
/// derived from the declaration record. Parsed strictly: only the two exact spellings are accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementAuthorityClass {
    /// Declared by an authority outside this process (the SWE_SEED transport). Escalates under a gated rule.
    External,
    /// Settled by SEA-Forge's own in-process judgement, with no external declarer.
    Local,
}

impl SettlementAuthorityClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::External => "external",
            Self::Local => "local",
        }
    }

    pub fn parse(value: &str) -> Result<Self, CepAuthorityError> {
        match value {
            "external" => Ok(Self::External),
            "local" => Ok(Self::Local),
            _ => Err(CepAuthorityError::Malformed(
                "settlement authority class must be external or local",
            )),
        }
    }
}

/// The trusted ingress a settlement declaration arrives through. The code that owns each path names its own
/// ingress; no caller passes a class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementIngress {
    /// The production SWE_SEED declaration sent to the strong settlement authority adapter.
    SweSeedDeclaration,
    /// Resume of a pending artifact transition, which declares through the SWE_SEED adapter.
    ArtifactTransitionResume,
    /// CEP `settle` / `settle_with` over evidence SEA-Forge committed itself.
    CepSettlement,
    /// Obligation closure over RealityTrace's ledger.
    ObligationClosure,
}

/// The settlement authority transport a verified ingress reaches, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementTransport {
    None,
    SweSeedAdapter,
}

/// What a trusted ingress verified about one settlement declaration: the ingress, the transport it reaches, any
/// adapter attestation it carries, the verified identity it acts as, and the evidence refs it rests on. Built only
/// through [`VerifiedSettlementIngress::verified`], which refuses any ingress/transport/attestation combination
/// that does not match the operation.
#[derive(Debug, Clone)]
pub struct VerifiedSettlementIngress<'a> {
    ingress: SettlementIngress,
    transport: SettlementTransport,
    adapter_attestation_ref: Option<&'a str>,
    caller: &'a Actor,
    evidence_refs: Vec<String>,
}

impl<'a> VerifiedSettlementIngress<'a> {
    pub fn verified(
        ingress: SettlementIngress,
        transport: SettlementTransport,
        adapter_attestation_ref: Option<&'a str>,
        caller: &'a Actor,
        evidence_refs: Vec<String>,
    ) -> Result<Self, CepAuthorityError> {
        let transport_matches = match ingress {
            SettlementIngress::SweSeedDeclaration | SettlementIngress::ArtifactTransitionResume => {
                transport == SettlementTransport::SweSeedAdapter
            }
            SettlementIngress::CepSettlement | SettlementIngress::ObligationClosure => {
                transport == SettlementTransport::None
            }
        };
        if !transport_matches {
            return Err(CepAuthorityError::Malformed(
                "settlement transport does not match its ingress",
            ));
        }
        if adapter_attestation_ref.is_some_and(str::is_empty) {
            return Err(CepAuthorityError::Malformed(
                "settlement adapter attestation is empty",
            ));
        }
        if adapter_attestation_ref.is_some() && transport != SettlementTransport::SweSeedAdapter {
            return Err(CepAuthorityError::Malformed(
                "settlement adapter attestation requires an adapter transport",
            ));
        }
        Ok(Self {
            ingress,
            transport,
            adapter_attestation_ref,
            caller,
            evidence_refs,
        })
    }

    pub fn ingress(&self) -> SettlementIngress {
        self.ingress
    }

    pub fn caller(&self) -> &Actor {
        self.caller
    }

    pub fn evidence_refs(&self) -> &[String] {
        &self.evidence_refs
    }
}

/// The one place a settlement authority class is decided. External when the operation reaches an adapter
/// transport, carries an adapter attestation, or enters through an external ingress. Local only for CEP
/// settlement and obligation closure with no transport. A caller cannot choose the result: it follows from the
/// verified ingress.
pub fn derive_settlement_authority_class(
    ingress: &VerifiedSettlementIngress<'_>,
) -> SettlementAuthorityClass {
    let external = ingress.transport == SettlementTransport::SweSeedAdapter
        || ingress.adapter_attestation_ref.is_some()
        || matches!(
            ingress.ingress,
            SettlementIngress::SweSeedDeclaration | SettlementIngress::ArtifactTransitionResume
        );
    if external {
        SettlementAuthorityClass::External
    } else {
        SettlementAuthorityClass::Local
    }
}

fn settlement_parameters(operation_id: &str, class: SettlementAuthorityClass) -> Value {
    json!({
        "operation_id": operation_id,
        "surface": "settlement_declaration",
        "settlement_authority_class": class.as_str(),
    })
}

/// Evaluate a governed surface against the policy and commit the decision. A refusal is an error and the
/// caller must not proceed. Shared by the CEP settlement callers and every SWE_SEED declaration path, so
/// no path reaches `declare()` without this decision.
#[allow(clippy::too_many_arguments)]
fn commit_gate_evaluation(
    engine: &PolicyAuthorityEngine,
    bundle: &AuthorityPolicyBundle,
    root: &Path,
    ledger: &LedgerStream,
    surface: &str,
    parameters: Value,
    operation_id: &str,
    caller: &Actor,
) -> Result<(AuthorityDecision, CommittedRecordRef), CepAuthorityError> {
    let action = AuthorityAction::Reserved {
        resource_type: surface.to_string(),
        resource_id: operation_id.to_string(),
        parameters,
    };
    let binding = bundle.resolve_identity(&caller.actor_id, caller.role.clone());
    let decision = engine.evaluate(AuthorityEvaluation {
        actor: caller,
        binding,
        run_id: operation_id,
        case_id: "cognate",
        plan_item_id: operation_id,
        sequence: 1,
        action: &action,
        workspace_root: root,
        evidence_refs: vec![format!("operation:{operation_id}")],
        artifacts_root: None,
        timeout_secs: None,
        env_keys: Default::default(),
        opa_candidate: None,
        environment: None,
    })?;
    // Every attempt is its own governance event: a re-evaluated decision is never byte-identical, so it
    // is committed plainly rather than under an idempotency key.
    let committed = ledger.commit_typed(
        "authority_decision",
        vec!["cognate".into(), operation_id.to_string()],
        &decision,
        vec![],
    )?;
    Ok((decision, committed))
}

/// Evaluate and commit a gate that must allow outright. Any other verdict is a refusal the caller must honor.
#[allow(clippy::too_many_arguments)]
fn commit_gate_decision(
    engine: &PolicyAuthorityEngine,
    bundle: &AuthorityPolicyBundle,
    root: &Path,
    ledger: &LedgerStream,
    surface: &str,
    parameters: Value,
    operation_id: &str,
    caller: &Actor,
) -> Result<(), CepAuthorityError> {
    let (decision, _) = commit_gate_evaluation(
        engine,
        bundle,
        root,
        ledger,
        surface,
        parameters,
        operation_id,
        caller,
    )?;
    if decision.verdict != Verdict::Allow {
        return Err(CepAuthorityError::Evidence(format!(
            "{} may not use {surface} for {operation_id}: {}",
            caller.actor_id, decision.reason
        )));
    }
    Ok(())
}

/// The outcome of gating one settlement declaration from a verified ingress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettlementGate {
    /// Policy allows this declaration and no settlement state holds it. The caller may call `declare()`.
    Allowed(SettlementAuthorityClass),
    /// One approved (or policy-allowed, previously pending) settlement was claimed for this attempt. The caller
    /// may call `declare()` once and must then record the outcome: `complete_settlement_resume` on success,
    /// `release_settlement_resume` when the authority was unavailable, `fail_settlement_resume` otherwise.
    /// The declaration must carry `pending_id` and `request_sha256` as its authorisation.
    Resumed {
        class: SettlementAuthorityClass,
        pending_id: String,
        request_sha256: String,
        attempt: u32,
    },
    /// The declaration for this request already completed. Nothing may be declared again.
    Completed {
        pending_id: String,
        declaration_ref: String,
    },
    /// Policy requires approval first. The pending record is durable and was written before this was returned.
    /// The caller must not call `declare()`. `replayed` is true when an identical request already held this record.
    Pending {
        record: Box<SettlementPendingRecord>,
        replayed: bool,
    },
}

/// A settlement declaration held for approval. Written to the authority ledger before a pending outcome is
/// returned. It is never an authoritative declaration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SettlementPendingRecord {
    pub pending_id: String,
    /// SHA-256 over the canonical request, which includes `case_id`. The same request identity in the same case
    /// always maps to the same record; the same operation in another case is a different record.
    pub request_sha256: String,
    /// The ingress identity hash that `request_sha256` binds to the declared content:
    /// `request_sha256 == bind(ingress_sha256, declared content)`. An authorised declaration must carry the declared
    /// content hash that reproduces `request_sha256` from this value. Empty only on a legacy record, which is refused.
    #[serde(default)]
    pub ingress_sha256: String,
    /// The case the request belongs to. Empty only on a legacy record written before the case was bound into
    /// the request hash. Such a record is never matched by any lookup and is never reused.
    #[serde(default)]
    pub case_id: String,
    pub operation_id: String,
    pub operation_kind: String,
    pub settlement_authority_class: String,
    pub ingress: String,
    pub actor_id: String,
    pub actor_role: String,
    pub identity_binding_hash: Option<String>,
    pub evidence_refs: Vec<String>,
    pub approval_requirement: String,
    /// The committed `authority_decision` that escalated this request.
    pub decision_ref: String,
    pub created_at: String,
    pub expires_at: String,
}

/// Where one settlement pending record stands. Derived from the append-only ledger and nothing else.
///
/// Transitions: `Pending -> Approved -> Resuming -> Completed`, or `Pending|Approved -> Rejected|Expired`,
/// or `Resuming -> Failed`. A released attempt returns to `Approved`. Terminal states are never left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettlementStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
    Resuming { attempt: u32 },
    Completed { declaration_ref: String },
    Failed { reason: String },
}

const SETTLEMENT_PENDING_KIND: &str = "settlement_pending";
const SETTLEMENT_PENDING_KEY: &str = "settlement-pending:";
const SETTLEMENT_PENDING_PREFIX: &str = "settlement-pending-";
const SETTLEMENT_STATE_KIND: &str = "settlement_state";
const SETTLEMENT_APPROVAL_REFUSED_KIND: &str = "settlement_approval_refused";
const SETTLEMENT_APPROVAL_REQUIREMENT: &str = "policy_escalate";
const SETTLEMENT_SURFACE: &str = "settlement_declaration";

/// The approval id a settlement pending record is approved under. It binds to exactly one pending record.
pub fn settlement_approval_id(pending_id: &str) -> String {
    format!("apr-{pending_id}")
}

/// How long an open claim is presumed live, from its `claimed_at`. Longer than the authority timeout any descriptor
/// may carry (enforced on load), so a lapsed lease means the claiming attempt is no longer calling the authority.
pub const SETTLEMENT_CLAIM_LEASE_SECS: i64 = 900;

/// Time reserved inside the lease for the work after the authority returns: reading its output, checking the echo,
/// and appending the declaration. A descriptor timeout must leave this headroom below the lease.
pub const SETTLEMENT_CLAIM_HEADROOM_SECS: i64 = 300;

fn state_key(pending_id: &str, suffix: &str) -> String {
    format!("settlement-state:{pending_id}:{suffix}")
}

impl SettlementIngress {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SweSeedDeclaration => "swe_seed_declaration",
            Self::ArtifactTransitionResume => "artifact_transition_resume",
            Self::CepSettlement => "cep_settlement",
            Self::ObligationClosure => "obligation_closure",
        }
    }
}

/// The canonical identity of one settlement declaration request. Field order is fixed by this struct and
/// evidence refs are sorted, so the hash is stable. Timestamps are excluded on purpose.
#[derive(serde::Serialize)]
struct CanonicalSettlementRequest<'a> {
    case_id: &'a str,
    ingress: &'a str,
    operation_id: &'a str,
    operation_kind: &'a str,
    settlement_authority_class: &'a str,
    actor_id: &'a str,
    actor_role: &'a str,
    identity_binding_hash: Option<&'a str>,
    evidence_refs: Vec<&'a str>,
    approval_requirement: &'a str,
}

/// The identity of one request, computed before anything is written. `request_sha256` is the ingress identity only;
/// the pending id derives from it, so a claim finds its record whatever content it presents. The approved request
/// hash binds this to the declared content.
struct SettlementIdentity {
    case_id: String,
    operation_id: String,
    request_sha256: String,
    pending_id: String,
    actor_role: String,
    identity_binding_hash: Option<String>,
}

fn settlement_identity(
    bundle: &AuthorityPolicyBundle,
    case_id: &str,
    operation_id: &str,
    class: SettlementAuthorityClass,
    ingress: &VerifiedSettlementIngress<'_>,
) -> Result<SettlementIdentity, CepAuthorityError> {
    use sha2::{Digest, Sha256};
    let caller = ingress.caller();
    let identity_binding_hash = bundle
        .resolve_identity(&caller.actor_id, caller.role.clone())
        .identity_binding_hash;
    let role = serde_json::to_value(&caller.role)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .ok_or(CepAuthorityError::Internal(
            "settlement caller role is not serializable".into(),
        ))?;
    let mut evidence_refs: Vec<&str> = ingress.evidence_refs().iter().map(String::as_str).collect();
    evidence_refs.sort_unstable();
    evidence_refs.dedup();
    let canonical = CanonicalSettlementRequest {
        case_id,
        ingress: ingress.ingress().as_str(),
        operation_id,
        operation_kind: SETTLEMENT_SURFACE,
        settlement_authority_class: class.as_str(),
        actor_id: &caller.actor_id,
        actor_role: &role,
        identity_binding_hash: identity_binding_hash.as_deref(),
        evidence_refs,
        approval_requirement: SETTLEMENT_APPROVAL_REQUIREMENT,
    };
    let bytes = serde_json::to_vec(&canonical).map_err(|_| {
        CepAuthorityError::Internal("settlement request is not serializable".into())
    })?;
    let request_sha256 = hex::encode(Sha256::digest(bytes));
    let pending_id = pending_id_for(&request_sha256);
    Ok(SettlementIdentity {
        case_id: case_id.to_string(),
        operation_id: operation_id.to_string(),
        request_sha256,
        pending_id,
        actor_role: role,
        identity_binding_hash,
    })
}

/// The pending id a canonical request hash maps to. A declaration's authorisation must derive from its own hash.
fn pending_id_for(request_sha256: &str) -> String {
    format!("{SETTLEMENT_PENDING_PREFIX}{}", &request_sha256[..32])
}

fn settlement_ledger(root: &Path) -> Result<LedgerStream, CepAuthorityError> {
    Ok(LedgerStream::open(
        root,
        LEDGER_ID,
        "sea-forge-cep-authority",
    )?)
}

fn malformed_pending() -> CepAuthorityError {
    CepAuthorityError::Internal(
        "ledger_integrity_error: malformed settlement_pending record".into(),
    )
}

fn pending_record_in(
    entries: &[LedgerEntry],
    pending_id: &str,
) -> Result<Option<SettlementPendingRecord>, CepAuthorityError> {
    entries
        .iter()
        .filter(|entry| {
            entry.record_kind == SETTLEMENT_PENDING_KIND
                && entry.payload.get("pending_id").and_then(Value::as_str) == Some(pending_id)
        })
        .map(|entry| serde_json::from_value(entry.payload.clone()).map_err(|_| malformed_pending()))
        .collect::<Result<Vec<SettlementPendingRecord>, _>>()?
        .into_iter()
        // A legacy record (no case bound) is quarantined: no lookup may match it.
        .find(|record| !record.case_id.is_empty())
        .map(Ok)
        .transpose()
}

fn require_pending(
    entries: &[LedgerEntry],
    pending_id: &str,
) -> Result<SettlementPendingRecord, CepAuthorityError> {
    pending_record_in(entries, pending_id)?.ok_or_else(|| {
        CepAuthorityError::Approval(format!("{pending_id} is not a settlement pending record"))
    })
}

/// Derive where one pending record stands from the ledger. Terminal records win; then an open claim; then the
/// approval resolution; then the window. Nothing here is cached.
fn status_in(
    entries: &[LedgerEntry],
    record: &SettlementPendingRecord,
    now: DateTime<Utc>,
) -> SettlementStatus {
    let prefix = format!("settlement-state:{}:", record.pending_id);
    let states: Vec<(&str, &LedgerEntry)> = entries
        .iter()
        .filter_map(|entry| {
            let suffix = entry.idempotency_key.as_deref()?.strip_prefix(&prefix)?;
            Some((suffix, entry))
        })
        .collect();
    let find = |suffix: &str| {
        states
            .iter()
            .find(|(s, _)| *s == suffix)
            .map(|(_, entry)| *entry)
    };
    // An attempt is closed by one create-only marker, so its latest closing outcome decides the status.
    let closing = states
        .iter()
        .filter_map(|(s, entry)| {
            let attempt = s.strip_prefix("closed:")?.parse::<u32>().ok()?;
            Some((attempt, *entry))
        })
        .max_by_key(|(attempt, _)| *attempt);
    if let Some((_, entry)) = closing {
        match text_at(&entry.payload, "/state").as_str() {
            "completed" => {
                return SettlementStatus::Completed {
                    declaration_ref: text_at(&entry.payload, "/declaration_ref"),
                }
            }
            "failed" => {
                return SettlementStatus::Failed {
                    reason: text_at(&entry.payload, "/reason"),
                }
            }
            _ => {}
        }
    }
    if let Some(entry) = find("completed") {
        return SettlementStatus::Completed {
            declaration_ref: text_at(&entry.payload, "/declaration_ref"),
        };
    }
    if let Some(entry) = find("failed") {
        return SettlementStatus::Failed {
            reason: text_at(&entry.payload, "/reason"),
        };
    }
    if find("rejected").is_some() {
        return SettlementStatus::Rejected;
    }
    if find("expired").is_some() {
        return SettlementStatus::Expired;
    }
    let latest = states
        .iter()
        .filter_map(|(s, _)| s.strip_prefix("resuming:")?.parse::<u32>().ok())
        .max();
    if let Some(attempt) = latest {
        if find(&format!("released:{attempt}")).is_none()
            && find(&format!("closed:{attempt}")).is_none()
        {
            return SettlementStatus::Resuming { attempt };
        }
    }
    let approval_id = settlement_approval_id(&record.pending_id);
    let resolution = find_entry(entries, &resolve_key(&approval_id)).map(|entry| {
        text_at(
            &entry.payload,
            "/action_request/action/parameters/resolution",
        )
    });
    let open = DateTime::parse_from_rfc3339(&record.expires_at)
        .map(|expires| now < expires.with_timezone(&Utc))
        .unwrap_or(false);
    match resolution.as_deref() {
        Some("rejected") => SettlementStatus::Rejected,
        Some("approved") if open => SettlementStatus::Approved,
        Some("approved") => SettlementStatus::Expired,
        _ if open => SettlementStatus::Pending,
        _ => SettlementStatus::Expired,
    }
}

/// Where a settlement pending record stands now, read back from the ledger.
pub fn settlement_status(
    root: &Path,
    pending_id: &str,
) -> Result<SettlementStatus, CepAuthorityError> {
    let entries = settlement_ledger(root)?.read_entries()?;
    let record = require_pending(&entries, pending_id)?;
    Ok(status_in(&entries, &record, Utc::now()))
}

/// The operation a settlement gate call is for. `declared_request_sha256` is the content hash of the declaration
/// the caller is about to make (see [`settlement_declaration_content_sha256`]). It is bound to the ingress identity
/// to form the approved request hash. A claim is refused, and its approval spent as `failed(content_mismatch)`,
/// when the declared content does not match the content that was approved.
pub struct SettlementOperation<'a> {
    pub case_id: &'a str,
    pub operation_id: &'a str,
    pub declared_request_sha256: &'a str,
}

/// The content hash of one settlement declaration request: the canonical hash (`hash_canonical`) of the request
/// the declaration will be made from, with the claim's own authorisation fields cleared. Every other field of the
/// request is covered, so changing any one of them changes this hash. Pass it as
/// `SettlementOperation::declared_request_sha256`.
pub fn settlement_declaration_content_sha256(
    request: &SettlementDeclarationRequest,
) -> Result<String, CepAuthorityError> {
    let mut content = request.clone();
    content.authorization_ref = None;
    content.authorization_request_sha256 = None;
    content.authorization_content_sha256 = None;
    crate::hash_canonical(&content).map_err(|error| CepAuthorityError::Internal(error.to_string()))
}

/// Bind the ingress identity of a settlement request to the declared content hash. The result is the approved
/// request hash: a pending record and its approval are bound to exactly this value.
pub(crate) fn bind_request_sha256(
    ingress_sha256: &str,
    declared_content_sha256: &str,
) -> Result<String, CepAuthorityError> {
    use sha2::{Digest, Sha256};
    #[derive(serde::Serialize)]
    struct Bound<'a> {
        ingress_sha256: &'a str,
        declared_content_sha256: &'a str,
    }
    let bytes = serde_json::to_vec(&Bound {
        ingress_sha256,
        declared_content_sha256,
    })
    .map_err(|_| CepAuthorityError::Internal("settlement request is not serializable".into()))?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

/// The approved request hash for one settlement declaration: the ingress identity (case, operation, actor, evidence)
/// bound to the declared content hash from [`settlement_declaration_content_sha256`].
pub fn settlement_request_sha256(
    bundle: &AuthorityPolicyBundle,
    case_id: &str,
    operation_id: &str,
    ingress: &VerifiedSettlementIngress<'_>,
    declared_content_sha256: &str,
) -> Result<String, CepAuthorityError> {
    let class = derive_settlement_authority_class(ingress);
    let identity = settlement_identity(bundle, case_id, operation_id, class, ingress)?;
    bind_request_sha256(&identity.request_sha256, declared_content_sha256)
}

/// Refuse an appended declaration that does not echo the request it was made from. The authority may add its own
/// fields (identifier, issue time, verdict, attestation). Every field the request carries must come back unchanged.
/// Returns an error naming the first field that differs.
pub fn verify_declaration_echoes_request(
    request: &SettlementDeclarationRequest,
    declaration: &SettlementDeclaration,
) -> Result<(), CepAuthorityError> {
    let echoed = [
        (
            "settlement_ref",
            request.settlement_ref == declaration.settlement_ref,
        ),
        ("run_id", request.run_id == declaration.run_id),
        ("case_id", request.case_id == declaration.case_id),
        (
            "plan_item_id",
            request.plan_item_id == declaration.plan_item_id,
        ),
        (
            "claim_manifest_sha256",
            request.claim_manifest_sha256 == declaration.claim_manifest_sha256,
        ),
        (
            "criteria_ref",
            request.criteria_ref == declaration.criteria_ref,
        ),
        (
            "criteria_sha256",
            request.criteria_sha256 == declaration.criteria_sha256,
        ),
        (
            "criteria_record_hash",
            request.criteria_record_hash == declaration.criteria_record_hash,
        ),
        (
            "criteria_declared_at",
            request.criteria_declared_at == declaration.criteria_declared_at,
        ),
        (
            "job_contract_ref",
            request.job_contract_ref == declaration.job_contract_ref,
        ),
        (
            "origin_refs",
            request.origin_refs == declaration.origin_refs,
        ),
        (
            "verifier_ref",
            request.verifier_ref == declaration.verifier_ref,
        ),
        (
            "verifier_sha256",
            request.verifier_sha256 == declaration.verifier_sha256,
        ),
        ("declarer", request.declarer == declaration.declarer),
        (
            "strength",
            request.requested_strength == declaration.strength,
        ),
        (
            "variation_tags",
            request.variation_tags == declaration.variation_tags,
        ),
        (
            "disruption_tags",
            request.disruption_tags == declaration.disruption_tags,
        ),
        (
            "orchestration_burden",
            request.orchestration_burden == declaration.orchestration_burden,
        ),
        (
            "source_evidence_refs",
            request.source_evidence_refs == declaration.source_evidence_refs,
        ),
        (
            "authored_by",
            request.authored_by == declaration.authored_by,
        ),
        (
            "authorization_ref",
            request.authorization_ref == declaration.authorization_ref,
        ),
        (
            "authorization_request_sha256",
            request.authorization_request_sha256 == declaration.authorization_request_sha256,
        ),
        (
            "authorization_content_sha256",
            request.authorization_content_sha256 == declaration.authorization_content_sha256,
        ),
        (
            "acting_entity_id",
            request.acting_entity_id == declaration.independence.acting_entity_id,
        ),
    ];
    match echoed.iter().find(|(_, same)| !same) {
        None => Ok(()),
        Some((field, _)) => Err(CepAuthorityError::Approval(format!(
            "settlement_content_mismatch: declaration {} does not echo the approved request in {field}; nothing was appended",
            declaration.declaration_id
        ))),
    }
}

/// Gate a settlement declaration from a verified ingress, before any code path calls `declare()`.
///
/// Policy is evaluated and committed on every call. An allow yields `Allowed` unless a settlement pending record
/// already holds this request, in which case the record's state decides. An approvable escalation writes (or
/// replays) a durable `settlement_pending` record and yields `Pending`, or, once the record is approved, `Resumed`.
/// Any other verdict is a refusal, which is committed and returned as an error.
pub fn authorize_settlement_declaration(
    engine: &PolicyAuthorityEngine,
    bundle: &AuthorityPolicyBundle,
    root: &Path,
    operation: SettlementOperation<'_>,
    ingress: &VerifiedSettlementIngress<'_>,
    approval_ttl: Duration,
) -> Result<SettlementGate, CepAuthorityError> {
    let SettlementOperation {
        case_id,
        operation_id,
        declared_request_sha256,
    } = operation;
    let ledger = settlement_ledger(root)?;
    let class = derive_settlement_authority_class(ingress);
    let mut identity = settlement_identity(bundle, case_id, operation_id, class, ingress)?;
    let request_sha256 = bind_request_sha256(&identity.request_sha256, declared_request_sha256)?;
    // The pending id must follow the bound request hash, the same value the commit key uses. Two calls with
    // different declared content then get different pending records, never two records under one id.
    identity.pending_id = pending_id_for(&request_sha256);
    let (decision, committed) = commit_gate_evaluation(
        engine,
        bundle,
        root,
        &ledger,
        SETTLEMENT_SURFACE,
        settlement_parameters(operation_id, class),
        operation_id,
        ingress.caller(),
    )?;
    if decision.verdict == Verdict::Allow {
        let entries = ledger.read_entries()?;
        let Some(record) = pending_record_in(&entries, &identity.pending_id)? else {
            // An external declaration is never authorised outright: it must pass an approval-gated rule.
            if class == SettlementAuthorityClass::External {
                return Err(CepAuthorityError::Approval(format!(
                    "{} may not declare externally for {operation_id} without an approval-gated rule",
                    ingress.caller().actor_id
                )));
            }
            return Ok(SettlementGate::Allowed(class));
        };
        return gate_on_status(&ledger, &record, class, &request_sha256, true, true);
    }
    if !approvable(&decision) {
        return Err(CepAuthorityError::Evidence(format!(
            "{} may not use settlement_declaration for {operation_id}: {}",
            ingress.caller().actor_id,
            decision.reason
        )));
    }
    let (record, replayed) = record_settlement_pending(
        &ledger,
        class,
        ingress,
        &identity,
        &request_sha256,
        committed.entry_ulid(),
        approval_ttl,
    )?;
    gate_on_status(&ledger, &record, class, &request_sha256, replayed, false)
}

/// Act on the current state of one pending record for a gate call. `outright` is true when policy allows the
/// request now: a pending record is then claimed without an approval, since policy no longer requires one.
fn gate_on_status(
    ledger: &LedgerStream,
    record: &SettlementPendingRecord,
    class: SettlementAuthorityClass,
    request_sha256: &str,
    replayed: bool,
    outright: bool,
) -> Result<SettlementGate, CepAuthorityError> {
    let entries = ledger.read_entries()?;
    match status_in(&entries, record, Utc::now()) {
        SettlementStatus::Completed { declaration_ref } => Ok(SettlementGate::Completed {
            pending_id: record.pending_id.clone(),
            declaration_ref,
        }),
        SettlementStatus::Resuming { attempt } => {
            refuse_approval(ledger, record, "already_claimed")?;
            Err(CepAuthorityError::Approval(format!(
                "settlement_resume_in_progress: {} attempt {attempt} is claimed and has no recorded outcome; reconcile it before resuming",
                record.pending_id
            )))
        }
        SettlementStatus::Failed { reason } => Err(CepAuthorityError::Approval(format!(
            "settlement {} failed: {reason}",
            record.pending_id
        ))),
        SettlementStatus::Rejected => {
            commit_terminal(ledger, record, "rejected", json!({}))?;
            Err(CepAuthorityError::Approval(format!(
                "settlement {} was rejected; no declaration was made",
                record.pending_id
            )))
        }
        SettlementStatus::Expired => {
            commit_terminal(ledger, record, "expired", json!({}))?;
            refuse_approval(ledger, record, "expired")?;
            Err(CepAuthorityError::Approval(format!(
                "settlement {} expired before it was approved; no declaration was made",
                record.pending_id
            )))
        }
        // An external record is claimed only once approved, even when policy now allows the request outright.
        SettlementStatus::Pending if !outright || class == SettlementAuthorityClass::External => {
            Ok(SettlementGate::Pending {
                record: Box::new(record.clone()),
                replayed,
            })
        }
        SettlementStatus::Pending | SettlementStatus::Approved => {
            // The content about to be declared must be the content that was approved. A mismatch spends the
            // approval as failed, and nothing is declared.
            if request_sha256 != record.request_sha256 {
                commit_terminal(
                    ledger,
                    record,
                    "failed",
                    json!({"reason": "content_mismatch"}),
                )?;
                return Err(CepAuthorityError::Approval(format!(
                    "settlement_content_mismatch: {} was approved for a different request; no declaration was made",
                    record.pending_id
                )));
            }
            claim_resume(ledger, &entries, record, class)
        }
    }
}

/// Claim one resume attempt. The claim is a create-only ledger record, so at most one attempt can exist per
/// attempt number, and an approval is spent by exactly one claim.
fn claim_resume(
    ledger: &LedgerStream,
    entries: &[LedgerEntry],
    record: &SettlementPendingRecord,
    class: SettlementAuthorityClass,
) -> Result<SettlementGate, CepAuthorityError> {
    let attempt = entries
        .iter()
        .filter_map(|entry| {
            let key = entry.idempotency_key.as_deref()?;
            let suffix = key.strip_prefix(&format!("settlement-state:{}:", record.pending_id))?;
            suffix.strip_prefix("resuming:")?.parse::<u32>().ok()
        })
        .max()
        .unwrap_or(0)
        + 1;
    let payload = state_payload(
        record,
        "resuming",
        json!({"attempt": attempt, "claimed_at": Utc::now().to_rfc3339()}),
    );
    match ledger.commit_typed_new(
        SETTLEMENT_STATE_KIND,
        state_key(&record.pending_id, &format!("resuming:{attempt}")),
        vec![record.operation_id.clone(), record.pending_id.clone()],
        &payload,
        vec![record.decision_ref.clone()],
    ) {
        Ok(_) => Ok(SettlementGate::Resumed {
            class,
            pending_id: record.pending_id.clone(),
            request_sha256: record.request_sha256.clone(),
            attempt,
        }),
        Err(_) => {
            refuse_approval(ledger, record, "already_used")?;
            Err(CepAuthorityError::Approval(format!(
                "settlement_approval_already_used: {} was already claimed by another resume",
                settlement_approval_id(&record.pending_id)
            )))
        }
    }
}

/// The payload of every settlement state record. It carries no timestamp, so an identical terminal transition
/// replays as the same record rather than conflicting.
fn state_payload(record: &SettlementPendingRecord, state: &str, extra: Value) -> Value {
    let mut payload = json!({
        "pending_id": record.pending_id,
        "request_sha256": record.request_sha256,
        "operation_id": record.operation_id,
        "settlement_authority_class": record.settlement_authority_class,
        "state": state,
    });
    if let (Value::Object(base), Value::Object(extra)) = (&mut payload, extra) {
        base.extend(extra);
    }
    payload
}

fn commit_terminal(
    ledger: &LedgerStream,
    record: &SettlementPendingRecord,
    suffix: &str,
    extra: Value,
) -> Result<(), CepAuthorityError> {
    ledger.commit_typed_once(
        SETTLEMENT_STATE_KIND,
        state_key(&record.pending_id, suffix),
        vec![record.operation_id.clone(), record.pending_id.clone()],
        &state_payload(record, suffix, extra),
        vec![record.decision_ref.clone()],
    )?;
    Ok(())
}

/// Close one claimed attempt with its outcome. The close is a single create-only marker per attempt, so an attempt
/// has at most one outcome: a reconciler and a worker cannot both settle it.
fn close_attempt(
    ledger: &LedgerStream,
    record: &SettlementPendingRecord,
    attempt: u32,
    outcome: &str,
    extra: Value,
) -> Result<(), CepAuthorityError> {
    ledger.commit_typed_new(
        SETTLEMENT_STATE_KIND,
        state_key(&record.pending_id, &format!("closed:{attempt}")),
        vec![record.operation_id.clone(), record.pending_id.clone()],
        &state_payload(record, outcome, extra),
        vec![record.decision_ref.clone()],
    )?;
    Ok(())
}

/// Audit a refused use of an approval. Every refusal is its own governance event.
fn refuse_approval(
    ledger: &LedgerStream,
    record: &SettlementPendingRecord,
    reason: &str,
) -> Result<(), CepAuthorityError> {
    #[derive(serde::Serialize)]
    struct Refusal<'a> {
        pending_id: &'a str,
        approval_id: String,
        request_sha256: &'a str,
        operation_id: &'a str,
        reason: &'a str,
    }
    ledger.commit_typed(
        SETTLEMENT_APPROVAL_REFUSED_KIND,
        vec![record.operation_id.clone(), record.pending_id.clone()],
        &Refusal {
            pending_id: &record.pending_id,
            approval_id: settlement_approval_id(&record.pending_id),
            request_sha256: &record.request_sha256,
            operation_id: &record.operation_id,
            reason,
        },
        vec![record.decision_ref.clone()],
    )?;
    Ok(())
}

/// Record the outcome of a claimed attempt that declared. Fenced to that attempt: once a newer attempt owns the
/// claim, or the attempt was failed or released, a stale worker's completion is refused. Idempotent for the same
/// declaration.
pub fn complete_settlement_resume(
    root: &Path,
    pending_id: &str,
    attempt: u32,
    declaration_ref: &str,
) -> Result<(), CepAuthorityError> {
    let ledger = settlement_ledger(root)?;
    let entries = ledger.read_entries()?;
    let record = require_pending(&entries, pending_id)?;
    match status_in(&entries, &record, Utc::now()) {
        SettlementStatus::Completed {
            declaration_ref: existing,
        } if existing == declaration_ref => Ok(()),
        SettlementStatus::Resuming { attempt: open } if open == attempt => match close_attempt(
            &ledger,
            &record,
            attempt,
            "completed",
            json!({"declaration_ref": declaration_ref}),
        ) {
            Ok(()) => Ok(()),
            // A concurrent settle of this attempt may have closed it with the same declaration.
            Err(error) => match settlement_status(root, pending_id)? {
                SettlementStatus::Completed {
                    declaration_ref: existing,
                } if existing == declaration_ref => Ok(()),
                _ => Err(error),
            },
        },
        other => Err(CepAuthorityError::Internal(format!(
            "settlement {pending_id} attempt {attempt} is {other:?}; it cannot complete"
        ))),
    }
}

/// Record that a claimed attempt failed for a reason that will not succeed on retry. Fenced to that attempt.
pub fn fail_settlement_resume(
    root: &Path,
    pending_id: &str,
    attempt: u32,
    reason: &str,
) -> Result<(), CepAuthorityError> {
    let ledger = settlement_ledger(root)?;
    let entries = ledger.read_entries()?;
    let record = require_pending(&entries, pending_id)?;
    match status_in(&entries, &record, Utc::now()) {
        SettlementStatus::Resuming { attempt: open } if open == attempt => {
            match close_attempt(
                &ledger,
                &record,
                attempt,
                "failed",
                json!({"reason": reason}),
            ) {
                Ok(()) => Ok(()),
                // A concurrent settle of this attempt may have failed it already: that outcome stands.
                Err(error) => match settlement_status(root, pending_id)? {
                    SettlementStatus::Failed { .. } => Ok(()),
                    _ => Err(error),
                },
            }
        }
        other => Err(CepAuthorityError::Internal(format!(
            "settlement {pending_id} attempt {attempt} is {other:?}; it cannot fail"
        ))),
    }
}

/// Release a claimed attempt that never reached the authority (the authority was unavailable). The approval
/// returns to `Approved` and a later attempt may claim it. Only the open attempt can be released.
pub fn release_settlement_resume(
    root: &Path,
    pending_id: &str,
    attempt: u32,
) -> Result<(), CepAuthorityError> {
    let ledger = settlement_ledger(root)?;
    let entries = ledger.read_entries()?;
    let record = require_pending(&entries, pending_id)?;
    match status_in(&entries, &record, Utc::now()) {
        SettlementStatus::Resuming { attempt: open } if open == attempt => close_attempt(
            &ledger,
            &record,
            attempt,
            "released",
            json!({"attempt": attempt}),
        ),
        other => Err(CepAuthorityError::Internal(format!(
            "settlement {pending_id} attempt {attempt} is {other:?}; only the open attempt can be released"
        ))),
    }
}

/// Crash recovery. A claimed attempt with no recorded outcome is settled from what the ledger holds: if the
/// caller found the declaration that attempt made, the attempt completes on it. Otherwise it stays open. A claim
/// is never failed automatically, because a timed-out authority call does not prove that no declaration was
/// appended. An open claim blocks every resume until `resolve_uncertain_settlement` records an operator's
/// resolution. Never a second declaration. Returns the status after reconciliation.
pub fn reconcile_settlement_resume(
    root: &Path,
    pending_id: &str,
    declaration_ref: Option<&str>,
) -> Result<SettlementStatus, CepAuthorityError> {
    reconcile_settlement_resume_at(root, pending_id, declaration_ref, Utc::now())
}

/// [`reconcile_settlement_resume`] judged at an explicit instant. Production recovery uses
/// [`reconcile_settlement_resume`]; this exists so lease boundaries can be tested without waiting.
pub fn reconcile_settlement_resume_at(
    root: &Path,
    pending_id: &str,
    declaration_ref: Option<&str>,
    now: DateTime<Utc>,
) -> Result<SettlementStatus, CepAuthorityError> {
    let ledger = settlement_ledger(root)?;
    let entries = ledger.read_entries()?;
    let record = require_pending(&entries, pending_id)?;
    // Without a declaration found in the ledger, the open claim is left as it is. The authority call may still have
    // appended one, or may have appended one and crashed before the ledger recorded it, so an open claim is never
    // failed here, even after its lease lapses. It blocks every resume until an operator resolves it with
    // `resolve_uncertain_settlement`.
    if let (SettlementStatus::Resuming { attempt }, Some(found)) =
        (status_in(&entries, &record, now), declaration_ref)
    {
        complete_settlement_resume(root, pending_id, attempt, found)?;
    }
    settlement_status(root, pending_id)
}

const SETTLEMENT_UNCERTAIN_RESOLUTION_KIND: &str = "settlement_uncertain_resolution";

/// How an operator resolves an open claim whose outcome is uncertain: the lease lapsed and the ledger holds no
/// declaration for the attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UncertainResolution {
    /// The operator found the declaration the attempt made in the external authority's record. The attempt
    /// completes on it, and the record is kept as the operator's evidence.
    Declared { declaration_ref: String },
    /// The operator attests that the external authority holds no declaration for this attempt. The claim is
    /// released, so a fenced retry can claim it again. A stale worker from this attempt cannot complete later.
    NotDeclared { evidence: String },
}

/// Resolve an uncertain open claim. Only an operator may resolve it, and only after the lease has lapsed, so the
/// attempt's authority call cannot still be running. The resolution is audited before the attempt is closed, and
/// the close is fenced to the attempt, so a retried resolution cannot settle a newer attempt.
pub fn resolve_uncertain_settlement(
    engine: &PolicyAuthorityEngine,
    bundle: &AuthorityPolicyBundle,
    root: &Path,
    pending_id: &str,
    attempt: u32,
    resolution: UncertainResolution,
    resolver: &Actor,
) -> Result<SettlementStatus, CepAuthorityError> {
    resolve_uncertain_settlement_at(
        engine,
        bundle,
        root,
        pending_id,
        attempt,
        resolution,
        resolver,
        Utc::now(),
    )
}

/// [`resolve_uncertain_settlement`] judged at an explicit instant, so lease boundaries can be tested.
#[allow(clippy::too_many_arguments)]
pub fn resolve_uncertain_settlement_at(
    engine: &PolicyAuthorityEngine,
    bundle: &AuthorityPolicyBundle,
    root: &Path,
    pending_id: &str,
    attempt: u32,
    resolution: UncertainResolution,
    resolver: &Actor,
    now: DateTime<Utc>,
) -> Result<SettlementStatus, CepAuthorityError> {
    if resolver.role != sea_forge_core::types::ActorRole::Operator {
        return Err(CepAuthorityError::Approval(format!(
            "{} may not resolve an uncertain settlement; only an operator may",
            resolver.actor_id
        )));
    }
    let ledger = settlement_ledger(root)?;
    let entries = ledger.read_entries()?;
    let record = require_pending(&entries, pending_id)?;
    // The same `approval_resolution` decision that governs resolving an approval governs resolving its uncertain
    // claim, evaluated before anything is read or written about the attempt. A refusal is audited, and fails closed.
    let approval_id = settlement_approval_id(&record.pending_id);
    let action = AuthorityAction::Reserved {
        resource_type: "approval_resolution".into(),
        resource_id: approval_id.clone(),
        parameters: json!({
            "approval_id": approval_id,
            "original_decision_id": record.decision_ref,
            "requester_id": record.actor_id,
            // The rule matches on the approval's own resolution vocabulary. The claim belongs to an approved
            // request, so the resolver exercises the same authority; `uncertain_resolution` says what for.
            "resolution": "approved",
            "uncertain_resolution": match &resolution {
                UncertainResolution::Declared { .. } => "declared",
                UncertainResolution::NotDeclared { .. } => "not_declared",
            },
            "required_approver_roles": [],
        }),
    };
    let binding = bundle.resolve_identity(&resolver.actor_id, resolver.role.clone());
    let decision = engine.evaluate(AuthorityEvaluation {
        actor: resolver,
        binding,
        run_id: &approval_id,
        case_id: "cognate",
        plan_item_id: &approval_id,
        sequence: 1,
        action: &action,
        workspace_root: root,
        evidence_refs: vec![format!("approval:{approval_id}")],
        artifacts_root: None,
        timeout_secs: None,
        env_keys: Default::default(),
        opa_candidate: None,
        environment: None,
    })?;
    if decision.verdict != Verdict::Allow {
        ledger.commit_typed(
            "authority_decision",
            vec![record.operation_id.clone(), record.pending_id.clone()],
            &decision,
            vec![record.decision_ref.clone()],
        )?;
        return Err(CepAuthorityError::Approval(format!(
            "{} may not resolve the uncertain settlement {approval_id}: {}",
            resolver.actor_id, decision.reason
        )));
    }
    match status_in(&entries, &record, now) {
        SettlementStatus::Resuming { attempt: open } if open == attempt => {}
        other => {
            return Err(CepAuthorityError::Internal(format!(
                "settlement {pending_id} attempt {attempt} is {other:?}; only the open attempt can be resolved"
            )))
        }
    }
    if !claim_lease_lapsed(&entries, &record, attempt, now)? {
        return Err(CepAuthorityError::Approval(format!(
            "settlement {pending_id} attempt {attempt} lease has not lapsed; the attempt may still be calling the authority"
        )));
    }
    let (kind, detail) = match &resolution {
        UncertainResolution::Declared { declaration_ref } => ("declared", declaration_ref.as_str()),
        UncertainResolution::NotDeclared { evidence } => ("not_declared", evidence.as_str()),
    };
    // One audit record per attempt, create-once: of several operators racing past the status check, the first
    // record wins. An identical retry replays it, and a conflicting resolution is refused here, so the audit log
    // never holds a resolution that did not take effect.
    ledger.commit_typed_once(
        SETTLEMENT_UNCERTAIN_RESOLUTION_KIND,
        format!("settlement-uncertain:{}:{attempt}", record.pending_id),
        vec![record.operation_id.clone(), record.pending_id.clone()],
        &json!({
            "pending_id": record.pending_id,
            "attempt": attempt,
            "resolution": kind,
            "detail": detail,
            "resolver_actor_id": resolver.actor_id,
        }),
        vec![record.decision_ref.clone()],
    )?;
    match resolution {
        UncertainResolution::Declared { declaration_ref } => {
            complete_settlement_resume(root, pending_id, attempt, &declaration_ref)?
        }
        UncertainResolution::NotDeclared { .. } => {
            release_settlement_resume(root, pending_id, attempt)?
        }
    }
    settlement_status(root, pending_id)
}

/// Whether an open attempt's lease has lapsed. The lease starts when the claim is written, which is before the
/// attempt calls the authority. `SETTLEMENT_CLAIM_LEASE_SECS` is longer than the longest authority timeout a
/// descriptor may carry (validated on load), so a lapsed lease means that call has ended.
fn claim_lease_lapsed(
    entries: &[LedgerEntry],
    record: &SettlementPendingRecord,
    attempt: u32,
    now: DateTime<Utc>,
) -> Result<bool, CepAuthorityError> {
    let key = state_key(&record.pending_id, &format!("resuming:{attempt}"));
    let entry = find_entry(entries, &key).ok_or_else(|| {
        CepAuthorityError::Internal(format!(
            "settlement {} attempt {attempt} has no claim record",
            record.pending_id
        ))
    })?;
    let claimed_at = DateTime::parse_from_rfc3339(&text_at(&entry.payload, "/claimed_at"))
        .map_err(|error| {
            CepAuthorityError::Internal(format!(
                "settlement {} attempt {attempt} claim has no valid claimed_at: {error}",
                record.pending_id
            ))
        })?
        .with_timezone(&Utc);
    Ok(now >= claimed_at + chrono::Duration::seconds(SETTLEMENT_CLAIM_LEASE_SECS))
}

/// Reads a conflicting racer makes for the record the winner wrote before it gives up.
const RACE_READ_ATTEMPTS: u32 = 50;

/// After a conflicting commit, look for the record the winner wrote. The winner's record can land a moment after
/// the loser's conflict, so re-read for a bounded interval rather than decide the record is absent.
fn existing_pending_after_conflict(
    mut read: impl FnMut() -> Result<Option<SettlementPendingRecord>, CepAuthorityError>,
    attempts: u32,
) -> Result<Option<SettlementPendingRecord>, CepAuthorityError> {
    for attempt in 0..attempts {
        if let Some(existing) = read()? {
            return Ok(Some(existing));
        }
        if attempt + 1 < attempts {
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }
    Ok(None)
}

/// Write the pending record once per canonical request. An identical request that races this one loses the
/// typed-once commit on the payload (timestamps differ); it then returns the record the winner wrote.
fn record_settlement_pending(
    ledger: &LedgerStream,
    class: SettlementAuthorityClass,
    ingress: &VerifiedSettlementIngress<'_>,
    identity: &SettlementIdentity,
    request_sha256: &str,
    decision_ref: &str,
    approval_ttl: Duration,
) -> Result<(SettlementPendingRecord, bool), CepAuthorityError> {
    if let Some(existing) = pending_record_in(&ledger.read_entries()?, &identity.pending_id)? {
        return Ok((existing, true));
    }
    let caller = ingress.caller();
    let now = Utc::now();
    let record = SettlementPendingRecord {
        pending_id: identity.pending_id.clone(),
        request_sha256: request_sha256.to_string(),
        ingress_sha256: identity.request_sha256.clone(),
        case_id: identity.case_id.clone(),
        operation_id: identity.operation_id.clone(),
        operation_kind: SETTLEMENT_SURFACE.into(),
        settlement_authority_class: class.as_str().into(),
        ingress: ingress.ingress().as_str().into(),
        actor_id: caller.actor_id.clone(),
        actor_role: identity.actor_role.clone(),
        identity_binding_hash: identity.identity_binding_hash.clone(),
        evidence_refs: ingress.evidence_refs().to_vec(),
        approval_requirement: SETTLEMENT_APPROVAL_REQUIREMENT.into(),
        decision_ref: decision_ref.to_string(),
        created_at: now.to_rfc3339(),
        expires_at: (now + approval_ttl).to_rfc3339(),
    };
    match ledger.commit_typed_once(
        SETTLEMENT_PENDING_KIND,
        format!("{SETTLEMENT_PENDING_KEY}{request_sha256}"),
        vec![identity.operation_id.clone(), identity.pending_id.clone()],
        &record,
        vec![decision_ref.to_string()],
    ) {
        Ok(_) => Ok((record, false)),
        Err(error) => match existing_pending_after_conflict(
            || pending_record_in(&ledger.read_entries()?, &identity.pending_id),
            RACE_READ_ATTEMPTS,
        )? {
            Some(existing) => Ok((existing, true)),
            None => Err(error.into()),
        },
    }
}

/// Every settlement pending record in the authority ledger, oldest first, read back from disk.
pub fn settlement_pending_records(
    root: &Path,
) -> Result<Vec<SettlementPendingRecord>, CepAuthorityError> {
    let ledger = LedgerStream::open(root, LEDGER_ID, "sea-forge-cep-authority")?;
    ledger
        .read_entries()?
        .into_iter()
        .filter(|entry| entry.record_kind == SETTLEMENT_PENDING_KIND)
        .map(|entry| serde_json::from_value(entry.payload).map_err(|_| malformed_pending()))
        .collect()
}

/// Standing of a settlement approval, derived from its pending record and the ledger.
fn settlement_standing_in(
    entries: &[LedgerEntry],
    approval_id: &str,
    now: DateTime<Utc>,
) -> Option<ApprovalStanding> {
    let pending_id = approval_id.strip_prefix("apr-")?;
    if !pending_id.starts_with(SETTLEMENT_PENDING_PREFIX) {
        return None;
    }
    let record = pending_record_in(entries, pending_id).ok()??;
    let state = match status_in(entries, &record, now) {
        SettlementStatus::Pending => ApprovalState::Pending,
        SettlementStatus::Approved => ApprovalState::Approved,
        SettlementStatus::Rejected => ApprovalState::Rejected,
        SettlementStatus::Expired => ApprovalState::Expired,
        SettlementStatus::Resuming { .. }
        | SettlementStatus::Completed { .. }
        | SettlementStatus::Failed { .. } => ApprovalState::Used,
    };
    let resolution = find_entry(entries, &resolve_key(approval_id));
    Some(ApprovalStanding {
        approval_id: approval_id.to_string(),
        state,
        requester: record.actor_id.clone(),
        world_ref: String::new(),
        kind: SETTLEMENT_SURFACE.into(),
        operation_name: String::new(),
        resource_id: record.operation_id.clone(),
        subject_actor_ref: String::new(),
        original_request_envelope_id: String::new(),
        escalation_entry: record.decision_ref.clone(),
        expires_at: DateTime::parse_from_rfc3339(&record.expires_at)
            .ok()?
            .with_timezone(&Utc),
        resolved_by: resolution.map(|r| text_at(&r.payload, "/action_request/actor/actor_id")),
    })
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

#[cfg(test)]
mod race_read_tests {
    use super::*;
    use std::cell::Cell;

    fn record(pending_id: &str) -> SettlementPendingRecord {
        SettlementPendingRecord {
            pending_id: pending_id.into(),
            request_sha256: "a".repeat(64),
            ingress_sha256: "b".repeat(64),
            case_id: "case_a".into(),
            operation_id: "run_a".into(),
            operation_kind: SETTLEMENT_SURFACE.into(),
            settlement_authority_class: "external".into(),
            ingress: "swe_seed_declaration".into(),
            actor_id: "cognate-service".into(),
            actor_role: "service".into(),
            identity_binding_hash: None,
            evidence_refs: vec![],
            approval_requirement: SETTLEMENT_APPROVAL_REQUIREMENT.into(),
            decision_ref: "dec_a".into(),
            created_at: "2026-11-03T00:00:00Z".into(),
            expires_at: "2026-11-04T00:00:00Z".into(),
        }
    }

    /// A conflicting racer can look before the winner's record is visible. Its re-read must find the record once
    /// it lands, rather than deciding the record is absent.
    #[test]
    fn conflict_reread_finds_the_winners_record_once_it_is_visible() {
        let reads = Cell::new(0);
        let found = existing_pending_after_conflict(
            || {
                reads.set(reads.get() + 1);
                Ok((reads.get() >= 3).then(|| record("settlement-pending-x")))
            },
            RACE_READ_ATTEMPTS,
        )
        .unwrap();
        assert_eq!(
            found.map(|r| r.pending_id),
            Some("settlement-pending-x".into())
        );
        assert_eq!(reads.get(), 3);
    }

    #[test]
    fn conflict_reread_gives_up_after_a_bounded_number_of_reads() {
        let reads = Cell::new(0);
        let found = existing_pending_after_conflict(
            || {
                reads.set(reads.get() + 1);
                Ok(None)
            },
            RACE_READ_ATTEMPTS,
        )
        .unwrap();
        assert!(found.is_none());
        assert_eq!(reads.get(), RACE_READ_ATTEMPTS);
    }
}
