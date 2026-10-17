//! Per-workload host identity for mutually untrusted agents (SEC-01/SEC-06).
//!
//! Each isolated workload runs under its own subordinate host UID/GID taken
//! from the launching user's `/etc/subuid` and `/etc/subgid` delegation. The
//! launcher maps exactly one subordinate ID into a fresh user namespace with
//! the distribution's setuid `newuidmap`/`newgidmap` (via util-linux
//! `unshare`), so the workload:
//!
//! - cannot read another workload's `/proc/<pid>/fd`, environ or memory, or
//!   signal it (distinct host UID, Yama ptrace scope);
//! - cannot join another workload's user namespace (owned by the launcher);
//! - cannot remap itself to another subordinate ID (the subordinate UID has no
//!   delegation of its own);
//! - keeps no supplementary host groups and no capabilities;
//! - reaches launcher-owned files only through ordinary DAC, plus the
//!   inherited Landlock posture applied by `sea-forge-workload-exec`.
//!
//! The namespace owner (the launcher's UID) can still enter the namespace;
//! the launcher is the trusted control plane, not a workload. Setup fails
//! closed when the delegation, mapping helpers, or exclusive slot cannot be
//! established — there is no fallback to a shared-UID launch.
use crate::{NetworkPosture, SandboxError};
use std::fs::{self, File, OpenOptions};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;

const UNAVAILABLE: &str = "workload_isolation_unavailable";
/// Inner UID/GID the workload sees; mapped to its subordinate host ID only.
pub const WORKLOAD_INNER_ID: u32 = 1000;
const UNSHARE: &str = "/usr/bin/unshare";
const MAPPERS: [&str; 2] = ["/usr/bin/newuidmap", "/usr/bin/newgidmap"];

fn unavailable(message: impl Into<String>) -> SandboxError {
    SandboxError::new(UNAVAILABLE, message)
}

/// The launching user's subordinate ID delegation and slot lock directory.
#[derive(Debug, Clone)]
pub struct WorkloadIdentityPool {
    uid_base: u32,
    gid_base: u32,
    count: u32,
    slots: PathBuf,
}

/// One exclusively held subordinate identity. The slot stays reserved for as
/// long as this value (and its lock) is alive.
#[derive(Debug)]
pub struct WorkloadIdentity {
    pub host_uid: u32,
    pub host_gid: u32,
    pub slot: u32,
    _lock: File,
}

fn delegation(file: &str, user: &str, id: u32) -> Result<(u32, u32), SandboxError> {
    let text = fs::read_to_string(file).map_err(|e| unavailable(format!("{file}: {e}")))?;
    text.lines()
        .filter_map(|line| {
            let mut fields = line.trim().split(':');
            let (owner, base, count) = (fields.next()?, fields.next()?, fields.next()?);
            (owner == user || owner == id.to_string())
                .then(|| Some((base.parse().ok()?, count.parse().ok()?)))
                .flatten()
        })
        .find(|(base, count)| *count > 1 && *base > 0)
        .ok_or_else(|| unavailable(format!("{file} has no delegation for {user}")))
}

/// Login name and home directory of `uid` from `/etc/passwd`.
fn passwd_entry(uid: u32) -> Result<(String, PathBuf), SandboxError> {
    let passwd = fs::read_to_string("/etc/passwd").map_err(|e| unavailable(e.to_string()))?;
    passwd
        .lines()
        .find_map(|line| {
            let fields: Vec<&str> = line.split(':').collect();
            (fields.len() > 5 && fields[2] == uid.to_string())
                .then(|| (fields[0].to_string(), PathBuf::from(fields[5])))
        })
        .ok_or_else(|| unavailable(format!("no passwd entry for uid {uid}")))
}

/// One slot-lock directory per launching user and host, whatever control
/// plane root launches the workload: the subordinate delegation is per user,
/// so every launcher drawing on it must contend on the same locks.
fn host_slot_dir(uid: u32, home: &Path) -> PathBuf {
    let runtime = PathBuf::from(format!("/run/user/{uid}"));
    let private = fs::metadata(&runtime)
        .is_ok_and(|meta| meta.is_dir() && meta.uid() == uid && meta.mode() & 0o077 == 0);
    if private {
        runtime.join("sea-forge").join("workload-slots")
    } else {
        home.join(".local/state/sea-forge/workload-slots")
    }
}

fn current_ids() -> Result<(u32, u32), SandboxError> {
    let meta = fs::metadata("/proc/self").map_err(|e| unavailable(e.to_string()))?;
    Ok((meta.uid(), meta.gid()))
}

impl WorkloadIdentityPool {
    /// Discover the delegation for the current user. Slot locks live in one
    /// per-user host directory (mode 0700) shared by every launcher, so two
    /// control planes can never hold the same subordinate identity.
    pub fn from_host() -> Result<Self, SandboxError> {
        for helper in [UNSHARE].iter().chain(MAPPERS.iter()) {
            let meta = fs::metadata(helper).map_err(|e| unavailable(format!("{helper}: {e}")))?;
            if MAPPERS.contains(helper) && (meta.uid() != 0 || meta.mode() & 0o4000 == 0) {
                return Err(unavailable(format!("{helper} is not a setuid-root mapper")));
            }
        }
        let (uid, _) = current_ids()?;
        if uid == 0 {
            return Err(unavailable("refusing to launch workloads from host root"));
        }
        let (user, home) = passwd_entry(uid)?;
        let (uid_base, uid_count) = delegation("/etc/subuid", &user, uid)?;
        let (gid_base, gid_count) = delegation("/etc/subgid", &user, uid)?;
        let slots = host_slot_dir(uid, &home);
        fs::create_dir_all(&slots).map_err(|e| unavailable(e.to_string()))?;
        for dir in [slots.parent().unwrap_or(&slots), slots.as_path()] {
            fs::set_permissions(dir, fs::Permissions::from_mode(0o700))
                .map_err(|e| unavailable(e.to_string()))?;
        }
        Ok(Self {
            uid_base,
            gid_base,
            count: uid_count.min(gid_count),
            slots,
        })
    }

    /// Reserve the first free subordinate identity. Slot 0 (the base ID) is
    /// never issued so a workload never collides with namespace-root mappings
    /// that other tools create from the same delegation.
    pub fn acquire(&self) -> Result<WorkloadIdentity, SandboxError> {
        for slot in 1..self.count {
            let lock = OpenOptions::new()
                .create(true)
                .truncate(false)
                .write(true)
                .open(self.slots.join(format!("{slot}.lock")))
                .map_err(|e| unavailable(e.to_string()))?;
            // A free lock is not enough: a workload that outlived its
            // launcher still runs under the slot's UID. Reap it as the
            // namespace owner, and skip the slot if anything survives.
            if lock.try_lock().is_ok() && reap_identity(self.uid_base + slot) {
                return Ok(WorkloadIdentity {
                    host_uid: self.uid_base + slot,
                    host_gid: self.gid_base + slot,
                    slot,
                    _lock: lock,
                });
            }
        }
        Err(unavailable("every subordinate workload identity is in use"))
    }
}

/// Host PIDs of processes holding `uid` as any of their real, effective,
/// saved or filesystem UIDs.
pub fn processes_of(uid: u32) -> Vec<i32> {
    let Ok(entries) = fs::read_dir("/proc") else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| entry.file_name().to_str()?.parse::<i32>().ok())
        .filter(|pid| {
            fs::read_to_string(format!("/proc/{pid}/status")).is_ok_and(|status| {
                status
                    .lines()
                    .find(|line| line.starts_with("Uid:"))
                    .is_some_and(|line| {
                        line.split_whitespace()
                            .skip(1)
                            .any(|id| id == uid.to_string())
                    })
            })
        })
        .collect()
}

/// Kill every process under `uid` and report whether none remain. The
/// launcher owns each workload's user namespace, so it may signal them.
fn reap_identity(uid: u32) -> bool {
    use rustix::process::{kill_process, Pid, Signal};
    for _ in 0..50 {
        let live = processes_of(uid);
        if live.is_empty() {
            return true;
        }
        for pid in live {
            if let Some(pid) = Pid::from_raw(pid) {
                let _ = kill_process(pid, Signal::KILL);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    processes_of(uid).is_empty()
}

/// End a workload's DAC reach into a tree granted by
/// [`grant_exclusive_access`]: mode 0700 sets the ACL mask to none, so the
/// named workload entry no longer grants anything while the launcher (owner)
/// keeps full access. Call once the workload has exited.
pub fn revoke_exclusive_access(root: &Path) -> Result<(), SandboxError> {
    fs::set_permissions(root, fs::Permissions::from_mode(0o700))
        .map_err(|e| unavailable(format!("revoke {}: {e}", root.display())))
}

// POSIX ACL xattr encoding (linux/posix_acl_xattr.h).
const ACL_USER_OBJ: u16 = 0x01;
const ACL_USER: u16 = 0x02;
const ACL_GROUP_OBJ: u16 = 0x04;
const ACL_MASK: u16 = 0x10;
const ACL_OTHER: u16 = 0x20;
const ACL_UNDEFINED_ID: u32 = u32::MAX;

fn encode_acl(named_users: &[u32]) -> Vec<u8> {
    let mut users = named_users.to_vec();
    users.sort_unstable();
    users.dedup();
    let mut entries: Vec<(u16, u16, u32)> = vec![(ACL_USER_OBJ, 7, ACL_UNDEFINED_ID)];
    entries.extend(users.into_iter().map(|uid| (ACL_USER, 7, uid)));
    entries.extend([
        (ACL_GROUP_OBJ, 0, ACL_UNDEFINED_ID),
        (ACL_MASK, 7, ACL_UNDEFINED_ID),
        (ACL_OTHER, 0, ACL_UNDEFINED_ID),
    ]);
    let mut bytes = 2u32.to_le_bytes().to_vec();
    for (tag, perm, id) in entries {
        bytes.extend(tag.to_le_bytes());
        bytes.extend(perm.to_le_bytes());
        bytes.extend(id.to_le_bytes());
    }
    bytes
}

fn set_xattr(path: &Path, name: &str, value: &[u8]) -> Result<(), SandboxError> {
    rustix::fs::setxattr(path, name, value, rustix::fs::XattrFlags::empty())
        .map_err(|e| unavailable(format!("{name} on {}: {e}", path.display())))
}

/// Grant one workload identity exclusive access to a launcher-owned tree.
///
/// Group and other access are removed. Directories also get a default ACL so
/// files the workload creates stay readable by the launcher (for trusted
/// observation) and by no other workload. Symlinks are refused.
pub fn grant_exclusive_access(
    root: &Path,
    identity: &WorkloadIdentity,
) -> Result<(), SandboxError> {
    let (launcher, _) = current_ids()?;
    let access = encode_acl(&[identity.host_uid]);
    let default = encode_acl(&[identity.host_uid, launcher]);
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        let meta = fs::symlink_metadata(&path).map_err(|e| unavailable(e.to_string()))?;
        if meta.file_type().is_symlink() {
            return Err(unavailable(format!("refusing symlink {}", path.display())));
        }
        if meta.uid() != launcher {
            return Err(unavailable(format!(
                "{} is not launcher-owned",
                path.display()
            )));
        }
        set_xattr(&path, "system.posix_acl_access", &access)?;
        if meta.is_dir() {
            set_xattr(&path, "system.posix_acl_default", &default)?;
            for entry in fs::read_dir(&path).map_err(|e| unavailable(e.to_string()))? {
                pending.push(entry.map_err(|e| unavailable(e.to_string()))?.path());
            }
        }
    }
    Ok(())
}

/// What the confined workload may touch, enforced inside the namespace by the
/// trusted `sea-forge-workload-exec` helper before it execs the workload.
#[derive(Debug, Clone)]
pub struct WorkloadConfinement {
    pub workspace_root: PathBuf,
    pub artifacts_root: PathBuf,
    /// Operator-declared runtime trees (agent installation, interpreter
    /// prefix) readable in addition to the system set; see
    /// [`validate_read_paths`].
    pub read_paths: Vec<PathBuf>,
    pub network: NetworkPosture,
}

/// Check operator-declared workload read paths: each must be absolute,
/// existing and canonical (no symlink indirection), and none may be `/`, a
/// home directory itself, or contain any `protected` path (the control-plane
/// root, launcher state). Returns the paths unchanged when every check holds.
pub fn validate_read_paths(
    paths: &[PathBuf],
    protected: &[&Path],
) -> Result<Vec<PathBuf>, SandboxError> {
    let refuse = |path: &Path, why: &str| {
        Err(SandboxError::new(
            "schema_error",
            format!("workload read path {}: {why}", path.display()),
        ))
    };
    for path in paths {
        if !path.is_absolute() {
            return refuse(path, "must be absolute");
        }
        match path.canonicalize() {
            Ok(canonical) if canonical == *path => {}
            Ok(_) => return refuse(path, "must be canonical (no symlinks or dot segments)"),
            Err(error) => return refuse(path, &error.to_string()),
        }
        if path.parent().is_none() || path.parent() == Some(Path::new("/home")) {
            return refuse(path, "exposes a whole home or the root filesystem");
        }
        for guarded in protected {
            let guarded = guarded
                .canonicalize()
                .unwrap_or_else(|_| guarded.to_path_buf());
            if guarded.starts_with(path) {
                return refuse(path, "contains a protected control-plane path");
            }
        }
    }
    Ok(paths.to_vec())
}

/// The launcher's end of a workload's lifeline pipe. The workload's pid
/// namespace ends as soon as this is dropped or the launcher dies, so keep it
/// alive exactly as long as the workload may run.
#[derive(Debug)]
pub struct Lifeline {
    _write: std::os::fd::OwnedFd,
    _read: std::os::fd::OwnedFd,
}

/// Build the launch command for `argv` under `identity`, and the lifeline
/// that bounds it. The caller supplies stdio, environment (already cleared)
/// and cwd, and must hold the [`Lifeline`] until the workload has exited.
/// `helper` is the absolute path to the `sea-forge-workload-exec` binary.
pub fn isolated_command(
    identity: &WorkloadIdentity,
    helper: &Path,
    confinement: &WorkloadConfinement,
    argv: &[String],
) -> Result<(Command, Lifeline), SandboxError> {
    use std::os::fd::AsRawFd;
    if argv.is_empty() {
        return Err(SandboxError::new("input_error", "workload argv is empty"));
    }
    if !helper.is_absolute() {
        return Err(unavailable("workload helper path must be absolute"));
    }
    // Only the read end crosses exec; the write end stays close-on-exec in
    // the launcher, so its closure is the launcher's exit.
    let (read, write) = rustix::pipe::pipe_with(rustix::pipe::PipeFlags::CLOEXEC)
        .map_err(|e| unavailable(format!("lifeline pipe: {e}")))?;
    rustix::io::fcntl_setfd(&read, rustix::io::FdFlags::empty())
        .map_err(|e| unavailable(format!("lifeline pipe: {e}")))?;
    // The helper reopens the read end by path as the subordinate identity.
    // Read-only for others: the workload can never open a second write end
    // that would keep the pipe alive past the launcher.
    rustix::fs::fchmod(&read, rustix::fs::Mode::from_raw_mode(0o444))
        .map_err(|e| unavailable(format!("lifeline pipe: {e}")))?;
    let mut command = Command::new(UNSHARE);
    command
        .env_clear()
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
        .args([
            "--pid",
            "--fork",
            "--kill-child",
            "--mount",
            "--mount-proc",
            "--ipc",
        ]);
    if confinement.network == NetworkPosture::Denied {
        command.arg("--net");
    }
    command
        .arg("--")
        .arg(helper)
        .arg("--workspace")
        .arg(&confinement.workspace_root)
        .arg("--artifacts")
        .arg(&confinement.artifacts_root);
    for path in &confinement.read_paths {
        command.arg("--read").arg(path);
    }
    for port in confinement.network.granted_ports() {
        command.arg("--tcp-port").arg(port.to_string());
    }
    command
        .arg("--lifeline")
        .arg(read.as_raw_fd().to_string())
        .arg("--")
        .args(argv);
    Ok((
        command,
        Lifeline {
            _write: write,
            _read: read,
        },
    ))
}

/// Spawn an interactive isolated workload whose stdio stays connected to the
/// caller (ACP agents). `env` replaces the environment entirely.
pub fn spawn_interactive(
    identity: &WorkloadIdentity,
    helper: &Path,
    confinement: &WorkloadConfinement,
    argv: &[String],
    cwd: &Path,
    env: &[(String, String)],
) -> Result<crate::jail::InteractiveJailChild, SandboxError> {
    use std::os::unix::process::CommandExt;
    use std::process::Stdio;
    let (mut command, lifeline) = isolated_command(identity, helper, confinement, argv)?;
    command
        .current_dir(cwd)
        .envs(env.iter().map(|(k, v)| (k.as_str(), v.as_str())))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);
    let mut child = command.spawn().map_err(|e| unavailable(e.to_string()))?;
    let missing = || unavailable("workload stdio unavailable");
    let stdin = child.stdin.take().ok_or_else(missing)?;
    let stdout = child.stdout.take().ok_or_else(missing)?;
    let stderr = child.stderr.take().ok_or_else(missing)?;
    Ok(crate::jail::InteractiveJailChild {
        child,
        stdin,
        stdout,
        stderr,
        lifeline: Some(lifeline),
    })
}

/// Locate the helper next to the running executable (installed layouts and
/// `target/<profile>`), failing closed when it is absent.
pub fn sibling_helper() -> Result<PathBuf, SandboxError> {
    sibling_binary("sea-forge-workload-exec")
}

/// Locate a SEA Forge binary installed next to the running executable.
pub fn sibling_binary(name: &str) -> Result<PathBuf, SandboxError> {
    let exe = std::env::current_exe().map_err(|e| unavailable(e.to_string()))?;
    let dir = exe
        .parent()
        .ok_or_else(|| unavailable("executable has no parent"))?;
    [dir.to_path_buf(), dir.join("..")]
        .into_iter()
        .map(|d| d.join(name))
        .find(|p| p.is_file())
        .and_then(|p| p.canonicalize().ok())
        .ok_or_else(|| unavailable(format!("{name} not found")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acl_encoding_is_sorted_and_denies_group_and_other() {
        let bytes = encode_acl(&[100_002, 1000, 100_002]);
        assert_eq!(&bytes[..4], &2u32.to_le_bytes());
        let entries: Vec<(u16, u16, u32)> = bytes[4..]
            .chunks(8)
            .map(|c| {
                (
                    u16::from_le_bytes([c[0], c[1]]),
                    u16::from_le_bytes([c[2], c[3]]),
                    u32::from_le_bytes([c[4], c[5], c[6], c[7]]),
                )
            })
            .collect();
        assert_eq!(
            entries,
            vec![
                (ACL_USER_OBJ, 7, ACL_UNDEFINED_ID),
                (ACL_USER, 7, 1000),
                (ACL_USER, 7, 100_002),
                (ACL_GROUP_OBJ, 0, ACL_UNDEFINED_ID),
                (ACL_MASK, 7, ACL_UNDEFINED_ID),
                (ACL_OTHER, 0, ACL_UNDEFINED_ID),
            ]
        );
    }
}
