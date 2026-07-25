//! SFWP precondition / expected-digest checks (Task 3).
//!
//! Protected verbs (`approve`, `reject`) may carry a [`Precondition`] that
//! pins the caller's expected digest of the record(s) they are acting on. If
//! the record's *current* digest (sha256 of its canonical JSON serialization)
//! no longer matches, the decision is stale: the server returns a structured
//! `rejected_as_stale` response and performs **no side effect** — mirroring the
//! "no side effect on deny" discipline already used on the authority-deny
//! paths in `lib.rs`. Never silently retried; recovery is the client's job.

use schemars::JsonSchema;
use sea_forge_core::errors::ForgeError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A precondition bundle attached to a protected request.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct Precondition {
    /// Optional digest of the active policy bundle the caller assumed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_bundle_digest: Option<String>,
    /// Per-record expected digests.
    #[serde(default)]
    pub records: Vec<RecordDigest>,
}

/// A single `{ref, expected_digest}` precondition entry.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RecordDigest {
    /// Opaque record reference (e.g. `case:<case_id>`, `approval:<id>`).
    pub r#ref: String,
    /// The sha256 digest the caller expects the referenced record to have.
    pub expected_digest: String,
}

/// A record whose current digest diverged from the caller's expectation.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChangedRecord {
    pub r#ref: String,
    pub expected_digest: String,
    /// The record's actual current digest, or `null` if it no longer exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_digest: Option<String>,
}

/// The structured `rejected_as_stale` response body. No side effect has been
/// performed when this is returned.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RejectedAsStale {
    /// Always `"rejected_as_stale"`.
    pub outcome: String,
    /// Always `"precondition_failed"`.
    pub code: String,
    pub changed_records: Vec<ChangedRecord>,
    /// Repair guidance the client should follow before any retry.
    pub next_actions: Vec<String>,
}

impl RejectedAsStale {
    pub fn new(changed_records: Vec<ChangedRecord>) -> Self {
        Self {
            outcome: "rejected_as_stale".into(),
            code: "precondition_failed".into(),
            changed_records,
            next_actions: vec![
                "refetch the authoritative record view".into(),
                "recompute the decision against current state".into(),
                "resubmit as a new episode if still intended".into(),
            ],
        }
    }
}

/// Compute the sha256 digest (`sha256:<hex>`) of a record's canonical JSON.
pub fn digest_of(value: &serde_json::Value) -> Result<String, ForgeError> {
    // Canonical form: serde_json with sorted keys via a BTreeMap round-trip is
    // overkill here; the ledger already uses a canonical_json helper, but that
    // is private. For precondition digests we serialize deterministically by
    // sorting object keys ourselves.
    let canonical = canonicalize(value);
    let bytes = serde_json::to_vec(&canonical)?;
    Ok(format!("sha256:{:x}", Sha256::digest(&bytes)))
}

/// Recursively sort object keys so digests are stable across map orderings.
fn canonicalize(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut sorted: std::collections::BTreeMap<String, serde_json::Value> =
                std::collections::BTreeMap::new();
            for (key, val) in map {
                sorted.insert(key.clone(), canonicalize(val));
            }
            serde_json::to_value(sorted).unwrap_or(serde_json::Value::Null)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(canonicalize).collect())
        }
        other => other.clone(),
    }
}

/// Resolves a record reference to its current authoritative value (or `None`
/// if it does not exist), so its digest can be recomputed.
pub trait RecordResolver {
    fn resolve(&self, r#ref: &str) -> Result<Option<serde_json::Value>, ForgeError>;
}

/// Evaluate a precondition against current state. Returns `Ok(None)` when all
/// digests match (proceed with the side effect) or `Ok(Some(rejected))` when
/// any digest is stale (perform no side effect, return the structured body).
pub fn evaluate<R: RecordResolver>(
    precondition: &Precondition,
    resolver: &R,
) -> Result<Option<RejectedAsStale>, ForgeError> {
    let mut changed = Vec::new();
    for record in &precondition.records {
        let current = resolver.resolve(&record.r#ref)?;
        let current_digest = match &current {
            Some(value) => Some(digest_of(value)?),
            None => None,
        };
        if current_digest.as_deref() != Some(record.expected_digest.as_str()) {
            changed.push(ChangedRecord {
                r#ref: record.r#ref.clone(),
                expected_digest: record.expected_digest.clone(),
                current_digest,
            });
        }
    }
    if changed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(RejectedAsStale::new(changed)))
    }
}
