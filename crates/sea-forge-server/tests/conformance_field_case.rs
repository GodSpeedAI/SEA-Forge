//! WP-14 conformance for `case.record.append` against `contracts/fde/`: every
//! fixture request, every refusal scenario, the two-phase freeze with crash
//! recovery, and the acceptance and negative tests the S1 gate names (AT-04,
//! NT-01, NT-16, NT-13, NT-32, NT-33, NT-42; and I-1, I-2, I-3).
//!
//! Test ids are in the function names (`at_04_*`, `nt_01_*`, `nt_16_*`) so
//! `scripts/fde-proof.d/<ID>.sh` can select them.
//!
//! Limit: scenario messages are not compared (the contract fixes codes and
//! invariants; message text is free), and a recorded response's `record_id`
//! is compared only for replays, where the ledger fixes it.

use sea_forge_server::sfwp::case_records::{
    append, append_faulted, deliver_anchors, freeze_status, recover, require_complete_freeze,
    vendored_schemas, verify_freeze_anchor, AnchorOrder, AnchorPublisher, AppendContext,
    CrashInjected, Fault, FileIndex, FreezeStatus, Recovered, ScriptPublisher,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

fn contracts() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fde")
}

fn read(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap_or_else(|e| panic!("{path:?}: {e}")))
        .unwrap()
}

fn fixture(name: &str) -> Value {
    read(&contracts().join("fixtures/case-record-append").join(name))
}

fn cell() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("cases/case-0001");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("case.json"), "{}").unwrap();
    // A committed case has an event ledger; its absence is a refusal (F7).
    fs::write(dir.join("case-events.jsonl"), "").unwrap();
    root
}

fn rt_register(root: &Path, ids: &[&str]) {
    let dir = root.join("realitytrace");
    fs::create_dir_all(&dir).unwrap();
    let mut text = fs::read_to_string(dir.join("records.jsonl")).unwrap_or_default();
    for id in ids {
        text.push_str(&format!("{}\n", json!({"record_id": id})));
    }
    fs::write(dir.join("records.jsonl"), text).unwrap();
}

fn rt_register_for(root: &Path, request: &Value) {
    let refs = &request["payload"]["realitytrace_refs"];
    let mut ids: Vec<String> = vec![refs["plan_id"].as_str().unwrap().into()];
    for f in ["question_ids", "threshold_ids"] {
        ids.extend(
            refs[f]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string()),
        );
    }
    let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    rt_register(root, &ids);
}

fn ctx<'a>(
    root: &'a Path,
    principal: &'a str,
    chain: &'a [String],
    idx: &'a FileIndex,
) -> AppendContext<'a> {
    AppendContext {
        root,
        case_id: "case-0001",
        principal,
        role: "operator",
        chain,
        rt: idx,
        anchor: None,
    }
}

fn run(root: &Path, principal: &str, chain: &[String], request: &Value) -> Value {
    let idx = FileIndex(root.to_path_buf());
    append(&ctx(root, principal, chain, &idx), request)
}

fn run_as(root: &Path, principal: &str, request: &Value) -> Value {
    run(root, principal, &[principal.to_string()], request)
}

fn write_event(root: &Path, plan_item: &str, actor: &str, timestamp: &str) {
    let path = root.join("cases/case-0001/case-events.jsonl");
    let mut text = fs::read_to_string(&path).unwrap_or_default();
    text.push_str(&format!(
        "{}\n",
        json!({"event_id": format!("ev-{actor}-{timestamp}"), "plan_item_id": plan_item,
               "actor_id": actor, "kind": "command_started", "timestamp": timestamp})
    ));
    fs::write(path, text).unwrap();
}

fn code(v: &Value) -> &str {
    v["code"].as_str().unwrap_or("")
}

struct Published(Value);
impl AnchorPublisher for Published {
    fn publish(&self, _: &str, _: &str, _: &str) -> Result<Value, String> {
        Ok(self.0.clone())
    }
}
struct Failing;
impl AnchorPublisher for Failing {
    fn publish(&self, _: &str, _: &str, _: &str) -> Result<Value, String> {
        Err("remote unreachable".into())
    }
}

// ---------------------------------------------------------------------------
// Contract gate
// ---------------------------------------------------------------------------

#[test]
fn contract_gate_vendored_schemas_match_contract_digests() {
    let manifest = fs::read_to_string(contracts().join("DIGESTS")).unwrap();
    for (name, text) in vendored_schemas() {
        let want = manifest
            .lines()
            .find_map(|l| {
                l.split_once("  ")
                    .filter(|(_, n)| *n == name)
                    .map(|(h, _)| h.to_string())
            })
            .unwrap_or_else(|| panic!("{name} is not in contracts/fde/DIGESTS"));
        assert_eq!(
            hex::encode(Sha256::digest(text.as_bytes())),
            want,
            "vendored {name} differs from contracts/fde (digest)"
        );
        assert_eq!(
            text,
            fs::read_to_string(contracts().join(name)).unwrap(),
            "vendored {name} differs from contracts/fde"
        );
    }
}

// ---------------------------------------------------------------------------
// Request / response / error tables
// ---------------------------------------------------------------------------

#[test]
fn valid_request_fixtures_are_recorded_and_settle_after_their_freeze() {
    // frozen (evaluated), frozen (exploratory)
    for name in [
        "request-episode-frozen.json",
        "request-episode-frozen-exploratory.json",
    ] {
        let root = cell();
        let request = fixture(&format!("valid/{name}"));
        rt_register_for(root.path(), &request);
        let out = run_as(root.path(), "case-keeper", &request);
        assert_eq!(out["outcome"], "recorded", "{name}: {out}");
        assert_eq!(out["kind"], "episode.frozen");
        assert_eq!(out["replayed"], false);
        assert_eq!(out["episode_key"]["case_id"], "case-0001");
    }
    // settled variants against the freeze each one names
    for (settled, frozen, result) in [
        (
            "request-episode-settled.json",
            "request-episode-frozen.json",
            "S",
        ),
        (
            "request-episode-settled-empty-reliability.json",
            "request-episode-frozen.json",
            "U",
        ),
        (
            "request-episode-settled-exploratory-unresolved.json",
            "request-episode-frozen-exploratory.json",
            "U",
        ),
    ] {
        let root = cell();
        let freeze = fixture(&format!("valid/{frozen}"));
        rt_register_for(root.path(), &freeze);
        assert_eq!(
            run_as(root.path(), "case-keeper", &freeze)["outcome"],
            "recorded"
        );
        let req = fixture(&format!("valid/{settled}"));
        assert_eq!(req["payload"]["result"], result);
        let out = run_as(root.path(), "verifier", &req);
        assert_eq!(out["outcome"], "recorded", "{settled}: {out}");
        assert_eq!(out["kind"], "episode.settled");
    }
}

#[test]
fn invalid_request_fixtures_are_refused_with_the_code_their_pointer_selects() {
    let manifest = fixture("manifest.json");
    let mut seen = 0;
    for entry in manifest["invalid"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["def"] == "Request")
    {
        let name = entry["file"].as_str().unwrap();
        let pointer = entry["pointer"].as_str().unwrap();
        let expected = if pointer == "/kind" {
            "KIND_UNKNOWN"
        } else if pointer == "/payload/result" || pointer.starts_with("/payload/reliability") {
            "RESULT_INVALID"
        } else {
            "SCHEMA_INVALID"
        };
        let root = cell();
        let request = fixture(&format!("invalid/{name}"));
        let out = run_as(root.path(), "case-keeper", &request);
        assert_eq!(out["outcome"], "refused", "{name}: {out}");
        assert_eq!(code(&out), expected, "{name}: {out}");
        assert!(
            out.get("invariant").is_none(),
            "{name}: schema-level refusals carry no invariant"
        );
        seen += 1;
    }
    assert_eq!(seen, 11, "manifest request rows exercised");
    // the semantic row: digest differs from episode_ref (NT-17 shape)
    for entry in manifest["semantic_invalid"].as_array().unwrap() {
        let root = cell();
        let request = fixture(&format!("invalid/{}", entry["file"].as_str().unwrap()));
        rt_register_for(root.path(), &request);
        let out = run_as(root.path(), "case-keeper", &request);
        assert_eq!(code(&out), entry["code"].as_str().unwrap(), "{out}");
    }
}

#[test]
fn nt_17_a_body_carrying_result_content_is_refused_by_the_closed_schema() {
    // The contract has no REFERENCE_DERIVED_FROM_RESULT code; the closed
    // reference body refuses a result field as SCHEMA_INVALID.
    let root = cell();
    let out = run_as(
        root.path(),
        "case-keeper",
        &fixture("invalid/request-frozen-reference-body-carries-result.json"),
    );
    assert_eq!(code(&out), "SCHEMA_INVALID", "{out}");
}

#[test]
fn nt_42_result_outside_sfuc_and_merged_reliability_are_refused() {
    let root = cell();
    for name in [
        "request-settled-result-not-sfuc.json",
        "request-settled-reliability-merged-into-result.json",
    ] {
        let out = run_as(
            root.path(),
            "verifier",
            &fixture(&format!("invalid/{name}")),
        );
        assert_eq!(code(&out), "RESULT_INVALID", "{name}: {out}");
    }
}

#[test]
fn response_and_error_fixtures_match_what_the_verb_emits_in_shape() {
    // The server's own outputs validate against the contract's Response and
    // Error defs: checked structurally through the fixtures' key sets.
    let recorded = fixture("valid/response-recorded.json");
    let root = cell();
    let request = fixture("valid/request-episode-frozen.json");
    rt_register_for(root.path(), &request);
    let out = run_as(root.path(), "case-keeper", &request);
    let keys = |v: &Value| {
        v.as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
    };
    assert_eq!(keys(&out), keys(&recorded));
    let refused = run_as(
        root.path(),
        "case-keeper",
        &json!({"kind": "episode.frozen"}),
    );
    assert!(keys(&refused).is_subset(&keys(&fixture("valid/error-freeze-key-exists.json"))));
    assert_eq!(refused["outcome"], "refused");
}

// ---------------------------------------------------------------------------
// Scenarios (I-1, I-2, I-3, idempotency, exploratory)
// ---------------------------------------------------------------------------

fn seed_ledger(root: &Path, scenario: &Value) {
    let request = &scenario["request"];
    let mut text = String::new();
    let mut intents = String::new();
    for (i, entry) in scenario["ledger_before"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let key = entry["idempotency_key"].as_str().unwrap();
        let same_body =
            key == request["idempotency_key"] && request["payload"]["digest"] == entry["digest"];
        // A seeded entry has no payload; a replay needs the body digest of the
        // request it answered, so it is taken from the request only when the
        // scenario says this key was this exact request.
        let body_digest = if same_body {
            sea_forge_server::sfwp::case_records::digest_of(
                &json!({"kind": request["kind"], "payload": request["payload"]}),
            )
            .unwrap()
        } else {
            format!("seeded-{key}")
        };
        let rec_id = key.rsplit('/').next().unwrap();
        let mut record = json!({
            "seq": i + 1, "record_id": rec_id, "kind": entry["kind"], "idempotency_key": key,
            "body_digest": body_digest, "episode_key": entry["episode_key"], "digest": entry["digest"],
            "principal": "case-keeper",
            "payload": {"realitytrace_refs": {"plan_id": "plan:seed", "question_ids": [], "threshold_ids": []}},
            "response": {"outcome": "recorded", "kind": entry["kind"], "record_id": rec_id,
                         "episode_key": entry["episode_key"], "replayed": false},
            "prev_hash": "0", "hash": "0",
        });
        if let Some(kind) = entry.get("episode_kind") {
            record["episode_kind"] = kind.clone();
        }
        text.push_str(&format!("{record}\n"));
        if entry["kind"] == "episode.frozen" {
            // A seeded freeze is a complete one: committed intent, RT plan.
            intents.push_str(&format!(
                "{}\n",
                json!({"idempotency_key": key, "state": "committed", "episode_key": entry["episode_key"],
                       "body_digest": record["body_digest"], "digest": entry["digest"], "payload": record["payload"]})
            ));
        }
    }
    rt_register(root, &["plan:seed"]);
    fs::write(root.join("cases/case-0001/field-records.jsonl"), text).unwrap();
    fs::write(
        root.join("cases/case-0001/field-freeze-intents.jsonl"),
        intents,
    )
    .unwrap();
}

#[test]
fn every_scenario_in_contracts_fde_produces_its_expected_response() {
    let manifest = fixture("manifest.json");
    let mut run_count = 0;
    for name in manifest["scenarios"].as_array().unwrap() {
        let scenario = fixture(&format!("scenarios/{}", name.as_str().unwrap()));
        let root = cell();
        seed_ledger(root.path(), &scenario);
        let request = &scenario["request"];
        if (scenario["rule"] != "I-1" || scenario["nt"] != "NT-33")
            && request["kind"] == "episode.frozen"
        {
            rt_register_for(root.path(), request);
        }
        let chain: Vec<String> = match scenario.get("authenticated_delegation_chain") {
            Some(c) => c
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect(),
            None => vec!["verifier".into()],
        };
        if let Some(executor) = scenario.get("episode_executor") {
            write_event(
                root.path(),
                "item-03",
                executor.as_str().unwrap(),
                "2026-11-12T10:00:00Z",
            );
        }
        let principal = chain.last().unwrap().clone();
        let out = run(root.path(), &principal, &chain, request);
        let id = scenario["id"].as_str().unwrap();
        let expect = &scenario["expect"];
        assert_eq!(out["outcome"], expect["outcome"], "{id}: {out}");
        if expect["outcome"] == "refused" {
            assert_eq!(out["code"], expect["code"], "{id}: {out}");
            assert_eq!(out.get("invariant"), expect.get("invariant"), "{id}: {out}");
        } else {
            assert_eq!(out["kind"], expect["kind"], "{id}");
            assert_eq!(out["episode_key"], expect["episode_key"], "{id}");
            assert_eq!(out["replayed"], expect["replayed"], "{id}");
            if expect["replayed"] == true {
                assert_eq!(out["record_id"], expect["record_id"], "{id}");
            }
        }
        run_count += 1;
    }
    assert_eq!(run_count, 11);
}

#[test]
fn replay_returns_the_original_response_and_writes_nothing() {
    let root = cell();
    let request = fixture("valid/request-episode-frozen.json");
    rt_register_for(root.path(), &request);
    let first = run_as(root.path(), "case-keeper", &request);
    let before = fs::read(root.path().join("cases/case-0001/field-records.jsonl")).unwrap();
    let second = run_as(root.path(), "case-keeper", &request);
    assert_eq!(second["replayed"], true);
    assert_eq!(second["record_id"], first["record_id"]);
    assert_eq!(
        before,
        fs::read(root.path().join("cases/case-0001/field-records.jsonl")).unwrap()
    );
    let mut other = request.clone();
    other["payload"]["reference"]["body"]["criteria"]["intention"] = json!("changed");
    assert_eq!(
        code(&run_as(root.path(), "case-keeper", &other)),
        "IDEMPOTENCY_CONFLICT"
    );
}

// ---------------------------------------------------------------------------
// Invariants with named negative tests
// ---------------------------------------------------------------------------

#[test]
fn nt_01_second_freeze_for_the_same_key_is_refused_whatever_its_digest() {
    let root = cell();
    let first = fixture("valid/request-episode-frozen.json");
    rt_register_for(root.path(), &first);
    assert_eq!(
        run_as(root.path(), "case-keeper", &first)["outcome"],
        "recorded"
    );
    // different criteria, same (case, item, n), new idempotency key
    let scenario = fixture("scenarios/i3-second-freeze-different-digest.json");
    let mut second = scenario["request"].clone();
    second["payload"]["reference"]["body"]["n"] =
        first["payload"]["reference"]["body"]["n"].clone();
    // recompute so only I-3 can refuse it
    let body = second["payload"]["reference"]["body"].clone();
    let digest = sea_forge_server::sfwp::case_records::digest_of(&body).unwrap();
    let key = &first["payload"]["reference"]["body"];
    let eref = format!(
        "episode:{}:{}:{}@sha256:{digest}",
        key["case_id"].as_str().unwrap(),
        key["plan_item_id"].as_str().unwrap(),
        key["n"]
    );
    second["payload"]["episode_ref"] = json!(eref);
    second["payload"]["reference"]["episode_ref"] = json!(eref);
    second["payload"]["digest"] = json!(digest);
    second["idempotency_key"] = json!("case-keeper/other");
    rt_register_for(root.path(), &second);
    let out = run_as(root.path(), "case-keeper", &second);
    assert_eq!(code(&out), "FREEZE_KEY_EXISTS", "{out}");
    assert_eq!(out["invariant"], "I-3");
    // nothing was written for it
    let records =
        fs::read_to_string(root.path().join("cases/case-0001/field-records.jsonl")).unwrap();
    assert_eq!(records.lines().count(), 1);
}

#[test]
fn nt_13_s_or_f_on_an_exploratory_episode_is_refused_u_is_accepted() {
    let root = cell();
    let freeze = fixture("valid/request-episode-frozen-exploratory.json");
    rt_register_for(root.path(), &freeze);
    run_as(root.path(), "case-keeper", &freeze);
    let mut settle = fixture("valid/request-episode-settled-exploratory-unresolved.json");
    for (result, ok) in [("S", false), ("F", false), ("U", true)] {
        settle["payload"]["result"] = json!(result);
        settle["idempotency_key"] = json!(format!("verifier/{result}"));
        let out = run_as(root.path(), "verifier", &settle);
        assert_eq!(out["outcome"] == "recorded", ok, "{result}: {out}");
        if !ok {
            assert_eq!(code(&out), "EXPLORATORY_NOT_SETTLEABLE");
        }
    }
}

#[test]
fn i_1_settle_needs_a_freeze_and_a_freeze_needs_its_realitytrace_records() {
    let root = cell();
    let settle = fixture("valid/request-episode-settled.json");
    let out = run_as(root.path(), "verifier", &settle);
    assert_eq!(
        (code(&out), out["invariant"].as_str()),
        ("FREEZE_MISSING", Some("I-1"))
    );
    // NT-33(b) shape at the verb: the record may not be written before the RT records exist
    let freeze = fixture("valid/request-episode-frozen.json");
    let out = run_as(root.path(), "case-keeper", &freeze);
    assert_eq!(
        (code(&out), out["invariant"].as_str()),
        ("FREEZE_INCOMPLETE", Some("I-1"))
    );
    assert!(!root
        .path()
        .join("cases/case-0001/field-records.jsonl")
        .exists());
}

#[test]
fn i_2_the_executor_in_the_settling_chain_cannot_settle() {
    let root = cell();
    let freeze = fixture("valid/request-episode-frozen.json");
    rt_register_for(root.path(), &freeze);
    run_as(root.path(), "case-keeper", &freeze);
    let item = freeze["payload"]["reference"]["body"]["plan_item_id"]
        .as_str()
        .unwrap()
        .to_string();
    write_event(
        root.path(),
        &item,
        "agent:builder-01",
        "2026-11-12T10:00:00Z",
    );
    let settle = fixture("valid/request-episode-settled.json");
    // the executor itself
    let out = run_as(root.path(), "agent:builder-01", &settle);
    assert_eq!(
        (code(&out), out["invariant"].as_str()),
        ("SELF_SETTLEMENT_REFUSED", Some("I-2")),
        "{out}"
    );
    // a settler whose delegation chain contains the executor
    let chain = vec![
        "human:operator".to_string(),
        "agent:builder-01".to_string(),
        "agent:verifier".to_string(),
    ];
    let out = run(root.path(), "agent:verifier", &chain, &settle);
    assert_eq!(code(&out), "SELF_SETTLEMENT_REFUSED", "{out}");
    // an independent principal settles
    assert_eq!(
        run_as(root.path(), "agent:verifier", &settle)["outcome"],
        "recorded"
    );
}

// ---------------------------------------------------------------------------
// Two-phase freeze, recovery, anchor
// ---------------------------------------------------------------------------

fn frozen_request() -> Value {
    fixture("valid/request-episode-frozen.json")
}

#[test]
fn crash_after_prepare_is_incomplete_then_recovered_by_replay_or_reconciler() {
    // replay path
    let root = cell();
    let request = frozen_request();
    rt_register_for(root.path(), &request);
    let idx = FileIndex(root.path().to_path_buf());
    let chain = vec!["case-keeper".to_string()];
    let c = ctx(root.path(), "case-keeper", &chain, &idx);
    assert_eq!(
        append_faulted(&c, &request, Some(Fault::AfterPrepare)),
        Err(CrashInjected(Fault::AfterPrepare))
    );
    let item = request["payload"]["reference"]["body"]["plan_item_id"]
        .as_str()
        .unwrap();
    assert_eq!(
        freeze_status(&c, item, 1),
        FreezeStatus::Incomplete(vec!["case_record", "anchor"]),
        "RealityTrace records exist, case record absent (NT-33a shape)"
    );
    let refused = require_complete_freeze(&c, item, 1).unwrap_err();
    assert_eq!(
        (code(&refused), refused["invariant"].as_str()),
        ("FREEZE_INCOMPLETE", Some("I-1"))
    );
    // a different key cannot take the half-held episode key
    let mut intruder = request.clone();
    intruder["idempotency_key"] = json!("case-keeper/intruder");
    assert_eq!(
        code(&run_as(root.path(), "case-keeper", &intruder)),
        "FREEZE_KEY_EXISTS"
    );
    let out = run_as(root.path(), "case-keeper", &request);
    assert_eq!(out["outcome"], "recorded", "{out}");
    let rt = fs::read_to_string(root.path().join("cases/case-0001/field-rt-outbox.jsonl")).unwrap();
    let ids: Vec<Value> = rt
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap()["record_id"].clone())
        .collect();
    let unique: std::collections::BTreeSet<String> = ids.iter().map(|v| v.to_string()).collect();
    assert_eq!(
        ids.len(),
        unique.len(),
        "outbox dedups on (system, record_id)"
    );
    assert_eq!(
        fs::read_to_string(root.path().join("cases/case-0001/field-records.jsonl"))
            .unwrap()
            .lines()
            .count(),
        1
    );

    // reconciler path, crash after the record is durable
    let root = cell();
    rt_register_for(root.path(), &request);
    let idx = FileIndex(root.path().to_path_buf());
    let c = ctx(root.path(), "case-keeper", &chain, &idx);
    assert_eq!(
        append_faulted(&c, &request, Some(Fault::AfterRecord)),
        Err(CrashInjected(Fault::AfterRecord))
    );
    assert_eq!(
        recover(&c),
        Recovered {
            completed: vec![request["idempotency_key"].as_str().unwrap().to_string()],
            abandoned: vec![]
        }
    );
    assert_eq!(
        fs::read_to_string(root.path().join("cases/case-0001/field-records.jsonl"))
            .unwrap()
            .lines()
            .count(),
        1,
        "no second record"
    );
    assert_eq!(recover(&c), Recovered::default(), "idempotent");
}

#[test]
fn a_prepared_freeze_whose_digest_cannot_be_completed_is_abandoned_and_n_plus_one_is_free() {
    let root = cell();
    let request = frozen_request();
    rt_register_for(root.path(), &request);
    let chain = vec!["case-keeper".to_string()];
    let idx = FileIndex(root.path().to_path_buf());
    let c = ctx(root.path(), "case-keeper", &chain, &idx);
    assert!(append_faulted(&c, &request, Some(Fault::AfterPrepare)).is_err());
    // the RealityTrace records disappear: the keeper cannot complete it
    fs::remove_file(root.path().join("realitytrace/records.jsonl")).unwrap();
    let out = recover(&c);
    assert_eq!(out.abandoned.len(), 1);
    let item = request["payload"]["reference"]["body"]["plan_item_id"]
        .as_str()
        .unwrap();
    assert_eq!(
        freeze_status(&c, item, 1),
        FreezeStatus::Missing,
        "abandoned intents do not count; nothing deleted"
    );
    assert!(fs::read_to_string(
        root.path()
            .join("cases/case-0001/field-freeze-intents.jsonl")
    )
    .unwrap()
    .contains("abandoned"));
    // n = 1 is free to be frozen again under a new key once its records exist
    rt_register_for(root.path(), &request);
    let mut again = request.clone();
    again["idempotency_key"] = json!("case-keeper/after-abandon");
    assert_eq!(
        run_as(root.path(), "case-keeper", &again)["outcome"],
        "recorded"
    );
}

fn anchored_freeze(root: &Path, push_time: &str) -> (Value, String) {
    let request = frozen_request();
    rt_register_for(root, &request);
    let chain = vec!["case-keeper".to_string()];
    let idx = FileIndex(root.to_path_buf());
    let publisher = Published(json!({"server_push_time": push_time, "receipt_id": "r-1"}));
    let c = AppendContext {
        root,
        case_id: "case-0001",
        principal: "case-keeper",
        role: "operator",
        chain: &chain,
        rt: &idx,
        anchor: Some(&publisher),
    };
    assert_eq!(append(&c, &request)["outcome"], "recorded");
    let item = request["payload"]["reference"]["body"]["plan_item_id"]
        .as_str()
        .unwrap()
        .to_string();
    (request, item)
}

#[test]
fn at_04_freeze_is_complete_only_with_record_realitytrace_and_anchor_and_precedes_the_first_run() {
    let root = cell();
    let request = frozen_request();
    rt_register_for(root.path(), &request);
    let chain = vec!["case-keeper".to_string()];
    let idx = FileIndex(root.path().to_path_buf());
    let item = request["payload"]["reference"]["body"]["plan_item_id"]
        .as_str()
        .unwrap()
        .to_string();
    // (c) record and RealityTrace exist, anchor missing: refused at the gate
    let c = ctx(root.path(), "case-keeper", &chain, &idx);
    assert_eq!(append(&c, &request)["outcome"], "recorded");
    assert_eq!(
        freeze_status(&c, &item, 1),
        FreezeStatus::Incomplete(vec!["anchor"])
    );
    assert_eq!(
        code(&require_complete_freeze(&c, &item, 1).unwrap_err()),
        "FREEZE_INCOMPLETE"
    );
    // a failing remote leaves it pending and the case record untouched
    assert_eq!(deliver_anchors(&c, &Failing), 0);
    assert_eq!(
        freeze_status(&c, &item, 1),
        FreezeStatus::Incomplete(vec!["anchor"])
    );
    // (b) record exists, RealityTrace record absent
    let other = cell();
    rt_register_for(other.path(), &request);
    let idx2 = FileIndex(other.path().to_path_buf());
    let c2 = ctx(other.path(), "case-keeper", &chain, &idx2);
    append(&c2, &request);
    fs::remove_file(other.path().join("realitytrace/records.jsonl")).unwrap();
    assert!(
        matches!(freeze_status(&c2, &item, 1), FreezeStatus::Incomplete(parts) if parts.contains(&"realitytrace_records"))
    );
    // anchored: complete, and the order check passes when the push precedes the first run
    let root = cell();
    let (_, item) = anchored_freeze(root.path(), "2026-11-12T09:00:00Z");
    write_event(
        root.path(),
        &item,
        "agent:builder-01",
        "2026-11-12T10:00:00Z",
    );
    let idx = FileIndex(root.path().to_path_buf());
    let c = ctx(root.path(), "case-keeper", &chain, &idx);
    assert_eq!(freeze_status(&c, &item, 1), FreezeStatus::Complete);
    assert_eq!(require_complete_freeze(&c, &item, 1), Ok(()));
    assert_eq!(
        verify_freeze_anchor(&c, &item, 1),
        Ok(AnchorOrder::ExternallyVerified)
    );
    // freeze order in the ledger: the freeze record is the first record of the case
    let first =
        fs::read_to_string(root.path().join("cases/case-0001/field-records.jsonl")).unwrap();
    assert!(first.lines().next().unwrap().contains("episode.frozen"));
}

#[test]
fn nt_16_an_anchor_absent_or_not_before_the_first_run_fails_whatever_the_chain_says() {
    let chain = vec!["case-keeper".to_string()];
    // anchor pushed after the first run record (a regenerated chain with the
    // freeze inserted early does not change the remote's push time)
    let root = cell();
    let (_, item) = anchored_freeze(root.path(), "2026-11-12T11:00:00Z");
    write_event(
        root.path(),
        &item,
        "agent:builder-01",
        "2026-11-12T10:00:00Z",
    );
    let idx = FileIndex(root.path().to_path_buf());
    let c = ctx(root.path(), "case-keeper", &chain, &idx);
    let out = verify_freeze_anchor(&c, &item, 1).unwrap_err();
    assert_eq!(code(&out), "ANCHOR_PREDATES_FAILED", "{out}");
    // anchor absent
    let root = cell();
    let request = frozen_request();
    rt_register_for(root.path(), &request);
    let idx = FileIndex(root.path().to_path_buf());
    let c = ctx(root.path(), "case-keeper", &chain, &idx);
    append(&c, &request);
    write_event(
        root.path(),
        &item,
        "agent:builder-01",
        "2026-11-12T10:00:00Z",
    );
    assert_eq!(
        code(&verify_freeze_anchor(&c, &item, 1).unwrap_err()),
        "ANCHOR_PREDATES_FAILED"
    );
}

#[test]
fn contract_digests_helper_agrees_with_fixture_digest() {
    // jcs/digest_of is the digest the fixtures were authored with.
    let request = frozen_request();
    let body = &request["payload"]["reference"]["body"];
    assert_eq!(
        sea_forge_server::sfwp::case_records::digest_of(body).unwrap(),
        request["payload"]["digest"].as_str().unwrap()
    );
}

// ---------------------------------------------------------------------------
// The SFWP verb over a real socket
// ---------------------------------------------------------------------------

#[cfg(unix)]
#[tokio::test]
async fn the_verb_answers_over_the_socket_with_the_contract_shapes() {
    use sea_forge_server::identity::IdentityBindings;
    use sea_forge_server::{run, ServerConfig};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let root = cell();
    let request = frozen_request();
    rt_register_for(root.path(), &request);
    let socket = std::env::temp_dir().join(format!("sf-wp14-{}.sock", std::process::id()));
    let config = ServerConfig {
        socket_path: socket.clone(),
        root: root.path().to_path_buf(),
        identity: IdentityBindings::local_operator("operator_local"),
        ..ServerConfig::default()
    };
    tokio::spawn(async move {
        let _ = run(config).await;
    });
    for _ in 0..500 {
        if socket.exists() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let (reader, mut writer) = UnixStream::connect(&socket).await.unwrap().into_split();
    let mut reader = BufReader::new(reader);
    let frame = |n: &str, req: &Value| {
        json!({"verb": "case_record_append", "case_id": "case-0001", "kind": req["kind"],
               "payload": req["payload"], "idempotency_key": req["idempotency_key"],
               "request_id": n, "actor": {"actor_id": "operator_local", "role": "operator"}})
    };
    async fn call<W: tokio::io::AsyncWrite + Unpin, R: tokio::io::AsyncBufRead + Unpin>(
        writer: &mut W,
        reader: &mut R,
        frame: Value,
    ) -> Value {
        writer
            .write_all(format!("{frame}\n").as_bytes())
            .await
            .unwrap();
        let mut out = String::new();
        reader.read_line(&mut out).await.unwrap();
        serde_json::from_str(out.trim()).unwrap()
    }
    let first = call(&mut writer, &mut reader, frame("req-1", &request)).await;
    assert_eq!(first["outcome"], "recorded", "{first}");
    let second = call(&mut writer, &mut reader, frame("req-2", &request)).await;
    assert_eq!(second["replayed"], true, "{second}");
    assert_eq!(second["record_id"], first["record_id"]);
    let _ = std::fs::remove_file(&socket);
}

// ---------------------------------------------------------------------------
// Review fixes: F2 (torn tail, corrupt lines)
// ---------------------------------------------------------------------------

fn records_path(root: &Path) -> PathBuf {
    root.join("cases/case-0001/field-records.jsonl")
}

#[test]
fn f2_torn_tail_is_repaired_before_append_and_the_answer_is_true() {
    let root = cell();
    let freeze = fixture("valid/request-episode-frozen.json");
    rt_register_for(root.path(), &freeze);
    assert_eq!(
        run_as(root.path(), "case-keeper", &freeze)["outcome"],
        "recorded"
    );
    let before = fs::read(records_path(root.path())).unwrap();
    let mut torn = before.clone();
    torn.extend_from_slice(br#"{"seq":2,"kind":"episode.sett"#);
    fs::write(records_path(root.path()), &torn).unwrap();
    let settle = fixture("valid/request-episode-settled.json");
    let out = run_as(root.path(), "agent:verifier", &settle);
    assert_eq!(out["outcome"], "recorded", "{out}");
    let text = fs::read_to_string(records_path(root.path())).unwrap();
    let lines: Vec<Value> = text
        .lines()
        .map(|l| serde_json::from_str(l).expect("every stored line parses"))
        .collect();
    assert_eq!(lines.len(), 2, "{text}");
    assert_eq!(lines[1]["kind"], "episode.settled");
    assert_eq!(lines[1]["record_id"], out["record_id"]);
    // The repair is durable and names what was dropped.
    let repairs =
        fs::read_to_string(root.path().join("cases/case-0001/field-repairs.jsonl")).unwrap();
    assert!(repairs.contains("field-records.jsonl") && repairs.contains("episode.sett"));
    // The verb's "recorded" is true: a replay finds the record.
    let again = run_as(root.path(), "agent:verifier", &settle);
    assert_eq!(
        (again["outcome"].as_str(), again["replayed"].as_bool()),
        (Some("recorded"), Some(true))
    );
    assert_eq!(again["record_id"], out["record_id"]);
}

#[test]
fn f2_a_corrupt_complete_line_refuses_and_writes_nothing() {
    let root = cell();
    let freeze = fixture("valid/request-episode-frozen.json");
    rt_register_for(root.path(), &freeze);
    run_as(root.path(), "case-keeper", &freeze);
    let mut bytes = fs::read(records_path(root.path())).unwrap();
    bytes.extend_from_slice(b"this is not json\n");
    fs::write(records_path(root.path()), &bytes).unwrap();
    let dir = root.path().join("cases/case-0001");
    let snapshot = |d: &Path| {
        let mut v: Vec<(String, Vec<u8>)> = fs::read_dir(d)
            .unwrap()
            .map(|e| {
                let e = e.unwrap();
                (
                    e.file_name().to_string_lossy().to_string(),
                    fs::read(e.path()).unwrap(),
                )
            })
            .collect();
        v.sort();
        v
    };
    let before = snapshot(&dir);
    let settle = fixture("valid/request-episode-settled.json");
    let out = run_as(root.path(), "agent:verifier", &settle);
    assert_eq!(out["outcome"], "refused", "{out}");
    assert!(
        out["message"].as_str().unwrap().contains("integrity"),
        "{out}"
    );
    assert_eq!(snapshot(&dir), before, "a refused verb writes nothing");
    assert_ne!(
        freeze_status(
            &ctx(root.path(), "x", &[], &FileIndex(root.path().into())),
            "item-03",
            1
        ),
        FreezeStatus::Complete,
        "a store that fails its integrity check is never a complete freeze"
    );
}

// ---------------------------------------------------------------------------
// Review fixes: F1 (settlement needs a complete freeze), F6 (one settlement),
// F7 (I-2 fails closed)
// ---------------------------------------------------------------------------

fn settle_request() -> Value {
    fixture("valid/request-episode-settled.json")
}

#[test]
fn f1_settlement_refuses_a_freeze_that_is_only_prepared_or_not_committed() {
    for fault in [Fault::AfterPrepare, Fault::AfterRecord] {
        let root = cell();
        let freeze = frozen_request();
        rt_register_for(root.path(), &freeze);
        let idx = FileIndex(root.path().to_path_buf());
        let chain = vec!["case-keeper".to_string()];
        let c = ctx(root.path(), "case-keeper", &chain, &idx);
        assert!(append_faulted(&c, &freeze, Some(fault)).is_err());
        let out = run_as(root.path(), "agent:verifier", &settle_request());
        assert_eq!(
            (code(&out), out["invariant"].as_str()),
            ("FREEZE_INCOMPLETE", Some("I-1")),
            "{fault:?}: {out}"
        );
        assert!(
            !fs::read_to_string(records_path(root.path()))
                .unwrap_or_default()
                .contains("episode.settled"),
            "nothing is written by the refusal"
        );
        // Completing the freeze (replay) lets the settlement through.
        assert_eq!(
            run_as(root.path(), "case-keeper", &freeze)["outcome"],
            "recorded"
        );
        assert_eq!(
            run_as(root.path(), "agent:verifier", &settle_request())["outcome"],
            "recorded"
        );
    }
}

#[test]
fn f1_with_an_anchor_configured_settlement_needs_the_published_anchor() {
    let root = cell();
    let freeze = frozen_request();
    rt_register_for(root.path(), &freeze);
    let idx = FileIndex(root.path().to_path_buf());
    let chain = vec!["agent:verifier".to_string()];
    let failing = Failing;
    let c = AppendContext {
        root: root.path(),
        case_id: "case-0001",
        principal: "agent:verifier",
        role: "operator",
        chain: &chain,
        rt: &idx,
        anchor: Some(&failing),
    };
    assert_eq!(append(&c, &freeze)["outcome"], "recorded");
    let out = append(&c, &settle_request());
    assert_eq!(
        (code(&out), out["invariant"].as_str()),
        ("FREEZE_INCOMPLETE", Some("I-1")),
        "{out}"
    );
    let ok = Published(json!({"server_push_time": "2026-11-12T09:00:00Z"}));
    let c2 = AppendContext {
        anchor: Some(&ok),
        ..c
    };
    assert_eq!(deliver_anchors(&c2, &ok), 1);
    assert_eq!(append(&c2, &settle_request())["outcome"], "recorded");
}

#[test]
fn f6_a_second_settlement_under_another_key_is_refused_same_key_replays() {
    let root = cell();
    let freeze = frozen_request();
    rt_register_for(root.path(), &freeze);
    run_as(root.path(), "case-keeper", &freeze);
    let first = run_as(root.path(), "agent:verifier", &settle_request());
    assert_eq!(first["outcome"], "recorded", "{first}");
    let mut second = settle_request();
    second["idempotency_key"] = json!("verifier/rec-0200");
    second["payload"]["result"] = json!("U");
    let out = run_as(root.path(), "agent:verifier", &second);
    assert_eq!(code(&out), "IDEMPOTENCY_CONFLICT", "{out}");
    let settled = fs::read_to_string(records_path(root.path()))
        .unwrap()
        .lines()
        .filter(|l| serde_json::from_str::<Value>(l).unwrap()["kind"] == "episode.settled")
        .count();
    assert_eq!(settled, 1, "only one settlement is stored");
    let again = run_as(root.path(), "agent:verifier", &settle_request());
    assert_eq!(again["replayed"], true);
    assert_eq!(again["record_id"], first["record_id"]);
}

fn settle_after_freeze(root: &Path) -> Value {
    let freeze = frozen_request();
    rt_register_for(root, &freeze);
    run_as(root, "case-keeper", &freeze);
    run_as(root, "agent:verifier", &settle_request())
}

#[test]
fn f7_missing_unreadable_or_identity_less_events_refuse_settlement() {
    let item = "item-03";
    let events = |root: &Path| root.join("cases/case-0001/case-events.jsonl");
    // missing file
    let root = cell();
    fs::remove_file(events(root.path())).unwrap();
    let out = settle_after_freeze(root.path());
    assert_eq!(out["outcome"], "refused", "missing events: {out}");
    // corrupt complete line
    let root = cell();
    fs::write(events(root.path()), "not json\n").unwrap();
    assert_eq!(settle_after_freeze(root.path())["outcome"], "refused");
    // event without actor_id
    let root = cell();
    fs::write(
        events(root.path()),
        format!(
            "{}\n",
            json!({"event_id": "e1", "plan_item_id": item, "timestamp": "2026-11-12T10:00:00Z"})
        ),
    )
    .unwrap();
    assert_eq!(settle_after_freeze(root.path())["outcome"], "refused");
    // non-string plan_item_id
    let root = cell();
    fs::write(
        events(root.path()),
        format!(
            "{}\n",
            json!({"event_id": "e1", "plan_item_id": 7, "actor_id": "a"})
        ),
    )
    .unwrap();
    assert_eq!(settle_after_freeze(root.path())["outcome"], "refused");
    // events with a case-level (null) item and an identity are fine
    let root = cell();
    fs::write(
        events(root.path()),
        format!(
            "{}\n",
            json!({"event_id": "e1", "plan_item_id": null, "actor_id": "a"})
        ),
    )
    .unwrap();
    assert_eq!(settle_after_freeze(root.path())["outcome"], "recorded");
}

#[test]
fn f7_executor_spelling_variants_do_not_evade_i_2() {
    let root = cell();
    let freeze = frozen_request();
    rt_register_for(root.path(), &freeze);
    run_as(root.path(), "case-keeper", &freeze);
    write_event(
        root.path(),
        "item-03",
        "  Agent:Builder-01 ",
        "2026-11-12T10:00:00Z",
    );
    let out = run_as(root.path(), "agent:builder-01", &settle_request());
    assert_eq!(code(&out), "SELF_SETTLEMENT_REFUSED", "{out}");
}

#[test]
fn f7_an_unverified_or_placeholder_principal_cannot_write() {
    let root = cell();
    let freeze = frozen_request();
    rt_register_for(root.path(), &freeze);
    for who in ["", "  ", "unknown", "none"] {
        let out = run_as(root.path(), who, &freeze);
        assert_eq!(out["outcome"], "refused", "{who:?}: {out}");
    }
    assert!(!records_path(root.path()).exists());
}

#[cfg(unix)]
#[tokio::test]
async fn f7_production_entry_point_enforces_i_2_and_fails_closed_on_identity_less_events() {
    use sea_forge_server::identity::IdentityBindings;
    use sea_forge_server::{run, ServerConfig};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let root = cell();
    let freeze = frozen_request();
    rt_register_for(root.path(), &freeze);
    let socket = std::env::temp_dir().join(format!("sf-wp14-i2-{}.sock", std::process::id()));
    let config = ServerConfig {
        socket_path: socket.clone(),
        root: root.path().to_path_buf(),
        identity: IdentityBindings::local_operator("operator_local"),
        ..ServerConfig::default()
    };
    tokio::spawn(async move {
        let _ = run(config).await;
    });
    for _ in 0..500 {
        if socket.exists() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let (reader, mut writer) = UnixStream::connect(&socket).await.unwrap().into_split();
    let mut reader = BufReader::new(reader);
    let mut n = 0;
    let mut call = |req: Value| {
        n += 1;
        let frame = json!({"verb": "case_record_append", "case_id": "case-0001", "kind": req["kind"],
               "payload": req["payload"], "idempotency_key": req["idempotency_key"],
               "request_id": format!("req-{n}"), "actor": {"actor_id": "operator_local", "role": "operator"}});
        (frame, ())
    };
    macro_rules! send {
        ($req:expr) => {{
            let (frame, _) = call($req);
            writer
                .write_all(format!("{frame}\n").as_bytes())
                .await
                .unwrap();
            let mut out = String::new();
            reader.read_line(&mut out).await.unwrap();
            serde_json::from_str::<Value>(out.trim()).unwrap()
        }};
    }
    assert_eq!(send!(freeze.clone())["outcome"], "recorded");
    // The server's real chain for this request is [operator_local]; the
    // executor of the item is the same identity, in another spelling.
    write_event(
        root.path(),
        "item-03",
        " Operator_Local",
        "2026-11-12T10:00:00Z",
    );
    let out = send!(settle_request());
    assert_eq!(code(&out), "SELF_SETTLEMENT_REFUSED", "{out}");
    // An event without an actor refuses instead of being skipped.
    fs::write(
        root.path().join("cases/case-0001/case-events.jsonl"),
        format!("{}\n", json!({"event_id": "e2", "plan_item_id": "item-03"})),
    )
    .unwrap();
    let mut other = settle_request();
    other["idempotency_key"] = json!("verifier/rec-0300");
    let out = send!(other);
    assert_eq!(out["outcome"], "refused", "{out}");
    assert_ne!(code(&out), "SELF_SETTLEMENT_REFUSED", "{out}");
    let _ = std::fs::remove_file(&socket);
}

// ---------------------------------------------------------------------------
// Review fixes: F5 (anchor receipt shape, explicit states, bounded publish)
// ---------------------------------------------------------------------------

#[cfg(unix)]
fn fake_script(dir: &Path, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("fde-anchor.sh");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn anchor_state(out: &Result<AnchorOrder, Value>) -> String {
    out.as_ref()
        .err()
        .map(|e| e["state"].as_str().unwrap_or("").to_string())
        .unwrap_or_default()
}

#[cfg(unix)]
#[test]
fn f5_what_the_script_prints_verifies_and_is_reported_local_time_only() {
    let root = cell();
    let tools = tempfile::tempdir().unwrap();
    let script = fake_script(
        tools.path(),
        "echo \"created fde-anchor/freeze/x on abc1234 (digest d)\"\necho \"tag_object_id 0123456789abcdef0123456789abcdef01234567\"\necho \"pushed fde-anchor/freeze/x\"",
    );
    let publisher = ScriptPublisher(script, std::time::Duration::from_secs(10));
    let request = frozen_request();
    rt_register_for(root.path(), &request);
    let idx = FileIndex(root.path().to_path_buf());
    let chain = vec!["case-keeper".to_string()];
    let c = AppendContext {
        root: root.path(),
        case_id: "case-0001",
        principal: "case-keeper",
        role: "operator",
        chain: &chain,
        rt: &idx,
        anchor: Some(&publisher),
    };
    assert_eq!(append(&c, &request)["outcome"], "recorded");
    let item = "item-03";
    assert_eq!(freeze_status(&c, item, 1), FreezeStatus::Complete);
    write_event(
        root.path(),
        item,
        "agent:builder-01",
        "2099-01-01T00:00:00Z",
    );
    assert_eq!(
        verify_freeze_anchor(&c, item, 1),
        Ok(AnchorOrder::AnchorTimeLocalOnly),
        "a local-clock push time is never reported as externally verified"
    );
}

#[cfg(unix)]
#[test]
fn f5_a_script_without_a_tag_object_id_or_that_hangs_publishes_nothing() {
    let tools = tempfile::tempdir().unwrap();
    let silent = ScriptPublisher(
        fake_script(tools.path(), "echo pushed"),
        std::time::Duration::from_secs(10),
    );
    assert!(silent.publish("freeze", "a1", &"0".repeat(64)).is_err());
    let tools2 = tempfile::tempdir().unwrap();
    let hang = ScriptPublisher(
        fake_script(tools2.path(), "sleep 30"),
        std::time::Duration::from_millis(500),
    );
    let started = std::time::Instant::now();
    let out = hang.publish("freeze", "a1", &"0".repeat(64));
    assert!(out.unwrap_err().contains("timed out"));
    assert!(started.elapsed() < std::time::Duration::from_secs(10));
}

#[test]
fn f5_each_failure_of_the_order_check_is_a_distinct_state() {
    let chain = vec!["case-keeper".to_string()];
    // no run events
    let root = cell();
    let (_, item) = anchored_freeze(root.path(), "2026-11-12T09:00:00Z");
    let idx = FileIndex(root.path().to_path_buf());
    let c = ctx(root.path(), "case-keeper", &chain, &idx);
    assert_eq!(
        anchor_state(&verify_freeze_anchor(&c, &item, 1)),
        "no_run_events"
    );
    // a run event without a usable timestamp
    let events = root.path().join("cases/case-0001/case-events.jsonl");
    fs::write(
        &events,
        format!("{}\n", json!({"plan_item_id": item, "actor_id": "a"})),
    )
    .unwrap();
    assert_eq!(
        anchor_state(&verify_freeze_anchor(&c, &item, 1)),
        "malformed_run_event"
    );
    // a corrupt events line
    fs::write(&events, "garbage\n").unwrap();
    assert_eq!(
        anchor_state(&verify_freeze_anchor(&c, &item, 1)),
        "malformed_run_event"
    );
    // anchor absent (record only)
    let root2 = cell();
    let request = frozen_request();
    rt_register_for(root2.path(), &request);
    let idx2 = FileIndex(root2.path().to_path_buf());
    let c2 = ctx(root2.path(), "case-keeper", &chain, &idx2);
    append(&c2, &request);
    assert_eq!(
        anchor_state(&verify_freeze_anchor(&c2, &item, 1)),
        "anchor_absent"
    );
    // receipt missing: published with no receipt
    let root3 = cell();
    rt_register_for(root3.path(), &request);
    let idx3 = FileIndex(root3.path().to_path_buf());
    let c3 = ctx(root3.path(), "case-keeper", &chain, &idx3);
    append(&c3, &request);
    let outbox = root3
        .path()
        .join("cases/case-0001/field-anchor-outbox.jsonl");
    let mut entry: Value =
        serde_json::from_str(fs::read_to_string(&outbox).unwrap().lines().next().unwrap()).unwrap();
    entry["state"] = json!("published");
    let mut text = fs::read_to_string(&outbox).unwrap();
    text.push_str(&format!("{entry}\n"));
    fs::write(&outbox, text).unwrap();
    assert_eq!(
        anchor_state(&verify_freeze_anchor(&c3, &item, 1)),
        "receipt_missing"
    );
    // no freeze record
    let root4 = cell();
    let idx4 = FileIndex(root4.path().to_path_buf());
    let c4 = ctx(root4.path(), "case-keeper", &chain, &idx4);
    assert_eq!(
        anchor_state(&verify_freeze_anchor(&c4, &item, 1)),
        "no_freeze_record"
    );
}

#[test]
fn f9_an_oversized_payload_is_refused_before_anything_is_written() {
    let root = cell();
    let mut request = frozen_request();
    request["payload"]["realitytrace_refs"]["question_ids"] = json!((0..40_000)
        .map(|i| format!("question:{i:08}"))
        .collect::<Vec<_>>());
    let out = run_as(root.path(), "case-keeper", &request);
    assert_eq!(code(&out), "SCHEMA_INVALID", "{out}");
    assert!(
        out["message"].as_str().unwrap().contains("exceeds"),
        "{out}"
    );
    assert!(!records_path(root.path()).exists());
    assert!(!root
        .path()
        .join("cases/case-0001/field-freeze-intents.jsonl")
        .exists());
}
