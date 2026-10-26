//! macOS App Sandbox jail: every escape is checked from outside the child.
//!
//! Each test runs real commands through `JailSandbox` and checks the host-side
//! effect (a canary file, a listener's accept queue, a process table entry),
//! not only the child's self-report. Runs only on macOS.
#![cfg(target_os = "macos")]

use sea_forge_core::types::{
    ExecutionRequest, ExecutionResult, ExecutionStatus, LifecycleContainment, Operation,
};
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
    // C1: no setsid escapee in the census, so the verified group kill claims
    // ProcessGroup — the strongest mechanism this backend provides.
    assert_eq!(
        result.lifecycle_containment,
        LifecycleContainment::ProcessGroup,
        "a fully terminated macOS jail run must claim the verified group kill"
    );

    thread::sleep(Duration::from_secs(5));
    assert!(
        !fx.workspace.join("late.txt").exists(),
        "background child wrote after the execution completed"
    );
}

// ---------------------------------------------------------------------------
// F08 cage-persists: a setsid descendant escapes the kill sweep but stays
// inside the App Sandbox cage.
// ---------------------------------------------------------------------------

/// Hidden helper: run inside the jail as the setsid'd descendant, driven by
/// env (mirrors the `net_probe_helper` pattern in conformance_m1.rs). The
/// parent test copies this binary into the granted workspace first, so the
/// jailed workload can read and exec it — reads outside the granted trees
/// are denied.
///
/// `SEA_FORGE_CAGE_WS`      absolute workspace path (sandbox-granted)
/// `SEA_FORGE_CAGE_OUTSIDE` absolute directory outside the granted trees
///
/// Sequence: mark the earliest reachable step (`helper_started.txt`), then
/// `setsid(2)` out of the workload's process group (the F08 escape), record
/// the pid, attempt the outside write and record its outcome in the
/// workspace, write `late.txt` into the workspace after a delay, then stay
/// alive for the parent's survivor assertion. The parent test owns killing
/// us. The markers discriminate a helper that never ran at all (exec or
/// test-harness failure: no `helper_started.txt`) from one that died at
/// `setsid` (`helper-failed.txt`) from one that recorded (`detached.pid`).
#[test]
fn cage_persists_helper() {
    let (Ok(workspace), Ok(outside)) = (
        std::env::var("SEA_FORGE_CAGE_WS"),
        std::env::var("SEA_FORGE_CAGE_OUTSIDE"),
    ) else {
        return; // not running as the helper
    };

    let _ = fs::write(format!("{workspace}/helper_started.txt"), "1");

    // setsid(2): a new session and process group that the workload's group
    // kill cannot reach. Fails (EPERM) only if this process were already a
    // group leader, which the background-spawn shape prevents.
    #[allow(unsafe_code)]
    {
        // SAFETY: setsid(2) has no pointer arguments; failure (EPERM) is
        // reported through the return value.
        if unsafe { libc::setsid() } < 0 {
            let _ = fs::write(
                format!("{workspace}/helper-failed.txt"),
                format!("setsid: {}", std::io::Error::last_os_error()),
            );
            return;
        }
    }
    fs::write(
        format!("{workspace}/detached.pid"),
        std::process::id().to_string(),
    )
    .expect("record the descendant pid");

    // The escape attempt: write outside the sandbox-allowed trees. The App
    // Sandbox must deny this even though the group kill already missed us.
    match fs::write(format!("{outside}/outside-marker.txt"), "escape") {
        Ok(()) => fs::write(format!("{workspace}/outside_denied.txt"), "0"),
        Err(_) => fs::write(format!("{workspace}/outside_denied.txt"), "1"),
    }
    .expect("record the outside-write outcome");

    // The late write INSIDE the authorized workspace, after the run has
    // settled (the workload exits as soon as the pid marker exists).
    thread::sleep(Duration::from_secs(3));
    fs::write(format!("{workspace}/late.txt"), "late").expect("late inside write");

    // Hold the pid alive for the parent's survivor assertion.
    thread::sleep(Duration::from_secs(30));
}

/// F08 cage-persists regression: a workload whose `setsid`ed descendant
/// outlives cancellation keeps writing INSIDE the authorized workspace, but
/// its write OUTSIDE the sandbox-allowed trees stays denied. Asserts the
/// documented F08 macOS boundary from outside the child (host-side pid
/// liveness, host-side file existence), following this file's convention of
/// never trusting the child's self-report.
///
/// Asserted invariants:
/// 1. **survivor-alive** — the descendant SURVIVES the cancellation. This
///    is the documented limitation (POSIX process groups cannot reach a new
///    session), asserted so a future change cannot silently claim lifecycle
///    containment this platform does not provide; a real fix (kernel-level
///    containment) must update this assertion and the F08 documentation
///    together, visibly.
/// 2. **outside-denied** (load-bearing) — the descendant's write outside
///    the granted trees is DENIED: escaping the kill sweep never becomes
///    escaping the cage, because the App Sandbox is inherited by every
///    descendant regardless of session.
/// 3. **inside-expected** — the descendant's late write into the authorized
///    workspace after the run completed is EXPECTED: precisely the F08
///    lifecycle residual accepted for this backend (the cage holds;
///    lifecycle does not).
#[test]
fn setsid_descendant_survives_the_sweep_but_stays_sandbox_caged() {
    let fx = fixture();
    let helper = fx.workspace.join("cage-helper");
    fs::copy(
        std::env::current_exe().expect("current test executable"),
        &helper,
    )
    .expect("copy the helper into the granted workspace");

    let mut env = BTreeMap::from([("PATH".to_string(), PATH_ENV.to_string())]);
    env.insert(
        "SEA_FORGE_CAGE_WS".to_string(),
        fx.workspace.display().to_string(),
    );
    env.insert(
        "SEA_FORGE_CAGE_OUTSIDE".to_string(),
        fx.outside.display().to_string(),
    );
    let request = ExecutionRequest {
        plan_item_id: "macos-app-sandbox-jail-cage-persists".into(),
        operation: Operation::ExecuteCommand {
            argv: sh(&format!(
                "'{helper}' --exact cage_persists_helper & \
                 i=0; while [ ! -f '{ws}/detached.pid' ] && [ $i -lt 200 ]; do \
                 i=$((i+1)); sleep 0.05; done; \
                 [ -f '{ws}/detached.pid' ] || {{ echo helper-never-recorded; exit 9; }}",
                helper = helper.display(),
                ws = fx.workspace.display(),
            )),
            cwd: ".".into(),
        },
        // The workload's own fallback budget is 200 × 50ms = 10s, strictly
        // inside this execute timeout: a helper that never records makes the
        // workload exit 9 with "helper-never-recorded" ~10s BEFORE the
        // deadline, so the failure surfaces as Completed-with-exit-9 (the
        // assertion below prints the artifacts) instead of a bare TimedOut
        // whose evidence is masked by the timeout boundary.
        timeout_secs: 20,
        env,
        compensating_controls: vec![],
    };

    let jail = JailSandbox::new().expect("macOS App Sandbox jail must be available");
    let spec = SandboxSpec {
        workspace_root: fx.workspace.clone(),
        artifacts_root: fx.artifacts.clone(),
        network: NetworkPosture::Denied,
    };
    let handle = jail.prepare(&spec).expect("prepare");
    let result = jail.execute(&handle, &request).expect("execute");
    assert_eq!(
        result.status,
        ExecutionStatus::Completed,
        "the workload must complete before the execute timeout; helper markers: \
         started={} failed={} pid_recorded={}; stdout: {}; stderr: {}",
        fx.workspace.join("helper_started.txt").exists(),
        fx.workspace.join("helper-failed.txt").exists(),
        fx.workspace.join("detached.pid").exists(),
        stdout_of(&fx),
        fs::read_to_string(fx.artifacts.join("stderr.txt")).unwrap_or_default(),
    );
    assert_eq!(
        result.exit_code,
        Some(0),
        "the workload itself must succeed; stderr: {}",
        fs::read_to_string(fx.artifacts.join("stderr.txt")).unwrap_or_default()
    );
    // C1: the result itself must carry the documented residual — the setsid
    // survivor is claimed as SandboxCagePersists (at least the one recorded
    // below), never as a full lifecycle containment.
    assert!(
        matches!(
            result.lifecycle_containment,
            LifecycleContainment::SandboxCagePersists { survivor_count } if survivor_count >= 1
        ),
        "the cage-persists run must claim SandboxCagePersists with at least one \
         survivor, got {:?}",
        result.lifecycle_containment
    );

    // INVARIANT 1 (documented limitation, asserted): the descendant survives
    // the cancellation. Liveness is checked from the host by pid — the
    // marker is the recorded identity, exactly what the runtime's census
    // also observed before the group kill.
    let survivor: i32 = fs::read_to_string(fx.workspace.join("detached.pid"))
        .expect("the descendant recorded its pid")
        .trim()
        .parse()
        .expect("pid");
    let alive = Command::new("/bin/kill")
        .args(["-0", &survivor.to_string()])
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    assert!(
        alive,
        "the setsid descendant {survivor} did not survive the sweep — if teardown now \
         reaches setsid descendants, update this test and the F08 macOS documentation \
         together"
    );

    // INVARIANT 2 (load-bearing): the outside write is DENIED — the cage
    // persists. The descendant itself reports the denial (it cannot write
    // outside), and the host confirms no file landed.
    let outcome = fs::read_to_string(fx.workspace.join("outside_denied.txt"))
        .expect("the descendant recorded its outside-write outcome");
    assert_eq!(
        outcome.trim(),
        "1",
        "the descendant's write outside the granted trees must be denied"
    );
    assert!(
        !fx.outside.join("outside-marker.txt").exists(),
        "the setsid descendant escaped the cage: a file exists outside the granted trees"
    );

    // INVARIANT 3 (expected residual): the late write lands inside the
    // authorized workspace after the run completed. This is precisely the
    // F08 lifecycle residual accepted for the macOS backend.
    let deadline = Instant::now() + Duration::from_secs(10);
    while !fx.workspace.join("late.txt").exists() {
        assert!(
            Instant::now() < deadline,
            "the descendant's expected late write inside the workspace never landed"
        );
        thread::sleep(Duration::from_millis(50));
    }

    // Tests own their processes: kill the survivor so nothing outlives the
    // test binary.
    Command::new("/bin/kill")
        .args(["-9", &survivor.to_string()])
        .status()
        .expect("kill the survivor");
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let alive = Command::new("/bin/kill")
            .args(["-0", &survivor.to_string()])
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
        if !alive {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "the survivor {survivor} survived its own cleanup kill"
        );
        thread::sleep(Duration::from_millis(50));
    }
}
