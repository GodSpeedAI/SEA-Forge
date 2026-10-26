use sea_forge_core::errors::ForgeError;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

pub fn list(root: &Path) -> Result<(), ForgeError> {
    let response = request(root, json!({"verb":"agent_list"}))?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

pub fn probe(
    root: &Path,
    endpoint: &str,
    prompt: &str,
    model: Option<&str>,
    policy: &str,
    entity: &str,
    process: &str,
) -> Result<u8, ForgeError> {
    let response = request(
        root,
        json!({
            "verb": "agent_probe",
            "endpoint": endpoint,
            "prompt": prompt,
            "model": model,
            "policy": policy,
            "entity": entity,
            "process": process,
        }),
    )?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    if response.get("error").is_some() {
        return Err(ForgeError::Input(
            response["error"]
                .as_str()
                .unwrap_or("agent probe failed")
                .into(),
        ));
    }
    // Exit codes follow repo convention (spec-minimum §12.2): accepted = 0,
    // rejected settlement = 3. Internal errors surface as 1 via the caller.
    Ok(if response["settlement"] == "accepted" {
        0
    } else {
        3
    })
}

#[allow(clippy::too_many_arguments)]
pub fn delegate(
    root: &Path,
    endpoint: &str,
    instruction: &str,
    run_id: Option<&str>,
    model: Option<&str>,
    max_turns: u32,
    token_budget: Option<u64>,
    agent_output_must_contain: Option<&str>,
    policy: &str,
    entity: &str,
    process: &str,
) -> Result<u8, ForgeError> {
    let criteria =
        agent_output_must_contain.map(|needle| json!({"agent_output_must_contain": needle}));
    let response = request(
        root,
        json!({
            "verb": "delegate",
            "endpoint": endpoint,
            "instruction": instruction,
            "run_id": run_id,
            "model": model,
            "max_turns": max_turns,
            "token_budget": token_budget,
            "criteria": criteria,
            "policy": policy,
            "entity": entity,
            "process": process,
        }),
    )?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    if response.get("error").is_some() {
        return Err(ForgeError::Input(
            response["error"]
                .as_str()
                .unwrap_or("agent delegation failed")
                .into(),
        ));
    }
    Ok(if response["settlement"] == "accepted" {
        0
    } else {
        3
    })
}

pub fn cancel(
    root: &Path,
    run_id: &str,
    policy: &str,
    entity: &str,
    process: &str,
) -> Result<u8, ForgeError> {
    let response = request(
        root,
        json!({
            "verb": "cancel_delegation",
            "run_id": run_id,
            "policy": policy,
            "entity": entity,
            "process": process,
        }),
    )?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    if response.get("error").is_some() {
        return Err(ForgeError::Input(
            response["error"]
                .as_str()
                .unwrap_or("agent delegation cancellation failed")
                .into(),
        ));
    }
    Ok(0)
}

pub(crate) fn request(root: &Path, mut request: Value) -> Result<Value, ForgeError> {
    // SF-005 / U-07: protected verbs carry an actor block. The CLI claims the
    // entity it was invoked as; the server still checks that claim against the
    // connection's uid, so asserting it here grants nothing the cell has not
    // already bound. Requests that carry no `entity` are inspect verbs, which
    // need no actor.
    if let Some(entity) = request
        .get("entity")
        .and_then(Value::as_str)
        .map(str::to_owned)
    {
        if let Some(object) = request.as_object_mut() {
            object
                .entry("actor")
                .or_insert_with(|| json!({"actor_id": entity, "role": "operator"}));
        }
    }
    let socket_path = root.join("server.sock");
    let line = exchange(&socket_path, &request)?;
    serde_json::from_str(line.trim()).map_err(ForgeError::from)
}

/// Unix: connect to the server socket, send one request line, read one response line.
#[cfg(unix)]
fn exchange(socket_path: &Path, request: &Value) -> Result<String, ForgeError> {
    let mut stream = UnixStream::connect(socket_path)
        .map_err(|error| ForgeError::io("connect to sea-forge-server", error))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(120)))
        .map_err(|error| ForgeError::io("configure server timeout", error))?;
    stream
        .write_all(format!("{}\n", serde_json::to_string(request)?).as_bytes())
        .map_err(|error| ForgeError::io("write server request", error))?;
    stream
        .shutdown(std::net::Shutdown::Write)
        .map_err(|error| ForgeError::io("close server request", error))?;
    let mut line = String::new();
    BufReader::new(stream)
        .read_line(&mut line)
        .map_err(|error| ForgeError::io("read server response", error))?;
    Ok(line)
}

/// Windows: open the server's named pipe, verify the server runs as this user
/// before sending anything, then exchange one request line and one response.
///
/// The response read has the same 120 s deadline as the Unix client. A
/// synchronous pipe handle has no read timeout, so the read runs on a helper
/// thread and the caller stops waiting after 120 s. The helper then stays
/// blocked on the handle until the process exits. A synchronous pipe also has
/// no half-close, so the request is delimited by its newline alone.
#[cfg(windows)]
fn exchange(socket_path: &Path, request: &Value) -> Result<String, ForgeError> {
    use std::os::windows::fs::OpenOptionsExt;
    let pipe_name = sea_forge_core::pipe_name_for(socket_path)
        .map_err(|error| ForgeError::io("derive server pipe name", error))?;
    // Identification-only QoS: the server can read this caller's SID but cannot
    // impersonate it, so a squatting pipe server gains no client authority.
    // Explicit access rights avoid GENERIC_WRITE, which would request
    // FILE_APPEND_DATA; the server's DACL does not grant that right.
    let mut pipe = std::fs::OpenOptions::new()
        .access_mode(CLIENT_PIPE_ACCESS)
        .security_qos_flags(SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION)
        .open(&pipe_name)
        .map_err(|error| ForgeError::io("connect to sea-forge-server", error))?;
    let server = sea_forge_os::pipe_server_sid(&pipe)
        .map_err(|error| ForgeError::io("read server pipe identity", error))?;
    let expected = expected_server_sid()
        .map_err(|error| ForgeError::io("read current user identity", error))?;
    if !server_identity_matches(&server, &expected) {
        return Err(ForgeError::Input(format!(
            "refusing pipe {pipe_name}: its server runs as {server}, not the expected server principal {expected}"
        )));
    }
    pipe.write_all(format!("{}\n", serde_json::to_string(request)?).as_bytes())
        .map_err(|error| ForgeError::io("write server request", error))?;
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        let result = BufReader::new(pipe).read_line(&mut line).map(|_| line);
        // The receiver is gone after a timeout; the send result is then unused.
        let _ = sender.send(result);
    });
    let line = receiver
        .recv_timeout(Duration::from_secs(120))
        .map_err(|_| {
            ForgeError::io(
                "read server response",
                std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "no response from sea-forge-server within 120 s",
                ),
            )
        })?
        .map_err(|error| ForgeError::io("read server response", error))?;
    Ok(line)
}

/// Optional trusted server principal. Unset means the server must run as the
/// calling user (the default). Set it to a SID such as a service account when
/// the server runs under another identity that this user is admitted to.
#[cfg(windows)]
const EXPECTED_SERVER_SID_ENV: &str = "SEA_FORGE_SERVER_SID";
#[cfg(windows)]
const SECURITY_SQOS_PRESENT: u32 = 0x0010_0000;
#[cfg(windows)]
const SECURITY_IDENTIFICATION: u32 = 0x0001_0000;
#[cfg(windows)]
const CLIENT_PIPE_ACCESS: u32 = sea_forge_os::CLIENT_PIPE_RIGHTS;

/// The server SID this client accepts: the configured trusted principal when
/// `SEA_FORGE_SERVER_SID` is set, otherwise the calling user.
#[cfg(windows)]
fn expected_server_sid() -> std::io::Result<String> {
    match std::env::var(EXPECTED_SERVER_SID_ENV) {
        Ok(configured) if !configured.trim().is_empty() => {
            let sid = configured.trim().to_string();
            if !sid.starts_with("S-1-") {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("{EXPECTED_SERVER_SID_ENV} is not a SID"),
                ));
            }
            Ok(sid)
        }
        _ => sea_forge_os::current_sid(),
    }
}

/// The pipe's server SID must be exactly the expected principal. A squatted
/// pipe owned by another principal fails this comparison before any request leaves.
#[cfg(windows)]
fn server_identity_matches(server_sid: &str, expected_sid: &str) -> bool {
    !server_sid.is_empty() && server_sid == expected_sid
}

#[cfg(all(test, windows))]
mod pipe_identity_tests {
    use super::server_identity_matches;

    #[test]
    fn a_different_server_sid_is_refused() {
        assert!(!server_identity_matches(
            "S-1-5-21-1-2-3-1001",
            "S-1-5-21-1-2-3-1002"
        ));
        assert!(!server_identity_matches("", "S-1-5-21-1-2-3-1002"));
        assert!(server_identity_matches(
            "S-1-5-21-1-2-3-1002",
            "S-1-5-21-1-2-3-1002"
        ));
    }
}
