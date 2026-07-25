#![forbid(unsafe_code)]

//! SEA Forge concurrent case server (spec-full §10.3, §11.1, M3).
//!
//! The server listens on a Unix domain socket, accepts NDJSON requests,
//! and dispatches case runs as subprocesses (`sea-forge run --plan ...`)
//! via `spawn_blocking`. Kernel logic stays synchronous; Tokio lives only
//! here.

use chrono::Utc;
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::{
    errors::ForgeError,
    ids::{case_id, random_id, run_id, valid_run_id},
    types::{
        Actor, ActorRole, AuthorityAction, CasePlan, DelegationTermination, SettlementEvent,
        SettlementStatus, TranscriptEvidence, TranscriptSummary, Verdict,
    },
};
use sea_forge_ledger::LedgerStream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::process::Command;
use tokio::sync::{broadcast, mpsc, Mutex, Semaphore};

use sfwp::correlation::RequestCorrelationStore;
use sfwp::events::EventFrame;

pub mod agent_probe;
pub mod case_dispatch;
pub mod config;
pub mod delegation;
pub mod sfwp;
pub mod swe_seed_reconciliation;
mod transcript_seal;

pub use config::ServerConfig;

/// A running case submitted to the server.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CaseEntry {
    pub case_id: String,
    pub state: String,
    pub exit_code: Option<i32>,
    pub run_dir: Option<String>,
}

/// Shared server state.
pub struct ServerState {
    pub config: ServerConfig,
    pub cases: Mutex<HashMap<String, CaseEntry>>,
    delegations: Mutex<HashMap<String, DelegationHandle>>,
    pub(crate) permission_broker: delegation::AcpApprovalBroker,
    pub semaphore: Arc<Semaphore>,
    /// SFWP request-correlation store (Task 3): answers `request.get_status`
    /// across reconnects.
    pub(crate) correlation: RequestCorrelationStore,
    /// The single durable SFWP events ledger; its `entry_ulid` is the event
    /// cursor. Guarded so concurrent connection tasks serialize their appends
    /// on top of the ledger's own file lock.
    events_ledger: Mutex<LedgerStream>,
    /// Live SFWP event fan-out. Subscribers ride on top of the durable ledger;
    /// the broadcast is never source truth.
    pub(crate) event_bus: broadcast::Sender<EventFrame>,
}

#[derive(Clone)]
struct DelegationHandle {
    case_id: String,
    cancel: Arc<AtomicBool>,
    requested: Arc<AtomicBool>,
}

#[derive(Serialize)]
struct RecoveredDelegationEvidence<'a> {
    #[serde(flatten)]
    evidence: &'a TranscriptEvidence,
    case_id: &'a str,
    item_id: &'a str,
}

#[derive(Serialize)]
struct RecoveredDelegationSettlement<'a> {
    #[serde(flatten)]
    settlement: &'a SettlementEvent,
    case_id: &'a str,
    item_id: &'a str,
}

#[derive(Serialize)]
struct RecoveredAcpSession<'a> {
    version: &'static str,
    case_id: &'a str,
    run_id: &'a str,
    plan_item_id: &'a str,
    endpoint_ref: &'a str,
    continuation_key: &'a str,
    protocol_version: u32,
}

impl ServerState {
    pub fn new(config: ServerConfig) -> Result<Self, ForgeError> {
        recover_cancelled_delegations(&config)?;
        // A SWE_SEED declaration may have been appended directly (out of
        // process) while the server was absent; join it against any
        // harvested evidence before this server accepts new work (M16 T18).
        swe_seed_reconciliation::reconcile_all_cases(&config.root)?;
        let max = config.max_concurrent_runs.max(1);
        let correlation = RequestCorrelationStore::open(&config.root)?;
        let events_ledger = sfwp::events::open_events_ledger(&config.root)?;
        let (event_bus, _) = broadcast::channel::<EventFrame>(256);
        Ok(Self {
            config,
            cases: Mutex::new(HashMap::new()),
            delegations: Mutex::new(HashMap::new()),
            permission_broker: delegation::AcpApprovalBroker::default(),
            semaphore: Arc::new(Semaphore::new(max)),
            correlation,
            events_ledger: Mutex::new(events_ledger),
            event_bus,
        })
    }

    /// Append an event to the durable events ledger and fan it out on the live
    /// broadcast channel. The `cursor` on the returned frame is the emitting
    /// ledger entry's `entry_ulid`. `SendError` (no active subscribers) is not
    /// an error and is ignored.
    pub(crate) async fn publish_event(
        &self,
        kind: &str,
        case_id: Option<&str>,
        run_id: Option<&str>,
        detail: serde_json::Value,
    ) -> Result<EventFrame, ForgeError> {
        let frame = {
            let ledger = self.events_ledger.lock().await;
            sfwp::events::append_event(&ledger, kind, case_id, run_id, detail)?
        };
        let _ = self.event_bus.send(frame.clone());
        Ok(frame)
    }

    /// Snapshot the durable events for gap recovery (`events.get_range`).
    pub(crate) async fn events_get_range(
        &self,
        from_cursor: Option<&str>,
        to_cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<EventFrame>, ForgeError> {
        let ledger = self.events_ledger.lock().await;
        sfwp::events::get_range(&ledger, from_cursor, to_cursor, limit)
    }

    /// Replay durable events after a cursor for a subscribe catch-up burst.
    pub(crate) async fn events_replay_after(
        &self,
        from_cursor: Option<&str>,
    ) -> Result<Vec<EventFrame>, ForgeError> {
        let ledger = self.events_ledger.lock().await;
        sfwp::events::replay_after(&ledger, from_cursor)
    }

    /// Reload config defensively between dispatches (§8.4).
    pub fn reload_config(&self) -> Result<ServerConfig, String> {
        match ServerConfig::load(&self.config.root.join("server.yaml")) {
            Ok(new_config) => Ok(new_config),
            Err(e) => Err(e),
        }
    }

    pub(crate) async fn begin_planned_delegation(
        &self,
        case_id: String,
        run_id: String,
    ) -> Result<Arc<AtomicBool>, ForgeError> {
        let cancel = Arc::new(AtomicBool::new(false));
        let handle = DelegationHandle {
            case_id,
            cancel: Arc::clone(&cancel),
            requested: Arc::new(AtomicBool::new(false)),
        };
        if self
            .delegations
            .lock()
            .await
            .insert(run_id, handle)
            .is_some()
        {
            return Err(ForgeError::Input("delegation run_id already active".into()));
        }
        Ok(cancel)
    }

    pub(crate) async fn end_delegation(&self, run_id: &str) {
        self.delegations.lock().await.remove(run_id);
    }
}

/// Recover a cancellation whose control record committed before server loss.
/// HTTP delegations are never resumed: recovery writes the sole terminal
/// rejected/cancelled outcome from ledger truth.
fn recover_cancelled_delegations(config: &ServerConfig) -> Result<(), ForgeError> {
    let ledgers = config.root.join("ledgers");
    if !ledgers.exists() {
        return Ok(());
    }
    for dir in std::fs::read_dir(&ledgers).map_err(|e| ForgeError::io("read ledgers", e))? {
        let dir = dir.map_err(|e| ForgeError::io("read ledger entry", e))?;
        let name = dir.file_name().to_string_lossy().into_owned();
        if !dir
            .file_type()
            .map_err(|e| ForgeError::io("read ledger entry type", e))?
            .is_dir()
            || !name.starts_with("case-")
        {
            continue;
        }
        let ledger = LedgerStream::open(&config.root, &name, "sea-forge-server")?;
        ledger.verify()?;
        let entries = ledger.read_entries()?;
        let mut terminal_runs: std::collections::HashSet<String> = entries
            .iter()
            .filter(|entry| entry.record_kind == "settlement")
            .filter_map(|entry| entry.payload["run_id"].as_str().map(str::to_owned))
            .collect();
        for control in entries
            .iter()
            .filter(|entry| entry.record_kind == "control_request")
        {
            let run = control
                .payload
                .get("run_id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| ForgeError::Input("invalid cancellation control run_id".into()))?;
            if terminal_runs.contains(run) {
                continue;
            }
            let item_id = control
                .payload
                .get("plan_item_id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| {
                    ForgeError::Input("invalid cancellation control plan_item_id".into())
                })?;
            let case_id = control
                .payload
                .get("case_id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| ForgeError::Input("invalid cancellation control case_id".into()))?;
            let plan_path = config.root.join("runs").join(run).join("plan.json");
            let plan: CasePlan = serde_json::from_slice(
                &std::fs::read(&plan_path)
                    .map_err(|e| ForgeError::io("read cancelled delegation plan", e))?,
            )?;
            if plan.case_id != case_id || plan.run_id != run {
                return Err(ForgeError::Input(
                    "cancelled delegation control does not match plan identity".into(),
                ));
            }
            let endpoint_ref = plan
                .items
                .iter()
                .find(|item| item.plan_item_id == item_id)
                .and_then(|item| item.operations.first())
                .and_then(|operation| match operation {
                    sea_forge_core::types::Operation::AgentTask { endpoint_ref, .. } => {
                        Some(endpoint_ref.as_str())
                    }
                    _ => None,
                })
                .ok_or_else(|| {
                    ForgeError::Input("cancelled delegation plan missing agent_task".into())
                })?;
            let evidence = TranscriptEvidence {
                run_id: run.into(),
                endpoint_ref: endpoint_ref.into(),
                turns_used: 0,
                termination: DelegationTermination::Cancelled,
                transcript_sha256: sea_forge_agent::transcript_sha256(&[]),
                summary: TranscriptSummary {
                    turn_count: 0,
                    tool_calls: 0,
                    final_excerpt: String::new(),
                },
                artifact_ref: None,
                harvested_refs: vec![],
            };
            let evidence_ref = agent_probe::commit_view(
                &ledger,
                "agent_task_evidence",
                vec![case_id.into(), run.into(), item_id.into()],
                &RecoveredDelegationEvidence {
                    evidence: &evidence,
                    case_id,
                    item_id,
                },
                &config
                    .root
                    .join("runs")
                    .join(run)
                    .join("transcript-evidence.json"),
                vec![control.entry_ulid.clone()],
            )?;
            let settlement = SettlementEvent {
                version: "0.2".into(),
                settlement_id: random_id("set")?,
                run_id: run.into(),
                status: SettlementStatus::Rejected,
                basis: vec!["cancelled".into()],
                review_required: false,
                settled_at: Utc::now().to_rfc3339(),
                criteria_ref: None,
            };
            agent_probe::commit_view(
                &ledger,
                "settlement",
                vec![case_id.into(), run.into(), item_id.into()],
                &RecoveredDelegationSettlement {
                    settlement: &settlement,
                    case_id,
                    item_id,
                },
                &config.root.join("runs").join(run).join("settlement.json"),
                vec![evidence_ref.entry_ulid().into()],
            )?;
            terminal_runs.insert(run.into());
        }

        for permission in entries
            .iter()
            .filter(|entry| entry.record_kind == "permission_request")
        {
            let run = permission.payload["run_id"]
                .as_str()
                .ok_or_else(|| ForgeError::Input("invalid ACP permission run_id".into()))?;
            if terminal_runs.contains(run) {
                continue;
            }
            let case_id = permission.payload["case_id"]
                .as_str()
                .ok_or_else(|| ForgeError::Input("invalid ACP permission case_id".into()))?;
            let item_id = permission.payload["plan_item_id"]
                .as_str()
                .ok_or_else(|| ForgeError::Input("invalid ACP permission item_id".into()))?;
            let continuation_key = permission.payload["session_id"]
                .as_str()
                .ok_or_else(|| ForgeError::Input("invalid ACP permission session_id".into()))?;
            let plan_path = config.root.join("runs").join(run).join("plan.json");
            let plan: CasePlan = serde_json::from_slice(
                &std::fs::read(&plan_path)
                    .map_err(|error| ForgeError::io("read orphaned ACP plan", error))?,
            )?;
            if plan.case_id != case_id || plan.run_id != run {
                return Err(ForgeError::Input(
                    "orphaned ACP permission does not match plan identity".into(),
                ));
            }
            let endpoint_ref = plan
                .items
                .iter()
                .find(|item| item.plan_item_id == item_id)
                .and_then(|item| item.operations.first())
                .and_then(|operation| match operation {
                    sea_forge_core::types::Operation::AgentTask { endpoint_ref, .. } => {
                        Some(endpoint_ref.as_str())
                    }
                    _ => None,
                })
                .ok_or_else(|| ForgeError::Input("orphaned ACP plan missing agent task".into()))?;
            let session_ref = agent_probe::commit_view(
                &ledger,
                "acp_session",
                vec![case_id.into(), run.into(), item_id.into()],
                &RecoveredAcpSession {
                    version: "0.2",
                    case_id,
                    run_id: run,
                    plan_item_id: item_id,
                    endpoint_ref,
                    continuation_key,
                    protocol_version: sea_forge_agent::ACP_PROTOCOL_VERSION,
                },
                &config.root.join("runs").join(run).join("acp-session.json"),
                vec![permission.entry_ulid.clone()],
            )?;
            let evidence = TranscriptEvidence {
                run_id: run.into(),
                endpoint_ref: endpoint_ref.into(),
                turns_used: 0,
                termination: DelegationTermination::AcpDisconnect,
                transcript_sha256: sea_forge_agent::transcript_sha256(&[]),
                summary: TranscriptSummary {
                    turn_count: 0,
                    tool_calls: 0,
                    final_excerpt: String::new(),
                },
                artifact_ref: None,
                harvested_refs: vec![],
            };
            let evidence_ref = agent_probe::commit_view(
                &ledger,
                "agent_task_evidence",
                vec![case_id.into(), run.into(), item_id.into()],
                &RecoveredDelegationEvidence {
                    evidence: &evidence,
                    case_id,
                    item_id,
                },
                &config
                    .root
                    .join("runs")
                    .join(run)
                    .join("transcript-evidence.json"),
                vec![
                    permission.entry_ulid.clone(),
                    session_ref.entry_ulid().into(),
                ],
            )?;
            let settlement = SettlementEvent {
                version: "0.2".into(),
                settlement_id: random_id("set")?,
                run_id: run.into(),
                status: SettlementStatus::Rejected,
                basis: vec!["acp_disconnect".into()],
                review_required: false,
                settled_at: Utc::now().to_rfc3339(),
                criteria_ref: None,
            };
            agent_probe::commit_view(
                &ledger,
                "settlement",
                vec![case_id.into(), run.into(), item_id.into()],
                &RecoveredDelegationSettlement {
                    settlement: &settlement,
                    case_id,
                    item_id,
                },
                &config.root.join("runs").join(run).join("settlement.json"),
                vec![evidence_ref.entry_ulid().into()],
            )?;
            terminal_runs.insert(run.into());
        }
    }
    Ok(())
}

/// NDJSON request envelope.
///
/// SFWP (Task 3) is layered additively per ADR-003: new methods are new
/// `verb`-tagged variants (`system_hello`, `events_subscribe`, ...), never a
/// nested wrapper frame and never a reshape of the existing verbs. Old clients
/// omitting the new verbs keep working; old servers reject unknown verbs
/// cleanly (serde unknown-variant). The optional `request_id`/`preconditions`
/// fields added to protected verbs are `#[serde(default)]`, so old clients that
/// omit them still parse. See `sfwp::mod` for the full seam rationale.
#[derive(Deserialize)]
#[serde(tag = "verb")]
#[serde(rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum Request {
    Submit {
        #[serde(flatten)]
        payload: SubmitPayload,
        #[serde(default)]
        request_id: Option<String>,
    },
    Status {
        case_id: String,
    },
    Approve {
        case_id: String,
        approval_id: String,
        #[serde(default)]
        note: Option<String>,
        #[serde(default)]
        request_id: Option<String>,
        #[serde(default)]
        preconditions: Option<sfwp::precondition::Precondition>,
    },
    Reject {
        case_id: String,
        approval_id: String,
        #[serde(default)]
        note: Option<String>,
        #[serde(default)]
        request_id: Option<String>,
        #[serde(default)]
        preconditions: Option<sfwp::precondition::Precondition>,
    },
    AgentList,
    AgentProbe {
        endpoint: String,
        prompt: String,
        #[serde(default)]
        model: Option<String>,
        #[serde(default = "default_policy")]
        policy: String,
        #[serde(default = "default_entity")]
        entity: String,
        #[serde(default = "default_process")]
        process: String,
    },
    Delegate {
        endpoint: String,
        instruction: String,
        #[serde(default)]
        run_id: Option<String>,
        #[serde(default)]
        model: Option<String>,
        max_turns: u32,
        #[serde(default)]
        token_budget: Option<u64>,
        #[serde(default)]
        criteria: sea_forge_core::types::SettlementCriteria,
        #[serde(default = "default_policy")]
        policy: String,
        #[serde(default = "default_entity")]
        entity: String,
        #[serde(default = "default_process")]
        process: String,
        #[serde(default)]
        request_id: Option<String>,
    },
    CancelDelegation {
        run_id: String,
        #[serde(default = "default_policy")]
        policy: String,
        #[serde(default = "default_entity")]
        entity: String,
        #[serde(default = "default_process")]
        process: String,
        #[serde(default)]
        request_id: Option<String>,
    },
    /// Thoth self-disclosure ask (M11 T14B). Dispatches to the same
    /// `sea_forge_thoth::service::ask` the CLI adapter calls, via
    /// `spawn_blocking` — one shared governance path, no server-side answer
    /// engine. Added additively per ADR-003; an old server rejects this tag
    /// cleanly (unknown enum variant), never silently misrouting it.
    Ask {
        kind: String,
        subject: String,
        #[serde(default = "default_purpose")]
        purpose: String,
        #[serde(default)]
        case: Option<String>,
        #[serde(default = "default_entity")]
        actor: String,
    },

    // --- SFWP additive methods (Task 3, ADR-003) ------------------------
    /// Negotiate protocol version and discover implemented methods.
    SystemHello {
        protocol_version: String,
        #[serde(default)]
        client: Option<String>,
    },
    /// Return the method catalog with each method's interaction class.
    SystemDescribe,
    /// Return generated schema references (optionally scoped to one method).
    SystemGetSchema {
        #[serde(default)]
        method: Option<String>,
    },
    /// Recover the status/outcome of a prior correlated request.
    RequestGetStatus {
        request_id: String,
    },
    /// Subscribe to live events; optionally replay from a durable cursor first.
    EventsSubscribe {
        #[serde(default)]
        from_cursor: Option<String>,
    },
    /// Drop the live event subscription for this connection.
    EventsUnsubscribe,
    /// Deterministic bounded read of durable events (gap recovery).
    EventsGetRange {
        #[serde(default)]
        from_cursor: Option<String>,
        #[serde(default)]
        to_cursor: Option<String>,
        #[serde(default)]
        limit: Option<u32>,
    },
    /// Read-only readiness projection (Task 5). Inspect method: shapes proven
    /// self-model/endpoint truth into a view; introduces no new truth, mutates
    /// nothing. `intended_operation` foregrounds the capabilities that operation
    /// depends on (operation-sensitivity).
    ReadinessGet {
        #[serde(default)]
        intended_operation: Option<sfwp::readiness::IntendedOperation>,
    },
}

#[derive(Deserialize)]
pub struct SubmitPayload {
    /// Server-owned dispatch (case_dispatch::submit) requires `plan` and
    /// ignores `intent`; intent-only submit is no longer supported via this
    /// endpoint. The field is retained for deserialization compatibility with
    /// older clients but has no effect on the dispatch path.
    #[serde(default)]
    intent: Option<String>,
    #[serde(default)]
    plan: Option<String>,
    #[serde(default = "default_policy")]
    policy: String,
    #[serde(default = "default_entity")]
    entity: String,
    #[serde(default = "default_process")]
    process: String,
    #[serde(default = "default_timeout")]
    timeout: u64,
}

fn default_policy() -> String {
    "sea-forge-policy.yaml".into()
}
fn default_entity() -> String {
    "operator_local".into()
}
fn default_process() -> String {
    "server".into()
}
fn default_timeout() -> u64 {
    60
}
fn default_purpose() -> String {
    "planning".into()
}

/// Start the server.
pub async fn run(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = config.socket_path.clone();
    let state = Arc::new(ServerState::new(config)?);

    // Remove stale socket.
    let _ = std::fs::remove_file(&socket_path);

    let listener = UnixListener::bind(&socket_path)?;

    // Set socket permissions to 0600 (§11.1).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))?;
    }

    tracing::info!("server listening on {}", socket_path.display());

    loop {
        let (stream, _) = listener.accept().await?;
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, state).await {
                tracing::warn!("connection error: {e}");
            }
        });
    }
}

async fn handle_connection(
    stream: UnixStream,
    state: Arc<ServerState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // All writes to the socket funnel through a single-writer task draining a
    // bounded mpsc channel, so unsolicited SFWP event lines can interleave
    // with normal request responses without racing the writer. The bound
    // imposes per-connection backpressure: request-response writes await send
    // completion, while the live event subscription uses nonblocking `try_send`
    // and drops frames when full (a client recovers any gap deterministically
    // via `events.get_range`).
    // ponytail: fixed per-connection cap; raise only if a measured subscriber
    // proves it drops too often under realistic event volume.
    const WRITER_CHANNEL_CAPACITY: usize = 256;
    let (tx, mut rx) = mpsc::channel::<String>(WRITER_CHANNEL_CAPACITY);
    let writer_task = tokio::spawn(async move {
        while let Some(payload) = rx.recv().await {
            if writer.write_all(payload.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    // The live event subscription for this connection, if any (`AbortHandle`
    // is enough — no generalized registry needed).
    let mut subscription: Option<tokio::task::JoinHandle<()>> = None;

    let result: Result<(), Box<dyn std::error::Error + Send + Sync>> = loop {
        line.clear();
        let n = match reader.read_line(&mut line).await {
            Ok(n) => n,
            Err(e) => break Err(Box::new(e) as _),
        };
        if n == 0 {
            break Ok(());
        }
        let request: Request = match serde_json::from_str(line.trim()) {
            Ok(r) => r,
            Err(e) => {
                let resp = serde_json::json!({"error": format!("bad request: {e}")});
                let _ = tx.send(format!("{resp}\n")).await;
                continue;
            }
        };

        // Connection-scoped SFWP verbs handled here (they need the per-conn
        // writer channel / subscription slot); everything else delegates to
        // the shared `handle_request`.
        match request {
            Request::EventsSubscribe { from_cursor } => {
                let response =
                    start_subscription(&state, &tx, &mut subscription, from_cursor).await;
                let _ = tx.send(format!("{response}\n")).await;
            }
            Request::EventsUnsubscribe => {
                if let Some(handle) = subscription.take() {
                    handle.abort();
                }
                let response = serde_json::json!({"ok": true, "subscribed": false});
                let _ = tx.send(format!("{response}\n")).await;
            }
            other => {
                let response = handle_request(other, &state).await;
                let _ = tx.send(format!("{response}\n")).await;
            }
        }
    };

    if let Some(handle) = subscription.take() {
        handle.abort();
    }
    drop(tx);
    let _ = writer_task.await;
    result
}

/// Spawn (or replace) the live event-subscription task for a connection.
/// Replays any durable events after `from_cursor` first, then streams live
/// frames from the broadcast channel onto the connection's writer channel.
async fn start_subscription(
    state: &Arc<ServerState>,
    tx: &mpsc::Sender<String>,
    subscription: &mut Option<tokio::task::JoinHandle<()>>,
    from_cursor: Option<String>,
) -> serde_json::Value {
    // Subscribe before replaying so no event committed during replay is lost.
    let mut rx = state.event_bus.subscribe();

    let replayed = match state.events_replay_after(from_cursor.as_deref()).await {
        Ok(frames) => frames,
        Err(error) => {
            return serde_json::json!({
                "error": error.to_string(),
                "error_class": error.class(),
            });
        }
    };
    let last_replayed_cursor = replayed.last().map(|frame| frame.cursor.clone());
    for frame in &replayed {
        let line = serde_json::json!({"type": "event", "event": frame});
        // Nonblocking send: a full queue drops the frame (the client recovers
        // it via `events.get_range`); a closed writer ends the subscription.
        match tx.try_send(format!("{line}\n")) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {}
            Err(mpsc::error::TrySendError::Closed(_)) => {
                return serde_json::json!({"error": "connection closed"});
            }
        }
    }

    // Replace any prior subscription on this connection.
    if let Some(handle) = subscription.take() {
        handle.abort();
    }
    let tx_for_task = tx.clone();
    let handle = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(frame) => {
                    // Skip frames already delivered in the replay burst.
                    if let Some(last) = &last_replayed_cursor {
                        if frame.cursor.as_str() <= last.as_str() {
                            continue;
                        }
                    }
                    let line = serde_json::json!({"type": "event", "event": frame});
                    // Drop on a full queue (gap recovered via
                    // `events.get_range`); only a closed writer ends the
                    // stream.
                    match tx_for_task.try_send(format!("{line}\n")) {
                        Ok(()) => {}
                        Err(mpsc::error::TrySendError::Full(_)) => {
                            // The client can recover the gap deterministically
                            // via events.get_range; keep the live stream going.
                            continue;
                        }
                        Err(mpsc::error::TrySendError::Closed(_)) => break,
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    // The client can recover the gap deterministically via
                    // events.get_range; keep the live stream going.
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });
    *subscription = Some(handle);

    serde_json::json!({
        "ok": true,
        "subscribed": true,
        "replayed": replayed.len(),
    })
}

pub async fn handle_request(request: Request, state: &Arc<ServerState>) -> serde_json::Value {
    match request {
        Request::Submit {
            payload,
            request_id,
        } => {
            record_pending(state, request_id.as_deref(), "case.submit");
            // Reload config defensively (§8.4).
            if let Ok(new_config) = state.reload_config() {
                tracing::info!("config reloaded successfully");
                let _ = new_config;
            } else {
                tracing::warn!("invalid config reload — keeping last-known-good");
            }

            let response = match case_dispatch::submit(payload, state).await {
                Ok(output) => {
                    let case_id = output.case_id;
                    let entry = CaseEntry {
                        case_id: case_id.clone(),
                        state: output.state.into(),
                        exit_code: Some(output.exit_code as i32),
                        run_dir: None,
                    };
                    state.cases.lock().await.insert(case_id.clone(), entry);

                    // Fire notify_command (failure is logged and ignored — §10.3).
                    if let Some(argv) = &state.config.notify_command {
                        if !argv.is_empty() {
                            let event = serde_json::json!({
                                "event": "run_finished",
                                "case_id": case_id,
                            });
                            let _ = fire_notify(argv, event);
                        }
                    }

                    // Emit a durable SFWP event for the successful mutation.
                    let _ = state
                        .publish_event(
                            "case.submitted",
                            Some(&case_id),
                            None,
                            serde_json::json!({"state": output.state}),
                        )
                        .await;

                    serde_json::json!({
                        "case_id": case_id,
                        "state": output.state,
                        "exit_code": output.exit_code,
                    })
                }
                Err(e) => serde_json::json!({"error": e.to_string()}),
            };
            record_outcome(state, request_id.as_deref(), "case.submit", &response);
            response
        }
        Request::Status { case_id } => {
            let cases = state.cases.lock().await;
            match cases.get(&case_id) {
                Some(entry) => serde_json::to_value(entry).unwrap_or_default(),
                None => serde_json::json!({"error": "case not found"}),
            }
        }
        Request::Approve {
            case_id,
            approval_id,
            note,
            request_id,
            preconditions,
        } => {
            decide(
                state,
                "approve",
                "approval.approve",
                "approval.approved",
                &case_id,
                &approval_id,
                note.as_deref(),
                request_id.as_deref(),
                preconditions.as_ref(),
            )
            .await
        }
        Request::Reject {
            case_id,
            approval_id,
            note,
            request_id,
            preconditions,
        } => {
            decide(
                state,
                "reject",
                "approval.reject",
                "approval.rejected",
                &case_id,
                &approval_id,
                note.as_deref(),
                request_id.as_deref(),
                preconditions.as_ref(),
            )
            .await
        }
        Request::AgentList => agent_probe::list(&state.config.agent),
        Request::AgentProbe {
            endpoint,
            prompt,
            model,
            policy,
            entity,
            process,
        } => {
            let permit = match state.semaphore.acquire().await {
                Ok(permit) => permit,
                Err(_) => return serde_json::json!({"error":"server semaphore unavailable"}),
            };
            let config = state.config.clone();
            let result = agent_probe::probe(
                &config,
                agent_probe::ProbeRequest {
                    endpoint_id: &endpoint,
                    prompt: &prompt,
                    model: model.as_deref(),
                    policy_path: &policy,
                    entity: &entity,
                    process: &process,
                },
                &agent_probe::EnvironmentCredentialResolver,
            )
            .await;
            drop(permit);
            match result {
                Ok(outcome) => serde_json::to_value(outcome).unwrap_or_else(
                    |_| serde_json::json!({"error":"probe response serialization failed"}),
                ),
                Err(error) => {
                    serde_json::json!({"error": error.to_string(), "error_class": error.class()})
                }
            }
        }
        Request::Delegate {
            endpoint,
            instruction,
            run_id: requested_run_id,
            model,
            max_turns,
            token_budget,
            criteria,
            policy,
            entity,
            process,
            request_id,
        } => {
            record_pending(state, request_id.as_deref(), "agent_run.delegate");
            let response = delegate_inner(
                state,
                &endpoint,
                &instruction,
                requested_run_id,
                model.as_deref(),
                max_turns,
                token_budget,
                criteria,
                &policy,
                &entity,
                &process,
            )
            .await;
            record_outcome(
                state,
                request_id.as_deref(),
                "agent_run.delegate",
                &response,
            );
            response
        }
        Request::CancelDelegation {
            run_id,
            policy,
            entity,
            process,
            request_id,
        } => {
            record_pending(state, request_id.as_deref(), "agent_run.cancel_delegation");
            let response = cancel_delegation(state, &run_id, &policy, &entity, &process).await;
            record_outcome(
                state,
                request_id.as_deref(),
                "agent_run.cancel_delegation",
                &response,
            );
            response
        }
        Request::Ask {
            kind,
            subject,
            purpose,
            case,
            actor,
        } => {
            let Some(question_kind) = sea_forge_thoth::protocol::parse_question_kind(&kind) else {
                return serde_json::json!({"error": format!("unknown question kind: {kind}")});
            };
            let root = state.config.root.clone();
            let result = tokio::task::spawn_blocking(move || {
                sea_forge_thoth::service::ask(
                    &root,
                    &actor,
                    question_kind,
                    &subject,
                    &purpose,
                    case.as_deref(),
                )
            })
            .await
            .map_err(|e| ForgeError::Internal(format!("ask task panic: {e}")))
            .and_then(|result| result);
            match result {
                Ok(answer) => serde_json::to_value(answer).unwrap_or_else(
                    |_| serde_json::json!({"error": "answer serialization failed"}),
                ),
                Err(error) => {
                    serde_json::json!({"error": error.to_string(), "error_class": error.class()})
                }
            }
        }

        // --- SFWP additive methods (Task 3) --------------------------------
        Request::SystemHello {
            protocol_version,
            client: _,
        } => match sfwp::hello(&protocol_version) {
            Ok(result) => serde_json::to_value(result)
                .unwrap_or_else(|_| serde_json::json!({"error": "hello serialization failed"})),
            // Rejected protocol versions are an explicit error response (not
            // a struct dump masquerading as a successful result): the
            // `error`/`error_class` discriminators mirror the rest of
            // `handle_request`, and the UnsupportedVersion payload fields are
            // surfaced verbatim for the client to act on.
            Err(unsupported) => serde_json::json!({
                "error": unsupported.error,
                "error_class": unsupported.error_class,
                "requested": unsupported.requested,
                "supported": unsupported.supported,
            }),
        },
        Request::SystemDescribe => serde_json::to_value(sfwp::describe())
            .unwrap_or_else(|_| serde_json::json!({"error": "describe serialization failed"})),
        Request::SystemGetSchema { method } => {
            serde_json::to_value(sfwp::get_schema(method.as_deref()))
                .unwrap_or_else(|_| serde_json::json!({"error": "get_schema serialization failed"}))
        }
        Request::RequestGetStatus { request_id } => match state.correlation.get(&request_id) {
            Ok(Some(record)) => serde_json::json!({
                "request_id": record.request_id,
                "status": record.status,
                "method": record.method,
                "outcome": record.outcome,
            }),
            Ok(None) => serde_json::json!({
                "request_id": request_id,
                "status": "unknown",
            }),
            Err(error) => {
                serde_json::json!({"error": error.to_string(), "error_class": error.class()})
            }
        },
        Request::EventsGetRange {
            from_cursor,
            to_cursor,
            limit,
        } => match state
            .events_get_range(from_cursor.as_deref(), to_cursor.as_deref(), limit)
            .await
        {
            Ok(events) => serde_json::json!({"events": events}),
            Err(error) => {
                serde_json::json!({"error": error.to_string(), "error_class": error.class()})
            }
        },
        // Inspect projection: always answers with a shaped view (infallible),
        // per `unknown ≠ unavailable`. Mirrors the thin `system.describe` arm —
        // call the typed builder and serialize its result directly, no wrapper.
        Request::ReadinessGet { intended_operation } => {
            let view = sfwp::readiness::get(
                &state.config,
                sfwp::readiness::ReadinessGetParams { intended_operation },
            );
            serde_json::to_value(view)
                .unwrap_or_else(|_| serde_json::json!({"error": "readiness serialization failed"}))
        }
        // These two are connection-scoped and handled inside
        // `handle_connection`; reaching here means a caller invoked
        // `handle_request` directly (in-process), where no live subscription
        // exists. Answer cleanly rather than panicking.
        Request::EventsSubscribe { .. } | Request::EventsUnsubscribe => {
            serde_json::json!({
                "error": "events.subscribe/unsubscribe require a live connection",
                "error_class": "input_error",
            })
        }
    }
}

/// Record a correlated request as `pending` before its work begins. No-op if
/// the client did not supply a `request_id`. Correlation-store failures are
/// logged, never fatal to the request.
fn record_pending(state: &Arc<ServerState>, request_id: Option<&str>, method: &str) {
    if let Some(id) = request_id {
        if let Err(error) = state.correlation.record_pending(id, method) {
            tracing::warn!("correlation record_pending failed: {error}");
        }
    }
}

/// Record a correlated request's terminal outcome. No-op without a
/// `request_id`. This is what makes `request.get_status` recover the outcome
/// after a client disconnects mid-request and reconnects.
fn record_outcome(
    state: &Arc<ServerState>,
    request_id: Option<&str>,
    method: &str,
    outcome: &serde_json::Value,
) {
    if let Some(id) = request_id {
        if let Err(error) = state.correlation.record_outcome(id, method, outcome) {
            tracing::warn!("correlation record_outcome failed: {error}");
        }
    }
}

/// Resolves precondition record refs against current ledger/case truth so a
/// stale `expected_digest` can be detected. Supported ref forms:
/// `case:<case_id>` and `approval:<approval_id>` (the latter resolves the
/// latest matching approval-shaped ledger record across the case ledgers).
struct LedgerRecordResolver<'a> {
    root: &'a Path,
    case_id: &'a str,
}

impl sfwp::precondition::RecordResolver for LedgerRecordResolver<'_> {
    fn resolve(&self, r#ref: &str) -> Result<Option<serde_json::Value>, ForgeError> {
        let (kind, id) = match r#ref.split_once(':') {
            Some(parts) => parts,
            None => return Ok(None),
        };
        // Do not create the case ledger as a side effect of a read-only
        // precondition check: if it does not exist, the referenced record does
        // not exist and the digest cannot match (correctly triggering a stale
        // rejection with no mutation).
        let ledger_dir = self
            .root
            .join("ledgers")
            .join(format!("case-{}", self.case_id));
        if !ledger_dir.exists() {
            return Ok(None);
        }
        let ledger = LedgerStream::open(
            self.root,
            format!("case-{}", self.case_id),
            "sea-forge-server",
        )?;
        let entries = ledger.read_entries()?;
        match kind {
            "case" => {
                // A `case:<id>` ref must name this resolver's own case; a
                // mismatched id is an unresolvable reference (returns the
                // established `None` so the precondition stale-rejects),
                // never a fingerprint of this case's ledger mislabeled with a
                // foreign id.
                if id != self.case_id {
                    return Ok(None);
                }
                // The canonical current case digest is the ordered list of
                // record kinds + payload hashes committed for this case — a
                // stable fingerprint that changes whenever the case advances.
                let fingerprint: Vec<serde_json::Value> = entries
                    .iter()
                    .map(|entry| {
                        serde_json::json!({
                            "kind": entry.record_kind,
                            "payload_hash": entry.payload_hash,
                            "ordinal": entry.append_ordinal,
                        })
                    })
                    .collect();
                Ok(Some(serde_json::json!({
                    "case_id": id,
                    "records": fingerprint,
                })))
            }
            "approval" => {
                // Latest ledger record whose payload names this approval id.
                let latest = entries.iter().rev().find(|entry| {
                    entry.payload["approval_id"].as_str() == Some(id)
                        || entry.payload["approvalId"].as_str() == Some(id)
                });
                Ok(latest.map(|entry| entry.payload.clone()))
            }
            _ => Ok(None),
        }
    }
}

/// Handle an `approve`/`reject` decision with optional precondition + request
/// correlation. On a stale precondition, no side effect is performed and a
/// structured `rejected_as_stale` body is returned (mirrors the no-side-effect
/// discipline of the authority-deny paths).
#[allow(clippy::too_many_arguments)]
async fn decide(
    state: &Arc<ServerState>,
    cli_verb: &str,
    method: &str,
    event_kind: &str,
    case_id: &str,
    approval_id: &str,
    note: Option<&str>,
    request_id: Option<&str>,
    preconditions: Option<&sfwp::precondition::Precondition>,
) -> serde_json::Value {
    record_pending(state, request_id, method);

    // Evaluate preconditions first — before any side effect.
    if let Some(precondition) = preconditions {
        let resolver = LedgerRecordResolver {
            root: &state.config.root,
            case_id,
        };
        match sfwp::precondition::evaluate(precondition, &resolver) {
            Ok(Some(rejected)) => {
                let response = serde_json::to_value(&rejected).unwrap_or_else(
                    |_| serde_json::json!({"error": "rejected_as_stale serialization failed"}),
                );
                // No side effect performed; still record the terminal outcome.
                record_outcome(state, request_id, method, &response);
                return response;
            }
            Ok(None) => {}
            Err(error) => {
                let response =
                    serde_json::json!({"error": error.to_string(), "error_class": error.class()});
                record_outcome(state, request_id, method, &response);
                return response;
            }
        }
    }

    let root = state.config.root.clone();
    let result = run_cli(
        &root,
        &[
            cli_verb,
            case_id,
            approval_id,
            "--root",
            root.to_str().unwrap_or("."),
        ],
        note,
    )
    .await;
    let response = match result {
        Ok(output) => {
            let _ = state.permission_broker.resolve(approval_id).await;
            let _ = state
                .publish_event(
                    event_kind,
                    Some(case_id),
                    None,
                    serde_json::json!({"approval_id": approval_id}),
                )
                .await;
            serde_json::json!({"ok": true, "output": output})
        }
        Err(e) => serde_json::json!({"error": e}),
    };
    record_outcome(state, request_id, method, &response);
    response
}

/// Inner delegation execution, factored out so the caller can uniformly record
/// the correlated outcome regardless of which early-exit path is taken.
#[allow(clippy::too_many_arguments)]
async fn delegate_inner(
    state: &Arc<ServerState>,
    endpoint: &str,
    instruction: &str,
    requested_run_id: Option<String>,
    model: Option<&str>,
    max_turns: u32,
    token_budget: Option<u64>,
    criteria: sea_forge_core::types::SettlementCriteria,
    policy: &str,
    entity: &str,
    process: &str,
) -> serde_json::Value {
    let delegation_run_id = match requested_run_id {
        Some(value) if valid_run_id(&value) => value,
        Some(_) => return serde_json::json!({"error":"invalid delegation run_id"}),
        None => match run_id() {
            Ok(value) => value,
            Err(error) => return serde_json::json!({"error": error.to_string()}),
        },
    };
    let delegation_case_id = match case_id() {
        Ok(value) => value,
        Err(error) => return serde_json::json!({"error": error.to_string()}),
    };
    let permit = match state.semaphore.acquire().await {
        Ok(permit) => permit,
        Err(_) => return serde_json::json!({"error":"server semaphore unavailable"}),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let requested = Arc::new(AtomicBool::new(false));
    let handle = DelegationHandle {
        case_id: delegation_case_id.clone(),
        cancel: Arc::clone(&cancel),
        requested,
    };
    if state
        .delegations
        .lock()
        .await
        .insert(delegation_run_id.clone(), handle)
        .is_some()
    {
        drop(permit);
        return serde_json::json!({"error":"delegation run_id already active"});
    }
    let config = state.config.clone();
    let run_for_execution = delegation_run_id.clone();
    let case_for_execution = delegation_case_id.clone();
    let cancel_for_execution = Arc::clone(&cancel);
    let result = delegation::execute_with_permission_broker(
        &config,
        delegation::DelegationRequest {
            endpoint_id: endpoint,
            instruction,
            model,
            max_turns,
            token_budget,
            criteria,
            policy_path: policy,
            entity,
            process,
            ..Default::default()
        },
        &agent_probe::EnvironmentCredentialResolver,
        delegation::DelegationEpisodeContext::standalone(&case_for_execution, &run_for_execution),
        move || cancel_for_execution.load(Ordering::SeqCst),
        Some(state.permission_broker.clone()),
    )
    .await;
    state.delegations.lock().await.remove(&delegation_run_id);
    drop(permit);
    match result {
        Ok(outcome) => {
            let _ = state
                .publish_event(
                    "agent_run.delegated",
                    Some(&delegation_case_id),
                    Some(&delegation_run_id),
                    serde_json::json!({"endpoint": endpoint}),
                )
                .await;
            serde_json::to_value(outcome).unwrap_or_else(
                |_| serde_json::json!({"error":"delegation response serialization failed"}),
            )
        }
        Err(error) => {
            serde_json::json!({"error": error.to_string(), "error_class": error.class()})
        }
    }
}

#[derive(Serialize)]
struct CancellationRequestRecord {
    version: &'static str,
    control_id: String,
    case_id: String,
    run_id: String,
    plan_item_id: &'static str,
    requester: String,
    authority_decision_ref: String,
    requested_at: String,
    ordinal: u64,
}

async fn cancel_delegation(
    state: &Arc<ServerState>,
    run_id: &str,
    policy_path: &str,
    entity: &str,
    process: &str,
) -> serde_json::Value {
    let handle = match state.delegations.lock().await.get(run_id).cloned() {
        Some(handle) => handle,
        None => return serde_json::json!({"error":"delegation run not active"}),
    };

    if handle
        .requested
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return serde_json::json!({
            "run_id": run_id,
            "state": "cancellation_already_requested"
        });
    }

    let result = record_cancellation(
        &state.config,
        run_id,
        &handle.case_id,
        policy_path,
        entity,
        process,
    );
    match result {
        Ok(control_id) => {
            handle.cancel.store(true, Ordering::SeqCst);
            let _ = state
                .publish_event(
                    "agent_run.cancellation_requested",
                    Some(&handle.case_id),
                    Some(run_id),
                    serde_json::json!({"control_id": control_id}),
                )
                .await;
            serde_json::json!({
                "run_id": run_id,
                "state": "cancellation_requested",
                "control_id": control_id
            })
        }
        Err(error) => {
            handle.requested.store(false, Ordering::SeqCst);
            serde_json::json!({"error": error.to_string(), "error_class": error.class()})
        }
    }
}

fn record_cancellation(
    config: &ServerConfig,
    run_id: &str,
    case_id: &str,
    policy_path: &str,
    entity: &str,
    process: &str,
) -> Result<String, sea_forge_core::ForgeError> {
    let policy = agent_probe::resolve_policy_path(&config.root, policy_path);
    let bundle = AuthorityPolicyBundle::load(&policy)?;
    let engine = PolicyAuthorityEngine::new(bundle.clone())?;
    let actor = Actor {
        actor_id: entity.into(),
        role: ActorRole::Operator,
    };
    let action = AuthorityAction::Reserved {
        resource_type: "run_cancel".into(),
        resource_id: run_id.into(),
        parameters: serde_json::json!({"process": process}),
    };
    let ledger = LedgerStream::open(&config.root, format!("case-{case_id}"), "sea-forge-server")?;
    let decision = engine.evaluate(AuthorityEvaluation {
        actor: &actor,
        binding: bundle.resolve_identity(&actor.actor_id, actor.role.clone()),
        run_id,
        case_id,
        plan_item_id: "control_cancel",
        sequence: 1,
        action: &action,
        workspace_root: &config.root,
        evidence_refs: vec![],
        artifacts_root: None,
        timeout_secs: None,
        env_keys: Default::default(),
        domainforge_candidate: None,
        environment: None,
    })?;
    let authority_ref = agent_probe::commit_view(
        &ledger,
        "authority_decision",
        vec![run_id.into(), "control_cancel".into()],
        &decision,
        &config
            .root
            .join("runs")
            .join(run_id)
            .join("cancellation-authority.json"),
        vec![],
    )?;
    if decision.verdict != Verdict::Allow {
        return Err(sea_forge_core::ForgeError::Input(
            "cancellation authority denied".into(),
        ));
    }

    let control_id = random_id("cancel")?;
    let record = CancellationRequestRecord {
        version: "0.2",
        control_id: control_id.clone(),
        case_id: case_id.into(),
        run_id: run_id.into(),
        plan_item_id: "agent_task",
        requester: entity.into(),
        authority_decision_ref: authority_ref.entry_ulid().into(),
        requested_at: Utc::now().to_rfc3339(),
        ordinal: 1,
    };
    agent_probe::commit_view(
        &ledger,
        "control_request",
        vec![case_id.into(), run_id.into(), "agent_task".into()],
        &record,
        &config
            .root
            .join("runs")
            .join(run_id)
            .join(format!("control-{control_id}.json")),
        vec![authority_ref.entry_ulid().into()],
    )?;
    Ok(control_id)
}

struct DispatchOutcome {
    case_id: String,
    state: &'static str,
    exit_code: u8,
}

async fn run_cli(_root: &Path, args: &[&str], note: Option<&str>) -> Result<String, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("resolve exe: {e}"))?
        .to_string_lossy()
        .into_owned();
    let cli_exe = {
        let p = std::path::PathBuf::from(&exe);
        let dir = p.parent().unwrap_or(std::path::Path::new("."));
        let cli = dir.join("sea-forge");
        if cli.exists() {
            cli.to_string_lossy().into_owned()
        } else {
            exe.clone()
        }
    };
    let mut cmd = Command::new(&cli_exe);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(note) = note {
        cmd.arg("--note").arg(note);
    }
    let output = cmd.output().await.map_err(|e| format!("spawn: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("exit {:?}: {stderr}", output.status.code()));
    }
    Ok(stdout)
}

fn fire_notify(argv: &[String], event: serde_json::Value) -> Result<(), String> {
    if argv.is_empty() {
        return Ok(());
    }
    let mut cmd = std::process::Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| format!("spawn notify: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = serde_json::to_writer(&mut stdin, &event);
        let _ = stdin.write_all(b"\n");
    }
    let _ = child.wait();
    Ok(())
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;

    #[tokio::test]
    async fn cancellation_is_authorized_and_durably_recorded_before_signalling() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("policy.yaml"),
            "version: \"0.1\"\nrules:\n  - name: allow-cancel\n    verdict: allow\n    actor_role: operator\n    operation_kind: run_cancel\n",
        )
        .unwrap();
        let config = ServerConfig {
            root: root.path().to_path_buf(),
            ..ServerConfig::default()
        };
        let state = Arc::new(ServerState::new(config).unwrap());
        let run = run_id().unwrap();
        let case = case_id().unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        state.delegations.lock().await.insert(
            run.clone(),
            DelegationHandle {
                case_id: case.clone(),
                cancel: Arc::clone(&cancel),
                requested: Arc::new(AtomicBool::new(false)),
            },
        );

        let response =
            cancel_delegation(&state, &run, "policy.yaml", "operator_local", "test").await;

        assert!(response.get("error").is_none(), "{response}");
        assert_eq!(response["state"], "cancellation_requested");
        assert!(cancel.load(Ordering::SeqCst));
        let ledger = LedgerStream::open(root.path(), format!("case-{case}"), "test").unwrap();
        let entries = ledger.read_entries().unwrap();
        assert!(entries
            .iter()
            .any(|entry| entry.record_kind == "authority_decision"));
        assert!(entries
            .iter()
            .any(|entry| entry.record_kind == "control_request"));
    }
}
