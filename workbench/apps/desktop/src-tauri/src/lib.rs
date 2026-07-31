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
pub mod supervisor;

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::Value;
use tauri::{Emitter, Manager};

use crate::events::{EventCursor, EventEmitter};
use crate::socket::SocketHandle;
use crate::supervisor::CellSupervisor;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
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

            // Decision U-06: this Workbench supervises its own kernel, and
            // adopts one that is already running rather than starting a second.
            // Synchronous on purpose — the window must not paint a governed
            // surface before it is settled whether there is a cell behind it.
            // A cold start costs a few hundred milliseconds; a failure is
            // bounded by `SPAWN_DEADLINE` and still opens the window, because
            // every surface already resolves its own standing from a catalog it
            // may be unable to negotiate.
            let supervisor = Arc::new(CellSupervisor::start(supervisor::resolve_cell()));
            log::info!(
                "cell {} — {:?}",
                supervisor.cell().root.display(),
                supervisor.supervision()
            );

            let handle = Arc::new(SocketHandle::new(
                supervisor.cell().socket.clone(),
                event_tx,
            ));
            app.manage(Arc::clone(&supervisor));
            supervisor::watch_for_signals(Arc::clone(&supervisor));

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
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        // Stop the kernel this process started. An adopted one is left alone —
        // `CellSupervisor::shutdown` only ever reaches a child it spawned.
        if let tauri::RunEvent::Exit = event {
            if let Some(supervisor) = app_handle.try_state::<Arc<CellSupervisor>>() {
                supervisor.shutdown();
            }
        }
    });
}
