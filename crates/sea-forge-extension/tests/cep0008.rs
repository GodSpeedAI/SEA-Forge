//! CEP-0008 v1 flat profile adapter tests.
//!
//! Proves the adapter's contract: deterministic output, all required fields
//! present, no forbidden fields, CEP sections distinct, metadata explicit,
//! provenance identifies the native ledger record, fail-closed on bad input,
//! and source envelope unchanged.

use sea_forge_core::types::{
    ArtifactRef, Attribution, CapabilityDelta, Intent, SemanticEnvelope, SettlementStatus,
};
use sea_forge_extension::cep0008::{
    project_to_cep0008_flat_v1, Cep0008FlatEnvelopeV1, Cep0008ProjectionInput,
};
use serde_json::Value;
use std::collections::BTreeSet;

fn test_envelope() -> SemanticEnvelope {
    SemanticEnvelope {
        version: "0.2".into(),
        run_id: "run_01".into(),
        case_ref: "case_01".into(),
        intent: Intent {
            intent_id: "int_01".into(),
            summary: "demo".into(),
            actor_id: "operator_local".into(),
            process_id: "proc".into(),
            created_at: "2026-07-16T00:00:00Z".into(),
        },
        plan_ref: "plan_01".into(),
        template_ref: Some("demo@0.1.0".into()),
        authority_decisions: vec!["auth_01".into()],
        evidence_refs: vec!["evi_01".into()],
        settlement_ref: "set_01".into(),
        capability_delta: CapabilityDelta {
            attempted_capability: "demo".into(),
            result: SettlementStatus::Accepted,
        },
        attribution: Attribution {
            entity_id: "operator_local".into(),
            process_id: "proc".into(),
            session_id: "case_01".into(),
        },
        artifact_refs: vec![ArtifactRef {
            evidence_id: "evi_01".into(),
            artifact_id: "art_01".into(),
            pre_mint_identity: "ifl:hash:abc".into(),
        }],
        extension_refs: vec![],
        projection_refs: vec![],
        cell_id: Some("cell_12345678".into()),
    }
}

fn test_input<'a>(env: &'a SemanticEnvelope) -> Cep0008ProjectionInput<'a> {
    Cep0008ProjectionInput {
        envelope: env,
        ledger_id: "ledger_main",
        entry_ulid: "01J9XKQZ8GTDNR4P3SMQVWHJEY",
        payload_hash: "sha256:abcdef0123456789",
        settlement_timestamp: "2026-07-16T12:00:00Z",
    }
}

fn project(env: &SemanticEnvelope) -> Cep0008FlatEnvelopeV1 {
    project_to_cep0008_flat_v1(&test_input(env)).unwrap()
}

fn project_json(env: &SemanticEnvelope) -> Value {
    serde_json::to_value(project(env)).unwrap()
}

/// Test 1: All six required flat-profile fields are present.
#[test]
fn cep0008_all_required_fields_present() {
    let json = project_json(&test_envelope());
    let required = [
        "schema_version",
        "event_id",
        "source_agent",
        "occurred_at",
        "payload",
        "provenance",
    ];
    for field in &required {
        assert!(
            json.get(field).is_some(),
            "required field '{field}' must be present"
        );
    }
}

/// Test 2: No forbidden top-level fields are emitted.
#[test]
fn cep0008_no_forbidden_top_level_fields() {
    let json = project_json(&test_envelope());
    let allowed: BTreeSet<&str> = [
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
    ]
    .into_iter()
    .collect();
    let actual: BTreeSet<&str> = json
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.as_str())
        .collect();
    let forbidden: Vec<&str> = actual.difference(&allowed).copied().collect();
    assert!(
        forbidden.is_empty(),
        "forbidden top-level fields: {forbidden:?}"
    );
}

/// Test 3: schema_version == "v1" and source == "sea-forge".
#[test]
fn cep0008_schema_version_and_source_correct() {
    let json = project_json(&test_envelope());
    assert_eq!(json["schema_version"], "v1");
    assert_eq!(json["source"], "sea-forge");
}

/// Test 4: CEP sections remain distinct inside payload.
#[test]
fn cep0008_payload_sections_distinct() {
    let json = project_json(&test_envelope());
    let payload = &json["payload"];
    assert!(payload.is_object());
    let sections = [
        "representations",
        "authority",
        "evidence",
        "settlements",
        "capabilities",
        "artifacts",
        "extensions",
        "projections",
    ];
    for section in &sections {
        assert!(
            payload.get(section).is_some(),
            "payload section '{section}' must be present"
        );
    }
    // Verify content is mapped correctly
    let env = test_envelope();
    let p = project(&env);
    assert_eq!(
        p.payload["authority"],
        serde_json::json!(env.authority_decisions)
    );
    assert_eq!(p.payload["evidence"], serde_json::json!(env.evidence_refs));
    assert_eq!(
        p.payload["settlements"]["settlement_ref"],
        env.settlement_ref
    );
    assert_eq!(
        p.payload["capabilities"]["attempted_capability"],
        env.capability_delta.attempted_capability
    );
}

/// Test 5: Boundary, completeness, omission, and partial-conformance metadata
/// are explicit.
#[test]
fn cep0008_metadata_explicit_boundary_completeness_omission() {
    let json = project_json(&test_envelope());
    let meta = &json["metadata"];
    assert!(meta.is_object());
    assert_eq!(meta["completeness_status"], "partial");
    assert_eq!(meta["conformance_status"], "partially_conformant");
    assert_eq!(meta["profile_id"], "cep-0008-flat-v1");
    assert!(meta.get("boundary_record").is_some());
    assert!(meta["boundary_record"]["included_sections"].is_array());
    assert!(meta["boundary_record"]["known_omissions"].is_array());
    assert!(meta["omission_status"].is_array());
    assert!(!meta["omission_status"].as_array().unwrap().is_empty());
}

/// Test 6: Provenance identifies the committed native ledger record.
#[test]
fn cep0008_provenance_identifies_ledger_record() {
    let env = test_envelope();
    let result = project(&env);
    let origin = &result.provenance.origin;
    assert!(origin.starts_with("urn:sea-forge:ledger:"));
    assert!(origin.contains("ledger_main"));
    assert!(origin.contains("01J9XKQZ8GTDNR4P3SMQVWHJEY"));
}

/// Test 7: Identical inputs serialize to identical bytes.
#[test]
fn cep0008_identical_inputs_identical_bytes() {
    let env = test_envelope();
    let a = serde_json::to_vec(&project(&env)).unwrap();
    let b = serde_json::to_vec(&project(&env)).unwrap();
    assert_eq!(
        a, b,
        "identical inputs must produce identical serialized bytes"
    );
}

/// Test 8: Malformed or missing required inputs fail closed.
#[test]
fn cep0008_empty_ledger_id_fails_closed() {
    let env = test_envelope();
    let mut input = test_input(&env);
    input.ledger_id = "";
    let err = project_to_cep0008_flat_v1(&input).unwrap_err();
    assert_eq!(err.class(), "schema_error");
}

#[test]
fn cep0008_empty_entry_ulid_fails_closed() {
    let env = test_envelope();
    let mut input = test_input(&env);
    input.entry_ulid = "";
    let err = project_to_cep0008_flat_v1(&input).unwrap_err();
    assert_eq!(err.class(), "schema_error");
}

#[test]
fn cep0008_empty_payload_hash_fails_closed() {
    let env = test_envelope();
    let mut input = test_input(&env);
    input.payload_hash = "";
    let err = project_to_cep0008_flat_v1(&input).unwrap_err();
    assert_eq!(err.class(), "schema_error");
}

#[test]
fn cep0008_empty_source_agent_fails_closed() {
    let mut env = test_envelope();
    env.attribution.entity_id = "".into();
    let err = project_to_cep0008_flat_v1(&test_input(&env)).unwrap_err();
    assert_eq!(err.class(), "schema_error");
}

#[test]
fn cep0008_empty_run_id_fails_closed() {
    let mut env = test_envelope();
    env.run_id = "".into();
    let err = project_to_cep0008_flat_v1(&test_input(&env)).unwrap_err();
    assert_eq!(err.class(), "schema_error");
}

#[test]
fn cep0008_empty_case_ref_fails_closed() {
    let mut env = test_envelope();
    env.case_ref = "".into();
    let err = project_to_cep0008_flat_v1(&test_input(&env)).unwrap_err();
    assert_eq!(err.class(), "schema_error");
}

#[test]
fn cep0008_empty_settlement_timestamp_fails_closed() {
    let env = test_envelope();
    let mut input = test_input(&env);
    input.settlement_timestamp = "";
    let err = project_to_cep0008_flat_v1(&input).unwrap_err();
    assert_eq!(err.class(), "schema_error");
}

#[test]
fn cep0008_invalid_rfc3339_timestamp_fails_closed() {
    let env = test_envelope();
    let mut input = test_input(&env);
    input.settlement_timestamp = "not-a-timestamp";
    let err = project_to_cep0008_flat_v1(&input).unwrap_err();
    assert_eq!(err.class(), "schema_error");
}

/// Test 9: The source SemanticEnvelope is unchanged after projection.
#[test]
fn cep0008_source_envelope_unchanged() {
    let env = test_envelope();
    let env_before = serde_json::to_vec(&env).unwrap();
    let _ = project(&env);
    let env_after = serde_json::to_vec(&env).unwrap();
    assert_eq!(
        env_before, env_after,
        "source SemanticEnvelope must not be mutated"
    );
}

/// Test 10: cell_id from native envelope appears in metadata, not top-level.
#[test]
fn cep0008_cell_id_in_metadata_not_top_level() {
    let env = test_envelope();
    let json = project_json(&env);
    assert!(
        json.get("cell_id").is_none(),
        "cell_id must not be top-level"
    );
    assert_eq!(json["metadata"]["cell_id"], "cell_12345678");
}

/// Test 11: envelope without cell_id omits it from metadata.
#[test]
fn cep0008_no_cell_id_when_absent() {
    let mut env = test_envelope();
    env.cell_id = None;
    let json = project_json(&env);
    assert!(json["metadata"].get("cell_id").is_none());
}

/// Test 12: event_type is "composite" (CEP-0008 §9).
#[test]
fn cep0008_event_type_is_composite() {
    let json = project_json(&test_envelope());
    assert_eq!(json["event_type"], "composite");
}

/// Test 13: trace_id maps to run_id, correlation_id maps to case_ref.
#[test]
fn cep0008_trace_and_correlation_mapped() {
    let env = test_envelope();
    let json = project_json(&env);
    assert_eq!(json["trace_id"], env.run_id);
    assert_eq!(json["correlation_id"], env.case_ref);
}

/// Test 14: idempotency_key maps to payload_hash, event_id maps to entry_ulid.
#[test]
fn cep0008_idempotency_and_event_id_mapped() {
    let env = test_envelope();
    let result = project(&env);
    assert_eq!(result.event_id, "01J9XKQZ8GTDNR4P3SMQVWHJEY");
    assert_eq!(result.idempotency_key, "sha256:abcdef0123456789");
}
