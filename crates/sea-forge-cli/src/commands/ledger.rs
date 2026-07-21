use crate::LedgerAction;
use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::{CasePlan, TraceEvent, TraceKind};
use sea_forge_ledger::types::LedgerStream;
use std::fs;
use std::path::Path;

pub fn execute(action: LedgerAction, root: &Path) -> Result<u8, ForgeError> {
    match action {
        LedgerAction::Verify { ledger_id } => {
            let stream = LedgerStream::open(root, &ledger_id, "cli_verify")?;
            match stream.verify() {
                Ok(()) => {
                    println!("ledger {ledger_id}: verified");
                    Ok(0)
                }
                Err(e) => {
                    println!("ledger {ledger_id}: VERIFICATION FAILED: {e}");
                    Ok(1)
                }
            }
        }
        LedgerAction::Prove {
            ledger_id,
            entry_ulid,
        } => {
            let stream = LedgerStream::open(root, &ledger_id, "cli_prove")?;
            let proof = stream.prove_entry(&entry_ulid)?;
            let mmr = stream.load_mmr()?;
            let entries = stream.read_entries()?;
            let entry = entries
                .iter()
                .find(|e| e.entry_ulid == entry_ulid)
                .ok_or_else(|| ForgeError::Input(format!("entry {entry_ulid} not found")))?;
            let proof_json = serde_json::json!({
                "entry_ulid": entry_ulid,
                "entry_hash": entry.entry_hash,
                "leaf_index": proof.leaf_index,
                "proof_hashes": proof.proof_hashes,
                "mmr_peaks": mmr.peaks,
            });
            println!("{}", serde_json::to_string_pretty(&proof_json)?);
            Ok(0)
        }
        LedgerAction::Replay { case } => replay_case(root, &case),
    }
}

/// Print the persisted dispatch/settlement order for a case (spec §17.2 T13.2).
///
/// Reads `.sea-forge/cases/<case_id>/{plan.json,case-events.jsonl}` and prints
/// one row per persisted `ItemActivated` / `SettlementRecorded` event in file
/// order. Each row carries the additive ordinal persisted on the event
/// payload. Never re-executes any episode; rejects missing, duplicate, or
/// non-monotonic ordinals.
fn replay_case(root: &Path, case_id: &str) -> Result<u8, ForgeError> {
    let case_dir = root.join("cases").join(case_id);
    let events_path = case_dir.join("case-events.jsonl");
    if !events_path.is_file() {
        return Err(ForgeError::Input(format!(
            "case events not found at {}",
            events_path.display()
        )));
    }
    let events_text = fs::read_to_string(&events_path)
        .map_err(|error| ForgeError::io("read case events", error))?;
    let mut events: Vec<TraceEvent> = Vec::new();
    for (index, line) in events_text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let event: TraceEvent = serde_json::from_str(line).map_err(|error| {
            ForgeError::Serialization(format!("case-events.jsonl line {}: {error}", index + 1))
        })?;
        events.push(event);
    }

    // Defensive: the persisted plan must load and the reducer must process the
    // event sequence without panic. The projection itself is not printed.
    if let Ok(plan_bytes) = fs::read(case_dir.join("plan.json")) {
        if let Ok(plan) = serde_json::from_slice::<CasePlan>(&plan_bytes) {
            let _ = sea_forge_planner::case_engine::replay_case(&plan.items, &events);
        }
    }

    validate_ordinals(&events)?;

    for event in &events {
        let item_id = event.plan_item_id.as_deref().unwrap_or("");
        match event.kind {
            TraceKind::ItemActivated => {
                let ordinal = ordinal_of(event, "dispatch_ordinal")?;
                let run_id = event
                    .payload
                    .get("run_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("case");
                println!("ordinal={ordinal} phase=dispatch item_id={item_id} run_id={run_id}");
            }
            TraceKind::SettlementRecorded => {
                let ordinal = ordinal_of(event, "settlement_ordinal")?;
                let run_id = event
                    .payload
                    .get("run_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("case");
                println!("ordinal={ordinal} phase=settle item_id={item_id} run_id={run_id}");
            }
            _ => {}
        }
    }
    Ok(0)
}

fn ordinal_of(event: &TraceEvent, key: &str) -> Result<u64, ForgeError> {
    event
        .payload
        .get(key)
        .and_then(|value| value.as_u64())
        .ok_or_else(|| {
            ForgeError::Input(format!(
                "{key} missing on {:?} event for item {}",
                event.kind,
                event.plan_item_id.as_deref().unwrap_or("")
            ))
        })
}

fn validate_ordinals(events: &[TraceEvent]) -> Result<(), ForgeError> {
    let mut last_dispatch: Option<u64> = None;
    let mut last_settlement: Option<u64> = None;
    for event in events {
        match event.kind {
            TraceKind::ItemActivated => {
                let ordinal = ordinal_of(event, "dispatch_ordinal")?;
                if last_dispatch.is_some_and(|prev| ordinal <= prev) {
                    return Err(ForgeError::Input(format!(
                        "non-monotonic or duplicate dispatch_ordinal: {ordinal} after {}",
                        last_dispatch.unwrap()
                    )));
                }
                last_dispatch = Some(ordinal);
            }
            TraceKind::SettlementRecorded => {
                let ordinal = ordinal_of(event, "settlement_ordinal")?;
                if last_settlement.is_some_and(|prev| ordinal <= prev) {
                    return Err(ForgeError::Input(format!(
                        "non-monotonic or duplicate settlement_ordinal: {ordinal} after {}",
                        last_settlement.unwrap()
                    )));
                }
                last_settlement = Some(ordinal);
            }
            _ => {}
        }
    }
    Ok(())
}
