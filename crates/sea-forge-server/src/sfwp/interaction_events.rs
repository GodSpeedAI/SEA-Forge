//! Semantic exhaust for `case.record.append` (WP-14, E §5): one CEP
//! `execution_trace` envelope per case ledger record and per authority
//! decision, carrying `extensions["godspeed.interaction_event"]` (schema
//! vendored by digest in `contracts/cep/`).
//!
//! Durability. The event is *derived* from the stored record, never from the
//! request, so it can always be rebuilt. [`enqueue_record`] writes the outbox
//! row right after the ledger line in the same critical section; if the
//! process dies between the two, [`reconcile_events`] (run by `recover` and by
//! every delivery pass) adds the missing row. A row is keyed by
//! `ie:<case_id>:<record_id>`: enqueueing twice, delivering twice, or replaying
//! a request never produces a second event. Delivery failure leaves the row
//! `pending`; it never reaches the case write.
//!
//! Stage limits, stated rather than hidden:
//! - Unsigned. E §5 makes `emitter.signature` required from WP-33 and absent
//!   before. The schema has no "unsigned" marker, so the `emitter` object is
//!   omitted; the outbox row records `"signature": "absent_before_WP-33"`.
//! - The default sink ([`SpoolSink`]) appends envelopes to
//!   `<root>/interaction-events/<case_id>.jsonl` for RealityTrace ingest
//!   (WP-16). No network transport exists in this crate.
//! - `semantic_ref` comes from the WP-13 translation map
//!   (`.sea/interaction/fde-sop/source-translation-map.md`) and only for the
//!   `godspeed-fde-sop` world. When the frozen episode pins another world, or
//!   no row maps, nothing is invented: the outbox records an `unmapped` gap
//!   row and no event is emitted (the schema has no unknown form for
//!   `semantic_ref`).

use super::case_records::{
    append_line, case_dir, decisions, now_rfc3339, read_lines, records, AppendContext,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The extension schema, vendored from the CEP repository by digest.
pub const EXTENSION_SCHEMA: &str =
    include_str!("../../contracts/cep/godspeed.interaction_event.v1.json");

const OUTBOX: &str = "field-interaction-outbox.jsonl";
const SOP_WORLD: &str = "godspeed-fde-sop";
const SOURCE_LEDGER: &str = "sea-forge.case-ledger";
const SOURCE_AUTHORITY: &str = "sea-forge.authority";

// ---------------------------------------------------------------------------
// Translation map rows used by this path (WP-13 `source-translation-map.md`)
// ---------------------------------------------------------------------------

/// One map row: the SOP clause, the declared concept it resolves to, and the
/// operation name the event reports.
struct MapRow {
    clause: &'static str,
    semantic_ref: &'static str,
    operation: &'static str,
}

/// Only concepts declared in the world are addressable: `Episode` (rows 2.1b,
/// 7.2d), `evidence::Settlement` (row 7.5a), policy `no_self_settlement` (row
/// 3.2b). The operations `nav_aim` and `nav_claim` are application-contract
/// elements, not concepts, so they are reported in `operation`, not in
/// `semantic_ref`.
fn map_row(what: &str) -> Option<MapRow> {
    match what {
        "episode.frozen" => Some(MapRow {
            clause: "7.2d",
            semantic_ref: "godspeed-fde-sop::Episode",
            operation: "nav_aim",
        }),
        "episode.settled" => Some(MapRow {
            clause: "7.5a",
            semantic_ref: "godspeed-fde-sop.evidence::Settlement",
            operation: "nav_claim",
        }),
        "decision.no_self_settlement" => Some(MapRow {
            clause: "3.2b",
            semantic_ref: "godspeed-fde-sop::no_self_settlement",
            operation: "nav_claim",
        }),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// declaration_key = concept:<uuidv5(DNS namespace, "namespace::name")>
// (DomainForge `ConceptId::from_concept`; the receiver recomputes it)
// ---------------------------------------------------------------------------

fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [
        0x6745_2301,
        0xEFCD_AB89,
        0x98BA_DCFE,
        0x1032_5476,
        0xC3D2_E1F0,
    ];
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 80];
        for (i, b) in chunk.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = h;
        for (i, wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5A82_7999),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };
            let t = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = t;
        }
        for (slot, v) in h.iter_mut().zip([a, b, c, d, e]) {
            *slot = slot.wrapping_add(v);
        }
    }
    let mut out = [0u8; 20];
    for (i, v) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

/// RFC 4122 UUIDv5 under the DNS namespace, lowercase hyphenated.
pub fn uuid5_dns(name: &str) -> String {
    const DNS: [u8; 16] = [
        0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30,
        0xc8,
    ];
    let mut data = DNS.to_vec();
    data.extend_from_slice(name.as_bytes());
    let digest = sha1(&data);
    let mut b = [0u8; 16];
    b.copy_from_slice(&digest[..16]);
    b[6] = (b[6] & 0x0f) | 0x50;
    b[8] = (b[8] & 0x3f) | 0x80;
    let hex: String = b.iter().map(|x| format!("{x:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// `concept:<uuid>` for a `namespace::name` semantic reference.
pub fn declaration_key(semantic_ref: &str) -> String {
    format!("concept:{}", uuid5_dns(semantic_ref))
}

// ---------------------------------------------------------------------------
// Event construction
// ---------------------------------------------------------------------------

fn world_name(world_ref: &str) -> Option<&str> {
    world_ref
        .strip_prefix("world:")?
        .split_once('@')
        .map(|(n, _)| n)
}

/// The facts of an episode that every event about it repeats, read from its
/// freeze record (the owner's untouched copy of the frozen reference).
struct Episode {
    world_ref: String,
    kind: String,
    provenance: String,
    case_id: String,
    plan_item_id: String,
    episode_ref: String,
    expectation_refs: Vec<String>,
}

fn episode_of(all: &[Value], episode_key: &Value) -> Option<Episode> {
    let freeze = all
        .iter()
        .find(|r| r["kind"] == "episode.frozen" && r["episode_key"] == *episode_key)?;
    let body = &freeze["payload"]["reference"]["body"];
    let refs = &freeze["payload"]["realitytrace_refs"];
    let mut expectation_refs: Vec<String> = Vec::new();
    for field in ["question_ids", "threshold_ids"] {
        for id in refs[field].as_array().into_iter().flatten() {
            if let Some(id) = id.as_str() {
                if !expectation_refs.iter().any(|e| e == id) {
                    expectation_refs.push(id.to_string());
                }
            }
        }
    }
    Some(Episode {
        world_ref: body["world_ref"].as_str()?.to_string(),
        kind: freeze["episode_kind"].as_str()?.to_string(),
        provenance: body["executor_provenance"].as_str()?.to_string(),
        case_id: episode_key["case_id"].as_str()?.to_string(),
        plan_item_id: episode_key["plan_item_id"].as_str()?.to_string(),
        episode_ref: freeze["payload"]["episode_ref"].as_str()?.to_string(),
        expectation_refs,
    })
}

/// What is known about the thing being reported.
struct Source<'a> {
    what: &'a str,
    system: &'static str,
    record_id: &'a str,
    sequence: u64,
    recorded_at: &'a str,
    decision_id: &'a str,
    causation_id: Option<&'a str>,
    outcome: &'static str,
    effect_refs: Vec<String>,
    settlement_ref: Option<String>,
}

/// Build the envelope, or the reason no schema-valid event can be built.
fn build(ctx: &AppendContext, ep: &Episode, src: &Source) -> Result<Value, String> {
    let Some(row) = map_row(src.what) else {
        return Err(format!("no translation-map row for {}", src.what));
    };
    if world_name(&ep.world_ref) != Some(SOP_WORLD) {
        return Err(format!(
            "episode pins {}, not the {SOP_WORLD} world the translation map covers; \
             no semantic_ref is invented",
            ep.world_ref
        ));
    }
    let event_id = event_id(ctx.case_id, src.record_id);
    let context = json!({"case_id": ep.case_id, "plan_item_id": ep.plan_item_id,
                             "episode_ref": ep.episode_ref});
    let correlation = json!({"case_id": ep.case_id, "episode_ref": ep.episode_ref});
    // Structured only, never parsed from a string (E §2); no assignment exists
    // on this path until WP-20.
    let mut ext = json!({
        "world_ref": ep.world_ref,
        "semantic_ref": row.semantic_ref,
        "declaration_key": declaration_key(row.semantic_ref),
        "actor": {"principal": ctx.principal, "role": ctx.role},
        "operation": row.operation,
        "resource": format!("cases/{}/episodes/{}", ep.case_id, ep.plan_item_id),
        "context": context,
        "correlation": correlation,
        "authority": {"decision_id": src.decision_id},
        "outcome": {"status": src.outcome, "effect_refs": src.effect_refs},
        "source": {"system": src.system, "record_id": src.record_id,
                   "sequence": src.sequence, "recorded_at": src.recorded_at},
        "executor_provenance": ep.provenance,
        "kind": ep.kind,
    });
    if let Some(cause) = src.causation_id {
        ext["causation_id"] = json!(cause);
    }
    if let Some(settlement) = &src.settlement_ref {
        ext["settlement_ref"] = json!(settlement);
    }
    if ep.kind == "evaluated" {
        ext["expectation_refs"] = json!(ep.expectation_refs);
    }
    let lineage = if src.decision_id == "not_required" {
        format!("{}:{}/{}", src.system, ctx.case_id, src.record_id)
    } else {
        src.decision_id.to_string()
    };
    Ok(json!({
        "envelope_id": format!("env-{event_id}"),
        "cep_version": "1.0.0",
        "envelope_version": "1.0",
        "envelope_kind": "execution_trace",
        "created_at": src.recorded_at,
        "created_by": "sea-forge-server",
        "scope": {"evaluation_context": "godspeed-execution_trace", "world_ref": ep.world_ref},
        "boundary_record": {
            "scope": "case.record.append interaction event",
            "included_sections": ["observations", "transformations"],
            "excluded_sections": [],
            "known_omissions": ["emitter.signature (absent before WP-33)"],
            "unknowns": [],
            "redactions": [],
            "compression_notes": [],
            "out_of_scope_entities": [],
            "classification": "internal",
            "limitations": [format!("semantic_ref from translation-map row {}", row.clause)]
        },
        "completeness_status": "partial",
        "omission_status": "marked",
        "provenance_refs": [format!("prov-{event_id}")],
        "validation_status": "valid",
        "lineage_refs": [lineage],
        "provenance": [{
            "provenance_id": format!("prov-{event_id}"),
            "source_system_refs": [src.system],
            "producer_refs": ["sea-forge-server"],
            "production_method": "derived_from_source_record",
            "created_at": src.recorded_at
        }],
        "transformations": [{
            "transformation_id": format!("tr-{event_id}"),
            "transformation_type": "capability_invocation"
        }],
        "observations": [{"observation_id": format!("obs-{event_id}"),
                          "method": "source-record-resolution"}],
        "extensions": {
            "cep.profile": {"profile_id": "godspeed.execution_trace", "profile_version": "1.0.0"},
            "godspeed.execution_trace": {
                "operation_id": row.operation,
                "authority_decision_ref": src.decision_id,
                "correlation_id": ep.case_id
            },
            "godspeed.interaction_event": ext
        }
    }))
}

fn event_id(case_id: &str, record_id: &str) -> String {
    format!("ie:{case_id}:{record_id}")
}

// ---------------------------------------------------------------------------
// Outbox
// ---------------------------------------------------------------------------

fn outbox_path(ctx: &AppendContext) -> PathBuf {
    case_dir(ctx.root, ctx.case_id).join(OUTBOX)
}

fn known(ctx: &AppendContext, id: &str) -> bool {
    read_lines(&outbox_path(ctx))
        .iter()
        .any(|l| l["event_id"] == id)
}

fn enqueue(
    ctx: &AppendContext,
    id: String,
    source: Value,
    built: Result<Value, String>,
) -> Result<(), String> {
    if known(ctx, &id) {
        return Ok(());
    }
    let row = match built {
        Ok(envelope) => json!({"event_id": id, "state": "pending", "source": source,
                               "signature": "absent_before_WP-33", "envelope": envelope}),
        Err(reason) => json!({"event_id": id, "state": "unmapped", "source": source,
                              "reason": reason}),
    };
    append_line(&outbox_path(ctx), &row)
}

/// Owe the event for one ledger record. Idempotent.
pub(super) fn enqueue_record(ctx: &AppendContext, record: &Value) -> Result<(), String> {
    let (Some(record_id), Some(kind)) = (record["record_id"].as_str(), record["kind"].as_str())
    else {
        return Err("record has no id or kind".into());
    };
    let all = records(ctx);
    let source = json!({"system": SOURCE_LEDGER, "record_id": record_id});
    let id = event_id(ctx.case_id, record_id);
    let Some(ep) = episode_of(&all, &record["episode_key"]) else {
        return enqueue(ctx, id, source, Err("freeze record not found".into()));
    };
    let decision_id = record["decision_id"].as_str().unwrap_or("not_required");
    let src = Source {
        what: kind,
        system: SOURCE_LEDGER,
        record_id,
        sequence: record["seq"].as_u64().unwrap_or(0),
        recorded_at: record["recorded_at"].as_str().unwrap_or(""),
        decision_id,
        causation_id: (kind == "episode.settled").then_some(decision_id),
        outcome: "succeeded",
        effect_refs: vec![format!("case-record:{}/{record_id}", ctx.case_id)],
        settlement_ref: record["payload"]["settlement_ref"]
            .as_str()
            .map(String::from),
    };
    let built = build(ctx, &ep, &src);
    enqueue(ctx, id, source, built)
}

/// Owe the event for one authority decision. Idempotent.
pub(super) fn enqueue_decision(ctx: &AppendContext, decision: &Value) -> Result<(), String> {
    let Some(decision_id) = decision["decision_id"].as_str() else {
        return Err("decision has no id".into());
    };
    let all = records(ctx);
    let source = json!({"system": SOURCE_AUTHORITY, "record_id": decision_id});
    let id = event_id(ctx.case_id, decision_id);
    let Some(ep) = episode_of(&all, &decision["episode_key"]) else {
        return enqueue(ctx, id, source, Err("freeze record not found".into()));
    };
    let src = Source {
        what: "decision.no_self_settlement",
        system: SOURCE_AUTHORITY,
        record_id: decision_id,
        sequence: decision["seq"].as_u64().unwrap_or(0),
        recorded_at: decision["recorded_at"].as_str().unwrap_or(""),
        decision_id,
        causation_id: None,
        outcome: if decision["decision"] == "allow" {
            "succeeded"
        } else {
            "denied"
        },
        effect_refs: vec![],
        settlement_ref: None,
    };
    let built = build(ctx, &ep, &src);
    enqueue(ctx, id, source, built)
}

/// Add the outbox row of every ledger record and decision that lacks one (a
/// crash between the ledger write and the enqueue). Idempotent.
pub fn reconcile_events(ctx: &AppendContext) -> usize {
    let mut added = 0;
    for record in records(ctx) {
        let id = event_id(ctx.case_id, record["record_id"].as_str().unwrap_or(""));
        if !known(ctx, &id) && enqueue_record(ctx, &record).is_ok() {
            added += 1;
        }
    }
    for decision in decisions(ctx) {
        let id = event_id(ctx.case_id, decision["decision_id"].as_str().unwrap_or(""));
        if !known(ctx, &id) && enqueue_decision(ctx, &decision).is_ok() {
            added += 1;
        }
    }
    added
}

// ---------------------------------------------------------------------------
// Delivery
// ---------------------------------------------------------------------------

/// Where envelopes go. Delivery must be idempotent on `event_id`: the outbox
/// retries until it hears success, so a sink can see an event twice.
pub trait InteractionSink {
    fn deliver(&self, event_id: &str, envelope: &Value) -> Result<(), String>;
}

/// Appends envelopes to `<root>/interaction-events/<case_id>.jsonl`, skipping
/// an `envelope_id` already present.
pub struct SpoolSink {
    dir: PathBuf,
}

impl SpoolSink {
    pub fn new(root: &Path) -> Self {
        Self {
            dir: root.join("interaction-events"),
        }
    }
}

impl InteractionSink for SpoolSink {
    fn deliver(&self, _event_id: &str, envelope: &Value) -> Result<(), String> {
        let case = envelope["extensions"]["godspeed.interaction_event"]["correlation"]["case_id"]
            .as_str()
            .ok_or("envelope has no correlation.case_id")?;
        fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        let path = self.dir.join(format!("{case}.jsonl"));
        if read_lines(&path)
            .iter()
            .any(|l| l["envelope_id"] == envelope["envelope_id"])
        {
            return Ok(());
        }
        append_line(&path, envelope)
    }
}

/// Deliver every pending row (after reconciling). A failing sink leaves the
/// row pending for the next pass; it is never an error to the caller.
/// Returns the number of rows newly marked delivered.
pub fn deliver_events(ctx: &AppendContext, sink: &dyn InteractionSink) -> usize {
    reconcile_events(ctx);
    let lines = read_lines(&outbox_path(ctx));
    let mut latest: BTreeMap<String, &Value> = BTreeMap::new();
    let mut pending: BTreeMap<String, &Value> = BTreeMap::new();
    for line in &lines {
        let Some(id) = line["event_id"].as_str() else {
            continue;
        };
        if line["state"] == "pending" {
            pending.entry(id.to_string()).or_insert(line);
        }
        latest.insert(id.to_string(), line);
    }
    let mut sent = 0;
    for (id, last) in latest {
        if last["state"] != "pending" {
            continue;
        }
        let Some(row) = pending.get(&id) else {
            continue;
        };
        if sink.deliver(&id, &row["envelope"]).is_ok()
            && append_line(
                &outbox_path(ctx),
                &json!({"event_id": id, "state": "delivered", "delivered_at": now_rfc3339()}),
            )
            .is_ok()
        {
            sent += 1;
        }
    }
    sent
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid5_matches_the_published_vector() {
        // RFC 4122 Appendix / Python: uuid5(NAMESPACE_DNS, "python.org").
        assert_eq!(
            uuid5_dns("python.org"),
            "886313e1-3b8a-5372-9b90-0c9aee199e5d"
        );
    }

    #[test]
    fn declaration_keys_match_domainforge_concept_ids() {
        // Captured in .sea/interaction/fde-sop/validation/domainforge-output.txt.
        assert_eq!(
            declaration_key("godspeed-fde-sop.evidence::Settlement"),
            "concept:11fc582b-b054-5386-b245-5a358e3c9327"
        );
        assert_eq!(
            declaration_key("godspeed-fde-sop.outcome::Settlement"),
            "concept:01d8c175-5650-5bb9-811a-a44720e16a94"
        );
    }
}
