//! Stateful model test of the Field Case write path (WP-21 part B, review
//! rounds 4 and 5). Random sequences of every verb run against one temp Field
//! Case, including corruption of every store kind, deletion of markers,
//! suspension and the `close-failed` then terminate sequence. After EVERY step
//! the invariants of `root-cause-classes-2026-11-17.md` are checked from the
//! files on disk, independently of the guard:
//!
//! - C1 no terminal state without a case-level close disposition;
//! - C2 after a close disposition only plan-ending actions and the kinds the
//!   registry accepts when closed land;
//! - C3 no gate item or milestone completes without a passing assessment;
//! - C4 a corrupt store accepts no write and changes no byte;
//! - C5 a refused verb changes no byte under the case directory;
//! - C6 a suspended case accepts only what the registry accepts suspended;
//! - C7 writer coverage, observed at runtime: an inotify watch on the case
//!   directory sees exactly the files `commit` journaled, no more, no fewer;
//! - the product rule: after `close-failed` the operator can terminate.
//!
//! `MODEL_CASES` sets the case count (default 256, the CI count; 5,000 locally).

mod common;

use inotify::{Inotify, WatchMask};
use proptest::prelude::*;
use sea_forge_case_runner::case_ops::{
    self,
    advance::AdvanceScope,
    case_guard::{commit, digest_of, journal_begin, journal_end, Batch, Op},
};
use sea_forge_core::types::*;
use sea_forge_core::RECORD_VERSION;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const CASE: &str = "case-0001";
const ZERO: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const ITEMS: [&str; 4] = [
    "task-a",
    "task-b",
    "gate_prove_value",
    "gate_allocate_authority",
];
const OUTBOXES: [&str; 3] = [
    "field-rt-outbox.jsonl",
    "field-anchor-outbox.jsonl",
    "field-interaction-outbox.jsonl",
];
/// Every auxiliary store: the file and a valid line.
const STORES: [&str; 11] = [
    "field-freeze-intents.jsonl",
    "field-decisions.jsonl",
    "field-approvals.jsonl",
    "field-rt-outbox.jsonl",
    "field-anchor-outbox.jsonl",
    "field-interaction-outbox.jsonl",
    "field-runs.jsonl",
    "field-links.jsonl",
    "field-assignments.jsonl",
    "field-exec-decisions.jsonl",
    "field-repairs.jsonl",
];

fn item(id: &str, kind: ItemKind, depends_on: &[&str]) -> PlanItem {
    PlanItem {
        external_ref: None,
        plan_item_id: id.into(),
        name: id.into(),
        operations: vec![],
        entry_criteria: vec![],
        entry_criteria_mode: EntryCriteriaMode::Any,
        exit_criteria: vec![],
        settlement_criteria: SettlementCriteria::default(),
        settlement_criteria_ref: None,
        item_kind: kind,
        sandbox_class: None,
        parent_stage: None,
        markers: ItemMarkers {
            required: true,
            ..Default::default()
        },
        max_instances: 1,
        depends_on: depends_on.iter().map(|s| s.to_string()).collect(),
        environment: None,
        proposed_by: None,
    }
}

#[derive(Debug, Clone)]
enum Step {
    Human(usize),
    Advance,
    Terminate,
    Reopen,
    Assess(usize, usize),
    Dispose(usize),
    Record(usize),
    /// A line appended to auxiliary store `i` of [`STORES`] (repairs excluded).
    Store(usize),
    /// `case.json` written with a state (active, completed, terminated, awaiting).
    CaseWrite(usize),
    Corrupt(usize),
    DeleteMarker(usize),
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        4 => (0..ITEMS.len()).prop_map(Step::Human),
        2 => Just(Step::Advance),
        2 => Just(Step::Terminate),
        1 => Just(Step::Reopen),
        3 => (0..2usize, 0..3usize).prop_map(|(g, s)| Step::Assess(g, s)),
        4 => (0..6usize).prop_map(Step::Dispose),
        2 => (0..4usize).prop_map(Step::Record),
        3 => (0..10usize).prop_map(Step::Store),
        2 => (0..4usize).prop_map(Step::CaseWrite),
        2 => (0..16usize).prop_map(Step::Corrupt),
        1 => (0..4usize).prop_map(Step::DeleteMarker),
    ]
}

struct World {
    _dir: tempfile::TempDir,
    root: PathBuf,
    case_dir: PathBuf,
    corrupted: bool,
}

fn case_value() -> Value {
    serde_json::to_value(Case {
        version: RECORD_VERSION.into(),
        case_id: CASE.into(),
        intent: Intent {
            intent_id: "int_1".into(),
            summary: "model".into(),
            actor_id: "operator_local".into(),
            process_id: "proc_1".into(),
            created_at: "2026-09-23T00:00:00Z".into(),
        },
        state: CaseState::Active,
        plan_ref: "plan.json".into(),
        run_ids: vec![],
        stages: vec![],
        close_reason: None,
        created_at: "2026-09-23T00:00:00Z".into(),
        closed_at: None,
    })
    .unwrap()
}

impl World {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let case_dir = root.join("cases").join(CASE);
        let plan = CasePlan {
            version: "0.2".into(),
            plan_id: "plan_1".into(),
            case_id: CASE.into(),
            run_id: "run_1".into(),
            intent_id: "int_1".into(),
            items: vec![
                item("task-a", ItemKind::HumanTask, &[]),
                item("task-b", ItemKind::HumanTask, &[]),
                item("gate_prove_value", ItemKind::Milestone, &["task-a"]),
                item("gate_allocate_authority", ItemKind::HumanTask, &[]),
            ],
            template_ref: Some("fde.field_case@1".into()),
            job_contract_ref: None,
        };
        // The case is created the way the writers create it: through the guard.
        let created = TraceEvent {
            version: RECORD_VERSION.into(),
            event_id: "cev-000001".into(),
            run_id: "case".into(),
            plan_item_id: None,
            kind: TraceKind::CaseCreated,
            actor_id: "operator_local".into(),
            timestamp: "2026-09-23T00:00:00Z".into(),
            payload: json!({}),
            cell_id: None,
        };
        commit(
            &case_dir,
            Batch::new()
                .with(Op::Case(case_value()))
                .with(Op::Plan(serde_json::to_value(&plan).unwrap()))
                .with(Op::Event(created)),
        )
        .unwrap();
        let rules: String = ["human_task_completion", "case_terminate", "case_reopen"]
            .iter()
            .map(|k| format!("  - name: allow-{k}\n    verdict: allow\n    actor_role: operator\n    operation_kind: {k}\n"))
            .collect();
        fs::write(
            root.join("policy.yaml"),
            format!("version: \"0.1\"\nrules:\n{rules}"),
        )
        .unwrap();
        Self {
            _dir: dir,
            root,
            case_dir,
            corrupted: false,
        }
    }

    fn policy(&self) -> PathBuf {
        self.root.join("policy.yaml")
    }

    /// Every byte under the case directory.
    fn tree_hash(&self) -> String {
        fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
            for e in fs::read_dir(dir).into_iter().flatten().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else {
                    out.push(p);
                }
            }
        }
        let mut files = Vec::new();
        walk(&self.case_dir, &mut files);
        files.sort();
        let mut h = Sha256::new();
        for f in files {
            h.update(
                f.strip_prefix(&self.case_dir)
                    .unwrap()
                    .to_string_lossy()
                    .as_bytes(),
            );
            h.update(fs::read(&f).unwrap_or_default());
        }
        hex::encode(h.finalize())
    }

    fn records(&self) -> Vec<Value> {
        read_lines(&self.case_dir.join("field-records.jsonl"))
    }

    fn events(&self) -> Vec<Value> {
        read_lines(&self.case_dir.join("case-events.jsonl"))
    }

    fn case_state(&self) -> String {
        fs::read(self.case_dir.join("case.json"))
            .ok()
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
            .and_then(|c| c["state"].as_str().map(String::from))
            .unwrap_or_default()
    }

    fn ledger_revision(&self) -> u64 {
        self.records()
            .iter()
            .filter(|r| r["kind"] == "field_case.revised")
            .filter_map(|r| r["revision"].as_u64())
            .max()
            .unwrap_or(1)
    }

    /// The record verb, as the server's appender does it: the next link of the
    /// chain, committed through THE guard.
    fn append_record(&self, mut record: Value) -> Result<(), String> {
        let existing = self.records();
        if record.get("payload").is_none() {
            record["payload"] = common::payload_for(&record);
        }
        record["body_digest"] = json!(digest_of(
            &json!({"kind": record["kind"], "payload": record["payload"]})
        )
        .unwrap());
        record["seq"] = json!(existing.len() as u64 + 1);
        record["record_id"] = json!(format!("rec-{:04}", existing.len() + 1));
        record["prev_hash"] = json!(existing
            .last()
            .and_then(|r| r["hash"].as_str())
            .unwrap_or(ZERO));
        record["recorded_at"] = json!("2026-11-15T08:00:00.000Z");
        record["hash"] = json!(digest_of(&record).unwrap());
        commit(&self.case_dir, Batch::new().with(Op::Record(record)))
            .map(drop)
            .map_err(|r| r.code.to_string())
    }
}

fn read_lines(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Life {
    Open,
    Suspended,
    Closed,
}

fn life(records: &[Value]) -> Life {
    let mut state = Life::Open;
    for r in records {
        if r["kind"] != "disposition.recorded" || r["level"] != "case" {
            continue;
        }
        let d = r["disposition"].as_str().unwrap_or("");
        if d.starts_with("close-") {
            return Life::Closed;
        }
        state = if d == "suspend" {
            Life::Suspended
        } else {
            Life::Open
        };
    }
    state
}

fn oracle_plan(w: &World) -> CasePlan {
    serde_json::from_slice(&fs::read(w.root.join("oracle-plan.json")).unwrap()).unwrap()
}

fn plan_finished(w: &World, events: &[Value]) -> bool {
    let plan = oracle_plan(w);
    let evs: Vec<TraceEvent> = events
        .iter()
        .filter_map(|e| serde_json::from_value(e.clone()).ok())
        .collect();
    sea_forge_planner::case_engine::replay_case(&plan.items, &evs)
        .items
        .iter()
        .all(|s| {
            matches!(
                s.status,
                sea_forge_planner::case_engine::ItemStatus::Completed
                    | sea_forge_planner::case_engine::ItemStatus::Terminated
                    | sea_forge_planner::case_engine::ItemStatus::Failed
            )
        })
}

fn gate_of(item: &str) -> Option<&str> {
    item.strip_prefix("gate_")
}

fn store_line(file: &str, n: usize) -> Value {
    match file {
        "field-freeze-intents.jsonl" => {
            json!({"idempotency_key": format!("k{n}"), "state": "prepared"})
        }
        "field-decisions.jsonl" => json!({"decision_id": format!("dec-{n}"), "decision": "allow"}),
        "field-approvals.jsonl" => json!({"approval_ref": format!("a{n}"), "consumed_by": "rec-1"}),
        "field-rt-outbox.jsonl" => {
            json!({"system": "realitytrace", "record_id": format!("r{n}"), "state": "pending"})
        }
        "field-anchor-outbox.jsonl" => json!({"id": format!("an{n}"), "state": "pending"}),
        "field-interaction-outbox.jsonl" => {
            json!({"event_id": format!("ev{n}"), "state": "pending"})
        }
        "field-runs.jsonl" => {
            json!({"run_id": format!("run-{n}"), "case_id": CASE, "episode_ref": "e", "verb": "item_execute"})
        }
        "field-links.jsonl" => {
            json!({"link_id": format!("link-{n}"), "case_id": CASE, "delegate": "d"})
        }
        "field-assignments.jsonl" => json!({"assignment_id": format!("as-{n}"), "state": "issued"}),
        _ => json!({"decision_id": format!("ed-{n}"), "run_id": "run-1"}),
    }
}

/// Run one step; returns the verb's outcome (`Ok` accepted, `Err` refused).
fn run(w: &World, s: &Step, n: usize) -> Result<(), String> {
    let policy = w.policy();
    match s {
        Step::Human(i) => {
            case_ops::complete_human_task(&w.root, &policy, "operator_local", CASE, ITEMS[*i], None)
                .map(drop)
                .map_err(|e| e.to_string())
        }
        Step::Advance => {
            let mut exec = |_: &PlanItem, _: &str| -> Result<SettlementEvent, _> {
                Err(sea_forge_core::errors::ForgeError::Internal(
                    "no episode".into(),
                ))
            };
            case_ops::advance::advance_case(
                &w.root,
                "operator_local",
                CASE,
                AdvanceScope::All,
                &mut |_| {},
                &mut exec,
            )
            .map(drop)
            .map_err(|e| e.to_string())
        }
        Step::Terminate => case_ops::terminate(&w.root, &policy, "operator_local", CASE, "stop")
            .map(drop)
            .map_err(|e| e.to_string()),
        Step::Reopen => case_ops::reopen(&w.root, &policy, "operator_local", CASE)
            .map(drop)
            .map_err(|e| e.to_string()),
        Step::Assess(g, st) => {
            let gate = ["prove_value", "allocate_authority"][*g];
            let state = ["met", "blocked", "not_applicable"][*st];
            w.append_record(json!({"kind": "gate.assessed", "gate": gate, "state": state}))
        }
        Step::Dispose(d) => {
            let d = [
                "close-satisfied",
                "continue",
                "suspend",
                "revise",
                "close-failed",
                "escalate",
            ][*d];
            w.append_record(json!({"kind": "disposition.recorded", "level": "case",
                                   "disposition": d}))
        }
        Step::Record(k) => {
            let kind = [
                "strategy.recorded",
                "field_case.revised",
                "episode.settled",
                "episode.frozen",
            ][*k];
            let mut record = json!({"kind": kind});
            if kind == "field_case.revised" {
                let base = w.ledger_revision();
                record["revision"] = json!(base + 1);
                let mut payload = common::payload_for(&record);
                payload["base_revision"] = json!(base);
                record["payload"] = payload;
            }
            w.append_record(record)
        }
        Step::Store(i) => commit(
            &w.case_dir,
            Batch::new().with(Op::Append {
                file: STORES[*i].into(),
                line: store_line(STORES[*i], n),
            }),
        )
        .map(drop)
        .map_err(|r| r.code.to_string()),
        Step::CaseWrite(k) => {
            let mut case = case_value();
            case["state"] = json!(["active", "completed", "terminated", "awaiting_approval"][*k]);
            sea_forge_case_runner::write_json(&w.case_dir.join("case.json"), &case)
                .map_err(|e| e.to_string())
        }
        Step::Corrupt(_) | Step::DeleteMarker(_) => unreachable!("handled by the caller"),
    }
}

fn append_garbage(path: PathBuf) {
    let mut text = fs::read_to_string(&path).unwrap_or_default();
    text.push_str("{garbage\n");
    fs::write(path, text).unwrap();
}

/// Corrupt one store kind: the plan, `case.json`, the event log, the document,
/// the record chain, or any auxiliary store.
fn corrupt(w: &mut World, k: usize) {
    let d = &w.case_dir;
    match k {
        0 => fs::write(d.join("plan.json"), "{not json").unwrap(),
        1 => fs::write(d.join("case.json"), "[]").unwrap(),
        2 => append_garbage(d.join("case-events.jsonl")),
        3 => fs::write(d.join("field_case.json"), "[]").unwrap(),
        4 => {
            let path = d.join("field-records.jsonl");
            let text = fs::read_to_string(&path).unwrap_or_default();
            match text.lines().next() {
                Some(first) if serde_json::from_str::<Value>(first).is_err() => {
                    append_garbage(path)
                }
                Some(first) => {
                    let mut r: Value = serde_json::from_str(first).unwrap();
                    r["hash"] = json!("ab".repeat(32));
                    let rest: Vec<&str> = text.lines().skip(1).collect();
                    let mut out = format!("{r}\n");
                    for l in rest {
                        out.push_str(l);
                        out.push('\n');
                    }
                    fs::write(&path, out).unwrap();
                }
                None => append_garbage(path),
            }
        }
        _ => append_garbage(d.join(STORES[(k - 5) % STORES.len()])),
    }
    w.corrupted = true;
}

/// Delete markers: the plan, the document, both, or `case.json`. (The ledger
/// file is itself a marker, created with the case; deleting it too is the
/// total wipe nothing can detect.)
fn delete_marker(w: &mut World, k: usize) {
    let d = &w.case_dir;
    match k {
        0 => drop(fs::remove_file(d.join("plan.json"))),
        1 => drop(fs::remove_file(d.join("field_case.json"))),
        2 => {
            drop(fs::remove_file(d.join("plan.json")));
            drop(fs::remove_file(d.join("field_case.json")));
        }
        _ => drop(fs::remove_file(d.join("case.json"))),
    }
    w.corrupted = true;
}

struct Watcher {
    ino: Inotify,
}

impl Watcher {
    fn new(dir: &Path) -> Self {
        let ino = Inotify::init().unwrap();
        ino.watches()
            .add(
                dir,
                WatchMask::CREATE
                    | WatchMask::MODIFY
                    | WatchMask::MOVED_TO
                    | WatchMask::MOVED_FROM
                    | WatchMask::DELETE
                    | WatchMask::CLOSE_WRITE,
            )
            .unwrap();
        Self { ino }
    }

    /// The files modified since the last drain (directories are not files).
    fn drain(&mut self) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        let mut buf = [0u8; 8192];
        loop {
            match self.ino.read_events(&mut buf) {
                Ok(events) => {
                    let mut any = false;
                    for e in events {
                        any = true;
                        if !e.mask.contains(inotify::EventMask::ISDIR) {
                            if let Some(name) = e.name {
                                out.insert(name.to_string_lossy().to_string());
                            }
                        }
                    }
                    if !any {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        out
    }
}

struct Snapshot {
    hash: String,
    events: usize,
    records: usize,
    life: Life,
    corrupted: bool,
    ended: bool,
    finished: bool,
    case_state: String,
}

/// The invariants, after one step.
fn check(
    w: &World,
    s: &Step,
    outcome: &Result<(), String>,
    before: &Snapshot,
    observed: &BTreeSet<String>,
    journaled: &BTreeSet<String>,
) -> Result<(), String> {
    let after = w.tree_hash();
    let records = w.records();
    let events = w.events();
    let new_events = &events[before.events.min(events.len())..];
    let new_records = &records[before.records.min(records.len())..];
    // C7: the files modified under the case directory are exactly the files
    // `commit` journaled.
    if observed != journaled {
        return Err(format!(
            "C7: observed writes {observed:?} differ from the committed writes {journaled:?} ({s:?}): {outcome:?}"
        ));
    }
    // C4: a corrupt store accepts no write and changes no byte.
    if before.corrupted && (after != before.hash || outcome.is_ok()) {
        return Err(format!(
            "C4: a corrupt store accepted a write ({s:?}): {outcome:?}"
        ));
    }
    // C5: a refused verb changes no byte under the case directory.
    if outcome.is_err() && after != before.hash {
        return Err(format!(
            "C5: a refused verb changed bytes ({s:?}): {outcome:?}"
        ));
    }
    // C1: terminal state or case-end event only behind a close disposition.
    let ended = events
        .iter()
        .any(|e| e["kind"] == "case_closed" || e["kind"] == "case_terminated");
    if (ended || matches!(w.case_state().as_str(), "completed" | "terminated"))
        && life(&records) != Life::Closed
    {
        return Err(format!(
            "C1: terminal with no close disposition ({s:?}): {outcome:?}"
        ));
    }
    // C2 (README D42 and the registry's Closed column): on a closed case an
    // accepted step lands only plan-ending actions, the kinds the registry
    // accepts when closed and their stores.
    if before.life == Life::Closed && outcome.is_ok() && !before.corrupted {
        let ends_plan = new_events.iter().any(|e| {
            e["kind"] == "case_terminated"
                || e["kind"] == "case_closed"
                || plan_finished(w, &events)
        });
        if !new_events.is_empty() && !ends_plan {
            return Err(format!(
                "C2: a closed case progressed without ending ({s:?}): {outcome:?}"
            ));
        }
        if new_records.iter().any(|r| r["kind"] != "episode.settled") {
            return Err(format!("C2: a closed case took a record ({s:?})"));
        }
        let closed_files = [
            "field-records.jsonl",
            "case-events.jsonl",
            "case.json",
            "field-decisions.jsonl",
            "field-approvals.jsonl",
            "field-rt-outbox.jsonl",
            "field-anchor-outbox.jsonl",
            "field-interaction-outbox.jsonl",
            "field-runs.jsonl",
            "field-links.jsonl",
            "field-assignments.jsonl",
            "field-exec-decisions.jsonl",
        ];
        if let Some(f) = observed
            .iter()
            .find(|f| !closed_files.contains(&f.as_str()))
        {
            return Err(format!("C2: a closed case wrote {f} ({s:?})"));
        }
        // Bookkeeping of a kind the registry refuses when closed.
        if observed.contains("field-freeze-intents.jsonl") {
            return Err(format!("C2: freeze bookkeeping on a closed case ({s:?})"));
        }
        // Execution state only while an action that ends the plan is possible.
        let exec = [
            "field-runs.jsonl",
            "field-links.jsonl",
            "field-assignments.jsonl",
            "field-exec-decisions.jsonl",
        ];
        if observed.iter().any(|f| exec.contains(&f.as_str())) && (before.ended || before.finished)
        {
            return Err(format!(
                "C2: execution state on an ended closed case ({s:?})"
            ));
        }
        // case.json moves only to a terminal state, behind an ending log.
        if observed.contains("case.json")
            && !matches!(w.case_state().as_str(), "completed" | "terminated")
        {
            return Err(format!(
                "C2: a non-terminal case.json write on a closed case ({s:?})"
            ));
        }
        if observed.contains("case.json") && !ended {
            return Err(format!(
                "C2: case.json written before the plan ended ({s:?})"
            ));
        }
    }
    // The product rule: after close-failed the operator can stop the case.
    if let (Step::Terminate, Life::Closed, false, false) =
        (s, before.life, before.ended, before.corrupted)
    {
        if before.case_state == "active" && outcome.is_err() {
            return Err(format!("terminate refused on a closed case: {outcome:?}"));
        }
    }
    // C3: no gate item or milestone completes without a passing assessment.
    for e in new_events {
        let item = e["plan_item_id"].as_str().unwrap_or("");
        let kind = e["kind"].as_str().unwrap_or("");
        if let Some(gate) = gate_of(item) {
            let completes = matches!(
                kind,
                "item_completed" | "milestone_achieved" | "human_task_completed"
            );
            let skips = kind == "item_terminated"
                && matches!(gate, "allocate_authority" | "integrate_govern");
            if completes || skips {
                let latest = records
                    .iter()
                    .rfind(|r| r["kind"] == "gate.assessed" && r["gate"] == gate)
                    .and_then(|r| r["state"].as_str());
                if !matches!(latest, Some("met" | "not_applicable")) {
                    return Err(format!(
                        "C3: {kind} on {item} with assessment {latest:?} ({s:?})"
                    ));
                }
            }
        }
    }
    // C6: a suspended case accepts only what the registry accepts suspended.
    if before.life == Life::Suspended && outcome.is_ok() {
        if !new_events.is_empty() {
            return Err(format!("C6: events written while suspended ({s:?})"));
        }
        for f in observed {
            let ok = matches!(
                f.as_str(),
                "field-records.jsonl"
                    | "field_case.json"
                    | "field-approvals.jsonl"
                    | "field-rt-outbox.jsonl"
                    | "field-anchor-outbox.jsonl"
                    | "field-interaction-outbox.jsonl"
            );
            if !ok {
                return Err(format!("C6: {f} written while suspended ({s:?})"));
            }
        }
        for r in new_records {
            let kind = r["kind"].as_str().unwrap_or("");
            let d = r["disposition"].as_str().unwrap_or("");
            let allowed = matches!(
                kind,
                "gate.assessed" | "field_case.revised" | "strategy.recorded"
            ) || (kind == "disposition.recorded"
                && r["level"] == "case"
                && (d == "continue" || d.starts_with("close-")));
            if !allowed {
                return Err(format!("C6: record {kind} accepted while suspended"));
            }
        }
    }
    Ok(())
}

fn run_sequence(steps: &[Step]) -> Result<(), String> {
    let mut w = World::new();
    // The plan is read by the oracle even after the plan file is corrupted.
    fs::write(
        w.root.join("oracle-plan.json"),
        fs::read(w.case_dir.join("plan.json")).unwrap(),
    )
    .unwrap();
    let mut watcher = Watcher::new(&w.case_dir);
    watcher.drain();
    for (n, s) in steps.iter().enumerate() {
        match s {
            Step::Corrupt(k) => {
                if std::env::var_os("MODEL_NO_CORRUPT").is_none() {
                    corrupt(&mut w, *k);
                    watcher.drain();
                }
                continue;
            }
            Step::DeleteMarker(k) => {
                if std::env::var_os("MODEL_NO_CORRUPT").is_none() {
                    delete_marker(&mut w, *k);
                    watcher.drain();
                }
                continue;
            }
            _ => {}
        }
        let events = w.events();
        let before = Snapshot {
            hash: w.tree_hash(),
            events: events.len(),
            records: w.records().len(),
            life: life(&w.records()),
            corrupted: w.corrupted,
            ended: events
                .iter()
                .any(|e| e["kind"] == "case_closed" || e["kind"] == "case_terminated"),
            finished: plan_finished(&w, &events),
            case_state: w.case_state(),
        };
        journal_begin();
        let outcome = run(&w, s, n);
        let journaled: BTreeSet<String> = journal_end()
            .iter()
            .filter_map(|p| p.strip_prefix(&w.case_dir).ok().map(Path::to_path_buf))
            .filter(|rel| rel.components().count() == 1)
            .map(|rel| rel.to_string_lossy().to_string())
            .collect();
        let observed = watcher.drain();
        check(&w, s, &outcome, &before, &observed, &journaled)?;
    }
    Ok(())
}

fn cases() -> u32 {
    std::env::var("MODEL_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(256)
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: cases(),
        max_shrink_iters: 2000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn every_verb_sequence_keeps_the_invariants(steps in proptest::collection::vec(step(), 1..16)) {
        if let Err(violation) = run_sequence(&steps) {
            prop_assert!(false, "{violation}");
        }
    }
}

/// The product defect, as a fixed sequence: `close-failed` on an unfinished
/// plan, then the operator terminates.
#[test]
fn close_failed_then_terminate_stops_the_case() {
    run_sequence(&[Step::Dispose(4), Step::Terminate]).unwrap();
    let w = World::new();
    fs::write(
        w.root.join("oracle-plan.json"),
        fs::read(w.case_dir.join("plan.json")).unwrap(),
    )
    .unwrap();
    w.append_record(
        json!({"kind": "disposition.recorded", "level": "case", "disposition": "close-failed"}),
    )
    .unwrap();
    case_ops::terminate(&w.root, &w.policy(), "operator_local", CASE, "stop").unwrap();
    assert_eq!(w.case_state(), "terminated");
}
