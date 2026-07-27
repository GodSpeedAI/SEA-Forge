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
use tauri::{AppHandle, Manager, State};

use crate::drafts::{self, Draft};
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
    /// `readiness.get` — mirrors the server's `Request::ReadinessGet`
    /// byte-for-byte (`verb: "readiness_get"`, optional `intended_operation`).
    /// Kept host-local (no server-crate dependency) exactly like `Precondition`
    /// above; the generated TS `ReadinessView`/`IntendedOperation` types are the
    /// frontend's source of truth for these shapes.
    ReadinessGet {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        intended_operation: Option<IntendedOperation>,
    },
    /// Case-authoring inspect methods (Task 6). Mirror the server's
    /// `Request::CaseEntryOptions`/`Request::CasePreflight` byte-for-byte.
    CaseEntryOptions,
    CasePreflight {
        template_ref: String,
        #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
        params: std::collections::BTreeMap<String, String>,
    },
    /// Case-navigation inspect methods (Task 7). Mirror the server's
    /// `Request::CaseList` / `CaseGetOverview` / `CaseGetHorizon`.
    CaseList,
    CaseGetOverview {
        case_id: String,
    },
    CaseGetHorizon {
        case_id: String,
    },
    /// `approval.list` (Task 7). `case_id` scopes the inbox to one case.
    ApprovalList {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        case_id: Option<String>,
    },
    /// Run-record inspect methods (Task 8). Mirror the server's
    /// `Request::RunList` / `RunGet`. `run.get` is what resolves the run ids the
    /// case, horizon, and event views already render.
    RunList {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        case_id: Option<String>,
    },
    RunGet {
        run_id: String,
    },
}

/// The operator's intended next operation, mirroring the server's
/// `sfwp::readiness::IntendedOperation`. Host-local shape (no server-crate
/// dependency); the generated TS type is the frontend's source of truth.
#[derive(Deserialize, serde::Serialize)]
pub struct IntendedOperation {
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_ref: Option<String>,
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
    /// `approval.decide` (Task 7) — the verdict-carrying form of
    /// `Approve`/`Reject`. Both spellings reach the same server-side `decide`
    /// path; this one lets an inbox that renders a row with two buttons send
    /// one command shape rather than branching on which verb to construct.
    ApprovalDecide {
        case_id: String,
        approval_id: String,
        /// `"approve"` or `"reject"`. The server refuses anything else rather
        /// than defaulting — a governed decision is never guessed.
        decision: String,
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
    /// `case.commit` (Task 6) — mirrors the server's `Request::CaseCommit`
    /// byte-for-byte. The single protected verb that actually creates a case;
    /// `preconditions` pins the template digest `case.preflight` returned.
    CaseCommit {
        template_ref: String,
        #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
        params: std::collections::BTreeMap<String, String>,
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        preconditions: Option<Precondition>,
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

/// Local case-authoring draft persistence (Task 6). Reversible,
/// non-authoritative host-owned state (`drafts.rs`) — never the socket, never
/// `.sea-forge/`. A draft only becomes real case state via `case.preflight`
/// -> `case.commit`.
fn app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().app_data_dir().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn draft_save(app: AppHandle, draft_id: String, state: Value) -> Result<Draft, String> {
    let dir = app_data_dir(&app)?;
    tauri::async_runtime::spawn_blocking(move || drafts::save(&dir, &draft_id, state))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn draft_load(app: AppHandle, draft_id: String) -> Result<Draft, String> {
    let dir = app_data_dir(&app)?;
    tauri::async_runtime::spawn_blocking(move || drafts::load(&dir, &draft_id))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn draft_list(app: AppHandle) -> Result<Vec<Draft>, String> {
    let dir = app_data_dir(&app)?;
    tauri::async_runtime::spawn_blocking(move || drafts::list(&dir))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn draft_delete(app: AppHandle, draft_id: String) -> Result<(), String> {
    let dir = app_data_dir(&app)?;
    tauri::async_runtime::spawn_blocking(move || drafts::delete(&dir, &draft_id))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole closed-enum boundary rests on these variants serializing to the
    /// exact wire line the server's `Request` deserializes. Nothing but a test
    /// enforces that: a renamed field or a `null` where the server expects an
    /// absent key fails at runtime, on the socket, with a serde error the
    /// renderer surfaces as an opaque string.
    #[test]
    fn run_query_variants_serialize_to_the_verbs_the_server_accepts() {
        let listing = serde_json::to_value(SfwpQuery::RunList { case_id: None }).unwrap();
        assert_eq!(listing, serde_json::json!({"verb": "run_list"}));

        // An omitted optional stays *absent*, never `null` — the server's
        // `#[serde(default)]` handles a missing key, not a null one.
        assert!(listing.get("case_id").is_none());

        let scoped = serde_json::to_value(SfwpQuery::RunList {
            case_id: Some("case-1".into()),
        })
        .unwrap();
        assert_eq!(
            scoped,
            serde_json::json!({"verb": "run_list", "case_id": "case-1"})
        );

        let record = serde_json::to_value(SfwpQuery::RunGet {
            run_id: "run-1".into(),
        })
        .unwrap();
        assert_eq!(
            record,
            serde_json::json!({"verb": "run_get", "run_id": "run-1"})
        );
    }
}
