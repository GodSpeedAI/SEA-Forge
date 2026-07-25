//! SFWP durable event stream (Task 3).
//!
//! Events are grounded on the ledger's `entry_ulid`: a single dedicated
//! `events` ledger is the durable, ordered source of truth, and its
//! `entry_ulid` is the opaque cursor SFWP clients carry. Live delivery rides a
//! `tokio::sync::broadcast` channel on top of that ledger — the broadcast is a
//! convenience for connected subscribers, never the source truth. On any gap
//! or doubt a client refetches deterministically via `events.get_range`, which
//! reads the ledger alone.

use schemars::JsonSchema;
use sea_forge_core::errors::ForgeError;
use sea_forge_ledger::LedgerStream;
use serde::{Deserialize, Serialize};

/// A single SFWP event frame. `cursor` is the emitting ledger entry's
/// `entry_ulid` — monotonic in append order, durable, and replay-addressable.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct EventFrame {
    /// Opaque durable cursor: the events-ledger `entry_ulid` for this event.
    pub cursor: String,
    /// Event kind, e.g. `case.submitted`, `approval.decided`.
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    /// Event-specific detail payload (already redaction-safe).
    pub detail: serde_json::Value,
    /// RFC3339 commit timestamp taken from the ledger entry.
    pub committed_at: String,
}

/// Maximum number of events replayed in a single `events.subscribe`
/// catch-up burst or returned by `events.get_range` — matches the bounded
/// philosophy of the rest of the server.
pub const EVENTS_REPLAY_CAP: usize = 500;

/// The ledger record kind used for the SFWP events stream.
pub const EVENT_RECORD_KIND: &str = "sfwp_event";

/// Open (or create) the single dedicated events ledger under the run root.
pub fn open_events_ledger(root: &std::path::Path) -> Result<LedgerStream, ForgeError> {
    LedgerStream::open(root, "events", "sea-forge-server")
}

/// The stored-on-ledger shape of an event (the `cursor` is the entry's own
/// `entry_ulid`, so it is not duplicated in the payload).
#[derive(Serialize, Deserialize)]
struct StoredEvent {
    kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    case_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    run_id: Option<String>,
    detail: serde_json::Value,
}

/// Append an event to the durable events ledger and return the built frame
/// whose `cursor` is the resulting entry's `entry_ulid`.
pub fn append_event(
    ledger: &LedgerStream,
    kind: &str,
    case_id: Option<&str>,
    run_id: Option<&str>,
    detail: serde_json::Value,
) -> Result<EventFrame, ForgeError> {
    let stored = StoredEvent {
        kind: kind.to_string(),
        case_id: case_id.map(str::to_string),
        run_id: run_id.map(str::to_string),
        detail,
    };
    let subject_refs: Vec<String> = case_id
        .into_iter()
        .chain(run_id)
        .map(str::to_string)
        .collect();
    let payload = serde_json::to_value(&stored)?;
    let entry = ledger.append(EVENT_RECORD_KIND, subject_refs, payload, vec![])?;
    Ok(EventFrame {
        cursor: entry.entry_ulid.clone(),
        kind: stored.kind,
        case_id: stored.case_id,
        run_id: stored.run_id,
        detail: stored.detail,
        committed_at: entry.committed_at,
    })
}

/// Reconstruct an [`EventFrame`] from a persisted events-ledger entry.
fn frame_from_entry(entry: &sea_forge_ledger::LedgerEntry) -> EventFrame {
    EventFrame {
        cursor: entry.entry_ulid.clone(),
        kind: entry.payload["kind"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        case_id: entry.payload["case_id"].as_str().map(str::to_string),
        run_id: entry.payload["run_id"].as_str().map(str::to_string),
        detail: entry
            .payload
            .get("detail")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        committed_at: entry.committed_at.clone(),
    }
}

/// Resolve a cursor (`entry_ulid`) to its `append_ordinal` by scanning the
/// (bounded) entry list. Returns `None` if the cursor is unknown.
fn ordinal_for_cursor(entries: &[sea_forge_ledger::LedgerEntry], cursor: &str) -> Option<u64> {
    entries
        .iter()
        .find(|entry| entry.entry_ulid == cursor)
        .map(|entry| entry.append_ordinal)
}

/// Replay events strictly *after* `from_cursor` (exclusive), capped at
/// [`EVENTS_REPLAY_CAP`]. If `from_cursor` is `None`, replays from the start.
/// Used by `events.subscribe` for its initial catch-up burst.
pub fn replay_after(
    ledger: &LedgerStream,
    from_cursor: Option<&str>,
) -> Result<Vec<EventFrame>, ForgeError> {
    let entries = ledger.read_entries()?;
    let start_ordinal = match from_cursor {
        Some(cursor) => ordinal_for_cursor(&entries, cursor),
        None => None,
    };
    let frames = entries
        .iter()
        .filter(|entry| entry.record_kind == EVENT_RECORD_KIND)
        .filter(|entry| match start_ordinal {
            Some(ord) => entry.append_ordinal > ord,
            None => true,
        })
        .take(EVENTS_REPLAY_CAP)
        .map(frame_from_entry)
        .collect();
    Ok(frames)
}

/// Deterministic bounded read from the durable events ledger only — the
/// gap-recovery path. Reads events whose `append_ordinal` is strictly greater
/// than `from_cursor`'s (if set) and less than or equal to `to_cursor`'s (if
/// set), capped by `limit` (default/maximum [`EVENTS_REPLAY_CAP`]).
pub fn get_range(
    ledger: &LedgerStream,
    from_cursor: Option<&str>,
    to_cursor: Option<&str>,
    limit: Option<u32>,
) -> Result<Vec<EventFrame>, ForgeError> {
    let entries = ledger.read_entries()?;
    let from_ordinal = from_cursor.and_then(|cursor| ordinal_for_cursor(&entries, cursor));
    let to_ordinal = to_cursor.and_then(|cursor| ordinal_for_cursor(&entries, cursor));
    let cap = limit
        .map(|value| (value as usize).min(EVENTS_REPLAY_CAP))
        .unwrap_or(EVENTS_REPLAY_CAP);
    let frames = entries
        .iter()
        .filter(|entry| entry.record_kind == EVENT_RECORD_KIND)
        .filter(|entry| match from_ordinal {
            Some(ord) => entry.append_ordinal > ord,
            None => true,
        })
        .filter(|entry| match to_ordinal {
            Some(ord) => entry.append_ordinal <= ord,
            None => true,
        })
        .take(cap)
        .map(frame_from_entry)
        .collect();
    Ok(frames)
}
