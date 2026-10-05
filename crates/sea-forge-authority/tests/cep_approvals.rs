//! Stage 7, SEA-Forge half: an escalation becomes an approval that is bound to one operation, single-use,
//! expiring, resolved only by an actor policy allows, and revalidated against current policy on use.

use sea_forge_authority::cep::{CepAuthorityError, CepAuthorityService, CepDecision};
use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_domainforge::{SeaSourceSet, SourceFile, WorldRegistry};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const SRC: &str = "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";

const GATED: &str = "version: \"0.1\"\nrules:\n  \
  - name: gated\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_action\n    requires_approval: true\n  \
  - name: approvers\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n";
const DENYING: &str = "version: \"0.1\"\nrules:\n  \
  - name: now-denied\n    verdict: deny\n    actor_role: service\n    operation_kind: cognate_action\n";
const OPEN: &str = "version: \"0.1\"\nrules:\n  \
  - name: open\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_action\n";

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
            sha256: format!("{:x}", Sha256::digest(SRC.as_bytes())),
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
    serde_yaml::from_str(yaml).unwrap()
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
) -> Value {
    let mut ext = json!({
        "operation_id": op, "correlation_id": "corr-1",
        "operation_kind": "action", "operation_name": "run.start", "resource_id": resource
    });
    if let Some(a) = approval {
        ext["approval_id"] = json!(a);
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
        "transformations": [{"transformation_id": "tr-1", "transformation_type": "action_invocation"}],
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

fn escalate(cell: &Cell, op: &str) -> (String, CepDecision) {
    let req = request(&cell.world, op, "agent.echo", "actor:alice", None, &[]);
    let d = with(cell, GATED, day(), |s| s.decide(&req, &service_actor())).unwrap();
    assert_eq!(ext(&d)["decision"], "escalate");
    (
        ext(&d)["approval_id"]
            .as_str()
            .expect("an approvable escalation issues an approval id")
            .to_string(),
        d,
    )
}

fn revalidate(
    cell: &Cell,
    policy: &str,
    op: &str,
    approval: &str,
    orig_op: &str,
) -> Result<CepDecision, CepAuthorityError> {
    let orig = format!("env-authority_request-{orig_op}");
    let req = request(
        &cell.world,
        op,
        "agent.echo",
        "actor:alice",
        Some(approval),
        &[&orig],
    );
    with(cell, policy, day(), |s| s.decide(&req, &service_actor()))
}

#[test]
fn an_escalation_issues_an_approval_that_starts_pending() {
    let c = cell();
    let (id, d) = escalate(&c, "op-1");
    assert_eq!(id, "apr-op-1");
    assert!(ext(&d)["approval_expires_at"].is_string());
    let listed = with(&c, GATED, day(), |s| s.list_approvals()).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].state.as_str(), "pending");
    assert_eq!(listed[0].requester, "cognate-service");
}

#[test]
fn approve_then_revalidate_allows_once_with_full_lineage() {
    let c = cell();
    let (id, _) = escalate(&c, "op-2");
    // Pending: not yet usable.
    assert!(
        matches!(revalidate(&c, GATED, "op-2b", &id, "op-2"), Err(CepAuthorityError::Approval(m)) if m.contains("pending"))
    );

    let resolved = with(&c, GATED, day(), |s| {
        s.resolve_approval(&id, true, &operator("op-alice"))
    })
    .unwrap();
    assert_eq!(resolved["state"], "approved");
    assert_eq!(resolved["resolved_by"], "op-alice");

    let allowed = revalidate(&c, GATED, "op-2c", &id, "op-2").unwrap();
    assert_eq!(ext(&allowed)["decision"], "allow");
    assert_eq!(ext(&allowed)["approval_used"], true);
    assert_eq!(ext(&allowed)["approved_by"], "op-alice");
    assert_eq!(
        allowed.envelope["lineage_refs"],
        json!(["env-authority_request-op-2c", "env-authority_request-op-2"])
    );

    // Single use: the same approval cannot authorize anything again.
    assert!(
        matches!(revalidate(&c, GATED, "op-2d", &id, "op-2"), Err(CepAuthorityError::Approval(m)) if m.contains("used"))
    );
    let standing = with(&c, GATED, day(), |s| s.list_approvals()).unwrap();
    assert_eq!(standing[0].state.as_str(), "used");
}

#[test]
fn a_rejected_approval_never_authorizes() {
    let c = cell();
    let (id, _) = escalate(&c, "op-3");
    with(&c, GATED, day(), |s| {
        s.resolve_approval(&id, false, &operator("op-alice"))
    })
    .unwrap();
    assert!(
        matches!(revalidate(&c, GATED, "op-3b", &id, "op-3"), Err(CepAuthorityError::Approval(m)) if m.contains("rejected"))
    );
}

#[test]
fn an_approval_is_resolved_once() {
    let c = cell();
    let (id, _) = escalate(&c, "op-4");
    with(&c, GATED, day(), |s| {
        s.resolve_approval(&id, true, &operator("op-alice"))
    })
    .unwrap();
    assert!(matches!(
        with(&c, GATED, day(), |s| s.resolve_approval(&id, false, &operator("op-bob"))),
        Err(CepAuthorityError::Approval(m)) if m.contains("not pending")
    ));
}

#[test]
fn the_requester_cannot_approve_their_own_request() {
    let c = cell();
    let (id, _) = escalate(&c, "op-5");
    // Same identity as the requester, even with a role that policy would otherwise allow.
    let same = Actor {
        actor_id: "cognate-service".into(),
        role: ActorRole::Operator,
    };
    assert!(matches!(
        with(&c, GATED, day(), |s| s.resolve_approval(&id, true, &same)),
        Err(CepAuthorityError::Approval(_))
    ));
    assert_eq!(
        with(&c, GATED, day(), |s| s.list_approvals()).unwrap()[0]
            .state
            .as_str(),
        "pending"
    );
}

#[test]
fn only_an_actor_policy_allows_can_resolve() {
    let c = cell();
    let (id, _) = escalate(&c, "op-6");
    // The service role has no approval_resolution rule.
    assert!(matches!(
        with(&c, GATED, day(), |s| s.resolve_approval(
            &id,
            true,
            &service_actor()
        )),
        Err(CepAuthorityError::Approval(_))
    ));
    assert_eq!(
        with(&c, GATED, day(), |s| s.list_approvals()).unwrap()[0]
            .state
            .as_str(),
        "pending"
    );
}

#[test]
fn an_approval_cannot_be_used_for_a_different_operation_subject_or_requester() {
    let c = cell();
    let (id, _) = escalate(&c, "op-7");
    with(&c, GATED, day(), |s| {
        s.resolve_approval(&id, true, &operator("op-alice"))
    })
    .unwrap();
    let orig = "env-authority_request-op-7";

    let other_resource = request(
        &c.world,
        "op-7b",
        "agent.other",
        "actor:alice",
        Some(&id),
        &[orig],
    );
    assert!(
        matches!(with(&c, GATED, day(), |s| s.decide(&other_resource, &service_actor())), Err(CepAuthorityError::Approval(m)) if m.contains("different"))
    );

    let other_subject = request(
        &c.world,
        "op-7c",
        "agent.echo",
        "actor:mallory",
        Some(&id),
        &[orig],
    );
    assert!(
        matches!(with(&c, GATED, day(), |s| s.decide(&other_subject, &service_actor())), Err(CepAuthorityError::Approval(m)) if m.contains("different"))
    );

    let other_requester = Actor {
        actor_id: "someone-else".into(),
        role: ActorRole::Service,
    };
    let ok_shape = request(
        &c.world,
        "op-7d",
        "agent.echo",
        "actor:alice",
        Some(&id),
        &[orig],
    );
    assert!(
        matches!(with(&c, GATED, day(), |s| s.decide(&ok_shape, &other_requester)), Err(CepAuthorityError::Approval(m)) if m.contains("different"))
    );

    let no_lineage = request(
        &c.world,
        "op-7e",
        "agent.echo",
        "actor:alice",
        Some(&id),
        &[],
    );
    assert!(
        matches!(with(&c, GATED, day(), |s| s.decide(&no_lineage, &service_actor())), Err(CepAuthorityError::Approval(m)) if m.contains("lineage"))
    );

    // None of those attempts consumed it: the genuine operation still works.
    assert_eq!(
        ext(&revalidate(&c, GATED, "op-7f", &id, "op-7").unwrap())["decision"],
        "allow"
    );
}

#[test]
fn unknown_and_forged_approvals_are_refused() {
    let c = cell();
    let (_real, _) = escalate(&c, "op-8");
    for bogus in ["apr-nothing", "apr-op-8-x", "op-8", "../apr-op-8"] {
        assert!(
            matches!(
                revalidate(&c, GATED, "op-8b", bogus, "op-8"),
                Err(CepAuthorityError::Approval(_))
            ),
            "{bogus}"
        );
    }
}

#[test]
fn an_approval_never_overrides_a_deny() {
    let c = cell();
    let (id, _) = escalate(&c, "op-9");
    with(&c, GATED, day(), |s| {
        s.resolve_approval(&id, true, &operator("op-alice"))
    })
    .unwrap();
    // Policy tightened after approval: the revalidation is a deny, and the approval is not consumed.
    let denied = revalidate(&c, DENYING, "op-9b", &id, "op-9").unwrap();
    assert_eq!(ext(&denied)["decision"], "deny");
    assert_eq!(
        with(&c, GATED, day(), |s| s.list_approvals()).unwrap()[0]
            .state
            .as_str(),
        "approved"
    );
}

#[test]
fn when_policy_now_allows_outright_the_approval_is_not_spent() {
    let c = cell();
    let (id, _) = escalate(&c, "op-10");
    with(&c, GATED, day(), |s| {
        s.resolve_approval(&id, true, &operator("op-alice"))
    })
    .unwrap();
    let allowed = revalidate(&c, OPEN, "op-10b", &id, "op-10").unwrap();
    assert_eq!(ext(&allowed)["decision"], "allow");
    assert!(ext(&allowed).get("approval_used").is_none());
    assert_eq!(
        with(&c, GATED, day(), |s| s.list_approvals()).unwrap()[0]
            .state
            .as_str(),
        "approved"
    );
}

#[test]
fn an_unacted_escalation_expires_and_cannot_be_approved() {
    let c = cell();
    let req = request(&c.world, "op-11", "agent.echo", "actor:alice", None, &[]);
    let d = with(&c, GATED, chrono::Duration::zero(), |s| {
        s.decide(&req, &service_actor())
    })
    .unwrap();
    let id = ext(&d)["approval_id"].as_str().unwrap().to_string();
    assert!(matches!(
        with(&c, GATED, chrono::Duration::zero(), |s| s.resolve_approval(&id, true, &operator("op-alice"))),
        Err(CepAuthorityError::Approval(m)) if m.contains("expired")
    ));
}

#[test]
fn an_approval_lapses_with_its_window_even_after_being_granted() {
    let c = cell();
    let ttl = chrono::Duration::seconds(2);
    let req = request(&c.world, "op-12", "agent.echo", "actor:alice", None, &[]);
    let d = with(&c, GATED, ttl, |s| s.decide(&req, &service_actor())).unwrap();
    let id = ext(&d)["approval_id"].as_str().unwrap().to_string();
    with(&c, GATED, ttl, |s| {
        s.resolve_approval(&id, true, &operator("op-alice"))
    })
    .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(2_300));
    let orig = "env-authority_request-op-12";
    let late = request(
        &c.world,
        "op-12b",
        "agent.echo",
        "actor:alice",
        Some(&id),
        &[orig],
    );
    assert!(
        matches!(with(&c, GATED, ttl, |s| s.decide(&late, &service_actor())), Err(CepAuthorityError::Approval(m)) if m.contains("expired"))
    );
}

#[test]
fn an_escalation_that_approval_cannot_fix_issues_no_approval() {
    // An agent identity with no sponsor escalates as `identity_unresolved`: nobody can approve that away.
    let c = cell();
    let agent = Actor {
        actor_id: "agent-x".into(),
        role: ActorRole::Agent,
    };
    let policy = "version: \"0.1\"\nrules:\n  - name: any\n    verdict: allow\n    actor_role: agent\n    operation_kind: cognate_action\n";
    let req = request(&c.world, "op-13", "agent.echo", "actor:alice", None, &[]);
    let d = with(&c, policy, day(), |s| s.decide(&req, &agent)).unwrap();
    assert_eq!(ext(&d)["decision"], "escalate");
    assert!(ext(&d).get("approval_id").is_none(), "{}", ext(&d));
}

// Profile conformance by the CEP repository's own validator (gated on CEP_REPO, like cep_authority.rs).
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
fn cep_validators_accept_the_escalation_and_the_approved_allow() {
    let Ok(repo) = std::env::var("CEP_REPO") else {
        eprintln!("CEP_REPO not set: skipping profile conformance");
        return;
    };
    let c = cell();
    let (id, escalated) = escalate(&c, "op-v");
    assert_eq!(
        cep_errors(&repo, "authority_decision", &escalated.envelope),
        Vec::<String>::new(),
        "escalate"
    );
    with(&c, GATED, day(), |s| {
        s.resolve_approval(&id, true, &operator("op-alice"))
    })
    .unwrap();
    let allowed = revalidate(&c, GATED, "op-vb", &id, "op-v").unwrap();
    assert_eq!(
        cep_errors(&repo, "authority_decision", &allowed.envelope),
        Vec::<String>::new(),
        "approved allow"
    );
    let orig = "env-authority_request-op-v";
    let req = request(
        &c.world,
        "op-vc",
        "agent.echo",
        "actor:alice",
        Some(&id),
        &[orig],
    );
    assert_eq!(
        cep_errors(&repo, "authority_request", &req),
        Vec::<String>::new(),
        "revalidation request"
    );
}
