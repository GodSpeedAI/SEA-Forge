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

/// Runs `scripts/fde-anchor.sh publish <kind> <id> <digest> --push`. Only built
/// when the operator configures a script path: pushing a tag is an external,
/// irreversible effect.
pub struct ScriptPublisher(pub PathBuf);

impl AnchorPublisher for ScriptPublisher {
    fn publish(&self, kind: &str, id: &str, digest: &str) -> Result<Value, String> {
        let out = std::process::Command::new(&self.0)
            .args(["publish", kind, id, digest, "--push"])
            .output()
            .map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(json!({"script": self.0.display().to_string(),
                      "stdout": String::from_utf8_lossy(&out.stdout).trim()}))
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    }
}

/// Everything one append needs besides the request.
pub struct AppendContext<'a> {
    pub root: &'a Path,
    pub case_id: &'a str,
    /// The authenticated principal of the request (the settler for `episode.settled`).
    pub principal: &'a str,
    /// The principal's authenticated delegation chain, root first, ending at
    /// `principal`. At this stage these are SEA-Forge actor identities (RD-21).
    pub chain: &'a [String],
    pub rt: &'a dyn RealityTraceIndex,
    pub anchor: Option<&'a dyn AnchorPublisher>,
}

// ---------------------------------------------------------------------------
// File helpers
// ---------------------------------------------------------------------------

fn case_dir(root: &Path, case_id: &str) -> PathBuf {
    root.join("cases").join(case_id)
}

/// Complete newline-terminated JSON lines; a torn trailing fragment is ignored.
fn read_lines(path: &Path) -> Vec<Value> {
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

fn append_line(path: &Path, value: &Value) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("open {}: {e}", path.display()))?;
    let mut line = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    line.push(b'\n');
    file.write_all(&line)
        .and_then(|_| file.sync_data())
        .map_err(|e| format!("write {}: {e}", path.display()))
}

fn records(ctx: &AppendContext) -> Vec<Value> {
    read_lines(&case_dir(ctx.root, ctx.case_id).join(RECORDS))
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
    if let Err(refusal) = validate_request(request) {
        return Ok(refusal);
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
    let hash = digest_of(&record).ok_or("record is not canonicalisable")?;
    record["hash"] = json!(hash);
    append_line(&case_dir(ctx.root, ctx.case_id).join(RECORDS), &record)?;
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
    // I-1: a settlement needs the freeze record.
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
    // I-2: the settling principal's chain must not contain an executor of the item.
    let executors = executors_of(ctx, &item);
    if ctx
        .chain
        .iter()
        .chain(std::iter::once(&ctx.principal.to_string()))
        .any(|c| executors.contains(c))
    {
        return refuse(
            "SELF_SETTLEMENT_REFUSED",
            "settling principal's chain contains the executor's identity",
            Some("I-2"),
        );
    }
    let record = json!({
        "kind": "episode.settled", "idempotency_key": key, "body_digest": body_digest,
        "episode_key": episode_key, "principal": ctx.principal, "chain": ctx.chain, "payload": payload,
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

/// Execution identities for a plan item: the actor of every case event that
/// names it, plus actors recorded on freeze records' executors. These are
/// SEA-Forge actor ids; the verb never trusts a caller-supplied tag.
fn executors_of(ctx: &AppendContext, plan_item_id: &str) -> BTreeSet<String> {
    read_lines(&case_dir(ctx.root, ctx.case_id).join("case-events.jsonl"))
        .iter()
        .filter(|e| e["plan_item_id"] == plan_item_id)
        .filter_map(|e| e["actor_id"].as_str().map(String::from))
        .collect()
}

// ---------------------------------------------------------------------------
// Freeze status, recovery, anchors
// ---------------------------------------------------------------------------

/// Whether an episode's freeze is complete enough to delegate against (I-1).
#[derive(Debug, PartialEq, Eq)]
pub enum FreezeStatus {
    /// No record and no intent.
    Missing,
    /// Parts present but not all three: `case_record`, `realitytrace_records`, `anchor`.
    Incomplete(Vec<&'static str>),
    Complete,
}

pub fn freeze_status(ctx: &AppendContext, plan_item_id: &str, n: u64) -> FreezeStatus {
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
    if record.is_none() {
        missing.push("case_record");
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
    if !anchored {
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

/// NT-16 / AT-04 order check from the ledger side: the freeze's anchor must be
/// published, and its server push time must be earlier than the first run
/// record (the first case event naming the plan item). The push time is read
/// from the publisher's receipt (`server_push_time`, RFC 3339); a ledger
/// regenerated with the freeze inserted early still fails, because the anchor
/// time is not ledger-controlled. `ANCHOR_PREDATES_FAILED` otherwise.
pub fn verify_freeze_anchor(ctx: &AppendContext, plan_item_id: &str, n: u64) -> Result<(), Value> {
    let fail = |why: &str| Err(refuse_anchor(why));
    let Some(frozen) = records(ctx).into_iter().find(|r| {
        r["kind"] == "episode.frozen" && r["episode_key"] == key_json(ctx.case_id, plan_item_id, n)
    }) else {
        return fail("no freeze record");
    };
    let digest = frozen["digest"].as_str().unwrap_or("");
    let anchor = read_lines(&case_dir(ctx.root, ctx.case_id).join(ANCHOR_OUTBOX))
        .into_iter()
        .find(|a| a["state"] == "published" && a["digest"] == digest);
    let Some(anchor) = anchor else {
        return fail("anchor tag absent");
    };
    let parse = |v: &Value| {
        v.as_str()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
    };
    let Some(pushed) = parse(&anchor["receipt"]["server_push_time"]) else {
        return fail("receipt has no server push time");
    };
    let first_run = read_lines(&case_dir(ctx.root, ctx.case_id).join("case-events.jsonl"))
        .iter()
        .filter(|e| e["plan_item_id"] == plan_item_id)
        .filter_map(|e| parse(&e["timestamp"]))
        .min();
    match first_run {
        Some(run) if pushed >= run => fail("anchor push time is not before the first run record"),
        _ => Ok(()),
    }
}

fn refuse_anchor(why: &str) -> Value {
    // anchor.v1 error table, not the case.record.append enum.
    json!({"outcome": "refused", "code": "ANCHOR_PREDATES_FAILED", "message": why})
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
pub fn recover(ctx: &AppendContext) -> Recovered {
    let mut out = Recovered::default();
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
    out
}

/// Publish every pending anchor whose freeze record is committed. A failure
/// leaves the entry pending (retried by the next call); it never touches the
/// case record.
pub fn deliver_anchors(ctx: &AppendContext, publisher: &dyn AnchorPublisher) -> usize {
    let dir = case_dir(ctx.root, ctx.case_id);
    let committed: BTreeSet<String> = records(ctx)
        .iter()
        .filter(|r| r["kind"] == "episode.frozen")
        .filter_map(|r| r["digest"].as_str().map(String::from))
        .collect();
    let lines = read_lines(&dir.join(ANCHOR_OUTBOX));
    let published: BTreeSet<String> = lines
        .iter()
        .filter(|a| a["state"] == "published")
        .filter_map(|a| a["id"].as_str().map(String::from))
        .collect();
    let mut sent = 0;
    for entry in lines.iter().filter(|a| a["state"] == "pending") {
        let (Some(id), Some(digest)) = (entry["id"].as_str(), entry["digest"].as_str()) else {
            continue;
        };
        if published.contains(id) || !committed.contains(digest) {
            continue;
        }
        if let Ok(receipt) = publisher.publish("freeze", id, digest) {
            let mut done = entry.clone();
            done["state"] = json!("published");
            done["receipt"] = receipt;
            if append_line(&dir.join(ANCHOR_OUTBOX), &done).is_ok() {
                sent += 1;
            }
        }
    }
    sent
}

/// The wire-level helper the SFWP verb uses: build the context from server state.
pub fn run_verb(
    root: &Path,
    case_id: &str,
    principal: &str,
    kind: Value,
    payload: Value,
    idempotency_key: Value,
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
    let script =
        std::env::var_os("SEA_FORGE_FDE_ANCHOR_SCRIPT").map(|p| ScriptPublisher(PathBuf::from(p)));
    let chain = vec![principal.to_string()];
    let ctx = AppendContext {
        root,
        case_id,
        principal,
        chain: &chain,
        rt: &index,
        anchor: script.as_ref().map(|s| s as &dyn AnchorPublisher),
    };
    Ok(append(&ctx, &Value::Object(request)))
}

fn valid_case_segment(id: &str) -> bool {
    sea_forge_core::path::valid_id_segment(id, 128)
}
