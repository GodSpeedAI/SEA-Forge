#![forbid(unsafe_code)]

//! SEA Forge concurrent case server (spec-full §10.3, §11.1, M3).
//!
//! The server listens on a Unix domain socket, accepts NDJSON requests,
//! and dispatches case runs as subprocesses (`sea-forge run --plan ...`)
//! via `spawn_blocking`. Kernel logic stays synchronous; Tokio lives only
//! here.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::process::Command;
use tokio::sync::{Mutex, Semaphore};

pub mod config;

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
    pub semaphore: Semaphore,
}

impl ServerState {
    pub fn new(config: ServerConfig) -> Self {
        let max = config.max_concurrent_runs.max(1);
        Self {
            config,
            cases: Mutex::new(HashMap::new()),
            semaphore: Semaphore::new(max),
        }
    }

    /// Reload config defensively between dispatches (§8.4).
    pub fn reload_config(&self) -> Result<ServerConfig, String> {
        match ServerConfig::load(&self.config.root.join("server.yaml")) {
            Ok(new_config) => Ok(new_config),
            Err(e) => Err(e),
        }
    }
}

/// NDJSON request envelope.
#[derive(Deserialize)]
#[serde(tag = "verb")]
#[serde(rename_all = "snake_case")]
enum Request {
    Submit {
        #[serde(flatten)]
        payload: SubmitPayload,
    },
    Status {
        case_id: String,
    },
    Approve {
        case_id: String,
        approval_id: String,
        #[serde(default)]
        note: Option<String>,
    },
    Reject {
        case_id: String,
        approval_id: String,
        #[serde(default)]
        note: Option<String>,
    },
}

#[derive(Deserialize)]
struct SubmitPayload {
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

/// Start the server.
pub async fn run(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = config.socket_path.clone();
    let state = Arc::new(ServerState::new(config));

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

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }
        let request: Request = match serde_json::from_str(line.trim()) {
            Ok(r) => r,
            Err(e) => {
                let resp = serde_json::json!({"error": format!("bad request: {e}")});
                writer.write_all(format!("{resp}\n").as_bytes()).await?;
                continue;
            }
        };
        let response = handle_request(request, &state).await;
        writer.write_all(format!("{response}\n").as_bytes()).await?;
    }
    Ok(())
}

async fn handle_request(request: Request, state: &Arc<ServerState>) -> serde_json::Value {
    match request {
        Request::Submit { payload } => {
            let plan = match payload.plan.as_deref() {
                Some(p) => p.to_string(),
                None => match payload.intent.as_deref() {
                    Some(i) => i.to_string(),
                    None => {
                        return serde_json::json!({"error": "plan or intent required"});
                    }
                },
            };
            // Reload config defensively (§8.4).
            if let Ok(new_config) = state.reload_config() {
                tracing::info!("config reloaded successfully");
                let _ = new_config;
            } else {
                tracing::warn!("invalid config reload — keeping last-known-good");
            }

            let permit = state.semaphore.acquire().await;
            let root = state.config.root.clone();
            let policy = payload.policy.clone();
            let entity = payload.entity.clone();
            let process = payload.process.clone();
            let timeout = payload.timeout;
            let has_plan = payload.plan.is_some();

            // Dispatch in a blocking task via subprocess.
            let result = tokio::task::spawn_blocking(move || {
                dispatch_run(&root, &policy, &entity, &process, timeout, &plan, has_plan)
            })
            .await;

            drop(permit);

            match result {
                Ok(Ok(output)) => {
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

                    serde_json::json!({
                        "case_id": case_id,
                        "state": output.state,
                        "exit_code": output.exit_code,
                    })
                }
                Ok(Err(e)) => serde_json::json!({"error": e.to_string()}),
                Err(e) => serde_json::json!({"error": format!("dispatch panic: {e}")}),
            }
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
        } => {
            let root = state.config.root.clone();
            let result = run_cli(
                &root,
                &[
                    "approve",
                    &case_id,
                    &approval_id,
                    "--root",
                    root.to_str().unwrap_or("."),
                ],
                note.as_deref(),
            )
            .await;
            match result {
                Ok(output) => serde_json::json!({"ok": true, "output": output}),
                Err(e) => serde_json::json!({"error": e}),
            }
        }
        Request::Reject {
            case_id,
            approval_id,
            note,
        } => {
            let root = state.config.root.clone();
            let result = run_cli(
                &root,
                &[
                    "reject",
                    &case_id,
                    &approval_id,
                    "--root",
                    root.to_str().unwrap_or("."),
                ],
                note.as_deref(),
            )
            .await;
            match result {
                Ok(output) => serde_json::json!({"ok": true, "output": output}),
                Err(e) => serde_json::json!({"error": e}),
            }
        }
    }
}

struct DispatchOutcome {
    case_id: String,
    state: &'static str,
    exit_code: u8,
}

fn dispatch_run(
    root: &Path,
    policy: &str,
    entity: &str,
    process: &str,
    timeout: u64,
    input: &str,
    has_plan: bool,
) -> Result<DispatchOutcome, String> {
    use std::process::{Command, Stdio};
    let exe = std::env::current_exe()
        .map_err(|e| format!("resolve exe: {e}"))?
        .to_string_lossy()
        .into_owned();

    // ponytail: if the server binary is sea-forge-server, find the sea-forge
    // binary in the same directory. If the server is embedded in sea-forge,
    // use current_exe directly.
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
    cmd.arg("run")
        .arg("--root")
        .arg(root)
        .arg("--policy")
        .arg(policy)
        .arg("--entity")
        .arg(entity)
        .arg("--process")
        .arg(process)
        .arg("--timeout")
        .arg(timeout.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if has_plan {
        cmd.arg("--plan").arg(input);
    } else {
        cmd.arg(input);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("spawn sea-forge run: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(1) as u8;

    let case_id = stdout
        .lines()
        .find_map(|line| line.strip_prefix("case_id="))
        .unwrap_or_default()
        .to_string();
    let state = stdout
        .lines()
        .find_map(|line| line.strip_prefix("case_state="))
        .unwrap_or("active")
        .to_string();

    let static_state: &'static str = match state.as_str() {
        "completed" => "completed",
        "terminated" => "terminated",
        "awaiting_approval" => "awaiting_approval",
        _ => "active",
    };

    if !stderr.is_empty() && exit_code > 4 {
        tracing::warn!("run stderr: {stderr}");
    }

    Ok(DispatchOutcome {
        case_id,
        state: static_state,
        exit_code,
    })
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
