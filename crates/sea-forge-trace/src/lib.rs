use chrono::Utc;
use sea_forge_core::{
    errors::ForgeError,
    ids::seq_id,
    types::{TraceEvent, TraceKind},
    RECORD_VERSION,
};
use serde_json::Value;
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
};

pub struct JsonlTraceRecorder {
    writer: BufWriter<File>,
    run_id: String,
    actor_id: String,
    sequence: usize,
}

pub fn append_internal_error(path: &Path, run_id: &str, actor_id: &str, error_class: &str) {
    let Ok(bytes) = std::fs::read(path) else {
        return;
    };
    if !bytes.ends_with(b"\n") {
        return;
    }
    let mut sequence = 0;
    for line in bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
    {
        if serde_json::from_slice::<TraceEvent>(line).is_err() {
            return;
        }
        sequence += 1;
    }
    let event = TraceEvent {
        version: RECORD_VERSION.into(),
        event_id: seq_id("tev", 4, sequence + 1),
        run_id: run_id.into(),
        plan_item_id: None,
        kind: TraceKind::InternalError,
        actor_id: actor_id.into(),
        timestamp: Utc::now().to_rfc3339(),
        payload: serde_json::json!({"error_class":error_class}),
        cell_id: None,
    };
    let Ok(mut encoded) = serde_json::to_vec(&event) else {
        return;
    };
    encoded.push(b'\n');
    if let Ok(mut file) = OpenOptions::new().append(true).open(path) {
        let _ = file.write_all(&encoded).and_then(|_| file.flush());
    }
}
impl JsonlTraceRecorder {
    pub fn create(path: &Path, run_id: &str, actor_id: &str) -> Result<Self, ForgeError> {
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .map_err(|e| ForgeError::io(format!("create {}", path.display()), e))?;
        Ok(Self {
            writer: BufWriter::new(file),
            run_id: run_id.into(),
            actor_id: actor_id.into(),
            sequence: 0,
        })
    }
    pub fn append(
        &mut self,
        kind: TraceKind,
        plan_item_id: Option<String>,
        payload: Value,
    ) -> Result<String, ForgeError> {
        self.sequence += 1;
        let event_id = seq_id("tev", 4, self.sequence);
        let event = TraceEvent {
            version: RECORD_VERSION.into(),
            event_id: event_id.clone(),
            run_id: self.run_id.clone(),
            plan_item_id,
            kind,
            actor_id: self.actor_id.clone(),
            timestamp: Utc::now().to_rfc3339(),
            payload,
            cell_id: None,
        };
        serde_json::to_writer(&mut self.writer, &event)?;
        self.writer
            .write_all(b"\n")
            .and_then(|_| self.writer.flush())
            .map_err(|e| ForgeError::io("flush trace event", e))?;
        Ok(event_id)
    }
}

// ── EventSink: external bridge contract (spec-full §2.3 E6, §7.4) ──
//
// The agent-memory-ledger NATS bridge consumes events with the contract fields
// below; the JSONL implementation here is the only in-process sink. A NATS
// adapter would live behind this trait as a plugin — never in the kernel.

use serde::Serialize;

/// Event payload shaped for the external NATS bridge. The bridge consumes
/// `sea.{domain}.{action}.{qualifier}` subjects; field names match the
/// SEA event contract (see ecosystem map: agent-memory-ledger).
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct SinkEvent {
    pub event_id: String,
    pub trace_id: String,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub idempotency_key: String,
    pub source_agent: String,
    pub occurred_at: String,
    pub schema_version: String,
    /// `sea.{domain}.{action}.{qualifier}` subject for the bridge.
    pub subject: String,
    pub payload: Value,
}

/// Sink that accepts `SinkEvent`s. Implementations: in-process JSONL only.
/// NATS / other buses are later plugins behind this trait (§2.3 E6).
pub trait EventSink {
    fn emit(&mut self, event: &SinkEvent) -> Result<(), ForgeError>;
}

/// Map a kernel `TraceEvent` into the external bridge contract.
pub fn trace_to_sink(event: &sea_forge_core::types::TraceEvent, _cell_id: &str) -> SinkEvent {
    SinkEvent {
        event_id: event.event_id.clone(),
        trace_id: event.run_id.clone(),
        correlation_id: event.plan_item_id.clone(),
        causation_id: None,
        idempotency_key: event.event_id.clone(),
        source_agent: event.actor_id.clone(),
        occurred_at: event.timestamp.clone(),
        schema_version: event.version.clone(),
        subject: subject_for(event.kind.clone()),
        payload: event.payload.clone(),
    }
}

/// Map a `TraceKind` onto the bridge's `sea.{domain}.{action}.{qualifier}`
/// subject taxonomy. Three-segment dotted form per ecosystem map.
pub fn subject_for(kind: TraceKind) -> String {
    use TraceKind::*;
    match kind {
        // Case lifecycle.
        CaseCreated | CaseClosed | CaseReopened | CaseTerminated => {
            let action = match kind {
                CaseCreated => "created",
                CaseClosed => "closed",
                CaseReopened => "reopened",
                CaseTerminated => "terminated",
                _ => unreachable!(),
            };
            format!("sea.case.lifecycle.{action}")
        }
        // Run lifecycle.
        RunStarted | RunFinished | RunHalted => {
            let action = match kind {
                RunStarted => "started",
                RunFinished => "finished",
                RunHalted => "halted",
                _ => unreachable!(),
            };
            format!("sea.run.lifecycle.{action}")
        }
        // Plan.
        PlanCreated | PlanMutated => {
            let action = match kind {
                PlanCreated => "created",
                PlanMutated => "mutated",
                _ => unreachable!(),
            };
            format!("sea.plan.lifecycle.{action}")
        }
        // Authority decisions.
        AuthorityEvaluated => "sea.governance.decision.evaluated".into(),
        // Workspace / sandbox.
        WorkspaceCreated => "sea.run.workspace.created".into(),
        // Commands.
        CommandStarted | CommandFinished => {
            let action = match kind {
                CommandStarted => "started",
                CommandFinished => "finished",
                _ => unreachable!(),
            };
            format!("sea.command.execution.{action}")
        }
        // Artifacts.
        ArtifactCaptured => "sea.artifact.capture.recorded".into(),
        // Settlement.
        SettlementRecorded => "sea.settlement.lifecycle.recorded".into(),
        // Milestones.
        MilestoneAchieved => "sea.case.milestone.achieved".into(),
        // Errors.
        InternalError => "sea.run.error.internal".into(),
        // Case-file items.
        CaseFileItemAdded => "sea.case.file.added".into(),
        // Item lifecycle.
        ItemEnabled | ItemActivated | ItemCompleted | ItemFailed | ItemTerminated => {
            let action = match kind {
                ItemEnabled => "enabled",
                ItemActivated => "activated",
                ItemCompleted => "completed",
                ItemFailed => "failed",
                ItemTerminated => "terminated",
                _ => unreachable!(),
            };
            format!("sea.case.item.{action}")
        }
        // Human-task.
        HumanTaskCompleted => "sea.case.human_task.completed".into(),
    }
}

/// JSONL implementation of `EventSink` — appends to `<root>/.sea-forge/events.jsonl`.
pub struct JsonlEventSink {
    writer: BufWriter<File>,
}

impl JsonlEventSink {
    pub fn create(path: &Path) -> Result<Self, ForgeError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ForgeError::io("create events.jsonl parent", e))?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| ForgeError::io(format!("open {}", path.display()), e))?;
        Ok(Self {
            writer: BufWriter::new(file),
        })
    }
}

impl EventSink for JsonlEventSink {
    fn emit(&mut self, event: &SinkEvent) -> Result<(), ForgeError> {
        serde_json::to_writer(&mut self.writer, event)?;
        self.writer
            .write_all(b"\n")
            .and_then(|_| self.writer.flush())
            .map_err(|e| ForgeError::io("flush sink event", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod sink_tests {
    use super::*;
    use sea_forge_core::types::TraceKind;
    use sea_forge_core::RECORD_VERSION;

    #[test]
    fn jsonl_event_sink_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.jsonl");
        let mut sink = JsonlEventSink::create(&path).unwrap();
        let event = SinkEvent {
            event_id: "tev_0001".into(),
            trace_id: "run_test".into(),
            correlation_id: Some("pi_1".into()),
            causation_id: None,
            idempotency_key: "tev_0001".into(),
            source_agent: "operator".into(),
            occurred_at: "2026-07-15T00:00:00Z".into(),
            schema_version: RECORD_VERSION.into(),
            subject: "sea.run.lifecycle.started".into(),
            payload: serde_json::json!({"k": "v"}),
        };
        sink.emit(&event).unwrap();
        // Force-drop to flush.
        drop(sink);
        let lines: Vec<serde_json::Value> = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0]["event_id"], "tev_0001");
        assert_eq!(lines[0]["subject"], "sea.run.lifecycle.started");
        // Subject is 4-segment sea.*.*.*.
        assert!(lines[0]["subject"].as_str().unwrap().matches('.').count() == 3);
    }

    #[test]
    fn subject_for_all_kinds_is_sea_dotted() {
        for kind in [
            TraceKind::CaseCreated,
            TraceKind::RunStarted,
            TraceKind::PlanCreated,
            TraceKind::AuthorityEvaluated,
            TraceKind::CommandStarted,
            TraceKind::ArtifactCaptured,
            TraceKind::SettlementRecorded,
            TraceKind::InternalError,
            TraceKind::HumanTaskCompleted,
            TraceKind::MilestoneAchieved,
            TraceKind::ItemCompleted,
            TraceKind::CaseFileItemAdded,
        ] {
            let s = subject_for(kind.clone());
            assert!(s.starts_with("sea."), "{kind:?} -> {s}");
            assert!(s.matches('.').count() == 3, "subject must be 4 parts: {s}");
        }
    }

    #[test]
    fn trace_to_sink_preserves_id_and_actor() {
        let event = sea_forge_core::types::TraceEvent {
            version: RECORD_VERSION.into(),
            event_id: "tev_0009".into(),
            run_id: "run_abc".into(),
            plan_item_id: Some("pi_2".into()),
            kind: TraceKind::RunFinished,
            actor_id: "agent_x".into(),
            timestamp: "2026-07-15T01:02:03Z".into(),
            payload: serde_json::json!({"status": "accepted"}),
            cell_id: None,
        };
        let sink = trace_to_sink(&event, "cell_deadbeef");
        assert_eq!(sink.event_id, "tev_0009");
        assert_eq!(sink.trace_id, "run_abc");
        assert_eq!(sink.correlation_id.as_deref(), Some("pi_2"));
        assert_eq!(sink.source_agent, "agent_x");
        assert_eq!(sink.subject, "sea.run.lifecycle.finished");
    }
}
