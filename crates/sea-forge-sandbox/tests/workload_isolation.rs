#![cfg(unix)]
//! SEC-01/SEC-06 cross-workload isolation against a real kernel boundary.
//!
//! A victim workload holds a secret open on fd 9; an attacker workload with a
//! different subordinate identity tries to steal it through procfs, signals,
//! namespace entry, re-mapping and direct file access. The regression target
//! is the 2026-10-16 probe where a same-UID process read a rootless
//! workload's open descriptor. A mutation control runs the same attacks with
//! the identity deliberately shared and requires them to succeed, so a
//! weakened launcher cannot pass silently.
//!
//! Hosts without subordinate-ID delegation report NOT-EXECUTABLE; set
//! `SEA_FORGE_REQUIRE_WORKLOAD_ISOLATION=1` to make that a failure.
#![cfg(target_os = "linux")]

use sea_forge_sandbox::workload::{
    grant_exclusive_access, isolated_command, WorkloadConfinement, WorkloadIdentity,
    WorkloadIdentityPool, WORKLOAD_INNER_ID,
};
use sea_forge_sandbox::NetworkPosture;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const HELPER: &str = env!("CARGO_BIN_EXE_sea-forge-workload-exec");

struct Fixture {
    _base: tempfile::TempDir,
    base: PathBuf,
    pool: WorkloadIdentityPool,
}

fn fixture() -> Option<Fixture> {
    let base = tempfile::tempdir().expect("tempdir");
    // Traverse-only so workloads can reach their own directories by name.
    fs::set_permissions(base.path(), fs::Permissions::from_mode(0o711)).expect("chmod base");
    match WorkloadIdentityPool::from_host() {
        Ok(pool) => Some(Fixture {
            base: base.path().to_path_buf(),
            _base: base,
            pool,
        }),
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

fn workload_dirs(base: &Path, name: &str, identity: &WorkloadIdentity) -> WorkloadConfinement {
    let root = base.join(name);
    let confinement = WorkloadConfinement {
        workspace_root: root.join("workspace"),
        artifacts_root: root.join("artifacts"),
        read_paths: Vec::new(),
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

fn spawn(identity: &WorkloadIdentity, confinement: &WorkloadConfinement, script: &str) -> Child {
    let (mut command, lifeline) = isolated_command(
        identity,
        Path::new(HELPER),
        confinement,
        &["/bin/sh".into(), "-c".into(), script.into()],
    )
    .unwrap();
    // These attacks are about identity, not lifetime: keep each workload's
    // lifeline open for the test process (workload_boundary.rs covers it).
    std::mem::forget(lifeline);
    command
        .env("PATH", "/usr/bin:/bin")
        .stdin(Stdio::null())
        .spawn()
        .expect("spawn isolated workload")
}

fn wait_for(path: &Path) -> String {
    let start = Instant::now();
    loop {
        if let Ok(text) = fs::read_to_string(path) {
            if text.ends_with("done\n") || text.ends_with("ready\n") {
                return text;
            }
        }
        assert!(
            start.elapsed() < Duration::from_secs(20),
            "timed out on {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn host_pid_of(uid: u32, comm: &str) -> u32 {
    for entry in fs::read_dir("/proc").unwrap().flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        let Ok(meta) = fs::metadata(entry.path()) else {
            continue;
        };
        let name = fs::read_to_string(entry.path().join("comm")).unwrap_or_default();
        if meta.uid() == uid && name.trim() == comm {
            return pid;
        }
    }
    panic!("no host process for uid {uid} ({comm})");
}

/// Victim: secret file in its own workspace, held open on fd 9.
fn start_victim(fx: &Fixture, victim: &WorkloadIdentity) -> (Child, PathBuf, u32) {
    let dirs = workload_dirs(&fx.base, "victim", victim);
    let ws = dirs.workspace_root.display().to_string();
    let script = format!(
        "echo SECRET-VICTIM > {ws}/secret && exec 9<{ws}/secret && \
         if cat /proc/self/fd/9 >/dev/null; then echo selfread:ok; fi > {ws}/status && \
         echo ready >> {ws}/status && exec sleep 30"
    );
    let child = spawn(victim, &dirs, &script);
    let status = wait_for(&dirs.workspace_root.join("status"));
    assert!(
        status.contains("selfread:ok"),
        "victim positive control: {status}"
    );
    let pid = host_pid_of(victim.host_uid, "sleep");
    (child, dirs.workspace_root.join("secret"), pid)
}

fn attack_script(
    out: &Path,
    victim_pid: u32,
    victim_uid: u32,
    secret: &Path,
    host_secret: &Path,
) -> String {
    let out = out.display();
    format!(
        "r() {{ n=$1; shift; if \"$@\" >/dev/null 2>&1; then echo \"$n:VULNERABLE\"; else echo \"$n:denied\"; fi; }}
         {{
         r proc_fd cat /proc/{victim_pid}/fd/9
         r proc_fd_list ls /proc/{victim_pid}/fd
         r proc_environ cat /proc/{victim_pid}/environ
         r proc_mem dd if=/proc/{victim_pid}/mem bs=1 count=1
         r signal kill -0 {victim_pid}
         r nsenter_victim nsenter -U -t {victim_pid} true
         r remap_victim unshare --user --map-users=0:{victim_uid}:1 true
         r remap_auto unshare --user --map-auto true
         r victim_file cat {secret}
         r launcher_file cat {host_secret}
         echo own_write > {out}.own && echo own_write:ok
         echo done
         }} > {out}",
        secret = secret.display(),
        host_secret = host_secret.display(),
    )
}

fn results(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn host_secret(fx: &Fixture) -> PathBuf {
    let path = fx.base.join("launcher.secret");
    fs::write(&path, "SECRET-LAUNCHER").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    path
}

/// Attacker sharing the uid boundary only (no pid/mount namespace), so the
/// host-UID separation itself is exercised against the victim's real pid.
fn uid_only_attacker(identity_uid: u32, identity_gid: u32, script: &str) {
    let status = Command::new("/usr/bin/unshare")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .arg("--user")
        .arg(format!("--map-users={WORKLOAD_INNER_ID}:{identity_uid}:1"))
        .arg(format!("--map-groups={WORKLOAD_INNER_ID}:{identity_gid}:1"))
        .arg(format!("--setuid={WORKLOAD_INNER_ID}"))
        .arg(format!("--setgid={WORKLOAD_INNER_ID}"))
        .args(["--", "/bin/sh", "-c", script])
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn distinct_workload_identities_cannot_reach_each_other() {
    let Some(fx) = fixture() else { return };
    let victim = fx.pool.acquire().unwrap();
    let attacker = fx.pool.acquire().unwrap();
    assert_ne!(victim.host_uid, attacker.host_uid);
    let launcher_secret = host_secret(&fx);
    let (mut victim_child, secret, victim_pid) = start_victim(&fx, &victim);
    // Trusted observation positive control: the launcher reads workload output.
    assert_eq!(fs::read_to_string(&secret).unwrap().trim(), "SECRET-VICTIM");

    let dirs = workload_dirs(&fx.base, "attacker", &attacker);
    let out = dirs.workspace_root.join("uid-only");
    uid_only_attacker(
        attacker.host_uid,
        attacker.host_gid,
        &attack_script(&out, victim_pid, victim.host_uid, &secret, &launcher_secret),
    );
    let uid_only = results(&wait_for(&out));

    let out = dirs.workspace_root.join("full");
    let mut child = spawn(
        &attacker,
        &dirs,
        &attack_script(&out, victim_pid, victim.host_uid, &secret, &launcher_secret),
    );
    let full = results(&wait_for(&out));
    child.wait().unwrap();
    victim_child.kill().ok();
    victim_child.wait().ok();

    for (layer, rows) in [("uid-only", &uid_only), ("full", &full)] {
        assert_eq!(rows.len(), 11, "{layer}: {rows:?}");
        for (name, verdict) in rows.iter() {
            let expected = if name == "own_write" { "ok" } else { "denied" };
            assert_eq!(verdict, expected, "{layer}: {name}");
        }
    }
}

#[test]
fn shared_identity_mutation_is_detected() {
    let Some(fx) = fixture() else { return };
    let victim = fx.pool.acquire().unwrap();
    let launcher_secret = host_secret(&fx);
    let (mut victim_child, secret, victim_pid) = start_victim(&fx, &victim);
    // Deliberately weakened launcher: the attacker reuses the victim identity.
    let out = fx.base.join("victim/workspace/mutation");
    uid_only_attacker(
        victim.host_uid,
        victim.host_gid,
        &attack_script(&out, victim_pid, victim.host_uid, &secret, &launcher_secret),
    );
    let rows = results(&wait_for(&out));
    victim_child.kill().ok();
    victim_child.wait().ok();
    let verdict = |n: &str| rows.iter().find(|(k, _)| k == n).map(|(_, v)| v.as_str());
    // Observed on Linux 6.18: a shared host UID exposes the victim's fd
    // table, signals and files. Descriptor *contents* stay behind the
    // victim's separate user namespace, which is defence in depth, not
    // identity isolation, so it is not what this control relies on.
    assert_eq!(verdict("proc_fd_list"), Some("VULNERABLE"), "{rows:?}");
    assert_eq!(verdict("victim_file"), Some("VULNERABLE"), "{rows:?}");
    assert_eq!(verdict("signal"), Some("VULNERABLE"), "{rows:?}");
    // The launcher's own files stay out of reach even then.
    assert_eq!(verdict("launcher_file"), Some("denied"), "{rows:?}");
}

#[test]
fn slots_are_exclusive_while_held() {
    let Some(fx) = fixture() else { return };
    let first = fx.pool.acquire().unwrap();
    let second = fx.pool.acquire().unwrap();
    assert_ne!(first.slot, second.slot);
    let slot = first.slot;
    drop(first);
    // Slots are shared host-wide, so a concurrent test may hold the released
    // slot for a moment; it must become acquirable again, never `second`'s.
    let start = Instant::now();
    let mut lower = Vec::new();
    loop {
        let next = fx.pool.acquire().unwrap();
        assert_ne!(next.slot, second.slot, "held slot issued twice");
        if next.slot == slot {
            break;
        }
        // Lowest free slot first: keep lower ones so the released one is next.
        if next.slot < slot {
            lower.push(next);
            continue;
        }
        drop(next);
        assert!(
            start.elapsed() < Duration::from_secs(20),
            "released slot never reissued"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}
