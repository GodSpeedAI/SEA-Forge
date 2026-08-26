//! GovernedWorkRequest ingress (convergence plan task T04; frozen edge E4).
//!
//! SWE_SEED is the exclusive authoritative producer of GovernedWorkRequest;
//! SEA-Forge is its consumer. This module is the acceptance gate every
//! governed work submission must pass BEFORE any authority evaluation or
//! execution can consider it:
//!
//! 1. envelope shape + exclusive producer authority (`swe_seed`, invariant
//!    I3) — a forged stamp from any other canonical component fails closed;
//! 2. canonical domain identity (ENV-I1/I2, I1): the declared model hash must
//!    be a real SHA-256 digest, never a known placeholder/fallback pseudo-hash,
//!    and must equal the locally resolved canonical model — so the model
//!    SEA-Forge evaluates semantic authority against IS the model the request
//!    declares (frozen falsifier: a valid-but-different DomainForge model);
//! 3. semantic work intent (frozen falsifier: an opaque command-only request):
//!    all eight required E4 payload fields must carry semantics;
//! 4. context binding (frozen falsifier: cross-wired packets): the referenced
//!    ContextPacketCreated envelope must be genuinely produced by
//!    `context_kernel`, correlate to the SAME work_request_id, match the
//!    declared reference, and be recorded as a causal parent (ENV-I4);
//! 5. distinct obligations: the SWE_SEED proof contract remains a separate
//!    fact from the operational settlement criteria (invariant I6 begins at
//!    this boundary).
//!
//! Pure function over decoded envelope JSON plus the locally resolved model
//! digest — transport-free by design, mirroring the SWE_SEED-side
//! `accept_work_requested` ingress this boundary composes with upstream.

use serde_json::Value;
use sha2::{Digest, Sha256};

/// Wire schema version of the canonical envelope family.
const SCHEMA_VERSION: &str = "v1";

/// The event type owned exclusively by `swe_seed` at this edge.
pub const EVENT_TYPE: &str = "GovernedWorkRequest";

/// The standalone-fallback pseudo-hash input (SWE_SEED federation parity):
/// `sha256("agentic_capability_loop")`. A request declaring it has no real
/// model behind it.
fn fallback_pseudo_hash() -> String {
    let mut h = Sha256::new();
    h.update(b"agentic_capability_loop");
    format!("{:x}", h.finalize())
}

/// Map a wire `source_agent` onto its canonical component id. Same vocabulary
/// and fail-closed posture as the SWE_SEED producer registry: hyphenated
/// legacy stamps alias their underscored preregistration ids; anything unknown
/// is refused rather than coerced.
fn canonical_agent(source_agent: &str) -> Option<&'static str> {
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

/// Why a submission cannot enter SEA-Forge as a governed work request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GovernedIngressError {
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
    /// Declared model hash is malformed or a known placeholder/fallback
    /// pseudo-identity (ENV-I2) — nothing manufactures identity here.
    PlaceholderIdentity { got: String },
    /// Declared model hash differs from the locally resolved canonical model
    /// (frozen falsifier: SEA-Forge must evaluate the DECLARED model).
    DomainDrift { declared: String, local: String },
    /// Required semantic-intent fields absent or empty: an opaque command
    /// invocation, not governable work.
    OpaqueCommand { missing: Vec<&'static str> },
    /// The proof contract collapsed into (or vanished beside) the operational
    /// settlement criteria — they are distinct frozen facts.
    CollapsedObligations,
    /// The context packet belongs to a different work request.
    CrossWiredContextPacket { expected: String, got: String },
    /// `context_packet_ref` names something other than the supplied packet.
    ContextRefMismatch {
        declared_ref: String,
        packet_id: String,
    },
    /// The derivation does not record the packet as its causal parent.
    CausalityMissing {
        expected_parent: String,
        recorded: Vec<String>,
    },
}

impl std::fmt::Display for GovernedIngressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
                "{EVENT_TYPE} may only be produced by '{authoritative}', forge attempted by '{got}'"
            ),
            Self::PlaceholderIdentity { got } => write!(
                f,
                "fallback/placeholder pseudo-hash rejected as domain identity: {got}"
            ),
            Self::DomainDrift { declared, local } => write!(
                f,
                "domain model drift: request declares {declared}, local canonical model is {local}"
            ),
            Self::OpaqueCommand { missing } => write!(
                f,
                "opaque command rejected — missing semantic intent fields: {}",
                missing.join(", ")
            ),
            Self::CollapsedObligations => write!(
                f,
                "proof_contract and settlement_criteria must remain distinct facts"
            ),
            Self::CrossWiredContextPacket { expected, got } => write!(
                f,
                "cross-wired context packet: work_request_id expected {expected:?}, packet carries {got:?}"
            ),
            Self::ContextRefMismatch {
                declared_ref,
                packet_id,
            } => write!(
                f,
                "context_packet_ref {declared_ref:?} does not name the supplied packet {packet_id:?}"
            ),
            Self::CausalityMissing {
                expected_parent,
                recorded,
            } => write!(
                f,
                "request does not cite its context packet as causal parent: expected {expected_parent}, recorded {recorded:?}"
            ),
        }
    }
}

impl std::error::Error for GovernedIngressError {}

/// The semantically governed work intent extracted by the gate.
#[derive(Debug, Clone, PartialEq)]
pub struct GovernedWorkIntent {
    pub work_request_id: String,
    pub affordance_id: String,
    pub actor_id: String,
    pub actor_role: Option<String>,
    pub intent: String,
    pub context_packet_ref: String,
    /// The SWE_SEED-owned proof obligation — deliberately kept as its own
    /// value alongside (never merged into) the settlement criteria.
    pub proof_contract: Value,
    pub settlement_criteria: Vec<String>,
}

fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Canonical identity gate for one declared model hash: structural validity,
/// then refusal of the known pseudo-identities (the standalone fallback
/// constant and the all-zero digest).
fn verify_declared_identity(declared: &str) -> Result<(), GovernedIngressError> {
    let placeholders = [fallback_pseudo_hash(), "0".repeat(64)];
    if !is_sha256_hex(declared) || placeholders.iter().any(|p| p == declared) {
        return Err(GovernedIngressError::PlaceholderIdentity {
            got: declared.to_string(),
        });
    }
    Ok(())
}

fn check_model_binding(
    envelope: &Value,
    label: &str,
    local_model_sha256: &str,
) -> Result<(), GovernedIngressError> {
    let declared = envelope
        .get("payload")
        .and_then(|p| p.get("domain_model_hash"))
        .and_then(Value::as_str)
        .ok_or_else(|| GovernedIngressError::PlaceholderIdentity {
            got: format!("<missing {label} domain_model_hash>"),
        })?;
    verify_declared_identity(declared)?;
    if declared != local_model_sha256 {
        return Err(GovernedIngressError::DomainDrift {
            declared: declared.to_string(),
            local: local_model_sha256.to_string(),
        });
    }
    Ok(())
}

fn causal_parents(envelope: &Value) -> Vec<&str> {
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

fn check_producer(
    envelope: &Value,
    expected_event: &'static str,
    authoritative: &'static str,
) -> Result<(), GovernedIngressError> {
    if envelope.get("schema_version").and_then(Value::as_str) != Some(SCHEMA_VERSION) {
        return Err(GovernedIngressError::MalformedEnvelope(
            "schema_version must be \"v1\"".into(),
        ));
    }
    match envelope.get("event_type").and_then(Value::as_str) {
        Some(got) if got == expected_event => {}
        Some(got) => {
            return Err(GovernedIngressError::WrongEventType {
                expected: expected_event,
                got: got.to_string(),
            })
        }
        None => {
            return Err(GovernedIngressError::MalformedEnvelope(
                "event_type missing".into(),
            ))
        }
    }
    let stamp = envelope
        .get("source_agent")
        .and_then(Value::as_str)
        .ok_or_else(|| GovernedIngressError::UnknownAgent {
            got: "<missing>".into(),
        })?;
    match canonical_agent(stamp) {
        None => Err(GovernedIngressError::UnknownAgent {
            got: stamp.to_string(),
        }),
        Some(canonical) if canonical == authoritative => Ok(()),
        Some(_) => Err(GovernedIngressError::NotAuthoritative {
            authoritative,
            got: stamp.to_string(),
        }),
    }
}

/// Accept one governed work submission at the SEA-Forge boundary.
///
/// `request` and `context_packet` are the decoded canonical envelopes (E4 and
/// its referenced E3 parent); `local_model_sha256` is SEA-Forge's own
/// resolution of the canonical DomainForge model identity (same manifest
/// resolution chain the producer used). Returns the parsed intent only after
/// every gate above passes; any failure means NO governed request exists.
pub fn accept_governed_work_request(
    request: &Value,
    context_packet: &Value,
    local_model_sha256: &str,
) -> Result<GovernedWorkIntent, GovernedIngressError> {
    // 1. Envelope shape + exclusive producer authority.
    let req = request.as_object().ok_or_else(|| {
        GovernedIngressError::MalformedEnvelope("envelope must be an object".into())
    })?;
    check_producer(request, EVENT_TYPE, "swe_seed")?;

    // 2. Canonical domain identity bound to OUR local resolution.
    check_model_binding(request, "request", local_model_sha256)?;

    let payload = req
        .get("payload")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            GovernedIngressError::MalformedEnvelope("payload must be an object".into())
        })?;

    // 3. Semantic work intent: an argv/command-only request fails here with
    //    the complete list of what made it ungovernable.
    let mut missing: Vec<&'static str> = Vec::new();
    let mut require_str = |key: &'static str| -> Option<String> {
        match payload.get(key).and_then(Value::as_str).map(str::trim) {
            Some(s) if !s.is_empty() => Some(s.to_string()),
            _ => {
                missing.push(key);
                None
            }
        }
    };
    let work_request_id = require_str("work_request_id");
    let affordance_id = require_str("affordance_id");
    let intent = require_str("intent");
    let context_packet_ref = require_str("context_packet_ref");

    let actor_role;
    match payload.get("actor") {
        Some(actor) => match actor.get("actor_id").and_then(Value::as_str).map(str::trim) {
            Some(id) if !id.is_empty() => {
                actor_role = actor
                    .get("role")
                    .and_then(Value::as_str)
                    .filter(|r| !r.is_empty())
                    .map(str::to_string);
            }
            _ => {
                missing.push("actor.actor_id");
                actor_role = None;
            }
        },
        None => {
            missing.push("actor");
            actor_role = None;
        }
    }

    let settlement_criteria: Option<Vec<String>> = match payload.get("settlement_criteria") {
        Some(Value::Array(items)) => {
            let criteria: Vec<String> = items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            if criteria.is_empty() {
                missing.push("settlement_criteria");
                None
            } else {
                Some(criteria)
            }
        }
        _ => {
            missing.push("settlement_criteria");
            None
        }
    };

    // 4. Distinct obligations: whatever occupies the proof_contract slot may
    //    never BE the operational settlement criteria.
    let criteria_json = settlement_criteria
        .as_ref()
        .map(|c| Value::Array(c.iter().cloned().map(Value::String).collect()));
    if let (Some(pc), Some(cj)) = (payload.get("proof_contract"), criteria_json.as_ref()) {
        if pc == cj {
            return Err(GovernedIngressError::CollapsedObligations);
        }
    }

    let proof_contract = match payload.get("proof_contract") {
        Some(pc @ Value::Object(_))
            if pc
                .get("criterion")
                .and_then(Value::as_str)
                .map(str::trim)
                .is_some_and(|c| !c.is_empty()) =>
        {
            Some(pc.clone())
        }
        Some(_) => {
            // Present but without a proof obligation inside it.
            missing.push("proof_contract.criterion");
            None
        }
        None => {
            missing.push("proof_contract");
            None
        }
    };

    if !missing.is_empty() {
        return Err(GovernedIngressError::OpaqueCommand { missing });
    }

    // Unwrap safely: every Option is Some once the missing-battery passed.
    let work_request_id = work_request_id.unwrap_or_default();
    let affordance_id = affordance_id.unwrap_or_default();
    let intent = intent.unwrap_or_default();
    let context_packet_ref = context_packet_ref.unwrap_or_default();
    let settlement_criteria = settlement_criteria.unwrap_or_default();
    let proof_value = proof_contract.unwrap_or(Value::Null);

    // 5. Context binding: genuinely CK-produced, same work request, matching
    //    ref, real identity, recorded causality.
    check_producer(context_packet, "ContextPacketCreated", "context_kernel")?;
    check_model_binding(context_packet, "packet", local_model_sha256)?;
    let packet_wr = context_packet
        .get("payload")
        .and_then(|p| p.get("work_request_id"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    if packet_wr != work_request_id {
        return Err(GovernedIngressError::CrossWiredContextPacket {
            expected: work_request_id.clone(),
            got: packet_wr.to_string(),
        });
    }
    let packet_id = context_packet
        .get("payload")
        .and_then(|p| p.get("context_packet_id"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            context_packet
                .get("event_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        });
    if context_packet_ref != packet_id {
        return Err(GovernedIngressError::ContextRefMismatch {
            declared_ref: context_packet_ref.clone(),
            packet_id: packet_id.clone(),
        });
    }
    let parents = causal_parents(request);
    let packet_event_id = context_packet
        .get("event_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !parents.contains(&packet_event_id) {
        return Err(GovernedIngressError::CausalityMissing {
            expected_parent: packet_event_id.to_string(),
            recorded: parents.into_iter().map(str::to_string).collect(),
        });
    }

    Ok(GovernedWorkIntent {
        work_request_id,
        affordance_id,
        actor_id: payload["actor"]["actor_id"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        actor_role,
        intent,
        context_packet_ref,
        proof_contract: proof_value,
        settlement_criteria,
    })
}
