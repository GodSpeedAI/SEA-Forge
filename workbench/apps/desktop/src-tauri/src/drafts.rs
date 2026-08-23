//! Local case-authoring drafts (Task 6).
//!
//! Draft storage spike outcome (`stack-and-dependencies.md`'s deferred-decision
//! table): **versioned local JSON files under the Tauri app data dir**, chosen
//! over the Tauri store plugin / SQLite because it needs zero new
//! dependencies (`app.path().app_data_dir()` and `std::fs` are already used
//! by `lib.rs` for the event cursor) while still satisfying the spike's real
//! requirements — survives `kill -9` (a real file write, `fsync`'d via
//! `File::sync_all`), never touches `.sea-forge/`, is trivially exportable
//! (it already is a JSON file). If drafts outgrow single-file-per-draft
//! (concurrent multi-window editing, large attachments), revisit SQLite then;
//! today's Workbench has neither.
//!
//! Drafts are **local, reversible, non-authoritative** state (per
//! `implementation-workflow.md`'s "local draft vs canonical state" table) —
//! this module never touches the socket, `.sea-forge/`, or SQL, and a draft
//! only becomes real case state through `case.preflight` -> `case.commit`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The on-disk envelope for one draft. `version` is a per-draft monotonic
/// counter (not a schema version) so a future conflict-detection UI has
/// something to compare against; bumped on every save.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Draft {
    pub draft_id: String,
    pub version: u32,
    /// Seconds since the Unix epoch (stdlib `SystemTime`, no time dependency
    /// needed for a host crate that otherwise has none).
    pub updated_at: u64,
    /// Opaque form state (`template_ref`, `params`, `intent_summary`, ...).
    /// The host never interprets this — it is the renderer's draft shape,
    /// validated (if at all) on the renderer side against generated contracts.
    pub state: Value,
}

fn drafts_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("drafts")
}

fn draft_path(app_data_dir: &Path, draft_id: &str) -> PathBuf {
    drafts_dir(app_data_dir).join(format!("{draft_id}.json"))
}

/// The draft id is interpolated into a filesystem path. A renderer-supplied id
/// must be a single safe segment (`[A-Za-z0-9_-]{1,64}`) so a compromised web
/// context cannot traverse outside the drafts directory (F-09).
fn valid_draft_id(draft_id: &str) -> bool {
    !draft_id.is_empty()
        && draft_id.len() <= 64
        && draft_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
}

fn invalid_draft_id_error(draft_id: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("invalid draft id: {draft_id}"),
    )
}

/// Save (create or update) a draft. Bumps `version` from the prior on-disk
/// value when one exists. Atomic write (tmp + rename) so a crash mid-write
/// never leaves a half-written draft.
pub fn save(app_data_dir: &Path, draft_id: &str, state: Value) -> std::io::Result<Draft> {
    if !valid_draft_id(draft_id) {
        return Err(invalid_draft_id_error(draft_id));
    }
    let dir = drafts_dir(app_data_dir);
    std::fs::create_dir_all(&dir)?;
    let prior_version = load(app_data_dir, draft_id).ok().map(|d| d.version);
    let draft = Draft {
        draft_id: draft_id.to_string(),
        version: prior_version.map_or(1, |v| v + 1),
        updated_at: now_epoch_secs(),
        state,
    };
    let path = draft_path(app_data_dir, draft_id);
    let tmp = path.with_extension("json.tmp");
    let mut file = std::fs::File::create(&tmp)?;
    use std::io::Write;
    file.write_all(serde_json::to_vec_pretty(&draft)?.as_slice())?;
    file.sync_all()?;
    std::fs::rename(&tmp, &path)?;
    Ok(draft)
}

/// Load one draft by id.
pub fn load(app_data_dir: &Path, draft_id: &str) -> std::io::Result<Draft> {
    if !valid_draft_id(draft_id) {
        return Err(invalid_draft_id_error(draft_id));
    }
    let bytes = std::fs::read(draft_path(app_data_dir, draft_id))?;
    serde_json::from_slice(&bytes).map_err(std::io::Error::from)
}

/// List every saved draft, most recently updated first. Malformed draft files
/// are skipped rather than failing the whole list.
pub fn list(app_data_dir: &Path) -> std::io::Result<Vec<Draft>> {
    let dir = drafts_dir(app_data_dir);
    let Ok(read_dir) = std::fs::read_dir(&dir) else {
        return Ok(Vec::new());
    };
    let mut drafts: Vec<Draft> = read_dir
        .flatten()
        .filter_map(|entry| std::fs::read(entry.path()).ok())
        .filter_map(|bytes| serde_json::from_slice(&bytes).ok())
        .collect();
    drafts.sort_by(|a: &Draft, b: &Draft| b.updated_at.cmp(&a.updated_at));
    Ok(drafts)
}

/// Delete a draft. Not an error if it never existed (idempotent — matches
/// "reversible": discarding a draft that is already gone is a no-op, never
/// a surfaced failure).
pub fn delete(app_data_dir: &Path, draft_id: &str) -> std::io::Result<()> {
    if !valid_draft_id(draft_id) {
        return Err(invalid_draft_id_error(draft_id));
    }
    match std::fs::remove_file(draft_path(app_data_dir, draft_id)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn now_epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn draft_id_traversal_is_refused() {
        let dir =
            std::env::temp_dir().join(format!("sea-forge-drafts-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for id in ["../../../home/user/.config/foo", "a/b", "", "..", "a b"] {
            assert!(
                save(&dir, id, json!({"x": 1})).is_err(),
                "draft_id {id:?} must be refused"
            );
            assert!(load(&dir, id).is_err(), "draft_id {id:?} must be refused");
            assert!(delete(&dir, id).is_err(), "draft_id {id:?} must be refused");
        }
        assert!(!dir.join("drafts").join("foo.json").exists());
        // A well-formed id round-trips.
        let draft = save(&dir, "draft_1", json!({"x": 1})).unwrap();
        assert_eq!(draft.draft_id, "draft_1");
        assert_eq!(load(&dir, "draft_1").unwrap().version, 1);
        delete(&dir, "draft_1").unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
