#![cfg(unix)]
//! SEC-01/SEC-06 boundaries found by the 2026-10-17 adversarial review.
//!
//! - ADV-A05/A06/A07: an isolated workload could read every world-readable
//!   host file (the operator's home, `/mnt/c`, `/tmp`, the control-plane
//!   root). It now reads only the system trees and declared runtime paths.
//! - ADV-A19/A20: a workload outlived a SIGKILLed launcher, and its slot UID
//!   was issued again while it still ran. The launch chain now dies with the
//!   launcher, and a slot is never reissued while its UID has processes.
//!
//! Each property has a mutation control that rebuilds the weakened launch and
//! requires the attack to succeed. Hosts without subordinate-ID delegation
//! report NOT-EXECUTABLE; set `SEA_FORGE_REQUIRE_WORKLOAD_ISOLATION=1` to make
//! that a failure.
#![cfg(target_os = "linux")]

use sea_forge_sandbox::workload::{
    grant_exclusive_access, isolated_command, processes_of, Lifeline, WorkloadConfinement,
    WorkloadIdentity, WorkloadIdentityPool,
};
use sea_forge_sandbox::NetworkPosture;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const HELPER: &str = env!("CARGO_BIN_EXE_sea-forge-workload-exec");

fn pool() -> Option<WorkloadIdentityPool> {
    match WorkloadIdentityPool::from_host() {
        Ok(pool) => Some(pool),
        Err(error) => {
            assert!(
                std::env::var_os("SEA_FORGE_REQUIRE_WORKLOAD_ISOLATION").is_none(),
                "workload isolation required but unavailable: {error}"
            );
            eprintln!("NOT-EXECUTABLE: {error}");
            None
        }
    }
}

fn confinement(
    base: &Path,
    identity: &WorkloadIdentity,
    reads: Vec<PathBuf>,
) -> WorkloadConfinement {
    let root = base.join(format!("w{}", identity.slot));
    let confinement = WorkloadConfinement {
        workspace_root: root.join("workspace"),
        artifacts_root: root.join("artifacts"),
        read_paths: reads,
        socket_paths: Vec::new(),
        network: NetworkPosture::Denied,
    };
    fs::create_dir_all(&confinement.workspace_root).unwrap();
    fs::create_dir_all(&confinement.artifacts_root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o711)).unwrap();
    grant_exclusive_access(&confinement.workspace_root, identity).unwrap();
    grant_exclusive_access(&confinement.artifacts_root, identity).unwrap();
    confinement
}

fn command(
    identity: &WorkloadIdentity,
    confinement: &WorkloadConfinement,
    script: &str,
) -> (Command, Lifeline) {
    let (mut command, lifeline) = isolated_command(
        identity,
        Path::new(HELPER),
        confinement,
        &["/bin/sh".into(), "-c".into(), script.into()],
    )
    .unwrap();
    command
        .env("PATH", "/usr/bin:/bin")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    (command, lifeline)
}

fn wait_until(what: &str, mut done: impl FnMut() -> bool) {
    let start = Instant::now();
    while !done() {
        assert!(
            start.elapsed() < Duration::from_secs(20),
            "timed out: {what}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// One `name=readable|denied` line per probe target.
fn probe_reads(
    identity: &WorkloadIdentity,
    confinement: &WorkloadConfinement,
    targets: &[(&str, PathBuf)],
) -> Vec<(String, String)> {
    let out = confinement.workspace_root.join("reads.txt");
    // Directories are probed by listing, files by reading their content:
    // `ls` on a file only stats it, which Landlock does not restrict.
    let mut script = String::from(
        "set +e\nprobe() { if [ -d \"$1\" ]; then ls \"$1\"; else cat \"$1\"; fi >/dev/null 2>&1; }\n",
    );
    for (name, path) in targets {
        script.push_str(&format!(
            "if probe '{p}'; then echo {name}=readable; else echo {name}=denied; fi >> '{o}'\n",
            p = path.display(),
            o = out.display()
        ));
    }
    let (mut probe, _lifeline) = command(identity, confinement, &script);
    let status = probe.status().unwrap();
    assert!(status.success());
    fs::read_to_string(&out)
        .unwrap()
        .lines()
        .filter_map(|l| {
            l.split_once('=')
                .map(|(a, b)| (a.to_string(), b.to_string()))
        })
        .collect()
}

#[test]
fn isolated_reads_are_limited_to_system_and_declared_trees() {
    let Some(pool) = pool() else { return };
    let base = tempfile::tempdir().unwrap();
    fs::set_permissions(base.path(), fs::Permissions::from_mode(0o711)).unwrap();
    // A world-readable "host" tree outside every grant (like ~/.gitconfig or
    // a 0644 control-plane file), and a declared runtime tree.
    let host = base.path().join("host");
    let runtime = base.path().join("runtime");
    for dir in [&host, &runtime] {
        fs::create_dir_all(dir).unwrap();
        fs::set_permissions(dir, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fs::write(host.join("world-readable.txt"), "HOST-FILE").unwrap();
    fs::write(runtime.join("agent.txt"), "RUNTIME").unwrap();
    for file in [host.join("world-readable.txt"), runtime.join("agent.txt")] {
        fs::set_permissions(file, fs::Permissions::from_mode(0o644)).unwrap();
    }
    let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| "/root".into()));
    let mut targets = vec![
        ("host_file", host.join("world-readable.txt")),
        ("home", home),
        ("tmp", PathBuf::from("/tmp")),
        ("run", PathBuf::from("/run")),
        ("var", PathBuf::from("/var/log")),
        ("sys", PathBuf::from("/sys/kernel")),
        ("declared", runtime.join("agent.txt")),
        ("system", PathBuf::from("/usr/bin/env")),
        ("etc", PathBuf::from("/etc/passwd")),
    ];
    if Path::new("/mnt/c").exists() {
        targets.push(("windows_drive", PathBuf::from("/mnt/c")));
    }
    let identity = pool.acquire().unwrap();
    let narrow = confinement(
        base.path(),
        &identity,
        vec![runtime.canonicalize().unwrap()],
    );
    let rows = probe_reads(&identity, &narrow, &targets);
    let verdict = |rows: &[(String, String)], name: &str| {
        rows.iter().find(|(n, _)| n == name).map(|(_, v)| v.clone())
    };
    for denied in [
        "host_file",
        "home",
        "tmp",
        "run",
        "var",
        "sys",
        "windows_drive",
    ] {
        if targets.iter().any(|(n, _)| *n == denied) {
            assert_eq!(
                verdict(&rows, denied).as_deref(),
                Some("denied"),
                "{denied}: {rows:?}"
            );
        }
    }
    // Positive controls: the agent can still run and read what it needs.
    for allowed in ["declared", "system", "etc"] {
        assert_eq!(
            verdict(&rows, allowed).as_deref(),
            Some("readable"),
            "{allowed}: {rows:?}"
        );
    }

    // Mutation control: declaring the host tree makes the same probe succeed,
    // so the denial above is the read posture, not a broken probe.
    let widened = WorkloadConfinement {
        read_paths: vec![
            runtime.canonicalize().unwrap(),
            host.canonicalize().unwrap(),
        ],
        ..narrow
    };
    fs::remove_file(widened.workspace_root.join("reads.txt")).unwrap();
    let rows = probe_reads(&identity, &widened, &targets);
    assert_eq!(
        verdict(&rows, "host_file").as_deref(),
        Some("readable"),
        "{rows:?}"
    );
}

const FIXTURE: &str = "SEA_FORGE_LAUNCHER_FIXTURE_DIR";
const MUTATION: &str = "SEA_FORGE_LAUNCHER_FIXTURE_NO_LIFELINE";

/// A disposable launcher process (this test binary, re-executed): it takes a
/// slot, starts a long-running workload exactly as production does, records
/// the workload's UID and slot, then waits to be killed.
#[test]
fn launcher_fixture_child() {
    let Some(dir) = std::env::var_os(FIXTURE).map(PathBuf::from) else {
        return;
    };
    let pool = WorkloadIdentityPool::from_host().unwrap();
    let identity = pool.acquire().unwrap();
    let dirs = confinement(&dir, &identity, Vec::new());
    let ready = dirs.workspace_root.join("ready");
    let script = format!("echo ready > '{}'; exec sleep 600", ready.display());
    let (production, lifeline) = command(&identity, &dirs, &script);
    let mut launch = if std::env::var_os(MUTATION).is_some() {
        // Mutation control: the same launch without the lifeline.
        let args: Vec<_> = production.get_args().collect();
        let at = args
            .iter()
            .position(|a| *a == "--lifeline")
            .expect("launch shape changed");
        let mut weakened = Command::new(production.get_program());
        weakened
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .args(&args[..at])
            .args(&args[at + 2..])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        weakened
    } else {
        production
    };
    let mut child = launch.spawn().unwrap();
    wait_until("workload ready", || ready.exists());
    fs::write(
        dir.join("identity"),
        format!("{} {}", identity.host_uid, identity.slot),
    )
    .unwrap();
    let _keep = (lifeline, identity);
    // Killed here by the test; the wait only runs if nobody does.
    std::thread::sleep(Duration::from_secs(600));
    let _ = child.wait();
}

/// A traversable directory for one launch: a reissued slot must not meet
/// the previous workload's files (granting refuses them as not
/// launcher-owned).
fn fresh(base: &Path, name: &str) -> PathBuf {
    let dir = base.join(name);
    fs::create_dir_all(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o711)).unwrap();
    dir
}

/// Run the launcher fixture, SIGKILL it once its workload runs, and return
/// the workload's (UID, slot).
fn kill_launcher_mid_episode(dir: &Path, mutation: bool) -> (u32, u32) {
    let mut launcher = Command::new(std::env::current_exe().unwrap());
    launcher
        .args([
            "--exact",
            "launcher_fixture_child",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(FIXTURE, dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        // Fixture failures should be visible in this test's output.
        .stderr(Stdio::inherit());
    if mutation {
        launcher.env(MUTATION, "1");
    }
    let mut launcher: Child = launcher.spawn().unwrap();
    let record = dir.join("identity");
    wait_until("launcher fixture ready", || record.exists());
    let text = fs::read_to_string(&record).unwrap();
    fs::remove_file(&record).unwrap();
    let (uid, slot) = text.split_once(' ').unwrap();
    let (uid, slot) = (uid.parse().unwrap(), slot.parse().unwrap());
    assert!(!processes_of(uid).is_empty(), "workload not running");
    launcher.kill().unwrap();
    launcher.wait().unwrap();
    (uid, slot)
}

#[test]
fn workload_dies_with_launcher_and_orphans_are_reaped_before_reissue() {
    let Some(pool) = pool() else { return };
    let base = tempfile::tempdir().unwrap();
    fs::set_permissions(base.path(), fs::Permissions::from_mode(0o711)).unwrap();

    // Production launch: SIGKILLing the launcher ends the workload.
    let (uid, _) = kill_launcher_mid_episode(&fresh(base.path(), "production"), false);
    wait_until("workload ended with its launcher", || {
        processes_of(uid).is_empty()
    });

    // Mutation control: without the lifeline the workload outlives a
    // SIGKILLed launcher under the slot's UID (ADV-A19).
    let (orphan_uid, orphan_slot) =
        kill_launcher_mid_episode(&fresh(base.path(), "mutation"), true);
    std::thread::sleep(Duration::from_millis(500));
    assert!(
        !processes_of(orphan_uid).is_empty(),
        "mutation control: orphan expected without the lifeline"
    );

    // ADV-A20: the dead launcher's slot lock is free, but the slot must not
    // be handed to the next workload while the orphan runs. Reissue reaps it.
    let mut held = Vec::new();
    while held.len() < 16 {
        let next = pool.acquire().unwrap();
        let slot = next.slot;
        held.push(next);
        if slot == orphan_slot {
            break;
        }
    }
    assert_eq!(
        held.last().map(|i| i.slot),
        Some(orphan_slot),
        "slot not reissued"
    );
    assert!(
        processes_of(orphan_uid).is_empty(),
        "slot reissued while its previous workload still runs"
    );
}

/// Connect attempts from inside a workload: `name=connected|refused`.
fn probe_connects(
    identity: &WorkloadIdentity,
    confinement: &WorkloadConfinement,
    targets: &[(&str, String)],
) -> Vec<(String, String)> {
    let out = confinement.workspace_root.join("connects.txt");
    let _ = fs::remove_file(&out);
    let mut script = String::new();
    for (name, target) in targets {
        script.push_str(&format!(
            "if /usr/bin/python3 -I -c \"import socket,sys; t=sys.argv[1]; \
             s=socket.socket(socket.AF_UNIX) if t.startswith('@') else socket.socket(); \
             s.settimeout(3); s.connect('\\0'+t[1:] if t.startswith('@') else ('127.0.0.1', int(t)))\" '{target}' \
             2>/dev/null; then echo {name}=connected; else echo {name}=refused; fi >> '{o}'\n",
            o = out.display()
        ));
    }
    let (mut probe, _lifeline) = command(identity, confinement, &script);
    assert!(probe.status().unwrap().success());
    fs::read_to_string(&out)
        .unwrap()
        .lines()
        .filter_map(|l| {
            l.split_once('=')
                .map(|(a, b)| (a.to_string(), b.to_string()))
        })
        .collect()
}

#[test]
fn isolated_network_layers_hold_independently() {
    use std::os::linux::net::SocketAddrExt;
    let Some(pool) = pool() else { return };
    if !Path::new("/usr/bin/python3").exists() {
        eprintln!("NOT-EXECUTABLE: /usr/bin/python3 absent");
        return;
    }
    let base = tempfile::tempdir().unwrap();
    fs::set_permissions(base.path(), fs::Permissions::from_mode(0o711)).unwrap();
    let granted = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let other = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let abstract_name = format!("sea-forge-adv-{}", std::process::id());
    let addr = std::os::unix::net::SocketAddr::from_abstract_name(&abstract_name).unwrap();
    let abstract_listener = std::os::unix::net::UnixListener::bind_addr(&addr).unwrap();
    // The host side can reach all three: the listeners are live.
    std::os::unix::net::UnixStream::connect_addr(&addr).unwrap();
    let _ = abstract_listener.accept().unwrap();
    let (gp, op) = (
        granted.local_addr().unwrap().port(),
        other.local_addr().unwrap().port(),
    );
    let targets = [
        ("granted", gp.to_string()),
        ("other", op.to_string()),
        ("abstract", format!("@{abstract_name}")),
    ];
    let identity = pool.acquire().unwrap();
    let dirs = confinement(base.path(), &identity, Vec::new());

    // A TCP grant drops the network namespace, so only Landlock separates
    // the granted port (positive control) from every other destination.
    let landlock_only = WorkloadConfinement {
        socket_paths: Vec::new(),
        network: NetworkPosture::from_granted_ports(vec![gp]),
        ..dirs.clone()
    };
    let rows = probe_connects(&identity, &landlock_only, &targets);
    let verdict = |rows: &[(String, String)], n: &str| {
        rows.iter().find(|(k, _)| k == n).map(|(_, v)| v.clone())
    };
    assert_eq!(
        verdict(&rows, "granted").as_deref(),
        Some("connected"),
        "{rows:?}"
    );
    assert_eq!(
        verdict(&rows, "other").as_deref(),
        Some("refused"),
        "{rows:?}"
    );
    assert_eq!(
        verdict(&rows, "abstract").as_deref(),
        Some("refused"),
        "{rows:?}"
    );

    // Default posture: network namespace and Landlock both deny.
    let rows = probe_connects(&identity, &dirs, &targets);
    for name in ["granted", "other", "abstract"] {
        assert_eq!(
            verdict(&rows, name).as_deref(),
            Some("refused"),
            "{name}: {rows:?}"
        );
    }
}
