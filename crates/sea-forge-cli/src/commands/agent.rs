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
