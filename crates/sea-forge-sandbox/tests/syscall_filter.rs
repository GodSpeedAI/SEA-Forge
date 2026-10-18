//! SEC-06 syscall surface of an isolated workload.
//!
//! Landlock confines what an isolated workload can open, connect to and
//! signal, but leaves two large kernel surfaces reachable: io_uring, and new
//! namespaces (an unprivileged user namespace grants the capabilities that
//! gate most of the kernel's privileged code). The workload exec helper
//! installs a seccomp filter after Landlock that refuses both with `EPERM`,
//! and refuses `clone3` with `ENOSYS` because its flags cannot be inspected
//! (libc then falls back to `clone`, whose flags are checked).
//!
//! The probe is arbitrary workload code (Python `ctypes`). Its mutation
//! control runs the same probe as the same kind of identity without the
//! helper, proving each refused operation is otherwise available here.
#![cfg(target_os = "linux")]

use sea_forge_sandbox::workload::{
    grant_exclusive_access, isolated_command, WorkloadConfinement, WorkloadIdentity,
    WorkloadIdentityPool, WORKLOAD_INNER_ID,
};
use sea_forge_sandbox::NetworkPosture;
use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

const HELPER: &str = env!("CARGO_BIN_EXE_sea-forge-workload-exec");
const PYTHON: &str = "/usr/bin/python3";

/// Each probe runs in a forked child, so a namespace it does create never
/// changes the probes after it. Values are `ok` or the errno name.
const PROBE: &str = r#"
import ctypes, errno, json, os, subprocess, sys, threading
libc = ctypes.CDLL(None, use_errno=True)
libc.syscall.restype = ctypes.c_long
def raw(nr, *args):
    r = libc.syscall(ctypes.c_long(nr), *[ctypes.c_long(a) for a in args])
    return r, ctypes.get_errno()
def isolated(fn):
    rd, wr = os.pipe()
    pid = os.fork()
    if pid == 0:
        os.close(rd)
        try: out = fn()
        except Exception as e: out = 'exception:' + type(e).__name__
        os.write(wr, out.encode()); os._exit(0)
    os.close(wr)
    out = os.read(rd, 256).decode(); os.close(rd); os.waitpid(pid, 0)
    return out
def result(r, e):
    if r == 0: os._exit(0)   # inside a child that clone created
    return 'ok' if r > 0 or r == 0 else errno.errorcode.get(e, str(e))
def call(nr, *args):
    r, e = raw(nr, *args)
    if r >= 0: return 'ok'
    return errno.errorcode.get(e, str(e))
NEWUSER, NEWNET, NEWNS, SIGCHLD = 0x10000000, 0x40000000, 0x00020000, 17
params = ctypes.create_string_buffer(120)
ROOT = ctypes.create_string_buffer(b'/')
def nested_mount():
    # A namespace's capabilities are what a mount escape needs; Landlock
    # alone also refuses topology changes, so this shows which layer holds.
    r, e = raw(272, NEWUSER | NEWNS)
    if r < 0: return 'unshare:' + errno.errorcode.get(e, str(e))
    r, e = raw(165, 0, ctypes.addressof(ROOT), 0, 0x4000 | 0x40000, 0)
    return 'ok' if r == 0 else 'mount:' + errno.errorcode.get(e, str(e))
clone_args = (ctypes.c_uint64 * 11)(NEWUSER, 0, 0, 0, SIGCHLD, 0, 0, 0, 0, 0, 0)
res = {
  'io_uring_setup': isolated(lambda: call(425, 1, ctypes.addressof(params))),
  'io_uring_enter': isolated(lambda: call(426, -1, 0, 0, 0, 0, 0)),
  'io_uring_register': isolated(lambda: call(427, -1, 0, 0, 0)),
  'unshare_user': isolated(lambda: call(272, NEWUSER)),
  'unshare_user_net': isolated(lambda: call(272, NEWUSER | NEWNET)),
  'unshare_user_mount': isolated(lambda: call(272, NEWUSER | NEWNS)),
  'clone_newuser': isolated(lambda: result(*raw(56, NEWUSER | SIGCHLD, 0, 0, 0, 0))),
  'clone3_newuser': isolated(lambda: result(*raw(435, ctypes.addressof(clone_args), 88))),
  'setns': isolated(lambda: call(308, -1, 0)),
  'nested_mount': isolated(nested_mount),
  # The x32 ABI: the same calls with bit 30 set (x86_64 only).
  'x32_io_uring_setup': isolated(lambda: call(0x40000000 | 425, 1, ctypes.addressof(params))),
  'x32_unshare_user': isolated(lambda: call(0x40000000 | 272, NEWUSER)),
}
# Positive controls: ordinary process, thread and file work still succeeds.
res['fork'] = isolated(lambda: 'ok')
box = []
t = threading.Thread(target=lambda: box.append('ok')); t.start(); t.join()
res['thread'] = box[0] if box else 'failed'
res['subprocess'] = 'ok' if subprocess.run(['/bin/true']).returncode == 0 else 'failed'
with open(os.path.join(sys.argv[1], 'written'), 'w') as f: f.write('x')
res['write'] = 'ok'
print(json.dumps(res))
"#;

fn pool() -> Option<WorkloadIdentityPool> {
    let missing = if !Path::new(PYTHON).exists() {
        Some(format!("{PYTHON} unavailable for the syscall probe"))
    } else {
        WorkloadIdentityPool::from_host()
            .err()
            .map(|e| e.to_string())
    };
    match missing {
        None => Some(WorkloadIdentityPool::from_host().unwrap()),
        Some(error) => {
            assert!(
                std::env::var_os("SEA_FORGE_REQUIRE_WORKLOAD_ISOLATION").is_none(),
                "workload isolation required but unavailable: {error}"
            );
            eprintln!("NOT-EXECUTABLE: {error}");
            None
        }
    }
}

fn confinement(base: &Path, identity: &WorkloadIdentity) -> WorkloadConfinement {
    let root = base.join(format!("w{}", identity.slot));
    let confinement = WorkloadConfinement {
        workspace_root: root.join("workspace"),
        artifacts_root: root.join("artifacts"),
        read_paths: vec![],
        network: NetworkPosture::Denied,
    };
    fs::create_dir_all(&confinement.workspace_root).unwrap();
    fs::create_dir_all(&confinement.artifacts_root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o711)).unwrap();
    fs::set_permissions(base, fs::Permissions::from_mode(0o711)).unwrap();
    grant_exclusive_access(&confinement.workspace_root, identity).unwrap();
    grant_exclusive_access(&confinement.artifacts_root, identity).unwrap();
    confinement
}

fn parse(out: std::process::Output) -> Value {
    assert!(
        out.status.success(),
        "probe failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}

/// The probe inside the full isolated launch (helper, Landlock, filter).
fn isolated_probe(identity: &WorkloadIdentity, confinement: &WorkloadConfinement) -> Value {
    let workspace = confinement.workspace_root.display().to_string();
    let (mut command, _lifeline) = isolated_command(
        identity,
        Path::new(HELPER),
        confinement,
        &[
            PYTHON.into(),
            "-I".into(),
            "-c".into(),
            PROBE.into(),
            workspace,
        ],
    )
    .unwrap();
    parse(
        command
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .output()
            .unwrap(),
    )
}

/// Mutation control: the same probe as the same kind of identity, without
/// the helper (no Landlock, no filter).
fn unfiltered_probe(identity: &WorkloadIdentity, workspace: &Path) -> Value {
    parse(
        Command::new("/usr/bin/unshare")
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .arg("--user")
            .arg(format!(
                "--map-users={WORKLOAD_INNER_ID}:{}:1",
                identity.host_uid
            ))
            .arg(format!(
                "--map-groups={WORKLOAD_INNER_ID}:{}:1",
                identity.host_gid
            ))
            .arg(format!("--setuid={WORKLOAD_INNER_ID}"))
            .arg(format!("--setgid={WORKLOAD_INNER_ID}"))
            .args(["--", PYTHON, "-I", "-c", PROBE])
            .arg(workspace)
            .stdin(Stdio::null())
            .output()
            .unwrap(),
    )
}

const REFUSED: [(&str, &str); 10] = [
    ("io_uring_setup", "EPERM"),
    ("io_uring_enter", "EPERM"),
    ("io_uring_register", "EPERM"),
    ("unshare_user", "EPERM"),
    ("unshare_user_net", "EPERM"),
    ("unshare_user_mount", "EPERM"),
    ("clone_newuser", "EPERM"),
    ("clone3_newuser", "ENOSYS"),
    ("setns", "EPERM"),
    ("nested_mount", "unshare:EPERM"),
];
/// x32 aliases: refused when the kernel offers the ABI at all.
const X32: [&str; 2] = ["x32_io_uring_setup", "x32_unshare_user"];
const PERMITTED: [&str; 4] = ["fork", "thread", "subprocess", "write"];

#[test]
fn isolated_workloads_cannot_reach_io_uring_or_create_namespaces() {
    let Some(pool) = pool() else { return };
    let base = tempfile::tempdir().unwrap();
    let identity = pool.acquire().unwrap();
    let confinement = confinement(base.path(), &identity);

    let control = unfiltered_probe(&identity, &confinement.workspace_root);
    eprintln!("unfiltered: {control}");
    // The attack is live on this host: every refused operation succeeds (or
    // reaches the kernel past the filter, failing on its bad descriptor or opcode).
    for (name, _) in REFUSED {
        let expected = match name {
            "io_uring_enter" | "setns" => "EBADF",
            "io_uring_register" => "EINVAL",
            _ => "ok",
        };
        assert_eq!(control[name], expected, "control {name}: {control}");
    }

    let seen = isolated_probe(&identity, &confinement);
    eprintln!("isolated: {seen}");
    for (name, errno) in REFUSED {
        assert_eq!(seen[name], errno, "{name}: {seen}");
    }
    for name in PERMITTED {
        assert_eq!(seen[name], "ok", "{name}: {seen}");
    }
    for name in X32 {
        match control[name].as_str() {
            Some("ok") => assert_eq!(seen[name], "EPERM", "{name}: {seen}"),
            // No x32 ABI in this kernel: nothing to refuse; report it.
            Some("ENOSYS") => eprintln!("NOT-EXECUTABLE: {name}: kernel has no x32 ABI"),
            other => panic!("control {name}: {other:?}"),
        }
    }
    assert_eq!(
        fs::read_to_string(confinement.workspace_root.join("written")).unwrap(),
        "x"
    );
}
