//! Stage 8, SEA-Forge half: evidence is recorded against an operation SEA-Forge allowed and settles nothing;
//! settlement is SEA-Forge's own, judged against criteria bound into the allow decision.

use sea_forge_authority::cep::{
    authorize_settlement_declaration, complete_settlement_resume,
    derive_settlement_authority_class, fail_settlement_resume, reconcile_settlement_resume,
    release_settlement_resume, settlement_approval_id, settlement_declaration_content_sha256,
    settlement_pending_records, settlement_request_sha256, settlement_status,
    verify_declaration_echoes_request, CepAuthorityError, CepAuthorityService,
    SettlementAuthorityClass, SettlementCriteria, SettlementGate, SettlementIngress,
    SettlementOperation, SettlementPendingRecord, SettlementStatus, SettlementTransport,
    VerifiedSettlementIngress, LEDGER_ID,
};
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{
    Actor, ActorRole, AuthorityAction, AuthorityDecision, DeclarationIndependence,
    DeclarationReliability, DeclarationStatus, Declarer, SettlementDeclaration,
    SettlementDeclarationRequest, SettlementStrength, Verdict,
};
use sea_forge_domainforge::{SeaSourceSet, SourceFile, WorldRegistry};
use sea_forge_ledger::LedgerStream;
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
            sha256: hex::encode(Sha256::digest(src.as_bytes())),
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
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(policy_yaml).unwrap();
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

// ── Corroboration: evidence is checked against what the evidence source recorded ───────────────────────

/// Stands in for RealityTrace's ledger: the items it recorded, by evidence id.
struct Recorded(Vec<Value>);

impl sea_forge_authority::cep::EvidenceSource for Recorded {
    fn name(&self) -> &'static str {
        "realitytrace"
    }
    fn recorded_item(&self, item: &Value) -> Result<Value, String> {
        self.0
            .iter()
            .find(|r| r["evidence_id"] == item["evidence_id"])
            .cloned()
            .ok_or_else(|| "not in the ledger".to_string())
    }
}

fn sourced(id: &str, direction: &str, reliability: &str) -> Value {
    let mut i = item(id, direction, reliability);
    i["source"] = json!("realitytrace");
    i
}

fn submit_checked(
    c: &Cell,
    p: &Value,
    recorded: &Recorded,
    required: bool,
) -> Result<sea_forge_authority::cep::EvidenceAck, CepAuthorityError> {
    with(c, &full_policy(), SettlementCriteria::default(), |s| {
        s.submit_evidence_with(
            p,
            &svc(),
            &sea_forge_authority::cep::Corroboration::Source {
                source: recorded,
                required,
            },
        )
    })
}

fn settle_corroborated(
    c: &Cell,
    op: &str,
) -> Result<sea_forge_authority::cep::SettlementOutcome, CepAuthorityError> {
    with(c, &full_policy(), SettlementCriteria::default(), |s| {
        s.settle_with(op, &svc(), Some("realitytrace"))
    })
}

#[test]
fn corroborated_evidence_settles_and_says_what_it_was_checked_against() {
    let c = cell();
    allow(&c, "op-c1");
    let recorded = Recorded(vec![item("ev-c1", "supports", "medium")]);
    let ack = submit_checked(
        &c,
        &packet(
            &c.world,
            "op-c1",
            "env-evidence_packet-c1",
            vec![sourced("ev-c1", "supports", "medium")],
        ),
        &recorded,
        true,
    )
    .unwrap();
    assert_eq!(ack.corroborated_by, Some("realitytrace"));
    assert_eq!(settle_corroborated(&c, "op-c1").unwrap().status, "settled");
}

#[test]
fn a_packet_that_flips_the_recorded_direction_is_refused_and_settles_nothing() {
    let c = cell();
    allow(&c, "op-c2");
    // RealityTrace recorded a failing verifier; the requester claims support.
    let recorded = Recorded(vec![item("ev-c2", "contradicts", "medium")]);
    let err = submit_checked(
        &c,
        &packet(
            &c.world,
            "op-c2",
            "env-evidence_packet-c2",
            vec![sourced("ev-c2", "supports", "medium")],
        ),
        &recorded,
        true,
    )
    .unwrap_err();
    assert!(err.to_string().contains("direction differs"), "{err}");
    // Nothing was committed: there is no corroborated evidence to settle on.
    let refused = settle_corroborated(&c, "op-c2").unwrap_err();
    assert!(
        refused.to_string().contains("no corroborated evidence"),
        "{refused}"
    );
}

#[test]
fn an_upgraded_reliability_or_an_unrecorded_item_is_refused() {
    let c = cell();
    allow(&c, "op-c3");
    let recorded = Recorded(vec![item("ev-c3", "supports", "low")]);
    let upgraded = submit_checked(
        &c,
        &packet(
            &c.world,
            "op-c3",
            "env-evidence_packet-c3a",
            vec![sourced("ev-c3", "supports", "medium")],
        ),
        &recorded,
        true,
    )
    .unwrap_err();
    assert!(
        upgraded.to_string().contains("reliability differs"),
        "{upgraded}"
    );
    let invented = submit_checked(
        &c,
        &packet(
            &c.world,
            "op-c3",
            "env-evidence_packet-c3b",
            vec![sourced("ev-invented", "supports", "medium")],
        ),
        &recorded,
        true,
    )
    .unwrap_err();
    assert!(
        invented.to_string().contains("cannot be corroborated"),
        "{invented}"
    );
}

#[test]
fn when_corroboration_is_required_unsourced_items_are_refused_and_old_packets_do_not_count() {
    let c = cell();
    allow(&c, "op-c4");
    let recorded = Recorded(vec![item("ev-c4", "supports", "medium")]);
    // An item that names no evidence source.
    let unsourced = submit_checked(
        &c,
        &packet(
            &c.world,
            "op-c4",
            "env-evidence_packet-c4a",
            vec![item("ev-x", "supports", "medium")],
        ),
        &recorded,
        true,
    )
    .unwrap_err();
    assert!(
        unsourced.to_string().contains("is not from realitytrace"),
        "{unsourced}"
    );
    // A packet accepted as stated (before corroboration was required) never decides a corroborated settlement.
    submit(
        &c,
        &packet(
            &c.world,
            "op-c4",
            "env-evidence_packet-c4b",
            vec![item("ev-old", "supports", "high")],
        ),
    )
    .unwrap();
    assert!(settle_corroborated(&c, "op-c4")
        .unwrap_err()
        .to_string()
        .contains("no corroborated evidence"));
    // Without the requirement, unsourced items pass through and sourced ones are still checked, but a
    // packet with any unchecked item is not marked corroborated, so it never counts once the
    // requirement is turned on.
    let flipped = submit_checked(
        &c,
        &packet(
            &c.world,
            "op-c4",
            "env-evidence_packet-c4d",
            vec![
                item("ev-z", "supports", "medium"),
                sourced("ev-c4", "contradicts", "medium"),
            ],
        ),
        &recorded,
        false,
    )
    .unwrap_err();
    assert!(
        flipped.to_string().contains("direction differs"),
        "{flipped}"
    );
    let lenient = submit_checked(
        &c,
        &packet(
            &c.world,
            "op-c4",
            "env-evidence_packet-c4c",
            vec![
                item("ev-y", "supports", "medium"),
                sourced("ev-c4", "supports", "medium"),
            ],
        ),
        &recorded,
        false,
    )
    .unwrap();
    assert_eq!(lenient.corroborated_by, None);
    assert!(settle_corroborated(&c, "op-c4")
        .unwrap_err()
        .to_string()
        .contains("no corroborated evidence"));
}

#[test]
fn a_recorded_item_without_a_direction_cannot_be_corroborated() {
    let c = cell();
    allow(&c, "op-c5");
    let recorded = Recorded(vec![
        json!({"evidence_id": "ev-c5", "reliability": "medium"}),
    ]);
    let err = submit_checked(
        &c,
        &packet(
            &c.world,
            "op-c5",
            "env-evidence_packet-c5",
            vec![sourced("ev-c5", "unknown", "medium")],
        ),
        &recorded,
        true,
    )
    .unwrap_err();
    assert!(err.to_string().contains("has no direction"), "{err}");
}

// ---- Settlement authority class: the gate every settlement_declaration caller passes ----------------

const SETTLE_OP: &str = "op-settle-class";

/// An external declaration is gated: the rule names `external` and escalates for approval.
const EXTERNAL_GATED: &str = "version: \"0.1\"\nrules:\n  - name: external-settlement\n    verdict: allow\n    requires_approval: true\n    actor_role: service\n    operation_kind: settlement_declaration\n    boundary_constraints:\n      settlement_authority_class: [external]\n";
/// A rule with no class boundary: reaches local declarations only.
const UNCONSTRAINED: &str = "version: \"0.1\"\nrules:\n  - name: settle\n    verdict: allow\n    actor_role: service\n    operation_kind: settlement_declaration\n";

fn class_policy_engine(policy_yaml: &str) -> (AuthorityPolicyBundle, PolicyAuthorityEngine) {
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(policy_yaml).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    (bundle, engine)
}

/// Run the real gate entry point through the verified SWE_SEED ingress, against a fresh ledger.
fn authorize_swe_seed(policy_yaml: &str) -> Result<SettlementGate, CepAuthorityError> {
    let root = tempfile::tempdir().unwrap();
    let (bundle, engine) = class_policy_engine(policy_yaml);
    let actor = svc();
    let ingress = VerifiedSettlementIngress::verified(
        SettlementIngress::SweSeedDeclaration,
        SettlementTransport::SweSeedAdapter,
        None,
        &actor,
        vec![],
    )?;
    authorize_settlement_declaration(
        &engine,
        &bundle,
        root.path(),
        SettlementOperation {
            case_id: "case_a",
            operation_id: SETTLE_OP,
            declared_request_sha256: CONTENT_A,
        },
        &ingress,
        chrono::Duration::hours(24),
    )
}

/// Run the real gate entry point through the verified CEP settlement ingress (no transport), fresh ledger.
fn authorize_cep(policy_yaml: &str) -> Result<SettlementGate, CepAuthorityError> {
    let root = tempfile::tempdir().unwrap();
    let (bundle, engine) = class_policy_engine(policy_yaml);
    let actor = svc();
    let ingress = VerifiedSettlementIngress::verified(
        SettlementIngress::CepSettlement,
        SettlementTransport::None,
        None,
        &actor,
        vec![],
    )?;
    authorize_settlement_declaration(
        &engine,
        &bundle,
        root.path(),
        SettlementOperation {
            case_id: "case_a",
            operation_id: SETTLE_OP,
            declared_request_sha256: CONTENT_A,
        },
        &ingress,
        chrono::Duration::hours(24),
    )
}

/// Evaluate a raw `settlement_declaration` Reserved action, so the parameters themselves are under test.
fn raw_verdict(policy_yaml: &str, parameters: Value) -> Verdict {
    let root = tempfile::tempdir().unwrap();
    let (bundle, engine) = class_policy_engine(policy_yaml);
    let action = AuthorityAction::Reserved {
        resource_type: "settlement_declaration".into(),
        resource_id: SETTLE_OP.into(),
        parameters,
    };
    let actor = svc();
    let binding = bundle.resolve_identity(&actor.actor_id, actor.role.clone());
    engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding,
            run_id: SETTLE_OP,
            case_id: "cognate",
            plan_item_id: SETTLE_OP,
            sequence: 1,
            action: &action,
            workspace_root: root.path(),
            evidence_refs: vec![],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            opa_candidate: None,
            environment: None,
        })
        .unwrap()
        .verdict
}

fn class_params(class: &str) -> Value {
    json!({
        "operation_id": SETTLE_OP,
        "surface": "settlement_declaration",
        "settlement_authority_class": class,
    })
}

#[test]
fn an_external_settlement_declaration_escalates_under_its_gated_rule() {
    // The gated rule escalates for approval: a pending record is written, not a refusal and not an allow.
    assert!(matches!(
        authorize_swe_seed(EXTERNAL_GATED).unwrap(),
        SettlementGate::Pending {
            replayed: false,
            ..
        }
    ));
    assert_eq!(
        raw_verdict(EXTERNAL_GATED, class_params("external")),
        Verdict::Escalate
    );
}

#[test]
fn a_local_settlement_declaration_is_allowed_by_an_unconstrained_rule() {
    assert_eq!(
        authorize_cep(UNCONSTRAINED).unwrap(),
        SettlementGate::Allowed(SettlementAuthorityClass::Local)
    );
    assert_eq!(
        raw_verdict(UNCONSTRAINED, class_params("local")),
        Verdict::Allow
    );
}

#[test]
fn an_external_settlement_declaration_is_not_reached_by_an_unconstrained_allow_rule() {
    // Fail closed: a plain allow never covers an external declarer.
    assert!(authorize_swe_seed(UNCONSTRAINED).is_err());
    assert_ne!(
        raw_verdict(UNCONSTRAINED, class_params("external")),
        Verdict::Allow
    );
}

#[test]
fn a_missing_or_unknown_settlement_authority_class_fails_closed_not_allow() {
    // Missing class on the Reserved action: no rule matches, so the verdict is never Allow.
    let missing = json!({"operation_id": SETTLE_OP, "surface": "settlement_declaration"});
    assert_ne!(raw_verdict(UNCONSTRAINED, missing.clone()), Verdict::Allow);
    assert_ne!(raw_verdict(EXTERNAL_GATED, missing), Verdict::Allow);
    // Unknown and miscased values are refused too.
    for unknown in ["internal", "External", "LOCAL", "", "local "] {
        assert_ne!(
            raw_verdict(UNCONSTRAINED, class_params(unknown)),
            Verdict::Allow,
            "class {unknown:?} must not be allowed"
        );
        assert!(
            SettlementAuthorityClass::parse(unknown).is_err(),
            "class {unknown:?} must not parse"
        );
    }
    assert_eq!(
        SettlementAuthorityClass::parse("external").unwrap(),
        SettlementAuthorityClass::External
    );
}

#[test]
fn an_external_class_rule_that_is_a_plain_allow_is_refused_at_policy_load() {
    let plain = "version: \"0.1\"\nrules:\n  - name: ext\n    verdict: allow\n    actor_role: service\n    operation_kind: settlement_declaration\n    boundary_constraints:\n      settlement_authority_class: [external]\n";
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(plain).unwrap();
    assert!(PolicyAuthorityEngine::new(bundle).is_err());
}

#[test]
fn a_settlement_authority_class_names_only_settlement_declarations() {
    let elsewhere = "version: \"0.1\"\nrules:\n  - name: ext\n    verdict: allow\n    requires_approval: true\n    actor_role: service\n    operation_kind: cognate_action\n    boundary_constraints:\n      settlement_authority_class: [external]\n";
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(elsewhere).unwrap();
    assert!(PolicyAuthorityEngine::new(bundle).is_err());
}

// ---- Settlement authority class: derived from the verified ingress, never passed by a caller ----------------

const ATTESTATION: &str = "attestation:swe-seed:1";

fn verified_for<'a>(
    ingress: SettlementIngress,
    transport: SettlementTransport,
    attestation: Option<&'a str>,
    actor: &'a Actor,
) -> Result<VerifiedSettlementIngress<'a>, CepAuthorityError> {
    VerifiedSettlementIngress::verified(ingress, transport, attestation, actor, vec![])
}

#[test]
fn swe_seed_declaration_ingress_derives_external() {
    let actor = svc();
    let ingress = verified_for(
        SettlementIngress::SweSeedDeclaration,
        SettlementTransport::SweSeedAdapter,
        None,
        &actor,
    )
    .unwrap();
    assert_eq!(
        derive_settlement_authority_class(&ingress),
        SettlementAuthorityClass::External
    );
}

#[test]
fn artifact_transition_resume_ingress_derives_external() {
    let actor = operator();
    let ingress = verified_for(
        SettlementIngress::ArtifactTransitionResume,
        SettlementTransport::SweSeedAdapter,
        Some(ATTESTATION),
        &actor,
    )
    .unwrap();
    assert_eq!(
        derive_settlement_authority_class(&ingress),
        SettlementAuthorityClass::External
    );
}

#[test]
fn cep_settle_ingress_derives_local_without_transport() {
    let actor = svc();
    let ingress = verified_for(
        SettlementIngress::CepSettlement,
        SettlementTransport::None,
        None,
        &actor,
    )
    .unwrap();
    assert_eq!(
        derive_settlement_authority_class(&ingress),
        SettlementAuthorityClass::Local
    );
}

#[test]
fn obligation_closure_ingress_derives_local_without_transport() {
    let actor = svc();
    let ingress = verified_for(
        SettlementIngress::ObligationClosure,
        SettlementTransport::None,
        None,
        &actor,
    )
    .unwrap();
    assert_eq!(
        derive_settlement_authority_class(&ingress),
        SettlementAuthorityClass::Local
    );
}

#[test]
fn an_external_looking_operation_cannot_be_classified_local_by_the_caller() {
    let actor = svc();
    // An external ingress with no transport is not a verified ingress, so it cannot be built at all.
    assert!(verified_for(
        SettlementIngress::SweSeedDeclaration,
        SettlementTransport::None,
        None,
        &actor
    )
    .is_err());
    assert!(verified_for(
        SettlementIngress::ArtifactTransitionResume,
        SettlementTransport::None,
        None,
        &actor
    )
    .is_err());
    // A CEP or obligation settlement cannot smuggle an adapter transport or attestation to look local-free.
    assert!(verified_for(
        SettlementIngress::CepSettlement,
        SettlementTransport::SweSeedAdapter,
        None,
        &actor
    )
    .is_err());
    assert!(verified_for(
        SettlementIngress::ObligationClosure,
        SettlementTransport::None,
        Some(ATTESTATION),
        &actor
    )
    .is_err());
    assert!(verified_for(
        SettlementIngress::SweSeedDeclaration,
        SettlementTransport::SweSeedAdapter,
        Some(""),
        &actor
    )
    .is_err());
    // An external operation that reaches the gate is never classified local, so a local-only rule refuses it.
    assert!(authorize_swe_seed(UNCONSTRAINED).is_err());
    assert!(matches!(
        authorize_swe_seed(EXTERNAL_GATED).unwrap(),
        SettlementGate::Pending { .. }
    ));
}

// ---- Durable pending state for an approval-gated external settlement -----------------------------------------

const CASE: &str = "case_a";

/// Two declared-content hashes that differ, standing for the declaration content approved and a changed one.
const CONTENT_A: &str = "sha256:declared-content-a";
const CONTENT_B: &str = "sha256:declared-content-b";

fn gate_swe_seed_in(
    root: &std::path::Path,
    policy_yaml: &str,
    evidence_refs: Vec<String>,
) -> Result<SettlementGate, CepAuthorityError> {
    gate_swe_seed_case(root, CASE, policy_yaml, evidence_refs)
}

fn gate_swe_seed_case(
    root: &std::path::Path,
    case_id: &str,
    policy_yaml: &str,
    evidence_refs: Vec<String>,
) -> Result<SettlementGate, CepAuthorityError> {
    gate_swe_seed_ttl(
        root,
        case_id,
        policy_yaml,
        evidence_refs,
        chrono::Duration::hours(24),
    )
}

fn gate_swe_seed_ttl(
    root: &std::path::Path,
    case_id: &str,
    policy_yaml: &str,
    evidence_refs: Vec<String>,
    ttl: chrono::Duration,
) -> Result<SettlementGate, CepAuthorityError> {
    let (bundle, engine) = class_policy_engine(policy_yaml);
    let actor = svc();
    let ingress = VerifiedSettlementIngress::verified(
        SettlementIngress::SweSeedDeclaration,
        SettlementTransport::SweSeedAdapter,
        None,
        &actor,
        evidence_refs,
    )?;
    let declared = CONTENT_A;
    authorize_settlement_declaration(
        &engine,
        &bundle,
        root,
        SettlementOperation {
            case_id,
            operation_id: SETTLE_OP,
            declared_request_sha256: declared,
        },
        &ingress,
        ttl,
    )
}

/// The approver's policy: the external gate, plus operators may resolve approvals.
fn approval_policy() -> String {
    format!(
        "{EXTERNAL_GATED}  - name: approve\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n"
    )
}

/// Resolve the settlement approval for `pending_id` as `resolver`, through the CEP resolution path.
fn resolve_in(
    root: &std::path::Path,
    pending_id: &str,
    approve: bool,
    resolver: &Actor,
) -> Result<Value, CepAuthorityError> {
    let policy = approval_policy();
    let (bundle, engine) = class_policy_engine(&policy);
    let worlds = sea_forge_domainforge::WorldRegistry::new();
    let service = CepAuthorityService {
        engine: &engine,
        bundle: &bundle,
        worlds: &worlds,
        root,
        approval_ttl: chrono::Duration::hours(24),
        criteria: SettlementCriteria::default(),
    };
    service.resolve_approval(&settlement_approval_id(pending_id), approve, resolver)
}

fn evidence_a() -> Vec<String> {
    vec!["evidence:a".into()]
}

fn ledger_count(root: &std::path::Path, kind: &str) -> usize {
    LedgerStream::open(root, LEDGER_ID, "test-reader")
        .unwrap()
        .read_entries()
        .unwrap()
        .iter()
        .filter(|entry| entry.record_kind == kind)
        .count()
}

/// Create one pending record and return it, with the approver policy in force.
fn pending_for(root: &std::path::Path, evidence: Vec<String>) -> SettlementPendingRecord {
    pending_of(gate_swe_seed_in(root, &approval_policy(), evidence).unwrap()).0
}

fn pending_of(gate: SettlementGate) -> (SettlementPendingRecord, bool) {
    match gate {
        SettlementGate::Pending { record, replayed } => (*record, replayed),
        other => panic!("expected a pending settlement, got {other:?}"),
    }
}

#[test]
fn a_pending_settlement_is_durable_and_reads_back_after_reload() {
    let root = tempfile::tempdir().unwrap();
    let (record, replayed) = pending_of(
        gate_swe_seed_in(root.path(), EXTERNAL_GATED, vec!["evidence:a".into()]).unwrap(),
    );
    assert!(!replayed);
    // Read back from the ledger on disk, as a fresh process would.
    let stored = settlement_pending_records(root.path()).unwrap();
    assert_eq!(stored, vec![record.clone()]);
    assert_eq!(record.operation_id, SETTLE_OP);
    assert_eq!(record.operation_kind, "settlement_declaration");
    assert_eq!(record.settlement_authority_class, "external");
    assert_eq!(record.ingress, "swe_seed_declaration");
    assert_eq!(record.actor_id, svc().actor_id);
    assert_eq!(record.approval_requirement, "policy_escalate");
    assert_eq!(record.evidence_refs, vec!["evidence:a".to_string()]);
    assert!(record.pending_id.starts_with("settlement-pending-"));
    assert!(!record.decision_ref.is_empty());
    let created = chrono::DateTime::parse_from_rfc3339(&record.created_at).unwrap();
    let expires = chrono::DateTime::parse_from_rfc3339(&record.expires_at).unwrap();
    assert!(expires > created);
}

#[test]
fn an_identical_operation_in_two_cases_cannot_share_a_pending_record_or_approval() {
    let root = tempfile::tempdir().unwrap();
    let policy = approval_policy();
    let (case_a, _) =
        pending_of(gate_swe_seed_case(root.path(), "case_a", &policy, evidence_a()).unwrap());
    let (case_b, replayed_b) =
        pending_of(gate_swe_seed_case(root.path(), "case_b", &policy, evidence_a()).unwrap());
    assert!(
        !replayed_b,
        "the same operation in another case is a new request"
    );
    assert_ne!(case_a.request_sha256, case_b.request_sha256);
    assert_ne!(case_a.pending_id, case_b.pending_id);
    assert_eq!(case_b.case_id, "case_b");
    assert_eq!(case_b.operation_id, case_a.operation_id);

    // Approving case_a's pending record must not approve case_b's.
    resolve_in(root.path(), &case_a.pending_id, true, &operator()).unwrap();
    assert_eq!(
        settlement_status(root.path(), &case_a.pending_id).unwrap(),
        SettlementStatus::Approved
    );
    assert_eq!(
        settlement_status(root.path(), &case_b.pending_id).unwrap(),
        SettlementStatus::Pending
    );
    // And case_b's gate call still yields a pending record, never a claim on case_a's approval.
    let again = gate_swe_seed_case(root.path(), "case_b", &policy, evidence_a()).unwrap();
    assert!(matches!(
        again,
        SettlementGate::Pending { replayed: true, .. }
    ));
}

#[test]
fn a_legacy_pending_record_without_a_case_is_never_reused() {
    let root = tempfile::tempdir().unwrap();
    // A record written before the case was bound into the request hash: no `case_id` in its payload.
    let legacy_sha = "b".repeat(64);
    let legacy_id = format!("settlement-pending-{}", &legacy_sha[..32]);
    let legacy = json!({
        "pending_id": legacy_id,
        "request_sha256": legacy_sha,
        "operation_id": SETTLE_OP,
        "operation_kind": "settlement_declaration",
        "settlement_authority_class": "external",
        "ingress": "swe_seed_declaration",
        "actor_id": svc().actor_id,
        "actor_role": "service",
        "identity_binding_hash": null,
        "evidence_refs": evidence_a(),
        "approval_requirement": "policy_escalate",
        "decision_ref": "dec_legacy",
        "created_at": chrono::Utc::now().to_rfc3339(),
        "expires_at": (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339(),
    });
    LedgerStream::open(root.path(), LEDGER_ID, "test-writer")
        .unwrap()
        .commit_typed_new(
            "settlement_pending",
            format!("settlement-pending:{legacy_sha}"),
            vec![SETTLE_OP.into(), legacy_id.clone()],
            &legacy,
            vec!["dec_legacy".into()],
        )
        .unwrap();

    // The same operation in the current case is a new request: it gets its own record, never the legacy one.
    let (fresh, replayed) =
        pending_of(gate_swe_seed_in(root.path(), EXTERNAL_GATED, evidence_a()).unwrap());
    assert!(!replayed);
    assert_ne!(fresh.pending_id, legacy_id);
    assert_eq!(fresh.case_id, CASE);
    assert_eq!(settlement_pending_records(root.path()).unwrap().len(), 2);
}

/// The approved request hash for `evidence_refs` under `policy_yaml`, bound to the declared content hash.
fn bound_request_sha256(
    policy_yaml: &str,
    case_id: &str,
    evidence_refs: Vec<String>,
    content_sha256: &str,
) -> String {
    let (bundle, _) = class_policy_engine(policy_yaml);
    let actor = svc();
    let ingress = VerifiedSettlementIngress::verified(
        SettlementIngress::SweSeedDeclaration,
        SettlementTransport::SweSeedAdapter,
        None,
        &actor,
        evidence_refs,
    )
    .unwrap();
    settlement_request_sha256(&bundle, case_id, SETTLE_OP, &ingress, content_sha256).unwrap()
}

/// Gate the approved request for `CASE`, presenting `declared` as the content about to be declared.
fn gate_declared(
    root: &std::path::Path,
    policy_yaml: &str,
    evidence_refs: Vec<String>,
    declared: &str,
) -> Result<SettlementGate, CepAuthorityError> {
    let (bundle, engine) = class_policy_engine(policy_yaml);
    let actor = svc();
    let ingress = VerifiedSettlementIngress::verified(
        SettlementIngress::SweSeedDeclaration,
        SettlementTransport::SweSeedAdapter,
        None,
        &actor,
        evidence_refs,
    )?;
    authorize_settlement_declaration(
        &engine,
        &bundle,
        root,
        SettlementOperation {
            case_id: CASE,
            operation_id: SETTLE_OP,
            declared_request_sha256: declared,
        },
        &ingress,
        chrono::Duration::hours(24),
    )
}

#[test]
fn a_claim_whose_declared_content_differs_from_the_approved_request_fails_without_a_declaration() {
    let root = tempfile::tempdir().unwrap();
    let policy = approval_policy();
    let (record, _) = pending_of(gate_swe_seed_in(root.path(), &policy, evidence_a()).unwrap());
    resolve_in(root.path(), &record.pending_id, true, &operator()).unwrap();

    // The declaration about to be made carries different content than the content that was approved.
    let err = gate_declared(root.path(), &policy, evidence_a(), CONTENT_B).unwrap_err();
    assert!(
        matches!(&err, CepAuthorityError::Approval(m) if m.contains("content_mismatch")),
        "{err:?}"
    );
    assert_eq!(
        settlement_status(root.path(), &record.pending_id).unwrap(),
        SettlementStatus::Failed {
            reason: "content_mismatch".into()
        }
    );
    // The approval is spent by the failure: nothing can claim it afterwards.
    assert!(gate_declared(root.path(), &policy, evidence_a(), CONTENT_A).is_err());
    assert!(gate_swe_seed_in(root.path(), &policy, evidence_a()).is_err());
}

/// The request a SWE_SEED declaration is made from. Only the fields that are fixed at request time are set.
fn declaration_request(criteria_ref: &str) -> SettlementDeclarationRequest {
    SettlementDeclarationRequest {
        settlement_ref: "set_a".into(),
        run_id: SETTLE_OP.into(),
        case_id: CASE.into(),
        plan_item_id: "item_a".into(),
        claim_manifest_sha256: "sha256:manifest".into(),
        criteria_ref: criteria_ref.into(),
        criteria_sha256: "sha256:criteria".into(),
        criteria_record_hash: "sha256:criteria-record".into(),
        criteria_declared_at: "2026-07-22T00:00:00Z".into(),
        execution_started_at: "2026-07-22T00:00:01Z".into(),
        job_contract_ref: None,
        origin_refs: vec![],
        verifier_ref: "swe_seed_harness".into(),
        verifier_sha256: "sha256:verifier".into(),
        acting_entity_id: "entity_a".into(),
        requested_strength: SettlementStrength::Strong,
        declarer: Declarer {
            actor_id: "operator_a".into(),
            authority_ref: "swe_seed".into(),
            role: "R-AA".into(),
            standing_basis: "external_verification".into(),
        },
        variation_tags: Default::default(),
        disruption_tags: vec![],
        orchestration_burden: None,
        source_evidence_refs: vec!["evd_a".into()],
        authored_by: None,
        authorization_ref: None,
        authorization_request_sha256: None,
    }
}

#[test]
fn a_claim_whose_one_declared_field_changed_after_approval_is_refused_as_content_mismatch() {
    let root = tempfile::tempdir().unwrap();
    let policy = approval_policy();
    let approved = settlement_declaration_content_sha256(&declaration_request("crit_a")).unwrap();
    let (record, _) =
        pending_of(gate_declared(root.path(), &policy, evidence_a(), &approved).unwrap());
    resolve_in(root.path(), &record.pending_id, true, &operator()).unwrap();

    // After approval, one declared field changes: only the criteria reference differs.
    let changed = settlement_declaration_content_sha256(&declaration_request("crit_b")).unwrap();
    assert_ne!(changed, approved);
    let err = gate_declared(root.path(), &policy, evidence_a(), &changed).unwrap_err();
    assert!(
        matches!(&err, CepAuthorityError::Approval(m) if m.contains("content_mismatch")),
        "{err:?}"
    );
    assert_eq!(
        settlement_status(root.path(), &record.pending_id).unwrap(),
        SettlementStatus::Failed {
            reason: "content_mismatch".into()
        }
    );
    // No declaration was made, and the spent approval cannot be claimed with the approved content either.
    assert_eq!(ledger_count(root.path(), "settlement_declaration"), 0);
    assert!(gate_declared(root.path(), &policy, evidence_a(), &approved).is_err());
}

#[test]
fn a_claim_whose_declared_content_matches_the_approved_request_resumes() {
    let root = tempfile::tempdir().unwrap();
    let policy = approval_policy();
    let (record, _) = pending_of(gate_swe_seed_in(root.path(), &policy, evidence_a()).unwrap());
    resolve_in(root.path(), &record.pending_id, true, &operator()).unwrap();
    assert_eq!(
        bound_request_sha256(&policy, CASE, evidence_a(), CONTENT_A),
        record.request_sha256
    );
    match gate_declared(root.path(), &policy, evidence_a(), CONTENT_A).unwrap() {
        SettlementGate::Resumed {
            pending_id,
            request_sha256,
            ..
        } => {
            assert_eq!(pending_id, record.pending_id);
            assert_eq!(request_sha256, record.request_sha256);
        }
        other => panic!("expected a claimed settlement, got {other:?}"),
    }
}

/// A declaration the authority would append for `request`: it echoes every requested field and adds its own.
fn echoing_declaration(request: &SettlementDeclarationRequest) -> SettlementDeclaration {
    SettlementDeclaration {
        version: "0.2".into(),
        declaration_id: "sdec_a".into(),
        settlement_ref: request.settlement_ref.clone(),
        run_id: request.run_id.clone(),
        case_id: request.case_id.clone(),
        plan_item_id: request.plan_item_id.clone(),
        claim_manifest_sha256: request.claim_manifest_sha256.clone(),
        status: DeclarationStatus::Accepted,
        strength: SettlementStrength::Strong,
        qualifies_for_capability: true,
        criteria_ref: request.criteria_ref.clone(),
        criteria_sha256: request.criteria_sha256.clone(),
        criteria_record_hash: request.criteria_record_hash.clone(),
        criteria_declared_at: request.criteria_declared_at.clone(),
        job_contract_ref: request.job_contract_ref.clone(),
        origin_refs: request.origin_refs.clone(),
        verifier_ref: request.verifier_ref.clone(),
        verifier_sha256: request.verifier_sha256.clone(),
        verification_evidence_refs: request.source_evidence_refs.clone(),
        declarer: request.declarer.clone(),
        independence: DeclarationIndependence {
            acting_entity_id: request.acting_entity_id.clone(),
            independent: true,
            basis: "test".into(),
        },
        reliability: DeclarationReliability {
            feedback_delay_ms: 0,
            attribution_confidence: "1.000000".into(),
            gaming_exposure: "0.000000".into(),
            hidden_debt_blindness: "0.000000".into(),
            weight: "1.000000".into(),
            basis: "test".into(),
        },
        variation_tags: request.variation_tags.clone(),
        disruption_tags: request.disruption_tags.clone(),
        orchestration_burden: request.orchestration_burden.clone(),
        issued_at: "2026-07-23T00:00:00Z".into(),
        source_evidence_refs: request.source_evidence_refs.clone(),
        adapter_attestation_ref: Some("att_a".into()),
        authored_by: request.authored_by.clone(),
        authorization_ref: request.authorization_ref.clone(),
        authorization_request_sha256: request.authorization_request_sha256.clone(),
        declaration_hash: "sha256:decl".into(),
    }
}

#[test]
fn an_appended_declaration_must_echo_the_request_it_was_made_from() {
    let request = declaration_request("crit_a");
    let mut declaration = echoing_declaration(&request);
    verify_declaration_echoes_request(&request, &declaration).unwrap();
    declaration.criteria_ref = "crit_b".into();
    let error = verify_declaration_echoes_request(&request, &declaration).unwrap_err();
    assert!(error.to_string().contains("criteria_ref"), "{error}");
}

#[test]
fn a_duplicate_request_returns_the_same_pending_record() {
    let root = tempfile::tempdir().unwrap();
    let refs_ab = vec!["evidence:a".to_string(), "evidence:b".to_string()];
    let refs_ba = vec!["evidence:b".to_string(), "evidence:a".to_string()];
    let (first, first_replayed) =
        pending_of(gate_swe_seed_in(root.path(), EXTERNAL_GATED, refs_ab).unwrap());
    let (second, second_replayed) =
        pending_of(gate_swe_seed_in(root.path(), EXTERNAL_GATED, refs_ba).unwrap());
    assert!(!first_replayed);
    assert!(
        second_replayed,
        "the identical request must replay, not re-record"
    );
    assert_eq!(first.pending_id, second.pending_id);
    assert_eq!(
        first, second,
        "the replayed record is the original, created_at included"
    );
    assert_eq!(settlement_pending_records(root.path()).unwrap().len(), 1);

    // A different request identity is a different pending record.
    let (other, _) = pending_of(
        gate_swe_seed_in(root.path(), EXTERNAL_GATED, vec!["evidence:c".into()]).unwrap(),
    );
    assert_ne!(other.pending_id, first.pending_id);
    assert_eq!(settlement_pending_records(root.path()).unwrap().len(), 2);
}

#[test]
fn no_settlement_declaration_is_authoritative_while_pending() {
    let root = tempfile::tempdir().unwrap();
    let gate = gate_swe_seed_in(root.path(), EXTERNAL_GATED, vec![]).unwrap();
    // Pending is not Allowed, so no caller may proceed to declare().
    assert!(matches!(gate, SettlementGate::Pending { .. }));
    let entries = LedgerStream::open(root.path(), LEDGER_ID, "test-reader")
        .unwrap()
        .read_entries()
        .unwrap();
    assert!(
        entries
            .iter()
            .all(|entry| entry.record_kind != "settlement_declaration"),
        "a pending settlement must not append a declaration"
    );
    for entry in entries
        .iter()
        .filter(|entry| entry.record_kind == "authority_decision")
    {
        let decision: AuthorityDecision = serde_json::from_value(entry.payload.clone()).unwrap();
        assert_ne!(
            decision.verdict,
            Verdict::Allow,
            "a pending settlement is never an allow"
        );
    }
}

// ---- Approval resolution, single consumption, and the resume state machine ----------------------------------

fn declared(gate: SettlementGate) -> (String, u32) {
    match gate {
        SettlementGate::Resumed {
            pending_id,
            attempt,
            ..
        } => (pending_id, attempt),
        other => panic!("expected a resumed settlement, got {other:?}"),
    }
}

#[test]
fn an_approved_settlement_resumes_once_and_a_second_resume_returns_the_completed_declaration() {
    let root = tempfile::tempdir().unwrap();
    let record = pending_for(root.path(), evidence_a());
    resolve_in(root.path(), &record.pending_id, true, &operator()).unwrap();
    assert_eq!(
        settlement_status(root.path(), &record.pending_id).unwrap(),
        SettlementStatus::Approved
    );

    let (pending_id, attempt) =
        declared(gate_swe_seed_in(root.path(), &approval_policy(), evidence_a()).unwrap());
    assert_eq!(
        (pending_id.as_str(), attempt),
        (record.pending_id.as_str(), 1)
    );
    complete_settlement_resume(root.path(), &pending_id, "decl-1").unwrap();

    // A second resume never claims again: it returns the completed declaration.
    match gate_swe_seed_in(root.path(), &approval_policy(), evidence_a()).unwrap() {
        SettlementGate::Completed {
            declaration_ref, ..
        } => assert_eq!(declaration_ref, "decl-1"),
        other => panic!("expected the completed declaration, got {other:?}"),
    }
    assert_eq!(
        ledger_count(root.path(), "settlement_state"),
        2,
        "resuming + completed, nothing more"
    );
}

#[test]
fn a_rejected_settlement_declares_nothing_and_stays_terminal() {
    let root = tempfile::tempdir().unwrap();
    let record = pending_for(root.path(), evidence_a());
    resolve_in(root.path(), &record.pending_id, false, &operator()).unwrap();
    for _ in 0..2 {
        assert!(gate_swe_seed_in(root.path(), &approval_policy(), evidence_a()).is_err());
    }
    assert_eq!(
        settlement_status(root.path(), &record.pending_id).unwrap(),
        SettlementStatus::Rejected
    );
    assert_eq!(
        ledger_count(root.path(), "settlement_state"),
        1,
        "one rejected terminal, written once"
    );
    // A rejected approval cannot be approved afterwards.
    assert!(resolve_in(root.path(), &record.pending_id, true, &operator()).is_err());
}

#[test]
fn an_expired_settlement_declares_nothing() {
    let root = tempfile::tempdir().unwrap();
    let short = gate_swe_seed_ttl(
        root.path(),
        CASE,
        &approval_policy(),
        evidence_a(),
        chrono::Duration::milliseconds(200),
    )
    .unwrap();
    let (record, _) = pending_of(short);
    std::thread::sleep(std::time::Duration::from_millis(400));
    // Approval after the window is refused, and so is any resume.
    assert!(resolve_in(root.path(), &record.pending_id, true, &operator()).is_err());
    assert!(gate_swe_seed_in(root.path(), &approval_policy(), evidence_a()).is_err());
    assert_eq!(
        settlement_status(root.path(), &record.pending_id).unwrap(),
        SettlementStatus::Expired
    );
    assert_eq!(
        ledger_count(root.path(), "settlement_state"),
        1,
        "one expired terminal"
    );
}

#[test]
fn the_requester_cannot_approve_their_own_pending_settlement() {
    let root = tempfile::tempdir().unwrap();
    let record = pending_for(root.path(), evidence_a());
    let err = resolve_in(root.path(), &record.pending_id, true, &svc()).unwrap_err();
    assert!(err.to_string().contains("self_approval_refused"), "{err}");
    assert_eq!(
        ledger_count(root.path(), "approval_self_resolution_refused"),
        1
    );
    assert_eq!(
        settlement_status(root.path(), &record.pending_id).unwrap(),
        SettlementStatus::Pending
    );
}

#[test]
fn a_non_operator_identity_cannot_approve_a_settlement() {
    let root = tempfile::tempdir().unwrap();
    let record = pending_for(root.path(), evidence_a());
    let other_service = Actor {
        actor_id: "other-service".into(),
        role: ActorRole::Service,
    };
    assert!(resolve_in(root.path(), &record.pending_id, true, &other_service).is_err());
    assert_eq!(
        settlement_status(root.path(), &record.pending_id).unwrap(),
        SettlementStatus::Pending
    );
}

#[test]
fn an_approval_does_not_bind_to_a_different_request_or_a_forged_pending_id() {
    let root = tempfile::tempdir().unwrap();
    let record = pending_for(root.path(), evidence_a());
    resolve_in(root.path(), &record.pending_id, true, &operator()).unwrap();

    // A different request identity is a different pending record: the approval never reaches it.
    match gate_swe_seed_in(root.path(), &approval_policy(), vec!["evidence:b".into()]).unwrap() {
        SettlementGate::Pending { record: other, .. } => {
            assert_ne!(other.pending_id, record.pending_id)
        }
        other => panic!("a mismatched request must not be resumed, got {other:?}"),
    }
    assert_eq!(
        settlement_status(root.path(), &record.pending_id).unwrap(),
        SettlementStatus::Approved
    );

    // A forged or unknown pending id is refused and audited.
    assert!(resolve_in(
        root.path(),
        "settlement-pending-00000000000000000000000000000000",
        true,
        &operator()
    )
    .is_err());
    assert_eq!(ledger_count(root.path(), "settlement_approval_refused"), 1);
}

#[test]
fn a_reused_approval_is_refused_and_audited() {
    let root = tempfile::tempdir().unwrap();
    let record = pending_for(root.path(), evidence_a());
    resolve_in(root.path(), &record.pending_id, true, &operator()).unwrap();
    let (pending_id, _) =
        declared(gate_swe_seed_in(root.path(), &approval_policy(), evidence_a()).unwrap());
    // The claim spent the approval: a second use is refused and audited while the first attempt is open.
    assert!(gate_swe_seed_in(root.path(), &approval_policy(), evidence_a()).is_err());
    assert_eq!(ledger_count(root.path(), "settlement_approval_refused"), 1);
    complete_settlement_resume(root.path(), &pending_id, "decl-1").unwrap();
    // After completion the request reports the completed declaration; it is not claimed again.
    assert!(matches!(
        gate_swe_seed_in(root.path(), &approval_policy(), evidence_a()).unwrap(),
        SettlementGate::Completed { .. }
    ));
    assert_eq!(ledger_count(root.path(), "settlement_approval_refused"), 1);
    // The resolution is written once: approving again is refused.
    assert!(resolve_in(root.path(), &record.pending_id, true, &operator()).is_err());
}

#[test]
fn concurrent_resumes_of_one_approval_declare_exactly_once() {
    let root = tempfile::tempdir().unwrap();
    let record = pending_for(root.path(), evidence_a());
    resolve_in(root.path(), &record.pending_id, true, &operator()).unwrap();
    let path = root.path().to_path_buf();
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let path = path.clone();
            std::thread::spawn(move || gate_swe_seed_in(&path, &approval_policy(), evidence_a()))
        })
        .collect();
    let outcomes: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let resumed = outcomes
        .iter()
        .filter(|o| matches!(o, Ok(SettlementGate::Resumed { .. })))
        .count();
    assert_eq!(
        resumed, 1,
        "exactly one resume may claim the approval: {outcomes:?}"
    );
    assert!(outcomes
        .iter()
        .all(|o| !matches!(o, Ok(SettlementGate::Completed { .. }))));
}

#[test]
fn a_crash_between_claim_and_completion_recovers_to_exactly_one_declaration() {
    let root = tempfile::tempdir().unwrap();
    let record = pending_for(root.path(), evidence_a());
    resolve_in(root.path(), &record.pending_id, true, &operator()).unwrap();
    // The process claims the approval, declares, and dies before recording completion.
    let (pending_id, _) =
        declared(gate_swe_seed_in(root.path(), &approval_policy(), evidence_a()).unwrap());
    assert!(matches!(
        settlement_status(root.path(), &pending_id).unwrap(),
        SettlementStatus::Resuming { attempt: 1 }
    ));
    // Recovery finds the declaration in the ledger and completes on it, once.
    let recovered = reconcile_settlement_resume(root.path(), &pending_id, Some("decl-1")).unwrap();
    assert_eq!(
        recovered,
        SettlementStatus::Completed {
            declaration_ref: "decl-1".into()
        }
    );
    let again = reconcile_settlement_resume(root.path(), &pending_id, Some("decl-1")).unwrap();
    assert_eq!(again, recovered);
    assert!(matches!(
        gate_swe_seed_in(root.path(), &approval_policy(), evidence_a()).unwrap(),
        SettlementGate::Completed { .. }
    ));
}

#[test]
fn a_crash_with_no_declaration_fails_the_attempt_and_declares_nothing() {
    let root = tempfile::tempdir().unwrap();
    let record = pending_for(root.path(), evidence_a());
    resolve_in(root.path(), &record.pending_id, true, &operator()).unwrap();
    let (pending_id, _) =
        declared(gate_swe_seed_in(root.path(), &approval_policy(), evidence_a()).unwrap());
    let recovered = reconcile_settlement_resume(root.path(), &pending_id, None).unwrap();
    assert_eq!(
        recovered,
        SettlementStatus::Failed {
            reason: "interrupted_before_declaration".into()
        }
    );
    assert!(gate_swe_seed_in(root.path(), &approval_policy(), evidence_a()).is_err());
    assert!(fail_settlement_resume(root.path(), &pending_id, "late").is_err());
}

#[test]
fn an_unavailable_authority_releases_the_attempt_for_a_later_resume() {
    let root = tempfile::tempdir().unwrap();
    let record = pending_for(root.path(), evidence_a());
    resolve_in(root.path(), &record.pending_id, true, &operator()).unwrap();
    let (pending_id, first) =
        declared(gate_swe_seed_in(root.path(), &approval_policy(), evidence_a()).unwrap());
    release_settlement_resume(root.path(), &pending_id, first).unwrap();
    assert_eq!(
        settlement_status(root.path(), &pending_id).unwrap(),
        SettlementStatus::Approved
    );
    let (_, second) =
        declared(gate_swe_seed_in(root.path(), &approval_policy(), evidence_a()).unwrap());
    assert_eq!(second, first + 1);
    complete_settlement_resume(root.path(), &pending_id, "decl-1").unwrap();
}

#[test]
fn identical_racing_requests_yield_one_pending_id() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().to_path_buf();
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let path = path.clone();
            std::thread::spawn(move || gate_swe_seed_in(&path, &approval_policy(), evidence_a()))
        })
        .collect();
    let ids: Vec<String> = handles
        .into_iter()
        .map(|h| match h.join().unwrap() {
            Ok(SettlementGate::Pending { record, .. }) => record.pending_id,
            other => panic!("every racing identical request must return pending, got {other:?}"),
        })
        .collect();
    assert!(ids.windows(2).all(|w| w[0] == w[1]), "{ids:?}");
    assert_eq!(settlement_pending_records(root.path()).unwrap().len(), 1);
}
