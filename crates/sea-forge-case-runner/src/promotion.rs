//! WP-22: the promotion saga and world stamping.
//!
//! One state machine over an append-only step log. A step is
//! `(precondition read from the log and the bytes on disk) -> idempotent effect
//! keyed by the saga key -> step record`. Nothing else holds saga state. The
//! reconciler is the same machine ([`drive`]) started in [`Mode::Recover`]: its
//! only outputs are resume to completion, roll back to the pre-saga bytes, or
//! uncertain (a sticky terminal record; the world is then refused as promoted).
//!
//! Layout under the store root: `pins.jsonl` (append-only pin history),
//! `frozen/<id>.json`, `stamps/<id>.json`, `evidence/<name>.json` (immutable,
//! the `world_ref` is inside the digest), `promotions/<id>/steps.jsonl`, and
//! `locks/<proposal>.lock` (one lock per proposal serializes its sagas).
//! In-flight cases keep their pin: a case's pin is written once, through the
//! case guard's `commit`, and no saga step ever touches a case directory.

use crate::case_ops::case_guard::{commit, digest_of, Batch, Op};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

/// A saga step. [`Step::ALL`] drives the crash-injection seam of the tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Step {
    Begin,
    Freeze,
    Pin,
    Stamp,
    Complete,
    Rollback,
}

impl Step {
    pub const ALL: [Step; 6] = [
        Step::Begin,
        Step::Freeze,
        Step::Pin,
        Step::Stamp,
        Step::Complete,
        Step::Rollback,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Step::Begin => "begin",
            Step::Freeze => "freeze",
            Step::Pin => "pin",
            Step::Stamp => "stamp",
            Step::Complete => "complete",
            Step::Rollback => "rollback",
        }
    }

    /// Exhaustive: a new variant breaks the build here, and `ALL` is then
    /// checked against it by the tests.
    pub const fn ordinal(self) -> usize {
        match self {
            Step::Begin => 0,
            Step::Freeze => 1,
            Step::Pin => 2,
            Step::Stamp => 3,
            Step::Complete => 4,
            Step::Rollback => 5,
        }
    }

    fn from_name(name: &str) -> Option<Step> {
        Step::ALL.into_iter().find(|s| s.name() == name)
    }
}

/// Where in a step a crash is injected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    BeforeEffect,
    /// Half of the effect's bytes reach a temp file; the effect never lands.
    MidEffect,
    AfterEffect,
    /// A torn (unterminated) step record.
    MidRecord,
    AfterRecord,
}

impl Phase {
    pub const ALL: [Phase; 5] = [
        Phase::BeforeEffect,
        Phase::MidEffect,
        Phase::AfterEffect,
        Phase::MidRecord,
        Phase::AfterRecord,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FaultPoint {
    pub step: Step,
    pub phase: Phase,
}

impl FaultPoint {
    /// Every fault point, generated from `Step::ALL` x `Phase::ALL`.
    pub fn all() -> Vec<FaultPoint> {
        Step::ALL
            .into_iter()
            .flat_map(|step| {
                Phase::ALL
                    .into_iter()
                    .map(move |phase| FaultPoint { step, phase })
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SagaError {
    Crash(FaultPoint),
    Refused { code: &'static str, message: String },
    Uncertain(String),
    Io(String),
}

fn io_err(what: &str, e: impl std::fmt::Display) -> SagaError {
    SagaError::Io(format!("{what}: {e}"))
}

fn refused(code: &'static str, message: impl Into<String>) -> SagaError {
    SagaError::Refused {
        code,
        message: message.into(),
    }
}

/// The saga key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    pub proposal_id: String,
    pub candidate_world_ref: String,
}

impl Key {
    pub fn id(&self) -> String {
        let mut h = Sha256::new();
        h.update(self.proposal_id.as_bytes());
        h.update([0]);
        h.update(self.candidate_world_ref.as_bytes());
        hex::encode(h.finalize())
    }
}

/// What a promotion asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Promotion {
    pub key: Key,
    pub base_world_ref: String,
    /// Keys the semantic diff changed.
    pub changed_keys: Vec<String>,
    /// The semantic diff's old key to new key mapping.
    pub mapping: Vec<(String, String)>,
}

impl Promotion {
    fn to_value(&self) -> Value {
        json!({
            "proposal_id": self.key.proposal_id,
            "candidate_world_ref": self.key.candidate_world_ref,
            "base_world_ref": self.base_world_ref,
            "changed_keys": self.changed_keys,
            "mapping": self.mapping,
        })
    }

    fn from_value(v: &Value) -> Option<Promotion> {
        let s = |p: &str| v[p].as_str().map(str::to_owned);
        Some(Promotion {
            key: Key {
                proposal_id: s("proposal_id")?,
                candidate_world_ref: s("candidate_world_ref")?,
            },
            base_world_ref: s("base_world_ref")?,
            changed_keys: v["changed_keys"]
                .as_array()?
                .iter()
                .map(|x| x.as_str().map(str::to_owned))
                .collect::<Option<_>>()?,
            mapping: v["mapping"]
                .as_array()?
                .iter()
                .map(|p| Some((p[0].as_str()?.to_owned(), p[1].as_str()?.to_owned())))
                .collect::<Option<_>>()?,
        })
    }
}

/// W5: the mapping is total over the changed keys, injective, and unambiguous,
/// or the promotion is refused (before any write).
pub fn map_keys(
    changed: &[String],
    mapping: &[(String, String)],
) -> Result<BTreeMap<String, String>, SagaError> {
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    for (old, new) in mapping {
        if let Some(prev) = map.insert(old.clone(), new.clone()) {
            if prev != *new {
                return Err(refused(
                    "KEY_AMBIGUOUS",
                    format!("{old} maps to {prev} and {new}"),
                ));
            }
        }
    }
    let mut targets: BTreeMap<&String, &String> = BTreeMap::new();
    for (old, new) in &map {
        if let Some(other) = targets.insert(new, old) {
            return Err(refused(
                "KEY_NOT_INJECTIVE",
                format!("{other} and {old} both map to {new}"),
            ));
        }
    }
    for key in changed {
        if !map.contains_key(key) {
            return Err(refused(
                "KEY_UNMAPPED",
                format!("changed key {key} has no mapping"),
            ));
        }
    }
    Ok(map)
}

/// A saga's place, judged from the bytes on disk and the log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Promoted,
    PreSaga,
    /// Fail closed, surfaced for the operator; never usable as promoted.
    Uncertain(String),
    /// A saga that has not reached a terminal record (run or reconcile it).
    InProgress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Forward,
    Recover,
}

/// Crash injection: a fault point (at most one) the run dies at.
#[derive(Debug, Clone, Copy, Default)]
pub struct Faults(pub Option<FaultPoint>);

impl Faults {
    fn at(&self, step: Step, phase: Phase) -> Result<(), SagaError> {
        match self.0 {
            Some(p) if p.step == step && p.phase == phase => Err(SagaError::Crash(p)),
            _ => Ok(()),
        }
    }
}

// ---------------------------------------------------------------------------
// Bytes on disk
// ---------------------------------------------------------------------------

fn pins_at(root: &Path) -> PathBuf {
    root.join("pins.jsonl")
}
fn frozen_at(root: &Path, id: &str) -> PathBuf {
    root.join("frozen").join(format!("{id}.json"))
}
fn stamp_at(root: &Path, id: &str) -> PathBuf {
    root.join("stamps").join(format!("{id}.json"))
}
fn log_at(root: &Path, id: &str) -> PathBuf {
    root.join("promotions").join(id).join("steps.jsonl")
}

/// Complete lines of a JSONL file and whether the tail is torn.
fn read_lines(at: &Path) -> Result<(Vec<Value>, bool), SagaError> {
    let bytes = match fs::read(at) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((Vec::new(), false)),
        Err(e) => return Err(io_err("read a log", e)),
    };
    let torn = !bytes.is_empty() && bytes.last() != Some(&b'\n');
    let text = String::from_utf8_lossy(&bytes);
    let mut lines: Vec<&str> = text.split('\n').collect();
    lines.pop(); // the unterminated tail (or the empty string after the last newline)
    let mut out = Vec::new();
    for line in lines {
        out.push(
            serde_json::from_str(line)
                .map_err(|e| SagaError::Uncertain(format!("unreadable log line: {e}")))?,
        );
    }
    Ok((out, torn))
}

fn cut_torn(at: &Path) -> Result<(), SagaError> {
    let Ok(bytes) = fs::read(at) else {
        return Ok(());
    };
    if let Some(end) = bytes.iter().rposition(|b| *b == b'\n') {
        if end + 1 != bytes.len() {
            let f = fs::OpenOptions::new()
                .write(true)
                .open(at)
                .map_err(|e| io_err("open a torn log", e))?;
            f.set_len(end as u64 + 1)
                .map_err(|e| io_err("cut a torn tail", e))?;
        }
    } else if !bytes.is_empty() {
        fs::write(at, b"").map_err(|e| io_err("cut a torn log", e))?;
    }
    Ok(())
}

/// Create `dir` and every missing ancestor, then fsync each created directory
/// and its parent, so a crash cannot lose the directory entry.
fn mkdir_durable(dir: &Path) -> Result<(), SagaError> {
    let mut missing = Vec::new();
    let mut cur = Some(dir);
    while let Some(d) = cur {
        if d.as_os_str().is_empty() || d.exists() {
            break;
        }
        missing.push(d.to_path_buf());
        cur = d.parent();
    }
    fs::create_dir_all(dir).map_err(|e| io_err("create a directory", e))?;
    for d in missing.iter() {
        fsync_dir(d)?;
        if let Some(p) = d.parent() {
            fsync_dir(p)?;
        }
    }
    Ok(())
}

fn fsync_dir(d: &Path) -> Result<(), SagaError> {
    let d = if d.as_os_str().is_empty() {
        Path::new(".")
    } else {
        d
    };
    fs::File::open(d)
        .and_then(|f| f.sync_all())
        .map_err(|e| io_err("fsync a directory", e))
}

fn append_line(
    at: &Path,
    line: &Value,
    torn_at: Option<&Faults>,
    f: (Step, Phase),
) -> Result<(), SagaError> {
    let created = !at.exists();
    if let Some(parent) = at.parent() {
        mkdir_durable(parent)?;
    }
    let mut bytes = serde_json::to_vec(line).map_err(|e| io_err("encode", e))?;
    bytes.push(b'\n');
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(at)
        .map_err(|e| io_err("open a log", e))?;
    if let Some(faults) = torn_at {
        if faults.at(f.0, f.1).is_err() {
            file.write_all(&bytes[..bytes.len() / 2])
                .map_err(|e| io_err("write", e))?;
            return faults.at(f.0, f.1);
        }
    }
    file.write_all(&bytes).map_err(|e| io_err("write", e))?;
    file.sync_all().map_err(|e| io_err("sync", e))?;
    if created {
        if let Some(parent) = at.parent() {
            fsync_dir(parent)?;
        }
    }
    Ok(())
}

/// Land `bytes` at `at` atomically. An existing file must carry the same
/// bytes (idempotent by content), else the saga is uncertain.
fn put_whole(at: &Path, bytes: &[u8], mid: Option<(&Faults, Step)>) -> Result<(), SagaError> {
    match fs::read(at) {
        Ok(have) if have == bytes => return Ok(()),
        Ok(_) => {
            return Err(SagaError::Uncertain(format!(
                "{} exists with other bytes",
                at.file_name().unwrap_or_default().to_string_lossy()
            )))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(io_err("read", e)),
    }
    let tmp = at.with_extension("tmp");
    mkdir_durable(at.parent().unwrap_or(Path::new(".")))?;
    if let Some((faults, step)) = mid {
        if faults.at(step, Phase::MidEffect).is_err() {
            fs::write(&tmp, &bytes[..bytes.len() / 2]).map_err(|e| io_err("write", e))?;
            return faults.at(step, Phase::MidEffect);
        }
    }
    {
        let mut f = fs::File::create(&tmp).map_err(|e| io_err("write", e))?;
        f.write_all(bytes).map_err(|e| io_err("write", e))?;
        f.sync_all().map_err(|e| io_err("sync", e))?;
    }
    fs::rename(&tmp, at).map_err(|e| io_err("rename", e))?;
    fsync_dir(at.parent().unwrap_or(Path::new(".")))
}

fn clean_tmp(root: &Path, id: &str) {
    for at in [frozen_at(root, id), stamp_at(root, id)] {
        let _ = fs::remove_file(at.with_extension("tmp"));
    }
    if let Ok(rd) = fs::read_dir(root.join("promotions").join(id)) {
        for e in rd.flatten() {
            if e.path().extension().is_some_and(|x| x == "tmp") {
                let _ = fs::remove_file(e.path());
            }
        }
    }
}

fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

// ---------------------------------------------------------------------------
// The pin history and the store
// ---------------------------------------------------------------------------

/// Create the store with its baseline world. Idempotent for the same baseline.
pub fn init_store(root: &Path, baseline: &str) -> Result<(), SagaError> {
    let line = json!({"key": "genesis", "from": Value::Null, "to": baseline, "seq": 0});
    if pins_at(root).exists() {
        let (lines, _) = read_lines(&pins_at(root))?;
        return if lines.first() == Some(&line) {
            Ok(())
        } else {
            Err(refused("STORE_EXISTS", "the store has another baseline"))
        };
    }
    append_line(
        &pins_at(root),
        &line,
        None,
        (Step::Begin, Phase::AfterRecord),
    )
}

/// The current pin: `to` of the last complete pin line.
pub fn current_pin(root: &Path) -> Result<String, SagaError> {
    let (lines, _) = read_lines(&pins_at(root))?;
    lines
        .last()
        .and_then(|l| l["to"].as_str().map(str::to_owned))
        .ok_or_else(|| SagaError::Uncertain("no pin history".into()))
}

fn pin_history(root: &Path) -> Result<Vec<Value>, SagaError> {
    Ok(read_lines(&pins_at(root))?.0)
}

fn expected_pin_line(req: &Promotion, seq: u64) -> Value {
    json!({"key": req.key.id(), "from": req.base_world_ref,
           "to": req.key.candidate_world_ref, "seq": seq})
}

fn frozen_bytes(req: &Promotion) -> Result<Vec<u8>, SagaError> {
    let map = map_keys(&req.changed_keys, &req.mapping)?;
    serde_json::to_vec(
        &json!({"key": req.key.id(), "world_ref": req.key.candidate_world_ref,
        "mapping": map}),
    )
    .map_err(|e| io_err("encode", e))
}

fn stamp_bytes(req: &Promotion, seq: u64) -> Result<Vec<u8>, SagaError> {
    serde_json::to_vec(
        &json!({"key": req.key.id(), "world_ref": req.key.candidate_world_ref,
        "pin_seq": seq}),
    )
    .map_err(|e| io_err("encode", e))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Presence {
    Absent,
    Match,
    Mismatch,
}

struct Ground {
    frozen: Presence,
    pin: Presence,
    stamp: Presence,
    pin_seq: Option<u64>,
}

fn file_presence(at: &Path, want: &[u8]) -> Presence {
    match fs::read(at) {
        Ok(b) if b == want => Presence::Match,
        Ok(_) => Presence::Mismatch,
        Err(_) => Presence::Absent,
    }
}

/// Read the effects of `req` from disk.
fn ground(root: &Path, req: &Promotion) -> Result<Ground, SagaError> {
    let id = req.key.id();
    let frozen = match frozen_bytes(req) {
        Ok(b) => file_presence(&frozen_at(root, &id), &b),
        Err(_) => Presence::Mismatch,
    };
    let hist = pin_history(root)?;
    let line = hist.iter().find(|l| l["key"] == id.as_str());
    let (pin, pin_seq) = match line {
        None => (Presence::Absent, None),
        Some(l) => {
            let seq = l["seq"].as_u64();
            let ok = seq.is_some_and(|s| *l == expected_pin_line(req, s));
            (
                if ok {
                    Presence::Match
                } else {
                    Presence::Mismatch
                },
                seq,
            )
        }
    };
    let stamp = match pin_seq.map(|s| stamp_bytes(req, s)) {
        Some(Ok(b)) => file_presence(&stamp_at(root, &id), &b),
        _ => {
            if stamp_at(root, &id).exists() {
                Presence::Mismatch
            } else {
                Presence::Absent
            }
        }
    };
    Ok(Ground {
        frozen,
        pin,
        stamp,
        pin_seq,
    })
}

// ---------------------------------------------------------------------------
// The log and the machine
// ---------------------------------------------------------------------------

#[derive(Default)]
struct LogState {
    req: Option<Promotion>,
    done: BTreeSet<Step>,
    uncertain: Option<String>,
}

fn fold(records: &[Value]) -> LogState {
    let mut st = LogState::default();
    for r in records {
        let Some(step) = r["step"].as_str().and_then(Step::from_name) else {
            st.uncertain = Some("unknown step in the log".into());
            continue;
        };
        if r["uncertain"].is_string() {
            st.uncertain = r["uncertain"].as_str().map(str::to_owned);
        }
        if step == Step::Begin {
            st.req = Promotion::from_value(&r["detail"]);
        }
        st.done.insert(step);
    }
    st
}

fn record(step: Step, id: &str, effect: Option<String>, detail: Value) -> Value {
    json!({"step": step.name(), "key": id, "effect": effect, "detail": detail})
}

fn lock_proposal(root: &Path, proposal: &str) -> Result<fs::File, SagaError> {
    fs::create_dir_all(root.join("locks")).map_err(|e| io_err("mkdir", e))?;
    let name = digest(proposal.as_bytes());
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("locks").join(format!("{name}.lock")))
        .map_err(|e| io_err("open the proposal lock", e))?;
    file.lock()
        .map_err(|e| io_err("take the proposal lock", e))?;
    Ok(file)
}

/// Judge the outcome from ground truth, read-only.
pub fn classify(root: &Path, key: &Key) -> Result<Outcome, SagaError> {
    let id = key.id();
    let (records, _) = read_lines(&log_at(root, &id))?;
    let st = fold(&records);
    if let Some(why) = &st.uncertain {
        return Ok(Outcome::Uncertain(why.clone()));
    }
    let Some(req) = &st.req else {
        // No Begin on disk: nothing of this saga may exist.
        let g = pin_history(root)?.iter().any(|l| l["key"] == id.as_str());
        return Ok(
            if g || frozen_at(root, &id).exists() || stamp_at(root, &id).exists() {
                Outcome::Uncertain("effects without a Begin record".into())
            } else {
                Outcome::PreSaga
            },
        );
    };
    let g = ground(root, req)?;
    let all = [g.frozen, g.pin, g.stamp];
    if all.contains(&Presence::Mismatch) {
        return Ok(Outcome::Uncertain(
            "an effect differs from the saga's bytes".into(),
        ));
    }
    if st.done.contains(&Step::Complete) {
        return Ok(
            if all.iter().all(|p| *p == Presence::Match) && !st.done.contains(&Step::Rollback) {
                Outcome::Promoted
            } else {
                Outcome::Uncertain("Complete is logged but the effects are not all present".into())
            },
        );
    }
    if st.done.contains(&Step::Rollback) {
        return Ok(if all.iter().all(|p| *p == Presence::Absent) {
            Outcome::PreSaga
        } else {
            Outcome::Uncertain("rolled back but effects remain".into())
        });
    }
    Ok(Outcome::InProgress)
}

/// A world may be used as promoted only when the saga is `Promoted`.
pub fn require_promoted(root: &Path, key: &Key) -> Result<(), SagaError> {
    match classify(root, key)? {
        Outcome::Promoted => Ok(()),
        Outcome::Uncertain(why) => Err(SagaError::Uncertain(why)),
        other => Err(refused("NOT_PROMOTED", format!("{other:?}"))),
    }
}

/// Run the promotion forward (W1: any number of runs is one run).
pub fn run(root: &Path, req: &Promotion, faults: Faults) -> Result<Outcome, SagaError> {
    drive(root, &req.key, Some(req), Mode::Forward, faults)
}

/// The reconciler: the same machine, started from the log.
pub fn reconcile(root: &Path, key: &Key, faults: Faults) -> Result<Outcome, SagaError> {
    drive(root, key, None, Mode::Recover, faults)
}

/// One transition: run `step` (effect, then its record). `Err` leaves whatever
/// a crash would leave.
fn exec(root: &Path, req: &Promotion, step: Step, faults: &Faults) -> Result<(), SagaError> {
    let id = req.key.id();
    let log = log_at(root, &id);
    faults.at(step, Phase::BeforeEffect)?;
    let mut effect = None;
    let mut detail = Value::Null;
    match step {
        Step::Begin => {
            detail = req.to_value();
            if faults.at(step, Phase::MidEffect).is_err() {
                fs::create_dir_all(log.parent().unwrap_or(Path::new(".")))
                    .map_err(|e| io_err("mkdir", e))?;
                fs::write(log.with_extension("tmp"), b"{\"half").map_err(|e| io_err("write", e))?;
                return faults.at(step, Phase::MidEffect);
            }
        }
        Step::Freeze => {
            let bytes = frozen_bytes(req)?;
            put_whole(&frozen_at(root, &id), &bytes, Some((faults, step)))?;
            effect = Some(digest(&bytes));
        }
        Step::Pin => {
            let _pins = lock_proposal(root, "\0pins")?;
            cut_torn(&pins_at(root))?;
            let hist = pin_history(root)?;
            let seq = match hist.iter().find(|l| l["key"] == id.as_str()) {
                Some(l) => {
                    let s = l["seq"].as_u64().unwrap_or(u64::MAX);
                    if *l != expected_pin_line(req, s) {
                        return Err(SagaError::Uncertain("the pin line differs".into()));
                    }
                    s
                }
                None => {
                    let now = current_pin(root)?;
                    if now != req.base_world_ref {
                        return Err(refused(
                            "PIN_STALE",
                            format!(
                                "promotion is against {}; the pin is {now}",
                                req.base_world_ref
                            ),
                        ));
                    }
                    let seq = hist.len() as u64;
                    let line = expected_pin_line(req, seq);
                    // The pin line is both effect and its own durable mark.
                    append_line(
                        &pins_at(root),
                        &line,
                        Some(faults),
                        (step, Phase::MidEffect),
                    )?;
                    seq
                }
            };
            effect = Some(digest(
                &serde_json::to_vec(&expected_pin_line(req, seq)).unwrap_or_default(),
            ));
        }
        Step::Stamp => {
            let seq = ground(root, req)?
                .pin_seq
                .ok_or_else(|| SagaError::Uncertain("stamp before the pin".into()))?;
            let bytes = stamp_bytes(req, seq)?;
            put_whole(&stamp_at(root, &id), &bytes, Some((faults, step)))?;
            effect = Some(digest(&bytes));
        }
        Step::Complete => {
            if faults.at(step, Phase::MidEffect).is_err() {
                fs::write(
                    root.join("promotions").join(&id).join("complete.tmp"),
                    b"half",
                )
                .map_err(|e| io_err("write", e))?;
                return faults.at(step, Phase::MidEffect);
            }
        }
        Step::Rollback => {
            faults.at(step, Phase::MidEffect).or_else(|e| {
                fs::write(
                    root.join("promotions").join(&id).join("rollback.tmp"),
                    b"half",
                )
                .map_err(|x| io_err("write", x))?;
                Err(e)
            })?;
            for at in [frozen_at(root, &id), stamp_at(root, &id)] {
                match fs::remove_file(&at) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(io_err("remove", e)),
                }
            }
        }
    }
    faults.at(step, Phase::AfterEffect)?;
    append_line(
        &log,
        &record(step, &id, effect, detail),
        Some(faults),
        (step, Phase::MidRecord),
    )?;
    faults.at(step, Phase::AfterRecord)
}

fn mark_uncertain(root: &Path, id: &str, why: &str) -> Result<(), SagaError> {
    let mut r = record(Step::Rollback, id, None, Value::Null);
    r["uncertain"] = json!(why);
    // The marker rides on a Rollback-named record so it is a log fact, not
    // an effect: classify reads `uncertain` first.
    append_line(
        &log_at(root, id),
        &r,
        None,
        (Step::Rollback, Phase::AfterRecord),
    )
}

/// The one machine. `req` is the caller's request (forward) or `None`
/// (recover: the request is the Begin record).
pub fn drive(
    root: &Path,
    key: &Key,
    req: Option<&Promotion>,
    mode: Mode,
    faults: Faults,
) -> Result<Outcome, SagaError> {
    if let Some(r) = req {
        // W5: refuse before any write.
        map_keys(&r.changed_keys, &r.mapping)?;
        if r.key != *key {
            return Err(refused("KEY_MISMATCH", "request and key differ"));
        }
    }
    let id = key.id();
    let _lock = lock_proposal(root, &key.proposal_id)?;
    clean_tmp(root, &id);
    {
        let _pins = lock_proposal(root, "\0pins")?;
        cut_torn(&pins_at(root))?;
    }
    cut_torn(&log_at(root, &id))?;
    let mut refusal: Option<SagaError> = None;
    loop {
        let (records, _) = read_lines(&log_at(root, &id))?;
        let st = fold(&records);
        if let Some(why) = &st.uncertain {
            return Ok(Outcome::Uncertain(why.clone()));
        }
        let logged = st.req.clone();
        let recover_unlogged = st.req.is_none() && mode == Mode::Recover;
        let Some(r) = logged.as_ref().or(req).filter(|_| !recover_unlogged) else {
            // No Begin: judge from the effects first. Any effect without a
            // log is Uncertain (fail closed); only an effect-free store is
            // PreSaga.
            return match classify(root, key)? {
                Outcome::Uncertain(why) => {
                    mark_uncertain(root, &id, &why)?;
                    Ok(Outcome::Uncertain(why))
                }
                _ => Ok(Outcome::PreSaga),
            };
        };
        match classify(root, key)? {
            Outcome::Promoted => return Ok(Outcome::Promoted),
            Outcome::PreSaga if st.req.is_some() || mode == Mode::Recover => {
                return refusal.map_or(Ok(Outcome::PreSaga), Err)
            }
            Outcome::Uncertain(why) => {
                mark_uncertain(root, &id, &why)?;
                return Ok(Outcome::Uncertain(why));
            }
            _ => {}
        }
        let g = if st.req.is_some() {
            Some(ground(root, r)?)
        } else {
            None
        };
        let pinned = g.as_ref().is_some_and(|g| g.pin == Presence::Match);
        let step = if !st.done.contains(&Step::Begin) {
            Step::Begin
        } else if refusal.is_some() || (mode == Mode::Recover && !pinned) {
            Step::Rollback
        } else if !st.done.contains(&Step::Freeze) {
            Step::Freeze
        } else if !st.done.contains(&Step::Pin) {
            Step::Pin
        } else if !st.done.contains(&Step::Stamp) {
            Step::Stamp
        } else {
            Step::Complete
        };
        match exec(root, r, step, &faults) {
            Ok(()) => {}
            Err(SagaError::Uncertain(why)) => {
                mark_uncertain(root, &id, &why)?;
                return Ok(Outcome::Uncertain(why));
            }
            Err(e @ SagaError::Refused { .. }) if step == Step::Pin => refusal = Some(e),
            Err(e) => return Err(e),
        }
    }
}

// ---------------------------------------------------------------------------
// Evidence, re-pin, case pins
// ---------------------------------------------------------------------------

fn evidence_at(root: &Path, name: &str) -> PathBuf {
    root.join("evidence").join(format!("{name}.json"))
}

/// Settle evidence under the world it was made in. The digest covers the
/// `world_ref`; the file is immutable (a different body is refused).
pub fn settle_evidence(
    root: &Path,
    name: &str,
    world_ref: &str,
    body: &Value,
) -> Result<(), SagaError> {
    let core = json!({"name": name, "world_ref": world_ref, "body": body});
    let d = digest_of(&core).ok_or_else(|| refused("EVIDENCE_INVALID", "not canonicalizable"))?;
    let bytes = serde_json::to_vec(
        &json!({"name": name, "world_ref": world_ref, "body": body, "digest": d}),
    )
    .map_err(|e| io_err("encode", e))?;
    // Check-and-write under one lock: a delayed writer sees the settled file.
    let _lock = lock_proposal(root, &format!("\0evidence\0{name}"))?;
    put_whole(&evidence_at(root, name), &bytes, None)
}

/// Read evidence and verify its digest; returns `(world_ref, digest)`.
pub fn read_evidence(root: &Path, name: &str) -> Result<(String, String), SagaError> {
    let v: Value =
        serde_json::from_slice(&fs::read(evidence_at(root, name)).map_err(|e| io_err("read", e))?)
            .map_err(|e| io_err("parse", e))?;
    let core = json!({"name": v["name"], "world_ref": v["world_ref"], "body": v["body"]});
    let d = digest_of(&core).unwrap_or_default();
    if v["digest"] != d.as_str() {
        return Err(SagaError::Uncertain("evidence digest mismatch".into()));
    }
    Ok((v["world_ref"].as_str().unwrap_or_default().into(), d))
}

/// Re-pin to an earlier world: a new pin line; nothing old is rewritten.
pub fn repin(root: &Path, to: &str, reason: &str) -> Result<(), SagaError> {
    let _pins = lock_proposal(root, "\0pins")?;
    cut_torn(&pins_at(root))?;
    let from = current_pin(root)?;
    let hist = pin_history(root)?;
    let id = format!(
        "repin-{}",
        digest(format!("{from}\0{to}\0{reason}").as_bytes())
    );
    if hist.iter().any(|l| l["key"] == id.as_str()) {
        return Ok(());
    }
    append_line(
        &pins_at(root),
        &json!({"key": id, "from": from, "to": to, "seq": hist.len()}),
        None,
        (Step::Pin, Phase::AfterRecord),
    )
}

/// Pin an opened case to the store's current world, once. A pin that exists
/// is kept: a promotion never moves an in-flight case. Written through the
/// case guard's `commit`.
pub fn pin_case(root: &Path, case_dir: &Path) -> Result<String, SagaError> {
    // Check, read the world and write under one per-case lock.
    let _lock = lock_proposal(root, &format!("\0case\0{}", case_dir.display()))?;
    let existing = case_pin(case_dir);
    if let Some(p) = existing {
        return Ok(p);
    }
    let world = current_pin(root)?;
    let bytes =
        serde_json::to_vec(&json!({"world_ref": world})).map_err(|e| io_err("encode", e))?;
    commit(
        case_dir,
        Batch::new().with(Op::RunFile {
            rel: "pin.json".into(),
            bytes,
        }),
    )
    .map_err(|e| refused(e.code, e.message))?;
    Ok(world)
}

/// The world a case is pinned to, if any.
pub fn case_pin(case_dir: &Path) -> Option<String> {
    let v: Value =
        serde_json::from_slice(&fs::read(case_dir.join("runs").join("pin.json")).ok()?).ok()?;
    v["world_ref"].as_str().map(str::to_owned)
}
