//! Golden wire fixture for `ApprovalStanding::to_json`.
//!
//! `contracts/fixtures/approval-standing.json` is a cross-repo wire contract: the Cognate SDK keeps a
//! copy and parses the same two objects. This test builds the same two standings and requires the
//! fixture to equal their `to_json` output as parsed JSON, so key order does not matter.
//!
//! To rewrite the fixture after an intentional wire change, run with `APPROVAL_STANDING_FIXTURE_WRITE=1`.

use chrono::{TimeZone, Utc};
use sea_forge_authority::cep::{ApprovalStanding, ApprovalState};
use serde_json::Value;
use std::path::PathBuf;

const WORLD: &str =
    "world:horizon@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn standing(
    n: u32,
    state: ApprovalState,
    hour: u32,
    minute: u32,
    resolved_by: Option<&str>,
) -> ApprovalStanding {
    ApprovalStanding {
        approval_id: format!("apr-cep:op-000{n}"),
        state,
        requester: "cognate-service".into(),
        world_ref: WORLD.into(),
        kind: "action".into(),
        operation_name: "run.start".into(),
        resource_id: "agent.echo".into(),
        subject_actor_ref: "actor:tenant-a/alice".into(),
        original_request_envelope_id: format!("env-authority_request-op-000{n}"),
        escalation_entry: format!("entry-op-000{n}"),
        expires_at: Utc.with_ymd_and_hms(2026, 11, 5, hour, minute, 0).unwrap(),
        resolved_by: resolved_by.map(str::to_string),
    }
}

fn standings() -> Vec<ApprovalStanding> {
    vec![
        standing(1, ApprovalState::Pending, 12, 0, None),
        standing(2, ApprovalState::Approved, 13, 30, Some("actor:operator")),
    ]
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/fixtures/approval-standing.json")
}

const WIRE_KEYS: [&str; 11] = [
    "approval_id",
    "state",
    "requester",
    "world_ref",
    "operation_kind",
    "operation_name",
    "resource_id",
    "subject_actor_ref",
    "request_ref",
    "expires_at",
    "resolved_by",
];

#[test]
fn fixture_matches_to_json_of_the_two_standings() {
    let expected: Vec<Value> = standings().iter().map(ApprovalStanding::to_json).collect();
    let path = fixture_path();
    if std::env::var_os("APPROVAL_STANDING_FIXTURE_WRITE").is_some() {
        let mut text = serde_json::to_string_pretty(&expected).unwrap();
        text.push('\n');
        std::fs::write(&path, text).unwrap();
    }
    let text = std::fs::read_to_string(&path).expect("fixture present");
    let fixture: Value = serde_json::from_str(&text).expect("fixture is JSON");
    assert_eq!(
        fixture,
        Value::Array(expected),
        "fixture drifted from ApprovalStanding::to_json"
    );
}

#[test]
fn fixture_objects_have_exactly_the_eleven_wire_keys() {
    let text = std::fs::read_to_string(fixture_path()).expect("fixture present");
    let fixture: Value = serde_json::from_str(&text).unwrap();
    let rows = fixture.as_array().expect("fixture is an array");
    assert_eq!(rows.len(), 2);
    for row in rows {
        let object = row.as_object().expect("row is an object");
        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();
        let mut want = WIRE_KEYS.to_vec();
        want.sort_unstable();
        assert_eq!(keys, want);
    }
    assert_eq!(rows[0]["state"], "pending");
    assert!(rows[0]["resolved_by"].is_null());
    assert_eq!(rows[1]["state"], "approved");
    assert!(rows[1]["resolved_by"].is_string());
}
