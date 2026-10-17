//! Stdio MCP mediation through the existing authority and exact action grant.
//!
//! The operator launches this broker with trusted configuration and connects
//! one workload to its stdio. This interface does not authenticate arbitrary
//! same-UID host processes or protect operator configuration from those hosts.
use crate::mcp_protocol::{decode_json, parse_frame, RpcRequest};
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, AuthorityAction, Verdict};
use sea_forge_ledger::LedgerStream;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

const MAX_FRAME: usize = 1024 * 1024;
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct Config {
    root: PathBuf,
    workspace: PathBuf,
    policy: PathBuf,
    actor: Actor,
    server: String,
    upstream_argv: Vec<String>,
    #[serde(default)]
    upstream_env: BTreeMap<String, String>,
}

fn digest(value: &Value) -> String {
    hex::encode(Sha256::digest(value.to_string().as_bytes()))
}

fn refusal(id: Option<&Value>, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0", "id":id.unwrap_or(&Value::Null), "error":{"code":code,"message":message}})
}

struct Broker {
    config: Config,
    bundle: AuthorityPolicyBundle,
    engine: PolicyAuthorityEngine,
    ledger: LedgerStream,
    session: String,
    sequence: usize,
    initialized: bool,
    ready: bool,
    _child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl Broker {
    fn start(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        if config.server.is_empty()
            || config.actor.actor_id.is_empty()
            || config.upstream_argv.is_empty()
            || config.upstream_argv.iter().any(String::is_empty)
            || !config.workspace.is_dir()
        {
            return Err("invalid MCP broker configuration".into());
        }
        let bundle = AuthorityPolicyBundle::load(&config.policy)?;
        Self::launch(config, bundle)
    }

    fn launch(
        config: Config,
        bundle: AuthorityPolicyBundle,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let engine = PolicyAuthorityEngine::new(bundle.clone())?;
        let session = format!("mcp-{}", crate::governed_execution_boundary::new_event_id());
        let ledger = LedgerStream::open(&config.root, &session, &config.actor.actor_id)?;
        let mut child = Command::new(&config.upstream_argv[0])
            .args(&config.upstream_argv[1..])
            .env_clear()
            .envs(&config.upstream_env)
            .current_dir(&config.workspace)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()?;
        let input = child.stdin.take().ok_or("upstream stdin unavailable")?;
        let output = BufReader::new(child.stdout.take().ok_or("upstream stdout unavailable")?);
        Ok(Self {
            config,
            bundle,
            engine,
            ledger,
            session,
            sequence: 0,
            initialized: false,
            ready: false,
            _child: child,
            input,
            output,
        })
    }

    fn action(&self, name: &str, request: &Value) -> AuthorityAction {
        AuthorityAction::Reserved {
            resource_type: "mcp_tool_call".into(),
            resource_id: json!([self.config.server, name]).to_string(),
            // Hash the complete canonical request instead of recording
            // potentially sensitive arguments in authority/evidence ledgers.
            parameters: json!({"server":self.config.server,"tool":name,"request_sha256":digest(request)}),
        }
    }

    fn evaluate(
        &mut self,
        action: &AuthorityAction,
    ) -> Result<sea_forge_core::types::AuthorityDecision, Box<dyn std::error::Error>> {
        self.sequence += 1;
        Ok(self.engine.evaluate(AuthorityEvaluation {
            actor: &self.config.actor,
            binding: self
                .bundle
                .resolve_identity(&self.config.actor.actor_id, self.config.actor.role.clone()),
            run_id: &self.session,
            case_id: &self.session,
            plan_item_id: "mcp-tool",
            sequence: self.sequence,
            action,
            workspace_root: &self.config.workspace,
            evidence_refs: vec![],
            artifacts_root: None,
            timeout_secs: Some(10),
            env_keys: Default::default(),
            domainforge_candidate: None,
            opa_candidate: None,
            environment: None,
        })?)
    }

    async fn send(&mut self, request: &Value) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = format!("{request}\n");
        tokio::time::timeout(UPSTREAM_TIMEOUT, async {
            self.input.write_all(bytes.as_bytes()).await?;
            self.input.flush().await
        })
        .await??;
        Ok(())
    }

    async fn exchange(
        &mut self,
        request: &RpcRequest,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        self.send(&request.value).await?;
        let mut bytes = Vec::new();
        tokio::time::timeout(
            UPSTREAM_TIMEOUT,
            (&mut self.output)
                .take((MAX_FRAME + 1) as u64)
                .read_until(b'\n', &mut bytes),
        )
        .await??;
        if bytes.len() > MAX_FRAME || !bytes.ends_with(b"\n") {
            return Err("invalid upstream frame".into());
        }
        let response = decode_json(&bytes)?;
        if response.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
            || response.get("id") != request.id.as_ref()
            || response.get("result").is_some() == response.get("error").is_some()
        {
            return Err("uncorrelated upstream response".into());
        }
        Ok(response)
    }

    async fn tool_call(
        &mut self,
        request: &RpcRequest,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let name = request.value["params"]["name"]
            .as_str()
            .ok_or("missing tool")?;
        let action = self.action(name, &request.value);
        let decision = self.evaluate(&action)?;
        let committed = self.ledger.commit_typed(
            "authority_decision",
            vec![self.session.clone()],
            &decision,
            vec![],
        )?;
        let decision_ref = committed.entry_ulid().to_owned();
        let attempt = self.ledger.commit_typed("mcp_attempt", vec![self.session.clone()],
            &json!({"session":self.session,"decision_id":decision.decision_id,"request_sha256":digest(&request.value),"tool":name}), vec![decision_ref.clone()])?;
        if decision.verdict != Verdict::Allow {
            self.ledger.commit_typed("mcp_settlement", vec![self.session.clone()],
                &json!({"decision_id":decision.decision_id,"status":"rejected","forwarded":false,"reason":"authority_not_granted"}), vec![attempt.entry_ulid().into()])?;
            return Ok(refusal(
                request.id.as_ref(),
                -32001,
                "tool call is not authorized",
            ));
        }
        let grant = self.engine.grant(&decision, &committed, &action, None)?;
        // The same parsed request is hashed here and serialized to upstream.
        // No attacker bytes or independently reinterpreted name are forwarded.
        let forwarded_action = self.action(name, &request.value);
        grant.authorize_with_controls(
            &forwarded_action,
            &self.session,
            "mcp-tool",
            &self.config.workspace,
            &decision.compensating_controls,
        )?;
        let forward = self.ledger.commit_typed(
            "mcp_forward_attempt",
            vec![self.session.clone()],
            &json!({"decision_id":decision.decision_id,"request_sha256":digest(&request.value)}),
            vec![attempt.entry_ulid().into()],
        )?;
        let response = self.exchange(request).await;
        let accepted = response.as_ref().is_ok_and(|value| {
            value.get("error").is_none()
                && value.pointer("/result/isError") != Some(&Value::Bool(true))
        });
        self.ledger.commit_typed("mcp_settlement", vec![self.session.clone()],
            &json!({"decision_id":decision.decision_id,"status":if accepted{"accepted"}else{"rejected"},"forwarded":true,
                "response_sha256":response.as_ref().ok().map(digest),"observation":"upstream_response_only"}), vec![forward.entry_ulid().into()])?;
        response
    }

    async fn dispatch(
        &mut self,
        request: &RpcRequest,
    ) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        let id = request.id.as_ref();
        let result = match request.method.as_str() {
            "initialize" if !self.initialized => {
                let response = self.exchange(request).await?;
                if response.get("result").is_some() {
                    self.initialized = true;
                }
                response
            }
            "notifications/initialized" if self.initialized && !self.ready => {
                self.send(&request.value).await?;
                self.ready = true;
                return Ok(None);
            }
            "ping" => json!({"jsonrpc":"2.0","id":id,"result":{}}),
            "tools/call" if self.ready => self.tool_call(request).await?,
            "tools/list" if self.ready => {
                let mut response = self.exchange(request).await?;
                if let Some(tools) = response
                    .pointer_mut("/result/tools")
                    .and_then(Value::as_array_mut)
                {
                    let mut permitted = Vec::new();
                    for tool in tools.iter() {
                        let Some(name) = tool.get("name").and_then(Value::as_str) else {
                            return Err("invalid upstream tool descriptor".into());
                        };
                        let probe = json!({"jsonrpc":"2.0","id":0,"method":"tools/call","params":{"name":name,"arguments":{}}});
                        let action = self.action(name, &probe);
                        let decision = self.evaluate(&action)?;
                        self.ledger.commit_typed(
                            "mcp_discovery_decision",
                            vec![self.session.clone()],
                            &decision,
                            vec![],
                        )?;
                        if decision.verdict == Verdict::Allow {
                            permitted.push(tool.clone());
                        }
                    }
                    *tools = permitted;
                }
                response
            }
            _ => refusal(id, -32601, "method unavailable in this session"),
        };
        Ok(id.map(|_| result))
    }

    async fn frame(&mut self, bytes: &[u8]) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        let (batch, requests) = match parse_frame(bytes) {
            Ok(frame) => frame,
            Err(_) => {
                self.ledger.commit_typed(
                    "mcp_protocol_refusal",
                    vec![self.session.clone()],
                    &json!({"frame_sha256":hex::encode(Sha256::digest(bytes)),"forwarded":false}),
                    vec![],
                )?;
                return Ok(Some(refusal(None, -32600, "invalid or ambiguous request")));
            }
        };
        let mut responses = Vec::new();
        for request in requests {
            if let Some(response) = self.dispatch(&request).await? {
                responses.push(response);
            }
        }
        if responses.is_empty() {
            Ok(None)
        } else if batch {
            Ok(Some(Value::Array(responses)))
        } else {
            Ok(responses.pop())
        }
    }
}

/// Run one operator-provisioned stdio session. Configuration and upstream
/// credentials belong to the trusted launcher, never to the workload input.
pub fn run(config_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = serde_json::from_slice(&std::fs::read(config_path)?)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let mut broker = {
        let _guard = runtime.enter();
        Broker::start(config)?
    };
    let input = std::io::stdin();
    let mut input = input.lock();
    let output = std::io::stdout();
    let mut output = output.lock();
    loop {
        let mut bytes = Vec::new();
        let count = std::io::Read::take(&mut input, (MAX_FRAME + 1) as u64)
            .read_until(b'\n', &mut bytes)?;
        if count == 0 {
            break;
        }
        if count > MAX_FRAME || !bytes.ends_with(b"\n") {
            return Err("MCP frame exceeds limit or is incomplete".into());
        }
        if let Some(response) = runtime.block_on(broker.frame(&bytes))? {
            writeln!(output, "{response}")?;
            output.flush()?;
        }
    }
    if broker._child.try_wait()?.is_none() {
        runtime.block_on(broker._child.kill())?;
    }
    runtime.block_on(broker._child.wait())?;
    Ok(())
}

/// One operator-configured upstream MCP server. Its argv and environment
/// (including any upstream credential) stay with the trusted launcher; the
/// workload only ever reaches the broker socket.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpServerConfig {
    pub name: String,
    pub argv: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

/// Stops a workload's broker (and kills an open session's upstream) on drop.
pub struct McpListenerGuard {
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
    root: PathBuf,
    server: String,
    actor: String,
}

impl McpListenerGuard {
    /// SEC-08 revocation barrier: stop accepting, end any open session and
    /// kill its upstream, and only then record the acknowledgement. Nothing
    /// can be forwarded for this workload after the returned record.
    pub fn revoke(mut self, reason: &str) -> Result<Value, Box<dyn std::error::Error>> {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(thread) = self.thread.take() {
            thread.join().map_err(|_| "MCP broker thread panicked")?;
        }
        let record = json!({"server": self.server, "reason": reason,
            "barrier": "listener_closed_session_ended_upstream_killed",
            "acknowledged_at": chrono::Utc::now().to_rfc3339()});
        LedgerStream::open(
            &self.root,
            format!("mcp-listener-{}", self.server),
            &self.actor,
        )?
        .commit_typed(
            "mcp_session_revoked",
            vec![self.server.clone()],
            &record,
            vec![],
        )?;
        Ok(record)
    }
}

impl Drop for McpListenerGuard {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Serve brokered sessions for exactly one isolated workload on `listener`.
///
/// Every connection must come from `workload_uid` (kernel `SO_PEERCRED`);
/// anything else is refused and recorded without reaching an upstream. Each
/// accepted connection is a fresh broker session with its own upstream
/// process, so reconnection never inherits another session's state.
#[allow(clippy::too_many_arguments)]
pub fn serve_workload(
    listener: std::os::unix::net::UnixListener,
    workload_uid: u32,
    root: PathBuf,
    workspace: PathBuf,
    bundle: AuthorityPolicyBundle,
    actor: Actor,
    server: McpServerConfig,
) -> std::io::Result<McpListenerGuard> {
    listener.set_nonblocking(true)?;
    let (stop_tx, mut stop) = tokio::sync::oneshot::channel::<()>();
    let (guard_root, guard_server, guard_actor) =
        (root.clone(), server.name.clone(), actor.actor_id.clone());
    let thread = std::thread::Builder::new()
        .name(format!("mcp-broker-{}", server.name))
        .spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            runtime.block_on(async move {
                let Ok(listener) = tokio::net::UnixListener::from_std(listener) else {
                    return;
                };
                let config = Config {
                    root: root.clone(),
                    workspace,
                    policy: PathBuf::new(),
                    actor: actor.clone(),
                    server: server.name.clone(),
                    upstream_argv: server.argv.clone(),
                    upstream_env: server.env.clone(),
                };
                loop {
                    let stream = tokio::select! {
                        _ = &mut stop => return,
                        accepted = listener.accept() => match accepted {
                            Ok((stream, _)) => stream,
                            Err(_) => return,
                        },
                    };
                    let peer = stream.peer_cred().ok().map(|cred| cred.uid());
                    if peer != Some(workload_uid) {
                        let stream_id = format!("mcp-listener-{}", server.name);
                        if let Ok(ledger) = LedgerStream::open(&root, &stream_id, &actor.actor_id) {
                            let _ = ledger.commit_typed(
                                "mcp_peer_refused",
                                vec![server.name.clone()],
                                &json!({"server": server.name, "expected_uid": workload_uid,
                                    "peer_uid": peer, "forwarded": false}),
                                vec![],
                            );
                        }
                        continue;
                    }
                    let Ok(mut broker) = Broker::launch(config.clone(), bundle.clone()) else {
                        return;
                    };
                    let stopped = tokio::select! {
                        _ = &mut stop => true,
                        _ = broker.serve_stream(stream) => false,
                    };
                    let _ = broker._child.kill().await;
                    if stopped {
                        return;
                    }
                }
            });
        })?;
    Ok(McpListenerGuard {
        stop: Some(stop_tx),
        thread: Some(thread),
        root: guard_root,
        server: guard_server,
        actor: guard_actor,
    })
}

impl Broker {
    async fn serve_stream(
        &mut self,
        stream: tokio::net::UnixStream,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (read, mut write) = stream.into_split();
        let mut read = BufReader::new(read);
        loop {
            let mut bytes = Vec::new();
            let count = (&mut read)
                .take((MAX_FRAME + 1) as u64)
                .read_until(b'\n', &mut bytes)
                .await?;
            if count == 0 {
                return Ok(());
            }
            if count > MAX_FRAME || !bytes.ends_with(b"\n") {
                return Err("MCP frame exceeds limit or is incomplete".into());
            }
            if let Some(response) = self.frame(&bytes).await? {
                write.write_all(format!("{response}\n").as_bytes()).await?;
                write.flush().await?;
            }
        }
    }
}
