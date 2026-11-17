//! `case_guard::admit_write` on its own, one rule per test (WP-21 part B,
//! review round 3). The guard decides from the ledger (the hash-chained
//! records and the event log), never from `case.json`, so these tests build a
//! Field Case on disk and propose writes straight at the guard. Each rule has
//! a test that fails when the rule is removed (see the mutation log).

use sea_forge_case_runner::case_ops::case_guard::{
    admit_file_write, admit_write, digest_of, CaseLock, Proposed,
};
use sea_forge_core::types::TraceKind;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const ZERO: &str = "0000000000000000000000000000000000000000000000000000000000000000";

struct Case {
    _dir: tempfile::TempDir,
    path: PathBuf,
    records: Vec<Value>,
}

fn item(id: &str, kind: &str) -> Value {
    let criteria =
        serde_json::to_value(sea_forge_core::types::SettlementCriteria::default()).unwrap();
    json!({"plan_item_id": id, "name": id, "settlement_criteria": criteria,
           "item_kind": kind, "markers": {"required": true}})
}

impl Case {
    fn new(items: Vec<Value>) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("case-0001");
        fs::create_dir_all(&path).unwrap();
        fs::write(
            path.join("plan.json"),
            json!({"version": "0.2", "plan_id": "p", "case_id": "case-0001", "run_id": "r",
                   "intent_id": "i", "items": items, "template_ref": "fde.field_case@1"})
            .to_string(),
        )
        .unwrap();
        fs::write(path.join("case-events.jsonl"), "").unwrap();
        fs::write(
            path.join("field_case.json"),
            sea_forge_core::field_case::opened_document("case-0001").to_string(),
        )
        .unwrap();
        Self {
            _dir: dir,
            path,
            records: Vec::new(),
        }
    }

    fn push(&mut self, mut record: Value) {
        let seq = self.records.len() as u64 + 1;
        let prev = self.records.last().map_or(ZERO.to_string(), |r| {
            r["hash"].as_str().unwrap().to_string()
        });
        if record.get("payload").is_some() {
            record["body_digest"] = json!(digest_of(
                &json!({"kind": record["kind"], "payload": record["payload"]})
            )
            .unwrap());
        }
        record["seq"] = json!(seq);
        record["record_id"] = json!(format!("rec-{seq:04}"));
        record["prev_hash"] = json!(prev);
        record["recorded_at"] = json!("2026-11-15T08:00:00.000Z");
        record["hash"] = json!(digest_of(&record).unwrap());
        self.records.push(record);
        self.flush();
    }

    fn flush(&self) {
        let text: String = self.records.iter().map(|r| format!("{r}\n")).collect();
        fs::write(self.path.join("field-records.jsonl"), text).unwrap();
    }

    fn disposition(&mut self, level: &str, d: &str) {
        self.push(
            json!({"kind": "disposition.recorded", "level": level, "disposition": d,
                         "payload": {"level": level, "disposition": d}}),
        );
    }

    fn assess(&mut self, gate: &str, state: &str) {
        self.push(
            json!({"kind": "gate.assessed", "gate": gate, "state": state,
                         "payload": {"gate": gate, "state": state}}),
        );
    }

    fn event(&self, kind: &str, item: &str) {
        let line = json!({"version": "0.2", "event_id": "cev", "run_id": "case",
                          "plan_item_id": item, "kind": kind, "actor_id": "a",
                          "timestamp": "2026-11-15T08:00:00Z", "payload": {}});
        let path = self.path.join("case-events.jsonl");
        let mut text = fs::read_to_string(&path).unwrap();
        text.push_str(&format!("{line}\n"));
        fs::write(path, text).unwrap();
    }

    fn admit(&self, kind: TraceKind, item: Option<&str>) -> Result<(), String> {
        admit_write(&self.path, &Proposed::Event { kind: &kind, item })
            .map(drop)
            .map_err(|r| r.code.to_string())
    }

    fn store(&self, file: &str) -> Result<(), String> {
        admit_write(&self.path, &Proposed::Store { file })
            .map(drop)
            .map_err(|r| r.code.to_string())
    }

    fn record(&self, record: Value) -> Result<(), String> {
        admit_write(&self.path, &Proposed::Record { record: &record })
            .map(drop)
            .map_err(|r| r.code.to_string())
    }

    /// The next record as the appender would write it (extends the chain).
    fn next(&self, mut r: Value) -> Value {
        r["seq"] = json!(self.records.len() as u64 + 1);
        r["prev_hash"] = json!(self.records.last().map_or(ZERO.to_string(), |x| x["hash"]
            .as_str()
            .unwrap()
            .to_string()));
        r
    }
}

fn err(code: &str) -> Result<(), String> {
    Err(code.to_string())
}

#[test]
fn an_open_case_admits_ordinary_work() {
    let c = Case::new(vec![item("task-a", "human_task")]);
    assert_eq!(c.admit(TraceKind::ItemActivated, Some("task-a")), Ok(()));
    assert_eq!(
        c.admit(TraceKind::HumanTaskCompleted, Some("task-a")),
        Ok(())
    );
    assert_eq!(c.store("field-runs.jsonl"), Ok(()));
}

// (a) lifecycle -------------------------------------------------------------

#[test]
fn suspended_refuses_every_event_and_execution_store() {
    let mut c = Case::new(vec![item("task-a", "human_task")]);
    c.disposition("case", "suspend");
    for kind in [
        TraceKind::ItemActivated,
        TraceKind::HumanTaskCompleted,
        TraceKind::ItemCompleted,
        TraceKind::RunStarted,
        TraceKind::PlanMutated,
        TraceKind::CaseTerminated,
        TraceKind::CaseClosed,
        TraceKind::CaseReopened,
    ] {
        assert_eq!(
            c.admit(kind.clone(), Some("task-a")),
            err("CASE_SUSPENDED"),
            "{kind:?}"
        );
    }
    assert_eq!(c.store("field-runs.jsonl"), err("CASE_SUSPENDED"));
    assert_eq!(c.store("field-assignments.jsonl"), err("CASE_SUSPENDED"));
    // Bookkeeping of an admitted record is not execution.
    assert_eq!(c.store("field-approvals.jsonl"), Ok(()));
}

#[test]
fn suspended_takes_only_the_records_the_contract_allows() {
    let mut c = Case::new(vec![item("task-a", "human_task")]);
    c.disposition("case", "suspend");
    let rec = |r: Value| c.record(c.next(r));
    assert_eq!(rec(json!({"kind": "gate.assessed"})), Ok(()));
    assert_eq!(rec(json!({"kind": "strategy.recorded"})), Ok(()));
    assert_eq!(rec(json!({"kind": "field_case.revised"})), Ok(()));
    assert_eq!(
        rec(json!({"kind": "disposition.recorded", "level": "case", "disposition": "continue"})),
        Ok(())
    );
    assert_eq!(
        rec(
            json!({"kind": "disposition.recorded", "level": "case", "disposition": "close-failed"})
        ),
        Ok(())
    );
    for refused in [
        json!({"kind": "episode.frozen"}),
        json!({"kind": "episode.settled"}),
        json!({"kind": "disposition.recorded", "level": "case", "disposition": "revise"}),
        json!({"kind": "disposition.recorded", "level": "episode", "disposition": "continue"}),
    ] {
        assert_eq!(rec(refused.clone()), err("CASE_SUSPENDED"), "{refused}");
    }
}

#[test]
fn closed_refuses_records_except_late_settlement() {
    let mut c = Case::new(vec![item("task-a", "human_task")]);
    c.disposition("case", "close-satisfied");
    let rec = |r: Value| c.record(c.next(r));
    assert_eq!(rec(json!({"kind": "episode.settled"})), Ok(()));
    for kind in [
        "episode.frozen",
        "gate.assessed",
        "field_case.revised",
        "strategy.recorded",
    ] {
        assert_eq!(rec(json!({"kind": kind})), err("CASE_CLOSED"), "{kind}");
    }
    assert_eq!(
        rec(json!({"kind": "disposition.recorded", "level": "case", "disposition": "continue"})),
        err("CASE_CLOSED")
    );
    assert_eq!(
        rec(json!({"kind": "disposition.recorded", "level": "case", "disposition": "suspend"})),
        err("CASE_CLOSED")
    );
}

#[test]
fn closed_takes_the_action_that_ends_the_plan_once() {
    let mut c = Case::new(vec![item("task-a", "human_task")]);
    c.disposition("case", "close-satisfied");
    // The plan is not finished: the final completion, then the close, once.
    assert_eq!(
        c.admit(TraceKind::HumanTaskCompleted, Some("task-a")),
        Ok(())
    );
    c.event("human_task_completed", "task-a");
    // A repeat of the action that ended it, with the plan now finished.
    assert_eq!(
        c.admit(TraceKind::HumanTaskCompleted, Some("task-a")),
        err("CASE_CLOSED")
    );
    assert_eq!(c.admit(TraceKind::RunStarted, None), err("CASE_CLOSED"));
    assert_eq!(c.store("field-runs.jsonl"), err("CASE_CLOSED"));
    // The case close the engine still owes is admitted once.
    assert_eq!(c.admit(TraceKind::CaseClosed, None), Ok(()));
    c.event("case_closed", "");
    assert_eq!(c.admit(TraceKind::CaseClosed, None), err("CASE_CLOSED"));
    assert_eq!(c.admit(TraceKind::CaseTerminated, None), err("CASE_CLOSED"));
}

#[test]
fn closed_does_not_repeat_an_item_already_done() {
    let mut c = Case::new(vec![
        item("task-a", "human_task"),
        item("task-b", "human_task"),
    ]);
    c.disposition("case", "close-satisfied");
    c.event("human_task_completed", "task-a");
    // The plan is not finished (task-b), but task-a is already done.
    assert_eq!(
        c.admit(TraceKind::HumanTaskCompleted, Some("task-a")),
        err("CASE_CLOSED")
    );
    assert_eq!(
        c.admit(TraceKind::HumanTaskCompleted, Some("task-b")),
        Ok(())
    );
}

#[test]
fn closed_refuses_plan_structure_and_reopen() {
    let mut c = Case::new(vec![item("task-a", "human_task")]);
    c.disposition("case", "close-satisfied");
    assert_eq!(
        c.admit(TraceKind::PlanMutated, Some("task-a")),
        err("CASE_CLOSED")
    );
    assert_eq!(
        c.admit(TraceKind::CaseFileItemAdded, None),
        err("CASE_CLOSED")
    );
    assert_eq!(c.admit(TraceKind::CaseReopened, None), err("CASE_CLOSED"));
}

// (b) the hard-gate rule, whatever the event type -----------------------------

#[test]
fn a_gate_item_completes_on_no_event_type_without_a_pass() {
    let mut c = Case::new(vec![item("gate_allocate_authority", "human_task")]);
    c.assess("allocate_authority", "blocked");
    for kind in [
        TraceKind::ItemCompleted,
        TraceKind::MilestoneAchieved,
        TraceKind::HumanTaskCompleted,
    ] {
        assert_eq!(
            c.admit(kind.clone(), Some("gate_allocate_authority")),
            err("HARD_GATE_NOT_WAIVABLE"),
            "{kind:?}"
        );
    }
    // A hard gate is not skipped either.
    assert_eq!(
        c.admit(TraceKind::ItemTerminated, Some("gate_allocate_authority")),
        err("HARD_GATE_NOT_WAIVABLE")
    );
    c.assess("allocate_authority", "met");
    assert_eq!(
        c.admit(
            TraceKind::HumanTaskCompleted,
            Some("gate_allocate_authority")
        ),
        Ok(())
    );
}

#[test]
fn a_soft_gate_may_be_skipped_but_not_completed_unassessed() {
    let c = Case::new(vec![item("gate_prove_value", "human_task")]);
    assert_eq!(
        c.admit(TraceKind::ItemTerminated, Some("gate_prove_value")),
        Ok(())
    );
    assert_eq!(
        c.admit(TraceKind::ItemCompleted, Some("gate_prove_value")),
        err("AUTHORITY_DENIED")
    );
}

// (c) R11 -------------------------------------------------------------------

#[test]
fn close_and_terminate_need_a_case_level_close_disposition() {
    let mut c = Case::new(vec![item("task-a", "human_task")]);
    assert_eq!(
        c.admit(TraceKind::CaseClosed, None),
        err("CLOSE_DISPOSITION_REQUIRED")
    );
    assert_eq!(
        c.admit(TraceKind::CaseTerminated, None),
        err("CLOSE_DISPOSITION_REQUIRED")
    );
    // An episode-level close or another case-level disposition is not one.
    c.disposition("episode", "close-satisfied");
    c.disposition("case", "continue");
    assert_eq!(
        c.admit(TraceKind::CaseTerminated, None),
        err("CLOSE_DISPOSITION_REQUIRED")
    );
    c.disposition("case", "close-failed");
    // Closed: only a write that ends a FINISHED plan is admitted (C2).
    assert_eq!(c.admit(TraceKind::CaseTerminated, None), err("CASE_CLOSED"));
    c.event("human_task_completed", "task-a");
    assert_eq!(c.admit(TraceKind::CaseTerminated, None), Ok(()));
}

// (d) the document, in either dialect --------------------------------------

#[test]
fn every_write_needs_the_document() {
    let mut c = Case::new(vec![item("task-a", "human_task")]);
    c.disposition("case", "continue");
    fs::remove_file(c.path.join("field_case.json")).unwrap();
    assert_eq!(
        c.admit(TraceKind::ItemActivated, Some("task-a")),
        err("STORE_INTEGRITY")
    );
    assert_eq!(c.store("field-runs.jsonl"), err("STORE_INTEGRITY"));
    assert_eq!(
        c.record(c.next(json!({"kind": "gate.assessed"}))),
        err("STORE_INTEGRITY")
    );
    // Another case's document, and a document ahead of its ledger.
    fs::write(
        c.path.join("field_case.json"),
        sea_forge_core::field_case::opened_document("other").to_string(),
    )
    .unwrap();
    assert_eq!(
        c.admit(TraceKind::ItemActivated, Some("task-a")),
        err("STORE_INTEGRITY")
    );
    fs::write(c.path.join("field_case.json"), {
        let mut d = sea_forge_core::field_case::opened_document("case-0001");
        d["revision"] = json!(9);
        d.to_string()
    })
    .unwrap();
    assert_eq!(
        c.admit(TraceKind::ItemActivated, Some("task-a")),
        err("STORE_INTEGRITY")
    );
}

#[test]
fn only_creation_runs_before_the_document() {
    let c = Case::new(vec![]);
    fs::remove_file(c.path.join("field_case.json")).unwrap();
    assert_eq!(c.admit(TraceKind::CaseCreated, None), Ok(()));
    assert_eq!(
        c.admit(TraceKind::ItemEnabled, Some("x")),
        err("STORE_INTEGRITY")
    );
    // Creation happens once: a second `case_created` is refused.
    c.event("case_created", "");
    assert_eq!(c.admit(TraceKind::CaseCreated, None), err("CASE_CLOSED"));
    assert_eq!(c.admit(TraceKind::PlanCreated, None), Ok(()));
}

// (e) full store integrity ---------------------------------------------------

#[test]
fn a_corrupt_record_refuses_every_write() {
    let mut c = Case::new(vec![item("task-a", "human_task")]);
    c.assess("choose_problem", "blocked");
    c.assess("choose_problem", "met");
    let good = c.records.clone();
    let mut mutate = |f: &dyn Fn(&mut Vec<Value>)| {
        let mut records = good.clone();
        f(&mut records);
        c.records = records;
        c.flush();
        let outcomes = [
            c.admit(TraceKind::ItemActivated, Some("task-a")),
            c.store("field-runs.jsonl"),
            c.record(json!({"kind": "gate.assessed", "seq": 3, "prev_hash": ZERO})),
        ];
        for outcome in outcomes {
            assert_eq!(outcome, err("STORE_INTEGRITY"));
        }
    };
    mutate(&|r| r[0]["hash"] = json!("ab".repeat(32)));
    mutate(&|r| r[1]["prev_hash"] = json!("cd".repeat(32)));
    mutate(&|r| r[1]["seq"] = json!(7));
    mutate(&|r| r[0]["state"] = json!("met")); // a field changed under its hash
                                               // A record re-sealed under its own hash with a wrong seq, or a wrong link:
                                               // only the chain checks catch these, not the digest.
    mutate(&|r| {
        r[1]["seq"] = json!(7);
        r[1].as_object_mut().unwrap().remove("hash");
        r[1]["hash"] = json!(digest_of(&r[1]).unwrap());
    });
    mutate(&|r| {
        r[1]["prev_hash"] = json!("cd".repeat(32));
        r[1].as_object_mut().unwrap().remove("hash");
        r[1]["hash"] = json!(digest_of(&r[1]).unwrap());
    });
    mutate(&|r| {
        // body_digest no longer the digest of kind and payload, hash re-sealed.
        r[0]["body_digest"] = json!("ef".repeat(32));
        r[0].as_object_mut().unwrap().remove("hash");
        r[0]["hash"] = json!(digest_of(&r[0]).unwrap());
        let h = r[0]["hash"].clone();
        r[1]["prev_hash"] = h;
        r[1].as_object_mut().unwrap().remove("hash");
        r[1]["hash"] = json!(digest_of(&r[1]).unwrap());
    });
}

#[test]
fn a_stale_record_is_refused() {
    let mut c = Case::new(vec![]);
    c.assess("choose_problem", "blocked");
    let stale = json!({"kind": "gate.assessed", "seq": 1, "prev_hash": ZERO});
    assert_eq!(c.record(stale), err("STORE_INTEGRITY"));
    assert_eq!(c.record(c.next(json!({"kind": "gate.assessed"}))), Ok(()));
}

// Deny by default ------------------------------------------------------------

#[test]
fn an_unknown_record_kind_or_store_file_is_refused() {
    let c = Case::new(vec![]);
    assert_eq!(
        c.record(c.next(json!({"kind": "invented.kind"}))),
        err("KIND_UNKNOWN")
    );
    assert_eq!(c.store("field-invented.jsonl"), err("KIND_UNKNOWN"));
    assert_eq!(c.store("case-events.jsonl"), err("KIND_UNKNOWN"));
}

#[test]
fn a_plain_case_is_not_touched() {
    let c = Case::new(vec![]);
    fs::write(
        c.path.join("plan.json"),
        json!({"version": "0.2", "plan_id": "p", "case_id": "case-0001", "run_id": "r",
               "intent_id": "i", "items": [], "template_ref": null})
        .to_string(),
    )
    .unwrap();
    // A plain case has no Field Case marker.
    fs::remove_file(c.path.join("field_case.json")).unwrap();
    assert_eq!(c.admit(TraceKind::CaseReopened, None), Ok(()));
    assert_eq!(c.store("whatever.jsonl"), Ok(()));
}

#[test]
fn a_path_outside_a_case_is_not_a_case_write() {
    let dir = tempfile::tempdir().unwrap();
    assert!(admit_file_write(&dir.path().join("x.jsonl"), &json!({}))
        .unwrap()
        .is_none());
}

// The lock -------------------------------------------------------------------

fn try_lock(path: &Path) -> bool {
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path.join(".case.lock"))
        .unwrap();
    match file.try_lock() {
        Ok(()) => true,
        Err(std::fs::TryLockError::WouldBlock) => false,
        Err(e) => panic!("{e}"),
    }
}

#[test]
fn the_token_holds_the_cross_process_lock_until_dropped() {
    let c = Case::new(vec![item("task-a", "human_task")]);
    let token = admit_write(
        &c.path,
        &Proposed::Event {
            kind: &TraceKind::ItemActivated,
            item: Some("task-a"),
        },
    )
    .unwrap();
    // Another file description of the same file (another process would be one).
    assert!(
        !try_lock(&c.path),
        "the guard and the append are one critical section"
    );
    drop(token);
    assert!(try_lock(&c.path), "released with the token");
}

#[test]
fn a_refused_write_releases_the_lock() {
    let mut c = Case::new(vec![item("task-a", "human_task")]);
    c.disposition("case", "suspend");
    assert!(c.admit(TraceKind::ItemActivated, Some("task-a")).is_err());
    assert!(try_lock(&c.path));
}

#[test]
fn the_lock_nests_on_one_thread_and_excludes_another() {
    let c = Case::new(vec![item("task-a", "human_task")]);
    let outer = CaseLock::acquire(&c.path).unwrap();
    // A verb that holds the lock and then appends does not deadlock itself.
    let inner = c.admit(TraceKind::ItemActivated, Some("task-a"));
    assert_eq!(inner, Ok(()));
    assert!(!try_lock(&c.path), "the verb still holds it");
    let path = c.path.clone();
    let waiter = std::thread::spawn(move || {
        let _t = admit_write(
            &path,
            &Proposed::Event {
                kind: &TraceKind::ItemActivated,
                item: Some("task-a"),
            },
        )
        .unwrap();
    });
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert!(!waiter.is_finished(), "another writer waits for the lock");
    drop(outer);
    waiter.join().unwrap();
    assert!(try_lock(&c.path));
}
