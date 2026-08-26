//! Governed execution boundary (convergence plan task T05; frozen edges
//! E5A/E5B, invariant I5).
//!
//! This module pins the authority-to-side-effect and
//! observation-to-authorized-invocation contracts at the SEA-Forge execution
//! boundary, over the same canonical v1 envelope family the upstream T04 gate
//! (`governed_work_ingress`) admits:
//!
//! - **E5A `AuthorizedInvocation`** (sea_forge -> execution_environment):
//!   emitted ONLY from a real [`AuthorityDecision`] whose verdict AND
//!   normalized disposition are `Allow` — for a deny/escalate/boundary/degraded
//!   decision no canonical invocation envelope exists at all. The envelope
//!   carries the frozen identity triple (work_request_id, invocation_id,
//!   authority_decision_id) plus operation/resource/execution_constraints and
//!   cites its GovernedWorkRequest parent (ENV-I4). The execution environment
//!   cannot self-assert authority: nothing on the wire is consultable as
//!   authority — only the decision object issued by the real engine.
//! - **E5B `ExecutionObservation`** (execution_environment -> sea_forge):
//!   adjudicated by [`InvocationLedger`], which binds every observation to the
//!   EXACT authorized invocation — same invocation_id, same recorded
//!   authority_decision_id, same work_request_id, current generation of its
//!   invocation slot, and citing that invocation's E5A event as causal parent.
//!   Late (superseded generation), unknown, duplicated, cross-wired, or
//!   self-authorizing observations never settle.
//!
//! Invariant I5 (authority precedes side effect) is composed with, not
//! reimplemented: the real runtime path already makes the side effect
//! unreachable without a move-only [`sea_forge_authority::ActionGrant`] minted
//! from an Allow decision committed to the ledger (`case_dispatch` /
//! `sea_forge_runtime::execute`). What this boundary adds is the
//! cross-component statement: no canonical AuthorizedInvocation exists for a
//! non-Allow decision, and no observation can settle as governed execution
//! without one.
//!
//! Pure functions over decoded envelope JSON plus real domain types —
//! transport-free by design, mirroring the T04 ingress precedent.

use getrandom::fill;
use sea_forge_core::types::{AuthorityAction, AuthorityDecision, NormalizedDisposition, Verdict};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

/// Wire schema version of the canonical envelope family.
pub(crate) const SCHEMA_VERSION: &str = "v1";

/// Canonical namespace of this capability loop (SWE_SEED federation parity).
pub const NAMESPACE: &str = "agentic_capability_loop";

/// Event type owned exclusively by `sea_forge` at edge E5A.
pub const AUTHORIZED_INVOCATION: &str = "AuthorizedInvocation";

/// Event type owned exclusively by `execution_environment` at edge E5B.
pub const EXECUTION_OBSERVATION: &str = "ExecutionObservation";

/// The standalone-fallback pseudo-hash input (federation parity):
/// `sha256("agentic_capability_loop")`. A payload declaring it has no real
/// model behind it.
pub(crate) fn fallback_pseudo_hash() -> String {
    let mut h = Sha256::new();
    h.update(b"agentic_capability_loop");
    format!("{:x}", h.finalize())
}

/// Map a wire `source_agent` onto its canonical component id. Same vocabulary
/// and fail-closed posture as the T04 ingress: hyphenated legacy stamps alias
/// their underscored preregistration ids; anything unknown is refused rather
/// than coerced.
pub(crate) fn canonical_agent(source_agent: &str) -> Option<&'static str> {
    match source_agent {
        "swe-seed" | "swe_seed" => Some("swe_seed"),
        "context-kernel" | "context_kernel" => Some("context_kernel"),
        "sea-forge" | "sea_forge" => Some("sea_forge"),
        "godspeed-agent" | "godspeed_agent" | "gsa" => Some("godspeed_agent"),
        "realitytrace" | "reality-trace" | "sxr" => Some("realitytrace"),
        "memory-ledger" | "memory_ledger" | "agent_memory_ledger" => Some("memory_ledger"),
        "execution-environment" | "execution_environment" => Some("execution_environment"),
        "external-environment" | "external_environment" => Some("external_environment"),
        _ => None,
    }
}

pub(crate) fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Structural validity + refusal of known pseudo-identities (the standalone
/// fallback constant and the all-zero digest). Nothing manufactures identity.
pub(crate) fn verify_model_identity(declared: &str) -> Result<(), String> {
    let placeholders = [fallback_pseudo_hash(), "0".repeat(64)];
    if !is_sha256_hex(declared) || placeholders.iter().any(|p| p == declared) {
        return Err(declared.to_string());
    }
    Ok(())
}

/// Non-empty after trimming; used for correlation/identity strings.
pub(crate) fn meaningful(s: Option<&str>) -> Option<String> {
    s.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Explicit placeholder identities that must never settle a binding.
pub(crate) const PLACEHOLDER_IDS: [&str; 5] =
    ["placeholder", "unknown", "none", "<missing>", "tbd"];

pub(crate) fn check_identity_string(field: &'static str, value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || PLACEHOLDER_IDS.contains(&trimmed.to_ascii_lowercase().as_str()) {
        return Err(field.to_string());
    }
    Ok(trimmed.to_string())
}

/// Compact sorted-key JSON of a payload, signature excluded — the exact
/// content hash input of the family's idempotency key (F-10 parity with the
/// SWE_SEED producer substrate).
pub(crate) fn canonical_payload_json(payload: &Value) -> String {
    fn sorted_without_signature(value: &Value) -> Value {
        match value {
            Value::Object(obj) => {
                let mut keys: Vec<_> = obj.keys().collect();
                keys.sort();
                let mut sorted = Map::new();
                for key in keys {
                    if key == "signature" {
                        continue;
                    }
                    sorted.insert(key.clone(), sorted_without_signature(&obj[key]));
                }
                Value::Object(sorted)
            }
            Value::Array(items) => {
                Value::Array(items.iter().map(sorted_without_signature).collect())
            }
            other => other.clone(),
        }
    }
    serde_json::to_string(&sorted_without_signature(payload)).unwrap_or_default()
}

/// Content-derived dedup key: `sha256("{correlation}|{event_type}|{payload}")`
/// (F-10). Stable across re-wraps that keep the payload intact.
pub(crate) fn idempotency_key(event_type: &str, payload: &Value) -> String {
    let content = format!("{}|{}|{}", "", event_type, canonical_payload_json(payload));
    format!("{:x}", Sha256::digest(content.as_bytes()))
}

pub(crate) fn utc_now() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S+00:00")
        .to_string()
}

/// Fresh UUID-v4-shaped event id from OS randomness.
pub(crate) fn new_event_id() -> String {
    let mut bytes = [0_u8; 16];
    fill(&mut bytes).expect("OS RNG is always available");
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

pub(crate) fn causal_parents(envelope: &Value) -> Vec<&str> {
    envelope
        .get("provenance")
        .and_then(|p| p.get("chain"))
        .and_then(Value::as_array)
        .map(|chain| {
            chain
                .iter()
                .filter_map(Value::as_str)
                .filter_map(|e| e.strip_prefix("caused_by:"))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn check_envelope_producer(
    envelope: &Value,
    expected_event: &'static str,
    authoritative: &'static str,
) -> Result<(), BoundaryError> {
    if envelope.get("schema_version").and_then(Value::as_str) != Some(SCHEMA_VERSION) {
        return Err(BoundaryError::MalformedEnvelope(
            "schema_version must be \"v1\"".into(),
        ));
    }
    match envelope.get("event_type").and_then(Value::as_str) {
        Some(got) if got == expected_event => {}
        Some(got) => {
            return Err(BoundaryError::WrongEventType {
                expected: expected_event,
                got: got.to_string(),
            })
        }
        None => {
            return Err(BoundaryError::MalformedEnvelope(
                "event_type missing".into(),
            ))
        }
    }
    let stamp = envelope
        .get("source_agent")
        .and_then(Value::as_str)
        .ok_or_else(|| BoundaryError::UnknownAgent {
            got: "<missing>".into(),
        })?;
    match canonical_agent(stamp) {
        None => Err(BoundaryError::UnknownAgent {
            got: stamp.to_string(),
        }),
        Some(canonical) if canonical == authoritative => Ok(()),
        Some(_) => Err(BoundaryError::NotAuthoritative {
            authoritative,
            got: stamp.to_string(),
        }),
    }
}

pub(crate) fn check_payload_model_binding(
    envelope: &Value,
    local_model_sha256: &str,
) -> Result<(), BoundaryError> {
    let declared = envelope
        .get("payload")
        .and_then(|p| p.get("domain_model_hash"))
        .and_then(Value::as_str)
        .ok_or_else(|| BoundaryError::PlaceholderIdentity {
            got: "<missing domain_model_hash>".into(),
        })?;
    verify_model_identity(declared).map_err(|got| BoundaryError::PlaceholderIdentity { got })?;
    if declared != local_model_sha256.trim() {
        return Err(BoundaryError::DomainDrift {
            declared: declared.to_string(),
            local: local_model_sha256.trim().to_string(),
        });
    }
    Ok(())
}

/// Why the boundary refused an envelope or an emission attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundaryError {
    /// Authority has not permitted this invocation (deny / escalate / boundary
    /// / degraded disposition): NO canonical AuthorizedInvocation exists.
    AuthorityNotGranted { disposition: String },
    /// The governed correlation is absent or blank.
    OpaqueWorkRequest,
    /// A required identity/correlation field is missing, blank, or a literal
    /// placeholder. Carries the offending field name.
    PlaceholderField { field: String },
    /// Declared model hash is malformed or a known placeholder/fallback
    /// pseudo-identity (ENV-I2).
    PlaceholderIdentity { got: String },
    /// Declared model hash differs from the locally resolved canonical model.
    DomainDrift { declared: String, local: String },
    /// Not a decodable canonical v1 envelope object.
    MalformedEnvelope(String),
    /// Right shape, wrong event type.
    WrongEventType { expected: &'static str, got: String },
    /// `source_agent` outside the canonical vocabulary entirely.
    UnknownAgent { got: String },
    /// A known component forging another's authoritative event (I3).
    NotAuthoritative {
        authoritative: &'static str,
        got: String,
    },
    /// Required payload fields absent, empty, or of the wrong shape.
    OpaquePayload { missing: Vec<&'static str> },
    /// `execution_status` outside the governed vocabulary.
    InvalidExecutionStatus { got: String },
    /// An optional-but-typed field arrived with the wrong shape.
    InvalidOptionalField { field: &'static str },
    /// The derivation does not record the required causal parent (ENV-I4).
    CausalityMissing {
        expected_parent: String,
        recorded: Vec<String>,
    },
    /// No authorized invocation exists under the cited identity.
    UnknownInvocation { invocation_id: String },
    /// The observation belongs to a superseded generation of its slot.
    LateObservation {
        invocation_id: String,
        settled_generation: u64,
        current_generation: u64,
    },
    /// The observation correlates to a different work request than the
    /// invocation actually authorized.
    CrossWiredWorkRequest { expected: String, got: String },
    /// The response self-supplied an authority decision that was not the one
    /// authorizing the invocation (fabricated authority has no effect).
    SelfAssertedAuthority { recorded: String, claimed: String },
}

impl std::fmt::Display for BoundaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthorityNotGranted { disposition } => write!(
                f,
                "authority has not permitted this invocation (disposition: {disposition}); no AuthorizedInvocation may exist"
            ),
            Self::OpaqueWorkRequest => write!(f, "work_request_id is missing or blank"),
            Self::PlaceholderField { field } => {
                write!(f, "{field} is a placeholder or blank identity")
            }
            Self::PlaceholderIdentity { got } => write!(
                f,
                "fallback/placeholder pseudo-hash rejected as domain identity: {got}"
            ),
            Self::DomainDrift { declared, local } => write!(
                f,
                "domain model drift: envelope declares {declared}, local canonical model is {local}"
            ),
            Self::MalformedEnvelope(m) => write!(f, "malformed canonical envelope: {m}"),
            Self::WrongEventType { expected, got } => {
                write!(f, "expected {expected} event, got '{got}'")
            }
            Self::UnknownAgent { got } => {
                write!(f, "source_agent outside the canonical vocabulary: {got}")
            }
            Self::NotAuthoritative {
                authoritative,
                got,
            } => write!(
                f,
                "event may only be produced by '{authoritative}', forge attempted by '{got}'"
            ),
            Self::OpaquePayload { missing } => write!(
                f,
                "opaque execution payload rejected — missing/invalid fields: {}",
                missing.join(", ")
            ),
            Self::InvalidExecutionStatus { got } => {
                write!(f, "execution_status outside the governed vocabulary: {got}")
            }
            Self::InvalidOptionalField { field } => {
                write!(f, "optional field {field} has the wrong shape")
            }
            Self::CausalityMissing {
                expected_parent,
                recorded,
            } => write!(
                f,
                "observation does not cite its authorized invocation as causal parent: expected {expected_parent}, recorded {recorded:?}"
            ),
            Self::UnknownInvocation { invocation_id } => write!(
                f,
                "no authorized invocation exists under invocation_id {invocation_id:?}"
            ),
            Self::LateObservation {
                invocation_id,
                settled_generation,
                current_generation,
            } => write!(
                f,
                "late observation: invocation {invocation_id} is generation {settled_generation} but slot is at generation {current_generation}"
            ),
            Self::CrossWiredWorkRequest { expected, got } => write!(
                f,
                "cross-wired observation: invocation was authorized for work_request_id {expected:?}, observation carries {got:?}"
            ),
            Self::SelfAssertedAuthority { recorded, claimed } => write!(
                f,
                "self-asserted authority rejected: invocation was authorized by decision {recorded:?}, response claims {claimed:?}"
            ),
        }
    }
}

impl std::error::Error for BoundaryError {}

// ---------------------------------------------------------------------------
// E5A — AuthorizedInvocation emission (producer: sea_forge)
// ---------------------------------------------------------------------------

/// A canonical AuthorizedInvocation emitted at the execution boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthorizedInvocation {
    /// The full canonical v1 envelope — the only thing that crosses to the
    /// execution environment.
    pub envelope: Value,
    pub event_id: String,
    pub invocation_id: String,
    pub work_request_id: String,
    pub authority_decision_id: String,
    pub operation: String,
    pub resource: String,
}

/// Total projection of an [`AuthorityAction`] into the frozen
/// `(operation, resource)` pair. Every variant maps; nothing defaults
/// silently.
fn operation_and_resource(action: &AuthorityAction) -> (String, String) {
    match action {
        AuthorityAction::WriteFile { path, .. } => ("write_file".into(), path.clone()),
        AuthorityAction::ExecuteCommand { argv, .. } => (
            "execute_command".into(),
            argv.first().cloned().unwrap_or_default(),
        ),
        AuthorityAction::ExternalApi { host } => ("external_api".into(), host.clone()),
        AuthorityAction::AgentProbe { host, .. } => ("external_api".into(), host.clone()),
        AuthorityAction::AgentTask { host, .. } => ("external_api".into(), host.clone()),
        AuthorityAction::GitCommit { .. } => ("git_commit".into(), "repository".into()),
        AuthorityAction::GithubPr { .. } => ("github_pr".into(), "pull_request".into()),
        AuthorityAction::Reserved {
            resource_type,
            resource_id,
            ..
        } => (resource_type.clone(), resource_id.clone()),
        // Unclassified actions are always denied by the engine, so this arm is
        // unreachable behind the Allow gate below; kept total regardless.
        AuthorityAction::Unclassified { raw_kind, .. } => ("unclassified".into(), raw_kind.clone()),
    }
}

fn fresh_invocation_id() -> String {
    let mut bytes = [0_u8; 8];
    fill(&mut bytes).expect("OS RNG is always available");
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("inv_{hex}")
}

/// Emit one canonical AuthorizedInvocation (E5A) from a REAL authority
/// decision.
///
/// Gates, in order:
/// 1. **Authority precedes existence (I5):** the decision must carry verdict
///    `Allow` AND normalized disposition `Allow`. For deny, escalate,
///    boundary, or degraded dispositions no invocation envelope is produced —
///    there is nothing for an execution environment to receive, and the
///    runtime path independently refuses any non-Allow grant.
/// 2. The governed correlation (`work_request_id`) must be meaningful.
/// 3. Canonical domain identity: the declared model hash must be structurally
///    valid, never a placeholder/fallback pseudo-hash, and equal to the
///    locally resolved canonical model (drift rejected).
/// 4. Causality (ENV-I4): the GovernedWorkRequest event id this invocation
///    descends from is mandatory — an authorized invocation without its
///    governing work request is unrepresentable here.
///
/// `grant_id` is projected verbatim when the caller holds one (optional frozen
/// field); it names the in-memory ActionGrant minted alongside, never a
/// self-asserted authority fact.
#[allow(clippy::too_many_arguments)]
pub fn emit_authorized_invocation(
    decision: &AuthorityDecision,
    work_request_id: &str,
    declared_model_sha256: &str,
    local_model_sha256: &str,
    governed_work_request_event_id: &str,
    grant_id: Option<&str>,
) -> Result<AuthorizedInvocation, BoundaryError> {
    // 1. Authority gate — deny/escalate/boundary/degraded produce NOTHING.
    if decision.verdict != Verdict::Allow
        || decision.normalized_disposition != NormalizedDisposition::Allow
    {
        return Err(BoundaryError::AuthorityNotGranted {
            disposition: format!("{:?}", decision.normalized_disposition),
        });
    }

    // 2. Meaningful governed correlation.
    let work_request_id =
        check_identity_string("work_request_id", work_request_id).map_err(|field| {
            if field == "work_request_id" {
                BoundaryError::OpaqueWorkRequest
            } else {
                BoundaryError::PlaceholderField { field }
            }
        })?;

    // 3. Canonical identity, drift-checked against our own resolution.
    verify_model_identity(declared_model_sha256)
        .map_err(|got| BoundaryError::PlaceholderIdentity { got })?;
    if declared_model_sha256 != local_model_sha256.trim() {
        return Err(BoundaryError::DomainDrift {
            declared: declared_model_sha256.to_string(),
            local: local_model_sha256.trim().to_string(),
        });
    }

    // 4. Mandatory causal lineage to the governing work request.
    let causal_parent = check_identity_string(
        "governed_work_request_event_id",
        governed_work_request_event_id,
    )
    .map_err(|field| BoundaryError::PlaceholderField { field })?;

    let (operation, resource) = operation_and_resource(&decision.operation);
    let invocation_id = fresh_invocation_id();

    // Execution constraints project ONLY authority-side facts the decision
    // itself carries — never anything an executor could have chosen.
    let mut constraints = Map::new();
    if let Some(class) = decision.sandbox_class_granted.as_deref() {
        constraints.insert("sandbox_class".into(), Value::String(class.to_string()));
    }
    if let Some(timeout) = decision
        .action_request
        .context
        .get("timeout_secs")
        .and_then(Value::as_u64)
    {
        constraints.insert("timeout_secs".into(), Value::from(timeout));
    }
    if !decision.compensating_controls.is_empty() {
        constraints.insert(
            "compensating_controls".into(),
            Value::Array(
                decision
                    .compensating_controls
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect(),
            ),
        );
    }

    let mut payload = Map::new();
    payload.insert(
        "domain_model_hash".into(),
        Value::String(declared_model_sha256.to_string()),
    );
    payload.insert("namespace".into(), Value::String(NAMESPACE.into()));
    payload.insert(
        "work_request_id".into(),
        Value::String(work_request_id.clone()),
    );
    payload.insert("invocation_id".into(), Value::String(invocation_id.clone()));
    payload.insert(
        "authority_decision_id".into(),
        Value::String(decision.decision_id.clone()),
    );
    payload.insert("operation".into(), Value::String(operation.clone()));
    payload.insert("resource".into(), Value::String(resource.clone()));
    payload.insert("execution_constraints".into(), Value::Object(constraints));
    if let Some(grant) = grant_id.map(str::trim).filter(|g| !g.is_empty()) {
        payload.insert("grant_id".into(), Value::String(grant.to_string()));
    }
    let payload = Value::Object(payload);

    let mut chain = vec![format!("domain_model_hash:{declared_model_sha256}")];
    chain.push(format!("caused_by:{causal_parent}"));
    let event_id = new_event_id();
    let envelope = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "event_id": event_id,
        "source_agent": "sea-forge",
        "event_type": AUTHORIZED_INVOCATION,
        "occurred_at": utc_now(),
        "idempotency_key": idempotency_key(AUTHORIZED_INVOCATION, &payload),
        "payload": payload,
        "provenance": {
            "origin": "sea-forge",
            "chain": chain,
        },
    });

    Ok(AuthorizedInvocation {
        envelope,
        event_id,
        invocation_id,
        work_request_id,
        authority_decision_id: decision.decision_id.clone(),
        operation,
        resource,
    })
}

// ---------------------------------------------------------------------------
// E5B — ExecutionObservation adjudication (producer: execution_environment)
// ---------------------------------------------------------------------------

/// One admitted generation of an invocation slot.
#[derive(Debug, Clone, PartialEq)]
struct IssuedInvocation {
    generation: u64,
    event_id: String,
    invocation_id: String,
    work_request_id: String,
    authority_decision_id: String,
    operation: String,
    resource: String,
}

#[derive(Debug, Default)]
struct SlotState {
    current_generation: u64,
    invocations: Vec<IssuedInvocation>,
}

/// What happened when an execution observation reached the boundary.
#[derive(Debug, Clone, PartialEq)]
pub enum ObservationOutcome {
    /// Bound to the EXACT authorized invocation and accepted. Authority
    /// identity comes from the LEDGER record, never from the observation.
    Settled {
        invocation_id: String,
        work_request_id: String,
        /// Recorded at authorization time — a fabricated decision inside the
        /// response cannot change this fact.
        authority_decision_id: String,
        execution_status: String,
        observed_effects: Value,
    },
    /// A redelivery of an already-settled observation (same event id or
    /// idempotency key). Explicitly consequence-free: the first settlement
    /// stands, nothing is rewritten.
    DuplicateDelivery,
}

impl ObservationOutcome {
    /// True only for a first-time settlement of this observation.
    pub fn settled(&self) -> bool {
        matches!(self, Self::Settled { .. })
    }
}

/// Registry of AuthorizedInvocations this process actually emitted and
/// admitted, keyed by invocation slot `(work_request_id, operation,
/// resource)`. Re-authorization of the same slot opens the next generation
/// and supersedes all earlier ones — observations bound to superseded
/// generations are LATE and never settle.
#[derive(Debug, Default)]
pub struct InvocationLedger {
    slots: BTreeMap<String, SlotState>,
    by_invocation_id: BTreeMap<String, String>,
    settled_event_ids: BTreeSet<String>,
    settled_idempotency_keys: BTreeSet<String>,
}

fn slot_key(work_request_id: &str, operation: &str, resource: &str) -> String {
    format!("{work_request_id}\u{1f}{operation}\u{1f}{resource}")
}

impl InvocationLedger {
    /// Admit one canonical AuthorizedInvocation envelope (E5A) into the
    /// ledger. Every field is parsed and validated from the WIRE SHAPE —
    /// admission is only possible for a fully-formed, exclusively-produced,
    /// identity-bound, causally-derived envelope:
    ///
    /// 1. exclusive producer `sea_forge` (I3) — any other stamp fails closed;
    /// 2. canonical model identity, drift-checked against the local
    ///    resolution;
    /// 3. every frozen payload field present and meaningful
    ///    (work_request_id / invocation_id / authority_decision_id /
    ///    operation / resource / execution_constraints);
    /// 4. at least one `caused_by:` causal parent (ENV-I4) — an orphan
    ///    invocation cannot enter the governed ledger;
    /// 5. re-admitting the SAME envelope is idempotent (ENV-I6); a NEW
    ///    envelope for the same slot opens the next generation and supersedes
    ///    all earlier ones.
    pub fn admit_authorized_invocation(
        &mut self,
        envelope: &Value,
        local_model_sha256: &str,
    ) -> Result<u64, BoundaryError> {
        let obj = envelope
            .as_object()
            .ok_or_else(|| BoundaryError::MalformedEnvelope("envelope must be an object".into()))?;
        check_envelope_producer(envelope, AUTHORIZED_INVOCATION, "sea_forge")?;
        check_payload_model_binding(envelope, local_model_sha256)?;

        let payload = obj
            .get("payload")
            .and_then(Value::as_object)
            .ok_or_else(|| BoundaryError::MalformedEnvelope("payload must be an object".into()))?;

        let mut missing: Vec<&'static str> = Vec::new();
        let mut require_str = |key: &'static str| -> Option<String> {
            match meaningful(payload.get(key).and_then(Value::as_str)) {
                Some(v) => Some(v),
                None => {
                    missing.push(key);
                    None
                }
            }
        };
        let work_request_id = require_str("work_request_id");
        let invocation_id = require_str("invocation_id");
        let authority_decision_id = require_str("authority_decision_id");
        let operation = require_str("operation");
        let resource = require_str("resource");
        match payload.get("execution_constraints") {
            Some(c @ Value::Object(_)) => {
                let _ = c;
            }
            _ => missing.push("execution_constraints"),
        }
        if !missing.is_empty() {
            return Err(BoundaryError::OpaquePayload { missing });
        }

        // Placeholder identities cannot bind.
        for (field, value) in [
            ("work_request_id", &work_request_id),
            ("invocation_id", &invocation_id),
            ("authority_decision_id", &authority_decision_id),
        ] {
            let value = value.as_deref().unwrap_or_default();
            check_identity_string(field, value)
                .map_err(|field| BoundaryError::PlaceholderField { field })?;
        }
        let invocation_id = invocation_id.unwrap_or_default();

        // ENV-I4: derived envelopes identify their causal parents.
        let parents = causal_parents(envelope);
        if parents.is_empty() {
            return Err(BoundaryError::CausalityMissing {
                expected_parent: "<any causal parent>".into(),
                recorded: Vec::new(),
            });
        }

        let event_id = envelope
            .get("event_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();

        // Idempotent re-admission of the same envelope identity (ENV-I6).
        if let Some(slot_key) = self.by_invocation_id.get(&invocation_id) {
            if let Some(state) = self.slots.get(slot_key) {
                if let Some(existing) = state
                    .invocations
                    .iter()
                    .find(|i| i.invocation_id == invocation_id)
                {
                    if existing.event_id == event_id {
                        return Ok(existing.generation);
                    }
                }
            }
        }

        let key = slot_key(
            work_request_id.as_deref().unwrap_or_default(),
            operation.as_deref().unwrap_or_default(),
            resource.as_deref().unwrap_or_default(),
        );
        let generation;
        {
            let state = self.slots.entry(key.clone()).or_default();
            state.current_generation += 1;
            generation = state.current_generation;
            state.invocations.push(IssuedInvocation {
                generation,
                event_id: event_id.clone(),
                invocation_id: invocation_id.clone(),
                work_request_id: work_request_id.unwrap_or_default(),
                authority_decision_id: authority_decision_id.unwrap_or_default(),
                operation: operation.unwrap_or_default(),
                resource: resource.unwrap_or_default(),
            });
        }
        self.by_invocation_id.insert(invocation_id, key);
        Ok(generation)
    }

    /// Adjudicate one canonical ExecutionObservation envelope (E5B).
    ///
    /// Binding chain — every link mandatory:
    /// * exclusive producer `execution_environment` (I3);
    /// * canonical model identity, drift-checked against the local resolution;
    /// * every frozen payload field parsed and validated (work_request_id /
    ///   invocation_id / execution_status / observed_effects, plus typed
    ///   optional refs);
    /// * the cited invocation must exist in THIS ledger — observations for
    ///   invocations that were never authorized here are unknown;
    /// * the observation MUST cite that invocation's E5A event id as causal
    ///   parent (ENV-I4) — so a stale observation from a previous cycle
    ///   cannot even name its way in;
    /// * the invocation must be the CURRENT generation of its slot — late
    ///   observations from superseded generations never settle;
    /// * the correlated work_request_id must match the authorized one;
    /// * an `authority_decision_id` claimed INSIDE the response must equal the
    ///   recorded one — a fabricated decision has no authority effect, and
    ///   the settlement outcome reports only the ledger-recorded decision.
    pub fn settle_execution_observation(
        &mut self,
        observation: &Value,
        local_model_sha256: &str,
    ) -> Result<ObservationOutcome, BoundaryError> {
        let obj = observation
            .as_object()
            .ok_or_else(|| BoundaryError::MalformedEnvelope("envelope must be an object".into()))?;
        check_envelope_producer(observation, EXECUTION_OBSERVATION, "execution_environment")?;

        // Duplicate delivery is recognized before anything else: a redelivery
        // of an already-settled observation is consequence-free regardless of
        // what its payload now claims.
        let event_id = obj
            .get("event_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let idem = obj
            .get("idempotency_key")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if self.settled_event_ids.contains(&event_id)
            || (!idem.is_empty() && self.settled_idempotency_keys.contains(&idem))
        {
            return Ok(ObservationOutcome::DuplicateDelivery);
        }

        check_payload_model_binding(observation, local_model_sha256)?;

        let payload = obj
            .get("payload")
            .and_then(Value::as_object)
            .ok_or_else(|| BoundaryError::MalformedEnvelope("payload must be an object".into()))?;

        // Parse/validate EVERY projected field.
        let mut missing: Vec<&'static str> = Vec::new();
        let work_request_id =
            match meaningful(payload.get("work_request_id").and_then(Value::as_str)) {
                Some(v) => v,
                None => {
                    missing.push("work_request_id");
                    String::new()
                }
            };
        let invocation_id = match meaningful(payload.get("invocation_id").and_then(Value::as_str)) {
            Some(v) => v,
            None => {
                missing.push("invocation_id");
                String::new()
            }
        };
        let execution_status = match payload.get("execution_status").and_then(Value::as_str) {
            Some(
                s @ ("completed"
                | "spawn_failed"
                | "timed_out"
                | "sandbox_violation"
                | "suspected_sandbox_violation"),
            ) => s.to_string(),
            Some(other) => {
                return Err(BoundaryError::InvalidExecutionStatus {
                    got: other.to_string(),
                })
            }
            None => {
                missing.push("execution_status");
                String::new()
            }
        };
        let observed_effects = match payload.get("observed_effects") {
            Some(e @ (Value::Array(_) | Value::Object(_))) => e.clone(),
            Some(_) => {
                return Err(BoundaryError::OpaquePayload {
                    missing: vec!["observed_effects"],
                })
            }
            None => {
                missing.push("observed_effects");
                Value::Null
            }
        };
        // Typed optional fields: wrong shapes are refused, not coerced.
        for field in ["stdout_ref", "stderr_ref"] {
            if let Some(v) = payload.get(field) {
                if v.as_str().is_none() {
                    return Err(BoundaryError::InvalidOptionalField { field });
                }
            }
        }
        for field in ["artifact_refs", "side_effect_refs"] {
            if let Some(v) = payload.get(field) {
                if v.as_array().is_none() {
                    return Err(BoundaryError::InvalidOptionalField { field });
                }
            }
        }
        if let Some(v) = payload.get("failure_details") {
            if !matches!(v, Value::String(_) | Value::Object(_)) {
                return Err(BoundaryError::InvalidOptionalField {
                    field: "failure_details",
                });
            }
        }
        if !missing.is_empty() {
            return Err(BoundaryError::OpaquePayload { missing });
        }

        // Placeholder identities cannot settle.
        for (field, value) in [
            ("work_request_id", &work_request_id),
            ("invocation_id", &invocation_id),
        ] {
            check_identity_string(field, value)
                .map_err(|field| BoundaryError::PlaceholderField { field })?;
        }

        // Bind to the invocation ACTUALLY authorized.
        let slot_key = self
            .by_invocation_id
            .get(&invocation_id)
            .ok_or_else(|| BoundaryError::UnknownInvocation {
                invocation_id: invocation_id.clone(),
            })?
            .clone();
        let state = self.slots.get(&slot_key).expect("slot key registered");
        let issued = state
            .invocations
            .iter()
            .find(|i| i.invocation_id == invocation_id)
            .expect("registered invocation");

        // ENV-I4: the observation must cite ITS OWN invocation's E5A event.
        let parents = causal_parents(observation);
        if !parents.contains(&issued.event_id.as_str()) {
            return Err(BoundaryError::CausalityMissing {
                expected_parent: issued.event_id.clone(),
                recorded: parents.into_iter().map(str::to_string).collect(),
            });
        }

        // Generation currency: superseded generations are LATE forever.
        if issued.generation != state.current_generation {
            return Err(BoundaryError::LateObservation {
                invocation_id: issued.invocation_id.clone(),
                settled_generation: issued.generation,
                current_generation: state.current_generation,
            });
        }

        // Correlation binding.
        if issued.work_request_id != work_request_id {
            return Err(BoundaryError::CrossWiredWorkRequest {
                expected: issued.work_request_id.clone(),
                got: work_request_id,
            });
        }

        // Fabricated/self-asserted authority inside the response has NO
        // authority effect: it either matches the recorded decision exactly
        // (and is redundant) or the observation is refused. The outcome below
        // reports the RECORDED decision either way.
        if let Some(claimed) =
            meaningful(payload.get("authority_decision_id").and_then(Value::as_str))
        {
            if claimed != issued.authority_decision_id {
                return Err(BoundaryError::SelfAssertedAuthority {
                    recorded: issued.authority_decision_id.clone(),
                    claimed,
                });
            }
        }
        if let Some(fabricated) = payload
            .get("authority_decision")
            .and_then(|d| d.get("decision_id"))
            .and_then(Value::as_str)
            .and_then(|s| meaningful(Some(s)))
        {
            if fabricated != issued.authority_decision_id {
                return Err(BoundaryError::SelfAssertedAuthority {
                    recorded: issued.authority_decision_id.clone(),
                    claimed: fabricated,
                });
            }
        }

        // Settle — once per observation identity.
        self.settled_event_ids.insert(event_id);
        if !idem.is_empty() {
            self.settled_idempotency_keys.insert(idem);
        }

        Ok(ObservationOutcome::Settled {
            invocation_id: issued.invocation_id.clone(),
            work_request_id: issued.work_request_id.clone(),
            authority_decision_id: issued.authority_decision_id.clone(),
            execution_status,
            observed_effects,
        })
    }

    /// Current generation of a slot, when the slot has any admitted
    /// invocations.
    pub fn current_generation(
        &self,
        work_request_id: &str,
        operation: &str,
        resource: &str,
    ) -> Option<u64> {
        self.slots
            .get(&slot_key(work_request_id, operation, resource))
            .map(|state| state.current_generation)
    }
}
