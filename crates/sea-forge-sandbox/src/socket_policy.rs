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
//! - ABI < 9 (x86_64 only): a seccomp user-notification filter on the three
//!   syscalls, served by a supervisor thread in the trusted
//!   `sea-forge-workload-exec` helper. Every pathname target is resolved and
//!   checked against the granted trees before the syscall continues; the
//!   syscall fails with `EACCES` otherwise. Two TOCTOU residuals (pathname
//!   double-fetch; rename between check and continue) cannot be closed by
//!   any supervisor; the compensating control is an immediate post-allow
//!   re-resolution that kills the workload when the validated socket inode
//!   is no longer what was allowed. Abstract sockets are continued untouched
//!   — the Landlock abstract-socket scope and the dedicated network
//!   namespace already govern them — and non-UNIX families pass through to
//!   the main ruleset's TCP/UDP rights.
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
// connect(2), sendto(2), sendmsg(2); libc send(2) is sendto with no address.
#[cfg(target_arch = "x86_64")]
const MONITORED: [i64; 3] = [42, 44, 46];

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

/// The destination a monitored syscall targets.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Target {
    /// No explicit address (send on an already-connected socket): the
    /// connection that carried it was itself gated at connect time.
    None,
    /// AF_UNIX with a pathname, in resolved absolute form (empty when the
    /// path did not resolve — refused).
    Pathname(Option<PathBuf>),
    /// Abstract AF_UNIX or another family: not this policy's decision.
    Passthrough,
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
                return Target::Passthrough;
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
        return Target::None;
    }
    let Some(sockaddr) = read_remote(pid, address, length) else {
        return Target::Passthrough;
    };
    let family = u16::from_le_bytes([sockaddr[0], sockaddr[1]]);
    if family != libc::AF_UNIX as u16 {
        return Target::Passthrough;
    }
    if sockaddr[2] == 0 {
        return Target::Passthrough; // abstract: Landlock scope + netns govern
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

/// Serve pathname-socket policy on `listener` until process exit. Blocks
/// the calling thread; intended for a dedicated trusted thread in the
/// helper, with `workload_pid` its direct child.
///
/// A pathname target is continued only when its resolved path lies under a
/// granted tree and is a socket inode; otherwise the syscall receives
/// `EACCES`. After continuing, the validated inode is re-resolved once; a
/// changed or vanished inode means the allow was raced in the window no
/// supervisor can close, so the workload is killed (documented compensating
/// control, ADR-014 §4 C).
#[cfg(target_arch = "x86_64")]
pub fn supervise(listener: i32, grants: Vec<PathBuf>, workload_pid: libc::pid_t) {
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
        let target = target_of(&notif);
        let mut allowed_inode: Option<(u64, u64)> = None;
        let error = match &target {
            Target::None | Target::Passthrough => None,
            Target::Pathname(None) => Some(libc::EACCES), // unresolvable: refuse
            Target::Pathname(Some(path)) => {
                let under_grant = granted.iter().any(|tree| path.starts_with(tree));
                match under_grant.then(|| socket_inode(path)) {
                    Some(Some(inode)) => {
                        allowed_inode = Some(inode);
                        None
                    }
                    _ => Some(libc::EACCES),
                }
            }
        };
        let resp = NotifResp {
            id: notif.id,
            val: 0,
            error: error.unwrap_or(0),
            flags: 0,
        };
        unsafe { libc::ioctl(listener, IOCTL_NOTIF_SEND as _, &resp) };
        // Compensating re-check for the double-fetch / rename windows.
        if let (Some((dev, ino)), Target::Pathname(Some(path))) = (allowed_inode, &target) {
            if socket_inode(path) != Some((dev, ino)) {
                unsafe { libc::kill(workload_pid, libc::SIGKILL) };
            }
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
    fn abstract_and_foreign_families_pass_through() {
        // Abstract: AF_UNIX, sun_path[0] == 0.
        let abstract_sock: Vec<u8> = [
            (libc::AF_UNIX as u16).to_le_bytes().as_slice(),
            &[0u8, 0, 0, 0],
        ]
        .concat();
        // AF_INET loopback: another family entirely.
        let inet: Vec<u8> = [
            (libc::AF_INET as u16).to_le_bytes().as_slice(),
            &[7, 0, 0x1f, 0x90, 127, 0, 0, 1],
        ]
        .concat();
        for bytes in [&abstract_sock[..], &inet[..]] {
            let address = bytes.as_ptr() as u64;
            let mut args = [0u64; 6];
            args[1] = address;
            args[2] = bytes.len() as u64;
            let notif = notif(42, args);
            assert_eq!(target_of(&notif), Target::Passthrough);
        }
    }

    #[test]
    fn connect_without_address_is_none() {
        let notif = notif(42, [3, 0, 0, 0, 0, 0]);
        assert_eq!(target_of(&notif), Target::None);
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
