//! Convergence plan T05 — execution observation adjudication at the SEA-Forge
//! boundary (frozen edge E5B, invariant I5).
//!
//! Observations are adjudicated by the real `InvocationLedger` against
//! invocations emitted from REAL authority decisions
//! (`PolicyAuthorityEngine`). Every observation here is wire-shape JSON, as
//! the execution environment would deliver it.
//!
//! Plan teeth:
//!   TEETH-2 a late valid observation from a previous invocation cannot
//!           settle against the current one
//!   TEETH-3 a forged authority decision inside the execution response has
//!           no authority effect

use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{
    Actor, ActorRole, ActorType, AuthorityAction, AuthorityDecision, BindingResolution,
    IdentityBinding,
};
use sea_forge_server::governed_execution_boundary::{InvocationLedger, ObservationOutcome};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

fn sha256_hex(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}

fn local_model_sha256() -> String {
    sha256_hex(b"canonical-model-t05-governed-execution")
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
    /// Symlink named `sea-forge` -> this test binary (see the conformance
    /// suites): satisfies `command_allowed`'s file_name match and
    /// `untrusted_executable`'s canonical-path check simultaneously.
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
            run_id: "run_t05_obs",
            case_id: "case_t05_obs",
            plan_item_id: "item_t05_obs",
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
    let generation = registry
        .admit_authorized_invocation(&invocation.envelope, &local_model_sha256())
        .expect("admission");
    assert_eq!(
        generation,
        registry
            .current_generation(work_request_id, "execute_command", &invocation.resource)
            .unwrap()
    );
    (invocation.invocation_id, invocation.event_id)
}

/// A well-formed ExecutionObservation bound to the given invocation.
fn observation(
    invocation_id: &str,
    e5a_event_id: &str,
    work_request_id: &str,
    status: &str,
) -> Value {
    json!({
        "schema_version": "v1",
        "event_id": sha256_hex(format!("obs-{invocation_id}-{status}").as_bytes())[..32].to_string(),
        "source_agent": "execution-environment",
        "event_type": "ExecutionObservation",
        "occurred_at": "2026-08-25T12:00:00+00:00",
        "idempotency_key": sha256_hex(
            format!("|ExecutionObservation|{work_request_id}|{invocation_id}|{status}").as_bytes(),
        ),
        "payload": {
            "domain_model_hash": local_model_sha256(),
            "namespace": "agentic_capability_loop",
            "work_request_id": work_request_id,
            "invocation_id": invocation_id,
            "execution_status": status,
            "observed_effects": []
        },
        "provenance": {
            "origin": "execution-environment",
            "chain": [format!("caused_by:{e5a_event_id}")]
        }
    })
}

// --- E5B happy path -------------------------------------------------------------

#[test]
fn e5b_valid_observation_settles_against_the_exact_authorized_invocation() {
    let h = harness("settle");
    let decision = allow_decision(&h, 1);
    let mut registry = InvocationLedger::default();
    let (invocation_id, e5a_event) = admit(
        &mut registry,
        &decision,
        "wr-t05-settle",
        "aaaa1111-2222-4333-8444-555555555555",
    );

    let outcome = registry
        .settle_execution_observation(
            &observation(&invocation_id, &e5a_event, "wr-t05-settle", "completed"),
            &local_model_sha256(),
        )
        .expect("the exact authorized invocation must settle");
    assert_eq!(
        outcome,
        ObservationOutcome::Settled {
            invocation_id,
            work_request_id: "wr-t05-settle".into(),
            // Reported from the LEDGER record — the authorization fact.
            authority_decision_id: decision.decision_id,
            execution_status: "completed".into(),
            observed_effects: json!([]),
        }
    );
    assert!(outcome.settled());
}

// --- TEETH 2: late observations from previous generations ------------------------

#[test]
fn t05_late_observation_from_previous_generation_cannot_settle_current_slot() {
    let h = harness("late");
    let mut registry = InvocationLedger::default();
    // Generation 1 of the invocation slot...
    let d1 = allow_decision(&h, 1);
    let (gen1_invocation, gen1_event) = admit(
        &mut registry,
        &d1,
        "wr-t05-late",
        "bbbb1111-2222-4333-8444-555555555555",
    );
    // ...re-authorized as generation 2 of the SAME slot.
    let d2 = allow_decision(&h, 2);
    assert_ne!(d1.decision_id, d2.decision_id);
    let (gen2_invocation, gen2_event) = admit(
        &mut registry,
        &d2,
        "wr-t05-late",
        "cccc1111-2222-4333-8444-555555555555",
    );
    assert_ne!(gen1_invocation, gen2_invocation);
    assert_eq!(
        registry.current_generation(
            "wr-t05-late",
            "execute_command",
            &h.sea_forge_argv0.to_string_lossy()
        ),
        Some(2)
    );

    // A perfectly valid observation from the PREVIOUS generation cannot
    // settle the current slot — even though it cites its own (real,
    // once-authorized) invocation correctly.
    let late = observation(&gen1_invocation, &gen1_event, "wr-t05-late", "completed");
    let err = registry
        .settle_execution_observation(&late, &local_model_sha256())
        .unwrap_err();
    assert!(
        matches!(err,
            sea_forge_server::governed_execution_boundary::BoundaryError::LateObservation {
                ref invocation_id,
                settled_generation: 1,
                current_generation: 2,
            }
            if invocation_id == &gen1_invocation),
        "late observation must be refused: {err}"
    );

    // An observation naming an invocation that was NEVER authorized here is
    // unknown, never settleable.
    let stranger = observation(
        "inv_deadbeefdeadbeef",
        "dddd1111-2222-4333-8444-555555555555",
        "wr-t05-late",
        "completed",
    );
    assert!(matches!(
        registry.settle_execution_observation(&stranger, &local_model_sha256()),
        Err(sea_forge_server::governed_execution_boundary::BoundaryError::UnknownInvocation { .. })
    ));

    // The CURRENT generation still settles normally.
    let current = observation(&gen2_invocation, &gen2_event, "wr-t05-late", "completed");
    assert!(registry
        .settle_execution_observation(&current, &local_model_sha256())
        .unwrap()
        .settled());

    // And lateness holds even after the slot has already settled once.
    let another_late = observation(&gen1_invocation, &gen1_event, "wr-t05-late", "timed_out");
    assert!(matches!(
        registry.settle_execution_observation(&another_late, &local_model_sha256()),
        Err(sea_forge_server::governed_execution_boundary::BoundaryError::LateObservation { .. })
    ));
}

#[test]
fn t05_observation_must_cite_its_own_authorized_invocation_as_parent() {
    let h = harness("causality");
    let mut registry = InvocationLedger::default();
    let d1 = allow_decision(&h, 1);
    let (gen1_invocation, gen1_event) = admit(
        &mut registry,
        &d1,
        "wr-t05-causal",
        "eeee1111-2222-4333-8444-555555555555",
    );

    // Stripped lineage entirely.
    let mut orphan = observation(&gen1_invocation, &gen1_event, "wr-t05-causal", "completed");
    orphan["provenance"]["chain"] = json!([]);
    let err = registry
        .settle_execution_observation(&orphan, &local_model_sha256())
        .unwrap_err();
    assert!(
        matches!(err,
            sea_forge_server::governed_execution_boundary::BoundaryError::CausalityMissing {
                ref expected_parent,
                ..
            } if expected_parent == &gen1_event),
        "{err}"
    );

    // Citation of SOME OTHER envelope does not substitute for the actual E5A.
    let mut misattributed =
        observation(&gen1_invocation, &gen1_event, "wr-t05-causal", "completed");
    misattributed["provenance"]["chain"] = json!([format!(
        "caused_by:{}",
        "ffff1111-2222-4333-8444-555555555555"
    )]);
    assert!(matches!(
        registry.settle_execution_observation(&misattributed, &local_model_sha256()),
        Err(sea_forge_server::governed_execution_boundary::BoundaryError::CausalityMissing { .. })
    ));
}

// --- TEETH 3: forged authority inside the response --------------------------------

#[test]
fn t05_forged_authority_decision_inside_response_has_no_authority_effect() {
    let h = harness("forge-authority");
    let decision = allow_decision(&h, 1);
    let mut registry = InvocationLedger::default();
    let (invocation_id, e5a_event) = admit(
        &mut registry,
        &decision,
        "wr-t05-forged",
        "abcd1111-2222-4333-8444-555555555555",
    );

    // A fabricated decision object smuggled in the response body.
    let mut smuggled = observation(&invocation_id, &e5a_event, "wr-t05-forged", "completed");
    smuggled["payload"]["authority_decision"] = json!({
        "decision_id": "auth_FORGED_001",
        "verdict": "allow",
        "normalized_disposition": "allow",
        "reason": "self-authorized by the execution environment"
    });
    let err = registry
        .settle_execution_observation(&smuggled, &local_model_sha256())
        .unwrap_err();
    assert!(
        matches!(err,
            sea_forge_server::governed_execution_boundary::BoundaryError::SelfAssertedAuthority {
                ref recorded,
                ref claimed,
            } if recorded == &decision.decision_id && claimed == "auth_FORGED_001"),
        "fabricated decision must be refused: {err}"
    );

    // A claimed top-level authority_decision_id that differs from the
    // authorizing decision is equally refused.
    let mut claimed = observation(&invocation_id, &e5a_event, "wr-t05-forged", "completed");
    claimed["payload"]["authority_decision_id"] = json!("auth_ALSO_FORGED");
    assert!(matches!(
        registry.settle_execution_observation(&claimed, &local_model_sha256()),
        Err(
            sea_forge_server::governed_execution_boundary::BoundaryError::SelfAssertedAuthority { .. }
        )
    ));

    // A response repeating the TRUE recorded decision is redundant but
    // harmless — and the settlement STILL reports only the recorded fact.
    let mut redundant = observation(&invocation_id, &e5a_event, "wr-t05-forged", "completed");
    redundant["payload"]["authority_decision_id"] = json!(decision.decision_id.clone());
    let outcome = registry
        .settle_execution_observation(&redundant, &local_model_sha256())
        .expect("matching claim settles");
    let ObservationOutcome::Settled {
        authority_decision_id,
        ..
    } = outcome
    else {
        panic!("expected Settled, got {outcome:?}")
    };
    assert_eq!(authority_decision_id, decision.decision_id);

    // The outcome type carries no authority capability at all: nothing a
    // forged response could populate exists on it.
    let probe = ObservationOutcome::DuplicateDelivery;
    assert!(!probe.settled());
}

// --- Cross-wired correlation -------------------------------------------------------

#[test]
fn t05_cross_wired_work_request_is_rejected() {
    let h = harness("cross-wire");
    let decision = allow_decision(&h, 1);
    let mut registry = InvocationLedger::default();
    let (invocation_id, e5a_event) = admit(
        &mut registry,
        &decision,
        "wr-t05-mine",
        "bcda1111-2222-4333-8444-555555555555",
    );

    // Correct invocation + correct causal citation, but someone else's work
    // request in the correlation field.
    let wired = observation(&invocation_id, &e5a_event, "wr-t05-THEIRS", "completed");
    let err = registry
        .settle_execution_observation(&wired, &local_model_sha256())
        .unwrap_err();
    assert!(
        matches!(err,
            sea_forge_server::governed_execution_boundary::BoundaryError::CrossWiredWorkRequest {
                ref expected,
                ref got,
            } if expected == "wr-t05-mine" && got == "wr-t05-THEIRS"),
        "{err}"
    );
}

// --- Duplicate delivery -------------------------------------------------------------

#[test]
fn t05_duplicate_delivery_is_consequence_free() {
    let h = harness("dup");
    let decision = allow_decision(&h, 1);
    let mut registry = InvocationLedger::default();
    let (invocation_id, e5a_event) = admit(
        &mut registry,
        &decision,
        "wr-t05-dup",
        "dcba1111-2222-4333-8444-555555555555",
    );
    let first = observation(&invocation_id, &e5a_event, "wr-t05-dup", "completed");
    assert!(registry
        .settle_execution_observation(&first, &local_model_sha256())
        .unwrap()
        .settled());

    // Exact redelivery: recognized as duplicate, no second settlement.
    let replay = first.clone();
    assert_eq!(
        registry
            .settle_execution_observation(&replay, &local_model_sha256())
            .unwrap(),
        ObservationOutcome::DuplicateDelivery
    );

    // A MUTATED redelivery (same event identity, different claimed status)
    // cannot rewrite the settled fact either.
    let mut mutated = first.clone();
    mutated["payload"]["execution_status"] = json!("timed_out");
    assert_eq!(
        registry
            .settle_execution_observation(&mutated, &local_model_sha256())
            .unwrap(),
        ObservationOutcome::DuplicateDelivery
    );

    // Same content under a fresh event id is still the same observation by
    // idempotency key — no consequence.
    let mut rewrapped = first;
    rewrapped["event_id"] = json!("11112222-3333-4777-8888-999900001111");
    assert_eq!(
        registry
            .settle_execution_observation(&rewrapped, &local_model_sha256())
            .unwrap(),
        ObservationOutcome::DuplicateDelivery
    );
}

// --- Producer authority both directions ----------------------------------------------

#[test]
fn t05_wrong_producer_on_observations_is_rejected_both_directions() {
    let h = harness("producers");
    let decision = allow_decision(&h, 1);
    let mut registry = InvocationLedger::default();
    let (invocation_id, e5a_event) = admit(
        &mut registry,
        &decision,
        "wr-t05-prod",
        "abcd4321-2222-4333-8444-555555555555",
    );

    // Only execution_environment may produce ExecutionObservation (I3).
    for forger in ["sea-forge", "swe-seed", "godspeed-agent", "context-kernel"] {
        let mut forged = observation(&invocation_id, &e5a_event, "wr-t05-prod", "completed");
        forged["source_agent"] = json!(forger);
        let err = registry
            .settle_execution_observation(&forged, &local_model_sha256())
            .unwrap_err();
        assert!(
            matches!(
                err,
                sea_forge_server::governed_execution_boundary::BoundaryError::NotAuthoritative {
                    authoritative: "execution_environment",
                    ..
                }
            ),
            "forger {forger} must be refused: {err}"
        );
    }

    // Unknown vocabulary fails closed.
    let mut mystery = observation(&invocation_id, &e5a_event, "wr-t05-prod", "completed");
    mystery["source_agent"] = json!("mystery-agent");
    assert!(matches!(
        registry.settle_execution_observation(&mystery, &local_model_sha256()),
        Err(sea_forge_server::governed_execution_boundary::BoundaryError::UnknownAgent { .. })
    ));

    // An AuthorizedInvocation delivered into the observation edge is simply
    // the wrong event type.
    let d2 = allow_decision(&h, 2);
    let invocation = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(
        &d2,
        "wr-t05-prod",
        &local_model_sha256(),
        &local_model_sha256(),
        "abcd4321-2222-4333-8444-555555555555",
        None,
    )
    .unwrap();
    assert!(matches!(
        registry.settle_execution_observation(&invocation.envelope, &local_model_sha256()),
        Err(
            sea_forge_server::governed_execution_boundary::BoundaryError::WrongEventType {
                expected: "ExecutionObservation",
                ..
            }
        )
    ));
}

// --- Missing / placeholder identities and malformed payloads --------------------------

#[test]
fn t05_missing_or_placeholder_identity_in_observation_is_rejected() {
    let h = harness("identity");
    let decision = allow_decision(&h, 1);
    let mut registry = InvocationLedger::default();
    let (invocation_id, e5a_event) = admit(
        &mut registry,
        &decision,
        "wr-t05-id",
        "abcd5678-2222-4333-8444-555555555555",
    );
    let base = || observation(&invocation_id, &e5a_event, "wr-t05-id", "completed");

    // Missing model hash.
    let mut missing_hash = base();
    missing_hash["payload"]
        .as_object_mut()
        .unwrap()
        .remove("domain_model_hash");
    assert!(matches!(
        registry.settle_execution_observation(&missing_hash, &local_model_sha256()),
        Err(
            sea_forge_server::governed_execution_boundary::BoundaryError::PlaceholderIdentity { .. }
        )
    ));

    // Standalone fallback pseudo-hash — even claiming it as the local truth.
    let fallback = {
        let mut hasher = Sha256::new();
        hasher.update(b"agentic_capability_loop");
        format!("{:x}", hasher.finalize())
    };
    for pseudo in [fallback.clone(), "0".repeat(64), "deadbeef".into()] {
        let mut bad = base();
        bad["payload"]["domain_model_hash"] = json!(pseudo);
        assert!(
            matches!(
                registry.settle_execution_observation(&bad, &local_model_sha256()),
                Err(sea_forge_server::governed_execution_boundary::BoundaryError::PlaceholderIdentity { .. })
            ),
            "pseudo-hash {pseudo} must never settle"
        );
    }

    // Drift: valid hash, different model than locally resolved.
    let drifted = base();
    let other = sha256_hex(b"a-different-canonical-model");
    let mut drifted = drifted;
    drifted["payload"]["domain_model_hash"] = json!(other);
    assert!(matches!(
        registry.settle_execution_observation(&drifted, &local_model_sha256()),
        Err(sea_forge_server::governed_execution_boundary::BoundaryError::DomainDrift { .. })
    ));

    // Placeholder invocation identity.
    let mut placeholder = base();
    placeholder["payload"]["invocation_id"] = json!("unknown");
    let err = registry
        .settle_execution_observation(&placeholder, &local_model_sha256())
        .unwrap_err();
    assert!(
        matches!(
            err,
            sea_forge_server::governed_execution_boundary::BoundaryError::PlaceholderField {
                ref field
            }
            if field == "invocation_id"
        ),
        "{err}"
    );

    // Required fields absent.
    for (field, value) in [
        ("observed_effects", json!(null)),
        ("execution_status", json!(null)),
        ("work_request_id", json!(null)),
        ("invocation_id", json!(null)),
    ] {
        let mut hole = base();
        hole["payload"][field] = value;
        assert!(
            matches!(
                registry.settle_execution_observation(&hole, &local_model_sha256()),
                Err(
                    sea_forge_server::governed_execution_boundary::BoundaryError::OpaquePayload { .. }
                )
            ),
            "missing {field} must be an opaque payload"
        );
    }

    // Status outside the governed vocabulary.
    let mut status = base();
    status["payload"]["execution_status"] = json!("exit_zero_success");
    assert!(matches!(
        registry.settle_execution_observation(&status, &local_model_sha256()),
        Err(
            sea_forge_server::governed_execution_boundary::BoundaryError::InvalidExecutionStatus { .. }
        )
    ));

    // Typed optional fields with wrong shapes are refused, not coerced.
    let mut refs = base();
    refs["payload"]["artifact_refs"] = json!("artifact.txt");
    assert!(matches!(
        registry.settle_execution_observation(&refs, &local_model_sha256()),
        Err(
            sea_forge_server::governed_execution_boundary::BoundaryError::InvalidOptionalField { .. }
        )
    ));
    let mut stdout = base();
    stdout["payload"]["stdout_ref"] = json!(42);
    assert!(matches!(
        registry.settle_execution_observation(&stdout, &local_model_sha256()),
        Err(
            sea_forge_server::governed_execution_boundary::BoundaryError::InvalidOptionalField { .. }
        )
    ));
}

// --- Admission-side negatives + ENV-I6 idempotency --------------------------------------

#[test]
fn t05_admission_requires_causality_full_payload_and_is_idempotent() {
    let h = harness("admission");
    let decision = allow_decision(&h, 1);
    let invocation = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(
        &decision,
        "wr-t05-adm",
        &local_model_sha256(),
        &local_model_sha256(),
        "abcd8765-2222-4333-8444-555555555555",
        None,
    )
    .unwrap();

    // Orphan invocation (no causal parents) cannot enter the ledger.
    let mut orphan = invocation.envelope.clone();
    orphan["provenance"]["chain"] =
        json!(["domain_model_hash:".to_string() + &local_model_sha256()]);
    let mut registry = InvocationLedger::default();
    assert!(matches!(
        registry.admit_authorized_invocation(&orphan, &local_model_sha256()),
        Err(sea_forge_server::governed_execution_boundary::BoundaryError::CausalityMissing { .. })
    ));

    // Missing frozen required fields.
    for field in [
        "work_request_id",
        "invocation_id",
        "authority_decision_id",
        "operation",
        "resource",
        "execution_constraints",
    ] {
        let mut hole = invocation.envelope.clone();
        hole["payload"].as_object_mut().unwrap().remove(field);
        let mut r = InvocationLedger::default();
        assert!(
            matches!(
                r.admit_authorized_invocation(&hole, &local_model_sha256()),
                Err(
                    sea_forge_server::governed_execution_boundary::BoundaryError::OpaquePayload { .. }
                )
            ),
            "missing {field} must refuse admission"
        );
    }

    // Idempotent re-admission of the SAME envelope (ENV-I6): same generation,
    // no supersession, slot unchanged.
    let mut registry = InvocationLedger::default();
    let gen_once = registry
        .admit_authorized_invocation(&invocation.envelope, &local_model_sha256())
        .unwrap();
    let gen_twice = registry
        .admit_authorized_invocation(&invocation.envelope, &local_model_sha256())
        .unwrap();
    assert_eq!(gen_once, gen_twice);
    assert_eq!(
        registry.current_generation("wr-t05-adm", "execute_command", &invocation.resource),
        Some(1)
    );

    // Re-authorization opens generation 2 and supersedes generation 1.
    let d2 = allow_decision(&h, 2);
    let (gen2_id, _) = admit(
        &mut registry,
        &d2,
        "wr-t05-adm",
        "abcd8765-2222-4333-8444-555555555555",
    );
    assert_ne!(gen2_id, invocation.invocation_id);
    assert_eq!(
        registry.current_generation("wr-t05-adm", "execute_command", &invocation.resource),
        Some(2)
    );
}

// --- Real T04 surface feeds the causality chain -------------------------------------------

#[test]
fn t05_real_t04_governed_work_request_feeds_invocation_causality() {
    // Load the cross-repo golden fixture: REAL SWE_SEED GovernedWorkRequest
    // output (regenerated by its convergence_t04 suite). Honest skip when
    // absent, matching the accepted T04 pattern.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/t04_governed_work_request.json");
    let Ok(body) = std::fs::read_to_string(&path) else {
        eprintln!(
            "SKIP: fixture not found at {} (run swe-seed-core convergence_t04_governed_submission to generate)",
            path.display()
        );
        return;
    };
    let body: Value = serde_json::from_str(&body).expect("fixture must be valid JSON");
    let local = body["domain_model_sha256"].as_str().unwrap();
    let governed_request_event_id = body["request"]["event_id"].as_str().unwrap();

    let h = harness("t04-compose");
    // The declared identity SEA-Forge resolved locally IS the fixture's model
    // — the emitter drift gate would refuse anything else.
    let decision = allow_decision(&h, 1);
    let invocation = sea_forge_server::governed_execution_boundary::emit_authorized_invocation(
        &decision,
        body["request"]["payload"]["work_request_id"]
            .as_str()
            .unwrap(),
        local,
        local,
        governed_request_event_id,
        None,
    )
    .expect("real T04 cycle feeds a canonical invocation");
    assert!(invocation.envelope["provenance"]["chain"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c.as_str() == Some(governed_request_event_id)
            || c.as_str() == Some(&format!("caused_by:{governed_request_event_id}"))));

    let mut registry = InvocationLedger::default();
    registry
        .admit_authorized_invocation(&invocation.envelope, local)
        .expect("admission over the real T04 identity");
    let mut obs = observation(
        &invocation.invocation_id,
        &invocation.event_id,
        body["request"]["payload"]["work_request_id"]
            .as_str()
            .unwrap(),
        "completed",
    );
    // The execution environment reports under the SAME canonical model the
    // whole cycle is bound to.
    obs["payload"]["domain_model_hash"] = json!(local);
    let outcome = registry
        .settle_execution_observation(&obs, local)
        .expect("settlement bound to the real T04 cycle");
    let ObservationOutcome::Settled {
        work_request_id,
        authority_decision_id,
        ..
    } = outcome
    else {
        panic!("expected Settled")
    };
    assert_eq!(work_request_id, "wr-gsf-001");
    assert_eq!(authority_decision_id, decision.decision_id);
}
