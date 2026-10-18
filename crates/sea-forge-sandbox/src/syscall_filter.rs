//! Seccomp filter for isolated workloads (SEC-06).
//!
//! Landlock already confines what a workload can open, connect to and
//! signal, and refuses mount-topology changes even inside a namespace the
//! workload creates. What it leaves reachable is kernel surface: io_uring,
//! and new namespaces, whose capabilities unlock most of the kernel's
//! privileged code. This filter refuses exactly those, nothing more:
//!
//! - `io_uring_setup`, `io_uring_enter`, `io_uring_register`: `EPERM`;
//! - `unshare`, `clone` with any namespace flag, and `setns`: `EPERM`;
//! - `clone3`: `ENOSYS`, since its flags sit in memory a filter cannot
//!   inspect; libc then falls back to `clone`, whose flags are checked.
//!
//! On x86_64 each syscall is refused at its x32 number too: x32 calls share
//! the x86_64 audit architecture, so the architecture check alone would let
//! them through. Any other architecture's calls (i386 `int 0x80`) fail that
//! check and kill the process. Installation needs `no_new_privs`, which the
//! Landlock restriction sets first; on an architecture without a filter
//! definition installation fails rather than running unfiltered.
#![cfg(target_os = "linux")]

use crate::SandboxError;
use seccompiler::{
    apply_filter, BpfProgram, SeccompAction, SeccompCmpArgLen, SeccompCmpOp, SeccompCondition,
    SeccompFilter, SeccompRule, TargetArch,
};
use std::collections::BTreeMap;

/// `CLONE_NEW*`: mount, cgroup, uts, ipc, user, pid, net, time.
const NAMESPACE_FLAGS: [u64; 8] = [
    0x0002_0000,
    0x0200_0000,
    0x0400_0000,
    0x0800_0000,
    0x1000_0000,
    0x2000_0000,
    0x4000_0000,
    0x0000_0080,
];

#[cfg(target_arch = "x86_64")]
const ABI_ALIASES: [i64; 2] = [0, 0x4000_0000];
#[cfg(not(target_arch = "x86_64"))]
const ABI_ALIASES: [i64; 1] = [0];

fn unavailable(error: impl std::fmt::Display) -> SandboxError {
    SandboxError::new("syscall_filter_unavailable", error.to_string())
}

/// Rules matching when argument 0 carries any namespace flag.
fn namespace_flag_rules() -> Result<Vec<SeccompRule>, SandboxError> {
    NAMESPACE_FLAGS
        .iter()
        .map(|&flag| {
            SeccompCondition::new(
                0,
                SeccompCmpArgLen::Qword,
                SeccompCmpOp::MaskedEq(flag),
                flag,
            )
            .and_then(|condition| SeccompRule::new(vec![condition]))
            .map_err(unavailable)
        })
        .collect()
}

fn program(
    arch: TargetArch,
    rules: Vec<(i64, Vec<SeccompRule>)>,
    errno: i32,
) -> Result<BpfProgram, SandboxError> {
    let mut map = BTreeMap::new();
    for (nr, rule) in rules {
        for alias in ABI_ALIASES {
            map.insert(nr | alias, rule.clone());
        }
    }
    SeccompFilter::new(
        map,
        SeccompAction::Allow,
        SeccompAction::Errno(errno as u32),
        arch,
    )
    .and_then(BpfProgram::try_from)
    .map_err(unavailable)
}

/// Install the workload filter on the calling thread; every thread and
/// process it later creates inherits it, and it cannot be removed.
pub fn restrict_current_thread() -> Result<(), SandboxError> {
    let arch = TargetArch::try_from(std::env::consts::ARCH).map_err(unavailable)?;
    let namespace = namespace_flag_rules()?;
    let refused = program(
        arch,
        vec![
            (libc::SYS_io_uring_setup, vec![]),
            (libc::SYS_io_uring_enter, vec![]),
            (libc::SYS_io_uring_register, vec![]),
            (libc::SYS_setns, vec![]),
            (libc::SYS_unshare, namespace.clone()),
            (libc::SYS_clone, namespace),
        ],
        libc::EPERM,
    )?;
    let opaque = program(arch, vec![(libc::SYS_clone3, vec![])], libc::ENOSYS)?;
    apply_filter(&refused).map_err(unavailable)?;
    apply_filter(&opaque).map_err(unavailable)
}
