#![cfg(target_os = "linux")]

//! SEC-01/SEC-06 on the planned-command path: an `execute_command` granted
//! the `isolated` class runs under its own subordinate host identity, with
//! the isolated read posture, and no shared-UID fallback.
//!
//! Authority admits only the trusted executable on the host path, so the
//! command is this test binary re-executed (`isolated_command_fixture`); the
//! isolated sandbox makes exactly that executable's directory readable.
//! Positive control: authorized work in the workspace succeeds. Denials: a
//! world-readable host file outside every grant, and another run's
//! workspace. Hosts without subordinate-ID delegation or the helper binary
//! report NOT-EXECUTABLE; set `SEA_FORGE_REQUIRE_WORKLOAD_ISOLATION=1` to
//! make that a failure.

use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{
    Actor, ActorRole, AuthorityAction, ExecutionRequest, ExecutionResult, ExecutionStatus,
    Operation, Verdict,
};
use sea_forge_ledger::LedgerStream;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

const POLICY: &str = "version: \"0.1\"\nidentity:\n  source: test\n  allow_unresolved: false\nidentity_bindings:\n  - principal: operator_local\n    actor_type: human\n    role: operator\nrules:\n  - name: isolated-self\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    sandbox_class: isolated\n";
const ACTION: &str = "ISOLATED_FIXTURE_ACTION";

/// Runs inside the isolated workload when re-executed with `ACTION` set.
#[test]
fn isolated_command_fixture() {
    let Ok(action) = std::env::var(ACTION) else {
        return;
    };
    let uid = fs::metadata("/proc/self").unwrap().uid();
    fs::write("uid.txt", uid.to_string()).unwrap();
    match action.split_once(':') {
        None if action == "make" => fs::write("made.txt", "made").unwrap(),
        Some(("read", path)) => {
            let text = fs::read_to_string(path).expect("read denied");
            fs::write("stolen.txt", text).unwrap();
        }
        _ => panic!("unknown fixture action {action}"),
    }
}

fn available() -> bool {
    let ok = sea_forge_sandbox::workload::WorkloadIdentityPool::from_host().is_ok()
        && sea_forge_sandbox::workload::sibling_helper().is_ok();
    if !ok {
        assert!(
            std::env::var_os("SEA_FORGE_REQUIRE_WORKLOAD_ISOLATION").is_none(),
            "workload isolation required but unavailable (build sea-forge-sandbox bins)"
        );
        eprintln!("NOT-EXECUTABLE: workload isolation prerequisites absent");
    }
    ok
}

/// Authorize and run one isolated fixture action; returns the result and
/// the UID the fixture saw inside its namespace.
fn run(root: &Path, run_id: &str, action: &str) -> (ExecutionResult, String) {
    let workspace = root.join(run_id).join("workspace");
    let artifacts = root.join(run_id).join("artifacts");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&artifacts).unwrap();
    fs::set_permissions(root.join(run_id), fs::Permissions::from_mode(0o711)).unwrap();
    let operation = Operation::ExecuteCommand {
        argv: vec![
            std::env::current_exe().unwrap().display().to_string(),
            "--exact".into(),
            "isolated_command_fixture".into(),
            "--test-threads=1".into(),
        ],
        cwd: ".".into(),
    };
    let action_value = AuthorityAction::from(&operation);
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(POLICY).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let actor = Actor {
        actor_id: "operator_local".into(),
        role: ActorRole::Operator,
    };
    let decision = engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding: bundle.resolve_identity(&actor.actor_id, actor.role.clone()),
            run_id,
            case_id: "case_isolated_command",
            plan_item_id: "item",
            sequence: 1,
            action: &action_value,
            workspace_root: &workspace,
            evidence_refs: vec!["intent:isolated_command".into()],
            artifacts_root: Some(&artifacts),
            timeout_secs: Some(30),
            env_keys: BTreeSet::from([ACTION.to_string()]),
            opa_candidate: None,
            environment: None,
        })
        .unwrap();
    assert_eq!(
        decision.verdict,
        Verdict::Allow,
        "{:?}",
        decision.reason_codes
    );
    let ledger =
        LedgerStream::open(&root.join("ledger"), format!("auth-{run_id}"), "test").unwrap();
    let committed = ledger
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    let grant = engine
        .grant(&decision, &committed, &action_value, None)
        .unwrap();
    assert_eq!(grant.sandbox_class(), "isolated");
    let request = ExecutionRequest {
        plan_item_id: "item".into(),
        operation,
        env: BTreeMap::from([(ACTION.to_string(), action.to_string())]),
        timeout_secs: 30,
        compensating_controls: Vec::new(),
    };
    let result =
        sea_forge_runtime::execute(grant, &request, run_id, &workspace, &artifacts).unwrap();
    let uid = fs::read_to_string(workspace.join("uid.txt")).unwrap_or_default();
    (result, uid)
}

#[test]
fn isolated_execute_command_runs_under_its_own_identity_and_read_posture() {
    if !available() {
        return;
    }
    let base = tempfile::tempdir().unwrap();
    let root = base.path();
    fs::create_dir_all(root.join("ledger")).unwrap();
    fs::set_permissions(root, fs::Permissions::from_mode(0o711)).unwrap();
    // World-readable host file in a world-traversable tree: only the read
    // posture (not DAC) keeps it from the command.
    let host = root.join("host");
    fs::create_dir_all(&host).unwrap();
    fs::set_permissions(&host, fs::Permissions::from_mode(0o755)).unwrap();
    let host_file = host.join("notes.txt");
    fs::write(&host_file, "HOST-NOTES").unwrap();
    fs::set_permissions(&host_file, fs::Permissions::from_mode(0o644)).unwrap();

    // Positive control and identity: the command works in its workspace,
    // under a subordinate host UID that is not the launcher's.
    let (result, inner_uid) = run(root, "run_a", "make");
    assert_eq!(result.status, ExecutionStatus::Completed, "{result:?}");
    assert_eq!(result.exit_code, Some(0), "{result:?}");
    let made = root.join("run_a/workspace/made.txt");
    let launcher = fs::metadata("/proc/self").unwrap().uid();
    let owner = fs::metadata(&made).unwrap().uid();
    assert_ne!(owner, launcher, "command ran under the launcher's UID");
    assert!(owner >= 100_000, "not a subordinate identity: {owner}");
    assert_eq!(inner_uid.trim(), "1000", "inner identity");
    // The launcher keeps trusted read access to what the workload wrote.
    assert_eq!(fs::read_to_string(&made).unwrap(), "made");

    // Denials: a world-readable host file, then another run's workspace.
    let (result, _) = run(root, "run_b", &format!("read:{}", host_file.display()));
    assert_ne!(result.exit_code, Some(0), "host file readable: {result:?}");
    assert!(!root.join("run_b/workspace/stolen.txt").exists());
    let (result, _) = run(root, "run_c", &format!("read:{}", made.display()));
    assert_ne!(
        result.exit_code,
        Some(0),
        "other run's workspace readable: {result:?}"
    );
    assert!(!root.join("run_c/workspace/stolen.txt").exists());
    // Grants are withdrawn once each execution ends.
    let mode = fs::metadata(root.join("run_a/workspace")).unwrap().mode();
    assert_eq!(mode & 0o077, 0, "workspace grant outlived the execution");
}
