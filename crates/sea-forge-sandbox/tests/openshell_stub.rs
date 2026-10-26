#![cfg(unix)]

//! Deterministic tests of the OpenShell adapter against stub executables.
//!
//! The `openshell` CLI and `openshell-prover` are replaced by POSIX `sh`
//! scripts written into a temporary directory. Each test injects failures by
//! creating marker files in `ctl/`.
//!
//! These tests run against stub executables. They establish the adapter's
//! control flow and failure handling, not OpenShell's isolation.

use sea_forge_core::types::{ExecutionRequest, ExecutionStatus, Operation};
use sea_forge_sandbox::{
    select_openshell, select_sandbox, ExecutionSandbox, NetworkPosture, OpenShellBinding,
    OpenShellSandbox, SandboxClass, SandboxError, SandboxHandle, SandboxSpec,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

const POLICY: &str = "version: 1\n";
const SANDBOX_JSON: &str = r#"{"schema":"domainforge.openshell.sandbox/v1","compute_driver":"vm","workspaces":[{"name":"workspace","sandbox_path":"/sandbox/workspace"}]}"#;
const MAPPING_REPORT: &str = r#"{"schema":"domainforge.openshell.mapping/v1","unmapped":[]}"#;
/// The sandbox image the authorized binding names. Every test binds it.
const IMAGE: &str = "registry.example/cell:1";

const OPENSHELL_STUB: &str = r##"#!/bin/sh
LOG="__LOG__"
CTL="__CTL__"
echo "openshell $*" >> "$LOG"
if [ "$1" = "--gateway" ]; then
  shift 2
fi
case "$1" in
  --version)
    if [ -f "$CTL/version" ]; then cat "$CTL/version"; else echo "openshell 0.1.3"; fi
    exit 0
    ;;
  status)
    if [ -f "$CTL/gateway_down" ]; then
      echo "connection refused" >&2
      exit 1
    fi
    exit 0
    ;;
  logs)
    if [ -f "$CTL/logs" ]; then cat "$CTL/logs"; fi
    exit 0
    ;;
  policy)
    echo '{"hash":"h1","version":3,"config_revision":"r7","policy_source":"sandbox"}'
    exit 0
    ;;
  sandbox)
    case "$2" in
      create)
        if [ -f "$CTL/create_fail" ]; then
          echo "create failed" >&2
          exit 1
        fi
        exit 0
        ;;
      get)
        case "$*" in
          *--policy-only*)
            if [ -f "$CTL/effective_policy" ]; then
              cat "$CTL/effective_policy"
            else
              echo "version: 1"
            fi
            exit 0
            ;;
        esac
        if [ -f "$CTL/deleted" ]; then exit 1; fi
        exit 0
        ;;
      delete)
        if [ -f "$CTL/delete_fail" ]; then
          echo "not found" >&2
          exit 1
        fi
        : > "$CTL/deleted"
        exit 0
        ;;
      upload)
        printf '%s\n' "$4" > "$CTL/upload_src"
        pwd -P > "$CTL/upload_cwd"
        exit 0
        ;;
      download)
        mkdir -p "$5" && echo done > "$5/result.txt"
        exit $?
        ;;
      exec)
        echo "stub-stdout"
        echo "stub-stderr" >&2
        if [ -f "$CTL/exec_exit" ]; then exit "$(cat "$CTL/exec_exit")"; fi
        exit 0
        ;;
    esac
    ;;
esac
echo "stub openshell: unhandled: $*" >&2
exit 2
"##;

const PROVER_STUB: &str = r##"#!/bin/sh
LOG="__LOG__"
CTL="__CTL__"
echo "openshell-prover $*" >> "$LOG"
if [ -f "$CTL/prover_effective_json" ]; then
  case "$2" in
    *effective-policy.yaml)
      cat "$CTL/prover_effective_json"
      exit 0
      ;;
  esac
fi
if [ -f "$CTL/prover_json" ]; then
  cat "$CTL/prover_json"
  exit 0
fi
echo '{"schema_version":1,"result":"within_boundary","exit_code":0,"coverage":{"domains":["filesystem","network_l4","network_rest","process","landlock"]},"counterexample":null,"reason":null}'
exit 0
"##;

struct Harness {
    _dir: tempfile::TempDir,
    root: PathBuf,
    binding: OpenShellBinding,
    spec: SandboxSpec,
    log: PathBuf,
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

fn write_script(path: &Path, body: &str, log: &Path, ctl_dir: &Path) {
    let text = body
        .replace("__LOG__", &log.display().to_string())
        .replace("__CTL__", &ctl_dir.display().to_string());
    fs::write(path, text).expect("write stub script");
    let mut permissions = fs::metadata(path).expect("stat stub script").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("make stub script executable");
}

fn harness() -> Harness {
    let dir = tempfile::tempdir().expect("create temp dir");
    let root = dir.path().to_path_buf();
    let projection = root.join("projection");
    let workspace = root.join("workspace");
    let ctl_dir = root.join("ctl");
    let bin = root.join("bin");
    for path in [&projection, &workspace, &ctl_dir, &bin] {
        fs::create_dir_all(path).expect("create directory");
    }

    fs::write(projection.join("policy.yaml"), POLICY).expect("write policy.yaml");
    fs::write(projection.join("sandbox.json"), SANDBOX_JSON).expect("write sandbox.json");
    fs::write(projection.join("mapping-report.json"), MAPPING_REPORT)
        .expect("write mapping-report.json");
    let lock = json!({
        "cell_id": "cell_test",
        "openshell_version": "0.1.3",
        "model": { "hash": "sha256:model" },
        "policy_digest": digest(POLICY.as_bytes()),
        "files": {
            "mapping-report.json": digest(MAPPING_REPORT.as_bytes()),
            "policy.yaml": digest(POLICY.as_bytes()),
            "sandbox.json": digest(SANDBOX_JSON.as_bytes()),
        },
    });
    fs::write(projection.join("openshell.lock"), lock.to_string()).expect("write openshell.lock");
    fs::write(workspace.join("hello.txt"), "hello\n").expect("write hello.txt");

    let log = root.join("calls.log");
    write_script(&bin.join("openshell"), OPENSHELL_STUB, &log, &ctl_dir);
    write_script(&bin.join("openshell-prover"), PROVER_STUB, &log, &ctl_dir);

    let binding = OpenShellBinding {
        projection_dir: projection,
        gateway: None,
        image: Some(IMAGE.into()),
        cli: bin.join("openshell"),
        prover: bin.join("openshell-prover"),
        local_microvm_gateway: false,
        native_executable: None,
    };
    let spec = SandboxSpec {
        workspace_root: workspace,
        artifacts_root: root.join("artifacts"),
        network: NetworkPosture::Denied,
    };
    Harness {
        _dir: dir,
        root,
        binding,
        spec,
        log,
    }
}

fn calls(h: &Harness) -> Vec<String> {
    match fs::read_to_string(&h.log) {
        Ok(text) => text.lines().map(str::to_string).collect(),
        Err(_) => Vec::new(),
    }
}

fn ctl(h: &Harness, name: &str, content: &str) {
    fs::write(h.root.join("ctl").join(name), content).expect("write control marker");
}

fn request(argv: &[&str], timeout: u64, env: &[(&str, &str)]) -> ExecutionRequest {
    let env: BTreeMap<String, String> = env
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    ExecutionRequest {
        plan_item_id: "item-1".into(),
        operation: Operation::ExecuteSandboxCommand {
            argv: argv.iter().map(|s| s.to_string()).collect(),
            cwd: ".".into(),
            cell_id: "cell_test".into(),
            model_hash: "sha256:model".into(),
            policy_digest: digest(POLICY.as_bytes()),
            image: IMAGE.into(),
        },
        timeout_secs: timeout,
        env,
        compensating_controls: vec![],
    }
}

fn error_class<T>(result: Result<T, SandboxError>) -> &'static str {
    match result {
        Ok(_) => panic!("expected an error"),
        Err(error) => error.class,
    }
}

fn has_prefix(lines: &[String], prefix: &str) -> bool {
    lines.iter().any(|line| line.starts_with(prefix))
}

fn find_after(lines: &[String], from: usize, what: &str, pred: impl Fn(&str) -> bool) -> usize {
    lines
        .iter()
        .enumerate()
        .skip(from)
        .find(|(_, line)| pred(line.as_str()))
        .map(|(index, _)| index)
        .unwrap_or_else(|| panic!("no {what} after line {from}; calls: {lines:#?}"))
}

fn read_record(h: &Harness) -> Value {
    let text = fs::read_to_string(h.root.join("artifacts/openshell/execution.json"))
        .expect("read execution.json");
    serde_json::from_str(&text).expect("parse execution.json")
}

#[test]
fn prepare_execute_download_destroy_happy_path() {
    let h = harness();
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    let handle = sandbox.prepare(&h.spec).expect("prepare");

    let lines = calls(&h);
    let prove_policy = find_after(&lines, 0, "prover check of policy.yaml", |l| {
        l.starts_with("openshell-prover check ") && l.contains("/projection/policy.yaml")
    });
    let create = find_after(&lines, prove_policy + 1, "sandbox create", |l| {
        l.starts_with("openshell sandbox create ")
            && l.contains(" --policy ")
            && l.contains(" --detach")
            && l.contains(" --no-auto-providers")
    });
    assert!(
        lines[create].contains("--from registry.example/cell:1"),
        "the create call must name the explicit image: {}",
        lines[create]
    );
    let get = find_after(&lines, create + 1, "sandbox get --policy-only", |l| {
        l.starts_with("openshell sandbox get ") && l.contains("--policy-only")
    });
    let prove_effective = find_after(&lines, get + 1, "prover check of effective policy", |l| {
        l.starts_with("openshell-prover check ") && l.contains("effective-policy.yaml")
    });
    let policy_get = find_after(&lines, prove_effective + 1, "policy get", |l| {
        l.starts_with("openshell policy get ")
    });
    find_after(&lines, policy_get + 1, "sandbox upload", |l| {
        l.starts_with("openshell sandbox upload ") && l.contains("--no-git-ignore")
    });
    // Contents land flat in the sandbox workspace: the CLI runs inside the
    // workspace and uploads `.` (a named directory would nest as
    // `<dest>/<dirname>/...`).
    let ctl_dir = h.root.join("ctl");
    assert_eq!(
        fs::read_to_string(ctl_dir.join("upload_src")).expect("upload_src"),
        ".\n"
    );
    assert_eq!(
        fs::read_to_string(ctl_dir.join("upload_cwd"))
            .expect("upload_cwd")
            .trim(),
        h.spec
            .workspace_root
            .canonicalize()
            .expect("workspace")
            .display()
            .to_string()
    );

    let record = read_record(&h);
    assert_eq!(record["cell_id"], "cell_test");
    assert_eq!(record["model_hash"], "sha256:model");
    assert_eq!(record["effective_policy"]["hash"], "h1");
    assert_eq!(record["effective_policy"]["version"], 3);
    assert_eq!(record["effective_policy"]["config_revision"], "r7");
    assert_eq!(record["prover"]["effective"]["result"], "within_boundary");
    assert_eq!(record["enforcement"]["driver"], "vm");
    assert_eq!(record["enforcement"]["provides"]["landlock_hard"], true);
    assert_eq!(record["enforcement"]["requires"]["egress_proxy"], false);

    let result = sandbox
        .execute(&handle, &request(&["echo", "hi"], 30, &[("FOO", "bar")]))
        .expect("execute");
    assert_eq!(result.status, ExecutionStatus::Completed);
    assert_eq!(result.exit_code, Some(0));
    let stdout = fs::read_to_string(h.root.join("artifacts/stdout.txt")).expect("read stdout");
    assert!(stdout.contains("stub-stdout"));
    let stderr = fs::read_to_string(h.root.join("artifacts/stderr.txt")).expect("read stderr");
    assert!(stderr.contains("stub-stderr"));

    let lines = calls(&h);
    let exec = lines
        .iter()
        .find(|l| l.starts_with("openshell sandbox exec "))
        .expect("exec call recorded");
    assert!(exec.contains("--timeout 30"));
    assert!(exec.contains("--workdir /sandbox/workspace"));
    assert!(exec.contains("--env FOO=bar"));
    assert!(exec.ends_with("-- echo hi"));

    sandbox
        .download_workspace(&handle, &h.root.join("out"))
        .expect("download workspace");
    assert!(h.root.join("out/result.txt").exists());

    sandbox.destroy(handle).expect("destroy");
    assert!(h.root.join("ctl/deleted").exists());
}

#[test]
fn unavailable_gateway_is_an_explicit_failure() {
    let h = harness();
    ctl(&h, "gateway_down", "");
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_gateway_unavailable"
    );
    assert!(!has_prefix(&calls(&h), "openshell sandbox create "));
}

#[test]
fn missing_cli_is_an_explicit_failure() {
    let h = harness();
    let mut binding = h.binding.clone();
    binding.cli = h.root.join("bin/does-not-exist");
    let sandbox = OpenShellSandbox::new(binding);
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_unavailable"
    );
}

#[test]
fn version_mismatch_is_refused() {
    let h = harness();
    ctl(&h, "version", "openshell 0.1.4-pre.3");
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_version_mismatch"
    );
    assert!(!has_prefix(&calls(&h), "openshell sandbox create "));
}

#[test]
fn tampered_policy_is_refused_before_anything_runs() {
    let h = harness();
    fs::write(h.binding.projection_dir.join("policy.yaml"), "version: 2\n")
        .expect("tamper with policy.yaml");
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_projection_tampered"
    );
    assert!(calls(&h).is_empty());
}

#[test]
fn policy_outside_prover_coverage_is_refused() {
    let h = harness();
    ctl(
        &h,
        "prover_json",
        r#"{"schema_version":1,"result":"unsupported","exit_code":2,"coverage":{"domains":[]},"counterexample":null,"reason":"uses protocol 'graphql'"}"#,
    );
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_policy_unverified"
    );
    assert!(!has_prefix(&calls(&h), "openshell sandbox create "));
}

#[test]
fn incomplete_prover_coverage_is_refused() {
    let h = harness();
    ctl(
        &h,
        "prover_json",
        r#"{"schema_version":1,"result":"within_boundary","exit_code":0,"coverage":{"domains":["filesystem"]},"counterexample":null,"reason":null}"#,
    );
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_policy_unverified"
    );
}

#[test]
fn provider_expanded_effective_policy_is_refused_and_cleaned_up() {
    let h = harness();
    ctl(
        &h,
        "prover_effective_json",
        r#"{"schema_version":1,"result":"exceeds_boundary","exit_code":1,"coverage":{"domains":["filesystem","network_l4","network_rest","process","landlock"]},"counterexample":{"host":"evil.example.com","port":443},"reason":"provider rule widens egress"}"#,
    );
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_effective_policy_exceeds_boundary"
    );

    let lines = calls(&h);
    let create = find_after(&lines, 0, "sandbox create", |l| {
        l.starts_with("openshell sandbox create ")
    });
    find_after(&lines, create + 1, "sandbox delete after create", |l| {
        l.starts_with("openshell sandbox delete ")
    });
    assert!(!has_prefix(&lines, "openshell sandbox upload "));
    assert!(!has_prefix(&lines, "openshell sandbox exec "));
}

#[test]
fn create_failure_is_reported_and_cleaned_up() {
    let h = harness();
    ctl(&h, "create_fail", "");
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_create_failed"
    );
    assert!(has_prefix(&calls(&h), "openshell sandbox delete "));
}

#[test]
fn timeout_exit_code_maps_to_timed_out() {
    let h = harness();
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    let handle = sandbox.prepare(&h.spec).expect("prepare");
    ctl(&h, "exec_exit", "124");
    let result = sandbox
        .execute(&handle, &request(&["sleep", "100"], 5, &[]))
        .expect("execute");
    assert_eq!(result.status, ExecutionStatus::TimedOut);
    assert_eq!(result.exit_code, None);
}

#[test]
fn denied_access_with_failure_is_a_sandbox_violation() {
    let h = harness();
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    let handle = sandbox.prepare(&h.spec).expect("prepare");
    ctl(&h, "exec_exit", "7");
    ctl(
        &h,
        "logs",
        "[1775014132.690] [sandbox] [OCSF ] [ocsf] NET:OPEN [MED] DENIED /usr/bin/curl(64) -> evil.example.com:443 [policy:- engine:opa] [reason:endpoint not allowed]\n",
    );
    let result = sandbox
        .execute(
            &handle,
            &request(&["curl", "https://evil.example.com"], 30, &[]),
        )
        .expect("execute");
    assert_eq!(result.status, ExecutionStatus::SandboxViolation);
    assert_eq!(result.exit_code, Some(7));
    let denials =
        fs::read_to_string(h.root.join("artifacts/openshell/denials.log")).expect("read denials");
    assert!(denials.contains("evil.example.com"));
}

#[test]
fn nonzero_exit_without_denials_is_completed() {
    let h = harness();
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    let handle = sandbox.prepare(&h.spec).expect("prepare");
    ctl(&h, "exec_exit", "3");
    let result = sandbox
        .execute(&handle, &request(&["false"], 30, &[]))
        .expect("execute");
    assert_eq!(result.status, ExecutionStatus::Completed);
    assert_eq!(result.exit_code, Some(3));
}

#[test]
fn host_environment_is_not_forwarded() {
    let h = harness();
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    let handle = sandbox.prepare(&h.spec).expect("prepare");
    let env = [("A", "1"), ("B", "2")];
    sandbox
        .execute(&handle, &request(&["env"], 30, &env))
        .expect("execute");
    let lines = calls(&h);
    let exec = lines
        .iter()
        .find(|l| l.starts_with("openshell sandbox exec "))
        .expect("exec call recorded");
    assert_eq!(exec.matches("--env").count(), env.len());
    assert!(exec.contains("--env A=1"));
    assert!(exec.contains("--env B=2"));
}

#[test]
fn destroy_is_idempotent() {
    let h = harness();
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    let handle = sandbox.prepare(&h.spec).expect("prepare");
    sandbox.destroy(handle).expect("first destroy");

    ctl(&h, "delete_fail", "");
    let second = SandboxHandle {
        class: SandboxClass::OpenShell,
        spec: h.spec.clone(),
    };
    sandbox
        .destroy(second)
        .expect("second destroy of an already removed sandbox");
}

#[test]
fn projection_with_missing_mapping_report_is_refused() {
    let h = harness();
    fs::remove_file(h.binding.projection_dir.join("mapping-report.json"))
        .expect("remove mapping-report.json");
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    // The lock still names mapping-report.json, so the file is read to check
    // its digest, and the missing read is reported as a projection error.
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_projection_error"
    );
    assert!(calls(&h).is_empty());
}

#[test]
fn projection_with_an_unexpected_extra_file_is_refused() {
    let h = harness();
    fs::write(
        h.binding.projection_dir.join("extra.txt"),
        "not in the projection",
    )
    .expect("write extra file");
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_projection_tampered"
    );
    assert!(calls(&h).is_empty());
}

#[test]
fn tampered_mapping_report_is_refused() {
    let h = harness();
    fs::write(
        h.binding.projection_dir.join("mapping-report.json"),
        r#"{"schema":"domainforge.openshell.mapping/v1","unmapped":["egress"]}"#,
    )
    .expect("tamper with mapping-report.json");
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_projection_tampered"
    );
    assert!(calls(&h).is_empty());
}

#[test]
fn unspecified_image_is_refused() {
    let h = harness();
    let mut binding = h.binding.clone();
    binding.image = None;
    let sandbox = OpenShellSandbox::new(binding);
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_image_unspecified"
    );
    assert!(calls(&h).is_empty());
}

#[test]
fn host_command_operation_is_refused_by_the_openshell_backend() {
    let h = harness();
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    let handle = sandbox.prepare(&h.spec).expect("prepare");
    let host_request = ExecutionRequest {
        plan_item_id: "item-1".into(),
        operation: Operation::ExecuteCommand {
            argv: vec!["echo".into(), "hi".into()],
            cwd: ".".into(),
        },
        timeout_secs: 30,
        env: BTreeMap::new(),
        compensating_controls: vec![],
    };
    assert_eq!(
        error_class(sandbox.execute(&handle, &host_request)),
        "input_error"
    );
    assert!(!has_prefix(&calls(&h), "openshell sandbox exec "));
}

#[test]
fn select_sandbox_never_downgrades_openshell() {
    assert_eq!(
        error_class(select_sandbox(SandboxClass::OpenShell)),
        "openshell_binding_missing"
    );
    let bound = select_openshell(OpenShellBinding::new("unused-projection"));
    assert_eq!(bound.class(), SandboxClass::OpenShell);
    assert_eq!(
        "openshell".parse::<SandboxClass>().ok(),
        Some(SandboxClass::OpenShell)
    );
    assert_eq!(SandboxClass::OpenShell.to_string(), "openshell");
}

#[test]
fn gateway_name_is_passed_through() {
    let h = harness();
    let mut binding = h.binding.clone();
    binding.gateway = Some("godspeed".into());
    let sandbox = OpenShellSandbox::new(binding);
    let handle = sandbox.prepare(&h.spec).expect("prepare");
    sandbox.destroy(handle).expect("destroy");

    let lines = calls(&h);
    let cli_lines: Vec<&String> = lines
        .iter()
        .filter(|l| l.starts_with("openshell ") && !l.starts_with("openshell --version"))
        .collect();
    assert!(!cli_lines.is_empty());
    for line in cli_lines {
        assert!(
            line.starts_with("openshell --gateway godspeed "),
            "gateway not passed through: {line}"
        );
    }
}

const HARD_LANDLOCK_POLICY: &str = "version: 1\nlandlock:\n  compatibility: hard_requirement\n";

/// Rewrites the projection's `sandbox.json` and `policy.yaml` and re-locks
/// them, so the projection still verifies. Used to vary the driver and policy.
fn reproject(h: &Harness, sandbox_json: &str, policy: &str) {
    let dir = &h.binding.projection_dir;
    fs::write(dir.join("policy.yaml"), policy).expect("write policy.yaml");
    fs::write(dir.join("sandbox.json"), sandbox_json).expect("write sandbox.json");
    let mut lock: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("openshell.lock")).expect("lock"))
            .expect("parse lock");
    lock["policy_digest"] = json!(digest(policy.as_bytes()));
    lock["files"]["policy.yaml"] = json!(digest(policy.as_bytes()));
    lock["files"]["sandbox.json"] = json!(digest(sandbox_json.as_bytes()));
    fs::write(dir.join("openshell.lock"), lock.to_string()).expect("write lock");
}

fn sandbox_json_for(driver: &str) -> String {
    SANDBOX_JSON.replace(
        "\"compute_driver\":\"vm\"",
        &format!("\"compute_driver\":\"{driver}\""),
    )
}

/// Writes `bytes` as the native executable and returns the native image that
/// pins it on this host, with the binding pointing at the file.
fn bind_native(h: &mut Harness, bytes: &[u8]) -> String {
    let exe = h.root.join("native-tool");
    fs::write(&exe, bytes).expect("write native executable");
    h.binding.native_executable = Some(exe);
    format!(
        "native:sha256:{}@{}-{}",
        hex::encode(Sha256::digest(bytes)),
        std::env::consts::OS,
        std::env::consts::ARCH
    )
}

#[test]
fn compute_driver_is_read_from_the_lock_and_refused_when_unsupported() {
    let h = harness();
    reproject(&h, &sandbox_json_for("docker"), POLICY);
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_driver_unsupported"
    );
    assert!(calls(&h).is_empty());
}

#[test]
fn unknown_compute_driver_is_a_projection_error() {
    let h = harness();
    reproject(&h, &sandbox_json_for("firecracker"), POLICY);
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_projection_error"
    );
    assert!(calls(&h).is_empty());
}

#[test]
fn mxc_cannot_enforce_a_landlock_hard_requirement_and_nothing_is_created() {
    let h = harness();
    reproject(&h, &sandbox_json_for("mxc"), HARD_LANDLOCK_POLICY);
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    let error = match sandbox.prepare(&h.spec) {
        Ok(_) => panic!("mxc must not enforce a hard Landlock requirement"),
        Err(error) => error,
    };
    assert_eq!(error.class, "openshell_enforcement_unavailable");
    assert!(error.message.contains("Landlock"), "{}", error.message);
    let lines = calls(&h);
    assert!(!has_prefix(&lines, "openshell sandbox create "));
    assert!(!has_prefix(&lines, "openshell-prover "));
}

#[test]
fn a_native_image_on_vm_is_refused_before_anything_runs() {
    let mut h = harness();
    let image = bind_native(&mut h, b"tool");
    let mut binding = h.binding.clone();
    binding.image = Some(image);
    let sandbox = OpenShellSandbox::new(binding);
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_workload_kind_mismatch"
    );
    assert!(calls(&h).is_empty());
}

#[test]
fn an_oci_image_on_mxc_is_refused_before_anything_runs() {
    let h = harness();
    reproject(&h, &sandbox_json_for("mxc"), POLICY);
    let sandbox = OpenShellSandbox::new(h.binding.clone());
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_workload_kind_mismatch"
    );
    assert!(calls(&h).is_empty());
}

#[test]
fn a_native_executable_whose_hash_differs_from_the_pin_is_refused() {
    let mut h = harness();
    let image = bind_native(&mut h, b"tool");
    // The pin names other bytes than the file holds.
    let pinned = image.replace(
        &hex::encode(Sha256::digest(b"tool")),
        &hex::encode(Sha256::digest(b"other")),
    );
    reproject(&h, &sandbox_json_for("mxc"), POLICY);
    let mut binding = h.binding.clone();
    binding.image = Some(pinned);
    let sandbox = OpenShellSandbox::new(binding);
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_executable_identity_mismatch"
    );
    assert!(!has_prefix(&calls(&h), "openshell sandbox create "));
}

#[test]
fn a_native_image_for_another_platform_is_refused() {
    let mut h = harness();
    let image = bind_native(&mut h, b"tool").replace(
        &format!("@{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        "@plan9-riscv64",
    );
    reproject(&h, &sandbox_json_for("mxc"), POLICY);
    let mut binding = h.binding.clone();
    binding.image = Some(image);
    let sandbox = OpenShellSandbox::new(binding);
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_executable_identity_mismatch"
    );
}

#[test]
fn a_matching_native_image_is_created_without_an_image_and_runs_only_its_executable() {
    let mut h = harness();
    let image = bind_native(&mut h, b"tool");
    reproject(&h, &sandbox_json_for("mxc"), POLICY);
    let exe = h
        .binding
        .native_executable
        .clone()
        .expect("native executable");
    let mut binding = h.binding.clone();
    binding.image = Some(image);
    let sandbox = OpenShellSandbox::new(binding);
    let handle = sandbox.prepare(&h.spec).expect("prepare");
    let lines = calls(&h);
    let create = find_after(&lines, 0, "sandbox create", |l| {
        l.starts_with("openshell sandbox create ")
    });
    assert!(!lines[create].contains("--from"), "{}", lines[create]);

    let exe_text = exe.display().to_string();
    let result = sandbox
        .execute(&handle, &request(&[exe_text.as_str(), "x"], 30, &[]))
        .expect("execute the pinned executable");
    assert_eq!(result.status, ExecutionStatus::Completed);
    let refused = sandbox.execute(&handle, &request(&["/bin/sh", "-c", "true"], 30, &[]));
    assert_eq!(
        error_class(refused),
        "openshell_executable_identity_mismatch"
    );
}

#[test]
fn an_unparseable_image_is_refused() {
    let h = harness();
    let mut binding = h.binding.clone();
    binding.image = Some("has a space".into());
    let sandbox = OpenShellSandbox::new(binding);
    assert_eq!(
        error_class(sandbox.prepare(&h.spec)),
        "openshell_image_invalid"
    );
    assert!(calls(&h).is_empty());
}
