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

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, State};

use crate::drafts::{self, Draft};
use crate::socket::SocketHandle;
use crate::supervisor::CellSupervisor;

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
    /// `asset.list` (Task 9). Takes no parameters: the catalog is small and the
    /// renderer filters by kind locally, so a server-side filter would be a
    /// second place for "which assets exist" to be decided.
    AssetList,
    /// `delegation.preview` (Task 10). Mirrors `Request::DelegationPreview`,
    /// whose params are `#[serde(flatten)]`ed — so these fields sit beside
    /// `verb` on the wire rather than nested under a `params` object.
    ///
    /// The field list is deliberately identical to the delegation *command*'s
    /// inputs. A preview knob the command cannot accept would let the renderer
    /// show a job contract nobody can run.
    DelegationPreview {
        endpoint: String,
        instruction: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        max_turns: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        token_budget: Option<u64>,
    },
    /// `delegation.list` (Task 11). Takes no parameters: the roster is the
    /// cell's delegations, and a server-side filter would be a second place to
    /// decide which delegations exist. `SfwpCommand::CancelDelegation` is the
    /// command half and has existed since Task 3 — this is what finally makes
    /// its targets enumerable.
    DelegationList,
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

/// A structured failure crossing the closed host bridge.
///
/// The server's refusal class must not be flattened into a string before the
/// renderer can decide whether a retry is lawful or whether a side effect is
/// known not to have happened.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BridgeCommandError {
    pub error_class: String,
    pub error: String,
    pub no_side_effect: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_lawful_action: Option<String>,
}

impl BridgeCommandError {
    fn refusal(
        error_class: impl Into<String>,
        error: impl Into<String>,
        next_lawful_action: impl Into<Option<String>>,
    ) -> Self {
        Self {
            error_class: error_class.into(),
            error: error.into(),
            no_side_effect: true,
            next_lawful_action: next_lawful_action.into(),
        }
    }

    fn uncertain(error: impl std::fmt::Display) -> Self {
        Self {
            error_class: "bridge_transport_error".into(),
            error: error.to_string(),
            no_side_effect: false,
            next_lawful_action: Some(
                "Reconnect and inspect the request outcome before retrying".into(),
            ),
        }
    }
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

/// Which cell this host dialed.
///
/// The host resolves the socket path from the cell contract
/// (`SEA_FORGE_SOCKET` > `SEA_FORGE_ROOT` > `$HOME/.sea-forge`), so it is the
/// only component that knows which cell the window is attached to. Exposed so
/// the renderer can name the active cell from the path it actually connected
/// to rather than from a constant — `router.tsx` used to default to the string
/// `"cell_local_01"`, which named nothing.
/// `supervision` reports how this window came to have a kernel: `adopted` (one
/// was already listening), `supervised` (this process started it, U-06), or
/// `unavailable` with the reason. The renderer needs the distinction because
/// "the cell is starting" and "there is no cell" look identical from a failed
/// call, and only one of them is the operator's problem to fix.
#[tauri::command]
pub fn sfwp_cell(
    state: State<'_, Arc<SocketHandle>>,
    supervisor: State<'_, Arc<CellSupervisor>>,
) -> Value {
    serde_json::json!({
        "socket_path": state.socket_path().to_string_lossy(),
        "root": supervisor.cell().root.to_string_lossy(),
        "supervision": supervisor.supervision(),
    })
}

/// Confirm initialization of the fresh root selected at application start.
/// This is deliberately a host action, not an SFWP command: no server exists
/// yet, and the host is the component that owns its lifecycle. The supervisor
/// rechecks the root while holding its lifecycle lock before any directory is
/// created, so a concurrent history write cannot be overwritten.
#[tauri::command]
pub async fn sfwp_initialize_cell(
    supervisor: State<'_, Arc<CellSupervisor>>,
) -> Result<Value, BridgeCommandError> {
    let supervisor = Arc::clone(&*supervisor);
    let supervision = tauri::async_runtime::spawn_blocking(move || supervisor.initialize())
        .await
        .map_err(BridgeCommandError::uncertain)?
        .map_err(|error| {
            BridgeCommandError::refusal(
                "cell_initialization_refused",
                error,
                Some(
                    "Inspect the cell root and open a different empty root or its existing history"
                        .into(),
                ),
            )
        })?;
    serde_json::to_value(supervision).map_err(BridgeCommandError::uncertain)
}

/// Ask the cell which actors this connection may claim (`identity.get`).
///
/// Inspect verb, so it needs no actor of its own — which is what makes it
/// usable as the way to discover one.
#[tauri::command]
pub async fn sfwp_identity(state: State<'_, Arc<SocketHandle>>) -> Result<Value, String> {
    state
        .call(serde_json::json!({"verb": "identity_get"}))
        .await
        .map_err(|e| e.to_string())
}

/// Issue a protected SFWP command (correlated by `request_id`).
///
/// The host attaches the actor; the renderer never does. `SfwpCommand` has no
/// actor field precisely so that a renderer bug or a compromised web context
/// cannot attribute work to a principal it is not — the closed enum makes that
/// unrepresentable rather than merely discouraged.
///
/// `act_as` selects among several bound actors (an operator who also approves,
/// say). It is checked against `identity.get` before use, so it can only ever
/// narrow the set the cell already granted this uid — never widen it.
#[tauri::command]
pub async fn sfwp_command(
    state: State<'_, Arc<SocketHandle>>,
    command: SfwpCommand,
    act_as: Option<String>,
) -> Result<Value, BridgeCommandError> {
    let mut request = serde_json::to_value(&command).map_err(BridgeCommandError::uncertain)?;

    // ponytail: resolved per command rather than cached. These are
    // human-initiated actions over a local Unix socket, so the extra round trip
    // is unmeasurable, and it means a `server.yaml` reload takes effect
    // immediately instead of after a restart. Cache it if a batch path ever
    // issues commands in a loop.
    let identity = state
        .call(serde_json::json!({"verb": "identity_get"}))
        .await
        .map_err(BridgeCommandError::uncertain)?;
    let actor = choose_actor(&identity, act_as.as_deref())?;

    if let Some(object) = request.as_object_mut() {
        object.insert("actor".into(), actor.clone());
        // `entity` is what reaches the authority engine and lands in the ledger
        // as the acting principal. Overwritten rather than trusted so the
        // verified identity and the recorded one cannot disagree — the server
        // refuses that mismatch too, but the renderer should not be able to
        // author a request that earns the refusal.
        if object.contains_key("entity") {
            object.insert("entity".into(), actor["actor_id"].clone());
        }
    }

    state
        .call(request)
        .await
        .map_err(BridgeCommandError::uncertain)
}

/// Pick the actor block to send, from what the cell says this connection holds.
///
/// Refuses rather than guesses in every ambiguous case. Acting as an
/// unspecified one of several identities would be the same fabrication SF-005
/// removed, just chosen by array order instead of by a constant.
fn choose_actor(identity: &Value, act_as: Option<&str>) -> Result<Value, BridgeCommandError> {
    let available = identity
        .get("available")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();

    if available.is_empty() {
        let refusal = identity.get("refusal");
        let message = refusal
            .and_then(|r| r.get("message"))
            .and_then(Value::as_str)
            .unwrap_or("this cell reported no identity bindings");
        return Err(BridgeCommandError::refusal(
            refusal
                .and_then(|r| r.get("error_class"))
                .and_then(Value::as_str)
                .unwrap_or("identity_unresolved"),
            message,
            refusal
                .and_then(|r| r.get("next_lawful_action"))
                .and_then(Value::as_str)
                .map(str::to_owned),
        ));
    }

    let chosen = match act_as {
        Some(wanted) => available
            .iter()
            .find(|entry| entry.get("actor_id").and_then(Value::as_str) == Some(wanted))
            .ok_or_else(|| {
                BridgeCommandError::refusal(
                    "identity_not_bound",
                    format!("this connection may not act as `{wanted}`"),
                    Some("Select a server-advertised actor".into()),
                )
            })?,
        None if available.len() == 1 => &available[0],
        None => {
            return Err(BridgeCommandError::refusal(
                "identity_ambiguous",
                format!(
                    "this connection may act as {} different actors; choose one explicitly",
                    available.len()
                ),
                Some("Select one of the server-advertised actors".into()),
            ))
        }
    };

    let actor_id = chosen
        .get("actor_id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            BridgeCommandError::refusal(
                "identity_unresolved",
                "identity.get returned an actor with no id",
                Some("Refresh identity bindings and choose an actor".into()),
            )
        })?;
    let role = chosen
        .get("roles")
        .and_then(Value::as_array)
        .and_then(|roles| roles.first())
        .and_then(Value::as_str)
        .ok_or_else(|| {
            BridgeCommandError::refusal(
                "identity_role_not_held",
                format!("actor `{actor_id}` holds no role in this cell"),
                Some("Select an actor with an eligible role".into()),
            )
        })?;

    Ok(serde_json::json!({"actor_id": actor_id, "role": role}))
}

/// Thin convenience wrapper over `SfwpQuery::RequestGetStatus` — the correlation
/// recovery entry point after a reconnect.
#[tauri::command]
pub async fn sfwp_request_status(
    state: State<'_, Arc<SocketHandle>>,
    request_id: String,
) -> Result<Value, String> {
    let request = serde_json::to_value(SfwpQuery::RequestGetStatus { request_id })
        .map_err(|e| e.to_string())?;
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

    /// A unit variant must serialize to the bare tag object — a stray field or
    /// a `null` payload would make the server's `Request::AssetList` fail to
    /// deserialize, and the catalog would read as a transport error rather than
    /// as an empty cell.
    #[test]
    fn asset_list_serializes_to_the_bare_verb() {
        assert_eq!(
            serde_json::to_value(SfwpQuery::AssetList).unwrap(),
            serde_json::json!({"verb": "asset_list"})
        );
    }

    /// The server flattens `DelegationPreviewParams` into the request, so these
    /// fields must land beside `verb` and not under a `params` key. An omitted
    /// optional must stay absent rather than serialize as `null`: the server's
    /// `Option<u64>` would reject an explicit null for `token_budget`, and "no
    /// token cap" would come back as a transport error.
    #[test]
    fn delegation_preview_flattens_its_params_beside_the_verb() {
        let minimal = serde_json::to_value(SfwpQuery::DelegationPreview {
            endpoint: "local".into(),
            instruction: "summarize".into(),
            model: None,
            max_turns: 4,
            token_budget: None,
        })
        .unwrap();
        assert_eq!(
            minimal,
            serde_json::json!({
                "verb": "delegation_preview",
                "endpoint": "local",
                "instruction": "summarize",
                "max_turns": 4,
            })
        );

        let full = serde_json::to_value(SfwpQuery::DelegationPreview {
            endpoint: "local".into(),
            instruction: "summarize".into(),
            model: Some("m1".into()),
            max_turns: 4,
            token_budget: Some(5000),
        })
        .unwrap();
        assert_eq!(full["model"], "m1");
        assert_eq!(full["token_budget"], 5000);
    }

    fn identity(actors: &[(&str, &[&str])]) -> Value {
        serde_json::json!({
            "configured": true,
            "uid": 1000,
            "available": actors.iter().map(|(id, roles)| {
                serde_json::json!({"actor_id": id, "roles": roles})
            }).collect::<Vec<_>>(),
        })
    }

    #[test]
    fn a_sole_bound_actor_is_used_without_being_named() {
        let actor = choose_actor(&identity(&[("operator_local", &["operator"])]), None).unwrap();
        assert_eq!(
            actor,
            serde_json::json!({"actor_id": "operator_local", "role": "operator"})
        );
    }

    /// The renderer picking among several is fine; the renderer *inventing* one
    /// is not. `act_as` can only ever select from what the cell already granted.
    #[test]
    fn act_as_selects_within_the_granted_set_and_cannot_leave_it() {
        let bound = identity(&[("operator_a", &["operator"]), ("operator_b", &["operator"])]);

        let chosen = choose_actor(&bound, Some("operator_b")).unwrap();
        assert_eq!(chosen["actor_id"], "operator_b");

        let refused = choose_actor(&bound, Some("operator_c")).unwrap_err();
        assert!(
            refused.error.contains("may not act as `operator_c`"),
            "unexpected refusal: {refused:?}"
        );
        assert_eq!(refused.error_class, "identity_not_bound");
        assert!(refused.no_side_effect);
    }

    /// Ambiguity must refuse, not default. Picking `available[0]` would
    /// reintroduce a fabricated identity chosen by array order — and would let
    /// a submitter approve their own work by happening to be listed first.
    #[test]
    fn several_bound_actors_with_no_choice_refuses_rather_than_picking_one() {
        let error = choose_actor(
            &identity(&[("operator_a", &["operator"]), ("operator_b", &["operator"])]),
            None,
        )
        .unwrap_err();
        assert!(error.error.contains("choose one explicitly"), "{error:?}");
        assert_eq!(error.error_class, "identity_ambiguous");
    }

    /// An unconfigured cell must surface the kernel's own reason, not a
    /// host-invented one, so the operator reads the same explanation the
    /// server would have given.
    #[test]
    fn no_available_actor_reports_the_cells_own_refusal() {
        let unconfigured = serde_json::json!({
            "configured": false,
            "available": [],
            "refusal": {
                "error_class": "identity_unconfigured",
                "message": "this cell configures no identity bindings"
            }
        });
        let error = choose_actor(&unconfigured, None).unwrap_err();
        assert!(
            error.error.contains("configures no identity bindings"),
            "{error:?}"
        );
        assert_eq!(error.error_class, "identity_unconfigured");
    }

    /// A suspended actor — bound, but holding no role — authorizes nothing.
    /// Sending a role we invented would be worse than refusing.
    #[test]
    fn an_actor_holding_no_role_is_refused() {
        let error = choose_actor(&identity(&[("suspended", &[])]), None).unwrap_err();
        assert!(error.error.contains("holds no role"), "{error:?}");
        assert_eq!(error.error_class, "identity_role_not_held");
    }

    /// The roster query and the cancel command are two halves of one control
    /// loop, so they are asserted together: a drift in either verb spelling
    /// leaves an operator able to see delegations but not stop them, or the
    /// reverse.
    #[test]
    fn the_delegation_roster_and_its_cancel_command_use_the_verbs_the_server_accepts() {
        assert_eq!(
            serde_json::to_value(SfwpQuery::DelegationList).unwrap(),
            serde_json::json!({"verb": "delegation_list"})
        );

        let cancel = serde_json::to_value(SfwpCommand::CancelDelegation {
            run_id: "run-1".into(),
            policy: None,
            entity: None,
            process: None,
            request_id: Some("req-1".into()),
        })
        .unwrap();
        assert_eq!(
            cancel,
            serde_json::json!({
                "verb": "cancel_delegation",
                "run_id": "run-1",
                "request_id": "req-1",
            })
        );
    }
}
