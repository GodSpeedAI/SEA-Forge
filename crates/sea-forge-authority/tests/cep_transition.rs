//! Stage 10: a world transition is the one explicit, governed, recorded way to move work from one immutable
//! semantic world to another. SEA-Forge recomputes the facts, a move across a semantic change always needs a
//! human, and an allowed move is a ledger fact that validates against cep's `world-transition.v1`.

use sea_forge_authority::cep::{CepAuthorityError, CepAuthorityService, CepDecision};
use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_domainforge::{SeaSourceSet, SourceFile, WorldRegistry};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const BASE: &str = "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";
/// Same meaning, different bytes: a comment-only edit changes the world digest and not the semantic closure.
const EDITED: &str = "// reviewed\n@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";
/// A different model.
const CHANGED: &str =
    "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\nentity \"Pump\" { key id: uuid }\n";
const OTHER: &str = "@namespace \"t\"\nentity \"Valve\" { key id: uuid }\n";

const OPEN: &str = "version: \"0.1\"\nrules:\n  \
  - name: open\n    verdict: allow\n    actor_role: service\n    operation_kind: world_transition\n  \
  - name: approvers\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n";
const DENYING: &str = "version: \"0.1\"\nrules:\n  \
  - name: no\n    verdict: deny\n    actor_role: service\n    operation_kind: world_transition\n";
const SILENT: &str = "version: \"0.1\"\nrules:\n  \
  - name: unrelated\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_action\n";

struct Cell {
    root: tempfile::TempDir,
    worlds: WorldRegistry,
    base: String,
    edited: String,
    changed: String,
    other: String,
}

fn register(worlds: &mut WorldRegistry, name: &str, src: &str) -> String {
    let set = SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![SourceFile {
            uri: "demo.sea".into(),
            sha256: format!("{:x}", Sha256::digest(src.as_bytes())),
            content: src.into(),
        }],
    };
    worlds.register_source_set(name, &set).unwrap().to_string()
}

fn cell() -> Cell {
    let mut worlds = WorldRegistry::new();
    let base = register(&mut worlds, "demo", BASE);
    let edited = register(&mut worlds, "demo", EDITED);
    let changed = register(&mut worlds, "demo", CHANGED);
    let other = register(&mut worlds, "demo", OTHER);
    Cell {
        root: tempfile::tempdir().unwrap(),
        worlds,
        base,
        edited,
        changed,
        other,
    }
}

fn with<T>(c: &Cell, policy: &str, f: impl FnOnce(&CepAuthorityService<'_>) -> T) -> T {
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(policy).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let service = CepAuthorityService {
        engine: &engine,
        bundle: &bundle,
        worlds: &c.worlds,
        root: c.root.path(),
        approval_ttl: chrono::Duration::hours(24),
        criteria: Default::default(),
    };
    f(&service)
}

fn service() -> Actor {
    Actor {
        actor_id: "cognate-service".into(),
        role: ActorRole::Service,
    }
}
fn operator(id: &str) -> Actor {
    Actor {
        actor_id: id.into(),
        role: ActorRole::Operator,
    }
}

/// A transition request pinned to `source`, asking to move to `target`.
fn request(
    source: &str,
    op: &str,
    claim: Value,
    resource: &str,
    approval: Option<&str>,
    lineage: &[&str],
) -> Value {
    let mut ext = json!({
        "operation_id": op, "correlation_id": "corr-1",
        "operation_kind": "world_transition", "operation_name": "transition", "resource_id": resource
    });
    if let Some(a) = approval {
        ext["approval_id"] = json!(a);
    }
    json!({
        "envelope_id": format!("env-authority_request-{op}"),
        "cep_version": "1.0.0", "envelope_version": "1.0", "envelope_kind": "authority_request",
        "created_at": "2026-10-05T12:00:00Z", "created_by": "cognate",
        "scope": {"evaluation_context": "cognate-authority_request", "world_ref": source},
        "boundary_record": {"scope": "s", "included_sections": ["authority"], "excluded_sections": [],
            "known_omissions": ["payload not carried"], "unknowns": [], "redactions": [],
            "compression_notes": [], "out_of_scope_entities": [], "classification": "internal", "limitations": []},
        "completeness_status": "partial", "omission_status": "marked",
        "provenance_refs": [format!("prov-{op}")], "validation_status": "valid", "lineage_refs": lineage,
        "provenance": [{"provenance_id": format!("prov-{op}"), "source_system_refs": ["cognate"],
            "producer_refs": ["cognate"], "production_method": "governed_operation", "created_at": "2026-10-05T12:00:00Z"}],
        "authority": [{"authority_id": "auth-1", "subject_actor_ref": "actor:alice"}],
        "transformations": [{"transformation_id": "tr-1", "transformation_type": "action_invocation"}],
        "references": [{"ref_id": "r1", "ref_type": "artifact_id", "target_uri_or_id": resource,
            "availability_status": "available", "integrity_status": "verifiable"}],
        "states": [{"state_id": "s1", "state_type": "declared_state", "subject_ref": "run:1"}],
        "extensions": {
            "cep.profile": {"profile_id": "godspeed.authority_request", "profile_version": "1.0.0"},
            "godspeed.authority_request": ext,
            "godspeed.world_transition": claim
        }
    })
}

fn claim(target: &str, kind: &str) -> Value {
    json!({"target_world_ref": target, "transition_kind": kind, "reason": "adopt the reviewed model", "compatibility": "compatible"})
}

fn mv(
    c: &Cell,
    policy: &str,
    op: &str,
    source: &str,
    target: &str,
    kind: &str,
) -> Result<CepDecision, CepAuthorityError> {
    let req = request(source, op, claim(target, kind), target, None, &[]);
    with(c, policy, |s| s.decide(&req, &service()))
}

fn decision(d: &CepDecision) -> &str {
    d.envelope["extensions"]["godspeed.authority_decision"]["decision"]
        .as_str()
        .unwrap()
}

#[test]
fn the_fixture_worlds_are_what_the_tests_assume() {
    let c = cell();
    let (_, a) = c.worlds.require(&c.base).unwrap();
    let (_, b) = c.worlds.require(&c.edited).unwrap();
    let (_, d) = c.worlds.require(&c.changed).unwrap();
    assert_ne!(
        c.base, c.edited,
        "a comment edit still mints a new world_ref"
    );
    assert_eq!(a.semantic_closure_hash, b.semantic_closure_hash);
    assert_ne!(a.semantic_closure_hash, d.semantic_closure_hash);
}

#[test]
fn a_meaning_preserving_move_follows_policy_and_is_recorded() {
    let c = cell();
    let d = mv(&c, OPEN, "t-1", &c.base, &c.edited, "source_edit_only").unwrap();
    assert_eq!(decision(&d), "allow");
    let facts = with(&c, OPEN, |s| s.transitions()).unwrap();
    assert_eq!(facts.len(), 1);
    let rec = &facts[0].record;
    assert_eq!(rec["source_world_ref"], c.base);
    assert_eq!(rec["target_world_ref"], c.edited);
    assert_eq!(rec["semantic_closure_equal"], true);
    assert_eq!(rec["compatibility"], "compatible");
    assert_eq!(rec["authority_decision_ref"], facts[0].decision_id);
    assert_eq!(facts[0].approval_id, None);
}

#[test]
fn a_move_across_a_semantic_change_is_floored_to_an_escalation_even_if_policy_allows() {
    let c = cell();
    let d = mv(&c, OPEN, "t-2", &c.base, &c.changed, "semantic_change").unwrap();
    assert_eq!(decision(&d), "escalate");
    assert_eq!(d.decision.reason, "world_transition_requires_approval");
    assert!(d.envelope["extensions"]["godspeed.authority_decision"]["approval_id"].is_string());
    assert!(
        with(&c, OPEN, |s| s.transitions()).unwrap().is_empty(),
        "nothing moved: an escalation is not a transition"
    );
}

#[test]
fn an_approved_move_is_allowed_once_and_names_its_approval() {
    let c = cell();
    let first = mv(&c, OPEN, "t-3", &c.base, &c.changed, "semantic_change").unwrap();
    let approval = first.envelope["extensions"]["godspeed.authority_decision"]["approval_id"]
        .as_str()
        .unwrap()
        .to_string();
    with(&c, OPEN, |s| {
        s.resolve_approval(&approval, true, &operator("op-alice"))
    })
    .unwrap();

    let req = request(
        &c.base,
        "t-3b",
        claim(&c.changed, "semantic_change"),
        &c.changed,
        Some(&approval),
        &["env-authority_request-t-3"],
    );
    let allowed = with(&c, OPEN, |s| s.decide(&req, &service())).unwrap();
    assert_eq!(decision(&allowed), "allow");
    let facts = with(&c, OPEN, |s| s.transitions()).unwrap();
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].approval_id.as_deref(), Some(approval.as_str()));

    // Single use.
    let again = request(
        &c.base,
        "t-3c",
        claim(&c.changed, "semantic_change"),
        &c.changed,
        Some(&approval),
        &["env-authority_request-t-3"],
    );
    assert!(matches!(
        with(&c, OPEN, |s| s.decide(&again, &service())),
        Err(CepAuthorityError::Approval(_))
    ));
    assert_eq!(with(&c, OPEN, |s| s.transitions()).unwrap().len(), 1);
}

#[test]
fn an_approval_for_one_target_cannot_be_spent_on_another() {
    let c = cell();
    let first = mv(&c, OPEN, "t-4", &c.base, &c.changed, "semantic_change").unwrap();
    let approval = first.envelope["extensions"]["godspeed.authority_decision"]["approval_id"]
        .as_str()
        .unwrap()
        .to_string();
    with(&c, OPEN, |s| {
        s.resolve_approval(&approval, true, &operator("op-alice"))
    })
    .unwrap();

    let req = request(
        &c.base,
        "t-4b",
        claim(&c.other, "semantic_change"),
        &c.other,
        Some(&approval),
        &["env-authority_request-t-4"],
    );
    let err = with(&c, OPEN, |s| s.decide(&req, &service())).unwrap_err();
    assert!(matches!(err, CepAuthorityError::Approval(_)), "{err}");
    assert!(with(&c, OPEN, |s| s.transitions()).unwrap().is_empty());
}

#[test]
fn the_requester_cannot_approve_their_own_move() {
    let c = cell();
    let first = mv(&c, OPEN, "t-5", &c.base, &c.changed, "semantic_change").unwrap();
    let approval = first.envelope["extensions"]["godspeed.authority_decision"]["approval_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(with(&c, OPEN, |s| s.resolve_approval(
        &approval,
        true,
        &service()
    ))
    .is_err());
}

#[test]
fn a_policy_deny_stays_a_deny_and_records_nothing() {
    let c = cell();
    for (op, target, kind) in [
        ("t-6a", c.edited.clone(), "source_edit_only"),
        ("t-6b", c.changed.clone(), "semantic_change"),
    ] {
        let d = mv(&c, DENYING, op, &c.base, &target, kind).unwrap();
        assert_eq!(decision(&d), "deny");
    }
    assert!(with(&c, DENYING, |s| s.transitions()).unwrap().is_empty());
}

#[test]
fn without_a_rule_a_transition_is_not_allowed() {
    let c = cell();
    let d = mv(&c, SILENT, "t-7", &c.base, &c.edited, "source_edit_only").unwrap();
    assert_ne!(decision(&d), "allow");
    assert!(with(&c, SILENT, |s| s.transitions()).unwrap().is_empty());
}

#[test]
fn claims_that_contradict_the_registered_worlds_are_refused() {
    let c = cell();
    // Claimed equal closure across a real change.
    let mut lie = claim(&c.changed, "migration");
    lie["semantic_closure_equal"] = json!(true);
    let req = request(&c.base, "t-8a", lie, &c.changed, None, &[]);
    assert!(matches!(
        with(&c, OPEN, |s| s.decide(&req, &service())),
        Err(CepAuthorityError::Transition(m)) if m.contains("semantic_closure_equal")
    ));
    // Claimed unequal closure across a comment edit.
    let mut lie = claim(&c.edited, "migration");
    lie["semantic_closure_equal"] = json!(false);
    let req = request(&c.base, "t-8b", lie, &c.edited, None, &[]);
    assert!(matches!(
        with(&c, OPEN, |s| s.decide(&req, &service())),
        Err(CepAuthorityError::Transition(_))
    ));
    // A kind that the facts contradict.
    for (target, kind) in [
        (&c.changed, "source_edit_only"),
        (&c.changed, "compiler_upgrade"),
        (&c.edited, "semantic_change"),
    ] {
        let r = mv(&c, OPEN, "t-8c", &c.base, target, kind);
        assert!(
            matches!(r, Err(CepAuthorityError::Transition(_))),
            "{kind}: {r:?}"
        );
    }
    assert!(with(&c, OPEN, |s| s.transitions()).unwrap().is_empty());
}

#[test]
fn a_transition_must_move_between_two_registered_different_worlds() {
    let c = cell();
    let same = mv(&c, OPEN, "t-9a", &c.base, &c.base, "migration");
    assert!(matches!(same, Err(CepAuthorityError::Transition(m)) if m.contains("same world")));

    let unknown = format!("world:demo@sha256:{}", "e".repeat(64));
    let r = mv(&c, OPEN, "t-9b", &c.base, &unknown, "migration");
    assert!(matches!(r, Err(CepAuthorityError::Transition(m)) if m.contains("target world")));

    // An alias is a mutable selector, not a world.
    let r = mv(&c, OPEN, "t-9c", &c.base, "world:demo", "migration");
    assert!(matches!(r, Err(CepAuthorityError::Transition(_))));

    // A source that is not registered fails before any decision.
    let r = mv(&c, OPEN, "t-9d", &unknown, &c.edited, "migration");
    assert!(r.is_err());
}

#[test]
fn the_request_must_bind_its_approval_to_the_target() {
    let c = cell();
    let req = request(
        &c.base,
        "t-10a",
        claim(&c.edited, "source_edit_only"),
        &c.other,
        None,
        &[],
    );
    assert!(matches!(
        with(&c, OPEN, |s| s.decide(&req, &service())),
        Err(CepAuthorityError::Malformed(m)) if m.contains("target")
    ));
    let mut req = request(
        &c.base,
        "t-10b",
        claim(&c.edited, "source_edit_only"),
        &c.edited,
        None,
        &[],
    );
    req["extensions"]["godspeed.authority_request"]["operation_name"] = json!("run.start");
    assert!(matches!(
        with(&c, OPEN, |s| s.decide(&req, &service())),
        Err(CepAuthorityError::Malformed(_))
    ));
}

#[test]
fn compatibility_is_provable_only_when_the_closures_are_equal() {
    let c = cell();
    // A sender claiming `breaking` across a comment edit is overruled by the registered identities.
    let mut breaking = claim(&c.edited, "source_edit_only");
    breaking["compatibility"] = json!("breaking");
    let req = request(&c.base, "t-11a", breaking, &c.edited, None, &[]);
    with(&c, OPEN, |s| s.decide(&req, &service())).unwrap();
    // Across a real change with no claim, compatibility stays unknown and a human is still required.
    let mut none = claim(&c.changed, "semantic_change");
    none.as_object_mut().unwrap().remove("compatibility");
    let req = request(&c.base, "t-11b", none, &c.changed, None, &[]);
    let d = with(&c, OPEN, |s| s.decide(&req, &service())).unwrap();
    assert_eq!(decision(&d), "escalate");
    let approval = d.envelope["extensions"]["godspeed.authority_decision"]["approval_id"]
        .as_str()
        .unwrap()
        .to_string();
    with(&c, OPEN, |s| {
        s.resolve_approval(&approval, true, &operator("op-alice"))
    })
    .unwrap();
    let mut again = claim(&c.changed, "semantic_change");
    again.as_object_mut().unwrap().remove("compatibility");
    let req = request(
        &c.base,
        "t-11c",
        again,
        &c.changed,
        Some(&approval),
        &["env-authority_request-t-11b"],
    );
    with(&c, OPEN, |s| s.decide(&req, &service())).unwrap();
    let facts = with(&c, OPEN, |s| s.transitions()).unwrap();
    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].record["compatibility"], "compatible");
    assert_eq!(facts[1].record["compatibility"], "unknown");
}

#[test]
fn a_replayed_allow_is_one_transition() {
    let c = cell();
    let a = mv(&c, OPEN, "t-12", &c.base, &c.edited, "source_edit_only").unwrap();
    let b = mv(&c, OPEN, "t-12", &c.base, &c.edited, "source_edit_only");
    // A byte-identical replay is idempotent; whatever it answers, the ledger holds one move.
    let _ = (a, b);
    assert_eq!(with(&c, OPEN, |s| s.transitions()).unwrap().len(), 1);
}

#[test]
fn the_record_validates_against_cep_when_the_checkout_is_present() {
    let Ok(cep) = std::env::var("CEP_REPO") else {
        eprintln!("SKIP: set CEP_REPO to validate the record against world-transition.v1");
        return;
    };
    let c = cell();
    mv(&c, OPEN, "t-13", &c.base, &c.edited, "source_edit_only").unwrap();
    let rec = with(&c, OPEN, |s| s.transitions()).unwrap()[0]
        .record
        .clone();
    let script = format!(
        "import json,sys\nsys.path.insert(0, {cep:?} + '/src')\nfrom cep.godspeed import validate_world_transition\nerrs = validate_world_transition(json.load(sys.stdin))\nprint(json.dumps(errs))\n"
    );
    let mut child = std::process::Command::new("uv")
        .args(["run", "--project", &cep, "python", "-c", &script])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("uv runs the cep validator");
    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(rec.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let errs: Value = serde_json::from_slice(&out.stdout).expect("validator prints a JSON list");
    assert_eq!(errs, json!([]), "cep rejected the record: {errs}");
}
