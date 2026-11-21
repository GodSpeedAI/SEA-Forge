//! The trusted read of a recorded assignment binding (assignment-binding.v1,
//! WP-23): joins the exact case, episode, assignment, link and run.
//!
//! Everything is read from the case store under the caller's lock discipline
//! and re-derived, never taken from a caller. The binding's digest recomputes
//! over its own members; the row's link and run must exist and name the same
//! episode and assignment. A legacy row (written before the binding existed,
//! so no `binding` member), a partial append (a binding whose link or run is
//! missing, or whose join names another episode), and a tampered digest all
//! fail closed with `ASSIGNMENT_BINDING_INVALID`: the reader never repairs,
//! guesses or falls back.

use std::path::Path;

use sea_forge_core::errors::ForgeError;
use serde_json::{json, Value};

use super::case_records::{self, canonical_actor, digest_of};
use sea_forge_case_runner::case_ops::case_snapshot_dir;

fn refuse(message: &str) -> Value {
    json!({"outcome": "refused", "code": "ASSIGNMENT_BINDING_INVALID",
           "message": message})
}

/// The wire read `assignment_binding_get` (README §11, WP-23): [`read`] for
/// exactly the link's delegate or delegator. Anyone else — and a legacy row,
/// which has no link join to authorize against — gets the SAME answer as for an
/// unknown assignment, so a binding's existence is never revealed (the §9 read
/// rule). The integrity refusals of [`read`] are answered only to a caller who
/// already proved membership.
pub fn get(
    root: &Path,
    case_id: &str,
    assignment_id: &str,
    viewer: &str,
) -> Result<Result<Value, Value>, ForgeError> {
    // One constant answer for every hidden path (unknown case, unknown
    // assignment, legacy row, unentitled caller): the response for a request
    // never depends on whether the binding exists (the §9 read rule).
    let hidden = || ForgeError::Input("unknown assignment binding".into());
    if !sea_forge_core::path::valid_id_segment(case_id, 128) {
        return Err(hidden());
    }
    let Ok(dir) = case_snapshot_dir(root, case_id) else {
        return Err(hidden());
    };
    if !dir.join("case.json").exists() {
        return Err(hidden());
    }
    let rows = case_records::read_lines_strict(&dir.join("field-assignments.jsonl"))
        .map_err(|_| hidden())?;
    let Some(row) = rows
        .iter()
        .rev()
        .find(|r| r["assignment_id"] == assignment_id)
    else {
        return Err(hidden());
    };
    // A legacy row has no binding: nothing to authorize against, nothing to
    // serve — the same answer as an unknown assignment, for every caller.
    if row["binding"].as_object().is_none() {
        return Err(hidden());
    }
    let Some(link_id) = row["link_id"].as_str() else {
        return Err(hidden());
    };
    let links =
        case_records::read_lines_strict(&dir.join("field-links.jsonl")).map_err(|_| hidden())?;
    let Some(link) = links.iter().rev().find(|l| l["link_id"] == link_id) else {
        return Err(hidden());
    };
    let viewer = canonical_actor(viewer).unwrap_or_default();
    let entitled = ["delegate", "delegator"].iter().any(|role| {
        link[*role]
            .as_str()
            .and_then(|p| canonical_actor(p).ok())
            .is_some_and(|p| p == viewer)
    });
    if !entitled {
        return Err(hidden());
    }
    Ok(read(root, case_id, assignment_id)
        .map(|joined| json!({"case_id": case_id, "binding": joined})))
}

/// Read the joined binding of `assignment_id`: `{case_id, episode_key,
/// assignment, link_id, run_id, binding_digest, link, run}`.
///
/// `Err` is the contract's refused Error value; it carries no side effect (a
/// read has none) and must be answered, not retried against another store.
pub fn read(root: &Path, case_id: &str, assignment_id: &str) -> Result<Value, Value> {
    let (case, item, n, k) = super::episode_gate::parse_assignment_parts(assignment_id)
        .ok_or_else(|| {
            refuse("assignment_id is not assign:<case>:<item>:<n>:<k>");
        })?;
    if case != case_id {
        return Err(refuse("assignment_id names another case"));
    }
    let dir = case_snapshot_dir(root, case_id)
        .map_err(|why| refuse(&format!("case_id does not resolve: {why}")))?;
    let read_store = |name: &str| -> Result<Vec<Value>, Value> {
        case_records::read_lines_strict(&dir.join(name))
            .map_err(|why| refuse(&format!("store integrity: {why}")))
    };
    let rows = read_store("field-assignments.jsonl")?;
    let row = rows
        .iter()
        .rev()
        .find(|r| r["assignment_id"] == assignment_id)
        .ok_or_else(|| refuse("no assignment row carries this id"))?;
    let binding = &row["binding"];
    if !binding.is_object() {
        return Err(refuse(
            "the latest row is a legacy row with no recorded binding",
        ));
    }
    sea_forge_case_runner::case_ops::contract::assignment_binding_is_valid(binding)
        .map_err(|why| refuse(&format!("the recorded binding is not valid: {why}")))?;
    let bound = &binding["assignment"];
    // The id, its parts and the episode must agree (assignment.v1 semantics).
    let id_ok = bound["assignment_id"] == assignment_id
        && bound["case_id"].as_str() == Some(case_id)
        && bound["step_id"].as_str() == Some(item.as_str())
        && bound["n"].as_u64() == Some(n)
        && bound["k"].as_u64() == Some(k);
    if !id_ok {
        return Err(refuse("the binding's assignment disagrees with its id"));
    }
    if row["episode_key"]["case_id"].as_str() != Some(case_id)
        || row["episode_key"]["plan_item_id"].as_str() != Some(item.as_str())
        || row["episode_key"]["n"].as_u64() != Some(n)
    {
        return Err(refuse("the assignment row names another episode"));
    }
    // The digest recomputes over exactly the bound members plus the joins.
    let body = json!({
        "assignment_id": bound["assignment_id"], "episode_ref": bound["episode_ref"],
        "brief": bound["brief"], "budget": bound["budget"],
        "link_id": binding["link_id"], "run_id": binding["run_id"],
    });
    if binding["binding_digest"] != digest_of(&body).unwrap_or_default().as_str() {
        return Err(refuse("the binding digest does not recompute"));
    }
    // The exact link and run, naming the same episode and assignment.
    let episode_ref = bound["episode_ref"].clone();
    let links = read_store("field-links.jsonl")?;
    let link = links
        .iter()
        .rev()
        .find(|l| l["link_id"] == binding["link_id"])
        .ok_or_else(|| {
            refuse("the binding cites a link that was never written (partial append)")
        })?;
    if link["case_id"].as_str() != Some(case_id)
        || link["assignment_id"].as_str() != Some(assignment_id)
        || link["episode_ref"] != episode_ref
    {
        return Err(refuse(
            "the binding's link names another episode or assignment",
        ));
    }
    let runs = read_store("field-runs.jsonl")?;
    let run = runs
        .iter()
        .rev()
        .find(|r| r["run_id"] == binding["run_id"])
        .ok_or_else(|| refuse("the binding cites a run that was never written (partial append)"))?;
    if run["case_id"].as_str() != Some(case_id)
        || run["link_id"] != binding["link_id"]
        || run["episode_ref"] != episode_ref
    {
        return Err(refuse("the binding's run names another episode or link"));
    }
    Ok(json!({
        "case_id": case_id,
        "episode_key": {"case_id": case_id, "plan_item_id": item, "n": n},
        "assignment": bound,
        "link_id": binding["link_id"], "run_id": binding["run_id"],
        "binding_digest": binding["binding_digest"],
        "version": binding["version"],
        "link": link, "run": run,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write(root: &Path, name: &str, rows: &[Value]) {
        let dir = root.join("cases").join("case-0001");
        std::fs::create_dir_all(&dir).unwrap();
        let text: String = rows
            .iter()
            .map(|r| serde_json::to_string(r).unwrap())
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        std::fs::write(dir.join(name), text).unwrap();
    }

    fn approval() -> Value {
        json!({
            "domain_meaning": "An evidence record names an independent source.",
            "context_refs": [{"ref": "doc:handoff-note-1", "trust": "untrusted_reference"}],
            "budget": [
                {"currency": "time", "limit": 14400, "unit": "seconds"},
                {"currency": "attention", "limit": "unknown", "unit": "operator_minutes"},
                {"currency": "money", "limit": 5, "unit": "USD"},
                {"currency": "risk", "limit": "unknown", "unit": "class"},
                {"currency": "recovery", "limit": 1, "unit": "revert_steps"}
            ]
        })
    }

    fn joined_stores(episode_ref: &str) -> (Value, Value, Value) {
        let brief = json!({
            "instruction": "Record the verifier's observation for gate G3.",
            "adoptable_problem": "Why the evidence record is missing after the run.",
            "domain_meaning": approval()["domain_meaning"],
            "context_refs": approval()["context_refs"],
        });
        let assignment = json!({
            "assignment_id": "assign:case-0001:item-03:1:1",
            "case_id": "case-0001", "step_id": "item-03", "n": 1, "k": 1,
            "episode_ref": episode_ref, "brief": brief,
            "budget": approval()["budget"], "issued_at": "2026-11-20T09:00:00Z",
        });
        let body = json!({"assignment_id": "assign:case-0001:item-03:1:1",
                          "episode_ref": episode_ref, "brief": brief,
                          "budget": approval()["budget"],
                          "link_id": "link-0001", "run_id": "run-0001"});
        let binding = json!({"version": 1, "assignment": assignment,
                             "link_id": "link-0001", "run_id": "run-0001",
                             "binding_digest": digest_of(&body).unwrap(),
                             "recorded_at": "2026-11-20T09:00:01Z"});
        let row = json!({"assignment_id": "assign:case-0001:item-03:1:1",
                         "state": "issued",
                         "episode_key": {"case_id": "case-0001", "plan_item_id": "item-03", "n": 1},
                         "link_id": "link-0001", "at": "2026-11-20T09:00:01Z",
                         "binding": binding});
        let link = json!({"link_id": "link-0001", "case_id": "case-0001",
                          "episode_ref": episode_ref,
                          "assignment_id": "assign:case-0001:item-03:1:1",
                          "delegator": "operator_local", "gateway_actor": null,
                          "delegate": "agent:http://127.0.0.1:8099/agent",
                          "issued_at": "2026-11-20T09:00:00Z",
                          "expires_at": "2026-11-20T17:00:00Z"});
        let run = json!({"run_id": "run-0001", "case_id": "case-0001",
                         "episode_ref": episode_ref, "verb": "delegate",
                         "role": "executor", "actor": "agent:http://127.0.0.1:8099/agent",
                         "on_behalf_of": "operator_local", "delegated": true,
                         "link_id": "link-0001", "workspace_ref": "ws:agent:x",
                         "started_at": "2026-11-20T09:00:01Z",
                         "recorded_at": "2026-11-20T09:00:01Z"});
        (row, link, run)
    }

    const EPISODE: &str = "episode:case-0001:item-03:1@sha256:1f1e2e28b65a8957374e40e7e85c67309c07a6577ad8f997b12b4ce9ce866879";

    #[test]
    fn a_recorded_binding_joins_case_episode_assignment_link_and_run() {
        let root = tempfile::tempdir().unwrap();
        let (row, link, run) = joined_stores(EPISODE);
        write(root.path(), "field-assignments.jsonl", &[row]);
        write(root.path(), "field-links.jsonl", &[link]);
        write(root.path(), "field-runs.jsonl", &[run]);
        let joined = read(root.path(), "case-0001", "assign:case-0001:item-03:1:1").unwrap();
        assert_eq!(joined["run"]["run_id"], "run-0001");
        assert_eq!(joined["link"]["link_id"], "link-0001");
        assert_eq!(
            joined["assignment"]["brief"]["adoptable_problem"],
            "Why the evidence record is missing after the run."
        );
    }

    #[test]
    fn a_legacy_row_without_a_binding_fails_closed() {
        let root = tempfile::tempdir().unwrap();
        let (mut row, link, run) = joined_stores(EPISODE);
        row.as_object_mut().unwrap().remove("binding");
        write(root.path(), "field-assignments.jsonl", &[row]);
        write(root.path(), "field-links.jsonl", &[link]);
        write(root.path(), "field-runs.jsonl", &[run]);
        let refused = read(root.path(), "case-0001", "assign:case-0001:item-03:1:1").unwrap_err();
        assert_eq!(refused["code"], "ASSIGNMENT_BINDING_INVALID");
        assert!(refused["message"].as_str().unwrap().contains("legacy"));
    }

    #[test]
    fn a_partial_append_without_the_run_fails_closed() {
        let root = tempfile::tempdir().unwrap();
        let (row, link, _) = joined_stores(EPISODE);
        write(root.path(), "field-assignments.jsonl", &[row]);
        write(root.path(), "field-links.jsonl", &[link]);
        let refused = read(root.path(), "case-0001", "assign:case-0001:item-03:1:1").unwrap_err();
        assert_eq!(refused["code"], "ASSIGNMENT_BINDING_INVALID");
        assert!(refused["message"].as_str().unwrap().contains("partial"));
    }

    #[test]
    fn a_tampered_digest_fails_closed() {
        let root = tempfile::tempdir().unwrap();
        let (mut row, link, run) = joined_stores(EPISODE);
        row["binding"]["assignment"]["brief"]["domain_meaning"] = json!("quietly rewritten");
        write(root.path(), "field-assignments.jsonl", &[row]);
        write(root.path(), "field-links.jsonl", &[link]);
        write(root.path(), "field-runs.jsonl", &[run]);
        let refused = read(root.path(), "case-0001", "assign:case-0001:item-03:1:1").unwrap_err();
        assert_eq!(refused["code"], "ASSIGNMENT_BINDING_INVALID");
        assert!(refused["message"].as_str().unwrap().contains("digest"));
    }

    #[test]
    fn a_link_naming_another_episode_fails_closed() {
        let root = tempfile::tempdir().unwrap();
        let (row, mut link, run) = joined_stores(EPISODE);
        link["episode_ref"] = json!("episode:case-0001:item-03:2@sha256:other");
        write(root.path(), "field-assignments.jsonl", &[row]);
        write(root.path(), "field-links.jsonl", &[link]);
        write(root.path(), "field-runs.jsonl", &[run]);
        let refused = read(root.path(), "case-0001", "assign:case-0001:item-03:1:1").unwrap_err();
        assert_eq!(refused["code"], "ASSIGNMENT_BINDING_INVALID");
    }
}
