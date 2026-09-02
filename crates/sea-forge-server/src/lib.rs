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
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{
    atomic::{AtomicBool, AtomicU8, Ordering},
    Arc,
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::process::Command;
use tokio::sync::{broadcast, mpsc, Mutex, Semaphore};

use sfwp::correlation::RequestCorrelationStore;
use sfwp::events::EventFrame;

/// At most eight protected socket requests may wait behind admitted work.
/// Together with `max_concurrent_runs` active permits, this bounds request work
/// below the 64-connection transport cap without serializing read-only views.
const REQUEST_ADMISSION_WAIT_QUEUE_CAPACITY: usize = 8;

pub mod agent_probe;
pub mod case_dispatch;
pub mod config;
pub mod delegation;
pub mod governed_execution_boundary;
pub mod governed_settlement_return;
pub mod governed_work_ingress;
pub mod identity;
pub mod sfwp;
pub mod swe_seed_reconciliation;
mod transcript_seal;

pub use config::{resolve_cell_root, resolve_socket_override, ServerConfig};

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
    /// The cell this server owns. Fixed for the process lifetime: `server.yaml`
    /// lives *inside* the cell it configures, so a reload cannot relocate it
    /// (see `docs/CELL_CONTRACT.md`). Keeping it out of the reloadable snapshot
    /// makes that unrepresentable rather than merely documented.
    pub root: PathBuf,
    /// Live configuration snapshot (§8.4). `reload_config` swaps the whole
    /// `Arc`; a dispatch that already took one keeps it to completion, so new
    /// configuration reaches future dispatches only.
    config: std::sync::RwLock<Arc<ServerConfig>>,
    pub cases: Mutex<HashMap<String, CaseEntry>>,
    delegations: Mutex<HashMap<String, DelegationHandle>>,
    pub(crate) permission_broker: delegation::AcpApprovalBroker,
    pub semaphore: Arc<Semaphore>,
    /// Server-wide request-work admission. Its capacity matches
    /// `max_concurrent_runs`; a request must hold one before it can create a
    /// correlation record or reach any governed effect.
    request_admission: Arc<Semaphore>,
    /// Bounded waiting room for request admission. The fixed limit is
    /// deliberately separate from run capacity: at most this many socket
    /// tasks may wait for 10 seconds, and every further protected request is
    /// refused as `server_busy` without creating durable state.
    request_admission_waiters: Arc<Semaphore>,
    /// SFWP request-correlation store (Task 3): answers `request.get_status`
    /// across reconnects.
    pub(crate) correlation: RequestCorrelationStore,
    /// Serializes check-and-bind for one request id. The cell lock guarantees
    /// this server is the only process writer; this mutex closes the remaining
    /// same-process race between concurrent socket connections.
    correlation_admission: Mutex<()>,
    /// The single durable SFWP events ledger; its `entry_ulid` is the event
    /// cursor. F-24: shared behind an `Arc` and accessed from blocking
    /// threads — the ledger's own `flock` serializes concurrent appends, so a
    /// tokio mutex (which used to be held across the blocking append) is not
    /// needed and would stall every publisher/subscriber on one slow fsync.
    events_ledger: Arc<LedgerStream>,
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
        config.validate().map_err(ForgeError::Input)?;
        recover_cancelled_delegations(&config)?;
        // A SWE_SEED declaration may have been appended directly (out of
        // process) while the server was absent; join it against any
        // harvested evidence before this server accepts new work (M16 T18).
        swe_seed_reconciliation::reconcile_all_cases(&config.root)?;
        let max = config.max_concurrent_runs;
        let correlation = RequestCorrelationStore::open(&config.root)?;
        // A process crash can leave a correlation record pending after it was
        // lawfully admitted. Settle that request as interrupted before serving
        // retries; otherwise the same id is either a duplicate risk or a
        // permanent pending lie after restart.
        correlation.settle_interrupted_requests()?;
        let events_ledger = sfwp::events::open_events_ledger(&config.root)?;
        let (event_bus, _) = broadcast::channel::<EventFrame>(256);
        // F-21: rebuild the in-memory `cases` map from disk at startup. Without
        // this, a restart answers `"case not found"` on the `status` verb for
        // every pre-restart case even though disk-backed `case.list` shows it.
        // `exit_code`/`run_dir` are re-derived per-dispatch and are not
        // reconstructible from the single `case.json` row, so they start `None`
        // (the map is a liveness cache; `case.list` remains disk authority).
        let mut cases = HashMap::new();
        for row in sfwp::case_views::list(&config.root).cases {
            cases.insert(
                row.case_id.clone(),
                CaseEntry {
                    case_id: row.case_id,
                    state: row.case_state,
                    exit_code: None,
                    run_dir: None,
                },
            );
        }
        Ok(Self {
            root: config.root.clone(),
            config: std::sync::RwLock::new(Arc::new(config)),
            cases: Mutex::new(cases),
            delegations: Mutex::new(HashMap::new()),
            permission_broker: delegation::AcpApprovalBroker::default(),
            semaphore: Arc::new(Semaphore::new(max)),
            request_admission: Arc::new(Semaphore::new(max)),
            request_admission_waiters: Arc::new(Semaphore::new(
                REQUEST_ADMISSION_WAIT_QUEUE_CAPACITY,
            )),
            correlation,
            correlation_admission: Mutex::new(()),
            events_ledger: Arc::new(events_ledger),
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
        // F-24: the append does flock + fsync + a tail scan — blocking work
        // belongs on a blocking thread, never on a tokio worker.
        let ledger = Arc::clone(&self.events_ledger);
        let kind = kind.to_string();
        let case_id = case_id.map(str::to_owned);
        let run_id = run_id.map(str::to_owned);
        let frame = tokio::task::spawn_blocking(move || {
            sfwp::events::append_event(
                &ledger,
                &kind,
                case_id.as_deref(),
                run_id.as_deref(),
                detail,
            )
        })
        .await
        .map_err(|error| ForgeError::Internal(format!("events append task panic: {error}")))??;
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
        let ledger = Arc::clone(&self.events_ledger);
        let from_cursor = from_cursor.map(str::to_owned);
        let to_cursor = to_cursor.map(str::to_owned);
        tokio::task::spawn_blocking(move || {
            sfwp::events::get_range(&ledger, from_cursor.as_deref(), to_cursor.as_deref(), limit)
        })
        .await
        .map_err(|error| ForgeError::Internal(format!("events read task panic: {error}")))?
    }

    /// Replay durable events after a cursor for a subscribe catch-up burst.
    pub(crate) async fn events_replay_after(
        &self,
        from_cursor: Option<&str>,
    ) -> Result<Vec<EventFrame>, ForgeError> {
        let ledger = Arc::clone(&self.events_ledger);
        let from_cursor = from_cursor.map(str::to_owned);
        tokio::task::spawn_blocking(move || {
            sfwp::events::replay_after(&ledger, from_cursor.as_deref())
        })
        .await
        .map_err(|error| ForgeError::Internal(format!("events replay task panic: {error}")))?
    }

    /// The current configuration snapshot.
    ///
    /// Hold the returned `Arc` for the whole of a dispatch: a reload that lands
    /// mid-dispatch swaps the shared slot, and re-reading it partway through
    /// would let one run straddle two configurations.
    pub fn config(&self) -> Arc<ServerConfig> {
        // A panic while *reading* a config snapshot cannot have left it torn —
        // the `Arc` is swapped whole — so a poisoned lock is recoverable here
        // and failing the request instead would be strictly worse.
        Arc::clone(&self.config.read().unwrap_or_else(|e| e.into_inner()))
    }

    /// Re-read `server.yaml` between dispatches and atomically swap the live
    /// snapshot (§8.4).
    ///
    /// An invalid file leaves the previous snapshot in place untouched — the
    /// swap only happens after the load and validation both succeed, so there
    /// is no window in which the server runs on a half-applied configuration.
    /// The caller is responsible for the operator-visible `invalid_reload_error`
    /// event on `Err`.
    ///
    /// An *absent* file is a no-op, not a reset. `ServerConfig::load` answers
    /// `Ok(default)` for a missing path because that is the correct answer at
    /// startup — a first run has no file and should get defaults. Applying that
    /// same answer to a reload would mean that deleting `server.yaml` on a
    /// running cell silently reverted every setting to its default, which is a
    /// fail-open of exactly the kind §8.4 exists to prevent. Nothing to re-read
    /// means nothing to change.
    pub fn reload_config(&self) -> Result<Arc<ServerConfig>, String> {
        let config_path = self.root.join("server.yaml");
        if !config_path.exists() {
            tracing::debug!(
                config_path = %config_path.display(),
                "no server.yaml to reload; keeping the current configuration"
            );
            return Ok(self.config());
        }
        let mut next = ServerConfig::load(&config_path)?;
        // The cell is fixed at startup. A `root:` or `socket_path:` key that
        // drifted into the file cannot move a running server's records or
        // socket out from under its own connections and in-flight runs.
        next.root = self.root.clone();
        next.socket_path = self.config().socket_path.clone();

        let next = Arc::new(next);
        *self.config.write().unwrap_or_else(|e| e.into_inner()) = Arc::clone(&next);
        Ok(next)
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
        /// Optional caller correlation for a governed probe. A supplied value
        /// makes a post-admission timeout recoverable through request.get_status.
        #[serde(default)]
        request_id: Option<String>,
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
        /// Who is asking. Named `actor_id` rather than `actor` because SF-005
        /// claimed `actor` for the identity block `{actor_id, role}` that every
        /// protected verb carries — and `ask` is protected. Two fields of
        /// different types under one wire name meant the identity gate could
        /// not parse this request's block, so `ask` was refused on every real
        /// socket while the suite stayed green (its tests call `handle_request`
        /// directly, below the gate).
        ///
        /// Ignored when the gate resolved an actor: a caller does not get to
        /// ask questions as someone else.
        #[serde(default = "default_entity", alias = "asker")]
        actor_id: String,
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
    /// Read-only identity projection (SF-005). Reports which actors *this*
    /// connection may claim, so a client can display a resolved identity and
    /// pick an actor instead of inventing one. Answered in `dispatch_bounded`
    /// rather than `handle_request` because it is the only inspect verb whose
    /// answer depends on the connection's peer credential.
    IdentityGet,
    /// Case-authoring inspect methods (Task 6, ADR-003). `entry_options` lists
    /// materialized templates; `preflight` is an audit-only dry run of
    /// `validate_proposal` over an instantiated draft — neither creates a
    /// case, run, or ledger.
    CaseEntryOptions,
    CasePreflight {
        #[serde(flatten)]
        params: sfwp::case::PreflightParams,
    },
    /// Case-authoring protected command (Task 6, ADR-003). Envelopes the
    /// existing `case_dispatch::submit` path: instantiates `template_ref` +
    /// `params` into a `CasePlan` (the same way `case.preflight` did), checks
    /// `preconditions` against the template's *current* bytes, and on match
    /// delegates to the same `commit_plan` helper `Submit` uses — one
    /// governance path, no forked commit logic. On a stale precondition, no
    /// case is created and a structured `rejected_as_stale` body is returned.
    CaseCommit {
        template_ref: String,
        #[serde(default)]
        params: std::collections::BTreeMap<String, String>,
        #[serde(default = "default_policy")]
        policy: String,
        #[serde(default = "default_entity")]
        entity: String,
        #[serde(default = "default_process")]
        process: String,
        #[serde(default = "default_timeout")]
        timeout: u64,
        #[serde(default)]
        request_id: Option<String>,
        #[serde(default)]
        preconditions: Option<sfwp::precondition::Precondition>,
    },
    /// Case-navigation inspect methods (Task 7, ADR-003). Read-only projections
    /// over committed case records — `case.json`, `plan.json`, per-run
    /// `settlement.json`, and the case's own `case-events.jsonl`. None creates
    /// a case, run, or ledger entry; see `sfwp::case_views`.
    CaseList,
    CaseGetOverview {
        case_id: String,
    },
    CaseGetHorizon {
        case_id: String,
    },
    /// List approvals awaiting a decision (Task 7, ADR-003). Read-only
    /// projection over the approvals journal; see `sfwp::approvals`.
    ApprovalList {
        #[serde(default)]
        case_id: Option<String>,
    },
    /// Resolve one approval. A thin envelope over the *same* `decide` path
    /// `Approve`/`Reject` use — including its precondition check and its
    /// correlated-outcome recording — so there is exactly one place an
    /// approval decision can be made. `decision` selects the arm; an
    /// unrecognized value is refused rather than defaulted, because guessing
    /// between approve and reject is never a safe default.
    ApprovalDecide {
        case_id: String,
        approval_id: String,
        decision: String,
        #[serde(default)]
        note: Option<String>,
        #[serde(default)]
        request_id: Option<String>,
        #[serde(default)]
        preconditions: Option<sfwp::precondition::Precondition>,
    },
    /// Run-record inspect methods (Task 8, ADR-003). Read-only projections over
    /// `<root>/runs/<run_id>/*` — the resolution target for the run ids the
    /// case, horizon, approval, and event views already emit. Neither creates a
    /// run or ledger entry; see `sfwp::run_views`.
    RunList {
        #[serde(default)]
        case_id: Option<String>,
    },
    RunGet {
        run_id: String,
    },
    /// Asset-catalog inspect method (Task 9, ADR-003). Read-only projection
    /// over materialized templates, configured agent endpoints, and the
    /// extension registry, with each endpoint's standing derived from the probe
    /// records rather than asserted. Creates nothing; see `sfwp::assets`.
    AssetList,
    /// Delegation job-contract inspect method (Task 10, ADR-003). Projects the
    /// contract a `delegate` with these exact inputs would run under — resolved
    /// model, caps, timeout, retention, and the authority action that *will be*
    /// submitted. It commits nothing and decides nothing: see
    /// `sfwp::delegation_preview` for why a preview verdict would be an
    /// authority claim with no record behind it.
    DelegationPreview {
        #[serde(flatten)]
        params: sfwp::delegation_preview::DelegationPreviewParams,
    },
    /// Delegation roster inspect method (Task 11, ADR-003). Joins the server's
    /// live delegation handles with the committed run records so the already-
    /// reachable `cancel_delegation` verb finally has enumerable targets.
    /// Creates nothing; see `sfwp::delegations` for why a run with neither a
    /// handle nor a settlement is reported as unresolved rather than guessed at.
    DelegationList,
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

/// `path` with `suffix` appended to its file name (`forge.sock` → `forge.sock.lock`).
///
/// Deliberately not `with_extension`, which would *replace* `.sock` and could
/// collide with an unrelated sibling.
fn suffixed(path: &Path, suffix: &str) -> Result<PathBuf, ForgeError> {
    let name = path.file_name().ok_or_else(|| {
        ForgeError::Input(format!("socket path has no file name: {}", path.display()))
    })?;
    let mut name = name.to_os_string();
    name.push(suffix);
    Ok(path.with_file_name(name))
}

/// The longest suffix `run` appends to the socket path before binding.
/// `.binding` is longer than `.lock`, so budgeting for it covers both.
const LONGEST_SOCKET_SUFFIX: usize = ".binding".len();

/// Reject a socket path that cannot fit in `sockaddr_un.sun_path` *before* the
/// server creates any directory, lock file, or ledger.
///
/// Without this check the kernel rejects the path at `bind()` — several side
/// effects later — with a bare `InvalidInput: path must be shorter than
/// SUN_LEN`, which names neither the offending path nor the fix. A cell root
/// nested a few directories deep is enough to trigger it, so an operator hits
/// this while their configuration looks perfectly reasonable.
///
/// The budget is the platform's `sun_path` capacity, less one byte for the NUL
/// terminator, less the longest suffix appended before binding.
/// Startup preflight for `notify_command` (§8.5): the configured argv0 must
/// exist and be executable.
///
/// This is the one hook whose failure is *deliberately* swallowed at runtime
/// ("failure is logged and ignored — §10.3"), which is right for a transient
/// hook error but wrong for a hook that can never run at all. Without this
/// check a typo in `notify_command` costs the operator every completion
/// notification for the life of the cell, and the only symptom is a log line
/// nobody reads because they are waiting on the notification.
fn check_notify_command(argv: Option<&[String]>) -> Result<(), ForgeError> {
    let Some([program, ..]) = argv else {
        // Unset, or set to an empty argv, which the notify path already skips.
        return Ok(());
    };

    // A bare name is resolved against PATH at spawn time, so resolve it the
    // same way here instead of stat-ing a relative path that would only
    // resolve against the current directory.
    let resolved = if program.contains(std::path::MAIN_SEPARATOR) {
        std::path::Path::new(program).is_file()
    } else {
        std::env::var_os("PATH")
            .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
            .unwrap_or(false)
    };

    if resolved {
        return Ok(());
    }
    Err(ForgeError::Input(format!(
        "notify_command argv0 not found: {program}\n\
         Fix or remove `notify_command` in server.yaml. A relative name is \
         resolved against PATH; use an absolute path to pin it."
    )))
}

fn check_socket_path_length(socket_path: &Path) -> Result<(), ForgeError> {
    // `sun_path` is 108 bytes on Linux and 104 on macOS; use the smaller bound
    // so a cell that starts on Linux is not rejected only after moving.
    const SUN_PATH_CAPACITY: usize = 104;
    let budget = SUN_PATH_CAPACITY - 1 - LONGEST_SOCKET_SUFFIX;

    let length = socket_path.as_os_str().as_encoded_bytes().len();
    if length <= budget {
        return Ok(());
    }
    Err(ForgeError::Input(format!(
        "socket path is {length} bytes but a Unix socket allows at most {budget}: {}\n\
         Choose a shorter cell root (SEA_FORGE_ROOT), or point SEA_FORGE_SOCKET at a \
         short path such as /run/user/$UID/sea-forge.sock while keeping records where they are.",
        socket_path.display()
    )))
}

/// Take an exclusive lifetime lock for the cell root. A socket override must
/// not let two servers mutate the same ledgers through different socket paths.
fn lock_cell_root(root: &Path) -> Result<std::fs::File, ForgeError> {
    std::fs::create_dir_all(root).map_err(|e| ForgeError::io("create server cell root", e))?;
    let lock_path = root.join(".server.lock");
    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .map_err(|e| ForgeError::io("open server cell lock", e))?;
    match lock_file.try_lock() {
        Ok(()) => Ok(lock_file),
        Err(std::fs::TryLockError::WouldBlock) => Err(ForgeError::io(
            format!(
                "another sea-forge server already owns cell {}",
                root.display()
            ),
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "server cell lock held"),
        )),
        Err(std::fs::TryLockError::Error(e)) => Err(ForgeError::io("lock server cell", e)),
    }
}

/// Refuse to publish a socket over a non-socket object. Replacing a stale
/// socket inode is safe; replacing an operator's regular file or symlink is not.
fn validate_socket_destination(socket_path: &Path) -> Result<(), ForgeError> {
    let metadata = match std::fs::symlink_metadata(socket_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(ForgeError::io("inspect server socket destination", error)),
    };
    if metadata.file_type().is_symlink() {
        return Err(ForgeError::Input(format!(
            "refusing to replace symlink configured as server socket: {}",
            socket_path.display()
        )));
    }
    #[cfg(unix)]
    if std::os::unix::fs::FileTypeExt::is_socket(&metadata.file_type()) {
        return Ok(());
    }
    Err(ForgeError::Input(format!(
        "refusing to replace non-socket configured as server socket: {}",
        socket_path.display()
    )))
}

/// Take the exclusive, process-lifetime lock guarding one socket path.
///
/// This is what makes a second server fail instead of silently unlinking a
/// *live* socket: two servers sharing one socket path would both append to the
/// same JSONL ledger and MMR, and the append-only invariant has no
/// cross-process reconciliation for interleaved writers. Uses the same
/// advisory-lock mechanism as the ledger's append stream, so the check is
/// atomic rather than a connect-then-bind race.
///
/// The returned handle must stay in scope for as long as the server runs;
/// dropping it releases the lock.
fn lock_socket_path(socket_path: &Path) -> Result<std::fs::File, ForgeError> {
    let lock_path = suffixed(socket_path, ".lock")?;
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| ForgeError::io("create server socket directory", e))?;
    }
    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .map_err(|e| ForgeError::io("open server socket lock", e))?;
    match lock_file.try_lock() {
        Ok(()) => Ok(lock_file),
        Err(std::fs::TryLockError::WouldBlock) => Err(ForgeError::io(
            format!(
                "another sea-forge server is already listening on {}",
                socket_path.display()
            ),
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "server socket lock held"),
        )),
        Err(std::fs::TryLockError::Error(e)) => Err(ForgeError::io("lock server socket", e)),
    }
}

/// Start the server.
pub async fn run(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = config.resolved_socket_path();

    // Fail before the first side effect: `ServerState::new` creates the cell's
    // directories and ledgers, so an unbindable socket path must be rejected
    // ahead of it rather than leaving a half-built cell behind.
    config.validate().map_err(ForgeError::Input)?;
    check_socket_path_length(&socket_path)?;
    check_notify_command(config.notify_command.as_deref())?;
    validate_socket_destination(&socket_path)?;

    // Both locks live for the server lifetime. The root lock makes an explicit
    // socket override unable to create a second concurrent writer for one cell.
    let _cell_lock = lock_cell_root(&config.root)?;
    let _socket_lock = lock_socket_path(&socket_path)?;
    let state = Arc::new(ServerState::new(config)?);

    // Bind on a staging path, restrict it to 0600, then rename into place.
    // `bind()` creates the socket with umask-derived permissions, so setting
    // the mode *after* binding the live path leaves a window in which other
    // local users can already connect (§11.1). Renaming publishes a socket
    // that is 0600 from the first instant it is reachable, and atomically
    // replaces any stale socket a crashed server left behind — which is why
    // no unlink of the live path is needed. `forbid(unsafe_code)` at the crate
    // root rules out setting `umask` around the bind instead.
    let staging_path = suffixed(&socket_path, ".binding")?;
    let _ = std::fs::remove_file(&staging_path);
    let listener = UnixListener::bind(&staging_path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&staging_path, std::fs::Permissions::from_mode(0o600))?;
    }
    std::fs::rename(&staging_path, &socket_path).inspect_err(|_| {
        // Leave no orphaned staging socket behind on a failed publish.
        let _ = std::fs::remove_file(&staging_path);
    })?;

    tracing::info!("server listening on {}", socket_path.display());

    // A connection holds a bounded writer queue and can otherwise wait for a
    // partial line forever. Admission is deliberately separate from governed
    // run permits: slow or idle peers must not create unbounded Tokio tasks.
    const MAX_CONNECTIONS: usize = 64;
    let connection_permits = Arc::new(Semaphore::new(MAX_CONNECTIONS));

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(accepted) => accepted,
            Err(e) => {
                // One failed accept is not a reason to take the governed
                // kernel down: EMFILE/ENFILE clear as descriptors are
                // released, and ECONNABORTED only means that client went away
                // between its connect and this accept. A listener that is
                // genuinely broken fails every iteration and surfaces as a
                // sustained warn stream rather than a silent process exit.
                // ponytail: fixed 100ms backoff so a permanently broken
                // listener cannot spin hot; a healthy accept never takes this
                // branch, so it costs nothing in the normal path.
                tracing::warn!("accept failed: {e}");
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                continue;
            }
        };
        // Read the peer's OS identity from the kernel *here*, while the
        // accepted stream is still whole. It is the one fact about the caller
        // that cannot be asserted (SF-005, U-07), and `into_split` inside
        // `handle_connection` would put it out of reach.
        let peer = crate::identity::PeerIdentity::of(&stream);
        if peer.is_none() {
            // Not fatal: inspect verbs stay available. Every protected verb on
            // this connection will be refused as `identity_unverifiable`.
            tracing::warn!("could not read peer credentials; protected verbs refused");
        }
        let permit = match Arc::clone(&connection_permits).try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => {
                tracing::warn!("connection refused: server connection limit reached");
                drop(stream);
                continue;
            }
        };
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            let _permit = permit;
            if let Err(e) = handle_connection(stream, state, peer).await {
                tracing::warn!("connection error: {e}");
            }
        });
    }
}

async fn handle_connection(
    stream: UnixStream,
    state: Arc<ServerState>,
    peer: Option<crate::identity::PeerIdentity>,
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

    // `read_line` grows its buffer until it finds a newline, so a client that
    // opens the socket and never sends one would otherwise consume memory
    // without bound. The cap is applied to the read itself, so the oversized
    // bytes are never allocated in the first place.
    // ponytail: fixed 1 MiB per NDJSON request line; raise only if a real plan
    // payload is measured near it.
    const MAX_REQUEST_BYTES: u64 = 1024 * 1024;
    const REQUEST_LINE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
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
        let n = match tokio::time::timeout(
            REQUEST_LINE_TIMEOUT,
            (&mut reader).take(MAX_REQUEST_BYTES).read_line(&mut line),
        )
        .await
        {
            Ok(Ok(n)) => n,
            Ok(Err(error)) => break Err(Box::new(error) as _),
            Err(_) => {
                break Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "request line exceeded the 10s server timeout",
                )) as _)
            }
        };
        if n == 0 {
            break Ok(());
        }
        // Hitting the cap with no terminator means the line was oversized:
        // reject it and close rather than parsing a truncated request.
        if n as u64 == MAX_REQUEST_BYTES && !line.ends_with('\n') {
            let resp = serde_json::json!({
                "error": format!("request line exceeds {MAX_REQUEST_BYTES} bytes")
            });
            let _ = tx.send(format!("{resp}\n")).await;
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
                let response = dispatch_bounded(other, &state, line.trim(), peer).await;
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

/// The per-request bound from §11.1. `events.subscribe` is excluded: it is
/// handled on the connection loop above and is long-lived by design.
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

// A queued request crosses exactly one of these terminal transitions. The CAS
// is the linearization point that makes a timeout's no-effect claim true:
// either timeout cancels while waiting, or admission wins and the response is
// explicitly post-admission.
const ADMISSION_WAITING: u8 = 0;
const ADMISSION_GRANTED: u8 = 1;
const ADMISSION_CANCELLED: u8 = 2;

/// Run one request under [`REQUEST_TIMEOUT`].
///
/// The handler runs on its own task, so expiry stops *waiting* for the work —
/// it does not cancel it. That distinction is the whole point: a `case.submit`
/// that has already appended to a ledger must finish, or the durable record and
/// the client's view of it would permanently disagree. `record_pending` runs
/// before the work starts, so a timed-out request is already answerable through
/// `request.get_status` and is upgraded to its real outcome when the work lands.
///
/// Awaiting the handle here rather than detaching it preserves per-connection
/// response ordering: NDJSON replies carry no sequence number, so a client
/// pairs them with requests positionally.
/// A reserved path to the bounded request-work admission pool.
///
/// An immediate permit is already lawful admission. A queued permit only
/// reserves one of the finite waiting-room slots; it is not admission and is
/// dropped if the request deadline expires before an active permit is issued.
enum AdmissionReservation {
    Immediate(tokio::sync::OwnedSemaphorePermit),
    Queued(tokio::sync::OwnedSemaphorePermit),
}

/// Return the caller-supplied correlation id for request variants that support
/// one. Keeping this exhaustive beside [`Request`] prevents a new mutating
/// variant from silently escaping the durable-locator rule.
fn request_id(request: &Request) -> Option<&str> {
    match request {
        Request::Submit { request_id, .. }
        | Request::Approve { request_id, .. }
        | Request::Reject { request_id, .. }
        | Request::AgentProbe { request_id, .. }
        | Request::Delegate { request_id, .. }
        | Request::CancelDelegation { request_id, .. }
        | Request::CaseCommit { request_id, .. }
        | Request::ApprovalDecide { request_id, .. } => {
            request_id.as_deref().filter(|id| !id.is_empty())
        }
        _ => None,
    }
}

/// Mutations with a durable governance boundary must be caller-correlated.
///
/// We select the specification's fail-closed option rather than minting an
/// opaque id that a client cannot know before a timeout response. `ask` may
/// still be protected (and therefore admitted) but is not a durable mutation.
/// `agent.probe` is durable: it records a case, run, and authority decision
/// before contacting the configured endpoint.
fn requires_durable_locator(request: &Request) -> bool {
    matches!(
        request,
        Request::Submit { .. }
            | Request::Approve { .. }
            | Request::Reject { .. }
            | Request::AgentProbe { .. }
            | Request::Delegate { .. }
            | Request::CancelDelegation { .. }
            | Request::CaseCommit { .. }
            | Request::ApprovalDecide { .. }
    )
}

fn durable_locator_refusal() -> serde_json::Value {
    serde_json::json!({
        "error": "this durable mutation requires a non-empty request_id before admission",
        "error_class": "durable_locator_required",
        "no_side_effect": true,
    })
}

/// Run the portions of a protected request that are lawful only after request
/// admission. In particular, the first correlation write is here rather than
/// in the connection loop, because that write is itself durable state.
async fn dispatch_admitted(
    request: Request,
    state: Arc<ServerState>,
    raw_line: String,
    verified_actor: Option<crate::identity::ResolvedActor>,
) -> serde_json::Value {
    if let Some((request_id, verb, hash)) = dedupe_key(&request, &raw_line) {
        // Check and bind are one admission claim. A second same-id request
        // waits here until the first has durably bound its payload, then sees
        // `Pending` or a terminal replay instead of starting duplicate work.
        let _correlation_admission = state.correlation_admission.lock().await;
        match state.correlation.check(&request_id, &hash) {
            Ok(crate::sfwp::correlation::DedupeVerdict::Replay(outcome)) => {
                tracing::info!(
                    request_id,
                    "duplicate request replayed from correlation store"
                );
                return outcome;
            }
            Ok(crate::sfwp::correlation::DedupeVerdict::Pending) => {
                tracing::info!(request_id, "duplicate request remains pending");
                return serde_json::json!({
                    "request_id": request_id,
                    "status": "pending",
                    "recover_with": "request.get_status",
                });
            }
            Ok(crate::sfwp::correlation::DedupeVerdict::Reused) => {
                tracing::warn!(
                    request_id,
                    "request refused: id already bound to another payload"
                );
                return serde_json::json!({
                    "error": format!(
                        "request_id `{request_id}` was already used for a different operation \
                         or payload; issue a new id rather than re-using this one"
                    ),
                    "error_class": "request_id_reused",
                    "no_side_effect": true,
                });
            }
            Ok(crate::sfwp::correlation::DedupeVerdict::Proceed) => {
                // Bind only after lawful admission. A busy refusal or a timeout
                // while waiting therefore cannot leave a correlation artifact.
                if let Err(error) =
                    state
                        .correlation
                        .record_pending(&request_id, &verb, Some(&hash))
                {
                    tracing::warn!(
                        request_id,
                        "request refused: correlation unwritable: {error}"
                    );
                    return serde_json::json!({
                        "error": format!(
                            "the correlation record for `{request_id}` could not be written, so \
                             this request cannot be made idempotent: {error}"
                        ),
                        "error_class": "idempotency_unverifiable",
                        "no_side_effect": true,
                    });
                }
            }
            Err(error) => {
                tracing::warn!(
                    request_id,
                    "request refused: correlation unreadable: {error}"
                );
                return serde_json::json!({
                    "error": format!(
                        "the correlation record for `{request_id}` could not be read, so this \
                         request cannot be shown to be a first attempt: {error}"
                    ),
                    "error_class": "idempotency_unverifiable",
                    "no_side_effect": true,
                });
            }
        }
        // This lock protects only check-and-bind. Keeping it across the
        // handler would falsely make an already-admitted request wait without
        // a discoverable locator and would serialize unrelated durable work.
        drop(_correlation_admission);
    }

    handle_request_as(request, &state, verified_actor.as_ref()).await
}

/// Run one request under [`REQUEST_TIMEOUT`], admitting side-effecting work
/// through a finite active pool and a finite waiting room. Read-only requests
/// retain the normal bounded wrapper and never consume admission capacity.
async fn dispatch_bounded(
    request: Request,
    state: &Arc<ServerState>,
    raw_line: &str,
    peer: Option<crate::identity::PeerIdentity>,
) -> serde_json::Value {
    // SF-005 / U-07. The identity gate remains before admission: a refused
    // request never enters the waiting room and cannot have an effect to undo.
    if matches!(request, Request::IdentityGet) {
        let view = state.config().identity.describe(peer);
        return serde_json::to_value(view)
            .unwrap_or_else(|_| serde_json::json!({"error": "identity serialization failed"}));
    }

    let mut verified_actor: Option<crate::identity::ResolvedActor> = None;
    if crate::identity::is_protected(&request) {
        let claim = crate::identity::ActorClaim::parse(raw_line);
        match state.config().identity.resolve(claim.as_ref(), peer) {
            Ok(actor) => {
                tracing::debug!(
                    actor_id = actor.actor_id(),
                    uid = actor.uid(),
                    "protected request attributed"
                );
                if let Some(entity) = request_entity(raw_line) {
                    if entity != actor.actor_id() {
                        let refusal = crate::identity::IdentityRefusal::EntityMismatch {
                            actor_id: actor.actor_id().into(),
                            entity,
                        };
                        tracing::warn!("protected request refused: {}", refusal.message());
                        return refusal.response();
                    }
                }
                if let Some((case_id, approval_id)) = approval_target(&request) {
                    match crate::identity::approval_submitter(&state.root, case_id, approval_id) {
                        Ok(Some(submitter)) if submitter == actor.actor_id() => {
                            let refusal = crate::identity::IdentityRefusal::SelfApproval {
                                actor_id: actor.actor_id().into(),
                                approval_id: approval_id.into(),
                            };
                            tracing::warn!(
                                actor_id = actor.actor_id(),
                                approval_id,
                                "approval refused: {}",
                                refusal.message()
                            );
                            return refusal.response();
                        }
                        Ok(_) => {}
                        Err(error) => {
                            tracing::warn!(
                                approval_id,
                                "approval refused: submitter unreadable: {error}"
                            );
                            return serde_json::json!({
                                "error": format!(
                                    "the submitter behind approval `{approval_id}` could not be \
                                     read, so separation of duty cannot be verified: {error}"
                                ),
                                "error_class": "separation_of_duty_unverifiable",
                                "no_side_effect": true,
                            });
                        }
                    }
                }
                verified_actor = Some(actor);
            }
            Err(refusal) => {
                tracing::warn!(
                    error_class = refusal.error_class(),
                    "protected request refused: {}",
                    refusal.message()
                );
                return refusal.response();
            }
        }

        if requires_durable_locator(&request) && request_id(&request).is_none() {
            tracing::warn!("protected durable mutation refused: request_id absent");
            return durable_locator_refusal();
        }

        let reservation = match Arc::clone(&state.request_admission).try_acquire_owned() {
            Ok(permit) => AdmissionReservation::Immediate(permit),
            Err(_) => match Arc::clone(&state.request_admission_waiters).try_acquire_owned() {
                Ok(ticket) => AdmissionReservation::Queued(ticket),
                Err(_) => {
                    tracing::warn!(
                        request_id = request_id(&request).unwrap_or("<none>"),
                        "protected request refused: admission queue full"
                    );
                    return serde_json::json!({
                        "error": "server request-work admission is full; retry after an admitted request settles",
                        "error_class": "server_busy",
                        "no_side_effect": true,
                    });
                }
            },
        };

        let admission = Arc::new(AtomicU8::new(
            if matches!(&reservation, AdmissionReservation::Immediate(_)) {
                ADMISSION_GRANTED
            } else {
                ADMISSION_WAITING
            },
        ));
        let request_id = request_id(&request).map(str::to_owned);
        let request_id_for_work = request_id.clone();
        let state = Arc::clone(state);
        let raw_line = raw_line.to_owned();
        let admission_for_work = Arc::clone(&admission);
        let handle = tokio::spawn(async move {
            let permit = match reservation {
                AdmissionReservation::Immediate(permit) => permit,
                AdmissionReservation::Queued(ticket) => {
                    let permit = Arc::clone(&state.request_admission)
                        .acquire_owned()
                        .await
                        .expect("request admission semaphore is never closed");
                    // The timeout and this task race only on this transition.
                    // If cancellation wins, drop both permits and return before
                    // the correlation write or any governed handler can run.
                    if admission_for_work
                        .compare_exchange(
                            ADMISSION_WAITING,
                            ADMISSION_GRANTED,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_err()
                    {
                        drop(ticket);
                        drop(permit);
                        return serde_json::json!({
                            "error": "request cancelled before admission",
                            "error_class": "request_cancelled",
                            "no_side_effect": true,
                        });
                    }
                    drop(ticket);
                    permit
                }
            };
            tracing::debug!(
                request_id = request_id_for_work.as_deref().unwrap_or("<none>"),
                "request admitted"
            );
            let response = dispatch_admitted(request, state, raw_line, verified_actor).await;
            drop(permit);
            response
        });
        return bounded_admitted(handle, REQUEST_TIMEOUT, request_id, admission).await;
    }

    let state = Arc::clone(state);
    bounded(
        async move { handle_request_as(request, &state, verified_actor.as_ref()).await },
        REQUEST_TIMEOUT,
        raw_line,
    )
    .await
}

/// The `(request_id, payload_hash)` a protected request dedupes on, or `None`
/// when this request is not eligible (unprotected, or carries no id).
///
/// Read off the raw line rather than matched per variant, for the reason
/// `request_entity` gives: `request_id` is optional on many protected verbs and
/// a per-variant match would silently skip any verb added later. Eligibility
/// reuses `is_protected` — the one exhaustive match — so the set of deduped
/// verbs cannot drift from the set of governed ones.
fn dedupe_key(request: &Request, raw_line: &str) -> Option<(String, String, String)> {
    if !crate::identity::is_protected(request) {
        return None;
    }
    let mut value: serde_json::Value = serde_json::from_str(raw_line).ok()?;
    let object = value.as_object_mut()?;
    // Removed before hashing: the id is the key, not part of what it keys.
    // Everything else stays in, `verb` included — so a reused id pointed at a
    // different operation is caught by the payload comparison alone.
    let request_id = object.remove("request_id")?.as_str()?.to_owned();
    if request_id.is_empty() {
        return None;
    }
    let verb = object
        .get("verb")
        .and_then(|verb| verb.as_str())
        .unwrap_or("unknown")
        .to_owned();
    let hash = crate::sfwp::correlation::payload_hash(&value);
    Some((request_id, verb, hash))
}

/// The `entity` a request attributes its work to, read off the raw line.
///
/// Read from the line rather than matched per variant for the same reason
/// `ActorClaim` is: `entity` appears on many protected verbs and is absent from
/// others, and a per-variant match would silently skip any verb added later.
fn request_entity(raw_line: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(raw_line)
        .ok()?
        .get("entity")
        .and_then(|entity| entity.as_str())
        .map(str::to_owned)
}

/// The `(case_id, approval_id)` an approval-resolving verb targets, if this
/// request is one.
///
/// Exhaustive over the three spellings that reach `decide`. A fourth would have
/// to be added here to be covered, which is why this returns a target rather
/// than a bool: the caller needs the ids anyway, so there is no version of this
/// that compiles while silently skipping the check.
fn approval_target(request: &Request) -> Option<(&str, &str)> {
    match request {
        Request::Approve {
            case_id,
            approval_id,
            ..
        }
        | Request::Reject {
            case_id,
            approval_id,
            ..
        }
        | Request::ApprovalDecide {
            case_id,
            approval_id,
            ..
        } => Some((case_id, approval_id)),
        _ => None,
    }
}

/// The bound itself, over any request future. Separated from
/// [`dispatch_bounded`] so the no-cancellation property can be asserted
/// directly: the whole guarantee lives in `spawn`-then-time-out-the-*handle*,
/// and collapsing it to `timeout(d, work)` would still compile and still pass
/// any test that only checked the response shape.
/// Await a protected request after it has either acquired admission or entered
/// the finite waiting room. A timeout before admission aborts the task; a
/// timeout after admission leaves its durable lifecycle running and observable.
async fn bounded_admitted(
    mut handle: tokio::task::JoinHandle<serde_json::Value>,
    limit: std::time::Duration,
    request_id: Option<String>,
    admission: Arc<AtomicU8>,
) -> serde_json::Value {
    match tokio::time::timeout(limit, &mut handle).await {
        Ok(Ok(response)) => response,
        Ok(Err(join_error)) => serde_json::json!({
            "error": format!("request handler failed: {join_error}"),
            "error_class": "internal_error",
        }),
        Err(_) => match admission.compare_exchange(
            ADMISSION_WAITING,
            ADMISSION_CANCELLED,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => {
                // Wait for the aborted task to observe cancellation before
                // making the no-effect claim. If it won the semaphore race it
                // sees ADMISSION_CANCELLED and releases the permit without
                // crossing `dispatch_admitted`.
                handle.abort();
                let _ = handle.await;
                tracing::warn!(
                    request_id = request_id.as_deref().unwrap_or("<none>"),
                    "request deadline reached before admission; work cancelled"
                );
                serde_json::json!({
                    "error": format!(
                        "request exceeded the {}s server timeout before admission; the work was cancelled",
                        limit.as_secs()
                    ),
                    "error_class": "request_timeout",
                    "timeout_seconds": limit.as_secs(),
                    "request_id": request_id,
                    "no_side_effect": true,
                })
            }
            Err(ADMISSION_GRANTED) => {
                tracing::warn!(
                    request_id = request_id.as_deref().unwrap_or("<none>"),
                    "request deadline reached after admission; durable work continues"
                );
                serde_json::json!({
                    "error": format!(
                        "request exceeded the {}s server timeout after admission; the work continues",
                        limit.as_secs()
                    ),
                    "error_class": "request_timeout",
                    "timeout_seconds": limit.as_secs(),
                    "request_id": request_id,
                    "recover_with": "request.get_status",
                })
            }
            Err(ADMISSION_CANCELLED) => {
                // Only this wrapper writes CANCELLED, so reaching it means a
                // future refactor attempted a second timeout settlement.
                serde_json::json!({
                    "error": "request admission was cancelled twice",
                    "error_class": "internal_error",
                })
            }
            Err(_) => unreachable!("admission state is one of the declared constants"),
        },
    }
}

async fn bounded<F>(work: F, limit: std::time::Duration, raw_line: &str) -> serde_json::Value
where
    F: std::future::Future<Output = serde_json::Value> + Send + 'static,
{
    let handle = tokio::spawn(work);

    match tokio::time::timeout(limit, handle).await {
        Ok(Ok(response)) => response,
        // A panicking handler must not take the connection down with it; the
        // client gets a typed failure and the loop keeps serving.
        Ok(Err(join_error)) => serde_json::json!({
            "error": format!("request handler failed: {join_error}"),
            "error_class": "internal_error",
        }),
        Err(_) => {
            // ponytail: re-parse for the id only on the timeout path — it costs
            // one JSON parse per 10-second stall and saves matching every
            // `request_id`-bearing variant by hand, which would silently miss
            // any variant added later.
            let request_id = serde_json::from_str::<serde_json::Value>(raw_line)
                .ok()
                .and_then(|value| {
                    value
                        .get("request_id")
                        .and_then(|id| id.as_str())
                        .map(str::to_owned)
                });
            tracing::warn!(
                request_id = request_id.as_deref().unwrap_or("<none>"),
                "request exceeded {}s; work continues",
                limit.as_secs()
            );
            serde_json::json!({
                "error": format!(
                    "request exceeded the {}s server timeout; the work was not cancelled",
                    limit.as_secs()
                ),
                "error_class": "request_timeout",
                "timeout_seconds": limit.as_secs(),
                "request_id": request_id,
                "recover_with": "request.get_status",
            })
        }
    }
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
    handle_request_as(request, state, None).await
}

/// [`handle_request`], carrying the actor the identity gate verified.
///
/// Separate entry point rather than a changed signature because the actor is
/// only knowable at the connection level (`SO_PEERCRED`), and every in-process
/// caller — the CLI-facing paths and the test suites — legitimately has none.
/// Those callers get `None` and the behaviour they always had; a socket request
/// carries the verified actor — id *and* role — through to the records it
/// produces.
pub async fn handle_request_as(
    request: Request,
    state: &Arc<ServerState>,
    verified: Option<&crate::identity::ResolvedActor>,
) -> serde_json::Value {
    // F-08: the gate verified this principal *and* its role on the socket
    // peer, so every authority evaluation below must see both. A request that
    // never crossed the socket (`handle_request`) legitimately has no verified
    // identity and keeps the historical local-operator shape — the fallback is
    // spelled once here rather than re-assumed at each evaluation site.
    let verified_role = verified
        .map(|actor| actor.role().clone())
        .unwrap_or_else(|| {
            tracing::debug!(
                "no verified socket identity for this request; \
             authority evaluations use the local-default operator role"
            );
            ActorRole::Operator
        });
    let actor_id = verified.map(|actor| actor.actor_id());
    match request {
        Request::Submit {
            payload,
            request_id,
        } => {
            record_pending(state, request_id.as_deref(), "case.submit");
            let response = commit_plan(state, payload, verified_role).await;
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
                actor_id,
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
                actor_id,
            )
            .await
        }
        Request::AgentList => agent_probe::list(&state.config().agent),
        Request::AgentProbe {
            endpoint,
            prompt,
            model,
            policy,
            entity,
            process,
            request_id,
        } => {
            record_pending(state, request_id.as_deref(), "agent.probe");
            let response = match state.semaphore.acquire().await {
                Ok(permit) => {
                    let config = state.config();
                    let result = agent_probe::probe(
                        &config,
                        agent_probe::ProbeRequest {
                            endpoint_id: &endpoint,
                            prompt: &prompt,
                            model: model.as_deref(),
                            policy_path: &policy,
                            entity: &entity,
                            process: &process,
                            actor_role: verified_role.clone(),
                        },
                        &agent_probe::EnvironmentCredentialResolver,
                    )
                    .await;
                    drop(permit);
                    match result {
                        Ok(outcome) => serde_json::to_value(outcome).unwrap_or_else(
                            |_| serde_json::json!({"error":"probe response serialization failed"}),
                        ),
                        Err(error) => serde_json::json!({
                            "error": error.to_string(),
                            "error_class": error.class(),
                        }),
                    }
                }
                Err(_) => serde_json::json!({
                    "error": "server semaphore unavailable",
                    "error_class": "server_unavailable",
                }),
            };
            record_outcome(state, request_id.as_deref(), "agent.probe", &response);
            response
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
                verified_role.clone(),
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
            let response =
                cancel_delegation(state, &run_id, &policy, &entity, &process, verified_role).await;
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
            actor_id: requested_actor,
        } => {
            let Some(question_kind) = sea_forge_thoth::protocol::parse_question_kind(&kind) else {
                return serde_json::json!({"error": format!("unknown question kind: {kind}")});
            };
            // Disclosure is scoped to the asker, so the asker must be the
            // identity the gate verified — not one the request nominated.
            let asker = actor_id.map(str::to_owned).unwrap_or(requested_actor);
            let root = state.root.clone();
            let result = tokio::task::spawn_blocking(move || {
                sea_forge_thoth::service::ask(
                    &root,
                    &asker,
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
                // Projected through the SFWP view rather than serialized
                // straight out of the kernel type, so the contract the
                // Workbench validates against is generated from something this
                // layer owns (ADR-005, GEN-01).
                Ok(answer) => serde_json::to_value(sfwp::thoth::ThothAnswerView::from(answer))
                    .unwrap_or_else(
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
                &state.config(),
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
        // Also connection-scoped: the answer is derived from the peer's uid,
        // which `dispatch_bounded` holds and this level does not. Answer with
        // the honest shape — no peer, therefore nothing claimable — rather than
        // inventing a uid or panicking.
        Request::IdentityGet => serde_json::to_value(state.config().identity.describe(None))
            .unwrap_or_else(|_| serde_json::json!({"error": "identity serialization failed"})),
        // Inspect projections (Task 6): shaped views, never propagated errors.
        Request::CaseEntryOptions => serde_json::to_value(sfwp::case::entry_options(&state.root))
            .unwrap_or_else(|_| serde_json::json!({"error": "entry_options serialization failed"})),
        Request::CasePreflight { params } => {
            serde_json::to_value(sfwp::case::preflight(&state.root, params))
                .unwrap_or_else(|_| serde_json::json!({"error": "preflight serialization failed"}))
        }
        Request::CaseList => serde_json::to_value(sfwp::case_views::list(&state.root))
            .unwrap_or_else(|_| serde_json::json!({"error": "case list serialization failed"})),
        Request::CaseGetOverview { case_id } => {
            case_view_response(sfwp::case_views::get_overview(&state.root, &case_id))
        }
        Request::CaseGetHorizon { case_id } => {
            case_view_response(sfwp::case_views::get_horizon(&state.root, &case_id))
        }
        Request::RunList { case_id } => {
            serde_json::to_value(sfwp::run_views::list(&state.root, case_id.as_deref()))
                .unwrap_or_else(|_| serde_json::json!({"error": "run list serialization failed"}))
        }
        Request::RunGet { run_id } => match sfwp::run_views::get(&state.root, &run_id) {
            Ok(view) => serde_json::to_value(view)
                .unwrap_or_else(|_| serde_json::json!({"error": "run view serialization failed"})),
            Err(error) => serde_json::json!({
                "error": error.message(),
                "error_class": error.class(),
            }),
        },
        Request::AssetList => {
            serde_json::to_value(sfwp::assets::list(&state.root, &state.config().agent))
                .unwrap_or_else(|_| serde_json::json!({"error": "asset list serialization failed"}))
        }
        Request::DelegationList => {
            // Snapshot the live handles under one lock, then project outside it:
            // the projection also touches the filesystem, and holding the
            // delegation map across that would put disk latency on the path of
            // every cancellation.
            let live = state
                .delegations
                .lock()
                .await
                .iter()
                .map(|(run_id, handle)| {
                    (
                        run_id.clone(),
                        sfwp::delegations::LiveDelegation {
                            case_id: handle.case_id.clone(),
                            cancellation_requested: handle.requested.load(Ordering::SeqCst),
                        },
                    )
                })
                .collect();
            serde_json::to_value(sfwp::delegations::list(&state.root, &live)).unwrap_or_else(
                |_| serde_json::json!({"error": "delegation list serialization failed"}),
            )
        }
        Request::DelegationPreview { params } => serde_json::to_value(
            sfwp::delegation_preview::preview(&state.root, &state.config().agent, params),
        )
        .unwrap_or_else(
            |_| serde_json::json!({"error": "delegation preview serialization failed"}),
        ),
        Request::ApprovalList { case_id } => {
            serde_json::to_value(sfwp::approvals::list(&state.root, case_id.as_deref()))
                .unwrap_or_else(
                    |_| serde_json::json!({"error": "approval list serialization failed"}),
                )
        }
        Request::ApprovalDecide {
            case_id,
            approval_id,
            decision,
            note,
            request_id,
            preconditions,
        } => {
            // Route to the same `decide` helper `Approve`/`Reject` use. An
            // unknown verdict is refused outright: defaulting it would resolve
            // a governed decision on the server's guess.
            let (cli_verb, method, event_kind) = match decision.as_str() {
                "approve" => ("approve", "approval.approve", "approval.approved"),
                "reject" => ("reject", "approval.reject", "approval.rejected"),
                other => {
                    return serde_json::json!({
                        "error": format!("unknown approval decision {other:?}; expected \"approve\" or \"reject\""),
                        "error_class": "invalid_decision",
                    })
                }
            };
            decide(
                state,
                cli_verb,
                method,
                event_kind,
                &case_id,
                &approval_id,
                note.as_deref(),
                request_id.as_deref(),
                preconditions.as_ref(),
                actor_id,
            )
            .await
        }
        Request::CaseCommit {
            template_ref,
            params,
            policy,
            entity,
            process,
            timeout,
            request_id,
            preconditions,
        } => {
            record_pending(state, request_id.as_deref(), "case.commit");

            // Evaluate the precondition first — before any side effect. Mirrors
            // `decide`'s "no side effect on stale" discipline for approve/reject.
            if let Some(precondition) = &preconditions {
                let resolver = TemplateRecordResolver { root: &state.root };
                match sfwp::precondition::evaluate(precondition, &resolver) {
                    Ok(Some(rejected)) => {
                        let response = serde_json::to_value(&rejected).unwrap_or_else(|_| {
                            serde_json::json!({"error": "rejected_as_stale serialization failed"})
                        });
                        record_outcome(state, request_id.as_deref(), "case.commit", &response);
                        return response;
                    }
                    Ok(None) => {}
                    Err(error) => {
                        let response = serde_json::json!({
                            "error": error.to_string(),
                            "error_class": error.class(),
                        });
                        record_outcome(state, request_id.as_deref(), "case.commit", &response);
                        return response;
                    }
                }
            }

            let response = match instantiate_plan_file(state, &template_ref, &params).await {
                Ok(plan_path) => {
                    let payload = SubmitPayload {
                        intent: None,
                        plan: Some(plan_path),
                        policy,
                        entity,
                        process,
                        timeout,
                    };
                    commit_plan(state, payload, verified_role).await
                }
                Err(error) => serde_json::json!({
                    "error": error.to_string(),
                    "error_class": error.class(),
                }),
            };
            record_outcome(state, request_id.as_deref(), "case.commit", &response);
            response
        }
    }
}

/// Shared commit path for `Submit` and `CaseCommit`: reload config
/// defensively (§8.4), dispatch through `case_dispatch::submit` (the single
/// governance path that mints the case, validates the plan, and drives the
/// dispatch loop), then record the case entry, fire the notify hook, and
/// publish the durable `case.submitted` event on success.
async fn commit_plan(
    state: &Arc<ServerState>,
    payload: SubmitPayload,
    actor_role: ActorRole,
) -> serde_json::Value {
    // The swap has already happened by the time this returns, so the dispatch
    // below runs on the reloaded snapshot. An invalid file keeps the previous
    // one and is reported to the operator — a silent log line would let a cell
    // run for days on a configuration its `server.yaml` no longer describes.
    if let Err(message) = state.reload_config() {
        tracing::warn!("invalid config reload — keeping last-known-good: {message}");
        let _ = state
            .publish_event(
                "invalid_reload_error",
                None,
                None,
                serde_json::json!({
                    "config_path": state.root.join("server.yaml").display().to_string(),
                    "message": message,
                }),
            )
            .await;
    }

    match case_dispatch::submit(payload, state, actor_role).await {
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
            if let Some(argv) = &state.config().notify_command {
                if !argv.is_empty() {
                    let event = serde_json::json!({
                        "event": "run_finished",
                        "case_id": case_id,
                    });
                    let argv = argv.clone();
                    // F-07: run the (bounded) notify hook on a blocking thread
                    // rather than the async worker, so a slow hook cannot park
                    // the runtime.
                    let _ = tokio::task::spawn_blocking(move || fire_notify(&argv, event)).await;
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
    }
}

/// Instantiate `template_ref` + `params` into a `CasePlan` (identical to what
/// `case.preflight` built) and write it to a scratch file under
/// `<root>/drafts/`, so `case.commit` can delegate to the existing
/// `case_dispatch::submit(SubmitPayload{plan: Some(path), ..})` path unchanged
/// — additive, no forked commit logic, no duplicate case-minting path.
async fn instantiate_plan_file(
    state: &Arc<ServerState>,
    template_ref: &str,
    params: &std::collections::BTreeMap<String, String>,
) -> Result<String, ForgeError> {
    let root = state.root.clone();
    let template_ref = template_ref.to_string();
    let params = params.clone();
    tokio::task::spawn_blocking(move || {
        let (name, version) = sea_forge_planner::templates::parse_template_ref(&template_ref)?;
        let path = root
            .join("templates")
            .join(format!("{name}@{version}.yaml"));
        let template = sea_forge_planner::templates::load(&path)?;
        let plan = sea_forge_planner::templates::instantiate(
            &template, &params, "pending", "pending", "pending",
        )?;
        let drafts_dir = root.join("drafts");
        std::fs::create_dir_all(&drafts_dir)
            .map_err(|e| ForgeError::io("create drafts directory", e))?;
        let scratch = drafts_dir.join(format!("{}.json", random_id("draft")?));
        std::fs::write(
            &scratch,
            serde_json::to_vec(&plan).map_err(|e| ForgeError::Serialization(e.to_string()))?,
        )
        .map_err(|e| ForgeError::io("write draft plan", e))?;
        // F-16: `SubmitPayload.plan` resolves strictly workspace-relative
        // under the state root, so the scratch draft is returned as its
        // cell-relative spelling rather than an absolute path.
        Ok(format!(
            "drafts/{}",
            scratch.file_name().unwrap_or_default().to_string_lossy()
        ))
    })
    .await
    .map_err(|e| ForgeError::Internal(format!("instantiate_plan_file task panic: {e}")))
    .and_then(|result| result)
}

/// Resolves a `template:<name>@<version>` precondition ref to its current
/// on-disk digest, so a `case.commit` whose template changed since
/// `case.preflight` computed the expected digest stale-rejects cleanly.
struct TemplateRecordResolver<'a> {
    root: &'a Path,
}

impl sfwp::precondition::RecordResolver for TemplateRecordResolver<'_> {
    fn resolve(&self, r#ref: &str) -> Result<Option<serde_json::Value>, ForgeError> {
        let Some(template_ref) = r#ref.strip_prefix("template:") else {
            return Ok(None);
        };
        Ok(sfwp::case::resolve_template_digest_source(
            self.root,
            template_ref,
        ))
    }
}

/// Shape a case-view result into a wire response.
///
/// A failed lookup carries a machine-readable `error_class` (`not_found` vs
/// `record_unreadable`) because those need different operator responses: the
/// first means "you asked for something that isn't here", the second means
/// "something that should be readable isn't" — an integrity signal. Collapsing
/// both into a generic error would hide the second inside the first.
fn case_view_response<T: serde::Serialize>(
    result: Result<T, sfwp::case_views::CaseViewError>,
) -> serde_json::Value {
    match result {
        Ok(view) => serde_json::to_value(view)
            .unwrap_or_else(|_| serde_json::json!({"error": "case view serialization failed"})),
        Err(error) => serde_json::json!({
            "error": error.message(),
            "error_class": error.class(),
        }),
    }
}

/// Record a correlated request as `pending` before its work begins. No-op if
/// the client did not supply a `request_id`. Correlation-store failures are
/// logged, never fatal to the request.
///
/// Passes no payload hash: the dispatch guard upstream has already bound the
/// id to one, and the store carries it forward rather than letting this second
/// write erase it.
fn record_pending(state: &Arc<ServerState>, request_id: Option<&str>, method: &str) {
    if let Some(id) = request_id {
        if let Err(error) = state.correlation.record_pending(id, method, None) {
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
    /// F-24: the case ledger is opened and read once per resolver — one scan
    /// covers a whole precondition bundle instead of one scan per record.
    entries: std::cell::RefCell<Option<Vec<sea_forge_ledger::types::LedgerEntry>>>,
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
        if self.entries.borrow().is_none() {
            let ledger = LedgerStream::open(
                self.root,
                format!("case-{}", self.case_id),
                "sea-forge-server",
            )?;
            *self.entries.borrow_mut() = Some(ledger.read_entries()?);
        }
        let entries_guard = self.entries.borrow();
        let entries: &[sea_forge_ledger::types::LedgerEntry] =
            entries_guard.as_deref().unwrap_or_default();
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
    // The actor the identity gate verified, when this arrived over a socket.
    actor_id: Option<&str>,
) -> serde_json::Value {
    record_pending(state, request_id, method);

    // Evaluate preconditions first — before any side effect.
    if let Some(precondition) = preconditions {
        let resolver = LedgerRecordResolver {
            root: &state.root,
            case_id,
            entries: std::cell::RefCell::new(None),
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

    let root = state.root.clone();
    let mut args = vec![
        cli_verb,
        case_id,
        approval_id,
        "--root",
        root.to_str().unwrap_or("."),
    ];
    // Attribute the resolution to the actor the gate verified. Without this the
    // CLI falls back to its own default (`operator_local`), so every approval
    // resolved over SFWP was recorded as `resolved_by=operator_local` no matter
    // who decided it — an audit trail naming the wrong person, and a
    // separation-of-duty check downstream comparing against the wrong actor.
    if let Some(actor_id) = actor_id {
        args.extend_from_slice(&["--actor", actor_id]);
    }
    let result = run_cli(&root, &args, note).await;
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
    actor_role: ActorRole,
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
    let config = state.config();
    let run_for_execution = delegation_run_id.clone();
    let case_for_execution = delegation_case_id.clone();
    let cancel_for_execution = Arc::clone(&cancel);
    // Resolve retention through the full precedence chain (endpoint → cell
    // `[agent]` → built-in), exactly as `case_dispatch` does for a planned
    // agent task. `DelegationRequest::transcript_retention` is documented as
    // "already resolved by the caller", and taking `Default::default()` here
    // silently pinned every socket-issued delegation to `summarized` — ignoring
    // an endpoint that had explicitly asked for `full` transcripts. There is no
    // item override on this path: `delegate` has no field for one.
    let resolved_retention = match sea_forge_agent::TranscriptRetentionMode::resolve(
        None,
        config.agent.endpoint(endpoint),
        &config.agent,
    ) {
        Ok(mode) => mode,
        Err(message) => {
            state.delegations.lock().await.remove(&delegation_run_id);
            drop(permit);
            return serde_json::json!({"error": message});
        }
    };
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
            // F-08: the delegation is evaluated for the verified role.
            actor_role,
            transcript_retention: resolved_retention,
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
    actor_role: ActorRole,
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

    // F-24: the cancellation decision loads the policy bundle and commits to
    // the case ledger — blocking work belongs on a blocking thread.
    let config_snapshot = state.config();
    let handle_case = handle.case_id.clone();
    let run = run_id.to_string();
    let pol = policy_path.to_string();
    let ent = entity.to_string();
    let proc_ = process.to_string();
    let result = match tokio::task::spawn_blocking(move || {
        record_cancellation(
            &config_snapshot,
            &run,
            &handle_case,
            &pol,
            &ent,
            &proc_,
            actor_role,
        )
    })
    .await
    {
        Ok(result) => result,
        Err(error) => Err(ForgeError::Internal(format!(
            "cancellation task panic: {error}"
        ))),
    };
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
    actor_role: ActorRole,
) -> Result<String, sea_forge_core::ForgeError> {
    let policy = agent_probe::resolve_policy_path(&config.root, policy_path)?;
    let bundle = AuthorityPolicyBundle::load(&policy)?;
    let engine = PolicyAuthorityEngine::new(bundle.clone())?;
    let actor = Actor {
        actor_id: entity.into(),
        // F-08: the cancellation decision is evaluated for the verified role.
        role: actor_role,
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

/// Wall-clock bound for the operator-configured `notify_command` hook.
const NOTIFY_TIMEOUT_SECS: u64 = 30;

fn fire_notify(argv: &[String], event: serde_json::Value) -> Result<(), String> {
    if argv.is_empty() {
        return Ok(());
    }
    let mut cmd = std::process::Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    // F-07: the notify hook's stdout/stderr are unused, so point them at
    // /dev/null rather than a pipe. A chatty hook that fills a 64 KiB pipe
    // would otherwise deadlock the parent in `wait()` while the child blocks
    // on write.
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = cmd.spawn().map_err(|e| format!("spawn notify: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = serde_json::to_writer(&mut stdin, &event);
        let _ = stdin.write_all(b"\n");
    }
    // F-07: bound the wait so a hung hook cannot park the worker forever.
    // `wait_timeout` kills (SIGKILL) and reaps the child on timeout.
    use wait_timeout::ChildExt;
    match child
        .wait_timeout(std::time::Duration::from_secs(NOTIFY_TIMEOUT_SECS))
        .map_err(|e| format!("wait notify: {e}"))?
    {
        Some(status) if status.success() => Ok(()),
        Some(_) => Err("notify hook exited nonzero".into()),
        None => Err("notify hook timed out".into()),
    }
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

        let response = cancel_delegation(
            &state,
            &run,
            "policy.yaml",
            "operator_local",
            "test",
            ActorRole::Operator,
        )
        .await;

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

#[cfg(test)]
mod socket_contract_tests {
    use super::*;

    #[test]
    fn a_bindable_socket_path_is_accepted() {
        assert!(check_socket_path_length(Path::new("/tmp/cell/server.sock")).is_ok());
    }

    #[test]
    fn an_overlong_socket_path_is_rejected_with_the_fix_named() {
        let long = PathBuf::from("/tmp")
            .join("x".repeat(120))
            .join("server.sock");
        let error = check_socket_path_length(&long).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("SEA_FORGE_ROOT"), "{message}");
        assert!(message.contains("SEA_FORGE_SOCKET"), "{message}");
    }

    #[test]
    fn the_budget_leaves_room_for_the_binding_suffix() {
        // A path that fits sun_path exactly but not once `.binding` is appended
        // must still be rejected — otherwise bind() fails on the staging path.
        let budget = 104 - 1 - LONGEST_SOCKET_SUFFIX;

        // `/` + (budget - 1) characters == exactly `budget` bytes.
        let at_budget = PathBuf::from("/").join("y".repeat(budget - 1));
        assert_eq!(at_budget.as_os_str().as_encoded_bytes().len(), budget);
        assert!(check_socket_path_length(&at_budget).is_ok());

        let over = PathBuf::from("/").join("y".repeat(budget));
        assert_eq!(over.as_os_str().as_encoded_bytes().len(), budget + 1);
        assert!(check_socket_path_length(&over).is_err());
    }
}

#[cfg(test)]
mod lifecycle_contract_tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::time::Duration;

    /// The load-bearing half of the §11.1 timeout: expiry stops *waiting*, it
    /// does not stop the work. A `case.submit` that has already begun appending
    /// to a ledger must finish that append, or the durable record and the
    /// client's view of it diverge permanently.
    ///
    /// Rewriting `bounded` as `timeout(limit, work)` passes every assertion
    /// about the response shape and fails this one.
    #[tokio::test]
    async fn a_timed_out_request_keeps_running() {
        static FINISHED: AtomicBool = AtomicBool::new(false);

        let response = bounded(
            async {
                tokio::time::sleep(Duration::from_millis(300)).await;
                FINISHED.store(true, Ordering::SeqCst);
                serde_json::json!({"ok": true})
            },
            Duration::from_millis(50),
            r#"{"verb":"case_submit","request_id":"req_abc"}"#,
        )
        .await;

        assert_eq!(response["error_class"], "request_timeout");
        assert!(
            !FINISHED.load(Ordering::SeqCst),
            "the bound returned only after the work finished — it did not bound anything"
        );

        tokio::time::sleep(Duration::from_millis(500)).await;
        assert!(
            FINISHED.load(Ordering::SeqCst),
            "the work was cancelled at the timeout; a half-written durable record would be \
             unrecoverable"
        );
    }

    /// The timeout response has to name the request the client should poll, or
    /// `recover_with: request.get_status` is advice the client cannot act on.
    #[tokio::test]
    async fn a_timeout_names_the_request_to_recover_with() {
        let response = bounded(
            std::future::pending::<serde_json::Value>(),
            Duration::from_millis(20),
            r#"{"verb":"case_submit","request_id":"req_recover_me"}"#,
        )
        .await;

        assert_eq!(response["request_id"], "req_recover_me");
        assert_eq!(response["recover_with"], "request.get_status");
        assert_eq!(response["timeout_seconds"], 0);
    }

    /// A verb that carries no `request_id` still gets a typed timeout rather
    /// than a dropped connection.
    #[tokio::test]
    async fn a_timeout_without_a_request_id_is_still_typed() {
        let response = bounded(
            std::future::pending::<serde_json::Value>(),
            Duration::from_millis(20),
            r#"{"verb":"agent_probe"}"#,
        )
        .await;

        assert_eq!(response["error_class"], "request_timeout");
        assert!(response["request_id"].is_null());
    }

    /// A panicking handler is a bug in one request, not a reason to drop a
    /// connection that may be carrying an event subscription.
    #[tokio::test]
    async fn a_panicking_handler_is_reported_not_propagated() {
        let response = bounded(
            async { panic!("handler bug") },
            Duration::from_secs(5),
            r#"{"verb":"case_submit"}"#,
        )
        .await;

        assert_eq!(response["error_class"], "internal_error");
    }

    #[test]
    fn an_unset_notify_command_preflights_clean() {
        assert!(check_notify_command(None).is_ok());
        assert!(check_notify_command(Some(&[])).is_ok());
    }

    #[test]
    fn a_notify_command_on_path_preflights_clean() {
        // `sh` is mandated by POSIX and is on PATH wherever this test runs.
        let argv = ["sh".to_string(), "-c".to_string(), "true".to_string()];
        assert!(check_notify_command(Some(&argv)).is_ok());
    }

    #[test]
    fn a_missing_notify_command_blocks_startup_with_the_fix_named() {
        let argv = ["/nonexistent/notify-hook".to_string()];
        let error = check_notify_command(Some(&argv)).unwrap_err().to_string();
        assert!(error.contains("/nonexistent/notify-hook"), "{error}");
        assert!(error.contains("server.yaml"), "{error}");
    }

    /// A bare name is resolved against PATH at spawn time. Preflighting it as a
    /// relative path would pass or fail on the server's working directory,
    /// which is not where the hook will be looked up.
    #[test]
    fn a_bare_name_is_preflighted_against_path_not_the_cwd() {
        let argv = ["definitely-not-a-real-program-xyzzy".to_string()];
        assert!(check_notify_command(Some(&argv)).is_err());
    }
}

#[cfg(test)]
mod notify_tests {
    use super::*;

    #[test]
    fn fire_notify_chatty_hook_does_not_deadlock() {
        // F-07 regression: a hook that writes far more than a pipe capacity to
        // stdout must not deadlock the parent in `wait()`.
        let argv = vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            "head -c 1048576 /dev/zero; exit 0".to_string(),
        ];
        let result = fire_notify(&argv, serde_json::json!({"event": "run_finished"}));
        assert!(result.is_ok(), "chatty hook must complete: {result:?}");
    }

    #[test]
    fn fire_notify_hung_hook_times_out() {
        // F-07 regression: a hook that sleeps forever must be killed by the
        // timeout, not park the caller indefinitely.
        let argv = vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            "sleep 3600".to_string(),
        ];
        let start = std::time::Instant::now();
        let result = fire_notify(&argv, serde_json::json!({"event": "run_finished"}));
        let elapsed = start.elapsed();
        assert!(result.is_err(), "hung hook must time out");
        assert!(
            elapsed < std::time::Duration::from_secs(60),
            "timeout must be bounded: {elapsed:?}"
        );
    }
}

#[cfg(test)]
mod status_rebuild_tests {
    use super::*;
    use sea_forge_core::types::{Case, CaseState, Intent};
    use serde_json::json;

    // A runnable helper to drop a pre-existing on-disk case into a temp root
    // before a fresh `ServerState` is constructed (F-21).
    fn seed_case_on_disk(root: &std::path::Path, case_id: &str, state: CaseState) {
        let case = Case {
            version: "0.2".to_string(),
            case_id: case_id.to_string(),
            intent: Intent {
                intent_id: "int_x".into(),
                summary: "seeded".into(),
                actor_id: "operator_local".into(),
                process_id: "test".into(),
                created_at: "2026-08-14T00:00:00Z".into(),
            },
            state,
            plan_ref: "plan_x".into(),
            run_ids: vec![],
            stages: vec![],
            close_reason: None,
            created_at: "2026-08-14T00:00:00Z".into(),
            closed_at: None,
        };
        let dir = root.join("cases").join(case_id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("case.json"), serde_json::to_vec(&case).unwrap()).unwrap();
    }

    #[tokio::test]
    async fn status_resolves_a_pre_restart_case_from_disk() {
        // F-21: a fresh `ServerState` (a "restart") must answer `status` for a
        // case that exists only on disk, instead of "case not found".
        let root = tempfile::tempdir().unwrap();
        let case_id = sea_forge_core::ids::case_id().unwrap();
        seed_case_on_disk(root.path(), &case_id, CaseState::Active);

        let config = ServerConfig {
            root: root.path().to_path_buf(),
            ..ServerConfig::default()
        };
        let state = Arc::new(ServerState::new(config).unwrap());

        let request: Request = serde_json::from_value(json!({
            "verb": "status",
            "case_id": case_id,
        }))
        .unwrap();
        let response = handle_request(request, &state).await;
        assert_eq!(response["case_id"], case_id, "{response}");
        assert_eq!(response["state"], "active", "{response}");
        assert!(
            response.get("error").is_none(),
            "status must not report case not found after restart: {response}"
        );
    }
}
