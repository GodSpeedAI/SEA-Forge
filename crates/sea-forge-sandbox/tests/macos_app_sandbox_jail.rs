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

/// The cage-persists helper's protocol, as C. The helper must be a tiny
/// program linked against the system library ONLY: a copied test binary
/// links whatever the build machine's toolchain links (the round-2 macOS
/// runner linked a nix libiconv, which the App Sandbox cage — correctly —
/// refused to load, `dyld: Library not loaded ... sandbox blocked open()`,
/// so the workload never ran). `cc` against `/usr/lib/libSystem.dylib` is
/// the one linkage the cage is guaranteed to allow.
const CAGE_HELPER_C: &str = r#"/* Generated by macos_app_sandbox_jail.rs. Do not edit by hand. */
/* Cage-persists helper: runs inside the jail as the F08 setsid descendant.
   argv[1] = absolute workspace path (sandbox-granted),
   argv[2] = absolute directory outside the granted trees.
   Sequence: mark the earliest reachable step (helper_started.txt), create a
   new session (the process-group kill cannot reach it — the F08 escape),
   fork; the parent lingers ~1.5 s before exiting so the detached child stays
   attached under the workload's ppid tree for the runtime census's 100 ms
   samples (the round-3 finding: a parent that exits within milliseconds
   reparents the child to launchd before any census slice, and the survivor
   was never recorded), then in the detached child: attempt the
   outside write and record its outcome, record this pid (after setsid), write
   late.txt into the workspace after a delay, then stay alive for the parent's
   survivor assertion (the parent test kills us). The markers discriminate a
   helper that never ran at all (no helper_started.txt), one that died at
   setsid/fork (helper-failed.txt), and one that recorded (detached.pid). */
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

static void write_marker(const char *dir, const char *name, const char *text) {
    char path[4096];
    int n = snprintf(path, sizeof(path), "%s/%s", dir, name);
    if (n <= 0 || (size_t)n >= sizeof(path)) {
        return;
    }
    int fd = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0600);
    if (fd < 0) {
        return;
    }
    ssize_t ignored = write(fd, text, strlen(text));
    (void)ignored;
    (void)close(fd);
}

int main(int argc, char **argv) {
    if (argc != 3) {
        return 2;
    }
    const char *workspace = argv[1];
    const char *outside = argv[2];
    char buffer[64];

    write_marker(workspace, "helper_started.txt", "1");

    /* setsid(2): a new session and process group that the workload's group
       kill cannot reach. Fails (EPERM) only if this process were already a
       group leader, which the background-spawn shape prevents. */
    if (setsid() < 0) {
        write_marker(workspace, "helper-failed.txt", strerror(errno));
        return 0;
    }

    /* Fork; the surviving descendant is this child, still in the new
       session. The parent stays resident ~1.5 s before exiting: the
       runtime's census only samples the workload's ppid tree while the
       workload runs, and this process is the child's link into that tree.
       Exiting within milliseconds (the round-3 finding) reparents the child
       to launchd before any 100 ms census slice, so the survivor was never
       recorded. 1.5 s is far inside the run's 20 s budget. */
    pid_t child = fork();
    if (child < 0) {
        write_marker(workspace, "helper-failed.txt", strerror(errno));
        return 0;
    }
    if (child > 0) {
        usleep(1500 * 1000);
        return 0; /* parent exits; the child continues detached */
    }

    /* The escape attempt: write outside the sandbox-allowed trees. The App
       Sandbox must deny this even though the group kill already missed us. */
    char outside_path[4096];
    int outside_fd = -1;
    int n = snprintf(outside_path, sizeof(outside_path),
                     "%s/outside-marker.txt", outside);
    if (n > 0 && (size_t)n < sizeof(outside_path)) {
        outside_fd = open(outside_path, O_WRONLY | O_CREAT | O_TRUNC, 0600);
    }
    if (outside_fd >= 0) {
        ssize_t ignored = write(outside_fd, "escape", 6);
        (void)ignored;
        (void)close(outside_fd);
        write_marker(workspace, "outside_denied.txt", "0");
    } else {
        write_marker(workspace, "outside_denied.txt", "1");
    }

    /* Record the detached pid: this process, after setsid (and fork). */
    snprintf(buffer, sizeof(buffer), "%ld\n", (long)getpid());
    write_marker(workspace, "detached.pid", buffer);

    /* The late write INSIDE the authorized workspace, after the run has
       settled (the workload exits once detached.pid exists). */
    sleep(3);
    write_marker(workspace, "late.txt", "late");

    /* Hold the pid alive for the parent's survivor assertion. */
    sleep(20);
    return 0;
}
"#;

/// F08 cage-persists regression: a workload whose `setsid`ed descendant
/// outlives cancellation keeps writing INSIDE the authorized workspace, but
/// its write OUTSIDE the sandbox-allowed trees stays denied. Asserts the
/// documented F08 macOS boundary from outside the child (host-side pid
/// liveness, host-side file existence), following this file's convention of
/// never trusting the child's self-report.
///
/// The helper is compiled at setup with `cc` against the system library only
/// (see `CAGE_HELPER_C`) and gated on `otool -L`: any dependency beyond
/// `/usr/lib/libSystem.B.dylib` fails the test loudly, because a foreign
/// dylib (a nix/homebrew toolchain's libiconv, the round-2 finding) cannot
/// be loaded inside the App Sandbox cage and the workload would time out
/// with no marker at all.
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
    // Generate and compile the helper: source in the temp root, binary in
    // the granted workspace (the jailed workload can only exec from there).
    let source = fx._root.path().join("cage-helper.c");
    fs::write(&source, CAGE_HELPER_C).expect("write cage-helper.c");
    let helper = fx.workspace.join("cage-helper");
    let compiled = Command::new("cc")
        .arg("-O1")
        .arg("-o")
        .arg(&helper)
        .arg(&source)
        .output()
        .expect("run cc for the cage helper");
    assert!(
        compiled.status.success(),
        "compiling the cage helper failed: {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    // The helper must link the system library only: the cage blocks foreign
    // dylibs (round-2: a nix libiconv), and a helper that cannot load never
    // records a marker.
    let listing = Command::new("otool")
        .arg("-L")
        .arg(&helper)
        .output()
        .expect("run otool -L on the compiled cage helper");
    assert!(
        listing.status.success(),
        "otool -L failed: {}",
        String::from_utf8_lossy(&listing.stderr)
    );
    let listing = String::from_utf8_lossy(&listing.stdout).into_owned();
    assert!(
        listing.contains("/usr/lib/libSystem.B.dylib"),
        "the cage helper must link the system library: {listing}"
    );
    let unexpected: Vec<&str> = listing
        .lines()
        .skip(1) // the first line is the binary's own path
        .map(str::trim)
        .filter_map(|line| line.split_whitespace().next())
        .filter(|dependency| *dependency != "/usr/lib/libSystem.B.dylib")
        .collect();
    assert!(
        unexpected.is_empty(),
        "the cage helper links libraries the App Sandbox cage may refuse to load \
         (round-2 finding: a nix libiconv left the helper dead inside the cage); \
         unexpected dependencies: {unexpected:?}\nfull otool -L output:\n{listing}"
    );

    let request = ExecutionRequest {
        plan_item_id: "macos-app-sandbox-jail-cage-persists".into(),
        operation: Operation::ExecuteCommand {
            argv: sh(&format!(
                "'{helper}' '{ws}' '{out}' & \
                 i=0; while [ ! -f '{ws}/detached.pid' ] && [ $i -lt 200 ]; do \
                 i=$((i+1)); sleep 0.05; done; \
                 [ -f '{ws}/detached.pid' ] || {{ echo helper-never-recorded; exit 9; }}; \
                 sleep 1",
                helper = helper.display(),
                ws = fx.workspace.display(),
                out = fx.outside.display(),
            )),
            cwd: ".".into(),
        },
        // The workload's own fallback budget is 200 × 50ms = 10s, strictly
        // inside this execute timeout: a helper that never records makes the
        // workload exit 9 with "helper-never-recorded" ~10s BEFORE the
        // deadline, so the failure surfaces as Completed-with-exit-9 (the
        // assertion below prints the artifacts) instead of a bare TimedOut
        // whose evidence is masked by the timeout boundary. The trailing
        // `sleep 1` keeps the workload alive for ~10 census slices AFTER the
        // marker appears: the census samples the workload's ppid tree only
        // while the workload lives, and an exit within ~50-100ms of the
        // marker left the recording to a single race-prone slice (round 12:
        // four green rounds then ProcessGroup).
        timeout_secs: 20,
        env: BTreeMap::from([("PATH".to_string(), PATH_ENV.to_string())]),
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
