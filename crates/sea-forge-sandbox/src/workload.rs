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

fn login_name(uid: u32) -> Result<String, SandboxError> {
    let passwd = fs::read_to_string("/etc/passwd").map_err(|e| unavailable(e.to_string()))?;
    passwd
        .lines()
        .find_map(|line| {
            let fields: Vec<&str> = line.split(':').collect();
            (fields.len() > 2 && fields[2] == uid.to_string()).then(|| fields[0].to_string())
        })
        .ok_or_else(|| unavailable(format!("no passwd entry for uid {uid}")))
}

fn current_ids() -> Result<(u32, u32), SandboxError> {
    let meta = fs::metadata("/proc/self").map_err(|e| unavailable(e.to_string()))?;
    Ok((meta.uid(), meta.gid()))
}

impl WorkloadIdentityPool {
    /// Discover the delegation for the current user. `state_dir` must be a
    /// launcher-private directory; it is created with mode 0700.
    pub fn from_host(state_dir: &Path) -> Result<Self, SandboxError> {
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
        let user = login_name(uid)?;
        let (uid_base, uid_count) = delegation("/etc/subuid", &user, uid)?;
        let (gid_base, gid_count) = delegation("/etc/subgid", &user, uid)?;
        let slots = state_dir.join("workload-slots");
        fs::create_dir_all(&slots).map_err(|e| unavailable(e.to_string()))?;
        for dir in [state_dir, slots.as_path()] {
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
            if lock.try_lock().is_ok() {
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
    pub network: NetworkPosture,
}

/// Build the launch command for `argv` under `identity`. The caller supplies
/// stdio, environment (already cleared) and cwd. `helper` is the absolute
/// path to the `sea-forge-workload-exec` binary.
pub fn isolated_command(
    identity: &WorkloadIdentity,
    helper: &Path,
    confinement: &WorkloadConfinement,
    argv: &[String],
) -> Result<Command, SandboxError> {
    if argv.is_empty() {
        return Err(SandboxError::new("input_error", "workload argv is empty"));
    }
    if !helper.is_absolute() {
        return Err(unavailable("workload helper path must be absolute"));
    }
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
    for port in confinement.network.granted_ports() {
        command.arg("--tcp-port").arg(port.to_string());
    }
    command.arg("--").args(argv);
    Ok(command)
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
    let mut command = isolated_command(identity, helper, confinement, argv)?;
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
    })
}

/// Locate the helper next to the running executable (installed layouts and
/// `target/<profile>`), failing closed when it is absent.
pub fn sibling_helper() -> Result<PathBuf, SandboxError> {
    let exe = std::env::current_exe().map_err(|e| unavailable(e.to_string()))?;
    let dir = exe
        .parent()
        .ok_or_else(|| unavailable("executable has no parent"))?;
    [dir.to_path_buf(), dir.join("..")]
        .into_iter()
        .map(|d| d.join("sea-forge-workload-exec"))
        .find(|p| p.is_file())
        .and_then(|p| p.canonicalize().ok())
        .ok_or_else(|| unavailable("sea-forge-workload-exec helper not found"))
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
