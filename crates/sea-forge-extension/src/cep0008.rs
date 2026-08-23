//! CEP-0008 v1 flat profile adapter (`spec/CEP-0008-semantic-envelope.md`).
//!
//! Projects SEA Forge's native `SemanticEnvelope` into the CEP-0008 v1 flat
//! wire profile. The native envelope is preserved unchanged; this module is a
//! pure, deterministic conversion with no filesystem, ledger, network, or
//! clock side effects.
//!
//! The output claims conformance to the flat structural profile plus **partial**
//! CEP semantic conformance. CEP-0008 §52 requires concepts that the flat wire
//! profile carries only inside opaque metadata; this adapter does not claim
//! full CEP-0008 conformance.

use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::SemanticEnvelope;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// ── Output structures ──

/// CEP-0008 §15 Envelope Provenance Record.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Cep0008Provenance {
    /// Maps to CEP-0008 provenance_id / derived_from.
    pub origin: String,
    /// Maps to CEP-0008 source_envelope_refs / transformation_refs.
    /// Serialized as empty: SEA Forge authority/evidence IDs are not
    /// source-envelope lineage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chain: Vec<String>,
}

/// CEP-0008 v1 flat profile envelope.
///
/// Maps to `sea.agent.event.v1.json` / the copied fixture
/// `semantic-envelope.schema.json`. `deny_unknown_fields` ensures no
/// unexpected top-level field can appear in serialized output.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Cep0008FlatEnvelopeV1 {
    pub schema_version: String,
    pub event_id: String,
    pub idempotency_key: String,
    pub source: String,
    pub source_agent: String,
    pub event_type: String,
    pub occurred_at: String,
    pub correlation_id: String,
    // causation_id omitted: SEA Forge has no source-envelope causation reference.
    pub trace_id: String,
    pub payload: Value,
    pub provenance: Cep0008Provenance,
    pub metadata: Value,
}

/// Allowed top-level field names per the CEP-0008 v1 flat schema
/// (`additionalProperties: false`).
const ALLOWED_TOP_LEVEL: &[&str] = &[
    "schema_version",
    "event_id",
    "idempotency_key",
    "source",
    "source_agent",
    "event_type",
    "occurred_at",
    "correlation_id",
    "causation_id",
    "trace_id",
    "payload",
    "provenance",
    "metadata",
];

/// Payload section keys included in the projection.
const PAYLOAD_SECTIONS: &[&str] = &[
    "representations",
    "authority",
    "evidence",
    "settlements",
    "capabilities",
    "artifacts",
    "extensions",
    "projections",
];

// ── Projection input ──

/// Typed projection input carrying the committed native envelope's ledger
/// identity and settlement timestamp.
pub struct Cep0008ProjectionInput<'a> {
    pub envelope: &'a SemanticEnvelope,
    pub ledger_id: &'a str,
    pub entry_ulid: &'a str,
    pub payload_hash: &'a str,
    pub settlement_timestamp: &'a str,
}

// ── Conversion function ──

/// Project a native `SemanticEnvelope` into the CEP-0008 v1 flat profile.
///
/// Pure: no filesystem, ledger, network, or clock operations.
/// Deterministic: identical inputs produce identical serialized bytes.
/// Fail-closed: malformed or missing required inputs return `schema_error`.
pub fn project_to_cep0008_flat_v1(
    input: &Cep0008ProjectionInput,
) -> Result<Cep0008FlatEnvelopeV1, ForgeError> {
    let env = input.envelope;

    // ── Validate required inputs (fail-closed) ──
    let require = |name: &str, val: &str| -> Result<(), ForgeError> {
        if val.trim().is_empty() {
            return Err(schema_error(format!("cep0008: {name} must not be empty")));
        }
        Ok(())
    };
    require("ledger_id", input.ledger_id)?;
    require("entry_ulid", input.entry_ulid)?;
    require("payload_hash", input.payload_hash)?;
    require("source_agent", &env.attribution.entity_id)?;
    require("run_id", &env.run_id)?;
    require("case_ref", &env.case_ref)?;
    require("settlement_timestamp", input.settlement_timestamp)?;

    // SUP-09i: emptiness alone is not grammar. These fields ride the wire as
    // `event_id` and `idempotency_key`, so a malformed value would poison
    // downstream consumers that trust the envelope's own identifiers.
    // Crockford base32: digits plus A-Z minus I, L, O, U.
    let valid_ulid = |v: &str| {
        v.len() == 26
            && v.chars().all(|c| {
                c.is_ascii_digit()
                    || (c.is_ascii_uppercase() && !matches!(c, 'I' | 'L' | 'O' | 'U'))
            })
    };
    if !valid_ulid(input.entry_ulid) {
        return Err(schema_error(format!(
            "cep0008: entry_ulid is not a valid 26-char ULID: {}",
            input.entry_ulid
        )));
    }
    let valid_sha = |v: &str| {
        v.strip_prefix("sha256:").is_some_and(|hex| {
            hex.len() == 64
                && hex
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        })
    };
    if !valid_sha(input.payload_hash) {
        return Err(schema_error(format!(
            "cep0008: payload_hash must be sha256:<64 lowercase hex>: {}",
            input.payload_hash
        )));
    }
    if !sea_forge_core::path::valid_id_segment(input.ledger_id, 128) {
        return Err(schema_error(format!(
            "cep0008: ledger_id is not a safe id segment: {}",
            input.ledger_id
        )));
    }

    // Validate RFC 3339 timestamp.
    if chrono::DateTime::parse_from_rfc3339(input.settlement_timestamp).is_err() {
        return Err(schema_error(format!(
            "cep0008: settlement_timestamp is not valid RFC 3339: {}",
            input.settlement_timestamp
        )));
    }

    // ── Build payload: CEP sections as separate keys ──
    let payload = json!({
        "representations": env.intent,
        "authority": env.authority_decisions,
        "evidence": env.evidence_refs,
        "settlements": {
            "settlement_ref": env.settlement_ref,
            "result": env.capability_delta.result,
        },
        "capabilities": {
            "attempted_capability": env.capability_delta.attempted_capability,
            "result": env.capability_delta.result,
        },
        "artifacts": env.artifact_refs,
        "extensions": env.extension_refs,
        "projections": env.projection_refs,
    });
    debug_assert!(payload.is_object(), "payload must be a JSON object");

    // ── Build provenance ──
    let provenance = Cep0008Provenance {
        origin: format!(
            "urn:sea-forge:ledger:{}:entry:{}",
            input.ledger_id, input.entry_ulid
        ),
        chain: vec![],
    };

    // ── Build metadata ──
    let mut metadata = json!({
        "scope": {
            "case": env.case_ref,
            "run": env.run_id,
        },
        "boundary_record": {
            "included_sections": PAYLOAD_SECTIONS,
            "known_omissions": [
                "causation_id: SEA Forge has no source-envelope causation reference",
                "provenance.chain: authority/evidence IDs are not source-envelope lineage",
            ],
        },
        "completeness_status": "partial",
        "omission_status": [
            "causation_id omitted: SEA Forge has no source-envelope causation reference",
            "provenance.chain empty: authority/evidence IDs are not source-envelope lineage",
            "CEP-0008 §52 concepts not fully carried in flat profile metadata",
        ],
        "conformance_status": "partially_conformant",
        "profile_id": "cep-0008-flat-v1",
        "native_envelope_version": env.version,
    });
    if let Some(cell_id) = &env.cell_id {
        metadata["cell_id"] = json!(cell_id);
    }
    debug_assert!(metadata.is_object(), "metadata must be a JSON object");

    let result = Cep0008FlatEnvelopeV1 {
        schema_version: "v1".into(),
        event_id: input.entry_ulid.into(),
        idempotency_key: input.payload_hash.into(),
        source: "sea-forge".into(),
        source_agent: env.attribution.entity_id.clone(),
        event_type: "composite".into(),
        occurred_at: input.settlement_timestamp.into(),
        correlation_id: env.case_ref.clone(),
        trace_id: env.run_id.clone(),
        payload,
        provenance,
        metadata,
    };

    // ── Verify no forbidden top-level fields in serialized output ──
    let serialized: Value = serde_json::to_value(&result)
        .map_err(|e| ForgeError::Serialization(format!("cep0008: serialize failed: {e}")))?;
    if let Some(obj) = serialized.as_object() {
        for key in obj.keys() {
            if !ALLOWED_TOP_LEVEL.contains(&key.as_str()) {
                return Err(schema_error(format!(
                    "cep0008: forbidden top-level field '{key}' in output"
                )));
            }
        }
    }

    Ok(result)
}

fn schema_error(message: String) -> ForgeError {
    ForgeError::Config {
        class: "schema_error",
        path: std::path::PathBuf::new(),
        message,
    }
}
