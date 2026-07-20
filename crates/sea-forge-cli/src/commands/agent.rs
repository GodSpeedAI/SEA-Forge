use sea_forge_core::errors::ForgeError;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
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
    model: Option<&str>,
    max_turns: u32,
    token_budget: Option<u64>,
    policy: &str,
    entity: &str,
    process: &str,
) -> Result<u8, ForgeError> {
    let response = request(
        root,
        json!({
            "verb": "delegate",
            "endpoint": endpoint,
            "instruction": instruction,
            "model": model,
            "max_turns": max_turns,
            "token_budget": token_budget,
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

fn request(root: &Path, request: Value) -> Result<Value, ForgeError> {
    let socket_path = root.join("server.sock");
    let mut stream = UnixStream::connect(&socket_path)
        .map_err(|error| ForgeError::io("connect to sea-forge-server", error))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(120)))
        .map_err(|error| ForgeError::io("configure server timeout", error))?;
    stream
        .write_all(format!("{}\n", serde_json::to_string(&request)?).as_bytes())
        .map_err(|error| ForgeError::io("write server request", error))?;
    stream
        .shutdown(std::net::Shutdown::Write)
        .map_err(|error| ForgeError::io("close server request", error))?;
    let mut line = String::new();
    BufReader::new(stream)
        .read_line(&mut line)
        .map_err(|error| ForgeError::io("read server response", error))?;
    serde_json::from_str(line.trim()).map_err(ForgeError::from)
}
