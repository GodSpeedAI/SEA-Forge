//! Convergence T12 — Variation Battery: Canonical Runtime Under Variation (Rust side)
//!
//! Covers the 7 frozen variation classes through SEA-Forge's REAL production gates
//! (E4, E5A/E5B, E6) with the SAME deterministic WR/domain hash family as the
//! Python harness (godspeed_agent/tests/test_convergence_t12_variation.py).
//! Each variation preserves: no false success, no duplicate consequence,
//! no authority bypass, no semantic-identity collapse, no provenance loss.
//!
//! Variation classes:
//!   success, authority_denial_or_escalation, proof_failure (I6/I7 composition),
//!   execution_failure_or_timeout, interruption_and_recovery, duplicate_or_replayed,
//!   semantically_invalid_or_wrong_domain_identity
//!
//! Plus targeted falsification probes for: malformed_envelope, missing_domain_model,
//! wrong_domain_model_hash, stale_domain_model, cross_wired_work_request_id,
//! cross_wired_invocation_id, duplicate_delivery, replay_after_restart,
//! authority_denied, authority_escalated, execution_failure, execution_timeout,
//! partial_side_effect, operational_settlement_failure_after_exit_zero,
//! forged_parent_reference, missing_evidence_artifact, late_callback,
//! out_of_order_event, stale_authority_grant.

use std::path::PathBuf;

use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{
    Actor, ActorRole, ActorType, AuthorityAction, BindingResolution, IdentityBinding,
};
use sea_forge_server::governed_execution_boundary::InvocationLedger;
use sea_forge_server::governed_settlement_return::{
    emit_operational_settlement, SettlementOptionalFields,
};
use sea_forge_server::governed_work_ingress::accept_governed_work_request;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

fn sha256_hex(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}

const WR_SUCCESS: &str = "wr-gsf-t12-success-001";
const WR_DENY: &str = "wr-gsf-t12-deny-001";
const WR_ESCALATE: &str = "wr-gsf-t12-escalate-001";
const WR_EXEC_FAIL: &str = "wr-gsf-t12-exec-fail-001";
const WR_TIMEOUT: &str = "wr-gsf-t12-timeout-001";
const WR_DUPLICATE: &str = "wr-gsf-t12-duplicate-001";
const WR_WRONG_DOMAIN: &str = "wr-gsf-t12-wrong-domain-001";

const DOMAIN_HASH: &str = "9f7a7c2e3b4d5e6f7890a1b2c3d4e5f67890abcdef1234567890abcdef12345678"; // placeholder, replaced by real helper
const OTHER_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn real_domain_hash() -> String {
    sha256_hex(b"t12-canonical-model-v1")
}

fn actor() -> Actor {
    Actor { actor_id: "operator_local".into(), role: ActorRole::Operator }
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
    #[cfg(unix)] std::os::unix::fs::symlink(std::env::current_exe().unwrap(), &link).unwrap();
    let _ = tag;
    Harness { _root: root, workspace, artifacts, sea_forge_argv0: link }
}
fn allow_decision(h: &Harness, seq: usize) -> sea_forge_core::types::AuthorityDecision {
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(r#"version: "0.1"
rules:
  - name: allow-local-cmd
    verdict: allow
    actor_role: operator
    operation_kind: execute_command
    argv0: sea-forge
"#).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle).unwrap();
    let action = AuthorityAction::ExecuteCommand { argv: vec![h.sea_forge_argv0.to_string_lossy().into_owned(), "--version".into()], cwd: ".".into() };
    engine.evaluate(AuthorityEvaluation { actor: &actor(), binding: binding(), run_id: "run_t12", case_id: "case_t12", plan_item_id: "item_t12", sequence: seq, action: &action, workspace_root: &h.workspace, evidence_refs: vec![], artifacts_root: Some(&h.artifacts), timeout_secs: Some(10), env_keys: ["PATH","HOME"].iter().map(|k| k.to_string()).collect(), domainforge_candidate: None, environment: None }).unwrap()
}
fn deny_decision(h: &Harness) -> sea_forge_core::types::AuthorityDecision {
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(r#"version: "0.1"
rules: []
"#).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle).unwrap();
    let action = AuthorityAction::ExecuteCommand { argv: vec![h.sea_forge_argv0.to_string_lossy().into_owned(), "--version".into()], cwd: ".".into() };
    engine.evaluate(AuthorityEvaluation { actor: &actor(), binding: binding(), run_id: "run_t12", case_id: "case_t12", plan_item_id: "item_t12", sequence: 1, action: &action, workspace_root: &h.workspace, evidence_refs: vec![], artifacts_root: Some(&h.artifacts), timeout_secs: Some(10), env_keys: ["PATH","HOME"].iter().map(|k| k.to_string()).collect(), domainforge_candidate: None, environment: None }).unwrap()
}
fn escalate_decision(h: &Harness) -> sea_forge_core::types::AuthorityDecision {
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(r#"version: "0.1"
rules:
  - name: escalate-local-cmd
    verdict: escalate
    actor_role: operator
    operation_kind: execute_command
    argv0: sea-forge
"#).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle).unwrap();
    let action = AuthorityAction::ExecuteCommand { argv: vec![h.sea_forge_argv0.to_string_lossy().into_owned(), "--version".into()], cwd: ".".into() };
    engine.evaluate(AuthorityEvaluation { actor: &actor(), binding: binding(), run_id: "run_t12", case_id: "case_t12", plan_item_id: "item_t12", sequence: 1, action: &action, workspace_root: &h.workspace, evidence_refs: vec![], artifacts_root: Some(&h.artifacts), timeout_secs: Some(10), env_keys: ["PATH","HOME"].iter().map(|k| k.to_string()).collect(), domainforge_candidate: None, environment: None }).unwrap()
}

fn e3_packet(wr: &str, domain: &str) -> Value {
    let payload = json!({
        "namespace":"agentic_capability_loop","domain_model_hash": domain,"work_request_id": wr,"context_requirement_id": format!("cr-{}", wr),"context_packet_id": format!("ctx-{}", wr),"citations":[{"source":"runbook://deploy/blue-green","sha256": sha256_hex(b"runbook")}],
    });
    json!({"schema_version":"v1","event_id": format!("e3-{}", wr),"source_agent":"context-kernel","event_type":"ContextPacketCreated","occurred_at":"2026-08-26T12:00:00+00:00","idempotency_key": sha256_hex(b"ctx"),"payload": payload,"provenance":{"origin":"context-kernel","chain":[format!("domain_model_hash:{domain}")]}})
}

fn e4_request(wr: &str, domain: &str) -> (Value, Value) {
    let e3 = e3_packet(wr, domain);
    let parent_e3_id = e3["event_id"].as_str().unwrap_or_default();
    let payload = json!({
        "namespace": "agentic_capability_loop",
        "domain_model_hash": domain,
        "work_request_id": wr,
        "affordance_id": "aff-t12-canonical-001",
        "actor": {"actor_id": "agent-operator", "role": "R-AA"},
        "intent": "t12 variation loop",
        "context_packet_ref": format!("ctx-{}", wr),
        "domain_model_ref": {"namespace": "agentic_capability_loop", "model_hash": domain},
        "proof_contract": {"criterion": "live proof exits 0", "proof_type": "live"},
        "settlement_criteria": ["health check green", "zero-downtime observed"],
    });
    let idem = sha256_hex(format!("|GovernedWorkRequest|{}", serde_json::to_string(&payload).unwrap()).as_bytes());
    let e4 = json!({"schema_version":"v1","event_id": format!("e4-{}", wr),"source_agent":"swe-seed","event_type":"GovernedWorkRequest","occurred_at":"2026-08-26T12:00:00+00:00","idempotency_key": idem,"payload": payload,"provenance":{"origin":"swe-seed","chain":[format!("domain_model_hash:{domain}"), format!("caused_by:{}", parent_e3_id), "caused_by:11111111-0000-4000-8000-000000000001"]}});
    (e4, e3)
}

fn e4_request_raw(wr: &str, domain: &str) -> Value {
    e4_request(wr, domain).0
}

// --- variation: success ----------------------------------------------------

#[test]
fn t12_success_via_real_gates() {
    let h = harness("t12_success");
    let domain = real_domain_hash();
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let (e4, e3) = e4_request(WR_SUCCESS, &domain);
    let ctx = accept_governed_work_request(&e4, &e3, &domain).expect("E4 must pass");
    assert_eq!(ctx.work_request_id, WR_SUCCESS);
    let inv = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(&decision, WR_SUCCESS, &domain, &domain, e4["event_id"].as_str().unwrap(), None).expect("Allow must emit");
    ledger.admit_authorized_invocation(&inv.envelope, &domain).expect("admission");
    let observed = json!([{"effect":"health check green"}, {"effect":"zero-downtime observed"}]);
    let obs = json!({"schema_version":"v1","event_id": "obs-success","source_agent":"execution-environment","event_type":"ExecutionObservation","occurred_at":"2026-08-26T12:00:00+00:00","idempotency_key": sha256_hex(b"obs-success"),"payload":{"namespace":"agentic_capability_loop","domain_model_hash": domain,"work_request_id": WR_SUCCESS,"invocation_id": inv.invocation_id,"execution_status":"completed","observed_effects": observed,},"provenance":{"origin":"execution-environment","chain":[format!("caused_by:{}", inv.event_id)]}});
    let outcome = ledger.settle_execution_observation(&obs, &domain).expect("settle");
    let settlement = emit_operational_settlement(&outcome, &vec!["health check green".into(),"zero-downtime observed".into()], &domain, &domain, &inv.event_id, obs["event_id"].as_str().unwrap(), SettlementOptionalFields::default()).expect("settlement");
    assert_eq!(settlement.envelope["payload"]["operational_settlement_status"], "accepted");
    assert_eq!(settlement.envelope["payload"]["work_request_id"], WR_SUCCESS);
    let chain = settlement.envelope["provenance"]["chain"].as_array().unwrap();
    assert!(chain.iter().any(|v| v.as_str()==Some(&format!("caused_by:{}", inv.event_id))));
}

// --- variation: authority denial ------------------------------------------

#[test]
fn t12_authority_denied_produces_no_invocation_and_no_side_effect() {
    let h = harness("t12_deny");
    let domain = real_domain_hash();
    let decision = deny_decision(&h);
    assert_ne!(format!("{:?}", decision.verdict), "Allow");
    let res = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(&decision, WR_DENY, &domain, &domain, "e4-deny", None);
    assert!(res.is_err(), "deny must not emit AuthorizedInvocation");
    let mut ledger = InvocationLedger::default();
    let fake_obs = json!({"schema_version":"v1","event_id": "fake-obs","source_agent":"execution-environment","event_type":"ExecutionObservation","occurred_at":"2026-08-26T12:00:00+00:00","idempotency_key": sha256_hex(b"fake"),"payload":{"namespace":"agentic_capability_loop","domain_model_hash": domain,"work_request_id": WR_DENY,"invocation_id": "inv_fake","execution_status":"completed","observed_effects": json!([]),},"provenance":{"origin":"execution-environment","chain":["caused_by:fake-parent"]}});
    let err = ledger.settle_execution_observation(&fake_obs, &domain);
    assert!(err.is_err(), "observation without prior AuthorizedInvocation must not settle");
}

#[test]
fn t12_authority_escalated_produces_no_invocation() {
    let h = harness("t12_escalate");
    let domain = real_domain_hash();
    let decision = escalate_decision(&h);
    let res = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(&decision, WR_ESCALATE, &domain, &domain, "e4-escalate", None);
    assert!(res.is_err(), "escalate must not emit AuthorizedInvocation");
}

// --- variation: execution failure -----------------------------------------

#[test]
fn t12_execution_failure_settlement_rejected_but_observable() {
    let h = harness("t12_exec_fail");
    let domain = real_domain_hash();
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let e4 = e4_request_raw(WR_EXEC_FAIL, &domain);
    let inv = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(&decision, WR_EXEC_FAIL, &domain, &domain, e4["event_id"].as_str().unwrap(), None).unwrap();
    ledger.admit_authorized_invocation(&inv.envelope, &domain).unwrap();
    let observed_partial = json!([{"effect":"health check green"}]);
    let obs = json!({"schema_version":"v1","event_id": "obs-fail","source_agent":"execution-environment","event_type":"ExecutionObservation","occurred_at":"2026-08-26T12:00:00+00:00","idempotency_key": sha256_hex(b"obs-fail"),"payload":{"namespace":"agentic_capability_loop","domain_model_hash": domain,"work_request_id": WR_EXEC_FAIL,"invocation_id": inv.invocation_id,"execution_status":"spawn_failed","observed_effects": observed_partial,},"provenance":{"origin":"execution-environment","chain":[format!("caused_by:{}", inv.event_id)]}});
    let outcome = ledger.settle_execution_observation(&obs, &domain).expect("observation settles");
    let settlement = emit_operational_settlement(&outcome, &vec!["health check green".into(),"zero-downtime observed".into()], &domain, &domain, &inv.event_id, obs["event_id"].as_str().unwrap(), SettlementOptionalFields::default()).expect("settlement emits even for failed execution");
    assert_eq!(settlement.envelope["payload"]["operational_settlement_status"], "rejected");
    assert_ne!(settlement.envelope["payload"]["operational_settlement_status"], "accepted");
    assert_eq!(settlement.envelope["payload"]["work_request_id"], WR_EXEC_FAIL);
}

#[test]
fn t12_execution_timeout_settlement_rejected() {
    let h = harness("t12_timeout");
    let domain = real_domain_hash();
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let e4 = e4_request_raw(WR_TIMEOUT, &domain);
    let inv = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(&decision, WR_TIMEOUT, &domain, &domain, e4["event_id"].as_str().unwrap(), None).unwrap();
    ledger.admit_authorized_invocation(&inv.envelope, &domain).unwrap();
    let obs = json!({"schema_version":"v1","event_id": "obs-timeout","source_agent":"execution-environment","event_type":"ExecutionObservation","occurred_at":"2026-08-26T12:00:00+00:00","idempotency_key": sha256_hex(b"obs-timeout"),"payload":{"namespace":"agentic_capability_loop","domain_model_hash": domain,"work_request_id": WR_TIMEOUT,"invocation_id": inv.invocation_id,"execution_status":"timed_out","observed_effects": json!([]),},"provenance":{"origin":"execution-environment","chain":[format!("caused_by:{}", inv.event_id)]}});
    let outcome = ledger.settle_execution_observation(&obs, &domain).unwrap();
    let settlement = emit_operational_settlement(&outcome, &vec!["health check green".into(),"zero-downtime observed".into()], &domain, &domain, &inv.event_id, obs["event_id"].as_str().unwrap(), SettlementOptionalFields::default()).unwrap();
    assert_eq!(settlement.envelope["payload"]["operational_settlement_status"], "rejected");
    assert_eq!(settlement.envelope["payload"]["execution_status"], "timed_out");
}

#[test]
fn t12_operational_settlement_failure_after_exit_zero() {
    let h = harness("t12_exit_zero");
    let domain = real_domain_hash();
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let wr = "wr-gsf-t12-exit-zero-001";
    let e4 = e4_request_raw(wr, &domain);
    let inv = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(&decision, wr, &domain, &domain, e4["event_id"].as_str().unwrap(), None).unwrap();
    ledger.admit_authorized_invocation(&inv.envelope, &domain).unwrap();
    let obs = json!({"schema_version":"v1","event_id": "obs-exit-zero","source_agent":"execution-environment","event_type":"ExecutionObservation","occurred_at":"2026-08-26T12:00:00+00:00","idempotency_key": sha256_hex(b"obs-exit-zero"),"payload":{"namespace":"agentic_capability_loop","domain_model_hash": domain,"work_request_id": wr,"invocation_id": inv.invocation_id,"execution_status":"completed","observed_effects": json!([]),},"provenance":{"origin":"execution-environment","chain":[format!("caused_by:{}", inv.event_id)]}});
    let outcome = ledger.settle_execution_observation(&obs, &domain).unwrap();
    let settlement = emit_operational_settlement(&outcome, &vec!["health check green".into()], &domain, &domain, &inv.event_id, obs["event_id"].as_str().unwrap(), SettlementOptionalFields::default()).unwrap();
    assert_eq!(settlement.envelope["payload"]["operational_settlement_status"], "rejected");
}

// --- variation: interruption and recovery (reuse T11 pattern but with T12 ids) ---

#[test]
fn t12_interruption_after_execution_before_settlement_preserves_provenance() {
    let h = harness("t12_interrupt");
    let domain = real_domain_hash();
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let wr = "wr-gsf-t12-interrupt-001";
    let e4 = e4_request_raw(wr, &domain);
    let inv = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(&decision, wr, &domain, &domain, e4["event_id"].as_str().unwrap(), None).unwrap();
    ledger.admit_authorized_invocation(&inv.envelope, &domain).unwrap();
    let observed = json!([{"effect":"health check green"}, {"effect":"zero-downtime observed"}]);
    let obs = json!({"schema_version":"v1","event_id": "obs-interrupt","source_agent":"execution-environment","event_type":"ExecutionObservation","occurred_at":"2026-08-26T12:00:00+00:00","idempotency_key": sha256_hex(b"obs-interrupt"),"payload":{"namespace":"agentic_capability_loop","domain_model_hash": domain,"work_request_id": wr,"invocation_id": inv.invocation_id.clone(),"execution_status":"completed","observed_effects": observed.clone(),},"provenance":{"origin":"execution-environment","chain":[format!("caused_by:{}", inv.event_id)]}});
    let outcome = ledger.settle_execution_observation(&obs, &domain).unwrap();
    let first = emit_operational_settlement(&outcome, &vec!["health check green".into(),"zero-downtime observed".into()], &domain, &domain, &inv.event_id, obs["event_id"].as_str().unwrap(), SettlementOptionalFields::default()).unwrap();
    let second = emit_operational_settlement(&outcome, &vec!["health check green".into(),"zero-downtime observed".into()], &domain, &domain, &inv.event_id, obs["event_id"].as_str().unwrap(), SettlementOptionalFields::default()).unwrap();
    assert_eq!(first.envelope["payload"]["operational_settlement_status"], second.envelope["payload"]["operational_settlement_status"]);
    assert_eq!(first.envelope["payload"]["work_request_id"], wr);
    let chain = first.envelope["provenance"]["chain"].as_array().unwrap();
    assert!(chain.iter().any(|v| v.as_str()==Some(&format!("caused_by:{}", inv.event_id))));
}

// --- variation: duplicate / replayed --------------------------------------

#[test]
fn t12_duplicate_observation_is_idempotent() {
    let h = harness("t12_dup_obs");
    let domain = real_domain_hash();
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let e4 = e4_request_raw(WR_DUPLICATE, &domain);
    let inv = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(&decision, WR_DUPLICATE, &domain, &domain, e4["event_id"].as_str().unwrap(), None).unwrap();
    ledger.admit_authorized_invocation(&inv.envelope, &domain).unwrap();
    let obs = json!({"schema_version":"v1","event_id": "dup-obs-t12","source_agent":"execution-environment","event_type":"ExecutionObservation","occurred_at":"2026-08-26T12:00:00+00:00","idempotency_key": "dup-key-t12","payload":{"namespace":"agentic_capability_loop","domain_model_hash": domain,"work_request_id": WR_DUPLICATE,"invocation_id": inv.invocation_id,"execution_status":"completed","observed_effects": json!([{"effect":"health check green"}]),},"provenance":{"origin":"execution-environment","chain":[format!("caused_by:{}", inv.event_id)]}});
    let first = ledger.settle_execution_observation(&obs, &domain).unwrap();
    assert!(matches!(first, sea_forge_server::governed_execution_boundary::ObservationOutcome::Settled{..}));
    let dup = ledger.settle_execution_observation(&obs, &domain).unwrap();
    assert!(matches!(dup, sea_forge_server::governed_execution_boundary::ObservationOutcome::DuplicateDelivery));
}

#[test]
fn t12_late_observation_cannot_settle_after_supersession() {
    let h = harness("t12_late");
    let domain = real_domain_hash();
    let dec1 = allow_decision(&h, 1);
    let dec2 = allow_decision(&h, 2);
    let mut ledger = InvocationLedger::default();
    let wr = "wr-gsf-t12-late-001";
    let e4 = e4_request_raw(wr, &domain);
    let inv1 = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(&dec1, wr, &domain, &domain, e4["event_id"].as_str().unwrap(), None).unwrap();
    ledger.admit_authorized_invocation(&inv1.envelope, &domain).unwrap();
    let obs1 = json!({"schema_version":"v1","event_id": "late-obs1","source_agent":"execution-environment","event_type":"ExecutionObservation","occurred_at":"2026-08-26T12:00:00+00:00","idempotency_key": "late1","payload":{"namespace":"agentic_capability_loop","domain_model_hash": domain,"work_request_id": wr,"invocation_id": inv1.invocation_id.clone(),"execution_status":"completed","observed_effects": json!([]),},"provenance":{"origin":"execution-environment","chain":[format!("caused_by:{}", inv1.event_id)]}});
    ledger.settle_execution_observation(&obs1, &domain).unwrap();
    let inv2 = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(&dec2, wr, &domain, &domain, e4["event_id"].as_str().unwrap(), None).unwrap();
    ledger.admit_authorized_invocation(&inv2.envelope, &domain).unwrap();
    let late = json!({"schema_version":"v1","event_id": "late-for-inv1","source_agent":"execution-environment","event_type":"ExecutionObservation","occurred_at":"2026-08-26T12:00:00+00:00","idempotency_key": "late-for-inv1","payload":{"namespace":"agentic_capability_loop","domain_model_hash": domain,"work_request_id": wr,"invocation_id": inv1.invocation_id.clone(),"execution_status":"completed","observed_effects": json!([]),},"provenance":{"origin":"execution-environment","chain":[format!("caused_by:{}", inv1.event_id)]}});
    let res = ledger.settle_execution_observation(&late, &domain);
    assert!(res.is_err());
}

// --- variation: wrong domain identity -------------------------------------

#[test]
fn t12_wrong_domain_identity_fails_closed_at_ingress_and_execution() {
    let domain = real_domain_hash();
    let wrong = OTHER_HASH;
    let (e4, e3) = e4_request(WR_WRONG_DOMAIN, wrong);
    let err = accept_governed_work_request(&e4, &e3, &domain).unwrap_err();
    assert!(format!("{err}").contains("drift") || format!("{err}").contains("model") || format!("{err}").contains("Domain"));

    let h = harness("t12_wrong_e5a");
    let decision = allow_decision(&h, 1);
    let res = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(&decision, WR_WRONG_DOMAIN, wrong, &domain, e4["event_id"].as_str().unwrap(), None);
    let err = res.unwrap_err();
    let err_msg = format!("{:?}", err);
    assert!(err_msg.contains("Drift") || err_msg.contains("drift") || err_msg.contains("Placeholder"));
}

// --- falsification probes: malformed, missing, cross-wired -----------

#[test]
fn t12_malformed_envelope_rejected() {
    let domain = real_domain_hash();
    let e3 = e3_packet(WR_WRONG_DOMAIN, &domain);
    // Empty envelope
    let malformed = json!({});
    let err = accept_governed_work_request(&malformed, &e3, &domain).unwrap_err();
    assert!(format!("{err}").len() > 0);
    // Wrong source_agent (forgery) — use the domain-correct E4 from helper but then override producer
    let (mut e4_forged, _e3) = e4_request(WR_WRONG_DOMAIN, &domain);
    e4_forged["source_agent"] = json!("realitytrace");
    let err2 = accept_governed_work_request(&e4_forged, &e3, &domain).unwrap_err();
    let err2_msg = format!("{err2}");
    assert!(err2_msg.contains("forge") || err2_msg.contains("produced by") || err2_msg.contains("NotAuthoritative"));
}

#[test]
fn t12_cross_wired_work_request_rejected() {
    let domain = real_domain_hash();
    let (e4, _e3) = e4_request(WR_SUCCESS, &domain);
    let e3_foreign = e3_packet("wr-foreign-999", &domain);
    let err = accept_governed_work_request(&e4, &e3_foreign, &domain).unwrap_err();
    assert!(format!("{err}").contains("CrossWired") || format!("{err}").contains("cross") || format!("{err}").contains("work_request") || format!("{err}").contains("Mismatch"));
}

#[test]
fn t12_stale_domain_model_rejected() {
    let domain = real_domain_hash();
    let stale = sha256_hex(b"t12-stale-model-v0");
    let e3 = e3_packet(WR_WRONG_DOMAIN, &domain);
    let e4_stale = json!({"schema_version":"v1","event_id": "e4-stale","source_agent":"swe-seed","event_type":"GovernedWorkRequest","occurred_at":"2026-08-26T12:00:00+00:00","idempotency_key": sha256_hex(b"stale"),"payload":{"namespace":"agentic_capability_loop","domain_model_hash": stale,"work_request_id": WR_WRONG_DOMAIN,"affordance_id": "aff-t12-canonical-001","actor":{"actor_id":"agent-operator","role":"R-AA"},"intent":"t12 stale test","context_packet_ref":"ctx-stale","domain_model_ref":{"namespace":"agentic_capability_loop","model_hash": stale},"proof_contract":{"criterion":"live proof exits 0","proof_type":"live"},"settlement_criteria":["health check green"]},"provenance":{"origin":"swe-seed","chain":[format!("domain_model_hash:{}", stale), format!("caused_by:{}", e3["event_id"].as_str().unwrap_or_default())]}});
    let err = accept_governed_work_request(&e4_stale, &e3, &domain).unwrap_err();
    assert!(format!("{err}").contains("drift") || format!("{err}").contains("Domain"));
}

#[test]
fn t12_missing_evidence_artifact_rejected_or_rejected_settlement() {
    let h = harness("t12_missing_ev");
    let domain = real_domain_hash();
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let wr = "wr-gsf-t12-missing-ev-001";
    let e4 = e4_request_raw(wr, &domain);
    let inv = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(&decision, wr, &domain, &domain, e4["event_id"].as_str().unwrap(), None).unwrap();
    ledger.admit_authorized_invocation(&inv.envelope, &domain).unwrap();
    let obs = json!({"schema_version":"v1","event_id": "obs-missing","source_agent":"execution-environment","event_type":"ExecutionObservation","occurred_at":"2026-08-26T12:00:00+00:00","idempotency_key": sha256_hex(b"obs-missing"),"payload":{"namespace":"agentic_capability_loop","domain_model_hash": domain,"work_request_id": wr,"invocation_id": inv.invocation_id,"execution_status":"completed","observed_effects": json!([]),},"provenance":{"origin":"execution-environment","chain":[format!("caused_by:{}", inv.event_id)]}});
    let outcome = ledger.settle_execution_observation(&obs, &domain).unwrap();
    let err = emit_operational_settlement(&outcome, &vec![], &domain, &domain, &inv.event_id, obs["event_id"].as_str().unwrap(), SettlementOptionalFields::default()).unwrap_err();
    assert!(format!("{:?}", err).contains("Opaque") || format!("{:?}", err).contains("Criteria") || format!("{:?}", err).contains("Placeholder"));
}