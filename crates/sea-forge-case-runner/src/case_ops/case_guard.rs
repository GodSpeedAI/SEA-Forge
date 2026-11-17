//! THE choke point of every Field Case mutation (WP-21 part B, review round 3).
//!
//! Every writer of a case (the two event appenders, the record appender of
//! every dialect, every other file of the case store, the CLI verbs, the
//! server verbs, delegation and execution) calls [`admit_write`] immediately
//! before its append. The call takes the one cross-process case lock (a file
//! lock on the case directory, shared by the CLI and the server) and the
//! returned [`WriteToken`] holds it: the guard and the append are one critical
//! section, with no release in between. A long execution re-admits at each
//! write.
//!
//! The guard decides from the LEDGER, never from `case.json`: the hash-chained
//! records (the disposition lifecycle) and the event log. For a Field Case it
//! applies, in this order:
//!
//! 1. full store integrity (hash chain and digests of every stored record,
//!    not only that they parse) and the document `field_case.json`, for every
//!    write in either dialect: otherwise `STORE_INTEGRITY`, nothing written;
//! 2. the lifecycle (README §6, R6, R13, D8, D32, D42): suspended refuses
//!    everything except the case-level `continue` and `close-*` records and
//!    the revision, strategy and gate records; closed refuses everything
//!    except late settlement and the single action that ends the plan, never
//!    a repeat of one already done;
//! 3. the hard-gate rule, for ANY event that completes a gate item or
//!    milestone, whatever its type (human, automatic, transition);
//! 4. R11 on every close and on terminate.
//!
//! Deny by default: [`event_class`], [`record_class`] and [`store_class`] are
//! matches with no wildcard arm, so a new event type, record kind or store
//! file fails to compile (or is refused) until it gets a rule.
//!
//! A case whose plan is not `fde.field_case@1` takes the lock and is otherwise
//! not touched.

use super::field_guard::{gate_item_check, is_field_case, Refusal};
use sea_forge_core::types::{CasePlan, TraceEvent, TraceKind};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// The contract's refusal (`code`, `message`, `no_side_effect`).
pub type ContractError = Refusal;

/// The case-record file: the hash-chained ledger of a Field Case.
pub const RECORDS: &str = "field-records.jsonl";
/// The case-event log.
pub const EVENTS: &str = "case-events.jsonl";
/// The document of a Field Case.
pub const DOCUMENT: &str = "field_case.json";
const LOCK_FILE: &str = ".case.lock";
const ZERO_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

// ---------------------------------------------------------------------------
// The case lock
// ---------------------------------------------------------------------------

thread_local! {
    /// The case locks this thread holds: a nested admission (a verb that holds
    /// the lock and then appends) re-uses it instead of deadlocking on its own
    /// file lock.
    static HELD: RefCell<HashMap<PathBuf, (usize, File)>> = RefCell::new(HashMap::new());
}

/// The cross-process lock of one case directory. Held until dropped; a thread
/// that already holds it nests (counted). Not `Send`: it belongs to the
/// thread that took it.
pub struct CaseLock {
    key: PathBuf,
    _thread_bound: PhantomData<*const ()>,
}

impl CaseLock {
    /// Block until this thread holds the case lock of `case_dir`.
    pub fn acquire(case_dir: &Path) -> Result<Self, ContractError> {
        let key = std::fs::canonicalize(case_dir).unwrap_or_else(|_| case_dir.to_path_buf());
        let nested = HELD.with(|held| {
            held.borrow_mut().get_mut(&key).map(|slot| {
                slot.0 += 1;
            })
        });
        if nested.is_none() {
            let file = OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .open(key.join(LOCK_FILE))
                .map_err(|e| integrity(format!("open the case lock: {e}")))?;
            file.lock()
                .map_err(|e| integrity(format!("take the case lock: {e}")))?;
            HELD.with(|held| held.borrow_mut().insert(key.clone(), (1, file)));
        }
        Ok(Self {
            key,
            _thread_bound: PhantomData,
        })
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

/// Proof that a write was admitted. It holds the case lock: keep it alive
/// until the append is done.
pub struct WriteToken {
    _lock: CaseLock,
}

// ---------------------------------------------------------------------------
// What is proposed
// ---------------------------------------------------------------------------

/// The write a caller is about to make.
pub enum Proposed<'a> {
    /// A trace event of `case-events.jsonl`.
    Event {
        kind: &'a TraceKind,
        item: Option<&'a str>,
    },
    /// A verb's whole consequence, admitted at once before any of it is
    /// written: each event is judged on the ledger plus the events before it
    /// in the batch, so a close that the batch would reach without its
    /// disposition refuses the batch with nothing written.
    Batch {
        events: &'a [(TraceKind, Option<String>)],
    },
    /// A line of `field-records.jsonl` (every dialect): the full record.
    Record { record: &'a Value },
    /// `case.json`, `plan.json` or a run file (`"runs"`), with the value about
    /// to be written.
    Projection { file: &'a str, value: &'a Value },
    /// Any other file of the case store, by file name.
    Store { file: &'a str },
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
    // A torn trailing fragment is not a line (the appender repairs it).
    let end = bytes.iter().rposition(|b| *b == b'\n').map_or(0, |i| i + 1);
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

/// The trace events written so far (a line that does not parse is skipped:
/// the event log feeds no integrity rule).
fn events_of(case_dir: &Path) -> Vec<TraceEvent> {
    let bytes = std::fs::read(case_dir.join(EVENTS)).unwrap_or_default();
    let end = bytes.iter().rposition(|b| *b == b'\n').map_or(0, |i| i + 1);
    bytes[..end]
        .split(|b| *b == b'\n')
        .filter_map(|line| serde_json::from_slice(line).ok())
        .collect()
}

fn plan_of(case_dir: &Path) -> Option<CasePlan> {
    serde_json::from_slice(&std::fs::read(case_dir.join("plan.json")).ok()?).ok()
}

/// Has the plan run out of things to do? Every item is finished, or the engine
/// has only the case close or terminate left.
fn plan_finished(case_dir: &Path, events: &[TraceEvent]) -> bool {
    use sea_forge_planner::case_engine::{next_case_actions, replay_case, CaseAction, ItemStatus};
    let Some(plan) = plan_of(case_dir) else {
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

/// The document requirement (README §6): `field_case.json` is present, is
/// JSON, names this case and is not ahead of its ledger. (A document BEHIND
/// its ledger is healed by the record appender, which owns the replay.)
fn check_document(case_dir: &Path, records: &[Value]) -> Result<(), ContractError> {
    let bytes = std::fs::read(case_dir.join(DOCUMENT))
        .map_err(|e| integrity(format!("{DOCUMENT} of a Field Case: {e}")))?;
    let doc: Value = serde_json::from_slice(&bytes)
        .map_err(|e| integrity(format!("{DOCUMENT} is not JSON: {e}")))?;
    let case_id = case_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if doc["case_id"].as_str() != Some(case_id) {
        return Err(integrity(format!("{DOCUMENT} names another case")));
    }
    // The document's structure: the template's members, every field group and
    // field present (the schema is closed), and the full schema when the host
    // installed it.
    let template = sea_forge_core::field_case::opened_document(case_id);
    let keys = |v: &Value| -> Vec<String> {
        v.as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    };
    if keys(&doc) != keys(&template) {
        return Err(integrity(format!(
            "{DOCUMENT} has other members than field-case.v1"
        )));
    }
    for (group, fields) in template["fields"].as_object().into_iter().flatten() {
        let have_fields = &doc["fields"][group];
        if !have_fields.is_object() || keys(have_fields) != keys(fields) {
            return Err(integrity(format!(
                "{DOCUMENT} fields.{group} is not field-case.v1"
            )));
        }
    }
    if let Some(validate) = DOCUMENT_VALIDATOR.get() {
        validate(&doc).map_err(|e| integrity(format!("{DOCUMENT}: {e}")))?;
    }
    let have = doc["revision"].as_u64().unwrap_or(0);
    let want = ledger_revision(records);
    if have > want {
        return Err(integrity(format!(
            "{DOCUMENT} is at revision {have}, ahead of its ledger ({want})"
        )));
    }
    Ok(())
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

/// What a record kind means to the guard.
enum RecordClass {
    /// `episode.frozen`: refused when suspended or closed.
    Freeze,
    /// `episode.settled`: refused when suspended; late evidence once closed.
    Settlement,
    /// `gate.assessed`, `field_case.revised`, `strategy.recorded`: accepted
    /// when suspended, refused once closed.
    Revision,
    /// `disposition.recorded`.
    Disposition,
}

/// `None` is a record kind the guard has no rule for: refused.
fn record_class(kind: &str) -> Option<RecordClass> {
    match kind {
        "episode.frozen" => Some(RecordClass::Freeze),
        "episode.settled" => Some(RecordClass::Settlement),
        "gate.assessed" | "field_case.revised" | "strategy.recorded" => Some(RecordClass::Revision),
        "disposition.recorded" => Some(RecordClass::Disposition),
        _ => None,
    }
}

/// What a store file means to the guard.
enum StoreClass {
    /// Bookkeeping of a record or a request that was admitted on its own
    /// (intents, decisions, outboxes, approvals, the document projection):
    /// lifecycle-neutral.
    Bookkeeping,
    /// Execution and delegation state (runs, links, assignments, execution
    /// decisions): refused when suspended; once closed only while the action
    /// that ends the plan is in flight.
    Execution,
}

/// `None` is a store file the guard has no rule for: refused.
fn store_class(file: &str) -> Option<StoreClass> {
    match file {
        "field-freeze-intents.jsonl"
        | "field-decisions.jsonl"
        | "field-rt-outbox.jsonl"
        | "field-anchor-outbox.jsonl"
        | "field-approvals.jsonl"
        | "field-repairs.jsonl"
        | "field-interaction-outbox.jsonl"
        | DOCUMENT => Some(StoreClass::Bookkeeping),
        "field-runs.jsonl"
        | "field-links.jsonl"
        | "field-assignments.jsonl"
        | "field-exec-decisions.jsonl" => Some(StoreClass::Execution),
        _ => None,
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
fn finishes_with(case_dir: &Path, events: &[TraceEvent], added: Vec<TraceEvent>) -> bool {
    use sea_forge_planner::case_engine::{next_case_actions, CaseAction};
    let Some(plan) = plan_of(case_dir) else {
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
        if gate_item_check(case_dir, &id, "complete").is_err() {
            break;
        }
        sim.push(hypothetical(&TraceKind::MilestoneAchieved, Some(&id)));
    }
    plan_finished(case_dir, &sim)
}

/// The lifecycle for a write that is part of the action that ends the plan.
/// A closed case admits only an action whose RESULT finishes the plan,
/// computed by simulating it (README §7 R13, D42); anything else, and any
/// repeat once the case has ended, is `CASE_CLOSED`.
fn in_flight(
    life: Lifecycle,
    case_dir: &Path,
    events: &[TraceEvent],
    hypo: Hypo,
) -> Result<(), ContractError> {
    match life {
        Lifecycle::Open => Ok(()),
        Lifecycle::Suspended => Err(suspended()),
        Lifecycle::Closed => {
            if ended(events) || plan_finished(case_dir, events) {
                return Err(closed());
            }
            let ok = match hypo {
                Hypo::Finishing(kind, item) => {
                    finishes_with(case_dir, events, vec![hypothetical(kind, item)])
                }
                Hypo::InFlight(Some(item)) => finishes_with(
                    case_dir,
                    events,
                    vec![hypothetical(&TraceKind::ItemCompleted, Some(item))],
                ),
                Hypo::InFlight(None) => plan_of(case_dir).is_some_and(|plan| {
                    plan.items.iter().any(|i| {
                        finishes_with(
                            case_dir,
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
    case_dir: &Path,
    kind: &TraceKind,
    item: Option<&str>,
    life: Lifecycle,
    events: &[TraceEvent],
) -> Result<(), ContractError> {
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
            in_flight(life, case_dir, events, hypo)?;
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
                gate_item_check(case_dir, item, how)?;
            }
            Ok(())
        }
        EventClass::Telemetry => in_flight(life, case_dir, events, Hypo::InFlight(None)),
        EventClass::Structure => match life {
            Lifecycle::Open => Ok(()),
            Lifecycle::Suspended => Err(suspended()),
            Lifecycle::Closed => Err(closed()),
        },
        // R11: any path to a terminal state needs a case-level close-*
        // disposition (README §7: "any path to a terminal state with no
        // case-level close-* disposition ... CLOSE_DISPOSITION_REQUIRED").
        // Both the close and the terminate end the case, once.
        EventClass::Close | EventClass::Terminate => match life {
            Lifecycle::Suspended => Err(suspended()),
            Lifecycle::Open => Err(refuse(
                "CLOSE_DISPOSITION_REQUIRED",
                "this action ends the case and no case-level close disposition exists",
            )),
            Lifecycle::Closed => {
                if ended(events) {
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

fn admit_record(
    case_dir: &Path,
    record: &Value,
    records: &[Value],
    life: Lifecycle,
) -> Result<(), ContractError> {
    let kind = record["kind"].as_str().unwrap_or("");
    let Some(class) = record_class(kind) else {
        return Err(refuse(
            "KIND_UNKNOWN",
            "the guard has no rule for this record kind",
        ));
    };
    // The record is the next link of the chain, as the ledger is now.
    let seq = records.len() as u64 + 1;
    let prev = records
        .last()
        .and_then(|r| r["hash"].as_str())
        .unwrap_or(ZERO_HASH);
    if record["seq"].as_u64() != Some(seq) || record["prev_hash"].as_str() != Some(prev) {
        return Err(integrity(
            "the record does not extend the ledger as it is now (stale write)".into(),
        ));
    }
    let _ = case_dir;
    match (life, class) {
        (Lifecycle::Open, _) => Ok(()),
        (Lifecycle::Closed, RecordClass::Settlement) => Ok(()),
        (Lifecycle::Closed, _) => Err(closed()),
        (Lifecycle::Suspended, RecordClass::Revision) => Ok(()),
        (Lifecycle::Suspended, RecordClass::Disposition) => {
            let d = record["disposition"].as_str().unwrap_or("");
            if record["level"] == "case" && (d == "continue" || d.starts_with("close-")) {
                Ok(())
            } else {
                Err(suspended())
            }
        }
        (Lifecycle::Suspended, RecordClass::Freeze | RecordClass::Settlement) => Err(suspended()),
    }
}

/// The aux stores that decide an invariant: each must parse completely.
const STRICT_STORES: [&str; 8] = [
    "field-freeze-intents.jsonl",
    "field-decisions.jsonl",
    "field-rt-outbox.jsonl",
    "field-anchor-outbox.jsonl",
    "field-runs.jsonl",
    "field-links.jsonl",
    "field-approvals.jsonl",
    "field-assignments.jsonl",
];

/// Installed by the server: the full `field-case.v1` schema check.
pub type DocumentValidator = fn(&Value) -> Result<(), String>;
static DOCUMENT_VALIDATOR: std::sync::OnceLock<DocumentValidator> = std::sync::OnceLock::new();

/// Register the full schema validation of the document (the server owns the
/// vendored schemas); the guard always also checks the document's structure.
pub fn set_document_validator(validator: DocumentValidator) {
    let _ = DOCUMENT_VALIDATOR.set(validator);
}

/// Every store of a Field Case parses and verifies before any write: the
/// plan, the aux stores, the document (structure, and the schema when the
/// host installed it) and the record chain. Corruption of any of them fails
/// closed with `STORE_INTEGRITY`, whatever the plan says.
fn verify_stores(case_dir: &Path, creating: bool) -> Result<Vec<Value>, ContractError> {
    if case_dir.join("plan.json").exists() && plan_of(case_dir).is_none() {
        return Err(integrity("plan.json does not parse as a case plan".into()));
    }
    for name in STRICT_STORES {
        let path = case_dir.join(name);
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(integrity(format!("read {name}: {e}"))),
        };
        let end = bytes.iter().rposition(|b| *b == b'\n').map_or(0, |i| i + 1);
        for (i, line) in bytes[..end].split(|b| *b == b'\n').enumerate() {
            if !line.is_empty() && serde_json::from_slice::<Value>(line).is_err() {
                return Err(integrity(format!("{name} line {} is not JSON", i + 1)));
            }
        }
    }
    let records = verified_records(case_dir)?;
    if !creating {
        check_document(case_dir, &records)?;
    }
    Ok(records)
}

fn proposed_event_check(
    case_dir: &Path,
    proposed: &Proposed,
    life: Lifecycle,
) -> Result<(), ContractError> {
    match proposed {
        Proposed::Event { kind, item } => {
            admit_event(case_dir, kind, *item, life, &events_of(case_dir))
        }
        Proposed::Batch { events: batch } => {
            let mut sim = events_of(case_dir);
            for (kind, item) in batch.iter() {
                admit_event(case_dir, kind, item.as_deref(), life, &sim)?;
                sim.push(hypothetical(kind, item.as_deref()));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Admit one write on the case at `case_dir`: take the case lock, decide from
/// the ledger, and return the token that holds the lock. Call it immediately
/// before the append and keep the token until the append is done. `Err` is the
/// contract's refusal; nothing is to be written.
pub fn admit_write(case_dir: &Path, proposed: &Proposed) -> Result<WriteToken, ContractError> {
    let lock = CaseLock::acquire(case_dir)?;
    let token = WriteToken { _lock: lock };
    if !is_field_case(case_dir) {
        return Ok(token);
    }
    let creating = matches!(
        proposed,
        Proposed::Event {
            kind: TraceKind::CaseCreated | TraceKind::PlanCreated,
            ..
        }
    );
    // (e) full store integrity and (d) the document, in either dialect.
    let records = verify_stores(case_dir, creating)?;
    let life = lifecycle(&records);
    match proposed {
        Proposed::Event { .. } | Proposed::Batch { .. } => {
            proposed_event_check(case_dir, proposed, life)?
        }
        Proposed::Record { record } => admit_record(case_dir, record, &records, life)?,
        Proposed::Store { file } => {
            let Some(class) = store_class(file) else {
                return Err(refuse(
                    "KIND_UNKNOWN",
                    "the guard has no rule for this store file",
                ));
            };
            match class {
                StoreClass::Bookkeeping => {}
                StoreClass::Execution => {
                    in_flight(life, case_dir, &events_of(case_dir), Hypo::InFlight(None))?
                }
            }
        }
        Proposed::Projection { file, value } => admit_projection(case_dir, file, value, life)?,
    }
    Ok(token)
}

fn terminal_state(state: &str) -> bool {
    matches!(state, "completed" | "terminated")
}

/// `case.json`, `plan.json` and the run files under `runs/`: projections and
/// outputs of the ledger, written only while the ledger allows them.
fn admit_projection(
    case_dir: &Path,
    file: &str,
    value: &Value,
    life: Lifecycle,
) -> Result<(), ContractError> {
    match file {
        // A Field Case's plan is rewritten only by its plan mutation (an
        // event), never by a submission over it.
        "plan.json" => Err(refuse(
            "KIND_UNKNOWN",
            "the plan of a Field Case is rewritten only by its plan mutation",
        )),
        "case.json" => {
            let new = value["state"].as_str().unwrap_or("");
            if terminal_state(new) {
                // R11: any path to a terminal state needs the close disposition.
                return match life {
                    Lifecycle::Closed => Ok(()),
                    Lifecycle::Suspended => Err(suspended()),
                    Lifecycle::Open => Err(refuse(
                        "CLOSE_DISPOSITION_REQUIRED",
                        "this write ends the case and no case-level close disposition exists",
                    )),
                };
            }
            let current: Value = std::fs::read(case_dir.join("case.json"))
                .ok()
                .and_then(|b| serde_json::from_slice(&b).ok())
                .unwrap_or(Value::Null);
            match life {
                Lifecycle::Open => Ok(()),
                Lifecycle::Suspended => Err(suspended()),
                // A closed case is not reopened by a state write.
                Lifecycle::Closed => {
                    if terminal_state(current["state"].as_str().unwrap_or("")) {
                        Err(closed())
                    } else {
                        Ok(())
                    }
                }
            }
        }
        _ => in_flight(life, case_dir, &events_of(case_dir), Hypo::InFlight(None)),
    }
}

/// The case directory a path belongs to, and the path inside it: the topmost
/// ancestor that holds a `plan.json` (run directories hold their own).
fn case_of(path: &Path) -> Option<(PathBuf, PathBuf)> {
    let mut found: Option<PathBuf> = None;
    for dir in path.ancestors().skip(1) {
        if dir.join("plan.json").is_file() || dir.join("field_case.json").is_file() {
            found = Some(dir.to_path_buf());
        }
    }
    let dir = found?;
    let rel = path.strip_prefix(&dir).ok()?.to_path_buf();
    Some((dir, rel))
}

/// [`admit_write`] for a write to a named file of the case store (the one
/// primitive the file-backed stores write through). A path that is not inside
/// a case directory is not a case write.
pub fn admit_file_write(path: &Path, value: &Value) -> Result<Option<WriteToken>, ContractError> {
    let Some((dir, rel)) = case_of(path) else {
        return Ok(None);
    };
    let name = rel.to_string_lossy().to_string();
    if name == RECORDS {
        admit_write(&dir, &Proposed::Record { record: value }).map(Some)
    } else if name == "case.json" || name == "plan.json" || rel.starts_with("runs") {
        admit_write(
            &dir,
            &Proposed::Projection {
                file: if rel.starts_with("runs") {
                    "runs"
                } else {
                    &name
                },
                value,
            },
        )
        .map(Some)
    } else {
        admit_write(&dir, &Proposed::Store { file: &name }).map(Some)
    }
}
