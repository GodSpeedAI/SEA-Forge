//! Convergence plan T05 — authorized execution at the SEA-Forge boundary
//! (frozen edge E5A, invariant I5).
//!
//! Every authority fact here comes from the REAL policy engine
//! (`PolicyAuthorityEngine`) and every governed side effect from the REAL
//! runtime (`sea_forge_runtime::execute`) — the same production seam
//! `case_dispatch` uses. The canonical AuthorizedInvocation envelope is only
//! obtainable through `emit_authorized_invocation`, which refuses any
//! non-Allow decision.
//!
//! Plan teeth:
//!   TEETH-1 deny/escalated authority => no governed side effect occurs
//!           through the governed path (this file)
//!   TEETH-2 late observation from a previous invocation cannot settle
//!           (convergence_t05_execution_observation.rs)
//!   TEETH-3 forged authority decision inside the execution response has no
//!           authority effect (convergence_t05_execution_observation.rs)

use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{
    Actor, ActorRole, ActorType, AuthorityAction, BindingResolution, ExecutionRequest,
    ExecutionStatus, IdentityBinding, NormalizedDisposition, Operation, Verdict,
};
use sea_forge_ledger::LedgerStream;
use sea_forge_server::governed_execution_boundary::{
    emit_authorized_invocation, BoundaryError, InvocationLedger,
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

fn sha256_hex(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}

/// A stable, real digest for the canonical model under test — never one of
/// the known pseudo-identities.
fn local_model_sha256() -> String {
    sha256_hex(b"canonical-model-t05-governed-execution")
}

const ALLOW_POLICY: &str = "version: \"0.1\"\nrules:\n  - name: allow-local-cmd\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n";
const ESCALATE_POLICY: &str = "version: \"0.1\"\nrules:\n  - name: escalate-local-cmd\n    verdict: escalate\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n";
const DENY_POLICY: &str = "version: \"0.1\"\nrules: []\n";

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
    /// Symlink named `sea-forge` -> this test binary. It satisfies BOTH
    /// argv0 checks at once (`command_allowed` matches on file_name,
    /// `untrusted_executable` requires the canonical path to equal
    /// current_exe) — the same fixture pattern the conformance suites use.
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

/// Evaluate one execute_command action through the REAL engine.
fn decide(
    policy: &str,
    h: &Harness,
    extra_env_keys: &[&str],
) -> sea_forge_core::types::AuthorityDecision {
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(policy).expect("policy yaml");
    let engine = PolicyAuthorityEngine::new(bundle).expect("engine");
    let mut env_keys: BTreeSet<String> = ["PATH", "HOME"].iter().map(|k| k.to_string()).collect();
    env_keys.extend(extra_env_keys.iter().map(|k| k.to_string()));
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
            run_id: "run_t05",
            case_id: "case_t05",
            plan_item_id: "item_t05",
            sequence: 1,
            action: &action,
            workspace_root: &h.workspace,
            evidence_refs: vec![],
            artifacts_root: Some(&h.artifacts),
            timeout_secs: Some(10),
            env_keys,
            domainforge_candidate: None,
            environment: None,
        })
        .expect("evaluation")
}

fn emit(
    decision: &sea_forge_core::types::AuthorityDecision,
) -> Result<sea_forge_server::governed_execution_boundary::AuthorizedInvocation, BoundaryError> {
    emit_authorized_invocation(
        decision,
        "wr-t05-001",
        &local_model_sha256(),
        &local_model_sha256(),
        "11111111-2222-4333-8444-555555555555",
        None,
    )
}

// --- E5A happy path: a REAL Allow decision yields the frozen contract ---------

#[test]
fn t05_allow_decision_emits_the_frozen_authorized_invocation() {
    let h = harness("emit");
    let decision = decide(ALLOW_POLICY, &h, &[]);
    assert_eq!(decision.verdict, Verdict::Allow);

    let invocation = emit(&decision).expect("Allow decision must emit");
    let env = &invocation.envelope;

    // Canonical v1 family shape.
    assert_eq!(env["schema_version"], "v1");
    assert_eq!(env["event_type"], "AuthorizedInvocation");
    assert_eq!(env["source_agent"], "sea-forge");
    assert_eq!(env["provenance"]["origin"], "sea-forge");

    // Frozen required payload fields, every one parsed and meaningful.
    assert_eq!(env["payload"]["work_request_id"], "wr-t05-001");
    assert!(
        env["payload"]["invocation_id"]
            .as_str()
            .unwrap()
            .starts_with("inv_"),
        "invocation_id minted by the emitter"
    );
    assert_eq!(
        env["payload"]["authority_decision_id"],
        decision.decision_id.as_str()
    );
    assert_eq!(env["payload"]["operation"], "execute_command");
    assert_eq!(
        env["payload"]["resource"],
        h.sea_forge_argv0.to_string_lossy().as_ref()
    );
    assert!(
        env["payload"]["execution_constraints"].is_object(),
        "execution_constraints is a required object"
    );
    assert_eq!(
        env["payload"]["execution_constraints"]["sandbox_class"],
        "local"
    );
    assert_eq!(env["payload"]["execution_constraints"]["timeout_secs"], 10);

    // Family identity + causality (ENV-I4): cites the GovernedWorkRequest.
    assert_eq!(env["payload"]["domain_model_hash"], local_model_sha256());
    assert_eq!(env["payload"]["namespace"], "agentic_capability_loop");
    let chain = env["provenance"]["chain"].as_array().unwrap();
    assert!(
        chain
            .iter()
            .any(|c| c == "caused_by:11111111-2222-4333-8444-555555555555"),
        "E5A must cite its governing work request: {chain:?}"
    );

    // Structurally returned record mirrors the envelope.
    assert_eq!(invocation.invocation_id, env["payload"]["invocation_id"]);
    assert_eq!(invocation.authority_decision_id, decision.decision_id);
}

#[test]
fn t05_emitted_envelope_matches_the_canonical_wire_family() {
    let h = harness("family");
    let decision = decide(ALLOW_POLICY, &h, &[]);
    let invocation = emit(&decision).unwrap();
    let env = &invocation.envelope;

    // Idempotency key: sha256("|AuthorizedInvocation|<sorted payload json>") —
    // recomputed here independently of the emitter.
    let payload = env["payload"].clone();
    let expected = {
        let content = format!("|AuthorizedInvocation|{payload}");
        sha256_hex(content.as_bytes())
    };
    assert_eq!(env["idempotency_key"], expected);
    assert_eq!(env["idempotency_key"].as_str().unwrap().len(), 64);

    // Event ids are UUID-v4 shaped.
    let event_id = env["event_id"].as_str().unwrap();
    assert_eq!(event_id.len(), 36);
    assert_eq!(&event_id[14..15], "4");

    // occurred_at carries the family's RFC3339 +00:00 shape.
    let occurred_at = env["occurred_at"].as_str().unwrap();
    assert!(
        occurred_at.ends_with("+00:00") && occurred_at.len() >= 25,
        "{occurred_at}"
    );
}

// --- TOOTH 1a: deny => no invocation, no grant, no governed side effect -------

#[test]
fn t05_denied_authority_produces_no_invocation_and_no_side_effect() {
    let h = harness("deny");
    let decision = decide(DENY_POLICY, &h, &[]);
    assert_eq!(decision.verdict, Verdict::Deny);

    // No canonical AuthorizedInvocation exists for a denied decision.
    let err = emit(&decision).unwrap_err();
    assert!(
        matches!(err, BoundaryError::AuthorityNotGranted { ref disposition }
            if disposition.contains("Deny")),
        "{err}"
    );

    // And the governed side effect is unreachable through the REAL runtime
    // seam: no ActionGrant can be minted from this decision even after it is
    // committed to the ledger.
    let root = tempfile::tempdir().unwrap();
    let ledger = LedgerStream::open(root.path(), "case-t05-deny", "writer").unwrap();
    let committed = ledger
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    let action = AuthorityAction::ExecuteCommand {
        argv: vec![
            h.sea_forge_argv0.to_string_lossy().into_owned(),
            "--version".into(),
        ],
        cwd: ".".into(),
    };
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(DENY_POLICY).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle).unwrap();
    let grant = engine.grant(&decision, &committed, &action, None);
    assert!(grant.is_err(), "a denied decision must never mint a grant");
}

// --- TOOTH 1b: escalated => no invocation, no governed side effect ------------

#[test]
fn t05_escalated_authority_produces_no_invocation_and_no_side_effect() {
    let h = harness("escalate");
    let decision = decide(ESCALATE_POLICY, &h, &[]);
    assert_eq!(decision.verdict, Verdict::Escalate);

    let err = emit(&decision).unwrap_err();
    assert!(
        matches!(err, BoundaryError::AuthorityNotGranted { ref disposition }
            if disposition.contains("Escalate")),
        "{err}"
    );

    let root = tempfile::tempdir().unwrap();
    let ledger = LedgerStream::open(root.path(), "case-t05-esc", "writer").unwrap();
    let committed = ledger
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    let action = AuthorityAction::ExecuteCommand {
        argv: vec![
            h.sea_forge_argv0.to_string_lossy().into_owned(),
            "--version".into(),
        ],
        cwd: ".".into(),
    };
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(ESCALATE_POLICY).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle).unwrap();
    assert!(engine.grant(&decision, &committed, &action, None).is_err());
}

// --- Both Allow-gate conjuncts are load-bearing --------------------------------

#[test]
fn t05_both_allow_conjuncts_are_load_bearing_on_real_decisions() {
    let h = harness("conjuncts");
    let decision = decide(ALLOW_POLICY, &h, &[]);

    // verdict=Allow + disposition=Degraded => refused.
    let mut degraded = serde_json::to_value(&decision).unwrap();
    degraded["normalized_disposition"] = serde_json::json!("degraded");
    let degraded: sea_forge_core::types::AuthorityDecision =
        serde_json::from_value(degraded).unwrap();
    assert!(matches!(
        emit(&degraded),
        Err(BoundaryError::AuthorityNotGranted { .. })
    ));

    // verdict=Deny + disposition=Allow => refused.
    let mut denied = serde_json::to_value(&decision).unwrap();
    denied["verdict"] = serde_json::json!("deny");
    let denied: sea_forge_core::types::AuthorityDecision = serde_json::from_value(denied).unwrap();
    assert!(matches!(
        emit(&denied),
        Err(BoundaryError::AuthorityNotGranted { .. })
    ));
    assert_ne!(
        NormalizedDisposition::Allow,
        NormalizedDisposition::Degraded
    );
}

// --- Identity / correlation / causality emission gates -------------------------

#[test]
fn t05_emission_refuses_placeholder_and_drifted_model_identity() {
    let h = harness("identity");
    let decision = decide(ALLOW_POLICY, &h, &[]);

    // Standalone fallback constant.
    let fallback = {
        let mut hasher = Sha256::new();
        hasher.update(b"agentic_capability_loop");
        format!("{:x}", hasher.finalize())
    };
    assert!(matches!(
        emit_authorized_invocation(
            &decision,
            "wr-t05-001",
            &fallback,
            &fallback,
            "11111111-2222-4333-8444-555555555555",
            None
        ),
        Err(BoundaryError::PlaceholderIdentity { .. })
    ));

    // All-zero digest.
    assert!(matches!(
        emit_authorized_invocation(
            &decision,
            "wr-t05-001",
            &"0".repeat(64),
            &"0".repeat(64),
            "11111111-2222-4333-8444-555555555555",
            None
        ),
        Err(BoundaryError::PlaceholderIdentity { .. })
    ));

    // Malformed hash.
    assert!(matches!(
        emit_authorized_invocation(
            &decision,
            "wr-t05-001",
            "deadbeef",
            "deadbeef",
            "11111111-2222-4333-8444-555555555555",
            None
        ),
        Err(BoundaryError::PlaceholderIdentity { .. })
    ));

    // Valid-but-different model than locally resolved => drift.
    let other = sha256_hex(b"a-different-canonical-model");
    assert!(matches!(
        emit_authorized_invocation(
            &decision,
            "wr-t05-001",
            &other,
            &local_model_sha256(),
            "11111111-2222-4333-8444-555555555555",
            None
        ),
        Err(BoundaryError::DomainDrift { .. })
    ));
}

#[test]
fn t05_emission_requires_meaningful_correlation_and_causality() {
    let h = harness("correlation");
    let decision = decide(ALLOW_POLICY, &h, &[]);

    assert!(matches!(
        emit_authorized_invocation(
            &decision,
            "   ",
            &local_model_sha256(),
            &local_model_sha256(),
            "11111111-2222-4333-8444-555555555555",
            None
        ),
        Err(BoundaryError::OpaqueWorkRequest)
    ));
    assert!(matches!(
        emit_authorized_invocation(
            &decision,
            "wr-t05-001",
            &local_model_sha256(),
            &local_model_sha256(),
            "unknown",
            None
        ),
        Err(BoundaryError::PlaceholderField { .. })
    ));
}

// --- Full governed cycle over REAL components: authorize -> emit -> execute ----

/// Side-effect helper executed as the governed child process (the engine's
/// trusted-self-executable gate permits only this test binary as argv[0]).
#[test]
fn t05_side_effect_helper() {
    if std::env::var("T05_SIDE_EFFECT_HELPER").as_deref() != Ok("1") {
        return;
    }
    std::fs::write(".t05_governed_marker", "side effect occurred").expect("marker");
}

#[test]
fn t05_governed_side_effect_runs_only_after_authorization_and_emission() {
    let h = harness("full-cycle");
    let argv0 = h.sea_forge_argv0.to_string_lossy().into_owned();
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(ALLOW_POLICY).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle).unwrap();

    let mut env: BTreeMap<String, String> = BTreeMap::new();
    env.insert("PATH".into(), std::env::var("PATH").unwrap_or_default());
    env.insert("HOME".into(), "/tmp".into());
    env.insert("T05_SIDE_EFFECT_HELPER".into(), "1".into());

    let action = AuthorityAction::ExecuteCommand {
        argv: vec![
            argv0.clone(),
            "--exact".into(),
            "t05_side_effect_helper".into(),
        ],
        cwd: ".".into(),
    };
    let env_keys: BTreeSet<String> = env.keys().cloned().collect();
    let decision = engine
        .evaluate(AuthorityEvaluation {
            actor: &actor(),
            binding: binding(),
            run_id: "run_t05_cycle",
            case_id: "case_t05_cycle",
            plan_item_id: "item_t05_cycle",
            sequence: 1,
            action: &action,
            workspace_root: &h.workspace,
            evidence_refs: vec![],
            artifacts_root: Some(&h.artifacts),
            timeout_secs: Some(10),
            env_keys,
            domainforge_candidate: None,
            environment: None,
        })
        .unwrap();
    assert_eq!(decision.verdict, Verdict::Allow);

    // 1. The canonical invocation exists BEFORE the side effect can occur.
    let invocation = emit(&decision).expect("emission precedes execution");
    let mut ledger_registry = InvocationLedger::default();
    let generation = ledger_registry
        .admit_authorized_invocation(&invocation.envelope, &local_model_sha256())
        .expect("admission");
    assert_eq!(generation, 1);

    // 2. Only now does the REAL runtime perform the governed side effect,
    //    using a grant minted from the SAME committed decision.
    let root = tempfile::tempdir().unwrap();
    let ledger = LedgerStream::open(root.path(), "case-t05-cycle", "writer").unwrap();
    let committed = ledger
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    let grant = engine
        .grant(&decision, &committed, &action, None)
        .expect("Allow decision mints a grant");
    let result = sea_forge_runtime::execute(
        grant,
        &ExecutionRequest {
            plan_item_id: "item_t05_cycle".into(),
            operation: Operation::ExecuteCommand {
                argv: vec![argv0, "--exact".into(), "t05_side_effect_helper".into()],
                cwd: ".".into(),
            },
            timeout_secs: 10,
            env,
            compensating_controls: vec![],
        },
        "run_t05_cycle",
        &h.workspace,
        &h.artifacts,
    )
    .expect("governed execution");
    assert_eq!(result.status, ExecutionStatus::Completed);
    assert!(
        h.workspace.join(".t05_governed_marker").exists(),
        "the governed side effect actually occurred — after authorization and emission"
    );

    // 3. The execution environment reports back against the EXACT authorized
    //    invocation and settles.
    let observation = serde_json::json!({
        "schema_version": "v1",
        "event_id": "99999999-8888-4777-8666-555555555555",
        "source_agent": "execution-environment",
        "event_type": "ExecutionObservation",
        "occurred_at": "2026-08-25T12:00:00+00:00",
        "idempotency_key": sha256_hex(b"obs-full-cycle"),
        "payload": {
            "domain_model_hash": local_model_sha256(),
            "namespace": "agentic_capability_loop",
            "work_request_id": "wr-t05-001",
            "invocation_id": invocation.invocation_id,
            "execution_status": "completed",
            "observed_effects": [{"kind": "file_write", "ref": ".t05_governed_marker"}]
        },
        "provenance": {
            "origin": "execution-environment",
            "chain": [format!("caused_by:{}", invocation.event_id)]
        }
    });
    let outcome = ledger_registry
        .settle_execution_observation(&observation, &local_model_sha256())
        .expect("the exact authorized invocation settles");
    match outcome {
        sea_forge_server::governed_execution_boundary::ObservationOutcome::Settled {
            authority_decision_id,
            execution_status,
            ..
        } => {
            assert_eq!(authority_decision_id, decision.decision_id);
            assert_eq!(execution_status, "completed");
        }
        other => panic!("expected Settled, got {other:?}"),
    }
}

// --- Admission-level producer forgery (E5A direction) ---------------------------

#[test]
fn t05_forged_authorized_invocation_stamps_are_refused_at_admission() {
    let h = harness("forge-e5a");
    let decision = decide(ALLOW_POLICY, &h, &[]);
    let invocation = emit(&decision).unwrap();

    // Any stamp other than sea_forge cannot enter the governed ledger —
    // including the execution environment self-authorizing.
    for forger in [
        "execution-environment",
        "swe-seed",
        "godspeed-agent",
        "context-kernel",
        "mystery-agent",
    ] {
        let mut forged = invocation.envelope.clone();
        forged["source_agent"] = serde_json::json!(forger);
        let mut registry = InvocationLedger::default();
        let err = registry
            .admit_authorized_invocation(&forged, &local_model_sha256())
            .unwrap_err();
        assert!(
            matches!(
                err,
                BoundaryError::NotAuthoritative {
                    authoritative: "sea_forge",
                    ..
                }
            ) || matches!(err, BoundaryError::UnknownAgent { .. }),
            "forger {forger} must be refused: {err}"
        );
    }

    // Unknown stamps fail closed rather than being coerced.
    let mut unknown = invocation.envelope.clone();
    unknown["source_agent"] = serde_json::json!("not-in-vocabulary");
    let mut registry = InvocationLedger::default();
    assert!(matches!(
        registry.admit_authorized_invocation(&unknown, &local_model_sha256()),
        Err(BoundaryError::UnknownAgent { .. })
    ));
}
