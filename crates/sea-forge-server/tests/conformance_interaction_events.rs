//! WP-14 part 2 / AT-05 (SEA-Forge half): semantic exhaust for
//! `case.record.append`. Every case ledger record and every authority decision
//! on the path yields exactly one schema-valid interaction event through a
//! durable outbox; delivery failure never blocks the write; replay is idempotent.

use sea_forge_server::sfwp::case_records::{append, AppendContext, FileIndex};
use sea_forge_server::sfwp::interaction_events::{
    declaration_key, deliver_events, reconcile_events, InteractionSink, SpoolSink, EXTENSION_SCHEMA,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};

fn contracts() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fde")
}

fn fixture(name: &str) -> Value {
    let path = contracts().join("fixtures/case-record-append").join(name);
    serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap()
}

fn cell() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("cases/case-0001");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("case.json"), "{}").unwrap();
    root
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
    let dir = root.join("realitytrace");
    fs::create_dir_all(&dir).unwrap();
    let mut text = fs::read_to_string(dir.join("records.jsonl")).unwrap_or_default();
    for id in ids {
        text.push_str(&format!("{}\n", json!({"record_id": id})));
    }
    fs::write(dir.join("records.jsonl"), text).unwrap();
}

fn run(root: &Path, principal: &str, request: &Value) -> Value {
    let idx = FileIndex(root.to_path_buf());
    let chain = vec![principal.to_string()];
    let ctx = AppendContext {
        root,
        case_id: "case-0001",
        principal,
        role: "operator",
        chain: &chain,
        rt: &idx,
        anchor: None,
    };
    append(&ctx, request)
}

fn with_ctx<T>(root: &Path, f: impl FnOnce(&AppendContext) -> T) -> T {
    let idx = FileIndex(root.to_path_buf());
    let chain = vec!["case-keeper".to_string()];
    f(&AppendContext {
        root,
        case_id: "case-0001",
        principal: "case-keeper",
        role: "operator",
        chain: &chain,
        rt: &idx,
        anchor: None,
    })
}

fn lines(root: &Path, file: &str) -> Vec<Value> {
    fs::read_to_string(root.join("cases/case-0001").join(file))
        .unwrap_or_default()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn pending_envelopes(root: &Path) -> Vec<Value> {
    lines(root, "field-interaction-outbox.jsonl")
        .into_iter()
        .filter(|r| r["state"] == "pending")
        .map(|r| r["envelope"].clone())
        .collect()
}

fn ext(envelope: &Value) -> &Value {
    &envelope["extensions"]["godspeed.interaction_event"]
}

fn validator() -> jsonschema::Validator {
    let schema: Value = serde_json::from_str(EXTENSION_SCHEMA).unwrap();
    jsonschema::validator_for(&schema).unwrap()
}

/// Freeze item-03 (evaluated), record an executor's event, settle as another
/// principal. Returns the root.
fn freeze_and_settle() -> tempfile::TempDir {
    let root = cell();
    let freeze = fixture("valid/request-episode-frozen.json");
    rt_register_for(root.path(), &freeze);
    assert_eq!(
        run(root.path(), "case-keeper", &freeze)["outcome"],
        "recorded"
    );
    let settle = fixture("valid/request-episode-settled.json");
    let out = run(root.path(), "human:verifier", &settle);
    assert_eq!(out["outcome"], "recorded", "{out}");
    root
}

struct Counting(RefCell<Vec<String>>);
impl InteractionSink for Counting {
    fn deliver(&self, event_id: &str, _: &Value) -> Result<(), String> {
        self.0.borrow_mut().push(event_id.to_string());
        Ok(())
    }
}
struct Failing;
impl InteractionSink for Failing {
    fn deliver(&self, _: &str, _: &Value) -> Result<(), String> {
        Err("emitter down".into())
    }
}

#[test]
fn vendored_extension_schema_matches_the_cep_digest() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("contracts/cep/DIGESTS"))
            .unwrap();
    let want = manifest.split_whitespace().next().unwrap();
    assert_eq!(
        hex::encode(Sha256::digest(EXTENSION_SCHEMA.as_bytes())),
        want
    );
    // When the CEP worktree is present, the vendored copy equals its source.
    let cep = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../fde-cep-integration/schemas/extensions/godspeed.interaction_event.v1.json");
    if let Ok(source) = fs::read_to_string(cep) {
        assert_eq!(source, EXTENSION_SCHEMA, "vendored copy drifted from CEP");
    }
}

#[test]
fn at_05_every_record_and_decision_yields_exactly_one_schema_valid_event() {
    let root = freeze_and_settle();
    let records = lines(root.path(), "field-records.jsonl");
    let decisions = lines(root.path(), "field-decisions.jsonl");
    assert_eq!((records.len(), decisions.len()), (2, 1));
    let envelopes = pending_envelopes(root.path());
    assert_eq!(
        envelopes.len(),
        3,
        "2 ledger records + 1 authority decision"
    );
    let v = validator();
    for env in &envelopes {
        let errors: Vec<String> = v.iter_errors(ext(env)).map(|e| e.to_string()).collect();
        assert!(errors.is_empty(), "{errors:?}\n{env}");
        assert_eq!(env["envelope_kind"], "execution_trace");
        assert_eq!(
            env["extensions"]["cep.profile"]["profile_id"],
            "godspeed.execution_trace"
        );
        // unsigned in S1 (E §5): no emitter block, no signature claim
        assert!(ext(env).get("emitter").is_none(), "{env}");
        // correlation is a structured object
        assert!(ext(env)["correlation"].is_object());
        assert_eq!(
            ext(env)["declaration_key"],
            declaration_key(ext(env)["semantic_ref"].as_str().unwrap())
        );
    }
    // one event per source record id, none repeated
    let mut ids: Vec<&str> = envelopes
        .iter()
        .map(|e| ext(e)["source"]["record_id"].as_str().unwrap())
        .collect();
    ids.sort();
    assert_eq!(ids, ["dec-0001", "rec-0001", "rec-0002"]);
}

#[test]
fn at_05_source_record_and_authority_decision_resolve_at_the_owner() {
    let root = freeze_and_settle();
    let records = lines(root.path(), "field-records.jsonl");
    let decisions = lines(root.path(), "field-decisions.jsonl");
    for env in pending_envelopes(root.path()) {
        let e = ext(&env);
        let (system, id, seq) = (
            e["source"]["system"].as_str().unwrap(),
            e["source"]["record_id"].as_str().unwrap(),
            e["source"]["sequence"].as_u64().unwrap(),
        );
        let owner = if system == "sea-forge.case-ledger" {
            &records
        } else {
            &decisions
        };
        let found = owner
            .iter()
            .find(|r| r["record_id"] == id || r["decision_id"] == id)
            .unwrap_or_else(|| panic!("{id} does not resolve at {system}"));
        assert_eq!(found["seq"], seq, "sequence is the owner's");
        let decision = e["authority"]["decision_id"].as_str().unwrap();
        if decision != "not_required" {
            assert!(
                decisions.iter().any(|d| d["decision_id"] == decision),
                "{decision} does not resolve"
            );
        }
    }
    // the freeze needs no authority decision on this path; the settlement cites one
    let by_op: Vec<(String, String)> = pending_envelopes(root.path())
        .iter()
        .map(|e| {
            (
                ext(e)["source"]["record_id"].as_str().unwrap().to_string(),
                ext(e)["authority"]["decision_id"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            )
        })
        .collect();
    assert!(by_op.contains(&("rec-0001".into(), "not_required".into())));
    assert!(by_op.contains(&("rec-0002".into(), "dec-0001".into())));
}

#[test]
fn at_05_semantic_refs_come_from_the_translation_map() {
    let root = freeze_and_settle();
    let refs: Vec<(String, String)> = pending_envelopes(root.path())
        .iter()
        .map(|e| {
            (
                ext(e)["source"]["record_id"].as_str().unwrap().to_string(),
                ext(e)["semantic_ref"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert!(refs.contains(&("rec-0001".into(), "godspeed-fde-sop::Episode".into())));
    assert!(refs.contains(&(
        "rec-0002".into(),
        "godspeed-fde-sop.evidence::Settlement".into()
    )));
    assert!(refs.contains(&(
        "dec-0001".into(),
        "godspeed-fde-sop::no_self_settlement".into()
    )));
}

#[test]
fn at_05_exploratory_event_is_valid_without_expectation_refs() {
    let root = cell();
    let freeze = fixture("valid/request-episode-frozen-exploratory.json");
    rt_register_for(root.path(), &freeze);
    assert_eq!(
        run(root.path(), "case-keeper", &freeze)["outcome"],
        "recorded"
    );
    let envs = pending_envelopes(root.path());
    assert_eq!(envs.len(), 1);
    assert_eq!(ext(&envs[0])["kind"], "exploratory");
    assert!(ext(&envs[0]).get("expectation_refs").is_none());
    assert_eq!(validator().iter_errors(ext(&envs[0])).count(), 0);
}

#[test]
fn at_05_denied_i2_decision_is_an_event_that_resolves_without_a_ledger_record() {
    let root = cell();
    let freeze = fixture("valid/request-episode-frozen.json");
    rt_register_for(root.path(), &freeze);
    run(root.path(), "case-keeper", &freeze);
    let item = freeze["payload"]["reference"]["body"]["plan_item_id"]
        .as_str()
        .unwrap();
    fs::write(
        root.path().join("cases/case-0001/case-events.jsonl"),
        format!(
            "{}\n",
            json!({"event_id": "ev-1", "plan_item_id": item, "actor_id": "agent:builder-01",
                   "kind": "command_started", "timestamp": "2026-11-12T10:00:00Z"})
        ),
    )
    .unwrap();
    let settle = fixture("valid/request-episode-settled.json");
    let out = run(root.path(), "agent:builder-01", &settle);
    assert_eq!(out["code"], "SELF_SETTLEMENT_REFUSED");
    assert_eq!(
        lines(root.path(), "field-records.jsonl").len(),
        1,
        "no settlement record"
    );
    let denied: Vec<Value> = pending_envelopes(root.path())
        .into_iter()
        .filter(|e| ext(e)["outcome"]["status"] == "denied")
        .collect();
    assert_eq!(denied.len(), 1);
    let e = ext(&denied[0]);
    assert_eq!(e["source"]["system"], "sea-forge.authority");
    assert_eq!(e["authority"]["decision_id"], e["source"]["record_id"]);
    assert_eq!(validator().iter_errors(e).count(), 0);
    // the same denied request again does not mint a second decision or event
    run(root.path(), "agent:builder-01", &settle);
    assert_eq!(lines(root.path(), "field-decisions.jsonl").len(), 1);
    assert_eq!(pending_envelopes(root.path()).len(), 2);
}

#[test]
fn injected_emitter_failure_leaves_the_write_committed_and_the_event_owed() {
    let root = freeze_and_settle();
    assert_eq!(
        lines(root.path(), "field-records.jsonl").len(),
        2,
        "writes committed"
    );
    let sent = with_ctx(root.path(), |c| deliver_events(c, &Failing));
    assert_eq!(sent, 0);
    let rows = lines(root.path(), "field-interaction-outbox.jsonl");
    assert_eq!(rows.iter().filter(|r| r["state"] == "pending").count(), 3);
    assert_eq!(
        rows.iter().filter(|r| r["state"] == "delivered").count(),
        0,
        "still owed"
    );
    // the emitter recovers: everything owed is delivered, once
    let sink = Counting(RefCell::new(vec![]));
    assert_eq!(with_ctx(root.path(), |c| deliver_events(c, &sink)), 3);
    assert_eq!(sink.0.borrow().len(), 3);
}

#[test]
fn an_outbox_write_failure_does_not_block_the_case_write_and_is_reconciled() {
    let root = cell();
    // make the outbox unwritable: a directory where the file should be
    fs::create_dir_all(
        root.path()
            .join("cases/case-0001/field-interaction-outbox.jsonl"),
    )
    .unwrap();
    let freeze = fixture("valid/request-episode-frozen.json");
    rt_register_for(root.path(), &freeze);
    let out = run(root.path(), "case-keeper", &freeze);
    assert_eq!(out["outcome"], "recorded", "{out}");
    assert_eq!(lines(root.path(), "field-records.jsonl").len(), 1);
    fs::remove_dir(
        root.path()
            .join("cases/case-0001/field-interaction-outbox.jsonl"),
    )
    .unwrap();
    assert_eq!(
        with_ctx(root.path(), reconcile_events),
        1,
        "the owed event is rebuilt from the ledger"
    );
    assert_eq!(
        with_ctx(root.path(), reconcile_events),
        0,
        "reconcile is idempotent"
    );
    assert_eq!(pending_envelopes(root.path()).len(), 1);
}

#[test]
fn replay_and_redelivery_do_not_duplicate_events() {
    let root = freeze_and_settle();
    // replaying both requests returns the originals and adds nothing
    let freeze = fixture("valid/request-episode-frozen.json");
    let settle = fixture("valid/request-episode-settled.json");
    assert_eq!(run(root.path(), "case-keeper", &freeze)["replayed"], true);
    assert_eq!(
        run(root.path(), "human:verifier", &settle)["replayed"],
        true
    );
    assert_eq!(pending_envelopes(root.path()).len(), 3);
    assert_eq!(lines(root.path(), "field-decisions.jsonl").len(), 1);
    // delivery to the spool is idempotent, also when the delivered mark is lost
    let sink = SpoolSink::new(root.path());
    assert_eq!(with_ctx(root.path(), |c| deliver_events(c, &sink)), 3);
    assert_eq!(with_ctx(root.path(), |c| deliver_events(c, &sink)), 0);
    let spool = root.path().join("interaction-events/case-0001.jsonl");
    assert_eq!(fs::read_to_string(&spool).unwrap().lines().count(), 3);
    let outbox = root
        .path()
        .join("cases/case-0001/field-interaction-outbox.jsonl");
    let kept: Vec<String> = fs::read_to_string(&outbox)
        .unwrap()
        .lines()
        .filter(|l| !l.contains("\"delivered\""))
        .map(String::from)
        .collect();
    fs::write(&outbox, kept.join("\n") + "\n").unwrap();
    assert_eq!(
        with_ctx(root.path(), |c| deliver_events(c, &sink)),
        3,
        "re-marked"
    );
    assert_eq!(
        fs::read_to_string(&spool).unwrap().lines().count(),
        3,
        "spool unchanged"
    );
}

#[test]
fn an_episode_in_another_world_records_a_gap_and_invents_no_semantic_ref() {
    let root = freeze_and_settle();
    let dir = root.path().join("cases/case-0001");
    let text = fs::read_to_string(dir.join("field-records.jsonl")).unwrap();
    fs::write(
        dir.join("field-records.jsonl"),
        text.replace("world:godspeed-fde-sop@", "world:other-world@"),
    )
    .unwrap();
    fs::remove_file(dir.join("field-interaction-outbox.jsonl")).unwrap();
    with_ctx(root.path(), reconcile_events);
    let rows = lines(root.path(), "field-interaction-outbox.jsonl");
    assert!(rows.iter().all(|r| r["state"] == "unmapped"), "{rows:?}");
    assert_eq!(rows.len(), 3);
    assert!(rows[0]["reason"]
        .as_str()
        .unwrap()
        .contains("no semantic_ref is invented"));
}
