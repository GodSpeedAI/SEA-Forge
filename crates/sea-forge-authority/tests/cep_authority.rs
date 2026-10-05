//! Stage 6: the SEA-Forge half of the CEP authority loop, against the real
//! policy engine, a real world registry and a real append-only ledger.

use sea_forge_authority::cep::{CepAuthorityError, CepAuthorityService, LEDGER_ID};
use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_domainforge::{SeaSourceSet, SourceFile, WorldRegistry};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const SRC: &str = "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";

fn world_registry() -> (WorldRegistry, String) {
    let mut reg = WorldRegistry::new();
    let set = SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![SourceFile {
            uri: "demo.sea".into(),
            sha256: format!("{:x}", Sha256::digest(SRC.as_bytes())),
            content: SRC.into(),
        }],
    };
    let world = reg.register_source_set("demo", &set).unwrap().to_string();
    (reg, world)
}

fn bundle(rules: &str) -> AuthorityPolicyBundle {
    serde_yaml::from_str(&format!(
        "version: \"0.1\"\nallow_degraded: true\nrules:\n{rules}"
    ))
    .unwrap()
}

fn rule(name: &str, kind: &str, extra: &str) -> String {
    format!("  - name: {name}\n    verdict: allow\n    actor_role: service\n    operation_kind: {kind}\n{extra}")
}

fn request(world: &str, op: &str, kind: &str, name: &str, resource: &str) -> Value {
    let id = format!("env-authority_request-{op}");
    json!({
        "envelope_id": id,
        "cep_version": "1.0.0", "envelope_version": "1.0", "envelope_kind": "authority_request",
        "created_at": "2026-10-04T12:00:00Z", "created_by": "cognate",
        "scope": {"evaluation_context": "cognate-authority_request", "world_ref": world},
        "boundary_record": {
            "scope": "authority request for one Cognate operation",
            "included_sections": ["authority", "references", "states", "transformations"],
            "excluded_sections": [], "known_omissions": ["payload content is not carried"],
            "unknowns": [], "redactions": [], "compression_notes": [],
            "out_of_scope_entities": [], "classification": "internal", "limitations": []
        },
        "completeness_status": "partial", "omission_status": "marked",
        "provenance_refs": [format!("prov-{op}")],
        "validation_status": "valid", "lineage_refs": [],
        "provenance": [{
            "provenance_id": format!("prov-{op}"), "source_system_refs": ["cognate"],
            "producer_refs": ["cognate"], "production_method": "governed_operation",
            "created_at": "2026-10-04T12:00:00Z"
        }],
        "authority": [{"authority_id": "auth-1", "subject_actor_ref": "actor:alice"}],
        "transformations": [{"transformation_id": "tr-1", "transformation_type": "capability_invocation"}],
        "references": [{
            "ref_id": "ref-resource-1", "ref_type": "artifact_id", "target_uri_or_id": resource,
            "availability_status": "available", "integrity_status": "verifiable"
        }],
        "states": [{"state_id": "st-1", "state_type": "declared_state", "subject_ref": "run:1"}],
        "extensions": {
            "cep.profile": {"profile_id": "godspeed.authority_request", "profile_version": "1.0.0"},
            "godspeed.authority_request": {
                "operation_id": op, "correlation_id": "corr-1",
                "operation_kind": kind, "operation_name": name, "resource_id": resource
            }
        }
    })
}

fn caller() -> Actor {
    Actor {
        actor_id: "cognate-service".into(),
        role: ActorRole::Service,
    }
}

struct Fixture {
    root: tempfile::TempDir,
    bundle: AuthorityPolicyBundle,
    engine: PolicyAuthorityEngine,
    worlds: WorldRegistry,
    world: String,
}

fn fixture(rules: &str) -> Fixture {
    let (worlds, world) = world_registry();
    let bundle = bundle(rules);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    Fixture {
        root: tempfile::tempdir().unwrap(),
        bundle,
        engine,
        worlds,
        world,
    }
}

impl Fixture {
    fn service(&self) -> CepAuthorityService<'_> {
        CepAuthorityService {
            engine: &self.engine,
            bundle: &self.bundle,
            worlds: &self.worlds,
            root: self.root.path(),
            approval_ttl: chrono::Duration::hours(24),
        }
    }
    fn ledger_dir_exists(&self) -> bool {
        self.root.path().join(LEDGER_ID).exists()
            || walk(self.root.path()).iter().any(|p| p.contains(LEDGER_ID))
    }
}

fn walk(dir: &std::path::Path) -> Vec<String> {
    let mut out = vec![];
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            out.push(e.path().to_string_lossy().into_owned());
            if e.path().is_dir() {
                out.extend(walk(&e.path()));
            }
        }
    }
    out
}

fn decision_of(d: &sea_forge_authority::cep::CepDecision) -> String {
    d.envelope["extensions"]["godspeed.authority_decision"]["decision"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn allow_is_committed_and_lineage_and_world_are_preserved() {
    let f = fixture(&rule("allow-actions", "cognate_action", ""));
    let req = request(&f.world, "op-1", "action", "run.start", "agent.echo");
    let out = f.service().decide(&req, &caller()).unwrap();
    assert_eq!(decision_of(&out), "allow");
    assert_eq!(out.envelope["envelope_kind"], "authority_decision");
    assert_eq!(
        out.envelope["lineage_refs"],
        json!(["env-authority_request-op-1"])
    );
    assert_eq!(out.envelope["scope"]["world_ref"], f.world);
    assert_eq!(
        out.envelope["extensions"]["godspeed.authority_decision"]["request_ref"],
        "env-authority_request-op-1"
    );
    assert_eq!(out.committed.record_kind(), "authority_decision");
    assert!(f.ledger_dir_exists());
}

#[test]
fn an_operation_surface_without_a_rule_is_denied() {
    // Only actions are permitted; a capability invocation has no rule.
    let f = fixture(&rule("allow-actions", "cognate_action", ""));
    let req = request(&f.world, "op-2", "capability", "tool.ping", "tool.ping");
    assert_eq!(
        decision_of(&f.service().decide(&req, &caller()).unwrap()),
        "deny"
    );
}

#[test]
fn a_role_without_a_rule_is_denied() {
    let f = fixture(&rule("allow-actions", "cognate_action", ""));
    let other = Actor {
        actor_id: "x".into(),
        role: ActorRole::Operator,
    };
    let req = request(&f.world, "op-3", "action", "run.start", "agent.echo");
    assert_eq!(
        decision_of(&f.service().decide(&req, &other).unwrap()),
        "deny"
    );
}

#[test]
fn approval_required_policy_escalates() {
    let f = fixture(&rule(
        "gated",
        "cognate_action",
        "    requires_approval: true\n",
    ));
    let req = request(&f.world, "op-4", "action", "run.start", "agent.echo");
    assert_eq!(
        decision_of(&f.service().decide(&req, &caller()).unwrap()),
        "escalate"
    );
}

#[test]
fn boundary_disposition_keeps_constraints_and_policy_basis() {
    let extra =
        "    disposition: boundary\n    boundary_constraints:\n      timeout_secs: [\"30\"]\n";
    let f = fixture(&rule("bounded", "cognate_action", extra));
    let req = request(&f.world, "op-5", "action", "run.start", "agent.echo");
    let out = f.service().decide(&req, &caller()).unwrap();
    assert_eq!(decision_of(&out), "boundary");
    assert_eq!(
        out.envelope["constraints"][0]["constraint_type"],
        "timeout_secs"
    );
    assert!(
        !out.envelope["extensions"]["godspeed.authority_decision"]["policy_basis"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_boundary_policy_without_constraints_cannot_even_be_loaded() {
    // The decision path also refuses to express a constrained disposition without constraints,
    // but the policy loader already makes that state unreachable.
    let yaml = format!(
        "version: \"0.1\"\nrules:\n{}",
        rule("bounded", "cognate_action", "    disposition: boundary\n")
    );
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(&yaml).unwrap();
    assert!(PolicyAuthorityEngine::new(bundle).is_err());
}

#[test]
fn unknown_world_is_refused_and_nothing_is_committed() {
    let f = fixture(&rule("allow-actions", "cognate_action", ""));
    let ghost = format!("world:demo@sha256:{}", "e".repeat(64));
    let err = f
        .service()
        .decide(
            &request(&ghost, "op-7", "action", "run.start", "a"),
            &caller(),
        )
        .unwrap_err();
    assert!(matches!(err, CepAuthorityError::World(_)), "{err:?}");
    assert!(!f.ledger_dir_exists());
}

#[test]
fn malformed_requests_fail_closed() {
    let f = fixture(&rule("allow-actions", "cognate_action", ""));
    let good = request(&f.world, "op-8", "action", "run.start", "agent.echo");
    let svc = f.service();

    let mut wrong_kind = good.clone();
    wrong_kind["envelope_kind"] = json!("authority_decision");
    assert!(matches!(
        svc.decide(&wrong_kind, &caller()),
        Err(CepAuthorityError::WrongKind(_))
    ));

    let mut wrong_profile = good.clone();
    wrong_profile["extensions"]["cep.profile"]["profile_id"] = json!("godspeed.evidence_packet");
    assert!(matches!(
        svc.decide(&wrong_profile, &caller()),
        Err(CepAuthorityError::WrongProfile(_))
    ));

    let mut bad_kind = good.clone();
    bad_kind["extensions"]["godspeed.authority_request"]["operation_kind"] = json!("shell");
    assert!(matches!(
        svc.decide(&bad_kind, &caller()),
        Err(CepAuthorityError::Malformed(_))
    ));

    let mut no_subject = good.clone();
    no_subject.as_object_mut().unwrap().remove("authority");
    assert!(matches!(
        svc.decide(&no_subject, &caller()),
        Err(CepAuthorityError::Malformed(_))
    ));

    let mut control = good.clone();
    control["extensions"]["godspeed.authority_request"]["resource_id"] = json!("a\u{0007}b");
    assert!(matches!(
        svc.decide(&control, &caller()),
        Err(CepAuthorityError::Malformed(_))
    ));

    let mut huge = good.clone();
    huge["extensions"]["padding"] = json!("x".repeat(70_000));
    assert!(matches!(
        svc.decide(&huge, &caller()),
        Err(CepAuthorityError::Oversize(_))
    ));
    assert!(!f.ledger_dir_exists());
}

#[test]
fn the_envelope_subject_cannot_choose_the_evaluated_role() {
    // The subject claims an operator-sounding identity; the evaluated role is the verified caller's.
    let f = fixture(&rule("allow-actions", "cognate_action", ""));
    let mut req = request(&f.world, "op-9", "action", "run.start", "agent.echo");
    req["authority"][0]["subject_actor_ref"] = json!("actor:operator");
    req["extensions"]["godspeed.authority_request"]["actor_role"] = json!("operator");
    let weak = Actor {
        actor_id: "cognate-service".into(),
        role: ActorRole::Agent,
    };
    assert_ne!(
        decision_of(&f.service().decide(&req, &weak).unwrap()),
        "allow"
    );
}

#[test]
fn a_subject_scoped_rule_allows_only_its_subjects() {
    let f = fixture(&rule(
        "alice-only",
        "cognate_action",
        "    subjects: [\"actor:alice\"]\n",
    ));
    let mut req = request(&f.world, "op-sub-1", "action", "run.start", "agent.echo");
    assert_eq!(
        decision_of(&f.service().decide(&req, &caller()).unwrap()),
        "allow"
    );
    req["extensions"]["godspeed.authority_request"]["operation_id"] = json!("op-sub-2");
    req["envelope_id"] = json!("env-authority_request-op-sub-2");
    req["authority"][0]["subject_actor_ref"] = json!("actor:bob");
    assert_eq!(
        decision_of(&f.service().decide(&req, &caller()).unwrap()),
        "deny"
    );
}

#[test]
fn a_subject_rule_can_precede_a_broader_one() {
    let rules = format!(
        "{}{}",
        rule(
            "alice-escalates",
            "cognate_action",
            "    subjects: [\"actor:alice\"]\n    requires_approval: true\n"
        ),
        rule("everyone-else", "cognate_action", "")
    );
    let f = fixture(&rules);
    let mut req = request(&f.world, "op-pre-1", "action", "run.start", "agent.echo");
    assert_eq!(
        decision_of(&f.service().decide(&req, &caller()).unwrap()),
        "escalate"
    );
    req["extensions"]["godspeed.authority_request"]["operation_id"] = json!("op-pre-2");
    req["envelope_id"] = json!("env-authority_request-op-pre-2");
    req["authority"][0]["subject_actor_ref"] = json!("actor:carol");
    // A different resource: an escalation parks an opaque constraint on its own resource.
    req["extensions"]["godspeed.authority_request"]["resource_id"] = json!("agent.other");
    assert_eq!(
        decision_of(&f.service().decide(&req, &caller()).unwrap()),
        "allow"
    );
}

#[test]
fn subjects_is_rejected_where_it_has_no_meaning() {
    for (kind, extra) in [
        ("write_file", "    subjects: [\"actor:alice\"]\n"),
        ("cognate_action", "    subjects: []\n"),
        ("cognate_action", "    subjects: [\"\"]\n"),
    ] {
        let yaml = format!("version: \"0.1\"\nrules:\n{}", rule("r", kind, extra));
        let bundle: AuthorityPolicyBundle = serde_yaml::from_str(&yaml).unwrap();
        assert!(
            PolicyAuthorityEngine::new(bundle).is_err(),
            "{kind} {extra}"
        );
    }
}

#[test]
fn rules_without_subjects_serialize_exactly_as_before() {
    let b = bundle(&rule("r", "cognate_action", ""));
    assert!(!serde_json::to_string(&b).unwrap().contains("subjects"));
}

// ---- profile conformance, validated by the CEP repository's own validator ----
//
// Gated on `CEP_REPO` (path to a canonical-evaluation-protocol checkout) like the
// DomainForge conformance tests are gated on a binary. Run with:
//   CEP_REPO=~/projects/cep cargo test -p sea-forge-authority --test cep_authority cep_validators

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
fn cep_validators_accept_every_emitted_decision_and_the_request() {
    let Ok(repo) = std::env::var("CEP_REPO") else {
        eprintln!("CEP_REPO not set: skipping profile conformance");
        return;
    };
    let cases = [
        ("allow", rule("r", "cognate_action", "")),
        ("deny", rule("r", "cognate_capability", "")),
        ("escalate", rule("r", "cognate_action", "    requires_approval: true\n")),
        ("boundary", rule("r", "cognate_action", "    disposition: boundary\n    boundary_constraints:\n      timeout_secs: [\"30\"]\n")),
        ("degraded", rule("r", "cognate_action", "    disposition: degraded\n    boundary_constraints:\n      timeout_secs: [\"30\"]\n    compensating_controls: [audit]\n")),
    ];
    for (expected, rules) in cases {
        let f = fixture(&rules);
        let req = request(
            &f.world,
            &format!("op-{expected}"),
            "action",
            "run.start",
            "agent.echo",
        );
        assert_eq!(
            cep_errors(&repo, "authority_request", &req),
            Vec::<String>::new(),
            "request for {expected}"
        );
        let out = f.service().decide(&req, &caller()).unwrap();
        assert_eq!(decision_of(&out), expected);
        assert_eq!(
            cep_errors(&repo, "authority_decision", &out.envelope),
            Vec::<String>::new(),
            "decision {expected}"
        );
    }
}
