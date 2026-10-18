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
#[cfg(target_os = "linux")]
fn main() {
    use std::io::Read;
    use std::os::unix::process::{CommandExt, ExitStatusExt};
    use std::path::PathBuf;

    fn fail(message: &str) -> ! {
        eprintln!("sea-forge-workload-exec: {message}");
        std::process::exit(126);
    }

    let mut args = std::env::args_os().skip(1);
    let (mut workspace, mut artifacts, mut reads, mut ports) = (None, None, Vec::new(), Vec::new());
    let mut lifeline = None;
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
    let network = sea_forge_sandbox::NetworkPosture::from_granted_ports(ports);
    if let Err(error) =
        sea_forge_sandbox::jail::restrict_current_thread(&workspace, &artifacts, &reads, &network)
    {
        fail(&error.to_string());
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
    // Reaped below by `wait(-1)` as the namespace's init, with every orphan.
    #[allow(clippy::zombie_processes)]
    let child = command
        .spawn()
        .unwrap_or_else(|e| fail(&format!("spawn failed: {e}")));
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

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("sea-forge-workload-exec: isolated workloads require Linux");
    std::process::exit(126);
}
