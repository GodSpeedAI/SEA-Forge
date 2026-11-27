//! Stage 6: the SEA-Forge half of the CEP authority loop, against the real
//! policy engine, a real world registry and a real append-only ledger.

use sea_forge_authority::cep::{CepAuthorityError, CepAuthorityService, LEDGER_ID};
use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_domainforge::{SeaSourceSet, SourceFile, WorldRegistry};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

mod common;
use common::cep_repo;

const SRC: &str = "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";

fn world_registry() -> (WorldRegistry, String) {
    let mut reg = WorldRegistry::new();
    let set = SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![SourceFile {
            uri: "demo.sea".into(),
            sha256: hex::encode(Sha256::digest(SRC.as_bytes())),
            content: SRC.into(),
        }],
    };
    let world = reg.register_source_set("demo", &set).unwrap().to_string();
    (reg, world)
}

fn bundle(rules: &str) -> AuthorityPolicyBundle {
    sea_forge_core::yaml::from_str(&format!(
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
            criteria: Default::default(),
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
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(&yaml).unwrap();
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
        let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(&yaml).unwrap();
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

#[test]
fn a_resource_scoped_escalation_precedes_a_broad_allow() {
    // Named consequential capabilities escalate; every other capability is allowed.
    let rules = format!(
        "{}{}",
        rule(
            "consequential-escalates",
            "cognate_capability",
            "    resources: [\"tool.bash\", \"process.exec\"]\n    requires_approval: true\n"
        ),
        rule("everything-else", "cognate_capability", "")
    );
    let f = fixture(&rules);
    let mut req = request(&f.world, "op-res-1", "capability", "invoke", "tool.bash");
    assert_eq!(
        decision_of(&f.service().decide(&req, &caller()).unwrap()),
        "escalate"
    );
    req["extensions"]["godspeed.authority_request"]["operation_id"] = json!("op-res-2");
    req["envelope_id"] = json!("env-authority_request-op-res-2");
    req["extensions"]["godspeed.authority_request"]["resource_id"] = json!("tool.read");
    assert_eq!(
        decision_of(&f.service().decide(&req, &caller()).unwrap()),
        "allow"
    );
}

#[test]
fn a_resource_scoped_rule_matches_only_its_resources() {
    let f = fixture(&rule(
        "read-only",
        "cognate_capability",
        "    resources: [\"tool.read\"]\n",
    ));
    let mut req = request(&f.world, "op-res-3", "capability", "invoke", "tool.read");
    assert_eq!(
        decision_of(&f.service().decide(&req, &caller()).unwrap()),
        "allow"
    );
    req["extensions"]["godspeed.authority_request"]["operation_id"] = json!("op-res-4");
    req["envelope_id"] = json!("env-authority_request-op-res-4");
    req["extensions"]["godspeed.authority_request"]["resource_id"] = json!("tool.bash");
    assert_eq!(
        decision_of(&f.service().decide(&req, &caller()).unwrap()),
        "deny"
    );
}

#[test]
fn resources_is_rejected_where_it_has_no_meaning() {
    for (kind, extra) in [
        ("write_file", "    resources: [\"tool.bash\"]\n"),
        ("cognate_capability", "    resources: []\n"),
        ("cognate_capability", "    resources: [\"\"]\n"),
    ] {
        let yaml = format!("version: \"0.1\"\nrules:\n{}", rule("r", kind, extra));
        let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(&yaml).unwrap();
        assert!(
            PolicyAuthorityEngine::new(bundle).is_err(),
            "{kind} {extra}"
        );
    }
}

#[test]
fn rules_without_resources_serialize_exactly_as_before() {
    let b = bundle(&rule("r", "cognate_capability", ""));
    assert!(!serde_json::to_string(&b).unwrap().contains("resources"));
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
    let Some(repo) = cep_repo() else {
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

// ---- execution_profile: bind an allow to one exact launch plan (#116) ----

const D1: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const D2: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";
const D3: &str = "sha256:3333333333333333333333333333333333333333333333333333333333333333";

fn profile_rule(name: &str, digests: &[&str], extra: &str) -> String {
    let values = digests
        .iter()
        .map(|d| format!("\"{d}\""))
        .collect::<Vec<_>>()
        .join(", ");
    rule(
        name,
        "cognate_capability",
        &format!("    disposition: boundary\n    boundary_constraints:\n      execution_profile: [{values}]\n{extra}"),
    )
}

fn capability_request(world: &str, op: &str, resource: &str, digest: Option<&str>) -> Value {
    let mut req = request(world, op, "capability", "invoke", resource);
    if let Some(d) = digest {
        req["extensions"]["godspeed.authority_request"]["claims"] =
            json!({ "execution_profile_digest": d });
    }
    req
}

fn load_error(rules: &str) -> bool {
    let bundle: AuthorityPolicyBundle =
        sea_forge_core::yaml::from_str(&format!("version: \"0.1\"\nrules:\n{rules}")).unwrap();
    PolicyAuthorityEngine::new(bundle).is_err()
}

#[test]
fn execution_profile_loads_only_well_formed_on_a_boundary_capability_rule() {
    let lower = "sha256:".to_string() + &"a".repeat(64);
    assert!(!load_error(&profile_rule("ok", &[D1, &lower], "")));
    let nine: Vec<String> = (0..9).map(|i| format!("sha256:{:064x}", i)).collect();
    let nine_refs: Vec<&str> = nine.iter().map(String::as_str).collect();
    assert!(!load_error(&profile_rule("eight", &nine_refs[..8], "")));
    let upper = format!("sha256:{}", "A".repeat(64));
    for (label, digests) in [
        ("empty", vec![]),
        ("malformed", vec!["sha256:abc"]),
        ("no prefix", vec![&D1[7..]]),
        ("uppercase", vec![upper.as_str()]),
        ("duplicate", vec![D1, D1]),
        ("too many", nine_refs.clone()),
    ] {
        assert!(load_error(&profile_rule("bad", &digests, "")), "{label}");
    }
    // Wrong operation kind: only cognate_capability may carry it.
    for kind in [
        "cognate_action",
        "write_file",
        "world_transition",
        "execute_command",
    ] {
        let r = rule(
            "wrong",
            kind,
            &format!("    disposition: boundary\n    boundary_constraints:\n      execution_profile: [\"{D1}\"]\n"),
        );
        assert!(load_error(&r), "{kind}");
    }
    // Not a boundary disposition, or not an allow: refused.
    let plain = rule(
        "plain",
        "cognate_capability",
        &format!("    boundary_constraints:\n      execution_profile: [\"{D1}\"]\n"),
    );
    assert!(load_error(&plain), "non-boundary");
    let denying = profile_rule("deny", &[D1], "").replace("verdict: allow", "verdict: deny");
    assert!(load_error(&denying), "non-allow");
}

#[test]
fn a_matching_digest_is_a_boundary_allow_carrying_exactly_that_digest() {
    let f = fixture(&profile_rule("exec", &[D1, D2], ""));
    let req = capability_request(&f.world, "op-p1", "cap.exec", Some(D2));
    let out = f.service().decide(&req, &caller()).unwrap();
    assert_eq!(decision_of(&out), "boundary");
    assert_eq!(out.decision.verdict, sea_forge_core::types::Verdict::Allow);
    assert_eq!(
        out.decision.boundary_constraints["execution_profile"],
        vec![D2.to_string()]
    );
    // The envelope exports it as a typed constraint whose description is the digest alone.
    let constraints = out.envelope["constraints"].as_array().unwrap();
    assert_eq!(constraints.len(), 1);
    assert_eq!(constraints[0]["constraint_type"], "execution_profile");
    assert_eq!(constraints[0]["description"], D2);
    // The committed decision records the claim the requester asserted.
    let recorded = out.decision.action_request.action.to_string();
    assert!(recorded.contains(D2), "claim digest recorded: {recorded}");
}

#[test]
fn a_different_or_missing_or_malformed_digest_is_denied_with_a_typed_reason() {
    let f = fixture(&profile_rule("exec", &[D1], ""));
    for (op, digest) in [
        ("op-p2", Some(D3)),
        ("op-p3", None),
        ("op-p4", Some("sha256:nope")),
    ] {
        let req = capability_request(&f.world, op, "cap.exec", digest);
        let out = f.service().decide(&req, &caller()).unwrap();
        assert_eq!(decision_of(&out), "deny", "{op}");
        assert!(out
            .decision
            .reason_codes
            .iter()
            .any(|c| c == "execution_profile_mismatch"));
        assert!(out.decision.boundary_constraints.is_empty());
    }
}

#[test]
fn a_less_specific_allow_cannot_stand_in_for_a_profile_rule() {
    // The broad allow comes first and would win by first-match; it must not grant a governed resource.
    let rules = format!(
        "{}{}",
        rule("broad", "cognate_capability", ""),
        profile_rule("exec", &[D1], "    resources: [\"cap.exec\"]\n")
    );
    let f = fixture(&rules);
    for (op, digest) in [("op-p5", Some(D3)), ("op-p6", None)] {
        let out = f
            .service()
            .decide(
                &capability_request(&f.world, op, "cap.exec", digest),
                &caller(),
            )
            .unwrap();
        assert_eq!(decision_of(&out), "deny", "{op}");
    }
    let right = f
        .service()
        .decide(
            &capability_request(&f.world, "op-p7", "cap.exec", Some(D1)),
            &caller(),
        )
        .unwrap();
    assert_eq!(decision_of(&right), "boundary");
    // A resource no profile rule governs is unaffected by the profile rule.
    let other = f
        .service()
        .decide(
            &capability_request(&f.world, "op-p8", "cap.other", None),
            &caller(),
        )
        .unwrap();
    assert_eq!(decision_of(&other), "allow");
}

#[test]
fn an_earlier_deny_still_denies_a_governed_resource() {
    let rules = format!(
        "{}{}",
        rule("deny-all", "cognate_capability", "").replace("verdict: allow", "verdict: deny"),
        profile_rule("exec", &[D1], "")
    );
    let f = fixture(&rules);
    let out = f
        .service()
        .decide(
            &capability_request(&f.world, "op-p9", "cap.exec", Some(D1)),
            &caller(),
        )
        .unwrap();
    assert_eq!(decision_of(&out), "deny");
    assert_eq!(out.decision.matched_rule.as_deref(), Some("deny-all"));
}

#[test]
fn escalation_on_a_profile_rule_exports_no_constraints() {
    let f = fixture(&profile_rule(
        "exec",
        &[D1],
        "    requires_approval: true\n",
    ));
    let out = f
        .service()
        .decide(
            &capability_request(&f.world, "op-p10", "cap.exec", Some(D1)),
            &caller(),
        )
        .unwrap();
    assert_eq!(decision_of(&out), "escalate");
    // As for any escalation, the envelope exports no constraints; nothing is spendable until approved.
    assert!(out.envelope["constraints"]
        .as_array()
        .is_none_or(|constraints| constraints.is_empty()));
    assert_eq!(
        out.decision.verdict,
        sea_forge_core::types::Verdict::Escalate
    );
}

#[test]
fn cep_validators_accept_an_execution_profile_boundary_decision_and_its_request() {
    let Some(repo) = cep_repo() else {
        return;
    };
    let f = fixture(&profile_rule("exec", &[D1, D2], ""));
    let req = capability_request(&f.world, "op-pv", "cap.exec", Some(D2));
    assert_eq!(
        cep_errors(&repo, "authority_request", &req),
        Vec::<String>::new(),
        "request"
    );
    let out = f.service().decide(&req, &caller()).unwrap();
    assert_eq!(decision_of(&out), "boundary");
    assert_eq!(
        cep_errors(&repo, "authority_decision", &out.envelope),
        Vec::<String>::new(),
        "decision"
    );
}

// ---- review follow-ups for #116 ----

fn policy_with(rules: &str) -> AuthorityPolicyBundle {
    sea_forge_core::yaml::from_str(&format!("version: \"0.1\"\nrules:\n{rules}")).unwrap()
}

#[test]
fn a_bundle_without_execution_profile_keeps_its_hash() {
    // Pinned value: the dimension adds no rule field, so existing bundles hash exactly as before.
    let mut bundle = policy_with(&format!(
        "{}{}",
        rule("allow-actions", "cognate_action", ""),
        rule(
            "gate",
            "cognate_capability",
            "    requires_approval: true\n"
        )
    ));
    bundle.refresh_policy_bundle_hash().unwrap();
    assert_eq!(
        bundle.policy_bundle_hash.as_deref(),
        Some("sha256:127b21fe85fa26685c615c360ced40cd81fb78f2dac61b033afca01d1261bd9c"),
        "update only if PolicyRule's serialized form changed on purpose"
    );
}

#[test]
fn an_approval_gated_rule_without_a_profile_is_skipped_for_a_governed_resource() {
    let rules = format!(
        "{}{}",
        rule(
            "gate",
            "cognate_capability",
            "    requires_approval: true\n"
        ),
        profile_rule("exec", &[D1], "    resources: [\"cap.exec\"]\n")
    );
    let f = fixture(&rules);
    let out = f
        .service()
        .decide(
            &capability_request(&f.world, "op-m1", "cap.exec", Some(D3)),
            &caller(),
        )
        .unwrap();
    assert_eq!(decision_of(&out), "deny");
    assert!(out
        .decision
        .reason_codes
        .iter()
        .any(|c| c == "execution_profile_mismatch"));
}

#[test]
fn two_profile_rules_each_match_only_their_own_digest() {
    let rules = format!(
        "{}{}",
        profile_rule("rule-a", &[D1], ""),
        profile_rule("rule-b", &[D2], "")
    );
    let f = fixture(&rules);
    for (op, digest, rule_name) in [("op-t1", D1, "rule-a"), ("op-t2", D2, "rule-b")] {
        let out = f
            .service()
            .decide(
                &capability_request(&f.world, op, "cap.exec", Some(digest)),
                &caller(),
            )
            .unwrap();
        assert_eq!(decision_of(&out), "boundary", "{op}");
        assert_eq!(out.decision.matched_rule.as_deref(), Some(rule_name));
        assert_eq!(
            out.decision.boundary_constraints["execution_profile"],
            vec![digest.to_string()]
        );
    }
    let other = f
        .service()
        .decide(
            &capability_request(&f.world, "op-t3", "cap.exec", Some(D3)),
            &caller(),
        )
        .unwrap();
    assert_eq!(decision_of(&other), "deny");
}

#[test]
fn the_committed_ledger_entry_records_the_claimed_digest() {
    use sea_forge_authority::cep::LEDGER_ID;
    use sea_forge_ledger::LedgerStream;
    let f = fixture(&profile_rule("exec", &[D1], ""));
    let out = f
        .service()
        .decide(
            &capability_request(&f.world, "op-l1", "cap.exec", Some(D1)),
            &caller(),
        )
        .unwrap();
    let entries = LedgerStream::open(f.root.path(), LEDGER_ID, "audit-reader")
        .unwrap()
        .read_entries()
        .unwrap();
    let entry = entries
        .iter()
        .find(|e| e.entry_ulid == out.committed.entry_ulid())
        .expect("the decision's ledger entry");
    assert_eq!(entry.record_kind, "authority_decision");
    assert_eq!(
        entry.payload["action_request"]["action"]["parameters"]["claims"]
            ["execution_profile_digest"],
        D1
    );
    assert_eq!(
        entry.payload["boundary_constraints"]["execution_profile"],
        json!([D1])
    );
}

#[test]
fn a_subject_scoped_profile_rule_does_not_govern_other_subjects() {
    // Governance is per rule scope: another subject falls through to the broad allow, which carries no
    // execution_profile constraint. The host refuses any spawn whose decision lacks one.
    let rules = format!(
        "{}{}",
        profile_rule("exec", &[D1], "    subjects: [\"actor:bob\"]\n"),
        rule("broad", "cognate_capability", "")
    );
    let f = fixture(&rules);
    let out = f
        .service()
        .decide(
            &capability_request(&f.world, "op-s1", "cap.exec", None),
            &caller(),
        )
        .unwrap();
    assert_eq!(decision_of(&out), "allow");
    assert!(!out
        .decision
        .boundary_constraints
        .contains_key("execution_profile"));
    assert!(out.envelope["constraints"]
        .as_array()
        .is_none_or(|c| c.is_empty()));
}

#[test]
fn the_mismatch_reason_is_only_for_requests_a_profile_rule_applies_to() {
    // Unknown subject: no profile rule applies, so the original default_deny stays.
    let f = fixture(&profile_rule(
        "exec",
        &[D1],
        "    subjects: [\"actor:bob\"]\n",
    ));
    let out = f
        .service()
        .decide(
            &capability_request(&f.world, "op-r1", "cap.exec", Some(D1)),
            &caller(),
        )
        .unwrap();
    assert_eq!(decision_of(&out), "deny");
    assert_eq!(out.decision.reason_codes, vec!["default_deny".to_string()]);
    // A resource no profile rule names: not governed either.
    let f = fixture(&profile_rule(
        "exec",
        &[D1],
        "    resources: [\"cap.exec\"]\n",
    ));
    let out = f
        .service()
        .decide(
            &capability_request(&f.world, "op-r3", "cap.other", Some(D1)),
            &caller(),
        )
        .unwrap();
    assert_eq!(out.decision.reason_codes, vec!["default_deny".to_string()]);
}

#[test]
fn a_non_profile_escalation_cannot_stand_in_for_a_profile_rule() {
    // Approving that escalation would otherwise yield an allow with no execution_profile constraint.
    let rules = format!(
        "{}{}",
        rule(
            "gate",
            "cognate_capability",
            "    requires_approval: true\n"
        ),
        profile_rule("exec", &[D1], "    resources: [\"cap.exec\"]\n")
    );
    let f = fixture(&rules);
    let out = f
        .service()
        .decide(
            &capability_request(&f.world, "op-e1", "cap.exec", Some(D3)),
            &caller(),
        )
        .unwrap();
    assert_eq!(decision_of(&out), "deny");
    assert!(out
        .decision
        .reason_codes
        .iter()
        .any(|c| c == "execution_profile_mismatch"));
    assert!(out.decision.approval_request_id.is_none());
    // A verdict-escalate rule is skipped the same way.
    let rules = format!(
        "{}{}",
        rule("review", "cognate_capability", "").replace("verdict: allow", "verdict: escalate"),
        profile_rule("exec", &[D1], "")
    );
    let f = fixture(&rules);
    let out = f
        .service()
        .decide(
            &capability_request(&f.world, "op-e2", "cap.exec", Some(D3)),
            &caller(),
        )
        .unwrap();
    assert_eq!(decision_of(&out), "deny");
}
