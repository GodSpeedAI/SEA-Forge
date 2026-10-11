//! Stage 8, SEA-Forge half: evidence is recorded against an operation SEA-Forge allowed and settles nothing;
//! settlement is SEA-Forge's own, judged against criteria bound into the allow decision.

use sea_forge_authority::cep::{CepAuthorityError, CepAuthorityService, SettlementCriteria};
use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_domainforge::{SeaSourceSet, SourceFile, WorldRegistry};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const SRC: &str = "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";
const OTHER: &str = "@namespace \"t\"\nentity \"Pump\" { key id: uuid }\n";

fn rule(name: &str, kind: &str) -> String {
    format!("  - name: {name}\n    verdict: allow\n    actor_role: service\n    operation_kind: {kind}\n")
}
fn policy(kinds: &[&str]) -> String {
    format!(
        "version: \"0.1\"\nrules:\n{}",
        kinds.iter().map(|k| rule(k, k)).collect::<String>()
    )
}
fn full_policy() -> String {
    policy(&[
        "cognate_action",
        "evidence_mutation",
        "settlement_declaration",
    ])
}

struct Cell {
    root: tempfile::TempDir,
    worlds: WorldRegistry,
    world: String,
    other_world: String,
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
    let world = register(&mut worlds, "demo", SRC);
    let other_world = register(&mut worlds, "other", OTHER);
    Cell {
        root: tempfile::tempdir().unwrap(),
        worlds,
        world,
        other_world,
    }
}

fn with<T>(
    c: &Cell,
    policy_yaml: &str,
    criteria: SettlementCriteria,
    f: impl FnOnce(&CepAuthorityService<'_>) -> T,
) -> T {
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(policy_yaml).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let service = CepAuthorityService {
        engine: &engine,
        bundle: &bundle,
        worlds: &c.worlds,
        root: c.root.path(),
        approval_ttl: chrono::Duration::hours(24),
        criteria,
    };
    f(&service)
}

fn svc() -> Actor {
    Actor {
        actor_id: "cognate-service".into(),
        role: ActorRole::Service,
    }
}
fn operator() -> Actor {
    Actor {
        actor_id: "op-alice".into(),
        role: ActorRole::Operator,
    }
}

fn auth_request(world: &str, op: &str) -> Value {
    json!({
        "envelope_id": format!("env-authority_request-{op}"),
        "cep_version": "1.0.0", "envelope_version": "1.0", "envelope_kind": "authority_request",
        "created_at": "2026-10-04T12:00:00Z", "created_by": "cognate",
        "scope": {"evaluation_context": "cognate-authority_request", "world_ref": world},
        "boundary_record": {"scope": "s", "included_sections": ["authority"], "excluded_sections": [],
            "known_omissions": ["payload not carried"], "unknowns": [], "redactions": [],
            "compression_notes": [], "out_of_scope_entities": [], "classification": "internal", "limitations": []},
        "completeness_status": "partial", "omission_status": "marked",
        "provenance_refs": [format!("prov-{op}")], "validation_status": "valid", "lineage_refs": [],
        "provenance": [{"provenance_id": format!("prov-{op}"), "source_system_refs": ["cognate"],
            "producer_refs": ["cognate"], "production_method": "governed_operation", "created_at": "2026-10-04T12:00:00Z"}],
        "authority": [{"authority_id": "auth-1", "subject_actor_ref": "actor:alice"}],
        "transformations": [{"transformation_id": format!("tr-{op}"), "transformation_type": "capability_invocation"}],
        "references": [{"ref_id": "r1", "ref_type": "artifact_id", "target_uri_or_id": "tool.ping",
            "availability_status": "available", "integrity_status": "verifiable"}],
        "states": [{"state_id": "s1", "state_type": "declared_state", "subject_ref": "run:1"}],
        "extensions": {
            "cep.profile": {"profile_id": "godspeed.authority_request", "profile_version": "1.0.0"},
            "godspeed.authority_request": {"operation_id": op, "correlation_id": "corr-1",
                "operation_kind": "action", "operation_name": "run.start", "resource_id": "agent.echo"}
        }
    })
}

fn item(id: &str, direction: &str, reliability: &str) -> Value {
    json!({"evidence_id": id, "evidence_type": "test_result", "question_ref": "q-1",
           "target_entity_ref": "concept:runtime_execution", "direction": direction, "reliability": reliability})
}

fn packet(world: &str, op: &str, envelope: &str, items: Vec<Value>) -> Value {
    json!({
        "envelope_id": envelope,
        "cep_version": "1.0.0", "envelope_version": "1.0", "envelope_kind": "evidence_packet",
        "created_at": "2026-10-04T12:05:00Z", "created_by": "cognate-realitytrace-adapter",
        "scope": {"evaluation_context": "godspeed-evidence_packet", "world_ref": world},
        "boundary_record": {"scope": "evidence for one operation", "included_sections": ["evidence", "questions"],
            "excluded_sections": [], "known_omissions": ["only RealityTrace-recorded evidence"], "unknowns": [],
            "redactions": [], "compression_notes": [], "out_of_scope_entities": [], "classification": "internal", "limitations": []},
        "completeness_status": "partial", "omission_status": "marked",
        "provenance_refs": [format!("prov-{envelope}")], "validation_status": "valid",
        "lineage_refs": [format!("env-execution_trace-{op}")],
        "provenance": [{"provenance_id": format!("prov-{envelope}"), "source_system_refs": ["realitytrace"],
            "producer_refs": ["cognate-realitytrace-adapter"], "production_method": "evidence_packaging", "created_at": "2026-10-04T12:05:00Z"}],
        "questions": [{"question_id": "q-1", "question_form": "Did the operation reach the declared state?"}],
        "evidence": items,
        "extensions": {
            "cep.profile": {"profile_id": "godspeed.evidence_packet", "profile_version": "1.0.0"},
            "godspeed.evidence_packet": {"operation_id": op,
                "authority_decision_ref": format!("env-authority_decision-{op}"),
                "execution_trace_ref": format!("env-execution_trace-{op}")}
        }
    })
}

fn allow(c: &Cell, op: &str) {
    with(c, &full_policy(), SettlementCriteria::default(), |s| {
        let d = s.decide(&auth_request(&c.world, op), &svc()).unwrap();
        assert_eq!(
            d.envelope["extensions"]["godspeed.authority_decision"]["decision"],
            "allow"
        );
    });
}

fn submit(c: &Cell, p: &Value) -> Result<(), CepAuthorityError> {
    with(c, &full_policy(), SettlementCriteria::default(), |s| {
        s.submit_evidence(p, &svc()).map(|_| ())
    })
}

fn settle(
    c: &Cell,
    op: &str,
) -> Result<sea_forge_authority::cep::SettlementOutcome, CepAuthorityError> {
    with(c, &full_policy(), SettlementCriteria::default(), |s| {
        s.settle(op, &svc())
    })
}

fn status(o: &sea_forge_authority::cep::SettlementOutcome) -> &str {
    o.envelope["settlements"][0]["settlement_status"]
        .as_str()
        .unwrap()
}

#[test]
fn supporting_evidence_settles_once_and_the_packet_names_its_basis() {
    let c = cell();
    allow(&c, "op-1");
    submit(
        &c,
        &packet(
            &c.world,
            "op-1",
            "env-evidence_packet-1",
            vec![item("ev-1", "supports", "medium")],
        ),
    )
    .unwrap();
    let done = settle(&c, "op-1").unwrap();
    assert_eq!(done.status, "settled");
    assert!(done.final_status && !done.replayed);
    assert_eq!(status(&done), "settled");
    assert_eq!(done.envelope["envelope_kind"], "settlement_packet");
    assert_eq!(done.envelope["scope"]["world_ref"], c.world);
    assert_eq!(
        done.envelope["lineage_refs"],
        json!(["env-evidence_packet-1"])
    );
    assert_eq!(
        done.envelope["settlements"][0]["transformation_ref"],
        "tr-op-1"
    );
    assert!(done.envelope["settlements"][0]["threshold_basis"]
        .as_str()
        .unwrap()
        .contains("min_supporting=1"));
    // Final: asking again returns the recorded settlement, it does not make another.
    let again = settle(&c, "op-1").unwrap();
    assert!(again.replayed && again.final_status);
    assert_eq!(again.envelope, done.envelope);
}

#[test]
fn an_execution_with_only_unknown_evidence_is_not_settled_and_can_be_settled_later() {
    let c = cell();
    allow(&c, "op-2");
    submit(
        &c,
        &packet(
            &c.world,
            "op-2",
            "env-evidence_packet-2a",
            vec![item("ev-a", "unknown", "medium")],
        ),
    )
    .unwrap();
    let first = settle(&c, "op-2").unwrap();
    assert_eq!(first.status, "unsettled");
    assert!(!first.final_status);
    assert!(first.envelope["settlements"][0]["missing_evidence"][0]
        .as_str()
        .unwrap()
        .contains("supporting"));
    // Same evidence, same answer, same record.
    assert_eq!(settle(&c, "op-2").unwrap().envelope, first.envelope);
    // Real supporting evidence arrives; now it can settle.
    submit(
        &c,
        &packet(
            &c.world,
            "op-2",
            "env-evidence_packet-2b",
            vec![item("ev-b", "supports", "high")],
        ),
    )
    .unwrap();
    assert_eq!(settle(&c, "op-2").unwrap().status, "settled");
}

#[test]
fn contradicting_evidence_rejects_and_later_support_cannot_undo_it() {
    let c = cell();
    allow(&c, "op-3");
    submit(
        &c,
        &packet(
            &c.world,
            "op-3",
            "env-evidence_packet-3a",
            vec![
                item("ev-s", "supports", "high"),
                item("ev-c", "contradicts", "high"),
            ],
        ),
    )
    .unwrap();
    let rejected = settle(&c, "op-3").unwrap();
    assert_eq!(rejected.status, "rejected");
    assert!(rejected.final_status);
    assert_eq!(
        rejected.envelope["settlements"][0]["unresolved_contradictions"],
        json!(["ev-c"])
    );
    submit(
        &c,
        &packet(
            &c.world,
            "op-3",
            "env-evidence_packet-3b",
            vec![item("ev-more", "supports", "high")],
        ),
    )
    .unwrap();
    let still = settle(&c, "op-3").unwrap();
    assert_eq!(still.status, "rejected");
    assert!(still.replayed);
}

#[test]
fn reliability_below_the_minimum_does_not_count_and_the_count_is_honored() {
    let c = cell();
    allow(&c, "op-4");
    submit(
        &c,
        &packet(
            &c.world,
            "op-4",
            "env-evidence_packet-4",
            vec![item("ev-low", "supports", "low")],
        ),
    )
    .unwrap();
    assert_eq!(settle(&c, "op-4").unwrap().status, "unsettled");

    // Two required: one supporting item is not enough. (Criteria are bound at allow time, so this operation
    // is allowed under the two-item criteria.)
    let two = SettlementCriteria {
        min_supporting: 2,
        min_reliability: "medium".into(),
    };
    with(&c, &full_policy(), two.clone(), |s| {
        s.decide(&auth_request(&c.world, "op-4b"), &svc()).unwrap()
    });
    let p = packet(
        &c.world,
        "op-4b",
        "env-evidence_packet-4b",
        vec![item("ev-1", "supports", "high")],
    );
    with(&c, &full_policy(), two.clone(), |s| {
        s.submit_evidence(&p, &svc())
    })
    .unwrap();
    assert_eq!(
        with(&c, &full_policy(), two.clone(), |s| s
            .settle("op-4b", &svc()))
        .unwrap()
        .status,
        "unsettled"
    );
    let p2 = packet(
        &c.world,
        "op-4b",
        "env-evidence_packet-4b2",
        vec![item("ev-2", "supports", "medium")],
    );
    with(&c, &full_policy(), two.clone(), |s| {
        s.submit_evidence(&p2, &svc())
    })
    .unwrap();
    assert_eq!(
        with(&c, &full_policy(), two, |s| s.settle("op-4b", &svc()))
            .unwrap()
            .status,
        "settled"
    );
}

#[test]
fn evidence_hangs_only_from_a_committed_allow() {
    let c = cell();
    // Never decided.
    assert!(matches!(
        submit(&c, &packet(&c.world, "op-ghost", "env-evidence_packet-g", vec![item("e", "supports", "high")])),
        Err(CepAuthorityError::Evidence(m)) if m.contains("no committed allow")
    ));
    // Denied (no rule for the surface): a decision exists, but not an allow.
    with(
        &c,
        &policy(&["evidence_mutation", "settlement_declaration"]),
        SettlementCriteria::default(),
        |s| {
            let d = s
                .decide(&auth_request(&c.world, "op-denied"), &svc())
                .unwrap();
            assert_eq!(
                d.envelope["extensions"]["godspeed.authority_decision"]["decision"],
                "deny"
            );
        },
    );
    assert!(matches!(
        submit(
            &c,
            &packet(
                &c.world,
                "op-denied",
                "env-evidence_packet-d",
                vec![item("e", "supports", "high")]
            )
        ),
        Err(CepAuthorityError::Evidence(_))
    ));
    // Escalated and not approved.
    let gated = "version: \"0.1\"\nrules:\n  - name: gated\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_action\n    requires_approval: true\n";
    with(&c, gated, SettlementCriteria::default(), |s| {
        s.decide(&auth_request(&c.world, "op-esc"), &svc()).unwrap()
    });
    assert!(matches!(
        submit(
            &c,
            &packet(
                &c.world,
                "op-esc",
                "env-evidence_packet-e",
                vec![item("e", "supports", "high")]
            )
        ),
        Err(CepAuthorityError::Evidence(_))
    ));
}

#[test]
fn a_broken_chain_is_refused() {
    let c = cell();
    allow(&c, "op-5");
    let good = packet(
        &c.world,
        "op-5",
        "env-evidence_packet-5",
        vec![item("e", "supports", "high")],
    );

    let mut other_world = good.clone();
    other_world["scope"]["world_ref"] = json!(c.other_world);
    assert!(
        matches!(submit(&c, &other_world), Err(CepAuthorityError::Evidence(m)) if m.contains("different world"))
    );

    let mut ghost_world = good.clone();
    ghost_world["scope"]["world_ref"] = json!(format!("world:demo@sha256:{}", "c".repeat(64)));
    assert!(matches!(
        submit(&c, &ghost_world),
        Err(CepAuthorityError::World(_))
    ));

    let mut wrong_decision = good.clone();
    wrong_decision["extensions"]["godspeed.evidence_packet"]["authority_decision_ref"] =
        json!("env-authority_decision-other");
    assert!(
        matches!(submit(&c, &wrong_decision), Err(CepAuthorityError::Evidence(m)) if m.contains("does not name"))
    );

    let mut no_lineage = good.clone();
    no_lineage["lineage_refs"] = json!(["env-something-else"]);
    assert!(
        matches!(submit(&c, &no_lineage), Err(CepAuthorityError::Evidence(m)) if m.contains("lineage"))
    );

    let mut wrong_kind = good.clone();
    wrong_kind["envelope_kind"] = json!("settlement_packet");
    assert!(matches!(
        submit(&c, &wrong_kind),
        Err(CepAuthorityError::WrongKind(_))
    ));

    let mut wrong_profile = good.clone();
    wrong_profile["extensions"]["cep.profile"]["profile_id"] = json!("godspeed.execution_trace");
    assert!(matches!(
        submit(&c, &wrong_profile),
        Err(CepAuthorityError::WrongProfile(_))
    ));

    let mut empty = good.clone();
    empty["evidence"] = json!([]);
    assert!(matches!(
        submit(&c, &empty),
        Err(CepAuthorityError::Malformed(_))
    ));

    let mut bad_direction = good.clone();
    bad_direction["evidence"][0]["direction"] = json!("definitely");
    assert!(matches!(
        submit(&c, &bad_direction),
        Err(CepAuthorityError::Malformed(_))
    ));

    let mut no_questions = good.clone();
    no_questions["questions"] = json!([]);
    assert!(matches!(
        submit(&c, &no_questions),
        Err(CepAuthorityError::Malformed(_))
    ));

    let mut huge = good.clone();
    huge["extensions"]["padding"] = json!("x".repeat(300_000));
    assert!(matches!(
        submit(&c, &huge),
        Err(CepAuthorityError::Oversize(_))
    ));

    // Nothing above was recorded, so there is still nothing to settle.
    assert!(
        matches!(settle(&c, "op-5"), Err(CepAuthorityError::Evidence(m)) if m.contains("no evidence"))
    );
    // And the genuine packet still works.
    submit(&c, &good).unwrap();
    assert_eq!(settle(&c, "op-5").unwrap().status, "settled");
}

#[test]
fn evidence_submission_is_idempotent_by_envelope_id() {
    let c = cell();
    allow(&c, "op-6");
    let p = packet(
        &c.world,
        "op-6",
        "env-evidence_packet-6",
        vec![item("e", "supports", "high")],
    );
    submit(&c, &p).unwrap();
    submit(&c, &p).unwrap();
    // One packet, not two: still one item, and one settlement at the end.
    let done = settle(&c, "op-6").unwrap();
    assert_eq!(done.envelope["evidence"].as_array().unwrap().len(), 1);
}

#[test]
fn settlement_is_judged_only_against_the_criteria_bound_at_allow_time() {
    let c = cell();
    allow(&c, "op-7");
    submit(
        &c,
        &packet(
            &c.world,
            "op-7",
            "env-evidence_packet-7",
            vec![item("e", "supports", "medium")],
        ),
    )
    .unwrap();
    let tightened = SettlementCriteria {
        min_supporting: 1,
        min_reliability: "high".into(),
    };
    let err = with(&c, &full_policy(), tightened, |s| s.settle("op-7", &svc())).unwrap_err();
    assert!(matches!(err, CepAuthorityError::Evidence(m) if m.contains("criteria changed")));
    // Under the original criteria it still settles.
    assert_eq!(settle(&c, "op-7").unwrap().status, "settled");
}

#[test]
fn policy_must_grant_evidence_and_settlement_and_a_refusal_is_recorded() {
    let c = cell();
    allow(&c, "op-8");
    let p = packet(
        &c.world,
        "op-8",
        "env-evidence_packet-8",
        vec![item("e", "supports", "high")],
    );
    // The operator role has no evidence_mutation rule.
    let r = with(&c, &full_policy(), SettlementCriteria::default(), |s| {
        s.submit_evidence(&p, &operator())
    });
    assert!(
        matches!(r, Err(CepAuthorityError::Evidence(m)) if m.contains("may not use evidence_mutation"))
    );
    submit(&c, &p).unwrap();
    let r = with(&c, &full_policy(), SettlementCriteria::default(), |s| {
        s.settle("op-8", &operator())
    });
    assert!(
        matches!(r, Err(CepAuthorityError::Evidence(m)) if m.contains("may not use settlement_declaration"))
    );
    // Without the settlement rule at all, not even the service can settle.
    let r = with(
        &c,
        &policy(&["cognate_action", "evidence_mutation"]),
        SettlementCriteria::default(),
        |s| s.settle("op-8", &svc()),
    );
    assert!(matches!(r, Err(CepAuthorityError::Evidence(_))));
}

#[test]
fn settlement_without_any_evidence_is_refused_not_invented() {
    let c = cell();
    allow(&c, "op-9");
    assert!(
        matches!(settle(&c, "op-9"), Err(CepAuthorityError::Evidence(m)) if m.contains("an execution alone cannot be settled"))
    );
}

#[test]
fn criteria_are_validated() {
    assert!(SettlementCriteria {
        min_supporting: 0,
        min_reliability: "medium".into()
    }
    .validate()
    .is_err());
    assert!(SettlementCriteria {
        min_supporting: 1,
        min_reliability: "excellent".into()
    }
    .validate()
    .is_err());
    assert!(SettlementCriteria::default().validate().is_ok());
    let a = SettlementCriteria {
        min_supporting: 1,
        min_reliability: "high".into(),
    };
    assert_ne!(a.sha256(), SettlementCriteria::default().sha256());
}

// Profile conformance by the CEP repository's own validator (gated on CEP_REPO).
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
fn cep_validators_accept_the_evidence_packet_and_every_settlement_status() {
    let Ok(repo) = std::env::var("CEP_REPO") else {
        eprintln!("CEP_REPO not set: skipping profile conformance");
        return;
    };
    let c = cell();
    for (op, items, expected) in [
        ("op-v1", vec![item("e1", "supports", "high")], "settled"),
        ("op-v2", vec![item("e2", "unknown", "medium")], "unsettled"),
        ("op-v3", vec![item("e3", "contradicts", "high")], "rejected"),
    ] {
        allow(&c, op);
        let p = packet(&c.world, op, &format!("env-evidence_packet-{op}"), items);
        assert_eq!(
            cep_errors(&repo, "evidence_packet", &p),
            Vec::<String>::new(),
            "evidence packet {op}"
        );
        submit(&c, &p).unwrap();
        let done = settle(&c, op).unwrap();
        assert_eq!(done.status, expected);
        assert_eq!(
            cep_errors(&repo, "settlement_packet", &done.envelope),
            Vec::<String>::new(),
            "settlement {expected}"
        );
    }
}
