//! `case_guard::check` and `commit` on their own, one rule per test (WP-21 part B,
//! review round 3). The guard decides from the ledger (the hash-chained
//! records and the event log), never from `case.json`, so these tests build a
//! Field Case on disk and propose writes straight at the guard. Each rule has
//! a test that fails when the rule is removed (see the mutation log).

mod common;

use sea_forge_case_runner::case_ops::case_guard::{
    check, commit, commit_file, digest_of, Batch, CaseLock, Op,
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
            path.join("case.json"),
            json!({"version": "0.2", "case_id": "case-0001", "intent": {"intent_id": "i", "summary": "s",
                   "actor_id": "a", "process_id": "p", "created_at": "2026-11-15T08:00:00Z"},
                   "state": "active", "plan_ref": "plan.json", "run_ids": [], "stages": [],
                   "close_reason": null, "created_at": "2026-11-15T08:00:00Z", "closed_at": null})
            .to_string(),
        )
        .unwrap();
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
        if record.get("payload").is_none() {
            record["payload"] = common::payload_for(&record);
        }
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
        // The document is the projection of the ledger (the commit derives it).
        let doc = sea_forge_case_runner::case_ops::case_guard::derived_document(
            "case-0001",
            &self.records,
        );
        fs::write(self.path.join("field_case.json"), doc.to_string()).unwrap();
    }

    fn disposition(&mut self, level: &str, d: &str) {
        self.push(json!({"kind": "disposition.recorded", "level": level, "disposition": d}));
    }

    fn assess(&mut self, gate: &str, state: &str) {
        self.push(json!({"kind": "gate.assessed", "gate": gate, "state": state}));
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
        check(
            &self.path,
            &Batch::of_events(&[(kind, item.map(str::to_owned))]),
        )
        .map_err(|r| {
            if r.code == "STORE_INTEGRITY" {
                eprintln!("{}", r.message);
            }
            r.code.to_string()
        })
    }

    fn store(&self, file: &str) -> Result<(), String> {
        check(
            &self.path,
            &Batch::new().with(Op::Append {
                file: file.into(),
                line: common::store_line(),
            }),
        )
        .map_err(|r| {
            if r.code == "STORE_INTEGRITY" {
                eprintln!("{}", r.message);
            }
            r.code.to_string()
        })
    }

    fn record(&self, record: Value) -> Result<(), String> {
        check(&self.path, &Batch::new().with(Op::Record(record))).map_err(|r| {
            if r.code == "STORE_INTEGRITY" {
                eprintln!("{}", r.message);
            }
            r.code.to_string()
        })
    }

    /// The next record as the appender would write it (extends the chain).
    fn next(&self, mut r: Value) -> Value {
        if r.get("payload").is_none() {
            if r["kind"] == "gate.assessed" && r.get("gate").is_none() {
                r["gate"] = json!("prove_value");
                r["state"] = json!("met");
            }
            r["payload"] = common::payload_for(&r);
        }
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
fn closed_case_can_commit_termination_and_its_projection() {
    let mut c = Case::new(vec![item("task-a", "human_task")]);
    c.disposition("case", "close-failed");
    let mut projection: Value =
        serde_json::from_slice(&fs::read(c.path.join("case.json")).unwrap()).unwrap();
    projection["state"] = json!("terminated");
    let batch = Batch::of_events(&[(TraceKind::CaseTerminated, None)])
        .with(Op::Case(projection.clone()))
        .with(Op::Case(projection.clone()));

    commit(&c.path, batch).unwrap();
    let stored: Value =
        serde_json::from_slice(&fs::read(c.path.join("case.json")).unwrap()).unwrap();
    assert_eq!(stored, projection);
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
    // Closed (D42): the actions that end the plan. A close needs a finished
    // plan; a terminate is itself an ending, so the operator can always stop
    // a case whose close was `close-failed` on an unfinished plan.
    assert_eq!(c.admit(TraceKind::CaseClosed, None), err("CASE_CLOSED"));
    assert_eq!(c.admit(TraceKind::CaseTerminated, None), Ok(()));
    c.event("case_terminated", "");
    assert_eq!(c.admit(TraceKind::CaseTerminated, None), err("CASE_CLOSED"));
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
    assert!(!commit_file(&dir.path().join("x.jsonl"), &json!({})).unwrap());
}

// The lock -------------------------------------------------------------------

fn lock_files(case: &Case) -> Vec<PathBuf> {
    let p = sea_forge_case_runner::case_ops::case_guard::lock_path(&case.path);
    if p.exists() {
        vec![p]
    } else {
        vec![]
    }
}

fn try_lock(case: &Case) -> bool {
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(
            lock_files(case)
                .first()
                .expect("a lock file exists outside the case directory"),
        )
        .unwrap();
    match file.try_lock() {
        Ok(()) => true,
        Err(std::fs::TryLockError::WouldBlock) => false,
        Err(e) => panic!("{e}"),
    }
}

#[test]
fn the_lock_file_lives_outside_the_case_directory_and_a_refusal_creates_nothing_inside() {
    let mut c = Case::new(vec![item("task-a", "human_task")]);
    c.disposition("case", "suspend");
    let before = tree(&c.path);
    assert!(c.admit(TraceKind::ItemActivated, Some("task-a")).is_err());
    assert_eq!(
        tree(&c.path),
        before,
        "a refusal changes nothing under the case"
    );
    assert!(!lock_files(&c).is_empty());
    assert!(!c.path.join(".case.lock").exists());
}

fn tree(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out: Vec<(String, Vec<u8>)> = fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| {
            (
                e.file_name().to_string_lossy().to_string(),
                fs::read(e.path()).unwrap_or_default(),
            )
        })
        .collect();
    out.sort();
    out
}

#[test]
fn the_commit_holds_the_cross_process_lock_while_it_decides_and_releases_it() {
    let c = Case::new(vec![item("task-a", "human_task")]);
    let lock = CaseLock::acquire(&c.path).unwrap();
    assert!(!try_lock(&c), "held while the verb works");
    drop(lock);
    assert!(try_lock(&c), "released with the lock");
    assert!(c.admit(TraceKind::ItemActivated, Some("task-a")).is_ok());
    assert!(try_lock(&c), "a decision releases it");
}

#[test]
fn a_refused_write_releases_the_lock() {
    let mut c = Case::new(vec![item("task-a", "human_task")]);
    c.disposition("case", "suspend");
    assert!(c.admit(TraceKind::ItemActivated, Some("task-a")).is_err());
    assert!(try_lock(&c));
}

#[test]
fn the_lock_nests_on_one_thread_and_excludes_another() {
    let c = Case::new(vec![item("task-a", "human_task")]);
    let outer = CaseLock::acquire(&c.path).unwrap();
    // A verb that holds the lock and then commits does not deadlock itself.
    let inner = c.admit(TraceKind::ItemActivated, Some("task-a"));
    assert_eq!(inner, Ok(()));
    assert!(!try_lock(&c), "the verb still holds it");
    let path = c.path.clone();
    let waiter = std::thread::spawn(move || {
        check(
            &path,
            &Batch::of_events(&[(TraceKind::ItemActivated, Some("task-a".into()))]),
        )
        .unwrap();
    });
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert!(!waiter.is_finished(), "another writer waits for the lock");
    drop(outer);
    waiter.join().unwrap();
    assert!(try_lock(&c));
}

#[test]
fn commit_writes_the_batch_and_derives_the_document() {
    let c = Case::new(vec![item("task-a", "human_task")]);
    // A field_case.revised record is committed; the document follows in the
    // same commit, from the ledger.
    let mut r = json!({"kind": "field_case.revised", "revision": 2});
    r["payload"] = common::payload_for(&r);
    let r = c.next(r);
    let mut r = r;
    r["body_digest"] =
        json!(digest_of(&json!({"kind": r["kind"], "payload": r["payload"]})).unwrap());
    r["hash"] = json!(digest_of(&r).unwrap());
    commit(&c.path, Batch::new().with(Op::Record(r))).unwrap();
    let doc: Value =
        serde_json::from_slice(&fs::read(c.path.join("field_case.json")).unwrap()).unwrap();
    assert_eq!(doc["revision"], 2);
    assert_eq!(doc["fields"]["purpose_and_outcome"]["signal"]["value"], "x");
}

#[test]
fn a_document_with_another_member_is_corrupt() {
    let mut c = Case::new(vec![item("task-a", "human_task")]);
    c.disposition("case", "continue");
    let mut doc = sea_forge_core::field_case::opened_document("case-0001");
    doc["extra"] = json!(1);
    fs::write(c.path.join("field_case.json"), doc.to_string()).unwrap();
    assert_eq!(
        c.admit(TraceKind::ItemActivated, Some("task-a")),
        err("STORE_INTEGRITY")
    );
}

#[test]
fn a_case_record_store_alone_marks_a_field_case() {
    // No document, a plan that does not parse: only the records say Field Case.
    let mut c = Case::new(vec![item("task-a", "human_task")]);
    c.disposition("case", "continue");
    fs::remove_file(c.path.join("field_case.json")).unwrap();
    fs::write(c.path.join("plan.json"), "{not json").unwrap();
    assert_eq!(
        c.admit(TraceKind::ItemActivated, Some("task-a")),
        err("STORE_INTEGRITY")
    );
}

// (f) residual detection and lock re-entrance --------------------------------

fn committed_case() -> Case {
    let c = Case::new(vec![item("task-a", "human_task")]);
    sea_forge_case_runner::case_ops::case_guard::commit(
        &c.path,
        sea_forge_case_runner::case_ops::case_guard::Batch::of_events(&[(
            TraceKind::ItemActivated,
            Some("task-a".into()),
        )]),
    )
    .unwrap();
    c
}

#[test]
fn the_case_index_detects_truncated_and_wiped_stores() {
    use sea_forge_case_runner::case_ops::case_guard::index_path;
    let c = committed_case();
    assert!(
        index_path(&c.path).is_file(),
        "every commit updates the index"
    );
    assert_eq!(
        c.admit(TraceKind::HumanTaskCompleted, Some("task-a")),
        Ok(())
    );
    // The event log is truncated after the commit.
    fs::write(c.path.join("case-events.jsonl"), "").unwrap();
    assert_eq!(
        c.admit(TraceKind::HumanTaskCompleted, Some("task-a")),
        err("STORE_INTEGRITY")
    );
    // Every field* file is wiped.
    let c = committed_case();
    for e in fs::read_dir(&c.path).unwrap().flatten() {
        if e.file_name().to_string_lossy().starts_with("field") {
            fs::remove_file(e.path()).unwrap();
        }
    }
    let plan = c.path.join("plan.json");
    let mut p: Value = serde_json::from_slice(&fs::read(&plan).unwrap()).unwrap();
    p["template_ref"] = json!(null);
    fs::write(&plan, p.to_string()).unwrap();
    assert_eq!(
        c.admit(TraceKind::HumanTaskCompleted, Some("task-a")),
        err("STORE_INTEGRITY")
    );
}

#[test]
fn a_nested_commit_joins_the_outer_lock_and_does_not_deadlock() {
    use sea_forge_case_runner::case_ops::case_guard::{commit, Batch, CaseLock};
    let c = committed_case();
    let path = c.path.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _outer = CaseLock::acquire(&path).unwrap();
        let r = commit(
            &path,
            Batch::of_events(&[(TraceKind::HumanTaskCompleted, Some("task-a".into()))]),
        );
        let _ = tx.send(r.is_ok());
    });
    assert_eq!(
        rx.recv_timeout(std::time::Duration::from_secs(5)),
        Ok(true),
        "a nested commit on the same case deadlocked or failed"
    );
}

#[test]
fn commit_error_conversions_keep_uncertainty_out_of_refusals() {
    use sea_forge_case_runner::case_ops::{case_guard::CommitError, field_guard::Refusal};
    use sea_forge_core::errors::ForgeError;

    let refusal = Refusal {
        code: "CASE_CLOSED",
        message: "closed".into(),
    };
    let error: ForgeError = CommitError::from(refusal).into();
    match error {
        ForgeError::Plan { class, message } => {
            assert_eq!(class, "episode_gate_refused");
            let value: serde_json::Value = serde_json::from_str(&message).unwrap();
            assert_eq!(value["no_side_effect"], true);
        }
        other => panic!("expected refusal, got {other:?}"),
    }
    let error: ForgeError =
        CommitError::PublicationUncertain(std::io::Error::other("sync failed")).into();
    match error {
        ForgeError::Io { context, source } => {
            assert!(context.contains("may have changed"));
            assert_eq!(source.to_string(), "sync failed");
        }
        other => panic!("expected operational error, got {other:?}"),
    }
}
