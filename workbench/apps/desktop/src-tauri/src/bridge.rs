//! Closed, typed SFWP command surface (Task 3, host half).
//!
//! Boundary rule (`workbench/AGENTS.md`): the renderer never gets a generic
//! `invoke_backend(method, arbitrary_json)` escape hatch. Instead the host
//! exposes exactly two serde-tagged enums whose variants mirror the server's
//! `crate::Request` shapes byte-for-byte (same `verb` tag, same field names),
//! so `serde_json::to_value` produces a wire line the server already accepts.
//! Responses stay `serde_json::Value`: domain typing/validation happens on the
//! frontend against the generated TS + AJV contracts (Part A), keeping the Rust
//! host a thin typed-input / opaque-output bridge and NOT a second source of
//! domain types.

use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;
use tauri::State;

use crate::socket::SocketHandle;

/// Read-only ("inspect") SFWP methods. Field shapes copied from the server's
/// `Request` variants; `#[serde(skip_serializing_if = "Option::is_none")]`
/// mirrors the server's `#[serde(default)]` so omitted optionals stay absent on
/// the wire (never `null`), matching what the server's `#[serde(default)]`
/// deserialization expects.
#[derive(Deserialize, serde::Serialize)]
#[serde(tag = "verb", rename_all = "snake_case")]
pub enum SfwpQuery {
    SystemHello {
        protocol_version: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        client: Option<String>,
    },
    SystemDescribe,
    SystemGetSchema {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        method: Option<String>,
    },
    RequestGetStatus {
        request_id: String,
    },
    EventsGetRange {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        from_cursor: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to_cursor: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        limit: Option<u32>,
    },
}

/// A precondition bundle mirroring `sfwp::precondition::Precondition`. Kept as a
/// host-local shape (not a dependency on the server crate at build time) so the
/// production host stays workspace-isolated; the generated TS `Precondition`
/// type is the frontend's source of truth for the same shape.
#[derive(Deserialize, serde::Serialize)]
pub struct Precondition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_bundle_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub records: Vec<RecordDigest>,
}

/// One `{ref, expected_digest}` precondition entry (mirrors the server's
/// `RecordDigest`; `ref` is a Rust keyword so it stays raw-identified).
#[derive(Deserialize, serde::Serialize)]
pub struct RecordDigest {
    pub r#ref: String,
    pub expected_digest: String,
}

/// Protected ("command") SFWP verbs that carry a `request_id` for correlation
/// recovery. Field shapes copied faithfully from the server's `Request`
/// variants (`Submit` flattens `SubmitPayload`; here we spell the flattened
/// fields directly so the closed enum stays self-describing).
#[derive(Deserialize, serde::Serialize)]
#[serde(tag = "verb", rename_all = "snake_case")]
pub enum SfwpCommand {
    Submit {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        intent: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        plan: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        policy: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entity: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        process: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timeout: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        request_id: Option<String>,
    },
    Approve {
        case_id: String,
        approval_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        request_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        preconditions: Option<Precondition>,
    },
    Reject {
        case_id: String,
        approval_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        request_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        preconditions: Option<Precondition>,
    },
    Delegate {
        endpoint: String,
        instruction: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        run_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        max_turns: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        token_budget: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        policy: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entity: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        process: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        request_id: Option<String>,
    },
    CancelDelegation {
        run_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        policy: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entity: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        process: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        request_id: Option<String>,
    },
}

/// Issue a read-only SFWP query. Serialize the closed enum to a wire line and
/// pass it through the socket's single `call()` chokepoint.
#[tauri::command]
pub async fn sfwp_query(
    state: State<'_, Arc<SocketHandle>>,
    query: SfwpQuery,
) -> Result<Value, String> {
    let request = serde_json::to_value(&query).map_err(|e| e.to_string())?;
    state.call(request).await.map_err(|e| e.to_string())
}

/// Issue a protected SFWP command (correlated by `request_id`).
#[tauri::command]
pub async fn sfwp_command(
    state: State<'_, Arc<SocketHandle>>,
    command: SfwpCommand,
) -> Result<Value, String> {
    let request = serde_json::to_value(&command).map_err(|e| e.to_string())?;
    state.call(request).await.map_err(|e| e.to_string())
}

/// Thin convenience wrapper over `SfwpQuery::RequestGetStatus` — the correlation
/// recovery entry point after a reconnect.
#[tauri::command]
pub async fn sfwp_request_status(
    state: State<'_, Arc<SocketHandle>>,
    request_id: String,
) -> Result<Value, String> {
    let request =
        serde_json::to_value(SfwpQuery::RequestGetStatus { request_id }).map_err(|e| e.to_string())?;
    state.call(request).await.map_err(|e| e.to_string())
}
