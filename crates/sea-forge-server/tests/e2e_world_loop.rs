//! End-to-end world loop, SEA-Forge's hop (driven by `scripts/e2e-world-loop.sh`).
//!
//! Reads the real E4 (SWE_SEED) and E3 (Context Kernel) from `E2E_DIR`, accepts
//! the request through the VERIFIED intake (the world is recomputed from the
//! source SEA-Forge itself loaded, and must equal the world DomainForge's CLI
//! gave the driver), decides with the real authority engine, and writes E5A, E5B
//! and E6. Skipped unless the driver sets `E2E_DIR`.
//!
//! Execution is simulated at E5B: the observation is a stand-in for what the
//! execution environment reports. Real execution under a CEP decision is
//! exercised by Cognate's `just sea-forge-live`.

use std::path::{Path, PathBuf};

use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{
    Actor, ActorRole, ActorType, AuthorityAction, BindingResolution, IdentityBinding,
};
use sea_forge_domainforge::{SeaSourceSet, SourceFile, WorldRegistry};
use sea_forge_server::governed_execution_boundary::{emit_authorized_invocation, InvocationLedger};
use sea_forge_server::governed_settlement_return::{
    emit_operational_settlement, SettlementOptionalFields,
};
use sea_forge_server::governed_work_ingress::accept_verified_governed_work_request;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

fn policy(verdict: &str) -> String {
    format!(
        "version: \"0.1\"\nrules:\n  - name: local-cmd\n    verdict: {verdict}\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n"
    )
}

fn sha256_hex(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}

fn read(dir: &Path, name: &str) -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(dir.join(name)).unwrap_or_else(|e| panic!("{name}: {e}")),
    )
    .unwrap()
}

fn write(dir: &Path, name: &str, v: &Value) {
    std::fs::write(
        dir.join(name),
        serde_json::to_string_pretty(v).unwrap() + "\n",
    )
    .unwrap();
}

#[test]
fn hop_3_verified_intake_decision_execution_and_settlement() {
    let Some(dir) = std::env::var_os("E2E_DIR").map(PathBuf::from) else {
        eprintln!("SKIP: E2E_DIR not set");
        return;
    };
    let world = std::env::var("E2E_WORLD_REF").expect("E2E_WORLD_REF");
    let hash = std::env::var("E2E_DOMAIN_HASH").expect("E2E_DOMAIN_HASH");
    let wr_id = std::env::var("E2E_WORK_REQUEST_ID").expect("E2E_WORK_REQUEST_ID");

    // SEA-Forge loads the source itself and must arrive at DomainForge's world.
    let src = std::fs::read_to_string(dir.join("demo.sea")).unwrap();
    let mut worlds = WorldRegistry::new();
    let registered = worlds
        .register_source_set(
            "demo",
            &SeaSourceSet {
                entry_uri: "demo.sea".into(),
                files: vec![SourceFile {
                    uri: "demo.sea".into(),
                    sha256: sha256_hex(src.as_bytes()),
                    content: src,
                }],
            },
        )
        .unwrap()
        .to_string();
    assert_eq!(
        registered, world,
        "SEA-Forge's recomputed world must equal DomainForge's"
    );

    // E4 intake: verified against the registry, not trusted on syntax.
    let e4 = read(&dir, "e4_governed_work_request.json");
    let e3 = read(&dir, "e3_context_packet.json");
    let intent = accept_verified_governed_work_request(&e4, &e3, &hash, &worlds)
        .expect("the real E4 passes the verified intake");
    assert_eq!(intent.world_ref, world);
    assert_eq!(intent.work_request_id, wr_id);
    assert_eq!(intent.context_completeness.as_deref(), Some("complete"));

    // The EXACT observer context is recorded on the intent: the authority
    // record can be tied to the precise context bundle available at decision
    // time. What was made available is never authority.
    let bound = intent
        .context_bundle_ref
        .as_ref()
        .expect("E4 binds its exact context bundle identity");
    assert_eq!(
        bound["envelope_id"],
        e4["payload"]["context_bundle_ref"]["envelope_id"],
    );
    assert_eq!(bound["world_ref"], world.as_str());

    // Decide with the real authority engine.
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("workspace");
    let artifacts = root.path().join("artifacts");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&artifacts).unwrap();
    let argv0 = root.path().join("sea-forge");
    #[cfg(unix)]
    std::os::unix::fs::symlink(std::env::current_exe().unwrap(), &argv0).unwrap();
    let actor = Actor {
        actor_id: "operator_local".into(),
        role: ActorRole::Operator,
    };
    let binding = IdentityBinding {
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
    };
    // `E2E_POLICY_VERDICT=deny` drives the denied path of the loop.
    let verdict = std::env::var("E2E_POLICY_VERDICT").unwrap_or_else(|_| "allow".into());
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(&policy(&verdict)).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle).unwrap();
    let action = AuthorityAction::ExecuteCommand {
        argv: vec![argv0.to_string_lossy().into_owned(), "--version".into()],
        cwd: ".".into(),
    };
    let decision = engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding,
            run_id: "run_e2e",
            case_id: "case_e2e",
            plan_item_id: "item_e2e",
            sequence: 1,
            action: &action,
            workspace_root: &workspace,
            evidence_refs: vec![],
            artifacts_root: Some(&artifacts),
            timeout_secs: Some(10),
            env_keys: ["PATH", "HOME"].iter().map(|k| k.to_string()).collect(),
            domainforge_candidate: None,
            environment: None,
        })
        .unwrap();

    // E5A authorised invocation, then E5B observation bound to exactly that invocation.
    let invocation = emit_authorized_invocation(
        &decision,
        &wr_id,
        &hash,
        &hash,
        e4["event_id"].as_str().unwrap(),
        None,
        &world,
    );
    if verdict != "allow" {
        // A denied request authorises nothing: no invocation, no settlement.
        assert!(
            invocation.is_err(),
            "a {verdict} decision must not authorise an invocation"
        );
        return;
    }
    let inv = invocation.expect("an Allow decision authorises the invocation");
    let mut ledger = InvocationLedger::default();
    ledger
        .admit_authorized_invocation(&inv.envelope, &hash)
        .unwrap();

    let criteria: Vec<String> = intent.settlement_criteria.clone();
    let observed: Vec<Value> = criteria.iter().map(|c| json!({"effect": c})).collect();
    let e5b_event_id = "55555555-0000-4000-8000-0000000e2e01".to_string();
    let obs = json!({
        "schema_version": "v1",
        "event_id": e5b_event_id,
        "source_agent": "execution-environment",
        "event_type": "ExecutionObservation",
        "occurred_at": "2026-10-05T12:00:00+00:00",
        "idempotency_key": sha256_hex(b"e2e-observation"),
        "payload": {
            "namespace": "agentic_capability_loop",
            "domain_model_hash": hash,
            "world_ref": world,
            "work_request_id": wr_id,
            "invocation_id": inv.invocation_id,
            "execution_status": "completed",
            "observed_effects": observed,
        },
        "provenance": {"origin": "execution-environment", "chain": [format!("caused_by:{}", inv.event_id)]}
    });
    let outcome = ledger
        .settle_execution_observation(&obs, &hash)
        .expect("the observation settles against its invocation");

    // E6 operational settlement from the settled observation against the declared criteria.
    let settlement = emit_operational_settlement(
        &outcome,
        &criteria,
        &hash,
        &hash,
        &inv.event_id,
        &e5b_event_id,
        SettlementOptionalFields::default(),
    )
    .expect("the settlement emits");
    assert_eq!(settlement.envelope["payload"]["world_ref"], world.as_str());

    write(&dir, "e5a_authorized_invocation.json", &inv.envelope);
    write(&dir, "e5b_execution_observation.json", &obs);
    write(&dir, "e6_operational_settlement.json", &settlement.envelope);
}

/// Falsification: a request whose context bundle identity does not sit in the
/// request's world is refused at intake -- the lineage never silently crosses
/// semantic worlds.
#[test]
#[allow(deprecated)]
fn a_context_bundle_ref_outside_the_world_is_refused() {
    let Some(dir) = std::env::var_os("E2E_DIR").map(PathBuf::from) else {
        eprintln!("SKIP: E2E_DIR not set");
        return;
    };
    let hash = std::env::var("E2E_DOMAIN_HASH").expect("E2E_DOMAIN_HASH");

    let mut e4 = read(&dir, "e4_governed_work_request.json");
    let e3 = read(&dir, "e3_context_packet.json");
    e4["payload"]["context_bundle_ref"]["world_ref"] =
        json!(format!("world:other@sha256:{}", "d".repeat(64)));

    let err =
        sea_forge_server::governed_work_ingress::accept_governed_work_request(&e4, &e3, &hash)
            .unwrap_err();
    assert!(
        format!("{err:?}").contains("World"),
        "a bundle from another world must be refused: {err:?}"
    );
}
