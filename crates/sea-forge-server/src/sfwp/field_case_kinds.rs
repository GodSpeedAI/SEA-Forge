//! `case.record.append` kinds `field_case.revised`, `strategy.recorded`,
//! `gate.assessed` and `disposition.recorded` (WP-21 part B,
//! `contracts/fde/README.md` sections 4, 5 and 6).
//!
//! Same store and same lock discipline as the two episode kinds (the caller
//! holds the case's keyed lock): records go into the hash-chained
//! `field-records.jsonl`; the Field Case document `field_case.json` is a
//! projection of the `field_case.revised` records and is rebuilt from them when
//! a crash left it behind. Nothing here advances a case by itself: a gate
//! assessment, a strategy or a revision changes records, never the plan.
//!
//! Precedence (README section 6, first that applies wins): `KIND_UNKNOWN` and
//! `SCHEMA_INVALID` (and the principal checks), `STORE_INTEGRITY`, idempotency,
//! `CASE_CLOSED`, `CASE_SUSPENDED`, `RESERVED_DECISION_REFUSED`,
//! `SELF_SETTLEMENT_REFUSED`, `HARD_GATE_NOT_WAIVABLE`, `AUTHORITY_DENIED`,
//! `REVISION_CONFLICT`.
//!
//! Readings of the contract where it is silent (fail closed, listed in the
//! WP-21 part B report): a kind of this family on a case that is not a Field
//! Case is `SCHEMA_INVALID`; an `approval_ref` that the request does not need
//! is `SCHEMA_INVALID`; a `strategy.recorded` whose `supersedes` is not the
//! latest strategy record is `REVISION_CONFLICT`; an approval is consumed
//! before the record is written (a crash then wastes the approval, it never
//! lets one approval authorise two records); the A3 assurance of an approval is
//! not checked (RD-21: enforcement is off until WP-31).

use super::case_records::{
    append_line, bound_approval_named, canonical_actor, case_dir, digest_of, integrity, io_refusal,
    lifecycle, next_record, records, refuse, store_refusal, vendored_schemas, AppendContext,
    Lifecycle, APPROVALS,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use sea_forge_core::field_case::{is_field_path, TEMPLATE_REF};

/// The document file of a Field Case, inside the case directory.
pub const DOCUMENT: &str = "field_case.json";

/// The boundary paths that are read-only after the case's first freeze
/// (`case-record-kinds.v1` `reserved_paths_after_freeze`).
pub const RESERVED_PATHS: [&str; 5] = [
    "world_and_boundary.world_ref_and_version",
    "observer_and_authority.independent_verifier",
    "observer_and_authority.settlement_and_disposition_authority",
    "observer_and_authority.delegated_actors",
    "observer_and_authority.risk_and_external_effect_limits",
];

/// Gates whose `not_applicable` needs the A3-backed approval (D12).
pub const HARD_GATES: [&str; 2] = ["allocate_authority", "integrate_govern"];

// ---------------------------------------------------------------------------
// Schemas
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

fn validator(entry: &str) -> jsonschema::Validator {
    let docs: std::collections::HashMap<String, Value> = vendored_schemas()
        .iter()
        .map(|(_, text)| {
            let doc: Value = serde_json::from_str(text).expect("vendored schema is JSON");
            (doc["$id"].as_str().expect("schema $id").to_string(), doc)
        })
        .collect();
    jsonschema::options()
        .with_retriever(Registry(docs))
        .build(&json!({ "$ref": entry }))
        .expect("vendored schemas compile")
}

fn request_validator() -> &'static jsonschema::Validator {
    static V: OnceLock<jsonschema::Validator> = OnceLock::new();
    V.get_or_init(|| validator("urn:sea-forge:fde:case-record-append:v2#/$defs/Request"))
}

fn document_validator() -> &'static jsonschema::Validator {
    static V: OnceLock<jsonschema::Validator> = OnceLock::new();
    V.get_or_init(|| validator("urn:sea-forge:fde:field-case:v1"))
}

/// Whether `document` is a valid `field-case.v1` document.
pub fn document_is_valid(document: &Value) -> Result<(), String> {
    let errors: Vec<String> = document_validator()
        .iter_errors(document)
        .map(|e| format!("{}: {e}", e.instance_path()))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

// ---------------------------------------------------------------------------
// The document
// ---------------------------------------------------------------------------

fn document_path(ctx: &AppendContext) -> PathBuf {
    case_dir(ctx.root, ctx.case_id).join(DOCUMENT)
}

/// Whether the case was opened from the Field Case template.
pub fn is_field_case(root: &std::path::Path, case_id: &str) -> Result<bool, String> {
    let path = case_dir(root, case_id).join("plan.json");
    match fs::read(&path) {
        Ok(bytes) => {
            let doc: Value = serde_json::from_slice(&bytes)
                .map_err(|e| format!("{} is not valid JSON: {e}", path.display()))?;
            Ok(doc["template_ref"].as_str() == Some(TEMPLATE_REF))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("read {}: {e}", path.display())),
    }
}

/// Apply the changes of one `field_case.revised` payload to a document and set
/// its revision.
fn apply(document: &mut Value, changes: &Value, revision: u64) {
    for change in changes.as_array().into_iter().flatten() {
        let (Some(path), field) = (change["path"].as_str(), &change["field"]) else {
            continue;
        };
        if let Some((group, name)) = path.split_once('.') {
            document["fields"][group][name] = field.clone();
        }
    }
    document["revision"] = json!(revision);
}

/// The document revision the ledger says is current: 1 at open, the `revision`
/// of the latest accepted `field_case.revised` record afterwards.
pub fn ledger_revision(records: &[Value]) -> u64 {
    records
        .iter()
        .filter(|r| r["kind"] == "field_case.revised")
        .filter_map(|r| r["revision"].as_u64())
        .max()
        .unwrap_or(1)
}

fn write_document(ctx: &AppendContext, document: &Value) -> Result<(), String> {
    let path = document_path(ctx);
    let tmp = path.with_extension("json.tmp");
    let mut bytes = serde_json::to_vec_pretty(document).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    {
        use std::io::Write;
        let mut file =
            fs::File::create(&tmp).map_err(|e| format!("create {}: {e}", tmp.display()))?;
        file.write_all(&bytes).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
    }
    fs::rename(&tmp, &path).map_err(|e| format!("rename {}: {e}", path.display()))
}

/// The case's current document, brought up to the ledger when a crash left the
/// file behind its records. `Err` is `STORE_INTEGRITY`: no file, an invalid
/// document, a document of another case, or one ahead of its ledger.
pub fn load_document(ctx: &AppendContext, all: &[Value]) -> Result<Value, String> {
    let path = document_path(ctx);
    let bytes = fs::read(&path)
        .map_err(|e| format!("store integrity: field_case.json of a Field Case: {e}"))?;
    let mut document: Value = serde_json::from_slice(&bytes)
        .map_err(|e| format!("store integrity: field_case.json is not JSON: {e}"))?;
    document_is_valid(&document).map_err(|e| format!("store integrity: field_case.json: {e}"))?;
    if document["case_id"] != ctx.case_id {
        return Err("store integrity: field_case.json names another case".into());
    }
    let have = document["revision"].as_u64().unwrap_or(0);
    let want = ledger_revision(all);
    if have > want {
        return Err(format!(
            "store integrity: field_case.json is at revision {have}, ahead of its ledger ({want})"
        ));
    }
    if have < want {
        // Replay the records the file is missing, in order.
        let mut healed = false;
        for r in all
            .iter()
            .filter(|r| r["kind"] == "field_case.revised")
            .filter(|r| r["revision"].as_u64().is_some_and(|n| n > have))
        {
            if !r["payload"]["changes"].is_array() {
                return Err(
                    "store integrity: a field_case.revised record carries no changes".into(),
                );
            }
            apply(
                &mut document,
                &r["payload"]["changes"],
                r["revision"].as_u64().unwrap_or(0),
            );
            healed = true;
        }
        if healed {
            document_is_valid(&document)
                .map_err(|e| format!("store integrity: replayed document: {e}"))?;
            write_document(ctx, &document).map_err(|e| format!("store integrity: {e}"))?;
        }
        if document["revision"].as_u64() != Some(want) {
            return Err("store integrity: field_case.json cannot be brought to its ledger".into());
        }
    }
    Ok(document)
}

// ---------------------------------------------------------------------------
// Append
// ---------------------------------------------------------------------------

fn response(kind: &str, record_id: &str, replayed: bool) -> Value {
    json!({"outcome": "recorded", "kind": kind, "record_id": record_id, "replayed": replayed})
}

/// The schema-level mapping of the contract: `/kind` is `KIND_UNKNOWN`,
/// everything else `SCHEMA_INVALID`.
fn validate(request: &Value) -> Result<(), Value> {
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
    Err(refuse("SCHEMA_INVALID", &message, None))
}

fn schema_invalid(message: &str) -> Value {
    refuse("SCHEMA_INVALID", message, None)
}

/// Checks the request schema cannot express (README section 6).
fn semantic_checks(ctx: &AppendContext, kind: &str, payload: &Value) -> Result<(), Value> {
    match kind {
        "field_case.revised" => {
            let mut seen = BTreeSet::new();
            for change in payload["changes"].as_array().into_iter().flatten() {
                let path = change["path"].as_str().unwrap_or("");
                if !is_field_path(path) {
                    return Err(schema_invalid(&format!("{path} is not a Field Case field")));
                }
                if !seen.insert(path) {
                    return Err(schema_invalid(&format!("{path} is changed twice")));
                }
            }
        }
        "gate.assessed" => {
            let assessor = payload["assessor"].as_str().unwrap_or("");
            match (canonical_actor(assessor), canonical_actor(ctx.principal)) {
                (Ok(a), Ok(p)) if a == p => {}
                _ => {
                    return Err(schema_invalid(
                        "assessor must equal the authenticated principal of the request",
                    ))
                }
            }
            let mut seen = BTreeSet::new();
            for e in payload["evidence_refs"].as_array().into_iter().flatten() {
                if !seen.insert(e.as_str().unwrap_or("")) {
                    return Err(schema_invalid("evidence_refs repeats an evidence id"));
                }
            }
            if payload["gate"] == "prove_reliability" && payload["proposer"].is_null() {
                return Err(schema_invalid(
                    "prove_reliability needs the proposer of the work it assesses (SOP 8.1)",
                ));
            }
            if payload["state"] != "not_applicable" && !payload["approval_ref"].is_null() {
                return Err(schema_invalid(
                    "approval_ref applies to a not_applicable assessment only",
                ));
            }
        }
        "disposition.recorded" => {
            if let Some(r) = payload["episode_ref"].as_str() {
                let case = r
                    .strip_prefix("episode:")
                    .and_then(|rest| rest.split(':').next());
                if case != Some(ctx.case_id) {
                    return Err(schema_invalid("episode_ref names another case"));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// The payload without its `approval_ref`: what an approval binds
/// (`record_digest`, README section 6 "Approvals").
fn record_digest(payload: &Value) -> Option<String> {
    let mut p = payload.clone();
    p.as_object_mut()?.remove("approval_ref");
    digest_of(&p)
}

/// The approval the record rests on: an allow-approval of this case, bound to
/// the canonical operation built from this very record, unexpired and unused.
fn require_approval(
    ctx: &AppendContext,
    operation: &str,
    mut params: Value,
    payload: &Value,
) -> Result<String, Value> {
    let denied = |why: &str| Err(refuse("AUTHORITY_DENIED", why, None));
    let Some(named) = payload["approval_ref"].as_str() else {
        return denied("the record needs an approval_ref");
    };
    let Some(digest) = record_digest(payload) else {
        return denied("the payload is not canonicalisable, so no approval can bind it");
    };
    params["record_digest"] = json!(digest);
    match bound_approval_named(ctx, operation, params, ctx.case_id, Some(named)) {
        Some(reference) => Ok(reference),
        None => {
            denied("no allow-approval of this case is bound to this record, unexpired and unused")
        }
    }
}

/// Single use: the approval is consumed by the record that uses it. Written
/// before the record, so a crash wastes an approval and never lets one
/// authorise two records.
fn consume(ctx: &AppendContext, reference: &str, record_id: &str) -> Result<(), String> {
    append_line(
        &case_dir(ctx.root, ctx.case_id).join(APPROVALS),
        &json!({"approval_ref": reference, "consumed_by": record_id}),
    )
}

/// Run one of the four part-B kinds. `request` carries the v2 `contract`
/// marker; the caller holds the case lock.
pub(super) fn append(ctx: &AppendContext, request: &Value) -> Value {
    let kind = request["kind"].as_str().unwrap_or("");
    let payload = &request["payload"];
    let key = request["idempotency_key"].as_str().unwrap_or("");
    if let Err(refusal) = validate(request) {
        return refusal;
    }
    if canonical_actor(ctx.principal).is_err()
        || ctx.chain.iter().any(|c| canonical_actor(c).is_err())
    {
        return schema_invalid("the request carries no verified principal (or chain member)");
    }
    if let Err(refusal) = semantic_checks(ctx, kind, payload) {
        return refusal;
    }
    // STORE_INTEGRITY: every read the rules below need.
    if let Err(e) = integrity(ctx) {
        return store_refusal(e);
    }
    match is_field_case(ctx.root, ctx.case_id) {
        Ok(true) => {}
        Ok(false) => return schema_invalid("this kind applies to a Field Case (fde.field_case@1)"),
        Err(e) => return store_refusal(e),
    }
    let all = records(ctx);
    let document = if kind == "field_case.revised" {
        match load_document(ctx, &all) {
            Ok(d) => Some(d),
            Err(e) => return store_refusal(e),
        }
    } else {
        None
    };
    let Some(body_digest) = digest_of(&json!({"kind": kind, "payload": payload})) else {
        return schema_invalid("payload is not canonicalisable (float or unsupported value)");
    };
    // Idempotency: a replay returns the original and is never refused by a
    // later rule (D43).
    if let Some(done) = all.iter().find(|r| r["idempotency_key"] == key) {
        if done["body_digest"] == body_digest.as_str() {
            let mut original = done["response"].clone();
            original["replayed"] = json!(true);
            return original;
        }
        return refuse(
            "IDEMPOTENCY_CONFLICT",
            "idempotency_key was used with a different body",
            None,
        );
    }
    // CASE_CLOSED is final for every kind of this family; a suspended case still
    // accepts revisions, strategies and gates, and a disposition only when it
    // is the case-level `continue` (lifts) or a `close-*` (closes).
    let state = lifecycle(&all);
    match state {
        Lifecycle::Closed => {
            return refuse(
                "CASE_CLOSED",
                "the case has a close disposition; closed is final",
                None,
            )
        }
        Lifecycle::Suspended if kind == "disposition.recorded" => {
            let d = payload["disposition"].as_str().unwrap_or("");
            let lifts_or_closes =
                payload["level"] == "case" && (d == "continue" || d.starts_with("close-"));
            if !lifts_or_closes {
                return refuse("CASE_SUSPENDED", "the case is suspended", None);
            }
        }
        _ => {}
    }
    let outcome = match kind {
        "field_case.revised" => revised(ctx, payload, &all, document),
        "strategy.recorded" => strategy(payload, &all),
        "gate.assessed" => gate(ctx, payload),
        _ => disposition(ctx, payload),
    };
    let plan = match outcome {
        Ok(p) => p,
        Err(refusal) => return refusal,
    };
    // Write: the approval's consumption, the record, then the projection.
    let record_id = format!("rec-{:04}", all.len() + 1);
    if let Some(reference) = &plan.approval {
        if let Err(e) = consume(ctx, reference, &record_id) {
            return io_refusal(e);
        }
    }
    let mut resp = response(kind, &record_id, false);
    if let Some(n) = plan.revision {
        resp["revision"] = json!(n);
    }
    if kind == "disposition.recorded" {
        let mut after = all.clone();
        after.push(plan.record_fields.clone());
        resp["lifecycle"] = json!(match lifecycle(&after) {
            Lifecycle::Open => "open",
            Lifecycle::Suspended => "suspended",
            Lifecycle::Closed => "closed",
        });
    }
    let mut record = json!({
        "kind": kind, "idempotency_key": key, "body_digest": body_digest,
        "principal": ctx.principal, "chain": ctx.chain, "payload": payload,
        "response": resp.clone(),
    });
    for (k, v) in plan.record_fields.as_object().into_iter().flatten() {
        record[k] = v.clone();
    }
    if let Some(n) = plan.revision {
        record["revision"] = json!(n);
    }
    let stored = match next_record(ctx, &all, record) {
        Ok(s) => s,
        Err(e) => return io_refusal(e),
    };
    if let Some(mut next_doc) = plan.document {
        // The record is durable; the projection follows. A crash here leaves the
        // file behind its ledger and the next read replays the record.
        next_doc["revision"] = json!(plan.revision.unwrap_or(0));
        if let Err(e) = write_document(ctx, &next_doc) {
            return io_refusal(e);
        }
    }
    // The stored record id is the one predicted above (same lock, same ledger).
    debug_assert_eq!(stored["record_id"], record_id);
    resp
}

/// What a kind decided: the response members, the record members and the
/// projection to write.
struct Plan {
    revision: Option<u64>,
    approval: Option<String>,
    document: Option<Value>,
    /// Top-level members stored on the record (`level` and `disposition`, which
    /// the lifecycle function reads; `gate` and `state`).
    record_fields: Value,
}

fn revised(
    ctx: &AppendContext,
    payload: &Value,
    all: &[Value],
    document: Option<Value>,
) -> Result<Plan, Value> {
    let mut document = document.ok_or_else(|| schema_invalid("no document"))?;
    let current = ledger_revision(all);
    let reserved: BTreeSet<&str> = payload["changes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| c["path"].as_str())
        .filter(|p| RESERVED_PATHS.contains(p))
        .collect();
    let frozen = all.iter().any(|r| r["kind"] == "episode.frozen");
    let boundary = frozen && !reserved.is_empty();
    // RESERVED_DECISION_REFUSED: a boundary path after the first freeze is the
    // reserved operation revise_frozen_boundary_field; no approval, no change.
    if boundary && payload["approval_ref"].is_null() {
        return Err(refuse(
            "RESERVED_DECISION_REFUSED",
            "a boundary field is read-only after the first freeze; revising it needs an approval_ref",
            None,
        ));
    }
    if !boundary && !payload["approval_ref"].is_null() {
        return Err(schema_invalid(
            "approval_ref applies to a boundary path revised after the first freeze only",
        ));
    }
    let mut approval = None;
    if boundary {
        let params = json!({
            "case_id": ctx.case_id,
            "paths": reserved.iter().collect::<Vec<_>>(),
            "base_revision": payload["base_revision"],
        });
        approval = Some(require_approval(
            ctx,
            "revise_frozen_boundary_field",
            params,
            payload,
        )?);
    }
    if payload["base_revision"].as_u64() != Some(current) {
        return Err(refuse(
            "REVISION_CONFLICT",
            &format!("base_revision is not the current revision ({current}); read case.field.get and retry"),
            None,
        ));
    }
    let next = current + 1;
    apply(&mut document, &payload["changes"], next);
    document_is_valid(&document)
        .map_err(|e| schema_invalid(&format!("the revised document is invalid: {e}")))?;
    Ok(Plan {
        revision: Some(next),
        approval,
        document: Some(document),
        record_fields: json!({}),
    })
}

fn strategy(payload: &Value, all: &[Value]) -> Result<Plan, Value> {
    let latest = all
        .iter()
        .filter(|r| r["kind"] == "strategy.recorded")
        .filter_map(|r| r["revision"].as_u64().map(|n| (n, r["record_id"].clone())))
        .max_by_key(|(n, _)| *n);
    let current = latest.as_ref().map_or(0, |(n, _)| *n);
    let revision = payload["revision"].as_u64().unwrap_or(0);
    let conflict = |why: String| Err(refuse("REVISION_CONFLICT", &why, None));
    if revision != current + 1 {
        return conflict(format!(
            "strategy revision {revision} does not follow the current revision {current}"
        ));
    }
    if let Some((_, id)) = &latest {
        if payload["supersedes"] != *id {
            return conflict("supersedes is not the latest strategy record".into());
        }
    }
    Ok(Plan {
        revision: Some(revision),
        approval: None,
        document: None,
        record_fields: json!({}),
    })
}

fn gate(ctx: &AppendContext, payload: &Value) -> Result<Plan, Value> {
    let gate = payload["gate"].as_str().unwrap_or("");
    // I-2 on the delegation chain: whoever the request speaks for, the assessor
    // may not be the proposer of the work (or stand behind it).
    if gate == "prove_reliability" {
        let proposer = payload["proposer"]
            .as_str()
            .and_then(|p| canonical_actor(p).ok());
        let standing: BTreeSet<String> = ctx
            .chain
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(ctx.principal))
            .filter_map(|c| canonical_actor(c).ok())
            .collect();
        if proposer.is_none_or(|p| standing.contains(&p)) {
            return Err(refuse(
                "SELF_SETTLEMENT_REFUSED",
                "the assessor of prove_reliability is, or acts for, the proposer of the work",
                Some("I-2"),
            ));
        }
    }
    let mut approval = None;
    if payload["state"] == "not_applicable" {
        let hard = HARD_GATES.contains(&gate);
        if payload["approval_ref"].is_null() {
            return Err(if hard {
                refuse(
                    "HARD_GATE_NOT_WAIVABLE",
                    "a hard gate cannot be not_applicable without an approval (SOP 8.1b)",
                    None,
                )
            } else {
                refuse(
                    "AUTHORITY_DENIED",
                    "not_applicable needs an authorized rationale (an approval)",
                    None,
                )
            });
        }
        let operation = if hard {
            "set_hard_gate_not_applicable"
        } else {
            "set_gate_not_applicable"
        };
        let params = json!({"case_id": ctx.case_id, "gate": gate, "state": "not_applicable"});
        approval = Some(require_approval(ctx, operation, params, payload)?);
    }
    Ok(Plan {
        revision: None,
        approval,
        document: None,
        record_fields: json!({"gate": gate, "state": payload["state"]}),
    })
}

fn disposition(ctx: &AppendContext, payload: &Value) -> Result<Plan, Value> {
    let level = payload["level"].as_str().unwrap_or("");
    let d = payload["disposition"].as_str().unwrap_or("");
    let operation = if level == "episode" {
        "set_episode_disposition"
    } else if d.starts_with("close-") {
        "set_case_close_disposition"
    } else {
        "set_case_disposition"
    };
    let params = json!({
        "level": level, "disposition": d, "case_id": ctx.case_id,
        "episode_ref": payload.get("episode_ref").cloned().unwrap_or(Value::Null),
    });
    let approval = require_approval(ctx, operation, params, payload)?;
    let mut fields = json!({"level": level, "disposition": d});
    if let Some(r) = payload.get("episode_ref") {
        fields["episode_ref"] = r.clone();
    }
    Ok(Plan {
        revision: None,
        approval: Some(approval),
        document: None,
        record_fields: fields,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disposition_record(level: &str, d: &str) -> Value {
        json!({"kind": "disposition.recorded", "level": level, "disposition": d})
    }

    /// Golden table of the lifecycle function (README section 6): `closed` when
    /// a case-level `close-*` exists, final; `suspended` when the latest
    /// case-level disposition is `suspend`; otherwise `open`; episode-level
    /// dispositions never change it.
    #[test]
    fn lifecycle_golden() {
        use Lifecycle::{Closed, Open, Suspended};
        let r = disposition_record;
        let cases: Vec<(&str, Vec<Value>, Lifecycle)> = vec![
            ("empty ledger", vec![], Open),
            (
                "no disposition",
                vec![json!({"kind": "episode.frozen"})],
                Open,
            ),
            ("suspend", vec![r("case", "suspend")], Suspended),
            (
                "suspend, continue",
                vec![r("case", "suspend"), r("case", "continue")],
                Open,
            ),
            (
                "suspend, continue, suspend",
                vec![
                    r("case", "suspend"),
                    r("case", "continue"),
                    r("case", "suspend"),
                ],
                Suspended,
            ),
            (
                "close-satisfied",
                vec![r("case", "close-satisfied")],
                Closed,
            ),
            ("close-failed", vec![r("case", "close-failed")], Closed),
            (
                "suspend, close-failed",
                vec![r("case", "suspend"), r("case", "close-failed")],
                Closed,
            ),
            (
                "close is final: continue after it",
                vec![r("case", "close-satisfied"), r("case", "continue")],
                Closed,
            ),
            (
                "close is final: suspend after it",
                vec![r("case", "close-failed"), r("case", "suspend")],
                Closed,
            ),
            ("revise", vec![r("case", "revise")], Open),
            ("escalate", vec![r("case", "escalate")], Open),
            (
                "episode-level close never closes",
                vec![r("episode", "close-satisfied")],
                Open,
            ),
            (
                "episode-level suspend never suspends",
                vec![r("episode", "suspend")],
                Open,
            ),
            (
                "episode-level continue does not lift a suspension",
                vec![r("case", "suspend"), r("episode", "continue")],
                Suspended,
            ),
            (
                "episode-level close does not end a suspension",
                vec![r("case", "suspend"), r("episode", "close-failed")],
                Suspended,
            ),
            (
                "other kinds are ignored",
                vec![
                    r("case", "suspend"),
                    json!({"kind": "gate.assessed", "level": "case", "disposition": "continue"}),
                ],
                Suspended,
            ),
        ];
        for (name, ledger, want) in cases {
            assert_eq!(lifecycle(&ledger), want, "{name}");
        }
    }

    #[test]
    fn reserved_paths_are_fields_of_the_document() {
        for p in RESERVED_PATHS {
            assert!(is_field_path(p), "{p}");
        }
        assert!(!is_field_path("framing"));
        assert!(!is_field_path("framing.nope"));
    }

    #[test]
    fn ledger_revision_follows_the_latest_revised_record() {
        assert_eq!(ledger_revision(&[]), 1);
        let r = |n| json!({"kind": "field_case.revised", "revision": n});
        assert_eq!(ledger_revision(&[r(2), r(3)]), 3);
    }
}
