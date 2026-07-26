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

/// Resolve the server socket path. Honors `SEA_FORGE_SOCKET` when set (dev/test
/// convenience); otherwise defaults to the conventional `.sea-forge/server.sock`
/// under the user's home directory, matching `ServerConfig`'s default layout.
fn resolve_socket_path() -> PathBuf {
    if let Ok(explicit) = std::env::var("SEA_FORGE_SOCKET") {
        return PathBuf::from(explicit);
    }
    let base = dirs_home().unwrap_or_else(|| PathBuf::from("."));
    base.join(".sea-forge").join("server.sock")
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

            let emitter = Arc::new(AppEmitter { app: app.handle().clone() });

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
            bridge::sfwp_request_status,
            bridge::draft_save,
            bridge::draft_load,
            bridge::draft_list,
            bridge::draft_delete,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
