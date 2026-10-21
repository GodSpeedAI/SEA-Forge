//! Conformance for the `evidence.recorded` contract (ADR-012 obligation 2),
//! accepted by the owner 2026-10-18 and registered at RealityTrace 42879a8,
//! the `sxr-core` pin. The registered row must equal
//! `contracts/evidence-recorded.record-kind.psv`, and emitted E8 must satisfy
//! the schema and the invariants below.

use jsonschema::Validator;
use serde_json::{json, Value};
use std::str::FromStr;
use sxr_core::canonical_json::canonical_json;
use sxr_core::record_kinds::RecordKind;

const SCHEMA: &str = include_str!("../contracts/evidence-recorded.v1.schema.json");
const ROW: &str = include_str!("../contracts/evidence-recorded.record-kind.psv");
/// A real E8 emitted by the pinned `sxr-core` during the SEC-12 slice.
const BUNDLE: &str = include_str!(
    "../../../.agents/evidence/security-v2/realitytrace-sec12-20261017/bundle/sec12-trace.json"
);

fn schema() -> Validator {
    jsonschema::validator_for(&serde_json::from_str(SCHEMA).unwrap()).unwrap()
}

fn emitted() -> Value {
    serde_json::from_str::<Value>(BUNDLE).unwrap()["evidence_recorded"].clone()
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(bytes))
}

/// Invariants a schema cannot state; RealityTrace's emitter guarantees them.
pub fn invariants(e8: &Value) -> Result<(), String> {
    let p = &e8["payload"];
    if p["domain_model_ref"]["model_hash"] != p["domain_model_hash"] {
        return Err("domain_model_ref.model_hash differs from domain_model_hash".into());
    }
    let d = &p["difference"];
    let same = d["expected_sha256"] == d["observed_sha256"];
    if (d["verdict"] == "equivalent") != same {
        return Err("verdict disagrees with the compared digests".into());
    }
    for (side, digest) in [
        ("expected_outcome", "expected_sha256"),
        ("observed_outcome", "observed_sha256"),
    ] {
        if sha256_hex(canonical_json(&p[side]).as_bytes()) != d[digest] {
            return Err(format!("{digest} is not the digest of {side}"));
        }
    }
    let key = sha256_hex(format!("|EvidenceRecorded|{}", canonical_json(p)).as_bytes());
    if e8["idempotency_key"] != key {
        return Err("idempotency_key is not content-derived".into());
    }
    if e8["provenance"]["chain"][0]
        != format!(
            "domain_model_hash:{}",
            p["domain_model_hash"].as_str().unwrap_or("")
        )
    {
        return Err("provenance chain does not open with the model anchor".into());
    }
    let chain: Vec<&str> = e8["provenance"]["chain"]
        .as_array()
        .map(|c| c.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let count = |prefix: &str| chain.iter().filter(|l| l.starts_with(prefix)).count();
    if count("domain_model_hash:") != 1 || count("caused_by:") != 2 || count("proof_producer:") > 1
    {
        return Err(
            "provenance chain needs one anchor, two caused_by parents, at most one producer".into(),
        );
    }
    Ok(())
}

#[test]
fn registered_row_matches_the_accepted_contract() {
    // The pinned RealityTrace must register the accepted row, unchanged.
    let kind = RecordKind::from_str("evidence.recorded")
        .expect("the sxr-core pin no longer registers evidence.recorded");
    let row: Vec<&str> = ROW.lines().nth(1).unwrap().split('|').collect();
    let def = kind.def();
    assert_eq!(
        [
            def.kind,
            def.envelope_kind,
            def.primary_entities,
            def.subject_refs,
            def.provenance,
            def.predecessors,
            def.subject_prefixes,
            def.settlement_dependency,
            def.authority_class,
        ],
        row.as_slice(),
        "RealityTrace registered a different evidence.recorded row"
    );
}

#[test]
fn proposed_row_matches_the_pinned_registry_format() {
    let pinned_header = "kind|envelope_kind|primary_entities|required_subject_refs|provenance|allowed_predecessors|required_subject_prefixes|settlement_dependency|authority_class";
    let mut lines = ROW.lines();
    assert_eq!(lines.next(), Some(pinned_header));
    let row: Vec<&str> = lines.next().unwrap().split('|').collect();
    assert_eq!(row.len(), 9);
    assert_eq!(row[0], "evidence.recorded");
    assert!(lines.next().is_none());
    // CEP owns the vocabulary; the row must map into it, not extend it.
    let cep = sxr_core::embedded_schema::schema_json();
    let vocabulary = cep["properties"]["envelope_kind"]["enum"]
        .as_array()
        .unwrap();
    assert!(vocabulary.iter().any(|k| k == row[1]), "{}", row[1]);
    // Same envelope kind as a registered evidence kind, distinct name.
    let siblings: Vec<_> = RecordKind::ALL
        .iter()
        .filter(|k| k.def().envelope_kind == row[1])
        .map(|k| k.as_str())
        .collect();
    assert!(siblings.contains(&"evidence.created"), "{siblings:?}");
}

#[test]
fn emitted_evidence_conforms() {
    let e8 = emitted();
    let schema = schema();
    if let Err(error) = schema.validate(&e8) {
        panic!("{error}");
    }
    invariants(&e8).unwrap();
}

#[test]
fn schema_and_invariants_reject_drift() {
    let schema = schema();
    let base = emitted();
    let mutate = |f: &dyn Fn(&mut Value)| {
        let mut v = base.clone();
        f(&mut v);
        v
    };
    let rejected_by_schema: Vec<(&str, Value)> = vec![
        (
            "plaintext field added",
            mutate(&|v| v["payload"]["raw_output"] = json!("secret")),
        ),
        (
            "envelope field added",
            mutate(&|v| v["credential"] = json!("x")),
        ),
        (
            "unknown verdict",
            mutate(&|v| v["payload"]["difference"]["verdict"] = json!("partial")),
        ),
        (
            "missing difference",
            mutate(&|v| {
                v["payload"].as_object_mut().unwrap().remove("difference");
            }),
        ),
        (
            "null observation",
            mutate(&|v| v["payload"]["observed_outcome"] = Value::Null),
        ),
        (
            "wrong producer",
            mutate(&|v| v["source_agent"] = json!("sea-forge")),
        ),
        (
            "schema version",
            mutate(&|v| v["schema_version"] = json!("v2")),
        ),
        (
            "bare settlement ref",
            mutate(&|v| v["payload"]["operational_settlement_ref"] = json!("8f9e")),
        ),
        (
            "short chain",
            mutate(&|v| {
                v["provenance"]["chain"].as_array_mut().unwrap().pop();
            }),
        ),
    ];
    for (name, value) in &rejected_by_schema {
        assert!(!schema.is_valid(value), "schema accepted: {name}");
    }
    let rejected_by_invariants: Vec<(&str, Value)> = vec![
        (
            "verdict flipped",
            mutate(&|v| v["payload"]["difference"]["verdict"] = json!("equivalent")),
        ),
        (
            "observation edited",
            mutate(&|v| v["payload"]["observed_outcome"][0]["effect"] = json!("edited")),
        ),
        (
            "model anchor drift",
            mutate(&|v| v["payload"]["domain_model_ref"]["model_hash"] = json!("a".repeat(64))),
        ),
        (
            "anchor not first",
            mutate(&|v| v["provenance"]["chain"].as_array_mut().unwrap().swap(0, 1)),
        ),
        (
            "parent replaced by a producer line",
            mutate(&|v| v["provenance"]["chain"][1] = json!("proof_producer:other")),
        ),
    ];
    for (name, value) in &rejected_by_invariants {
        // Schema-valid, so only the invariants can catch it.
        assert!(
            schema.is_valid(value),
            "schema rejects {name}; invariant untested"
        );
        assert!(invariants(value).is_err(), "invariants accepted: {name}");
    }
}
