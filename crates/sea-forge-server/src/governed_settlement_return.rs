//! Governed operational-settlement return (convergence plan task T06; frozen
//! edge E6, invariants I6/I7).
//!
//! SEA-Forge owns operational settlement (`settlement_semantics`:
//! `operational_settlement.owner = sea_forge`): the question "did the governed
//! execution satisfy its declared authority, execution, evidence, and
//! operational settlement contract?" is answered HERE, from the ledger-settled
//! [`ObservationOutcome`] of the T05 boundary compared against the settlement
//! criteria DECLARED on the governed request chain (E4) — never from process
//! exit status alone. A `completed` execution whose observed effects do not
//! attest every declared criterion settles `Rejected` (plan tooth 1: exit-zero
//! false settlement is impossible).
//!
//! The evaluated settlement is projected into one canonical v1
//! `OperationalSettlement` envelope stamped by the exclusive authoritative
//! producer `sea_forge`, carrying all seven frozen required payload fields,
//! citing its ACTUAL causal chain (the AuthorizedInvocation E5A event and the
//! ExecutionObservation E5B event), with content-addressed evidence references
//! where the substrate permits (ENV-I7). The envelope says nothing about
//! proof: SWE_SEED's proof contract remains a distinct obligation downstream
//! (I6), and nothing on this surface reaches developmental settlement or
//! capability state (I7) — those facts are not representable here.
//!
//! Pure functions over decoded envelope JSON plus real kernel types —
//! transport-free by design, mirroring the T04/T05 boundary precedents.

use sea_forge_core::types::SettlementStatus;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::governed_execution_boundary::{
    canonical_payload_json, causal_parents, check_envelope_producer, check_identity_string,
    check_payload_model_binding, idempotency_key, new_event_id, utc_now, verify_model_identity,
    ObservationOutcome, NAMESPACE, SCHEMA_VERSION,
};

/// Event type owned exclusively by `sea_forge` at edge E6.
pub const OPERATIONAL_SETTLEMENT: &str = "OperationalSettlement";

/// The frozen required payload fields of an OperationalSettlement (edge E6).
pub const REQUIRED_FIELDS: [&str; 7] = [
    "work_request_id",
    "authority_decision_id",
    "execution_status",
    "observed_effects",
    "operational_settlement_status",
    "evidence_refs",
    "domain_model_ref",
];

/// The governed execution-status vocabulary carried over from edge E5B.
const EXECUTION_STATUSES: [&str; 5] = [
    "completed",
    "spawn_failed",
    "timed_out",
    "sandbox_violation",
    "suspected_sandbox_violation",
];

/// Wire forms of [`SettlementStatus`] for the frozen
/// `operational_settlement_status` field. Escalated cannot occur behind the
/// T05 Allow gate, so it is not part of this edge's wire vocabulary.
const SETTLEMENT_STATUSES: [&str; 2] = ["accepted", "rejected"];

fn sha256_hex(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}

/// Why an operational settlement could not be evaluated, emitted, or accepted
/// back as canonical wire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettlementReturnError {
    /// Evaluation requires a ledger-SETTLED observation; a duplicate delivery
    /// settles nothing and therefore evaluates nothing.
    NothingSettled,
    /// No declared criteria to evaluate against, or a blank criterion among
    /// them — vacuous acceptance is unrepresentable.
    OpaqueCriteria,
    /// The governed correlation or a required identity field is missing,
    /// blank, or a literal placeholder.
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
    /// `operational_settlement_status` outside the governed wire vocabulary —
    /// which also refuses any proof-shaped verdict smuggled into the field.
    InvalidSettlementStatus { got: String },
    /// An optional-but-typed field arrived with the wrong shape.
    InvalidOptionalField { field: &'static str },
    /// An evidence reference is empty, whitespace-bearing, or (for
    /// content-addressed refs) not a SHA-256 digest.
    InvalidEvidenceRef { got: String },
    /// The payload namespace is missing or is not the canonical namespace.
    NamespaceMismatch { expected: String, got: String },
    /// `domain_model_ref.model_hash` disagrees with the declared identity.
    RefIdentityMismatch { declared: String, ref_hash: String },
    /// The derivation does not record the required causal chain (ENV-I4):
    /// both the AuthorizedInvocation and ExecutionObservation event ids are
    /// mandatory parents of an operational settlement.
    CausalityMissing {
        expected: Vec<String>,
        recorded: Vec<String>,
    },
}

impl std::fmt::Display for SettlementReturnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NothingSettled => write!(
                f,
                "nothing settled: a duplicate delivery evaluates and projects nothing"
            ),
            Self::OpaqueCriteria => {
                write!(f, "declared settlement criteria absent or blank")
            }
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
                "opaque OperationalSettlement payload rejected — missing/invalid fields: {}",
                missing.join(", ")
            ),
            Self::InvalidExecutionStatus { got } => {
                write!(f, "execution_status outside the governed vocabulary: {got}")
            }
            Self::InvalidSettlementStatus { got } => write!(
                f,
                "operational_settlement_status outside the governed vocabulary: {got}"
            ),
            Self::InvalidOptionalField { field } => {
                write!(f, "optional field {field} has the wrong shape")
            }
            Self::InvalidEvidenceRef { got } => {
                write!(f, "malformed evidence reference: {got:?}")
            }
            Self::NamespaceMismatch { expected, got } => write!(
                f,
                "payload namespace must be {expected:?}, got {got:?}"
            ),
            Self::RefIdentityMismatch {
                declared,
                ref_hash,
            } => write!(
                f,
                "domain_model_ref names {ref_hash} but the envelope declares {declared}"
            ),
            Self::CausalityMissing {
                expected,
                recorded,
            } => write!(
                f,
                "operational settlement does not cite its actual invocation chain: expected {expected:?}, recorded {recorded:?}"
            ),
        }
    }
}

impl std::error::Error for SettlementReturnError {}

/// The shared family gates (`check_envelope_producer`,
/// `check_payload_model_binding`) report in T05's `BoundaryError` vocabulary;
/// this edge re-expresses the overlapping variants as its own typed error.
/// Variants that only arise inside the invocation ledger are unreachable from
/// those gates and fail closed as malformed rather than being guessed at.
impl From<crate::governed_execution_boundary::BoundaryError> for SettlementReturnError {
    fn from(e: crate::governed_execution_boundary::BoundaryError) -> Self {
        use crate::governed_execution_boundary::BoundaryError;
        match e {
            BoundaryError::PlaceholderField { field } => Self::PlaceholderField { field },
            BoundaryError::PlaceholderIdentity { got } => Self::PlaceholderIdentity { got },
            BoundaryError::DomainDrift { declared, local } => Self::DomainDrift { declared, local },
            BoundaryError::MalformedEnvelope(m) => Self::MalformedEnvelope(m),
            BoundaryError::WrongEventType { expected, got } => {
                Self::WrongEventType { expected, got }
            }
            BoundaryError::UnknownAgent { got } => Self::UnknownAgent { got },
            BoundaryError::NotAuthoritative { authoritative, got } => {
                Self::NotAuthoritative { authoritative, got }
            }
            BoundaryError::OpaquePayload { missing } => Self::OpaquePayload { missing },
            BoundaryError::InvalidExecutionStatus { got } => Self::InvalidExecutionStatus { got },
            BoundaryError::InvalidOptionalField { field } => Self::InvalidOptionalField { field },
            BoundaryError::CausalityMissing {
                expected_parent,
                recorded,
            } => Self::CausalityMissing {
                expected: vec![expected_parent],
                recorded,
            },
            other => Self::MalformedEnvelope(format!("unreachable boundary condition: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Evaluation — observed effects vs DECLARED criteria (never exit status)
// ---------------------------------------------------------------------------

/// The outcome of comparing observed effects against the declared settlement
/// criteria. Status reuses the kernel's own settlement vocabulary so this edge
/// speaks the same language as every other settlement site.
#[derive(Debug, Clone, PartialEq)]
pub struct SettlementEvaluation {
    pub status: SettlementStatus,
    /// Durable why-strings, same spirit as `sea_forge_settlement::settle`.
    pub basis: Vec<String>,
    pub satisfied_criteria: Vec<String>,
    pub unsatisfied_criteria: Vec<String>,
}

/// Evaluate one ledger-settled execution observation against the settlement
/// criteria DECLARED on the governed request chain (E4
/// `settlement_criteria`). Composition, not reimplementation: per-criterion
/// satisfaction is decided by the existing settlement-machinery evaluator
/// (`sea_forge_settlement::evaluate_agent_output`) over the serialized
/// observed effects, and non-completed statuses map onto `settle()`'s own
/// rejection basis vocabulary.
///
/// - A `duplicate delivery` never evaluates: nothing settled, nothing exists.
/// - Any non-`completed` status rejects outright (spawn failure, timeout,
///   jail violation, suspected jail violation) — mirroring `settle()`.
/// - `completed` (the exit-zero analogue) contributes NOTHING on its own:
///   every declared criterion must be attested by the observed effects, so an
///   exit-zero run that violated its declared contract settles [`SettlementStatus::Rejected`].
pub fn evaluate_operational_settlement(
    outcome: &ObservationOutcome,
    declared_criteria: &[String],
) -> Result<SettlementEvaluation, SettlementReturnError> {
    let (execution_status, observed_effects) = match outcome {
        ObservationOutcome::Settled {
            execution_status,
            observed_effects,
            ..
        } => (execution_status.as_str(), observed_effects),
        ObservationOutcome::DuplicateDelivery => return Err(SettlementReturnError::NothingSettled),
    };

    // Vacuous acceptance is unrepresentable: the governed chain always
    // declares at least one meaningful criterion (T04 ingress enforces it),
    // and this evaluator refuses to proceed without one.
    if declared_criteria.is_empty() || declared_criteria.iter().any(|c| c.trim().is_empty()) {
        return Err(SettlementReturnError::OpaqueCriteria);
    }

    let mut basis = Vec::new();
    if execution_status != "completed" {
        // Same durable bases the kernel's own settle() records for these
        // outcomes — this edge never invents a second truth vocabulary.
        basis.push(
            match execution_status {
                "spawn_failed" => "spawn_failed",
                "timed_out" => "timed_out",
                "sandbox_violation" => "jail_violation",
                _ => "suspected_jail_violation",
            }
            .to_string(),
        );
        return Ok(SettlementEvaluation {
            status: SettlementStatus::Rejected,
            basis,
            satisfied_criteria: Vec::new(),
            unsatisfied_criteria: declared_criteria
                .iter()
                .map(|c| c.trim().to_string())
                .collect(),
        });
    }

    let effects_text = canonical_payload_json(observed_effects);
    let mut satisfied = Vec::new();
    let mut unsatisfied = Vec::new();
    for criterion in declared_criteria {
        let criterion = criterion.trim();
        match sea_forge_settlement::evaluate_agent_output(&effects_text, Some(criterion)) {
            Some(true) => {
                satisfied.push(criterion.to_string());
                basis.push(format!("criterion_satisfied:{criterion}"));
            }
            _ => {
                unsatisfied.push(criterion.to_string());
                basis.push(format!("criterion_unsatisfied:{criterion}"));
            }
        }
    }

    basis.push("status_completed".into());
    Ok(SettlementEvaluation {
        status: if unsatisfied.is_empty() {
            SettlementStatus::Accepted
        } else {
            SettlementStatus::Rejected
        },
        basis,
        satisfied_criteria: satisfied,
        unsatisfied_criteria: unsatisfied,
    })
}

// ---------------------------------------------------------------------------
// Projection — the canonical v1 OperationalSettlement envelope (producer:
// sea_forge)
// ---------------------------------------------------------------------------

/// Optional frozen fields passed through verbatim when the caller holds them.
/// Typed: wrong shapes are refused at validation, never coerced.
#[derive(Debug, Clone, Default)]
pub struct SettlementOptionalFields<'a> {
    pub case_id: Option<&'a str>,
    pub run_id: Option<&'a str>,
    /// Must be a JSON array when present.
    pub artifact_refs: Option<&'a Value>,
    pub transcript_ref: Option<&'a str>,
    /// String or object when present.
    pub failure_reason: Option<&'a Value>,
}

/// One canonical OperationalSettlement projected at the settlement-return
/// boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct OperationalSettlementReturn {
    /// The full canonical v1 envelope — the only thing that crosses back to
    /// SWE_SEED.
    pub envelope: Value,
    pub event_id: String,
    pub work_request_id: String,
    /// The LEDGER-recorded authority decision of the authorized invocation.
    pub authority_decision_id: String,
    pub invocation_id: String,
    pub evaluation: SettlementEvaluation,
}

/// Validate one evidence reference: `<kind>:<value>` with no whitespace; a
/// `sha256:`-prefixed reference MUST carry a full lowercase digest (ENV-I7:
/// content-addressed where the substrate permits).
fn validate_evidence_ref(r: &str) -> Result<(), SettlementReturnError> {
    let trimmed = r.trim();
    if trimmed.is_empty() || trimmed.contains(char::is_whitespace) || !trimmed.contains(':') {
        return Err(SettlementReturnError::InvalidEvidenceRef { got: r.to_string() });
    }
    if let Some(digest) = trimmed.strip_prefix("sha256:") {
        if !crate::governed_execution_boundary::is_sha256_hex(digest) {
            return Err(SettlementReturnError::InvalidEvidenceRef { got: r.to_string() });
        }
    }
    Ok(())
}

/// Full canonical-wire validation of an OperationalSettlement envelope — the
/// exact battery the projection self-checks against, and the battery the
/// cross-repo consumer independently mirrors. Every required field is parsed
/// AND validated from the wire shape (no T04-D1 gaps); the payload namespace
/// is checked (no T04-D3 gap).
pub fn validate_operational_settlement_wire(
    envelope: &Value,
    local_model_sha256: &str,
) -> Result<(), SettlementReturnError> {
    let obj = envelope.as_object().ok_or_else(|| {
        SettlementReturnError::MalformedEnvelope("envelope must be an object".into())
    })?;
    if obj.get("schema_version").and_then(Value::as_str) != Some(SCHEMA_VERSION) {
        return Err(SettlementReturnError::MalformedEnvelope(
            "schema_version must be \"v1\"".into(),
        ));
    }
    check_envelope_producer(envelope, OPERATIONAL_SETTLEMENT, "sea_forge")?;
    check_payload_model_binding(envelope, local_model_sha256)?;

    let payload = obj
        .get("payload")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            SettlementReturnError::MalformedEnvelope("payload must be an object".into())
        })?;

    // Namespace gate (T04-D3 closed here): the loop namespace rides inside
    // the payload in the v1 family and must be exact.
    let namespace = payload
        .get("namespace")
        .and_then(Value::as_str)
        .unwrap_or("");
    if namespace != NAMESPACE {
        return Err(SettlementReturnError::NamespaceMismatch {
            expected: NAMESPACE.into(),
            got: namespace.to_string(),
        });
    }

    // Presence battery first: report EVERYTHING that is missing.
    let mut missing: Vec<&'static str> = Vec::new();
    for field in REQUIRED_FIELDS {
        let present = match payload.get(field) {
            None | Some(Value::Null) => false,
            Some(Value::String(s)) => !s.trim().is_empty(),
            Some(_) => true,
        };
        if !present {
            missing.push(field);
        }
    }
    if !missing.is_empty() {
        return Err(SettlementReturnError::OpaquePayload { missing });
    }

    let work_request_id = payload
        .get("work_request_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let authority_decision_id = payload
        .get("authority_decision_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    for (field, value) in [
        ("work_request_id", work_request_id),
        ("authority_decision_id", authority_decision_id),
    ] {
        check_identity_string(field, value)
            .map_err(|field| SettlementReturnError::PlaceholderField { field })?;
    }

    match payload.get("execution_status").and_then(Value::as_str) {
        Some(s) if EXECUTION_STATUSES.contains(&s) => {}
        Some(other) => {
            return Err(SettlementReturnError::InvalidExecutionStatus {
                got: other.to_string(),
            })
        }
        None => unreachable!("presence battery"),
    }
    match payload
        .get("operational_settlement_status")
        .and_then(Value::as_str)
    {
        Some(s) if SETTLEMENT_STATUSES.contains(&s) => {}
        Some(other) => {
            return Err(SettlementReturnError::InvalidSettlementStatus {
                got: other.to_string(),
            })
        }
        None => unreachable!("presence battery"),
    }
    match payload.get("observed_effects") {
        Some(Value::Array(_) | Value::Object(_)) => {}
        _ => {
            return Err(SettlementReturnError::OpaquePayload {
                missing: vec!["observed_effects"],
            })
        }
    }

    let declared = payload
        .get("domain_model_hash")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let evidence_refs = payload
        .get("evidence_refs")
        .and_then(Value::as_array)
        .expect("presence battery");
    if evidence_refs.is_empty() {
        return Err(SettlementReturnError::OpaquePayload {
            missing: vec!["evidence_refs"],
        });
    }
    for r in evidence_refs {
        match r.as_str() {
            Some(s) => validate_evidence_ref(s)?,
            None => {
                return Err(SettlementReturnError::InvalidEvidenceRef {
                    got: "<non-string ref>".into(),
                })
            }
        }
    }

    let model_ref = payload
        .get("domain_model_ref")
        .and_then(Value::as_object)
        .ok_or_else(|| SettlementReturnError::OpaquePayload {
            missing: vec!["domain_model_ref"],
        })?;
    if model_ref.get("namespace").and_then(Value::as_str) != Some(NAMESPACE) {
        return Err(SettlementReturnError::NamespaceMismatch {
            expected: NAMESPACE.into(),
            got: model_ref
                .get("namespace")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        });
    }
    let ref_hash = model_ref
        .get("model_hash")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if ref_hash != declared {
        return Err(SettlementReturnError::RefIdentityMismatch {
            declared: declared.to_string(),
            ref_hash: ref_hash.to_string(),
        });
    }

    // Typed optional fields: refused, not coerced.
    for field in ["case_id", "run_id", "transcript_ref"] {
        if let Some(v) = payload.get(field) {
            if v.as_str().is_none() {
                return Err(SettlementReturnError::InvalidOptionalField { field });
            }
        }
    }
    if let Some(v) = payload.get("artifact_refs") {
        if v.as_array().is_none() {
            return Err(SettlementReturnError::InvalidOptionalField {
                field: "artifact_refs",
            });
        }
    }
    if let Some(v) = payload.get("failure_reason") {
        if !matches!(v, Value::String(_) | Value::Object(_)) {
            return Err(SettlementReturnError::InvalidOptionalField {
                field: "failure_reason",
            });
        }
    }

    // ENV-I4: a derived settlement identifies its causal parents.
    if causal_parents(envelope).is_empty() {
        return Err(SettlementReturnError::CausalityMissing {
            expected: vec![
                "<authorized_invocation event id>".into(),
                "<execution_observation event id>".into(),
            ],
            recorded: Vec::new(),
        });
    }
    Ok(())
}

/// Evaluate a ledger-settled observation against the declared criteria and
/// project the result into the canonical v1 OperationalSettlement envelope.
///
/// Gates, in order:
/// 1. **Settled input only:** the observation outcome must be a first-time
///    ledger settlement; a duplicate delivery projects nothing.
/// 2. **Evaluation:** [`evaluate_operational_settlement`] compares the
///    settled observed effects against the DECLARED criteria — exit-zero
///    (`completed`) alone can never produce [`SettlementStatus::Accepted`].
/// 3. **Meaningful correlation + chain identities:** work request,
///    AuthorizedInvocation (E5A) and ExecutionObservation (E5B) event ids are
///    mandatory; the authority decision id comes ONLY from the ledger record.
/// 4. **Canonical domain identity:** structural validity, placeholder/fallback
///    refusal, and drift against the locally resolved model.
/// 5. **Self-validation:** the built envelope must pass the full
///    [`validate_operational_settlement_wire`] battery — all seven frozen
///    required fields present AND validated before anything is returned.
///
/// The envelope's idempotency key is content-derived from the payload alone,
/// so re-projecting the SAME settled facts yields the SAME stable identity
/// regardless of fresh event ids/timestamps (ENV-I6), while any mutation of
/// the projected facts yields a different identity — which the receiving side
/// treats as a conflicting re-settlement of the same invocation chain, never
/// as a fresh fact.
#[allow(clippy::too_many_arguments)]
pub fn emit_operational_settlement(
    settled: &ObservationOutcome,
    declared_criteria: &[String],
    declared_model_sha256: &str,
    local_model_sha256: &str,
    authorized_invocation_event_id: &str,
    execution_observation_event_id: &str,
    optionals: SettlementOptionalFields<'_>,
) -> Result<OperationalSettlementReturn, SettlementReturnError> {
    let evaluation = evaluate_operational_settlement(settled, declared_criteria)?;

    let (invocation_id, work_request_id, authority_decision_id, execution_status, observed_effects) =
        match settled {
            ObservationOutcome::Settled {
                invocation_id,
                work_request_id,
                authority_decision_id,
                execution_status,
                observed_effects,
            } => (
                invocation_id.clone(),
                work_request_id.clone(),
                authority_decision_id.clone(),
                execution_status.clone(),
                observed_effects.clone(),
            ),
            ObservationOutcome::DuplicateDelivery => {
                return Err(SettlementReturnError::NothingSettled)
            }
        };

    for (field, value) in [
        ("invocation_id", invocation_id.as_str()),
        ("work_request_id", work_request_id.as_str()),
        ("authority_decision_id", authority_decision_id.as_str()),
        (
            "authorized_invocation_event_id",
            authorized_invocation_event_id,
        ),
        (
            "execution_observation_event_id",
            execution_observation_event_id,
        ),
    ] {
        check_identity_string(field, value)
            .map_err(|field| SettlementReturnError::PlaceholderField { field })?;
    }

    verify_model_identity(declared_model_sha256)
        .map_err(|got| SettlementReturnError::PlaceholderIdentity { got })?;
    if declared_model_sha256 != local_model_sha256.trim() {
        return Err(SettlementReturnError::DomainDrift {
            declared: declared_model_sha256.to_string(),
            local: local_model_sha256.trim().to_string(),
        });
    }

    let wire_status = match evaluation.status {
        SettlementStatus::Accepted => "accepted",
        SettlementStatus::Rejected => "rejected",
        SettlementStatus::Escalated => {
            return Err(SettlementReturnError::InvalidSettlementStatus {
                got: "escalated".into(),
            })
        }
    };

    // Evidence: content-addressed digest of exactly the settled effects
    // (ENV-I7), plus precise pointers into the immutable upstream chain.
    let effects_digest = sha256_hex(canonical_payload_json(&observed_effects).as_bytes());
    let evidence_refs = Value::Array(vec![
        Value::String(format!("sha256:{effects_digest}")),
        Value::String(format!(
            "authorized_invocation:{authorized_invocation_event_id}"
        )),
        Value::String(format!(
            "execution_observation:{execution_observation_event_id}"
        )),
    ]);

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
    payload.insert(
        "authority_decision_id".into(),
        Value::String(authority_decision_id.clone()),
    );
    payload.insert("invocation_id".into(), Value::String(invocation_id.clone()));
    payload.insert(
        "execution_status".into(),
        Value::String(execution_status.clone()),
    );
    payload.insert("observed_effects".into(), observed_effects.clone());
    payload.insert(
        "operational_settlement_status".into(),
        Value::String(wire_status.to_string()),
    );
    payload.insert("evidence_refs".into(), evidence_refs);
    payload.insert(
        "domain_model_ref".into(),
        serde_json::json!({"namespace": NAMESPACE, "model_hash": declared_model_sha256}),
    );
    if let Some(case_id) = optionals.case_id.map(str::trim).filter(|c| !c.is_empty()) {
        payload.insert("case_id".into(), Value::String(case_id.to_string()));
    }
    if let Some(run_id) = optionals.run_id.map(str::trim).filter(|r| !r.is_empty()) {
        payload.insert("run_id".into(), Value::String(run_id.to_string()));
    }
    if let Some(artifact_refs) = optionals.artifact_refs {
        payload.insert("artifact_refs".into(), artifact_refs.clone());
    }
    if let Some(transcript_ref) = optionals
        .transcript_ref
        .map(str::trim)
        .filter(|t| !t.is_empty())
    {
        payload.insert(
            "transcript_ref".into(),
            Value::String(transcript_ref.to_string()),
        );
    }
    if let Some(failure_reason) = optionals.failure_reason {
        payload.insert("failure_reason".into(), failure_reason.clone());
    }
    let payload = Value::Object(payload);

    let event_id = new_event_id();
    let envelope = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "event_id": event_id,
        "source_agent": "sea-forge",
        "event_type": OPERATIONAL_SETTLEMENT,
        "occurred_at": utc_now(),
        "idempotency_key": idempotency_key(OPERATIONAL_SETTLEMENT, &payload),
        "payload": payload,
        "provenance": {
            "origin": "sea-forge",
            "chain": [
                format!("domain_model_hash:{declared_model_sha256}"),
                format!("caused_by:{authorized_invocation_event_id}"),
                format!("caused_by:{execution_observation_event_id}"),
            ],
        },
    });

    // Self-validation: the projection is only returned when it survives the
    // full canonical wire battery.
    validate_operational_settlement_wire(&envelope, local_model_sha256)?;

    Ok(OperationalSettlementReturn {
        envelope,
        event_id,
        work_request_id,
        authority_decision_id,
        invocation_id,
        evaluation,
    })
}
