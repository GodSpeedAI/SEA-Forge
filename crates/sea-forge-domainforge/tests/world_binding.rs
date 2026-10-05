//! Stage 4: SEA-Forge binds governance to a verified, registered `world_ref`.

use domainforge_core::application::envelope::{
    build_cep_envelope, CepEnvelopeParams, CepWorldParams,
};
use domainforge_core::application::resolve_semantic_envelope;
use domainforge_core::application::world::{WorldName, WorldRefError};
use sea_forge_domainforge::{
    load_validate, SeaSourceSet, SourceFile, WorldBindingError, WorldRegistry,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const DEMO: &str = include_str!("fixtures/demo.sea");

fn set(content: &str) -> SeaSourceSet {
    SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![SourceFile {
            uri: "demo.sea".into(),
            sha256: format!("{:x}", Sha256::digest(content.as_bytes())),
            content: content.into(),
        }],
    }
}

/// A real DomainForge CEP `semantic_snapshot` for `content`, world-bound.
fn snapshot(content: &str, name: &str) -> Value {
    let sources = json!({ "demo.sea": content }).to_string();
    let doc = resolve_semantic_envelope("demo.sea", &sources).unwrap();
    let world_name: WorldName = name.parse().unwrap();
    build_cep_envelope(&CepEnvelopeParams {
        doc: Some(&doc),
        model_valid: true,
        source_set_hash: &doc.inputs.source_set_hash,
        invalid_declared_checkpoint_hash: None,
        diagnostics: &[],
        scope: json!({"subject": "demo"}),
        entry_logical_path: "demo.sea",
        inline_threshold_bytes: 1024,
        envelope_id: "env:test",
        created_at: "2026-10-04T00:00:00Z",
        registry_content_hash: None,
        resolved_namespaces: &[],
        world: Some(CepWorldParams {
            name: &world_name,
            alias: None,
            label: None,
        }),
    })
}

fn registry() -> (WorldRegistry, String) {
    let mut reg = WorldRegistry::new();
    let world = reg.register_source_set("demo", &set(DEMO)).unwrap();
    (reg, world.to_string())
}

#[test]
fn registered_world_verifies_a_real_snapshot() {
    let (reg, world_ref) = registry();
    let snap = snapshot(DEMO, "demo");
    assert_eq!(snap["scope"]["world_ref"], world_ref);
    assert_eq!(reg.verify_snapshot(&snap).unwrap().to_string(), world_ref);
}

#[test]
fn registration_is_idempotent_and_cached_by_world_ref() {
    let (mut reg, world_ref) = registry();
    let again = reg.register_source_set("demo", &set(DEMO)).unwrap();
    assert_eq!(again.to_string(), world_ref);
    assert_eq!(reg.len(), 1);
    assert!(reg.require(&world_ref).is_ok());
}

#[test]
fn unknown_world_fails_closed_even_with_a_valid_snapshot() {
    let reg = WorldRegistry::new();
    let err = reg.verify_snapshot(&snapshot(DEMO, "demo")).unwrap_err();
    assert!(matches!(
        err,
        WorldBindingError::World(WorldRefError::UnknownWorld(_))
    ));
}

#[test]
fn a_changed_world_is_unknown_until_registered() {
    let (mut reg, _) = registry();
    let changed = format!("// edit\n{DEMO}");
    let snap = snapshot(&changed, "demo");
    assert!(reg.verify_snapshot(&snap).is_err());
    reg.register_source_set("demo", &set(&changed)).unwrap();
    assert!(reg.verify_snapshot(&snap).is_ok());
    assert_eq!(reg.len(), 2);
}

#[test]
fn tampered_identity_is_rejected_by_recomputation() {
    let (reg, _) = registry();
    let mut snap = snapshot(DEMO, "demo");
    snap["extensions"]["domainforge.identity"]["domain_model_identity"]["semantic_closure_hash"] =
        json!(format!("sha256:{}", "0".repeat(64)));
    assert!(matches!(
        reg.verify_snapshot(&snap).unwrap_err(),
        WorldBindingError::World(WorldRefError::DigestMismatch { .. })
    ));
}

#[test]
fn forged_world_ref_over_a_genuine_identity_is_rejected() {
    let (reg, _) = registry();
    let mut snap = snapshot(DEMO, "demo");
    snap["scope"]["world_ref"] = json!(format!("world:demo@sha256:{}", "a".repeat(64)));
    assert!(reg.verify_snapshot(&snap).is_err());
}

#[test]
fn missing_or_malformed_pieces_fail_closed() {
    let (reg, _) = registry();
    let mut no_ref = snapshot(DEMO, "demo");
    no_ref["scope"].as_object_mut().unwrap().remove("world_ref");
    assert_eq!(
        reg.verify_snapshot(&no_ref).unwrap_err(),
        WorldBindingError::MissingWorldRef
    );

    let mut no_id = snapshot(DEMO, "demo");
    no_id["extensions"]
        .as_object_mut()
        .unwrap()
        .remove("domainforge.identity");
    assert_eq!(
        reg.verify_snapshot(&no_id).unwrap_err(),
        WorldBindingError::MissingIdentity
    );

    let mut bad_ref = snapshot(DEMO, "demo");
    bad_ref["scope"]["world_ref"] = json!("world:demo");
    assert!(reg.verify_snapshot(&bad_ref).is_err());
}

#[test]
fn invalid_model_snapshot_is_never_bound() {
    let (reg, _) = registry();
    let mut snap = snapshot(DEMO, "demo");
    snap["extensions"]["domainforge"] = json!({"model_validation_status": "invalid"});
    assert_eq!(
        reg.verify_snapshot(&snap).unwrap_err(),
        WorldBindingError::InvalidModel
    );
}

#[test]
fn model_ref_binds_and_reverifies_against_its_world() {
    let (reg, world_ref) = registry();
    let mut model = load_validate(&set(DEMO)).unwrap().model_ref;
    assert_eq!(
        reg.verify_model_ref(&model).unwrap_err(),
        WorldBindingError::ModelRefUnbound
    );
    reg.bind(&mut model, &world_ref).unwrap();
    assert_eq!(model.world_ref.as_deref(), Some(world_ref.as_str()));
    assert_eq!(reg.verify_model_ref(&model).unwrap().to_string(), world_ref);

    model.semantic_closure_hash = Some(format!("sha256:{}", "1".repeat(64)));
    assert_eq!(
        reg.verify_model_ref(&model).unwrap_err(),
        WorldBindingError::ModelRefMismatch {
            field: "semantic_closure_hash"
        }
    );
}

#[test]
fn model_ref_of_a_different_world_cannot_bind() {
    let (mut reg, _) = registry();
    let other = format!("// edit\n{DEMO}");
    let other_ref = reg
        .register_source_set("demo", &set(&other))
        .unwrap()
        .to_string();
    let mut model = load_validate(&set(DEMO)).unwrap().model_ref;
    // Same semantic closure, different source text => different d_content_hash.
    assert_eq!(
        reg.bind(&mut model, &other_ref).unwrap_err(),
        WorldBindingError::ModelRefMismatch {
            field: "d_content_hash"
        }
    );
    assert!(model.world_ref.is_none());
}

#[test]
fn unbound_records_still_deserialize_and_omit_the_field() {
    let model = load_validate(&set(DEMO)).unwrap().model_ref;
    let mut value = serde_json::to_value(&model).unwrap();
    assert!(value.get("world_ref").is_none());
    value.as_object_mut().unwrap().remove("world_ref");
    let back: sea_forge_domainforge::DomainModelRef = serde_json::from_value(value).unwrap();
    assert_eq!(back, model);
}
