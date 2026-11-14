//! S1 gate driver (WP-19): replays SFWP frames against a real `run_authenticated` server.
//!
//!   fde_s1_driver <server-config.json> <frames.ndjson> <responses.ndjson>
//!
//! `server-config.json` is a `ServerConfig` (the cell root is its `root`). Each line of
//! `frames.ndjson` is `{"actor":{"actor_id":"..","role":".."},"frame":{..}}`; the frame is sent over a
//! private authenticated connection minted for that actor by the trusted-issuer test driver
//! (`tests/support/authenticated.rs`), exactly as the conformance tests do. This proves capability
//! authentication, not isolated host workload identity: assurance stays `session` (pre-S3).
//! One line of `responses.ndjson` is written per frame. State lives on disk under the cell root, so
//! the process can be restarted between steps.
#![cfg(unix)]
#![allow(dead_code)]

#[path = "../tests/support/authenticated.rs"]
mod authenticated;

use sea_forge_core::types::ActorRole;
use sea_forge_server::ServerConfig;
use serde_json::{json, Value};
use std::io::Write;

fn arg(n: usize, what: &str) -> String {
    std::env::args()
        .nth(n)
        .unwrap_or_else(|| panic!("usage: fde_s1_driver <server-config.json> <frames.ndjson> <responses.ndjson> (missing {what})"))
}

#[tokio::main]
async fn main() {
    let config: ServerConfig =
        serde_json::from_str(&std::fs::read_to_string(arg(1, "config")).unwrap())
            .expect("server config");
    let frames = std::fs::read_to_string(arg(2, "frames")).unwrap();
    let mut out = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(arg(3, "responses"))
        .unwrap();
    let mut cell = authenticated::Cell::start(config).await;
    for (i, line) in frames.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        let step: Value = serde_json::from_str(line).expect("frame line");
        let actor_id = step["actor"]["actor_id"].as_str().expect("actor_id");
        let role: ActorRole = serde_json::from_value(step["actor"]["role"].clone()).expect("role");
        let actor = authenticated::principal(actor_id, role);
        let response = cell.call_for(&actor, step["frame"].clone()).await;
        let row = json!({"i": i, "actor": step["actor"], "request_id": step["frame"]["request_id"], "response": response});
        writeln!(out, "{row}").unwrap();
    }
    cell.shutdown().await;
}
