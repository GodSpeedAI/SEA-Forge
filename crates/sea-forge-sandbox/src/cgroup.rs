//! F08 lifecycle containment: one Linux cgroup-v2 leaf per spawn.
//!
//! A process group cannot reach a descendant that `setsid(2)`s itself into a
//! new session, so a workload's detached grandchildren survive both the
//! completion-path and the timeout-path group kill (the F08 finding). cgroup
//! membership is the containment that closes this: it is inherited across
//! fork and independent of session, so every descendant — detached or not —
//! stays inside the leaf.
//!
//! Mechanism (adapted from ProcessKit's cgroup-v2 support, MIT):
//!
//! - **Leaf per spawn**: the run gets `sea-forge-<owner pid>-<seq>` under the
//!   delegated root. A single `mkdir` is the delegation gate — an
//!   EACCES/EPERM there means the root is not delegated to this process.
//!   The gate is followed by a join-authorization check: kernel delegation
//!   containment (cgroup-v2 §2-5-2) authorizes a `cgroup.procs` write only
//!   when the writer can write the common ancestor's `cgroup.procs`, so a
//!   root across that boundary from the spawning process reports
//!   [`CGroupContainment::Unavailable`] instead of failing every spawn
//!   after leaf creation.
//! - **Pre-exec join, no escape window**: the parent pre-opens the leaf's
//!   `cgroup.procs` and hands a descriptor-owning closure to
//!   [`std::os::unix::process::CommandExt::pre_exec`]. The child writes its
//!   own pid into the leaf between fork and exec, so it cannot run any
//!   workload-controlled code before it is contained.
//! - **Kill**: `cgroup.kill` (kernel >= 5.14) is the primary path — atomic,
//!   reaching every descendant regardless of session. Two triggers fall back
//!   to the pidfd sweep: the control file missing (kernel < 5.14), and its
//!   write failing for any other reason — delegation containment applies to
//!   `cgroup.kill` beyond what the delegation of the directory and
//!   `cgroup.procs` grants (observed on the CI runner: the pre-exec join and
//!   teardown both authorized, but the `cgroup.kill` write returned EACCES),
//!   and the mechanism that drains the leaf is an implementation detail. The
//!   sweep lists `cgroup.procs` (which delegation does keep readable),
//!   pinning each pid with `pidfd_open` and re-confirming membership after
//!   pinning before `pidfd_send_signal` — a pid that was recycled between
//!   reads is no longer a member and is never signalled. A raw `kill(2)` on
//!   a bare pid is refused by construction.
//! - **Verified teardown**: emptiness is read back from `cgroup.procs`
//!   (bounded), and `rmdir` succeeds only for an empty cgroup — the kernel
//!   enforces the check. An empty leaf whose `rmdir` is nonetheless denied
//!   (EACCES/EPERM/EBUSY) after the bounded retries is a typed degrade, not
//!   an error: containment succeeded and removal is hygiene, so the removal
//!   refusal is printed on stderr, the leaf is left in place under its
//!   owner-prefixed name — which is its registration for
//!   [`destroy_process_leaves`]: the prefix scan re-adopts it and
//!   re-attempts the removal under the same degrade rule — and the teardown
//!   succeeds. A leaf that still lists members remains an error.
//!
//!   Kernel-behavior note (round-3 CI probe, kernel 6.17-azure): the drained
//!   leaf read back empty, then `rmdir` failed EACCES on three attempts while
//!   the parent root probed as runner-owned (`mode 755 uid 1001 gid 1001`,
//!   the same uid as the deleting process) — so the VFS permission check was
//!   satisfied and the denial is cgroupfs-specific. The same probe window
//!   showed `cgroup.procs` writes and root `mkdir` succeeding while the
//!   `cgroup.kill` write was denied the same way. Candidate explanation:
//!   cgroupfs delegation propagation on this kernel denies control
//!   operations the file modes appear to grant (candidate upstream note);
//!   the contract here treats removal as degradeable hygiene exactly so the
//!   containment claim never depends on it.
//!
//! Degrade is typed, never silent: [`containment`] reports
//! [`CGroupContainment::Unavailable`] with the reason when containment cannot
//! be provided, the caller keeps the existing process-group kill, and the
//! reason is surfaced on stderr (the crate's observability channel) plus
//! returned through [`acquire_spawn_lease`]/[`attach_lease`] so execute and
//! destroy can act on it. Under `SEA_FORGE_REQUIRE_CGROUP=1` an unavailable
//! cgroup fails closed: the spawn is refused with a `cgroup_unavailable`
//! error instead of masquerading a process-group-only run as contained.
//!
//! Root resolution: `SEA_FORGE_CGROUP_ROOT` names a pre-delegated cgroup-v2
//! directory (CI wires this). Unset, the process's own cgroup is used when
//! it is writable — enabling controllers is deliberately not attempted,
//! because the auto-detected root necessarily contains this process and
//! cgroup v2 forbids enabling controllers on a cgroup with processes
//! (no-internal-process rule). Lifecycle containment needs no controllers.
#![cfg(target_os = "linux")]
// The workspace denies `unsafe_code` so that unsafe is a choice, not an
// accident. This module is that choice, in exactly one place: the pre-exec
// join registration (`CommandExt::pre_exec` has no safe wrapper). The
// callback itself is written in safe Rust — a fixed stack buffer, no
// allocation, one `write(2)` through `std` — and its safety argument is
// stated inline.
#![allow(unsafe_code)]

use crate::SandboxError;
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

/// Environment variable naming a pre-delegated cgroup-v2 directory the
/// runtime may create leaves under (wired by the CI cgroup preflight).
pub const CGROUP_ROOT_ENV: &str = "SEA_FORGE_CGROUP_ROOT";

/// When set to `1`, a run without available cgroup containment is refused
/// instead of degrading to the process-group kill (fail-closed required lane).
pub const REQUIRE_ENV: &str = "SEA_FORGE_REQUIRE_CGROUP";

/// The cgroup-v2 mount lifecycle containment resolves its default root from.
const CGROUP2_MOUNT: &str = "/sys/fs/cgroup";

/// Prefix of every leaf and probe directory this module creates, scoped by
/// owner pid so concurrent sandbox processes never collide or clean up after
/// each other.
const LEAF_PREFIX: &str = "sea-forge";

/// Sequence number disambiguating leaves of one process.
static LEAF_SEQ: AtomicU64 = AtomicU64::new(0);

/// Poll interval while waiting for a leaf to drain.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Bounded wait for killed members to leave the leaf.
const VERIFY_DEADLINE: Duration = Duration::from_secs(5);

/// `rmdir` attempts before a leaf removal is declared failed. After the last
/// member dies, the kernel releases the cgroup asynchronously — the css
/// freeing is RCU-deferred and the final unpinning runs on the cgroup
/// destroy workqueue — so an `rmdir(2)` that lands while the just-drained
/// leaf is still being released can fail transiently. The verified
/// teardown's empty read-back proves membership drained, not that the
/// release completed; a short bounded retry absorbs that release race
/// instead of failing an otherwise-clean teardown. Deliberately bounded
/// (≤ 2 backoff sleeps on the unhappy path): a genuinely undeletable leaf
/// must still surface as an error, not as a hang.
const REMOVE_ATTEMPTS: u32 = 3;

/// Sleep between [`CgroupLease::remove`] retry attempts.
const REMOVE_RETRY_BACKOFF: Duration = Duration::from_millis(10);

/// The typed availability of cgroup lifecycle containment on this host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CGroupContainment {
    /// Containment can be provided; leaves are created under `root`.
    Active {
        /// The delegated cgroup-v2 directory (env override or the process's
        /// own cgroup).
        root: PathBuf,
    },
    /// Containment cannot be provided. The reason is the honest degrade:
    /// callers keep the process-group kill and must surface it.
    Unavailable {
        /// Why containment is unavailable (probe error, not a guess).
        reason: String,
    },
}

/// Why a verified kill did not (or could not) complete.
#[derive(Debug)]
pub enum KillError {
    /// Processes still present in the leaf after the deadline.
    Survivors(Vec<libc::pid_t>),
    /// The containment machinery itself failed (never silently downgraded).
    Io(io::Error),
}

/// A run's cgroup-v2 leaf, from creation to verified removal.
#[derive(Debug)]
pub struct CgroupLease {
    leaf: PathBuf,
    /// The leaf's `cgroup.procs`, pre-opened write-only so the pre-exec join
    /// can move the child in without any path resolution after fork.
    procs: File,
}

impl CgroupLease {
    /// Create a fresh leaf under `root`. The single `mkdir` is the
    /// delegation gate; the leaf is empty and holds no controllers.
    pub fn create(root: &Path, owner_pid: u32) -> io::Result<Self> {
        let leaf = root.join(format!(
            "{LEAF_PREFIX}-{owner_pid}-{}",
            LEAF_SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        // Deliberately create_dir, not create_dir_all: exactly one mkdir at
        // the delegated root must succeed, or containment is not delegated.
        fs::create_dir(&leaf)?;
        let procs = OpenOptions::new()
            .append(true)
            .open(leaf.join("cgroup.procs"))?;
        Ok(Self { leaf, procs })
    }

    /// Re-adopt an existing leaf directory (verified teardown of a leaked
    /// leaf during [`destroy_process_leaves`]).
    fn open_existing(leaf: PathBuf) -> io::Result<Self> {
        let procs = OpenOptions::new()
            .append(true)
            .open(leaf.join("cgroup.procs"))?;
        Ok(Self { leaf, procs })
    }

    /// The leaf's path (diagnostics, tests).
    pub fn leaf(&self) -> &Path {
        &self.leaf
    }

    /// Register the pre-exec join on `command`: between fork and exec the
    /// child writes its own pid into the leaf's `cgroup.procs`, so it is
    /// contained before any workload-controlled code runs. A failed join
    /// fails the spawn (nothing execs uncontained).
    pub fn attach(command: &mut std::process::Command, lease: &CgroupLease) -> io::Result<()> {
        use std::io::Write;
        use std::os::unix::process::CommandExt;

        let mut procs = lease.procs.try_clone()?;
        let join = move || {
            // Encode getpid() into a stack buffer: the pre-exec window must
            // not allocate (async-signal-safety discipline between fork and
            // exec).
            let pid = std::process::id();
            let mut buf = [0u8; 11]; // u32::MAX is 10 digits, plus '\n'
            let mut end = buf.len();
            buf[end - 1] = b'\n';
            let mut value = pid;
            loop {
                end -= 1;
                buf[end] = b'0' + (value % 10) as u8;
                value /= 10;
                if value == 0 {
                    break;
                }
            }
            procs.write_all(&buf[end..])?;
            Ok(())
        };
        // SAFETY: the callback only writes the child's own pid into the
        // pre-opened `cgroup.procs` descriptor — one syscall, no allocation,
        // no environment access — and runs before exec, so there is no
        // uncontained window. Registration itself is infallible; a failed
        // write surfaces later as a failed spawn (nothing execs uncontained).
        unsafe { command.pre_exec(join) };
        Ok(())
    }

    /// The pids currently listed in the leaf's `cgroup.procs`.
    ///
    /// Members inside a nested PID namespace read as `0` from here and cannot
    /// be addressed by pid from this namespace, so `0` is never returned as a
    /// signal target (the primary `cgroup.kill` path still reaches them).
    pub fn procs(&self) -> io::Result<Vec<libc::pid_t>> {
        // A leaf that already vanished (concurrent verified teardown) has no
        // members by definition; ENOENT reads as empty instead of failing the
        // kill/cleanup path that raced it.
        let text = match fs::read_to_string(self.leaf.join("cgroup.procs")) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };
        Ok(text
            .split_whitespace()
            .filter_map(|token| token.parse::<libc::pid_t>().ok())
            .filter(|pid| *pid > 0)
            .collect())
    }

    /// Kill every member of the leaf and verify the leaf drains.
    ///
    /// Primary: write `1` to `cgroup.kill` (atomic, whole subtree — reaches
    /// setsid descendants). Two triggers fall back to the pidfd sweep, which
    /// is still membership-scoped and PID-reuse-safe: the control file does
    /// not exist (kernel < 5.14), and the write fails for any other reason —
    /// delegation containment can deny the kill file even where the leaf and
    /// its `cgroup.procs` are delegated (the round-2 CI runner), which is a
    /// degrade of the kill *mechanism*, never of the containment contract.
    /// The sweep is announced on stderr; either path must leave the leaf
    /// reading back empty within `deadline` or the kill fails.
    pub fn kill_verified(&self, deadline: Duration) -> Result<(), KillError> {
        match fs::write(self.leaf.join("cgroup.kill"), b"1") {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => self.pidfd_sweep(deadline)?,
            Err(e) => {
                // Kernel >= 5.14 but the write is refused (EACCES/EPERM from
                // delegation containment, or any other failure): degrade the
                // mechanism, not the contract — the pidfd sweep below is
                // membership-scoped and PID-reuse-safe, and only a leaf that
                // is still non-empty after the deadline is an error.
                eprintln!(
                    "sea-forge-sandbox: cgroup.kill write on leaf {} failed ({e}); \
                     draining via the membership-scoped pidfd sweep instead",
                    self.leaf.display()
                );
                self.pidfd_sweep(deadline)?
            }
        }
        let start = Instant::now();
        loop {
            let procs = self.procs().map_err(KillError::Io)?;
            if procs.is_empty() {
                return Ok(());
            }
            if start.elapsed() >= deadline {
                return Err(KillError::Survivors(procs));
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    }

    /// Kill the leaf's members one pid at a time when `cgroup.kill` is
    /// unavailable. Each pid is pinned with `pidfd_open` first and its
    /// membership re-confirmed after pinning: a pid that exited and was
    /// recycled between the listing and the signal is no longer a member, so
    /// the recycled (unrelated) process is never signalled.
    fn pidfd_sweep(&self, deadline: Duration) -> Result<(), KillError> {
        use rustix::process::{pidfd_open, pidfd_send_signal, Pid, PidfdFlags, Signal};

        let start = Instant::now();
        loop {
            let procs = self.procs().map_err(KillError::Io)?;
            if procs.is_empty() {
                return Ok(());
            }
            // Pin every current member before signalling anything.
            let mut pinned = Vec::with_capacity(procs.len());
            for pid in &procs {
                let Some(rustix_pid) = Pid::from_raw(*pid) else {
                    // Unreachable for members this module lists (procs()
                    // filters 0), but never open pid 0 by accident.
                    continue;
                };
                match pidfd_open(rustix_pid, PidfdFlags::empty()) {
                    Ok(fd) => pinned.push((*pid, fd)),
                    // ESRCH: the member already exited — nothing to kill.
                    Err(e) if e == rustix::io::Errno::SRCH => {}
                    // e.g. ENOSYS on kernels without pidfd (pre-5.3): the
                    // remaining members cannot be addressed safely at all,
                    // so this fails loudly instead of guessing.
                    Err(e) => return Err(KillError::Io(io::Error::from(e))),
                }
            }
            // Membership re-confirmed AFTER pinning: this is the PID-reuse
            // guard. A pinned pid absent from the re-read either exited or
            // was recycled; in both cases the pidfd must not be signalled.
            let members: HashSet<libc::pid_t> =
                self.procs().map_err(KillError::Io)?.into_iter().collect();
            for (pid, fd) in &pinned {
                if members.contains(pid) {
                    match pidfd_send_signal(fd, Signal::KILL) {
                        Ok(()) => {}
                        // The pinned member died between the re-read and the
                        // signal — the desired outcome, just early.
                        Err(e) if e == rustix::io::Errno::SRCH => {}
                        Err(e) if e == rustix::io::Errno::NOENT => {}
                        Err(e) => return Err(KillError::Io(io::Error::from(e))),
                    }
                }
            }
            if start.elapsed() >= deadline {
                return Err(KillError::Survivors(self.procs().map_err(KillError::Io)?));
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    }

    /// Remove the leaf. `rmdir` succeeds only for an empty cgroup — the
    /// kernel enforces the emptiness this module's verified teardown relies
    /// on; a refusal names the remaining members, and — because the failing
    /// syscall is the rmdir itself and its permission check runs against the
    /// parent — the delegated parent's mode and owner as probe output, so
    /// host-side evidence names the denied operation instead of guessing it.
    ///
    /// Degrade: a leaf verified EMPTY by the read-back below whose `rmdir`
    /// still fails EACCES/EPERM/EBUSY after the bounded retries is reported
    /// on stderr and recorded for [`destroy_process_leaves`] (it stays in
    /// place under its owner-prefixed name, so the destroy sweep's scan
    /// re-adopts it and re-attempts the removal under this same rule), and
    /// this returns `Ok(())` — containment already succeeded, removal is
    /// hygiene, and an undeletable empty leaf must never fail a settled run.
    /// A non-empty leaf, or any other error, remains an error.
    pub fn remove(&self) -> io::Result<()> {
        let mut last_err: Option<io::Error> = None;
        for attempt in 0..REMOVE_ATTEMPTS {
            match fs::remove_dir(&self.leaf) {
                Ok(()) => {
                    unregister_leaf(&self.leaf);
                    return Ok(());
                }
                // Already removed by a concurrent verified teardown of the
                // same lease: cleanup is idempotent.
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    unregister_leaf(&self.leaf);
                    return Ok(());
                }
                Err(e) => {
                    last_err = Some(e);
                    if attempt + 1 < REMOVE_ATTEMPTS {
                        std::thread::sleep(REMOVE_RETRY_BACKOFF);
                    }
                }
            }
        }
        let err = last_err.expect("at least one rmdir attempt ran");
        let kind = err.kind();
        let members = self.procs().unwrap_or_default();
        if members.is_empty() && undeletable_denial(&err) {
            eprintln!(
                "sea-forge-sandbox: cgroup leaf {} is empty but rmdir was denied ({err}); \
                 recorded for the destroy sweep",
                self.leaf.display()
            );
            return Ok(());
        }
        Err(io::Error::new(
            kind,
            format!(
                "cgroup leaf {}: rmdir failed after {REMOVE_ATTEMPTS} attempts: {err}; \
                 remaining members: {members:?}; {}",
                self.leaf.display(),
                parent_probe(&self.leaf),
            ),
        ))
    }
}

/// Whether a failed `rmdir` is the empty-but-undeletable cgroupfs refusal the
/// teardown degrades on: permission-class (EACCES/EPERM) or busy-class
/// (EBUSY) denials of the removal syscall itself, observed on kernel
/// 6.17-azure where cgroupfs denies the rmdir of a drained, runner-owned
/// leaf while procs writes and mkdir work (see the module note). Any other
/// error — or a leaf that still lists members — stays an error.
fn undeletable_denial(err: &io::Error) -> bool {
    matches!(
        err.raw_os_error(),
        Some(libc::EACCES) | Some(libc::EPERM) | Some(libc::EBUSY)
    )
}

/// Metadata probe of `leaf`'s parent (the delegated root) for error text: an
/// `rmdir` permission failure is checked against the parent, so its mode and
/// owner are the first facts CI evidence needs. A probe failure is reported
/// honestly instead of guessed away.
fn parent_probe(leaf: &Path) -> String {
    let Some(parent) = leaf.parent() else {
        return "leaf has no parent directory".to_owned();
    };
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    match fs::metadata(parent) {
        Ok(meta) => format!(
            "parent dir {}: mode {:o} uid {} gid {}",
            parent.display(),
            meta.permissions().mode() & 0o7777,
            meta.uid(),
            meta.gid(),
        ),
        Err(e) => format!("parent dir {} could not be probed: {e}", parent.display()),
    }
}

/// Resolve lifecycle containment for the current host.
pub fn containment() -> CGroupContainment {
    match std::env::var(CGROUP_ROOT_ENV) {
        Ok(root) if root.is_empty() => CGroupContainment::Unavailable {
            reason: format!("{CGROUP_ROOT_ENV} is set but empty"),
        },
        Ok(root) => probe_root(PathBuf::from(root)),
        Err(_) => match self_cgroup_path() {
            Some(path) => probe_root(Path::new(CGROUP2_MOUNT).join(&path)),
            None => CGroupContainment::Unavailable {
                reason: "no cgroup-v2 membership line (0::) in /proc/self/cgroup".to_owned(),
            },
        },
    }
}

/// Whether a `SEA_FORGE_REQUIRE_CGROUP=1` required lane is in effect.
fn require_requested() -> bool {
    std::env::var(REQUIRE_ENV).as_deref() == Ok("1")
}

/// Check `root` is a real cgroup-v2 mount delegated to this process. A
/// writable non-cgroup directory would fake containment, so the filesystem
/// type is verified before the write probe.
fn probe_root(root: PathBuf) -> CGroupContainment {
    match rustix::fs::statfs(&root) {
        Ok(stat) if stat.f_type as u64 == libc::CGROUP2_SUPER_MAGIC as u64 => {}
        Ok(stat) => {
            return CGroupContainment::Unavailable {
                reason: format!(
                    "cgroup root {} is not a cgroup-v2 mount (f_type {:#x}); \
                     point {CGROUP_ROOT_ENV} at a delegated cgroup-v2 directory",
                    root.display(),
                    stat.f_type as u64
                ),
            }
        }
        Err(e) => {
            return CGroupContainment::Unavailable {
                reason: format!("cannot stat cgroup root {}: {e}", root.display()),
            }
        }
    }
    // The mkdir probe is the delegation gate: only a root under which this
    // process can create and remove a child directory counts as delegated.
    let probe = root.join(format!(
        "{LEAF_PREFIX}-probe-{}-{}",
        std::process::id(),
        LEAF_SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    if let Err(e) = fs::create_dir(&probe) {
        return CGroupContainment::Unavailable {
            reason: format!(
                "cgroup root {} is not delegated to this process (probe mkdir failed): {e}",
                root.display()
            ),
        };
    }
    if let Err(e) = fs::remove_dir(&probe) {
        return CGroupContainment::Unavailable {
            reason: format!(
                "cgroup probe directory {} could not be removed: {e}",
                probe.display()
            ),
        };
    }
    // The mkdir gate proves directory delegation; the join check proves the
    // kernel will authorize the pre-exec migration itself (§2-5-2 containment
    // rides on the common ancestor, not on the leaf's own file bits).
    if let Err(reason) = join_authorized(&root) {
        return CGroupContainment::Unavailable { reason };
    }
    CGroupContainment::Active { root }
}

/// The process's own cgroup-v2 path relative to the cgroup-v2 mount, from
/// the unified-hierarchy line of `/proc/self/cgroup` (the leading `/` is
/// stripped so it composes with the mount path via `Path::join`).
fn self_cgroup_path() -> Option<String> {
    let text = fs::read_to_string("/proc/self/cgroup").ok()?;
    text.lines().find_map(|line| {
        line.strip_prefix("0::")
            .map(|path| path.trim_start_matches('/').to_owned())
    })
}

/// The longest path shared by both inputs, component-wise. The cgroup-v2
/// mount root is returned when the paths share nothing (different mounts).
fn common_ancestor(a: &Path, b: &Path) -> PathBuf {
    let mut common = PathBuf::new();
    for (x, y) in a.components().zip(b.components()) {
        if x != y {
            break;
        }
        common.push(x);
    }
    if common.as_os_str().is_empty() {
        common.push(CGROUP2_MOUNT);
    }
    common
}

/// Verify the kernel will authorize the pre-exec join before an Active
/// verdict is claimed.
///
/// Kernel cgroup-v2 delegation containment (Documentation/admin-guide/
/// cgroup-v2.rst §2-5-2, enforced by `cgroup_procs_write_permission`):
/// writing a pid into a `cgroup.procs` requires write access to the
/// `cgroup.procs` file of the COMMON ANCESTOR of the source (the joining
/// process's current cgroup) and destination (the leaf) cgroups — a
/// delegatee can migrate processes within the delegated subtree but can
/// never pull in from outside it. The pre-exec join performs exactly such a
/// write from this process's forked child, so a root that passes the mkdir
/// delegation gate but sits across this boundary from the spawning process
/// would fail every spawn with EACCES after the leaf was already created
/// (the observed round-1 CI failure: job cgroup `/`, delegated root
/// `/sys/fs/cgroup/sea-forge`, common ancestor `/`).
fn join_authorized(root: &Path) -> Result<(), String> {
    let self_rel = self_cgroup_path().ok_or_else(|| {
        "this process has no cgroup-v2 membership line (0::) in /proc/self/cgroup, so the \
         common ancestor the kernel checks for the pre-exec join cannot be determined"
            .to_owned()
    })?;
    let self_abs = Path::new(CGROUP2_MOUNT).join(self_rel);
    let ancestor = common_ancestor(root, &self_abs);
    let procs = ancestor.join("cgroup.procs");
    let meta = fs::metadata(&procs).map_err(|e| {
        format!(
            "cannot stat the pre-exec join's common-ancestor cgroup.procs {}: {e}",
            procs.display()
        )
    })?;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let mode = meta.permissions().mode();
    // The process's effective credentials, which the kernel's
    // inode_permission consults for the DAC check below (safe rustix
    // accessors; this module's unsafe budget stays on the pre-exec join).
    let (euid, egid) = (
        rustix::process::geteuid().as_raw(),
        rustix::process::getegid().as_raw(),
    );
    // The kernel's inode_permission(MAY_WRITE) over DAC bits: owner, group,
    // or other write bit for this process's effective credentials. Deliberately
    // fails closed (an exotic ACL that grants access reads as unauthorized —
    // a typed Unavailable, never a silently broken spawn).
    let writable = (meta.uid() == euid && mode & 0o200 != 0)
        || (meta.gid() == egid && mode & 0o020 != 0)
        || mode & 0o002 != 0;
    if writable {
        return Ok(());
    }
    Err(format!(
        "kernel cgroup-v2 delegation containment denies the pre-exec join: the common ancestor \
         of this process's cgroup ({}) and the delegated root ({}) is {}, whose cgroup.procs \
         is not writable by this user (owner uid {}, mode {mode:#o}) — a delegated subtree can \
         only be joined from inside it, or when the common ancestor's cgroup.procs is itself \
         delegated (cgroup-v2 §2-5-2 Delegation Containment)",
        self_abs.display(),
        root.display(),
        ancestor.display(),
        meta.uid()
    ))
}

/// Acquire the spawn's containment leaf, honouring the required-lane
/// contract. `Err` under [`REQUIRE_ENV`]`=1` when containment is unavailable
/// (fail closed — the caller must refuse the spawn). Otherwise an
/// unavailable cgroup degrades to `Ok(None)` with the reason surfaced on
/// stderr; the caller keeps the process-group kill.
///
/// This performs filesystem operations (mkdir under the delegated root), so
/// it must run on a thread whose Landlock domain still allows writes to the
/// cgroup root — for the jail backend that is the unrestricted caller thread
/// before `apply_landlock` restricts the runner thread.
pub fn acquire_spawn_lease(
    backend: &str,
    owner_pid: u32,
) -> Result<Option<CgroupLease>, SandboxError> {
    match containment() {
        CGroupContainment::Active { root } => match CgroupLease::create(&root, owner_pid) {
            Ok(lease) => {
                register_leaf(lease.leaf());
                Ok(Some(lease))
            }
            Err(e) if require_requested() => Err(SandboxError::new(
                "cgroup_unavailable",
                format!(
                    "{REQUIRE_ENV}=1 but the cgroup leaf could not be created under {}: {e}",
                    root.display()
                ),
            )),
            Err(e) => {
                report_degrade(
                    backend,
                    &format!("cgroup leaf creation under {} failed: {e}", root.display()),
                );
                Ok(None)
            }
        },
        CGroupContainment::Unavailable { reason } if require_requested() => Err(SandboxError::new(
            "cgroup_unavailable",
            format!(
                "{REQUIRE_ENV}=1 but cgroup containment is unavailable, failing closed instead \
                 of a process-group-only run: {reason}"
            ),
        )),
        CGroupContainment::Unavailable { reason } => {
            report_degrade(backend, &reason);
            Ok(None)
        }
    }
}

/// Attach the pre-exec join to `command` for the acquired lease. Same
/// required-lane contract as [`acquire_spawn_lease`]; on a non-required
/// degrade the leaf is removed before reporting so nothing leaks.
pub fn attach_lease(
    command: &mut std::process::Command,
    lease: Option<CgroupLease>,
    backend: &str,
) -> Result<Option<CgroupLease>, SandboxError> {
    let Some(lease) = lease else {
        return Ok(None);
    };
    match CgroupLease::attach(command, &lease) {
        Ok(()) => Ok(Some(lease)),
        Err(e) if require_requested() => Err(SandboxError::new(
            "cgroup_unavailable",
            format!("{REQUIRE_ENV}=1 but the pre-exec cgroup join could not be registered: {e}"),
        )),
        Err(e) => {
            let mut reason = format!("pre-exec cgroup join could not be registered: {e}");
            if let Err(remove_err) = lease.remove() {
                reason.push_str(&format!("; leaf cleanup also failed: {remove_err}"));
            }
            report_degrade(backend, &reason);
            Ok(None)
        }
    }
}

/// Surface a typed containment degrade on stderr (the crate's observability
/// channel; there is no logging facade in the kernel crates). The degrade is
/// always typed first — this print only makes it visible to an operator.
pub fn report_degrade(backend: &str, reason: &str) {
    eprintln!(
        "sea-forge-sandbox: lifecycle containment unavailable for the {backend} backend, \
         proceeding with the process-group kill only: {reason}"
    );
}

/// Verified teardown of every leaf this process leaked (a spawn-failure
/// window, a teardown that errored before removal, or an empty-but-
/// undeletable leaf recorded for the sweep by [`CgroupLease::remove`]'s
/// degrade). Empty leaves are removed; leaves that still hold members are
/// killed first and required to drain. An empty leaf whose removal this
/// kernel refuses (the same EACCES/EPERM/EBUSY class as at teardown) is
/// re-attempted and degrades the same way — printed, not an error — so a
/// leaf the filesystem refuses to release can never fail `destroy`. Every
/// other error propagates — `destroy` is never a silent no-op.
/// Leaves created by this process that have not (yet) been removed. The
/// destroy sweep verifies exactly these — never a filesystem glob by owner
/// prefix, which would sweep the live leaves of unrelated sandboxes in the
/// same process (one `destroy()` must not touch another sandbox's leaf).
static REGISTERED_LEAVES: std::sync::Mutex<Vec<std::path::PathBuf>> =
    std::sync::Mutex::new(Vec::new());

fn register_leaf(leaf: &std::path::Path) {
    // A poisoned lock is an already-panicking sweeper; the registry is
    // hygiene, so recovery proceeds with the current (possibly stale) list.
    if let Ok(mut leaves) = REGISTERED_LEAVES.lock() {
        leaves.push(leaf.to_path_buf());
    }
}

fn unregister_leaf(leaf: &std::path::Path) {
    if let Ok(mut leaves) = REGISTERED_LEAVES.lock() {
        leaves.retain(|l| l != leaf);
    }
}

pub fn destroy_process_leaves() -> Result<(), SandboxError> {
    let leaves = match REGISTERED_LEAVES.lock() {
        Ok(mut leaves) => std::mem::take(&mut *leaves),
        Err(_) => return Ok(()),
    };
    let mut first_error = None;
    for leaf in leaves {
        match CgroupLease::open_existing(leaf.clone()) {
            // The leaf vanished between the scan and the open: already clean.
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => {
                first_error.get_or_insert(SandboxError::new(
                    "internal_error",
                    format!("cgroup leaf {} could not be reopened: {e}", leaf.display()),
                ));
            }
            Ok(lease) => {
                if let Err(err) = lease
                    .kill_verified(VERIFY_DEADLINE)
                    .and_then(|()| lease.remove().map_err(KillError::Io))
                {
                    first_error.get_or_insert(match err {
                        KillError::Survivors(pids) => SandboxError::new(
                            "internal_error",
                            format!(
                                "cgroup leaf {} retained processes after destroy kill: {pids:?}",
                                lease.leaf().display()
                            ),
                        ),
                        KillError::Io(e) => SandboxError::new(
                            "internal_error",
                            format!("cgroup leaf {} destroy failed: {e}", lease.leaf().display()),
                        ),
                    });
                }
            }
        }
    }
    match first_error {
        Some(err) => Err(err),
        None => Ok(()),
    }
}
