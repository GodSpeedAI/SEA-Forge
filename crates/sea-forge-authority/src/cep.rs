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
use chrono::Utc;
use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::{
    Actor, AuthorityAction, AuthorityDecision, NormalizedDisposition, Verdict,
};
use sea_forge_domainforge::WorldRegistry;
use sea_forge_ledger::{CommittedRecordRef, LedgerStream};
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
}

impl CepAuthorityService<'_> {
    /// Decide one request as the verified `caller`. The decision is committed
    /// before it is returned.
    pub fn decide(
        &self,
        envelope: &Value,
        caller: &Actor,
    ) -> Result<CepDecision, CepAuthorityError> {
        let request = parse_request(envelope, self.worlds)?;
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
        let decision = self.engine.evaluate(AuthorityEvaluation {
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
        })?;
        let ledger = LedgerStream::open(self.root, LEDGER_ID, "sea-forge-cep-authority")?;
        let committed = ledger.commit_typed_once(
            "authority_decision",
            format!("cep:{}", request.operation_id),
            vec!["cognate".into(), request.operation_id.clone()],
            &decision,
            vec![],
        )?;
        let envelope = decision_envelope(envelope, &request, &decision, &committed)?;
        Ok(CepDecision {
            envelope,
            decision,
            committed,
        })
    }
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
        "lineage_refs": [request.envelope_id],
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
