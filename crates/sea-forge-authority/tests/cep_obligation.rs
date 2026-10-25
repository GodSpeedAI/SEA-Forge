//! Obligation settlement from RealityTrace's own ledger (Horizon HORIZON-001 1.1.0, REQ-NAV-011/012): the
//! request's claims are recorded with the decision; the closure's C is SEA-Forge's own read of RealityTrace's
//! head, stamped by SEA-Forge's clock; only admissible, known results decide, by the REQ-NAV-012 table.
//! Real time with short windows: each test allows, submits execution evidence, then waits for the close.

use chrono::{DateTime, Duration, Utc};
use sea_forge_authority::cep::{
    argv_digest, CepAuthorityError, CepAuthorityService, ObligationSource, SettlementCriteria,
    SettlementOutcome, SourceRow,
};
use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_domainforge::{SeaSourceSet, SourceFile, WorldRegistry};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::cell::RefCell;

const SRC: &str = "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";
const LEDGER: &str = "rt-ledger-1";
const POLICY: &str = "version: \"0.1\"\nrules:\n  - name: a\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_action\n  - name: e\n    verdict: allow\n    actor_role: service\n    operation_kind: evidence_mutation\n  - name: s\n    verdict: allow\n    actor_role: service\n    operation_kind: settlement_declaration\n";

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

fn with<T>(c: &Cell, f: impl FnOnce(&CepAuthorityService<'_>) -> T) -> T {
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(POLICY).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let service = CepAuthorityService {
        engine: &engine,
        bundle: &bundle,
        worlds: &c.worlds,
        root: c.root.path(),
        approval_ttl: Duration::hours(24),
        criteria: SettlementCriteria::default(),
    };
    f(&service)
}

fn svc() -> Actor {
    Actor {
        actor_id: "cognate-service".into(),
        role: ActorRole::Service,
    }
}

/// A fake RealityTrace ledger the test can append to after the closure (to prove C is frozen).
struct FakeRt {
    rows: RefCell<Vec<SourceRow>>,
}

impl ObligationSource for FakeRt {
    fn ledger_rows(&self, ledger_id: &str) -> Result<Vec<SourceRow>, String> {
        Ok(self
            .rows
            .borrow()
            .iter()
            .filter(|r| r.ledger_id == ledger_id)
            .cloned()
            .collect())
    }
}

fn argv() -> Vec<Value> {
    vec![
        json!("/pinned/verifier"),
        json!("--intent"),
        json!("11111111-1111-4111-8111-111111111111"),
    ]
}

fn claims(closes_at: DateTime<Utc>, grace: i64) -> Value {
    json!({
        "modality": "obligation",
        "expectation_digest": format!("sha256:{}", "e".repeat(64)),
        "threshold_id": "thr-1",
        "threshold_ledger": LEDGER,
        "threshold_position": 3,
        "window_closes_at": closes_at.to_rfc3339(),
        "close_grace_s": grace,
        "verifier_argv_digest": argv_digest(&argv()),
    })
}

fn auth_request(world: &str, op: &str, claims: Value) -> Value {
    json!({
        "envelope_id": format!("env-authority_request-{op}"),
        "cep_version": "1.0.0", "envelope_version": "1.0", "envelope_kind": "authority_request",
        "created_at": "2026-10-23T12:00:00Z", "created_by": "cognate",
        "scope": {"evaluation_context": "cognate-authority_request", "world_ref": world},
        "boundary_record": {"scope": "s", "included_sections": ["authority"], "excluded_sections": [],
            "known_omissions": ["payload not carried"], "unknowns": [], "redactions": [],
            "compression_notes": [], "out_of_scope_entities": [], "classification": "internal", "limitations": []},
        "completeness_status": "partial", "omission_status": "marked",
        "provenance_refs": [format!("prov-{op}")], "validation_status": "valid", "lineage_refs": [],
        "provenance": [{"provenance_id": format!("prov-{op}"), "source_system_refs": ["cognate"],
            "producer_refs": ["cognate"], "production_method": "governed_operation", "created_at": "2026-10-23T12:00:00Z"}],
        "authority": [{"authority_id": "auth-1", "subject_actor_ref": "actor:alice"}],
        "transformations": [{"transformation_id": format!("tr-{op}"), "transformation_type": "capability_invocation"}],
        "references": [{"ref_id": "r1", "ref_type": "artifact_id", "target_uri_or_id": "horizon.write",
            "availability_status": "available", "integrity_status": "verifiable"}],
        "states": [{"state_id": "s1", "state_type": "declared_state", "subject_ref": "run:1"}],
        "extensions": {
            "cep.profile": {"profile_id": "godspeed.authority_request", "profile_version": "1.0.0"},
            "godspeed.authority_request": {"operation_id": op, "correlation_id": "corr-1",
                "operation_kind": "action", "operation_name": "run.start", "resource_id": "agent.echo", "claims": claims}
        }
    })
}

fn execution_packet(world: &str, op: &str) -> Value {
    json!({
        "envelope_id": format!("env-evidence_packet-{op}"),
        "cep_version": "1.0.0", "envelope_version": "1.0", "envelope_kind": "evidence_packet",
        "created_at": "2026-10-23T12:00:01Z", "created_by": "cognate-realitytrace-adapter",
        "scope": {"evaluation_context": "godspeed-evidence_packet", "world_ref": world},
        "boundary_record": {"scope": "execution evidence", "included_sections": ["evidence", "questions"],
            "excluded_sections": [], "known_omissions": ["only the execution trace"], "unknowns": [],
            "redactions": [], "compression_notes": [], "out_of_scope_entities": [], "classification": "internal", "limitations": []},
        "completeness_status": "partial", "omission_status": "marked",
        "provenance_refs": [format!("prov-ev-{op}")], "validation_status": "valid",
        "lineage_refs": [format!("env-execution_trace-{op}")],
        "provenance": [{"provenance_id": format!("prov-ev-{op}"), "source_system_refs": ["realitytrace"],
            "producer_refs": ["cognate-realitytrace-adapter"], "production_method": "evidence_packaging", "created_at": "2026-10-23T12:00:01Z"}],
        "questions": [{"question_id": "q-exec", "question_form": "Did the operation execute?"}],
        "evidence": [{"evidence_id": format!("ev-{op}"), "evidence_type": "execution_trace", "question_ref": "q-exec",
            "target_entity_ref": "concept:runtime_execution", "direction": "supports", "reliability": "medium"}],
        "extensions": {
            "cep.profile": {"profile_id": "godspeed.evidence_packet", "profile_version": "1.0.0"},
            "godspeed.evidence_packet": {"operation_id": op,
                "authority_decision_ref": format!("env-authority_decision-{op}"),
                "execution_trace_ref": format!("env-execution_trace-{op}")}
        }
    })
}

fn threshold_row(ordinal: u64, id: &str) -> SourceRow {
    SourceRow {
        ledger_id: LEDGER.into(),
        ordinal,
        record_kind: "threshold.declared".into(),
        payload: json!({"states": [{"state_id": id, "state_type": "settlement_threshold"}]}),
    }
}

#[derive(Clone)]
struct Run {
    exit: Option<i64>,
    started: DateTime<Utc>,
    completed: Option<DateTime<Utc>>,
    crashed: bool,
    argv: Vec<Value>,
}

fn obs_row(ordinal: u64, id: &str, run: &Run) -> SourceRow {
    let mut content = json!({
        "verifier_id": "command@1",
        "invocation": {"argv": run.argv, "launcher": [], "timeout_s": 30},
        "started_at": run.started.to_rfc3339(),
        "exit_code": run.exit,
        "crashed": run.crashed,
    });
    if let Some(done) = run.completed {
        content["completed_at"] = json!(done.to_rfc3339());
    }
    SourceRow {
        ledger_id: LEDGER.into(),
        ordinal,
        record_kind: "observation.recorded".into(),
        payload: json!({"observations": [{"observation_id": id}], "representations": [{"inline_content": content}]}),
    }
}

/// An allowed obligation with execution evidence submitted; `floor` is just after SEA-Forge received it.
struct Scenario {
    c: Cell,
    op: String,
    closes_at: DateTime<Utc>,
    floor: DateTime<Utc>,
    rt: FakeRt,
}

fn scenario(window_ms: i64, grace: i64) -> Scenario {
    let c = cell();
    let op = "op-1".to_string();
    let closes_at = Utc::now() + Duration::milliseconds(window_ms);
    with(&c, |s| {
        s.decide(
            &auth_request(&c.world, &op, claims(closes_at, grace)),
            &svc(),
        )
        .unwrap()
    });
    with(&c, |s| {
        s.submit_evidence(&execution_packet(&c.world, &op), &svc())
            .unwrap()
    });
    let floor = Utc::now() + Duration::milliseconds(50);
    Scenario {
        c,
        op,
        closes_at,
        floor,
        rt: FakeRt {
            rows: RefCell::new(vec![threshold_row(3, "thr-1")]),
        },
    }
}

impl Scenario {
    fn run(&self, exit: Option<i64>) -> Run {
        Run {
            exit,
            started: self.floor + Duration::milliseconds(10),
            completed: Some(self.floor + Duration::milliseconds(20)),
            crashed: false,
            argv: argv(),
        }
    }
    /// A run that started and completed before SEA-Forge received the execution evidence.
    fn baseline(&self, exit: Option<i64>) -> Run {
        Run {
            exit,
            started: self.floor - Duration::seconds(5),
            completed: Some(self.floor - Duration::seconds(4)),
            crashed: false,
            argv: argv(),
        }
    }
    fn add(&self, row: SourceRow) {
        self.rt.rows.borrow_mut().push(row);
    }
    /// Discrimination at B = 5 (did not observe the statement) plus verification runs from ordinal 7.
    fn standard(&self, runs: &[Run]) -> Vec<String> {
        self.add(obs_row(
            5,
            "obs-d",
            &Run {
                exit: Some(1),
                started: Utc::now() - Duration::seconds(5),
                completed: Some(Utc::now() - Duration::seconds(4)),
                crashed: false,
                argv: argv(),
            },
        ));
        runs.iter()
            .enumerate()
            .map(|(i, run)| {
                let id = format!("obs-v{i}");
                self.add(obs_row(7 + i as u64, &id, run));
                id
            })
            .collect()
    }
    fn wait_close(&self) {
        let remaining = self.closes_at - Utc::now() + Duration::milliseconds(20);
        if remaining > Duration::zero() {
            std::thread::sleep(remaining.to_std().unwrap());
        }
    }
    fn close(
        &self,
        ids: &[String],
    ) -> Result<sea_forge_authority::cep::ClosureAck, CepAuthorityError> {
        with(&self.c, |s| {
            s.record_closure(&self.op, "obs-d", ids, &svc(), &self.rt)
        })
    }
    fn settle(&self) -> Result<SettlementOutcome, CepAuthorityError> {
        with(&self.c, |s| s.settle_obligation(&self.op, &svc(), &self.rt))
    }
    /// Close on time and settle.
    fn verdict(&self, runs: &[Run]) -> (String, String) {
        let ids = self.standard(runs);
        self.wait_close();
        self.close(&ids).unwrap();
        epistemic(&self.settle().unwrap())
    }
}

fn epistemic(o: &SettlementOutcome) -> (String, String) {
    let ext = &o.envelope["extensions"]["godspeed.settlement_packet"];
    (
        ext["epistemic_status"].as_str().unwrap().to_string(),
        o.status.to_string(),
    )
}

const W: i64 = 1200;

#[test]
fn the_requests_claims_are_recorded_with_the_decision() {
    let c = cell();
    let closes_at = Utc::now() + Duration::seconds(60);
    let d = with(&c, |s| {
        s.decide(
            &auth_request(&c.world, "op-1", claims(closes_at, 30)),
            &svc(),
        )
        .unwrap()
    });
    let action = serde_json::to_value(&d.decision.action_request.action).unwrap();
    assert_eq!(action["parameters"]["claims"], claims(closes_at, 30));
}

#[test]
fn malformed_claims_are_refused_before_any_decision() {
    let c = cell();
    let err = with(&c, |s| {
        s.decide(
            &auth_request(&c.world, "op-1", json!({"nested": {"no": true}})),
            &svc(),
        )
        .unwrap_err()
    });
    assert_eq!(err.class(), "cep_request_invalid");
    let err = with(&c, |s| {
        s.decide(
            &auth_request(&c.world, "op-2", json!({"Bad-Key": "x"})),
            &svc(),
        )
        .unwrap_err()
    });
    assert!(err.to_string().contains("must match"), "{err}");
}

#[test]
fn known_zero_is_supported_and_settled_once() {
    let s = scenario(W, 30);
    assert_eq!(
        s.verdict(&[s.run(Some(0))]),
        ("supported".into(), "settled".into())
    );
    let again = s.settle().unwrap();
    assert!(again.replayed && again.final_status);
}

#[test]
fn known_nonzero_is_contradicted_and_rejected() {
    let s = scenario(W, 30);
    assert_eq!(
        s.verdict(&[s.run(Some(2))]),
        ("contradicted".into(), "rejected".into())
    );
}

#[test]
fn both_known_results_are_inconsistent_and_unsettled() {
    let s = scenario(W, 30);
    assert_eq!(
        s.verdict(&[s.run(Some(0)), s.run(Some(1))]),
        ("inconsistent".into(), "unsettled".into())
    );
}

#[test]
fn only_undefined_results_are_unresolved_and_rejected() {
    let s = scenario(W, 30);
    let timeout = Run {
        completed: None,
        ..s.run(None)
    };
    let crashed = Run {
        crashed: true,
        ..s.run(Some(0))
    };
    assert_eq!(
        s.verdict(&[timeout, crashed]),
        ("unresolved".into(), "rejected".into())
    );
}

#[test]
fn a_known_zero_alongside_undefined_runs_is_supported() {
    let s = scenario(W, 30);
    assert_eq!(
        s.verdict(&[s.run(None), s.run(Some(0))]),
        ("supported".into(), "settled".into())
    );
}

#[test]
fn a_wrong_argv_or_a_late_or_missing_completion_is_undefined() {
    let s = scenario(W, 30);
    let wrong_intent = Run {
        argv: vec![
            json!("/pinned/verifier"),
            json!("--intent"),
            json!("22222222-2222-4222-8222-222222222222"),
        ],
        ..s.run(Some(0))
    };
    let late = Run {
        completed: Some(s.closes_at + Duration::milliseconds(1)),
        ..s.run(Some(0))
    };
    let missing = Run {
        completed: None,
        ..s.run(Some(0))
    };
    assert_eq!(
        s.verdict(&[wrong_intent, late, missing]),
        ("unresolved".into(), "rejected".into())
    );
}

#[test]
fn a_run_started_before_execution_evidence_is_not_admissible() {
    let s = scenario(W, 30);
    let early = Run {
        started: s.floor - Duration::seconds(2),
        ..s.run(Some(0))
    };
    assert_eq!(
        s.verdict(&[early]),
        ("unresolved".into(), "rejected".into())
    );
}

#[test]
fn c_is_read_at_closure_so_a_run_appended_later_is_excluded() {
    let s = scenario(W, 30);
    let ids = s.standard(&[s.run(Some(1))]);
    s.wait_close();
    let ack = s.close(&[ids[0].clone(), "obs-late".into()]).unwrap();
    assert_eq!((ack.b, ack.c, ack.on_time), (5, 7, true));
    // A passing run appended after C: named in the closure, but beyond C, so it cannot rescue the result.
    s.add(obs_row(9, "obs-late", &s.run(Some(0))));
    assert_eq!(
        epistemic(&s.settle().unwrap()),
        ("contradicted".into(), "rejected".into())
    );
}

#[test]
fn no_closure_by_deadline_plus_grace_is_unresolved_even_with_a_passing_run() {
    let s = scenario(W, 0);
    s.standard(&[s.run(Some(0))]);
    s.wait_close();
    std::thread::sleep(std::time::Duration::from_millis(300));
    assert_eq!(
        epistemic(&s.settle().unwrap()),
        ("unresolved".into(), "rejected".into())
    );
}

#[test]
fn a_closure_after_deadline_plus_grace_cannot_prove_c_on_time() {
    let s = scenario(W, 0);
    let ids = s.standard(&[s.run(Some(0))]);
    s.wait_close();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let ack = s.close(&ids).unwrap();
    assert!(!ack.on_time);
    assert_eq!(
        epistemic(&s.settle().unwrap()),
        ("unresolved".into(), "rejected".into())
    );
}

#[test]
fn before_the_window_closes_closure_and_settlement_are_pending() {
    let s = scenario(60_000, 30);
    let ids = s.standard(&[s.run(Some(0))]);
    assert!(s
        .close(&ids)
        .unwrap_err()
        .to_string()
        .contains("still open"));
    let err = s.settle().unwrap_err();
    assert!(err.to_string().contains("pending"), "{err}");
    assert!(
        matches!(err, CepAuthorityError::SettlementPending(_)),
        "{err:?}"
    );
    assert_eq!(err.class(), "cep_settlement_pending");
}

#[test]
fn after_the_window_closes_but_before_the_grace_ends_an_unrecorded_closure_is_pending() {
    // Window closed, grace still running, no closure recorded yet: retry later, nothing written.
    let s = scenario(W, 30);
    s.standard(&[s.run(Some(0))]);
    s.wait_close();
    let err = s.settle().unwrap_err();
    assert!(
        err.to_string()
            .contains("closure has not been recorded yet"),
        "{err}"
    );
    assert!(
        matches!(err, CepAuthorityError::SettlementPending(_)),
        "{err:?}"
    );
    assert_eq!(err.class(), "cep_settlement_pending");
}

#[test]
fn a_threshold_that_differs_from_realitytraces_record_is_refused() {
    let s = scenario(W, 30);
    s.rt.rows.borrow_mut()[0] = threshold_row(3, "thr-other");
    let ids = s.standard(&[s.run(Some(0))]);
    s.wait_close();
    s.close(&ids).unwrap();
    let err = s.settle().unwrap_err();
    assert!(err.to_string().contains("do not match its record"), "{err}");
    // A genuine evidence refusal is still a refusal, not a retry-later pending.
    assert!(matches!(err, CepAuthorityError::Evidence(_)), "{err:?}");
    assert_eq!(err.class(), "cep_evidence_refused");
}

#[test]
fn a_discrimination_run_that_already_observed_the_statement_is_refused() {
    let s = scenario(W, 30);
    s.add(obs_row(5, "obs-d", &s.baseline(Some(0))));
    s.add(obs_row(7, "obs-v0", &s.run(Some(0))));
    s.wait_close();
    // Refused before any closure is frozen, so nothing can settle on it.
    assert!(s
        .close(&["obs-v0".into()])
        .unwrap_err()
        .to_string()
        .contains("before acting"));
    let err = s.settle().unwrap_err();
    assert!(err.to_string().contains("pending"), "{err}");
    assert_eq!(err.class(), "cep_settlement_pending");
}

/// False success: a failing run proves "not observed before acting" only if it is a completed, uncrashed run
/// of exactly the pinned verifier argv. Any other failing observation after T is refused at closure.
#[test]
fn a_discrimination_run_of_another_command_crashed_or_incomplete_is_refused_at_closure() {
    type Spoil = fn(Run) -> Run;
    let cases: [(&str, Spoil); 3] = [
        ("pinned verifier argv", |r| Run {
            argv: vec![json!("/bin/false")],
            ..r
        }),
        ("crashed", |r| Run {
            crashed: true,
            exit: None,
            ..r
        }),
        ("no recorded completion", |r| Run {
            completed: None,
            ..r
        }),
    ];
    for (why, bad) in cases {
        let s = scenario(W, 30);
        s.add(obs_row(5, "obs-d", &bad(s.baseline(Some(1)))));
        s.add(obs_row(7, "obs-v0", &s.run(Some(0))));
        s.wait_close();
        let err = s.close(&["obs-v0".into()]).unwrap_err().to_string();
        assert!(err.contains(why), "{why}: {err}");
        let settled = s.settle().unwrap_err().to_string();
        assert!(
            settled.contains("pending"),
            "{why}: no closure, no settlement: {settled}"
        );
    }
}

/// A failing run that completed after the operation executed is not a baseline before acting; otherwise a later
/// passing verification would settle an effect that may already have held.
#[test]
fn a_discrimination_run_completed_after_execution_is_refused_at_closure() {
    let s = scenario(W, 30);
    s.add(obs_row(5, "obs-d", &s.run(Some(1))));
    s.add(obs_row(7, "obs-v0", &s.run(Some(0))));
    s.wait_close();
    let err = s.close(&["obs-v0".into()]).unwrap_err().to_string();
    assert!(
        err.contains("completed after the execution evidence"),
        "{err}"
    );
    assert!(s.settle().unwrap_err().to_string().contains("pending"));
}

#[test]
fn a_discrimination_run_not_after_the_threshold_is_refused_at_closure() {
    let s = scenario(W, 30);
    s.add(obs_row(
        2,
        "obs-d",
        &Run {
            exit: Some(1),
            ..s.run(Some(1))
        },
    ));
    s.wait_close();
    assert!(s
        .close(&[])
        .unwrap_err()
        .to_string()
        .contains("not after the threshold"));
}

#[test]
fn prohibition_is_not_supported() {
    let c = cell();
    let mut prohibition = claims(Utc::now() - Duration::seconds(1), 30);
    prohibition["modality"] = json!("prohibition");
    with(&c, |s| {
        s.decide(&auth_request(&c.world, "op-1", prohibition), &svc())
            .unwrap()
    });
    let rt = FakeRt {
        rows: RefCell::new(vec![]),
    };
    let err = with(&c, |s| {
        s.settle_obligation("op-1", &svc(), &rt).unwrap_err()
    });
    assert!(err.to_string().contains("not supported"), "{err}");
}

#[test]
fn an_obligation_without_execution_evidence_is_refused() {
    let c = cell();
    let closes_at = Utc::now() + Duration::milliseconds(300);
    // No grace, so the missing closure is no longer "pending" when settlement is asked for.
    with(&c, |s| {
        s.decide(
            &auth_request(&c.world, "op-1", claims(closes_at, 0)),
            &svc(),
        )
        .unwrap()
    });
    std::thread::sleep(std::time::Duration::from_millis(400));
    let rt = FakeRt {
        rows: RefCell::new(vec![threshold_row(3, "thr-1")]),
    };
    let err = with(&c, |s| {
        s.settle_obligation("op-1", &svc(), &rt).unwrap_err()
    });
    assert!(err.to_string().contains("no execution evidence"), "{err}");
}
