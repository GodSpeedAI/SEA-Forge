//! Convergence plan T06 — operational settlement return at the SEA-Forge
//! boundary (frozen edge E6, invariants I6/I7).
//!
//! Settlements are evaluated from ledger-settled ExecutionObservations
//! (real `InvocationLedger`, real authority decisions via the production
//! engine) against the DECLARED settlement criteria, and projected into the
//! canonical v1 OperationalSettlement envelope. Everything here is wire-shape
//! JSON, exactly what crosses back to SWE_SEED.
//!
//! Plan teeth:
//!   TOOTH 1 exit code 0 while declared criteria fail ⇒ settlement records
//!          failure/non-success (never success)
//!   TOOTH 2 is machine-checked on the SWE_SEED receiving side
//!           (`convergence_t06_operational_settlement.rs` there): consuming
//!           OperationalSettlement cannot promote developmental state.

use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{
    Actor, ActorRole, ActorType, AuthorityAction, AuthorityDecision, BindingResolution,
    IdentityBinding, SettlementStatus,
};
use sea_forge_server::governed_execution_boundary::InvocationLedger;
use sea_forge_server::governed_settlement_return::{
    emit_operational_settlement, evaluate_operational_settlement,
    validate_operational_settlement_wire, SettlementOptionalFields, SettlementReturnError,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

fn sha256_hex(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}

/// Independent reimplementation of the family's canonical payload form:
/// sorted-key compact JSON with `signature` excluded. Deliberately NOT
/// imported from the library — the wire-family check recomputes from the
/// published contract.
fn independent_canonical_payload_json(value: &Value) -> String {
    fn sorted_without_signature(value: &Value) -> Value {
        match value {
            Value::Object(obj) => {
                let mut keys: Vec<_> = obj.keys().collect();
                keys.sort();
                let mut sorted = serde_json::Map::new();
                for key in keys {
                    if key == "signature" {
                        continue;
                    }
                    sorted.insert(key.clone(), sorted_without_signature(&obj[key]));
                }
                Value::Object(sorted)
            }
            Value::Array(items) => {
                Value::Array(items.iter().map(sorted_without_signature).collect())
            }
            other => other.clone(),
        }
    }
    serde_json::to_string(&sorted_without_signature(value)).unwrap_or_default()
}

fn local_model_sha256() -> String {
    sha256_hex(b"canonical-model-t06-settlement-return")
}

/// The standalone-fallback pseudo-hash (ENV-I2 placeholder).
fn fallback_pseudo_hash() -> String {
    sha256_hex(b"agentic_capability_loop")
}

const ALLOW_POLICY: &str = "version: \"0.1\"\nrules:\n  - name: allow-local-cmd\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n";

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
    workspace: std::path::PathBuf,
    artifacts: std::path::PathBuf,
    /// Symlink named `sea-forge` -> this test binary (conformance-suite
    /// convention): satisfies the policy's argv0 match.
    sea_forge_argv0: std::path::PathBuf,
}

fn harness(name: &str) -> Harness {
    let root = tempfile::tempdir().expect("tempdir");
    let workspace = root.path().join("workspace");
    let artifacts = root.path().join("artifacts");
    std::fs::create_dir_all(&workspace).expect("workspace");
    std::fs::create_dir_all(&artifacts).expect("artifacts");
    let link = root.path().join("sea-forge");
    #[cfg(unix)]
    std::os::unix::fs::symlink(std::env::current_exe().expect("current_exe"), &link)
        .expect("sea-forge symlink");
    let _ = name;
    Harness {
        _root: root,
        workspace,
        artifacts,
        sea_forge_argv0: link,
    }
}

/// One REAL Allow decision through the production engine.
fn allow_decision(h: &Harness, sequence: usize) -> AuthorityDecision {
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(ALLOW_POLICY).expect("policy yaml");
    let engine = PolicyAuthorityEngine::new(bundle).expect("engine");
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
            run_id: "run_t06_return",
            case_id: "case_t06_return",
            plan_item_id: "item_t06_return",
            sequence,
            action: &action,
            workspace_root: &h.workspace,
            evidence_refs: vec![],
            artifacts_root: Some(&h.artifacts),
            timeout_secs: Some(10),
            env_keys: ["PATH", "HOME"].iter().map(|k| k.to_string()).collect(),
            domainforge_candidate: None,
            environment: None,
        })
        .expect("evaluation")
}

/// Emit + admit one authorized invocation into a fresh ledger slot.
fn admit(
    registry: &mut InvocationLedger,
    decision: &AuthorityDecision,
    work_request_id: &str,
    governed_parent: &str,
) -> (String, String) {
    let invocation = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(
        decision,
        work_request_id,
        &local_model_sha256(),
        &local_model_sha256(),
        governed_parent,
        None,
    )
    .expect("Allow decision must emit");
    registry
        .admit_authorized_invocation(&invocation.envelope, &local_model_sha256())
        .expect("admission");
    (invocation.invocation_id, invocation.event_id)
}

/// A well-formed ExecutionObservation bound to the given invocation.
fn observation(
    invocation_id: &str,
    e5a_event_id: &str,
    work_request_id: &str,
    status: &str,
    observed_effects: Value,
    extra_payload: Value,
) -> Value {
    let mut payload = json!({
        "domain_model_hash": local_model_sha256(),
        "namespace": "agentic_capability_loop",
        "work_request_id": work_request_id,
        "invocation_id": invocation_id,
        "execution_status": status,
        "observed_effects": observed_effects,
    });
    if let (Some(base), Some(extra)) = (payload.as_object_mut(), extra_payload.as_object()) {
        for (k, v) in extra {
            base.insert(k.clone(), v.clone());
        }
    }
    json!({
        "schema_version": "v1",
        "event_id": sha256_hex(format!("obs-{invocation_id}-{status}").as_bytes())[..32].to_string(),
        "source_agent": "execution-environment",
        "event_type": "ExecutionObservation",
        "occurred_at": "2026-08-25T12:00:00+00:00",
        "idempotency_key": sha256_hex(
            format!("|ExecutionObservation|{work_request_id}|{invocation_id}|{status}").as_bytes(),
        ),
        "payload": payload,
        "provenance": {
            "origin": "execution-environment",
            "chain": [format!("caused_by:{e5a_event_id}")]
        }
    })
}

/// Drive the REAL T05 boundary to a settled observation.
fn settle(
    registry: &mut InvocationLedger,
    obs: &Value,
) -> sea_forge_server::governed_execution_boundary::ObservationOutcome {
    registry
        .settle_execution_observation(obs, &local_model_sha256())
        .expect("observation must settle")
}

fn declared_criteria() -> Vec<String> {
    vec!["health check green".into(), "zero-downtime observed".into()]
}

/// Observed effects attesting every declared criterion.
fn attesting_effects() -> Value {
    json!([
        {"effect": "health check green", "probe": "lb/ready"},
        {"effect": "zero-downtime observed", "window_ms": 0}
    ])
}

const REQUIRED_FIELDS: [&str; 7] = [
    "work_request_id",
    "authority_decision_id",
    "execution_status",
    "observed_effects",
    "operational_settlement_status",
    "evidence_refs",
    "domain_model_ref",
];

// --- E6 positive path over the REAL chain --------------------------------------

#[test]
fn e6_full_governed_chain_settles_operationally_and_emits_the_frozen_envelope() {
    let h = harness("e6-positive");
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let (invocation_id, e5a_event) = admit(
        &mut ledger,
        &decision,
        "wr-t06-positive",
        "dddd1111-2222-4333-8444-555555555555",
    );

    let obs = observation(
        &invocation_id,
        &e5a_event,
        "wr-t06-positive",
        "completed",
        attesting_effects(),
        json!(null),
    );
    let settled = settle(&mut ledger, &obs);
    let e5b_event = obs["event_id"].as_str().unwrap().to_string();

    let returned = emit_operational_settlement(
        &settled,
        &declared_criteria(),
        &local_model_sha256(),
        &local_model_sha256(),
        &e5a_event,
        &e5b_event,
        SettlementOptionalFields {
            case_id: Some("case_t06"),
            run_id: Some("run_t06"),
            artifact_refs: Some(&json!(["sha256:abcdef"])),
            transcript_ref: Some("transcript:t06"),
            // A satisfied settlement carries no failure reason.
            failure_reason: None,
        },
    )
    .expect("attested criteria must settle operationally");

    // Evaluation composed from the LEDGER facts.
    assert_eq!(returned.evaluation.status, SettlementStatus::Accepted);
    assert_eq!(returned.work_request_id, "wr-t06-positive");
    assert_eq!(returned.authority_decision_id, decision.decision_id);
    assert_eq!(returned.invocation_id, invocation_id);

    let env = &returned.envelope;
    assert_eq!(env["schema_version"], "v1");
    assert_eq!(env["event_type"], "OperationalSettlement");
    assert_eq!(env["source_agent"], "sea-forge");
    assert_eq!(env["provenance"]["origin"], "sea-forge");

    // All seven frozen required fields present AND validated (self-check ran).
    for field in REQUIRED_FIELDS {
        assert!(
            env["payload"].get(field).is_some(),
            "frozen required field {field} missing"
        );
    }
    assert_eq!(env["payload"]["work_request_id"], "wr-t06-positive");
    assert_eq!(
        env["payload"]["authority_decision_id"],
        decision.decision_id
    );
    assert_eq!(env["payload"]["invocation_id"], invocation_id);
    assert_eq!(env["payload"]["execution_status"], "completed");
    assert_eq!(env["payload"]["operational_settlement_status"], "accepted");
    assert_eq!(env["payload"]["namespace"], "agentic_capability_loop");
    assert_eq!(env["payload"]["domain_model_hash"], local_model_sha256());
    assert_eq!(
        env["payload"]["domain_model_ref"]["model_hash"],
        local_model_sha256()
    );

    // Causality cites the ACTUAL chain: E5A invocation + E5B observation.
    let chain = env["provenance"]["chain"].as_array().unwrap();
    assert!(
        chain.contains(&json!(format!("caused_by:{e5a_event}"))),
        "must cite its AuthorizedInvocation parent: {chain:?}"
    );
    assert!(
        chain.contains(&json!(format!("caused_by:{e5b_event}"))),
        "must cite its ExecutionObservation parent: {chain:?}"
    );

    // ENV-I7: content-addressed evidence over exactly the settled effects.
    let refs = env["payload"]["evidence_refs"].as_array().unwrap();
    let expected_digest = independent_canonical_payload_json(&attesting_effects());
    assert_eq!(
        refs[0],
        json!(format!("sha256:{}", sha256_hex(expected_digest.as_bytes()))),
        "evidence must be content-addressed over the settled effects"
    );

    // Optional frozen fields passed through typed.
    assert_eq!(env["payload"]["case_id"], "case_t06");
    assert_eq!(env["payload"]["run_id"], "run_t06");
    assert_eq!(env["payload"]["artifact_refs"], json!(["sha256:abcdef"]));
    assert_eq!(env["payload"]["transcript_ref"], "transcript:t06");
}

// --- TOOTH 1: exit zero with violated criteria records failure ------------------

#[test]
fn t06_tooth1_exit_zero_with_violated_criteria_records_failure() {
    let h = harness("tooth1");
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let (invocation_id, e5a_event) = admit(
        &mut ledger,
        &decision,
        "wr-t06-tooth1",
        "eeee1111-2222-4333-8444-555555555555",
    );

    // The execution environment reports an explicit EXIT CODE 0 alongside
    // `completed` — and observed effects that attest NOTHING.
    let obs = observation(
        &invocation_id,
        &e5a_event,
        "wr-t06-tooth1",
        "completed",
        json!([]),
        json!({"exit_code": 0}),
    );
    let settled = settle(&mut ledger, &obs);
    let e5b_event = obs["event_id"].as_str().unwrap().to_string();

    let returned = emit_operational_settlement(
        &settled,
        &declared_criteria(),
        &local_model_sha256(),
        &local_model_sha256(),
        &e5a_event,
        &e5b_event,
        SettlementOptionalFields {
            failure_reason: Some(&json!({"reason": "declared criteria unmet"})),
            ..Default::default()
        },
    )
    .expect("a failed-criteria settlement still projects (as rejection)");

    assert_eq!(
        returned.evaluation.status,
        SettlementStatus::Rejected,
        "exit zero with violated criteria MUST record non-success"
    );
    assert_eq!(
        returned.envelope["payload"]["operational_settlement_status"],
        "rejected"
    );
    for criterion in declared_criteria() {
        assert!(returned
            .evaluation
            .basis
            .contains(&format!("criterion_unsatisfied:{criterion}")));
    }
    assert_eq!(
        returned.envelope["payload"]["failure_reason"],
        json!({"reason": "declared criteria unmet"})
    );
}

#[test]
fn t06_exit_zero_alone_cannot_produce_acceptance_even_with_status_completed() {
    let h = harness("exit-only");
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let (invocation_id, e5a_event) = admit(
        &mut ledger,
        &decision,
        "wr-t06-exit-only",
        "ffff1111-2222-4333-8444-555555555555",
    );

    let obs = observation(
        &invocation_id,
        &e5a_event,
        "wr-t06-exit-only",
        "completed",
        json!({"exit_code": 0, "stdout_tail": "all done"}),
        json!({"exit_code": 0}),
    );
    let settled = settle(&mut ledger, &obs);
    let e5b_event = obs["event_id"].as_str().unwrap().to_string();

    let returned = emit_operational_settlement(
        &settled,
        &declared_criteria(),
        &local_model_sha256(),
        &local_model_sha256(),
        &e5a_event,
        &e5b_event,
        SettlementOptionalFields::default(),
    )
    .unwrap();
    assert_eq!(returned.evaluation.status, SettlementStatus::Rejected);
    assert_eq!(
        returned.evaluation.unsatisfied_criteria,
        declared_criteria()
    );
}

#[test]
fn t06_partial_attestation_is_rejection_not_success() {
    let h = harness("partial");
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let (invocation_id, e5a_event) = admit(
        &mut ledger,
        &decision,
        "wr-t06-partial",
        "aaaa2222-2222-4333-8444-555555555555",
    );

    let obs = observation(
        &invocation_id,
        &e5a_event,
        "wr-t06-partial",
        "completed",
        json!([{"effect": "health check green"}]),
        json!(null),
    );
    let settled = settle(&mut ledger, &obs);
    let e5b_event = obs["event_id"].as_str().unwrap().to_string();

    let returned = emit_operational_settlement(
        &settled,
        &declared_criteria(),
        &local_model_sha256(),
        &local_model_sha256(),
        &e5a_event,
        &e5b_event,
        SettlementOptionalFields::default(),
    )
    .unwrap();
    assert_eq!(returned.evaluation.status, SettlementStatus::Rejected);
    assert_eq!(
        returned.evaluation.satisfied_criteria,
        vec!["health check green"]
    );
    assert_eq!(
        returned.evaluation.unsatisfied_criteria,
        vec!["zero-downtime observed"]
    );
}

#[test]
fn t06_non_completed_statuses_never_settle_accepted_regardless_of_effects() {
    for status in [
        "spawn_failed",
        "timed_out",
        "sandbox_violation",
        "suspected_sandbox_violation",
    ] {
        let h = harness(status);
        let decision = allow_decision(&h, 1);
        let mut ledger = InvocationLedger::default();
        let work_request_id = format!("wr-t06-{status}");
        let (invocation_id, e5a_event) = admit(
            &mut ledger,
            &decision,
            &work_request_id,
            "bbbb2222-2222-4333-8444-555555555555",
        );

        let obs = observation(
            &invocation_id,
            &e5a_event,
            &work_request_id,
            status,
            attesting_effects(),
            json!({"exit_code": 0}),
        );
        let settled = settle(&mut ledger, &obs);
        let e5b_event = obs["event_id"].as_str().unwrap().to_string();

        let returned = emit_operational_settlement(
            &settled,
            &declared_criteria(),
            &local_model_sha256(),
            &local_model_sha256(),
            &e5a_event,
            &e5b_event,
            SettlementOptionalFields::default(),
        )
        .unwrap();
        assert_eq!(
            returned.evaluation.status,
            SettlementStatus::Rejected,
            "{status}"
        );
        assert_eq!(
            returned.envelope["payload"]["operational_settlement_status"], "rejected",
            "{status}"
        );
    }

    // Basis vocabulary mirrors the kernel's own settle() sites.
    let evaluation = evaluate_operational_settlement(
        &sea_forge_server::governed_execution_boundary::ObservationOutcome::Settled {
            invocation_id: "inv_x".into(),
            work_request_id: "wr_x".into(),
            authority_decision_id: "dec_x".into(),
            execution_status: "sandbox_violation".into(),
            observed_effects: json!([]),
        },
        &declared_criteria(),
    )
    .unwrap();
    assert_eq!(evaluation.basis, vec!["jail_violation"]);
}

#[test]
fn t06_evaluation_requires_a_ledger_settled_observation() {
    let duplicate =
        sea_forge_server::governed_execution_boundary::ObservationOutcome::DuplicateDelivery;
    let err = evaluate_operational_settlement(&duplicate, &declared_criteria()).unwrap_err();
    assert_eq!(err, SettlementReturnError::NothingSettled);

    let err = emit_operational_settlement(
        &duplicate,
        &declared_criteria(),
        &local_model_sha256(),
        &local_model_sha256(),
        "aaaa3333-2222-4333-8444-555555555555",
        "bbbb3333-2222-4333-8444-555555555555",
        SettlementOptionalFields::default(),
    )
    .unwrap_err();
    assert_eq!(err, SettlementReturnError::NothingSettled);
}

#[test]
fn t06_empty_or_blank_declared_criteria_are_refused() {
    let settled = sea_forge_server::governed_execution_boundary::ObservationOutcome::Settled {
        invocation_id: "inv_x".into(),
        work_request_id: "wr_x".into(),
        authority_decision_id: "dec_x".into(),
        execution_status: "completed".into(),
        observed_effects: json!([{"effect": "health check green"}]),
    };
    for criteria in [Vec::new(), vec!["   ".to_string()]] {
        let err = evaluate_operational_settlement(&settled, &criteria).unwrap_err();
        assert_eq!(err, SettlementReturnError::OpaqueCriteria);
    }
}

// --- Criteria come from the DECLARED governed request chain ---------------------

#[test]
fn t06_criteria_come_from_the_declared_governed_request_chain() {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/t04_governed_work_request.json");
    if !fixture_path.exists() {
        eprintln!(
            "SKIP: T04 golden fixture absent at {}",
            fixture_path.display()
        );
        return;
    }
    let fixture: Value =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).unwrap()).unwrap();
    let model = fixture["domain_model_sha256"].as_str().unwrap().to_string();

    // The REAL T04 ingress parses the governed request chain and hands back
    // its DECLARED settlement criteria.
    let intent = sea_forge_server::governed_work_ingress::accept_governed_work_request(
        &fixture["request"],
        &fixture["context_packet"],
        &model,
    )
    .expect("T04 golden fixture must ingress");
    assert!(!intent.settlement_criteria.is_empty());

    let h = harness("declared-chain");
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let (invocation_id, e5a_event) = admit(
        &mut ledger,
        &decision,
        &intent.work_request_id,
        "cccc2222-2222-4333-8444-555555555555",
    );

    // Effects attesting exactly what the REQUEST declared.
    let effects = json!(intent
        .settlement_criteria
        .iter()
        .map(|c| json!({"effect": c}))
        .collect::<Vec<_>>());
    let obs = observation(
        &invocation_id,
        &e5a_event,
        &intent.work_request_id,
        "completed",
        effects,
        json!(null),
    );
    let settled = settle(&mut ledger, &obs);
    let e5b_event = obs["event_id"].as_str().unwrap().to_string();

    let returned = emit_operational_settlement(
        &settled,
        &intent.settlement_criteria,
        &model,
        &model,
        &e5a_event,
        &e5b_event,
        SettlementOptionalFields::default(),
    )
    .unwrap();
    assert_eq!(returned.evaluation.status, SettlementStatus::Accepted);

    // A criterion from some OTHER request is not satisfied by these effects:
    // declaring it too flips the settlement to rejection.
    let mut foreign = intent.settlement_criteria.clone();
    foreign.push("database migrated".into());
    let returned_foreign = emit_operational_settlement(
        &settled,
        &foreign,
        &model,
        &model,
        &e5a_event,
        &e5b_event,
        SettlementOptionalFields::default(),
    )
    .unwrap();
    assert_eq!(
        returned_foreign.evaluation.status,
        SettlementStatus::Rejected
    );
    assert_eq!(
        returned_foreign.evaluation.unsatisfied_criteria,
        vec!["database migrated"]
    );
}

// --- Canonical wire family -------------------------------------------------------

#[test]
fn t06_emitted_envelope_matches_the_canonical_wire_family() {
    let h = harness("wire-family");
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let (invocation_id, e5a_event) = admit(
        &mut ledger,
        &decision,
        "wr-t06-wire",
        "dddd2222-2222-4333-8444-555555555555",
    );

    let obs = observation(
        &invocation_id,
        &e5a_event,
        "wr-t06-wire",
        "completed",
        attesting_effects(),
        json!(null),
    );
    let settled = settle(&mut ledger, &obs);
    let e5b_event = obs["event_id"].as_str().unwrap().to_string();

    let first = emit_operational_settlement(
        &settled,
        &declared_criteria(),
        &local_model_sha256(),
        &local_model_sha256(),
        &e5a_event,
        &e5b_event,
        SettlementOptionalFields::default(),
    )
    .unwrap();
    let second = emit_operational_settlement(
        &settled,
        &declared_criteria(),
        &local_model_sha256(),
        &local_model_sha256(),
        &e5a_event,
        &e5b_event,
        SettlementOptionalFields::default(),
    )
    .unwrap();

    // Independent recomputation of the family's content-derived idempotency
    // key: sha256("|OperationalSettlement|<sorted-key payload json>").
    let payload = &first.envelope["payload"];
    let recomputed = sha256_hex(
        format!(
            "|OperationalSettlement|{}",
            independent_canonical_payload_json(payload)
        )
        .as_bytes(),
    );
    assert_eq!(first.envelope["idempotency_key"], json!(recomputed));

    // Stable identity across re-projection of the SAME settled facts
    // (ENV-I6): fresh event ids, identical content identity.
    assert_ne!(first.event_id, second.event_id);
    assert_eq!(
        first.envelope["idempotency_key"],
        second.envelope["idempotency_key"]
    );

    // Any mutation of the projected facts yields a DIFFERENT identity — the
    // receiving side will treat that as a conflicting re-settlement, never a
    // fresh fact.
    let settled_mutated = match &settled {
        sea_forge_server::governed_execution_boundary::ObservationOutcome::Settled {
            invocation_id,
            work_request_id,
            authority_decision_id,
            execution_status,
            ..
        } => sea_forge_server::governed_execution_boundary::ObservationOutcome::Settled {
            invocation_id: invocation_id.clone(),
            work_request_id: work_request_id.clone(),
            authority_decision_id: authority_decision_id.clone(),
            execution_status: execution_status.clone(),
            observed_effects: json!([{"effect": "health check green"}, {"effect": "zero-downtime observed"}, {"effect": "root installed"}]),
        },
        _ => unreachable!(),
    };
    let smuggled = emit_operational_settlement(
        &settled_mutated,
        &declared_criteria(),
        &local_model_sha256(),
        &local_model_sha256(),
        &e5a_event,
        &e5b_event,
        SettlementOptionalFields::default(),
    )
    .unwrap();
    assert_ne!(
        first.envelope["idempotency_key"], smuggled.envelope["idempotency_key"],
        "mutated observed_effects must change stable identity"
    );

    validate_operational_settlement_wire(&first.envelope, &local_model_sha256())
        .expect("emitted envelope survives the full wire battery");
}

// --- Identity gates --------------------------------------------------------------

#[test]
fn t06_emission_refuses_placeholder_and_drifted_model_identity() {
    let h = harness("identity");
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let (invocation_id, e5a_event) = admit(
        &mut ledger,
        &decision,
        "wr-t06-id",
        "eeee2222-2222-4333-8444-555555555555",
    );

    let obs = observation(
        &invocation_id,
        &e5a_event,
        "wr-t06-id",
        "completed",
        attesting_effects(),
        json!(null),
    );
    let settled = settle(&mut ledger, &obs);
    let e5b_event = obs["event_id"].as_str().unwrap().to_string();

    for bad in [
        fallback_pseudo_hash(),
        "0".repeat(64),
        "deadbeef".to_string(),
        local_model_sha256().to_uppercase(),
    ] {
        let err = emit_operational_settlement(
            &settled,
            &declared_criteria(),
            &bad,
            &local_model_sha256(),
            &e5a_event,
            &e5b_event,
            SettlementOptionalFields::default(),
        )
        .unwrap_err();
        assert!(
            matches!(err, SettlementReturnError::PlaceholderIdentity { .. }),
            "{bad} must be refused: {err}"
        );
    }

    // Drift: the declared model is not the locally resolved canonical model.
    let elsewhere = sha256_hex(b"a-different-canonical-model");
    let err = emit_operational_settlement(
        &settled,
        &declared_criteria(),
        &elsewhere,
        &local_model_sha256(),
        &e5a_event,
        &e5b_event,
        SettlementOptionalFields::default(),
    )
    .unwrap_err();
    assert!(matches!(err, SettlementReturnError::DomainDrift { .. }));
}

#[test]
fn t06_emission_requires_the_actual_invocation_chain_as_causality() {
    let h = harness("causality");
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let (invocation_id, e5a_event) = admit(
        &mut ledger,
        &decision,
        "wr-t06-causality",
        "affe1111-2222-4333-8444-555555555555",
    );

    let obs = observation(
        &invocation_id,
        &e5a_event,
        "wr-t06-causality",
        "completed",
        attesting_effects(),
        json!(null),
    );
    let settled = settle(&mut ledger, &obs);
    let e5b_event = obs["event_id"].as_str().unwrap().to_string();

    for (e5a, e5b) in [
        ("", &*e5b_event),
        ("  ", &*e5b_event),
        (&*e5a_event, ""),
        (&*e5a_event, "placeholder"),
        (&*e5a_event, "<missing>"),
    ] {
        let err = emit_operational_settlement(
            &settled,
            &declared_criteria(),
            &local_model_sha256(),
            &local_model_sha256(),
            e5a,
            e5b,
            SettlementOptionalFields::default(),
        )
        .unwrap_err();
        assert!(
            matches!(err, SettlementReturnError::PlaceholderField { .. }),
            "chain ids ({e5a:?}, {e5b:?}) must be mandatory: {err}"
        );
    }
}

// --- Wire battery: every projected field validated --------------------------------

fn valid_returned_envelope(harness_tag: &str, seq: usize) -> (Harness, Value) {
    let h = harness(harness_tag);
    let decision = allow_decision(&h, seq);
    let mut ledger = InvocationLedger::default();
    let (invocation_id, e5a_event) = admit(
        &mut ledger,
        &decision,
        "wr-t06-battery",
        "affe2222-2222-4333-8444-555555555555",
    );
    let obs = observation(
        &invocation_id,
        &e5a_event,
        "wr-t06-battery",
        "completed",
        attesting_effects(),
        json!(null),
    );
    let settled = settle(&mut ledger, &obs);
    let e5b_event = obs["event_id"].as_str().unwrap().to_string();
    let returned = emit_operational_settlement(
        &settled,
        &declared_criteria(),
        &local_model_sha256(),
        &local_model_sha256(),
        &e5a_event,
        &e5b_event,
        SettlementOptionalFields::default(),
    )
    .unwrap();
    (h, returned.envelope)
}

#[test]
fn t06_every_projected_field_is_validated_on_the_wire() {
    let (_h, envelope) = valid_returned_envelope("battery-missing", 1);

    // Removing ANY frozen required field is refused, naming the field.
    for field in REQUIRED_FIELDS {
        let mut tampered = envelope.clone();
        tampered["payload"]
            .as_object_mut()
            .unwrap()
            .remove(field)
            .unwrap_or_else(|| panic!("{field} existed"));
        let err = validate_operational_settlement_wire(&tampered, &local_model_sha256())
            .expect_err("missing required field must be refused");
        assert!(
            matches!(err, SettlementReturnError::OpaquePayload { .. }),
            "{field}: {err}"
        );
    }

    // Vocabulary refusals — including a proof-shaped verdict smuggled into
    // the settlement-status field (I6 direction).
    for (field, value, _label) in [
        (
            "execution_status",
            json!("exit_zero"),
            "InvalidExecutionStatus",
        ),
        (
            "operational_settlement_status",
            json!("proof_passed"),
            "InvalidSettlementStatus",
        ),
        (
            "operational_settlement_status",
            json!("success"),
            "InvalidSettlementStatus",
        ),
    ] {
        let mut tampered = envelope.clone();
        tampered["payload"][field] = value;
        let err = validate_operational_settlement_wire(&tampered, &local_model_sha256())
            .expect_err("vocabulary violation must be refused");
        let rejected = matches!(
            err,
            SettlementReturnError::InvalidExecutionStatus { .. }
                | SettlementReturnError::InvalidSettlementStatus { .. }
        );
        assert!(rejected, "{field}: {err}");
    }

    // Shape refusals.
    for (field, value) in [
        ("observed_effects", json!("attested")),
        ("evidence_refs", json!([])),
        ("evidence_refs", json!(["not-a-ref"])),
        ("evidence_refs", json!(["sha256:zz"])),
        ("case_id", json!(42)),
        ("run_id", json!(null)),
        ("transcript_ref", json!([])),
        ("artifact_refs", json!({"a": 1})),
        ("failure_reason", json!(42)),
    ] {
        let mut tampered = envelope.clone();
        tampered["payload"][field] = value;
        assert!(
            validate_operational_settlement_wire(&tampered, &local_model_sha256()).is_err(),
            "{field} shape violation must be refused"
        );
    }

    // Namespace gate (T04-D3 closed): swapping the loop namespace fails.
    let mut tampered = envelope.clone();
    tampered["payload"]["namespace"] = json!("some_other_namespace");
    let err = validate_operational_settlement_wire(&tampered, &local_model_sha256()).unwrap_err();
    assert!(matches!(
        err,
        SettlementReturnError::NamespaceMismatch { .. }
    ));

    // domain_model_ref must agree with the declared identity.
    let mut tampered = envelope.clone();
    tampered["payload"]["domain_model_ref"]["model_hash"] = json!(sha256_hex(b"another-model"));
    let err = validate_operational_settlement_wire(&tampered, &local_model_sha256()).unwrap_err();
    assert!(matches!(
        err,
        SettlementReturnError::RefIdentityMismatch { .. }
    ));

    // Causality gap: stripping the chain fails closed.
    let mut tampered = envelope.clone();
    tampered["provenance"]["chain"] = json!([]);
    let err = validate_operational_settlement_wire(&tampered, &local_model_sha256()).unwrap_err();
    assert!(matches!(
        err,
        SettlementReturnError::CausalityMissing { .. }
    ));

    // Drift against the local resolution fails.
    let err =
        validate_operational_settlement_wire(&envelope, &sha256_hex(b"locally-resolved-elsewhere"))
            .unwrap_err();
    assert!(matches!(err, SettlementReturnError::DomainDrift { .. }));
}

#[test]
fn t06_wrong_producer_stamps_are_refused_at_wire_validation() {
    let (_h, envelope) = valid_returned_envelope("battery-producer", 1);

    for forger in [
        "swe-seed",
        "godspeed-agent",
        "execution-environment",
        "context-kernel",
    ] {
        let mut forged = envelope.clone();
        forged["source_agent"] = json!(forger);
        let err = validate_operational_settlement_wire(&forged, &local_model_sha256()).unwrap_err();
        assert!(
            matches!(err, SettlementReturnError::NotAuthoritative { .. }),
            "{forger} forging OperationalSettlement must be refused: {err}"
        );
    }

    let mut mystery = envelope.clone();
    mystery["source_agent"] = json!("mystery-component");
    let err = validate_operational_settlement_wire(&mystery, &local_model_sha256()).unwrap_err();
    assert!(matches!(err, SettlementReturnError::UnknownAgent { .. }));

    let mut crossed = envelope.clone();
    crossed["event_type"] = json!("ExecutionObservation");
    let err = validate_operational_settlement_wire(&crossed, &local_model_sha256()).unwrap_err();
    assert!(matches!(err, SettlementReturnError::WrongEventType { .. }));

    let mut schema = envelope.clone();
    schema["schema_version"] = json!("v2");
    assert!(validate_operational_settlement_wire(&schema, &local_model_sha256()).is_err());
}

// --- Cross-repo golden fixture -----------------------------------------------------

#[test]
fn t06_writes_golden_fixture_for_swe_seed_adjudication() {
    let seed_root = std::env::var("SWE_SEED_ROOT").unwrap_or_else(|_| {
        format!(
            "{}/projects/SWE_SEED",
            std::env::var("HOME").unwrap_or_default()
        )
    });
    let fixture = std::path::Path::new(&seed_root)
        .join("crates/swe-seed-core/tests/fixtures/t06_operational_settlement.json");
    if !std::path::Path::new(&seed_root)
        .join("crates/swe-seed-core")
        .exists()
    {
        eprintln!(
            "SKIP: SWE_SEED checkout not found at {} (fixture not regenerated)",
            fixture.display()
        );
        return;
    }

    let h = harness("golden-fixture");
    let decision = allow_decision(&h, 1);
    let mut ledger = InvocationLedger::default();
    let work_request_id = "wr-gsf-001";
    let (invocation_id, e5a_event) = admit(
        &mut ledger,
        &decision,
        work_request_id,
        "2c17da10-04d7-434d-8a79-4b31a7036dc1",
    );

    let obs = observation(
        &invocation_id,
        &e5a_event,
        work_request_id,
        "completed",
        attesting_effects(),
        json!(null),
    );
    let settled = settle(&mut ledger, &obs);
    let e5b_event = obs["event_id"].as_str().unwrap().to_string();

    let returned = emit_operational_settlement(
        &settled,
        &declared_criteria(),
        &local_model_sha256(),
        &local_model_sha256(),
        &e5a_event,
        &e5b_event,
        SettlementOptionalFields::default(),
    )
    .expect("golden settlement must project");
    assert_eq!(returned.evaluation.status, SettlementStatus::Accepted);

    // The fixture carries the producing model digest out-of-band (the T04
    // convention) plus the exact causal chain ids, so the consumer binds to
    // independently declared identities rather than trusting the envelope.
    let body = json!({
        "domain_model_sha256": local_model_sha256(),
        "originating_work_request_id": work_request_id,
        "expected_chain": [e5a_event, e5b_event],
        "settlement": returned.envelope,
    });
    if let Some(parent) = fixture.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(
        &fixture,
        format!("{}\n", serde_json::to_string_pretty(&body).unwrap()),
    )
    .expect("write golden fixture");
    println!("regenerated {}", fixture.display());

    // Round-trip: what SWE_SEED receives decodes as JSON.
    let decoded: Value = serde_json::from_str(&std::fs::read_to_string(&fixture).unwrap()).unwrap();
    assert_eq!(decoded["settlement"]["event_type"], "OperationalSettlement");
}
