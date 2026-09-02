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
    /// Hash of the request payload this id was first used for (SF-006).
    /// Additive and optional: records written before SF-006 have none, and a
    /// record without one is not dedupeable (see [`RequestCorrelationStore::check`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_hash: Option<String>,
}

/// What dispatch should do with a request that carries a `request_id`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DedupeVerdict {
    /// Unknown id or a legacy record with no hash — execute.
    Proceed,
    /// The same id is already lawfully admitted but has no terminal outcome.
    /// Return its locator rather than running a concurrent duplicate.
    Pending,
    /// The same id and payload already reached a terminal state; return this
    /// stored response instead of running the work a second time.
    Replay(serde_json::Value),
    /// The id is already bound to a different payload. Refuse.
    Reused,
}

/// A deterministic hash of a request payload, stable across serializations.
///
/// Keys are sorted before writing, so this does not depend on serde_json's
/// `preserve_order` feature being off — a transitive dependency turning it on
/// would otherwise silently make every dedupe key unstable.
pub fn payload_hash(payload: &serde_json::Value) -> String {
    use sha2::{Digest, Sha256};
    let mut canonical = String::new();
    canonicalize(payload, &mut canonical);
    format!("{:x}", Sha256::digest(canonical.as_bytes()))
}

/// Write `value` in canonical (sorted-key) JSON form.
///
/// ponytail: recursion matches the payload's nesting depth, which serde_json
/// has already bounded at parse time; an explicit stack would only matter if
/// requests arrived pre-parsed from somewhere without that limit.
fn canonicalize(value: &serde_json::Value, out: &mut String) {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&str> = map.keys().map(String::as_str).collect();
            keys.sort_unstable();
            out.push('{');
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                // Through serde_json so quotes and escapes match the wire form.
                out.push_str(&serde_json::Value::String((*key).to_owned()).to_string());
                out.push(':');
                canonicalize(&map[*key], out);
            }
            out.push('}');
        }
        serde_json::Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                canonicalize(item, out);
            }
            out.push(']');
        }
        scalar => out.push_str(&scalar.to_string()),
    }
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

    /// Decide whether a request bearing `request_id` may execute (SF-006).
    ///
    /// Compares the payload hash only. The verb is *inside* the hashed payload,
    /// so a changed operation is already a changed payload — comparing `method`
    /// as well would only add a second, weaker copy of the same check that
    /// disagrees with it whenever a handler records a different spelling.
    pub fn check(&self, request_id: &str, payload_hash: &str) -> Result<DedupeVerdict, ForgeError> {
        let Some(record) = self.get(request_id)? else {
            return Ok(DedupeVerdict::Proceed);
        };
        // Pre-SF-006 records normally remain usable: their old format lacks
        // the hash needed to prove a retry is the same payload. The one safe
        // exception is our restart settlement. Once it records an interrupted
        // terminal outcome, a retry must replay that failure rather than turn
        // a crash-recovery record into a duplicate governed effect.
        let Some(stored) = record.payload_hash.as_deref() else {
            if record.status == RequestStatus::Failed
                && record.outcome.as_ref().is_some_and(|outcome| {
                    outcome
                        .get("error_class")
                        .and_then(serde_json::Value::as_str)
                        == Some("request_interrupted")
                })
            {
                return Ok(record
                    .outcome
                    .map_or(DedupeVerdict::Pending, DedupeVerdict::Replay));
            }
            return Ok(DedupeVerdict::Proceed);
        };
        if stored != payload_hash {
            return Ok(DedupeVerdict::Reused);
        }
        match record.status {
            // A pending record proves a request crossed the durable boundary,
            // but not whether its process is still live. Retrying it as new
            // work would make a timed-out or restarted request execute twice;
            // preserve the safe answer instead and let request.get_status
            // expose the unresolved lifecycle.
            RequestStatus::Pending => Ok(DedupeVerdict::Pending),
            RequestStatus::Completed | RequestStatus::Failed => Ok(record
                .outcome
                .map_or(DedupeVerdict::Pending, DedupeVerdict::Replay)),
        }
    }

    /// Whether `request_id` already holds a terminal record with a response.
    ///
    /// Guards both writes below. The packet mandates that a `Pending` record
    /// lets a retry proceed, so two executions of one id are possible by
    /// design; what must not follow is the second one demoting the first back
    /// to `Pending` and blanking its stored response, which would make an
    /// outcome already reported to a client unreachable by its own id.
    ///
    /// No payload comparison is needed: a terminal record reached this far only
    /// because dispatch matched its hash, since a mismatch is refused upstream.
    fn is_settled(&self, request_id: &str) -> bool {
        self.get(request_id).ok().flatten().is_some_and(|record| {
            record.status != RequestStatus::Pending && record.outcome.is_some()
        })
    }

    /// Record that a request has started (`pending`). Overwrites any prior
    /// record for the id (a fresh episode), except a settled one for the same
    /// payload — see [`Self::is_settled`].
    ///
    /// `payload_hash` is `None` from handlers, which run downstream of the
    /// dispatch guard that already bound the id; the existing hash is carried
    /// forward so that second write cannot erase the dedupe key.
    pub fn record_pending(
        &self,
        request_id: &str,
        method: &str,
        payload_hash: Option<&str>,
    ) -> Result<(), ForgeError> {
        if self.is_settled(request_id) {
            return Ok(());
        }
        let record = RequestRecord {
            request_id: request_id.to_string(),
            method: method.to_string(),
            status: RequestStatus::Pending,
            submitted_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            outcome: None,
            payload_hash: self.carry_hash(request_id, payload_hash)?,
        };
        self.write_atomic(&record)
    }

    /// The hash to persist: the supplied one, else whatever the id already has.
    fn carry_hash(
        &self,
        request_id: &str,
        supplied: Option<&str>,
    ) -> Result<Option<String>, ForgeError> {
        if let Some(hash) = supplied {
            return Ok(Some(hash.to_owned()));
        }
        Ok(self.get(request_id)?.and_then(|record| record.payload_hash))
    }

    /// Record a terminal outcome. `failed` is chosen when the outcome value
    /// carries an `error` field, otherwise `completed`.
    ///
    /// First outcome wins. A later execution of the same id — which the
    /// `Pending => Proceed` rule permits — must not replace the response an
    /// earlier one already published, or a client holding that id would be
    /// told about a mutation it never saw and lose the one it did.
    pub fn record_outcome(
        &self,
        request_id: &str,
        method: &str,
        outcome: &serde_json::Value,
    ) -> Result<(), ForgeError> {
        if self.is_settled(request_id) {
            return Ok(());
        }
        let status = if outcome.get("error").is_some() {
            RequestStatus::Failed
        } else {
            RequestStatus::Completed
        };
        // Preserve the original submitted_at — and the payload hash, without
        // which the terminal record could not be recognised as a duplicate of
        // the request that produced it.
        let prior = self.get(request_id)?;
        let submitted_at = prior
            .as_ref()
            .map(|record| record.submitted_at.clone())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
        let record = RequestRecord {
            request_id: request_id.to_string(),
            method: method.to_string(),
            status,
            submitted_at,
            completed_at: Some(chrono::Utc::now().to_rfc3339()),
            outcome: Some(outcome.clone()),
            payload_hash: prior.and_then(|record| record.payload_hash),
        };
        self.write_atomic(&record)
    }

    /// Settle requests that were durable-pending when a prior server process
    /// died. A restarted server has no live task that can safely resume them;
    /// reporting an explicit interrupted failure preserves discoverability and
    /// prevents a retry from duplicating an effect whose final state is unknown.
    ///
    /// Startup calls this before accepting sockets. Any unreadable record or
    /// failed terminal write aborts startup rather than reopening a cell that
    /// cannot truthfully account for its admitted work.
    pub fn settle_interrupted_requests(&self) -> Result<usize, ForgeError> {
        let mut settled = 0;
        for entry in std::fs::read_dir(&self.dir)
            .map_err(|error| ForgeError::io("read request correlations", error))?
        {
            let entry =
                entry.map_err(|error| ForgeError::io("read request correlation entry", error))?;
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                continue;
            }
            let bytes = std::fs::read(&path)
                .map_err(|error| ForgeError::io("read request correlation", error))?;
            let record: RequestRecord = serde_json::from_slice(&bytes)?;
            if record.status != RequestStatus::Pending {
                continue;
            }
            let outcome = serde_json::json!({
                "error": "server restarted before this admitted request settled",
                "error_class": "request_interrupted",
                "request_id": record.request_id,
                "recover_with": "request.get_status",
            });
            self.record_outcome(&record.request_id, &record.method, &outcome)?;
            settled += 1;
        }
        Ok(settled)
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn store() -> (tempfile::TempDir, RequestCorrelationStore) {
        let root = tempfile::tempdir().unwrap();
        let store = RequestCorrelationStore::open(root.path()).unwrap();
        (root, store)
    }

    #[test]
    fn payload_hash_is_stable_under_key_reordering() {
        // The two maps below are built by inserting the same keys in opposite
        // orders. `serde_json::Map` is a `BTreeMap` today and normalizes that
        // on its own; the sort in `canonicalize` is what keeps the answer the
        // same if a transitive dependency ever turns `preserve_order` on, and
        // this assertion is what would catch it if the sort went away.
        let mut forward = serde_json::Map::new();
        forward.insert("verb".into(), json!("submit"));
        forward.insert("entity".into(), json!("operator_local"));
        forward.insert("plan".into(), json!({"steps": 2, "id": "plan_a"}));

        let mut reverse = serde_json::Map::new();
        reverse.insert("plan".into(), json!({"id": "plan_a", "steps": 2}));
        reverse.insert("entity".into(), json!("operator_local"));
        reverse.insert("verb".into(), json!("submit"));

        let forward = serde_json::Value::Object(forward);
        let reverse = serde_json::Value::Object(reverse);
        assert_eq!(payload_hash(&forward), payload_hash(&reverse));

        // And it hashes the content, not just the key set — otherwise the
        // assertion above would hold for a constant.
        let changed = json!({
            "verb": "submit",
            "entity": "operator_local",
            "plan": {"id": "plan_b", "steps": 2},
        });
        assert_ne!(payload_hash(&forward), payload_hash(&changed));
    }

    #[test]
    fn a_record_written_before_sf_006_proceeds_rather_than_being_refused() {
        // Written as raw JSON with no `payload_hash` key at all, because that
        // is the shape already sitting under `<root>/requests` in every cell
        // that ran an earlier build: the point is that such a file still parses
        // and still lets its id be used, not that `None` behaves like `None`.
        let (root, store) = store();
        std::fs::write(
            root.path().join("requests").join("legacy-1.json"),
            r#"{"request_id":"legacy-1","method":"submit","status":"completed",
                "submitted_at":"2026-01-01T00:00:00Z",
                "completed_at":"2026-01-01T00:00:01Z",
                "outcome":{"state":"completed"}}"#,
        )
        .unwrap();

        let record = store
            .get("legacy-1")
            .unwrap()
            .expect("legacy record parses");
        assert_eq!(record.payload_hash, None);
        assert_eq!(
            store.check("legacy-1", "any-hash-at-all").unwrap(),
            DedupeVerdict::Proceed,
        );
    }

    #[test]
    fn restart_settles_a_pending_record_as_interrupted_without_reexecuting_it() {
        let (_root, store) = store();
        let payload = json!({"verb": "submit", "plan": "plan.json"});
        let hash = payload_hash(&payload);
        store
            .record_pending("req-interrupted", "case.submit", Some(&hash))
            .unwrap();

        assert_eq!(store.settle_interrupted_requests().unwrap(), 1);
        let record = store.get("req-interrupted").unwrap().unwrap();
        assert_eq!(record.status, RequestStatus::Failed);
        assert_eq!(
            record
                .outcome
                .as_ref()
                .and_then(|outcome| outcome["error_class"].as_str()),
            Some("request_interrupted")
        );
        assert_eq!(
            store.check("req-interrupted", &hash).unwrap(),
            DedupeVerdict::Replay(record.outcome.unwrap())
        );
    }

    #[test]
    fn restart_settlement_prevents_a_hashless_pending_record_from_reexecuting() {
        let (_root, store) = store();
        store
            .record_pending("req-legacy-pending", "case.submit", None)
            .unwrap();

        assert_eq!(store.settle_interrupted_requests().unwrap(), 1);
        assert!(matches!(
            store
                .check("req-legacy-pending", "new-payload-hash")
                .unwrap(),
            DedupeVerdict::Replay(_),
        ));
    }

    #[test]
    fn a_pending_record_refuses_concurrent_retry_without_erasing_its_locator() {
        let (_root, store) = store();
        let payload = json!({"verb": "submit", "plan": "plan.json"});
        let hash = payload_hash(&payload);
        store
            .record_pending("req-pending", "case.submit", Some(&hash))
            .unwrap();

        assert_eq!(
            store.check("req-pending", &hash).unwrap(),
            DedupeVerdict::Pending,
            "a retry while the admitted request is pending must not execute again",
        );
        let record = store.get("req-pending").unwrap().unwrap();
        assert_eq!(record.status, RequestStatus::Pending);
        assert_eq!(record.payload_hash.as_deref(), Some(hash.as_str()));
    }

    #[test]
    fn a_terminal_record_replays_its_own_payload_and_refuses_a_different_one() {
        let (_root, store) = store();
        let payload = json!({"verb": "submit", "plan": "plan.json", "entity": "operator_local"});
        let hash = payload_hash(&payload);

        store
            .record_pending("req-1", "submit", Some(&hash))
            .unwrap();
        assert_eq!(
            store.check("req-1", &hash).unwrap(),
            DedupeVerdict::Pending,
            "a request still in flight must not execute a concurrent duplicate",
        );

        let outcome = json!({"state": "completed", "case_id": "case_1"});
        store.record_outcome("req-1", "submit", &outcome).unwrap();
        assert_eq!(
            store.check("req-1", &hash).unwrap(),
            DedupeVerdict::Replay(outcome),
        );

        let edited = json!({"verb": "submit", "plan": "other.json", "entity": "operator_local"});
        assert_eq!(
            store.check("req-1", &payload_hash(&edited)).unwrap(),
            DedupeVerdict::Reused,
        );
    }

    #[test]
    fn a_pending_request_still_dedupes_after_the_store_is_reopened() {
        // What a restart looks like from here: the process is gone and with it
        // any in-memory verdict, but `<root>/requests` is still on disk. A
        // retry arriving afterwards must still be measured against the hash the
        // first attempt bound the id to — including through the terminal write,
        // which is the only writer that could drop it.
        let root = tempfile::tempdir().unwrap();
        let payload = json!({"verb": "case_commit", "template_ref": "sequential_agents@0.1.0"});
        let hash = payload_hash(&payload);

        let before = RequestCorrelationStore::open(root.path()).unwrap();
        before
            .record_pending("req-restart", "case_commit", Some(&hash))
            .unwrap();
        drop(before);

        let after = RequestCorrelationStore::open(root.path()).unwrap();
        let pending = after
            .get("req-restart")
            .unwrap()
            .expect("the pending record survives the restart");
        assert_eq!(pending.status, RequestStatus::Pending);
        assert_eq!(pending.payload_hash.as_deref(), Some(hash.as_str()));

        let outcome = json!({"state": "completed", "case_id": "case_restart"});
        after
            .record_outcome("req-restart", "case.commit", &outcome)
            .unwrap();
        assert_eq!(
            after.check("req-restart", &hash).unwrap(),
            DedupeVerdict::Replay(outcome),
            "the terminal write must carry the hash forward or the id stops deduping",
        );
    }

    #[test]
    fn a_second_pending_write_cannot_reopen_a_settled_record() {
        // `Pending => Proceed` is mandated, so two executions of one id are
        // possible by design. This is what must not follow from that: the
        // second one arrives, calls `record_pending`, and puts the id back to
        // `Pending` with no outcome — after which `check` says `Proceed` and
        // the mutation the client was already told about runs a second time.
        let (_root, store) = store();
        let payload = json!({"verb": "case_commit", "template_ref": "demo@0.1.0"});
        let hash = payload_hash(&payload);
        store
            .record_pending("req-settled", "case_commit", Some(&hash))
            .unwrap();
        let outcome = json!({"state": "completed", "case_id": "case_first"});
        store
            .record_outcome("req-settled", "case_commit", &outcome)
            .unwrap();

        store
            .record_pending("req-settled", "case_commit", Some(&hash))
            .unwrap();

        let record = store
            .get("req-settled")
            .unwrap()
            .expect("the record survives the second pending write");
        assert_eq!(record.status, RequestStatus::Completed);
        assert_eq!(record.outcome, Some(outcome.clone()));
        assert_eq!(
            store.check("req-settled", &hash).unwrap(),
            DedupeVerdict::Replay(outcome),
            "a reopened record stops replaying, which is how one id becomes two cases",
        );
    }

    #[test]
    fn a_later_outcome_cannot_replace_the_one_already_published() {
        // Same second execution, one step further along: it finishes and
        // reports its own result. Overwriting here would leave the id pointing
        // at a case the client never saw and no longer pointing at the one it
        // did, so recovery by that id would hand back the wrong mutation.
        let (_root, store) = store();
        let payload = json!({"verb": "case_commit", "template_ref": "demo@0.1.0"});
        let hash = payload_hash(&payload);
        store
            .record_pending("req-twice", "case_commit", Some(&hash))
            .unwrap();
        let first = json!({"state": "completed", "case_id": "case_first"});
        store
            .record_outcome("req-twice", "case_commit", &first)
            .unwrap();

        let second = json!({"state": "completed", "case_id": "case_second"});
        store
            .record_outcome("req-twice", "case_commit", &second)
            .unwrap();

        let record = store.get("req-twice").unwrap().expect("record exists");
        assert_eq!(record.outcome, Some(first.clone()));
        assert_eq!(
            store.check("req-twice", &hash).unwrap(),
            DedupeVerdict::Replay(first),
        );
    }

    /// The dispatch guard in `lib.rs` refuses a request with
    /// `idempotency_unverifiable` when this write fails, on the grounds that
    /// an unrecorded id cannot be recognised as a duplicate next time. That
    /// branch is only reachable if the failure is reported at all, which is
    /// what this covers: a `write_atomic` that ever swallowed its error would
    /// leave the guard permanently unreachable and every test above still
    /// green. Unix-only because a read-only *directory* is what withholds
    /// create permission; the Windows readonly bit does not.
    #[cfg(unix)]
    #[test]
    fn an_unwritable_store_reports_the_failure_rather_than_swallowing_it() {
        use std::os::unix::fs::PermissionsExt;

        let (root, store) = store();
        let dir = root.path().join("requests");
        let original = std::fs::metadata(&dir).unwrap().permissions();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o555)).unwrap();

        // Running as root ignores the mode bits entirely, and CI may. Probe
        // before asserting so that case skips rather than passing on a store
        // that was never actually unwritable.
        let probe = dir.join("probe.tmp");
        let enforced = std::fs::write(&probe, b"x").is_err();
        let result = if enforced {
            Some(store.record_pending("req-readonly", "case_commit", Some("hash")))
        } else {
            None
        };

        // Before any assertion: a 0o555 directory would otherwise defeat the
        // tempdir's own cleanup and leak it for the rest of the machine's life.
        std::fs::set_permissions(&dir, original).unwrap();
        let _ = std::fs::remove_file(&probe);

        match result {
            Some(result) => assert!(result.is_err()),
            None => eprintln!("skipped: the directory mode is not enforced for this user"),
        }
    }
}
