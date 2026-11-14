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
const ANCHOR: &str = include_str!("../../contracts/fde/anchor.v1.json");

const ZERO_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
/// Largest `payload` (canonical JSON bytes) one `case.record.append` may carry.
/// The socket frame cap is 1 MiB per request line (`MAX_REQUEST_BYTES` in
/// `lib.rs`); a payload is stored in the record, the freeze intent and the
/// outbox, so it is held to a quarter of the frame.
pub const MAX_PAYLOAD_BYTES: usize = 256 * 1024;

const RECORDS: &str = "field-records.jsonl";
const INTENTS: &str = "field-freeze-intents.jsonl";
const RT_OUTBOX: &str = "field-rt-outbox.jsonl";
const ANCHOR_OUTBOX: &str = "field-anchor-outbox.jsonl";

/// The vendored schema texts, for the contract gate test.
pub fn vendored_schemas() -> [(&'static str, &'static str); 3] {
    [
        ("anchor.v1.json", ANCHOR),
        ("case-record-append.v1.json", CASE_RECORD_APPEND),
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

fn refuse(code: &str, message: &str, invariant: Option<&str>) -> Value {
    let mut out = json!({"outcome": "refused", "code": code, "message": message});
    if let Some(invariant) = invariant {
        out["invariant"] = json!(invariant);
    }
    out
}

fn is_refusal(v: &Value) -> bool {
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

/// Existence of RealityTrace records, resolved at their owner.
pub trait RealityTraceIndex {
    fn exists(&self, record_id: &str) -> bool;
}

/// Reads `<root>/realitytrace/records.jsonl` (`{"record_id": ...}` per line).
pub struct FileIndex(pub PathBuf);

impl RealityTraceIndex for FileIndex {
    fn exists(&self, record_id: &str) -> bool {
        read_lines(&self.0.join("realitytrace").join("records.jsonl"))
            .iter()
            .any(|line| line["record_id"] == record_id)
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

const REPAIRS: &str = "field-repairs.jsonl";

/// Every store file whose lines decide an invariant, an id or idempotency must
/// parse completely before anything is decided or written.
pub(super) fn integrity(ctx: &AppendContext) -> Result<(), String> {
    let dir = case_dir(ctx.root, ctx.case_id);
    for name in [RECORDS, INTENTS, DECISIONS, RT_OUTBOX, ANCHOR_OUTBOX] {
        read_lines_strict(&dir.join(name)).map_err(|e| format!("store integrity: {e}"))?;
    }
    Ok(())
}

fn sync_append(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("open {}: {e}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_data())
        .map_err(|e| format!("write {}: {e}", path.display()))
}

/// A crash mid-append leaves a trailing fragment with no newline. Appending to
/// it would glue the next record onto the fragment, so before every append the
/// fragment is cut back to the last newline. The cut is noted durably in
/// `field-repairs.jsonl` (beside the file, with the dropped text) BEFORE the
/// truncation, so a repair never silently discards bytes.
fn repair_torn_tail(path: &Path) -> Result<(), String> {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(format!("read {}: {e}", path.display())),
    };
    if bytes.last().is_none_or(|b| *b == b'\n') {
        return Ok(());
    }
    let keep = bytes.iter().rposition(|b| *b == b'\n').map_or(0, |i| i + 1);
    let fragment = &bytes[keep..];
    let note = json!({
        "repaired_at": now_rfc3339(),
        "file": path.file_name().map(|n| n.to_string_lossy().to_string()),
        "kept_bytes": keep, "dropped_bytes": fragment.len(),
        "dropped_sha256": hex::encode(Sha256::digest(fragment)),
        "dropped_text": String::from_utf8_lossy(&fragment[..fragment.len().min(512)]),
    });
    let log = path.with_file_name(REPAIRS);
    let mut line = serde_json::to_vec(&note).map_err(|e| e.to_string())?;
    line.push(b'\n');
    let log_torn = fs::read(&log).is_ok_and(|b| b.last().is_some_and(|c| *c != b'\n'));
    if log_torn {
        line.insert(0, b'\n');
    }
    sync_append(&log, &line)?;
    let file = OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| format!("open {}: {e}", path.display()))?;
    file.set_len(keep as u64)
        .and_then(|_| file.sync_all())
        .map_err(|e| format!("repair {}: {e}", path.display()))
}

pub(super) fn append_line(path: &Path, value: &Value) -> Result<(), String> {
    repair_torn_tail(path)?;
    let mut line = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    line.push(b'\n');
    sync_append(path, &line)
}

pub(super) fn records(ctx: &AppendContext) -> Vec<Value> {
    read_lines(&case_dir(ctx.root, ctx.case_id).join(RECORDS))
}

pub(super) const DECISIONS: &str = "field-decisions.jsonl";

pub(super) fn decisions(ctx: &AppendContext) -> Vec<Value> {
    read_lines(&case_dir(ctx.root, ctx.case_id).join(DECISIONS))
}

pub(super) fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// Record the authority decision the I-2 check makes for a settlement attempt.
/// SEA-Forge is the owner of these records (`dec-NNNN`); the interaction event
/// for the decision and the settlement's `authority.decision_id` resolve here.
/// Idempotent on `(idempotency_key, body_digest, decision)`: a crash-retry or
/// a replayed denial reuses the first record rather than minting another.
fn record_decision(
    ctx: &AppendContext,
    key: &str,
    body_digest: &str,
    episode_key: &Value,
    allowed: bool,
    executors: &BTreeSet<String>,
) -> Result<Value, String> {
    let decision = if allowed { "allow" } else { "deny" };
    let existing = decisions(ctx);
    if let Some(found) = existing.iter().find(|d| {
        d["idempotency_key"] == key && d["body_digest"] == body_digest && d["decision"] == decision
    }) {
        return Ok(found.clone());
    }
    let seq = existing.len() as u64 + 1;
    let record = json!({
        "decision_id": format!("dec-{seq:04}"), "seq": seq, "policy": "no_self_settlement",
        "invariant": "I-2", "decision": decision, "idempotency_key": key,
        "body_digest": body_digest, "episode_key": episode_key, "principal": ctx.principal,
        "role": ctx.role, "chain": ctx.chain, "executors": executors,
        "recorded_at": now_rfc3339(),
    });
    append_line(&case_dir(ctx.root, ctx.case_id).join(DECISIONS), &record)?;
    Ok(record)
}

/// Latest intent line per idempotency key.
fn latest_intents(ctx: &AppendContext) -> Vec<Value> {
    let mut latest: Vec<Value> = Vec::new();
    for line in read_lines(&case_dir(ctx.root, ctx.case_id).join(INTENTS)) {
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

fn put_intent(
    ctx: &AppendContext,
    state: &str,
    base: &Value,
    record_id: Option<&str>,
) -> Result<(), String> {
    let mut line = base.clone();
    line["state"] = json!(state);
    if let Some(id) = record_id {
        line["record_id"] = json!(id);
    }
    append_line(&case_dir(ctx.root, ctx.case_id).join(INTENTS), &line)
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

/// Run `case.record.append` and return the contract's response or refusal.
pub fn append(ctx: &AppendContext, request: &Value) -> Value {
    append_faulted(ctx, request, None).unwrap_or_else(|crash| {
        // Unreachable without a fault; keep the contract shape if it ever is.
        refuse("SCHEMA_INVALID", &format!("{crash:?}"), None)
    })
}

/// As [`append`], with a crash injected at a freeze phase boundary. The crash
/// returns `Err` *after* the named boundary's writes are durable, exactly what
/// a process death there leaves on disk.
pub fn append_faulted(
    ctx: &AppendContext,
    request: &Value,
    fault: Option<Fault>,
) -> Result<Value, CrashInjected> {
    if serde_json::to_vec(&request["payload"]).map_or(true, |b| b.len() > MAX_PAYLOAD_BYTES) {
        return Ok(refuse(
            "SCHEMA_INVALID",
            &format!("payload exceeds {MAX_PAYLOAD_BYTES} bytes"),
            None,
        ));
    }
    if let Err(refusal) = validate_request(request) {
        return Ok(refusal);
    }
    if let Err(e) = integrity(ctx) {
        return Ok(io_refusal(e));
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
    let kind = request["kind"].as_str().unwrap_or("");
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
    let Some(body_digest) = digest_of(&json!({"kind": kind, "payload": payload})) else {
        return Ok(refuse(
            "SCHEMA_INVALID",
            "payload is not canonicalisable (float or unsupported value)",
            None,
        ));
    };

    // Idempotency first: a replay must return the original, not trip I-3.
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
                        return Ok(io_refusal(e));
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

    match kind {
        "episode.frozen" => freeze(ctx, payload, key, &body_digest, &records, &intents, fault),
        "episode.settled" => Ok(settle(ctx, payload, key, &body_digest, &records)),
        _ => Ok(refuse(
            "KIND_UNKNOWN",
            "kind is not a registered kind",
            None,
        )),
    }
}

fn next_record(ctx: &AppendContext, records: &[Value], mut record: Value) -> Result<Value, String> {
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
    let hash = digest_of(&record).ok_or("record is not canonicalisable")?;
    record["hash"] = json!(hash);
    append_line(&case_dir(ctx.root, ctx.case_id).join(RECORDS), &record)?;
    // The interaction event is owed from this moment. Enqueueing is best
    // effort: the record is committed either way, and `reconcile_events`
    // derives any event the ledger holds but the outbox lacks (WP-14: an
    // emitter failure never blocks the case write).
    let _ = super::interaction_events::enqueue_record(ctx, &record);
    Ok(record)
}

fn io_refusal(error: String) -> Value {
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
) -> Result<Value, CrashInjected> {
    let (case, item, n, digest) = match check_freeze_digests(payload) {
        Ok(parts) => parts,
        Err(refusal) => return Ok(refusal),
    };
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
    let episode_key = key_json(&case, &item, n);
    let kind_of_episode = payload["reference"]["body"]["kind"]
        .as_str()
        .unwrap_or("evaluated");

    // Phase 1: prepare. Intent and the owed outbox entries are durable before
    // the case record exists.
    let intent = json!({
        "idempotency_key": key, "body_digest": body_digest, "episode_key": episode_key,
        "episode_kind": kind_of_episode, "digest": digest, "payload": payload,
    });
    let already_prepared = intents.iter().any(|i| i["idempotency_key"] == key);
    if !already_prepared {
        if let Err(e) = put_intent(ctx, "prepared", &intent, None) {
            return Ok(io_refusal(e));
        }
        if let Err(e) = enqueue_freeze_outbox(ctx, payload, &case, &item, n, &digest) {
            return Ok(io_refusal(e));
        }
    }
    if fault == Some(Fault::AfterPrepare) {
        return Err(CrashInjected(Fault::AfterPrepare));
    }
    commit_freeze(ctx, &intent, records, fault)
}

/// Phase 2: append the case record, mark the intent committed, then try the anchor.
fn commit_freeze(
    ctx: &AppendContext,
    intent: &Value,
    records: &[Value],
    fault: Option<Fault>,
) -> Result<Value, CrashInjected> {
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
                Err(e) => return Ok(io_refusal(e)),
            }
        }
    };
    if fault == Some(Fault::AfterRecord) {
        return Err(CrashInjected(Fault::AfterRecord));
    }
    let rec_id = stored["record_id"].as_str().unwrap_or("").to_string();
    if let Err(e) = put_intent(ctx, "committed", intent, Some(&rec_id)) {
        return Ok(io_refusal(e));
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

fn enqueue_freeze_outbox(
    ctx: &AppendContext,
    payload: &Value,
    case: &str,
    item: &str,
    n: u64,
    digest: &str,
) -> Result<(), String> {
    let dir = case_dir(ctx.root, ctx.case_id);
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
    let existing = read_lines(&dir.join(RT_OUTBOX));
    for mut entry in entries {
        // Dropped on (system, record_id): E §11 "Outbox duplicates".
        if existing
            .iter()
            .any(|e| e["system"] == entry["system"] && e["record_id"] == entry["record_id"])
        {
            continue;
        }
        entry["state"] = json!("pending");
        append_line(&dir.join(RT_OUTBOX), &entry)?;
    }
    let id = anchor_id(case, item, n, digest);
    let anchors = read_lines(&dir.join(ANCHOR_OUTBOX));
    if !anchors.iter().any(|a| a["id"] == id.as_str()) {
        append_line(
            &dir.join(ANCHOR_OUTBOX),
            &json!({"state": "pending", "kind": "freeze", "id": id, "tag": format!("fde-anchor/freeze/{id}"),
                    "subject_ref": episode_ref, "digest": digest}),
        )?;
    }
    Ok(())
}

/// The repo's identity canonicalisation for actor strings
/// (`governed_execution_boundary::check_identity_string`: trimmed, empty and
/// placeholder spellings refused), compared case-insensitively. Case folding
/// can only merge two spellings, so it can only add refusals, never miss one.
/// (Unicode NFC is not applied: the crate has no normalisation dependency.)
fn canonical_actor(raw: &str) -> Result<String, String> {
    crate::governed_execution_boundary::check_identity_string("actor", raw)
        .map(|t| t.to_lowercase())
}

fn settle(
    ctx: &AppendContext,
    payload: &Value,
    key: &str,
    body_digest: &str,
    records: &[Value],
) -> Value {
    let Some((case, item, n, _)) = parse_episode_ref(payload["episode_ref"].as_str().unwrap_or(""))
    else {
        return refuse("SCHEMA_INVALID", "payload.episode_ref does not parse", None);
    };
    let episode_key = key_json(&case, &item, n);
    // I-1: a settlement needs a COMPLETE freeze (both phases, the realitytrace
    // records, and the anchor when an anchor publisher is configured), not
    // merely a record.
    match freeze_state(ctx, &item, n, ctx.anchor.is_some()) {
        FreezeStatus::Complete => {}
        FreezeStatus::Missing => {
            return refuse(
                "FREEZE_MISSING",
                "no freeze record for the episode",
                Some("I-1"),
            )
        }
        FreezeStatus::Incomplete(parts) => {
            return refuse(
                "FREEZE_INCOMPLETE",
                &format!("freeze incomplete: missing {}", parts.join(", ")),
                Some("I-1"),
            )
        }
    }
    let Some(freeze) = records
        .iter()
        .find(|r| r["kind"] == "episode.frozen" && r["episode_key"] == episode_key)
    else {
        return refuse(
            "FREEZE_MISSING",
            "no freeze record for the episode",
            Some("I-1"),
        );
    };
    // The settlement must cite the frozen digest, not another one.
    if let (Some(frozen), Some((_, _, _, cited))) = (
        freeze["digest"].as_str(),
        parse_episode_ref(payload["episode_ref"].as_str().unwrap_or("")),
    ) {
        if frozen != cited {
            return refuse(
                "EPISODE_REF_DIGEST_MISMATCH",
                "episode_ref digest differs from the frozen episode",
                None,
            );
        }
    }
    // Exploratory episodes settle only U or C.
    if freeze["episode_kind"] == "exploratory"
        && matches!(payload["result"].as_str(), Some("S") | Some("F"))
    {
        return refuse(
            "EXPLORATORY_NOT_SETTLEABLE",
            "an exploratory episode cannot settle S or F",
            None,
        );
    }
    // One settlement per episode key. The same idempotency key replays before
    // this point; a different key for a settled episode is a conflict (the
    // contract enum has no dedicated code; IDEMPOTENCY_CONFLICT is the closest).
    if records
        .iter()
        .any(|r| r["kind"] == "episode.settled" && r["episode_key"] == episode_key)
    {
        return refuse(
            "IDEMPOTENCY_CONFLICT",
            "the episode is already settled under another idempotency key",
            None,
        );
    }
    // I-2: the settling principal's chain must not contain an executor of the item.
    let executors = match executors_of(ctx, &item) {
        Ok(e) => e,
        Err(why) => return io_refusal(why),
    };
    let contained = ctx
        .chain
        .iter()
        .chain(std::iter::once(&ctx.principal.to_string()))
        .filter_map(|c| canonical_actor(c).ok())
        .any(|c| executors.contains(&c));
    let decision =
        match record_decision(ctx, key, body_digest, &episode_key, !contained, &executors) {
            Ok(d) => d,
            Err(e) => return io_refusal(e),
        };
    let _ = super::interaction_events::enqueue_decision(ctx, &decision);
    if contained {
        return refuse(
            "SELF_SETTLEMENT_REFUSED",
            "settling principal's chain contains the executor's identity",
            Some("I-2"),
        );
    }
    let record = json!({
        "kind": "episode.settled", "idempotency_key": key, "body_digest": body_digest,
        "episode_key": episode_key, "principal": ctx.principal, "chain": ctx.chain, "payload": payload,
        "decision_id": decision["decision_id"],
    });
    let rec_id = format!("rec-{:04}", records.len() + 1);
    let mut record = record;
    record["response"] = response("episode.settled", &rec_id, &episode_key, false);
    match next_record(ctx, records, record) {
        Ok(stored) => response(
            "episode.settled",
            stored["record_id"].as_str().unwrap_or(""),
            &episode_key,
            false,
        ),
        Err(e) => io_refusal(e),
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
fn freeze_state(
    ctx: &AppendContext,
    plan_item_id: &str,
    n: u64,
    require_anchor: bool,
) -> FreezeStatus {
    if integrity(ctx).is_err() {
        return FreezeStatus::Incomplete(vec!["store_integrity"]);
    }
    let key = key_json(ctx.case_id, plan_item_id, n);
    let record = records(ctx)
        .into_iter()
        .find(|r| r["kind"] == "episode.frozen" && r["episode_key"] == key);
    let intent = latest_intents(ctx)
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
            let committed = latest_intents(ctx)
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
    let anchored = read_lines(&case_dir(ctx.root, ctx.case_id).join(ANCHOR_OUTBOX))
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
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
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
    let mut first_run: Option<chrono::DateTime<chrono::FixedOffset>> = None;
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
pub fn recover(ctx: &AppendContext) -> Recovered {
    let mut out = Recovered::default();
    if integrity(ctx).is_err() {
        return out;
    }
    for intent in latest_intents(ctx) {
        if intent["state"] != "prepared" {
            continue;
        }
        let key = intent["idempotency_key"].as_str().unwrap_or("").to_string();
        let payload = &intent["payload"];
        let digest_ok =
            check_freeze_digests(payload).is_ok_and(|(_, _, _, d)| intent["digest"] == d.as_str());
        let refs = &payload["realitytrace_refs"];
        let rt_ok = std::iter::once(&refs["plan_id"])
            .chain(refs["question_ids"].as_array().into_iter().flatten())
            .chain(refs["threshold_ids"].as_array().into_iter().flatten())
            .all(|v| v.as_str().is_some_and(|id| ctx.rt.exists(id)));
        if digest_ok && rt_ok {
            let records = records(ctx);
            if commit_freeze(ctx, &intent, &records, None).is_ok_and(|r| !is_refusal(&r)) {
                out.completed.push(key);
                continue;
            }
        }
        let _ = put_intent(ctx, "abandoned", &intent, None);
        out.abandoned.push(key);
    }
    if let Some(publisher) = ctx.anchor {
        let _ = deliver_anchors(ctx, publisher);
    }
    super::interaction_events::reconcile_events(ctx);
    out
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

fn with_store_ctx<T>(root: &Path, case_id: &str, f: impl FnOnce(&AppendContext) -> T) -> T {
    let index = FileIndex(root.to_path_buf());
    f(&AppendContext {
        root,
        case_id,
        principal: "anchor-delivery",
        role: "service",
        chain: &[],
        rt: &index,
        anchor: None,
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
pub fn recover_for(root: &Path, case_id: &str) -> Recovered {
    if !valid_case_segment(case_id) {
        return Recovered::default();
    }
    with_store_ctx(root, case_id, recover)
}

/// The wire-level helper the SFWP verb uses: build the context from server state.
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
) -> Result<Value, sea_forge_core::errors::ForgeError> {
    if !valid_case_segment(case_id) || !case_dir(root, case_id).join("case.json").exists() {
        return Err(sea_forge_core::errors::ForgeError::Input(format!(
            "unknown case {case_id}"
        )));
    }
    let mut request = Map::new();
    request.insert("kind".into(), kind);
    request.insert("payload".into(), payload);
    request.insert("idempotency_key".into(), idempotency_key);
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
    };
    let result = append(&ctx, &Value::Object(request));
    // Deliver whatever is owed (this write's events and any earlier backlog).
    // A failing emitter leaves the events in the outbox; it never changes the
    // case write's response.
    let _ = super::interaction_events::deliver_events(
        &ctx,
        &super::interaction_events::SpoolSink::new(root),
    );
    Ok(result)
}

fn valid_case_segment(id: &str) -> bool {
    sea_forge_core::path::valid_id_segment(id, 128)
}
