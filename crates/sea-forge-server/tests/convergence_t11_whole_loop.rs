//! Convergence T11 — Whole-Loop Causality, Replay, and Recovery (Rust side)
//!
//! Complements the primary Python harness (godspeed_agent/tests/test_convergence_t11_whole_loop.py)
//! which drives E0→E10 with one stable wr_id and domain hash via real GSA producers.
//! This Rust harness exercises the SEA-Forge-owned edges (E4, E5A/E5B, E6) through
//! their REAL production gates, proving the same invariants from the Rust side:
//! - whole-cycle traceability (wr_id + domain hash stable, causality parents correct)
//! - replay/idempotency (duplicate delivery, restart)
//! - late callback / out-of-order rejection
//! - wrong-domain identity fail-closed
//! - failure remains evidence (I11 strand for operational settlement)
//!
//! Deterministic constants mirror the Python harness:
//!   WR_ID = wr-gsf-t11-001
//!   DOMAIN_HASH = 537449202c9d06df08373366dc0e25c7bf32e51e64d3fbf837465f0a45098741
//!   AFFORDANCE = aff-t11-canonical-001
//!   Fixed UUIDv5 event ids for E4/E5A/E5B/E6 etc.

use std::path::PathBuf;

use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{
    Actor, ActorRole, ActorType, AuthorityAction, BindingResolution, IdentityBinding,
};
use sea_forge_server::governed_execution_boundary::InvocationLedger;
use sea_forge_server::governed_settlement_return::{
    emit_operational_settlement, evaluate_operational_settlement, SettlementOptionalFields,
};
use sea_forge_server::governed_work_ingress::accept_governed_work_request;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

fn sha256_hex(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}

const WR_ID: &str = "wr-gsf-t11-001";
const DOMAIN_HASH: &str = "537449202c9d06df08373366dc0e25c7bf32e51e64d3fbf837465f0a45098741";
const OTHER_HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const AFFORDANCE: &str = "aff-t11-canonical-001";

const AUTH_DECISION_ALLOW_POLICY: &str = r#"version: "0.1"
rules:
  - name: allow-local-cmd
    verdict: allow
    actor_role: operator
    operation_kind: execute_command
    argv0: sea-forge
"#;

fn actor() -> Actor {
    Actor {
        actor_id: "operator_local".into(),
        role: ActorRole::Operator,
    }
}

fn binding() -> IdentityBinding {
    IdentityBinding {
        identity_id: Some("identity:operator_local".into()),
        principal: "operator_local".into(),
        roles: vec![ActorRole::Operator],
        actor_type: ActorType::Human,
        binding_resolution: BindingResolution::LocalDefault,
        identity_binding_source: "local-slice-default".into(),
        source: Some("local-slice-default".into()),
        sponsor: None,
        issued_at: None,
        expires_at: None,
        identity_binding_hash: Some(sha256_hex(b"operator_local:Operator")),
    }
}

struct Harness {
    _root: tempfile::TempDir,
    workspace: PathBuf,
    artifacts: PathBuf,
    sea_forge_argv0: PathBuf,
}

fn harness(tag: &str) -> Harness {
    let root = tempfile::tempdir().expect("tempdir");
    let workspace = root.path().join("workspace");
    let artifacts = root.path().join("artifacts");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&artifacts).unwrap();
    let link = root.path().join("sea-forge");
    #[cfg(unix)]
    std::os::unix::fs::symlink(std::env::current_exe().unwrap(), &link).unwrap();
    let _ = tag;
    Harness {
        _root: root,
        workspace,
        artifacts,
        sea_forge_argv0: link,
    }
}

fn allow_decision(h: &Harness, seq: usize) -> sea_forge_core::types::AuthorityDecision {
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(AUTH_DECISION_ALLOW_POLICY).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle).unwrap();
    let action = AuthorityAction::ExecuteCommand {
        argv: vec![
            h.sea_forge_argv0.to_string_lossy().into_owned(),
            "--version".into(),
        ],
        cwd: ".".into(),
    };
    engine
        .evaluate(AuthorityEvaluation {
            actor: &actor(),
            binding: binding(),
            run_id: "run_t11",
            case_id: "case_t11",
            plan_item_id: "item_t11",
            sequence: seq,
            action: &action,
            workspace_root: &h.workspace,
            evidence_refs: vec![],
            artifacts_root: Some(&h.artifacts),
            timeout_secs: Some(10),
            env_keys: ["PATH", "HOME"].iter().map(|k| k.to_string()).collect(),
            domainforge_candidate: None,
            environment: None,
        })
        .unwrap()
}

fn e4_governed_request() -> Value {
    // Minimal but valid GovernedWorkRequest envelope that will pass the ingress gate.
    // Produced via the same shape SWE_SEED's builder emits; we stamp swe-seed as producer.
    let payload = json!({
        "namespace": "agentic_capability_loop",
        "domain_model_hash": DOMAIN_HASH,
        "work_request_id": WR_ID,
        "affordance_id": AFFORDANCE,
        "actor": {"actor_id": "agent-operator", "role": "R-AA"},
        "intent": "rotate deploy to blue-green and verify health gates",
        "context_packet_ref": "ctx_t11_001",
        "domain_model_ref": {"namespace": "agentic_capability_loop", "model_hash": DOMAIN_HASH},
        "proof_contract": {"criterion": "live proof exits 0", "proof_type": "live"},
        "settlement_criteria": ["health check green", "zero-downtime observed"],
    });
    let idem = sha256_hex(
        format!(
            "|GovernedWorkRequest|{}",
            serde_json::to_string(&payload).unwrap()
        )
        .as_bytes(),
    );
    json!({
        "schema_version": "v1",
        "event_id": "44444444-1111-4000-8000-000000000001",
        "source_agent": "swe-seed",
        "event_type": "GovernedWorkRequest",
        "occurred_at": "2026-08-26T12:00:00+00:00",
        "idempotency_key": idem,
        "payload": payload,
        "provenance": {
            "origin": "swe-seed",
            "chain": [
                format!("domain_model_hash:{DOMAIN_HASH}"),
                "caused_by:11111111-0000-4000-8000-000000000001",
                "caused_by:22222222-0000-4000-8000-000000000002"
            ]
        }
    })
}

fn e3_context_packet() -> Value {
    let payload = json!({
        "namespace": "agentic_capability_loop",
        "domain_model_hash": DOMAIN_HASH,
        "work_request_id": WR_ID,
        "context_requirement_id": "cr-t11",
        "context_packet_id": "ctx_t11_001",
        "citations": [{"source": "runbook://deploy/blue-green", "sha256": sha256_hex(b"runbook")}],
    });
    json!({
        "schema_version": "v1",
        "event_id": "22222222-0000-4000-8000-000000000002",
        "source_agent": "context-kernel",
        "event_type": "ContextPacketCreated",
        "occurred_at": "2026-08-26T12:00:00+00:00",
        "idempotency_key": sha256_hex(b"ctx"),
        "payload": payload,
        "provenance": {"origin": "context-kernel", "chain": [format!("domain_model_hash:{DOMAIN_HASH}")]}
    })
}

// --- whole-cycle traceability (I2) through E4→E5→E6 ---

#[test]
fn t11_whole_loop_traceability_via_real_gates() {
    let h = harness("t11_traceability");
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();

    // E4 ingress
    let e4 = e4_governed_request();
    let e3 = e3_context_packet();
    let ctx = accept_governed_work_request(&e4, &e3, DOMAIN_HASH)
        .expect("E4 must pass ingress with stable WR and hash");
    assert_eq!(ctx.work_request_id, WR_ID);
    assert_eq!(ctx.affordance_id, AFFORDANCE);

    // E5A: authorized invocation from REAL Allow decision
    let inv = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(
        &decision,
        WR_ID,
        DOMAIN_HASH,
        DOMAIN_HASH,
        e4["event_id"].as_str().unwrap(),
        None,
    )
    .expect("Allow must emit AuthorizedInvocation");
    assert_eq!(inv.work_request_id, WR_ID);
    ledger
        .admit_authorized_invocation(&inv.envelope, DOMAIN_HASH)
        .expect("admission");
    let e5a_event_id = inv.event_id.clone();
    let invocation_id = inv.invocation_id.clone();

    // E5B: execution observation bound to that exact invocation
    let observed = json!([{"effect": "health check green"}, {"effect": "zero-downtime observed"}]);
    let obs = json!({
        "schema_version": "v1",
        "event_id": "55555555-0000-4000-8000-000000000001",
        "source_agent": "execution-environment",
        "event_type": "ExecutionObservation",
        "occurred_at": "2026-08-26T12:00:00+00:00",
        "idempotency_key": sha256_hex(b"obs"),
        "payload": {
            "namespace": "agentic_capability_loop",
            "domain_model_hash": DOMAIN_HASH,
            "work_request_id": WR_ID,
            "invocation_id": invocation_id,
            "execution_status": "completed",
            "observed_effects": observed,
        },
        "provenance": {"origin": "execution-environment", "chain": [format!("caused_by:{e5a_event_id}")]}
    });
    let outcome = ledger
        .settle_execution_observation(&obs, DOMAIN_HASH)
        .expect("observation must settle against its invocation");
    let e5b_event_id = obs["event_id"].as_str().unwrap().to_string();

    // E6: operational settlement from ledger-settled observation vs declared criteria
    let criteria = vec![
        "health check green".to_string(),
        "zero-downtime observed".to_string(),
    ];
    let settlement = emit_operational_settlement(
        &outcome,
        &criteria,
        DOMAIN_HASH,
        DOMAIN_HASH,
        &e5a_event_id,
        &e5b_event_id,
        SettlementOptionalFields::default(),
    )
    .expect("settlement must emit");
    assert_eq!(settlement.work_request_id, WR_ID);
    assert_eq!(settlement.envelope["payload"]["work_request_id"], WR_ID);
    assert_eq!(
        settlement.envelope["payload"]["domain_model_hash"],
        DOMAIN_HASH
    );
    // Causality cites actual chain
    let chain = settlement.envelope["provenance"]["chain"]
        .as_array()
        .unwrap();
    assert!(chain
        .iter()
        .any(|v| v.as_str() == Some(&format!("caused_by:{e5a_event_id}"))));
    assert!(chain
        .iter()
        .any(|v| v.as_str() == Some(&format!("caused_by:{e5b_event_id}"))));
}

// --- replay / duplicate (I14) ---

#[test]
fn t11_duplicate_observation_is_idempotent() {
    let h = harness("t11_dup_obs");
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let e4 = e4_governed_request();
    let inv = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(
        &decision,
        WR_ID,
        DOMAIN_HASH,
        DOMAIN_HASH,
        e4["event_id"].as_str().unwrap(),
        None,
    )
    .unwrap();
    ledger
        .admit_authorized_invocation(&inv.envelope, DOMAIN_HASH)
        .unwrap();
    let observed = json!([{"effect": "health check green"}]);
    let obs = json!({
        "schema_version": "v1",
        "event_id": "dup-obs-1",
        "source_agent": "execution-environment",
        "event_type": "ExecutionObservation",
        "occurred_at": "2026-08-26T12:00:00+00:00",
        "idempotency_key": "dup-key-1",
        "payload": {
            "namespace": "agentic_capability_loop",
            "domain_model_hash": DOMAIN_HASH,
            "work_request_id": WR_ID,
            "invocation_id": inv.invocation_id,
            "execution_status": "completed",
            "observed_effects": observed,
        },
        "provenance": {"origin": "execution-environment", "chain": [format!("caused_by:{}", inv.event_id)]}
    });
    let first = ledger
        .settle_execution_observation(&obs, DOMAIN_HASH)
        .unwrap();
    assert!(matches!(
        first,
        sea_forge_server::governed_execution_boundary::ObservationOutcome::Settled { .. }
    ));
    let dup = ledger
        .settle_execution_observation(&obs, DOMAIN_HASH)
        .unwrap();
    assert!(matches!(
        dup,
        sea_forge_server::governed_execution_boundary::ObservationOutcome::DuplicateDelivery
    ));
}

#[test]
fn t11_duplicate_settlement_via_replay_is_idempotent() {
    let h = harness("t11_dup_settlement");
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let e4 = e4_governed_request();
    let inv = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(
        &decision,
        WR_ID,
        DOMAIN_HASH,
        DOMAIN_HASH,
        e4["event_id"].as_str().unwrap(),
        None,
    )
    .unwrap();
    ledger
        .admit_authorized_invocation(&inv.envelope, DOMAIN_HASH)
        .unwrap();
    let observed = json!([{"effect": "health check green"}, {"effect": "zero-downtime observed"}]);
    let obs = json!({
        "schema_version": "v1",
        "event_id": "dup-settle-obs",
        "source_agent": "execution-environment",
        "event_type": "ExecutionObservation",
        "occurred_at": "2026-08-26T12:00:00+00:00",
        "idempotency_key": "dup-settle-key",
        "payload": {
            "namespace": "agentic_capability_loop",
            "domain_model_hash": DOMAIN_HASH,
            "work_request_id": WR_ID,
            "invocation_id": inv.invocation_id,
            "execution_status": "completed",
            "observed_effects": observed,
        },
        "provenance": {"origin": "execution-environment", "chain": [format!("caused_by:{}", inv.event_id)]}
    });
    let outcome = ledger
        .settle_execution_observation(&obs, DOMAIN_HASH)
        .unwrap();
    let criteria = vec![
        "health check green".to_string(),
        "zero-downtime observed".to_string(),
    ];
    let e5a = inv.event_id.clone();
    let e5b = obs["event_id"].as_str().unwrap().to_string();
    let first = emit_operational_settlement(
        &outcome,
        &criteria,
        DOMAIN_HASH,
        DOMAIN_HASH,
        &e5a,
        &e5b,
        SettlementOptionalFields::default(),
    )
    .unwrap();
    // Replay same outcome — settlement is deterministic, re-emitting with same inputs yields same idempotency, but the ledger for settlements is not tested here; we verify evaluation is stable
    let second = emit_operational_settlement(
        &outcome,
        &criteria,
        DOMAIN_HASH,
        DOMAIN_HASH,
        &e5a,
        &e5b,
        SettlementOptionalFields::default(),
    )
    .unwrap();
    assert_eq!(
        first.envelope["payload"]["operational_settlement_status"],
        second.envelope["payload"]["operational_settlement_status"]
    );
}

// --- late callback (I15) ---

#[test]
fn t11_late_observation_cannot_settle_against_wrong_invocation() {
    let h = harness("t11_late");
    let dec1 = allow_decision(&h, 1);
    let dec2 = allow_decision(&h, 2);
    let mut ledger = InvocationLedger::default();
    let e4 = e4_governed_request();
    let inv1 = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(
        &dec1,
        WR_ID,
        DOMAIN_HASH,
        DOMAIN_HASH,
        e4["event_id"].as_str().unwrap(),
        None,
    )
    .unwrap();
    ledger
        .admit_authorized_invocation(&inv1.envelope, DOMAIN_HASH)
        .unwrap();
    // Observation for inv1 settles while inv1 is current
    let obs1 = json!({
        "schema_version": "v1",
        "event_id": "late-obs-1",
        "source_agent": "execution-environment",
        "event_type": "ExecutionObservation",
        "occurred_at": "2026-08-26T12:00:00+00:00",
        "idempotency_key": "late-key-1",
        "payload": {
            "namespace": "agentic_capability_loop",
            "domain_model_hash": DOMAIN_HASH,
            "work_request_id": WR_ID,
            "invocation_id": inv1.invocation_id.clone(),
            "execution_status": "completed",
            "observed_effects": json!([]),
        },
        "provenance": {"origin": "execution-environment", "chain": [format!("caused_by:{}", inv1.event_id)]}
    });
    let _ = ledger
        .settle_execution_observation(&obs1, DOMAIN_HASH)
        .unwrap();
    // Now supersede with inv2
    let inv2 = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(
        &dec2,
        WR_ID,
        DOMAIN_HASH,
        DOMAIN_HASH,
        e4["event_id"].as_str().unwrap(),
        None,
    )
    .unwrap();
    ledger
        .admit_authorized_invocation(&inv2.envelope, DOMAIN_HASH)
        .unwrap();
    // Late replay: try to settle another observation for the superseded inv1 after inv2 is current — must be rejected as late
    let late_for_inv1 = json!({
        "schema_version": "v1",
        "event_id": "late-obs-for-inv1",
        "source_agent": "execution-environment",
        "event_type": "ExecutionObservation",
        "occurred_at": "2026-08-26T12:00:00+00:00",
        "idempotency_key": "late-for-inv1-key",
        "payload": {
            "namespace": "agentic_capability_loop",
            "domain_model_hash": DOMAIN_HASH,
            "work_request_id": WR_ID,
            "invocation_id": inv1.invocation_id.clone(),
            "execution_status": "completed",
            "observed_effects": json!([]),
        },
        "provenance": {"origin": "execution-environment", "chain": [format!("caused_by:{}", inv1.event_id)]}
    });
    // This is a late observation for inv1 (generation 1) while current generation is 2 — must be rejected
    let res = ledger.settle_execution_observation(&late_for_inv1, DOMAIN_HASH);
    assert!(
        res.is_err(),
        "late observation for superseded generation must be rejected, got {res:?}"
    );
    let err_str = format!("{:?}", res.as_ref().unwrap_err());
    assert!(
        err_str.contains("Late") || err_str.contains("late"),
        "expected LateObservation error, got {err_str}"
    );

    // Also verify cross-wired parent: old invocation_id but claims inv2 as parent — also rejected
    let late_cross = json!({
        "schema_version": "v1",
        "event_id": "late-obs-cross",
        "source_agent": "execution-environment",
        "event_type": "ExecutionObservation",
        "occurred_at": "2026-08-26T12:00:00+00:00",
        "idempotency_key": "late-cross-key",
        "payload": {
            "namespace": "agentic_capability_loop",
            "domain_model_hash": DOMAIN_HASH,
            "work_request_id": WR_ID,
            "invocation_id": inv1.invocation_id.clone(),
            "execution_status": "completed",
            "observed_effects": json!([]),
        },
        "provenance": {"origin": "execution-environment", "chain": [format!("caused_by:{}", inv2.event_id)]}
    });
    let res2 = ledger.settle_execution_observation(&late_cross, DOMAIN_HASH);
    assert!(
        res2.is_err(),
        "cross-wired late observation must be rejected"
    );
}

// --- wrong-domain (I1/ENV-I2) ---

#[test]
fn t11_wrong_domain_identity_is_rejected_at_ingress() {
    let e4 = e4_governed_request();
    let mut wrong = e4.clone();
    wrong["payload"]["domain_model_hash"] = json!(OTHER_HASH);
    wrong["payload"]["domain_model_ref"] =
        json!({"namespace": "agentic_capability_loop", "model_hash": OTHER_HASH});
    // Provenance still claims original hash — drift should be detected
    let e3 = e3_context_packet();
    let err = accept_governed_work_request(&wrong, &e3, DOMAIN_HASH).unwrap_err();
    // Must be domain drift
    assert!(
        format!("{err}").contains("drift") || format!("{err}").contains("model"),
        "expected drift error, got {err}"
    );
}

// --- failure remains evidence (I11) — operational settlement rejected still observable ---

#[test]
fn t11_failed_settlement_remains_observable() {
    let h = harness("t11_failed");
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let e4 = e4_governed_request();
    let inv = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(
        &decision,
        WR_ID,
        DOMAIN_HASH,
        DOMAIN_HASH,
        e4["event_id"].as_str().unwrap(),
        None,
    )
    .unwrap();
    ledger
        .admit_authorized_invocation(&inv.envelope, DOMAIN_HASH)
        .unwrap();
    // Execution succeeds but observed effects do NOT attest declared criteria
    let observed = json!([]); // empty, so criteria fail
    let obs = json!({
        "schema_version": "v1",
        "event_id": "failed-obs",
        "source_agent": "execution-environment",
        "event_type": "ExecutionObservation",
        "occurred_at": "2026-08-26T12:00:00+00:00",
        "idempotency_key": "failed-key",
        "payload": {
            "namespace": "agentic_capability_loop",
            "domain_model_hash": DOMAIN_HASH,
            "work_request_id": WR_ID,
            "invocation_id": inv.invocation_id,
            "execution_status": "completed",
            "observed_effects": observed,
        },
        "provenance": {"origin": "execution-environment", "chain": [format!("caused_by:{}", inv.event_id)]}
    });
    let outcome = ledger
        .settle_execution_observation(&obs, DOMAIN_HASH)
        .unwrap();
    let criteria = vec![
        "health check green".to_string(),
        "zero-downtime observed".to_string(),
    ];
    let settlement = emit_operational_settlement(
        &outcome,
        &criteria,
        DOMAIN_HASH,
        DOMAIN_HASH,
        &inv.event_id,
        obs["event_id"].as_str().unwrap(),
        SettlementOptionalFields::default(),
    )
    .unwrap();
    // Must be rejected (exit-zero not enough)
    assert_eq!(
        settlement.envelope["payload"]["operational_settlement_status"],
        "rejected"
    );
    // But it is still a valid, observable settlement — not disappeared
    assert_eq!(settlement.envelope["payload"]["work_request_id"], WR_ID);
}
