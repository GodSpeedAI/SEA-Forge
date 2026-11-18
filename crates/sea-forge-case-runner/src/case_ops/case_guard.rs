//! THE choke point of every Field Case mutation (WP-21 part B, review round 5).
//!
//! Nothing writes under a case directory except [`commit`]. A caller builds a
//! typed [`Batch`] (record appends, event appends, store appends, projection
//! updates) and `commit` does ALL of it, in one critical section:
//!
//! 1. take the case lock, whose file lives OUTSIDE the case directory
//!    (`<root>/.case-locks/`), so a refusal creates nothing under the case;
//! 2. [`load_verified`]: every store of the case, strictly, or STORE_INTEGRITY;
//! 3. [`decide`] on the post-state of the WHOLE batch (the batch is folded onto
//!    the verified state; the invariants are checked on the result);
//! 4. write exactly the batch's operations, then the projections the ledger
//!    determines (the document `field_case.json` is rebuilt from the record
//!    chain in the same commit), then release.
//!
//! There is no token and no raw write method: an unadmitted write cannot be
//! expressed. The lifecycle rules come from the contract's per-kind registry
//! ([`super::contract`], README section 5: the `Suspended` and `Closed`
//! columns), loaded as data. Every file a batch touches must be classified by
//! [`classify`] (a registry kind's file, an outbox, an execution store, or a
//! projection); anything else is refused (deny by default).
//!
//! Contract lines this rests on:
//! - README section 6: "Lifecycle is the pure function of E section 3 over the
//!   ledger: `closed` when a case-level `close-*` disposition exists (final,
//!   `CASE_CLOSED` for anything after it that is refused below)".
//! - README D42: "a closed case refuses every write except the action that
//!   ends the plan (CASE_CLOSED is final, D32)". The actions that end the plan
//!   are finishing it (a completion, `CaseClosed`) and `CaseTerminated`: after
//!   `close-failed` on an unfinished plan the operator can still stop the case.
//! - README R11: "any path to a terminal state with no case-level `close-*`
//!   disposition ... `CLOSE_DISPOSITION_REQUIRED`".
//! - README section 5, registry table: what a suspended and a closed case do
//!   with each kind (for example `episode.settled`: suspended refuse, closed
//!   accept as late evidence; `gate.assessed`: suspended accept, closed refuse).
//!
//! A case that is not a Field Case (no `field*` artifact, plan not
//! `fde.field_case@1`) is written without a decision.

use super::contract::{self, Rule};
use super::field_guard::{gate_check_records, is_field_case, Refusal};
use sea_forge_core::types::{CasePlan, TraceEvent, TraceKind};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::Write as _;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// The contract's refusal (`code`, `message`, `no_side_effect`).
pub type ContractError = Refusal;

/// The case-record file: the hash-chained ledger of a Field Case.
pub const RECORDS: &str = "field-records.jsonl";
/// The case-event log.
pub const EVENTS: &str = "case-events.jsonl";
/// The document of a Field Case: a projection of the record chain.
pub const DOCUMENT: &str = "field_case.json";
const REPAIRS: &str = "field-repairs.jsonl";
const ZERO_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

// ---------------------------------------------------------------------------
// The case lock
// ---------------------------------------------------------------------------

thread_local! {
    /// The case locks this thread holds: a nested acquisition (a verb that
    /// holds the lock and then commits) re-uses it instead of deadlocking on
    /// its own file lock.
    static HELD: RefCell<HashMap<PathBuf, (usize, File)>> = RefCell::new(HashMap::new());
    /// Test builds: every path `commit` writes (see [`journal_begin`]).
    static JOURNAL: RefCell<Option<Vec<PathBuf>>> = const { RefCell::new(None) };
}

/// Start journaling the paths `commit` writes on this thread.
pub fn journal_begin() {
    JOURNAL.with(|j| *j.borrow_mut() = Some(Vec::new()));
}

/// Stop journaling and return every path written since [`journal_begin`].
pub fn journal_end() -> Vec<PathBuf> {
    JOURNAL.with(|j| j.borrow_mut().take().unwrap_or_default())
}

fn note(path: &Path) {
    JOURNAL.with(|j| {
        if let Some(list) = j.borrow_mut().as_mut() {
            list.push(path.to_path_buf());
        }
    });
}

/// Where the lock of `case_dir` lives: `<root>/.case-locks/<id>-<hash>.lock`,
/// never inside the case directory.
fn lock_file(case_dir: &Path) -> (PathBuf, String) {
    let parent = case_dir.parent().unwrap_or(Path::new("."));
    let key = std::fs::canonicalize(parent)
        .unwrap_or_else(|_| parent.to_path_buf())
        .join(case_dir.file_name().unwrap_or_default());
    let root = store_root(case_dir);
    let dir = root.join(".case-locks");
    let tag = hex::encode(&Sha256::digest(key.to_string_lossy().as_bytes())[..6]);
    let name = format!(
        "{}-{tag}.lock",
        key.file_name().unwrap_or_default().to_string_lossy()
    );
    (dir, name)
}

/// The root directory of the case store `case_dir` lives in.
fn store_root(case_dir: &Path) -> PathBuf {
    let parent = case_dir.parent().unwrap_or(Path::new("."));
    let key = std::fs::canonicalize(parent)
        .unwrap_or_else(|_| parent.to_path_buf())
        .join(case_dir.file_name().unwrap_or_default());
    let parent = key.parent().unwrap_or(Path::new("."));
    if parent.file_name().is_some_and(|n| n == "cases") {
        parent.parent().unwrap_or(parent).to_path_buf()
    } else {
        parent.to_path_buf()
    }
}

/// The case index entry of `case_dir`: `<root>/.case-index/<case_id>.json`,
/// outside the case directory. It records what a commit left (`field`, the
/// ledger head digest, the event count) so a store that lost files or lines
/// afterwards is detected (residual detection).
pub fn index_path(case_dir: &Path) -> PathBuf {
    let id = case_dir.file_name().unwrap_or_default().to_string_lossy();
    store_root(case_dir)
        .join(".case-index")
        .join(format!("{id}.json"))
}

fn check_index(
    case_dir: &Path,
    field: bool,
    records: &[Value],
    events: &[TraceEvent],
) -> Result<(), ContractError> {
    let Ok(bytes) = std::fs::read(index_path(case_dir)) else {
        return Ok(());
    };
    let idx: Value = serde_json::from_slice(&bytes)
        .map_err(|e| integrity(format!("the case index does not parse: {e}")))?;
    if idx["field"] == true && !field {
        return Err(integrity(
            "the case index says this is a Field Case but its field artifacts are gone".into(),
        ));
    }
    if !field {
        return Ok(());
    }
    let head = idx["ledger_head_digest"].as_str().unwrap_or(ZERO_HASH);
    if head != ZERO_HASH && !records.iter().any(|r| r["hash"].as_str() == Some(head)) {
        return Err(integrity(
            "the ledger lost records the case index saw (its head digest is gone)".into(),
        ));
    }
    if (events.len() as u64) < idx["event_count"].as_u64().unwrap_or(0) {
        return Err(integrity(
            "the event log lost events the case index saw".into(),
        ));
    }
    Ok(())
}

fn write_index(
    case_dir: &Path,
    records: &[Value],
    event_count: usize,
) -> Result<(), ContractError> {
    let head = records
        .last()
        .and_then(|r| r["hash"].as_str())
        .unwrap_or(ZERO_HASH);
    let bytes = serde_json::to_vec(&json!({
        "field": true, "ledger_head_digest": head, "event_count": event_count
    }))
    .map_err(|e| integrity(e.to_string()))?;
    replace_bytes(case_dir, &index_path(case_dir), &bytes)
}

/// The lock file of `case_dir` (outside it).
pub fn lock_path(case_dir: &Path) -> PathBuf {
    let (dir, name) = lock_file(case_dir);
    dir.join(name)
}

/// The cross-process lock of one case directory. Held until dropped; a thread
/// that already holds it nests (counted). Not `Send`.
pub struct CaseLock {
    key: PathBuf,
    _thread_bound: PhantomData<*const ()>,
}

impl CaseLock {
    /// Block until this thread holds the case lock of `case_dir`. Creates
    /// nothing under `case_dir`.
    pub fn acquire(case_dir: &Path) -> Result<Self, ContractError> {
        if super::versioned_store::is_generation_data(&store_root(case_dir)) {
            return Err(integrity("published case generations are immutable".into()));
        }
        let (dir, name) = lock_file(case_dir);
        let key = dir.join(name);
        let nested = HELD.with(|held| {
            held.borrow_mut().get_mut(&key).map(|slot| {
                slot.0 += 1;
            })
        });
        if nested.is_none() {
            std::fs::create_dir_all(&dir)
                .map_err(|e| integrity(format!("create the lock directory: {e}")))?;
            let file = OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .open(&key)
                .map_err(|e| integrity(format!("open the case lock: {e}")))?;
            file.lock()
                .map_err(|e| integrity(format!("take the case lock: {e}")))?;
            HELD.with(|held| held.borrow_mut().insert(key.clone(), (1, file)));
        }
        let lock = Self {
            key,
            _thread_bound: PhantomData,
        };
        require_legacy_writer(case_dir)?;
        Ok(lock)
    }
}

impl Drop for CaseLock {
    fn drop(&mut self) {
        HELD.with(|held| {
            let mut held = held.borrow_mut();
            if let Some(slot) = held.get_mut(&self.key) {
                slot.0 -= 1;
                if slot.0 == 0 {
                    // Dropping the file closes it, which releases the lock.
                    held.remove(&self.key);
                }
            }
        });
    }
}

// ---------------------------------------------------------------------------
// What is proposed
// ---------------------------------------------------------------------------

/// One operation of a batch.
#[derive(Debug, Clone)]
pub enum Op {
    /// A line of `field-records.jsonl`: the full, chain-linked record.
    Record(Value),
    /// A trace event of `case-events.jsonl`, fully built.
    Event(TraceEvent),
    /// A line of a declared auxiliary store (intents, decisions, approvals,
    /// outboxes, execution stores), by file name.
    Append { file: String, line: Value },
    /// `case.json`: a projection; its state must agree with the event log.
    Case(Value),
    /// `plan.json`: written at creation, or together with its `PlanMutated`.
    Plan(Value),
    /// A file under `runs/` (run outputs), by path inside `runs/`.
    RunFile { rel: String, bytes: Vec<u8> },
}

/// A verb's whole consequence, committed all or nothing.
#[derive(Debug, Clone, Default)]
pub struct Batch {
    pub ops: Vec<Op>,
}

impl Batch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, op: Op) -> Self {
        self.ops.push(op);
        self
    }

    pub fn push(&mut self, op: Op) {
        self.ops.push(op);
    }

    /// Events as the engine would append them (kinds and items only): for
    /// [`check`], where nothing is written.
    pub fn of_events(events: &[(TraceKind, Option<String>)]) -> Self {
        Self {
            ops: events
                .iter()
                .map(|(kind, item)| Op::Event(hypothetical(kind, item.as_deref())))
                .collect(),
        }
    }
}

/// What a commit wrote.
#[derive(Debug, Default)]
pub struct Committed {
    pub events: Vec<TraceEvent>,
}

fn integrity(message: String) -> ContractError {
    Refusal {
        code: "STORE_INTEGRITY",
        message: format!("case store failed: {message}"),
    }
}

fn refuse(code: &'static str, message: &str) -> ContractError {
    Refusal {
        code,
        message: message.into(),
    }
}

// ---------------------------------------------------------------------------
// Digests (RFC 8785 for the integer, string, array, object subset)
// ---------------------------------------------------------------------------

fn jcs(value: &Value) -> Option<String> {
    Some(match value {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            if n.is_f64() {
                return None;
            }
            n.to_string()
        }
        Value::String(s) => serde_json::to_string(s).ok()?,
        Value::Array(items) => {
            let parts: Option<Vec<String>> = items.iter().map(jcs).collect();
            format!("[{}]", parts?.join(","))
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
            let mut parts = Vec::with_capacity(keys.len());
            for key in keys {
                parts.push(format!(
                    "{}:{}",
                    serde_json::to_string(key).ok()?,
                    jcs(&map[key])?
                ));
            }
            format!("{{{}}}", parts.join(","))
        }
    })
}

/// SHA-256 over the RFC 8785 form of `value`, 64 lowercase hex.
pub fn digest_of(value: &Value) -> Option<String> {
    Some(hex::encode(Sha256::digest(jcs(value)?.as_bytes())))
}

// ---------------------------------------------------------------------------
// The ledger
// ---------------------------------------------------------------------------

/// The disposition lifecycle of a case (README §6), a pure function of the
/// case-level dispositions of the ledger.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Lifecycle {
    Open,
    Suspended,
    Closed,
}

/// The lifecycle of the case from its case-level dispositions: a `close-*` is
/// final; otherwise suspended when the latest one is `suspend`.
pub fn lifecycle(records: &[Value]) -> Lifecycle {
    let mut state = Lifecycle::Open;
    for r in records {
        if r["kind"] != "disposition.recorded" || r["level"] != "case" {
            continue;
        }
        let d = r["disposition"].as_str().unwrap_or("");
        if d.starts_with("close-") {
            return Lifecycle::Closed;
        }
        state = if d == "suspend" {
            Lifecycle::Suspended
        } else {
            Lifecycle::Open
        };
    }
    state
}

/// Read the record file strictly and verify the chain and the digests: every
/// line parses, `seq` counts from 1, `prev_hash` links to the previous `hash`,
/// `hash` is the digest of the record without it, and `body_digest` (when the
/// record carries a payload) is the digest of `{kind, payload}`.
pub fn verified_records(case_dir: &Path) -> Result<Vec<Value>, ContractError> {
    let path = case_dir.join(RECORDS);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(integrity(format!("read {RECORDS}: {e}"))),
    };
    // Strict to the last byte: anything after the last newline-terminated
    // record is STORE_INTEGRITY. Repair is an explicit operator act.
    if bytes.last().is_some_and(|b| *b != b'\n') {
        return Err(integrity(format!(
            "{RECORDS} ends with a torn fragment (no newline)"
        )));
    }
    let end = bytes.len();
    let mut out: Vec<Value> = Vec::new();
    let mut prev = ZERO_HASH.to_string();
    for (i, line) in bytes[..end].split(|b| *b == b'\n').enumerate() {
        if line.is_empty() {
            continue;
        }
        let record: Value = serde_json::from_slice(line)
            .map_err(|e| integrity(format!("{RECORDS} line {} is not JSON: {e}", i + 1)))?;
        let seq = out.len() as u64 + 1;
        let bad = |why: &str| integrity(format!("{RECORDS} record {seq}: {why}"));
        if record["seq"].as_u64() != Some(seq) {
            return Err(bad("seq is not the next integer"));
        }
        if record["prev_hash"].as_str() != Some(prev.as_str()) {
            return Err(bad("prev_hash does not link to the previous record"));
        }
        let Some(stored) = record["hash"].as_str().map(str::to_string) else {
            return Err(bad("no hash"));
        };
        let mut body = record.clone();
        if let Some(map) = body.as_object_mut() {
            map.remove("hash");
        }
        if digest_of(&body).as_deref() != Some(stored.as_str()) {
            return Err(bad("hash is not the digest of the record"));
        }
        if let (Some(body_digest), Some(kind), true) = (
            record["body_digest"].as_str(),
            record["kind"].as_str(),
            record.get("payload").is_some(),
        ) {
            let expect = digest_of(&json!({"kind": kind, "payload": record["payload"]}));
            if expect.as_deref() != Some(body_digest) {
                return Err(bad("body_digest is not the digest of kind and payload"));
            }
        }
        prev = stored;
        out.push(record);
    }
    Ok(out)
}

/// Has the plan run out of things to do? Every item is finished, or the engine
/// has only the case close or terminate left.
fn plan_finished(plan: &Option<CasePlan>, events: &[TraceEvent]) -> bool {
    use sea_forge_planner::case_engine::{next_case_actions, replay_case, CaseAction, ItemStatus};
    let Some(plan) = plan else {
        return false;
    };
    let projection = replay_case(&plan.items, events);
    projection.items.iter().all(|s| {
        matches!(
            s.status,
            ItemStatus::Completed | ItemStatus::Terminated | ItemStatus::Failed
        )
    }) || next_case_actions(&plan.items, events).iter().any(|a| {
        matches!(
            a,
            CaseAction::CompleteCase | CaseAction::TerminateCase { .. }
        )
    })
}

fn ledger_revision(records: &[Value]) -> u64 {
    records
        .iter()
        .filter(|r| r["kind"] == "field_case.revised")
        .filter_map(|r| r["revision"].as_u64())
        .max()
        .unwrap_or(1)
}

// ---------------------------------------------------------------------------
// Files: every one belongs to a registry kind, a declared projection, or is
// refused
// ---------------------------------------------------------------------------

/// What a file of the case directory is to the guard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileClass {
    /// `field-records.jsonl`: each line is a registry kind.
    Ledger,
    /// `case-events.jsonl`.
    Events,
    /// `field_case.json`: derived from the ledger inside the commit; no
    /// caller writes it.
    Document,
    /// `case.json`: a projection whose state must agree with the event log.
    CaseJson,
    /// `plan.json`: created, or rewritten with its `PlanMutated` event.
    PlanJson,
    /// A store that belongs to registry kinds: a line is admitted when one of
    /// them is accepted by the lifecycle (their Suspended / Closed columns).
    Owned(&'static [&'static str]),
    /// Obligations and deliveries of ledger facts (interaction, RealityTrace
    /// and anchor outboxes): they carry no case state.
    Outbox,
    /// Execution and delegation state: refused when suspended; once closed
    /// only while the action that ends the plan is in flight.
    Execution,
    /// Files under `runs/`: run outputs, same rule as execution state.
    RunOutput,
}

/// The file table. `None` is a file the guard has no rule for: refused.
pub fn classify(rel: &str) -> Option<FileClass> {
    if rel == "runs" || rel.starts_with("runs/") {
        return Some(FileClass::RunOutput);
    }
    Some(match rel {
        RECORDS => FileClass::Ledger,
        EVENTS => FileClass::Events,
        DOCUMENT => FileClass::Document,
        "case.json" => FileClass::CaseJson,
        "plan.json" => FileClass::PlanJson,
        "field-freeze-intents.jsonl" => FileClass::Owned(&["episode.frozen"]),
        "field-decisions.jsonl" => FileClass::Owned(&["episode.settled"]),
        "field-approvals.jsonl" => FileClass::Owned(&[
            "disposition.recorded",
            "gate.assessed",
            "field_case.revised",
            "episode.settled",
        ]),
        "field-rt-outbox.jsonl"
        | "field-anchor-outbox.jsonl"
        | "field-interaction-outbox.jsonl" => FileClass::Outbox,
        "field-runs.jsonl"
        | "field-links.jsonl"
        | "field-assignments.jsonl"
        | "field-exec-decisions.jsonl" => FileClass::Execution,
        _ => return None,
    })
}

/// The members every line of a store carries (the shape check of the strict
/// load, and of an append).
const STORE_SHAPES: [(&str, &[&str]); 11] = [
    ("field-freeze-intents.jsonl", &["idempotency_key", "state"]),
    ("field-decisions.jsonl", &["decision_id", "decision"]),
    ("field-approvals.jsonl", &["approval_ref"]),
    ("field-rt-outbox.jsonl", &["system", "record_id", "state"]),
    ("field-anchor-outbox.jsonl", &["id", "state"]),
    ("field-interaction-outbox.jsonl", &["event_id", "state"]),
    (
        "field-runs.jsonl",
        &["run_id", "case_id", "episode_ref", "verb"],
    ),
    ("field-links.jsonl", &["link_id", "case_id", "delegate"]),
    ("field-assignments.jsonl", &["assignment_id", "state"]),
    ("field-exec-decisions.jsonl", &["decision_id", "run_id"]),
    (REPAIRS, &["file", "dropped_sha256"]),
];

fn shape_ok(file: &str, line: &Value) -> Result<(), String> {
    let Some((_, keys)) = STORE_SHAPES.iter().find(|(n, _)| *n == file) else {
        return Err(format!("{file} is not a store of the case"));
    };
    let Some(map) = line.as_object() else {
        return Err("a line is not a JSON object".into());
    };
    for key in *keys {
        if !map.contains_key(*key) {
            return Err(format!("a line of {file} has no `{key}`"));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Strict load: every line of every store, or STORE_INTEGRITY
// ---------------------------------------------------------------------------

/// The complete lines of a store file. A torn trailing fragment (a crash
/// mid-append, no newline) is not a line; the next commit repairs it.
fn complete_lines(path: &Path) -> Result<Vec<Vec<u8>>, ContractError> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(integrity(format!("read {}: {e}", path.display()))),
    };
    let end = bytes.iter().rposition(|b| *b == b'\n').map_or(0, |i| i + 1);
    Ok(bytes[..end]
        .split(|b| *b == b'\n')
        .filter(|l| !l.is_empty())
        .map(<[u8]>::to_vec)
        .collect())
}

/// Every event line parses as a trace event: no line is skipped.
fn events_of(case_dir: &Path, strict: bool) -> Result<Vec<TraceEvent>, ContractError> {
    let mut out = Vec::new();
    for (i, line) in complete_lines(&case_dir.join(EVENTS))?.iter().enumerate() {
        match serde_json::from_slice(line) {
            Ok(e) => out.push(e),
            // A case that is not a Field Case proper keeps its previous
            // reading: an event the engine cannot read is not history.
            Err(_) if !strict => {}
            Err(e) => {
                return Err(integrity(format!(
                    "{EVENTS} line {} is not a trace event: {e}",
                    i + 1
                )))
            }
        }
    }
    Ok(out)
}

fn plan_of(case_dir: &Path) -> Option<CasePlan> {
    serde_json::from_slice(&std::fs::read(case_dir.join("plan.json")).ok()?).ok()
}

// ---------------------------------------------------------------------------
// The rules: closed matches, no wildcard arm
// ---------------------------------------------------------------------------

/// What an event type means to the guard.
enum EventClass {
    /// The case is being created: allowed once, before the ledger exists.
    Creation,
    /// Plan progress. `gate` is the gate rule that applies (`complete` or
    /// `skip`); `finishes` marks an event that ends an item.
    Progress {
        gate: Option<&'static str>,
        finishes: bool,
    },
    /// Run telemetry of an execution in flight.
    Telemetry,
    /// A change to the plan or its case file items.
    Structure,
    /// `CaseClosed`.
    Close,
    /// `CaseTerminated`.
    Terminate,
    /// `CaseReopened`.
    Reopen,
}

/// Every trace event type has a rule here. No wildcard: a new `TraceKind`
/// does not compile until it is classified.
fn event_class(kind: &TraceKind) -> EventClass {
    match kind {
        TraceKind::CaseCreated | TraceKind::PlanCreated => EventClass::Creation,
        TraceKind::ItemEnabled | TraceKind::ItemActivated => EventClass::Progress {
            gate: None,
            finishes: false,
        },
        TraceKind::ItemCompleted | TraceKind::MilestoneAchieved | TraceKind::HumanTaskCompleted => {
            EventClass::Progress {
                gate: Some("complete"),
                finishes: true,
            }
        }
        TraceKind::ItemTerminated => EventClass::Progress {
            gate: Some("skip"),
            finishes: true,
        },
        TraceKind::ItemFailed => EventClass::Progress {
            gate: None,
            finishes: true,
        },
        TraceKind::RunStarted
        | TraceKind::AuthorityEvaluated
        | TraceKind::WorkspaceCreated
        | TraceKind::CommandStarted
        | TraceKind::CommandFinished
        | TraceKind::ArtifactCaptured
        | TraceKind::SettlementRecorded
        | TraceKind::RunHalted
        | TraceKind::RunFinished
        | TraceKind::InternalError => EventClass::Telemetry,
        TraceKind::PlanMutated | TraceKind::CaseFileItemAdded => EventClass::Structure,
        TraceKind::CaseClosed => EventClass::Close,
        TraceKind::CaseTerminated => EventClass::Terminate,
        TraceKind::CaseReopened => EventClass::Reopen,
    }
}

fn closed() -> ContractError {
    refuse(
        "CASE_CLOSED",
        "the case has a close disposition; closed is final",
    )
}

fn suspended() -> ContractError {
    refuse("CASE_SUSPENDED", "the case is suspended")
}

/// What a write does to the plan, for the closed-case rule.
enum Hypo<'a> {
    /// The event finishes an item: simulate it.
    Finishing(&'a TraceKind, Option<&'a str>),
    /// Work in flight toward the finishing action (item, or any item).
    InFlight(Option<&'a str>),
}

fn hypothetical(kind: &TraceKind, item: Option<&str>) -> TraceEvent {
    TraceEvent {
        version: "0.2".into(),
        event_id: "guard-hypothetical".into(),
        run_id: "case".into(),
        plan_item_id: item.map(str::to_owned),
        kind: kind.clone(),
        actor_id: "case-guard".into(),
        timestamp: "1970-01-01T00:00:00Z".into(),
        payload: json!({}),
        cell_id: None,
    }
}

/// Does the plan finish when `added` has happened, with the engine's automatic
/// milestones that follow (an unassessed gate stops the cascade)?
fn finishes_with(st: &CaseState, events: &[TraceEvent], added: Vec<TraceEvent>) -> bool {
    use sea_forge_planner::case_engine::{next_case_actions, CaseAction};
    let Some(plan) = st.plan.clone() else {
        return false;
    };
    let mut sim = events.to_vec();
    sim.extend(added);
    for _ in 0..=plan.items.len() {
        let Some(id) = next_case_actions(&plan.items, &sim)
            .iter()
            .find_map(|a| match a {
                CaseAction::AchieveMilestone(id) => Some(id.clone()),
                _ => None,
            })
        else {
            break;
        };
        if gate_check_records(&st.records, &id, "complete").is_err() {
            break;
        }
        sim.push(hypothetical(&TraceKind::MilestoneAchieved, Some(&id)));
    }
    plan_finished(&st.plan, &sim)
}

/// The lifecycle for a write that is part of the action that ends the plan.
/// A closed case admits only an action whose RESULT finishes the plan,
/// computed by simulating it (README §7 R13, D42); anything else, and any
/// repeat once the case has ended, is `CASE_CLOSED`.
fn in_flight(st: &CaseState, events: &[TraceEvent], hypo: Hypo) -> Result<(), ContractError> {
    match st.life {
        Lifecycle::Open => Ok(()),
        Lifecycle::Suspended => Err(suspended()),
        Lifecycle::Closed => {
            if ended(events) || plan_finished(&st.plan, events) {
                return Err(closed());
            }
            let ok = match hypo {
                Hypo::Finishing(kind, item) => {
                    finishes_with(st, events, vec![hypothetical(kind, item)])
                }
                Hypo::InFlight(Some(item)) => finishes_with(
                    st,
                    events,
                    vec![hypothetical(&TraceKind::ItemCompleted, Some(item))],
                ),
                Hypo::InFlight(None) => st.plan.as_ref().is_some_and(|plan| {
                    plan.items.iter().any(|i| {
                        finishes_with(
                            st,
                            events,
                            vec![hypothetical(
                                &TraceKind::ItemCompleted,
                                Some(&i.plan_item_id),
                            )],
                        )
                    })
                }),
            };
            if ok {
                Ok(())
            } else {
                Err(closed())
            }
        }
    }
}

/// Has the case already been closed or terminated by an event?
fn ended(events: &[TraceEvent]) -> bool {
    events
        .iter()
        .any(|e| matches!(e.kind, TraceKind::CaseClosed | TraceKind::CaseTerminated))
}

fn admit_event(
    st: &CaseState,
    kind: &TraceKind,
    item: Option<&str>,
    events: &[TraceEvent],
) -> Result<(), ContractError> {
    let life = st.life;
    match event_class(kind) {
        EventClass::Creation => {
            if life == Lifecycle::Open && !events.iter().any(|e| &e.kind == kind) {
                Ok(())
            } else {
                Err(closed())
            }
        }
        EventClass::Progress { gate, finishes } => {
            let hypo = if finishes {
                Hypo::Finishing(kind, item)
            } else {
                Hypo::InFlight(item)
            };
            in_flight(st, events, hypo)?;
            // A closed case takes the action that ends the plan once: an item
            // that is already done is not done again.
            if life == Lifecycle::Closed && finishes {
                if let Some(item) = item {
                    let done = events.iter().any(|e| {
                        e.plan_item_id.as_deref() == Some(item)
                            && matches!(
                                e.kind,
                                TraceKind::ItemCompleted
                                    | TraceKind::ItemFailed
                                    | TraceKind::ItemTerminated
                                    | TraceKind::HumanTaskCompleted
                                    | TraceKind::MilestoneAchieved
                            )
                    });
                    if done {
                        return Err(closed());
                    }
                }
            }
            if let (Some(how), Some(item)) = (gate, item) {
                gate_check_records(&st.records, item, how)?;
            }
            Ok(())
        }
        EventClass::Telemetry => in_flight(st, events, Hypo::InFlight(None)),
        EventClass::Structure => match life {
            Lifecycle::Open => Ok(()),
            Lifecycle::Suspended => Err(suspended()),
            Lifecycle::Closed => Err(closed()),
        },
        // R11: any path to a terminal state needs a case-level close-*
        // disposition (README R11: "any path to a terminal state with no
        // case-level close-* disposition ... CLOSE_DISPOSITION_REQUIRED").
        // Both the close and the terminate end the case, once.
        EventClass::Close | EventClass::Terminate => match life {
            Lifecycle::Suspended => Err(suspended()),
            Lifecycle::Open => Err(refuse(
                "CLOSE_DISPOSITION_REQUIRED",
                "this action ends the case and no case-level close disposition exists",
            )),
            // C2 / D42 ("a closed case refuses every write except the action
            // that ends the plan"): once, and while the case has not ended.
            // `CaseTerminated` IS an action that ends the plan, finished or
            // not (the operator stops a case whose close was `close-failed`);
            // `CaseClosed` ends a plan that is finished.
            Lifecycle::Closed => {
                let terminate = matches!(event_class(kind), EventClass::Terminate);
                if ended(events) || !(terminate || plan_finished(&st.plan, events)) {
                    Err(closed())
                } else {
                    Ok(())
                }
            }
        },
        // The contract is silent on reopening. Most conservative reading:
        // closed is final (D32, R13), so a Field Case is never reopened.
        EventClass::Reopen => match life {
            Lifecycle::Suspended => Err(suspended()),
            Lifecycle::Closed => Err(refuse(
                "CASE_CLOSED",
                "closed is final; the contract has no reopen of a Field Case",
            )),
            Lifecycle::Open => Err(integrity(
                "the ledger holds no close disposition, so there is nothing to reopen".into(),
            )),
        },
    }
}

/// The document of a Field Case, derived from its record chain: the opened
/// document with every `field_case.revised` applied in order.
pub fn derived_document(case_id: &str, records: &[Value]) -> Value {
    let mut doc = sea_forge_core::field_case::opened_document(case_id);
    for r in records.iter().filter(|r| r["kind"] == "field_case.revised") {
        for change in r["payload"]["changes"].as_array().into_iter().flatten() {
            if let (Some(path), field) = (change["path"].as_str(), &change["field"]) {
                if let Some((group, name)) = path.split_once('.') {
                    doc["fields"][group][name] = field.clone();
                }
            }
        }
        if let Some(rev) = r["revision"].as_u64() {
            doc["revision"] = json!(rev);
        }
    }
    doc
}

/// The document requirement (README section 6): `field_case.json` is present,
/// is JSON, validates against `field-case.v1`, names this case, is not ahead
/// of its ledger and, at the ledger's revision, is what the ledger derives.
/// (A document BEHIND its ledger is healed by the next commit.)
fn check_document(case_dir: &Path, records: &[Value]) -> Result<(), ContractError> {
    let bytes = std::fs::read(case_dir.join(DOCUMENT))
        .map_err(|e| integrity(format!("{DOCUMENT} of a Field Case: {e}")))?;
    let doc: Value = serde_json::from_slice(&bytes)
        .map_err(|e| integrity(format!("{DOCUMENT} is not JSON: {e}")))?;
    let case_id = case_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if doc["case_id"].as_str() != Some(case_id) {
        return Err(integrity(format!("{DOCUMENT} names another case")));
    }
    contract::document_is_valid(&doc).map_err(|e| integrity(format!("{DOCUMENT}: {e}")))?;
    let have = doc["revision"].as_u64().unwrap_or(0);
    let want = ledger_revision(records);
    if have > want {
        return Err(integrity(format!(
            "{DOCUMENT} is at revision {have}, ahead of its ledger ({want})"
        )));
    }
    if have == want && doc != derived_document(case_id, records) {
        return Err(integrity(format!("{DOCUMENT} differs from its ledger")));
    }
    Ok(())
}

fn terminal_state(state: &str) -> bool {
    matches!(state, "completed" | "terminated")
}

fn plan_says_field(case_dir: &Path) -> bool {
    std::fs::read(case_dir.join("plan.json")).is_ok_and(|bytes| {
        match serde_json::from_slice::<Value>(&bytes) {
            Ok(plan) => plan["template_ref"] == super::field_guard::FIELD_CASE_TEMPLATE,
            Err(_) => {
                String::from_utf8_lossy(&bytes).contains(super::field_guard::FIELD_CASE_TEMPLATE)
            }
        }
    })
}

pub fn index_says_field(case_dir: &Path) -> bool {
    std::fs::read(index_path(case_dir))
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .is_some_and(|i| i["field"] == true)
}

/// Everything a decision rests on, loaded all-or-nothing from every store of
/// the case (see [`load_verified`]).
#[derive(Clone)]
pub struct CaseState {
    /// A Field Case (by the plan or by any `field*` artifact).
    pub field: bool,
    /// A Field Case proper (its plan names the Field template, or the case
    /// index recorded it as one): plan, `case.json`, event log and document
    /// are required and verified. A case that only has a v1 ledger (no Field
    /// template) keeps its previous, lenient rules.
    pub strict: bool,
    pub records: Vec<Value>,
    pub life: Lifecycle,
    pub events: Vec<TraceEvent>,
    pub plan: Option<CasePlan>,
    /// `case.json`'s `state`.
    pub case_state: Option<String>,
    /// `case.json` as stored: a projection write that changes nothing is no
    /// write.
    pub case_json: Option<Value>,
}

/// The verified state of the case at `case_dir`: strictly, from the plan,
/// `case.json`, the event log, every auxiliary store, the record chain (each
/// payload validated against the registry's schema) and the document; or
/// `STORE_INTEGRITY`. Any `field*` artifact makes it a Field Case; a plain
/// case is returned unverified with `field: false`. `creating` is the
/// creation of a Field Case: the files it is about to create may be absent.
pub fn load_verified(case_dir: &Path, creating: bool) -> Result<CaseState, ContractError> {
    let field = creating || is_field_case(case_dir);
    let case_json = std::fs::read(case_dir.join("case.json"))
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok());
    let case_state = case_json
        .as_ref()
        .and_then(|c| c["state"].as_str().map(String::from));
    if !field {
        check_index(case_dir, false, &[], &[])?;
        return Ok(CaseState {
            field,
            strict: false,
            records: Vec::new(),
            life: Lifecycle::Open,
            events: Vec::new(),
            plan: None,
            case_state,
            case_json,
        });
    }
    let plan = plan_of(case_dir);
    let strict = !creating
        && (plan_says_field(case_dir)
            || index_says_field(case_dir)
            // A plan that exists and does not parse is corruption, not a bare case.
            || (case_dir.join("plan.json").exists() && plan.is_none()));
    if strict {
        if plan.is_none() {
            return Err(integrity(
                "plan.json of a Field Case is missing or does not parse".into(),
            ));
        }
        if case_state.is_none() {
            return Err(integrity(
                "case.json of a Field Case is missing or has no state".into(),
            ));
        }
    }
    if strict {
        for name in std::iter::once(EVENTS).chain(STORE_SHAPES.iter().map(|(n, _)| *n)) {
            if std::fs::read(case_dir.join(name))
                .is_ok_and(|b| b.last().is_some_and(|c| *c != b'\n'))
            {
                return Err(integrity(format!(
                    "{name} ends with a torn fragment (no newline)"
                )));
            }
        }
    }
    let events = events_of(case_dir, strict)?;
    let records = if strict {
        verified_records(case_dir)?
    } else {
        // As before for a case that is not a Field Case proper: the ledger is
        // read as it stands (a record append verifies the chain itself).
        complete_lines(&case_dir.join(RECORDS))?
            .iter()
            .filter_map(|l| serde_json::from_slice(l).ok())
            .collect()
    };
    if strict {
        for (i, r) in records.iter().enumerate() {
            let kind = r["kind"].as_str().unwrap_or("");
            contract::payload_is_valid(kind, &r["payload"])
                .map_err(|e| integrity(format!("{RECORDS} record {}: payload: {e}", i + 1)))?;
        }
    }
    // A case that is not a Field Case proper reads the stores it always read.
    const PLAIN_STORES: [&str; 6] = [
        "field-freeze-intents.jsonl",
        "field-decisions.jsonl",
        "field-approvals.jsonl",
        "field-rt-outbox.jsonl",
        "field-anchor-outbox.jsonl",
        "field-runs.jsonl",
    ];
    for (name, _) in STORE_SHAPES
        .iter()
        .filter(|(n, _)| strict || PLAIN_STORES.contains(n) || *n == "field-links.jsonl")
    {
        // An outbox carries no case state: one that cannot be read as a file
        // never blocks the case write (the outbox write fails and is reconciled).
        if name.contains("outbox") && case_dir.join(name).is_dir() {
            continue;
        }
        for (i, line) in complete_lines(&case_dir.join(name))?.iter().enumerate() {
            let value: Value = serde_json::from_slice(line)
                .map_err(|e| integrity(format!("{name} line {} is not JSON: {e}", i + 1)))?;
            shape_ok(name, &value).map_err(|e| integrity(format!("{name} line {}: {e}", i + 1)))?;
        }
    }
    if strict {
        check_document(case_dir, &records)?;
    }
    check_index(case_dir, true, &records, &events)?;
    let life = lifecycle(&records);
    if strict && ended(&events) && life != Lifecycle::Closed {
        return Err(integrity(
            "the event log ends the case but the ledger holds no close disposition".into(),
        ));
    }
    if strict && case_state.as_deref().is_some_and(terminal_state) && !ended(&events) {
        return Err(integrity(
            "case.json is terminal but the event log does not end the case".into(),
        ));
    }
    Ok(CaseState {
        field,
        strict,
        life,
        records,
        events,
        plan,
        case_state,
        case_json,
    })
}

// ---------------------------------------------------------------------------
// The decision
// ---------------------------------------------------------------------------

fn admit_record(st: &CaseState, record: &Value) -> Result<(), ContractError> {
    let kind = record["kind"].as_str().unwrap_or("");
    let Some(rules) = contract::registry().get(kind) else {
        return Err(refuse(
            "KIND_UNKNOWN",
            "the registry has no such record kind",
        ));
    };
    // The record is the next link of the chain, as the ledger is now.
    let seq = st.records.len() as u64 + 1;
    let prev = st
        .records
        .last()
        .and_then(|r| r["hash"].as_str())
        .unwrap_or(ZERO_HASH);
    if record["seq"].as_u64() != Some(seq) || record["prev_hash"].as_str() != Some(prev) {
        return Err(integrity(
            "the record does not extend the ledger as it is now (stale write)".into(),
        ));
    }
    contract::payload_is_valid(kind, &record["payload"])
        .map_err(|e| refuse("SCHEMA_INVALID", &format!("payload: {e}")))?;
    match st.life {
        Lifecycle::Open => Ok(()),
        Lifecycle::Closed => match rules.closed {
            Rule::Accept => Ok(()),
            _ => Err(closed()),
        },
        Lifecycle::Suspended => match rules.suspended {
            Rule::Accept => Ok(()),
            Rule::LiftOrCloseOnly => {
                let d = record["disposition"].as_str().unwrap_or("");
                if record["level"] == "case" && (d == "continue" || d.starts_with("close-")) {
                    Ok(())
                } else {
                    Err(suspended())
                }
            }
            Rule::Refuse => Err(suspended()),
        },
    }
}

/// A store line that belongs to registry kinds: admitted when the lifecycle
/// accepts one of them.
fn admit_owned(st: &CaseState, kinds: &[&str]) -> Result<(), ContractError> {
    let ok = kinds.iter().any(|k| {
        contract::registry().get(*k).is_some_and(|r| match st.life {
            Lifecycle::Open => true,
            Lifecycle::Closed => r.closed == Rule::Accept,
            Lifecycle::Suspended => r.suspended == Rule::Accept,
        })
    });
    match (ok, st.life) {
        (true, _) => Ok(()),
        (false, Lifecycle::Closed) => Err(closed()),
        (false, _) => Err(suspended()),
    }
}

/// `case.json`: its state agrees with the event log. Terminal only once the
/// events end the case (R11, C1); a terminal state is written once; a closed
/// case is not reopened by a state write (C2).
fn admit_case(st: &CaseState, value: &Value) -> Result<(), ContractError> {
    let new = value["state"].as_str().unwrap_or("");
    let over = ended(&st.events);
    if st.case_state.as_deref().is_some_and(terminal_state) && st.case_state.as_deref() != Some(new)
    {
        return Err(closed());
    }
    if terminal_state(new) {
        if !over {
            return Err(match st.life {
                Lifecycle::Open => refuse(
                    "CLOSE_DISPOSITION_REQUIRED",
                    "this write ends the case and no case-level close disposition exists",
                ),
                Lifecycle::Suspended => suspended(),
                Lifecycle::Closed => closed(),
            });
        }
        // Terminal and the log agrees: the projection of an ended case (also
        // its repeat, and the heal of a projection a crash left behind).
        return Ok(());
    }
    if over {
        return Err(closed());
    }
    match st.life {
        Lifecycle::Open => Ok(()),
        Lifecycle::Suspended => Err(suspended()),
        Lifecycle::Closed => Err(closed()),
    }
}

/// THE decision, pure: fold the whole batch onto the verified state, op by op,
/// and check each op and the invariants on the RESULT. Nothing is read or
/// written here. C1 terminal only behind a close disposition; C2 a closed case
/// admits only plan-ending actions and the kinds the registry accepts when
/// closed; C3 no gate item or milestone completes without a passing
/// assessment; C6 a suspended case admits only what the registry accepts when
/// suspended. A file or kind the guard has no rule for is refused.
pub fn decide(st: &CaseState, batch: &Batch) -> Result<(), ContractError> {
    let mut sim = st.clone();
    let mutates_plan = batch
        .ops
        .iter()
        .any(|op| matches!(op, Op::Event(e) if e.kind == TraceKind::PlanMutated));
    for op in batch.ops.iter().filter(|op| !matches!(op, Op::Case(_))) {
        match op {
            Op::Record(r) => {
                admit_record(&sim, r)?;
                sim.records.push(r.clone());
                sim.life = lifecycle(&sim.records);
            }
            Op::Event(e) => {
                admit_event(&sim, &e.kind, e.plan_item_id.as_deref(), &sim.events)?;
                sim.events.push(e.clone());
            }
            Op::Append { file, line } => {
                match classify(file) {
                    Some(FileClass::Owned(kinds)) => admit_owned(&sim, kinds)?,
                    Some(FileClass::Outbox) => {}
                    Some(FileClass::Execution) => {
                        in_flight(&sim, &sim.events, Hypo::InFlight(None))?
                    }
                    _ => {
                        return Err(refuse(
                            "KIND_UNKNOWN",
                            "the guard has no rule for this store file",
                        ))
                    }
                }
                shape_ok(file, line).map_err(|e| refuse("SCHEMA_INVALID", &e))?;
            }
            Op::Plan(_) => {
                if st.plan.is_some() && !mutates_plan {
                    return Err(refuse(
                        "KIND_UNKNOWN",
                        "the plan of a Field Case is rewritten only with its plan mutation",
                    ));
                }
            }
            Op::RunFile { .. } => in_flight(&sim, &sim.events, Hypo::InFlight(None))?,
            Op::Case(_) => {}
        }
    }
    for op in &batch.ops {
        if let Op::Case(v) = op {
            if sim.case_json.as_ref() != Some(v) {
                admit_case(&sim, v)?;
                sim.case_state = v["state"].as_str().map(String::from);
                sim.case_json = Some(v.clone());
            }
        }
    }
    // The result: a terminal event log rests on a close disposition (R11).
    if !ended(&st.events) && ended(&sim.events) && sim.life != Lifecycle::Closed {
        return Err(refuse(
            "CLOSE_DISPOSITION_REQUIRED",
            "this action ends the case and no case-level close disposition exists",
        ));
    }
    Ok(())
}

/// The creation of a Field Case: its plan is being written, or its creation
/// event is (the files creation makes may not exist yet).
fn creating_field(case_dir: &Path, batch: &Batch) -> bool {
    // Creation mode only for a virgin case: no index and no case file at all.
    // A deleted plan on an existing case is damage, not creation.
    let virgin = !index_path(case_dir).exists()
        && std::fs::read_dir(case_dir).map_or(true, |mut d| d.next().is_none());
    let new_plan = virgin
        && batch.ops.iter().any(|op| {
            matches!(op, Op::Plan(p) if p["template_ref"] == super::field_guard::FIELD_CASE_TEMPLATE)
        });
    // The first record of the ledger (a v1 kind on a plain case, as before)
    // makes the case a Field Case: its ledger is the marker.
    let first_record =
        batch.ops.iter().any(|op| matches!(op, Op::Record(_))) && !is_field_case(case_dir);
    // A Field Case whose creation has not made its document yet admits its
    // creation events (and nothing else) before it.
    let creation_events = is_field_case(case_dir)
        && !case_dir.join(DOCUMENT).exists()
        && !batch.ops.is_empty()
        && batch.ops.iter().all(|op| {
            matches!(op, Op::Event(e) if matches!(e.kind, TraceKind::CaseCreated | TraceKind::PlanCreated))
        });
    new_plan || first_record || creation_events
}

fn require_legacy_writer(case_dir: &Path) -> Result<(), ContractError> {
    let Some(case_id) = case_dir.file_name().and_then(|name| name.to_str()) else {
        return Ok(());
    };
    if !sea_forge_core::path::valid_id_segment(case_id, 128) {
        return Ok(());
    }
    let namespace = super::versioned_store::namespace(&store_root(case_dir), case_id)
        .map_err(|error| integrity(error.to_string()))?;
    if namespace.is_some() {
        return Err(integrity("case requires the versioned writer".into()));
    }
    Ok(())
}

/// Decide `batch` on the case at `case_dir` under the case lock without
/// writing anything: the pre-admission of a verb that has effects of its own
/// (a ledger commit, a run) to order before its write. `commit` decides again.
pub fn check(case_dir: &Path, batch: &Batch) -> Result<(), ContractError> {
    contained(case_dir, batch)?;
    let _lock = CaseLock::acquire(case_dir)?;
    let creating = creating_field(case_dir, batch);
    let st = load_verified(case_dir, creating)?;
    if st.field || creating {
        decide(&st, batch)?;
    }
    Ok(())
}

/// Raw-file ops are contained: a `RunFile` path is relative, made of plain
/// names only (no `..`, no root, no prefix), and no existing component of it
/// under `runs/` is a symlink. It therefore cannot reach a store file, and the
/// effect `decide` judges is the effect that is written.
fn contained(case_dir: &Path, batch: &Batch) -> Result<(), ContractError> {
    use std::path::Component;
    for op in &batch.ops {
        let Op::RunFile { rel, .. } = op else {
            continue;
        };
        let escape = |why: &str| refuse("PATH_ESCAPE", &format!("run file {rel:?}: {why}"));
        let mut at = case_dir.join("runs");
        if std::fs::symlink_metadata(&at).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(escape("the runs directory is a symlink"));
        }
        for part in Path::new(rel).components() {
            let Component::Normal(name) = part else {
                return Err(escape("only plain relative names are allowed"));
            };
            at.push(name);
            if std::fs::symlink_metadata(&at).is_ok_and(|m| m.file_type().is_symlink()) {
                return Err(escape("a path component is a symlink"));
            }
        }
        if rel.contains('\\') || rel.contains('\0') {
            return Err(escape("illegal character"));
        }
    }
    Ok(())
}

/// Admit-and-apply: lock, load verified, decide on the post-state of the whole
/// batch, write exactly the batch (and the projections the ledger derives),
/// release. `Err` is the contract's refusal; nothing was written.
pub fn commit(case_dir: &Path, batch: Batch) -> Result<Committed, ContractError> {
    contained(case_dir, &batch)?;
    let _lock = CaseLock::acquire(case_dir)?;
    let creating = creating_field(case_dir, &batch);
    let st = load_verified(case_dir, creating)?;
    let field = st.field || creating;
    if field {
        decide(&st, &batch)?;
    }
    let derive = st.strict
        || batch.ops.iter().any(|op| {
            matches!(op, Op::Plan(p) if p["template_ref"] == super::field_guard::FIELD_CASE_TEMPLATE)
        });
    apply(case_dir, &st, batch, field, derive)
}

fn io(what: &str, e: std::io::Error) -> ContractError {
    integrity(format!("{what}: {e}"))
}

fn append_bytes(path: &Path, bytes: &[u8]) -> Result<(), ContractError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| io(&format!("open {}", path.display()), e))?;
    note(path);
    file.write_all(bytes)
        .and_then(|()| file.sync_data())
        .map_err(|e| io(&format!("write {}", path.display()), e))
}

fn append_value(path: &Path, value: &Value) -> Result<(), ContractError> {
    let mut line = serde_json::to_vec(value).map_err(|e| integrity(e.to_string()))?;
    line.push(b'\n');
    append_bytes(path, &line)
}

/// Replace `path` (temp file OUTSIDE the case directory, then rename).
fn replace_bytes(case_dir: &Path, path: &Path, bytes: &[u8]) -> Result<(), ContractError> {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let (dir, _) = lock_file(case_dir);
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tmp = dir.join(format!(".tmp-{}-{n}", std::process::id()));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| io("create directory", e))?;
    }
    std::fs::create_dir_all(&dir).map_err(|e| io("create the lock directory", e))?;
    let mut file = File::create(&tmp).map_err(|e| io("create temp file", e))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| io("write temp file", e))?;
    drop(file);
    note(path);
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        io(&format!("replace {}", path.display()), e)
    })
}

/// A crash mid-append leaves a trailing fragment with no newline. Appending
/// to it would glue the next line onto it, so the fragment is cut back to the
/// last newline, and the cut is noted durably in `field-repairs.jsonl` BEFORE
/// the truncation: a repair never silently discards bytes.
fn repair_tail(
    case_dir: &Path,
    name: &str,
    done: &mut HashSet<String>,
) -> Result<(), ContractError> {
    if !done.insert(name.to_string()) {
        return Ok(());
    }
    let path = case_dir.join(name);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(io(&format!("read {}", path.display()), e)),
    };
    if bytes.last().is_none_or(|b| *b == b'\n') {
        return Ok(());
    }
    let keep = bytes.iter().rposition(|b| *b == b'\n').map_or(0, |i| i + 1);
    let fragment = &bytes[keep..];
    let note_line = json!({
        "repaired_at": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        "file": name, "kept_bytes": keep, "dropped_bytes": fragment.len(),
        "dropped_sha256": hex::encode(Sha256::digest(fragment)),
        "dropped_text": String::from_utf8_lossy(&fragment[..fragment.len().min(512)]),
    });
    let log = case_dir.join(REPAIRS);
    if std::fs::read(&log).is_ok_and(|b| b.last().is_some_and(|c| *c != b'\n')) {
        append_bytes(&log, b"\n")?;
    }
    append_value(&log, &note_line)?;
    let file = OpenOptions::new()
        .write(true)
        .open(&path)
        .map_err(|e| io(&format!("open {}", path.display()), e))?;
    note(&path);
    file.set_len(keep as u64)
        .and_then(|()| file.sync_all())
        .map_err(|e| io(&format!("repair {}", path.display()), e))
}

fn apply(
    case_dir: &Path,
    st: &CaseState,
    batch: Batch,
    field: bool,
    derive: bool,
) -> Result<Committed, ContractError> {
    std::fs::create_dir_all(case_dir).map_err(|e| io("create the case directory", e))?;
    let mut repaired = HashSet::new();
    let mut committed = Committed::default();
    let mut records = st.records.clone();
    let mut case_json = st.case_json.clone();
    for op in batch.ops {
        match op {
            Op::Record(r) => {
                repair_tail(case_dir, RECORDS, &mut repaired)?;
                append_value(&case_dir.join(RECORDS), &r)?;
                records.push(r);
            }
            Op::Event(e) => {
                repair_tail(case_dir, EVENTS, &mut repaired)?;
                append_value(
                    &case_dir.join(EVENTS),
                    &serde_json::to_value(&e).map_err(|x| integrity(x.to_string()))?,
                )?;
                committed.events.push(e);
            }
            Op::Append { file, line } => {
                repair_tail(case_dir, &file, &mut repaired)?;
                append_value(&case_dir.join(&file), &line)?;
            }
            Op::Case(v) => {
                let previous = if field {
                    case_json.as_ref()
                } else {
                    st.case_json.as_ref()
                };
                if previous == Some(&v) {
                    continue;
                }
                let bytes = serde_json::to_vec_pretty(&v).map_err(|e| integrity(e.to_string()))?;
                replace_bytes(case_dir, &case_dir.join("case.json"), &bytes)?;
                case_json = Some(v);
            }
            Op::Plan(v) => {
                let bytes = serde_json::to_vec_pretty(&v).map_err(|e| integrity(e.to_string()))?;
                replace_bytes(case_dir, &case_dir.join("plan.json"), &bytes)?;
            }
            Op::RunFile { rel, bytes } => {
                replace_bytes(case_dir, &case_dir.join("runs").join(rel), &bytes)?;
            }
        }
    }
    if field && derive {
        // The ledger file exists from creation on: it is the Field Case
        // marker no deletion of the plan or the document can undo.
        if !case_dir.join(RECORDS).exists() {
            append_bytes(&case_dir.join(RECORDS), b"")?;
        }
        // The projection the ledger determines, in the same commit.
        let case_id = case_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let derived = derived_document(case_id, &records);
        let current: Option<Value> = std::fs::read(case_dir.join(DOCUMENT))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok());
        if current.as_ref() != Some(&derived) {
            let mut bytes =
                serde_json::to_vec_pretty(&derived).map_err(|e| integrity(e.to_string()))?;
            bytes.push(b'\n');
            replace_bytes(case_dir, &case_dir.join(DOCUMENT), &bytes)?;
        }
        write_index(case_dir, &records, st.events.len() + committed.events.len())?;
    }
    Ok(committed)
}

// ---------------------------------------------------------------------------
// Path-based entry points for the writers that name a file
// ---------------------------------------------------------------------------

/// The case directory a path belongs to, and the path inside it: the
/// directory directly under a `cases` directory, else the topmost ancestor
/// that holds a Field Case artifact or a plan.
pub fn case_of(path: &Path) -> Option<(PathBuf, PathBuf)> {
    let mut found: Option<PathBuf> = None;
    for dir in path.ancestors().skip(1) {
        let under_cases = dir
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|n| n == "cases");
        if under_cases || dir.join("plan.json").is_file() || is_field_case(dir) {
            found = Some(dir.to_path_buf());
            if under_cases {
                break;
            }
        }
    }
    let dir = found?;
    let rel = path.strip_prefix(&dir).ok()?.to_path_buf();
    Some((dir, rel))
}

fn op_for_file(rel: &str, value: &Value) -> Result<Op, ContractError> {
    let derived = || {
        refuse(
            "KIND_UNKNOWN",
            "this file is a projection the commit derives; it is not written directly",
        )
    };
    Ok(match classify(rel) {
        Some(FileClass::Ledger) => Op::Record(value.clone()),
        Some(FileClass::CaseJson) => Op::Case(value.clone()),
        Some(FileClass::PlanJson) => Op::Plan(value.clone()),
        Some(FileClass::RunOutput) => Op::RunFile {
            rel: rel.strip_prefix("runs/").unwrap_or(rel).to_string(),
            bytes: serde_json::to_vec_pretty(value).map_err(|e| integrity(e.to_string()))?,
        },
        Some(FileClass::Owned(_) | FileClass::Outbox | FileClass::Execution) => Op::Append {
            file: rel.to_string(),
            line: value.clone(),
        },
        Some(FileClass::Document | FileClass::Events) => return Err(derived()),
        None => {
            return Err(refuse(
                "KIND_UNKNOWN",
                "the guard has no rule for this file of the case",
            ))
        }
    })
}

/// Create the empty event log of a case that has none (legacy migration). A
/// Field Case's log is only ever appended by events: refused for one.
pub fn init_events(case_dir: &Path) -> Result<(), ContractError> {
    let _lock = CaseLock::acquire(case_dir)?;
    if case_dir.join(EVENTS).exists() {
        return Ok(());
    }
    if is_field_case(case_dir) {
        return Err(refuse(
            "KIND_UNKNOWN",
            "the event log of a Field Case is created only with its events",
        ));
    }
    std::fs::create_dir_all(case_dir).map_err(|e| io("create the case directory", e))?;
    append_bytes(&case_dir.join(EVENTS), b"")
}

/// Commit one file write named by path: a JSON document for `case.json`,
/// `plan.json` or a run file, a line for the stores. `Ok(false)` is a path that
/// is not in a case directory (the caller writes it as it likes). A file of a
/// case that is not a Field Case and has no rule is not refused, but the
/// caller still writes it only through [`commit_plain`].
pub fn commit_file(path: &Path, value: &Value) -> Result<bool, ContractError> {
    let Some((dir, rel)) = case_of(path) else {
        return Ok(false);
    };
    let name = rel.to_string_lossy().replace('\\', "/");
    let op = match op_for_file(&name, value) {
        Ok(op) => op,
        Err(e) => {
            // Deny by default for a Field Case; a plain case's own files pass.
            let _lock = CaseLock::acquire(&dir)?;
            if load_verified(&dir, false)?.field {
                return Err(e);
            }
            return Ok(false);
        }
    };
    commit(&dir, Batch::new().with(op))?;
    Ok(true)
}

/// Several store lines of ONE case committed as one batch: all or nothing.
/// `Ok(false)` when the files are not of a case directory (nothing written).
pub fn commit_files(items: &[(std::path::PathBuf, Value)]) -> Result<bool, ContractError> {
    let mut dir: Option<std::path::PathBuf> = None;
    let mut batch = Batch::new();
    for (path, value) in items {
        let Some((d, rel)) = case_of(path) else {
            return Ok(false);
        };
        if dir.as_ref().is_some_and(|x| *x != d) {
            return Ok(false);
        }
        let name = rel.to_string_lossy().replace('\\', "/");
        batch = batch.with(op_for_file(&name, value)?);
        dir = Some(d);
    }
    match dir {
        Some(d) => commit(&d, batch).map(|_| true),
        None => Ok(false),
    }
}

/// The whole consequence of `first` for the engine: `first` itself, the
/// automatic milestones that follow (an unassessed gate stops the cascade),
/// and the case close when the plan is then finished. A verb admits this
/// batch BEFORE it writes the first event, so a close refused by R11 leaves
/// nothing behind and a retry cannot append a second completion.
pub fn cascade_batch(
    case_dir: &Path,
    items: &[sea_forge_core::types::PlanItem],
    events: &[TraceEvent],
    first: (TraceKind, Option<String>),
) -> Vec<(TraceKind, Option<String>)> {
    use sea_forge_planner::case_engine::{next_case_actions, CaseAction};
    let mut sim = events.to_vec();
    sim.push(hypothetical(&first.0, first.1.as_deref()));
    let mut batch = vec![first];
    let field = is_field_case(case_dir);
    while let [action] = next_case_actions(items, &sim).as_slice() {
        match action {
            CaseAction::AchieveMilestone(id) => {
                if field && super::field_guard::gate_item_check(case_dir, id, "complete").is_err() {
                    break;
                }
                batch.push((TraceKind::MilestoneAchieved, Some(id.clone())));
                sim.push(hypothetical(&TraceKind::MilestoneAchieved, Some(id)));
            }
            CaseAction::CompleteCase => {
                batch.push((TraceKind::CaseClosed, None));
                break;
            }
            _ => break,
        }
    }
    batch
}

/// Closed means only the finishing action (C2). An engine pass is a sequence
/// of writes, so on a closed case it is admitted as a whole or not at all:
/// does the pass, run by the engine alone (sandboxed and agent items assumed
/// to complete, human work parks it), end the plan? The caller refuses with
/// `CASE_CLOSED` before the first write when it does not.
pub fn engine_pass_finishes(st: &CaseState) -> bool {
    use sea_forge_core::types::ItemKind;
    use sea_forge_planner::case_engine::{next_case_actions, CaseAction};
    let Some(plan) = &st.plan else {
        return false;
    };
    let mut sim = st.events.clone();
    for _ in 0..(plan.items.len() * 4 + 4) {
        let actions = next_case_actions(&plan.items, &sim);
        let Some(action) = actions.first() else {
            return false;
        };
        match action {
            CaseAction::CompleteCase => return true,
            CaseAction::TerminateCase { .. } | CaseAction::ParkHumanTask(_) => return false,
            CaseAction::Enable(id) => sim.push(hypothetical(&TraceKind::ItemEnabled, Some(id))),
            CaseAction::Activate(id) => {
                let engine_run = plan.items.iter().any(|i| {
                    &i.plan_item_id == id
                        && matches!(i.item_kind, ItemKind::SandboxedTask | ItemKind::AgentTask)
                });
                if !engine_run {
                    return false;
                }
                sim.push(hypothetical(&TraceKind::ItemActivated, Some(id)));
                sim.push(hypothetical(&TraceKind::ItemCompleted, Some(id)));
            }
            CaseAction::AchieveMilestone(id) => {
                if gate_check_records(&st.records, id, "complete").is_err() {
                    return false;
                }
                sim.push(hypothetical(&TraceKind::MilestoneAchieved, Some(id)));
            }
        }
    }
    false
}
