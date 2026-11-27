//! #116: an approval on an execution-profile rule is bound to the launch-plan digest it was granted for.

use sea_forge_authority::cep::{CepAuthorityError, CepAuthorityService, CepDecision};
use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_domainforge::{SeaSourceSet, SourceFile, WorldRegistry};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

mod common;
use common::cep_repo;

const SRC: &str = "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";

const D1: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const D2: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";
const POLICY: &str = "version: \"0.1\"\nrules:\n  \
  - name: exec\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_capability\n    requires_approval: true\n    disposition: boundary\n    boundary_constraints:\n      execution_profile: [\"sha256:1111111111111111111111111111111111111111111111111111111111111111\", \"sha256:2222222222222222222222222222222222222222222222222222222222222222\"]\n  \
  - name: approvers\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n";
struct Cell {
    root: tempfile::TempDir,
    worlds: WorldRegistry,
    world: String,
}

fn cell() -> Cell {
    let mut worlds = WorldRegistry::new();
    let set = SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![SourceFile {
            uri: "demo.sea".into(),
            sha256: hex::encode(Sha256::digest(SRC.as_bytes())),
            content: SRC.into(),
        }],
    };
    let world = worlds
        .register_source_set("demo", &set)
        .unwrap()
        .to_string();
    Cell {
        root: tempfile::tempdir().unwrap(),
        worlds,
        world,
    }
}

fn bundle(yaml: &str) -> AuthorityPolicyBundle {
    sea_forge_core::yaml::from_str(yaml).unwrap()
}

/// A fresh engine per call, as the server does: opaque constraints never carry between requests.
fn with<T>(
    cell: &Cell,
    policy: &str,
    ttl: chrono::Duration,
    f: impl FnOnce(&CepAuthorityService<'_>) -> T,
) -> T {
    let bundle = bundle(policy);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let service = CepAuthorityService {
        engine: &engine,
        bundle: &bundle,
        worlds: &cell.worlds,
        root: cell.root.path(),
        approval_ttl: ttl,
        criteria: Default::default(),
    };
    f(&service)
}

fn day() -> chrono::Duration {
    chrono::Duration::hours(24)
}

fn service_actor() -> Actor {
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

fn request(
    world: &str,
    op: &str,
    resource: &str,
    subject: &str,
    approval: Option<&str>,
    lineage: &[&str],
    digest: Option<&str>,
) -> Value {
    let mut ext = json!({
        "operation_id": op, "correlation_id": "corr-1",
        "operation_kind": "capability", "operation_name": "invoke", "resource_id": resource
    });
    if let Some(a) = approval {
        ext["approval_id"] = json!(a);
    }
    if let Some(d) = digest {
        ext["claims"] = json!({ "execution_profile_digest": d });
    }
    json!({
        "envelope_id": format!("env-authority_request-{op}"),
        "cep_version": "1.0.0", "envelope_version": "1.0", "envelope_kind": "authority_request",
        "created_at": "2026-10-04T12:00:00Z", "created_by": "cognate",
        "scope": {"evaluation_context": "cognate-authority_request", "world_ref": world},
        "boundary_record": {"scope": "s", "included_sections": ["authority"], "excluded_sections": [],
            "known_omissions": ["payload not carried"], "unknowns": [], "redactions": [],
            "compression_notes": [], "out_of_scope_entities": [], "classification": "internal", "limitations": []},
        "completeness_status": "partial", "omission_status": "marked",
        "provenance_refs": [format!("prov-{op}")], "validation_status": "valid", "lineage_refs": lineage,
        "provenance": [{"provenance_id": format!("prov-{op}"), "source_system_refs": ["cognate"],
            "producer_refs": ["cognate"], "production_method": "governed_operation", "created_at": "2026-10-04T12:00:00Z"}],
        "authority": [{"authority_id": "auth-1", "subject_actor_ref": subject}],
        "transformations": [{"transformation_id": "tr-1", "transformation_type": "capability_invocation"}],
        "references": [{"ref_id": "r1", "ref_type": "artifact_id", "target_uri_or_id": resource,
            "availability_status": "available", "integrity_status": "verifiable"}],
        "states": [{"state_id": "s1", "state_type": "declared_state", "subject_ref": "run:1"}],
        "extensions": {
            "cep.profile": {"profile_id": "godspeed.authority_request", "profile_version": "1.0.0"},
            "godspeed.authority_request": ext
        }
    })
}

fn ext(d: &CepDecision) -> &Value {
    &d.envelope["extensions"]["godspeed.authority_decision"]
}

fn decide(c: &Cell, req: &Value) -> Result<CepDecision, CepAuthorityError> {
    with(c, POLICY, day(), |s| s.decide(req, &service_actor()))
}

fn approved(c: &Cell, op: &str, digest: Option<&str>) -> String {
    let req = request(&c.world, op, "cap.exec", "actor:alice", None, &[], digest);
    let d = decide(c, &req).unwrap();
    assert_eq!(ext(&d)["decision"], "escalate");
    let id = ext(&d)["approval_id"].as_str().unwrap().to_string();
    with(c, POLICY, day(), |s| {
        s.resolve_approval(&id, true, &operator("op-alice"))
    })
    .unwrap();
    id
}

fn follow_up(
    c: &Cell,
    op: &str,
    orig: &str,
    id: &str,
    digest: Option<&str>,
) -> Result<CepDecision, CepAuthorityError> {
    let lineage = format!("env-authority_request-{orig}");
    let req = request(
        &c.world,
        op,
        "cap.exec",
        "actor:alice",
        Some(id),
        &[&lineage],
        digest,
    );
    decide(c, &req)
}

#[test]
fn an_approval_is_spent_only_on_the_digest_it_was_granted_for() {
    let c = cell();
    let id = approved(&c, "op-a1", Some(D1));
    let allowed = follow_up(&c, "op-a1b", "op-a1", &id, Some(D1)).unwrap();
    assert_eq!(ext(&allowed)["decision"], "boundary");
    assert_eq!(ext(&allowed)["approval_used"], true);
    let constraints = allowed.envelope["constraints"].as_array().unwrap();
    assert_eq!(constraints.len(), 1);
    assert_eq!(constraints[0]["constraint_type"], "execution_profile");
    assert_eq!(constraints[0]["description"], D1);
    assert_eq!(
        allowed.decision.boundary_constraints["execution_profile"],
        vec![D1.to_string()]
    );
}

#[test]
fn a_different_or_missing_digest_cannot_spend_the_approval() {
    for (n, digest) in [("a2", Some(D2)), ("a3", None)] {
        let c = cell();
        let id = approved(&c, &format!("op-{n}"), Some(D1));
        let refused = follow_up(&c, &format!("op-{n}b"), &format!("op-{n}"), &id, digest);
        assert!(
            matches!(refused, Err(CepAuthorityError::Approval(_))),
            "{n}: {refused:?}"
        );
        // The approval was not consumed by the refusal: the right digest still works.
        let ok = follow_up(&c, &format!("op-{n}c"), &format!("op-{n}"), &id, Some(D1)).unwrap();
        assert_eq!(ext(&ok)["decision"], "boundary", "{n}");
    }
}

fn cep_errors(repo: &str, profile: &str, envelope: &Value) -> Vec<String> {
    use std::io::Write;
    let script = "import json,sys; from cep.godspeed import validate_profile, verify_semantics; \
                  e=json.load(sys.stdin); p=sys.argv[1]; \
                  print(json.dumps(validate_profile(e,p)+verify_semantics(e,p)))";
    let mut child = std::process::Command::new("uv")
        .args(["run", "--project", repo, "python", "-c", script, profile])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("uv is required when CEP_REPO is set");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(envelope.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "cep validator failed to run");
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn cep_validators_accept_the_approved_execution_profile_allow() {
    let Some(repo) = cep_repo() else {
        return;
    };
    let c = cell();
    let id = approved(&c, "op-v1", Some(D1));
    let allowed = follow_up(&c, "op-v1b", "op-v1", &id, Some(D1)).unwrap();
    assert_eq!(
        cep_errors(&repo, "authority_decision", &allowed.envelope),
        Vec::<String>::new()
    );
}

const PLAIN_POLICY: &str = "version: \"0.1\"\nrules:\n  \
  - name: gated\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_capability\n    requires_approval: true\n    resources: [\"cap.plain\"]\n  \
  - name: approvers\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n";

#[test]
fn an_approval_granted_without_a_digest_is_refused_for_a_follow_up_that_claims_one() {
    let c = cell();
    let with_plain =
        |f: &dyn Fn(&CepAuthorityService<'_>) -> Result<CepDecision, CepAuthorityError>| {
            with(&c, PLAIN_POLICY, day(), |s| f(s))
        };
    let req = request(
        &c.world,
        "op-n1",
        "cap.plain",
        "actor:alice",
        None,
        &[],
        None,
    );
    let d = with_plain(&|s| s.decide(&req, &service_actor())).unwrap();
    assert_eq!(ext(&d)["decision"], "escalate");
    let id = ext(&d)["approval_id"].as_str().unwrap().to_string();
    with(&c, PLAIN_POLICY, day(), |s| {
        s.resolve_approval(&id, true, &operator("op-alice"))
    })
    .unwrap();

    let lineage = "env-authority_request-op-n1";
    let claiming = request(
        &c.world,
        "op-n1b",
        "cap.plain",
        "actor:alice",
        Some(&id),
        &[lineage],
        Some(D1),
    );
    let refused = with_plain(&|s| s.decide(&claiming, &service_actor()));
    assert!(matches!(refused, Err(CepAuthorityError::Approval(_))));
    // Not consumed: the same approval still serves a follow-up with no claim.
    let plain = request(
        &c.world,
        "op-n1c",
        "cap.plain",
        "actor:alice",
        Some(&id),
        &[lineage],
        None,
    );
    let ok = with_plain(&|s| s.decide(&plain, &service_actor())).unwrap();
    assert_eq!(ext(&ok)["decision"], "allow");
    assert_eq!(ext(&ok)["approval_used"], true);
}

const REPO_POLICY: &str = "version: \"0.1\"\nrules:\n  \
  - name: repo\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_capability\n    requires_approval: true\n    disposition: boundary\n    boundary_constraints:\n      repository: [\"org/repo\"]\n  \
  - name: approvers\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n";

#[test]
fn an_approved_allow_on_any_boundary_rule_exports_its_constraints() {
    let c = cell();
    let req = request(
        &c.world,
        "op-r1",
        "cap.repo",
        "actor:alice",
        None,
        &[],
        None,
    );
    let d = with(&c, REPO_POLICY, day(), |s| s.decide(&req, &service_actor())).unwrap();
    let id = ext(&d)["approval_id"].as_str().unwrap().to_string();
    with(&c, REPO_POLICY, day(), |s| {
        s.resolve_approval(&id, true, &operator("op-alice"))
    })
    .unwrap();
    let follow = request(
        &c.world,
        "op-r1b",
        "cap.repo",
        "actor:alice",
        Some(&id),
        &["env-authority_request-op-r1"],
        None,
    );
    let ok = with(&c, REPO_POLICY, day(), |s| {
        s.decide(&follow, &service_actor())
    })
    .unwrap();
    assert_eq!(ext(&ok)["decision"], "boundary");
    let constraints = ok.envelope["constraints"].as_array().unwrap();
    assert_eq!(constraints[0]["constraint_type"], "repository");
    assert_eq!(constraints[0]["description"], "org/repo");
}
