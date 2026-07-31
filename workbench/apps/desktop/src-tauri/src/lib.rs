//! SEA Forge Workbench desktop host (Tauri 2).
//!
//! Task 3 (host half): the host owns the SFWP transport. It connects to the
//! server's Unix socket, exposes a closed typed command surface to the renderer
//! (`bridge`), and runs a durable reconnect + gap-recovery event loop
//! (`events`) that forwards `EventFrame`s to the frontend. See
//! `docs/decisions/ADR-004-workbench-stack.md` for the workspace boundary.

pub mod bridge;
pub mod drafts;
pub mod events;
pub mod socket;

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::Value;
use tauri::{Emitter, Manager};

use crate::events::{EventCursor, EventEmitter};
use crate::socket::SocketHandle;

/// The SFWP protocol major version the host negotiates.
const SFWP_PROTOCOL_VERSION: &str = "1";

/// Emits event frames to the frontend over the Tauri event bus. The payload is
/// the raw `EventFrame` (see `@sea-forge/contracts`), so renderer-side consumers
/// validate it against the generated AJV validator directly.
struct AppEmitter {
    app: tauri::AppHandle,
}

impl EventEmitter for AppEmitter {
    fn emit_event(&self, frame: &Value) {
        if let Err(error) = self.app.emit("sfwp://event", frame) {
            log::warn!("failed to emit sfwp event: {error}");
        }
    }
}

/// The socket file name inside a cell root. Must match
/// `sea_forge_server::config::SOCKET_FILE_NAME` — the desktop host is a
/// separate Cargo workspace (ADR-004 / K-06), so the constant is duplicated
/// rather than imported, and `resolve_socket_path_from` is tested against the
/// same cases as the server's `resolved_socket_path`.
const SOCKET_FILE_NAME: &str = "server.sock";

/// The conventional cell root directory name.
const DEFAULT_ROOT_DIR: &str = ".sea-forge";

/// Resolve the server socket path using the one cell contract shared by the
/// server, the CLI, and this host (see `docs/CELL_CONTRACT.md`):
///
/// 1. `SEA_FORGE_SOCKET` — explicit socket override, wins outright.
/// 2. `SEA_FORGE_ROOT` — the cell root; the socket is `<root>/server.sock`.
/// 3. Neither set — `$HOME/.sea-forge/server.sock`.
///
/// Step 3 differs from the server's CWD-relative default *on purpose*: a
/// windowed application has no meaningful working directory, so anchoring it at
/// `$HOME` is the only default that names a stable cell. A packaged install
/// sets `SEA_FORGE_ROOT` for both surfaces, which is why steps 1 and 2 are the
/// documented procedure and step 3 is a convenience for a home-directory cell.
fn resolve_socket_path() -> PathBuf {
    resolve_socket_path_from(
        std::env::var_os("SEA_FORGE_SOCKET").map(PathBuf::from),
        std::env::var_os("SEA_FORGE_ROOT").map(PathBuf::from),
        dirs_home(),
    )
}

/// Pure core of [`resolve_socket_path`], so the contract is testable without
/// mutating process environment.
fn resolve_socket_path_from(
    socket_override: Option<PathBuf>,
    root: Option<PathBuf>,
    home: Option<PathBuf>,
) -> PathBuf {
    if let Some(explicit) = socket_override {
        return explicit;
    }
    let base = root.unwrap_or_else(|| {
        home.unwrap_or_else(|| PathBuf::from("."))
            .join(DEFAULT_ROOT_DIR)
    });
    base.join(SOCKET_FILE_NAME)
}

/// Best-effort home-dir resolution without pulling in an extra crate.
fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Event sink: the socket read loop pushes pushed EventFrames onto
            // this channel; the event loop drains it.
            let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<Value>();

            let socket_path = resolve_socket_path();
            let handle = Arc::new(SocketHandle::new(socket_path, event_tx));

            // Durable cursor lives under the app data dir so it survives restart.
            let cursor_path = app
                .path()
                .app_data_dir()
                .map(|dir| dir.join("sfwp-cursor.json"))
                .unwrap_or_else(|_| PathBuf::from("sfwp-cursor.json"));
            let cursor = Arc::new(EventCursor::load(cursor_path));

            let emitter = Arc::new(AppEmitter {
                app: app.handle().clone(),
            });

            // Tauri commands call through the same shared handle
            // (`State<'_, Arc<SocketHandle>>`); the event loop takes its own
            // clone of the same Arc, so both paths share one connection.
            app.manage(Arc::clone(&handle));

            // Start the reconnect/gap-recovery/live-subscribe loop off the main
            // thread using Tauri's own tokio runtime (no second runtime).
            tauri::async_runtime::spawn(events::run_event_loop(
                handle,
                cursor,
                emitter,
                event_rx,
                SFWP_PROTOCOL_VERSION.to_string(),
            ));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bridge::sfwp_query,
            bridge::sfwp_command,
            bridge::sfwp_identity,
            bridge::sfwp_cell,
            bridge::sfwp_request_status,
            bridge::draft_save,
            bridge::draft_load,
            bridge::draft_list,
            bridge::draft_delete,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_override_wins_outright() {
        let resolved = resolve_socket_path_from(
            Some(PathBuf::from("/run/user/1000/sea-forge.sock")),
            Some(PathBuf::from("/tmp/cell")),
            Some(PathBuf::from("/home/op")),
        );
        assert_eq!(resolved, PathBuf::from("/run/user/1000/sea-forge.sock"));
    }

    #[test]
    fn cell_root_composes_the_same_socket_as_the_server() {
        // Mirrors `ServerConfig::relative_socket_path_composes_under_root`:
        // SEA_FORGE_ROOT=/tmp/cell must name /tmp/cell/server.sock on both
        // sides of the transport, or the host cannot reach its server.
        let resolved = resolve_socket_path_from(
            None,
            Some(PathBuf::from("/tmp/cell")),
            Some(PathBuf::from("/home/op")),
        );
        assert_eq!(resolved, PathBuf::from("/tmp/cell/server.sock"));
    }

    #[test]
    fn falls_back_to_a_home_anchored_cell() {
        let resolved = resolve_socket_path_from(None, None, Some(PathBuf::from("/home/op")));
        assert_eq!(resolved, PathBuf::from("/home/op/.sea-forge/server.sock"));
    }
}
