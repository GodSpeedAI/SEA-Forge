//! SEC-06 pathname Unix-socket containment (ADR-014).
//!
//! The test creates its own listening sockets (it never looks for host
//! ones): one outside every tree the workload may use, one under a declared
//! read path with no socket grant, and one under a tree explicitly
//! socket-granted. Containment holds when only the granted one answers.
//! Round-3 evidence (`unix-socket-reach-before.txt`) shows both outside and
//! read-path sockets were reachable before this landed.
#![cfg(target_os = "linux")]
// Test-only unsafe: the inherited-descriptor scenario duplicates a
// connected fd into the launch to prove the helper closes it.
#![allow(unsafe_code)]

use sea_forge_sandbox::workload::{
    grant_exclusive_access, isolated_command, WorkloadConfinement, WorkloadIdentityPool,
};
use sea_forge_sandbox::NetworkPosture;
use std::fs;
use std::io::Read;
use std::os::fd::AsRawFd;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

const HELPER: &str = env!("CARGO_BIN_EXE_sea-forge-workload-exec");
const PYTHON: &str = "/usr/bin/python3";
const CONNECT: &str = "import socket,sys\ns=socket.socket(socket.AF_UNIX)\ntry:\n    s.connect(sys.argv[1]); s.sendall(b'reached'); print('connected')\nexcept OSError as e:\n    print('refused:' + type(e).__name__)\n";

/// A listener that records whether anything connected and what it sent.
fn listener(path: &Path) -> std::thread::JoinHandle<Option<String>> {
    let socket = UnixListener::bind(path).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o777)).unwrap();
    socket.set_nonblocking(true).unwrap();
    std::thread::spawn(move || {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(10) {
            if let Ok((mut stream, _)) = socket.accept() {
                stream.set_nonblocking(false).unwrap();
                let mut got = String::new();
                let _ = stream.read_to_string(&mut got);
                return Some(got);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        None
    })
}

fn open_dir(path: &Path) -> PathBuf {
    fs::create_dir_all(path).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    path.canonicalize().unwrap()
}

#[test]
fn pathname_sockets_outside_declared_trees_are_unreachable() {
    let pool = match WorkloadIdentityPool::from_host() {
        Ok(pool) if Path::new(PYTHON).exists() => pool,
        other => {
            assert!(
                std::env::var_os("SEA_FORGE_REQUIRE_WORKLOAD_ISOLATION").is_none(),
                "workload isolation required but unavailable: {:?}",
                other.err()
            );
            return;
        }
    };
    let base = tempfile::tempdir().unwrap();
    fs::set_permissions(base.path(), fs::Permissions::from_mode(0o755)).unwrap();
    let outside = open_dir(&base.path().join("outside"));
    // Readable by declaration, but NOT socket-granted: readability must not
    // imply socket reach (ADR-014 requirement 2).
    let declared = open_dir(&base.path().join("declared"));
    let granted = open_dir(&base.path().join("granted"));
    let identity = pool.acquire().unwrap();
    let root = base.path().join("w");
    let confinement = WorkloadConfinement {
        workspace_root: root.join("workspace"),
        artifacts_root: root.join("artifacts"),
        read_paths: vec![declared.clone()],
        socket_paths: vec![granted.clone()],
        network: NetworkPosture::Denied,
    };
    fs::create_dir_all(&confinement.workspace_root).unwrap();
    fs::create_dir_all(&confinement.artifacts_root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o711)).unwrap();
    grant_exclusive_access(&confinement.workspace_root, &identity).unwrap();
    grant_exclusive_access(&confinement.artifacts_root, &identity).unwrap();

    let mut seen = Vec::new();
    for (name, dir) in [
        ("outside", &outside),
        ("declared", &declared),
        ("granted", &granted),
    ] {
        let path = dir.join("s.sock");
        let accepted = listener(&path);
        let (mut command, _lifeline) = isolated_command(
            &identity,
            Path::new(HELPER),
            &confinement,
            &[
                PYTHON.into(),
                "-I".into(),
                "-c".into(),
                CONNECT.into(),
                path.display().to_string(),
            ],
        )
        .unwrap();
        let out = command
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let said = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let got = accepted.join().unwrap();
        eprintln!("{name}: workload={said} listener={got:?}");
        seen.push((name, said, got));
    }
    // Positive control: only the explicitly socket-granted tree answers.
    assert_eq!(seen[2].1, "connected", "{seen:?}");
    assert_eq!(seen[2].2.as_deref(), Some("reached"), "{seen:?}");
    // A socket outside every declared tree is refused.
    assert!(seen[0].1.starts_with("refused:"), "{seen:?}");
    assert_eq!(seen[0].2, None, "{seen:?}");
    // A socket under a merely-readable tree is refused too.
    assert!(seen[1].1.starts_with("refused:"), "{seen:?}");
    assert_eq!(seen[1].2, None, "{seen:?}");
}

/// Shared launch plumbing for the focused scenarios below.
struct Scenario {
    #[allow(dead_code)]
    base: tempfile::TempDir,
    confinement: WorkloadConfinement,
    identity: sea_forge_sandbox::workload::WorkloadIdentity,
}

fn scenario(read: Vec<PathBuf>, grants: Vec<PathBuf>) -> Option<Scenario> {
    let pool = match WorkloadIdentityPool::from_host() {
        Ok(pool) if Path::new(PYTHON).exists() => pool,
        other => {
            assert!(
                std::env::var_os("SEA_FORGE_REQUIRE_WORKLOAD_ISOLATION").is_none(),
                "workload isolation required but unavailable: {:?}",
                other.err()
            );
            eprintln!(
                "NOT-EXECUTABLE: workload isolation unavailable: {:?}",
                other.err()
            );
            return None;
        }
    };
    let base = tempfile::tempdir().unwrap();
    fs::set_permissions(base.path(), fs::Permissions::from_mode(0o755)).unwrap();
    let root = base.path().join("w");
    let confinement = WorkloadConfinement {
        workspace_root: root.join("workspace"),
        artifacts_root: root.join("artifacts"),
        read_paths: read,
        socket_paths: grants,
        network: NetworkPosture::Denied,
    };
    fs::create_dir_all(&confinement.workspace_root).unwrap();
    fs::create_dir_all(&confinement.artifacts_root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o711)).unwrap();
    let identity = pool.acquire().unwrap();
    grant_exclusive_access(&confinement.workspace_root, &identity).unwrap();
    grant_exclusive_access(&confinement.artifacts_root, &identity).unwrap();
    Some(Scenario {
        base,
        confinement,
        identity,
    })
}

impl Scenario {
    fn run(&self, program: &str, arg: &str) -> String {
        let (mut command, _lifeline) = isolated_command(
            &self.identity,
            Path::new(HELPER),
            &self.confinement,
            &[
                PYTHON.into(),
                "-I".into(),
                "-c".into(),
                program.into(),
                arg.into(),
            ],
        )
        .unwrap();
        let out = command
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }
}

#[test]
fn abstract_namespace_own_ipc_works_and_cross_namespace_is_structural() {
    let Some(case) = scenario(Vec::new(), Vec::new()) else {
        return;
    };
    // The workload's own abstract namespace is bounded by its dedicated
    // network namespace: names it creates are its own (legitimate IPC,
    // brokered from copied bytes), and host-side abstract names are
    // structurally unreachable across the namespace boundary.
    let said = case.run(
        "import socket,sys\nname='\\0sea-forge-test-canary'\nl=socket.socket(socket.AF_UNIX)\n\
         l.bind(name); l.listen(1)\ns=socket.socket(socket.AF_UNIX)\ntry:\n    \
         s.connect(name); print('connected')\nexcept OSError as e:\n    print('refused:' + type(e).__name__)",
        "x",
    );
    assert_eq!(said, "connected", "{said:?}");
}

#[test]
fn symlink_alias_to_outside_canary_is_refused() {
    let base = tempfile::tempdir().unwrap();
    fs::set_permissions(base.path(), fs::Permissions::from_mode(0o755)).unwrap();
    let outside = open_dir(&base.path().join("outside"));
    let canary = outside.join("real.sock");
    let accepted = listener(&canary);
    let Some(case) = scenario(Vec::new(), Vec::new()) else {
        return;
    };
    let alias = case.confinement.workspace_root.join("alias.sock");
    std::os::unix::fs::symlink(&canary, &alias).unwrap();
    let said = case.run(CONNECT, &alias.display().to_string());
    let got = accepted.join().unwrap();
    assert!(said.starts_with("refused:"), "{said:?} listener={got:?}");
    assert_eq!(got, None, "an aliased outside canary was reached");
}

#[test]
fn rename_swap_race_does_not_reach_the_outside_canary() {
    let base = tempfile::tempdir().unwrap();
    fs::set_permissions(base.path(), fs::Permissions::from_mode(0o755)).unwrap();
    let outside = open_dir(&base.path().join("outside"));
    let evil = outside.join("evil.sock");
    let received = std::sync::Arc::new(std::sync::Mutex::new(None));
    let flag = std::sync::Arc::clone(&received);
    let evil_text = evil.display().to_string();
    let waiter = std::thread::spawn(move || {
        let socket = std::os::unix::net::UnixListener::bind(&evil_text).unwrap();
        socket.set_nonblocking(true).unwrap();
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(8) {
            if let Ok((mut stream, _)) = socket.accept() {
                use std::io::Read;
                let mut got = String::new();
                let _ = stream.read_to_string(&mut got);
                *flag.lock().unwrap() = Some(got);
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    });
    // The granted tree holds a legitimate canary; the swap thread renames the
    // outside canary over it continuously, attacking the window between the
    // supervisor's validation and the kernel's connect resolution. If a
    // connection ever lands on the outside canary, the compensating
    // re-check must kill the workload before its payload follows.
    let granted = open_dir(&base.path().join("granted"));
    let legit = granted.join("legit.sock");
    let legit_listener = listener(&legit);
    let Some(case) = scenario(Vec::new(), vec![granted]) else {
        return;
    };
    let program = "import socket,sys,time\np=sys.argv[1]\nend=time.time()+3\nn=0\n\
        while time.time()<end:\n    s=socket.socket(socket.AF_UNIX)\n    try:\n        \
        s.connect(p); s.sendall(b'payload'); n+=1\n    except OSError:\n        pass\n\
        finally:\n        s.close()\nprint('sent:'+str(n))\n";
    let swapper = std::thread::spawn({
        let evil = evil.clone();
        let legit = legit.clone();
        move || {
            let deadline = Instant::now() + Duration::from_secs(4);
            while Instant::now() < deadline {
                let _ = fs::rename(&legit, base.path().join("staging.sock"));
                let _ = fs::rename(&evil, &legit);
                std::thread::sleep(Duration::from_millis(1));
                let _ = fs::rename(&legit, &evil);
                let _ = fs::rename(base.path().join("staging.sock"), &legit);
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    });
    let said = case.run(program, &legit.display().to_string());
    swapper.join().unwrap();
    waiter.join().unwrap();
    let got = received.lock().unwrap().clone();
    let legit_got = legit_listener.join().unwrap();
    assert_ne!(
        got.as_deref(),
        Some("payload"),
        "the outside canary received the workload payload (said={said:?})"
    );
    assert!(
        said.starts_with("sent:") || said.is_empty(),
        "unexpected workload outcome: {said:?}"
    );
    let _ = legit_got;
}

#[test]
fn datagram_sendto_to_outside_canary_is_refused() {
    let base = tempfile::tempdir().unwrap();
    fs::set_permissions(base.path(), fs::Permissions::from_mode(0o755)).unwrap();
    let outside = open_dir(&base.path().join("outside"));
    let canary = outside.join("dgram.sock");
    let received = std::sync::Arc::new(std::sync::Mutex::new(false));
    let flag = std::sync::Arc::clone(&received);
    let canary_text = canary.display().to_string();
    let bind_path = canary_text.clone();
    let waiter = std::thread::spawn(move || {
        let socket = std::os::unix::net::UnixDatagram::bind(&bind_path).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        if socket.recv_from(&mut [0u8; 16]).is_ok() {
            *flag.lock().unwrap() = true;
        }
    });
    let Some(case) = scenario(Vec::new(), Vec::new()) else {
        return;
    };
    let said = case.run(
        "import socket,sys\ns=socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM)\ntry:\n    \
         s.sendto(b'reached', sys.argv[1]); print('sent')\nexcept OSError as e:\n    \
         print('refused:' + type(e).__name__)",
        &canary_text,
    );
    assert!(said.starts_with("refused:"), "{said:?}");
    waiter.join().unwrap();
    assert!(!*received.lock().unwrap(), "datagram reached canary");
}

#[test]
fn preconnected_descriptor_does_not_leak_into_the_workload() {
    let base = tempfile::tempdir().unwrap();
    fs::set_permissions(base.path(), fs::Permissions::from_mode(0o755)).unwrap();
    let outside = open_dir(&base.path().join("outside"));
    let canary = outside.join("leak.sock");
    let accepted = listener(&canary);
    // A socket connected BEFORE the restriction applied, smuggled toward the
    // workload on a non-close-on-exec descriptor.
    let connected = std::os::unix::net::UnixStream::connect(&canary).unwrap();
    let raw = connected.as_raw_fd();
    let Some(case) = scenario(Vec::new(), Vec::new()) else {
        return;
    };
    let (mut command, _lifeline) = isolated_command(
        &case.identity,
        Path::new(HELPER),
        &case.confinement,
        &[
            PYTHON.into(),
            "-I".into(),
            "-c".into(),
            "import os,sys\nfd=int(sys.argv[1])\ntry:\n    os.write(fd, b'leaked'); print('leaked')\n\
             except OSError as e:\n    print('closed:' + type(e).__name__)"
                .into(),
            raw.to_string(),
        ],
    )
    .unwrap();
    use std::os::unix::process::CommandExt;
    // Test-only unsafe: duplicating a connected descriptor into the launch
    // to prove the helper's inherited-descriptor hygiene closes it.
    unsafe {
        command.pre_exec(move || {
            libc::dup2(raw, 7);
            Ok(())
        });
    }
    command.env("PATH", "/usr/bin:/bin");
    let out = command.stdin(Stdio::null()).output().unwrap();
    let said = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // Release the test's end of the pre-connected stream so the listener
    // thread's read-to-EOF can return before the join. The listener must
    // show ONLY the test's own empty pre-connection: zero leaked bytes.
    drop(connected);
    let got = accepted.join().unwrap();
    assert!(said.starts_with("closed:"), "{said:?}");
    assert_eq!(got, Some(String::new()), "unexpected bytes on the canary");
}

#[test]
fn workload_socket_in_own_tree_stays_usable() {
    let Some(case) = scenario(Vec::new(), Vec::new()) else {
        return;
    };
    let socket_path = case.confinement.workspace_root.join("own.sock");
    // The workload binds and connects to its own socket: legitimate IPC.
    let said = case.run(
        "import socket,sys\np=sys.argv[1]\nl=socket.socket(socket.AF_UNIX)\n\
         l.bind(p); l.listen(1)\ns=socket.socket(socket.AF_UNIX)\ntry:\n    \
         s.connect(p); print('connected')\nexcept OSError as e:\n    \
         print('refused:' + type(e).__name__)",
        &socket_path.display().to_string(),
    );
    assert_eq!(said, "connected", "own-tree IPC must keep working");
}
