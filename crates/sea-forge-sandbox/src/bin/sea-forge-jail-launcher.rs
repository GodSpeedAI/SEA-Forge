//! Trusted launcher for the macOS App Sandbox jail.
//!
//! `JailSandbox` copies this binary into a fresh private directory per
//! execution, signs the copy with App Sandbox entitlements that name the
//! workspace and artifacts trees, and runs `<copy> argv...`. The kernel applies
//! the sandbox at exec time from the signature, and the sandbox is inherited by
//! every descendant. This helper therefore does exactly one thing: it replaces
//! itself with `argv[1..]` and keeps the process id, so the caller's timeout and
//! process-group handling act on the workload directly.
//!
//! Usage: `sea-forge-jail-launcher <program> [args...]`
#[cfg(target_os = "macos")]
fn main() {
    use std::os::unix::process::CommandExt;

    let mut args = std::env::args_os().skip(1);
    let Some(program) = args.next() else {
        eprintln!("usage: sea-forge-jail-launcher <argv...>");
        std::process::exit(2);
    };
    let error = std::process::Command::new(&program).args(args).exec();
    eprintln!(
        "sea-forge-jail-launcher: exec {} failed: {error}",
        program.to_string_lossy()
    );
    std::process::exit(127);
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("sea-forge-jail-launcher: the macOS App Sandbox launcher only runs on macOS");
    std::process::exit(1);
}
