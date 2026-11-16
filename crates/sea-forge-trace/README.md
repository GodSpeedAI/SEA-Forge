# sea-forge-trace

## Overview

SEA Forge trace event recording. Owns two trace surfaces: the per-run JSONL
trace journal (`JsonlTraceRecorder`) and the external event-sink contract that
maps kernel `TraceEvent`s onto the `sea.{domain}.{action}.{qualifier}` subject
taxonomy for the NATS bridge, with JSONL as the only in-process implementation
today (`src/lib.rs` module docs).

## Interface

- Crate contract lives in [./src/README.md](./src/README.md) (single module
  `src/lib.rs`).
- Key entry points, cited from source:
  - `JsonlTraceRecorder` — run journal writer (`src/lib.rs:JsonlTraceRecorder`).
  - `append_internal_error(path, run_id, actor_id, error_class)` —
    `src/lib.rs:append_internal_error`.
  - Bridge contract: `SinkEvent`, `EventSink`, `JsonlEventSink`,
    `trace_to_sink`, `subject_for` — `src/lib.rs:EventSink`,
    `src/lib.rs:JsonlEventSink`.

## Invariants

- Internal-error appends stream the journal instead of slurping it — memory
  stays bounded by a single line on the error path
  (`src/lib.rs:append_internal_error`).
- `EventSink` is the only extension seam for external buses; JSONL is the sole
  implementation today (§2.3 E6, `src/lib.rs:EventSink`).

## Side Effects

- Appends JSONL records to run trace journals and to
  `<root>/.sea-forge/events.jsonl` (`src/lib.rs:JsonlTraceRecorder`,
  `src/lib.rs:JsonlEventSink::create`).

## Verification

- Run: `cargo test -p sea-forge-trace` (repo-wide: `just test`)
