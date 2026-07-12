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
        };
        serde_json::to_writer(&mut self.writer, &event)?;
        self.writer
            .write_all(b"\n")
            .and_then(|_| self.writer.flush())
            .map_err(|e| ForgeError::io("flush trace event", e))?;
        Ok(event_id)
    }
}
