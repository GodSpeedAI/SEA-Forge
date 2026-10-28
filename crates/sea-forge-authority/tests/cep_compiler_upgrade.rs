//! `compiler_upgrade` is the one explicit, auditable move for a producer upgrade (for example DomainForge
//! 0.21.0 to 0.21.2). SEA-Forge allows it only when every meaning-bearing identity field is equal and only
//! `producer_version` moved. A prior identity is a place to move work out of, never to act in.

use sea_forge_authority::cep::{CepAuthorityError, CepAuthorityService, CepDecision};
use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_domainforge::{SeaSourceSet, SourceFile, WorldRegistry};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// The real DomainForge 0.21.0 identity for `BASE` (see the fixture's provenance).
const FIXTURE_0_21_0: &str = include_str!("fixtures/identities/demo-tank.domainforge-0.21.0.json");
const BASE: &str = "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";
const OLD_REF: &str =
    "world:demo@sha256:c2d3978a06cad4f44b8f50ef4a9e570c4800f1ba6b7e4ac76a0128337e0f3439";
const NEW_REF: &str =
    "world:demo@sha256:7ddedbcee1a7031c5af7b5ecd54d2eb3298f4d6149c0eb88fd2db191323351d3";
/// A different model: a real semantic change, never an upgrade.
const CHANGED: &str =
    "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\nentity \"Pump\" { key id: uuid }\n";

const OPEN: &str = "version: \"0.1\"\nrules:\n  \
  - name: open\n    verdict: allow\n    actor_role: service\n    operation_kind: world_transition\n  \
  - name: actions\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_action\n  \
  - name: approvers\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n";

fn service() -> Actor {
    Actor {
        actor_id: "cognate-service".into(),
        role: ActorRole::Service,
    }
}

fn in_process(worlds: &mut WorldRegistry, src: &str) -> String {
    let set = SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![SourceFile {
            uri: "demo.sea".into(),
            sha256: hex::encode(Sha256::digest(src.as_bytes())),
            content: src.into(),
        }],
    };
    worlds
        .register_source_set("demo", &set)
        .unwrap()
        .to_string()
}

/// The 0.21.0 fixture as a prior identity, and the in-process (linked) world for `BASE`.
fn upgrade_pair() -> (WorldRegistry, String, String) {
    let mut reg = WorldRegistry::new();
    let old = reg
        .register_prior_identity_json("demo", FIXTURE_0_21_0)
        .unwrap()
        .to_string();
    let new = in_process(&mut reg, BASE);
    (reg, old, new)
}

fn with_worlds<T>(
    root: &std::path::Path,
    worlds: &WorldRegistry,
    policy: &str,
    f: impl FnOnce(&CepAuthorityService<'_>) -> T,
) -> T {
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(policy).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let service = CepAuthorityService {
        engine: &engine,
        bundle: &bundle,
        worlds,
        root,
        approval_ttl: chrono::Duration::hours(24),
        criteria: Default::default(),
    };
    f(&service)
}

/// An authority_request envelope (shape as in cep_transition.rs). `kind` is the operation kind, `claim` the
/// `godspeed.world_transition` extension for a transition.
fn envelope(source: &str, op: &str, kind: &str, claim: Option<Value>, lineage: &[&str]) -> Value {
    let mut ext = json!({
        "operation_id": op, "correlation_id": "corr-1",
        "operation_kind": kind, "operation_name": if kind == "world_transition" { "transition" } else { "author" },
        "resource_id": "tank"
    });
    if kind == "world_transition" {
        ext["resource_id"] = json!(claim.as_ref().unwrap()["target_world_ref"]);
    }
    let mut extensions = json!({
        "cep.profile": {"profile_id": "godspeed.authority_request", "profile_version": "1.0.0"},
        "godspeed.authority_request": ext
    });
    if let Some(c) = claim {
        extensions["godspeed.world_transition"] = c;
    }
    json!({
        "envelope_id": format!("env-authority_request-{op}"),
        "cep_version": "1.0.0", "envelope_version": "1.0", "envelope_kind": "authority_request",
        "created_at": "2026-10-21T12:00:00Z", "created_by": "cognate",
        "scope": {"evaluation_context": "cognate-authority_request", "world_ref": source},
        "boundary_record": {"scope": "s", "included_sections": ["authority"], "excluded_sections": [],
            "known_omissions": ["payload not carried"], "unknowns": [], "redactions": [],
            "compression_notes": [], "out_of_scope_entities": [], "classification": "internal", "limitations": []},
        "completeness_status": "partial", "omission_status": "marked",
        "provenance_refs": [format!("prov-{op}")], "validation_status": "valid", "lineage_refs": lineage,
        "provenance": [{"provenance_id": format!("prov-{op}"), "source_system_refs": ["cognate"],
            "producer_refs": ["cognate"], "production_method": "governed_operation", "created_at": "2026-10-21T12:00:00Z"}],
        "authority": [{"authority_id": "auth-1", "subject_actor_ref": "actor:alice"}],
        "transformations": [{"transformation_id": "tr-1", "transformation_type": "action_invocation"}],
        "references": [{"ref_id": "r1", "ref_type": "artifact_id", "target_uri_or_id": "tank",
            "availability_status": "available", "integrity_status": "verifiable"}],
        "states": [{"state_id": "s1", "state_type": "declared_state", "subject_ref": "run:1"}],
        "extensions": extensions
    })
}

fn transition(source: &str, target: &str, op: &str, kind: &str) -> Value {
    envelope(
        source,
        op,
        "world_transition",
        Some(
            json!({"target_world_ref": target, "transition_kind": kind, "reason": "adopt the producer upgrade", "compatibility": "compatible"}),
        ),
        &[],
    )
}

fn decision(d: &CepDecision) -> &str {
    d.envelope["extensions"]["godspeed.authority_decision"]["decision"]
        .as_str()
        .unwrap()
}

#[test]
fn the_recorded_0_21_0_and_the_0_21_1_refs_are_what_these_tests_assume() {
    let (_, old, new) = upgrade_pair();
    assert_eq!(old, OLD_REF);
    assert_eq!(new, NEW_REF);
}

#[test]
fn a_compiler_upgrade_from_0_21_0_to_the_linked_build_is_allowed_and_recorded_with_both_producer_versions(
) {
    let root = tempfile::tempdir().unwrap();
    let (reg, old, new) = upgrade_pair();
    let d = with_worlds(root.path(), &reg, OPEN, |s| {
        s.decide(
            &transition(&old, &new, "up-1", "compiler_upgrade"),
            &service(),
        )
    })
    .unwrap();
    assert_eq!(decision(&d), "allow");
    let facts = with_worlds(root.path(), &reg, OPEN, |s| s.transitions()).unwrap();
    assert_eq!(facts.len(), 1);
    let rec = &facts[0].record;
    assert_eq!(rec["transition_kind"], "compiler_upgrade");
    assert_eq!(rec["source_world_ref"], OLD_REF);
    assert_eq!(rec["target_world_ref"], NEW_REF);
    assert_eq!(rec["semantic_closure_equal"], true);
    assert_eq!(rec["compatibility"], "compatible");
    assert_eq!(
        rec["evidence_refs"],
        json!([
            "domainforge.producer_version.source:0.21.0",
            "domainforge.producer_version.target:0.21.2"
        ])
    );
}

/// Each meaning-bearing field, changed alone on a copy of the 0.21.0 identity. The copy is registered as the
/// prior, so its ref is its own and the refusal must name exactly the field that moved.
#[test]
fn a_compiler_upgrade_is_refused_when_any_meaning_bearing_field_differs() {
    let mutations: Vec<(&str, Value)> = vec![
        (
            "semantic_closure_hash",
            json!(format!("sha256:{}", "3".repeat(64))),
        ),
        (
            "source_set_hash",
            json!(format!("sha256:{}", "1".repeat(64))),
        ),
        ("semantic_pack_set_hash", Value::Null),
        (
            "registry_content_hash",
            json!(format!("sha256:{}", "2".repeat(64))),
        ),
        ("identity_scheme_version", json!("v3-next")),
        ("language_schema_version", json!("domainforge-ast/v4")),
        (
            "compiler_interpretation_version",
            json!("domainforge-interpretation/v2"),
        ),
        (
            "canonicalization_version",
            json!("domainforge-canonicalization/v2"),
        ),
        ("producer", json!("another-producer")),
    ];
    for (field, value) in mutations {
        let root = tempfile::tempdir().unwrap();
        let mut doc: Value = serde_json::from_str(FIXTURE_0_21_0).unwrap();
        doc[field] = value;
        let mut reg = WorldRegistry::new();
        if field == "identity_scheme_version" {
            // DomainForge supports one identity scheme: an unknown one never registers at all.
            assert!(reg
                .register_prior_identity_json("demo", &doc.to_string())
                .is_err());
            continue;
        }
        let old = reg
            .register_prior_identity_json("demo", &doc.to_string())
            .unwrap()
            .to_string();
        let new = in_process(&mut reg, BASE);
        let r = with_worlds(root.path(), &reg, OPEN, |s| {
            s.decide(
                &transition(&old, &new, "up-m", "compiler_upgrade"),
                &service(),
            )
        });
        match r {
            Err(CepAuthorityError::Transition(m)) => {
                assert!(m.contains(field), "{field}: refusal did not name it: {m}")
            }
            other => panic!("{field}: expected a misdeclared refusal, got {other:?}"),
        }
        assert!(with_worlds(root.path(), &reg, OPEN, |s| s.transitions())
            .unwrap()
            .is_empty());
    }
}

#[test]
fn a_compiler_upgrade_is_refused_when_the_producer_version_is_equal() {
    let root = tempfile::tempdir().unwrap();
    let mut doc: Value = serde_json::from_str(FIXTURE_0_21_0).unwrap();
    // Equality is against the LINKED producer version, so the case survives pin bumps.
    doc["producer_version"] = json!(sea_forge_domainforge::EXPECTED_DOMAINFORGE_VERSION);
    let mut reg = WorldRegistry::new();
    let old = reg
        .register_prior_identity_json("demo", &doc.to_string())
        .unwrap()
        .to_string();
    let new = in_process(&mut reg, BASE);
    let r = with_worlds(root.path(), &reg, OPEN, |s| {
        s.decide(
            &transition(&old, &new, "up-eq", "compiler_upgrade"),
            &service(),
        )
    });
    assert!(
        matches!(&r, Err(CepAuthorityError::Transition(m)) if m.contains("not an upgrade")),
        "{r:?}"
    );
}

#[test]
fn a_source_edit_only_claim_across_the_upgrade_is_refused_as_an_upgrade_in_disguise() {
    let root = tempfile::tempdir().unwrap();
    let (reg, old, new) = upgrade_pair();
    let r = with_worlds(root.path(), &reg, OPEN, |s| {
        s.decide(
            &transition(&old, &new, "up-se", "source_edit_only"),
            &service(),
        )
    });
    assert!(
        matches!(&r, Err(CepAuthorityError::Transition(m)) if m.contains("producer")),
        "{r:?}"
    );
    assert!(with_worlds(root.path(), &reg, OPEN, |s| s.transitions())
        .unwrap()
        .is_empty());
}

#[test]
fn a_real_semantic_change_is_never_a_compiler_upgrade() {
    let root = tempfile::tempdir().unwrap();
    let mut reg = WorldRegistry::new();
    let old = reg
        .register_prior_identity_json("demo", FIXTURE_0_21_0)
        .unwrap()
        .to_string();
    let changed = in_process(&mut reg, CHANGED);
    let r = with_worlds(root.path(), &reg, OPEN, |s| {
        s.decide(
            &transition(&old, &changed, "up-sc", "compiler_upgrade"),
            &service(),
        )
    });
    assert!(matches!(r, Err(CepAuthorityError::Transition(_))), "{r:?}");
}

#[test]
fn a_prior_identity_is_refused_as_a_transition_target() {
    let root = tempfile::tempdir().unwrap();
    let (reg, old, new) = upgrade_pair();
    let r = with_worlds(root.path(), &reg, OPEN, |s| {
        s.decide(
            &transition(&new, &old, "up-back", "compiler_upgrade"),
            &service(),
        )
    });
    assert!(
        matches!(&r, Err(CepAuthorityError::Transition(m)) if m.contains("prior identity")),
        "{r:?}"
    );
}

#[test]
fn a_governed_action_in_a_prior_identity_is_refused_and_the_same_action_in_the_current_world_is_allowed(
) {
    let root = tempfile::tempdir().unwrap();
    let (reg, old, new) = upgrade_pair();
    let in_prior = envelope(&old, "act-old", "action", None, &[]);
    let r = with_worlds(root.path(), &reg, OPEN, |s| s.decide(&in_prior, &service()));
    match &r {
        Err(e @ CepAuthorityError::World(m)) => {
            assert!(m.contains("prior identity"), "{m}");
            assert_eq!(e.class(), "cep_world_refused");
        }
        other => panic!("expected a world refusal, got {other:?}"),
    }
    let in_current = envelope(&new, "act-new", "action", None, &[]);
    let d = with_worlds(root.path(), &reg, OPEN, |s| {
        s.decide(&in_current, &service())
    })
    .unwrap();
    assert_eq!(decision(&d), "allow");
}

/// Work decided in the 0.21.0 world can be cited in the linked (0.21.2) world only through the allowed upgrade.
#[test]
fn lineage_crosses_the_upgrade_only_through_the_allowed_compiler_upgrade() {
    let root = tempfile::tempdir().unwrap();
    // While 0.21.0 was still the current world, work was decided in it.
    let mut before = WorldRegistry::new();
    let old_current = before
        .register_identity_json("demo", FIXTURE_0_21_0)
        .unwrap()
        .to_string();
    assert_eq!(old_current, OLD_REF);
    let new_current = in_process(&mut before, BASE);
    assert_eq!(new_current, NEW_REF);
    let earlier = with_worlds(root.path(), &before, OPEN, |s| {
        s.decide(
            &envelope(&old_current, "lin-a", "action", None, &[]),
            &service(),
        )
    })
    .unwrap();
    assert_eq!(decision(&earlier), "allow");

    // After the upgrade, 0.21.0 is a prior identity.
    let (after, old, new) = upgrade_pair();
    let carried = |transition_ref: Option<&str>| {
        let mut req = envelope(
            &new,
            "lin-b",
            "action",
            None,
            &["env-authority_request-lin-a"],
        );
        if let Some(t) = transition_ref {
            req["extensions"]["godspeed.authority_request"]["transition_ref"] = json!(t);
        }
        req
    };
    let no_licence = with_worlds(root.path(), &after, OPEN, |s| {
        s.decide(&carried(None), &service())
    });
    assert!(
        matches!(&no_licence, Err(CepAuthorityError::Transition(_))),
        "{no_licence:?}"
    );

    let up = with_worlds(root.path(), &after, OPEN, |s| {
        s.decide(
            &transition(&old, &new, "lin-up", "compiler_upgrade"),
            &service(),
        )
    })
    .unwrap();
    let licence = up.committed.entry_ulid().to_string();
    let ok = with_worlds(root.path(), &after, OPEN, |s| {
        s.decide(&carried(Some(&licence)), &service())
    })
    .unwrap();
    assert_eq!(decision(&ok), "allow");
}
