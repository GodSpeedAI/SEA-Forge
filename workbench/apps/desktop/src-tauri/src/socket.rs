//! SFWP NDJSON socket client (Task 3, host half).
//!
//! One Unix-socket connection carries two kinds of inbound lines (see the
//! server's `handle_connection`):
//!   * responses — one plain-JSON line per request, in FIFO send order;
//!   * events    — unsolicited `{"type":"event","event":<EventFrame>}` lines,
//!     pushed only after this connection issued `events_subscribe`, interleaved
//!     between responses but never reordering them.
//!
//! A line is an event iff `value["type"] == "event"`; everything else is the
//! response to this connection's oldest still-unanswered request. That single
//! discriminator is what lets one connection multiplex both streams without any
//! extra framing.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::unix::OwnedWriteHalf;
use tokio::net::UnixStream;
use tokio::sync::{mpsc, oneshot, Mutex};

/// Errors surfaced by the socket client's request path.
#[derive(Debug)]
pub enum SocketError {
    /// The transport failed while connecting.
    Connect(String),
    /// Serializing the outbound request line failed.
    Encode(String),
    /// The write half failed while sending a request line.
    Write(String),
    /// The connection dropped (EOF / read error) before this request's
    /// response arrived. The caller should reconnect and, if the request was
    /// correlated, recover its terminal outcome via `request.get_status`.
    Disconnected,
}

impl std::fmt::Display for SocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SocketError::Connect(e) => write!(f, "socket connect failed: {e}"),
            SocketError::Encode(e) => write!(f, "request encode failed: {e}"),
            SocketError::Write(e) => write!(f, "socket write failed: {e}"),
            SocketError::Disconnected => {
                write!(f, "connection dropped before response arrived")
            }
        }
    }
}

impl std::error::Error for SocketError {}

/// A pushed event frame handed to the connection's event sink.
pub type EventSink = mpsc::UnboundedSender<Value>;

/// A single live SFWP connection. Cloneable handle: all clones share one
/// underlying stream, write path, and pending-response queue.
#[derive(Clone)]
pub struct SocketClient {
    inner: Arc<Inner>,
}

struct Inner {
    /// Guards the write half AND the pending-response queue so that
    /// "write the request line, then register where its response goes" is one
    /// atomic critical section — concurrent callers cannot interleave a write
    /// between another caller's write and its queue push, which would corrupt
    /// FIFO response pairing.
    write: Mutex<WriteState>,
    /// Notified exactly once when the read loop ends (EOF/error). Lets the
    /// event/reconnect loop wake immediately on disconnect instead of blocking
    /// on the pushed-event channel (whose sender lives on past the socket).
    disconnected: tokio::sync::Notify,
}

struct WriteState {
    writer: OwnedWriteHalf,
    /// One `oneshot::Sender` per still-unanswered request, in send order. The
    /// read loop pops the front for each non-event line it reads.
    pending: VecDeque<oneshot::Sender<Value>>,
    /// Set once the read loop has ended (EOF/error). No further requests can be
    /// paired, so `call` fails fast instead of hanging forever.
    closed: bool,
}

impl SocketClient {
    /// Connect to `socket_path`, spawn the background read loop, and forward any
    /// pushed event frames to `event_sink`. The read loop runs on the ambient
    /// tokio runtime (`tokio::spawn`), which under Tauri is the same runtime as
    /// `tauri::async_runtime::spawn`.
    pub async fn connect(
        socket_path: &Path,
        event_sink: EventSink,
    ) -> Result<Self, SocketError> {
        let stream = UnixStream::connect(socket_path)
            .await
            .map_err(|e| SocketError::Connect(e.to_string()))?;
        let (read_half, write_half) = stream.into_split();

        let inner = Arc::new(Inner {
            write: Mutex::new(WriteState {
                writer: write_half,
                pending: VecDeque::new(),
                closed: false,
            }),
            disconnected: tokio::sync::Notify::new(),
        });

        // Background read loop.
        let inner_for_read = Arc::clone(&inner);
        tokio::spawn(async move {
            let mut reader = BufReader::new(read_half);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) | Err(_) => break, // EOF or read error -> disconnected.
                    Ok(_) => {}
                }
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let value: Value = match serde_json::from_str(trimmed) {
                    Ok(v) => v,
                    Err(_) => continue, // Skip unparseable lines defensively.
                };

                if value.get("type").and_then(Value::as_str) == Some("event") {
                    // Pushed event frame; forward the inner `event` payload.
                    let frame = value.get("event").cloned().unwrap_or(Value::Null);
                    let _ = event_sink.send(frame);
                    continue;
                }

                // Otherwise: response to the oldest unanswered request.
                let mut state = inner_for_read.write.lock().await;
                if let Some(tx) = state.pending.pop_front() {
                    let _ = tx.send(value);
                }
            }

            // Disconnected: fail every still-pending request so callers unblock,
            // then wake anyone awaiting the disconnect signal.
            {
                let mut state = inner_for_read.write.lock().await;
                state.closed = true;
                while let Some(tx) = state.pending.pop_front() {
                    drop(tx); // Dropping the sender resolves the receiver to `Err`.
                }
            }
            inner_for_read.disconnected.notify_waiters();
        });

        Ok(Self { inner })
    }

    /// Resolve when this client's connection has dropped (EOF/read error). Used
    /// by the reconnect loop to wake promptly rather than polling. If the socket
    /// is already closed, returns immediately.
    pub async fn wait_disconnected(&self) {
        // Register interest BEFORE checking `closed`, so a disconnect that fires
        // between the check and the await can't be missed (Notify only wakes
        // futures created before `notify_waiters`).
        let notified = self.inner.disconnected.notified();
        if self.inner.write.lock().await.closed {
            return;
        }
        notified.await;
    }

    /// The single chokepoint every command/query goes through: serialize the
    /// request, write its line, register the response slot, and await the
    /// matching response line. Atomic write+register guarantees FIFO pairing.
    pub async fn call(&self, request: Value) -> Result<Value, SocketError> {
        let line = serde_json::to_string(&request)
            .map_err(|e| SocketError::Encode(e.to_string()))?;

        let rx = {
            let mut state = self.inner.write.lock().await;
            if state.closed {
                return Err(SocketError::Disconnected);
            }
            state
                .writer
                .write_all(line.as_bytes())
                .await
                .map_err(|e| SocketError::Write(e.to_string()))?;
            state
                .writer
                .write_all(b"\n")
                .await
                .map_err(|e| SocketError::Write(e.to_string()))?;
            state
                .writer
                .flush()
                .await
                .map_err(|e| SocketError::Write(e.to_string()))?;
            let (tx, rx) = oneshot::channel();
            state.pending.push_back(tx);
            rx
        };

        // A dropped sender (read loop ended) resolves to `Err` -> Disconnected.
        rx.await.map_err(|_| SocketError::Disconnected)
    }
}

/// Shared, reconnectable handle managed by Tauri (`.manage(...)`). Holds the
/// current live client behind a mutex so the reconnect loop can swap it in
/// place while Tauri commands keep calling through the same handle.
pub struct SocketHandle {
    socket_path: PathBuf,
    current: Mutex<Option<SocketClient>>,
    event_sink: EventSink,
}

impl SocketHandle {
    pub fn new(socket_path: PathBuf, event_sink: EventSink) -> Self {
        Self {
            socket_path,
            current: Mutex::new(None),
            event_sink,
        }
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Return the current live client, connecting one if none exists yet.
    pub async fn client(&self) -> Result<SocketClient, SocketError> {
        let mut guard = self.current.lock().await;
        if let Some(client) = guard.as_ref() {
            return Ok(client.clone());
        }
        let client =
            SocketClient::connect(&self.socket_path, self.event_sink.clone()).await?;
        *guard = Some(client.clone());
        Ok(client)
    }

    /// Force a fresh connection, replacing any stale one (reconnect path).
    pub async fn reconnect(&self) -> Result<SocketClient, SocketError> {
        let client =
            SocketClient::connect(&self.socket_path, self.event_sink.clone()).await?;
        let mut guard = self.current.lock().await;
        *guard = Some(client.clone());
        Ok(client)
    }

    /// Convenience: call through the current client, transparently connecting
    /// if needed. Does NOT auto-retry on disconnect — correlation recovery is
    /// the caller's explicit responsibility via `request.get_status`.
    pub async fn call(&self, request: Value) -> Result<Value, SocketError> {
        self.client().await?.call(request).await
    }
}
