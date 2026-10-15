#![cfg(unix)]

//! Routing of `execute_sandbox_command`: the host entry point refuses it, and
//! the OpenShell entry point runs it only when the grant and the request both
//! match the authorized projection. The `openshell` CLI is a POSIX stub that
//! logs every call, so a refused request is shown to create nothing.

use sea_forge_authority::{
    ActionGrant, AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine,
};
use sea_forge_core::types::{
    Actor, ActorRole, AuthorityAction, ExecutionRequest, ExecutionStatus, Operation,
};
use sea_forge_ledger::LedgerStream;
use sea_forge_sandbox::OpenShellBinding;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const CELL: &str = "cell_test";
const MODEL: &str = "sha256:model";
const IMAGE: &str = "registry.example/cell:1";
const RUN_ID: &str = "run_sandbox_command";
const POLICY: &str = "version: 1\n";
const SANDBOX_JSON: &str = r#"{"schema":"domainforge.openshell.sandbox/v1","workspaces":[{"name":"workspace","sandbox_path":"/sandbox/workspace"}]}"#;
const MAPPING_REPORT: &str = r#"{"schema":"domainforge.openshell.mapping/v1","unmapped":[]}"#;

const OPENSHELL_STUB: &str = r##"#!/bin/sh
LOG="__LOG__"
echo "openshell $*" >> "$LOG"
if [ "$1" = "--gateway" ]; then
  shift 2
fi
case "$1" in
  --version) echo "openshell 0.1.3"; exit 0 ;;
  status) exit 0 ;;
  logs) exit 0 ;;
  policy) echo '{"hash":"h1","version":3,"config_revision":"r7","policy_source":"sandbox"}'; exit 0 ;;
  sandbox)
    case "$2" in
      create|delete|upload) exit 0 ;;
      get) echo "version: 1"; exit 0 ;;
      download) mkdir -p "$5" && echo done > "$5/result.txt"; exit $? ;;
      exec) echo "stub-stdout"; exit 0 ;;
    esac
    ;;
esac
echo "stub openshell: unhandled: $*" >&2
exit 2
"##;

const PROVER_STUB: &str = r##"#!/bin/sh
LOG="__LOG__"
echo "openshell-prover $*" >> "$LOG"
echo '{"schema_version":1,"result":"within_boundary","exit_code":0,"coverage":{"domains":["filesystem","network_l4","network_rest","process","landlock"]},"counterexample":null,"reason":null}'
exit 0
"##;

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    projection: PathBuf,
    workspace: PathBuf,
    artifacts: PathBuf,
    result_dir: PathBuf,
    cli: PathBuf,
    prover: PathBuf,
    log: PathBuf,
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

/// The policy digest of the projection the fixture writes.
fn projection_digest() -> String {
    digest(POLICY.as_bytes())
}

fn write_script(path: &Path, body: &str, log: &Path) {
    let text = body.replace("__LOG__", &log.display().to_string());
    fs::write(path, text).expect("write stub script");
    let mut permissions = fs::metadata(path).expect("stat stub").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("make stub executable");
}

/// A projection with exactly its three artifacts and a lock covering them, plus
/// stub `openshell` and `openshell-prover` executables that log their calls.
fn fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().to_path_buf();
    let projection = root.join("projection");
    let workspace = root.join("workspace");
    let bin = root.join("bin");
    for path in [&projection, &workspace, &bin] {
        fs::create_dir_all(path).expect("create directory");
    }
    fs::write(projection.join("policy.yaml"), POLICY).expect("write policy.yaml");
    fs::write(projection.join("sandbox.json"), SANDBOX_JSON).expect("write sandbox.json");
    fs::write(projection.join("mapping-report.json"), MAPPING_REPORT)
        .expect("write mapping-report.json");
    let lock = json!({
        "cell_id": CELL,
        "openshell_version": "0.1.3",
        "model": { "hash": MODEL },
        "policy_digest": projection_digest(),
        "files": {
            "mapping-report.json": digest(MAPPING_REPORT.as_bytes()),
            "policy.yaml": digest(POLICY.as_bytes()),
            "sandbox.json": digest(SANDBOX_JSON.as_bytes()),
        },
    });
    fs::write(projection.join("openshell.lock"), lock.to_string()).expect("write lock");

    let log = root.join("calls.log");
    let cli = bin.join("openshell");
    let prover = bin.join("openshell-prover");
    write_script(&cli, OPENSHELL_STUB, &log);
    write_script(&prover, PROVER_STUB, &log);

    Fixture {
        root: root.clone(),
        projection,
        workspace,
        artifacts: root.join("artifacts"),
        result_dir: root.join("out"),
        cli,
        prover,
        log,
        _dir: dir,
    }
}

fn binding(fx: &Fixture, image: Option<&str>) -> OpenShellBinding {
    OpenShellBinding {
        projection_dir: fx.projection.clone(),
        gateway: None,
        image: image.map(str::to_string),
        cli: fx.cli.clone(),
        prover: fx.prover.clone(),
        local_microvm_gateway: false,
    }
}

fn policy_yaml(policy_digest: &str, image: &str) -> String {
    format!(
        r#"version: "0.1"
rules:
  - name: sandbox-shell
    verdict: allow
    actor_role: operator
    operation_kind: execute_sandbox_command
    argv0: "/bin/sh"
    sandbox_class: openshell
    boundary_constraints:
      cell_id: ["{CELL}"]
      model_hash: ["{MODEL}"]
      openshell_policy_digest: ["{policy_digest}"]
      sandbox_image: ["{image}"]
"#
    )
}

/// The sandbox-command operation for `argv`, bound to the given digest and image.
fn sandbox_op(argv: &[&str], policy_digest: &str, image: &str) -> Operation {
    Operation::ExecuteSandboxCommand {
        argv: argv.iter().map(|arg| arg.to_string()).collect(),
        cwd: ".".into(),
        cell_id: CELL.into(),
        model_hash: MODEL.into(),
        policy_digest: policy_digest.into(),
        image: image.into(),
    }
}

fn request_for(operation: Operation) -> ExecutionRequest {
    ExecutionRequest {
        plan_item_id: "item_sandbox".into(),
        operation,
        timeout_secs: 30,
        env: BTreeMap::new(),
        compensating_controls: vec![],
    }
}

/// Obtains a real grant for `action` from a policy that binds `policy_digest`
/// and `image`, the way the case runner does: evaluate, commit the decision to
/// a ledger stream, then grant.
fn grant_for(
    fx: &Fixture,
    policy_digest: &str,
    image: &str,
    action: &AuthorityAction,
) -> ActionGrant {
    let bundle: AuthorityPolicyBundle =
        sea_forge_core::yaml::from_str(&policy_yaml(policy_digest, image)).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let actor = Actor {
        actor_id: "operator_local".into(),
        role: ActorRole::Operator,
    };
    let identity = bundle.resolve_identity(&actor.actor_id, actor.role.clone());
    let decision = engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding: identity,
            run_id: RUN_ID,
            case_id: "case_sandbox_command",
            plan_item_id: "item_sandbox",
            sequence: 1,
            action,
            workspace_root: &fx.workspace,
            evidence_refs: vec!["intent:sandbox_command".into()],
            artifacts_root: Some(&fx.artifacts),
            timeout_secs: Some(30),
            env_keys: BTreeSet::new(),
            domainforge_candidate: None,
            opa_candidate: None,
            environment: None,
        })
        .unwrap();
    assert_eq!(
        decision.verdict,
        sea_forge_core::types::Verdict::Allow,
        "the fixture policy must allow the sandbox command"
    );
    let ledger_root = fx.root.join("ledger");
    fs::create_dir_all(&ledger_root).unwrap();
    let stream = LedgerStream::open(&ledger_root, "sandbox-command-authority", "test").unwrap();
    let committed = stream
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    engine
        .grant(&decision, &committed, action, None)
        .expect("the allowed sandbox command must mint a grant")
}

fn calls(fx: &Fixture) -> Vec<String> {
    match fs::read_to_string(&fx.log) {
        Ok(text) => text.lines().map(str::to_string).collect(),
        Err(_) => Vec::new(),
    }
}

#[test]
fn guest_command_cannot_run_on_the_host() {
    let fx = fixture();
    let marker = fx.root.join("host-marker");
    let touch = format!("touch {}", marker.display());
    let operation = sandbox_op(
        &["/bin/sh", "-c", touch.as_str()],
        &projection_digest(),
        IMAGE,
    );
    let action = AuthorityAction::from(&operation);
    let grant = grant_for(&fx, &projection_digest(), IMAGE, &action);

    let error = sea_forge_runtime::execute(
        grant,
        &request_for(operation),
        RUN_ID,
        &fx.workspace,
        &fx.artifacts,
    )
    .expect_err("the host entry point must refuse a sandbox command");

    assert_eq!(error.class(), "sandbox_command_requires_openshell");
    assert!(
        !marker.exists(),
        "the guest command must not run on the host"
    );
}

#[test]
fn openshell_execution_requires_the_authorized_projection() {
    let fx = fixture();
    let digest = projection_digest();

    // (a) The grant was bound to a policy digest other than the projection's.
    let other_digest = "sha256:other-policy";
    let operation = sandbox_op(&["/bin/sh", "-c", "true"], other_digest, IMAGE);
    let action = AuthorityAction::from(&operation);
    let grant = grant_for(&fx, other_digest, IMAGE, &action);
    let error = sea_forge_runtime::execute_openshell(
        grant,
        &request_for(operation),
        RUN_ID,
        &fx.workspace,
        &fx.artifacts,
        binding(&fx, Some(IMAGE)),
        &fx.result_dir,
    )
    .expect_err("a grant for another policy must be refused");
    assert_eq!(error.class(), "openshell_binding_mismatch", "(a)");
    assert!(
        !calls(&fx)
            .iter()
            .any(|c| c.starts_with("openshell sandbox create")),
        "(a) must create nothing"
    );

    // (b) The projection is bound to an image other than the authorized one.
    let operation = sandbox_op(&["/bin/sh", "-c", "true"], &digest, IMAGE);
    let action = AuthorityAction::from(&operation);
    let grant = grant_for(&fx, &digest, IMAGE, &action);
    let error = sea_forge_runtime::execute_openshell(
        grant,
        &request_for(operation),
        RUN_ID,
        &fx.workspace,
        &fx.artifacts,
        binding(&fx, Some("registry.example/other:9")),
        &fx.result_dir,
    )
    .expect_err("a binding image other than the authorized one must be refused");
    assert_eq!(error.class(), "openshell_binding_mismatch", "(b)");
    assert!(
        !calls(&fx)
            .iter()
            .any(|c| c.starts_with("openshell sandbox create")),
        "(b) must create nothing"
    );

    // (c) A host execute_command request reaches the OpenShell entry point.
    // The sandbox grant's class is openshell, so the operation check decides.
    let operation = sandbox_op(&["/bin/sh", "-c", "true"], &digest, IMAGE);
    let action = AuthorityAction::from(&operation);
    let grant = grant_for(&fx, &digest, IMAGE, &action);
    let host_request = request_for(Operation::ExecuteCommand {
        argv: vec!["/bin/sh".into(), "-c".into(), "true".into()],
        cwd: ".".into(),
    });
    let error = sea_forge_runtime::execute_openshell(
        grant,
        &host_request,
        RUN_ID,
        &fx.workspace,
        &fx.artifacts,
        binding(&fx, Some(IMAGE)),
        &fx.result_dir,
    )
    .expect_err("a host command must not run through OpenShell");
    assert_eq!(error.class(), "sandbox_command_required", "(c)");
    assert!(
        !calls(&fx)
            .iter()
            .any(|c| c.starts_with("openshell sandbox create")),
        "(c) must create nothing"
    );

    // (d) Everything matches: the sandbox is created, executed, downloaded and
    // destroyed, and the exec runs the guest argv.
    let operation = sandbox_op(&["/bin/sh", "-c", "true"], &digest, IMAGE);
    let action = AuthorityAction::from(&operation);
    let grant = grant_for(&fx, &digest, IMAGE, &action);
    let result = sea_forge_runtime::execute_openshell(
        grant,
        &request_for(operation),
        RUN_ID,
        &fx.workspace,
        &fx.artifacts,
        binding(&fx, Some(IMAGE)),
        &fx.result_dir,
    )
    .expect("the matching sandbox command must execute");
    assert_eq!(result.status, ExecutionStatus::Completed);

    let lines = calls(&fx);
    let position = |prefix: &str| {
        lines
            .iter()
            .position(|line| line.starts_with(prefix))
            .unwrap_or_else(|| panic!("no `{prefix}` call in {lines:#?}"))
    };
    let create = position("openshell sandbox create");
    let exec = position("openshell sandbox exec");
    let download = position("openshell sandbox download");
    let delete = position("openshell sandbox delete");
    assert!(create < exec && exec < download && download < delete);
    assert!(lines[create].contains(&format!("--from {IMAGE}")));
    assert!(
        lines[exec].ends_with("-- /bin/sh -c true"),
        "exec must end with the guest argv: {}",
        lines[exec]
    );
    assert!(fx.result_dir.join("result.txt").exists());
}
