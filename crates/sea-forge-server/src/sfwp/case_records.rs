//! `case.record.append` (WP-14): the one Field Case write verb, kinds
//! `episode.frozen` and `episode.settled`, per `contracts/fde/README.md` §3.
//!
//! The store is synchronous and file-backed under `cases/<case_id>/`; the SFWP
//! verb runs it on a blocking thread under the case's keyed lock, so every
//! check below and the write that follows it are one critical section.
//!
//! Files (all append-only, nothing is deleted):
//! - `field-records.jsonl`: the hash-chained case records (the ledger of this
//!   verb). `seq` and `prev_hash` order and chain them.
//! - `field-freeze-intents.jsonl`: phase state of each freeze (`prepared`,
//!   `committed`, `abandoned`); the latest line per idempotency key wins.
//! - `field-rt-outbox.jsonl`: RealityTrace writes owed by a freeze, keyed
//!   `(system, record_id)`. Delivery is NOT implemented here (see below).
//! - `field-anchor-outbox.jsonl`: external anchors owed by a freeze
//!   (`pending` until a publisher succeeds).
//!
//! Freeze order (RD-12, E §5): (1) RealityTrace plan/question/threshold exist,
//! (2) the `episode.frozen` case record, (3) the external anchor. `prepare`
//! validates everything and writes the intent and outbox entries; `commit`
//! appends the record. A crash between them leaves a `prepared` intent: the
//! same request replayed completes it, [`recover`] completes or abandons it
//! (E §11 "Partial freeze"), and [`freeze_status`] reports it incomplete so
//! nothing can be delegated against it.
//!
//! Stage limits, stated rather than hidden: the RealityTrace records are
//! *checked* through [`RealityTraceIndex`] (the default reads
//! `<root>/realitytrace/records.jsonl`) and *owed* through the outbox; no
//! in-process RealityTrace writer or delivery agent exists in this crate yet.
//! The anchor is published only when an [`AnchorPublisher`] is supplied.

use sea_forge_case_runner::case_ops::case_guard::{self, Batch, CommitError, Op};
use sea_forge_case_runner::case_ops::field_guard::Refusal;
use sea_forge_core::errors::ForgeError;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Schemas vendored by digest from `contracts/fde/` (see `contracts/fde/DIGESTS`
/// in this crate); `tests/conformance_field_case.rs` fails when a copy differs.
const CASE_RECORD_APPEND: &str = include_str!("../../contracts/fde/case-record-append.v1.json");
const FROZEN_REFERENCE: &str = include_str!("../../contracts/fde/frozen-reference.v1.json");
const CASE_RECORD_APPEND_V2: &str = include_str!("../../contracts/fde/case-record-append.v2.json");
const FIELD_CASE: &str = include_str!("../../contracts/fde/field-case.v1.json");
const ANCHOR: &str = include_str!("../../contracts/fde/anchor.v1.json");

const ZERO_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
/// Largest `payload` (canonical JSON bytes) one `case.record.append` may carry.
/// The socket frame cap is 1 MiB per request line (`MAX_REQUEST_BYTES` in
/// `lib.rs`); a payload is stored in the record, the freeze intent and the
/// outbox, so it is held to a quarter of the frame.
pub const MAX_PAYLOAD_BYTES: usize = 256 * 1024;

pub(super) const RECORDS: &str = "field-records.jsonl";
const INTENTS: &str = "field-freeze-intents.jsonl";
const RT_OUTBOX: &str = "field-rt-outbox.jsonl";
const ANCHOR_OUTBOX: &str = "field-anchor-outbox.jsonl";

/// The vendored schema texts, for the contract gate test.
pub fn vendored_schemas() -> [(&'static str, &'static str); 5] {
    [
        ("anchor.v1.json", ANCHOR),
        ("case-record-append.v1.json", CASE_RECORD_APPEND),
        ("case-record-append.v2.json", CASE_RECORD_APPEND_V2),
        ("field-case.v1.json", FIELD_CASE),
        ("frozen-reference.v1.json", FROZEN_REFERENCE),
    ]
}

// ---------------------------------------------------------------------------
// RFC 8785 canonical JSON and digests
// ---------------------------------------------------------------------------

/// RFC 8785 canonical JSON for objects, arrays, strings, integers, booleans
/// and null. Keys sort by UTF-16 code units. A float is refused (`None`), not
/// hashed wrongly. (`sea-forge-core`'s `jcs-nfc-v1` is a different profile and
/// is not used for these digests.)
pub fn jcs(value: &Value) -> Option<String> {
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
// Schema registry
// ---------------------------------------------------------------------------

struct Registry(std::collections::HashMap<String, Value>);

impl jsonschema::Retrieve for Registry {
    fn retrieve(
        &self,
        uri: &jsonschema::Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.0
            .get(uri.as_str())
            .cloned()
            .ok_or_else(|| format!("unknown schema {uri}").into())
    }
}

fn request_validator() -> &'static jsonschema::Validator {
    static VALIDATOR: OnceLock<jsonschema::Validator> = OnceLock::new();
    VALIDATOR.get_or_init(|| {
        let docs: std::collections::HashMap<String, Value> =
            [CASE_RECORD_APPEND, FROZEN_REFERENCE, ANCHOR]
                .iter()
                .map(|text| {
                    let doc: Value = serde_json::from_str(text).expect("vendored schema is JSON");
                    (doc["$id"].as_str().expect("schema $id").to_string(), doc)
                })
                .collect();
        jsonschema::options()
            .with_retriever(Registry(docs))
            .build(&json!({"$ref": "urn:sea-forge:fde:case-record-append:v1#/$defs/Request"}))
            .expect("vendored schemas compile")
    })
}

// ---------------------------------------------------------------------------
// Refusals and the response shapes
// ---------------------------------------------------------------------------

pub(super) fn refuse(code: &str, message: &str, invariant: Option<&str>) -> Value {
    let mut out = json!({"outcome": "refused", "code": code, "message": message});
    if let Some(invariant) = invariant {
        out["invariant"] = json!(invariant);
    }
    out
}

pub(super) fn is_refusal(v: &Value) -> bool {
    v["outcome"] == "refused"
}

/// Where in a freeze a crash is injected (tests only reach this through
/// [`append_faulted`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// After the intent and outbox entries are durable, before the record.
    AfterPrepare,
    /// After the record is durable, before the intent is marked committed.
    AfterRecord,
}

/// The injected crash: the process "died" at the named boundary.
#[derive(Debug, PartialEq, Eq)]
pub struct CrashInjected(pub Fault);

#[derive(Debug)]
pub(super) enum AppendFailure {
    Injected(CrashInjected),
    Operational(ForgeError),
}

impl From<CrashInjected> for AppendFailure {
    fn from(error: CrashInjected) -> Self {
        Self::Injected(error)
    }
}

impl AppendFailure {
    fn into_error(self) -> ForgeError {
        match self {
            Self::Operational(error) => error,
            Self::Injected(fault) => ForgeError::io(
                "case record append interrupted",
                std::io::Error::other(format!("{fault:?}")),
            ),
        }
    }
}

pub(super) fn record_commit_reply(error: CommitError) -> Result<Value, AppendFailure> {
    match error {
        CommitError::Refused(refusal) => Ok(refuse(refusal.code, &refusal.message, None)),
        uncertain @ CommitError::PublicationUncertain(_) => {
            Err(AppendFailure::Operational(uncertain.into()))
        }
    }
}

/// Existence of RealityTrace records, resolved at their owner.
pub trait RealityTraceIndex {
    fn exists(&self, record_id: &str) -> bool;
    /// Whether the plan, a question and a threshold exist for the episode key,
    /// found by the key the owner records them under (an episode with RealityTrace
    /// records but no case record yet, README §7 R3). Default: unknown, so none.
    fn episode_complete(&self, _episode_key: &Value) -> bool {
        false
    }
}

/// Reads `<root>/realitytrace/records.jsonl` (`{"record_id": ...}` per line).
pub struct FileIndex(pub PathBuf);

impl RealityTraceIndex for FileIndex {
    fn exists(&self, record_id: &str) -> bool {
        read_lines(&self.0.join("realitytrace").join("records.jsonl"))
            .iter()
            .any(|line| line["record_id"] == record_id)
    }

    fn episode_complete(&self, key: &Value) -> bool {
        let lines = read_lines(&self.0.join("realitytrace").join("records.jsonl"));
        ["plan:", "question:", "threshold:"].iter().all(|prefix| {
            lines.iter().any(|l| {
                l["episode_key"] == *key
                    && l["record_id"]
                        .as_str()
                        .is_some_and(|id| id.starts_with(prefix))
            })
        })
    }
}

/// Publishes an external anchor; returns the verifier receipt on success.
pub trait AnchorPublisher {
    fn publish(&self, kind: &str, id: &str, digest: &str) -> Result<Value, String>;
}

/// Runs `scripts/fde-anchor.sh publish <kind> <id> <digest> --push` with a
/// bounded run time (the child is killed at the deadline). Only built when the
/// operator configures a script path: pushing a tag is an external,
/// irreversible effect.
///
/// The receipt is what the script actually prints (`tag_object_id <oid>`) plus
/// the tag name. The remote server's push time is NOT obtainable from a plain
/// `git push`, so the receipt carries the local clock at push return as
/// `local_push_time` with `time_source: "local_clock"`; the verifier reports
/// that as [`AnchorOrder::AnchorTimeLocalOnly`], never as externally verified.
pub struct ScriptPublisher(pub PathBuf, pub std::time::Duration);

/// Default bound on one publish.
pub const ANCHOR_PUBLISH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

impl AnchorPublisher for ScriptPublisher {
    fn publish(&self, kind: &str, id: &str, digest: &str) -> Result<Value, String> {
        use std::io::Read;
        use std::process::Stdio;
        let mut child = std::process::Command::new(&self.0)
            .args(["publish", kind, id, digest, "--push"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        let deadline = std::time::Instant::now() + self.1;
        let status = loop {
            match child.try_wait().map_err(|e| e.to_string())? {
                Some(status) => break status,
                None if std::time::Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("anchor publish timed out after {:?}", self.1));
                }
                None => std::thread::sleep(std::time::Duration::from_millis(25)),
            }
        };
        let mut stdout = String::new();
        let mut stderr = String::new();
        if let Some(mut o) = child.stdout.take() {
            let _ = o.read_to_string(&mut stdout);
        }
        if let Some(mut e) = child.stderr.take() {
            let _ = e.read_to_string(&mut stderr);
        }
        if !status.success() {
            return Err(stderr.trim().to_string());
        }
        let oid = stdout
            .lines()
            .find_map(|l| l.strip_prefix("tag_object_id "))
            .map(str::trim)
            .filter(|o| matches!(o.len(), 40 | 64) && o.bytes().all(|b| b.is_ascii_hexdigit()))
            .ok_or("the publish script printed no tag_object_id")?
            .to_string();
        Ok(json!({"script": self.0.display().to_string(),
                  "tag": format!("fde-anchor/{kind}/{id}"), "tag_object_id": oid,
                  "stdout": stdout.trim(), "local_push_time": now_rfc3339(),
                  "time_source": "local_clock"}))
    }
}

/// Stands in for the real publisher inside the case lock: the push itself runs
/// after the lock is released ([`anchor_work`], [`record_anchor_receipt`]), so
/// an inline attempt only leaves the outbox entry pending.
pub struct DeferredPublisher;

impl AnchorPublisher for DeferredPublisher {
    fn publish(&self, _: &str, _: &str, _: &str) -> Result<Value, String> {
        Err("anchor delivery is deferred until the case lock is released".into())
    }
}

/// The script publisher configured by `SEA_FORGE_FDE_ANCHOR_SCRIPT` (and
/// `SEA_FORGE_FDE_ANCHOR_TIMEOUT_SECS`), when set.
pub fn configured_publisher() -> Option<ScriptPublisher> {
    let script = std::env::var_os("SEA_FORGE_FDE_ANCHOR_SCRIPT")?;
    let timeout = std::env::var("SEA_FORGE_FDE_ANCHOR_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|s| (1..=600).contains(s))
        .map_or(ANCHOR_PUBLISH_TIMEOUT, std::time::Duration::from_secs);
    Some(ScriptPublisher(PathBuf::from(script), timeout))
}

/// What the server knows about the cell that the request cannot say (WP-21):
/// every field defaults to "not configured", under which the S1 behaviour is
/// unchanged. A configured value is enforced and fails closed.
#[derive(Clone, Copy, Debug)]
pub struct AppendPolicy<'a> {
    /// The world currently pinned in the WP-18 world pin record. `Some` makes
    /// a freeze naming another `world_ref` `WORLD_NOT_PINNED`.
    pub pinned_world: Option<&'a str>,
    /// The case-keeper service identity. `Some` means anchoring is configured:
    /// a freeze needs its anchor, the anchor must have been pushed by this
    /// identity, and the model anchor must precede the first run (NT-18, NT-48).
    pub case_keeper: Option<&'a str>,
    /// The identity the configured anchor credential publishes as. Checked at
    /// freeze against `case_keeper`, before the record is written.
    pub anchor_publisher: Option<&'a str>,
    /// The authenticated principal is a human (an operator-role session), not
    /// an agent or a service: its settlement needs a bound approval.
    pub principal_human: bool,
}

impl AppendPolicy<'static> {
    /// Nothing configured: the S1 behaviour.
    pub const NONE: AppendPolicy<'static> = AppendPolicy {
        pinned_world: None,
        case_keeper: None,
        anchor_publisher: None,
        principal_human: false,
    };
}

/// Everything one append needs besides the request.
pub struct AppendContext<'a> {
    pub root: &'a Path,
    pub case_id: &'a str,
    /// The authenticated principal of the request (the settler for `episode.settled`).
    pub principal: &'a str,
    /// The principal's SEA-Forge identity role (not a SOP role), for the
    /// interaction event's `actor.role`.
    pub role: &'a str,
    /// The principal's authenticated delegation chain, root first, ending at
    /// `principal`. At this stage these are SEA-Forge actor identities (RD-21).
    pub chain: &'a [String],
    pub rt: &'a dyn RealityTraceIndex,
    pub anchor: Option<&'a dyn AnchorPublisher>,
    pub policy: &'a AppendPolicy<'a>,
}

impl AppendContext<'_> {
    /// Whether the freeze must be anchored (a publisher or a case keeper is configured).
    fn anchor_configured(&self) -> bool {
        self.anchor.is_some() || self.policy.case_keeper.is_some()
    }
}

// ---------------------------------------------------------------------------
// File helpers
// ---------------------------------------------------------------------------

pub(super) fn case_dir(root: &Path, case_id: &str) -> PathBuf {
    root.join("cases").join(case_id)
}

/// Complete newline-terminated JSON lines; a torn trailing fragment is ignored
/// and an unparseable complete line is skipped. Only for readers that feed no
/// invariant (delivery, spool de-duplication); everything that decides an
/// invariant, an id or idempotency goes through [`integrity`] first.
pub(super) fn read_lines(path: &Path) -> Vec<Value> {
    let bytes = fs::read(path).unwrap_or_default();
    let end = match bytes.iter().rposition(|b| *b == b'\n') {
        Some(i) => i + 1,
        None => 0,
    };
    bytes[..end]
        .split(|b| *b == b'\n')
        .filter(|l| !l.is_empty())
        .filter_map(|l| serde_json::from_slice(l).ok())
        .collect()
}

/// As [`read_lines`] but an unreadable file or ANY unparseable complete line is
/// an error (a torn trailing fragment is still not a line). A missing file is empty.
pub(super) fn read_lines_strict(path: &Path) -> Result<Vec<Value>, String> {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("read {}: {e}", path.display())),
    };
    let end = bytes.iter().rposition(|b| *b == b'\n').map_or(0, |i| i + 1);
    let mut out = Vec::new();
    for (i, line) in bytes[..end].split(|b| *b == b'\n').enumerate() {
        if line.is_empty() {
            continue;
        }
        out.push(
            serde_json::from_slice(line)
                .map_err(|e| format!("{} line {} is not valid JSON: {e}", path.display(), i + 1))?,
        );
    }
    Ok(out)
}

pub(super) const RUNS: &str = "field-runs.jsonl";
pub(super) const LINKS: &str = "field-links.jsonl";

/// Every store file whose lines decide an invariant, an id or idempotency must
/// parse completely before anything is decided or written.
pub(super) fn integrity(ctx: &AppendContext) -> Result<(), String> {
    integrity_in(&case_dir(ctx.root, ctx.case_id))
}

pub(super) fn integrity_in(dir: &Path) -> Result<(), String> {
    // THE guard's strict load: every line of every store, the record chain and
    // payload schemas, the document (README section 3 STORE_INTEGRITY).
    sea_forge_case_runner::case_ops::case_guard::load_verified(dir, false)
        .map_err(|r| format!("store integrity: {}", r.message))?;
    // The hash chain and the digests of every stored record (README section 3
    // STORE_INTEGRITY), also for a case that is not a Field Case proper.
    sea_forge_case_runner::case_ops::case_guard::verified_records(dir)
        .map(drop)
        .map_err(|r| format!("store integrity: {}", r.message))
}

/// The one primitive every file of a case store is appended through. THE
/// guard (`case_guard::admit_write`) runs here, with the case lock held until
/// the line is durable: no writer reaches a case file without it. A refusal is
/// returned as `GUARD:<CODE>:<message>` (see [`io_refusal`]).
pub(super) fn append_line(path: &Path, value: &Value) -> Result<(), String> {
    match sea_forge_case_runner::case_ops::case_guard::commit_file(path, value)
        .map_err(|r| format!("GUARD:{}:{}", r.code, r.message))?
    {
        true => Ok(()),
        // Not a file of a case directory (a spool): appended plainly.
        false => {
            let mut line = serde_json::to_vec(value).map_err(|e| e.to_string())?;
            line.push(b'\n');
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .map_err(|e| format!("open {}: {e}", path.display()))?;
            file.write_all(&line)
                .and_then(|_| file.sync_data())
                .map_err(|e| format!("write {}: {e}", path.display()))
        }
    }
}

pub(super) fn records(ctx: &AppendContext) -> Vec<Value> {
    read_lines(&case_dir(ctx.root, ctx.case_id).join(RECORDS))
}

pub(super) const DECISIONS: &str = "field-decisions.jsonl";

pub(super) fn decisions(ctx: &AppendContext) -> Vec<Value> {
    read_lines(&case_dir(ctx.root, ctx.case_id).join(DECISIONS))
}

pub(super) fn now_rfc3339() -> String {
    sea_forge_core::timestamp::Timestamp::now_utc().to_rfc3339_millis_z()
}

/// Prepare the I-2 allow decision for the settlement batch. A decision
/// written by a prior attempt retains its identity on the retry.
fn settlement_decision(
    ctx: &AppendContext,
    key: &str,
    body_digest: &str,
    episode_key: &Value,
    executors: &BTreeSet<String>,
) -> (Value, bool) {
    let existing = decisions(ctx);
    if let Some(found) = existing.iter().find(|d| {
        d["idempotency_key"] == key && d["body_digest"] == body_digest && d["decision"] == "allow"
    }) {
        return (found.clone(), false);
    }
    let seq = existing.len() as u64 + 1;
    let record = json!({
        "decision_id": format!("dec-{seq:04}"), "seq": seq, "policy": "no_self_settlement",
        "invariant": "I-2", "decision": "allow", "idempotency_key": key,
        "body_digest": body_digest, "episode_key": episode_key, "principal": ctx.principal,
        "role": ctx.role, "chain": ctx.chain, "executors": executors,
        "recorded_at": now_rfc3339(),
    });
    (record, true)
}

/// A refused verb changes no bytes except its declared refusal record: the
/// deny decision and its interaction-outbox row, committed as ONE batch (the
/// contract, AT-05 and interaction map 3.2b, requires both).
fn record_refusal(
    ctx: &AppendContext,
    key: &str,
    body_digest: &str,
    episode_key: &Value,
    executors: &BTreeSet<String>,
) -> Result<(), String> {
    use sea_forge_case_runner::case_ops::case_guard::{commit, Batch, Op};
    let existing = decisions(ctx);
    if existing.iter().any(|d| {
        d["idempotency_key"] == key && d["body_digest"] == body_digest && d["decision"] == "deny"
    }) {
        return Ok(());
    }
    let seq = existing.len() as u64 + 1;
    let record = json!({
        "decision_id": format!("dec-{seq:04}"), "seq": seq, "policy": "no_self_settlement",
        "invariant": "I-2", "decision": "deny", "idempotency_key": key,
        "body_digest": body_digest, "episode_key": episode_key, "principal": ctx.principal,
        "role": ctx.role, "chain": ctx.chain, "executors": executors,
        "recorded_at": now_rfc3339(),
    });
    let mut batch = Batch::new().with(Op::Append {
        file: DECISIONS.into(),
        line: record.clone(),
    });
    if let Some(row) = super::interaction_events::decision_row(ctx, &record)? {
        batch.push(Op::Append {
            file: super::interaction_events::OUTBOX_FILE.into(),
            line: row,
        });
    }
    commit(&case_dir(ctx.root, ctx.case_id), batch)
        .map(|_| ())
        .map_err(|r| format!("GUARD:{}:{}", r.code, r.message))
}

/// Latest intent line per idempotency key.
pub(super) fn latest_intents(ctx: &AppendContext) -> Vec<Value> {
    latest_intents_in(&case_dir(ctx.root, ctx.case_id))
}

pub(super) fn latest_intents_in(dir: &Path) -> Vec<Value> {
    let mut latest: Vec<Value> = Vec::new();
    for line in read_lines(&dir.join(INTENTS)) {
        match latest
            .iter_mut()
            .find(|l| l["idempotency_key"] == line["idempotency_key"])
        {
            Some(slot) => *slot = line,
            None => latest.push(line),
        }
    }
    latest
}

fn intent_op(state: &str, base: &Value, record_id: Option<&str>) -> Op {
    let mut line = base.clone();
    line["state"] = json!(state);
    if let Some(id) = record_id {
        line["record_id"] = json!(id);
    }
    Op::Append {
        file: INTENTS.into(),
        line,
    }
}

fn put_intent(
    ctx: &AppendContext,
    state: &str,
    base: &Value,
    record_id: Option<&str>,
) -> Result<(), CommitError> {
    let batch = Batch::new().with(intent_op(state, base, record_id));
    case_guard::commit_with_outcome(&case_dir(ctx.root, ctx.case_id), batch).map(drop)
}

fn episode_key_of(payload_ref: &Value) -> Option<(String, String, u64)> {
    let body = &payload_ref["body"];
    Some((
        body["case_id"].as_str()?.to_string(),
        body["plan_item_id"].as_str()?.to_string(),
        body["n"].as_u64()?,
    ))
}

fn key_json(case_id: &str, item: &str, n: u64) -> Value {
    json!({"case_id": case_id, "plan_item_id": item, "n": n})
}

/// `episode:<case>:<item>:<n>@sha256:<digest>` -> (case, item, n, digest).
fn parse_episode_ref(r: &str) -> Option<(String, String, u64, String)> {
    let rest = r.strip_prefix("episode:")?;
    let (head, digest) = rest.split_once("@sha256:")?;
    let mut parts = head.split(':');
    let (case, item, n) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() {
        return None;
    }
    Some((case.into(), item.into(), n.parse().ok()?, digest.into()))
}

fn response(kind: &str, record_id: &str, key: &Value, replayed: bool) -> Value {
    json!({"outcome": "recorded", "kind": kind, "record_id": record_id,
           "episode_key": key, "replayed": replayed})
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Schema-level validation with the contract's code mapping (README §3, "SCHEMA_INVALID vs
/// RESULT_INVALID/KIND_UNKNOWN").
fn validate_request(request: &Value) -> Result<(), Value> {
    let errors: Vec<(String, String)> = request_validator()
        .iter_errors(request)
        .map(|e| (e.instance_path().to_string(), e.to_string()))
        .collect();
    if errors.is_empty() {
        return Ok(());
    }
    let message = errors
        .iter()
        .map(|(p, m)| format!("{}: {m}", if p.is_empty() { "/" } else { p }))
        .collect::<Vec<_>>()
        .join("; ");
    if errors.iter().any(|(p, _)| p == "/kind") {
        return Err(refuse("KIND_UNKNOWN", &message, None));
    }
    let result_only = errors
        .iter()
        .all(|(p, _)| p == "/payload/result" || p.starts_with("/payload/reliability"));
    if result_only {
        return Err(refuse("RESULT_INVALID", &message, None));
    }
    Err(refuse("SCHEMA_INVALID", &message, None))
}

/// The digest checks of a freeze payload (NT-17 shape: the three digests agree
/// with the recomputed body digest, and the key parts equal the body's).
fn check_freeze_digests(payload: &Value) -> Result<(String, String, u64, String), Value> {
    let mismatch = |why: &str| refuse("EPISODE_REF_DIGEST_MISMATCH", why, None);
    let (case, item, n, ref_digest) =
        parse_episode_ref(payload["episode_ref"].as_str().unwrap_or(""))
            .ok_or_else(|| mismatch("episode_ref does not parse"))?;
    let reference = &payload["reference"];
    if reference["episode_ref"] != payload["episode_ref"] {
        return Err(mismatch(
            "reference.episode_ref differs from payload.episode_ref",
        ));
    }
    let Some((bcase, bitem, bn)) = episode_key_of(reference) else {
        return Err(mismatch("reference.body has no episode key"));
    };
    if (&case, &item, n) != (&bcase, &bitem, bn) {
        return Err(mismatch("episode_ref key parts differ from reference.body"));
    }
    let Some(recomputed) = digest_of(&reference["body"]) else {
        return Err(refuse(
            "SCHEMA_INVALID",
            "reference.body is not canonicalisable (float or unsupported value)",
            None,
        ));
    };
    if payload["digest"].as_str() != Some(&ref_digest) || ref_digest != recomputed {
        return Err(mismatch(
            "payload.digest, episode_ref and the recomputed body digest disagree",
        ));
    }
    Ok((case, item, n, ref_digest))
}

// ---------------------------------------------------------------------------
// Append
// ---------------------------------------------------------------------------

/// The `contract` marker that selects the v2 response dialect (README §6). It
/// selects the dialect only: every rule applies to every writer.
pub const V2_MARKER: &str = "fde.case-record-append@2";
/// Largest encoded request (README §6, `PAYLOAD_TOO_LARGE`).
pub const MAX_REQUEST_BYTES: usize = 1_048_576;
/// The four kinds only v2 writers may name; part B of WP-21 implements them.
const V2_ONLY_KINDS: [&str; 4] = [
    "field_case.revised",
    "strategy.recorded",
    "gate.assessed",
    "disposition.recorded",
];
/// README §6 `REFERENCE_DERIVED_FROM_RESULT`: the closed result-field list.
const RESULT_FIELDS: [&str; 6] = [
    "result",
    "outcome",
    "observation",
    "settlement",
    "verdict",
    "residual",
];

/// A v2 answer plus the SFWP frame class a v1 writer receives for the two
/// codes that have no v1 refusal (`STORE_INTEGRITY`, `PAYLOAD_TOO_LARGE`).
pub struct Answer {
    pub value: Value,
    pub frame_class: Option<&'static str>,
}

/// Run `case.record.append` and return the contract's response or refusal.
pub fn append(ctx: &AppendContext, request: &Value) -> Value {
    append_answer(ctx, request).value
}

/// [`append`], with the SFWP frame class a v1 writer is owed (the verb turns it
/// into an error frame; direct callers see the `SCHEMA_INVALID` fallback).
pub fn append_answer(ctx: &AppendContext, request: &Value) -> Answer {
    try_append_answer(ctx, request).unwrap_or_else(|error| Answer {
        value: crate::forge_error_response(&error),
        frame_class: None,
    })
}

fn try_append_answer(ctx: &AppendContext, request: &Value) -> Result<Answer, ForgeError> {
    let v2 = request.get("contract").and_then(Value::as_str) == Some(V2_MARKER);
    let value = append_inner(ctx, request, None).map_err(AppendFailure::into_error)?;
    let frame_class = if v2 {
        None
    } else {
        match value["code"].as_str() {
            Some("STORE_INTEGRITY") => Some("store_integrity"),
            Some("PAYLOAD_TOO_LARGE") => Some("input_error"),
            _ => None,
        }
    };
    Ok(Answer {
        value: if v2 { value } else { downgrade(value) },
        frame_class,
    })
}

/// README §6 "v1 writers": each v2-only code replaced by the fallback that is
/// itself a refusal, the v2 code name prefixing the message.
fn downgrade(value: Value) -> Value {
    let Some(code) = value["code"].as_str() else {
        return value;
    };
    let (fallback, invariant) = match code {
        "WORLD_NOT_PINNED"
        | "DELEGATION_LINK_INVALID"
        | "REFERENCE_DERIVED_FROM_RESULT"
        | "AUTHORITY_DENIED"
        | "STORE_INTEGRITY"
        | "PAYLOAD_TOO_LARGE"
        | "CASE_SUSPENDED"
        | "CASE_CLOSED"
        | "VERIFIER_CONTEXT_MISMATCH" => ("SCHEMA_INVALID", None),
        "SETTLEMENT_EXISTS" => ("IDEMPOTENCY_CONFLICT", None),
        "KIND_IMMUTABLE" => ("FREEZE_KEY_EXISTS", Some("I-3")),
        "MODEL_ANCHOR_LATE" | "ANCHOR_PUBLISHER_INVALID" => ("FREEZE_INCOMPLETE", Some("I-1")),
        _ => return value,
    };
    let message = value["message"].as_str().unwrap_or("");
    refuse(fallback, &format!("{code}: {message}"), invariant)
}

/// A string quoting a RealityTrace result id (`observation:`, `evidence:`,
/// `settlement:`, `residual:` + the 26-character id).
fn quotes_result_id(text: &str) -> bool {
    ["observation:", "evidence:", "settlement:", "residual:"]
        .iter()
        .any(|prefix| {
            text.match_indices(prefix).any(|(at, _)| {
                let id: Vec<char> = text[at + prefix.len()..].chars().take(26).collect();
                id.len() == 26
                    && id
                        .iter()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            })
        })
}

fn strings_quote_result(value: &Value) -> bool {
    match value {
        Value::String(s) => quotes_result_id(s),
        Value::Array(items) => items.iter().any(strings_quote_result),
        Value::Object(map) => map.values().any(strings_quote_result),
        _ => false,
    }
}

/// Best-effort NT-17 detection (README §6): a frozen reference body carrying a
/// result field or quoting a RealityTrace result id. A paraphrase passes; the
/// real protection is ordering (freeze and anchor before the first run).
fn derived_from_result(payload: &Value) -> Option<String> {
    let body = payload["reference"]["body"].as_object()?;
    if let Some(field) = RESULT_FIELDS.iter().find(|f| body.contains_key(**f)) {
        return Some(format!("reference.body carries the result field {field:?}"));
    }
    strings_quote_result(&payload["reference"]["body"])
        .then(|| "reference.body quotes a RealityTrace result id".to_string())
}

/// The lifecycle of the case from its case-level dispositions (README §6): a
/// `close-*` is final; otherwise suspended when the latest one is `suspend`.
pub use sea_forge_case_runner::case_ops::case_guard::{lifecycle, Lifecycle};

/// As [`append`], with a crash injected at a freeze phase boundary. The crash
/// returns `Err` *after* the named boundary's writes are durable, exactly what
/// a process death there leaves on disk.
pub fn append_faulted(
    ctx: &AppendContext,
    request: &Value,
    fault: Option<Fault>,
) -> Result<Value, CrashInjected> {
    match append_inner(ctx, request, fault) {
        Ok(value) => Ok(value),
        Err(AppendFailure::Injected(fault)) => Err(fault),
        Err(AppendFailure::Operational(error)) => Ok(crate::forge_error_response(&error)),
    }
}

fn append_inner(
    ctx: &AppendContext,
    request: &Value,
    fault: Option<Fault>,
) -> Result<Value, AppendFailure> {
    // The marker selects the dialect only; an unknown marker is not a dialect.
    match request.get("contract") {
        None | Some(Value::Null) => {}
        Some(Value::String(m)) if m == V2_MARKER => {}
        Some(_) => {
            return Ok(refuse(
                "SCHEMA_INVALID",
                "contract is not a known case.record.append contract",
                None,
            ))
        }
    }
    let v2 = request.get("contract").is_some_and(|c| c == V2_MARKER);
    // 1. PAYLOAD_TOO_LARGE: nothing is read or written.
    if serde_json::to_vec(request).map_or(true, |b| b.len() > MAX_REQUEST_BYTES) {
        return Ok(refuse(
            "PAYLOAD_TOO_LARGE",
            &format!("request exceeds {MAX_REQUEST_BYTES} bytes"),
            None,
        ));
    }
    if serde_json::to_vec(&request["payload"]).map_or(true, |b| b.len() > MAX_PAYLOAD_BYTES) {
        return Ok(refuse(
            "SCHEMA_INVALID",
            &format!("payload exceeds {MAX_PAYLOAD_BYTES} bytes"),
            None,
        ));
    }
    let kind = request["kind"].as_str().unwrap_or("");
    // 2. The four v2-only kinds (WP-21 part B) have their own path: the same
    // precedence, a v2 payload schema and the Field Case document.
    if v2 && V2_ONLY_KINDS.contains(&kind) {
        return super::field_case_kinds::append(ctx, request);
    }
    // 3. SCHEMA_INVALID and its specific forms. The v1 schema is closed, so the
    // marker is not part of what it validates.
    if kind == "episode.frozen" {
        if let Some(why) = derived_from_result(&request["payload"]) {
            return Ok(refuse("REFERENCE_DERIVED_FROM_RESULT", &why, None));
        }
    }
    let mut stripped = request.clone();
    if let Some(map) = stripped.as_object_mut() {
        map.remove("contract");
    }
    if let Err(refusal) = validate_request(&stripped) {
        return Ok(refusal);
    }
    let payload = &request["payload"];
    let key = request["idempotency_key"].as_str().unwrap_or("");
    // The payload's episode_ref must name the addressed case.
    let Some((pcase, ..)) = parse_episode_ref(payload["episode_ref"].as_str().unwrap_or("")) else {
        return Ok(refuse(
            "SCHEMA_INVALID",
            "payload.episode_ref does not parse",
            None,
        ));
    };
    if pcase != ctx.case_id {
        return Ok(refuse(
            "SCHEMA_INVALID",
            "payload.episode_ref names another case",
            None,
        ));
    }
    // 4. STORE_INTEGRITY: any read the following need fails.
    if let Err(e) = integrity(ctx) {
        return Ok(store_refusal(e));
    }
    // README section 6: every v2 append on a Field Case, the episode kinds
    // included, rests on the document; missing or invalid is STORE_INTEGRITY
    // and nothing is written.
    // Either dialect: the marker selects the response dialect only.
    match super::field_case_kinds::is_field_case(ctx.root, ctx.case_id) {
        Ok(true) => {
            if let Err(e) = super::field_case_kinds::load_document(ctx, &records(ctx)) {
                return Ok(store_refusal(e));
            }
        }
        Ok(false) => {}
        Err(e) => return Ok(store_refusal(e)),
    }
    // Every write needs an authenticated principal (E §8: freeze is P1,
    // settle is P2). The server passes an empty principal when the request
    // carried no verified identity; the shared `operator_local` fallback is
    // never invented here.
    if canonical_actor(ctx.principal).is_err()
        || ctx.chain.iter().any(|c| canonical_actor(c).is_err())
    {
        return Ok(refuse(
            "SCHEMA_INVALID",
            "the request carries no verified principal (or chain member)",
            None,
        ));
    }
    let Some(body_digest) = digest_of(&json!({"kind": kind, "payload": payload})) else {
        return Ok(refuse(
            "SCHEMA_INVALID",
            "payload is not canonicalisable (float or unsupported value)",
            None,
        ));
    };

    // 5. Idempotency first: a replay must return the original, not trip I-3.
    let records = records(ctx);
    if let Some(done) = records.iter().find(|r| r["idempotency_key"] == key) {
        if done["body_digest"] == body_digest.as_str() {
            // A crash between the record and the committed mark leaves the
            // intent `prepared`; the replay completes that second phase.
            if done["kind"] == "episode.frozen" {
                let pending = latest_intents(ctx)
                    .into_iter()
                    .find(|i| i["idempotency_key"] == key && i["state"] == "prepared");
                if let Some(intent) = pending {
                    let rec_id = done["record_id"].as_str().unwrap_or("");
                    if let Err(e) = put_intent(ctx, "committed", &intent, Some(rec_id)) {
                        return record_commit_reply(e);
                    }
                }
            }
            let original = &done["response"];
            return Ok(response(
                original["kind"].as_str().unwrap_or(kind),
                original["record_id"].as_str().unwrap_or(""),
                &original["episode_key"],
                true,
            ));
        }
        return Ok(refuse(
            "IDEMPOTENCY_CONFLICT",
            "idempotency_key was used with a different body",
            None,
        ));
    }
    let intents = latest_intents(ctx);
    if let Some(prior) = intents.iter().find(|i| i["idempotency_key"] == key) {
        if prior["body_digest"] != body_digest.as_str() {
            return Ok(refuse(
                "IDEMPOTENCY_CONFLICT",
                "idempotency_key was used with a different body",
                None,
            ));
        }
    }
    // 6. CASE_CLOSED, CASE_SUSPENDED (settlement is the one kind a closed
    // case still accepts: late evidence, M3).
    match (lifecycle(&records), kind) {
        (Lifecycle::Closed, "episode.settled") => {}
        (Lifecycle::Closed, _) => {
            return Ok(refuse(
                "CASE_CLOSED",
                "the case has a close disposition; closed is final",
                None,
            ))
        }
        (Lifecycle::Suspended, _) => {
            return Ok(refuse("CASE_SUSPENDED", "the case is suspended", None))
        }
        (Lifecycle::Open, _) => {}
    }

    match kind {
        "episode.frozen" => freeze(ctx, payload, key, &body_digest, &records, &intents, fault),
        "episode.settled" => settle(ctx, payload, key, &body_digest, &records),
        _ => Ok(refuse(
            "KIND_UNKNOWN",
            "kind is not a registered kind",
            None,
        )),
    }
}

/// A failed integrity read: `STORE_INTEGRITY` (v1 writers get the `SCHEMA_INVALID`
/// fallback with the cause in the message, as before WP-21).
pub(super) fn store_refusal(error: String) -> Value {
    refuse(
        "STORE_INTEGRITY",
        &format!("case record store failed: {error}"),
        None,
    )
}

pub(super) fn next_record(
    ctx: &AppendContext,
    records: &[Value],
    record: Value,
) -> Result<Value, CommitError> {
    next_record_with(ctx, records, record, None)
}

/// Commit the record and its optional approval consumption together.
pub(super) fn next_record_with(
    ctx: &AppendContext,
    records: &[Value],
    record: Value,
    also: Option<(&str, Value)>,
) -> Result<Value, CommitError> {
    let mut batch = Batch::new();
    if let Some((file, line)) = also {
        batch.push(Op::Append {
            file: file.into(),
            line,
        });
    }
    let record = commit_record(ctx, records, record, batch, None)?;
    // WP-14: the outbox can reconstruct any delivery owed by the ledger.
    use super::interaction_events::{enqueue_record, keep_uncertainty};
    keep_uncertainty(enqueue_record(ctx, &record))?;
    Ok(record)
}

fn commit_record(
    ctx: &AppendContext,
    records: &[Value],
    mut record: Value,
    mut batch: Batch,
    after_record: Option<Op>,
) -> Result<Value, CommitError> {
    let seq = records.len() as u64 + 1;
    let prev = records
        .last()
        .and_then(|r| r["hash"].as_str())
        .unwrap_or(ZERO_HASH)
        .to_string();
    record["seq"] = json!(seq);
    record["record_id"] = json!(format!("rec-{seq:04}"));
    record["prev_hash"] = json!(prev);
    record["recorded_at"] = json!(now_rfc3339());
    let hash = digest_of(&record).ok_or_else(|| Refusal {
        code: "SCHEMA_INVALID",
        message: "case record store failed: record is not canonicalisable".into(),
    })?;
    record["hash"] = json!(hash);
    batch.push(Op::Record(record.clone()));
    if let Some(op) = after_record {
        batch.push(op);
    }
    case_guard::commit_with_outcome(&case_dir(ctx.root, ctx.case_id), batch)?;
    Ok(record)
}

pub(super) fn io_refusal(error: String) -> Value {
    // A refusal of THE guard keeps its contract code.
    if let Some(rest) = error.strip_prefix("GUARD:") {
        if let Some((code, message)) = rest.split_once(':') {
            return refuse(code, message, None);
        }
    }
    // No contract code exists for a storage failure; fail closed as a schema
    // refusal carrying the cause rather than reporting success.
    refuse(
        "SCHEMA_INVALID",
        &format!("case record store failed: {error}"),
        None,
    )
}

fn freeze(
    ctx: &AppendContext,
    payload: &Value,
    key: &str,
    body_digest: &str,
    records: &[Value],
    intents: &[Value],
    fault: Option<Fault>,
) -> Result<Value, AppendFailure> {
    let (case, item, n, digest) = match check_freeze_digests(payload) {
        Ok(parts) => parts,
        Err(refusal) => return Ok(refusal),
    };
    // WORLD_NOT_PINNED: the frozen world is the world pinned in the WP-18 pin
    // record (S1 finding F8), not a field of a case document.
    let world_ref = payload["reference"]["body"]["world_ref"]
        .as_str()
        .unwrap_or("");
    if let Some(pinned) = ctx.policy.pinned_world {
        if pinned != world_ref {
            return Ok(refuse(
                "WORLD_NOT_PINNED",
                "freeze names a world_ref that is not the pinned world",
                None,
            ));
        }
    }
    // ANCHOR_PUBLISHER_INVALID at freeze, before anything is written so the
    // idempotency key is not burned: the configured credential must publish
    // as the case-keeper identity (NT-48).
    if let (Some(keeper), Some(publisher)) = (ctx.policy.case_keeper, ctx.policy.anchor_publisher) {
        if canonical_actor(keeper) != canonical_actor(publisher) {
            return Ok(refuse(
                "ANCHOR_PUBLISHER_INVALID",
                "the configured anchor credential is not the case-keeper identity",
                None,
            ));
        }
    }
    // KIND_IMMUTABLE (I-3, NT-13) precedes FREEZE_KEY_EXISTS: the episode kind
    // of an existing freeze never changes (an exploratory episode relabelled
    // evaluated would count as a test).
    let kind_of_episode = payload["reference"]["body"]["kind"]
        .as_str()
        .unwrap_or("evaluated");
    let existing_kind = records
        .iter()
        .find(|r| r["kind"] == "episode.frozen" && r["episode_key"] == key_json(&case, &item, n))
        .map(|r| r["episode_kind"].clone())
        .or_else(|| {
            intents
                .iter()
                .find(|i| {
                    i["state"] == "prepared"
                        && i["idempotency_key"] != key
                        && i["episode_key"] == key_json(&case, &item, n)
                })
                .map(|i| i["episode_kind"].clone())
        });
    if existing_kind.is_some_and(|k| k != kind_of_episode) {
        return Ok(refuse(
            "KIND_IMMUTABLE",
            "the episode kind of an existing freeze cannot change",
            Some("I-3"),
        ));
    }
    // I-3: one freeze per (case, item, n), whatever the digest, under a
    // different idempotency key. A prepared (uncommitted, not abandoned)
    // intent under another key also holds the key.
    let held_by_record = records
        .iter()
        .any(|r| r["kind"] == "episode.frozen" && r["episode_key"] == key_json(&case, &item, n));
    let held_by_intent = intents.iter().any(|i| {
        i["state"] == "prepared"
            && i["idempotency_key"] != key
            && i["episode_key"] == key_json(&case, &item, n)
    });
    if held_by_record || held_by_intent {
        return Ok(refuse(
            "FREEZE_KEY_EXISTS",
            "a freeze already exists for this episode key; changed criteria allocate n+1",
            Some("I-3"),
        ));
    }
    // I-1: the RealityTrace plan, question and threshold must exist first.
    let refs = &payload["realitytrace_refs"];
    let mut wanted: Vec<String> = vec![refs["plan_id"].as_str().unwrap_or("").to_string()];
    for field in ["question_ids", "threshold_ids"] {
        wanted.extend(
            refs[field]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str().map(String::from)),
        );
    }
    if let Some(missing) = wanted.iter().find(|id| !ctx.rt.exists(id)) {
        return Ok(refuse(
            "FREEZE_INCOMPLETE",
            &format!("RealityTrace plan, question or threshold not found: {missing}"),
            Some("I-1"),
        ));
    }
    // MODEL_ANCHOR_LATE at freeze, only when the case already holds a run
    // event (README §6): the model anchor must precede it.
    if let Err(why) = model_anchor_check(ctx, world_ref) {
        return Ok(why);
    }
    let episode_key = key_json(&case, &item, n);

    // Phase 1: prepare. Intent and the owed outbox entries are durable before
    // the case record exists.
    let intent = json!({
        "idempotency_key": key, "body_digest": body_digest, "episode_key": episode_key,
        "episode_kind": kind_of_episode, "digest": digest, "payload": payload,
    });
    let already_prepared = intents.iter().any(|i| i["idempotency_key"] == key);
    if let Err(error) = prepare_freeze(ctx, &intent, &item, n, &digest, !already_prepared) {
        return record_commit_reply(error);
    }
    if fault == Some(Fault::AfterPrepare) {
        return Err(CrashInjected(Fault::AfterPrepare).into());
    }
    commit_freeze(ctx, &intent, records, fault)
}

/// Phase 2: append the case record, mark the intent committed, then try the anchor.
fn commit_freeze(
    ctx: &AppendContext,
    intent: &Value,
    records: &[Value],
    fault: Option<Fault>,
) -> Result<Value, AppendFailure> {
    let kind = "episode.frozen";
    let key = intent["idempotency_key"].as_str().unwrap_or("");
    let record = json!({
        "kind": kind, "idempotency_key": key, "body_digest": intent["body_digest"],
        "episode_key": intent["episode_key"], "episode_kind": intent["episode_kind"],
        "digest": intent["digest"], "principal": ctx.principal, "payload": intent["payload"],
    });
    // Idempotent: a record for this key may already exist if a crash hit after
    // the append; re-use it instead of appending a second one.
    let existing = records
        .iter()
        .find(|r| r["idempotency_key"] == key)
        .cloned();
    let stored = match existing {
        Some(r) => r,
        None => {
            let mut record = record;
            record["response"] = json!(null);
            let rec_id = format!("rec-{:04}", records.len() + 1);
            record["response"] = response(kind, &rec_id, &intent["episode_key"], false);
            match next_record(ctx, records, record) {
                Ok(r) => r,
                Err(e) => return record_commit_reply(e),
            }
        }
    };
    if fault == Some(Fault::AfterRecord) {
        return Err(CrashInjected(Fault::AfterRecord).into());
    }
    let rec_id = stored["record_id"].as_str().unwrap_or("").to_string();
    if let Err(e) = put_intent(ctx, "committed", intent, Some(&rec_id)) {
        return record_commit_reply(e);
    }
    if let Some(publisher) = ctx.anchor {
        let _ = deliver_anchors(ctx, publisher);
    }
    Ok(response(kind, &rec_id, &intent["episode_key"], false))
}

fn anchor_id(case: &str, item: &str, n: u64, digest: &str) -> String {
    let raw = format!("{case}-{item}-{n}-{}", &digest[..12]);
    let clean: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed: String = clean.chars().take(128).collect();
    if trimmed.starts_with(|c: char| c.is_ascii_alphanumeric()) {
        trimmed
    } else {
        format!("a{}", &trimmed[..trimmed.len().min(127)])
    }
}

fn prepare_freeze(
    ctx: &AppendContext,
    intent: &Value,
    item: &str,
    n: u64,
    digest: &str,
    new_intent: bool,
) -> Result<(), CommitError> {
    let dir = case_dir(ctx.root, ctx.case_id);
    let payload = &intent["payload"];
    let episode_ref = payload["episode_ref"].as_str().unwrap_or("");
    let refs = &payload["realitytrace_refs"];
    // The contract import carries the frozen reference body; plan, question and
    // threshold ids are the ones the caller already wrote (checked above).
    let mut entries = vec![json!({"system": "realitytrace",
        "record_id": format!("verification.contract:{episode_ref}"), "op": "verification.contract_imported",
        "body": payload["reference"]})];
    entries.push(json!({"system": "realitytrace", "record_id": refs["plan_id"], "op": "plan.created", "episode_ref": episode_ref}));
    for q in refs["question_ids"].as_array().into_iter().flatten() {
        entries.push(json!({"system": "realitytrace", "record_id": q, "op": "question.declared", "episode_ref": episode_ref}));
    }
    for t in refs["threshold_ids"].as_array().into_iter().flatten() {
        entries.push(json!({"system": "realitytrace", "record_id": t, "op": "threshold.declared", "episode_ref": episode_ref}));
    }
    let mut effects = Vec::new();
    if new_intent {
        effects.push(intent_op("prepared", intent, None));
    }
    let mut known: BTreeSet<String> = read_lines(&dir.join(RT_OUTBOX))
        .into_iter()
        .filter(|row| row["system"] == "realitytrace")
        .filter_map(|row| row["record_id"].as_str().map(str::to_owned))
        .collect();
    for mut entry in entries {
        // Dropped on (system, record_id): E §11 "Outbox duplicates".
        if !known.insert(entry["record_id"].as_str().unwrap_or_default().to_owned()) {
            continue;
        }
        entry["state"] = json!("pending");
        effects.push(Op::Append {
            file: RT_OUTBOX.into(),
            line: entry,
        });
    }
    let id = anchor_id(ctx.case_id, item, n, digest);
    let anchors = read_lines(&dir.join(ANCHOR_OUTBOX));
    if !anchors.iter().any(|a| a["id"] == id.as_str()) {
        effects.push(Op::Append {
            file: ANCHOR_OUTBOX.into(),
            line: json!({"state": "pending", "kind": "freeze", "id": id, "tag": format!("fde-anchor/freeze/{id}"),
                         "subject_ref": episode_ref, "digest": digest}),
        });
    }
    if effects.is_empty() {
        return Ok(());
    }
    let mut batch = Batch::new();
    for effect in effects {
        batch.push(effect);
    }
    case_guard::commit_with_outcome(&dir, batch).map(drop)
}

/// The repo's identity canonicalisation for actor strings
/// (`governed_execution_boundary::check_identity_string`: trimmed, empty and
/// placeholder spellings refused), compared case-insensitively. Case folding
/// can only merge two spellings, so it can only add refusals, never miss one.
/// (Unicode NFC is not applied: the crate has no normalisation dependency.)
pub(super) fn canonical_actor(raw: &str) -> Result<String, String> {
    crate::governed_execution_boundary::check_identity_string("actor", raw)
        .map(|t| t.to_lowercase())
}

pub(super) fn parse_time(v: &Value) -> Option<sea_forge_core::timestamp::Timestamp> {
    v.as_str()
        .and_then(|t| sea_forge_core::timestamp::Timestamp::parse_rfc3339(t).ok())
}

/// The earliest run event of the case: any case event with a timestamp and any
/// run record. `Err` when a line that must carry a timestamp cannot be read.
fn first_run_event(
    ctx: &AppendContext,
) -> Result<Option<sea_forge_core::timestamp::Timestamp>, String> {
    let dir = case_dir(ctx.root, ctx.case_id);
    let mut first = None;
    let mut consider = |t: sea_forge_core::timestamp::Timestamp| {
        first = Some(first.map_or(t, |f: sea_forge_core::timestamp::Timestamp| f.min(t)));
    };
    for event in read_lines_strict(&dir.join("case-events.jsonl"))? {
        if let Some(t) = parse_time(&event["timestamp"]) {
            consider(t);
        }
    }
    for run in read_lines_strict(&dir.join(RUNS))? {
        consider(parse_time(&run["started_at"]).ok_or("a run record has no usable started_at")?);
    }
    Ok(first)
}

/// The published anchor of `kind` whose subject (model) or digest (freeze) matches.
fn published_anchor(ctx: &AppendContext, kind: &str, subject_or_digest: &str) -> Option<Value> {
    read_lines(&case_dir(ctx.root, ctx.case_id).join(ANCHOR_OUTBOX))
        .into_iter()
        .find(|a| {
            a["state"] == "published"
                && a["kind"] == kind
                && (a["subject_ref"] == subject_or_digest || a["digest"] == subject_or_digest)
        })
}

/// NT-18: the `model` anchor of the episode's world must exist and its push
/// time must precede the first run event. Only when anchoring is configured
/// (a case keeper is named). An absent anchor is late by definition once a run exists. With no run event at all
/// nothing can have preceded the anchor, so the check passes (the fixtures
/// `anchor-publisher-invalid` and the freeze scenarios rely on this reading;
/// reported as a contract ambiguity in the WP-21a evidence).
fn model_anchor_check(ctx: &AppendContext, world_ref: &str) -> Result<(), Value> {
    if ctx.policy.case_keeper.is_none() {
        return Ok(());
    }
    let first = first_run_event(ctx).map_err(store_refusal)?;
    if first.is_none() {
        return Ok(());
    }
    let late = |why: &str| Err(refuse("MODEL_ANCHOR_LATE", why, None));
    let Some(anchor) = published_anchor(ctx, "model", world_ref) else {
        return late("the model anchor for the episode's world is absent");
    };
    let receipt = &anchor["receipt"];
    let Some(pushed) = parse_time(&receipt["server_push_time"])
        .or_else(|| parse_time(&receipt["local_push_time"]))
    else {
        return late("the model anchor has no usable push time");
    };
    match first {
        Some(run) if pushed >= run => late("model anchored after the first run event"),
        _ => Ok(()),
    }
}

/// NT-48 at settlement: the pushed freeze tag was published by the case keeper.
fn freeze_anchor_publisher(ctx: &AppendContext, digest: &str) -> Result<(), Value> {
    let Some(keeper) = ctx.policy.case_keeper else {
        return Ok(());
    };
    let invalid = |why: &str| Err(refuse("ANCHOR_PUBLISHER_INVALID", why, None));
    let Some(anchor) = published_anchor(ctx, "freeze", digest) else {
        return Ok(()); // an absent anchor is FREEZE_INCOMPLETE, decided earlier
    };
    match anchor["receipt"]["pusher_identity"]
        .as_str()
        .map(canonical_actor)
    {
        Some(Ok(pusher)) if canonical_actor(keeper).is_ok_and(|k| k == pusher) => Ok(()),
        Some(_) => invalid("the freeze anchor was not published by the case keeper"),
        None => invalid("the freeze anchor receipt names no publisher"),
    }
}

pub(super) const APPROVALS: &str = "field-approvals.jsonl";

/// README §6 "Approvals": an `ApprovalRecord` bound to this record's canonical
/// operation, of this case, `allow`, unexpired and not yet used. Returns its
/// `approval_ref`. Fail closed: no store, no record, any mismatch is `None`.
fn bound_approval(
    ctx: &AppendContext,
    operation: &str,
    params: Value,
    subject_ref: &str,
) -> Option<String> {
    bound_approval_named(ctx, operation, params, subject_ref, None, None)
}

/// [`bound_approval`], restricted to the approval the request names when
/// `named` is given (an `approval_ref` in a payload is not free text).
pub(super) fn bound_approval_named(
    ctx: &AppendContext,
    operation: &str,
    params: Value,
    subject_ref: &str,
    named: Option<&str>,
    resume: Option<&str>,
) -> Option<String> {
    let digest = digest_of(&json!({
        "operation": operation, "subject_ref": subject_ref,
        "case_id": ctx.case_id, "params": params,
    }))?;
    let lines = read_lines(&case_dir(ctx.root, ctx.case_id).join(APPROVALS));
    let used: BTreeSet<String> = lines
        .iter()
        // A consumption bound to the record that is being retried does not
        // count: its write never landed (`resume` is that record's digest).
        .filter(|l| resume.is_none() || l["record_digest"].as_str() != resume)
        .filter_map(|l| l["consumed_by"].as_str().and(l["approval_ref"].as_str()))
        .map(String::from)
        .collect();
    let now = sea_forge_core::timestamp::Timestamp::now_utc();
    lines.iter().find_map(|a| {
        let reference = a["approval_ref"].as_str()?;
        let live = named.is_none_or(|n| n == reference)
            && a["operation"] == operation
            && a["operation_digest"] == digest.as_str()
            && a["case_id"] == ctx.case_id
            && a["outcome"] == "allow"
            && a["used_by"].is_null()
            && !used.contains(reference)
            && parse_time(&a["expires_at"]).is_some_and(|t| t > now);
        live.then(|| reference.to_string())
    })
}

fn settle(
    ctx: &AppendContext,
    payload: &Value,
    key: &str,
    body_digest: &str,
    records: &[Value],
) -> Result<Value, AppendFailure> {
    let Some((case, item, n, _)) = parse_episode_ref(payload["episode_ref"].as_str().unwrap_or(""))
    else {
        return Ok(refuse(
            "SCHEMA_INVALID",
            "payload.episode_ref does not parse",
            None,
        ));
    };
    let episode_key = key_json(&case, &item, n);
    // I-1: a settlement needs a COMPLETE freeze (both phases, the realitytrace
    // records, and the anchor when anchoring is configured), not merely a record.
    match freeze_state(ctx, &item, n, ctx.anchor_configured()) {
        FreezeStatus::Complete => {}
        FreezeStatus::Missing => {
            return Ok(refuse(
                "FREEZE_MISSING",
                "no freeze record for the episode",
                Some("I-1"),
            ))
        }
        FreezeStatus::Incomplete(parts) => {
            return Ok(refuse(
                "FREEZE_INCOMPLETE",
                &format!("freeze incomplete: missing {}", parts.join(", ")),
                Some("I-1"),
            ))
        }
    }
    let Some(freeze) = records
        .iter()
        .find(|r| r["kind"] == "episode.frozen" && r["episode_key"] == episode_key)
    else {
        return Ok(refuse(
            "FREEZE_MISSING",
            "no freeze record for the episode",
            Some("I-1"),
        ));
    };
    // The settlement must cite the frozen digest, not another one.
    if let (Some(frozen), Some((_, _, _, cited))) = (
        freeze["digest"].as_str(),
        parse_episode_ref(payload["episode_ref"].as_str().unwrap_or("")),
    ) {
        if frozen != cited {
            return Ok(refuse(
                "EPISODE_REF_DIGEST_MISMATCH",
                "episode_ref digest differs from the frozen episode",
                None,
            ));
        }
    }
    // NT-18 and NT-48 at settlement (anchoring configured).
    let world_ref = freeze["payload"]["reference"]["body"]["world_ref"]
        .as_str()
        .unwrap_or("");
    if let Err(refusal) = model_anchor_check(ctx, world_ref) {
        return Ok(refusal);
    }
    if let Err(refusal) = freeze_anchor_publisher(ctx, freeze["digest"].as_str().unwrap_or("")) {
        return Ok(refusal);
    }
    // One settlement per episode key. The same idempotency key replays before
    // this point; a different key for a settled episode is SETTLEMENT_EXISTS
    // (a v1 writer receives IDEMPOTENCY_CONFLICT with that prefix).
    if records
        .iter()
        .any(|r| r["kind"] == "episode.settled" && r["episode_key"] == episode_key)
    {
        return Ok(refuse(
            "SETTLEMENT_EXISTS",
            "the episode already has a settlement",
            None,
        ));
    }
    // Exploratory episodes settle only U or C.
    if freeze["episode_kind"] == "exploratory"
        && matches!(payload["result"].as_str(), Some("S") | Some("F"))
    {
        return Ok(refuse(
            "EXPLORATORY_NOT_SETTLEABLE",
            "an exploratory episode cannot settle S or F",
            None,
        ));
    }
    // I-2: the settling principal's chain must not contain an executor of the
    // item: the actors of its case events (S1) plus the executor set of the run
    // records and delegation links of the episode (README §8). A malformed
    // record fails closed (DELEGATION_LINK_INVALID).
    let mut executors = match executors_of(ctx, &item) {
        Ok(e) => e,
        Err(why) => return Ok(io_refusal(why)),
    };
    let episode_ref = payload["episode_ref"].as_str().unwrap_or("");
    let analysis =
        match super::run_records::analyse(ctx.root, ctx.case_id, episode_ref, ctx.principal) {
            Ok(Ok(a)) => a,
            Ok(Err(why)) => return Ok(refuse("DELEGATION_LINK_INVALID", &why, Some("I-2"))),
            Err(why) => return Ok(store_refusal(why)),
        };
    executors.extend(analysis.executors.iter().cloned());
    let contained = ctx
        .chain
        .iter()
        .chain(std::iter::once(&ctx.principal.to_string()))
        .filter_map(|c| canonical_actor(c).ok())
        .any(|c| executors.contains(&c));
    if contained {
        if let Err(e) = record_refusal(ctx, key, body_digest, &episode_key, &executors) {
            return Ok(io_refusal(e));
        }
        return Ok(refuse(
            "SELF_SETTLEMENT_REFUSED",
            "settling principal's chain contains the executor's identity",
            Some("I-2"),
        ));
    }
    // The verifier context is derived from run records, never asserted (H2).
    // Only an episode that has run records can derive one.
    if analysis.runs_exist {
        let declared = payload["independence"]["verifier_context"]
            .as_str()
            .unwrap_or("");
        let confirmed = matches!(payload["result"].as_str(), Some("S") | Some("F"));
        if declared != analysis.derived_context
            || (analysis.sibling && confirmed && analysis.derived_context != "fresh")
        {
            return Ok(refuse(
                "VERIFIER_CONTEXT_MISMATCH",
                "the declared verifier context differs from the one derived from run records, or a sibling of an executor settles without a fresh context",
                None,
            ));
        }
    }
    // A human settlement needs an approval bound to this very record (README
    // §6 "Approvals"; v1 writers are subject to it too).
    let mut approval = None;
    if ctx.policy.principal_human {
        let episode_ref = payload["episode_ref"].as_str().unwrap_or("");
        let params = json!({
            "case_id": ctx.case_id, "episode_ref": episode_ref,
            "result": payload["result"], "settlement_ref": payload["settlement_ref"],
            "record_digest": digest_of(payload).unwrap_or_default(),
        });
        approval = bound_approval(ctx, "settle_episode_human", params, episode_ref);
        if approval.is_none() {
            return Ok(refuse(
                "AUTHORITY_DENIED",
                "a human settlement needs an approval bound to this record",
                None,
            ));
        }
    }
    let (decision, new_decision) =
        settlement_decision(ctx, key, body_digest, &episode_key, &executors);
    let rec_id = format!("rec-{:04}", records.len() + 1);
    let mut batch = Batch::new();
    let consumption = approval.map(|reference| Op::Append {
        file: APPROVALS.into(),
        line: json!({"approval_ref": reference, "consumed_by": rec_id}),
    });
    if new_decision {
        batch.push(Op::Append {
            file: DECISIONS.into(),
            line: decision.clone(),
        });
    }
    let record = json!({
        "kind": "episode.settled", "idempotency_key": key, "body_digest": body_digest,
        "episode_key": episode_key, "principal": ctx.principal, "chain": ctx.chain, "payload": payload,
        "decision_id": decision["decision_id"],
        "response": response("episode.settled", &rec_id, &episode_key, false),
    });
    match commit_record(ctx, records, record, batch, consumption) {
        Ok(stored) => {
            use super::interaction_events::{enqueue_decision, enqueue_record, keep_uncertainty};
            if let Err(error) = keep_uncertainty(enqueue_decision(ctx, &decision))
                .and_then(|_| keep_uncertainty(enqueue_record(ctx, &stored)))
            {
                return record_commit_reply(error);
            }
            Ok(response("episode.settled", &rec_id, &episode_key, false))
        }
        Err(e) => record_commit_reply(e),
    }
}

/// Canonical execution identities for a plan item: the actor of every case
/// event that names it. Fails closed: an unreadable or missing events file, an
/// unparseable event line, or an event without a usable `actor_id` (or with a
/// non-string `plan_item_id`) is an error, never an empty executor set. An
/// event with `plan_item_id: null` is a case-level event (`TraceEvent`'s
/// `plan_item_id` is optional) and names no item.
fn executors_of(ctx: &AppendContext, plan_item_id: &str) -> Result<BTreeSet<String>, String> {
    let path = case_dir(ctx.root, ctx.case_id).join("case-events.jsonl");
    if !path.exists() {
        return Err(format!(
            "executor identities cannot be established: {} is missing",
            path.display()
        ));
    }
    let wanted = plan_item_id.trim().to_lowercase();
    let mut out = BTreeSet::new();
    for event in read_lines_strict(&path)? {
        let actor = event["actor_id"]
            .as_str()
            .ok_or_else(|| format!("case event {} has no actor_id", event["event_id"]))
            .and_then(|a| {
                canonical_actor(a)
                    .map_err(|_| "case event actor_id is empty or a placeholder".to_string())
            })?;
        match &event["plan_item_id"] {
            Value::Null => {}
            Value::String(item) => {
                if item.trim().to_lowercase() == wanted {
                    out.insert(actor);
                }
            }
            other => return Err(format!("case event plan_item_id is not a string: {other}")),
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Freeze status, recovery, anchors
// ---------------------------------------------------------------------------

/// Whether an episode's freeze is complete enough to delegate against (I-1).
#[derive(Debug, PartialEq, Eq)]
pub enum FreezeStatus {
    /// No record and no intent.
    Missing,
    /// Parts present but not all: `case_record`, `intent_committed`, `realitytrace_records`, `anchor`.
    Incomplete(Vec<&'static str>),
    Complete,
}

pub fn freeze_status(ctx: &AppendContext, plan_item_id: &str, n: u64) -> FreezeStatus {
    freeze_state(ctx, plan_item_id, n, true)
}

/// [`freeze_status`] with the anchor requirement chosen by the caller:
/// settlement requires the anchor only when a publisher is configured.
pub(super) fn freeze_state(
    ctx: &AppendContext,
    plan_item_id: &str,
    n: u64,
    require_anchor: bool,
) -> FreezeStatus {
    freeze_state_in(
        ctx,
        &case_dir(ctx.root, ctx.case_id),
        plan_item_id,
        n,
        require_anchor,
    )
}

pub(super) fn freeze_state_in(
    ctx: &AppendContext,
    dir: &Path,
    plan_item_id: &str,
    n: u64,
    require_anchor: bool,
) -> FreezeStatus {
    if integrity_in(dir).is_err() {
        return FreezeStatus::Incomplete(vec!["store_integrity"]);
    }
    let key = key_json(ctx.case_id, plan_item_id, n);
    let record = read_lines(&dir.join(RECORDS))
        .into_iter()
        .find(|r| r["kind"] == "episode.frozen" && r["episode_key"] == key);
    let intent = latest_intents_in(dir)
        .into_iter()
        .find(|i| i["episode_key"] == key && i["state"] != "abandoned");
    let Some(source) = record.as_ref().or(intent.as_ref()) else {
        return FreezeStatus::Missing;
    };
    let mut missing = Vec::new();
    match &record {
        None => missing.push("case_record"),
        Some(r) => {
            // Both phases: the intent for the record's key must be committed.
            let committed = latest_intents_in(dir)
                .iter()
                .any(|i| i["idempotency_key"] == r["idempotency_key"] && i["state"] == "committed");
            if !committed {
                missing.push("intent_committed");
            }
        }
    }
    let refs = &source["payload"]["realitytrace_refs"];
    let ids: Vec<&str> = std::iter::once(&refs["plan_id"])
        .chain(refs["question_ids"].as_array().into_iter().flatten())
        .chain(refs["threshold_ids"].as_array().into_iter().flatten())
        .filter_map(|v| v.as_str())
        .collect();
    if ids.is_empty() || ids.iter().any(|id| !ctx.rt.exists(id)) {
        missing.push("realitytrace_records");
    }
    let digest = source["digest"].as_str().unwrap_or("");
    let anchored = read_lines(&dir.join(ANCHOR_OUTBOX))
        .iter()
        .any(|a| a["state"] == "published" && a["digest"] == digest);
    if require_anchor && !anchored {
        missing.push("anchor");
    }
    if missing.is_empty() {
        FreezeStatus::Complete
    } else {
        FreezeStatus::Incomplete(missing)
    }
}

/// The I-1 gate a delegation or governed operation consults before moving an
/// evaluated episode: `Ok` only for a complete freeze. WP-20 wires it to the
/// delegation payload; no delegation verb calls it yet.
pub fn require_complete_freeze(
    ctx: &AppendContext,
    plan_item_id: &str,
    n: u64,
) -> Result<(), Value> {
    match freeze_status(ctx, plan_item_id, n) {
        FreezeStatus::Complete => Ok(()),
        FreezeStatus::Missing => Err(refuse(
            "FREEZE_MISSING",
            "no freeze for the episode",
            Some("I-1"),
        )),
        FreezeStatus::Incomplete(parts) => Err(refuse(
            "FREEZE_INCOMPLETE",
            &format!("freeze incomplete: missing {}", parts.join(", ")),
            Some("I-1"),
        )),
    }
}

/// What a passing order check established about the anchor's time.
#[derive(Debug, PartialEq, Eq)]
pub enum AnchorOrder {
    /// The receipt carries the remote server's own push time.
    ExternallyVerified,
    /// The receipt's push time is the local clock at push return
    /// (`anchor_time_local_only`): the tag exists, its time is not independent.
    AnchorTimeLocalOnly,
}

/// NT-16 / AT-04 order check from the ledger side: the freeze's anchor must be
/// published with a receipt, and its push time must be earlier than the first
/// run record (the first case event naming the plan item). Every failure is a
/// distinct `state` under `ANCHOR_PREDATES_FAILED`: `store_integrity`,
/// `no_freeze_record`, `anchor_absent`, `receipt_missing`, `malformed_run_event`,
/// `no_run_events`, `push_not_before_first_run`. Nothing is `Ok` by default.
pub fn verify_freeze_anchor(
    ctx: &AppendContext,
    plan_item_id: &str,
    n: u64,
) -> Result<AnchorOrder, Value> {
    let fail = |state: &str, why: &str| Err(refuse_anchor(state, why));
    if let Err(e) = integrity(ctx) {
        return fail("store_integrity", &e);
    }
    let Some(frozen) = records(ctx).into_iter().find(|r| {
        r["kind"] == "episode.frozen" && r["episode_key"] == key_json(ctx.case_id, plan_item_id, n)
    }) else {
        return fail("no_freeze_record", "no freeze record");
    };
    let digest = frozen["digest"].as_str().unwrap_or("");
    let anchor = read_lines(&case_dir(ctx.root, ctx.case_id).join(ANCHOR_OUTBOX))
        .into_iter()
        .find(|a| a["state"] == "published" && a["digest"] == digest);
    let Some(anchor) = anchor else {
        return fail("anchor_absent", "anchor tag absent");
    };
    let parse = |v: &Value| {
        v.as_str()
            .and_then(|t| sea_forge_core::timestamp::Timestamp::parse_rfc3339(t).ok())
    };
    let receipt = &anchor["receipt"];
    let (pushed, order) = if let Some(t) = parse(&receipt["server_push_time"]) {
        (t, AnchorOrder::ExternallyVerified)
    } else if receipt["time_source"] == "local_clock" {
        match parse(&receipt["local_push_time"]) {
            Some(t) => (t, AnchorOrder::AnchorTimeLocalOnly),
            None => return fail("receipt_missing", "receipt has no usable push time"),
        }
    } else {
        return fail("receipt_missing", "receipt has no push time");
    };
    let events = match read_lines_strict(&case_dir(ctx.root, ctx.case_id).join("case-events.jsonl"))
    {
        Ok(e) => e,
        Err(e) => return fail("malformed_run_event", &e),
    };
    let mut first_run: Option<sea_forge_core::timestamp::Timestamp> = None;
    for event in &events {
        match &event["plan_item_id"] {
            Value::Null => {}
            Value::String(item) if item != plan_item_id => {}
            Value::String(_) => match parse(&event["timestamp"]) {
                Some(t) => first_run = Some(first_run.map_or(t, |f| f.min(t))),
                None => {
                    return fail(
                        "malformed_run_event",
                        "a run event for the item has no usable timestamp",
                    )
                }
            },
            _ => {
                return fail(
                    "malformed_run_event",
                    "a run event's plan_item_id is not a string",
                )
            }
        }
    }
    match first_run {
        None => fail("no_run_events", "no run event names the plan item"),
        Some(run) if pushed >= run => fail(
            "push_not_before_first_run",
            "anchor push time is not before the first run record",
        ),
        Some(_) => Ok(order),
    }
}

fn refuse_anchor(state: &str, why: &str) -> Value {
    // anchor.v1 error table, not the case.record.append enum.
    json!({"outcome": "refused", "code": "ANCHOR_PREDATES_FAILED", "state": state, "message": why})
}

/// Outcome of [`recover`] for one case.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Recovered {
    pub completed: Vec<String>,
    pub abandoned: Vec<String>,
}

/// E §11 "Partial freeze": complete every `prepared` intent whose RealityTrace
/// records exist and whose digest still matches, otherwise mark it
/// `abandoned` (nothing is deleted; the next freeze allocates `n+1`).
/// Idempotent. Then retries pending anchors when a publisher is supplied.
///
/// The caller MUST hold the case's lock (a recovery interleaved with an append
/// would race on `seq` and the outbox files); the server entry point that
/// does so is `recover_field_case` in `lib.rs`.
pub fn recover(ctx: &AppendContext) -> Result<Recovered, ForgeError> {
    let mut out = Recovered::default();
    if integrity(ctx).is_err() {
        return Ok(out);
    }
    for intent in latest_intents(ctx) {
        if intent["state"] != "prepared" {
            continue;
        }
        let key = intent["idempotency_key"].as_str().unwrap_or("").to_string();
        let payload = &intent["payload"];
        let checked = check_freeze_digests(payload)
            .ok()
            .filter(|(_, _, _, digest)| intent["digest"] == digest.as_str());
        let refs = &payload["realitytrace_refs"];
        let rt_ok = std::iter::once(&refs["plan_id"])
            .chain(refs["question_ids"].as_array().into_iter().flatten())
            .chain(refs["threshold_ids"].as_array().into_iter().flatten())
            .all(|v| v.as_str().is_some_and(|id| ctx.rt.exists(id)));
        if let Some((_, item, n, digest)) = checked.filter(|_| rt_ok) {
            prepare_freeze(ctx, &intent, &item, n, &digest, false)?;
            let records = records(ctx);
            if !is_refusal(
                &commit_freeze(ctx, &intent, &records, None).map_err(AppendFailure::into_error)?,
            ) {
                out.completed.push(key);
                continue;
            }
        }
        put_intent(ctx, "abandoned", &intent, None)?;
        out.abandoned.push(key);
    }
    if let Some(publisher) = ctx.anchor {
        let _ = deliver_anchors(ctx, publisher);
    }
    super::interaction_events::reconcile_events(ctx)?;
    Ok(out)
}

/// The pending anchors whose freeze record is committed and that are not yet
/// published. Read-only; the caller publishes them without holding the case
/// lock, then records each receipt with [`record_anchor_receipt`].
pub fn anchor_work(ctx: &AppendContext) -> Vec<Value> {
    if integrity(ctx).is_err() {
        return Vec::new();
    }
    let committed: BTreeSet<String> = records(ctx)
        .iter()
        .filter(|r| r["kind"] == "episode.frozen")
        .filter_map(|r| r["digest"].as_str().map(String::from))
        .collect();
    let lines = read_lines(&case_dir(ctx.root, ctx.case_id).join(ANCHOR_OUTBOX));
    let published: BTreeSet<String> = lines
        .iter()
        .filter(|a| a["state"] == "published")
        .filter_map(|a| a["id"].as_str().map(String::from))
        .collect();
    let mut seen = BTreeSet::new();
    lines
        .iter()
        .filter(|a| a["state"] == "pending")
        .filter(|a| match (a["id"].as_str(), a["digest"].as_str()) {
            (Some(id), Some(digest)) => {
                !published.contains(id) && committed.contains(digest) && seen.insert(id.to_string())
            }
            _ => false,
        })
        .cloned()
        .collect()
}

/// Record a publisher receipt for `entry` (call under the case lock). A
/// no-op when the entry was published meanwhile.
pub fn record_anchor_receipt(
    ctx: &AppendContext,
    entry: &Value,
    receipt: Value,
) -> Result<bool, String> {
    integrity(ctx)?;
    let path = case_dir(ctx.root, ctx.case_id).join(ANCHOR_OUTBOX);
    let id = entry["id"].as_str().unwrap_or("");
    if read_lines(&path)
        .iter()
        .any(|a| a["state"] == "published" && a["id"] == id)
    {
        return Ok(false);
    }
    let mut done = entry.clone();
    done["state"] = json!("published");
    done["receipt"] = receipt;
    append_line(&path, &done)?;
    Ok(true)
}

/// Publish every pending anchor whose freeze record is committed, in the
/// caller's thread (so under whatever lock the caller holds; each publish is
/// bounded by the publisher's own timeout). A failure leaves the entry
/// pending (retried by the next call); it never touches the case record.
pub fn deliver_anchors(ctx: &AppendContext, publisher: &dyn AnchorPublisher) -> usize {
    let mut sent = 0;
    for entry in anchor_work(ctx) {
        let (Some(id), Some(digest)) = (entry["id"].as_str(), entry["digest"].as_str()) else {
            continue;
        };
        if let Ok(receipt) = publisher.publish("freeze", id, digest) {
            if record_anchor_receipt(ctx, &entry, receipt) == Ok(true) {
                sent += 1;
            }
        }
    }
    sent
}

pub(super) fn with_store_ctx<T>(
    root: &Path,
    case_id: &str,
    f: impl FnOnce(&AppendContext) -> T,
) -> T {
    let index = FileIndex(root.to_path_buf());
    f(&AppendContext {
        root,
        case_id,
        principal: "anchor-delivery",
        role: "service",
        chain: &[],
        rt: &index,
        anchor: None,
        policy: &AppendPolicy::NONE,
    })
}

/// [`anchor_work`] for a case, without a request context.
pub fn anchor_work_for(root: &Path, case_id: &str) -> Vec<Value> {
    if !valid_case_segment(case_id) {
        return Vec::new();
    }
    with_store_ctx(root, case_id, anchor_work)
}

/// [`record_anchor_receipt`] for a case; call under the case lock.
pub fn record_anchor_receipt_for(
    root: &Path,
    case_id: &str,
    entry: &Value,
    receipt: Value,
) -> Result<bool, String> {
    with_store_ctx(root, case_id, |ctx| {
        record_anchor_receipt(ctx, entry, receipt)
    })
}

/// [`recover`] for a case without a request context; call under the case lock.
pub fn recover_for(root: &Path, case_id: &str) -> Result<Recovered, ForgeError> {
    if !valid_case_segment(case_id) {
        return Ok(Recovered::default());
    }
    with_store_ctx(root, case_id, recover)
}

/// `<root>/fde-config.json`: the cell's anchoring configuration (WP-21). Absent
/// means anchoring is not configured. Keys: `case_keeper_identity` (the service
/// identity that publishes anchors) and `anchor_publisher_identity` (the
/// identity the configured anchor credential publishes as, when it is not the
/// case keeper). An unreadable or malformed file is an error, never "not
/// configured".
pub fn fde_config(root: &Path) -> Result<Value, String> {
    let path = root.join("fde-config.json");
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text)
            .map_err(|e| format!("{} is not valid JSON: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
        Err(e) => Err(format!("read {}: {e}", path.display())),
    }
}

/// What the execution gate (README §7) reads about one episode and its case.
#[derive(Debug)]
pub struct EpisodeFacts {
    pub lifecycle: Lifecycle,
    /// RealityTrace plan, question and threshold exist.
    pub realitytrace: bool,
    /// The `episode.frozen` record exists and its intent is committed.
    pub case_record: bool,
    /// A freeze anchor exists for the episode.
    pub anchor: bool,
    /// Anchoring is configured, so the anchor is part of a complete freeze.
    pub anchor_required: bool,
    /// The anchor's pusher is the case keeper (true when none is configured).
    pub anchor_publisher_ok: bool,
    /// The kind recorded at the freeze.
    pub frozen_kind: Option<String>,
    /// The highest `n` allocated for the plan item (0 when none).
    pub latest_n: u64,
    pub settled: bool,
    /// The latest case-level disposition.
    pub latest_disposition: Option<String>,
    /// The frozen `episode_ref` (`episode:<case>:<item>:<n>@sha256:<digest>`).
    pub episode_ref: Option<String>,
    /// The world the episode froze.
    pub world_ref: Option<String>,
}

/// Read [`EpisodeFacts`] from the case store: `Err` when it fails its integrity
/// check or the cell's anchoring configuration is unreadable (the gate fails
/// closed on both).
pub fn episode_facts(
    root: &Path,
    case_id: &str,
    plan_item_id: &str,
    n: u64,
) -> Result<EpisodeFacts, String> {
    if !valid_case_segment(case_id) {
        return Err(format!("unsafe case id {case_id}"));
    }
    let dir = sea_forge_case_runner::case_ops::case_snapshot_dir(root, case_id)
        .map_err(|e| format!("store integrity: {e}"))?;
    episode_facts_in(root, &dir, case_id, plan_item_id, n)
}

/// Read episode facts from the caller's already resolved case snapshot.
pub(super) fn episode_facts_in(
    root: &Path,
    dir: &Path,
    case_id: &str,
    plan_item_id: &str,
    n: u64,
) -> Result<EpisodeFacts, String> {
    let config = fde_config(root)?;
    let policy = AppendPolicy {
        pinned_world: None,
        case_keeper: config["case_keeper_identity"].as_str(),
        anchor_publisher: None,
        principal_human: false,
    };
    let index = FileIndex(root.to_path_buf());
    let ctx = AppendContext {
        root,
        case_id,
        principal: "execution-gate",
        role: "service",
        chain: &[],
        rt: &index,
        anchor: None,
        policy: &policy,
    };
    integrity_in(dir)?;
    let key = key_json(case_id, plan_item_id, n);
    let all = read_lines(&dir.join(RECORDS));
    let intents = latest_intents_in(dir);
    let record = all
        .iter()
        .find(|r| r["kind"] == "episode.frozen" && r["episode_key"] == key);
    let intent = intents
        .iter()
        .find(|i| i["episode_key"] == key && i["state"] != "abandoned");
    let source = record.or(intent);
    let committed = record.is_some_and(|r| {
        intents
            .iter()
            .any(|i| i["idempotency_key"] == r["idempotency_key"] && i["state"] == "committed")
    });
    let refs_exist = source.is_some_and(|src| {
        let refs = &src["payload"]["realitytrace_refs"];
        std::iter::once(&refs["plan_id"])
            .chain(refs["question_ids"].as_array().into_iter().flatten())
            .chain(refs["threshold_ids"].as_array().into_iter().flatten())
            .all(|v| v.as_str().is_some_and(|id| ctx.rt.exists(id)))
    });
    let prefix = format!("episode:{case_id}:{plan_item_id}:{n}@");
    let digest = source.and_then(|src| src["digest"].as_str()).unwrap_or("");
    let anchor = published_anchor_for(dir, &prefix, digest);
    let publisher_ok = match (ctx.policy.case_keeper, &anchor) {
        (Some(keeper), Some(a)) => a["receipt"]["pusher_identity"]
            .as_str()
            .map(canonical_actor)
            .is_some_and(|p| p.is_ok() && p == canonical_actor(keeper)),
        _ => true,
    };
    let latest_n = all
        .iter()
        .filter(|r| r["kind"] == "episode.frozen")
        .chain(intents.iter().filter(|i| i["state"] != "abandoned"))
        .filter(|r| {
            r["episode_key"]["case_id"] == case_id
                && r["episode_key"]["plan_item_id"] == plan_item_id
        })
        .filter_map(|r| r["episode_key"]["n"].as_u64())
        .max()
        .unwrap_or(0);
    Ok(EpisodeFacts {
        lifecycle: lifecycle(&all),
        realitytrace: refs_exist || (source.is_none() && ctx.rt.episode_complete(&key)),
        case_record: committed,
        anchor: anchor.is_some(),
        anchor_required: ctx.anchor_configured(),
        anchor_publisher_ok: publisher_ok,
        frozen_kind: source.and_then(|src| src["episode_kind"].as_str().map(String::from)),
        latest_n,
        settled: all
            .iter()
            .any(|r| r["kind"] == "episode.settled" && r["episode_key"] == key),
        world_ref: source
            .and_then(|src| src["payload"]["reference"]["body"]["world_ref"].as_str())
            .map(String::from),
        episode_ref: source
            .and_then(|src| src["digest"].as_str())
            .map(|d| format!("episode:{case_id}:{plan_item_id}:{n}@sha256:{d}")),
        latest_disposition: all
            .iter()
            .rev()
            .find(|r| r["kind"] == "disposition.recorded" && r["level"] == "case")
            .and_then(|r| r["disposition"].as_str().map(String::from)),
    })
}

fn published_anchor_for(dir: &Path, subject_prefix: &str, digest: &str) -> Option<Value> {
    read_lines(&dir.join(ANCHOR_OUTBOX)).into_iter().find(|a| {
        a["state"] == "published"
            && a["kind"] == "freeze"
            && (a["subject_ref"]
                .as_str()
                .is_some_and(|r| r.starts_with(subject_prefix))
                || (!digest.is_empty() && a["digest"] == digest))
    })
}

/// Owe and try to deliver the interaction event of a committed run record.
/// The caller holds the case lock.
pub fn enqueue_run_event(root: &Path, case_id: &str, run: &Value) -> Result<(), ForgeError> {
    use super::interaction_events::{deliver_events, enqueue_run, keep_uncertainty, SpoolSink};
    if !valid_case_segment(case_id) {
        return Ok(());
    }
    with_store_ctx(root, case_id, |ctx| {
        keep_uncertainty(enqueue_run(ctx, run))?;
        deliver_events(ctx, &SpoolSink::new(root))?;
        Ok(())
    })
}

/// The wire-level helper the SFWP verb uses: build the context from server state.
/// `contract` is the request's dialect marker (or null), `pinned_world` the
/// world pinned in the WP-18 pin record (`None`: no pin source configured).
#[allow(clippy::too_many_arguments)]
pub fn run_verb(
    root: &Path,
    case_id: &str,
    principal: &str,
    role: &str,
    kind: Value,
    payload: Value,
    idempotency_key: Value,
    chain: Vec<String>,
    contract: Value,
    pinned_world: Option<String>,
) -> Result<Value, sea_forge_core::errors::ForgeError> {
    if !valid_case_segment(case_id) || !case_dir(root, case_id).join("case.json").exists() {
        return Err(ForgeError::Input(format!("unknown case {case_id}")));
    }
    let mut request = Map::new();
    request.insert("kind".into(), kind);
    request.insert("payload".into(), payload);
    request.insert("idempotency_key".into(), idempotency_key);
    if !contract.is_null() {
        request.insert("contract".into(), contract);
    }
    let config = fde_config(root).map_err(|e| ForgeError::Plan {
        class: "store_integrity",
        message: e,
    })?;
    let policy = AppendPolicy {
        pinned_world: pinned_world.as_deref(),
        case_keeper: config["case_keeper_identity"].as_str(),
        anchor_publisher: config["anchor_publisher_identity"].as_str(),
        // Derived from the authenticated role, never from the id's spelling.
        principal_human: role == "operator",
    };
    let index = FileIndex(root.to_path_buf());
    // Inside the case lock the configured publisher is only a marker that an
    // anchor is required; the push runs after the lock is released.
    let script = configured_publisher().map(|_| DeferredPublisher);
    // `chain` is the delegation chain the identity gate established for this
    // request, root first, ending at the principal; it never defaults to a
    // caller-chosen value. An empty one is completed with the principal alone.
    let mut chain = chain;
    if chain.last().map(String::as_str) != Some(principal) {
        chain.push(principal.to_string());
    }
    let ctx = AppendContext {
        root,
        case_id,
        principal,
        role,
        chain: &chain,
        rt: &index,
        anchor: script.as_ref().map(|s| s as &dyn AnchorPublisher),
        policy: &policy,
    };
    let answer = try_append_answer(&ctx, &Value::Object(request))?;
    // Deliver whatever is owed (this write's events and any earlier backlog).
    // A failing emitter leaves the events owed. Uncertainty in the case's
    // own outbox must reach the caller as an operational error.
    super::interaction_events::deliver_events(
        &ctx,
        &super::interaction_events::SpoolSink::new(root),
    )?;
    match answer.frame_class {
        Some(class) => Err(ForgeError::Plan {
            class,
            message: answer.value["message"].as_str().unwrap_or("").to_string(),
        }),
        None => Ok(answer.value),
    }
}

fn valid_case_segment(id: &str) -> bool {
    sea_forge_core::path::valid_id_segment(id, 128)
}
