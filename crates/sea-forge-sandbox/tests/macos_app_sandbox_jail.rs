//! macOS App Sandbox jail: every escape is checked from outside the child.
//!
//! Each test runs real commands through `JailSandbox` and checks the host-side
//! effect (a canary file, a listener's accept queue, a process table entry),
//! not only the child's self-report. Runs only on macOS.
#![cfg(target_os = "macos")]

use sea_forge_core::types::{ExecutionRequest, ExecutionResult, ExecutionStatus, Operation};
use sea_forge_sandbox::{ExecutionSandbox, JailSandbox, NetworkPosture, SandboxSpec};
use std::{
    collections::BTreeMap,
    fs,
    io::ErrorKind,
    net::TcpListener,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, Instant},
};

const PATH_ENV: &str = "/usr/bin:/bin:/usr/sbin:/sbin";

/// Temporary tree: `workspace/` (granted), `artifacts/` (granted), and
/// `outside/` plus `home/` (not granted), all under one temp root.
struct Fixture {
    _root: tempfile::TempDir,
    workspace: PathBuf,
    artifacts: PathBuf,
    outside: PathBuf,
    home: PathBuf,
}

fn fixture() -> Fixture {
    let root = tempfile::tempdir().expect("temp root");
    let base = root.path().to_path_buf();
    let fx = Fixture {
        workspace: base.join("workspace"),
        artifacts: base.join("artifacts"),
        outside: base.join("outside"),
        home: base.join("home"),
        _root: root,
    };
    for dir in [&fx.workspace, &fx.artifacts, &fx.outside, &fx.home] {
        fs::create_dir_all(dir).expect("create fixture dir");
    }
    fx
}

fn request(argv: Vec<String>, timeout_secs: u64) -> ExecutionRequest {
    ExecutionRequest {
        plan_item_id: "macos-app-sandbox-jail".into(),
        operation: Operation::ExecuteCommand {
            argv,
            cwd: ".".into(),
        },
        timeout_secs,
        env: BTreeMap::from([("PATH".to_string(), PATH_ENV.to_string())]),
        compensating_controls: vec![],
    }
}

fn sh(script: &str) -> Vec<String> {
    vec!["/bin/sh".into(), "-c".into(), script.into()]
}

fn run_with(
    fx: &Fixture,
    network: NetworkPosture,
    argv: Vec<String>,
    timeout_secs: u64,
) -> Result<ExecutionResult, sea_forge_sandbox::SandboxError> {
    let jail = JailSandbox::new().expect("macOS App Sandbox jail must be available");
    let spec = SandboxSpec {
        workspace_root: fx.workspace.clone(),
        artifacts_root: fx.artifacts.clone(),
        network,
    };
    let handle = jail.prepare(&spec).expect("prepare");
    jail.execute(&handle, &request(argv, timeout_secs))
}

fn run(fx: &Fixture, script: &str) -> ExecutionResult {
    run_with(fx, NetworkPosture::Denied, sh(script), 20).expect("execute")
}

fn stdout_of(fx: &Fixture) -> String {
    fs::read_to_string(fx.artifacts.join("stdout.txt")).unwrap_or_default()
}

#[test]
fn write_inside_workspace_succeeds() {
    let fx = fixture();
    let target = fx.workspace.join("inside.txt");
    let result = run(&fx, &format!("echo sf-inside > '{}'", target.display()));
    assert_eq!(result.status, ExecutionStatus::Completed);
    assert_eq!(result.exit_code, Some(0));
    assert_eq!(fs::read_to_string(&target).unwrap().trim(), "sf-inside");
}

#[test]
fn write_outside_workspace_is_denied() {
    let fx = fixture();
    let canary = fx.outside.join("escape.txt");
    let result = run(
        &fx,
        &format!("echo ATTEMPT; echo escape > '{}'", canary.display()),
    );
    assert!(stdout_of(&fx).contains("ATTEMPT"), "attempt did not run");
    assert_ne!(result.exit_code, Some(0), "outside write must fail");
    assert!(!canary.exists(), "outside write left a file behind");
}

#[test]
fn symlink_through_workspace_is_denied() {
    let fx = fixture();
    let link = fx.workspace.join("link");
    symlink(&fx.outside, &link).expect("create symlink");
    let canary = fx.outside.join("via-link.txt");
    let result = run(
        &fx,
        &format!(
            "echo ATTEMPT; echo escape > '{}/via-link.txt'",
            link.display()
        ),
    );
    assert!(stdout_of(&fx).contains("ATTEMPT"), "attempt did not run");
    assert_ne!(result.exit_code, Some(0), "symlink-through write must fail");
    assert!(!canary.exists(), "symlink-through write reached the target");
}

#[test]
fn secret_read_outside_workspace_is_denied() {
    let fx = fixture();
    let secret = fx.home.join(".sf-secret");
    let secret_value = "sf-secret-value-7f3a";
    fs::write(&secret, format!("{secret_value}\n")).unwrap();
    let leak = fx.workspace.join("leak.txt");
    let result = run(
        &fx,
        &format!(
            "echo ATTEMPT; cat '{}' > '{}'",
            secret.display(),
            leak.display()
        ),
    );
    assert!(stdout_of(&fx).contains("ATTEMPT"), "attempt did not run");
    assert_ne!(result.exit_code, Some(0), "secret read must fail");
    let leaked = fs::read_to_string(&leak).unwrap_or_default();
    assert!(
        !leaked.contains(secret_value),
        "secret reached the workspace"
    );
}

#[test]
fn tcp_connect_is_denied_and_control_connects() {
    assert!(
        Path::new("/usr/bin/nc").is_file(),
        "/usr/bin/nc is required for the network probe"
    );
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
    listener.set_nonblocking(true).expect("nonblocking");
    let port = listener.local_addr().expect("addr").port();

    // Control: the same probe without the sandbox reaches the listener.
    let control = Command::new("/usr/bin/nc")
        .args(["-z", "-w", "2", "127.0.0.1", &port.to_string()])
        .status()
        .expect("control nc");
    assert!(control.success(), "control connect must succeed");
    assert!(listener.accept().is_ok(), "control connection not queued");

    let fx = fixture();
    let script = format!("echo ATTEMPT; /usr/bin/nc -z -w 2 127.0.0.1 {port}; echo RC=$?");
    let result = run(&fx, &script);
    let out = stdout_of(&fx);
    assert!(out.contains("ATTEMPT"), "attempt did not run: {out}");
    assert!(!out.contains("RC=0"), "sandboxed connect succeeded: {out}");
    assert_eq!(result.exit_code, Some(0), "shell itself should finish");
    match listener.accept() {
        Err(error) if error.kind() == ErrorKind::WouldBlock => {}
        other => panic!("sandboxed connection reached the listener: {other:?}"),
    }
}

#[test]
fn allow_tcp_ports_is_refused() {
    let fx = fixture();
    let marker = fx.workspace.join("ran.txt");
    let result = run_with(
        &fx,
        NetworkPosture::AllowTcpPorts(vec![8080]),
        sh(&format!("touch '{}'", marker.display())),
        20,
    );
    let error = result.expect_err("per-port network posture must be refused");
    assert_eq!(error.class, "jail_unavailable");
    assert!(error.message.contains("cannot restrict network by port"));
    assert!(!marker.exists(), "refused execution must not run");
}

#[test]
fn timeout_kills_the_process_tree() {
    let fx = fixture();
    let pidfile = fx.workspace.join("grandchild.pid");
    let script = format!("sleep 300 & echo $! > '{}'; sleep 300", pidfile.display());
    let result = run_with(&fx, NetworkPosture::Denied, sh(&script), 3).expect("execute");
    assert_eq!(result.status, ExecutionStatus::TimedOut);
    assert_eq!(result.exit_code, None);

    let pid = fs::read_to_string(&pidfile).expect("grandchild pid was recorded");
    let pid = pid.trim();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let alive = Command::new("/bin/kill")
            .args(["-0", pid])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !alive {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "grandchild {pid} survived the timeout"
        );
        thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn background_child_does_not_outlive_completed_execution() {
    let fx = fixture();
    let result = run(&fx, "(sleep 3; echo late > late.txt) & exit 0");
    assert_eq!(result.status, ExecutionStatus::Completed);

    thread::sleep(Duration::from_secs(5));
    assert!(
        !fx.workspace.join("late.txt").exists(),
        "background child wrote after the execution completed"
    );
}
