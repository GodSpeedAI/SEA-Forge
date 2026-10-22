//! Prior identities (compiler_upgrade lineage): a superseded identity is registered under its world's name,
//! its ref matches its document, and it can never be the current world.

use sea_forge_domainforge::{SeaSourceSet, SourceFile, WorldRegistry};
use sha2::{Digest, Sha256};

const SRC: &str = "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";
const FIXTURE_0_21_0: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../sea-forge-authority/tests/fixtures/identities/demo-tank.domainforge-0.21.0.json"
));

fn current(src: &str) -> (WorldRegistry, String) {
    let mut reg = WorldRegistry::new();
    let set = SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![SourceFile {
            uri: "demo.sea".into(),
            sha256: hex::encode(Sha256::digest(src.as_bytes())),
            content: src.into(),
        }],
    };
    let world = reg.register_source_set("demo", &set).unwrap().to_string();
    (reg, world)
}

#[test]
fn the_0_21_0_fixture_mints_the_recorded_prior_ref_and_the_0_21_1_build_mints_the_current_one() {
    let (mut reg, now) = current(SRC);
    assert_eq!(
        now,
        "world:demo@sha256:a71ff6d4d9941a937a2a2b7fd5ed1c1eabb2242b4e6919440927ca1772cbb6ca"
    );
    let prior = reg
        .register_prior_identity_json("demo", FIXTURE_0_21_0)
        .unwrap()
        .to_string();
    assert_eq!(
        prior,
        "world:demo@sha256:c2d3978a06cad4f44b8f50ef4a9e570c4800f1ba6b7e4ac76a0128337e0f3439"
    );
    assert!(reg.is_prior(&prior));
    assert!(!reg.is_prior(&now));
    // The prior still resolves, so it can be the source of a transition.
    assert!(reg.require(&prior).is_ok());
}

#[test]
fn a_prior_identity_equal_to_a_current_world_is_refused_and_keeps_its_role() {
    // The 0.21.0 document registered as current: it cannot then also be a prior (one role per world).
    let (mut reg, _) = current(SRC);
    let current_ref = reg
        .register_identity_json("demo", FIXTURE_0_21_0)
        .unwrap()
        .to_string();
    assert!(reg
        .register_prior_identity_json("demo", FIXTURE_0_21_0)
        .is_err());
    assert!(!reg.is_prior(&current_ref));

    // And the reverse: a registered prior cannot be re-registered as current.
    let (mut reg2, _) = current(SRC);
    let prior_ref = reg2
        .register_prior_identity_json("demo", FIXTURE_0_21_0)
        .unwrap()
        .to_string();
    assert!(reg2.register_identity_json("demo", FIXTURE_0_21_0).is_err());
    assert!(reg2.is_prior(&prior_ref));
}

#[test]
fn a_prior_identity_must_be_a_strict_identity_document() {
    let mut reg = WorldRegistry::new();
    assert!(reg
        .register_prior_identity_json("demo", "{not json")
        .is_err());
    let mut v: serde_json::Value = serde_json::from_str(FIXTURE_0_21_0).unwrap();
    v["surprise"] = serde_json::json!("extra");
    assert!(reg
        .register_prior_identity_json("demo", &v.to_string())
        .is_err());
    assert!(reg.is_empty());
}

#[test]
fn a_mutated_prior_document_gets_a_ref_of_its_own_and_is_not_the_recorded_one() {
    let mut reg = WorldRegistry::new();
    let mut v: serde_json::Value = serde_json::from_str(FIXTURE_0_21_0).unwrap();
    v["source_set_hash"] = serde_json::json!(format!("sha256:{}", "0".repeat(64)));
    let mutated = reg
        .register_prior_identity_json("demo", &v.to_string())
        .unwrap();
    assert_ne!(
        mutated.to_string(),
        "world:demo@sha256:c2d3978a06cad4f44b8f50ef4a9e570c4800f1ba6b7e4ac76a0128337e0f3439"
    );
}
