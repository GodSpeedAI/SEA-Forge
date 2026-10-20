//! Pathname Unix-socket containment for isolated workloads (SEC-06, ADR-014).
//!
//! `connect(2)`/`sendto(2)`/`sendmsg(2)` to a pathname AF_UNIX socket are not
//! file opens, so the filesystem Landlock ruleset does not mediate them, and
//! no Landlock right for it exists before ABI 9
//! (`LANDLOCK_ACCESS_FS_RESOLVE_UNIX`). This module closes that gap with the
//! first mechanism the running kernel supports, probed at apply time and
//! never silently degraded:
//!
//! - ABI >= 9: a second Landlock ruleset handling only `RESOLVE_UNIX`,
//!   granted per socket tree with ordinary `path_beneath` rules. Kernel
//!   enforced and race-free; sockets created inside the workload's own
//!   Landlock domain stay reachable, which is exactly the legitimate-IPC
//!   allowance.
//! - ABI < 9 (x86_64 only): a seccomp user-notification filter on
//!   connect/sendto/sendmsg/sendmmsg, served by a supervisor thread that the
//!   helper creates BEFORE any restriction is applied, so it inherits
//!   neither the filters nor the Landlock domain. Nothing a workload writes
//!   is ever continued: per `seccomp_unotify(2)`'s TOCTOU warning (and the
//!   independent adversarial review's family-flip attack, where another
//!   workload thread rewrites a blocked syscall's address between the
//!   supervisor's read and the kernel's re-fetch), every trapped syscall is
//!   either BROKERED — performed by the supervisor from bytes it read once,
//!   with connect outcomes verified against the kernel-recorded
//!   `getpeername` peer — or REFUSED with the workload killed before the
//!   reply. Abstract AF_UNIX is refused (the round-2/3 posture of
//!   unreachable abstract sockets, kept by policy where no kernel scope
//!   applies to the broker); INET/INET6 is allowed only to ports the
//!   network posture grants; sendmmsg is refused outright.
//!
//! Failure is fail-closed: listener closure (helper exit) makes the kernel
//! answer the workload's monitored syscalls with `ENOSYS`, and any setup
//! error refuses the launch (`workload_isolation_unavailable`). On a
//! non-x86_64 kernel without ABI 9 there is no supervisor implementation;
//! isolation reports unavailable rather than running unconfined.
#![cfg(target_os = "linux")]
// The workspace denies `unsafe_code` so that unsafe is a choice, not an
// accident. This module is that choice: seccomp BPF, the listener ioctls
// and the ABI-9 Landlock right have no safe wrappers in the pinned
// dependencies (seccompiler 0.5 has no user-notification; landlock 0.4
// stops at ABI 7). Every unsafe block below is a raw syscall with its
// safety argument stated inline.
#![allow(unsafe_code)]

use crate::SandboxError;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

const UNAVAILABLE: &str = "workload_isolation_unavailable";

fn unavailable(message: impl Into<String>) -> SandboxError {
    SandboxError::new(UNAVAILABLE, message)
}

// Raw syscall numbers; the pinned landlock crate (0.4.x) cannot express ABI 9.
const SYS_LANDLOCK_CREATE_RULESET: libc::c_long = 444;
const SYS_LANDLOCK_ADD_RULE: libc::c_long = 445;
const SYS_LANDLOCK_RESTRICT_SELF: libc::c_long = 446;
/// `LANDLOCK_ACCESS_FS_RESOLVE_UNIX` (ABI 9): look up pathname UNIX sockets.
const ACCESS_FS_RESOLVE_UNIX: u64 = 1 << 16;
const LANDLOCK_RULE_PATH_BENEATH: libc::c_int = 1;

#[repr(C)]
struct LandlockRulesetAttr {
    handled_access_fs: u64,
}

#[repr(C)]
struct LandlockPathBeneathAttr {
    allowed_access: u64,
    parent_fd: libc::c_int,
}

/// The running kernel's Landlock ABI version (0 = no Landlock).
pub fn landlock_abi_version() -> u32 {
    // LANDLOCK_CREATE_RULESET_VERSION = 1 << 0.
    let version = unsafe {
        libc::syscall(
            SYS_LANDLOCK_CREATE_RULESET,
            std::ptr::null::<u8>(),
            0usize,
            1,
        )
    };
    if version >= 0 {
        version as u32
    } else {
        0
    }
}

/// Which containment mechanism was applied, for logs, tests and evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applied {
    /// Second Landlock ruleset handling `RESOLVE_UNIX`.
    LandlockResolveUnix,
    /// Seccomp user-notification filter installed (kernel Landlock ABI < 9);
    /// `listener` must be served by [`supervise`] on a trusted thread.
    #[cfg(target_arch = "x86_64")]
    SeccompSupervisor { listener: i32 },
}

/// Apply pathname-socket containment to the calling thread for `grants`
/// (absolute socket trees). Call after the main Landlock restriction (which
/// sets `no_new_privs`). The supervisor variant installs only the filter;
/// the caller then spawns [`supervise`] — thread filters are inherited, but
/// the supervisor's own syscalls (ioctl, process_vm_readv, kill) are all
/// allowed by this filter.
pub fn apply(grants: &[PathBuf]) -> Result<Applied, SandboxError> {
    if landlock_abi_version() >= 9 {
        apply_resolve_unix(grants)?;
        Ok(Applied::LandlockResolveUnix)
    } else {
        #[cfg(target_arch = "x86_64")]
        {
            let listener = install_supervisor_filter()?;
            Ok(Applied::SeccompSupervisor { listener })
        }
        #[cfg(not(target_arch = "x86_64"))]
        Err(unavailable(
            "kernel Landlock ABI < 9 and no Unix-socket supervisor exists for this architecture",
        ))
    }
}

/// Second Landlock ruleset handling only `RESOLVE_UNIX`, granted per tree.
/// With no grants it denies pathname-socket resolution everywhere, matching
/// the default-deny posture of the filesystem dimension.
fn apply_resolve_unix(grants: &[PathBuf]) -> Result<(), SandboxError> {
    let attr = LandlockRulesetAttr {
        handled_access_fs: ACCESS_FS_RESOLVE_UNIX,
    };
    let fd = unsafe {
        libc::syscall(
            SYS_LANDLOCK_CREATE_RULESET,
            &attr,
            std::mem::size_of::<LandlockRulesetAttr>(),
            0u32,
        )
    };
    if fd < 0 {
        return Err(unavailable(format!(
            "landlock_create_ruleset(RESOLVE_UNIX): {}",
            std::io::Error::last_os_error()
        )));
    }
    let fd = fd as libc::c_int;
    for tree in grants {
        let path = tree
            .canonicalize()
            .map_err(|e| unavailable(format!("socket grant {}: {e}", tree.display())))?;
        let Ok(cpath) = std::ffi::CString::new(path.into_os_string().into_vec()) else {
            return Err(unavailable("socket grant path contains a NUL byte"));
        };
        let parent = unsafe { libc::open(cpath.as_ptr(), libc::O_PATH | libc::O_CLOEXEC) };
        if parent < 0 {
            unsafe { libc::close(fd) };
            return Err(unavailable(format!(
                "open socket grant {}: {}",
                tree.display(),
                std::io::Error::last_os_error()
            )));
        }
        let rule = LandlockPathBeneathAttr {
            allowed_access: ACCESS_FS_RESOLVE_UNIX,
            parent_fd: parent,
        };
        let added = unsafe {
            libc::syscall(
                SYS_LANDLOCK_ADD_RULE,
                fd,
                LANDLOCK_RULE_PATH_BENEATH,
                &rule,
                0u32,
            )
        };
        let error = std::io::Error::last_os_error();
        unsafe { libc::close(parent) };
        if added < 0 {
            unsafe { libc::close(fd) };
            return Err(unavailable(format!(
                "landlock_add_rule(RESOLVE_UNIX, {}): {error}",
                tree.display()
            )));
        }
    }
    let restricted = unsafe { libc::syscall(SYS_LANDLOCK_RESTRICT_SELF, fd, 0u32) };
    let error = std::io::Error::last_os_error();
    unsafe { libc::close(fd) };
    if restricted < 0 {
        return Err(unavailable(format!(
            "landlock_restrict_self(RESOLVE_UNIX): {error}"
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Seccomp user-notification supervisor (kernel Landlock ABI < 9, x86_64).
// ---------------------------------------------------------------------------

const SECCOMP_RET_USER_NOTIF: u32 = 0x7fc0_0000;
const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;
const SECCOMP_FILTER_FLAG_NEW_LISTENER: u32 = 1 << 3;
const AUDIT_ARCH_X86_64: u32 = 0x4000_0003;
const X32: u64 = 0x4000_0000;
// connect(2), sendto(2), sendmsg(2), sendmmsg(2); `send(2)` is sendto with
// no address. sendmmsg is trapped and refused (found untrapped by the
// independent adversarial review: a race-free datagram path to any
// DAC-admissible pathname socket).
#[cfg(target_arch = "x86_64")]
const MONITORED: [i64; 4] = [42, 44, 46, 307];

// Listener ioctls per include/uapi/linux/seccomp.h ('!' magic):
// RECV = _IOWR('!', 0, seccomp_notif[80]), SEND = _IOWR('!', 1, resp[24]).
#[cfg(target_arch = "x86_64")]
const IOCTL_NOTIF_RECV: libc::c_ulong = 0xc050_2100;
#[cfg(target_arch = "x86_64")]
const IOCTL_NOTIF_SEND: libc::c_ulong = 0xc018_2101;

/// `struct seccomp_notif`: id, pid, flags, then `seccomp_data` inline.
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Notif {
    id: u64,
    pid: u32,
    flags: u32,
    nr: i32,
    arch: u32,
    _ip: u64,
    args: [u64; 6],
}

/// `struct seccomp_notif_resp`.
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct NotifResp {
    id: u64,
    val: i64,
    error: i32,
    flags: u32,
}

fn insn(code: u16, jt: u8, jf: u8, k: u32) -> libc::sock_filter {
    libc::sock_filter { code, jt, jf, k }
}

/// The BPF program: USER_NOTIF for the monitored syscalls (native and x32
/// numbers — x32 shares the x86_64 audit architecture, which is why the
/// arch check alone cannot exclude it), ALLOW otherwise. Foreign
/// architectures fall through to ALLOW *in this filter*; the main deny
/// filter already kills them, and the most-restrictive action wins across
/// stacked filters, so this adds no weakness.
#[cfg(target_arch = "x86_64")]
fn bpf_program() -> Vec<libc::sock_filter> {
    // BPF opcode classes used here, composed per linux/filter.h:
    // BPF_LD(0x00)|BPF_W(0x00)|BPF_ABS(0x20), BPF_JMP(0x05)|BPF_JEQ(0x10)|BPF_K(0x00),
    // BPF_RET(0x06)|BPF_K(0x00). The zero-valued flags fold away.
    const LD_W_ABS: u16 = 0x20;
    const JMP_JEQ_K: u16 = 0x05 | 0x10;
    const RET_K: u16 = 0x06;
    let monitored: Vec<u32> = MONITORED
        .iter()
        .flat_map(|&n| [n as u64, n as u64 + X32])
        .map(|n| n as u32)
        .collect();
    let mut prog = vec![
        insn(LD_W_ABS, 0, 0, 4), // arch
        // Native x86_64: skip the ALLOW and continue to the nr checks (jt=1);
        // anything else: fall through to ALLOW in this filter (jf=0) — the
        // stacked deny filter already kills foreign architectures, and the
        // most-restrictive action wins.
        insn(JMP_JEQ_K, 1, 0, AUDIT_ARCH_X86_64),
        insn(RET_K, 0, 0, SECCOMP_RET_ALLOW),
    ];
    for &number in &monitored {
        prog.push(insn(LD_W_ABS, 0, 0, 0)); // nr (numbers fit 32 bits)
                                            // match -> next insn (USER_NOTIF); no match -> skip it, keep looking
        prog.push(insn(JMP_JEQ_K, 0, 1, number));
        prog.push(insn(RET_K, 0, 0, SECCOMP_RET_USER_NOTIF));
    }
    prog.push(insn(RET_K, 0, 0, SECCOMP_RET_ALLOW));
    prog
}

/// Install the USER_NOTIF filter on the calling thread and return the
/// listener fd. The filter is inherited by everything the calling thread
/// spawns; serve it from a thread spawned *before* this call, which stays
/// unfiltered (seccomp filters are per-thread).
#[cfg(target_arch = "x86_64")]
pub fn install_supervisor_filter() -> Result<i32, SandboxError> {
    let program = bpf_program();
    let mut fprog = libc::sock_fprog {
        len: program.len() as u16,
        filter: program.as_ptr() as *mut libc::sock_filter,
    };
    let listener = unsafe {
        libc::syscall(
            libc::SYS_seccomp,
            1, // SECCOMP_SET_MODE_FILTER
            SECCOMP_FILTER_FLAG_NEW_LISTENER,
            &mut fprog,
        )
    };
    if listener < 0 {
        let error = std::io::Error::last_os_error();
        let cause = match error.raw_os_error() {
            Some(libc::EBUSY) => {
                "this process chain already inherited a user-notify seccomp \
                filter, and the kernel forbids a second listener (has_duplicate_listener); \
                pathname-socket supervision is unavailable here"
            }
            _ => "seccomp USER_NOTIF filter install failed",
        };
        return Err(unavailable(format!("{cause}: {error}")));
    }
    Ok(listener as i32)
}

/// Whether a seccomp user-notification filter can be installed in this
/// process chain, probed in a throwaway child. Preconditions differ from
/// the real install only in the program (a trivial allow-all); the kernel
/// checks (no_new_privs, single-listener) are identical.
#[cfg(target_arch = "x86_64")]
pub fn supervisor_installable() -> Result<(), String> {
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        return Err(format!("fork: {}", std::io::Error::last_os_error()));
    }
    if pid == 0 {
        // Safety: single-purpose child that only calls two raw syscalls.
        unsafe {
            libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
            let code = insn(0x06, 0, 0, SECCOMP_RET_ALLOW);
            let mut fprog = libc::sock_fprog {
                len: 1,
                filter: &code as *const _ as *mut libc::sock_filter,
            };
            let ok = libc::syscall(
                libc::SYS_seccomp,
                1,
                SECCOMP_FILTER_FLAG_NEW_LISTENER,
                &mut fprog,
            ) >= 0;
            libc::_exit(if ok { 0 } else { 1 });
        }
    }
    let mut status = 0;
    let waited = unsafe { libc::waitpid(pid, &mut status, 0) };
    if waited != pid || !libc::WIFEXITED(status) {
        return Err("socket-gate probe child did not exit cleanly".into());
    }
    match libc::WEXITSTATUS(status) {
        0 => Ok(()),
        _ => Err(
            "a seccomp user-notify filter cannot be installed in this process chain \
             (an inherited listener already exists, or seccomp is refused); isolated \
             workloads cannot be launched with pathname-socket containment here"
                .into(),
        ),
    }
}

/// The destination a monitored syscall targets. Everything the supervisor
/// decides is BROKERED or REFUSED — under `seccomp_unotify(2)`'s TOCTOU
/// warning no trapped syscall is ever continued: the independent adversarial
/// review showed a CONTINUE classified from supervisor-read bytes is racy
/// (a blocked syscall's address buffer can be rewritten by another workload
/// thread between the supervisor's read and the kernel's re-fetch, flipping
/// AF_INET to AF_UNIX), so there is no continuation and no re-fetch at all.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Target {
    /// No explicit address (send on an already-connected socket, or a
    /// `connect(fd, NULL, 0)` dissolve): the connection was itself gated.
    None,
    /// AF_UNIX with a pathname, in resolved absolute form (`None` when the
    /// path did not resolve — refused).
    Pathname(Option<PathBuf>),
    /// Abstract AF_UNIX: refused on the supervised path (no kernel scope
    /// applies to the broker's own syscalls, so the round-2/3 posture of
    /// unreachable abstract sockets is kept by refusing).
    Abstract,
    /// INET/INET6 connect or datagram send: the granted port plus the raw
    /// sockaddr bytes as first read (the broker sends exactly these bytes,
    /// never a kernel re-fetch of workload memory).
    Inet(u16, Vec<u8>),
    /// Unreadable destination memory: fail closed (refuse and kill) rather
    /// than continue past an unread classification.
    Refuse,
}

#[cfg(target_arch = "x86_64")]
fn read_remote(pid: libc::pid_t, address: u64, size: usize) -> Option<Vec<u8>> {
    let mut buffer = vec![0u8; size];
    let local = libc::iovec {
        iov_base: buffer.as_mut_ptr().cast(),
        iov_len: size,
    };
    let remote = libc::iovec {
        iov_base: address as *mut libc::c_void,
        iov_len: size,
    };
    let read = unsafe { libc::process_vm_readv(pid, &local, 1, &remote, 1, 0) };
    if read > 0 {
        buffer.truncate(read as usize);
        Some(buffer)
    } else {
        None
    }
}

/// Decode the destination sockaddr of `notif` into a [`Target`].
#[cfg(target_arch = "x86_64")]
fn target_of(notif: &Notif) -> Target {
    let pid = notif.pid as libc::pid_t;
    let (address, length) = match notif.nr {
        // connect(fd, addr, addrlen)
        42 => (notif.args[1], notif.args[2].min(110) as usize),
        // sendto(fd, buf, len, flags, addr, addrlen)
        44 => (notif.args[4], notif.args[5].min(110) as usize),
        46 => {
            // sendmsg: struct msghdr { void *msg_name; socklen_t msg_namelen; ... }
            let Some(header) = read_remote(pid, notif.args[1], 16) else {
                return Target::Refuse;
            };
            let pointer = u64::from_le_bytes(header[0..8].try_into().unwrap());
            let name_len = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
            if pointer == 0 || name_len == 0 {
                return Target::None;
            }
            (pointer, name_len.min(110))
        }
        _ => return Target::None,
    };
    if address == 0 || length < 3 {
        // connect(fd, NULL, 0) (the dissolve form) and address-less sends.
        return Target::None;
    }
    let Some(sockaddr) = read_remote(pid, address, length) else {
        return Target::Refuse;
    };
    let family = u16::from_le_bytes([sockaddr[0], sockaddr[1]]);
    match family {
        x if x == libc::AF_UNIX as u16 => {
            if sockaddr[2] == 0 {
                return Target::Abstract;
            }
        }
        x if x == libc::AF_INET as u16 => {
            if sockaddr.len() < 6 {
                return Target::Refuse;
            }
            let port = u16::from_be_bytes([sockaddr[2], sockaddr[3]]);
            return Target::Inet(port, sockaddr);
        }
        x if x == libc::AF_INET6 as u16 => {
            if sockaddr.len() < 6 {
                return Target::Refuse;
            }
            let port = u16::from_be_bytes([sockaddr[2], sockaddr[3]]);
            return Target::Inet(port, sockaddr);
        }
        _ => return Target::Refuse,
    }
    let bytes = &sockaddr[2..];
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let raw = PathBuf::from(std::ffi::OsString::from_vec(bytes[..end].to_vec()));
    let absolute = if raw.is_absolute() {
        raw
    } else {
        match std::fs::read_link(format!("/proc/{pid}/cwd")) {
            Ok(cwd) => cwd.join(raw),
            Err(_) => return Target::Pathname(None),
        }
    };
    Target::Pathname(absolute.canonicalize().ok())
}

/// `(st_dev, st_ino)` of `path` when it is a socket inode (`O_PATH` + fstat,
/// so a swapped entry cannot redirect the check).
fn socket_inode(path: &Path) -> Option<(u64, u64)> {
    let cpath = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_PATH | libc::O_CLOEXEC) };
    if fd < 0 {
        return None;
    }
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    let ok = unsafe { libc::fstat(fd, &mut stat) } == 0;
    unsafe { libc::close(fd) };
    if ok && stat.st_mode & libc::S_IFMT == libc::S_IFSOCK {
        Some((stat.st_dev, stat.st_ino))
    } else {
        None
    }
}

/// Who decides a monitored syscall, per `seccomp_unotify(2)`'s warning that
/// a userspace verdict followed by `SECCOMP_USER_NOTIF_FLAG_CONTINUE` is a
/// TOCTOU race: under CONTINUE the kernel re-fetches the arguments and
/// executes the syscall, so a workload can rewrite its buffer between the
/// supervisor's check and the kernel's fetch. The rule implemented by
/// [`supervise`]:
///
/// - CONTINUE only when the *kernel itself* is the security authority for
///   the target on its own fetch: non-UNIX families fall to the Landlock
///   network rights, and abstract AF_UNIX falls to the Landlock
///   abstract-socket scope. Whatever the workload races its address into,
///   the kernel enforces the policy against the bytes it actually fetched;
///   the supervisor delegates rather than decides.
/// - Brokered — pathname AF_UNIX and address-less sends: the kernel never
///   executes the workload's syscall at all. The supervisor performs the
///   equivalent operation itself from memory it read once, verifies the
///   outcome from kernel state, and only then replies to the frozen
///   workload.
/// - Refuse — everything a grant does not admit. Policy denials are
///   preventive on their own (the kernel never performs the syscall), so
///   [`broker_deny`] replies and lets the frozen workload observe a plain
///   `EACCES`; only race-class and uninterpretable requests kill.
///
/// Deny a monitored syscall with `error`.
#[cfg(target_arch = "x86_64")]
fn broker_deny(listener: i32, notif: &Notif, error: i32) {
    let resp = NotifResp {
        id: notif.id,
        val: 0,
        error,
        flags: 0,
    };
    unsafe { libc::ioctl(listener, IOCTL_NOTIF_SEND as _, &resp) };
}

/// Duplicate one of the workload's descriptors into the supervisor by
/// `pidfd_open` + `pidfd_getfd` (kernel-verified identity; the helper is
/// the workload's parent inside the same user namespace).
#[cfg(target_arch = "x86_64")]
unsafe fn dup_workload_fd(pid: libc::pid_t, fd: i32) -> Result<i32, std::io::Error> {
    let pidfd = libc::syscall(libc::SYS_pidfd_open, pid, 0u32);
    if pidfd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    let pidfd = pidfd as i32;
    let duplicated = libc::syscall(libc::SYS_pidfd_getfd, pidfd, fd, 0u32);
    let error = std::io::Error::last_os_error();
    libc::close(pidfd);
    if duplicated < 0 {
        return Err(error);
    }
    Ok(duplicated as i32)
}

/// The bound name of `fd`'s peer, as the kernel recorded it when the peer
/// bound (pathname sockets only; abstract/unbound peers give `None`).
#[cfg(target_arch = "x86_64")]
fn peer_bound_path(fd: i32) -> Option<PathBuf> {
    let mut storage: libc::sockaddr_storage = unsafe { std::mem::zeroed() };
    let mut length = std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;
    if unsafe {
        libc::getpeername(
            fd,
            &mut storage as *mut _ as *mut libc::sockaddr,
            &mut length,
        )
    } != 0
    {
        return None;
    }
    if storage.ss_family != libc::AF_UNIX as u16 {
        return None;
    }
    let unix = unsafe { &*(std::ptr::addr_of!(storage) as *const libc::sockaddr_un) };
    let bytes: &[u8] = unsafe {
        std::slice::from_raw_parts(unix.sun_path.as_ptr() as *const u8, length as usize - 2)
    };
    if bytes.first() == Some(&0) || bytes.is_empty() {
        return None; // abstract or unnamed peer: not a pathname decision
    }
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let bound = PathBuf::from(std::ffi::OsString::from_vec(bytes[..end].to_vec()));
    // A peer that bound a relative or symlinked name yields non-canonical
    // text; resolve it in the (shared) mount view so the grant comparison
    // sees the same form the grants were canonicalized to. An unresolvable
    // peer name is refused.
    Some(bound.canonicalize().unwrap_or(bound))
}

fn peer_under_grant(peer: Option<PathBuf>, granted: &[PathBuf]) -> bool {
    peer.is_some_and(|peer| granted.iter().any(|tree| peer.starts_with(tree)))
}

/// Pin `fd` to `path` (connect), require the kernel-recorded peer to sit
/// under a granted tree, and return the descriptor still pinned. Any
/// mismatch is the swap race detected — the caller kills the workload.
#[cfg(target_arch = "x86_64")]
fn pin_and_verify(
    fd: i32,
    path: &Path,
    granted: &[PathBuf],
    workload_pid: libc::pid_t,
) -> Result<(), std::io::Error> {
    let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    address.sun_family = libc::AF_UNIX as u16;
    let raw = path.as_os_str().as_bytes();
    if raw.len() >= address.sun_path.len() {
        return Err(std::io::Error::from_raw_os_error(libc::ENAMETOOLONG));
    }
    let sun_path: &mut [i8] = &mut address.sun_path;
    let destination: &mut [u8] =
        unsafe { std::slice::from_raw_parts_mut(sun_path.as_mut_ptr() as *mut u8, sun_path.len()) };
    destination[..raw.len()].copy_from_slice(raw);
    let length = 2 + raw.len();
    // Non-blocking + bounded poll: a full accept queue must not freeze the
    // supervisor; the workload stays frozen either way until the reply.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL, 0) };
    let _ = unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
    let connected = unsafe {
        libc::connect(
            fd,
            &address as *const _ as *const libc::sockaddr,
            length as libc::socklen_t,
        )
    };
    let connected = if connected == 0 {
        Ok(())
    } else {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EINPROGRESS) {
            Err(error)
        } else {
            let mut poll = [libc::pollfd {
                fd,
                events: libc::POLLOUT,
                revents: 0,
            }];
            let ready = unsafe { libc::poll(poll.as_mut_ptr(), 1, 5_000) };
            if ready <= 0 {
                Err(std::io::Error::from_raw_os_error(libc::ETIMEDOUT))
            } else {
                let mut code = 0i32;
                let mut size = std::mem::size_of::<i32>() as libc::socklen_t;
                unsafe {
                    libc::getsockopt(
                        fd,
                        libc::SOL_SOCKET,
                        libc::SO_ERROR,
                        &mut code as *mut _ as *mut libc::c_void,
                        &mut size,
                    )
                };
                if code == 0 {
                    Ok(())
                } else {
                    Err(std::io::Error::from_raw_os_error(code))
                }
            }
        }
    };
    let _ = unsafe { libc::fcntl(fd, libc::F_SETFL, flags) };
    connected?;
    if !peer_under_grant(peer_bound_path(fd), granted) {
        // The kernel's own record disagrees with the validated path: the
        // name was swapped inside the check-to-connect window. This is the
        // one refusal where a reply alone cannot be trusted — the workload
        // is killed before it is ever resumed.
        unsafe { libc::kill(workload_pid, libc::SIGKILL) };
        return Err(std::io::Error::from_raw_os_error(libc::EACCES));
    }
    Ok(())
}

/// Perform a brokered `connect` for the workload's frozen syscall.
#[cfg(target_arch = "x86_64")]
fn broker_connect(
    notif: &Notif,
    path: &Path,
    granted: &[PathBuf],
    workload_pid: libc::pid_t,
) -> Result<(), i32> {
    unsafe {
        let fd = dup_workload_fd(workload_pid, notif.args[0] as i32).map_err(|_| libc::EACCES)?;
        let result = pin_and_verify(fd, path, granted, workload_pid);
        // Drop our reference; the workload's own descriptor keeps any
        // established connection alive.
        libc::close(fd);
        result.map_err(|_| libc::EACCES)
    }
}

/// Cap on payload bytes a brokered send may move, read through
/// `process_vm_readv` before anything is sent. A larger request is not
/// truncated — it refuses and kills, fail closed.
#[cfg(target_arch = "x86_64")]
const BROKER_MAX_PAYLOAD: usize = 1 << 20;

/// Read the payload a frozen send would move (iovec walk through
/// `process_vm_readv`), or kill-and-refuse on any unreadable or oversize
/// request. Single fetch: the supervisor sends these bytes, not the kernel
/// re-reading workload memory.
#[cfg(target_arch = "x86_64")]
fn broker_payload(notif: &Notif, sendmsg: bool, workload_pid: libc::pid_t) -> Result<Vec<u8>, i32> {
    let pid = notif.pid as libc::pid_t;
    let (base, length): (u64, u64) = if !sendmsg {
        (notif.args[1], notif.args[2])
    } else {
        // struct msghdr: msg_iov (ptr), msg_iovlen (size_t) at offsets 16/24.
        let header = match read_remote(pid, notif.args[1], 40) {
            Some(header) => header,
            None => {
                unsafe { libc::kill(workload_pid, libc::SIGKILL) };
                return Err(libc::EACCES);
            }
        };
        let iov = u64::from_le_bytes(header[16..24].try_into().unwrap());
        let iovlen = u64::from_le_bytes(header[24..32].try_into().unwrap());
        if iov == 0 || iovlen == 0 || iovlen > 1024 {
            return Ok(Vec::new());
        }
        let mut total = 0u64;
        let mut payload = Vec::new();
        if iovlen > 1024 {
            // Not silently empty: an unreadable iovec array refuses, and
            // the workload is killed (it cannot be trusted to interpret
            // the denial).
            unsafe { libc::kill(workload_pid, libc::SIGKILL) };
            return Err(libc::EACCES);
        }
        for index in 0..iovlen as usize {
            let entry = match read_remote(pid, iov + (index as u64) * 16, 16) {
                Some(entry) => entry,
                None => {
                    unsafe { libc::kill(workload_pid, libc::SIGKILL) };
                    return Err(libc::EACCES);
                }
            };
            let pointer = u64::from_le_bytes(entry[0..8].try_into().unwrap());
            let span = u64::from_le_bytes(entry[8..16].try_into().unwrap());
            if span > 0 {
                total += span;
                if total > BROKER_MAX_PAYLOAD as u64 {
                    return Err(libc::EACCES);
                }
                let chunk = match read_remote(pid, pointer, span as usize) {
                    Some(chunk) => chunk,
                    None => {
                        unsafe { libc::kill(workload_pid, libc::SIGKILL) };
                        return Err(libc::EACCES);
                    }
                };
                payload.extend_from_slice(&chunk);
            }
        }
        return Ok(payload);
    };
    if base == 0 || length == 0 {
        return Ok(Vec::new());
    }
    if length > BROKER_MAX_PAYLOAD as u64 {
        unsafe { libc::kill(workload_pid, libc::SIGKILL) };
        return Err(libc::EACCES);
    }
    match read_remote(pid, base, length as usize) {
        Some(bytes) => Ok(bytes),
        None => {
            unsafe { libc::kill(workload_pid, libc::SIGKILL) };
            Err(libc::EACCES)
        }
    }
}

/// Perform a brokered send on behalf of the frozen workload: pin the
/// workload's own datagram socket to the validated path, require the
/// kernel-recorded peer to sit under a granted tree, send the copied
/// payload, and best-effort dissolve the pin so later unaddressed sends
/// keep the socket's prior semantics (a surviving pin can only point at a
/// grant-verified peer).
#[cfg(target_arch = "x86_64")]
fn broker_send(
    notif: &Notif,
    path: &Path,
    granted: &[PathBuf],
    workload_pid: libc::pid_t,
    sendmsg: bool,
) -> Result<i64, i32> {
    // Unreadable or oversize payloads deny (and the caller's refuse path
    // kills the frozen workload); nothing is sent.
    let payload = broker_payload(notif, sendmsg, workload_pid)?;
    unsafe {
        let fd = dup_workload_fd(workload_pid, notif.args[0] as i32).map_err(|_| libc::EACCES)?;
        let result = (|| {
            pin_and_verify(fd, path, granted, workload_pid).map_err(|_| libc::EACCES)?;
            let sent = if !sendmsg {
                libc::send(
                    fd,
                    payload.as_ptr() as *const libc::c_void,
                    payload.len(),
                    notif.args[3] as i32,
                )
            } else {
                // Connected send of the copied payload: the association
                // pinned above carries the destination.
                let mut header: libc::msghdr = std::mem::zeroed();
                header.msg_iov = &mut libc::iovec {
                    iov_base: payload.as_ptr() as *mut libc::c_void,
                    iov_len: payload.len(),
                } as *mut _;
                header.msg_iovlen = 1;
                libc::sendmsg(fd, &header, notif.args[2] as i32)
            };
            if sent < 0 {
                return Err(libc::EACCES);
            }
            // Best-effort dissolution; a refused dissolve is harmless.
            let mut unspec: libc::sockaddr_un = std::mem::zeroed();
            unspec.sun_family = libc::AF_UNSPEC as u16;
            libc::connect(fd, &unspec as *const _ as *const libc::sockaddr, 2);
            Ok(sent as i64)
        })();
        libc::close(fd);
        result
    }
}

/// Serve socket policy on `listener` until process exit. Blocks the calling
/// thread; intended for a dedicated trusted thread in the helper created
/// BEFORE any restriction is applied (so neither the seccomp filters nor the
/// Landlock domain apply to it — its own brokered syscalls must not trap on
/// the listener it serves, and its canonicalization must see granted trees
/// the workload cannot read), with `workload_pid` its direct child and
/// `granted_ports` the TCP ports `NetworkPosture` allows.
///
/// Every trapped syscall is brokered or refused; nothing is continued
/// (see [`Target`]). A pathname target is performed only when its resolved
/// path lies under a granted tree and is a socket inode, with the
/// kernel-recorded peer name (`getpeername`) re-verified against the grants
/// after the pin; a mismatch (the rename/swap race) kills the workload
/// before the reply lands — it is frozen inside the intercepted syscall and
/// can execute nothing past it, so a raced swap costs it its life and costs
/// the outside peer an empty connection, never data.
#[cfg(target_arch = "x86_64")]
pub fn supervise(
    listener: i32,
    grants: Vec<PathBuf>,
    granted_ports: Vec<u16>,
    workload_pid: libc::pid_t,
) {
    let granted: Vec<PathBuf> = grants
        .iter()
        .filter_map(|tree| tree.canonicalize().ok())
        .collect();
    loop {
        let mut notif = Notif::default();
        if unsafe { libc::ioctl(listener, IOCTL_NOTIF_RECV as _, &mut notif) } < 0 {
            let error = std::io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(libc::ENOSYS) | Some(libc::EBADF)) {
                return; // listener retired: helper is shutting down
            }
            continue;
        }
        // sendmmsg moves multiple datagrams with per-message destinations;
        // brokering its full mmsghdr array is out of scope, and no
        // SEA-Forge workload path uses it, so it is refused outright.
        let target = target_of(&notif);
        if std::env::var_os("SEA_FORGE_SOCKET_SUPERVISOR_DEBUG").is_some() {
            eprintln!(
                "[socket-supervisor] recv nr={} target={:?}",
                notif.nr, target
            );
        }
        let outcome: Result<Option<i64>, i32> = if notif.nr == 307 {
            Err(libc::EACCES) // policy denial: refused, workload continues
        } else {
            match &target {
                Target::None => {
                    if notif.nr == 42 {
                        // connect(fd, NULL, 0) or the AF_UNSPEC dissolve:
                        // performed as written on the duplicate.
                        broker_dissolve(&notif, workload_pid).map(|_| None)
                    } else {
                        // An address-less send on an already-connected
                        // socket: the connection was brokered at connect
                        // time, so the send is performed from copied
                        // payload bytes on the duplicate.
                        broker_connected_send(&notif, workload_pid, notif.nr == 46).map(Some)
                    }
                }
                Target::Pathname(None) => Err(libc::EACCES),
                Target::Pathname(Some(path)) => {
                    let under_grant = granted.iter().any(|tree| path.starts_with(tree));
                    let socket = if under_grant {
                        socket_inode(path.as_path())
                    } else {
                        None
                    };
                    if socket.is_none() {
                        Err(libc::EACCES)
                    } else if notif.nr == 42 {
                        broker_connect(&notif, path.as_path(), &granted, workload_pid).map(|_| None)
                    } else {
                        broker_send(
                            &notif,
                            path.as_path(),
                            &granted,
                            workload_pid,
                            notif.nr == 46,
                        )
                        .map(Some)
                    }
                }
                // Abstract sockets are bounded by the dedicated network
                // namespace: only names created inside the workload's own
                // namespace are addressable, so its own abstract IPC is
                // legitimate and the broker performs it from copied bytes.
                Target::Abstract => broker_abstract(&notif, workload_pid).map(Some),
                Target::Inet(port, raw) => {
                    if !granted_ports.contains(port) {
                        Err(libc::EACCES)
                    } else {
                        broker_inet(&notif, raw, workload_pid).map(Some)
                    }
                }
                // Unreadable destination memory: uninterpretable, kill.
                Target::Refuse => {
                    unsafe { libc::kill(workload_pid, libc::SIGKILL) };
                    Err(libc::EACCES)
                }
            }
        };
        if std::env::var_os("SEA_FORGE_SOCKET_SUPERVISOR_DEBUG").is_some() {
            eprintln!(
                "[socket-supervisor] done nr={} outcome={:?}",
                notif.nr, outcome
            );
        }
        match outcome {
            Ok(Some(value)) => {
                // Brokered completion: the operation already happened,
                // performed by this thread on verified state.
                let resp = NotifResp {
                    id: notif.id,
                    val: value,
                    error: 0,
                    flags: 0,
                };
                unsafe { libc::ioctl(listener, IOCTL_NOTIF_SEND as _, &resp) };
            }
            Ok(None) => {
                // Brokered completion with a void result (connect, dissolve).
                let resp = NotifResp {
                    id: notif.id,
                    val: 0,
                    error: 0,
                    flags: 0,
                };
                unsafe { libc::ioctl(listener, IOCTL_NOTIF_SEND as _, &resp) };
            }
            Err(error) => broker_deny(listener, &notif, error),
        }
    }
}

/// Perform an abstract-socket operation as the workload wrote it. The
/// destination bytes were read once from the frozen workload; the shared
/// network namespace bounds which abstract names exist at all.
#[cfg(target_arch = "x86_64")]
fn broker_abstract(notif: &Notif, workload_pid: libc::pid_t) -> Result<i64, i32> {
    let (address, length) = match notif.nr {
        42 => (notif.args[1], notif.args[2].min(110) as usize),
        44 => (notif.args[4], notif.args[5].min(110) as usize),
        _ => return Err(libc::EACCES),
    };
    let payload = if notif.nr == 42 {
        Vec::new()
    } else {
        broker_payload(notif, notif.nr == 46, workload_pid)?
    };
    let sockaddr = read_remote(notif.pid as libc::pid_t, address, length).ok_or_else(|| {
        unsafe { libc::kill(workload_pid, libc::SIGKILL) };
        libc::EACCES
    })?;
    unsafe {
        let fd = dup_workload_fd(workload_pid, notif.args[0] as i32).map_err(|_| libc::EACCES)?;
        let raw_result: i64 = if notif.nr == 42 {
            libc::connect(
                fd,
                sockaddr.as_ptr() as *const libc::sockaddr,
                sockaddr.len() as libc::socklen_t,
            ) as i64
        } else {
            libc::sendto(
                fd,
                payload.as_ptr() as *const libc::c_void,
                payload.len(),
                notif.args[3] as i32,
                sockaddr.as_ptr() as *const libc::sockaddr,
                sockaddr.len() as libc::socklen_t,
            ) as i64
        };
        libc::close(fd);
        if raw_result < 0 {
            Err(std::io::Error::last_os_error()
                .raw_os_error()
                .unwrap_or(libc::EACCES))
        } else {
            Ok(raw_result as i64)
        }
    }
}

/// Perform `connect` as the workload wrote it (the dissolve forms: NULL or
/// a zero-length address), from the supervisor's unrestricted context.
#[cfg(target_arch = "x86_64")]
fn broker_dissolve(notif: &Notif, workload_pid: libc::pid_t) -> Result<(), i32> {
    unsafe {
        let fd = dup_workload_fd(workload_pid, notif.args[0] as i32).map_err(|_| libc::EACCES)?;
        let result = libc::connect(fd, std::ptr::null(), 0);
        let error = std::io::Error::last_os_error();
        libc::close(fd);
        if result == 0 {
            Ok(())
        } else {
            Err(error.raw_os_error().unwrap_or(libc::EACCES))
        }
    }
}

/// Perform an INET/INET6 operation as the workload wrote it, from the raw
/// sockaddr bytes the supervisor read once: port already checked against
/// the grant, payload already copied, so the kernel never re-reads workload
/// memory and no double-fetch exists. A `connect` pins the destination; a
/// datagram send goes to exactly these bytes.
#[cfg(target_arch = "x86_64")]
fn broker_inet(notif: &Notif, raw: &[u8], workload_pid: libc::pid_t) -> Result<i64, i32> {
    let payload = broker_payload(notif, notif.nr == 46, workload_pid)?;
    unsafe {
        let fd = dup_workload_fd(workload_pid, notif.args[0] as i32).map_err(|_| libc::EACCES)?;
        let result = (|| {
            if notif.nr == 42 {
                let rc = libc::connect(
                    fd,
                    raw.as_ptr() as *const libc::sockaddr,
                    raw.len() as libc::socklen_t,
                );
                if rc != 0 {
                    return Err(std::io::Error::last_os_error()
                        .raw_os_error()
                        .unwrap_or(libc::EACCES));
                }
                return Ok(0i64);
            }
            let sent = if notif.nr != 46 {
                libc::sendto(
                    fd,
                    payload.as_ptr() as *const libc::c_void,
                    payload.len(),
                    notif.args[3] as i32,
                    raw.as_ptr() as *const libc::sockaddr,
                    raw.len() as libc::socklen_t,
                )
            } else {
                let mut header: libc::msghdr = std::mem::zeroed();
                let mut iov = libc::iovec {
                    iov_base: payload.as_ptr() as *mut libc::c_void,
                    iov_len: payload.len(),
                };
                header.msg_iov = &mut iov;
                header.msg_iovlen = 1;
                header.msg_name = raw.as_ptr() as *mut libc::c_void;
                header.msg_namelen = raw.len() as libc::socklen_t;
                libc::sendmsg(fd, &header, notif.args[2] as i32)
            };
            if sent < 0 {
                return Err(std::io::Error::last_os_error()
                    .raw_os_error()
                    .unwrap_or(libc::EACCES));
            }
            Ok(sent as i64)
        })();
        libc::close(fd);
        result
    }
}

/// Address-less send (Target::None): perform the send on a duplicate of the
/// workload's already-connected socket from copied payload bytes. The
/// destination was fixed by an earlier brokered connect and cannot change.
#[cfg(target_arch = "x86_64")]
fn broker_connected_send(
    notif: &Notif,
    workload_pid: libc::pid_t,
    sendmsg: bool,
) -> Result<i64, i32> {
    let payload = broker_payload(notif, sendmsg, workload_pid)?;
    unsafe {
        let fd = dup_workload_fd(workload_pid, notif.args[0] as i32).map_err(|_| libc::EACCES)?;
        let sent = if !sendmsg {
            libc::send(
                fd,
                payload.as_ptr() as *const libc::c_void,
                payload.len(),
                notif.args[3] as i32,
            )
        } else {
            let mut header: libc::msghdr = std::mem::zeroed();
            let mut iov = libc::iovec {
                iov_base: payload.as_ptr() as *mut libc::c_void,
                iov_len: payload.len(),
            };
            header.msg_iov = &mut iov;
            header.msg_iovlen = 1;
            libc::sendmsg(fd, &header, notif.args[2] as i32)
        };
        libc::close(fd);
        if sent < 0 {
            Err(libc::EACCES)
        } else {
            Ok(sent as i64)
        }
    }
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;

    /// Interpret the generated BPF program over synthetic `seccomp_data`
    /// (the five opcodes it uses), so the trap set is proven in-process on
    /// hosts where the filter itself cannot be installed.
    fn run(program: &[libc::sock_filter], arch: u32, nr: i32) -> u32 {
        let mut accumulator = 0u32;
        let mut pc = 0usize;
        while pc < program.len() {
            let ins = &program[pc];
            match ins.code {
                0x20 => accumulator = word_of(arch, nr, ins.k), // BPF_LD|BPF_W|BPF_ABS
                0x15 => {
                    // BPF_JMP|BPF_JEQ|BPF_K
                    pc += if accumulator == ins.k {
                        ins.jt as usize
                    } else {
                        ins.jf as usize
                    };
                }
                0x06 => return ins.k, // BPF_RET|BPF_K
                _ => panic!("unexpected opcode {}", ins.code),
            }
            pc += 1;
        }
        panic!("program fell through");
    }

    fn word_of(arch: u32, nr: i32, offset: u32) -> u32 {
        match offset {
            0 => nr as u32,
            4 => arch,
            _ => panic!("unexpected offset {offset}"),
        }
    }

    #[test]
    fn filter_traps_exactly_the_monitored_syscalls_both_abi_aliases() {
        let program = bpf_program();
        for syscall in MONITORED {
            for alias in [0, X32] {
                let number = (syscall as u64 + alias) as u32;
                let action = run(&program, AUDIT_ARCH_X86_64, number as i32);
                assert_eq!(action, SECCOMP_RET_USER_NOTIF, "nr {number} must trap");
            }
        }
        // Everything else on x86_64 passes through (the stacked deny filter
        // governs its own set).
        for other in [1, 2, 41, 43, 45, 257, 435] {
            let action = run(&program, AUDIT_ARCH_X86_64, other);
            assert_eq!(action, SECCOMP_RET_ALLOW, "nr {other} must pass");
        }
        // Foreign audit architectures are not trapped by this filter.
        assert_eq!(run(&program, 0x40000028, 42), SECCOMP_RET_ALLOW);
    }

    fn notif(nr: i32, args: [u64; 6]) -> Notif {
        Notif {
            id: 1,
            pid: std::process::id(),
            flags: 0,
            nr,
            arch: AUDIT_ARCH_X86_64,
            _ip: 0,
            args,
        }
    }

    #[test]
    fn decodes_unix_pathname_and_grants_by_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.sock");
        std::os::unix::net::UnixListener::bind(&path).unwrap();
        let textual = format!("{}\0", path.display());
        let sockaddr = [
            (libc::AF_UNIX as u16).to_le_bytes().as_slice(),
            textual.as_bytes(),
        ]
        .concat();
        let address = sockaddr.as_ptr() as u64;
        let notif = notif(42, [3, address, sockaddr.len() as u64, 0, 0, 0]);
        match target_of(&notif) {
            Target::Pathname(Some(decoded)) => {
                assert_eq!(decoded, path.canonicalize().unwrap());
            }
            other => panic!("expected pathname, got {other:?}"),
        }
    }

    #[test]
    fn abstract_is_refused_and_inet_carries_its_port() {
        // Abstract: AF_UNIX, sun_path[0] == 0 -> refused (no continuation).
        let abstract_sock: Vec<u8> = [
            (libc::AF_UNIX as u16).to_le_bytes().as_slice(),
            &[0u8, 0, 0, 0],
        ]
        .concat();
        let address = abstract_sock.as_ptr() as u64;
        let mut args = [0u64; 6];
        args[1] = address;
        args[2] = abstract_sock.len() as u64;
        assert_eq!(target_of(&notif(42, args)), Target::Abstract);
        // AF_INET carries the port in network byte order for the grant check.
        let inet: Vec<u8> = [
            (libc::AF_INET as u16).to_le_bytes().as_slice(),
            &[0x1f, 0x90, 127, 0, 0, 1],
        ]
        .concat();
        let address = inet.as_ptr() as u64;
        let mut args = [0u64; 6];
        args[1] = address;
        args[2] = inet.len() as u64;
        match target_of(&notif(42, args)) {
            Target::Inet(8080, raw) => assert_eq!(raw, inet),
            other => panic!("expected inet, got {other:?}"),
        }
    }

    #[test]
    fn connect_without_address_is_none() {
        let notif = notif(42, [3, 0, 0, 0, 0, 0]);
        assert_eq!(target_of(&notif), Target::None);
    }

    #[test]
    fn peer_verification_requires_an_absolute_path_under_a_grant() {
        let grant = PathBuf::from("/run/postgresql");
        let granted = vec![grant.clone()];
        assert!(peer_under_grant(
            Some(PathBuf::from("/run/postgresql/.s.PGSQL.5432")),
            &granted
        ));
        // A sibling prefix is not a grant ("/run/postgresql-evil" is not
        // under "/run/postgresql").
        assert!(!peer_under_grant(
            Some(PathBuf::from("/run/postgresql-evil/s")),
            &granted
        ));
        // Abstract/unnamed peers (None) and outside trees are refused.
        assert!(!peer_under_grant(None, &granted));
        assert!(!peer_under_grant(
            Some(PathBuf::from("/tmp/canary")),
            &granted
        ));
        // With no grants, no peer is acceptable.
        assert!(!peer_under_grant(Some(grant), &[]));
    }

    #[test]
    fn unresolvable_pathname_refuses() {
        let missing: Vec<u8> = [
            (libc::AF_UNIX as u16).to_le_bytes().as_slice(),
            b"/nonexistent-ux-gate/s.sock\0",
        ]
        .concat();
        let mut args = [0u64; 6];
        args[1] = missing.as_ptr() as u64;
        args[2] = missing.len() as u64;
        let notif = notif(42, args);
        assert_eq!(target_of(&notif), Target::Pathname(None));
    }
}
