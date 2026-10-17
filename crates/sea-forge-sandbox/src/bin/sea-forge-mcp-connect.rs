//! Workload-side stdio relay to a SEA Forge MCP broker socket.
//!
//! Untrusted: it only moves bytes. The broker authenticates the peer UID and
//! authorizes every call; bypassing this relay reaches nothing more.
#[cfg(unix)]
fn main() {
    use std::io::{copy, stdin, stdout, Write};
    use std::os::unix::net::UnixStream;

    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("usage: sea-forge-mcp-connect <socket>");
        std::process::exit(2);
    };
    let stream = match UnixStream::connect(&path) {
        Ok(stream) => stream,
        Err(error) => {
            eprintln!("sea-forge-mcp-connect: {error}");
            std::process::exit(1);
        }
    };
    let mut upstream = stream.try_clone().expect("clone broker stream");
    let writer = std::thread::spawn(move || {
        let _ = copy(&mut stdin().lock(), &mut upstream);
        let _ = upstream.shutdown(std::net::Shutdown::Write);
    });
    let mut reader = stream;
    let mut out = stdout().lock();
    let _ = copy(&mut reader, &mut out);
    let _ = out.flush();
    drop(writer);
}

#[cfg(not(unix))]
fn main() {
    eprintln!("sea-forge-mcp-connect requires Unix domain sockets");
    std::process::exit(2);
}
