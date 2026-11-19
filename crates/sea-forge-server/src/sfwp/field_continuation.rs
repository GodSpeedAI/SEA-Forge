//! The read verbs of a Field Case: `case.field.get` and `case.continuation`
//! (WP-21 part B, `contracts/fde/README.md` section 9, SOP 10.1).
//!
//! The continuation is computed on every read from the ledger and the case
//! store and is never stored. A section whose source this server cannot read
//! is `unknown`, never an empty "none": RealityTrace and Cognate are not read
//! from here yet, so `disputed_claims`, `latest_observations` and `work` are
//! `unknown` with empty lists. The caller holds the case's keyed lock (the read
//! may bring a document left behind by a crash up to its ledger).

use super::case_records::{
    fde_config, freeze_state_in, integrity_in, latest_intents_in, lifecycle, now_rfc3339,
    parse_time, read_lines, refuse, with_store_ctx, AppendContext, FreezeStatus, Lifecycle,
    APPROVALS, RECORDS,
};
use super::field_case_kinds::{is_field_case_in, load_document_in};
use sea_forge_core::errors::ForgeError;
use serde_json::{json, Value};
use std::path::Path;

/// The nine gates in SOP 8.1 order.
pub const GATES: [&str; 9] = [
    "choose_problem",
    "discover_reality",
    "allocate_authority",
    "prove_value",
    "prove_reliability",
    "integrate_govern",
    "earn_trust",
    "measure",
    "productize",
];

const UNPINNED: &str =
    "world:unpinned@sha256:0000000000000000000000000000000000000000000000000000000000000000";

fn lifecycle_name(l: Lifecycle) -> &'static str {
    match l {
        Lifecycle::Open => "open",
        Lifecycle::Suspended => "suspended",
        Lifecycle::Closed => "closed",
    }
}

/// Run `f` for a Field Case the caller is entitled to read. `Err` is the SFWP
/// error for a case that does not exist; `Ok(Err(frame))` is a contract
/// refusal (`NOT_A_FIELD_CASE`, `STORE_INTEGRITY`).
fn with_case<T>(
    root: &Path,
    case_id: &str,
    f: impl FnOnce(&AppendContext, &Path, Value, Vec<Value>) -> T,
) -> Result<Result<T, Value>, ForgeError> {
    if !sea_forge_core::path::valid_id_segment(case_id, 128) {
        return Err(ForgeError::Input(format!("unknown case {case_id}")));
    }
    let dir = sea_forge_case_runner::case_ops::case_snapshot_dir(root, case_id)?;
    if !dir.join("case.json").exists() {
        return Err(ForgeError::Input(format!("unknown case {case_id}")));
    }
    with_store_ctx(root, case_id, |ctx| {
        match is_field_case_in(&dir) {
            Ok(true) => {}
            Ok(false) => {
                return Ok(Err(refuse(
                    "NOT_A_FIELD_CASE",
                    "the case template is not fde.field_case@1",
                    None,
                )))
            }
            Err(e) => return Ok(Err(store(e))),
        }
        if let Err(e) = integrity_in(&dir) {
            return Ok(Err(store(e)));
        }
        let all = read_lines(&dir.join(RECORDS));
        match load_document_in(case_id, &dir, &all) {
            Ok(document) => Ok(Ok(f(ctx, &dir, document, all))),
            Err(e) => Ok(Err(store(e))),
        }
    })
}

fn store(message: String) -> Value {
    refuse("STORE_INTEGRITY", &message, None)
}

/// `case.field.get`: `{case_id, revision, document}`.
pub fn field_get(root: &Path, case_id: &str) -> Result<Value, ForgeError> {
    Ok(
        with_case(root, case_id, |_, _, document, _| {
            json!({"case_id": case_id, "revision": document["revision"], "document": document})
        })?
        .unwrap_or_else(|refusal| refusal),
    )
}

/// `case.continuation`: `{case_id, continuation}`. `viewer` is the
/// authenticated principal of the caller and `pinned_world` the world in the
/// WP-18 pin record (`None`: no pin source configured).
pub fn continuation(
    root: &Path,
    case_id: &str,
    viewer: &str,
    pinned_world: Option<&str>,
) -> Result<Value, ForgeError> {
    Ok(with_case(root, case_id, |ctx, dir, document, all| {
        json!({"case_id": case_id,
               "continuation": compute(ctx, dir, &document, &all, viewer, pinned_world)})
    })?
    .unwrap_or_else(|refusal| refusal))
}

fn field<'a>(document: &'a Value, group: &str, name: &str) -> &'a Value {
    &document["fields"][group][name]
}

/// A field's value as a string when it is known and a string.
fn text_of(v: &Value) -> Option<String> {
    (v["status"] != "unknown")
        .then(|| v["value"].as_str().map(String::from))
        .flatten()
}

fn is_world_ref(s: &str) -> bool {
    s.strip_prefix("world:")
        .and_then(|rest| rest.split_once("@sha256:"))
        .is_some_and(|(name, hex)| {
            !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "._-".contains(c))
                && hex.len() == 64
                && hex.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
        })
}

/// Gates whose latest assessment predates new evidence (a settlement recorded
/// after it): the gate must be reassessed. Nothing changes the gate itself;
/// the continuation only says so. Returns `(gate, assessed_by_record_id)`.
pub fn gates_to_reassess(all: &[Value]) -> Vec<(String, String)> {
    let last_evidence = all
        .iter()
        .filter(|r| r["kind"] == "episode.settled")
        .filter_map(|r| r["seq"].as_u64())
        .max();
    let Some(last_evidence) = last_evidence else {
        return Vec::new();
    };
    GATES
        .iter()
        .filter_map(|gate| {
            let latest = all
                .iter()
                .rfind(|r| r["kind"] == "gate.assessed" && r["gate"] == *gate)?;
            let stale = matches!(latest["state"].as_str(), Some("met") | Some("blocked"))
                && latest["seq"].as_u64().is_some_and(|s| s < last_evidence);
            stale.then(|| {
                (
                    (*gate).to_string(),
                    latest["record_id"].as_str().unwrap_or("").to_string(),
                )
            })
        })
        .collect()
}

fn gate_evidence(all: &[Value]) -> Vec<Value> {
    GATES
        .iter()
        .map(|gate| {
            match all
                .iter()
                .rfind(|r| r["kind"] == "gate.assessed" && r["gate"] == *gate)
            {
                Some(r) => json!({
                    "gate": gate, "state": r["state"],
                    "evidence_refs": r["payload"]["evidence_refs"],
                    "record_id": r["record_id"],
                }),
                None => json!({"gate": gate, "state": "unassessed", "evidence_refs": [], "record_id": null}),
            }
        })
        .collect()
}

fn episodes(ctx: &AppendContext, dir: &Path, all: &[Value], anchored: bool) -> Vec<Value> {
    let mut out = Vec::new();
    let mut seen: Vec<Value> = Vec::new();
    for r in all.iter().filter(|r| r["kind"] == "episode.frozen") {
        let key = r["episode_key"].clone();
        let (item, n) = (
            key["plan_item_id"].as_str().unwrap_or(""),
            key["n"].as_u64().unwrap_or(0),
        );
        let settled = all
            .iter()
            .find(|s| s["kind"] == "episode.settled" && s["episode_key"] == key);
        let state = if let Some(s) = settled {
            out.push(json!({
                "episode_key": key, "episode_ref": r["payload"]["episode_ref"],
                "kind": r["episode_kind"], "state": "settled",
                "result": s["payload"]["result"], "settlement_ref": s["payload"]["settlement_ref"],
            }));
            seen.push(key);
            continue;
        } else if matches!(
            freeze_state_in(ctx, dir, item, n, anchored),
            FreezeStatus::Complete
        ) {
            "frozen"
        } else {
            "pending"
        };
        out.push(json!({
            "episode_key": key, "episode_ref": r["payload"]["episode_ref"],
            "kind": r["episode_kind"], "state": state, "result": null, "settlement_ref": null,
        }));
        seen.push(key);
    }
    // A freeze whose record is not written yet (a prepared intent).
    for intent in latest_intents_in(dir) {
        if intent["state"] == "abandoned" || seen.contains(&intent["episode_key"]) {
            continue;
        }
        out.push(json!({
            "episode_key": intent["episode_key"],
            "episode_ref": intent["payload"]["episode_ref"],
            "kind": intent["episode_kind"], "state": "pending", "result": null, "settlement_ref": null,
        }));
    }
    out
}

fn authority(dir: &Path, case_lead: Option<&str>, viewer: &str) -> Value {
    let lines = read_lines(&dir.join(APPROVALS));
    let now = sea_forge_core::timestamp::Timestamp::now_utc();
    let who = |s: &str| super::case_records::canonical_actor(s).ok();
    let viewer_c = who(viewer);
    let is_lead = case_lead.and_then(who).is_some() && case_lead.and_then(who) == viewer_c;
    let mut pending = Vec::new();
    let mut decisions = Vec::new();
    for a in lines.iter().filter(|a| a["operation"].is_string()) {
        let live = parse_time(&a["expires_at"]).is_some_and(|t| t > now);
        if a["outcome"] == "pending" {
            let approver = a["approver"].as_str().and_then(who);
            let sees = is_lead || (approver.is_some() && approver == viewer_c);
            pending.push(json!({
                "pending_id": if sees { a["pending_id"].clone() } else { Value::Null },
                "operation": a["operation"], "class": a["class"].as_str().unwrap_or("P2"),
                "expires_at": a["expires_at"],
            }));
        } else if matches!(a["outcome"].as_str(), Some("allow") | Some("deny")) {
            let outcome = if a["outcome"] == "allow" && !live {
                "expired"
            } else {
                a["outcome"].as_str().unwrap_or("deny")
            };
            decisions.push(json!({
                "decision_id": a["approval_ref"], "operation": a["operation"],
                "outcome": outcome, "assurance": a["assurance"].as_str().unwrap_or("session"),
            }));
        }
    }
    json!({"pending_approvals": pending, "decisions": decisions, "a3_channel": "unknown"})
}

/// The next affordable action or the blocker, who can authorize it and what
/// would settle it (SOP 10.1 item 9). Pure over the ledger and the document.
pub fn next_move(document: &Value, all: &[Value], state: Lifecycle, episodes: &[Value]) -> Value {
    let authorizer = text_of(field(
        document,
        "observer_and_authority",
        "settlement_and_disposition_authority",
    ))
    .or_else(|| text_of(field(document, "observer_and_authority", "case_lead")))
    .unwrap_or_else(|| "role:operator".into());
    let action = |description: String, would_settle: Option<String>| {
        json!({"kind": "action", "description": description,
               "authorizer": authorizer, "would_settle": would_settle})
    };
    match state {
        Lifecycle::Closed => {
            return json!({"kind": "action",
                "description": "The case is closed; no further move is owed. Late settlement evidence is still accepted.",
                "authorizer": null, "would_settle": null})
        }
        Lifecycle::Suspended => {
            return json!({"kind": "blocker", "dimension": "authority",
                "description": "The case is suspended: only a case-level continue (lifts it) or a close disposition (ends it) is accepted.",
                "authorizer": authorizer, "would_settle": null})
        }
        Lifecycle::Open => {}
    }
    if let Some(e) = episodes.iter().find(|e| e["state"] == "pending") {
        return action(
            format!(
                "Complete the freeze of episode {} (record, RealityTrace records, anchor).",
                e["episode_key"]["plan_item_id"].as_str().unwrap_or("?")
            ),
            None,
        );
    }
    if let Some(e) = episodes.iter().find(|e| e["state"] == "frozen") {
        let r = e["episode_ref"].as_str().unwrap_or("?").to_string();
        return action(
            format!("Settle {r}: an independent verifier outside the executor's delegation chain settles it."),
            Some(format!("{r} settled S, F, U or C with the verifier's own observation")),
        );
    }
    let stated = field(document, "purpose_and_outcome", "stated_problem");
    let adopted = field(document, "purpose_and_outcome", "adopted_problem");
    if stated["status"] == "unknown" || adopted["status"] == "unknown" {
        return action("Record the stated and adopted problem.".into(), None);
    }
    if !all.iter().any(|r| r["kind"] == "strategy.recorded") {
        return action("Record the strategy (SOP 4.4).".into(), None);
    }
    if let Some((gate, by)) = gates_to_reassess(all).into_iter().next() {
        return action(
            format!("Reassess gate {gate}: evidence changed since {by}."),
            Some(format!("gate.assessed {gate} with the new evidence")),
        );
    }
    let gates = gate_evidence(all);
    if let Some(g) = gates.iter().find(|g| g["state"] == "unassessed") {
        return action(
            format!("Assess gate {}.", g["gate"].as_str().unwrap_or("?")),
            Some("gate.assessed with evidence refs".into()),
        );
    }
    action(
        "Record a case-level disposition: continue, revise, escalate, suspend or close.".into(),
        None,
    )
}

fn compute(
    ctx: &AppendContext,
    dir: &Path,
    document: &Value,
    all: &[Value],
    viewer: &str,
    pinned_world: Option<&str>,
) -> Value {
    let state = lifecycle(all);
    let anchored = fde_config(ctx.root)
        .map(|c| c["case_keeper_identity"].is_string())
        .unwrap_or(false);
    let eps = episodes(ctx, dir, all, anchored);
    let case_lead = text_of(field(document, "observer_and_authority", "case_lead"));
    let pinned = pinned_world.filter(|w| is_world_ref(w));
    let declared = text_of(field(
        document,
        "world_and_boundary",
        "world_ref_and_version",
    ))
    .filter(|w| is_world_ref(w));
    let (world_ref, world_known) = match (pinned, declared) {
        (Some(w), _) => (w.to_string(), true),
        // Not pinned: the declared reference is shown, but it is not the pin.
        (None, Some(w)) => (w, false),
        (None, None) => (UNPINNED.to_string(), false),
    };
    let known = |b: bool| if b { "known" } else { "unknown" };
    // Unmeasured is "unknown", never 0 (README section 9).
    let budgets: Vec<Value> = ["time", "attention", "money", "risk", "recovery"]
        .iter()
        .map(|c| json!({"currency": c, "remaining": "unknown"}))
        .collect();
    json!({
        "availability": {
            "case": "known", "pinned_world": known(world_known), "gate_evidence": "known",
            "episodes": "known", "authority": "known", "disputed_claims": "unknown",
            "latest_observations": "unknown", "work": "unknown", "next": "known",
        },
        "computed_from": {
            "ledger_head": all.last().map(|r| r["record_id"].clone()).unwrap_or(Value::Null),
            "realitytrace_snapshot": null, "cognate_ref": null, "computed_at": now_rfc3339(),
        },
        "case": {
            "case_id": ctx.case_id, "lifecycle": lifecycle_name(state),
            "owner": field(document, "purpose_and_outcome", "owner_sponsor"),
            "purpose": field(document, "purpose_and_outcome", "jtbd_outcome"),
            "current_decision": field(document, "continuity", "current_decision"),
        },
        "pinned_world": {
            "world_ref": world_ref,
            "checkpoint": field(document, "world_and_boundary", "checkpoint"),
        },
        "gate_evidence": gate_evidence(all),
        "episodes": eps,
        "authority": authority(dir, case_lead.as_deref(), viewer),
        "disputed_claims": [],
        "latest_observations": {"observations": [], "unknown_evidence": [], "contradictory_evidence": []},
        "work": {
            "active_assignments": [], "artifact_locations": [], "verified_commands": [],
            "remaining_budgets": budgets,
        },
        "next": next_move(document, all, state, &eps),
    })
}
