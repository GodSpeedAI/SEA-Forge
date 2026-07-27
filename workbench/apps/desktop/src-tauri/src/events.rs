//! Durable event cursor, reconnect, and gap recovery (Task 3, host half).
//!
//! The host owns the durable event stream so the renderer stays inside the
//! typed bridge. This module:
//!   * persists `last_cursor` to a small JSON file under the app data dir, so a
//!     restart (not merely a reconnect) resumes from the right position;
//!   * on every (re)connect, drains `events.get_range` from the persisted cursor
//!     until an empty page proves the backlog is exhausted — the deterministic
//!     gap-recovery path, independent of `events.subscribe`'s own bounded
//!     catch-up burst;
//!   * then `events.subscribe`s and forwards every pushed `EventFrame` to the
//!     frontend via `app.emit("sfwp://event", frame)`, persisting the cursor
//!     after every event (catch-up AND live) so no progress is lost on a crash;
//!   * on any read/connect error, backs off and reconnects from the top, which
//!     re-derives any gap via the persisted cursor.
//!
//! The emitted payload is the raw `EventFrame` shape (see the generated TS type
//! in `@sea-forge/contracts`), so Task 4+ consumers validate it directly.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::sync::{mpsc, Mutex};

use crate::socket::SocketHandle;

/// Fixed reconnect backoff. This is a reconnect loop, not a scheduler — a small
/// constant delay is sufficient and keeps the behavior legible/testable.
const RECONNECT_BACKOFF: Duration = Duration::from_millis(500);

/// Durable event cursor persisted across restarts.
pub struct EventCursor {
    path: PathBuf,
    last: Mutex<Option<String>>,
}

impl EventCursor {
    /// Load the persisted cursor (if any) from `path`. A missing/corrupt file is
    /// treated as "no cursor yet" — the first run then replays from the start.
    pub fn load(path: PathBuf) -> Self {
        let last = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<CursorFile>(&s).ok())
            .and_then(|f| f.last_cursor);
        Self {
            path,
            last: Mutex::new(last),
        }
    }

    pub async fn get(&self) -> Option<String> {
        self.last.lock().await.clone()
    }

    /// Advance and durably persist the cursor. Persistence is best-effort: a
    /// write failure is logged, not fatal (the next successful write recovers).
    pub async fn set(&self, cursor: String) {
        let mut guard = self.last.lock().await;
        *guard = Some(cursor.clone());
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let body = serde_json::to_vec(&CursorFile {
            last_cursor: Some(cursor),
        })
        .unwrap_or_default();
        if let Err(error) = std::fs::write(&self.path, body) {
            log::warn!("failed to persist event cursor: {error}");
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CursorFile {
    last_cursor: Option<String>,
}

/// A sink the loop forwards each `EventFrame` to. In production this wraps
/// `app.emit`; in tests it can be any channel, so the loop is exercisable
/// without a full Tauri `AppHandle`.
pub trait EventEmitter: Send + Sync + 'static {
    fn emit_event(&self, frame: &Value);
}

/// Run the reconnect + gap-recovery + live-subscribe loop forever. Spawn this
/// once from Tauri `setup` via `tauri::async_runtime::spawn`. `event_rx` is the
/// receiver end of the sink passed to `SocketHandle`/`SocketClient`; the read
/// loop pushes pushed event frames onto it, and this loop drains them.
pub async fn run_event_loop<E: EventEmitter>(
    handle: Arc<SocketHandle>,
    cursor: Arc<EventCursor>,
    emitter: Arc<E>,
    mut event_rx: mpsc::UnboundedReceiver<Value>,
    protocol_version: String,
) {
    loop {
        match connect_and_pump(&handle, &cursor, &emitter, &mut event_rx, &protocol_version)
            .await
        {
            Ok(()) => { /* pump returns only on disconnect */ }
            Err(error) => {
                log::warn!("sfwp event loop error: {error}; backing off");
            }
        }
        tokio::time::sleep(RECONNECT_BACKOFF).await;
    }
}

/// One connection lifecycle: reconnect, hello, drain backlog, subscribe, then
/// forward live events until the connection drops.
async fn connect_and_pump<E: EventEmitter>(
    handle: &Arc<SocketHandle>,
    cursor: &Arc<EventCursor>,
    emitter: &Arc<E>,
    event_rx: &mut mpsc::UnboundedReceiver<Value>,
    protocol_version: &str,
) -> Result<(), String> {
    // Drain any events buffered from a previous (now-dead) connection so a
    // stale frame can't be mistaken for a live one after reconnect.
    while event_rx.try_recv().is_ok() {}

    let client = handle.reconnect().await.map_err(|e| e.to_string())?;

    // Negotiate.
    let hello = client
        .call(json!({"verb": "system_hello", "protocol_version": protocol_version}))
        .await
        .map_err(|e| e.to_string())?;
    if hello.get("error").is_some() {
        return Err(format!("hello rejected: {hello}"));
    }

    // Deterministic gap recovery: drain get_range from the persisted cursor
    // until an empty page proves the backlog is exhausted.
    catch_up(&client, cursor, emitter).await?;

    // Subscribe from the (now-advanced) cursor and forward live frames.
    let from_cursor = cursor.get().await;
    let sub = client
        .call(json!({"verb": "events_subscribe", "from_cursor": from_cursor}))
        .await
        .map_err(|e| e.to_string())?;
    if sub.get("error").is_some() {
        return Err(format!("subscribe rejected: {sub}"));
    }

    // Forward pushed frames until the connection drops. The pushed-event
    // channel's sender lives on the shared `SocketHandle` and never closes, so
    // we can't rely on `recv()` returning `None` to detect death — instead we
    // race each `recv()` against the client's explicit disconnect signal.
    loop {
        tokio::select! {
            biased;
            _ = client.wait_disconnected() => {
                // Connection dropped: return so the outer loop backs off and
                // reconnects, re-deriving any gap via the persisted cursor.
                return Ok(());
            }
            maybe_frame = event_rx.recv() => {
                match maybe_frame {
                    Some(frame) => forward(cursor, emitter, &frame).await,
                    None => return Ok(()), // all senders gone -> shutting down
                }
            }
        }
    }
}

/// Drain `events.get_range` pages from the persisted cursor until exhausted,
/// emitting and persisting each recovered event in order.
///
/// Termination is on an **empty** page, not a short one. `get_range` reads
/// strictly *after* the cursor and clamps any requested `limit` to its own cap
/// (`sfwp::events::get_range`), so an empty page is the only signal that means
/// "exhausted" without encoding an assumption about that cap here. A short-page
/// rule would need the host to know the server's page size: guess too high and
/// the drain stops early, silently dropping the rest of the backlog — the one
/// failure this loop exists to prevent. One extra round trip per reconnect buys
/// immunity to that drift.
async fn catch_up<E: EventEmitter>(
    client: &crate::socket::SocketClient,
    cursor: &Arc<EventCursor>,
    emitter: &Arc<E>,
) -> Result<(), String> {
    loop {
        let from_cursor = cursor.get().await;
        let page = client
            .call(json!({
                "verb": "events_get_range",
                "from_cursor": from_cursor,
            }))
            .await
            .map_err(|e| e.to_string())?;
        if page.get("error").is_some() {
            return Err(format!("get_range failed: {page}"));
        }
        let events = page
            .get("events")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if events.is_empty() {
            return Ok(());
        }
        for frame in &events {
            forward(cursor, emitter, frame).await;
        }
        // The next page is read strictly after the cursor, so a non-empty page
        // that failed to advance it would re-read forever. That only happens if
        // frames arrive without a `cursor`, which is contract drift — report it
        // rather than spin, because a hung reconnect is harder to diagnose than
        // a named error.
        if cursor.get().await == from_cursor {
            return Err(format!(
                "get_range returned {} event(s) but none advanced the cursor",
                events.len()
            ));
        }
    }
}

/// Emit a frame to the frontend and durably advance the cursor to it.
async fn forward<E: EventEmitter>(cursor: &Arc<EventCursor>, emitter: &Arc<E>, frame: &Value) {
    emitter.emit_event(frame);
    if let Some(c) = frame.get("cursor").and_then(Value::as_str) {
        cursor.set(c.to_string()).await;
    }
}
