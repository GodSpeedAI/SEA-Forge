//! Shared fixtures: records the guard's strict load accepts (each payload
//! satisfies the registry's schema for its kind).
#![allow(dead_code)]

use serde_json::{json, Value};

/// The payload a record of `record`'s kind carries, from its other members
/// (`level`/`disposition` or `gate`/`state`).
pub fn payload_for(record: &Value) -> Value {
    match record["kind"].as_str().unwrap_or("") {
        "disposition.recorded" => {
            let mut p = json!({
                "level": record["level"], "disposition": record["disposition"],
                "rationale": "fixture", "approval_ref": "appr-fixture"
            });
            if record["level"] == "episode" {
                p["episode_ref"] = json!("episode:case-0001:task-a:1@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            }
            p
        }
        "gate.assessed" => json!({
            "gate": record["gate"], "state": record["state"],
            "evidence_refs": ["ev-fixture"], "rationale": "fixture", "assessor": "assessor-a"
        }),
        "field_case.revised" => json!({
            "base_revision": 1,
            "changes": [{"path": "purpose_and_outcome.signal",
                         "field": {"status": "declared", "source_ref": {"kind": "operator", "ref": "o"}, "value": "x"}}],
            "rationale": "fixture"
        }),
        "strategy.recorded" => json!({
            "revision": 1, "supersedes": null, "where_to_play": "w", "how_to_win": "h",
            "underserved_outcome": "u", "tested_assumption": "t", "abandonment_trigger": "a",
            "revisit_reason": null
        }),
        "episode.frozen" => fixture("request-episode-frozen.json")["payload"].clone(),
        "episode.settled" => fixture("request-episode-settled.json")["payload"].clone(),
        _ => Value::Null,
    }
}

fn fixture(name: &str) -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/fde/fixtures/case-record-append/valid")
        .join(name);
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

/// A line carrying every member any store requires (for `check`, which
/// writes nothing).
pub fn store_line() -> Value {
    let mut line = serde_json::Map::new();
    for key in [
        "idempotency_key",
        "state",
        "decision_id",
        "decision",
        "approval_ref",
        "consumed_by",
        "system",
        "record_id",
        "id",
        "event_id",
        "run_id",
        "case_id",
        "episode_ref",
        "verb",
        "link_id",
        "delegate",
        "assignment_id",
        "file",
        "dropped_sha256",
    ] {
        line.insert(key.into(), json!("x"));
    }
    Value::Object(line)
}
