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
use sea_forge_core::errors::ForgeError;
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
use std::time::{Duration, Instant};
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
            native_executable: None,
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
    authoritative_with_links(root, &[]);
}

/// Like `authoritative`, with symbolic links `(path, target)` in the base
/// tree. The export checks them out as links in the workspace.
fn authoritative_with_links(root: &Path, links: &[(&str, &Path)]) {
    git(root, &["init", "-q", "-b", "main", "seed"]);
    let seed = root.join("seed");
    write(&seed, "README.md", "hello\n");
    write(&seed, "src/app.py", "print('v1')\n");
    write(&seed, "pyproject.toml", "[project]\nname='x'\n");
    write(&seed, "uv.lock", "lock-v1\n");
    for (rel, target) in links {
        let path = seed.join(rel);
        fs::create_dir_all(path.parent().expect("parent directory")).expect("create directories");
        std::os::unix::fs::symlink(target, &path).expect("create symlink");
    }
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

/// A fresh temporary root whose base tree carries the given symbolic links.
fn new_root_with_links(links: &[(&str, &Path)]) -> TempDir {
    let root = tempfile::tempdir().expect("temp root");
    authoritative_with_links(root.path(), links);
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
    try_run_live(root, live, image, argv0, run, argv).expect("live episode")
}

/// Like `run_live`, but returns SEA-Forge's refusal instead of panicking.
fn try_run_live(
    root: &TempDir,
    live: &Live,
    image: &str,
    argv0: &str,
    run: &str,
    argv: &[&str],
) -> Result<(CellEpisode, LedgerStream), ForgeError> {
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
    )?;
    Ok((episode, stream))
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

/// Claim (narrowed): an outbound connection to 203.0.113.10:443 did not
/// succeed, and no `CONNECTED` was observed. No egress observed.
///
/// This does not show that the sandbox's network enforcement refused the
/// connection. OpenShell 0.1.3 logged no `DENIED` record for that destination in
/// live run 40281322154 (`denials.log` was empty), so the cause (policy, missing
/// route or no listener) is not attributed. The denial record is printed below
/// as evidence.
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
    let denials = read_or_empty(&run_dir.join("artifacts/openshell/denials.log"));
    println!(
        "egress evidence: status {:?}, denials.log {denials:?}",
        execution.status
    );
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

/// Regular files and symbolic links under `dir`. A link is reported, never
/// followed: following one would read the host file it points at.
fn walk_without_following(dir: &Path, files: &mut Vec<PathBuf>, links: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read directory") {
        let path = entry.expect("directory entry").path();
        let meta = fs::symlink_metadata(&path).expect("file metadata");
        if meta.file_type().is_symlink() {
            links.push(path);
        } else if meta.is_dir() {
            walk_without_following(&path, files, links);
        } else {
            files.push(path);
        }
    }
}

/// Runs one episode that must complete with exit 0, checks its ledger, and
/// returns its stdout and the result directory it came back to.
fn run_completed(root: &TempDir, live: &Live, run: &str, argv: &[&str]) -> (String, PathBuf) {
    let (episode, stream) = run_live(root, live, &live.image, "sh", run, argv);
    let execution = episode.execution.as_ref().expect("execution");
    assert_eq!(
        execution.status,
        ExecutionStatus::Completed,
        "{run} completes"
    );
    assert_eq!(execution.exit_code, Some(0), "{run} exits 0");
    stream.verify().expect("ledger verifies");
    let run_dir = root.path().join("runs").join(run);
    (
        read_or_empty(&run_dir.join("artifacts/stdout.txt")),
        run_dir.join("result"),
    )
}

// ── 7. A workspace symlink cannot disclose host files ──

/// Refusal classes that name the symlink or transfer layer that stopped it.
const SYMLINK_REFUSALS: &[&str] = &[
    "openshell_transfer_failed",
    "openshell_create_failed",
    "export_failed",
    "malformed_workspace",
];

#[test]
#[ignore = "requires a live OpenShell MicroVM gateway (SEA_FORGE_OPENSHELL_LIVE=1)"]
fn live_workspace_symlink_cannot_reach_host_files() {
    let live = live();
    // Host-only secrets, outside the workspace and the repository. The
    // symlinks are committed in the base tree, so the export places them in
    // the workspace before the episode.
    let host = tempfile::tempdir().expect("host directory");
    let stamp = nanos();
    let file_secret = format!("host-file-secret-{stamp}");
    let dir_secret = format!("host-dir-secret-{stamp}");
    let secret_file = host.path().join("secret.txt");
    fs::write(&secret_file, &file_secret).expect("write host file secret");
    let secret_dir = host.path().join("secret-dir");
    write(&secret_dir, "inner.txt", &dir_secret);
    let root = new_root_with_links(&[
        ("leak-file", secret_file.as_path()),
        ("leak-dir", secret_dir.as_path()),
    ]);

    let run = "run_live_symlink";
    let run_dir = root.path().join("runs").join(run);
    let sandbox = adapter_sandbox_name(&run_dir.join("artifacts"));
    let read = "cat leak-file > out-file.txt 2>&1; cat leak-dir/inner.txt > out-dir.txt 2>&1; ls -la > listing.txt 2>&1; true";
    let secrets = [file_secret.as_str(), dir_secret.as_str()];

    match try_run_live(&root, &live, &live.image, "sh", run, &["sh", "-c", read]) {
        Err(error) => {
            let class = error.class();
            println!("symlink scenario: refused by SEA-Forge, class {class}: {error:?}");
            assert!(
                SYMLINK_REFUSALS.contains(&class),
                "unexpected refusal class {class}: {error:?}"
            );
            assert!(
                !live.sandbox_exists(&sandbox),
                "sandbox {sandbox} remains after a refusal"
            );
            for dir in [run_dir.join("artifacts"), run_dir.join("result")] {
                let (mut files, mut links) = (Vec::new(), Vec::new());
                if dir.exists() {
                    walk_without_following(&dir, &mut files, &mut links);
                }
                for file in files {
                    let text = read_or_empty(&file);
                    for secret in secrets {
                        assert!(!text.contains(secret), "host secret in {}", file.display());
                    }
                }
            }
        }
        Ok((episode, stream)) => {
            let execution = episode.execution.as_ref().expect("execution");
            println!(
                "symlink scenario: accepted; status {:?}, exit {:?}, settlement {:?}",
                execution.status, execution.exit_code, episode.settlement.status
            );
            assert!(!live.sandbox_exists(&sandbox), "sandbox {sandbox} is gone");
            stream.verify().expect("ledger verifies");

            let result = run_dir.join("result");
            let (mut files, mut links) = (Vec::new(), Vec::new());
            for dir in [run_dir.join("artifacts"), result.clone()] {
                if dir.exists() {
                    walk_without_following(&dir, &mut files, &mut links);
                }
            }
            for link in &links {
                let target = fs::read_link(link).expect("read link");
                println!("link came back: {} -> {}", link.display(), target.display());
            }
            for file in &files {
                let text = read_or_empty(file);
                for secret in secrets {
                    assert!(!text.contains(secret), "host secret in {}", file.display());
                }
            }
            // Printed only after the disclosure check passed.
            let out = read_or_empty(&result.join("out-file.txt"));
            println!(
                "out-file.txt after the link was read: {:?}",
                out.chars().take(200).collect::<String>()
            );
        }
    }
}

// ── 8. Two concurrent sandboxes cannot see each other's files ──

/// Claim: a sandbox cannot read a token that a concurrently running sandbox
/// holds. Episode A writes its token and stays alive; episode B starts only
/// once A's sandbox exists and then searches the readable trees. B first plants
/// its own control token in `/tmp`, so a silent search cannot pass: the probe
/// must exit 0 and find that control. A's sandbox must still exist when B has
/// finished, which shows the two really overlapped.
/// The searched trees are the ones the sandbox user can read: `/home` is
/// refused with permission denied in the live VM (run 40282555261), so it is
/// not searched.
#[test]
#[ignore = "requires a live OpenShell MicroVM gateway (SEA_FORGE_OPENSHELL_LIVE=1)"]
fn live_concurrent_sandboxes_are_isolated() {
    let live = live();
    let token = format!("sf-token-a-{}", nanos());
    let control = format!("sf-control-b-{}", nanos());
    // B gets the token reversed, so its own argv never holds the literal.
    let reversed: String = token.chars().rev().collect();
    let writer = format!("printf '%s' '{token}' > a-token.txt; sleep 90; cat a-token.txt");
    let searcher = format!(
        "printf '%s' '{control}' > /tmp/b-control.txt; T=$(printf '%s' '{reversed}' | rev); \
         timeout 60 grep -rIa --exclude=found.txt --exclude=errors.txt -e \"$T\" -e '{control}' /sandbox /tmp /var /opt > found.txt 2> errors.txt; \
         echo \"grep_exit=$?\"; exit 0"
    );
    let root_a = new_root();
    let root_b = new_root();
    let name_a = adapter_sandbox_name(
        &root_a
            .path()
            .join("runs")
            .join("run_live_iso_a")
            .join("artifacts"),
    );

    let ((stdout_a, _), (stdout_b, result_b)) = std::thread::scope(|scope| {
        let a =
            scope.spawn(|| run_completed(&root_a, &live, "run_live_iso_a", &["sh", "-c", &writer]));
        let deadline = Instant::now() + Duration::from_secs(600);
        while !live.sandbox_exists(&name_a) {
            assert!(
                !a.is_finished() && Instant::now() < deadline,
                "episode A never came up"
            );
            std::thread::sleep(Duration::from_secs(2));
        }
        // Settle delay: A's upload and token write happen after creation.
        std::thread::sleep(Duration::from_secs(30));
        assert!(
            !a.is_finished(),
            "episode A finished before B could overlap it"
        );
        let b = scope
            .spawn(|| run_completed(&root_b, &live, "run_live_iso_b", &["sh", "-c", &searcher]));
        let b_result = b.join().expect("episode B");
        assert!(
            live.sandbox_exists(&name_a),
            "A's sandbox was gone before B finished, so the episodes did not overlap"
        );
        (a.join().expect("episode A"), b_result)
    });

    // A really held its token: its own sandbox echoed it back.
    assert!(
        stdout_a.contains(&token),
        "episode A did not hold its token"
    );
    println!("episode B stdout: {stdout_b:?}");
    let grep_exit = stdout_b
        .lines()
        .find_map(|line| line.strip_prefix("grep_exit="))
        .unwrap_or("missing");
    let errors = read_or_empty(&result_b.join("errors.txt"));
    assert_eq!(
        grep_exit, "0",
        "B's search must complete over readable trees: {stdout_b:?}; grep stderr: {errors}"
    );
    assert!(!stdout_b.contains(&token), "B's stdout holds A's token");
    let found = read_or_empty(&result_b.join("found.txt"));
    assert!(
        found.contains(&control),
        "B's positive control in its own /tmp was not found: the search did not run"
    );
    assert!(!found.contains(&token), "B's search found A's token");
    let (mut files, mut links) = (Vec::new(), Vec::new());
    walk_without_following(&result_b, &mut files, &mut links);
    for file in files {
        let text = read_or_empty(&file);
        assert!(
            !text.contains(&token),
            "B's result {} holds A's token",
            file.display()
        );
    }
}

// ── 9. A tampered projection is refused before any sandbox is created ──

#[test]
#[ignore = "requires a live OpenShell MicroVM gateway (SEA_FORGE_OPENSHELL_LIVE=1)"]
fn live_tampered_projection_is_refused_before_create() {
    let live = live();
    let root = new_root();
    // A copy of the real projection with one read-write path added to the
    // policy and openshell.lock left as it was.
    let tampered = tempfile::tempdir().expect("tampered projection directory");
    for entry in fs::read_dir(&live.projection).expect("projection directory") {
        let entry = entry.expect("directory entry");
        if entry.file_type().expect("file type").is_file() {
            fs::copy(entry.path(), tampered.path().join(entry.file_name()))
                .expect("copy projection file");
        }
    }
    let policy_path = tampered.path().join("policy.yaml");
    let policy = fs::read_to_string(&policy_path).expect("policy.yaml");
    let edited = policy.replacen("  - /tmp\n", "  - /tmp\n  - /home\n", 1);
    assert_ne!(edited, policy, "the edit must add a read_write path");
    fs::write(&policy_path, edited).expect("write tampered policy");
    let forged = Live {
        projection: tampered.path().to_path_buf(),
        gateway: live.gateway.clone(),
        image: live.image.clone(),
        cli: live.cli.clone(),
        prover: live.prover.clone(),
    };

    let run = "run_live_tampered";
    let refusal = match try_run_live(
        &root,
        &forged,
        &forged.image,
        "sh",
        run,
        &["sh", "-c", "true"],
    ) {
        Ok(_) => panic!("a tampered projection must not run an episode"),
        Err(error) => error,
    };
    let class = refusal.class();
    println!("tampered projection: refused before create, class {class}: {refusal:?}");
    assert_eq!(class, "openshell_projection_tampered");

    let name = adapter_sandbox_name(&root.path().join("runs").join(run).join("artifacts"));
    let listed = Command::new(&live.cli)
        .args(["--gateway", &live.gateway, "sandbox", "list"])
        .output()
        .expect("openshell must run");
    assert!(
        listed.status.success(),
        "sandbox list failed: {}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let text = String::from_utf8_lossy(&listed.stdout);
    assert!(
        !text.contains(&name),
        "sandbox {name} exists after a refused run:\n{text}"
    );
    assert!(!live.sandbox_exists(&name), "sandbox {name} exists");
}
