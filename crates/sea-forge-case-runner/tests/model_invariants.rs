//! Stateful model test of the Field Case write path (WP-21 part B, review
//! round 4). Random sequences of every verb run against one temp Field Case,
//! including corruption injections and suspension; after EVERY step the six
//! invariants of `root-cause-classes-2026-11-17.md` are checked from the files
//! on disk, independently of the guard:
//!
//! - C1 no terminal state without a case-level close disposition;
//! - C2 after a close disposition, a write that lands finishes the plan;
//! - C3 no gate item or milestone completes without a passing assessment;
//! - C4 a corrupt store accepts no write;
//! - C5 a refused verb changes no byte under the case directory;
//! - C6 a suspended case accepts only the contract's permitted writes.
//!
//! `MODEL_CASES` sets the case count (default 256, the CI count).

use proptest::prelude::*;
use sea_forge_case_runner::case_ops::{
    self,
    advance::AdvanceScope,
    case_guard::{admit_file_write, digest_of},
};
use sea_forge_core::types::*;
use sea_forge_core::RECORD_VERSION;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
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
    Corrupt(usize),
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        4 => (0..ITEMS.len()).prop_map(Step::Human),
        2 => Just(Step::Advance),
        1 => Just(Step::Terminate),
        1 => Just(Step::Reopen),
        3 => (0..2usize, 0..3usize).prop_map(|(g, s)| Step::Assess(g, s)),
        3 => (0..4usize).prop_map(Step::Dispose),
        2 => (0..4usize).prop_map(Step::Record),
        1 => (0..4usize).prop_map(Step::Corrupt),
    ]
}

struct World {
    _dir: tempfile::TempDir,
    root: PathBuf,
    case_dir: PathBuf,
    corrupted: bool,
}

impl World {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let case_dir = root.join("cases").join(CASE);
        fs::create_dir_all(&case_dir).unwrap();
        let case = Case {
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
        };
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
        fs::write(
            case_dir.join("case.json"),
            serde_json::to_vec(&case).unwrap(),
        )
        .unwrap();
        fs::write(
            case_dir.join("plan.json"),
            serde_json::to_vec(&plan).unwrap(),
        )
        .unwrap();
        fs::write(case_dir.join("case-events.jsonl"), b"").unwrap();
        fs::write(
            case_dir.join("field_case.json"),
            sea_forge_core::field_case::opened_document(CASE).to_string(),
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

    /// Every byte under the case directory (the lock file is not state).
    fn tree_hash(&self) -> String {
        fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
            for e in fs::read_dir(dir).into_iter().flatten().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.file_name().is_some_and(|n| n != ".case.lock") {
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

    /// The record verb, as the server's appender does it: the next link of the
    /// chain, written only after THE guard admits it.
    fn append_record(&self, mut record: Value) -> Result<(), String> {
        let existing = self.records();
        record["seq"] = json!(existing.len() as u64 + 1);
        record["prev_hash"] = json!(existing
            .last()
            .and_then(|r| r["hash"].as_str())
            .unwrap_or(ZERO));
        record["recorded_at"] = json!("2026-11-15T08:00:00.000Z");
        record["hash"] = json!(digest_of(&record).unwrap());
        let path = self.case_dir.join("field-records.jsonl");
        let _token = admit_file_write(&path, &record).map_err(|r| r.code.to_string())?;
        let mut text = fs::read_to_string(&path).unwrap_or_default();
        text.push_str(&format!("{record}\n"));
        fs::write(&path, text).unwrap();
        Ok(())
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

fn plan_finished(events: &[Value]) -> bool {
    let plan: CasePlan =
        serde_json::from_slice(&fs::read(PLAN_PATH.with(|p| p.borrow().clone())).unwrap()).unwrap();
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

thread_local! {
    static PLAN_PATH: std::cell::RefCell<PathBuf> = const { std::cell::RefCell::new(PathBuf::new()) };
}

fn gate_of(item: &str) -> Option<&str> {
    item.strip_prefix("gate_")
}

/// Run one step; returns the verb's outcome (`Ok` accepted, `Err` refused).
fn run(w: &World, s: &Step) -> Result<(), String> {
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
            w.append_record(
                json!({"kind": "gate.assessed", "gate": gate, "state": state,
                                   "payload": {"gate": gate, "state": state}}),
            )
        }
        Step::Dispose(d) => {
            let d = ["close-satisfied", "continue", "suspend", "revise"][*d];
            w.append_record(json!({"kind": "disposition.recorded", "level": "case",
                                   "disposition": d, "payload": {"disposition": d}}))
        }
        Step::Record(k) => {
            let kind = [
                "strategy.recorded",
                "field_case.revised",
                "episode.settled",
                "episode.frozen",
            ][*k];
            w.append_record(json!({"kind": kind, "payload": {"n": 1}}))
        }
        Step::Corrupt(_) => unreachable!("handled by the caller"),
    }
}

fn corrupt(w: &mut World, k: usize) {
    match k {
        0 => fs::write(w.case_dir.join("plan.json"), "{not json").unwrap(),
        1 => fs::write(w.case_dir.join("field-runs.jsonl"), "{torn\n").unwrap(),
        2 => fs::write(w.case_dir.join("field_case.json"), "[]").unwrap(),
        _ => {
            let path = w.case_dir.join("field-records.jsonl");
            let text = fs::read_to_string(&path).unwrap_or_default();
            if let Some(first) = text.lines().next() {
                let mut r: Value = serde_json::from_str(first).unwrap();
                r["hash"] = json!("ab".repeat(32));
                let rest: Vec<&str> = text.lines().skip(1).collect();
                let mut out = format!("{r}\n");
                for l in rest {
                    out.push_str(l);
                    out.push('\n');
                }
                fs::write(&path, out).unwrap();
            } else {
                fs::write(w.case_dir.join("field-runs.jsonl"), "{torn\n").unwrap();
            }
        }
    }
    w.corrupted = true;
}

/// The invariants, after one step. `before` is the state the step started from.
fn check(
    w: &World,
    s: &Step,
    outcome: &Result<(), String>,
    before: &Snapshot,
) -> Result<(), String> {
    let after = w.tree_hash();
    let records = w.records();
    let events = w.events();
    let new_events = &events[before.events.min(events.len())..];
    // C4, C5: a corrupt store accepts no write; a refused verb changes no byte.
    if before.corrupted && after != before.hash {
        return Err(format!(
            "C4: a corrupt store accepted a write ({s:?}): {outcome:?}"
        ));
    }
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
    // C2: a write that lands on a closed case finishes the plan.
    if before.life == Life::Closed && !new_events.is_empty() && !plan_finished(&events) {
        return Err(format!(
            "C2: a closed case progressed without finishing ({s:?}): {outcome:?}"
        ));
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
    // C6: a suspended case accepts only the contract's permitted writes.
    if before.life == Life::Suspended {
        if !new_events.is_empty() {
            return Err(format!(
                "C6: events written while suspended ({s:?}): {outcome:?}"
            ));
        }
        if let (Ok(()), Step::Record(k)) = (outcome, s) {
            if matches!(k, 2 | 3) {
                return Err(format!("C6: record {k} accepted while suspended"));
            }
        }
        if let (Ok(()), Step::Dispose(d)) = (outcome, s) {
            if matches!(d, 3) {
                return Err("C6: revise accepted while suspended".into());
            }
        }
    }
    // A closed case takes only late settlement among records.
    if before.life == Life::Closed {
        if let (Ok(()), Step::Record(k)) = (outcome, s) {
            if *k != 2 {
                return Err(format!("C6/closed: record {k} accepted on a closed case"));
            }
        }
        if let (Ok(()), Step::Dispose(_) | Step::Assess(..)) = (outcome, s) {
            return Err("closed: a disposition or assessment was accepted".into());
        }
    }
    Ok(())
}

struct Snapshot {
    hash: String,
    events: usize,
    life: Life,
    corrupted: bool,
}

fn run_sequence(steps: &[Step]) -> Result<(), String> {
    let mut w = World::new();
    PLAN_PATH.with(|p| *p.borrow_mut() = w.case_dir.join("plan.json"));
    // The plan is read by the oracle even after the plan file is corrupted.
    let plan_backup = fs::read(w.case_dir.join("plan.json")).unwrap();
    let oracle_plan = w.root.join("oracle-plan.json");
    fs::write(&oracle_plan, plan_backup).unwrap();
    PLAN_PATH.with(|p| *p.borrow_mut() = oracle_plan);
    for s in steps {
        if let Step::Corrupt(k) = s {
            if std::env::var_os("MODEL_NO_CORRUPT").is_some() {
                continue;
            }
            corrupt(&mut w, *k);
            continue;
        }
        let before = Snapshot {
            hash: w.tree_hash(),
            events: w.events().len(),
            life: life(&w.records()),
            corrupted: w.corrupted,
        };
        let outcome = run(&w, s);
        check(&w, s, &outcome, &before)?;
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
    fn every_verb_sequence_keeps_c1_to_c6(steps in proptest::collection::vec(step(), 1..14)) {
        if let Err(violation) = run_sequence(&steps) {
            prop_assert!(false, "{violation}");
        }
    }
}
