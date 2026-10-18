//! SEC-06 alternate transport: pathname Unix sockets.
//!
//! Landlock confines opening files, but `connect(2)` to a pathname socket is
//! not an open. This test creates its own listening sockets (it never looks
//! for host ones): one outside every tree the workload may use, and one under
//! a declared read path, as the PostgreSQL socket directory is. It reports
//! which an isolated workload can reach.
#![cfg(target_os = "linux")]

use sea_forge_sandbox::workload::{
    grant_exclusive_access, isolated_command, WorkloadConfinement, WorkloadIdentityPool,
};
use sea_forge_sandbox::NetworkPosture;
use std::fs;
use std::io::Read;
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
    path.to_path_buf()
}

// Known open limit (CURRENT_STATUS revision 110): this currently fails, an
// isolated workload reaches the outside socket. Run with `--ignored` to see
// it; remove the attribute once pathname sockets are contained.
#[test]
#[ignore = "open limit: pathname UNIX sockets outside declared trees are reachable"]
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
    let declared = open_dir(&base.path().join("declared"));
    let identity = pool.acquire().unwrap();
    let root = base.path().join("w");
    let confinement = WorkloadConfinement {
        workspace_root: root.join("workspace"),
        artifacts_root: root.join("artifacts"),
        read_paths: vec![declared.clone()],
        network: NetworkPosture::Denied,
    };
    fs::create_dir_all(&confinement.workspace_root).unwrap();
    fs::create_dir_all(&confinement.artifacts_root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o711)).unwrap();
    grant_exclusive_access(&confinement.workspace_root, &identity).unwrap();
    grant_exclusive_access(&confinement.artifacts_root, &identity).unwrap();

    let mut seen = Vec::new();
    for (name, dir) in [("outside", &outside), ("declared", &declared)] {
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
    // Positive control: a declared runtime socket stays reachable.
    assert_eq!(seen[1].1, "connected", "{seen:?}");
    assert_eq!(seen[1].2.as_deref(), Some("reached"), "{seen:?}");
    // Negative: a socket outside every declared tree is not.
    assert!(seen[0].1.starts_with("refused:"), "{seen:?}");
    assert_eq!(seen[0].2, None, "{seen:?}");
}
