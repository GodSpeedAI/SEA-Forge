//! Trusted in-namespace step of an isolated workload launch.
//!
//! `unshare` has already switched to the workload's subordinate identity.
//! This helper applies the Landlock posture to itself and then execs the
//! workload, so the restriction is in force before any workload code runs.
//! Usage: `sea-forge-workload-exec --workspace P --artifacts P
//! [--tcp-port N]... -- argv...`
#[cfg(target_os = "linux")]
fn main() {
    use std::os::unix::process::CommandExt;
    use std::path::PathBuf;

    fn fail(message: &str) -> ! {
        eprintln!("sea-forge-workload-exec: {message}");
        std::process::exit(126);
    }

    let mut args = std::env::args_os().skip(1);
    let (mut workspace, mut artifacts, mut ports) = (None, None, Vec::new());
    loop {
        let flag = args.next().unwrap_or_else(|| fail("missing -- separator"));
        match flag.to_str() {
            Some("--") => break,
            Some("--workspace") => workspace = args.next().map(PathBuf::from),
            Some("--artifacts") => artifacts = args.next().map(PathBuf::from),
            Some("--tcp-port") => ports.push(
                args.next()
                    .and_then(|p| p.to_str()?.parse::<u16>().ok())
                    .unwrap_or_else(|| fail("invalid --tcp-port")),
            ),
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
        sea_forge_sandbox::jail::restrict_current_thread(&workspace, &artifacts, &network)
    {
        fail(&error.to_string());
    }
    let error = std::process::Command::new(&argv[0]).args(&argv[1..]).exec();
    fail(&format!("exec failed: {error}"));
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("sea-forge-workload-exec: isolated workloads require Linux");
    std::process::exit(126);
}
