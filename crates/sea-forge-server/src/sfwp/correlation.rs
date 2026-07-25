//! SFWP request-correlation store (Task 3).
//!
//! Durable enough to answer `request.get_status` across a client reconnect:
//! one small JSON file per `request_id` under `<root>/requests/`, written
//! atomically (tmp + rename — the repo has no shared atomic-write helper, so
//! this is the minimal local one). A protected handler records `pending`
//! before doing the work and `completed`/`failed` (with the outcome value)
//! after, so a client that disconnects mid-request can reconnect and recover
//! the terminal outcome without re-issuing the command (retry != replay).

use schemars::JsonSchema;
use sea_forge_core::errors::ForgeError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Lifecycle status of a correlated request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    Pending,
    Completed,
    Failed,
}

/// The persisted correlation record for one `request_id`.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RequestRecord {
    pub request_id: String,
    pub method: String,
    pub status: RequestStatus,
    pub submitted_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// The original response value once the request reached a terminal state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<serde_json::Value>,
}

/// File-backed correlation store rooted at `<root>/requests/`.
#[derive(Clone)]
pub struct RequestCorrelationStore {
    dir: PathBuf,
}

impl RequestCorrelationStore {
    /// Open (creating the directory if needed) the store under `root`.
    pub fn open(root: &Path) -> Result<Self, ForgeError> {
        let dir = root.join("requests");
        std::fs::create_dir_all(&dir)
            .map_err(|error| ForgeError::io("create requests dir", error))?;
        Ok(Self { dir })
    }

    fn path_for(&self, request_id: &str) -> PathBuf {
        // Filesystem-safe: request ids are server/CLI generated tokens; guard
        // against separators just in case a client supplies its own.
        let safe: String = request_id
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        self.dir.join(format!("{safe}.json"))
    }

    fn write_atomic(&self, record: &RequestRecord) -> Result<(), ForgeError> {
        let path = self.path_for(&record.request_id);
        let tmp = path.with_extension("json.tmp");
        let bytes = serde_json::to_vec_pretty(record)?;
        std::fs::write(&tmp, &bytes)
            .map_err(|error| ForgeError::io("write request correlation tmp", error))?;
        std::fs::rename(&tmp, &path)
            .map_err(|error| ForgeError::io("rename request correlation", error))?;
        Ok(())
    }

    /// Record that a request has started (`pending`). Overwrites any prior
    /// record for the id (a fresh episode).
    pub fn record_pending(&self, request_id: &str, method: &str) -> Result<(), ForgeError> {
        let record = RequestRecord {
            request_id: request_id.to_string(),
            method: method.to_string(),
            status: RequestStatus::Pending,
            submitted_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            outcome: None,
        };
        self.write_atomic(&record)
    }

    /// Record a terminal outcome. `failed` is chosen when the outcome value
    /// carries an `error` field, otherwise `completed`.
    pub fn record_outcome(
        &self,
        request_id: &str,
        method: &str,
        outcome: &serde_json::Value,
    ) -> Result<(), ForgeError> {
        let status = if outcome.get("error").is_some() {
            RequestStatus::Failed
        } else {
            RequestStatus::Completed
        };
        // Preserve the original submitted_at if a pending record exists.
        let submitted_at = self
            .get(request_id)?
            .map(|record| record.submitted_at)
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
        let record = RequestRecord {
            request_id: request_id.to_string(),
            method: method.to_string(),
            status,
            submitted_at,
            completed_at: Some(chrono::Utc::now().to_rfc3339()),
            outcome: Some(outcome.clone()),
        };
        self.write_atomic(&record)
    }

    /// Look up a correlation record; `Ok(None)` if the id is unknown.
    pub fn get(&self, request_id: &str) -> Result<Option<RequestRecord>, ForgeError> {
        let path = self.path_for(request_id);
        match std::fs::read(&path) {
            Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(ForgeError::io("read request correlation", error)),
        }
    }
}
