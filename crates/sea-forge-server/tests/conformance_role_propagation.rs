//! F-08 regression: the server must evaluate authority for the role the
//! identity gate actually verified, not a hardcoded `ActorRole::Operator`.
//!
//! One policy whose *only* allow rule targets `R-SO` (SecurityOfficer); one
//! plan item dispatched twice against it:
//! - carrying a `ResolvedActor` resolved through real `IdentityBindings` for
//!   an actor bound to `R-SO` ⇒ the authority decision is `allow`;
//! - carrying no verified identity (`handle_request`, the in-process shape
//!   that legitimately never crossed the socket gate) ⇒ the evaluation falls
//!   back to the local-operator principal and the same action is denied.
//!
//! Before F-08 both halves evaluated as Operator, so a security officer bound
//! by the gate could never exercise a role-keyed rule server-side.

use sea_forge_core::types::{ActorRole, CasePlan, ItemKind, Operation, PlanItem};
use sea_forge_server::{
    handle_request, handle_request_as, identity::IdentityBindings, Request, ServerConfig,
    ServerState,
};
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// A peer uid the fabricated connection presents (`PeerIdentity` is pub).
const PEER_UID: u32 = 4242;
/// The actor the gate verifies, bound to exactly one non-operator role.
const ACTOR: &str = "so_local";

fn so_policy(root: &Path) -> std::path::PathBuf {
    let path = root.join("policy.yaml");
    fs::write(
        &path,
        // The ONLY rule in the bundle targets SecurityOfficer. An evaluation
        // made as any other role finds no matching rule and fails closed.
        "version: \"0.1\"\nrules:\n  - name: command\n    verdict: allow\n    actor_role: R-SO\n    operation_kind: execute_command\n    argv0: sea-forge\n",
    )
    .unwrap();
    path
}

fn trusted_self_executable(dir: &Path) -> std::path::PathBuf {
    let link = dir.join("sea-forge");
    #[cfg(unix)]
    std::os::unix::fs::symlink(std::env::current_exe().unwrap(), &link).unwrap();
    link
}

fn sandboxed_task(argv: Vec<String>) -> PlanItem {
    PlanItem {
        plan_item_id: "task_a".into(),
        name: "task_a".into(),
        operations: vec![Operation::ExecuteCommand {
            argv,
            cwd: ".".into(),
        }],
        entry_criteria: vec![],
        entry_criteria_mode: Default::default(),
        exit_criteria: vec![],
        settlement_criteria: Default::default(),
        settlement_criteria_ref: None,
        item_kind: ItemKind::SandboxedTask,
        sandbox_class: None,
        parent_stage: None,
        // Required: an optional item is auto-completed without ever
        // dispatching, so the authority evaluation would never run.
        markers: sea_forge_core::types::ItemMarkers {
            required: true,
            ..Default::default()
        },
        max_instances: 1,
        depends_on: vec![],
        environment: None,
        proposed_by: None,
    }
}

fn plan(items: Vec<PlanItem>) -> CasePlan {
    CasePlan {
        version: "0.2".into(),
        plan_id: "plan_role".into(),
        case_id: "case_placeholder".into(),
        run_id: "run_placeholder".into(),
        intent_id: "int_role".into(),
        items,
        template_ref: None,
        job_contract_ref: None,
    }
}

fn state(root: &Path) -> Arc<ServerState> {
    Arc::new(
        ServerState::new(ServerConfig {
            socket_path: root.join("unused.sock"),
            root: root.to_path_buf(),
            ..ServerConfig::default()
        })
        .unwrap(),
    )
}

fn submit_request(root: &Path, plan_path: &Path, policy_path: &Path) -> Request {
    // F-16: plan/policy references are cell-relative spellings under the root.
    serde_json::from_value(serde_json::json!({
        "verb": "submit",
        "plan": plan_path.strip_prefix(root).unwrap().to_string_lossy(),
        "policy": policy_path.strip_prefix(root).unwrap().to_string_lossy(),
        "entity": ACTOR,
        "process": "test",
        "timeout": 60,
    }))
    .unwrap()
}

/// Resolve the gate exactly as `dispatch_bounded` would for a socket peer:
/// claim + peer credential against real bindings. This is the only way to
/// obtain a `ResolvedActor` — its construction stays sealed behind `resolve`.
fn resolved_security_officer() -> sea_forge_server::identity::ResolvedActor {
    let bindings = IdentityBindings {
        bindings: vec![sea_forge_server::identity::IdentityBinding {
            uid: PEER_UID,
            actor_id: ACTOR.into(),
            roles: vec![ActorRole::SecurityOfficer],
        }],
    };
    let claim = sea_forge_server::identity::ActorClaim {
        actor_id: ACTOR.into(),
        role: ActorRole::SecurityOfficer,
    };
    bindings
        .resolve(
            Some(&claim),
            Some(sea_forge_server::identity::PeerIdentity { uid: PEER_UID }),
        )
        .expect("a bound actor claiming a held role must resolve")
}

fn authority_verdicts(root: &Path, case_id: &str) -> Vec<String> {
    let ledger =
        sea_forge_ledger::LedgerStream::open(root, format!("case-{case_id}"), "audit").unwrap();
    ledger
        .read_entries()
        .unwrap()
        .iter()
        .filter(|entry| entry.record_kind == "authority_decision")
        .map(|entry| {
            entry.payload["verdict"]
                .as_str()
                .unwrap_or_default()
                .to_owned()
        })
        .collect()
}

#[test]
fn f08_verified_role_reaches_the_authority_evaluation() {
    let root = tempfile::tempdir().unwrap();
    let policy_path = so_policy(root.path());
    // The trusted executable is this test binary; libtest args make the
    // child a fast, exit-zero no-op (the conformance_case_episode idiom).
    let argv = vec![
        trusted_self_executable(root.path())
            .to_string_lossy()
            .into_owned(),
        "--exact".into(),
        "__no_test_matches_this_name__".into(),
    ];
    let plan_path = root.path().join("plan.json");
    fs::write(
        &plan_path,
        serde_json::to_vec(&plan(vec![sandboxed_task(argv)])).unwrap(),
    )
    .unwrap();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // Half 1 — the verified SecurityOfficer: the role-keyed rule must fire.
    let verified_state = state(root.path());
    let resolved = resolved_security_officer();
    let response = rt.block_on(handle_request_as(
        submit_request(root.path(), &plan_path, &policy_path),
        &verified_state,
        Some(&resolved),
    ));
    let case_id = response["case_id"].as_str().unwrap_or("").to_owned();
    assert!(!case_id.is_empty(), "{response}");
    let verdicts = authority_verdicts(root.path(), &case_id);
    assert_eq!(
        verdicts,
        vec!["allow".to_string()],
        "the verified R-SO principal must be evaluated under the R-SO rule: {verdicts:?}"
    );

    // Half 2 — no verified identity (in-process shape): the fallback evaluates
    // as the local-operator principal, finds no operator rule, and is denied.
    let root2 = tempfile::tempdir().unwrap();
    let policy_path2 = so_policy(root2.path());
    let argv2 = vec![
        trusted_self_executable(root2.path())
            .to_string_lossy()
            .into_owned(),
        "--exact".into(),
        "__no_test_matches_this_name__".into(),
    ];
    let plan_path2 = root2.path().join("plan.json");
    fs::write(
        &plan_path2,
        serde_json::to_vec(&plan(vec![sandboxed_task(argv2)])).unwrap(),
    )
    .unwrap();
    let state2 = state(root2.path());
    let response2 = rt.block_on(handle_request(
        submit_request(root2.path(), &plan_path2, &policy_path2),
        &state2,
    ));
    let case_id2 = response2["case_id"].as_str().unwrap_or("").to_owned();
    assert!(!case_id2.is_empty(), "{response2}");
    let verdicts2 = authority_verdicts(root2.path(), &case_id2);
    assert!(
        !verdicts2.is_empty() && verdicts2.iter().all(|v| v != "allow"),
        "without a verified identity the same action must not match the \
         R-SO-only rule (evaluated as the operator fallback): {verdicts2:?}"
    );
}
