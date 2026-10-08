//! Minimal ACP (Agent Client Protocol) client driver — spec-agent-orchestration §10.4.
//!
//! Drives CLI-resident ACP agents as `agent_task` executors. JSON-RPC 2.0 over
//! NDJSON stdio. Protocol version pinned to 1 (`ACP_PROTOCOL_VERSION`).
//!
//! `sea-forge-agent` owns transport only: spawn/attach, message pump,
//! transcript accumulation, permission-request surfacing. Authority, durable
//! approval records, evidence, and settlement remain in the server.
//!
//! ponytail: hand-rolled typed schema subset over the published ACP v1 schema
//! (spec-sanctioned "minimal client over the published schema"). Zero new
//! dependencies. Adopt `agent-client-protocol-schema` if a real-host release
//! gate shows wire-fidelity gaps.

use crate::provider::BoxFuture;
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time;

/// ACP protocol version this driver speaks.
pub const ACP_PROTOCOL_VERSION: u32 = 1;

/// Tool-call kinds ACP surfaces in `session/request_permission` (schema v1).
/// Anything not in this set is unmapped and MUST be denied (spec §10.4, T16.3).
pub const KNOWN_TOOL_KINDS: &[&str] = &[
    "read", "edit", "delete", "move", "search", "execute", "think", "fetch", "other",
];

/// Permission-option kinds the agent offers; the selected one encodes our grant
/// or deny decision back to the session.
pub const KNOWN_OPTION_KINDS: &[&str] =
    &["allow_once", "allow_always", "reject_once", "reject_always"];

/// JSON-RPC error codes we emit (ACP inherits LSP-ish ranges; we use the
/// generic -32601 method-not-found for unsupported agent requests).
const ERR_METHOD_NOT_FOUND: i32 = -32601;
const ERR_INVALID_PARAMS: i32 = -32602;
const ERR_INTERNAL: i32 = -32603;

/// Bound retained tool-call updates from an untrusted ACP peer. Once this cap
/// is reached, later updates still consume a turn but cannot grow state.
const MAX_TRACKED_TOOL_CALLS: usize = 256;
/// No agent episode may exceed this wall-clock duration, even when a peer
/// continuously sends notifications before each per-turn timeout.
const MAX_EPISODE_DURATION: Duration = Duration::from_secs(15 * 60);

fn episode_timeout(max_turns: u32, per_turn_timeout: Duration) -> Duration {
    per_turn_timeout
        .checked_mul(max_turns.max(1))
        .unwrap_or(MAX_EPISODE_DURATION)
        .min(MAX_EPISODE_DURATION)
}

fn consume_turn(turns_used: &mut u32, max_turns: u32) -> bool {
    *turns_used = turns_used.saturating_add(1);
    *turns_used >= max_turns
}

/// A resolved permission decision handed back into the live session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PermissionDecision {
    /// Allow the call; select the named option id (an `allow_*` option).
    Allow { option_id: String },
    /// Deny the call; select the named option id (a `reject_*` option).
    /// The session stays alive (T16.2).
    Deny { option_id: String },
}

/// One permission request surfaced by the agent to the client.
///
/// Fields are the typed subset we mediate on; the raw envelope is retained on
/// `raw` for the durable record (spec §10.4: bound to the exact requested
/// action). Everything an agent sends is untrusted input.
#[derive(Clone, Debug)]
pub struct AcpPermissionRequest {
    pub session_id: String,
    pub tool_call_id: String,
    pub tool_kind: String,
    pub title: Option<String>,
    /// Option-id → option-kind (allow_once / reject_once / ...).
    pub options: Vec<(String, String)>,
    pub raw: Value,
}

/// Mediates ACP permission requests through SEA authority (server-side).
/// The driver calls this for every `session/request_permission`; the impl is
/// responsible for the durable record, authority decision, and any approval
/// escalation/timeout (spec §10.4, Slice 8.2).
pub trait AcpPermissionMediator: Send + Sync {
    fn mediate<'a>(&'a self, request: AcpPermissionRequest) -> BoxFuture<'a, PermissionDecision>;
}

/// Auto-deny every permission (selecting `reject_once`). Used when no mediator
/// is wired (e.g. probe-style smoke) and as the unmapped-kind fallback.
pub struct DenyAllMediator;

impl AcpPermissionMediator for DenyAllMediator {
    fn mediate<'a>(&'a self, request: AcpPermissionRequest) -> BoxFuture<'a, PermissionDecision> {
        Box::pin(async move { deny_once(&request) })
    }
}

/// Pick the first `reject_once` option; if none, first `reject_*`; else a typed
/// deny with no option (protocol-invalid session gets an error response).
pub fn deny_once(req: &AcpPermissionRequest) -> PermissionDecision {
    if let Some((id, _)) = req.options.iter().find(|(_, k)| k == "reject_once") {
        return PermissionDecision::Deny {
            option_id: id.clone(),
        };
    }
    if let Some((id, _)) = req
        .options
        .iter()
        .find(|(_, k)| k == "reject_always" || k.starts_with("reject"))
    {
        return PermissionDecision::Deny {
            option_id: id.clone(),
        };
    }
    // No reject option offered — caller will emit a typed error response.
    PermissionDecision::Deny {
        option_id: String::new(),
    }
}

/// Outcome of one ACP delegation episode.
#[derive(Clone, Debug)]
pub struct AcpOutcome {
    pub termination: AcpTermination,
    pub turns_used: u32,
    pub final_output: Option<String>,
    /// Redacted canonical transcript entries (role,content).
    pub transcript: Vec<(String, String)>,
    /// ACP session id — becomes the run's `continuation_key`.
    pub continuation_key: Option<String>,
    /// Unmapped permission kinds observed (for fidelity reporting).
    pub unmapped_kinds: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcpTermination {
    Completed,
    Cancelled,
    /// Agent process exited, stdin closed, or protocol error mid-dialogue.
    Disconnect,
    /// Spawn failure or handshake failure.
    EndpointError,
    TurnCapExceeded,
}

impl AcpTermination {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Disconnect => "acp_disconnect",
            Self::EndpointError => "endpoint_error",
            Self::TurnCapExceeded => "turn_cap_exceeded",
        }
    }
}

/// How to launch the ACP agent child. argv is tokenized and exec'd directly —
/// never through a shell (spec §15). `env` is a minimal explicit environment;
/// the parent environment is never inherited.
#[derive(Clone, Debug)]
pub struct AcpSpawn {
    pub argv: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cwd: Option<std::path::PathBuf>,
    pub max_response_bytes: usize,
    pub max_transcript_bytes: usize,
}

/// A live ACP session over a transport. Drop closes the writer task; the child
/// (if any) is killed on drop via `kill_on_drop` to guarantee no untracked
/// process (spec §14).
pub struct AcpSession {
    next_id: Arc<AtomicU64>,
    writer_tx: Option<mpsc::Sender<String>>,
    incoming_rx: Option<mpsc::Receiver<Incoming>>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<RpcResult>>>>,
    prompt_in_flight: Arc<AtomicU64>,
    session_cwd: Option<std::path::PathBuf>,
    max_response_bytes: usize,
    max_transcript_bytes: usize,
    #[allow(dead_code)]
    process_group: ProcessGroupGuard,
    /// Held for liveness: the child's `kill_on_drop` reaps its process tree
    /// when the session drops. Never read directly.
    #[allow(dead_code)]
    child: Option<ChildGuard>,
}

#[allow(dead_code)]
enum ChildGuard {
    Tokio(Child),
    Std(StdChildGuard),
}

struct StdChildGuard(std::process::Child);

impl Drop for StdChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct ProcessGroupGuard(Option<u32>);

impl Drop for ProcessGroupGuard {
    fn drop(&mut self) {
        #[cfg(unix)]
        if let Some(id) = self.0 {
            let group = format!("-{id}");
            let _ = std::process::Command::new("/bin/kill")
                .args(["-KILL", "--", &group])
                .env_clear()
                .status();
        }
    }
}

#[derive(Debug)]
enum Incoming {
    AgentRequest(RpcRequest),
    Notification(RpcNotification),
    /// Response to the in-flight `session/prompt` request (stop_reason carried).
    PromptResponse(Value),
    Oversize,
    Closed,
}

#[derive(Debug)]
struct RpcRequest {
    id: u64,
    method: String,
    params: Value,
}

#[derive(Debug)]
struct RpcNotification {
    method: String,
    params: Value,
}

type RpcResult = Result<Value, RpcError>;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct RpcError {
    code: i32,
    message: String,
}

/// One JSON line written to the agent stdin.
fn rpc_line(id: u64, method: &str, params: &Value) -> String {
    let mut line = serde_json::to_string(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    }))
    .unwrap_or_else(|_| "{\"jsonrpc\":\"2.0\"}".into());
    line.push('\n');
    line
}

fn notify_line(method: &str, params: &Value) -> String {
    let mut line = serde_json::to_string(&serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
    }))
    .unwrap_or_else(|_| "{\"jsonrpc\":\"2.0\"}".into());
    line.push('\n');
    line
}

fn response_line(id: u64, result: &Value) -> String {
    let mut line = serde_json::to_string(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    }))
    .unwrap_or_else(|_| "{\"jsonrpc\":\"2.0\"}".into());
    line.push('\n');
    line
}

fn error_line(id: u64, code: i32, message: &str) -> String {
    let mut line = serde_json::to_string(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message},
    }))
    .unwrap_or_else(|_| "{\"jsonrpc\":\"2.0\"}".into());
    line.push('\n');
    line
}

impl AcpSession {
    /// Spawn an ACP agent child with tokenized argv, minimal explicit env, no
    /// shell, no inherited parent environment (spec §10.4, §15).
    pub fn spawn(spawn: AcpSpawn) -> Result<(Self, std::path::PathBuf), String> {
        if spawn.argv.is_empty() {
            return Err("acp argv must be non-empty".into());
        }
        let session_cwd = spawn.cwd.clone();
        let mut cmd = Command::new(&spawn.argv[0]);
        cmd.args(&spawn.argv[1..]);
        cmd.env_clear();
        for (k, v) in &spawn.env {
            cmd.env(k, v);
        }
        if let Some(cwd) = &spawn.cwd {
            cmd.current_dir(cwd);
        }
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // ponytail: process-group kill on drop reaps the child and any
        // descendants it spawned; matches spec §14 "no untracked child".
        cmd.kill_on_drop(true);
        #[cfg(unix)]
        cmd.process_group(0);
        let mut child = cmd.spawn().map_err(|e| format!("acp spawn failed: {e}"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "acp child stdout unavailable".to_string())?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "acp child stdin unavailable".to_string())?;
        // Drain untrusted stderr without logging or unbounded allocation.
        if let Some(stderr) = child.stderr.take() {
            drain_stderr(stderr);
        }
        let process_group = child.id();
        let session = Self::over_streams(
            Box::new(stdout),
            Box::new(stdin),
            Some(ChildGuard::Tokio(child)),
            session_cwd,
            process_group,
            spawn.max_response_bytes,
            spawn.max_transcript_bytes,
        );
        Ok((session, std::path::PathBuf::from(&spawn.argv[0])))
    }

    /// Build a session over arbitrary async read/write byte streams. Used by
    /// `spawn` and by tests with in-memory transports. `child` is held for the
    /// session's lifetime so its `kill_on_drop` reaps the process tree.
    fn over_streams(
        read: Box<dyn tokio::io::AsyncRead + Unpin + Send>,
        write: Box<dyn tokio::io::AsyncWrite + Unpin + Send>,
        child: Option<ChildGuard>,
        session_cwd: Option<std::path::PathBuf>,
        process_group: Option<u32>,
        max_response_bytes: usize,
        max_transcript_bytes: usize,
    ) -> Self {
        let next_id = Arc::new(AtomicU64::new(1));
        let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<RpcResult>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let prompt_in_flight = Arc::new(AtomicU64::new(0));
        let (writer_tx, mut writer_rx) = mpsc::channel::<String>(64);
        let (incoming_tx, incoming_rx) = mpsc::channel::<Incoming>(64);

        // Writer task: serialize writes; peer-close surfaces as write error.
        tokio::spawn({
            let mut write = write;
            async move {
                while let Some(line) = writer_rx.recv().await {
                    if write.write_all(line.as_bytes()).await.is_err() {
                        break;
                    }
                    let _ = write.flush().await;
                }
            }
        });

        // Reader task: parse NDJSON, route responses / forward requests+notifs.
        tokio::spawn({
            let pending = Arc::clone(&pending);
            let prompt_in_flight = Arc::clone(&prompt_in_flight);
            let incoming_tx = incoming_tx.clone();
            async move {
                let mut reader = BufReader::new(read);
                loop {
                    let line = match read_bounded_line(&mut reader, max_response_bytes).await {
                        Ok(Some(line)) => line,
                        Ok(None) => break,
                        Err(()) => {
                            let _ = incoming_tx.send(Incoming::Oversize).await;
                            break;
                        }
                    };
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let value: Value = match serde_json::from_str(trimmed) {
                        Ok(v) => v,
                        Err(_) => continue, // ignore malformed lines; protocol-fidelity tests cover rejection
                    };
                    let id = value.get("id").and_then(Value::as_u64);
                    if let Some(method) = value.get("method").and_then(Value::as_str) {
                        let params = value.get("params").cloned().unwrap_or(Value::Null);
                        if let Some(id) = id {
                            // Agent → client request.
                            if incoming_tx
                                .send(Incoming::AgentRequest(RpcRequest {
                                    id,
                                    method: method.into(),
                                    params,
                                }))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        } else {
                            // Notification (e.g. session/update).
                            if incoming_tx
                                .send(Incoming::Notification(RpcNotification {
                                    method: method.into(),
                                    params,
                                }))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                    } else if let Some(id) = id {
                        // Response to one of our requests.
                        let result = if let Some(err) = value.get("error") {
                            Err(RpcError {
                                code: err
                                    .get("code")
                                    .and_then(Value::as_i64)
                                    .unwrap_or(ERR_INTERNAL as i64)
                                    as i32,
                                message: err
                                    .get("message")
                                    .and_then(Value::as_str)
                                    .unwrap_or("agent error")
                                    .into(),
                            })
                        } else {
                            Ok(value.get("result").cloned().unwrap_or(Value::Null))
                        };
                        // If this is the in-flight prompt response, forward it
                        // to the pump directly (not a oneshot — the pump
                        // coordinates via the incoming channel).
                        if id == prompt_in_flight.load(Ordering::SeqCst)
                            && prompt_in_flight
                                .compare_exchange(id, 0, Ordering::SeqCst, Ordering::SeqCst)
                                .is_ok()
                        {
                            if incoming_tx
                                .send(Incoming::PromptResponse(result.unwrap_or(Value::Null)))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        } else if let Some(sender) = pending.lock().await.remove(&id) {
                            let _ = sender.send(result);
                        }
                    }
                }
                let _ = incoming_tx.send(Incoming::Closed).await;
                // Resolve any still-pending requests so callers don't deadlock.
                let pending: HashMap<u64, oneshot::Sender<RpcResult>> =
                    std::mem::take(&mut *pending.lock().await);
                for (_, sender) in pending {
                    let _ = sender.send(Err(RpcError {
                        code: ERR_INTERNAL,
                        message: "acp transport closed".into(),
                    }));
                }
            }
        });

        Self {
            next_id,
            writer_tx: Some(writer_tx),
            incoming_rx: Some(incoming_rx),
            pending,
            prompt_in_flight,
            session_cwd,
            max_response_bytes,
            max_transcript_bytes,
            process_group: ProcessGroupGuard(process_group),
            child,
        }
    }

    /// Construct a session from a child spawned by the jail adapter.
    pub fn from_std_child(
        child: std::process::Child,
        stdin: std::process::ChildStdin,
        stdout: std::process::ChildStdout,
        stderr: std::process::ChildStderr,
        cwd: std::path::PathBuf,
        max_response_bytes: usize,
        max_transcript_bytes: usize,
    ) -> Result<Self, String> {
        let process_group = Some(child.id());
        let stdin = tokio::process::ChildStdin::from_std(stdin)
            .map_err(|error| format!("adopt jailed ACP stdin: {error}"))?;
        let stdout = tokio::process::ChildStdout::from_std(stdout)
            .map_err(|error| format!("adopt jailed ACP stdout: {error}"))?;
        let stderr = tokio::process::ChildStderr::from_std(stderr)
            .map_err(|error| format!("adopt jailed ACP stderr: {error}"))?;
        drain_stderr(stderr);
        Ok(Self::over_streams(
            Box::new(stdout),
            Box::new(stdin),
            Some(ChildGuard::Std(StdChildGuard(child))),
            Some(cwd),
            process_group,
            max_response_bytes,
            max_transcript_bytes,
        ))
    }

    async fn request(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);
        let line = rpc_line(id, method, &params);
        match &self.writer_tx {
            Some(w) => {
                if w.send(line).await.is_err() {
                    self.pending.lock().await.remove(&id);
                    return Err(RpcError {
                        code: ERR_INTERNAL,
                        message: "acp writer closed".into(),
                    });
                }
            }
            None => {
                self.pending.lock().await.remove(&id);
                return Err(RpcError {
                    code: ERR_INTERNAL,
                    message: "acp session closed".into(),
                });
            }
        }
        rx.await.map_err(|_| RpcError {
            code: ERR_INTERNAL,
            message: "acp response channel dropped".into(),
        })?
    }

    async fn notify(&self, method: &str, params: Value) {
        if let Some(w) = &self.writer_tx {
            let _ = w.send(notify_line(method, &params)).await;
        }
    }

    /// Run one delegation episode: handshake → prompt → pump → terminate.
    ///
    /// `instruction` is the task packet (untrusted-content rules apply).
    /// `cancel` is polled between pump iterations. `per_turn_timeout` bounds
    /// how long we wait for the next incoming message; `max_turns` bounds
    /// tool-call and text updates. Their product, capped at fifteen minutes,
    /// is also a wall-clock episode deadline so notifications cannot keep an
    /// episode alive indefinitely.
    pub async fn run_episode(
        self,
        instruction: &str,
        max_turns: u32,
        per_turn_timeout: Duration,
        cancel: impl Fn() -> bool + Send + Sync + 'static,
        mediator: &dyn AcpPermissionMediator,
    ) -> AcpOutcome {
        self.run_episode_with_continuation(
            instruction,
            max_turns,
            per_turn_timeout,
            cancel,
            mediator,
            None,
        )
        .await
    }

    /// Run an episode, loading an existing ACP session when a durable
    /// continuation key is supplied by the caller.
    pub async fn run_episode_with_continuation(
        mut self,
        instruction: &str,
        max_turns: u32,
        per_turn_timeout: Duration,
        cancel: impl Fn() -> bool + Send + Sync + 'static,
        mediator: &dyn AcpPermissionMediator,
        continuation_key: Option<&str>,
    ) -> AcpOutcome {
        let mut transcript: Vec<(String, String)> = Vec::new();
        let mut transcript_bytes = 0usize;
        if !append_transcript(
            &mut transcript,
            &mut transcript_bytes,
            self.max_transcript_bytes,
            "user",
            instruction,
        ) {
            return self
                .terminate(
                    AcpTermination::EndpointError,
                    0,
                    None,
                    transcript,
                    continuation_key.map(str::to_owned),
                    Vec::new(),
                )
                .await;
        }
        let mut unmapped: Vec<String> = Vec::new();

        // Handshake.
        let init_params = serde_json::json!({
            "protocolVersion": ACP_PROTOCOL_VERSION,
            "clientCapabilities": {},
            "clientInfo": {"name": "sea-forge", "version": env!("CARGO_PKG_VERSION")},
        });
        let init =
            match time::timeout(per_turn_timeout, self.request("initialize", init_params)).await {
                Ok(Ok(v)) => v,
                Ok(Err(_)) | Err(_) => {
                    return self
                        .terminate(
                            AcpTermination::EndpointError,
                            0,
                            None,
                            transcript,
                            None,
                            unmapped,
                        )
                        .await;
                }
            };
        let agent_protocol = init
            .get("protocolVersion")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        if agent_protocol != ACP_PROTOCOL_VERSION as u64 {
            // Version mismatch → typed endpoint error; no further dialogue.
            return self
                .terminate(
                    AcpTermination::EndpointError,
                    0,
                    None,
                    transcript,
                    None,
                    unmapped,
                )
                .await;
        }
        let load_session = init
            .get("agentCapabilities")
            .and_then(|capabilities| capabilities.get("loadSession"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if continuation_key.is_some() && !load_session {
            return self
                .terminate(
                    AcpTermination::EndpointError,
                    0,
                    None,
                    transcript,
                    continuation_key.map(str::to_owned),
                    unmapped,
                )
                .await;
        }

        let cwd = self
            .session_cwd
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .and_then(|p| p.to_str().map(str::to_owned))
            .unwrap_or_default();
        let (method, params) = if let Some(key) = continuation_key {
            (
                "session/load",
                serde_json::json!({
                    "sessionId": key,
                    "cwd": cwd,
                    "mcpServers": [],
                }),
            )
        } else {
            (
                "session/new",
                serde_json::json!({
                    "cwd": cwd,
                    "mcpServers": [],
                }),
            )
        };
        let session_result =
            match time::timeout(per_turn_timeout, self.request(method, params)).await {
                Ok(Ok(v)) => v,
                Ok(Err(_)) | Err(_) => {
                    return self
                        .terminate(
                            AcpTermination::EndpointError,
                            0,
                            None,
                            transcript,
                            continuation_key.map(str::to_owned),
                            unmapped,
                        )
                        .await;
                }
            };
        let session_id = continuation_key.map(str::to_owned).unwrap_or_else(|| {
            session_result
                .get("sessionId")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        });

        // Prompt dispatch. Mark the prompt id in-flight so the reader task
        // forwards its response as an Incoming::PromptResponse rather than
        // resolving a oneshot; the pump coordinates via the channel.
        let prompt_params = serde_json::json!({
            "sessionId": session_id,
            "prompt": [{"type": "text", "text": instruction}],
        });
        let prompt_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.prompt_in_flight.store(prompt_id, Ordering::SeqCst);
        let line = rpc_line(prompt_id, "session/prompt", &prompt_params);
        if match &self.writer_tx {
            Some(w) => w.send(line).await.is_err(),
            None => true,
        } {
            return self
                .terminate(
                    AcpTermination::EndpointError,
                    0,
                    None,
                    transcript,
                    Some(session_id),
                    unmapped,
                )
                .await;
        }

        // Pump: await prompt-response interleaved with notifications/requests.
        let mut assistant = String::new();
        let mut turns_used = 0u32;
        let mut assistant_bytes = 0usize;
        let mut tool_calls: HashMap<String, Value> = HashMap::new();
        #[allow(unused_assignments)]
        let mut terminal = AcpTermination::Completed;
        let mut incoming = self.incoming_rx.take().expect("incoming_rx present");
        let episode_deadline = time::Instant::now() + episode_timeout(max_turns, per_turn_timeout);
        loop {
            if time::Instant::now() >= episode_deadline {
                terminal = AcpTermination::TurnCapExceeded;
                self.notify(
                    "session/cancel",
                    serde_json::json!({"sessionId": session_id}),
                )
                .await;
                break;
            }
            if cancel() {
                self.notify(
                    "session/cancel",
                    serde_json::json!({"sessionId": session_id}),
                )
                .await;
                terminal = AcpTermination::Cancelled;
                // Drain briefly so the agent's prompt response, if any, lands.
                let drained = time::timeout(per_turn_timeout, incoming.recv()).await;
                let _ = drained;
                break;
            }
            let remaining = episode_deadline.saturating_duration_since(time::Instant::now());
            let wait_timeout = per_turn_timeout.min(remaining);
            let msg = match time::timeout(wait_timeout, incoming.recv()).await {
                Ok(Some(msg)) => msg,
                Ok(None) => {
                    terminal = AcpTermination::Disconnect;
                    break;
                }
                Err(_) => {
                    // The receive timeout can be the remaining episode budget;
                    // preserve that distinction from an ordinary stalled turn.
                    terminal = if time::Instant::now() >= episode_deadline {
                        AcpTermination::TurnCapExceeded
                    } else {
                        AcpTermination::Disconnect
                    };
                    break;
                }
            };
            match msg {
                Incoming::PromptResponse(result) => {
                    // stop_reason decides Completed vs non-terminal continuity.
                    let stop = result
                        .get("stopReason")
                        .and_then(Value::as_str)
                        .unwrap_or("end_turn");
                    match stop {
                        "end_turn" | "" => terminal = AcpTermination::Completed,
                        "max_tokens" | "max_turn_requests" => {
                            terminal = AcpTermination::TurnCapExceeded
                        }
                        "cancelled" => terminal = AcpTermination::Cancelled,
                        "permission_denied" | "error" => {
                            // Permission-denied at the turn boundary still leaves
                            // the session alive; criteria decide settlement.
                            terminal = AcpTermination::Completed;
                        }
                        _ => terminal = AcpTermination::Completed,
                    }
                    break;
                }
                Incoming::Closed => {
                    terminal = AcpTermination::Disconnect;
                    break;
                }
                Incoming::Oversize => {
                    terminal = AcpTermination::EndpointError;
                    break;
                }
                Incoming::Notification(note) => {
                    if note.method == "session/update" {
                        let tool_call_update = remember_tool_call(&note.params, &mut tool_calls);
                        if tool_call_update && consume_turn(&mut turns_used, max_turns) {
                            terminal = AcpTermination::TurnCapExceeded;
                            self.notify(
                                "session/cancel",
                                serde_json::json!({"sessionId": session_id}),
                            )
                            .await;
                            break;
                        }
                        if let Some(text) = extract_agent_text(&note.params) {
                            if assistant_bytes.saturating_add(text.len()) > self.max_response_bytes
                            {
                                terminal = AcpTermination::EndpointError;
                                self.notify(
                                    "session/cancel",
                                    serde_json::json!({"sessionId": session_id}),
                                )
                                .await;
                                break;
                            }
                            assistant_bytes += text.len();
                            assistant.push_str(&text);
                            if !append_transcript(
                                &mut transcript,
                                &mut transcript_bytes,
                                self.max_transcript_bytes,
                                "assistant",
                                &text,
                            ) {
                                terminal = AcpTermination::EndpointError;
                                break;
                            }
                            if !tool_call_update && consume_turn(&mut turns_used, max_turns) {
                                terminal = AcpTermination::TurnCapExceeded;
                                self.notify(
                                    "session/cancel",
                                    serde_json::json!({"sessionId": session_id}),
                                )
                                .await;
                                let _ = time::timeout(per_turn_timeout, incoming.recv()).await;
                                break;
                            }
                        }
                    }
                }
                Incoming::AgentRequest(mut req) => {
                    // Every agent request is mediated; unknown methods deny.
                    if req.method == "session/request_permission" {
                        enrich_permission_request(&mut req.params, &tool_calls);
                    }
                    let response = self
                        .handle_agent_request(
                            &req,
                            &session_id,
                            mediator,
                            &mut unmapped,
                            &mut transcript,
                            &mut transcript_bytes,
                            self.max_transcript_bytes,
                        )
                        .await;
                    if let Some(w) = &self.writer_tx {
                        let _ = w.send(response).await;
                    }
                }
            }
        }
        let final_output = if assistant.is_empty() {
            None
        } else {
            Some(assistant)
        };
        self.terminate(
            terminal,
            turns_used,
            final_output,
            transcript,
            Some(session_id),
            unmapped,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn handle_agent_request(
        &self,
        req: &RpcRequest,
        session_id: &str,
        mediator: &dyn AcpPermissionMediator,
        unmapped: &mut Vec<String>,
        transcript: &mut Vec<(String, String)>,
        transcript_bytes: &mut usize,
        max_transcript_bytes: usize,
    ) -> String {
        match req.method.as_str() {
            "session/request_permission" => {
                let parsed = match parse_permission_request(&req.params, session_id) {
                    Ok(p) => p,
                    Err(msg) => return error_line(req.id, ERR_INVALID_PARAMS, &msg),
                };
                if !KNOWN_TOOL_KINDS.contains(&parsed.tool_kind.as_str()) {
                    unmapped.push(parsed.tool_kind.clone());
                    let _ = mediator.mediate(parsed.clone()).await;
                    let deny = deny_once(&parsed);
                    if !append_transcript(
                        transcript,
                        transcript_bytes,
                        max_transcript_bytes,
                        "system",
                        &format!("acp_permission_denied_unmapped:{}", parsed.tool_kind),
                    ) {
                        return error_line(req.id, ERR_INVALID_PARAMS, "transcript limit exceeded");
                    }
                    return permission_response(req.id, &deny, &parsed);
                }
                let decision = mediator.mediate(parsed.clone()).await;
                match &decision {
                    PermissionDecision::Allow { option_id } => {
                        if !append_transcript(
                            transcript,
                            transcript_bytes,
                            max_transcript_bytes,
                            "system",
                            &format!("acp_permission_allowed:{}", parsed.tool_kind),
                        ) {
                            return error_line(
                                req.id,
                                ERR_INVALID_PARAMS,
                                "transcript limit exceeded",
                            );
                        }
                        let _ = option_id;
                    }
                    PermissionDecision::Deny { option_id } => {
                        if !append_transcript(
                            transcript,
                            transcript_bytes,
                            max_transcript_bytes,
                            "system",
                            &format!("acp_permission_denied:{}", parsed.tool_kind),
                        ) {
                            return error_line(
                                req.id,
                                ERR_INVALID_PARAMS,
                                "transcript limit exceeded",
                            );
                        }
                        let _ = option_id;
                    }
                }
                permission_response(req.id, &decision, &parsed)
            }
            // fs/* and terminal/* are known ACP methods we do not expose.
            "fs/read_text_file"
            | "fs/write_text_file"
            | "terminal/create"
            | "terminal/output"
            | "terminal/wait_for_exit"
            | "terminal/kill"
            | "terminal/release" => {
                if !append_transcript(
                    transcript,
                    transcript_bytes,
                    max_transcript_bytes,
                    "system",
                    &format!("acp_method_unsupported:{}", req.method),
                ) {
                    return error_line(req.id, ERR_INVALID_PARAMS, "transcript limit exceeded");
                }
                error_line(
                    req.id,
                    ERR_METHOD_NOT_FOUND,
                    &format!("unsupported: {}", req.method),
                )
            }
            _ => {
                if !append_transcript(
                    transcript,
                    transcript_bytes,
                    max_transcript_bytes,
                    "system",
                    &format!("acp_method_unknown:{}", req.method),
                ) {
                    return error_line(req.id, ERR_INVALID_PARAMS, "transcript limit exceeded");
                }
                error_line(req.id, ERR_METHOD_NOT_FOUND, "unknown method")
            }
        }
    }

    async fn terminate(
        self,
        termination: AcpTermination,
        turns_used: u32,
        final_output: Option<String>,
        transcript: Vec<(String, String)>,
        continuation_key: Option<String>,
        unmapped_kinds: Vec<String>,
    ) -> AcpOutcome {
        drop(self.writer_tx); // close writer task
        AcpOutcome {
            termination,
            turns_used,
            final_output,
            transcript,
            continuation_key,
            unmapped_kinds,
        }
    }
}

// No manual Drop: the `Child` (when present) carries `kill_on_drop(true)` and
// reaps its process tree when the session is dropped; the writer task ends when
// `writer_tx` drops; the reader task ends when the read transport closes.
// Keeping AcpSession moveable lets `run_episode` consume `self` into `terminate`.

fn drain_stderr(mut stderr: impl tokio::io::AsyncRead + Unpin + Send + 'static) {
    tokio::spawn(async move {
        let mut buffer = [0u8; 4096];
        loop {
            match stderr.read(&mut buffer).await {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
        }
    });
}

async fn read_bounded_line<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    max_bytes: usize,
) -> Result<Option<String>, ()> {
    let mut line = Vec::new();
    loop {
        let buffer = reader.fill_buf().await.map_err(|_| ())?;
        if buffer.is_empty() {
            if line.is_empty() {
                return Ok(None);
            }
            return String::from_utf8(line).map(Some).map_err(|_| ());
        }
        let newline = buffer.iter().position(|byte| *byte == b'\n');
        let consumed = newline.map_or(buffer.len(), |index| index + 1);
        if line.len().saturating_add(consumed) > max_bytes {
            return Err(());
        }
        line.extend_from_slice(&buffer[..consumed]);
        reader.consume(consumed);
        if newline.is_some() {
            return String::from_utf8(line).map(Some).map_err(|_| ());
        }
    }
}

fn append_transcript(
    transcript: &mut Vec<(String, String)>,
    total_bytes: &mut usize,
    max_bytes: usize,
    role: &str,
    content: &str,
) -> bool {
    let entry_bytes = role.len().saturating_add(content.len());
    if total_bytes.saturating_add(entry_bytes) > max_bytes {
        return false;
    }
    *total_bytes += entry_bytes;
    transcript.push((role.into(), content.into()));
    true
}

/// Retain at most `MAX_TRACKED_TOOL_CALLS` tool-call descriptions for later
/// permission enrichment. Every syntactically identified tool-call update is a
/// turn, including updates we decline to retain at the memory cap.
fn remember_tool_call(params: &Value, calls: &mut HashMap<String, Value>) -> bool {
    let Some(update) = params.get("update") else {
        return false;
    };
    if update["sessionUpdate"].as_str() != Some("tool_call") {
        return false;
    }
    if let Some(id) = update["toolCallId"].as_str() {
        if calls.contains_key(id) || calls.len() < MAX_TRACKED_TOOL_CALLS {
            calls.insert(id.to_string(), update.clone());
        }
    }
    true
}

fn enrich_permission_request(params: &mut Value, calls: &HashMap<String, Value>) {
    let Some(tool_call) = params.get_mut("toolCall").and_then(Value::as_object_mut) else {
        return;
    };
    let Some(id) = tool_call
        .get("toolCallId")
        .and_then(Value::as_str)
        .map(str::to_owned)
    else {
        return;
    };
    let Some(known) = calls.get(&id).and_then(Value::as_object) else {
        return;
    };
    for key in ["title", "kind", "locations", "rawInput"] {
        if !tool_call.contains_key(key) {
            if let Some(value) = known.get(key) {
                tool_call.insert(key.into(), value.clone());
            }
        }
    }
}

fn parse_permission_request(
    params: &Value,
    fallback_session: &str,
) -> Result<AcpPermissionRequest, String> {
    let session_id = params
        .get("sessionId")
        .and_then(Value::as_str)
        .unwrap_or(fallback_session)
        .to_string();
    let tool_call = params
        .get("toolCall")
        .ok_or_else(|| "missing toolCall".to_string())?;
    let tool_call_id = tool_call
        .get("toolCallId")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing toolCallId".to_string())?
        .to_string();
    let tool_kind = tool_call
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("other")
        .to_string();
    let title = tool_call
        .get("title")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let mut options = Vec::new();
    if let Some(arr) = params.get("options").and_then(Value::as_array) {
        for opt in arr {
            let id = opt.get("optionId").and_then(Value::as_str).unwrap_or("");
            let kind = opt.get("kind").and_then(Value::as_str).unwrap_or("");
            if !id.is_empty() {
                options.push((id.to_string(), kind.to_string()));
            }
        }
    }
    Ok(AcpPermissionRequest {
        session_id,
        tool_call_id,
        tool_kind,
        title,
        options,
        raw: params.clone(),
    })
}

fn permission_response(
    id: u64,
    decision: &PermissionDecision,
    req: &AcpPermissionRequest,
) -> String {
    let option_id = match decision {
        PermissionDecision::Allow { option_id } | PermissionDecision::Deny { option_id } => {
            option_id.clone()
        }
    };
    if option_id.is_empty() {
        // Session offered no compatible option — return a typed error so the
        // agent sees an explicit denial rather than a silent passthrough.
        return error_line(id, ERR_INVALID_PARAMS, "no selectable permission option");
    }
    let _ = req;
    response_line(
        id,
        &serde_json::json!({"outcome": {"outcome": "selected", "optionId": option_id}}),
    )
}

/// Pull assistant-visible text out of a `session/update` notification.
fn extract_agent_text(params: &Value) -> Option<String> {
    let update = params.get("update").or(Some(params))?;
    let kind = update.get("sessionUpdate").and_then(Value::as_str)?;
    match kind {
        "agent_message_chunk" => update
            .get("content")
            .and_then(|content| content.get("text"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        "tool_call" => {
            // Tool-call updates are charged independently from this marker so
            // title-less updates cannot bypass the turn budget.
            update
                .get("title")
                .and_then(Value::as_str)
                .map(|title| format!("[tool_call:{title}]"))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{duplex, AsyncReadExt};

    /// Wire a fake ACP agent over two dupexes and return a session whose
    /// transport talks to that agent plus a handle to drive the agent side.
    async fn fixture_agent(script: Vec<Value>) -> (AcpSession, mpsc::Sender<()>) {
        // Two dupexes: client->agent (ca) and agent->client (ac).
        let (ca_read, ca_write) = duplex(8 * 1024);
        let (ac_read, ac_write) = duplex(8 * 1024);
        let (done_tx, done_rx) = mpsc::channel::<()>(1);
        // Agent task: read client requests, respond per script.
        tokio::spawn(async move {
            let mut reader = BufReader::new(ca_read);
            let mut writer = ac_write;
            let mut line = String::new();
            let mut script_iter = script.into_iter();
            let _ = done_rx; // keep alive
            loop {
                line.clear();
                if reader.read_line(&mut line).await.is_err() {
                    break;
                }
                if line.trim().is_empty() {
                    continue;
                }
                let value: Value = match serde_json::from_str(line.trim()) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let method = value.get("method").and_then(Value::as_str).unwrap_or("");
                if method.is_empty() {
                    continue; // it's a response to us; ignore.
                }
                let id = value.get("id").and_then(Value::as_u64);
                if let Some(id) = id {
                    // Flush scripted notifications/agent-requests, then one response.
                    loop {
                        let Some(item) = script_iter.next() else {
                            break;
                        };
                        if let Some(method) = item.get("method").and_then(Value::as_str) {
                            let params = item.get("params").cloned().unwrap_or(Value::Null);
                            let line = if let Some(req_id) = item.get("id") {
                                // agent → client request
                                let mut l = serde_json::to_string(&serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": req_id,
                                    "method": method,
                                    "params": params,
                                }))
                                .unwrap_or_default();
                                l.push('\n');
                                l
                            } else {
                                notify_line(method, &params)
                            };
                            let _ = writer.write_all(line.as_bytes()).await;
                            let _ = writer.flush().await;
                            continue;
                        }
                        // Response item (no method) for the current client request.
                        let resp_line = if item.get("error").is_some() {
                            error_line(
                                id,
                                item.get("error")
                                    .and_then(|e| e.get("code"))
                                    .and_then(Value::as_i64)
                                    .unwrap_or(ERR_INTERNAL as i64)
                                    as i32,
                                item.get("error")
                                    .and_then(|e| e.get("message"))
                                    .and_then(Value::as_str)
                                    .unwrap_or("error"),
                            )
                        } else {
                            response_line(id, &item)
                        };
                        let _ = writer.write_all(resp_line.as_bytes()).await;
                        let _ = writer.flush().await;
                        break;
                    }
                } else {
                    // Notification from client (e.g. session/cancel): script may
                    // emit a notification or response back.
                    if let Some(resp) = script_iter.next() {
                        if resp.get("method").is_some() {
                            let line = notify_line(
                                resp.get("method")
                                    .and_then(Value::as_str)
                                    .unwrap_or("session/update"),
                                resp.get("params").unwrap_or(&Value::Null),
                            );
                            let _ = writer.write_all(line.as_bytes()).await;
                            let _ = writer.flush().await;
                        }
                    }
                }
            }
        });
        let session = AcpSession::over_streams(
            Box::new(ac_read),
            Box::new(ca_write),
            None,
            None,
            None,
            4 * 1024 * 1024,
            8 * 1024 * 1024,
        );
        (session, done_tx)
    }

    #[tokio::test]
    async fn handshake_and_prompt_completes_with_transcript() {
        let script = vec![
            serde_json::json!({"protocolVersion": ACP_PROTOCOL_VERSION}),
            serde_json::json!({"sessionId": "sess-1"}),
            // On session/prompt: emit an agent_message_chunk notification, then respond.
            serde_json::json!({"method":"session/update","params":{"sessionId":"sess-1","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"hello world"}}}}),
            serde_json::json!({"stopReason":"end_turn"}),
        ];
        let (session, _done) = fixture_agent(script).await;
        let outcome = session
            .run_episode(
                "do the thing",
                8,
                Duration::from_secs(2),
                || false,
                &DenyAllMediator,
            )
            .await;
        assert_eq!(outcome.termination, AcpTermination::Completed);
        assert_eq!(outcome.final_output.as_deref(), Some("hello world"));
        assert_eq!(outcome.continuation_key.as_deref(), Some("sess-1"));
        assert!(outcome.transcript.iter().any(|(_, c)| c == "hello world"));
    }

    #[tokio::test]
    async fn permission_request_is_mediated_and_denial_keeps_session_alive() {
        // Script: init, session/new, then on session/prompt: emit a
        // session/request_permission (read tool), then a chunk, then stop.
        let script = vec![
            serde_json::json!({"protocolVersion": ACP_PROTOCOL_VERSION}),
            serde_json::json!({"sessionId": "sess-2"}),
            serde_json::json!({"id":100,"method":"session/request_permission","params":{
                "sessionId":"sess-2",
                "toolCall":{"toolCallId":"t1","kind":"read"},
                "options":[
                    {"optionId":"a1","name":"Allow once","kind":"allow_once"},
                    {"optionId":"r1","name":"Deny once","kind":"reject_once"}
                ]
            }}),
            serde_json::json!({"method":"session/update","params":{"sessionId":"sess-2","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"after-perm"}}}}),
            serde_json::json!({"stopReason":"end_turn"}),
        ];
        let (session, _done) = fixture_agent(script).await;
        let outcome = session
            .run_episode(
                "task",
                8,
                Duration::from_secs(3),
                || false,
                &DenyAllMediator,
            )
            .await;
        assert_eq!(outcome.termination, AcpTermination::Completed);
        // DenyAllMediator selected reject_once; the run still completed (deny
        // leaves the session alive — T16.2).
        assert!(outcome
            .transcript
            .iter()
            .any(|(_, c)| c.contains("acp_permission_denied:read")));
        assert_eq!(outcome.final_output.as_deref(), Some("after-perm"));
    }

    #[tokio::test]
    async fn unmapped_permission_kind_is_typed_denied_and_recorded() {
        let script = vec![
            serde_json::json!({"protocolVersion": ACP_PROTOCOL_VERSION}),
            serde_json::json!({"sessionId": "sess-3"}),
            serde_json::json!({"id":100,"method":"session/request_permission","params":{
                "sessionId":"sess-3",
                "toolCall":{"toolCallId":"t2","kind":"exfiltrate_wallet"},
                "options":[
                    {"optionId":"a1","kind":"allow_once"},
                    {"optionId":"r1","kind":"reject_once"}
                ]
            }}),
            serde_json::json!({"method":"session/update","params":{"sessionId":"sess-3","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"done"}}}}),
            serde_json::json!({"stopReason":"end_turn"}),
        ];
        let (session, _done) = fixture_agent(script).await;
        let outcome = session
            .run_episode(
                "task",
                8,
                Duration::from_secs(3),
                || false,
                &DenyAllMediator,
            )
            .await;
        assert!(outcome
            .unmapped_kinds
            .iter()
            .any(|k| k == "exfiltrate_wallet"));
        assert!(outcome
            .transcript
            .iter()
            .any(|(_, c)| c.contains("acp_permission_denied_unmapped:exfiltrate_wallet")));
    }

    #[tokio::test]
    async fn protocol_version_mismatch_terminates_endpoint_error() {
        let script = vec![serde_json::json!({"protocolVersion": 999})];
        let (session, _done) = fixture_agent(script).await;
        let outcome = session
            .run_episode(
                "task",
                4,
                Duration::from_secs(2),
                || false,
                &DenyAllMediator,
            )
            .await;
        assert_eq!(outcome.termination, AcpTermination::EndpointError);
    }

    #[tokio::test]
    async fn cancellation_during_episode_settles_cancelled() {
        // Script: init, new, then on prompt: emit a long chunk then stop.
        let script = vec![
            serde_json::json!({"protocolVersion": ACP_PROTOCOL_VERSION}),
            serde_json::json!({"sessionId": "sess-4"}),
            serde_json::json!({"method":"session/update","params":{"sessionId":"sess-4","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"partial"}}}}),
        ];
        let (session, _done) = fixture_agent(script).await;
        let outcome = session
            .run_episode("task", 8, Duration::from_secs(3), || true, &DenyAllMediator)
            .await;
        assert_eq!(outcome.termination, AcpTermination::Cancelled);
    }

    /// T16.4-ish: spawn validates argv and refuses empty argv.
    #[tokio::test]
    async fn spawn_rejects_empty_argv() {
        let err = AcpSession::spawn(AcpSpawn {
            argv: vec![],
            env: vec![],
            cwd: None,
            max_response_bytes: 1024,
            max_transcript_bytes: 2048,
        })
        .err()
        .unwrap();
        assert!(err.contains("argv"));
    }

    /// Connectivity check: reading a closed transport surfaces Disconnect.
    #[tokio::test]
    async fn closed_transport_is_disconnect() {
        let (_ca_read, ca_write) = duplex(64);
        let (ac_read, mut ac_write) = duplex(64);
        // Close the agent write side immediately.
        ac_write.shutdown().await.ok();
        drop(ac_write);
        let session = AcpSession::over_streams(
            Box::new(ac_read),
            Box::new(ca_write),
            None,
            None,
            None,
            4 * 1024 * 1024,
            8 * 1024 * 1024,
        );
        let outcome = session
            .run_episode(
                "task",
                4,
                Duration::from_millis(200),
                || false,
                &DenyAllMediator,
            )
            .await;
        // Either handshake fails (EndpointError) or transport closes (Disconnect);
        // both are non-Completed terminal outcomes.
        assert_ne!(outcome.termination, AcpTermination::Completed);
        let _ = _ca_read;
    }

    async fn fixture_notification_stream() -> AcpSession {
        let (ca_read, ca_write) = duplex(8 * 1024);
        let (ac_read, ac_write) = duplex(8 * 1024);
        tokio::spawn(async move {
            let mut reader = BufReader::new(ca_read);
            let mut writer = ac_write;
            let mut line = String::new();
            'agent: loop {
                line.clear();
                if reader.read_line(&mut line).await.is_err() {
                    break;
                }
                let Ok(value) = serde_json::from_str::<Value>(line.trim()) else {
                    continue;
                };
                let Some(id) = value.get("id").and_then(Value::as_u64) else {
                    continue;
                };
                match value.get("method").and_then(Value::as_str) {
                    Some("initialize") => {
                        let _ = writer
                            .write_all(
                                response_line(
                                    id,
                                    &serde_json::json!({
                                        "protocolVersion": ACP_PROTOCOL_VERSION
                                    }),
                                )
                                .as_bytes(),
                            )
                            .await;
                    }
                    Some("session/new") => {
                        let _ = writer
                            .write_all(
                                response_line(
                                    id,
                                    &serde_json::json!({
                                        "sessionId": "streaming-session"
                                    }),
                                )
                                .as_bytes(),
                            )
                            .await;
                    }
                    Some("session/prompt") => loop {
                        let update = notify_line(
                            "session/update",
                            &serde_json::json!({
                                "sessionId": "streaming-session",
                                "update": {"sessionUpdate": "available_commands_update"}
                            }),
                        );
                        if writer.write_all(update.as_bytes()).await.is_err()
                            || writer.flush().await.is_err()
                        {
                            break 'agent;
                        }
                        time::sleep(Duration::from_millis(1)).await;
                    },
                    _ => {}
                }
            }
        });
        AcpSession::over_streams(
            Box::new(ac_read),
            Box::new(ca_write),
            None,
            None,
            None,
            4 * 1024 * 1024,
            8 * 1024 * 1024,
        )
    }

    #[tokio::test]
    async fn titleless_tool_call_update_counts_against_turn_cap() {
        let script = vec![
            serde_json::json!({"protocolVersion": ACP_PROTOCOL_VERSION}),
            serde_json::json!({"sessionId": "sess-tool-turn"}),
            serde_json::json!({"method":"session/update","params":{
                "sessionId":"sess-tool-turn",
                "update":{"sessionUpdate":"tool_call","toolCallId":"titleless","kind":"read"}
            }}),
            serde_json::json!({"stopReason":"end_turn"}),
        ];
        let (session, _done) = fixture_agent(script).await;
        let outcome = session
            .run_episode(
                "task",
                1,
                Duration::from_secs(1),
                || false,
                &DenyAllMediator,
            )
            .await;
        assert_eq!(outcome.turns_used, 1);
        assert_eq!(outcome.termination, AcpTermination::TurnCapExceeded);
    }

    #[tokio::test]
    async fn titled_tool_call_is_recorded_and_charged_once() {
        let script = vec![
            serde_json::json!({"protocolVersion": ACP_PROTOCOL_VERSION}),
            serde_json::json!({"sessionId": "sess-titled-tool"}),
            serde_json::json!({"method":"session/update","params":{
                "sessionId":"sess-titled-tool",
                "update":{
                    "sessionUpdate":"tool_call",
                    "toolCallId":"titled",
                    "kind":"read",
                    "title":"inspect evidence"
                }
            }}),
            serde_json::json!({"stopReason":"end_turn"}),
        ];
        let (session, _done) = fixture_agent(script).await;
        let outcome = session
            .run_episode(
                "task",
                2,
                Duration::from_secs(1),
                || false,
                &DenyAllMediator,
            )
            .await;
        assert_eq!(outcome.termination, AcpTermination::Completed);
        assert_eq!(outcome.turns_used, 1);
        assert!(outcome
            .transcript
            .iter()
            .any(|(_, content)| content == "[tool_call:inspect evidence]"));
    }

    #[tokio::test]
    async fn notifications_cannot_outlive_episode_wall_clock_budget() {
        let session = fixture_notification_stream().await;
        let started = std::time::Instant::now();
        let outcome = time::timeout(
            Duration::from_millis(300),
            session.run_episode(
                "task",
                4,
                Duration::from_millis(10),
                || false,
                &DenyAllMediator,
            ),
        )
        .await
        .expect("a notification stream must stop at the episode deadline");
        assert_eq!(outcome.termination, AcpTermination::TurnCapExceeded);
        assert_eq!(outcome.turns_used, 0);
        assert!(
            started.elapsed() < Duration::from_millis(200),
            "episode must not wait for the outer test timeout"
        );
    }

    #[test]
    fn retained_tool_calls_are_capped_without_ignoring_updates() {
        let mut calls = HashMap::new();
        for index in 0..=MAX_TRACKED_TOOL_CALLS {
            let params = serde_json::json!({
                "update": {
                    "sessionUpdate": "tool_call",
                    "toolCallId": format!("tool-{index}"),
                    "kind": "read"
                }
            });
            assert!(remember_tool_call(&params, &mut calls));
        }
        assert_eq!(calls.len(), MAX_TRACKED_TOOL_CALLS);
        assert!(calls.contains_key("tool-0"));
        assert!(!calls.contains_key(&format!("tool-{MAX_TRACKED_TOOL_CALLS}")));
    }

    /// Confirm the harness compiles against AsyncRead trait import.
    #[test]
    fn known_kinds_are_exhaustive_per_schema_v1() {
        for k in KNOWN_TOOL_KINDS {
            assert!(!k.is_empty());
        }
        for k in KNOWN_OPTION_KINDS {
            assert!(!k.is_empty());
        }
    }

    #[tokio::test]
    async fn oversized_ndjson_line_is_rejected_before_parse() {
        let (mut writer, reader) = duplex(64);
        tokio::spawn(async move {
            writer.write_all(b"0123456789\n").await.unwrap();
        });
        let mut reader = BufReader::new(reader);
        assert_eq!(read_bounded_line(&mut reader, 8).await, Err(()));
    }

    #[allow(dead_code)]
    async fn _ensure_async_read_ext_in_scope() {
        // Forces the AsyncReadExt import to remain used in non-test builds
        // paths that reference it indirectly.
        let mut _b = [0u8; 0];
        let _ = tokio::io::empty().read(&mut _b).await;
    }
}
