#![forbid(unsafe_code)]

use chrono::Utc;
use sea_forge_core::{errors::ForgeError, ids, types::*, RECORD_VERSION};
use sea_forge_ledger::LedgerStream;
use sea_forge_planner::case_engine::{next_case_actions, CaseAction};
use serde::Serialize;
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Synchronous case lifecycle operations shared by the CLI and server.
pub struct CaseRunner;

impl CaseRunner {
    pub fn initialize_case(case_dir: &Path) -> Result<(PathBuf, PathBuf), ForgeError> {
        let runs_dir = case_dir.join("runs");
        fs::create_dir_all(&runs_dir).map_err(|error| ForgeError::io("create case runs", error))?;
        Ok((runs_dir, case_dir.join("case-events.jsonl")))
    }

    pub fn next_ready_actions(items: &[PlanItem], events: &[TraceEvent]) -> Vec<CaseAction> {
        next_case_actions(items, events)
    }

    pub fn append_event(
        path: &Path,
        stream: &LedgerStream,
        events: &mut Vec<TraceEvent>,
        kind: TraceKind,
        item_id: Option<&str>,
        mut payload: serde_json::Value,
    ) -> Result<(), ForgeError> {
        // Additive persisted ordinals (spec §17.2 T13.2): each dispatch/
        // activation and settlement gets a 1-based monotonic ordinal derived
        // from prior persisted events. Additive only — old readers ignore it.
        // ponytail: O(n) scan per append; switch to a runner-held counter if
        // a case ever accumulates thousands of dispatch/settlement events.
        match kind {
            TraceKind::ItemActivated => {
                let next = Self::next_ordinal(events, TraceKind::ItemActivated, "dispatch_ordinal");
                payload["dispatch_ordinal"] = serde_json::json!(next);
            }
            TraceKind::SettlementRecorded => {
                let next =
                    Self::next_ordinal(events, TraceKind::SettlementRecorded, "settlement_ordinal");
                payload["settlement_ordinal"] = serde_json::json!(next);
            }
            _ => {}
        }
        Self::commit_event(
            path,
            stream,
            events,
            TraceEvent {
                version: RECORD_VERSION.into(),
                event_id: ids::seq_id("cev", 6, events.len() + 1),
                run_id: "case".into(),
                plan_item_id: item_id.map(str::to_owned),
                kind,
                actor_id: "case_engine".into(),
                timestamp: Utc::now().to_rfc3339(),
                payload,
                cell_id: None,
            },
        )
    }

    fn next_ordinal(events: &[TraceEvent], kind: TraceKind, key: &str) -> u64 {
        events
            .iter()
            .filter(|event| event.kind == kind)
            .filter_map(|event| event.payload.get(key).and_then(|value| value.as_u64()))
            .max()
            .unwrap_or(0)
            + 1
    }

    pub fn commit_event(
        path: &Path,
        stream: &LedgerStream,
        events: &mut Vec<TraceEvent>,
        event: TraceEvent,
    ) -> Result<(), ForgeError> {
        stream.commit_typed("case_event", vec![], &event, vec![])?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|error| ForgeError::io("open case events", error))?;
        serde_json::to_writer(&mut file, &event)?;
        file.write_all(b"\n")
            .and_then(|_| file.flush())
            .map_err(|error| ForgeError::io("append case event", error))?;
        events.push(event);
        Ok(())
    }

    pub fn run_sandboxed_episode<T>(
        execute: impl FnOnce() -> Result<T, ForgeError>,
    ) -> Result<T, ForgeError> {
        execute()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn apply_episode_completion(
        case: &mut Case,
        run_id: &str,
        item_id: &str,
        instance: u32,
        settlement: &SettlementEvent,
        case_events: &Path,
        stream: &LedgerStream,
        events: &mut Vec<TraceEvent>,
    ) -> Result<bool, ForgeError> {
        if !case.run_ids.iter().any(|existing| existing == run_id) {
            case.run_ids.push(run_id.into());
        }
        let accepted = settlement.status == SettlementStatus::Accepted;
        Self::append_event(
            case_events,
            stream,
            events,
            if accepted {
                TraceKind::ItemCompleted
            } else {
                TraceKind::ItemFailed
            },
            Some(item_id),
            json!({"instance": instance, "settlement": settlement.status}),
        )?;
        if accepted {
            Self::append_event(
                case_events,
                stream,
                events,
                TraceKind::MilestoneAchieved,
                Some(item_id),
                json!({"instance": instance}),
            )?;
        }
        Ok(accepted)
    }
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), ForgeError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| ForgeError::io("create JSON parent", error))?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .map_err(|error| ForgeError::io("write JSON", error))
}
