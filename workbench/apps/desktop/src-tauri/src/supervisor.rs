//! Cell supervision: the packaged Workbench starts the kernel it talks to.
//!
//! Decision U-06, resolved here: **Tauri-supervised sidecar, with adoption.**
//!
//! The alternative on the table was a separately installed service that the
//! operator starts by hand. It was rejected on the packet's own stated outcome
//! — "a user installs the packaged product on a clean Linux host and completes
//! the full governed journey without source-tree knowledge". A Workbench that
//! opens onto a dead socket until the operator finds and runs a second binary
//! does not meet that, no matter how well it renders the failure.
//!
//! Adoption is what keeps the decision reversible. If a server is already
//! listening on this cell's socket — started by systemd, by `just`, by a
//! terminal, by another Workbench — this process attaches to it and never
//! spawns, never signals, never stops it. So the "separate service" model
//! still works exactly as it would have; it is simply no longer *required*.
//! Reverting the decision means not shipping the sidecar, not unpicking code.
//!
//! Two guarantees make the spawn safe rather than merely convenient:
//!
//!   * The server itself fails closed on a double start. `lock_socket_path`
//!     takes an exclusive lock for the process lifetime, so even if the probe
//!     below races another launch, the loser exits instead of two kernels
//!     serving one cell.
//!   * A stale socket left by a crashed server is not a trap. The server binds
//!     a staging path and renames it into place, which atomically replaces the
//!     dead file. So "connect failed" always means "no live server", never
//!     "someone left a file here".

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Deserialize;

/// The socket file name inside a cell root. Must match
/// `sea_forge_server::config::SOCKET_FILE_NAME`; `src-tauri` is a separate
/// Cargo workspace (ADR-004 / K-06), so the constant is duplicated rather than
/// imported and pinned by the tests below.
pub const SOCKET_FILE_NAME: &str = "server.sock";

/// The conventional cell root directory name.
pub const DEFAULT_ROOT_DIR: &str = ".sea-forge";

/// The sidecar's file name beside this executable and in the bundle.
const SERVER_BIN_NAME: &str = "sea-forge-server";

/// Where a supervised server's stderr is captured, relative to the cell root.
/// The operator needs the reason a spawn failed, and a windowed application has
/// nowhere else to put it.
const SERVER_LOG_NAME: &str = "workbench-server.log";

/// The persisted schema that this build can inspect without starting a server.
/// The server remains the authority for rebuilding a compatible self-model;
/// this host-side check only stops an unknown future schema from being treated
/// as a blank or compatible cell before the sidecar would write to it.
const SELF_MODEL_SCHEMA_VERSION: &str = "self_model.v1";
const SELF_MODEL_MANIFEST: &str = ".sea-forge/self-model/manifest.json";

/// How long to wait for a freshly spawned server to publish its socket.
///
/// ponytail: a fixed deadline rather than a readiness handshake. The server
/// binds after creating the cell's directories and ledgers, which is fast on a
/// warm cell and bounded on a cold one. If cold-start ever outgrows this,
/// the upgrade is to read the server's own "listening" line off the captured
/// stderr instead of polling the socket.
const SPAWN_DEADLINE: Duration = Duration::from_secs(15);

/// Poll interval while waiting for the socket to accept.
const PROBE_INTERVAL: Duration = Duration::from_millis(50);

/// A resolved cell: where the records live, and where the socket is.
///
/// The two are resolved together because a supervised server must be told
/// *both*. Its own defaults are CWD-relative, and a windowed application has no
/// meaningful working directory — so passing only one would let the Workbench
/// and its own kernel disagree about which cell they are in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub root: PathBuf,
    pub socket: PathBuf,
}

/// Resolve the cell from the process environment, per `docs/CELL_CONTRACT.md`.
pub fn resolve_cell() -> Cell {
    resolve_cell_from(
        std::env::var_os("SEA_FORGE_SOCKET").map(PathBuf::from),
        std::env::var_os("SEA_FORGE_ROOT").map(PathBuf::from),
        home_dir(),
    )
}

/// Pure core of [`resolve_cell`], so the contract is testable without mutating
/// process environment.
///
/// `SEA_FORGE_SOCKET` overrides only the socket, never the root. That is the
/// remedy the server prints when a cell root is too deep for a Unix socket
/// ("point `SEA_FORGE_SOCKET` at a short path *while keeping records where they
/// are*"), so following that advice must not silently relocate the records.
pub fn resolve_cell_from(
    socket_override: Option<PathBuf>,
    root: Option<PathBuf>,
    home: Option<PathBuf>,
) -> Cell {
    // A windowed application has no meaningful working directory, so the
    // default anchors at $HOME rather than following the server's CWD-relative
    // default. A packaged install that wants a different cell sets the env.
    let root = root.unwrap_or_else(|| {
        home.unwrap_or_else(|| PathBuf::from("."))
            .join(DEFAULT_ROOT_DIR)
    });
    let socket = socket_override.unwrap_or_else(|| root.join(SOCKET_FILE_NAME));
    Cell { root, socket }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// How this Workbench came to have a kernel to talk to. Serialized to the
/// renderer so the shell can say which of these is true rather than inferring
/// it from whether calls happen to be working.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Supervision {
    /// A server was already listening. This process did not start it and will
    /// not stop it.
    Adopted,
    /// This process started the server and owns its lifetime.
    Supervised { pid: u32 },
    /// A fresh root has no records to open. Starting it is a durable effect, so
    /// the renderer must obtain an explicit operator confirmation first.
    InitializationRequired,
    /// No server is reachable and this process could not start one. The app
    /// still opens: every surface already resolves its own standing from a
    /// catalog it cannot negotiate, so an unreachable cell renders honestly
    /// rather than not at all.
    Unavailable {
        error_class: String,
        message: String,
    },
}

struct SupervisionState {
    supervision: Supervision,
    child: Option<Child>,
}

/// Is something accepting connections on `socket` right now?
///
/// A blocking `connect(2)` on a Unix socket either completes or fails
/// immediately — there is no network round trip to wait on — so this is cheap
/// enough to poll. A stale socket file fails with `ECONNREFUSED`, which is the
/// distinction that makes "adopt vs spawn" decidable at all.
pub fn is_listening(socket: &Path) -> bool {
    std::os::unix::net::UnixStream::connect(socket).is_ok()
}

/// Locate the server binary this Workbench should supervise.
///
/// Order, and why each rung exists:
///   1. `SEA_FORGE_SERVER_BIN` — the operator's and the E2E harness's explicit
///      override; this is also the developer loop's escape hatch when no
///      sidecar has been staged or installed.
///   2. A sibling of this executable — where `bundle.externalBin` puts the
///      sidecar in the installed package, and where `tauri dev` stages it.
///
/// There is deliberately no third rung on `PATH` (F-25.c). Whatever a `PATH`
/// search finds is not the kernel this build was packaged with, and starting
/// it would point an unverified binary at the operator's cell. When neither
/// rung names a file, resolution fails closed and startup reports
/// `server_binary_not_found`.
pub fn resolve_server_binary() -> Result<PathBuf, String> {
    resolve_server_binary_from(
        std::env::var_os("SEA_FORGE_SERVER_BIN").map(PathBuf::from),
        std::env::current_exe().ok().and_then(|exe| {
            let dir = exe.parent()?.to_path_buf();
            Some(dir)
        }),
    )
}

/// Pure core of [`resolve_server_binary`].
pub fn resolve_server_binary_from(
    explicit: Option<PathBuf>,
    exe_dir: Option<PathBuf>,
) -> Result<PathBuf, String> {
    if let Some(explicit) = explicit {
        if explicit.is_file() {
            return Ok(explicit);
        }
        // An override that does not exist is a configuration mistake, not a
        // reason to quietly fall through to a different binary than the one the
        // operator named.
        return Err(format!(
            "SEA_FORGE_SERVER_BIN points at {}, which is not a file",
            explicit.display()
        ));
    }

    if let Some(dir) = exe_dir {
        let sibling = dir.join(SERVER_BIN_NAME);
        if sibling.is_file() {
            return Ok(sibling);
        }
    }

    Err(format!(
        "no `{SERVER_BIN_NAME}` binary found beside this application. \
         Install the packaged SEA Forge Workbench, or set SEA_FORGE_SERVER_BIN \
         to the path of a sea-forge-server binary."
    ))
}

/// Owns whatever server process this Workbench started. Adopted servers are
/// deliberately not represented here: nothing this type can do should ever
/// reach a process it did not create.
pub struct CellSupervisor {
    cell: Cell,
    binary: Result<PathBuf, String>,
    state: Mutex<SupervisionState>,
}

impl CellSupervisor {
    /// Attach to this cell's server, starting one if nothing is listening.
    pub fn start(cell: Cell) -> Self {
        Self::start_with(cell, resolve_server_binary())
    }

    /// [`start`](Self::start) with the binary lookup already done, so the whole
    /// adopt/spawn/refuse decision is testable without a binary on `PATH` or a
    /// particular `current_exe()`.
    pub fn start_with(cell: Cell, binary: Result<PathBuf, String>) -> Self {
        let state = if is_listening(&cell.socket) {
            supervise_existing(&cell, &binary)
        } else {
            match inspect_cell_root(&cell.root) {
                Ok(CellRootState::Fresh) => SupervisionState {
                    supervision: Supervision::InitializationRequired,
                    child: None,
                },
                Ok(CellRootState::Existing) => supervise_existing(&cell, &binary),
                Err((error_class, message)) => SupervisionState {
                    supervision: Supervision::Unavailable {
                        error_class,
                        message,
                    },
                    child: None,
                },
            }
        };

        Self {
            cell,
            binary,
            state: Mutex::new(state),
        }
    }

    pub fn cell(&self) -> &Cell {
        &self.cell
    }

    pub fn supervision(&self) -> Supervision {
        self.state
            .lock()
            .map(|state| state.supervision.clone())
            .unwrap_or_else(|_| Supervision::Unavailable {
                error_class: "supervisor_state_unavailable".into(),
                message: "the cell supervisor state could not be read".into(),
            })
    }

    /// Start a pending fresh cell after explicit operator confirmation. If a
    /// second process created history while the confirmation was open, refuse
    /// rather than treating that now-populated root as the blank directory the
    /// user originally inspected.
    pub fn initialize(&self) -> Result<Supervision, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "the cell supervisor state could not be locked".to_string())?;
        if !matches!(state.supervision, Supervision::InitializationRequired) {
            return Ok(state.supervision.clone());
        }

        if !matches!(
            inspect_cell_root(&self.cell.root).map_err(|(_, message)| message)?,
            CellRootState::Fresh
        ) {
            return Err(format!(
                "{} now contains history; refuse to initialize over an existing cell",
                self.cell.root.display()
            ));
        }

        *state = supervise_existing(&self.cell, &self.binary);
        Ok(state.supervision.clone())
    }

    /// Stop a server this process started. Adopted servers are left running:
    /// this Workbench did not start them and has no standing to stop them.
    ///
    /// ponytail: `Child::kill` is SIGKILL, because std offers no SIGTERM
    /// without a new dependency. That is sound rather than merely expedient —
    /// the kernel's records are append-only with durable writes and surviving
    /// an abrupt stop is an explicit acceptance criterion — but it does end
    /// in-flight sandboxed work. Operators who need work to outlive the window
    /// run the server separately and get the adopt path, which is documented in
    /// OPERATIONS_AND_STARTUP.md. Upgrade path if graceful drain is ever
    /// wanted: send SIGTERM via `nix`/`libc` and wait, falling back to kill.
    pub fn shutdown(&self) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if let Some(mut child) = state.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// The only self-model manifest fields the host needs to inspect before a
/// sidecar exists. The kernel owns its full shape and all migration work.
#[derive(Deserialize)]
struct SelfModelManifestHeader {
    schema_version: String,
}

/// Whether a resolved root is fresh or recognizable history, without creating
/// it. A missing/empty directory is fresh; a non-empty root remains compatible
/// unless it explicitly declares an unknown or malformed self-model schema.
///
/// That narrow refusal prevents a newer cell from being silently written by an
/// older Workbench. It deliberately does not invent a separate host migration:
/// a compatible self-model migration belongs to the kernel's append-only
/// `ensure_init` lifecycle, not this process supervisor.
enum CellRootState {
    Fresh,
    Existing,
}

fn inspect_cell_root(root: &Path) -> Result<CellRootState, (String, String)> {
    match std::fs::read_dir(root) {
        Ok(mut entries) => {
            if entries.next().is_none() {
                Ok(CellRootState::Fresh)
            } else {
                inspect_existing_cell_root(root)
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(CellRootState::Fresh),
        Err(error) => Err((
            "cell_root_unreadable".into(),
            format!("cannot inspect cell root {}: {error}", root.display()),
        )),
    }
}

fn inspect_existing_cell_root(root: &Path) -> Result<CellRootState, (String, String)> {
    let manifest = root.join(SELF_MODEL_MANIFEST);
    let bytes = match std::fs::read(&manifest) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(CellRootState::Existing)
        }
        Err(error) => {
            return Err((
                "cell_history_unreadable".into(),
                format!(
                    "cannot inspect self-model history at {}: {error}",
                    manifest.display()
                ),
            ))
        }
    };
    let header: SelfModelManifestHeader = serde_json::from_slice(&bytes).map_err(|error| {
        (
            "cell_history_incompatible".into(),
            format!(
                "self-model history at {} is malformed ({error}); open it with a compatible SEA Forge build instead of initializing it as a new cell",
                manifest.display()
            ),
        )
    })?;
    if header.schema_version != SELF_MODEL_SCHEMA_VERSION {
        return Err((
            "cell_history_incompatible".into(),
            format!(
                "self-model history at {} uses unsupported schema {}; this Workbench supports {}. Open it with a compatible SEA Forge build instead of initializing it as a new cell",
                manifest.display(),
                header.schema_version,
                SELF_MODEL_SCHEMA_VERSION,
            ),
        ));
    }
    Ok(CellRootState::Existing)
}

fn supervise_existing(cell: &Cell, binary: &Result<PathBuf, String>) -> SupervisionState {
    // Probe first. Spawning into a live cell would be refused by the server's
    // own socket lock, but the refusal would be indistinguishable from a real
    // startup failure — so ask before acting.
    if is_listening(&cell.socket) {
        return SupervisionState {
            supervision: Supervision::Adopted,
            child: None,
        };
    }

    let binary = match binary {
        Ok(binary) => binary,
        Err(message) => {
            return SupervisionState {
                supervision: Supervision::Unavailable {
                    error_class: "server_binary_not_found".into(),
                    message: message.clone(),
                },
                child: None,
            }
        }
    };

    match spawn_server(binary, cell) {
        Ok(child) => {
            let pid = child.id();
            SupervisionState {
                supervision: Supervision::Supervised { pid },
                child: Some(child),
            }
        }
        Err((error_class, message)) => SupervisionState {
            supervision: Supervision::Unavailable {
                error_class,
                message,
            },
            child: None,
        },
    }
}

impl Drop for CellSupervisor {
    fn drop(&mut self) {
        // Belt and braces: the Tauri `RunEvent::Exit` hook is the intended
        // path, but a panic unwinding past it must not leak a kernel.
        self.shutdown();
    }
}

/// Stop the supervised kernel when this process is signalled.
///
/// Neither of the other two shutdown paths covers a signal. `RunEvent::Exit`
/// is emitted by Tauri's event loop, which a signalled process never reaches,
/// and the default disposition for `SIGTERM`/`SIGINT` terminates without
/// unwinding, so `Drop` never runs either. Observed directly: `kill <app-pid>`
/// on the packaged Workbench left its `sea-forge-server` running, reparented,
/// still holding the socket and the cell — an operator who "closed" the
/// application still had a governed kernel serving.
///
/// `SIGKILL` remains uncatchable, by anyone. That case is covered instead by
/// adoption: the next launch finds the survivor and attaches to it rather than
/// starting a rival, so the leak is recoverable rather than corrupting.
pub fn watch_for_signals(supervisor: std::sync::Arc<CellSupervisor>) {
    tauri::async_runtime::spawn(async move {
        use tokio::signal::unix::{signal, SignalKind};
        let (Ok(mut term), Ok(mut interrupt)) = (
            signal(SignalKind::terminate()),
            signal(SignalKind::interrupt()),
        ) else {
            log::warn!("cannot install signal handlers; a signalled exit will leak the kernel");
            return;
        };
        tokio::select! {
            _ = term.recv() => {}
            _ = interrupt.recv() => {}
        }
        supervisor.shutdown();
        // Exit explicitly: the signal's default disposition was replaced by
        // installing the handler, so returning here would leave the process
        // running with no kernel and no window path to stop it.
        std::process::exit(0);
    });
}

/// Environment variables a supervised sidecar may receive, as an explicit
/// allowlist.
///
/// Principle (F-25.b): clear the environment, then pass only these — never
/// inherit-and-add. The desktop process carries whatever launched the session,
/// including credential material the kernel has no business reading, so every
/// name here must be something `sea-forge-server` demonstrably reads:
///
///   * `PATH` / `HOME` — forwarded to governed child processes at dispatch
///     time (`case_dispatch`), and `notify_command` argv0 is resolved against
///     `PATH` at config validation.
///   * `TMPDIR` — honored by `std::env::temp_dir` staging (the identity probe
///     and the sandbox's safe-join parent) instead of silently defaulting.
///   * `RUST_LOG` — the sidecar's own log filter (`EnvFilter`); keeping it
///     tunable keeps the captured `workbench-server.log` diagnostic.
///
/// A name absent from this process's environment is simply not passed. This
/// deliberately excludes everything else: a credential reference configured on
/// an agent endpoint resolves against the sidecar's environment, so one that
/// pointed into the desktop session now fails closed with the server's own
/// `missing_credential_error`, naming the variable. Operators who source
/// credentials from their session run the server themselves and take the
/// documented adoption path.
const SIDECAR_ENV_ALLOWLIST: &[&str] = &["HOME", "PATH", "RUST_LOG", "TMPDIR"];

/// Start the server on `cell` and wait for it to publish its socket.
///
/// Both `SEA_FORGE_ROOT` and `SEA_FORGE_SOCKET` are passed explicitly: the
/// child's own defaults are CWD-relative and this process's working directory
/// is whatever the desktop launcher happened to use.
fn spawn_server(binary: &Path, cell: &Cell) -> Result<Child, (String, String)> {
    if let Err(e) = std::fs::create_dir_all(&cell.root) {
        return Err((
            "cell_root_unwritable".into(),
            format!("cannot create cell root {}: {e}", cell.root.display()),
        ));
    }

    let log_path = cell.root.join(SERVER_LOG_NAME);
    let log = std::fs::File::create(&log_path).map_err(|e| {
        (
            "cell_root_unwritable".into(),
            format!("cannot write {}: {e}", log_path.display()),
        )
    })?;

    // F-25.b: start from a cleared environment and pass only the allowlist
    // above plus the two variables that name this cell. `Command::env` only
    // ever adds, so without the clear the sidecar would inherit everything the
    // desktop session carries — credential material included.
    let mut command = Command::new(binary);
    command.env_clear();
    for name in SIDECAR_ENV_ALLOWLIST {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    let mut child = command
        .env("SEA_FORGE_ROOT", &cell.root)
        .env("SEA_FORGE_SOCKET", &cell.socket)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(log))
        .spawn()
        .map_err(|e| {
            (
                "server_spawn_failed".into(),
                format!("cannot start {}: {e}", binary.display()),
            )
        })?;

    let deadline = Instant::now() + SPAWN_DEADLINE;
    loop {
        if is_listening(&cell.socket) {
            return Ok(child);
        }
        // An exited child will never publish a socket, so stop waiting for it
        // and report why it left. The server refuses to start on a bad config,
        // an unbindable socket path, or an unrunnable notify command, and each
        // of those refusals names its own remedy on stderr.
        match child.try_wait() {
            Ok(Some(status)) => {
                let reason = tail_of(&log_path);
                return Err((
                    "server_start_refused".into(),
                    format!("the cell refused to start ({status}): {reason}"),
                ));
            }
            Ok(None) => {}
            Err(e) => {
                let _ = child.kill();
                return Err((
                    "server_spawn_failed".into(),
                    format!("lost track of the server process: {e}"),
                ));
            }
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err((
                "server_start_timeout".into(),
                format!(
                    "the server did not publish {} within {}s: {}",
                    cell.socket.display(),
                    SPAWN_DEADLINE.as_secs(),
                    tail_of(&log_path)
                ),
            ));
        }
        std::thread::sleep(PROBE_INTERVAL);
    }
}

/// The last few hundred bytes of the server's stderr, so a failure can name its
/// own remedy instead of merely reporting that something went wrong.
fn tail_of(log_path: &Path) -> String {
    const LIMIT: usize = 600;
    let mut buffer = String::new();
    if std::fs::File::open(log_path)
        .and_then(|mut f| f.read_to_string(&mut buffer))
        .is_err()
    {
        return "no output was captured".into();
    }
    let trimmed = buffer.trim();
    if trimmed.is_empty() {
        return "the server exited without explanation".into();
    }
    let start = trimmed.len().saturating_sub(LIMIT);
    // Slice on a char boundary; server output is UTF-8 but need not be ASCII.
    let start = (start..trimmed.len())
        .find(|i| trimmed.is_char_boundary(*i))
        .unwrap_or(0);
    trimmed[start..].replace('\n', " | ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_override_moves_the_socket_but_not_the_records() {
        // This is the remedy the server prints for an over-long cell root, so
        // following it must not silently relocate the ledgers.
        let cell = resolve_cell_from(
            Some(PathBuf::from("/run/user/1000/sea-forge.sock")),
            Some(PathBuf::from("/tmp/cell")),
            Some(PathBuf::from("/home/op")),
        );
        assert_eq!(cell.socket, PathBuf::from("/run/user/1000/sea-forge.sock"));
        assert_eq!(cell.root, PathBuf::from("/tmp/cell"));
    }

    #[test]
    fn cell_root_composes_the_same_socket_as_the_server() {
        // Mirrors `ServerConfig::relative_socket_path_composes_under_root`:
        // SEA_FORGE_ROOT=/tmp/cell must name /tmp/cell/server.sock on both
        // sides of the transport, or the host cannot reach its server.
        let cell = resolve_cell_from(
            None,
            Some(PathBuf::from("/tmp/cell")),
            Some(PathBuf::from("/home/op")),
        );
        assert_eq!(cell.root, PathBuf::from("/tmp/cell"));
        assert_eq!(cell.socket, PathBuf::from("/tmp/cell/server.sock"));
    }

    #[test]
    fn falls_back_to_a_home_anchored_cell() {
        let cell = resolve_cell_from(None, None, Some(PathBuf::from("/home/op")));
        assert_eq!(cell.root, PathBuf::from("/home/op/.sea-forge"));
        assert_eq!(
            cell.socket,
            PathBuf::from("/home/op/.sea-forge/server.sock")
        );
    }

    #[test]
    fn a_socket_only_override_still_keeps_records_in_the_default_cell() {
        let cell = resolve_cell_from(
            Some(PathBuf::from("/tmp/s.sock")),
            None,
            Some("/home/op".into()),
        );
        assert_eq!(cell.root, PathBuf::from("/home/op/.sea-forge"));
        assert_eq!(cell.socket, PathBuf::from("/tmp/s.sock"));
    }

    #[test]
    fn the_packaged_sidecar_beside_the_app_is_found() {
        let dir = tempfile::tempdir().unwrap();
        let sidecar = dir.path().join(SERVER_BIN_NAME);
        std::fs::write(&sidecar, b"#!/bin/true").unwrap();

        let found = resolve_server_binary_from(None, Some(dir.path().to_path_buf())).unwrap();
        assert_eq!(found, sidecar);
    }

    /// With no override and no sibling, resolution must fail closed even when
    /// something named like the kernel exists elsewhere on the system. A
    /// `PATH` search used to pick exactly such a stranger up.
    #[test]
    fn without_a_sibling_or_override_resolution_fails_closed() {
        let elsewhere = tempfile::tempdir().unwrap();
        std::fs::write(elsewhere.path().join(SERVER_BIN_NAME), b"#!/bin/true").unwrap();

        let empty = tempfile::tempdir().unwrap();
        let error = resolve_server_binary_from(None, Some(empty.path().to_path_buf())).unwrap_err();
        assert!(error.contains("SEA_FORGE_SERVER_BIN"), "{error}");
    }

    /// An override naming a missing file must not fall through to some other
    /// binary. Starting a different server than the operator named is worse
    /// than not starting one.
    #[test]
    fn a_broken_override_refuses_rather_than_falling_through() {
        let error =
            resolve_server_binary_from(Some(PathBuf::from("/nonexistent/sea-forge-server")), None)
                .unwrap_err();
        assert!(error.contains("SEA_FORGE_SERVER_BIN"), "{error}");
    }

    #[test]
    fn nothing_anywhere_names_the_remedy() {
        let empty = tempfile::tempdir().unwrap();
        let error = resolve_server_binary_from(None, Some(empty.path().to_path_buf())).unwrap_err();
        assert!(error.contains("SEA_FORGE_SERVER_BIN"), "{error}");
    }

    #[test]
    fn a_stale_socket_file_is_not_mistaken_for_a_live_server() {
        // The server replaces a stale socket atomically on bind, so "a file is
        // here" must never be read as "a server is here" — otherwise the
        // Workbench would adopt a corpse and never start a kernel.
        let dir = tempfile::tempdir().unwrap();
        let stale = dir.path().join(SOCKET_FILE_NAME);
        std::fs::write(&stale, b"not a socket").unwrap();
        assert!(!is_listening(&stale));
    }

    #[test]
    fn a_listening_socket_is_detected() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join(SOCKET_FILE_NAME);
        let _listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
        assert!(is_listening(&socket));
    }

    /// The supervisor must attach to a server it did not start, and must not
    /// hold a child it could later signal.
    #[test]
    fn an_already_listening_cell_is_adopted_not_restarted() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join(SOCKET_FILE_NAME);
        let _listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();

        let supervisor = CellSupervisor::start_with(
            Cell {
                root: dir.path().to_path_buf(),
                socket,
            },
            // A binary is available, and must still go unused: adoption is
            // decided by the probe, never by whether a spawn was possible.
            Ok(PathBuf::from("/bin/false")),
        );
        assert_eq!(supervisor.supervision(), Supervision::Adopted);
        assert!(supervisor.state.lock().unwrap().child.is_none());
    }

    /// A missing binary must leave the app usable and say what to do, not
    /// panic and not silently look like a connection failure.
    #[test]
    fn an_unstartable_cell_reports_why_instead_of_panicking() {
        let dir = tempfile::tempdir().unwrap();
        let supervisor = CellSupervisor::start_with(
            Cell {
                root: dir.path().to_path_buf(),
                socket: dir.path().join(SOCKET_FILE_NAME),
            },
            resolve_server_binary_from(None, None),
        );
        match supervisor.initialize().unwrap() {
            Supervision::Unavailable {
                error_class,
                message,
            } => {
                assert_eq!(error_class, "server_binary_not_found");
                assert!(message.contains("SEA_FORGE_SERVER_BIN"), "{message}");
            }
            other => panic!("expected Unavailable, got {other:?}"),
        }
    }

    /// A missing root is a user choice, not an implicit side effect of opening
    /// the desktop application. The host must wait for the explicit Initialize
    /// action before it creates a record directory or starts a server there.
    #[test]
    fn a_missing_cell_root_waits_for_explicit_initialization_without_writing() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("new-cell");
        let supervisor = CellSupervisor::start_with(
            Cell {
                socket: root.join(SOCKET_FILE_NAME),
                root: root.clone(),
            },
            Ok(PathBuf::from("/bin/false")),
        );

        assert!(matches!(
            supervisor.supervision(),
            Supervision::InitializationRequired
        ));
        assert!(
            !root.exists(),
            "opening a new cell must not create its root before confirmation"
        );
    }

    /// The blank-root check happens again at the commit point. A history that
    /// appears while the operator is reading the confirmation must survive
    /// untouched, and the supervisor must not start a server against it.
    #[test]
    fn initialization_refuses_if_history_appears_after_the_fresh_root_check() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("new-cell");
        let supervisor = CellSupervisor::start_with(
            Cell {
                socket: root.join(SOCKET_FILE_NAME),
                root: root.clone(),
            },
            Ok(PathBuf::from("/bin/false")),
        );

        std::fs::create_dir_all(&root).unwrap();
        let history = root.join("history.jsonl");
        std::fs::write(&history, "committed history\n").unwrap();

        let error = supervisor.initialize().unwrap_err();
        assert!(error.contains("refuse to initialize"), "{error}");
        assert_eq!(
            std::fs::read_to_string(history).unwrap(),
            "committed history\n"
        );
        assert!(!root.join(SOCKET_FILE_NAME).exists());
    }

    /// A non-empty directory is not automatically a SEA Forge cell. An unknown
    /// persisted self-model schema must be refused before the sidecar gets a
    /// chance to write any new ledgers or views into it.
    #[test]
    fn an_incompatible_cell_history_is_refused_before_server_start() {
        let root = tempfile::tempdir().unwrap();
        let manifest = root.path().join(".sea-forge/self-model/manifest.json");
        std::fs::create_dir_all(manifest.parent().unwrap()).unwrap();
        std::fs::write(
            &manifest,
            r#"{
                "schema_version": "self_model.v999",
                "release_id": "future-release",
                "kernel_version": "999.0.0",
                "current_snapshot_id": null,
                "initialized_at": "2026-08-03T00:00:00Z"
            }"#,
        )
        .unwrap();

        let socket = root.path().join(SOCKET_FILE_NAME);
        let supervisor = CellSupervisor::start_with(
            Cell {
                root: root.path().to_path_buf(),
                socket: socket.clone(),
            },
            Ok(PathBuf::from("/bin/false")),
        );

        match supervisor.supervision() {
            Supervision::Unavailable {
                error_class,
                message,
            } => {
                assert_eq!(error_class, "cell_history_incompatible");
                assert!(message.contains("self_model.v999"), "{message}");
            }
            other => panic!("expected incompatible-history refusal, got {other:?}"),
        }
        assert!(
            !socket.exists(),
            "a refused history must not start a server"
        );
    }

    /// A binary that starts and immediately exits must be reported as a refusal
    /// carrying the server's own words — not as a generic timeout, and not
    /// after burning the full deadline.
    #[test]
    fn a_server_that_refuses_to_start_reports_its_own_reason() {
        let dir = tempfile::tempdir().unwrap();
        let fake = dir.path().join(SERVER_BIN_NAME);
        std::fs::write(
            &fake,
            "#!/bin/sh\necho 'error_class: server_config_error: agent endpoint unreachable' >&2\nexit 1\n",
        )
        .unwrap();
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let root = tempfile::tempdir().unwrap();
        let started = Instant::now();
        let result = spawn_server(
            &fake,
            &Cell {
                root: root.path().to_path_buf(),
                socket: root.path().join(SOCKET_FILE_NAME),
            },
        );
        let (error_class, message) = result.expect_err("a server that exits cannot be running");
        assert_eq!(error_class, "server_start_refused");
        assert!(message.contains("server_config_error"), "{message}");
        assert!(
            started.elapsed() < SPAWN_DEADLINE,
            "an exited child must be noticed, not waited out"
        );
    }

    /// The whole point of the sidecar: a confirmed fresh cell with nothing
    /// listening ends up with a live socket, and the supervisor owns the process
    /// that published it.
    #[test]
    fn a_cold_cell_gets_a_supervised_server() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join(SOCKET_FILE_NAME);
        let fake = dir.path().join(SERVER_BIN_NAME);
        // A stand-in that does what the real server does at the point this
        // module cares about: publish the socket named by SEA_FORGE_SOCKET and
        // keep accepting. Using the real binary here would make a unit test
        // depend on a cross-workspace build; `tests/packaged_stack.rs` drives
        // the real one.
        // `-k` keeps it listening after the probe's connection closes; without
        // it the socket vanishes between `start_with` and the assertion below.
        std::fs::write(
            &fake,
            "#!/bin/sh\nexec nc -klU \"$SEA_FORGE_SOCKET\" >/dev/null 2>&1\n",
        )
        .unwrap();
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        if Command::new("sh")
            .arg("-c")
            .arg("command -v nc")
            .stdout(Stdio::null())
            .status()
            .map(|s| !s.success())
            .unwrap_or(true)
        {
            eprintln!("skipping: no `nc` to stand in for a listening server");
            return;
        }

        let supervisor = CellSupervisor::start_with(
            Cell {
                root: dir.path().to_path_buf(),
                socket: socket.clone(),
            },
            Ok(fake),
        );
        match supervisor.initialize().unwrap() {
            Supervision::Supervised { pid } => assert!(pid > 0),
            other => panic!("expected Supervised after confirmation, got {other:?}"),
        }
        assert!(is_listening(&socket));

        supervisor.shutdown();
        // Idempotent: the Drop impl runs after an explicit shutdown.
        supervisor.shutdown();
    }
}
