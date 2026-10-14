#![cfg(unix)]

//! Governed cell episodes against a live OpenShell MicroVM gateway.
//!
//! Nothing here is stubbed: the `openshell` and `openshell-prover` binaries are
//! the real CLIs, the projection is the real RepairAgent OpenShell projection,
//! and the authoritative repository is a temporary git repository.
//!
//! Every test is ignored by default. `scripts/openshell/live-proof.sh` runs
//! them with `--include-ignored` after starting a gateway. A test that runs
//! without the required environment panics; it never passes silently.
//!
//! Required when running:
//! - `SEA_FORGE_OPENSHELL_LIVE=1`
//! - `SEA_FORGE_OPENSHELL_GATEWAY`: the registered gateway name
//! - `SEA_FORGE_OPENSHELL_PROJECTION`: absolute path to the projection directory
//! - `SEA_FORGE_OPENSHELL_IMAGE`: the bootstrap sandbox image
//! - `SEA_FORGE_OPENSHELL_SECOND_IMAGE`: a non-bootstrap image, needed by
//!   `live_non_bootstrap_image_is_used` only
//!
//! `openshell` and `openshell-prover` must be on PATH.

use sea_forge_case_runner::cell_episode::{run_cell_episode, CellEpisode, CellEpisodeRequest};
use sea_forge_core::types::{
    ExecutionRequest, ExecutionStatus, Operation, SettlementCriteria, SettlementStatus, Verdict,
};
use sea_forge_ledger::LedgerStream;
use sea_forge_sandbox::{select_openshell, NetworkPosture, OpenShellBinding, SandboxSpec};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

const WHEN: &str = "2026-01-02T00:00:00Z";
const ENTITY: &str = "operator_local";
const CASE: &str = "case_cell_live";
const ITEM: &str = "item_repair";
const REFERENCE: &str = "refs/heads/main";
const REPAIR: &str = "printf 'print(\"v2\")\\n' > src/app.py";

/// The live environment, validated. Panics if any required part is missing.
struct Live {
    projection: PathBuf,
    gateway: String,
    image: String,
    cli: PathBuf,
    prover: PathBuf,
}

fn required(name: &str) -> String {
    match std::env::var(name) {
        Ok(value) if !value.is_empty() => value,
        _ => panic!("{name} must be set to run the live OpenShell tests"),
    }
}

fn on_path(tool: &str) -> PathBuf {
    std::env::var_os("PATH")
        .and_then(|paths| {
            std::env::split_paths(&paths)
                .map(|dir| dir.join(tool))
                .find(|candidate| candidate.is_file())
        })
        .unwrap_or_else(|| panic!("{tool} must be on PATH to run the live OpenShell tests"))
}

fn live() -> Live {
    if std::env::var("SEA_FORGE_OPENSHELL_LIVE").as_deref() != Ok("1") {
        panic!("live OpenShell tests need SEA_FORGE_OPENSHELL_LIVE=1 and a running MicroVM gateway; refusing to pass without one");
    }
    Live {
        projection: PathBuf::from(required("SEA_FORGE_OPENSHELL_PROJECTION")),
        gateway: required("SEA_FORGE_OPENSHELL_GATEWAY"),
        image: required("SEA_FORGE_OPENSHELL_IMAGE"),
        cli: on_path("openshell"),
        prover: on_path("openshell-prover"),
    }
}

impl Live {
    fn binding(&self, image: &str) -> OpenShellBinding {
        OpenShellBinding {
            projection_dir: self.projection.clone(),
            gateway: Some(self.gateway.clone()),
            image: Some(image.to_string()),
            cli: self.cli.clone(),
            prover: self.prover.clone(),
            local_microvm_gateway: true,
        }
    }

    /// `openshell sandbox get <name>` succeeds only while the sandbox exists.
    fn sandbox_exists(&self, name: &str) -> bool {
        Command::new(&self.cli)
            .args(["--gateway", &self.gateway, "sandbox", "get", name])
            .output()
            .expect("openshell must run")
            .status
            .success()
    }
}

fn git(dir: &Path, args: &[&str]) -> String {
    // Start from an empty environment: a Git hook exports GIT_DIR and
    // GIT_INDEX_FILE, and an inherited value would aim these fixture commands
    // at the repository running the tests instead of the temporary one.
    let out = Command::new("git")
        .current_dir(dir)
        .env_clear()
        .envs(std::env::var_os("PATH").map(|path| ("PATH", path)))
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z")
        .args(args)
        .output()
        .expect("git must be installed");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn write(dir: &Path, rel: &str, content: impl AsRef<[u8]>) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().expect("parent directory")).expect("create directories");
    fs::write(&path, content).expect("write file");
}

/// A bare authoritative repository at `root/auth.git` with one base commit.
fn authoritative(root: &Path) {
    git(root, &["init", "-q", "-b", "main", "seed"]);
    let seed = root.join("seed");
    write(&seed, "README.md", "hello\n");
    write(&seed, "src/app.py", "print('v1')\n");
    write(&seed, "pyproject.toml", "[project]\nname='x'\n");
    write(&seed, "uv.lock", "lock-v1\n");
    git(&seed, &["add", "-A"]);
    git(&seed, &["commit", "-q", "-m", "base"]);
    git(root, &["clone", "-q", "--bare", "seed", "auth.git"]);
}

fn main_ref(repo: &Path) -> String {
    git(repo, &["rev-parse", "refs/heads/main"])
}

/// A fresh temporary root holding the authoritative repository.
fn new_root() -> TempDir {
    let root = tempfile::tempdir().expect("temp root");
    authoritative(root.path());
    root
}

/// A projection lock field, read from the real projection.
fn lock_value(projection: &Path, pointer: &str) -> String {
    let bytes = fs::read(projection.join("openshell.lock")).expect("openshell.lock");
    let lock: Value = serde_json::from_slice(&bytes).expect("lock json");
    lock.pointer(pointer)
        .and_then(Value::as_str)
        .expect("lock field")
        .to_string()
}

/// A policy that allows the sandbox command under `argv0`, bound to `image`,
/// and that requires a security officer to promote.
fn write_policy(root: &Path, live: &Live, image: &str, argv0: &str) -> PathBuf {
    let repo = fs::canonicalize(root.join("auth.git"))
        .expect("canonical repository")
        .display()
        .to_string();
    let cell = lock_value(&live.projection, "/cell_id");
    let model = lock_value(&live.projection, "/model/hash");
    let digest = lock_value(&live.projection, "/policy_digest");
    let text = format!(
        r#"version: "0.1"
identity_bindings:
  - principal: operator_local
    actor_type: human
    role: operator
  - principal: security_officer
    actor_type: human
    role: R-SO
rules:
  - name: execute-in-openshell
    verdict: allow
    actor_role: operator
    operation_kind: execute_sandbox_command
    argv0: {argv0}
    sandbox_class: openshell
    boundary_constraints:
      cell_id: ["{cell}"]
      model_hash: ["{model}"]
      openshell_policy_digest: ["{digest}"]
      sandbox_image: ["{image}"]
  - name: promote-changeset
    verdict: allow
    actor_role: operator
    operation_kind: promote_changeset
    requires_approval: true
    boundary_constraints:
      repository: ["{repo}"]
      reference: ["{REFERENCE}"]
sod_rules:
  - name: promote_requires_security_officer
    requester_role: operator
    approver_role: R-SO
    operation_kind: promote_changeset
    allow_same_principal: false
"#
    );
    let path = root.join("policy-live.yaml");
    fs::write(&path, text).expect("write policy");
    path
}

/// Runs one governed episode on the live gateway and returns it with its ledger.
fn run_live(
    root: &TempDir,
    live: &Live,
    image: &str,
    argv0: &str,
    run: &str,
    argv: &[&str],
) -> (CellEpisode, LedgerStream) {
    let stream = LedgerStream::open(&root.path().join("ledger"), format!("case-{CASE}"), ENTITY)
        .expect("open ledger");
    let policy = write_policy(root.path(), live, image, argv0);
    let run_dir = root.path().join("runs").join(run);
    let repo = root.path().join("auth.git");
    let episode = run_cell_episode(
        CellEpisodeRequest {
            policy_path: policy.as_path(),
            entity: ENTITY,
            case_id: CASE,
            run_id: run,
            plan_item_id: ITEM,
            run_dir: &run_dir,
            repository: &repo,
            reference: REFERENCE,
            binding: live.binding(image),
            argv: argv.iter().map(|arg| arg.to_string()).collect(),
            env: BTreeMap::new(),
            criteria: SettlementCriteria {
                require_exit_zero: true,
                ..Default::default()
            },
            opa: None,
            committed_at: WHEN,
        },
        &stream,
    )
    .expect("live episode");
    (episode, stream)
}

/// The run's own record of the sandbox it used, from `artifacts/openshell`.
fn execution_record(run_dir: &Path) -> Value {
    let text = fs::read_to_string(run_dir.join("artifacts/openshell/execution.json"))
        .expect("openshell execution.json");
    serde_json::from_str(&text).expect("execution.json is JSON")
}

fn read_or_empty(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

/// Every regular file under `dir`, recursively.
fn files_under(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read directory") {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            files_under(&path, out);
        } else {
            out.push(path);
        }
    }
}

/// The sandbox name the adapter derives for `artifacts_root`: `sf-` and the
/// first 16 hex digits of the SHA-256 of the path.
fn adapter_sandbox_name(artifacts_root: &Path) -> String {
    let out = Command::new("sh")
        .arg("-c")
        .arg("printf '%s' \"$1\" | sha256sum")
        .arg("sh")
        .arg(artifacts_root)
        .output()
        .expect("sh and sha256sum must run");
    let digest = String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .expect("sha256 digest")
        .to_string();
    format!("sf-{}", &digest[..16])
}

fn nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after the epoch")
        .as_nanos()
}

// ── 1. An authorized repair writes a file in a MicroVM and is retrieved ──

#[test]
#[ignore = "requires a live OpenShell MicroVM gateway (SEA_FORGE_OPENSHELL_LIVE=1)"]
fn live_authorized_episode_runs_in_a_microvm() {
    let live = live();
    let root = new_root();
    let repo = root.path().join("auth.git");
    let before = main_ref(&repo);
    let (episode, stream) = run_live(
        &root,
        &live,
        &live.image,
        "sh",
        "run_live_write",
        &["sh", "-c", REPAIR],
    );
    let run_dir = root.path().join("runs").join("run_live_write");

    assert_eq!(episode.decision.verdict, Verdict::Allow);
    let execution = episode.execution.as_ref().expect("execution");
    assert_eq!(execution.status, ExecutionStatus::Completed);
    assert_eq!(execution.exit_code, Some(0));

    let openshell = run_dir.join("artifacts/openshell");
    for name in ["execution.json", "effective-policy.yaml", "denials.log"] {
        assert!(openshell.join(name).exists(), "evidence file {name}");
    }

    let changeset = episode.changeset.as_ref().expect("changeset");
    assert!(
        changeset
            .entries
            .iter()
            .any(|entry| entry.path == "src/app.py"),
        "the changeset holds the written file"
    );

    let request = episode
        .promotion_request
        .as_ref()
        .expect("promotion request");
    assert_eq!(request.decision.verdict, Verdict::Escalate);
    assert_eq!(main_ref(&repo), before, "the ref is unchanged");

    let sandbox = execution_record(&run_dir)["sandbox"]
        .as_str()
        .expect("sandbox name")
        .to_string();
    assert!(!live.sandbox_exists(&sandbox), "sandbox {sandbox} is gone");
    stream.verify().expect("ledger verifies");
}

// ── 2. Egress to a host outside the policy is denied ──

#[test]
#[ignore = "requires a live OpenShell MicroVM gateway (SEA_FORGE_OPENSHELL_LIVE=1)"]
fn live_network_egress_is_denied() {
    let live = live();
    let root = new_root();
    // 203.0.113.0/24 is documentation space; the policy names no such host.
    let connect = "exec 3<>/dev/tcp/203.0.113.10/443 && echo CONNECTED";
    let (episode, stream) = run_live(
        &root,
        &live,
        &live.image,
        "bash",
        "run_live_egress",
        &["bash", "-c", connect],
    );
    let run_dir = root.path().join("runs").join("run_live_egress");

    let execution = episode.execution.as_ref().expect("execution");
    assert_ne!(
        execution.exit_code,
        Some(0),
        "an outbound connection outside the policy must not succeed"
    );
    let stdout = read_or_empty(&run_dir.join("artifacts/stdout.txt"));
    assert!(!stdout.contains("CONNECTED"), "stdout: {stdout}");
    assert_ne!(episode.settlement.status, SettlementStatus::Accepted);
    assert!(episode.promotion_request.is_none());
    stream.verify().expect("ledger verifies");
}

// ── 3. The host filesystem is not visible inside the sandbox ──

#[test]
#[ignore = "requires a live OpenShell MicroVM gateway (SEA_FORGE_OPENSHELL_LIVE=1)"]
fn live_host_filesystem_is_not_visible() {
    let live = live();
    let root = new_root();
    // A marker on the host, outside the workspace and outside the repository.
    let host_dir = tempfile::tempdir().expect("host directory");
    let marker = host_dir.path().join("host-only-marker.txt");
    let secret = format!("host-only-{}", nanos());
    fs::write(&marker, &secret).expect("write marker");

    let read = format!("cat '{}'", marker.display());
    let (episode, stream) = run_live(
        &root,
        &live,
        &live.image,
        "sh",
        "run_live_host_fs",
        &["sh", "-c", &read],
    );
    let run_dir = root.path().join("runs").join("run_live_host_fs");

    let execution = episode.execution.as_ref().expect("execution");
    assert_ne!(
        execution.exit_code,
        Some(0),
        "reading a host file from the sandbox must fail"
    );
    let stdout = read_or_empty(&run_dir.join("artifacts/stdout.txt"));
    assert!(!stdout.contains(&secret), "the marker leaked to stdout");

    let mut files = Vec::new();
    let result = run_dir.join("result");
    if result.exists() {
        files_under(&result, &mut files);
    }
    for file in files {
        let text = read_or_empty(&file);
        assert!(
            !text.contains(&secret),
            "the marker leaked into {}",
            file.display()
        );
    }
    stream.verify().expect("ledger verifies");
}

// ── 4. A command past its timeout is cancelled and its sandbox destroyed ──

#[test]
#[ignore = "requires a live OpenShell MicroVM gateway (SEA_FORGE_OPENSHELL_LIVE=1)"]
fn live_timeout_cancels_the_command() {
    let live = live();
    let root = new_root();
    let workspace = root.path().join("timeout-workspace");
    write(&workspace, "README.md", "hello\n");
    let artifacts = root.path().join("timeout-artifacts");
    let name = adapter_sandbox_name(&artifacts);

    // The episode runs with the projection's own timeout, which is far longer
    // than a test should wait. The adapter takes the timeout per request, so
    // this test drives it directly with the smallest useful value.
    let sandbox = select_openshell(live.binding(&live.image));
    let spec = SandboxSpec {
        workspace_root: workspace,
        artifacts_root: artifacts,
        network: NetworkPosture::Denied,
    };
    let handle = sandbox.prepare(&spec).expect("sandbox prepared");
    assert!(live.sandbox_exists(&name), "sandbox {name} is listed");

    let lock = |pointer: &str| lock_value(&live.projection, pointer);
    let request = ExecutionRequest {
        plan_item_id: ITEM.into(),
        operation: Operation::ExecuteSandboxCommand {
            argv: vec!["sleep".into(), "120".into()],
            cwd: ".".into(),
            cell_id: lock("/cell_id"),
            model_hash: lock("/model/hash"),
            policy_digest: lock("/policy_digest"),
            image: live.image.clone(),
        },
        timeout_secs: 5,
        env: BTreeMap::new(),
        compensating_controls: vec![],
    };
    let result = sandbox
        .execute(&handle, &request)
        .expect("the execution returns a result");
    assert_eq!(result.status, ExecutionStatus::TimedOut);

    sandbox.destroy(handle).expect("sandbox destroyed");
    assert!(!live.sandbox_exists(&name), "sandbox {name} is gone");
}

// ── 5. A non-bootstrap image runs when the policy and binding pin it ──

#[test]
#[ignore = "requires a live OpenShell MicroVM gateway (SEA_FORGE_OPENSHELL_LIVE=1)"]
fn live_non_bootstrap_image_is_used() {
    let live = live();
    let second = std::env::var("SEA_FORGE_OPENSHELL_SECOND_IMAGE")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            panic!("SEA_FORGE_OPENSHELL_SECOND_IMAGE must name a non-bootstrap image for this test")
        });
    assert_ne!(
        second, live.image,
        "the second image must differ from the bootstrap image"
    );
    let root = new_root();
    let (episode, stream) = run_live(
        &root,
        &live,
        &second,
        "sh",
        "run_live_second_image",
        &["sh", "-c", REPAIR],
    );
    let run_dir = root.path().join("runs").join("run_live_second_image");

    assert_eq!(episode.decision.verdict, Verdict::Allow);
    let execution = episode.execution.as_ref().expect("execution");
    assert_eq!(execution.status, ExecutionStatus::Completed);
    assert_eq!(execution.exit_code, Some(0));
    assert_eq!(execution_record(&run_dir)["image"], second.as_str());
    stream.verify().expect("ledger verifies");
}

// ── 6. The workspace tree is retrieved and the sandbox is cleaned up ──

#[test]
#[ignore = "requires a live OpenShell MicroVM gateway (SEA_FORGE_OPENSHELL_LIVE=1)"]
fn live_artifacts_are_retrieved_and_sandbox_cleaned_up() {
    let live = live();
    let root = new_root();
    let (episode, stream) = run_live(
        &root,
        &live,
        &live.image,
        "sh",
        "run_live_retrieve",
        &["sh", "-c", REPAIR],
    );
    let run_dir = root.path().join("runs").join("run_live_retrieve");

    assert_eq!(
        episode.execution.as_ref().expect("execution").status,
        ExecutionStatus::Completed
    );
    let result = run_dir.join("result");
    assert_eq!(
        read_or_empty(&result.join("README.md")),
        "hello\n",
        "the unchanged workspace file came back"
    );
    assert!(
        read_or_empty(&result.join("src/app.py")).contains("v2"),
        "the written file came back"
    );

    let sandbox = execution_record(&run_dir)["sandbox"]
        .as_str()
        .expect("sandbox name")
        .to_string();
    assert!(!live.sandbox_exists(&sandbox), "sandbox {sandbox} is gone");
    stream.verify().expect("ledger verifies");
}
