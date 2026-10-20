//! Trusted in-namespace step of an isolated workload launch.
//!
//! `unshare` has already switched to the workload's subordinate identity and
//! made this helper pid 1 of a fresh pid namespace. The helper applies the
//! Landlock posture and the syscall filter to itself, then runs the workload
//! as its child, so both are in force before any workload code runs.
//!
//! With `--lifeline FD`, the helper stays as the namespace's init: it reaps
//! every orphan and exits as soon as the launcher's end of the lifeline pipe
//! closes (the launcher exited or was killed). The kernel then kills every
//! process in the pid namespace, so no workload outlives its launcher. The
//! user-namespace switch clears any parent-death signal, which is why this
//! does not rely on one.
//!
//! Usage: `sea-forge-workload-exec --workspace P --artifacts P
//! [--read P]... [--tcp-port N]... [--lifeline FD] -- argv...`
// The workspace denies unsafe so it is always a stated choice; the one raw
// close_range syscall below is that choice (inherited-descriptor hygiene).
#![allow(unsafe_code)]
#[cfg(target_os = "linux")]
fn main() {
    use std::io::Read;
    use std::os::unix::process::{CommandExt, ExitStatusExt};
    use std::path::PathBuf;

    // SEC-06 inherited-descriptor hygiene: everything above stdio arrives
    // marked close-on-exec, so no descriptor opened before the restriction
    // applied (a connected socket, a privileged file) can leak into the
    // workload across its exec. The lifeline read end is deliberately
    // non-cloexec across the launch chain; it is re-opened by path below
    // after this marking, and that re-opened description is close-on-exec.
    // Safety: one raw syscall with constant arguments, before any thread or
    // restriction exists; it cannot touch memory.
    unsafe {
        libc::syscall(libc::SYS_close_range, 3u32, u32::MAX, 4u32); // CLOSE_RANGE_CLOEXEC
    }

    fn fail(message: &str) -> ! {
        eprintln!("sea-forge-workload-exec: {message}");
        std::process::exit(126);
    }

    let mut args = std::env::args_os().skip(1);
    let (mut workspace, mut artifacts, mut reads, mut ports) = (None, None, Vec::new(), Vec::new());
    let (mut socket_grants, mut lifeline) = (Vec::new(), None);
    loop {
        let flag = args.next().unwrap_or_else(|| fail("missing -- separator"));
        match flag.to_str() {
            Some("--") => break,
            Some("--workspace") => workspace = args.next().map(PathBuf::from),
            Some("--artifacts") => artifacts = args.next().map(PathBuf::from),
            Some("--read") => reads.push(
                args.next()
                    .map(PathBuf::from)
                    .filter(|p| p.is_absolute())
                    .unwrap_or_else(|| fail("invalid --read")),
            ),
            Some("--socket-grant") => socket_grants.push(
                args.next()
                    .map(PathBuf::from)
                    .filter(|p| p.is_absolute())
                    .unwrap_or_else(|| fail("invalid --socket-grant")),
            ),
            Some("--tcp-port") => ports.push(
                args.next()
                    .and_then(|p| p.to_str()?.parse::<u16>().ok())
                    .unwrap_or_else(|| fail("invalid --tcp-port")),
            ),
            Some("--lifeline") => {
                let fd = args
                    .next()
                    .and_then(|p| p.to_str()?.parse::<u32>().ok())
                    .unwrap_or_else(|| fail("invalid --lifeline"));
                // Reopen the inherited read end by path (no unsafe fd
                // adoption); this description is close-on-exec.
                lifeline = Some(
                    std::fs::File::open(format!("/proc/self/fd/{fd}"))
                        .unwrap_or_else(|e| fail(&format!("lifeline unavailable: {e}"))),
                );
            }
            _ => fail("unknown argument"),
        }
    }
    let (Some(workspace), Some(artifacts)) = (workspace, artifacts) else {
        fail("--workspace and --artifacts are required");
    };
    let argv: Vec<_> = args.collect();
    if argv.is_empty() {
        fail("empty workload argv");
    }
    let network = sea_forge_sandbox::NetworkPosture::from_granted_ports(ports.clone());
    // The supervisor thread is created BEFORE any restriction is applied, so
    // it inherits neither the seccomp filters nor the Landlock domain: its
    // own brokered connect/send calls must not trap on the listener it
    // serves (a filtered supervisor deadlocks on the first granted
    // connect), and its canonicalization must see granted trees the
    // workload cannot read. It parks until the listener and the workload
    // pid arrive; with no supervisor variant it never wakes.
    let (supervisor_tx, supervisor_rx) =
        std::sync::mpsc::channel::<(i32, Vec<PathBuf>, Vec<u16>, i32)>();
    #[cfg(target_arch = "x86_64")]
    std::thread::spawn(move || {
        if std::env::var_os("SEA_FORGE_SOCKET_SUPERVISOR_DEBUG").is_some() {
            eprintln!("[workload-exec] supervisor thread parked");
        }
        if let Ok((listener, grants, granted_ports, pid)) = supervisor_rx.recv() {
            if std::env::var_os("SEA_FORGE_SOCKET_SUPERVISOR_DEBUG").is_some() {
                eprintln!("[workload-exec] supervisor serving listener={listener} pid={pid}");
            }
            sea_forge_sandbox::socket_policy::supervise(listener, grants, granted_ports, pid);
        }
    });
    #[cfg(not(target_arch = "x86_64"))]
    drop(supervisor_rx);
    if let Err(error) =
        sea_forge_sandbox::jail::restrict_current_thread(&workspace, &artifacts, &reads, &network)
    {
        fail(&error.to_string());
    }
    // Pathname Unix-socket containment (SEC-06, ADR-014), probe-gated between
    // a Landlock RESOLVE_UNIX ruleset and the seccomp supervisor. The
    // workload's own trees are always granted: any socket there was created
    // by the workload itself (DAC), which is the legitimate-IPC allowance.
    // Fail closed on any setup error.
    let mut socket_reach = socket_grants;
    socket_reach.push(workspace.clone());
    socket_reach.push(artifacts.clone());
    let seccomp_filters_before = seccomp_filter_count();
    let socket_listener = match sea_forge_sandbox::socket_policy::apply(&socket_reach) {
        Ok(sea_forge_sandbox::socket_policy::Applied::LandlockResolveUnix) => None,
        #[cfg(target_arch = "x86_64")]
        Ok(sea_forge_sandbox::socket_policy::Applied::SeccompSupervisor { listener }) => {
            Some(listener)
        }
        Err(error) => fail(&error.to_string()),
    };
    if std::env::var_os("SEA_FORGE_SOCKET_SUPERVISOR_DEBUG").is_some() {
        eprintln!(
            "[workload-exec] mechanism={:?} listener={:?} seccomp_filters {} -> {}",
            if socket_listener.is_some() {
                "supervisor"
            } else {
                "landlock"
            },
            socket_listener,
            seccomp_filters_before,
            seccomp_filter_count()
        );
    }
    // After Landlock (which sets no_new_privs), before any workload code.
    if let Err(error) = sea_forge_sandbox::syscall_filter::restrict_current_thread() {
        fail(&error.to_string());
    }
    let mut command = std::process::Command::new(&argv[0]);
    command.args(&argv[1..]);
    let Some(mut lifeline) = lifeline else {
        let error = command.exec();
        fail(&format!("exec failed: {error}"));
    };
    let supervisor = socket_listener.map(|listener| (supervisor_tx, listener));
    // Reaped below by `wait(-1)` as the namespace's init, with every orphan.
    #[allow(clippy::zombie_processes)]
    let child = command
        .spawn()
        .unwrap_or_else(|e| fail(&format!("spawn failed: {e}")));
    if let Some((tx, listener)) = supervisor {
        if std::env::var_os("SEA_FORGE_SOCKET_SUPERVISOR_DEBUG").is_some() {
            eprintln!(
                "[workload-exec] sending listener={listener} pid={} to supervisor",
                child.id()
            );
        }
        let sent = tx.send((listener, socket_reach, ports, child.id() as i32));
        if std::env::var_os("SEA_FORGE_SOCKET_SUPERVISOR_DEBUG").is_some() {
            eprintln!("[workload-exec] supervisor handoff sent={:?}", sent.is_ok());
        }
    }
    // EOF (or any error) on the lifeline means the launcher is gone.
    std::thread::spawn(move || {
        let mut byte = [0u8; 1];
        while matches!(lifeline.read(&mut byte), Ok(n) if n > 0) {}
        std::process::exit(137);
    });
    // As init, reap every child; leave with the workload's own status.
    let workload = child.id() as i32;
    loop {
        match rustix::process::wait(rustix::process::WaitOptions::empty()) {
            Ok(Some((pid, status))) if pid.as_raw_nonzero().get() == workload => {
                let status = std::process::ExitStatus::from_raw(status.as_raw());
                std::process::exit(
                    status
                        .code()
                        .unwrap_or_else(|| 128 + status.signal().unwrap_or(9)),
                );
            }
            Ok(_) => {}
            Err(_) => std::process::exit(126),
        }
    }
}

/// The process's attached seccomp filter count (/proc/self/status), for
/// the launch audit: the user-notify filter must increment it before the
/// workload is spawned.
#[cfg(target_os = "linux")]
fn seccomp_filter_count() -> u32 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find(|line| line.starts_with("Seccomp_filters:"))
                .and_then(|line| line.split_whitespace().last()?.parse().ok())
        })
        .unwrap_or(u32::MAX)
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("sea-forge-workload-exec: isolated workloads require Linux");
    std::process::exit(126);
}
